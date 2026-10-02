//! Overheating indicator TOjuli per orientation, NTA 8800:2025+C1:2026
//! §5.7 (pages 113–120, formula 5.40) with the July cooling balance of 7.2.2
//! per orientation.
//!
//! Step A: oriented elements to outside air (tilt > 5°, §7.6.6.4) form
//! `A_T;or`; orientations with `A_T;or ≤ 3 m²` are not assessed. Elements to
//! an unheated or heated adjacent space (AOR/AVR) take no part: they are
//! neither in `A_T` nor in `H_C;D` of 5.40, and step B lists no component for
//! them, so their transmission is left out of the per-orientation balance.
//! Step B/2: thermal bridges with explicit ψ/χ belong to the orientation of
//! their construction parts; a bridge on parts of several orientations is
//! split equally over them. Bridges without orientation and horizontal parts
//! are distributed by `A_T;or / Σ A_T` together with ground, ventilation,
//! internal gains and thermal capacity. Ground uses the July `H_gr;an` of
//! annex D.1 in the balance and in 5.40 and `H_C;g;adj` in the time
//! constant; ventilation uses `H_C;ve` with `b_v` (chapter 11 flows when the
//! zone gives them). `a_C;red` (7.7) scales the need and the booster
//! heat-pump extraction `Q_C;HP;juli` is split by 5.41a–c. Recoverable
//! system losses are not distributed over the orientations yet (step B). Results are rounded up to 0,01 K; the Bbl 4.149b
//! limit is 1,20. A zone with an active cooling system of demonstrated
//! capacity (§5.7.1) has TOjuli = 0.

