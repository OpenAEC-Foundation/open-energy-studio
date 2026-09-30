//! Monthly heating and cooling need of one calculation zone, NTA 8800 chapter 7.
//!
//! Implements the monthly balance
//! `Q_H;nd = Q_H;ht − η_H;gn · Q_H;gn` and
//! `Q_C;nd = Q_C;gn − η_C;ht · Q_C;ht` (with the 7.6 gate) for explicitly
//! supplied heat-transfer coefficients. Gains are derived here: residential
//! internal gains (7.21–7.24), window solar gains (7.32 with 7.40) and opaque
//! solar gains (7.33), both reduced by sky radiation (7.39). Effective thermal
//! capacity follows table 7.10 and 7.45; utilisation follows 7.46–7.57.
//!
//! Formula numbers and constants come from the Open Heatloss Studio norm
//! analyses (C1–C5, 2026-07-13), which transcribe NTA 8800:2025+C1:2026 with
//! page references. They still require an independent review against the
//! licensed norm text. Corrections this module does not apply are listed in
//! [`OMITTED_CORRECTIONS`] and returned with every result.

use crate::climate::{self, Orientation, CLIMATE_SOURCE, MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 7.40: time-averaged angle-of-incidence correction on `g_gl;n`.
pub const F_W: f64 = 0.90;
/// Annex C.2: external surface resistance in m²K/W.
pub const R_SE: f64 = 0.04;
/// 7.39: external long-wave radiation coefficient in W/(m²K).
pub const H_LR_E: f64 = 4.14;
/// 7.39: mean difference between outdoor air and sky temperature in K.
pub const DELTA_THETA_SKY: f64 = 11.0;
/// 7.6.6.3: solar absorption coefficient of opaque outer surfaces.
pub const ALPHA_SOL: f64 = 0.6;
/// 7.51: reference parameters of the monthly method.
pub const A_0: f64 = 1.0;
pub const TAU_0_H: f64 = 15.0;
/// 7.21: internal heat per occupant in W.
pub const INTERNAL_HEAT_PER_OCCUPANT_W: f64 = 180.0;

pub const OMITTED_CORRECTIONS: &[&str] = &[
    "7.9.2 intermittent heating reduction a_H;red",
    "7.9.4.2 residential temperature levelling (7.78)",
    "movable solar shading and separate g_gl;C",
    "tilts other than 0° and 90° (table 17.2 columns not transcribed)",
    "annex B detailed thermal capacity",
    "table 7.10 footnote c is the caller's column choice",
];

const SCOPE: &str = "nta8800_chapter_7_monthly_need_single_zone_unverified";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonthlyDemandInput {
    pub zone_id: String,
    pub usable_floor_area_m2: f64,
    pub area_source_reference: String,
    pub setpoints: Setpoints,
    pub transmission: Transmission,
    pub ventilation_flows: Vec<VentilationFlow>,
    pub thermal_mass: ThermalMass,
    pub internal_gains: InternalGains,
    pub window_inventory_complete: bool,
    pub windows: Vec<Window>,
    pub opaque_inventory_complete: bool,
    pub opaque_elements: Vec<OpaqueElement>,
}

/// Table 7.13: residential 20 °C heating and 24 °C cooling. Supplied
/// explicitly so a utility function cannot silently inherit dwelling values.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Setpoints {
    pub heating_c: f64,
    pub cooling_c: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Transmission {
    /// `H_tr` excluding ground floors: direct, via unheated spaces (b applied)
    /// and thermal bridges, in W/K.
    pub conductance_w_per_k: f64,
    pub source_reference: String,
    /// Ground route of chapter 8.3 supplied by the caller; `None` only when
    /// the zone has no ground contact.
    pub ground: Option<GroundTransfer>,
    pub ground_inventory_confirmed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundTransfer {
    /// `H_g;adj` used in the time constant (7.57), W/K.
    pub adjusted_conductance_w_per_k: f64,
    /// Monthly ground heat transfer at the heating setpoint, kWh.
    pub heating_kwh: Vec<f64>,
    /// Monthly ground heat transfer at the cooling setpoint, kWh.
    pub cooling_kwh: Vec<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VentilationFlow {
    pub id: String,
    pub source_reference: String,
    pub months: Vec<VentilationMonth>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VentilationMonth {
    pub month: u8,
    /// `H_ve;k;mi = ρ·c·q_v;k;mi` in W/K.
    pub conductance_w_per_k: f64,
    /// Supply temperature; `None` means outdoor air at `θ_e;avg;mi`.
    #[serde(default)]
    pub supply_temperature_c: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MassClass {
    Light,
    Heavy,
    VeryHeavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CeilingColumn {
    ClosedOrSuspended,
    OpenOrNone,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThermalMass {
    pub floor: MassClass,
    pub wall: MassClass,
    pub ceiling: CeilingColumn,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum InternalGains {
    /// 7.21–7.24 for the residential function.
    Residential {
        #[serde(rename = "dwellingCount")]
        dwelling_count: u32,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Utility functions: flux per m² from the applicable table, W/m².
    Declared {
        #[serde(rename = "heatFluxWPerM2")]
        heat_flux_w_per_m2: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Window {
    pub id: String,
    pub area_m2: f64,
    pub orientation: Orientation,
    pub tilt_deg: f64,
    /// Perpendicular `g_gl;n`.
    pub g_perpendicular: f64,
    pub frame_fraction: f64,
    pub u_value_w_per_m2k: f64,
    /// `F_sh;obst` for external obstructions, constant over the year.
    pub obstruction_factor: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpaqueElement {
    pub id: String,
    pub area_m2: f64,
    pub orientation: Orientation,
    pub tilt_deg: f64,
    pub u_value_w_per_m2k: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyDemandAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub climate_source: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub omitted_corrections: &'static [&'static str],
    pub specific_heat_capacity_kj_per_m2k: Option<f64>,
    pub monthly: Vec<MonthResult>,
    pub annual_heating_need_kwh: Option<f64>,
    pub annual_cooling_need_kwh: Option<f64>,
    pub issues: Vec<DemandIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthResult {
    pub month: u8,
    pub hours: f64,
    pub outdoor_temperature_c: f64,
    pub time_constant_h: f64,
    pub a: f64,
    pub internal_gains_kwh: f64,
    pub window_solar_gains_kwh: f64,
    pub opaque_solar_gains_kwh: f64,
    pub heating: BalanceTerms,
    pub cooling: BalanceTerms,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceTerms {
    pub transmission_kwh: f64,
    pub ventilation_kwh: f64,
    pub heat_transfer_kwh: f64,
    pub gains_kwh: f64,
    pub gamma: Option<f64>,
    pub utilization: f64,
    pub need_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DemandIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> DemandIssue {
    DemandIssue {
        code,
        path: path.into(),
    }
}

/// Table 7.10, `D_m;int;eff` in kJ/(m²K).
pub fn specific_heat_capacity(floor: MassClass, wall: MassClass, ceiling: CeilingColumn) -> f64 {
    use MassClass::{Heavy, Light, VeryHeavy};
    let (closed, open) = match (floor, wall) {
        (Light, Light) => (55.0, 80.0),
        (Light, Heavy) | (Heavy | VeryHeavy, Light) => (110.0, 180.0),
        (Heavy, Heavy) | (Light, VeryHeavy) => (180.0, 360.0),
        (Heavy | VeryHeavy, VeryHeavy) | (VeryHeavy, Heavy) => (250.0, 450.0),
    };
    match ceiling {
        CeilingColumn::ClosedOrSuspended => closed,
        CeilingColumn::OpenOrNone => open,
    }
}

/// 7.22–7.24: occupants per dwelling from mean usable area per dwelling.
pub fn occupants_per_dwelling(area_per_dwelling_m2: f64) -> f64 {
    if area_per_dwelling_m2 <= 30.0 {
        1.0
    } else if area_per_dwelling_m2 <= 100.0 {
        2.28 - 1.28 / 70.0 * (100.0 - area_per_dwelling_m2)
    } else {
        1.28 + 0.01 * area_per_dwelling_m2
    }
}

/// 7.6.6.4: sky view factor by tilt (0° horizontal, 90° vertical).
pub fn sky_view_factor(tilt_deg: f64) -> f64 {
    if tilt_deg <= 5.0 {
        1.0
    } else if tilt_deg <= 75.0 {
        0.75
    } else {
        0.5
    }
}

/// 7.46–7.49: gain utilisation for heating.
pub fn heating_utilization(gamma: f64, a: f64) -> f64 {
    if gamma <= 0.0 {
        1.0
    } else if (gamma - 1.0).abs() < 1e-9 {
        a / (a + 1.0)
    } else {
        (1.0 - gamma.powf(a)) / (1.0 - gamma.powf(a + 1.0))
    }
}

/// 7.52–7.55: loss utilisation for cooling, with `γ_C = Q_C;gn / Q_C;ht`.
pub fn cooling_utilization(gamma: f64, a: f64) -> f64 {
    if (gamma - 1.0).abs() < 1e-9 {
        a / (a + 1.0)
    } else {
        (1.0 - gamma.powf(-a)) / (1.0 - gamma.powf(-(a + 1.0)))
    }
}

/// 7.39: sky radiation loss of one envelope element in kWh.
fn sky_loss_kwh(tilt_deg: f64, u: f64, area: f64, hours: f64) -> f64 {
    sky_view_factor(tilt_deg) * R_SE * u * area * H_LR_E * DELTA_THETA_SKY * hours * 0.001
}

fn finite_nonneg(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn check_reference(value: &str, path: String, issues: &mut Vec<DemandIssue>) {
    if value.trim().is_empty() {
        issues.push(issue("source_reference_required", path));
    }
}

fn check_tilt(tilt: f64, path: String, issues: &mut Vec<DemandIssue>) {
    if tilt != 0.0 && tilt != 90.0 {
        issues.push(issue("tilt_unsupported", path));
    }
}

fn validate(input: &MonthlyDemandInput, issues: &mut Vec<DemandIssue>) {
    if input.zone_id.trim().is_empty() {
        issues.push(issue("zone_id_required", "zoneId"));
    }
    if !input.usable_floor_area_m2.is_finite() || input.usable_floor_area_m2 <= 0.0 {
        issues.push(issue("usable_floor_area_invalid", "usableFloorAreaM2"));
    }
    check_reference(
        &input.area_source_reference,
        "areaSourceReference".into(),
        issues,
    );
    let setpoints = &input.setpoints;
    if !setpoints.heating_c.is_finite()
        || !setpoints.cooling_c.is_finite()
        || setpoints.cooling_c < setpoints.heating_c
    {
        issues.push(issue("setpoints_invalid", "setpoints"));
    }
    check_reference(
        &setpoints.source_reference,
        "setpoints.sourceReference".into(),
        issues,
    );

    let transmission = &input.transmission;
    if !finite_nonneg(transmission.conductance_w_per_k) {
        issues.push(issue(
            "transmission_conductance_invalid",
            "transmission.conductanceWPerK",
        ));
    }
    check_reference(
        &transmission.source_reference,
        "transmission.sourceReference".into(),
        issues,
    );
    if !transmission.ground_inventory_confirmed {
        issues.push(issue(
            "ground_inventory_unconfirmed",
            "transmission.groundInventoryConfirmed",
        ));
    }
    if let Some(ground) = &transmission.ground {
        if !finite_nonneg(ground.adjusted_conductance_w_per_k) {
            issues.push(issue(
                "ground_conductance_invalid",
                "transmission.ground.adjustedConductanceWPerK",
            ));
        }
        for (name, values) in [
            ("heatingKwh", &ground.heating_kwh),
            ("coolingKwh", &ground.cooling_kwh),
        ] {
            if values.len() != 12 || values.iter().any(|value| !value.is_finite()) {
                issues.push(issue(
                    "ground_monthly_invalid",
                    format!("transmission.ground.{name}"),
                ));
            }
        }
        check_reference(
            &ground.source_reference,
            "transmission.ground.sourceReference".into(),
            issues,
        );
    }

    if input.ventilation_flows.is_empty() {
        issues.push(issue("ventilation_flow_required", "ventilationFlows"));
    }
    let mut flow_ids = HashSet::new();
    for (index, flow) in input.ventilation_flows.iter().enumerate() {
        let path = format!("ventilationFlows[{index}]");
        if flow.id.trim().is_empty() || !flow_ids.insert(flow.id.as_str()) {
            issues.push(issue("ventilation_flow_id_invalid", format!("{path}.id")));
        }
        check_reference(
            &flow.source_reference,
            format!("{path}.sourceReference"),
            issues,
        );
        if flow.months.len() != 12 {
            issues.push(issue(
                "ventilation_twelve_months_required",
                format!("{path}.months"),
            ));
        }
        let mut months = HashSet::new();
        for (month_index, row) in flow.months.iter().enumerate() {
            let row_path = format!("{path}.months[{month_index}]");
            if !(1..=12).contains(&row.month) || !months.insert(row.month) {
                issues.push(issue(
                    "ventilation_month_invalid",
                    format!("{row_path}.month"),
                ));
            }
            if !finite_nonneg(row.conductance_w_per_k) {
                issues.push(issue(
                    "ventilation_conductance_invalid",
                    format!("{row_path}.conductanceWPerK"),
                ));
            }
            if row
                .supply_temperature_c
                .is_some_and(|value| !value.is_finite())
            {
                issues.push(issue(
                    "ventilation_supply_temperature_invalid",
                    format!("{row_path}.supplyTemperatureC"),
                ));
            }
        }
    }

    check_reference(
        &input.thermal_mass.source_reference,
        "thermalMass.sourceReference".into(),
        issues,
    );
    match &input.internal_gains {
        InternalGains::Residential {
            dwelling_count,
            source_reference,
        } => {
            if *dwelling_count == 0 {
                issues.push(issue(
                    "dwelling_count_invalid",
                    "internalGains.dwellingCount",
                ));
            }
            check_reference(
                source_reference,
                "internalGains.sourceReference".into(),
                issues,
            );
        }
        InternalGains::Declared {
            heat_flux_w_per_m2,
            source_reference,
        } => {
            if !finite_nonneg(*heat_flux_w_per_m2) {
                issues.push(issue(
                    "internal_heat_flux_invalid",
                    "internalGains.heatFluxWPerM2",
                ));
            }
            check_reference(
                source_reference,
                "internalGains.sourceReference".into(),
                issues,
            );
        }
    }

    if !input.window_inventory_complete {
        issues.push(issue(
            "window_inventory_incomplete",
            "windowInventoryComplete",
        ));
    }
    if !input.opaque_inventory_complete {
        issues.push(issue(
            "opaque_inventory_incomplete",
            "opaqueInventoryComplete",
        ));
    }
    let mut element_ids = HashSet::new();
    for (index, window) in input.windows.iter().enumerate() {
        let path = format!("windows[{index}]");
        if window.id.trim().is_empty() || !element_ids.insert(window.id.as_str()) {
            issues.push(issue("element_id_invalid", format!("{path}.id")));
        }
        if !window.area_m2.is_finite() || window.area_m2 <= 0.0 {
            issues.push(issue("element_area_invalid", format!("{path}.areaM2")));
        }
        check_tilt(window.tilt_deg, format!("{path}.tiltDeg"), issues);
        if !(0.0..=1.0).contains(&window.g_perpendicular) {
            issues.push(issue("window_g_invalid", format!("{path}.gPerpendicular")));
        }
        if !(0.0..1.0).contains(&window.frame_fraction) {
            issues.push(issue(
                "window_frame_fraction_invalid",
                format!("{path}.frameFraction"),
            ));
        }
        if !window.u_value_w_per_m2k.is_finite() || window.u_value_w_per_m2k <= 0.0 {
            issues.push(issue(
                "element_u_value_invalid",
                format!("{path}.uValueWPerM2k"),
            ));
        }
        if !(0.0..=1.0).contains(&window.obstruction_factor) {
            issues.push(issue(
                "window_obstruction_factor_invalid",
                format!("{path}.obstructionFactor"),
            ));
        }
        check_reference(
            &window.source_reference,
            format!("{path}.sourceReference"),
            issues,
        );
    }
    for (index, element) in input.opaque_elements.iter().enumerate() {
        let path = format!("opaqueElements[{index}]");
        if element.id.trim().is_empty() || !element_ids.insert(element.id.as_str()) {
            issues.push(issue("element_id_invalid", format!("{path}.id")));
        }
        if !element.area_m2.is_finite() || element.area_m2 <= 0.0 {
            issues.push(issue("element_area_invalid", format!("{path}.areaM2")));
        }
        check_tilt(element.tilt_deg, format!("{path}.tiltDeg"), issues);
        if !element.u_value_w_per_m2k.is_finite() || element.u_value_w_per_m2k <= 0.0 {
            issues.push(issue(
                "element_u_value_invalid",
                format!("{path}.uValueWPerM2k"),
            ));
        }
        check_reference(
            &element.source_reference,
            format!("{path}.sourceReference"),
            issues,
        );
    }
}

pub fn assess_monthly_demand(input: &MonthlyDemandInput) -> MonthlyDemandAssessment {
    let fingerprint =
        input_fingerprint(&serde_json::to_value(input).expect("typed input serializes"));
    let mut issues = Vec::new();
    validate(input, &mut issues);

    let mass = &input.thermal_mass;
    let d_m = specific_heat_capacity(mass.floor, mass.wall, mass.ceiling);
    let mut monthly = Vec::with_capacity(12);
    if issues.is_empty() {
        monthly = compute(input, d_m, &mut issues);
    }
    let valid = issues.is_empty();
    if !valid {
        monthly.clear();
    }
    let annual_heating = valid.then(|| monthly.iter().map(|row| row.heating.need_kwh).sum());
    let annual_cooling = valid.then(|| monthly.iter().map(|row| row.cooling.need_kwh).sum());
    MonthlyDemandAssessment {
        status: if valid {
            "calculated_unverified"
        } else {
            "invalid"
        },
        scope: SCOPE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        climate_source: CLIMATE_SOURCE,
        input_fingerprint: fingerprint,
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        omitted_corrections: OMITTED_CORRECTIONS,
        specific_heat_capacity_kj_per_m2k: valid.then_some(d_m),
        monthly,
        annual_heating_need_kwh: annual_heating,
        annual_cooling_need_kwh: annual_cooling,
        issues,
    }
}

fn compute(
    input: &MonthlyDemandInput,
    d_m: f64,
    issues: &mut Vec<DemandIssue>,
) -> Vec<MonthResult> {
    let area = input.usable_floor_area_m2;
    // 7.45: C_m;int;eff in J/K.
    let capacity_j_per_k = d_m * 1000.0 * area;
    let transmission = &input.transmission;
    let ground_adjusted = transmission
        .ground
        .as_ref()
        .map_or(0.0, |ground| ground.adjusted_conductance_w_per_k);
    let mut results = Vec::with_capacity(12);
    for month in 1..=12u8 {
        let index = usize::from(month - 1);
        let hours = MONTH_HOURS[index];
        let outdoor = OUTDOOR_TEMPERATURE_C[index];

        let mut ventilation_conductance = 0.0;
        let mut ventilation_heating = 0.0;
        let mut ventilation_cooling = 0.0;
        for flow in &input.ventilation_flows {
            let row = flow
                .months
                .iter()
                .find(|row| row.month == month)
                .expect("validated twelve unique months");
            let supply = row.supply_temperature_c.unwrap_or(outdoor);
            ventilation_conductance += row.conductance_w_per_k;
            ventilation_heating +=
                row.conductance_w_per_k * (input.setpoints.heating_c - supply) * hours / 1000.0;
            ventilation_cooling +=
                row.conductance_w_per_k * (input.setpoints.cooling_c - supply) * hours / 1000.0;
        }
        let (ground_heating, ground_cooling) =
            transmission.ground.as_ref().map_or((0.0, 0.0), |ground| {
                (ground.heating_kwh[index], ground.cooling_kwh[index])
            });
        let transmission_heating =
            transmission.conductance_w_per_k * (input.setpoints.heating_c - outdoor) * hours
                / 1000.0
                + ground_heating;
        let transmission_cooling =
            transmission.conductance_w_per_k * (input.setpoints.cooling_c - outdoor) * hours
                / 1000.0
                + ground_cooling;

        let internal = match &input.internal_gains {
            InternalGains::Residential { dwelling_count, .. } => {
                let dwellings = f64::from(*dwelling_count);
                INTERNAL_HEAT_PER_OCCUPANT_W
                    * dwellings
                    * occupants_per_dwelling(area / dwellings)
                    * 0.001
                    * hours
            }
            InternalGains::Declared {
                heat_flux_w_per_m2, ..
            } => heat_flux_w_per_m2 * area * hours / 1000.0,
        };

        let mut window_solar = 0.0;
        for window in &input.windows {
            let irradiance =
                climate::irradiance_w_per_m2(window.orientation, window.tilt_deg, month)
                    .expect("validated tilt");
            let gross = F_W
                * window.g_perpendicular
                * window.area_m2
                * (1.0 - window.frame_fraction)
                * window.obstruction_factor
                * irradiance
                * hours
                * 0.001;
            window_solar += gross
                - sky_loss_kwh(
                    window.tilt_deg,
                    window.u_value_w_per_m2k,
                    window.area_m2,
                    hours,
                );
        }
        let mut opaque_solar = 0.0;
        for element in &input.opaque_elements {
            let irradiance =
                climate::irradiance_w_per_m2(element.orientation, element.tilt_deg, month)
                    .expect("validated tilt");
            let gross = ALPHA_SOL
                * R_SE
                * element.u_value_w_per_m2k
                * element.area_m2
                * irradiance
                * hours
                * 0.001;
            opaque_solar += gross
                - sky_loss_kwh(
                    element.tilt_deg,
                    element.u_value_w_per_m2k,
                    element.area_m2,
                    hours,
                );
        }

        let conductance =
            transmission.conductance_w_per_k + ground_adjusted + ventilation_conductance;
        if conductance <= 0.0 {
            issues.push(issue(
                "total_conductance_nonpositive",
                format!("month[{month}]"),
            ));
            return Vec::new();
        }
        // 7.57 and 7.51.
        let tau = capacity_j_per_k / 3600.0 / conductance;
        let a = A_0 + tau / TAU_0_H;
        let gains = internal + window_solar + opaque_solar;

        let heat_transfer_heating = transmission_heating + ventilation_heating;
        let heating = if heat_transfer_heating <= 0.0 {
            BalanceTerms {
                transmission_kwh: transmission_heating,
                ventilation_kwh: ventilation_heating,
                heat_transfer_kwh: heat_transfer_heating,
                gains_kwh: gains,
                gamma: None,
                utilization: 1.0,
                need_kwh: 0.0,
            }
        } else {
            let gamma = gains / heat_transfer_heating;
            let eta = heating_utilization(gamma, a);
            BalanceTerms {
                transmission_kwh: transmission_heating,
                ventilation_kwh: ventilation_heating,
                heat_transfer_kwh: heat_transfer_heating,
                gains_kwh: gains,
                gamma: Some(gamma),
                utilization: eta,
                need_kwh: (heat_transfer_heating - eta * gains).max(0.0),
            }
        };

        let heat_transfer_cooling = transmission_cooling + ventilation_cooling;
        if heat_transfer_cooling <= 0.0 && gains > 0.0 {
            // Outdoor or supply air warmer than the cooling setpoint: the
            // utilisation route for this case is not transcribed.
            issues.push(issue(
                "cooling_heat_transfer_nonpositive_unsupported",
                format!("month[{month}]"),
            ));
            return Vec::new();
        }
        let cooling = if gains <= 0.0 || heat_transfer_cooling / gains > 2.0 {
            // 7.6 gate: (1/γ_C) > 2 → no cooling need; no gains → none either.
            BalanceTerms {
                transmission_kwh: transmission_cooling,
                ventilation_kwh: ventilation_cooling,
                heat_transfer_kwh: heat_transfer_cooling,
                gains_kwh: gains,
                gamma: (gains > 0.0).then(|| gains / heat_transfer_cooling),
                utilization: 0.0,
                need_kwh: 0.0,
            }
        } else {
            let gamma = gains / heat_transfer_cooling;
            let eta = cooling_utilization(gamma, a);
            BalanceTerms {
                transmission_kwh: transmission_cooling,
                ventilation_kwh: ventilation_cooling,
                heat_transfer_kwh: heat_transfer_cooling,
                gains_kwh: gains,
                gamma: Some(gamma),
                utilization: eta,
                need_kwh: (gains - eta * heat_transfer_cooling).max(0.0),
            }
        };
        let row = MonthResult {
            month,
            hours,
            outdoor_temperature_c: outdoor,
            time_constant_h: tau,
            a,
            internal_gains_kwh: internal,
            window_solar_gains_kwh: window_solar,
            opaque_solar_gains_kwh: opaque_solar,
            heating,
            cooling,
        };
        let values = [
            row.heating.need_kwh,
            row.cooling.need_kwh,
            row.heating.heat_transfer_kwh,
            row.cooling.heat_transfer_kwh,
            row.time_constant_h,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            issues.push(issue("monthly_result_overflow", format!("month[{month}]")));
            return Vec::new();
        }
        results.push(row);
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    pub(crate) fn sample() -> MonthlyDemandInput {
        serde_json::from_value(json!({
            "zoneId": "rz-1",
            "usableFloorAreaM2": 100.0,
            "areaSourceReference": "synthetic plan",
            "setpoints": {"heatingC": 20.0, "coolingC": 24.0, "sourceReference": "table 7.13 residential"},
            "transmission": {
                "conductanceWPerK": 80.0,
                "sourceReference": "synthetic envelope",
                "ground": null,
                "groundInventoryConfirmed": true
            },
            "ventilationFlows": [{
                "id": "natural",
                "sourceReference": "synthetic flow",
                "months": (1..=12).map(|month| json!({"month": month, "conductanceWPerK": 40.0})).collect::<Vec<_>>()
            }],
            "thermalMass": {"floor": "very_heavy", "wall": "light", "ceiling": "open_or_none", "sourceReference": "synthetic"},
            "internalGains": {"method": "residential", "dwellingCount": 1, "sourceReference": "single dwelling"},
            "windowInventoryComplete": true,
            "windows": [{
                "id": "w-south", "areaM2": 10.0, "orientation": "south", "tiltDeg": 90.0,
                "gPerpendicular": 0.6, "frameFraction": 0.25, "uValueWPerM2k": 1.2,
                "obstructionFactor": 1.0, "sourceReference": "synthetic window"
            }],
            "opaqueInventoryComplete": true,
            "opaqueElements": [{
                "id": "roof", "areaM2": 50.0, "orientation": "south", "tiltDeg": 0.0,
                "uValueWPerM2k": 0.16, "sourceReference": "synthetic roof"
            }]
        }))
        .unwrap()
    }

    #[test]
    fn table_7_10_and_occupancy_bands() {
        assert_eq!(
            specific_heat_capacity(
                MassClass::Light,
                MassClass::Light,
                CeilingColumn::ClosedOrSuspended
            ),
            55.0
        );
        assert_eq!(
            specific_heat_capacity(
                MassClass::VeryHeavy,
                MassClass::Light,
                CeilingColumn::OpenOrNone
            ),
            180.0
        );
        assert_eq!(
            specific_heat_capacity(
                MassClass::VeryHeavy,
                MassClass::VeryHeavy,
                CeilingColumn::OpenOrNone
            ),
            450.0
        );
        assert_eq!(occupants_per_dwelling(30.0), 1.0);
        assert!((occupants_per_dwelling(67.0) - 1.676_571_428_571_428_6).abs() < 1e-12);
        assert!((occupants_per_dwelling(133.06) - 2.6106).abs() < 1e-12);
        // Continuity at the 100 m² band edge.
        assert!((occupants_per_dwelling(100.0) - 2.28).abs() < 1e-12);
    }

    #[test]
    fn utilisation_limits() {
        assert_eq!(heating_utilization(0.0, 2.0), 1.0);
        assert!((heating_utilization(1.0, 2.0) - 2.0 / 3.0).abs() < 1e-12);
        assert!((heating_utilization(0.5, 2.0) - (1.0 - 0.25) / (1.0 - 0.125)).abs() < 1e-12);
        assert!((cooling_utilization(1.0, 2.0) - 2.0 / 3.0).abs() < 1e-12);
        assert!((cooling_utilization(2.0, 2.0) - (1.0 - 0.25) / (1.0 - 0.125)).abs() < 1e-12);
    }

    #[test]
    fn hand_checked_january_balance() {
        let result = assess_monthly_demand(&sample());
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        let hours = 744.0;
        let q_tr = 80.0 * (20.0 - 2.61) * hours / 1000.0;
        let q_ve = 40.0 * (20.0 - 2.61) * hours / 1000.0;
        assert!((jan.heating.transmission_kwh - q_tr).abs() < 1e-9);
        assert!((jan.heating.ventilation_kwh - q_ve).abs() < 1e-9);
        let internal = 180.0 * 2.28 * 0.001 * hours;
        assert!((jan.internal_gains_kwh - internal).abs() < 1e-9);
        let window = 0.9 * 0.6 * 10.0 * 0.75 * 60.1 * hours * 0.001
            - 0.5 * 0.04 * 1.2 * 10.0 * 4.14 * 11.0 * hours * 0.001;
        assert!((jan.window_solar_gains_kwh - window).abs() < 1e-9);
        let roof = 0.6 * 0.04 * 0.16 * 50.0 * 28.0 * hours * 0.001
            - 1.0 * 0.04 * 0.16 * 50.0 * 4.14 * 11.0 * hours * 0.001;
        assert!((jan.opaque_solar_gains_kwh - roof).abs() < 1e-9);
        let tau = 180.0 * 1000.0 * 100.0 / 3600.0 / 120.0;
        assert!((jan.time_constant_h - tau).abs() < 1e-9);
        let a = 1.0 + tau / 15.0;
        let gains = internal + window + roof;
        let gamma = gains / (q_tr + q_ve);
        let eta = (1.0 - gamma.powf(a)) / (1.0 - gamma.powf(a + 1.0));
        assert!((jan.heating.need_kwh - (q_tr + q_ve - eta * gains)).abs() < 1e-9);
        assert_eq!(jan.cooling.need_kwh, 0.0);
        assert!(result.annual_heating_need_kwh.unwrap() > result.monthly[0].heating.need_kwh);
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn gate_and_cooling_month() {
        let mut input = sample();
        input.windows[0].area_m2 = 40.0;
        let result = assess_monthly_demand(&input);
        let july = &result.monthly[6];
        let ratio = july.cooling.heat_transfer_kwh / july.cooling.gains_kwh;
        assert!(ratio <= 2.0);
        assert!(july.cooling.need_kwh > 0.0);
        let base = assess_monthly_demand(&sample());
        let jan = &base.monthly[0];
        assert!(jan.cooling.heat_transfer_kwh / jan.cooling.gains_kwh > 2.0);
        assert_eq!(jan.cooling.need_kwh, 0.0);
    }

    #[test]
    fn heavier_mass_lowers_heating_need() {
        let light = {
            let mut input = sample();
            input.thermal_mass.floor = MassClass::Light;
            input.thermal_mass.ceiling = CeilingColumn::ClosedOrSuspended;
            assess_monthly_demand(&input)
                .annual_heating_need_kwh
                .unwrap()
        };
        let heavy = assess_monthly_demand(&sample())
            .annual_heating_need_kwh
            .unwrap();
        assert!(heavy < light);
    }

    #[test]
    fn supply_temperature_reduces_ventilation_loss() {
        let mut input = sample();
        for row in &mut input.ventilation_flows[0].months {
            row.supply_temperature_c = Some(15.0);
        }
        let result = assess_monthly_demand(&input);
        let jan = &result.monthly[0];
        assert!((jan.heating.ventilation_kwh - 40.0 * 5.0 * 744.0 / 1000.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_incomplete_or_unsupported_input_without_numbers() {
        let mut input = sample();
        input.windows[0].tilt_deg = 45.0;
        input.window_inventory_complete = false;
        input.transmission.ground_inventory_confirmed = false;
        input.ventilation_flows[0].months.pop();
        input.opaque_elements[0].id = "w-south".into();
        let result = assess_monthly_demand(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.monthly.is_empty());
        assert!(result.annual_heating_need_kwh.is_none());
        let codes: Vec<_> = result.issues.iter().map(|item| item.code).collect();
        for code in [
            "tilt_unsupported",
            "window_inventory_incomplete",
            "ground_inventory_unconfirmed",
            "ventilation_twelve_months_required",
            "element_id_invalid",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    #[test]
    fn rejects_unknown_fields() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["windows"][0]["gValue"] = json!(0.5);
        assert!(serde_json::from_value::<MonthlyDemandInput>(value).is_err());
    }

    #[test]
    fn ground_terms_enter_transfer_and_time_constant() {
        let mut input = sample();
        input.transmission.ground = Some(GroundTransfer {
            adjusted_conductance_w_per_k: 30.0,
            heating_kwh: vec![100.0; 12],
            cooling_kwh: vec![150.0; 12],
            source_reference: "synthetic 8.3".into(),
        });
        let result = assess_monthly_demand(&input);
        let base = assess_monthly_demand(&sample());
        let jan = &result.monthly[0];
        assert!(
            (jan.heating.transmission_kwh - base.monthly[0].heating.transmission_kwh - 100.0).abs()
                < 1e-9
        );
        assert!((jan.time_constant_h - 180.0 * 1000.0 * 100.0 / 3600.0 / 150.0).abs() < 1e-9);
    }
}
