//! Overheating indicator TOjuli per orientation, NTA 8800 §5.7 (formula 5.40)
//! with the July cooling balance of 7.2.2 per orientation.
//!
//! Oriented elements (tilt > 5°, §7.6.6.4) keep their own transmission and
//! solar gain. Horizontal elements, thermal bridges, unheated-space
//! transmission, ground, ventilation, internal gains and thermal capacity are
//! distributed pro rata by `A_T;or / Σ A_T`. Ground uses the July
//! `H_gr;an` of annex D.1 in the balance and in 5.40, and `H_C;g;adj` in the
//! time constant; ventilation uses `H_C;ve` with `b_v`; the need includes
//! `a_C;red` (7.7). Orientations with `A_T;or ≤ 3 m²` are not assessed.
//! Results are rounded up to 0,01 K; the Bbl 4.149b limit is 1,20. A zone
//! with sufficient active cooling may use TOjuli = 0 (§5.7.1).

use crate::climate::{Orientation, MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use crate::direct_transmission::assess_direct_transmission;
use crate::monthly_demand::{
    assess_monthly_demand, cooling_reduction_factor, cooling_utilization, internal_gains_kwh,
    opaque_solar_kwh, ventilation_conductance, window_solar_kwh, with_resolved_ventilation,
    MonthlyDemandInput, Transmission, A_0, TAU_0_H,
};
use crate::solar_shading::Balance;
use serde::Serialize;

pub const JULY: u8 = 7;
/// Bbl 4.149b paragraph 1.
pub const TOJULI_LIMIT_K: f64 = 1.20;
/// §5.7.2 step A: orientations with at most this area are skipped.
pub const MIN_ORIENTATION_AREA_M2: f64 = 3.0;
/// §7.6.6.4: surfaces up to this tilt count as horizontal.
pub const HORIZONTAL_TILT_MAX_DEG: f64 = 5.0;

const ORIENTATIONS: [Orientation; 8] = [
    Orientation::North,
    Orientation::NorthEast,
    Orientation::East,
    Orientation::SouthEast,
    Orientation::South,
    Orientation::SouthWest,
    Orientation::West,
    Orientation::NorthWest,
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrientationResult {
    pub orientation: Orientation,
    pub area_m2: f64,
    pub share: f64,
    pub assessed: bool,
    pub conductance_w_per_k: f64,
    pub cooling_need_july_kwh: f64,
    pub tojuli_k: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TojuliAssessment {
    pub zone_id: String,
    pub status: &'static str,
    pub active_cooling: bool,
    pub orientations: Vec<OrientationResult>,
    pub max_tojuli_k: Option<f64>,
    pub meets_bbl_limit: Option<bool>,
    pub issues: Vec<TojuliIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TojuliIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: &str) -> TojuliIssue {
    TojuliIssue {
        code,
        path: path.into(),
    }
}

fn round_up(value: f64) -> f64 {
    (value * 100.0 - 1e-9).ceil().max(0.0) / 100.0
}

fn invalid(zone_id: &str, active_cooling: bool, issues: Vec<TojuliIssue>) -> TojuliAssessment {
    TojuliAssessment {
        zone_id: zone_id.to_owned(),
        status: "invalid",
        active_cooling,
        orientations: Vec::new(),
        max_tojuli_k: None,
        meets_bbl_limit: None,
        issues,
    }
}

pub fn assess_tojuli(input: &MonthlyDemandInput, active_cooling: bool) -> TojuliAssessment {
    let demand = assess_monthly_demand(input);
    if demand.status != "calculated_unverified" {
        return invalid(
            &input.zone_id,
            active_cooling,
            vec![issue("tojuli_demand_invalid", "demand")],
        );
    }
    let resolved = with_resolved_ventilation(input, &demand);
    let input = &resolved;
    let Transmission::Components(components) = &input.transmission else {
        return invalid(
            &input.zone_id,
            active_cooling,
            vec![issue("tojuli_components_required", "transmission")],
        );
    };
    if active_cooling {
        // §5.7.1: sufficient active cooling allows TOjuli = 0 for all orientations.
        return TojuliAssessment {
            zone_id: input.zone_id.clone(),
            status: "calculated_unverified",
            active_cooling,
            orientations: Vec::new(),
            max_tojuli_k: Some(0.0),
            meets_bbl_limit: Some(true),
            issues: Vec::new(),
        };
    }
    let summary = demand
        .transmission
        .as_ref()
        .expect("valid demand has a summary");
    let direct = assess_direct_transmission(&components.direct);
    let element_conductance = direct.element_conductance_w_per_k.unwrap_or(0.0);
    let bridges = direct.linear_bridge_conductance_w_per_k.unwrap_or(0.0)
        + direct.point_bridge_conductance_w_per_k.unwrap_or(0.0);

    // Classify envelope elements into oriented and horizontal buckets.
    let mut area = [0.0; 8];
    let mut conductance = [0.0; 8];
    let mut solar = [0.0; 8];
    let mut horizontal_conductance = 0.0;
    let mut horizontal_solar = 0.0;
    let mut listed_conductance = 0.0;
    let mut bucket = |orientation: Orientation, tilt: f64, a: f64, h: f64, gain: f64| {
        listed_conductance += h;
        if tilt <= HORIZONTAL_TILT_MAX_DEG {
            horizontal_conductance += h;
            horizontal_solar += gain;
        } else {
            let index = ORIENTATIONS
                .iter()
                .position(|item| *item == orientation)
                .expect("known orientation");
            area[index] += a;
            conductance[index] += h;
            solar[index] += gain;
        }
    };
    for window in &input.windows {
        bucket(
            window.orientation,
            window.tilt_deg,
            window.area_m2,
            window.area_m2 * window.u_value_w_per_m2k,
            window_solar_kwh(window, JULY, Balance::Cooling),
        );
    }
    for element in &input.opaque_elements {
        bucket(
            element.orientation,
            element.tilt_deg,
            element.area_m2,
            element.area_m2 * element.u_value_w_per_m2k,
            opaque_solar_kwh(element, JULY),
        );
    }
    if (listed_conductance - element_conductance).abs() > 1e-6 * element_conductance.max(1.0) {
        // The oriented split must cover exactly the direct element conductance.
        return invalid(
            &input.zone_id,
            active_cooling,
            vec![issue(
                "tojuli_envelope_inconsistent",
                "windows/opaqueElements",
            )],
        );
    }
    let total_area: f64 = area.iter().sum();
    if total_area <= 0.0 {
        return invalid(
            &input.zone_id,
            active_cooling,
            vec![issue(
                "tojuli_no_oriented_elements",
                "windows/opaqueElements",
            )],
        );
    }

    let index = usize::from(JULY - 1);
    let hours = MONTH_HOURS[index];
    let outdoor = OUTDOOR_TEMPERATURE_C[index];
    let setpoint = input.setpoints.cooling_c;
    let pool_conductance =
        horizontal_conductance + bridges + summary.unheated_conductance_w_per_k.unwrap_or(0.0);
    // 5.40 and step B: H_gr;an;juli in the balance and the denominator; the
    // time constant (7.58) uses the seasonal H_C;g;adj.
    let ground_conductance = summary.ground_monthly_conductance_w_per_k[index];
    let ground_adjusted = summary.ground_cooling_adjusted_w_per_k;
    let ground_july = summary.ground_kwh(JULY, setpoint);
    // 7.19 with b_v for the cooling balance.
    let ventilation_conductance = ventilation_conductance(input, JULY, Balance::Cooling);
    let ventilation_july = ventilation_conductance * (setpoint - outdoor) * hours / 1000.0;
    let cooling_reduction = cooling_reduction_factor(input.usage_function);
    let floor_area = input.usable_floor_area_m2;
    let internal_july = internal_gains_kwh(input, usize::from(JULY - 1));
    let capacity = demand
        .specific_heat_capacity_kj_per_m2k
        .expect("valid demand")
        * 1000.0
        * floor_area;

    let mut results = Vec::with_capacity(8);
    for (index, orientation) in ORIENTATIONS.iter().enumerate() {
        let share = area[index] / total_area;
        let direct_or = conductance[index] + share * pool_conductance;
        let total_or = direct_or + share * ground_conductance + share * ventilation_conductance;
        let tau_conductance = direct_or + share * ground_adjusted + share * ventilation_conductance;
        let heat_transfer = direct_or * (setpoint - outdoor) * hours / 1000.0
            + share * ground_july
            + share * ventilation_july;
        let gains = share * internal_july + solar[index] + share * horizontal_solar;
        // 7.6/7.7 with 7.52–7.54 and 7.58.
        let gamma = (heat_transfer != 0.0).then(|| gains / heat_transfer);
        let need = if gains <= 0.0
            || tau_conductance <= 0.0
            || gamma.is_some_and(|gamma| gamma > 0.0 && 1.0 / gamma > 2.0)
        {
            0.0
        } else {
            let tau = share * capacity / 3600.0 / tau_conductance;
            let a = A_0 + tau / TAU_0_H;
            let eta = gamma.map_or(1.0, |gamma| cooling_utilization(gamma, a));
            (cooling_reduction * (gains - eta * heat_transfer)).max(0.0)
        };
        let assessed = area[index] > MIN_ORIENTATION_AREA_M2;
        let tojuli =
            (assessed && total_or > 0.0).then(|| round_up(need * 1000.0 / (total_or * hours)));
        results.push(OrientationResult {
            orientation: *orientation,
            area_m2: area[index],
            share,
            assessed,
            conductance_w_per_k: total_or,
            cooling_need_july_kwh: need,
            tojuli_k: tojuli,
        });
    }
    let max = results
        .iter()
        .filter_map(|item| item.tojuli_k)
        .fold(None, |current: Option<f64>, value| {
            Some(current.map_or(value, |max| max.max(value)))
        });
    TojuliAssessment {
        zone_id: input.zone_id.clone(),
        status: "calculated_unverified",
        active_cooling,
        orientations: results,
        max_tojuli_k: max,
        meets_bbl_limit: max.map(|value| value <= TOJULI_LIMIT_K),
        issues: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_performance::assess_project_performance;
    use serde_json::Value;

    fn demand() -> MonthlyDemandInput {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let result = assess_project_performance(&project);
        result.derived_input.unwrap().space_heating.demand
    }

    #[test]
    fn per_orientation_shares_and_south_dominates() {
        let input = demand();
        let result = assess_tojuli(&input, false);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let shares: f64 = result.orientations.iter().map(|item| item.share).sum();
        assert!((shares - 1.0).abs() < 1e-12);
        let south = result
            .orientations
            .iter()
            .find(|item| item.orientation == Orientation::South)
            .unwrap();
        let north = result
            .orientations
            .iter()
            .find(|item| item.orientation == Orientation::North)
            .unwrap();
        // South wall 30 m² plus the 45° south roof 52 m².
        assert!((south.area_m2 - 82.0).abs() < 1e-9);
        assert!(south.cooling_need_july_kwh >= north.cooling_need_july_kwh);
        // Diagonal orientations have no area and are not assessed.
        assert!(result
            .orientations
            .iter()
            .filter(|item| item.area_m2 == 0.0)
            .all(|item| !item.assessed && item.tojuli_k.is_none()));
        let max = result.max_tojuli_k.unwrap();
        assert_eq!(result.meets_bbl_limit, Some(max <= 1.2));
        assert_eq!((max * 100.0).round() / 100.0, max);
    }

    #[test]
    fn formula_5_40_for_one_orientation() {
        let input = demand();
        let result = assess_tojuli(&input, false);
        let south = result
            .orientations
            .iter()
            .find(|item| item.orientation == Orientation::South)
            .unwrap();
        let expected =
            round_up(south.cooling_need_july_kwh * 1000.0 / (south.conductance_w_per_k * 744.0));
        assert_eq!(south.tojuli_k, Some(expected));
    }

    #[test]
    fn active_cooling_and_explicit_transmission() {
        let input = demand();
        let cooled = assess_tojuli(&input, true);
        assert_eq!(cooled.max_tojuli_k, Some(0.0));
        assert_eq!(cooled.meets_bbl_limit, Some(true));
        let explicit: MonthlyDemandInput = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-monthly-demand-synthetic.json"
        ))
        .unwrap();
        let result = assess_tojuli(&explicit, false);
        assert_eq!(result.issues[0].code, "tojuli_components_required");
    }

    #[test]
    fn rounding_is_upward_to_hundredths() {
        assert_eq!(round_up(0.831), 0.84);
        assert_eq!(round_up(0.83), 0.83);
        assert_eq!(round_up(-0.2), 0.0);
    }
}