use crate::climate::{Orientation, MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use crate::direct_transmission::{assess_direct_transmission, EnvelopeSide};
use crate::monthly_demand::{
    assess_monthly_demand, cooling_reduction_factor, cooling_utilization, internal_gains_kwh,
    opaque_solar_kwh, ventilation_conductance, window_solar_kwh, with_resolved_ventilation,
    MonthlyDemandInput, Transmission, A_0, TAU_0_H,
};
use crate::solar_shading::Balance;
use serde::{Deserialize, Serialize};

pub const JULY: u8 = 7;
/// Bbl 4.149b paragraph 1.
pub const TOJULI_LIMIT_K: f64 = 1.20;
/// §5.7.2 step A: orientations with at most this area are skipped.
pub const MIN_ORIENTATION_AREA_M2: f64 = 3.0;
/// §7.6.6.4: surfaces up to this tilt count as horizontal.
pub const HORIZONTAL_TILT_MAX_DEG: f64 = 5.0;
/// §5.7.1 method 3: window area below this share of `A_g;tot`.
pub const SMALL_WINDOW_AREA_RATIO: f64 = 0.2;

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

/// §5.7.1: systems that count as active cooling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveCoolingSystem {
    /// Table 10.29 compression chillers.
    CompressionTable10_29,
    /// Table 10.30 absorption chillers.
    AbsorptionTable10_30,
    /// Table 10.34 free cooling, except dew-point cooling on humidified ETA.
    FreeCoolingTable10_34,
    /// Dew-point cooling on ventilation air with humidified exhaust: needs
    /// capacity method 1 or 2 (note 2).
    DewPointCoolingHumidifiedExhaust,
    HeatPumpWithCoolingEmitter,
    ExternalColdWithCoolingEmitter,
    SplitUnitsInEveryHabitableRoom,
    /// Utility buildings: another active cooling system.
    OtherUtility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SolarLimitationCriterion {
    /// `A_w < 0,2 · A_g;tot`; checked against the window inventory.
    SmallWindowArea,
    /// More than 95 % of the glazing (45°–315° and horizontal) has table 7.4a/b
    /// shading, `g_gl ≤ 0,4`, or `F_sh;obst;juli < 0,67` (declared).
    ShadedGlazing,
}

/// §5.7.1: one of three methods shows sufficient capacity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum CoolingCapacityEvidence {
    DynamicCoolingLoad {
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    AnnexAa {
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    SolarLimitation {
        criterion: SolarLimitationCriterion,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

impl CoolingCapacityEvidence {
    fn source_reference(&self) -> &str {
        match self {
            Self::DynamicCoolingLoad { source_reference }
            | Self::AnnexAa { source_reference }
            | Self::SolarLimitation {
                source_reference, ..
            } => source_reference,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActiveCoolingEvidence {
    pub system: ActiveCoolingSystem,
    pub capacity: CoolingCapacityEvidence,
    pub source_reference: String,
}

/// Inputs of 5.40 that come from outside chapter 7.
#[derive(Debug, Clone, Copy)]
pub struct TojuliOptions<'a> {
    pub residential: bool,
    pub active_cooling: Option<&'a ActiveCoolingEvidence>,
    /// `Q_C;HP;juli;zi` (10.3.2) in kWh; 0 without a booster heat pump.
    pub booster_heat_pump_july_kwh: f64,
}

impl Default for TojuliOptions<'_> {
    fn default() -> Self {
        Self {
            residential: true,
            active_cooling: None,
            booster_heat_pump_july_kwh: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrientationResult {
    pub orientation: Orientation,
    pub area_m2: f64,
    pub share: f64,
    pub assessed: bool,
    pub conductance_w_per_k: f64,
    pub cooling_need_july_kwh: f64,
    pub booster_heat_pump_july_kwh: f64,
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

/// A utility zone without a use function cannot get `a_C;red`.
pub fn use_function_required(zone_id: &str) -> TojuliAssessment {
    invalid(
        zone_id,
        false,
        vec![issue("tojuli_use_function_required", "labelFunction")],
    )
}

/// Checks the §5.7.1 evidence against the zone; empty means accepted.
pub fn validate_active_cooling(
    evidence: &ActiveCoolingEvidence,
    input: &MonthlyDemandInput,
    residential: bool,
    path: &str,
) -> Vec<TojuliIssue> {
    let mut issues = Vec::new();
    if evidence.source_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            &format!("{path}.sourceReference"),
        ));
    }
    if evidence.capacity.source_reference().trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            &format!("{path}.capacity.sourceReference"),
        ));
    }
    if residential && evidence.system == ActiveCoolingSystem::OtherUtility {
        issues.push(issue(
            "active_cooling_system_not_listed",
            &format!("{path}.system"),
        ));
    }
    if let CoolingCapacityEvidence::SolarLimitation { criterion, .. } = &evidence.capacity {
        if evidence.system == ActiveCoolingSystem::DewPointCoolingHumidifiedExhaust {
            // Note 2 of §5.7.1: only methods 1 or 2.
            issues.push(issue(
                "dew_point_cooling_requires_capacity_calculation",
                &format!("{path}.capacity.method"),
            ));
        }
        if *criterion == SolarLimitationCriterion::SmallWindowArea {
            if !input.window_inventory_complete {
                issues.push(issue(
                    "window_inventory_incomplete",
                    &format!("{path}.capacity.criterion"),
                ));
            }
            let window_area: f64 = input.windows.iter().map(|window| window.area_m2).sum();
            if window_area >= SMALL_WINDOW_AREA_RATIO * input.usable_floor_area_m2 {
                issues.push(issue(
                    "solar_limitation_not_met",
                    &format!("{path}.capacity.criterion"),
                ));
            }
        }
    }
    issues
}

fn side_index(side: EnvelopeSide) -> Option<usize> {
    Some(match side {
        EnvelopeSide::North => 0,
        EnvelopeSide::NorthEast => 1,
        EnvelopeSide::East => 2,
        EnvelopeSide::SouthEast => 3,
        EnvelopeSide::South => 4,
        EnvelopeSide::SouthWest => 5,
        EnvelopeSide::West => 6,
        EnvelopeSide::NorthWest => 7,
        EnvelopeSide::Horizontal => return None,
    })
}

/// Splits a bridge conductance equally over its distinct construction
/// parts; horizontal parts and unspecified bridges go to the pool.
fn assign_bridge(
    conductance: f64,
    sides: &[EnvelopeSide],
    oriented: &mut [f64; 8],
    pool: &mut f64,
) {
    let mut distinct: Vec<EnvelopeSide> = Vec::new();
    for side in sides {
        if !distinct.contains(side) {
            distinct.push(*side);
        }
    }
    if distinct.is_empty() {
        *pool += conductance;
        return;
    }
    let part = conductance / distinct.len() as f64;
    for side in distinct {
        match side_index(side) {
            Some(index) => oriented[index] += part,
            None => *pool += part,
        }
    }
}

pub fn assess_tojuli(input: &MonthlyDemandInput, options: TojuliOptions<'_>) -> TojuliAssessment {
    let has_active = options.active_cooling.is_some();
    let demand = assess_monthly_demand(input);
    if demand.status != "calculated_unverified" {
        return invalid(
            &input.zone_id,
            has_active,
            vec![issue("tojuli_demand_invalid", "demand")],
        );
    }
    let resolved = with_resolved_ventilation(input, &demand);
    let input = &resolved;
    if let Some(evidence) = options.active_cooling {
        let issues = validate_active_cooling(evidence, input, options.residential, "activeCooling");
        if !issues.is_empty() {
            return invalid(&input.zone_id, has_active, issues);
        }
        // §5.7.2: an active cooling system allows TOjuli = 0 for all orientations.
        return TojuliAssessment {
            zone_id: input.zone_id.clone(),
            status: "calculated_unverified",
            active_cooling: true,
            orientations: Vec::new(),
            max_tojuli_k: Some(0.0),
            meets_bbl_limit: Some(true),
            issues: Vec::new(),
        };
    }
    let Transmission::Components(components) = &input.transmission else {
        return invalid(
            &input.zone_id,
            has_active,
            vec![issue("tojuli_components_required", "transmission")],
        );
    };
    let summary = demand
        .transmission
        .as_ref()
        .expect("valid demand has a summary");
    let direct = assess_direct_transmission(&components.direct);
    let element_conductance = direct.element_conductance_w_per_k.unwrap_or(0.0);

    // Step A and steps 1–3: oriented and horizontal buckets.
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
            has_active,
            vec![issue(
                "tojuli_envelope_inconsistent",
                "windows/opaqueElements",
            )],
        );
    }
    // Step 2: explicit thermal bridges to their orientations.
    for bridge in &components.direct.linear_bridges {
        assign_bridge(
            bridge.length_m * bridge.psi_w_per_mk,
            &bridge.orientations,
            &mut conductance,
            &mut horizontal_conductance,
        );
    }
    for bridge in &components.direct.point_bridges {
        assign_bridge(
            bridge.chi_w_per_k,
            &bridge.orientations,
            &mut conductance,
            &mut horizontal_conductance,
        );
    }
    let total_area: f64 = area.iter().sum();
    if total_area <= 0.0 {
        return invalid(
            &input.zone_id,
            has_active,
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
    let internal_july = internal_gains_kwh(input, index);
    let capacity = demand
        .specific_heat_capacity_kj_per_m2k
        .expect("valid demand")
        * 1000.0
        * floor_area;

    let mut results = Vec::with_capacity(8);
    for (index, orientation) in ORIENTATIONS.iter().enumerate() {
        let share = area[index] / total_area;
        // Steps 4–5: H_C;D;juli;or.
        let direct_or = conductance[index] + share * horizontal_conductance;
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
        results.push(OrientationResult {
            orientation: *orientation,
            area_m2: area[index],
            share,
            assessed: area[index] > MIN_ORIENTATION_AREA_M2,
            conductance_w_per_k: total_or,
            cooling_need_july_kwh: need,
            booster_heat_pump_july_kwh: 0.0,
            tojuli_k: None,
        });
    }
    // 5.41a–c: booster heat-pump extraction split by the cooling need.
    let total_need: f64 = results.iter().map(|item| item.cooling_need_july_kwh).sum();
    for item in &mut results {
        if total_need > 0.0 {
            item.booster_heat_pump_july_kwh =
                options.booster_heat_pump_july_kwh * item.cooling_need_july_kwh / total_need;
        }
        if item.assessed && item.conductance_w_per_k > 0.0 {
            // 5.40
            item.tojuli_k = Some(round_up(
                (item.cooling_need_july_kwh - item.booster_heat_pump_july_kwh) * 1000.0
                    / (item.conductance_w_per_k * hours),
            ));
        }
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
        active_cooling: false,
        orientations: results,
        max_tojuli_k: max,
        meets_bbl_limit: max.map(|value| value <= TOJULI_LIMIT_K),
        issues: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direct_transmission::LinearBridge;
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

    fn south(result: &TojuliAssessment) -> &OrientationResult {
        result
            .orientations
            .iter()
            .find(|item| item.orientation == Orientation::South)
            .unwrap()
    }

    fn evidence(capacity: CoolingCapacityEvidence) -> ActiveCoolingEvidence {
        ActiveCoolingEvidence {
            system: ActiveCoolingSystem::CompressionTable10_29,
            capacity,
            source_reference: "installation design".into(),
        }
    }

    #[test]
    fn per_orientation_shares_and_south_dominates() {
        let input = demand();
        let result = assess_tojuli(&input, TojuliOptions::default());
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let shares: f64 = result.orientations.iter().map(|item| item.share).sum();
        assert!((shares - 1.0).abs() < 1e-12);
        let north = result
            .orientations
            .iter()
            .find(|item| item.orientation == Orientation::North)
            .unwrap();
        // South wall 30 m² plus the 45° south roof 52 m².
        assert!((south(&result).area_m2 - 82.0).abs() < 1e-9);
        assert!(south(&result).cooling_need_july_kwh >= north.cooling_need_july_kwh);
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
        let result = assess_tojuli(&input, TojuliOptions::default());
        let south = south(&result);
        let expected =
            round_up(south.cooling_need_july_kwh * 1000.0 / (south.conductance_w_per_k * 744.0));
        assert_eq!(south.tojuli_k, Some(expected));
    }

    #[test]
    fn oriented_bridge_moves_from_pool_to_its_orientation() {
        let mut input = demand();
        let Transmission::Components(components) = &mut input.transmission else {
            panic!("components expected");
        };
        components.direct.linear_bridges.push(LinearBridge {
            id: "corner".into(),
            length_m: 10.0,
            psi_w_per_mk: 0.2,
            source_reference: "detail".into(),
            orientations: vec![EnvelopeSide::South, EnvelopeSide::West],
        });
        let base = assess_tojuli(&demand(), TojuliOptions::default());
        let result = assess_tojuli(&input, TojuliOptions::default());
        // 2 W/K split equally: 1 W/K to south and 1 W/K to west.
        let delta = south(&result).conductance_w_per_k - south(&base).conductance_w_per_k;
        assert!((delta - 1.0).abs() < 1e-9, "{delta}");
        let mut distinct = [0.0; 8];
        let mut pool = 0.0;
        assign_bridge(
            3.0,
            &[
                EnvelopeSide::North,
                EnvelopeSide::Horizontal,
                EnvelopeSide::North,
            ],
            &mut distinct,
            &mut pool,
        );
        assert!((distinct[0] - 1.5).abs() < 1e-12 && (pool - 1.5).abs() < 1e-12);
    }

    #[test]
    fn booster_heat_pump_and_cooling_reduction_follow_5_41_and_7_7() {
        let input = demand();
        let base = assess_tojuli(&input, TojuliOptions::default());
        let total: f64 = base
            .orientations
            .iter()
            .map(|item| item.cooling_need_july_kwh)
            .sum();
        assert!(total > 0.0);
        let options = TojuliOptions {
            booster_heat_pump_july_kwh: 0.5 * total,
            ..TojuliOptions::default()
        };
        let result = assess_tojuli(&input, options);
        let item = south(&result);
        let base_item = south(&base);
        assert!((item.cooling_need_july_kwh - base_item.cooling_need_july_kwh).abs() < 1e-9);
        // 5.41b uses the reduced needs, so the share is unchanged.
        let expected_hp = 0.5 * total * base_item.cooling_need_july_kwh / total;
        assert!((item.booster_heat_pump_july_kwh - expected_hp).abs() < 1e-9);
        let expected = round_up(
            (item.cooling_need_july_kwh - expected_hp) * 1000.0
                / (item.conductance_w_per_k * 744.0),
        );
        assert_eq!(item.tojuli_k, Some(expected));
    }

    #[test]
    fn active_cooling_needs_valid_capacity_evidence() {
        let input = demand();
        let dynamic = evidence(CoolingCapacityEvidence::DynamicCoolingLoad {
            source_reference: "GTO load calculation".into(),
        });
        let cooled = assess_tojuli(
            &input,
            TojuliOptions {
                active_cooling: Some(&dynamic),
                ..TojuliOptions::default()
            },
        );
        assert_eq!(cooled.max_tojuli_k, Some(0.0));
        assert_eq!(cooled.meets_bbl_limit, Some(true));

        let mut dew = evidence(CoolingCapacityEvidence::SolarLimitation {
            criterion: SolarLimitationCriterion::ShadedGlazing,
            source_reference: "shading inventory".into(),
        });
        dew.system = ActiveCoolingSystem::DewPointCoolingHumidifiedExhaust;
        let codes: Vec<_> = validate_active_cooling(&dew, &input, true, "activeCooling")
            .iter()
            .map(|item| item.code)
            .collect();
        assert_eq!(codes, ["dew_point_cooling_requires_capacity_calculation"]);

        let mut other = dynamic.clone();
        other.system = ActiveCoolingSystem::OtherUtility;
        assert!(!validate_active_cooling(&other, &input, true, "a").is_empty());
        assert!(validate_active_cooling(&other, &input, false, "a").is_empty());

        // Method 3 with A_w < 0,2·A_g is checked against the windows.
        let small = evidence(CoolingCapacityEvidence::SolarLimitation {
            criterion: SolarLimitationCriterion::SmallWindowArea,
            source_reference: "window inventory".into(),
        });
        let window_area: f64 = input.windows.iter().map(|item| item.area_m2).sum();
        let met = window_area < 0.2 * input.usable_floor_area_m2;
        assert_eq!(
            validate_active_cooling(&small, &input, true, "a").is_empty(),
            met && input.window_inventory_complete
        );
    }

    #[test]
    fn explicit_transmission_is_rejected() {
        let explicit: MonthlyDemandInput = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-monthly-demand-synthetic.json"
        ))
        .unwrap();
        let result = assess_tojuli(&explicit, TojuliOptions::default());
        assert_eq!(result.issues[0].code, "tojuli_components_required");
    }

    #[test]
    fn rounding_is_upward_to_hundredths() {
        assert_eq!(round_up(0.831), 0.84);
        assert_eq!(round_up(0.83), 0.83);
        assert_eq!(round_up(-0.2), 0.0);
    }
}
