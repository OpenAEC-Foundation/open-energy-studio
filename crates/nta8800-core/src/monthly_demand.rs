//! Monthly heating and cooling need of one calculation zone, NTA 8800 chapter 7.
//!
//! Implements the monthly balance of 7.2 (p. 165–168): heating
//! `Q_H;nd = Q_H;ht − η_H;gn · Q_H;gn` with the gates 7.1/7.2, cooling
//! `Q_C;nd = a_C;red · (Q_C;gn − η_C;ht · Q_C;ht)` with the gate 7.6.
//! Transmission (7.14/7.15) and ventilation (7.18–7.20) use the calculation
//! temperature of §7.9 (p. 211–220): intermittent heating 7.59–7.73 with
//! tables 7.14/7.15, dwelling temperature levelling 7.78/7.79 and the
//! intermittent-cooling factor 7.74/7.75. Ground transfer uses the monthly
//! `H_g;an;mi` of annex D; the time constants 7.57/7.58 use the seasonal
//! `H_H/C;g;adj` and the monthly `H_H/C;ve` including `b_v`. Gains: internal
//! (7.21–7.24 or declared), window solar (7.32, 7.40, 7.42) and opaque solar
//! (7.33), both reduced by sky radiation (7.39). Utilisation 7.46–7.56.
//!
//! Corrections this module does not apply are listed in
//! [`OMITTED_CORRECTIONS`] and returned with every result.

use crate::climate::{self, Orientation, CLIMATE_SOURCE, MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use crate::direct_transmission::{assess_direct_transmission, DirectTransmissionInput};
use crate::ground::{ground_monthly, slab_coefficients, EdgeThermalBridges, SlabOnGround};
use crate::solar_shading::{
    movable_shading_factor, obstruction_factor, validate_obstruction, Balance, MovableShading,
    Obstruction,
};
use crate::unheated_transmission::{assess_unheated_transmission, UnheatedTransmissionInput};
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
/// 7.51/7.56: reference parameters of the monthly method.
pub const A_0: f64 = 1.0;
pub const TAU_0_H: f64 = 15.0;
/// 7.21: internal heat per occupant in W.
pub const INTERNAL_HEAT_PER_OCCUPANT_W: f64 = 180.0;
/// 7.2: heating need is zero above this heat-balance ratio.
pub const GAMMA_H_MAX: f64 = 2.0;
/// 7.74: empirical correlation factor for intermittent cooling.
pub const B_C_RED_WKND: f64 = 0.3;
/// 7.78: time fraction of moderate heating.
pub const F_MOD_T: f64 = 0.8;
/// 7.78: internal specific heat-transfer coefficient, W/(m²K).
pub const H_INT_SPEC: f64 = 2.0;

pub const OMITTED_CORRECTIONS: &[&str] = &[
    "7.3–7.5 and 7.7–7.9 recoverable losses are applied by the heating chain (apply_recoverable_losses); Q_C;ls;rbl of chapter 10 is still 0",
    "8.5 adjacent heated spaces H_A",
    "§17.3.8 extended obstruction method (hourly NEN 5060) enters as declared factors",
    "table 7.10 footnote c is the caller's column choice",
    "annex D for floors other than slab on ground (crawlspace, basement)",
];

const SCOPE: &str = "nta8800_chapter_7_monthly_need_single_zone_unverified";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonthlyDemandInput {
    pub zone_id: String,
    pub usable_floor_area_m2: f64,
    pub area_source_reference: String,
    /// Usage function of the zone for tables 7.13–7.15 (the largest one
    /// when `function_areas` lists several).
    pub usage_function: UsageFunction,
    /// §6.5.3: functions with their areas in a mixed zone; values are
    /// weighted by usable floor area. Empty for a single-function zone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub function_areas: Vec<UsageFunctionArea>,
    /// `f_mod;sp` of 7.78; required for the residential function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dwelling_type: Option<DwellingType>,
    pub setpoints: Setpoints,
    pub transmission: Transmission,
    /// Explicit H_ve flows; leave empty when `ventilation` is given.
    #[serde(default)]
    pub ventilation_flows: Vec<VentilationFlow>,
    /// Chapter 11 input; the kernel derives the flows and the fixed C1 run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ventilation: Option<crate::ventilation::VentilationInput>,
    pub thermal_mass: ThermalMass,
    pub internal_gains: InternalGains,
    pub window_inventory_complete: bool,
    pub windows: Vec<Window>,
    pub opaque_inventory_complete: bool,
    pub opaque_elements: Vec<OpaqueElement>,
    /// Adjacent unheated sunrooms (7.30b, §7.6.4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sunrooms: Vec<Sunroom>,
    /// Maatwerkadvies only (ISSO 82.2/75.2 table 2.2): actual-use
    /// parameters. A run with a fit is never a label calculation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_fit: Option<UsageFit>,
}

/// ISSO 82.2/75.2 §2.5, table 2.2: user-dependent parameters of the
/// maatwerkadvies. With a fit, `setpoints` may deviate from table 7.13.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageFit {
    /// θ_int;set;H;low (day and weekend), °C.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reduced_setpoint_c: Option<f64>,
    /// t_H;red;day, h per (work)day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day_reduction_h: Option<f64>,
    /// t_H;red;wknd, h per weekend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weekend_reduction_h: Option<f64>,
    /// t_C;red;wknd, h per weekend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooling_weekend_reduction_h: Option<f64>,
    /// f_mod;sp (residential).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_fraction: Option<f64>,
    /// N_p;woon;zi, occupants of the zone in total (residential).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupants: Option<f64>,
    /// q_H/C;int;tot per person, W (residential; persons, appliances and
    /// lighting, mean over the year).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_gain_per_person_w: Option<f64>,
    /// ISSO 82.2 table 2.7 / 75.2 table 2.8 ventilation practice factors
    /// for the chapter 11 route.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ventilation_practice: Option<crate::ventilation::VentilationPractice>,
    /// q_Oc·f_τ + q_A for utility functions, W/m².
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupancy_appliance_w_per_m2: Option<f64>,
    pub source_reference: String,
}

/// Usage functions of tables 7.13–7.15.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageFunction {
    AssemblyChildCare,
    OtherAssembly,
    Cell,
    HealthcareWithBeds,
    OtherHealthcare,
    Office,
    Lodging,
    Education,
    Sport,
    Retail,
    Residential,
}

impl UsageFunction {
    /// Table 7.13 `θ_int;set;H;stc`, °C.
    pub fn heating_setpoint_c(self) -> f64 {
        match self {
            Self::HealthcareWithBeds => 22.0,
            Self::Sport => 16.0,
            Self::Residential => 20.0,
            _ => 21.0,
        }
    }

    /// Table 7.13 `θ_int;set;C;stc`, °C.
    pub fn cooling_setpoint_c(self) -> f64 {
        24.0
    }

    /// Table 7.14 reduced setpoint for night and weekend, °C.
    pub fn reduced_setpoint_c(self) -> f64 {
        match self {
            Self::Sport => 14.0,
            _ => 16.0,
        }
    }

    /// Table 7.15: `(t_H;red;day, t_H;red;wknd, t_C;red;wknd)` in h.
    pub fn reduction_hours(self) -> (f64, f64, f64) {
        match self {
            Self::Cell | Self::HealthcareWithBeds => (8.0, 0.0, 0.0),
            Self::Lodging | Self::Retail => (13.0, 24.0, 24.0),
            Self::Residential => (10.0, 0.0, 0.0),
            _ => (14.0, 48.0, 48.0),
        }
    }

    pub fn is_residential(self) -> bool {
        self == Self::Residential
    }
}

/// 7.78: `f_mod;sp` 0,5 for apartment buildings (woongebouwen), 0,6 for all
/// other dwellings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DwellingType {
    ApartmentBuilding,
    Other,
}

impl DwellingType {
    pub fn spatial_fraction(self) -> f64 {
        match self {
            Self::ApartmentBuilding => 0.5,
            Self::Other => 0.6,
        }
    }
}

/// Table 7.13 `θ_int;set;H/C;stc`. Supplied explicitly with a source and
/// checked against the table for the usage function.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Setpoints {
    pub heating_c: f64,
    pub cooling_c: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Transmission {
    /// Coefficients determined elsewhere and supplied as totals.
    Explicit(ExplicitTransmission),
    /// Coefficients composed here from chapter 8 components.
    Components(ComponentTransmission),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentTransmission {
    /// Direct to outdoor air: 8.1 `Σ A·U + Σ L·ψ + Σ χ`.
    pub direct: DirectTransmissionInput,
    /// Via unheated spaces with supplied reduction factors `b`.
    pub unheated: Option<UnheatedTransmissionInput>,
    /// Slab-on-ground floors, §8.3.
    pub ground_floors: Vec<SlabOnGround>,
    pub ground_inventory_confirmed: bool,
    /// 7.3.3 vertical pipes through the thermal envelope open to outdoor
    /// air (rainwater, sewer and vent stacks), `H_p` of 7.17.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vertical_pipes: Vec<VerticalPipe>,
}

/// One vertical pipe of 7.3.3.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerticalPipe {
    pub id: String,
    /// N_bouwlaag;j: storeys of the zone (all of them, note 2 of 7.3.3).
    pub storeys: u32,
    /// Table 7.1 footnote a: more than 90 % insulated with λ ≤ 0,1.
    pub insulated: bool,
    /// Zones or adjacent heated spaces the pipe borders, this zone
    /// included; H_p is split evenly (7.3.3). Default 1.
    #[serde(default = "one_zone")]
    pub shared_zones: u32,
    pub source_reference: String,
}

fn one_zone() -> u32 {
    1
}

/// Table 7.1, W/K per storey.
pub const VERTICAL_PIPE_UNINSULATED_W_PER_K: f64 = 1.8;
pub const VERTICAL_PIPE_INSULATED_W_PER_K: f64 = 0.5;

/// 7.17 with the split over the bordering zones.
pub fn vertical_pipe_conductance_w_per_k(pipes: &[VerticalPipe]) -> f64 {
    pipes
        .iter()
        .map(|pipe| {
            let specific = if pipe.insulated {
                VERTICAL_PIPE_INSULATED_W_PER_K
            } else {
                VERTICAL_PIPE_UNINSULATED_W_PER_K
            };
            f64::from(pipe.storeys) * specific / f64::from(pipe.shared_zones.max(1))
        })
        .sum()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExplicitTransmission {
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
    /// Annex D.1 `H_g;an;mi` for January–December, W/K.
    pub monthly_conductance_w_per_k: Vec<f64>,
    /// Annex D.2 `H_H;g;adj` for the heating time constant (7.57), W/K.
    pub heating_adjusted_conductance_w_per_k: f64,
    /// Annex D.3 `H_C;g;adj` for the cooling time constant (7.58), W/K.
    pub cooling_adjusted_conductance_w_per_k: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VentilationFlow {
    pub id: String,
    pub source_reference: String,
    pub months: Vec<VentilationMonth>,
}

/// One air flow `k` of 7.19 in one month.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VentilationMonth {
    pub month: u8,
    /// `ρ_a·c_a·q_v;k;H;mi / 3600` for the heating balance, W/K.
    pub conductance_w_per_k: f64,
    /// Supply temperature for the heating balance; `None` means outdoor air
    /// at `θ_e;avg;mi`.
    #[serde(default)]
    pub supply_temperature_c: Option<f64>,
    /// Cooling-balance conductance; `None` means the heating value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooling_conductance_w_per_k: Option<f64>,
    /// Cooling-balance supply temperature; `None` means the heating value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooling_supply_temperature_c: Option<f64>,
}

impl VentilationMonth {
    /// `(conductance, supply temperature)` for one balance.
    fn balance(&self, balance: Balance, outdoor: f64) -> (f64, f64) {
        let heating_supply = self.supply_temperature_c.unwrap_or(outdoor);
        match balance {
            Balance::Heating => (self.conductance_w_per_k, heating_supply),
            Balance::Cooling => (
                self.cooling_conductance_w_per_k
                    .unwrap_or(self.conductance_w_per_k),
                self.cooling_supply_temperature_c.unwrap_or(heating_supply),
            ),
        }
    }
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
    /// Annex B elements; when given they replace table 7.10.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annex_b_elements: Vec<crate::annex_b::MassElement>,
}

impl ThermalMass {
    /// `D_m` in kJ/(m²·K): table 7.10, or annex B over the zone area.
    pub fn specific_capacity_kj_per_m2k(&self, usable_floor_area_m2: f64) -> f64 {
        if self.annex_b_elements.is_empty() {
            specific_heat_capacity(self.floor, self.wall, self.ceiling)
        } else {
            crate::annex_b::zone_capacity_j_per_k(&self.annex_b_elements)
                / 1000.0
                / usable_floor_area_m2
        }
    }
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
    /// 7.25–7.29 for utility functions: persons and appliances from tables
    /// 7.2/7.3, recoverable lighting loss from W_t (chapter 14) and
    /// recoverable hot-water losses (13.1.2).
    Utility {
        lighting: UtilityLighting,
        /// Q_W;ls;rbl per month, kWh; empty means none.
        #[serde(default, rename = "hotWaterRecoverableKwh")]
        hot_water_recoverable_kwh: Vec<f64>,
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

/// Φ_int;L of 7.28.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum UtilityLighting {
    /// Taken from the chapter 14 lighting of the same zone by the
    /// building-performance calculation.
    Chapter14,
    /// W_t (14.2.2) with f_L.
    Declared {
        #[serde(rename = "annualKwh")]
        annual_kwh: f64,
        recovery: LightingRecovery,
    },
    /// Φ_int;L in W as resolved from chapter 14.
    Resolved {
        #[serde(rename = "gainW")]
        gain_w: f64,
    },
}

/// f_L of 7.28.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LightingRecovery {
    /// P_n determined forfaitarily (14.3.4): 0,3.
    ForfaitPower,
    /// At least 70 % of the luminaires (by P_n) extracted: 0,5.
    ExtractedLuminaires,
    /// 1,0.
    Other,
}

impl LightingRecovery {
    fn factor(self) -> f64 {
        match self {
            Self::ForfaitPower => 0.3,
            Self::ExtractedLuminaires => 0.5,
            Self::Other => 1.0,
        }
    }
}

impl UsageFunction {
    /// Tables 7.2 and 7.3: q_Oc·f_τ + q_A, W/m².
    pub fn occupancy_and_appliance_flux_w_per_m2(self) -> f64 {
        let (q_oc, f_tau, q_a) = self.occupancy_table();
        q_oc * f_tau + q_a
    }

    /// Tables 7.2 and 7.3: (q_Oc W/m², f_τ, q_A W/m²).
    pub fn occupancy_table(self) -> (f64, f64, f64) {
        match self {
            Self::AssemblyChildCare => (10.0, 0.30, 1.0),
            Self::OtherAssembly => (10.0, 0.15, 1.0),
            Self::Cell => (3.0, 0.80, 2.0),
            Self::HealthcareWithBeds => (5.0, 0.80, 4.0),
            Self::OtherHealthcare => (5.0, 0.30, 3.0),
            Self::Office => (5.0, 0.30, 4.0),
            Self::Lodging => (3.0, 0.40, 2.0),
            Self::Education => (10.0, 0.30, 2.0),
            Self::Sport => (3.0, 0.30, 1.0),
            Self::Retail => (3.0, 0.40, 3.0),
            Self::Residential => (0.0, 0.0, 0.0),
        }
    }

    /// §5.4.2: fixed lighting flux q_L for the BENG 1 run, W/m².
    pub fn fixed_lighting_flux_w_per_m2(self) -> f64 {
        match self {
            Self::AssemblyChildCare | Self::OtherAssembly | Self::HealthcareWithBeds => 2.5,
            Self::Cell => 2.25,
            Self::OtherHealthcare | Self::Office => 1.25,
            Self::Lodging => 1.75,
            Self::Education => 1.0,
            Self::Sport | Self::Retail => 3.0,
            Self::Residential => 0.0,
        }
    }
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
    /// External obstruction `F_sh;obst` per balance (§17.3).
    pub obstruction: Obstruction,
    /// Movable solar shading (7.42/7.43); see [`ShadingControl`] for the
    /// heating balance.
    ///
    /// [`ShadingControl`]: crate::solar_shading::ShadingControl
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movable_shading: Option<MovableShading>,
    /// Annex A: switchable or otherwise dynamic g and U per month.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic: Option<crate::annex_a::DynamicTransparent>,
    pub source_reference: String,
}

impl Window {
    /// Perpendicular g of the month (annex A.2 when dynamic). The nominal
    /// `g_gl;n` is rounded down to a multiple of 0,05 (§7.6.6.1.2, below
    /// 7.40); the A.2 monthly weighted value of a dynamic window is used as
    /// calculated.
    pub fn g_for_month(&self, month_index: usize) -> f64 {
        self.dynamic.as_ref().map_or_else(
            || (self.g_perpendicular / 0.05 + 1e-9).floor() * 0.05,
            |item| item.g_for_month(month_index),
        )
    }

    /// U of the month (annex A.1 when dynamic).
    pub fn u_for_month(&self, month_index: usize) -> f64 {
        self.dynamic
            .as_ref()
            .map_or(self.u_value_w_per_m2k, |item| item.u_for_month(month_index))
    }
}

/// Annex A: monthly change of `H_D` from dynamic windows whose nominal U
/// is part of the transmission input, W/K.
pub fn dynamic_window_correction_w_per_k(input: &MonthlyDemandInput, month_index: usize) -> f64 {
    input
        .windows
        .iter()
        .filter(|window| window.dynamic.is_some())
        .map(|window| window.area_m2 * (window.u_for_month(month_index) - window.u_value_w_per_m2k))
        .sum()
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
    pub transmission: Option<TransmissionSummary>,
    pub monthly: Vec<MonthResult>,
    pub annual_heating_need_kwh: Option<f64>,
    pub annual_cooling_need_kwh: Option<f64>,
    /// True after 7.3–7.8 applied the recoverable system losses.
    pub recoverable_losses_applied: bool,
    /// Annual needs before the recoverable losses (BENG 1 basis, §5.4.2).
    pub annual_heating_need_without_recoverable_kwh: Option<f64>,
    pub annual_cooling_need_without_recoverable_kwh: Option<f64>,
    /// Chapter 11 result when the zone gives `ventilation`.
    pub ventilation: Option<crate::ventilation::VentilationResult>,
    /// §5.4.2 need with the fixed C1 system, for BENG 1.
    pub fixed_c1: Option<FixedC1Demand>,
    /// 9.28/9.29: monthly heating need for the heating limit, kWh (with
    /// chapter 11 input only).
    pub heating_limit_need_kwh: Vec<f64>,
    pub issues: Vec<DemandIssue>,
}

/// §5.4.2: Q_H;nd and Q_C;nd with the fixed C1 ventilation (§5.4.3) and,
/// for utility functions, Φ_int;W = 0 and the fixed lighting flux q_L.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedC1Demand {
    pub status: &'static str,
    pub monthly_heating_need_kwh: Vec<f64>,
    pub monthly_cooling_need_kwh: Vec<f64>,
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
    pub internal_gains_kwh: f64,
    /// Window solar gains on the heating balance.
    pub window_solar_gains_kwh: f64,
    /// Window solar gains on the cooling balance.
    pub window_solar_cooling_kwh: f64,
    pub opaque_solar_gains_kwh: f64,
    /// Annex D.1 `H_g;an;mi`, W/K.
    pub ground_conductance_w_per_k: f64,
    pub heating: BalanceTerms,
    pub cooling: BalanceTerms,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceTerms {
    /// `θ_int;set;H/C` after levelling (7.76/7.77), °C.
    pub setpoint_c: f64,
    /// `a_H;red` (7.60) or `a_C;red` (7.74).
    pub reduction_factor: f64,
    /// `θ_int;calc;H/C` (7.59, 7.9.3), °C.
    pub calculation_temperature_c: f64,
    /// `H_H/C;ve` (7.19 with `b_v`), W/K.
    pub ventilation_conductance_w_per_k: f64,
    /// `τ_H/C` (7.57/7.58), h.
    pub time_constant_h: f64,
    /// `a_H/C` (7.51/7.56).
    pub a: f64,
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

/// 7.6.6.4: sky view factor by tilt (0° facing up, 90° vertical, above 90°
/// overhanging towards the ground).
pub fn sky_view_factor(tilt_deg: f64) -> f64 {
    if tilt_deg <= 5.0 {
        1.0
    } else if tilt_deg <= 75.0 {
        0.75
    } else if tilt_deg <= 90.0 {
        0.5
    } else {
        0.0
    }
}

/// 7.46–7.49: gain utilisation for heating; `gains_kwh` decides between
/// 7.48 and 7.49 when `γ_H ≤ 0`.
pub fn heating_utilization(gamma: f64, a: f64, gains_kwh: f64) -> f64 {
    if gamma <= 0.0 {
        if gains_kwh > 0.0 {
            1.0 / gamma
        } else {
            1.0
        }
    } else if (gamma - 1.0).abs() < 1e-9 {
        a / (a + 1.0)
    } else {
        (1.0 - gamma.powf(a)) / (1.0 - gamma.powf(a + 1.0))
    }
}

/// 7.52–7.54: loss utilisation for cooling, with `γ_C = Q_C;gn / Q_C;ht`.
pub fn cooling_utilization(gamma: f64, a: f64) -> f64 {
    if gamma <= 0.0 {
        1.0
    } else if (gamma - 1.0).abs() < 1e-9 {
        a / (a + 1.0)
    } else {
        (1.0 - gamma.powf(-a)) / (1.0 - gamma.powf(-(a + 1.0)))
    }
}

/// 7.74/7.75: reduction factor for intermittent cooling.
pub fn cooling_reduction_factor(function: UsageFunction) -> f64 {
    cooling_reduction_from_hours(function.reduction_hours().2)
}

/// 7.74/7.75 for `t_C;red;wknd` in hours.
pub fn cooling_reduction_from_hours(weekend_hours: f64) -> f64 {
    let fraction = weekend_hours / (24.0 * 7.0);
    1.0 - fraction + B_C_RED_WKND * fraction
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageFunctionArea {
    pub function: UsageFunction,
    pub area_m2: f64,
}

/// The calculation values of §6.5.3 for a zone: those of its function, or
/// the area-weighted values of several functions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionProfile {
    pub heating_setpoint_c: f64,
    pub cooling_setpoint_c: f64,
    pub reduced_setpoint_c: f64,
    pub day_reduction_h: f64,
    pub weekend_reduction_h: f64,
    pub cooling_weekend_reduction_h: f64,
    /// q_Oc·f_τ + q_A (tables 7.2/7.3), W/m².
    pub occupancy_appliance_w_per_m2: f64,
    /// §5.4.2 q_L, W/m².
    pub fixed_lighting_w_per_m2: f64,
}

impl FunctionProfile {
    pub fn of(function: UsageFunction) -> Self {
        let (day, weekend, cooling_weekend) = function.reduction_hours();
        Self {
            heating_setpoint_c: function.heating_setpoint_c(),
            cooling_setpoint_c: function.cooling_setpoint_c(),
            reduced_setpoint_c: function.reduced_setpoint_c(),
            day_reduction_h: day,
            weekend_reduction_h: weekend,
            cooling_weekend_reduction_h: cooling_weekend,
            occupancy_appliance_w_per_m2: function.occupancy_and_appliance_flux_w_per_m2(),
            fixed_lighting_w_per_m2: function.fixed_lighting_flux_w_per_m2(),
        }
    }

    /// §6.5.3: weighted by usable floor area.
    pub fn weighted(parts: &[UsageFunctionArea]) -> Self {
        let total: f64 = parts.iter().map(|part| part.area_m2).sum();
        let mean = |value: fn(&FunctionProfile) -> f64| {
            parts
                .iter()
                .map(|part| value(&Self::of(part.function)) * part.area_m2)
                .sum::<f64>()
                / total
        };
        Self {
            heating_setpoint_c: mean(|p| p.heating_setpoint_c),
            cooling_setpoint_c: mean(|p| p.cooling_setpoint_c),
            reduced_setpoint_c: mean(|p| p.reduced_setpoint_c),
            day_reduction_h: mean(|p| p.day_reduction_h),
            weekend_reduction_h: mean(|p| p.weekend_reduction_h),
            cooling_weekend_reduction_h: mean(|p| p.cooling_weekend_reduction_h),
            occupancy_appliance_w_per_m2: mean(|p| p.occupancy_appliance_w_per_m2),
            fixed_lighting_w_per_m2: mean(|p| p.fixed_lighting_w_per_m2),
        }
    }

    pub fn cooling_reduction_factor(&self) -> f64 {
        cooling_reduction_from_hours(self.cooling_weekend_reduction_h)
    }
}

/// The §6.5.3 profile of a demand input.
pub fn function_profile(input: &MonthlyDemandInput) -> FunctionProfile {
    let mut profile = if input.function_areas.is_empty() {
        FunctionProfile::of(input.usage_function)
    } else {
        FunctionProfile::weighted(&input.function_areas)
    };
    // Maatwerkadvies: the actual use replaces tables 7.2/7.3 and 7.13–7.15.
    if let Some(fit) = &input.usage_fit {
        profile.heating_setpoint_c = input.setpoints.heating_c;
        profile.cooling_setpoint_c = input.setpoints.cooling_c;
        if let Some(value) = fit.reduced_setpoint_c {
            profile.reduced_setpoint_c = value;
        }
        if let Some(value) = fit.day_reduction_h {
            profile.day_reduction_h = value;
        }
        if let Some(value) = fit.weekend_reduction_h {
            profile.weekend_reduction_h = value;
        }
        if let Some(value) = fit.cooling_weekend_reduction_h {
            profile.cooling_weekend_reduction_h = value;
        }
        if let Some(value) = fit.occupancy_appliance_w_per_m2 {
            profile.occupancy_appliance_w_per_m2 = value;
        }
    }
    profile
}

fn check_usage_fit(fit: &UsageFit, residential: bool, issues: &mut Vec<DemandIssue>) {
    let bad = |value: Option<f64>, min: f64, max: f64| {
        value.is_some_and(|value| !value.is_finite() || value < min || value > max)
    };
    for (value, min, max, path) in [
        (
            fit.reduced_setpoint_c,
            -10.0,
            40.0,
            "usageFit.reducedSetpointC",
        ),
        (fit.day_reduction_h, 0.0, 24.0, "usageFit.dayReductionH"),
        (
            fit.weekend_reduction_h,
            0.0,
            48.0,
            "usageFit.weekendReductionH",
        ),
        (
            fit.cooling_weekend_reduction_h,
            0.0,
            48.0,
            "usageFit.coolingWeekendReductionH",
        ),
        (fit.spatial_fraction, 0.0, 1.0, "usageFit.spatialFraction"),
        (fit.occupants, 0.0, 1.0e6, "usageFit.occupants"),
        (
            fit.internal_gain_per_person_w,
            0.0,
            2000.0,
            "usageFit.internalGainPerPersonW",
        ),
        (
            fit.occupancy_appliance_w_per_m2,
            0.0,
            500.0,
            "usageFit.occupancyApplianceWPerM2",
        ),
    ] {
        if bad(value, min, max) {
            issues.push(issue("usage_fit_value_invalid", path));
        }
    }
    let residential_only = fit.spatial_fraction.is_some()
        || fit.occupants.is_some()
        || fit.internal_gain_per_person_w.is_some();
    if residential_only && !residential {
        issues.push(issue("usage_fit_residential_only", "usageFit"));
    }
    if fit.occupancy_appliance_w_per_m2.is_some() && residential {
        issues.push(issue(
            "usage_fit_utility_only",
            "usageFit.occupancyApplianceWPerM2",
        ));
    }
    if fit.source_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            "usageFit.sourceReference",
        ));
    }
}

/// 7.20: supply-temperature correction `b_v` against the standard setpoint;
/// 1 when the setpoint equals the outdoor temperature (no defined ratio).
pub fn supply_temperature_factor(setpoint_c: f64, supply_c: f64, outdoor_c: f64) -> f64 {
    let denominator = setpoint_c - outdoor_c;
    if denominator.abs() < 1e-9 {
        1.0
    } else {
        (setpoint_c - supply_c) / denominator
    }
}

/// `H_H/C;ve;mi` of 7.19 (W/K, with `b_v` of 7.20) for one month and balance.
pub(crate) fn ventilation_conductance(
    input: &MonthlyDemandInput,
    month: u8,
    balance: Balance,
) -> f64 {
    let outdoor = OUTDOOR_TEMPERATURE_C[usize::from(month - 1)];
    let setpoint = match balance {
        Balance::Heating => input.setpoints.heating_c,
        Balance::Cooling => input.setpoints.cooling_c,
    };
    input
        .ventilation_flows
        .iter()
        .map(|flow| {
            let row = flow
                .months
                .iter()
                .find(|row| row.month == month)
                .expect("validated twelve unique months");
            let (conductance, supply) = row.balance(balance, outdoor);
            conductance * supply_temperature_factor(setpoint, supply, outdoor)
        })
        .sum()
}

/// Intermittency for one heating month (7.59–7.73).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatingIntermittency {
    /// `a_H;red` (7.60).
    pub reduction_factor: f64,
    /// `θ_int;calc;H` (7.59), °C.
    pub calculation_temperature_c: f64,
}

/// Inputs of 7.60–7.73 for one month.
#[derive(Debug, Clone, Copy)]
pub struct IntermittencyInput {
    pub profile: FunctionProfile,
    /// `θ_int;set;H` after levelling (7.76), °C.
    pub setpoint_c: f64,
    pub outdoor_c: f64,
    pub annual_outdoor_c: f64,
    pub hours: f64,
    /// `τ_H` (7.57), h.
    pub time_constant_h: f64,
    /// `Q_H;gn`, kWh.
    pub gains_kwh: f64,
    /// `H_H;tr;excl.gf + H_H;ve`, W/K.
    pub conductance_w_per_k: f64,
    /// `H_gr;an;mi`, W/K.
    pub ground_conductance_w_per_k: f64,
}

/// 7.64/7.65 with 7.66–7.72 for one interruption period of `period_h`.
fn mean_reduction(period_h: f64, input: &IntermittencyInput, d_float: f64) -> f64 {
    if period_h <= 0.0 {
        return 0.0;
    }
    let difference = input.setpoint_c - input.outdoor_c;
    let low = input.profile.reduced_setpoint_c;
    // 7.70–7.72
    let d_set = if difference <= 0.0 {
        1.0
    } else if low - input.outdoor_c <= 0.0 {
        0.0
    } else {
        ((low - input.outdoor_c) / difference).min(1.0)
    };
    let tau = input.time_constant_h;
    let period_ratio = period_h / tau;
    // 7.66–7.69
    let f_low = if d_set - d_float <= 0.0 {
        1.0
    } else if d_float >= 1.0 {
        0.0
    } else {
        let low_ratio = -((d_set - d_float) / (1.0 - d_float)).ln();
        low_ratio / period_ratio
    };
    if f_low >= 1.0 {
        // 7.64
        d_float + (1.0 - d_float) / period_ratio * (1.0 - (-period_ratio).exp())
    } else {
        // 7.65
        (1.0 - d_set) / period_ratio + f_low * d_float + (1.0 - f_low) * d_set
    }
}

/// 7.59–7.73: reduction factor and calculation temperature for heating.
pub fn heating_intermittency(input: &IntermittencyInput) -> HeatingIntermittency {
    let (day_h, weekend_h) = (
        input.profile.day_reduction_h,
        input.profile.weekend_reduction_h,
    );
    let difference = input.setpoint_c - input.outdoor_c;
    // 7.73, clamped to [0, 1].
    let d_float = if difference <= 0.0 {
        1.0
    } else {
        let losses = (input.conductance_w_per_k * difference
            + input.ground_conductance_w_per_k * (input.setpoint_c - input.annual_outdoor_c))
            * input.hours;
        if losses <= 0.0 {
            1.0
        } else {
            (input.gains_kwh * 1000.0 / losses).clamp(0.0, 1.0)
        }
    };
    // 7.62/7.63
    let f_day = day_h * (7.0 - weekend_h / 24.0) / (24.0 * 7.0);
    let f_weekend = weekend_h / (24.0 * 7.0);
    // 7.61
    let a_day = 1.0 - f_day + f_day * mean_reduction(day_h, input, d_float);
    let a_weekend = 1.0 - f_weekend + f_weekend * mean_reduction(weekend_h, input, d_float);
    // 7.60
    let reduction_factor = 1.0 - (1.0 - a_day) - (1.0 - a_weekend);
    HeatingIntermittency {
        reduction_factor,
        // 7.59
        calculation_temperature_c: reduction_factor * difference + input.outdoor_c,
    }
}

/// 7.78/7.79: setpoint lowering by temperature levelling in dwellings, K.
pub fn levelling_reduction_k(
    dwelling: DwellingType,
    specific_conductance_w_per_m2k: f64,
    standard_setpoint_c: f64,
    outdoor_c: f64,
) -> f64 {
    levelling_reduction_with_fraction_k(
        dwelling.spatial_fraction(),
        specific_conductance_w_per_m2k,
        standard_setpoint_c,
        outdoor_c,
    )
}

/// 7.78/7.79 with an explicit `f_mod;sp` (maatwerkadvies fit).
pub fn levelling_reduction_with_fraction_k(
    f_sp: f64,
    specific_conductance_w_per_m2k: f64,
    standard_setpoint_c: f64,
    outdoor_c: f64,
) -> f64 {
    let h_e = f_sp * specific_conductance_w_per_m2k;
    (F_MOD_T * f_sp) * h_e * (standard_setpoint_c - outdoor_c) / (h_e + H_INT_SPEC)
}

/// 7.39: sky radiation loss of one envelope element in kWh.
fn sky_loss_kwh(tilt_deg: f64, u: f64, area: f64, hours: f64) -> f64 {
    sky_view_factor(tilt_deg) * R_SE * u * area * H_LR_E * DELTA_THETA_SKY * hours * 0.001
}

/// 7.32 with 7.40, 7.42/7.43 and 7.39: net solar gain of one window in kWh.
pub(crate) fn window_solar_kwh(window: &Window, month: u8, balance: Balance) -> f64 {
    let hours = MONTH_HOURS[usize::from(month - 1)];
    let irradiance = climate::irradiance_w_per_m2(window.orientation, window.tilt_deg, month)
        .expect("validated tilt");
    let obstruction = obstruction_factor(
        &window.obstruction,
        window.orientation,
        window.tilt_deg,
        month,
        balance,
    )
    .expect("validated obstruction");
    let shading = movable_shading_factor(
        window.movable_shading.as_ref(),
        window.orientation,
        window.tilt_deg,
        month,
        balance,
    );
    let index = usize::from(month - 1);
    F_W * window.g_for_month(index)
        * window.area_m2
        * (1.0 - window.frame_fraction)
        * obstruction
        * shading
        * irradiance
        * hours
        * 0.001
        - sky_loss_kwh(
            window.tilt_deg,
            window.u_for_month(index),
            window.area_m2,
            hours,
        )
}

/// 7.33 and 7.39: net solar gain of one opaque element in kWh.
pub(crate) fn opaque_solar_kwh(element: &OpaqueElement, month: u8) -> f64 {
    let hours = MONTH_HOURS[usize::from(month - 1)];
    let irradiance = climate::irradiance_w_per_m2(element.orientation, element.tilt_deg, month)
        .expect("validated tilt");
    ALPHA_SOL * R_SE * element.u_value_w_per_m2k * element.area_m2 * irradiance * hours * 0.001
        - sky_loss_kwh(
            element.tilt_deg,
            element.u_value_w_per_m2k,
            element.area_m2,
            hours,
        )
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
    if !(0.0..=180.0).contains(&tilt) {
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
    let function = input.usage_function;
    if !input.function_areas.is_empty() {
        let total: f64 = input.function_areas.iter().map(|part| part.area_m2).sum();
        if input
            .function_areas
            .iter()
            .any(|part| !(part.area_m2.is_finite() && part.area_m2 > 0.0))
            || (total - input.usable_floor_area_m2).abs() > 0.01 * input.usable_floor_area_m2
        {
            issues.push(issue("function_areas_invalid", "functionAreas"));
        }
        // §6.5.2: no other function next to the residential function.
        if input
            .function_areas
            .iter()
            .any(|part| part.function.is_residential())
        {
            issues.push(issue("function_areas_residential_mixed", "functionAreas"));
        }
        let largest = input
            .function_areas
            .iter()
            .max_by(|a, b| a.area_m2.total_cmp(&b.area_m2))
            .map(|part| part.function);
        if largest != Some(function) {
            issues.push(issue("usage_function_not_largest", "usageFunction"));
        }
    }
    if let Some(fit) = &input.usage_fit {
        check_usage_fit(fit, function.is_residential(), issues);
    }
    let profile = function_profile(input);
    if (setpoints.heating_c - profile.heating_setpoint_c).abs() > 1e-9
        || (setpoints.cooling_c - profile.cooling_setpoint_c).abs() > 1e-9
    {
        issues.push(issue("setpoints_table_7_13_mismatch", "setpoints"));
    }
    match (function.is_residential(), input.dwelling_type) {
        (true, None) => issues.push(issue("dwelling_type_required", "dwellingType")),
        (false, Some(_)) => issues.push(issue("dwelling_type_not_applicable", "dwellingType")),
        _ => {}
    }
    if matches!(input.internal_gains, InternalGains::Residential { .. })
        && !function.is_residential()
    {
        issues.push(issue(
            "internal_gains_function_mismatch",
            "internalGains.method",
        ));
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
                .cooling_conductance_w_per_k
                .is_some_and(|value| !finite_nonneg(value))
            {
                issues.push(issue(
                    "ventilation_conductance_invalid",
                    format!("{row_path}.coolingConductanceWPerK"),
                ));
            }
            for (field, value) in [
                ("supplyTemperatureC", row.supply_temperature_c),
                (
                    "coolingSupplyTemperatureC",
                    row.cooling_supply_temperature_c,
                ),
            ] {
                if value.is_some_and(|value| !value.is_finite()) {
                    issues.push(issue(
                        "ventilation_supply_temperature_invalid",
                        format!("{row_path}.{field}"),
                    ));
                }
            }
        }
    }

    for (index, room) in input.sunrooms.iter().enumerate() {
        let path = format!("sunrooms[{index}]");
        let fractions_ok = [
            room.glazing_g_heating,
            room.glazing_g_cooling,
            room.exterior_frame_fraction,
            room.reduction_factor,
            room.distribution_factor,
        ]
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value));
        if !fractions_ok || !finite_nonneg(room.zone_conductance_w_per_k) {
            issues.push(issue("sunroom_invalid", path.clone()));
        }
        if room.surfaces.iter().any(|surface| {
            !(surface.area_m2.is_finite()
                && surface.area_m2 > 0.0
                && (0.0..=1.0).contains(&surface.absorptance)
                && (0.0..=180.0).contains(&surface.tilt_deg)
                && surface.azimuth_deg.is_finite())
        }) {
            issues.push(issue("sunroom_surface_invalid", format!("{path}.surfaces")));
        }
        check_reference(
            &room.source_reference,
            format!("{path}.sourceReference"),
            issues,
        );
    }
    check_reference(
        &input.thermal_mass.source_reference,
        "thermalMass.sourceReference".into(),
        issues,
    );
    issues.extend(
        crate::annex_b::validate_mass_elements(
            &input.thermal_mass.annex_b_elements,
            "thermalMass.annexBElements",
        )
        .into_iter()
        .map(|item| issue(item.code, item.path)),
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
        InternalGains::Utility {
            lighting,
            hot_water_recoverable_kwh,
            source_reference,
        } => {
            if input.usage_function.is_residential() {
                issues.push(issue(
                    "internal_gains_function_mismatch",
                    "internalGains.method",
                ));
            }
            match lighting {
                UtilityLighting::Chapter14 => issues.push(issue(
                    "lighting_gain_requires_chapter_14",
                    "internalGains.lighting",
                )),
                UtilityLighting::Declared { annual_kwh, .. } => {
                    if !finite_nonneg(*annual_kwh) {
                        issues.push(issue(
                            "lighting_energy_invalid",
                            "internalGains.lighting.annualKwh",
                        ));
                    }
                }
                UtilityLighting::Resolved { gain_w } => {
                    if !finite_nonneg(*gain_w) {
                        issues.push(issue(
                            "lighting_energy_invalid",
                            "internalGains.lighting.gainW",
                        ));
                    }
                }
            }
            if !(hot_water_recoverable_kwh.is_empty() || hot_water_recoverable_kwh.len() == 12)
                || hot_water_recoverable_kwh
                    .iter()
                    .any(|value| !finite_nonneg(*value))
            {
                issues.push(issue(
                    "hot_water_recoverable_invalid",
                    "internalGains.hotWaterRecoverableKwh",
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
        if let Some(dynamic) = &window.dynamic {
            issues.extend(
                dynamic
                    .validate(&format!("{path}.dynamic"))
                    .into_iter()
                    .map(|item| issue(item.code, item.path)),
            );
        }
        if !window.u_value_w_per_m2k.is_finite() || window.u_value_w_per_m2k <= 0.0 {
            issues.push(issue(
                "element_u_value_invalid",
                format!("{path}.uValueWPerM2k"),
            ));
        }
        for (code, suffix) in validate_obstruction(&window.obstruction, window.tilt_deg) {
            issues.push(issue(code, format!("{path}.obstruction{suffix}")));
        }
        if let Obstruction::Declared {
            source_reference, ..
        } = &window.obstruction
        {
            check_reference(
                source_reference,
                format!("{path}.obstruction.sourceReference"),
                issues,
            );
        }
        if let Some(shading) = &window.movable_shading {
            if !(0.0..=1.0).contains(&shading.reduction_factor) {
                issues.push(issue(
                    "window_shading_factor_invalid",
                    format!("{path}.movableShading.reductionFactor"),
                ));
            }
            if !shading.control.fits_function(function.is_residential()) {
                issues.push(issue(
                    "window_shading_control_function_mismatch",
                    format!("{path}.movableShading.control"),
                ));
            }
            check_reference(
                &shading.source_reference,
                format!("{path}.movableShading.sourceReference"),
                issues,
            );
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

/// Coefficients and ground terms after resolving either transmission route.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransmissionSummary {
    pub method: &'static str,
    /// `H_tr` excluding ground, W/K.
    pub conductance_w_per_k: f64,
    pub direct_conductance_w_per_k: Option<f64>,
    pub unheated_conductance_w_per_k: Option<f64>,
    /// 7.3.3 `H_p` (components route), W/K.
    pub vertical_pipe_conductance_w_per_k: Option<f64>,
    /// Steady `H_g` (8.36/8.37) summed over the slabs (components route), W/K.
    pub ground_steady_conductance_w_per_k: Option<f64>,
    /// Annex D.1 `H_g;an;mi`, W/K.
    pub ground_monthly_conductance_w_per_k: [f64; 12],
    /// Annex D.2 `H_H;g;adj`, W/K.
    pub ground_heating_adjusted_w_per_k: f64,
    /// Annex D.3 `H_C;g;adj`, W/K.
    pub ground_cooling_adjusted_w_per_k: f64,
    /// `θ_e;avg;an` used in 7.14/7.15 and D.1–D.3 (mean of table 17.1), °C.
    pub annual_mean_outdoor_temperature_c: f64,
}

impl TransmissionSummary {
    /// 7.14/7.15 ground term in kWh at calculation temperature `theta_c`.
    pub(crate) fn ground_kwh(&self, month: u8, theta_c: f64) -> f64 {
        let index = usize::from(month - 1);
        self.ground_monthly_conductance_w_per_k[index]
            * (theta_c - self.annual_mean_outdoor_temperature_c)
            * MONTH_HOURS[index]
            / 1000.0
    }
}

pub fn annual_mean_outdoor_temperature_c() -> f64 {
    OUTDOOR_TEMPERATURE_C.iter().sum::<f64>() / 12.0
}

fn resolve_transmission(
    input: &MonthlyDemandInput,
    issues: &mut Vec<DemandIssue>,
) -> Option<TransmissionSummary> {
    let prior = issues.len();
    let annual_mean = annual_mean_outdoor_temperature_c();
    match &input.transmission {
        Transmission::Explicit(transmission) => {
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
            let mut monthly = [0.0; 12];
            let mut heating_adjusted = 0.0;
            let mut cooling_adjusted = 0.0;
            if let Some(ground) = &transmission.ground {
                for (name, value) in [
                    (
                        "heatingAdjustedConductanceWPerK",
                        ground.heating_adjusted_conductance_w_per_k,
                    ),
                    (
                        "coolingAdjustedConductanceWPerK",
                        ground.cooling_adjusted_conductance_w_per_k,
                    ),
                ] {
                    if !finite_nonneg(value) {
                        issues.push(issue(
                            "ground_conductance_invalid",
                            format!("transmission.ground.{name}"),
                        ));
                    }
                }
                let values = &ground.monthly_conductance_w_per_k;
                if values.len() != 12 || values.iter().any(|value| !value.is_finite()) {
                    issues.push(issue(
                        "ground_monthly_invalid",
                        "transmission.ground.monthlyConductanceWPerK",
                    ));
                } else {
                    monthly.copy_from_slice(values);
                }
                check_reference(
                    &ground.source_reference,
                    "transmission.ground.sourceReference".into(),
                    issues,
                );
                heating_adjusted = ground.heating_adjusted_conductance_w_per_k;
                cooling_adjusted = ground.cooling_adjusted_conductance_w_per_k;
            }
            (issues.len() == prior).then_some(TransmissionSummary {
                method: "explicit",
                conductance_w_per_k: transmission.conductance_w_per_k,
                direct_conductance_w_per_k: None,
                unheated_conductance_w_per_k: None,
                vertical_pipe_conductance_w_per_k: None,
                ground_steady_conductance_w_per_k: None,
                ground_monthly_conductance_w_per_k: monthly,
                ground_heating_adjusted_w_per_k: heating_adjusted,
                ground_cooling_adjusted_w_per_k: cooling_adjusted,
                annual_mean_outdoor_temperature_c: annual_mean,
            })
        }
        Transmission::Components(components) => {
            let direct = assess_direct_transmission(&components.direct);
            issues.extend(
                direct
                    .issues
                    .iter()
                    .map(|item| issue(item.code, format!("transmission.direct.{}", item.path))),
            );
            let unheated = components
                .unheated
                .as_ref()
                .map(assess_unheated_transmission);
            if let Some(unheated) = &unheated {
                issues.extend(
                    unheated.issues.iter().map(|item| {
                        issue(item.code, format!("transmission.unheated.{}", item.path))
                    }),
                );
            }
            if !components.ground_inventory_confirmed {
                issues.push(issue(
                    "ground_inventory_unconfirmed",
                    "transmission.groundInventoryConfirmed",
                ));
            }
            let mut pipe_ids = HashSet::new();
            for (index, pipe) in components.vertical_pipes.iter().enumerate() {
                let path = format!("transmission.verticalPipes[{index}]");
                if pipe.id.trim().is_empty() || !pipe_ids.insert(pipe.id.as_str()) {
                    issues.push(issue("vertical_pipe_id_invalid", format!("{path}.id")));
                }
                if pipe.storeys == 0 {
                    issues.push(issue(
                        "vertical_pipe_storeys_invalid",
                        format!("{path}.storeys"),
                    ));
                }
                if pipe.shared_zones == 0 {
                    issues.push(issue(
                        "vertical_pipe_shared_zones_invalid",
                        format!("{path}.sharedZones"),
                    ));
                }
                check_reference(
                    &pipe.source_reference,
                    format!("{path}.sourceReference"),
                    issues,
                );
            }
            let mut slabs = Vec::new();
            let mut ids = HashSet::new();
            for (index, slab) in components.ground_floors.iter().enumerate() {
                let path = format!("transmission.groundFloors[{index}]");
                if slab.id.trim().is_empty() || !ids.insert(slab.id.as_str()) {
                    issues.push(issue("ground_floor_id_invalid", format!("{path}.id")));
                }
                check_reference(
                    &slab.source_reference,
                    format!("{path}.sourceReference"),
                    issues,
                );
                if let EdgeThermalBridges::Detailed { bridges } = &slab.edge_thermal_bridges {
                    for (bridge_index, bridge) in bridges.iter().enumerate() {
                        check_reference(
                            &bridge.source_reference,
                            format!(
                                "{path}.edgeThermalBridges.bridges[{bridge_index}].sourceReference"
                            ),
                            issues,
                        );
                    }
                }
                for (insulation_index, insulation) in slab.edge_insulation.iter().enumerate() {
                    check_reference(
                        &insulation.source_reference,
                        format!("{path}.edgeInsulation[{insulation_index}].sourceReference"),
                        issues,
                    );
                }
                match slab_coefficients(slab) {
                    Some(value) => slabs.push(value),
                    None => issues.push(issue("ground_floor_invalid", path)),
                }
            }
            if issues.len() != prior {
                return None;
            }
            let direct_conductance = direct.total_direct_conductance_w_per_k?;
            let unheated_conductance = match &unheated {
                Some(result) => Some(result.total_reduced_conductance_w_per_k?),
                None => None,
            };
            let ground = ground_monthly(
                &slabs,
                input.setpoints.heating_c,
                input.setpoints.cooling_c,
                annual_mean,
            );
            // 7.16: H_p adds to the transfer to outdoor air.
            let pipes = vertical_pipe_conductance_w_per_k(&components.vertical_pipes);
            Some(TransmissionSummary {
                method: "components",
                conductance_w_per_k: direct_conductance
                    + unheated_conductance.unwrap_or(0.0)
                    + pipes,
                direct_conductance_w_per_k: Some(direct_conductance),
                unheated_conductance_w_per_k: unheated_conductance,
                vertical_pipe_conductance_w_per_k: Some(pipes),
                ground_steady_conductance_w_per_k: Some(
                    slabs.iter().map(|slab| slab.steady_w_per_k).sum(),
                ),
                ground_monthly_conductance_w_per_k: ground.monthly_w_per_k,
                ground_heating_adjusted_w_per_k: ground.heating_adjusted_w_per_k,
                ground_cooling_adjusted_w_per_k: ground.cooling_adjusted_w_per_k,
                annual_mean_outdoor_temperature_c: annual_mean,
            })
        }
    }
}

/// 7.1–7.3 with `extra_kwh` added to Q_H;ht, the gains and `a` unchanged
/// (9.28/9.29), without recoverable losses.
pub fn heating_need_with_extra_transfer(terms: &BalanceTerms, extra_kwh: f64) -> f64 {
    let transfer = terms.heat_transfer_kwh + extra_kwh;
    if transfer == 0.0 {
        return 0.0;
    }
    let gamma = terms.gains_kwh / transfer;
    if (gamma <= 0.0 && terms.gains_kwh > 0.0) || gamma > GAMMA_H_MAX {
        return 0.0;
    }
    let eta = heating_utilization(gamma, terms.a, terms.gains_kwh);
    (transfer - eta * terms.gains_kwh).max(0.0)
}

/// 7.6/7.7 with `extra_kwh` added to Q_C;ht, the gains, `a` and `a_C;red`
/// unchanged (10.19/10.20), without recoverable losses.
pub fn cooling_need_with_extra_transfer(terms: &BalanceTerms, extra_kwh: f64) -> f64 {
    let transfer = terms.heat_transfer_kwh + extra_kwh;
    let gamma = (transfer != 0.0).then(|| terms.gains_kwh / transfer);
    if terms.gains_kwh <= 0.0 || gamma.is_some_and(|gamma| gamma > 0.0 && 1.0 / gamma > 2.0) {
        return 0.0;
    }
    let eta = gamma.map_or(1.0, |gamma| cooling_utilization(gamma, terms.a));
    (terms.reduction_factor * (terms.gains_kwh - eta * transfer)).max(0.0)
}

/// Absorbing opaque surface inside an adjacent unheated sunroom (7.34).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SunroomSurface {
    pub area_m2: f64,
    /// α_sol (7.6.6.3).
    pub absorptance: f64,
    /// 0 = north, clockwise.
    pub azimuth_deg: f64,
    pub tilt_deg: f64,
}

/// Adjacent unheated sunroom (AOS) for 7.30b and 7.34–7.38.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sunroom {
    pub id: String,
    /// g_gl;ue of the sunroom glazing for the heating and cooling balance
    /// (7.35; area-weighted, including seasonal shading).
    pub glazing_g_heating: f64,
    pub glazing_g_cooling: f64,
    /// F_fr;ue of the sunroom envelope (7.35a).
    pub exterior_frame_fraction: f64,
    /// b_U of the sunroom (8.4.1).
    pub reduction_factor: f64,
    /// H_zi;ztu between this zone and the sunroom (8.4.1), W/K.
    pub zone_conductance_w_per_k: f64,
    /// F_zi;ztu (7.36): this zone's share when several zones border it.
    #[serde(default = "one_f64")]
    pub distribution_factor: f64,
    pub surfaces: Vec<SunroomSurface>,
    pub source_reference: String,
}

fn one_f64() -> f64 {
    1.0
}

/// 7.30b: the indirect sunroom gains of a zone for one balance, kWh.
pub fn sunroom_gains_kwh(input: &MonthlyDemandInput, month_index: usize, balance: Balance) -> f64 {
    let month = month_index as u8 + 1;
    let hours = MONTH_HOURS[month_index];
    input
        .sunrooms
        .iter()
        .map(|room| {
            let g = match balance {
                Balance::Heating => room.glazing_g_heating,
                Balance::Cooling => room.glazing_g_cooling,
            };
            // 7.35 and 7.34 with F_sh;obst = 1.
            let f_sol = g * (1.0 - room.exterior_frame_fraction);
            let absorbed: f64 = room
                .surfaces
                .iter()
                .map(|surface| {
                    surface.absorptance
                        * surface.area_m2
                        * climate::irradiance_at(surface.azimuth_deg, surface.tilt_deg, month)
                            .unwrap_or(0.0)
                })
                .sum();
            let gains = f_sol * absorbed * 0.001 * hours;
            // 7.37: heating only, capped at 1.
            let cap = match balance {
                Balance::Heating if gains > 0.0 => (room.reduction_factor
                    * room.zone_conductance_w_per_k
                    * (input.setpoints.heating_c - OUTDOOR_TEMPERATURE_C[month_index])
                    * 0.001
                    * hours
                    / gains)
                    .clamp(0.0, 1.0),
                _ => 1.0,
            };
            (1.0 - room.reduction_factor) * room.distribution_factor * cap * gains
        })
        .sum()
}

/// Area-weighted (f_τ, q_A) of tables 7.2/7.3 for a zone (§6.5.3).
pub fn occupancy_time_and_appliances(input: &MonthlyDemandInput) -> (f64, f64) {
    if input.function_areas.is_empty() {
        let (_, f_tau, q_a) = input.usage_function.occupancy_table();
        return (f_tau, q_a);
    }
    let total: f64 = input.function_areas.iter().map(|part| part.area_m2).sum();
    if total <= 0.0 {
        return (0.0, 0.0);
    }
    input
        .function_areas
        .iter()
        .fold((0.0, 0.0), |(f_tau, q_a), part| {
            let (_, f, q) = part.function.occupancy_table();
            let share = part.area_m2 / total;
            (f_tau + share * f, q_a + share * q)
        })
}

/// Monthly internal gains Q_int (7.21–7.29), kWh; `month_index` 0–11.
pub fn internal_gains_kwh(input: &MonthlyDemandInput, month_index: usize) -> f64 {
    let area = input.usable_floor_area_m2;
    let hours = MONTH_HOURS[month_index];
    match &input.internal_gains {
        InternalGains::Residential { dwelling_count, .. } => {
            let dwellings = f64::from(*dwelling_count);
            let fit = input.usage_fit.as_ref();
            let occupants = fit
                .and_then(|fit| fit.occupants)
                .unwrap_or_else(|| dwellings * occupants_per_dwelling(area / dwellings));
            let per_person = fit
                .and_then(|fit| fit.internal_gain_per_person_w)
                .unwrap_or(INTERNAL_HEAT_PER_OCCUPANT_W);
            per_person * occupants * 0.001 * hours
        }
        InternalGains::Utility {
            lighting,
            hot_water_recoverable_kwh,
            ..
        } => {
            // 7.25–7.29; Φ_V and Φ_proc are 0 (7.5.3.5/7.5.3.6).
            let persons_and_appliances =
                function_profile(input).occupancy_appliance_w_per_m2 * area;
            let lighting = match lighting {
                UtilityLighting::Chapter14 => 0.0,
                UtilityLighting::Declared {
                    annual_kwh,
                    recovery,
                } => recovery.factor() * annual_kwh * 1000.0 / climate::YEAR_HOURS,
                UtilityLighting::Resolved { gain_w } => *gain_w,
            };
            (persons_and_appliances + lighting) * hours / 1000.0
                + hot_water_recoverable_kwh
                    .get(month_index)
                    .copied()
                    .unwrap_or(0.0)
        }
        InternalGains::Declared {
            heat_flux_w_per_m2, ..
        } => heat_flux_w_per_m2 * area * hours / 1000.0,
    }
}

/// The input with the chapter 11 flows of an assessment filled in, for
/// callers (TOjuli) that read `ventilation_flows` directly.
pub fn with_resolved_ventilation(
    input: &MonthlyDemandInput,
    assessment: &MonthlyDemandAssessment,
) -> MonthlyDemandInput {
    let mut resolved = input.clone();
    if let Some(ventilation) = &assessment.ventilation {
        resolved.ventilation = None;
        resolved.ventilation_flows = ventilation.monthly_demand_flows(CHAPTER_11_SOURCE);
    }
    resolved
}

const CHAPTER_11_SOURCE: &str = "NTA 8800 chapter 11, derived by the kernel";

/// Replace `ventilation` by its chapter 7 flows; returns the resolved input,
/// the chapter 11 result and the C1 flows.
fn resolve_ventilation(
    input: &MonthlyDemandInput,
    issues: &mut Vec<DemandIssue>,
) -> (
    MonthlyDemandInput,
    Option<crate::ventilation::VentilationResult>,
    Option<Vec<VentilationFlow>>,
) {
    let mut resolved = input.clone();
    let Some(ventilation) = input.ventilation.as_ref() else {
        return (resolved, None, None);
    };
    resolved.ventilation = None;
    if !input.ventilation_flows.is_empty() {
        issues.push(issue(
            "ventilation_flows_and_chapter_11_exclusive",
            "ventilationFlows",
        ));
    }
    if (ventilation.usable_floor_area_m2 - input.usable_floor_area_m2).abs() > 1e-6 {
        issues.push(issue(
            "ventilation_area_mismatch",
            "ventilation.usableFloorAreaM2",
        ));
    }
    if (ventilation.heating_setpoint_c - input.setpoints.heating_c).abs() > 1e-9
        || (ventilation.cooling_setpoint_c - input.setpoints.cooling_c).abs() > 1e-9
    {
        issues.push(issue(
            "ventilation_setpoint_mismatch",
            "ventilation.heatingSetpointC",
        ));
    }
    let mut ventilation = ventilation.clone();
    if let Some(practice) = input
        .usage_fit
        .as_ref()
        .and_then(|fit| fit.ventilation_practice.clone())
    {
        ventilation.practice = Some(practice);
    }
    let assessment = crate::ventilation::assess_ventilation(&ventilation);
    issues.extend(
        assessment
            .issues
            .into_iter()
            .map(|item| issue(item.code, format!("ventilation.{}", item.path))),
    );
    let (Some(actual), Some(fixed)) = (assessment.actual, assessment.fixed_c1) else {
        return (resolved, None, None);
    };
    resolved.ventilation_flows = actual.monthly_demand_flows(CHAPTER_11_SOURCE);
    let c1_flows = fixed.monthly_demand_flows(CHAPTER_11_SOURCE);
    (resolved, Some(actual), Some(c1_flows))
}

/// §5.4.2 conditions on an already resolved input.
fn fixed_c1_input(
    resolved: &MonthlyDemandInput,
    flows: Vec<VentilationFlow>,
) -> Result<MonthlyDemandInput, DemandIssue> {
    let mut c1 = resolved.clone();
    c1.ventilation_flows = flows;
    c1.internal_gains = match &resolved.internal_gains {
        InternalGains::Residential { .. } => resolved.internal_gains.clone(),
        InternalGains::Utility {
            source_reference, ..
        } => {
            let profile = function_profile(resolved);
            InternalGains::Declared {
                heat_flux_w_per_m2: profile.occupancy_appliance_w_per_m2
                    + profile.fixed_lighting_w_per_m2,
                source_reference: format!("{source_reference}; §5.4.2 fixed q_L, Φ_int;W = 0"),
            }
        }
        InternalGains::Declared { .. } => {
            return Err(issue(
                "fixed_c1_requires_utility_internal_gain_split",
                "internalGains.method",
            ))
        }
    };
    Ok(c1)
}

pub fn assess_monthly_demand(input: &MonthlyDemandInput) -> MonthlyDemandAssessment {
    let fingerprint =
        input_fingerprint(&serde_json::to_value(input).expect("typed input serializes"));
    let mut issues = Vec::new();
    let (resolved, ventilation, c1_flows) = resolve_ventilation(input, &mut issues);
    let mut assessment = assess_resolved(&resolved, fingerprint, issues);
    if assessment.status == "calculated_unverified" {
        if let Some(result) = &ventilation {
            // 9.28/9.29, read literally: Q_H;ve plus Q_H;ϑHstook;in;air in
            // the heat transfer of 7.2.1; H_ve and τ stay as in 7.4.2.
            assessment.heating_limit_need_kwh = assessment
                .monthly
                .iter()
                .zip(&result.months)
                .map(|(row, month)| {
                    heating_need_with_extra_transfer(&row.heating, month.heating_limit_air_kwh)
                        + month.frost_protection_electricity_kwh
                        + month.grille_preheating_electricity_kwh
                })
                .collect();
        }
        assessment.ventilation = ventilation;
        if let Some(flows) = c1_flows {
            assessment.fixed_c1 = Some(match fixed_c1_input(&resolved, flows) {
                Ok(c1_input) => {
                    let c1 = assess_resolved(&c1_input, String::new(), Vec::new());
                    FixedC1Demand {
                        status: c1.status,
                        monthly_heating_need_kwh: c1
                            .monthly
                            .iter()
                            .map(|row| row.heating.need_kwh)
                            .collect(),
                        monthly_cooling_need_kwh: c1
                            .monthly
                            .iter()
                            .map(|row| row.cooling.need_kwh)
                            .collect(),
                        annual_heating_need_kwh: c1.annual_heating_need_kwh,
                        annual_cooling_need_kwh: c1.annual_cooling_need_kwh,
                        issues: c1.issues,
                    }
                }
                Err(problem) => FixedC1Demand {
                    status: "unavailable",
                    monthly_heating_need_kwh: Vec::new(),
                    monthly_cooling_need_kwh: Vec::new(),
                    annual_heating_need_kwh: None,
                    annual_cooling_need_kwh: None,
                    issues: vec![problem],
                },
            });
        }
    }
    assessment
}

fn assess_resolved(
    input: &MonthlyDemandInput,
    fingerprint: String,
    mut issues: Vec<DemandIssue>,
) -> MonthlyDemandAssessment {
    validate(input, &mut issues);
    let transmission = resolve_transmission(input, &mut issues);

    let mass = &input.thermal_mass;
    let d_m = mass.specific_capacity_kj_per_m2k(input.usable_floor_area_m2);
    let mut monthly = Vec::with_capacity(12);
    if let (true, Some(transmission)) = (issues.is_empty(), &transmission) {
        monthly = compute(input, transmission, d_m, &mut issues);
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
        transmission: transmission.filter(|_| valid),
        monthly,
        annual_heating_need_kwh: annual_heating,
        annual_cooling_need_kwh: annual_cooling,
        recoverable_losses_applied: false,
        annual_heating_need_without_recoverable_kwh: annual_heating,
        annual_cooling_need_without_recoverable_kwh: annual_cooling,
        ventilation: None,
        fixed_c1: None,
        heating_limit_need_kwh: Vec::new(),
        issues,
    }
}

/// 7.3–7.5 (heating) and 7.7–7.9 (cooling): the needs with the recoverable
/// losses of the heating system `Q_H;ls;rbl` (9.2.5) and the cooling system
/// `Q_C;ls;rbl` (10.2), monthly in kWh. The utilisation factors keep the
/// gains without these losses; their effect enters through Δη (7.4/7.8).
/// Not for the BENG 1 run (§5.4.2).
pub fn apply_recoverable_losses(
    assessment: &MonthlyDemandAssessment,
    heating_recoverable_kwh: &[f64],
    cooling_recoverable_kwh: &[f64],
) -> MonthlyDemandAssessment {
    let mut adjusted = assessment.clone();
    if assessment.status != "calculated_unverified" {
        return adjusted;
    }
    for (index, row) in adjusted.monthly.iter_mut().enumerate() {
        let net = heating_recoverable_kwh.get(index).copied().unwrap_or(0.0)
            - cooling_recoverable_kwh.get(index).copied().unwrap_or(0.0);
        let heating = &mut row.heating;
        if let Some(gamma) = heating.gamma {
            let gated = (gamma <= 0.0 && heating.gains_kwh > 0.0) || gamma > GAMMA_H_MAX;
            if !gated {
                let gains_incl = heating.gains_kwh + net;
                let eta_incl = heating_utilization(
                    gains_incl / heating.heat_transfer_kwh,
                    heating.a,
                    gains_incl,
                );
                let delta = eta_incl - heating.utilization;
                heating.need_kwh = (heating.heat_transfer_kwh
                    - heating.utilization * heating.gains_kwh
                    - delta * heating.gains_kwh
                    - heating.utilization * net)
                    .max(0.0);
            }
        }
        let cooling = &mut row.cooling;
        let gated = cooling.gains_kwh <= 0.0
            || cooling
                .gamma
                .is_some_and(|gamma| gamma > 0.0 && 1.0 / gamma > 2.0);
        if !gated {
            let gains_incl = cooling.gains_kwh + net;
            let eta_incl = if cooling.heat_transfer_kwh != 0.0 {
                cooling_utilization(gains_incl / cooling.heat_transfer_kwh, cooling.a)
            } else {
                1.0
            };
            let delta = eta_incl - cooling.utilization;
            cooling.need_kwh = (cooling.reduction_factor
                * (cooling.gains_kwh - cooling.utilization * cooling.heat_transfer_kwh + net
                    - delta * cooling.heat_transfer_kwh))
                .max(0.0);
        }
    }
    adjusted.annual_heating_need_kwh =
        Some(adjusted.monthly.iter().map(|r| r.heating.need_kwh).sum());
    adjusted.annual_cooling_need_kwh =
        Some(adjusted.monthly.iter().map(|r| r.cooling.need_kwh).sum());
    adjusted.recoverable_losses_applied = true;
    adjusted
}

fn compute(
    input: &MonthlyDemandInput,
    transmission: &TransmissionSummary,
    d_m: f64,
    issues: &mut Vec<DemandIssue>,
) -> Vec<MonthResult> {
    let area = input.usable_floor_area_m2;
    let profile = function_profile(input);
    // 7.45: C_m;int;eff in J/K.
    let capacity_j_per_k = d_m * 1000.0 * area;
    let h_tr = transmission.conductance_w_per_k;
    let heating_standard = input.setpoints.heating_c;
    let cooling_setpoint = input.setpoints.cooling_c;
    let annual_outdoor = transmission.annual_mean_outdoor_temperature_c;
    let cooling_reduction = profile.cooling_reduction_factor();
    let mut results = Vec::with_capacity(12);
    for month in 1..=12u8 {
        let index = usize::from(month - 1);
        let hours = MONTH_HOURS[index];
        let outdoor = OUTDOOR_TEMPERATURE_C[index];
        let ground_monthly = transmission.ground_monthly_conductance_w_per_k[index];
        // Annex A: dynamic window U in H_D.
        let h_tr = h_tr + dynamic_window_correction_w_per_k(input, index);

        let internal = internal_gains_kwh(input, index);
        let window_solar: f64 = input
            .windows
            .iter()
            .map(|window| window_solar_kwh(window, month, Balance::Heating))
            .sum();
        let window_solar_cooling: f64 = input
            .windows
            .iter()
            .map(|window| window_solar_kwh(window, month, Balance::Cooling))
            .sum();
        let opaque_solar: f64 = input
            .opaque_elements
            .iter()
            .map(|element| opaque_solar_kwh(element, month))
            .sum();
        // 7.12/7.13 with balance-specific window gains (§17.3, 7.42).
        // 7.30b: indirect gains through adjacent unheated sunrooms.
        let gains = internal
            + window_solar
            + opaque_solar
            + sunroom_gains_kwh(input, index, Balance::Heating);
        let gains_cooling = internal
            + window_solar_cooling
            + opaque_solar
            + sunroom_gains_kwh(input, index, Balance::Cooling);

        // 7.19/7.20 and the time constants 7.57/7.58.
        let h_ve_heating = ventilation_conductance(input, month, Balance::Heating);
        let h_ve_cooling = ventilation_conductance(input, month, Balance::Cooling);
        let conductance_heating =
            h_tr + transmission.ground_heating_adjusted_w_per_k + h_ve_heating;
        let conductance_cooling =
            h_tr + transmission.ground_cooling_adjusted_w_per_k + h_ve_cooling;
        if conductance_heating <= 0.0 || conductance_cooling <= 0.0 {
            issues.push(issue(
                "total_conductance_nonpositive",
                format!("month[{month}]"),
            ));
            return Vec::new();
        }
        let tau_heating = capacity_j_per_k / 3600.0 / conductance_heating;
        let tau_cooling = capacity_j_per_k / 3600.0 / conductance_cooling;
        let a_heating = A_0 + tau_heating / TAU_0_H;
        let a_cooling = A_0 + tau_cooling / TAU_0_H;

        // 7.76–7.79: setpoint after levelling (dwellings only).
        let levelling = match input.dwelling_type {
            Some(dwelling) if input.usage_function.is_residential() => {
                levelling_reduction_with_fraction_k(
                    input
                        .usage_fit
                        .as_ref()
                        .and_then(|fit| fit.spatial_fraction)
                        .unwrap_or_else(|| dwelling.spatial_fraction()),
                    conductance_heating / area,
                    heating_standard,
                    outdoor,
                )
            }
            _ => 0.0,
        };
        let heating_setpoint = heating_standard - levelling;
        // 7.59–7.73
        let intermittency = heating_intermittency(&IntermittencyInput {
            profile,
            setpoint_c: heating_setpoint,
            outdoor_c: outdoor,
            annual_outdoor_c: annual_outdoor,
            hours,
            time_constant_h: tau_heating,
            gains_kwh: gains,
            conductance_w_per_k: h_tr + h_ve_heating,
            ground_conductance_w_per_k: ground_monthly,
        });
        let theta_heating = intermittency.calculation_temperature_c;

        // 7.14/7.15 and 7.18.
        let transmission_heating = h_tr * (theta_heating - outdoor) * hours / 1000.0
            + transmission.ground_kwh(month, theta_heating);
        let transmission_cooling = h_tr * (cooling_setpoint - outdoor) * hours / 1000.0
            + transmission.ground_kwh(month, cooling_setpoint);
        let ventilation_heating = h_ve_heating * (theta_heating - outdoor) * hours / 1000.0;
        let ventilation_cooling = h_ve_cooling * (cooling_setpoint - outdoor) * hours / 1000.0;

        // 7.1–7.3 and 7.46–7.50.
        let heat_transfer_heating = transmission_heating + ventilation_heating;
        let (gamma_heating, eta_heating, need_heating) = if heat_transfer_heating == 0.0 {
            (None, 1.0, 0.0)
        } else {
            let gamma = gains / heat_transfer_heating;
            let eta = heating_utilization(gamma, a_heating, gains);
            let need = if (gamma <= 0.0 && gains > 0.0) || gamma > GAMMA_H_MAX {
                0.0
            } else {
                (heat_transfer_heating - eta * gains).max(0.0)
            };
            (Some(gamma), eta, need)
        };

        // 7.6–7.7 and 7.52–7.55.
        let heat_transfer_cooling = transmission_cooling + ventilation_cooling;
        let gamma_cooling =
            (heat_transfer_cooling != 0.0).then(|| gains_cooling / heat_transfer_cooling);
        let eta_cooling = gamma_cooling.map_or(1.0, |gamma| cooling_utilization(gamma, a_cooling));
        let need_cooling = if gains_cooling <= 0.0
            || gamma_cooling.is_some_and(|gamma| gamma > 0.0 && 1.0 / gamma > 2.0)
        {
            0.0
        } else {
            (cooling_reduction * (gains_cooling - eta_cooling * heat_transfer_cooling)).max(0.0)
        };

        let row = MonthResult {
            month,
            hours,
            outdoor_temperature_c: outdoor,
            internal_gains_kwh: internal,
            window_solar_gains_kwh: window_solar,
            window_solar_cooling_kwh: window_solar_cooling,
            opaque_solar_gains_kwh: opaque_solar,
            ground_conductance_w_per_k: ground_monthly,
            heating: BalanceTerms {
                setpoint_c: heating_setpoint,
                reduction_factor: intermittency.reduction_factor,
                calculation_temperature_c: theta_heating,
                ventilation_conductance_w_per_k: h_ve_heating,
                time_constant_h: tau_heating,
                a: a_heating,
                transmission_kwh: transmission_heating,
                ventilation_kwh: ventilation_heating,
                heat_transfer_kwh: heat_transfer_heating,
                gains_kwh: gains,
                gamma: gamma_heating,
                utilization: eta_heating,
                need_kwh: need_heating,
            },
            cooling: BalanceTerms {
                setpoint_c: cooling_setpoint,
                reduction_factor: cooling_reduction,
                calculation_temperature_c: cooling_setpoint,
                ventilation_conductance_w_per_k: h_ve_cooling,
                time_constant_h: tau_cooling,
                a: a_cooling,
                transmission_kwh: transmission_cooling,
                ventilation_kwh: ventilation_cooling,
                heat_transfer_kwh: heat_transfer_cooling,
                gains_kwh: gains_cooling,
                gamma: gamma_cooling,
                utilization: eta_cooling,
                need_kwh: need_cooling,
            },
        };
        let values = [
            row.heating.need_kwh,
            row.cooling.need_kwh,
            row.heating.heat_transfer_kwh,
            row.cooling.heat_transfer_kwh,
            row.heating.time_constant_h,
            row.cooling.time_constant_h,
            row.heating.calculation_temperature_c,
            row.heating.utilization,
            row.cooling.utilization,
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
            "usageFunction": "residential",
            "dwellingType": "other",
            "setpoints": {"heatingC": 20.0, "coolingC": 24.0, "sourceReference": "table 7.13 residential"},
            "transmission": {
                "method": "explicit",
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
                "obstruction": {"method": "declared", "heating": vec![1.0; 12], "cooling": vec![1.0; 12], "sourceReference": "synthetic: no obstruction"}, "sourceReference": "synthetic window"
            }],
            "opaqueInventoryComplete": true,
            "opaqueElements": [{
                "id": "roof", "areaM2": 50.0, "orientation": "south", "tiltDeg": 0.0,
                "uValueWPerM2k": 0.16, "sourceReference": "synthetic roof"
            }]
        }))
        .unwrap()
    }

    fn office() -> MonthlyDemandInput {
        let mut input = sample();
        input.usage_function = UsageFunction::Office;
        input.dwelling_type = None;
        input.setpoints.heating_c = 21.0;
        input.internal_gains = InternalGains::Declared {
            heat_flux_w_per_m2: 4.0,
            source_reference: "synthetic utility gains".into(),
        };
        input
    }

    fn chapter_11(input: &MonthlyDemandInput, function: &str, variant: &str) -> serde_json::Value {
        let category = if function == "residential" {
            "residential"
        } else {
            "utility"
        };
        json!({
            "zoneId": input.zone_id,
            "usableFloorAreaM2": input.usable_floor_area_m2,
            "category": category,
            "functions": [{"function": function, "areaM2": input.usable_floor_area_m2}],
            "dwellingCount": if category == "residential" { 1 } else { 0 },
            "buildingHeightM": 9.0,
            "constructionYear": 2020,
            "heatingSetpointC": input.setpoints.heating_c,
            "coolingSetpointC": input.setpoints.cooling_c,
            "system": {"kind": "single", "unit": {"variant": variant, "ducts": "luka_a_b_c", "equipmentReference": "synthetic"}},
            "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "synthetic"},
            "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020},
            "sourceReference": "synthetic"
        })
    }

    #[test]
    fn cooling_need_with_extra_transfer_follows_7_6_and_10_19() {
        let result = valid(&sample());
        for row in &result.monthly {
            let terms = &row.cooling;
            let plain = cooling_need_with_extra_transfer(terms, 0.0);
            assert!((plain - terms.need_kwh).abs() < 1e-9);
            // A larger transfer never raises the cooling need.
            assert!(cooling_need_with_extra_transfer(terms, 50.0) <= plain + 1e-9);
        }
        let july = &result.monthly[6].cooling;
        if july.need_kwh > 0.0 {
            let extra = 30.0;
            let transfer = july.heat_transfer_kwh + extra;
            let eta = cooling_utilization(july.gains_kwh / transfer, july.a);
            let expected = (july.reduction_factor * (july.gains_kwh - eta * transfer)).max(0.0);
            assert!((cooling_need_with_extra_transfer(july, extra) - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn chapter_11_route_matches_explicit_flows_and_adds_the_c1_run() {
        let mut input = sample();
        input.ventilation_flows.clear();
        input.ventilation =
            Some(serde_json::from_value(chapter_11(&input, "residential", "d5c")).unwrap());
        let result = valid(&input);
        let ventilation = result.ventilation.as_ref().unwrap();
        let mut explicit = input.clone();
        explicit.ventilation = None;
        explicit.ventilation_flows = ventilation.monthly_demand_flows("test");
        let reference = valid(&explicit);
        for (a, b) in result.monthly.iter().zip(&reference.monthly) {
            assert!((a.heating.need_kwh - b.heating.need_kwh).abs() < 1e-9);
            assert!((a.cooling.need_kwh - b.cooling.need_kwh).abs() < 1e-9);
        }
        // 9.29 literally: Q_air = q·ρc·((θ_SUP − Δθ_fan) − θ_e)·t is 0 here
        // (no heat recovery, no defrost, no duct loss), so the need is equal.
        assert_eq!(result.heating_limit_need_kwh.len(), 12);
        let ventilation = result.ventilation.as_ref().unwrap();
        assert!(ventilation.months[0].heating_limit_air_kwh.abs() < 1e-9);
        assert!(
            (result.heating_limit_need_kwh[0] - result.monthly[0].heating.need_kwh).abs() < 1e-9
        );
        // An extra transfer raises the need through 7.1–7.3 with the same a.
        let jan = &result.monthly[0].heating;
        let extra = heating_need_with_extra_transfer(jan, 100.0);
        let gamma = jan.gains_kwh / (jan.heat_transfer_kwh + 100.0);
        let eta = heating_utilization(gamma, jan.a, jan.gains_kwh);
        assert!((extra - (jan.heat_transfer_kwh + 100.0 - eta * jan.gains_kwh)).abs() < 1e-9);
        // The fixed C1 run ventilates more than demand-controlled D.5c.
        let c1 = result.fixed_c1.as_ref().unwrap();
        assert_eq!(c1.status, "calculated_unverified", "{:?}", c1.issues);
        assert!(c1.annual_heating_need_kwh.unwrap() > result.annual_heating_need_kwh.unwrap());
        // Explicit flows and chapter 11 together are rejected.
        let mut both = input.clone();
        both.ventilation_flows = reference_flows();
        assert!(assess_monthly_demand(&both)
            .issues
            .iter()
            .any(|item| item.code == "ventilation_flows_and_chapter_11_exclusive"));
    }

    fn reference_flows() -> Vec<VentilationFlow> {
        sample().ventilation_flows
    }

    #[test]
    fn annex_b_capacity_replaces_table_7_10() {
        let base = valid(&sample());
        let mut input = sample();
        input.thermal_mass.annex_b_elements = vec![crate::annex_b::MassElement {
            id: "floor".into(),
            area_m2: 100.0,
            layers: vec![crate::annex_b::MassLayer {
                thickness_m: 0.2,
                conductivity_w_per_mk: 2.0,
                density_kg_per_m3: 2400.0,
                specific_heat_j_per_kgk: 1000.0,
                open_suspended_ceiling: false,
            }],
            both_sides: false,
            source_reference: "drawing".into(),
        }];
        let result = valid(&input);
        // 2 400·1 000·0,1·100 J/K over 100 m² = 240 kJ/(m²·K).
        assert!((result.specific_heat_capacity_kj_per_m2k.unwrap() - 240.0).abs() < 1e-9);
        assert_ne!(
            result.specific_heat_capacity_kj_per_m2k,
            base.specific_heat_capacity_kj_per_m2k
        );
    }

    #[test]
    fn nominal_g_is_rounded_down_to_0_05() {
        let mut window = sample().windows[0].clone();
        window.g_perpendicular = 0.63;
        assert!((window.g_for_month(6) - 0.60).abs() < 1e-12);
        window.g_perpendicular = 0.65;
        assert!((window.g_for_month(6) - 0.65).abs() < 1e-12);
    }

    #[test]
    fn annex_a_dynamic_window_changes_g_and_u() {
        let base = valid(&sample());
        let mut input = sample();
        let window = &mut input.windows[0];
        window.dynamic = Some(crate::annex_a::DynamicTransparent::SingleState {
            state: crate::annex_a::DynamicState {
                id: "tinted".into(),
                g_perpendicular: 0.3,
                u_value_w_per_m2k: 1.7,
            },
            source_reference: "product".into(),
        });
        let result = valid(&input);
        let window = &input.windows[0];
        // g halves (0,6 → 0,3); U +0,5 adds 10 m²·0,5 to the sky loss and H_D.
        let solar_base = window_solar_kwh(&sample().windows[0], 7, Balance::Cooling);
        let solar = window_solar_kwh(window, 7, Balance::Cooling);
        assert!(solar < solar_base);
        assert!((dynamic_window_correction_w_per_k(&input, 0) - 5.0).abs() < 1e-12);
        // Explicit route: H_tr 80 W/K becomes 85 W/K in January.
        let theta = result.monthly[0].heating.calculation_temperature_c;
        assert!(
            (result.monthly[0].heating.transmission_kwh - 85.0 * (theta - 2.61) * 744.0 / 1000.0)
                .abs()
                < 1e-9
        );
        assert!(
            result.monthly[0].heating.transmission_kwh > base.monthly[0].heating.transmission_kwh
        );
    }

    #[test]
    fn recoverable_losses_follow_7_3_and_7_7() {
        let base = valid(&sample());
        let heating = vec![50.0; 12];
        let adjusted = apply_recoverable_losses(&base, &heating, &[]);
        assert!(adjusted.recoverable_losses_applied);
        assert_eq!(
            adjusted.annual_heating_need_without_recoverable_kwh,
            base.annual_heating_need_kwh
        );
        let jan = &base.monthly[0].heating;
        let gains_incl = jan.gains_kwh + 50.0;
        let eta_incl = heating_utilization(gains_incl / jan.heat_transfer_kwh, jan.a, gains_incl);
        let expected = jan.heat_transfer_kwh
            - jan.utilization * jan.gains_kwh
            - (eta_incl - jan.utilization) * jan.gains_kwh
            - jan.utilization * 50.0;
        assert!((adjusted.monthly[0].heating.need_kwh - expected.max(0.0)).abs() < 1e-9);
        assert!(adjusted.monthly[0].heating.need_kwh < jan.need_kwh);
        // 7.7: heating-system losses add to the cooling need in summer.
        let july = &base.monthly[6].cooling;
        if july.need_kwh > 0.0 {
            assert!(adjusted.monthly[6].cooling.need_kwh > july.need_kwh);
        }
    }

    #[test]
    fn utility_internal_gains_follow_7_25_to_7_29() {
        let mut input = office();
        input.internal_gains = InternalGains::Utility {
            lighting: UtilityLighting::Declared {
                annual_kwh: 876.0,
                recovery: LightingRecovery::ForfaitPower,
            },
            hot_water_recoverable_kwh: vec![10.0; 12],
            source_reference: "synthetic".into(),
        };
        // Office: 5·0,30 + 4 = 5,5 W/m²; lighting 0,3·876·1000/8760 = 30 W.
        let expected = (5.5 * 100.0 + 30.0) * 744.0 / 1000.0 + 10.0;
        assert!((internal_gains_kwh(&input, 0) - expected).abs() < 1e-9);
        input.ventilation_flows.clear();
        input.ventilation =
            Some(serde_json::from_value(chapter_11(&input, "office", "c1")).unwrap());
        let result = valid(&input);
        let c1 = result.fixed_c1.unwrap();
        assert_eq!(c1.status, "calculated_unverified", "{:?}", c1.issues);
        // Declared utility gains cannot be split for §5.4.2.
        input.internal_gains = InternalGains::Declared {
            heat_flux_w_per_m2: 6.0,
            source_reference: "synthetic".into(),
        };
        let declared = valid(&input);
        assert_eq!(declared.fixed_c1.unwrap().status, "unavailable");
    }

    fn valid(input: &MonthlyDemandInput) -> MonthlyDemandAssessment {
        let result = assess_monthly_demand(input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        result
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
        assert!((occupants_per_dwelling(100.0) - 2.28).abs() < 1e-12);
    }

    #[test]
    fn utilisation_limits_follow_7_46_to_7_54() {
        assert!((heating_utilization(1.0, 2.0, 1.0) - 2.0 / 3.0).abs() < 1e-12);
        assert!((heating_utilization(0.5, 2.0, 1.0) - (1.0 - 0.25) / (1.0 - 0.125)).abs() < 1e-12);
        // 7.48: γ ≤ 0 with positive gains gives 1/γ; 7.49 gives 1.
        assert_eq!(heating_utilization(-0.5, 2.0, 10.0), -2.0);
        assert_eq!(heating_utilization(-0.5, 2.0, -10.0), 1.0);
        assert!((cooling_utilization(1.0, 2.0) - 2.0 / 3.0).abs() < 1e-12);
        assert!((cooling_utilization(2.0, 2.0) - (1.0 - 0.25) / (1.0 - 0.125)).abs() < 1e-12);
        // 7.54
        assert_eq!(cooling_utilization(-1.0, 2.0), 1.0);
    }

    #[test]
    fn sunroom_gains_follow_7_30b_and_7_34_to_7_37() {
        let mut input = sample();
        input.sunrooms.push(Sunroom {
            id: "serre".into(),
            glazing_g_heating: 0.6,
            glazing_g_cooling: 0.6,
            exterior_frame_fraction: 0.2,
            reduction_factor: 0.8,
            zone_conductance_w_per_k: 30.0,
            distribution_factor: 1.0,
            surfaces: vec![SunroomSurface {
                area_m2: 12.0,
                absorptance: 0.6,
                azimuth_deg: 180.0,
                tilt_deg: 0.0,
            }],
            source_reference: "drawing".into(),
        });
        let jan_irradiance = climate::irradiance_at(180.0, 0.0, 1).unwrap();
        let gains = 0.6 * 0.8 * 0.6 * 12.0 * jan_irradiance * 0.001 * 744.0;
        let cap = (0.8 * 30.0 * (20.0 - 2.61) * 0.001 * 744.0 / gains).min(1.0);
        let expected = (1.0 - 0.8) * cap * gains;
        assert!((sunroom_gains_kwh(&input, 0, Balance::Heating) - expected).abs() < 1e-9);
        // Cooling: no 7.37 cap.
        let july_irradiance = climate::irradiance_at(180.0, 0.0, 7).unwrap();
        let july = 0.2 * 0.6 * 0.8 * 0.6 * 12.0 * july_irradiance * 0.001 * 744.0;
        assert!((sunroom_gains_kwh(&input, 6, Balance::Cooling) - july).abs() < 1e-9);
        let base = valid(&sample());
        let with = valid(&input);
        assert!(with.monthly[0].heating.need_kwh < base.monthly[0].heating.need_kwh);
    }

    #[test]
    fn mixed_zone_uses_area_weighted_values_6_5_3() {
        let parts = vec![
            UsageFunctionArea {
                function: UsageFunction::Office,
                area_m2: 60.0,
            },
            UsageFunctionArea {
                function: UsageFunction::Retail,
                area_m2: 40.0,
            },
        ];
        let profile = FunctionProfile::weighted(&parts);
        assert!((profile.heating_setpoint_c - 21.0).abs() < 1e-12);
        assert!((profile.day_reduction_h - (0.6 * 14.0 + 0.4 * 13.0)).abs() < 1e-12);
        // Office 5·0,30 + 4 = 5,5; retail 3·0,40 + 3 = 4,2 W/m².
        assert!((profile.occupancy_appliance_w_per_m2 - (0.6 * 5.5 + 0.4 * 4.2)).abs() < 1e-12);
        let hours = 0.6 * 48.0 + 0.4 * 24.0;
        assert!(
            (profile.cooling_reduction_factor() - cooling_reduction_from_hours(hours)).abs()
                < 1e-12
        );
        let mut input = office();
        input.function_areas = parts.clone();
        valid(&input);
        input.usage_function = UsageFunction::Retail;
        let codes: Vec<_> = assess_monthly_demand(&input)
            .issues
            .iter()
            .map(|item| item.code)
            .collect();
        assert!(codes.contains(&"usage_function_not_largest"));
        input.usage_function = UsageFunction::Office;
        input.function_areas[1].area_m2 = 10.0;
        assert!(assess_monthly_demand(&input)
            .issues
            .iter()
            .any(|item| item.code == "function_areas_invalid"));
    }

    #[test]
    fn usage_function_tables_7_13_to_7_15() {
        assert_eq!(UsageFunction::HealthcareWithBeds.heating_setpoint_c(), 22.0);
        assert_eq!(UsageFunction::Sport.heating_setpoint_c(), 16.0);
        assert_eq!(UsageFunction::Office.heating_setpoint_c(), 21.0);
        assert_eq!(UsageFunction::Sport.reduced_setpoint_c(), 14.0);
        assert_eq!(UsageFunction::Residential.reduced_setpoint_c(), 16.0);
        assert_eq!(UsageFunction::Retail.reduction_hours(), (13.0, 24.0, 24.0));
        assert_eq!(UsageFunction::Cell.reduction_hours(), (8.0, 0.0, 0.0));
        // 7.74/7.75: office f = 48/168, a = 1 − f + 0,3·f.
        let f = 48.0 / 168.0;
        assert!(
            (cooling_reduction_factor(UsageFunction::Office) - (1.0 - f + 0.3 * f)).abs() < 1e-12
        );
        assert_eq!(cooling_reduction_factor(UsageFunction::Residential), 1.0);
    }

    fn intermittency(function: UsageFunction, tau: f64) -> HeatingIntermittency {
        heating_intermittency(&IntermittencyInput {
            profile: FunctionProfile::of(function),
            setpoint_c: 20.0,
            outdoor_c: 0.0,
            annual_outdoor_c: 10.0,
            hours: 744.0,
            time_constant_h: tau,
            gains_kwh: 0.0,
            conductance_w_per_k: 100.0,
            ground_conductance_w_per_k: 0.0,
        })
    }

    #[test]
    fn intermittent_heating_7_64_branch_by_hand() {
        // No gains: dθ_float = 0; θ_low 16 → dθ_set = 0,8; τ 50 h, t 10 h.
        // t_low/τ = −ln 0,8 = 0,2231 → f_low = 1,116 ≥ 1 → 7.64.
        let result = intermittency(UsageFunction::Residential, 50.0);
        let d_red = 1.0 / 0.2 * (1.0 - (-0.2_f64).exp());
        let f_day = 10.0 * 7.0 / 168.0;
        let a = 1.0 - f_day + f_day * d_red;
        assert!((result.reduction_factor - a).abs() < 1e-12);
        assert!((result.calculation_temperature_c - a * 20.0).abs() < 1e-12);
        assert!((result.reduction_factor - 0.960_977).abs() < 1e-6);
    }

    #[test]
    fn intermittent_heating_7_65_branch_and_weekend_by_hand() {
        // τ 5 h: t/τ = 2, f_low = 0,2231/2 < 1 → 7.65.
        let result = intermittency(UsageFunction::Residential, 5.0);
        let f_low = -(0.8_f64.ln()) / 2.0;
        let d_red = (1.0 - 0.8) / 2.0 + (1.0 - f_low) * 0.8;
        let f_day = 10.0 * 7.0 / 168.0;
        assert!((result.reduction_factor - (1.0 - f_day + f_day * d_red)).abs() < 1e-12);
        // Office: 14 h per weekday over 5 days, 48 h weekend (7.62/7.63).
        let office = intermittency(UsageFunction::Office, 50.0);
        let f_day = 14.0 * (7.0 - 2.0) / 168.0;
        let f_wknd = 48.0 / 168.0;
        let mean = |t: f64| {
            let ratio = t / 50.0;
            let f_low = -(0.8_f64.ln()) / ratio;
            if f_low >= 1.0 {
                (1.0 - (-ratio).exp()) / ratio
            } else {
                0.2 / ratio + (1.0 - f_low) * 0.8
            }
        };
        let a_day = 1.0 - f_day + f_day * mean(14.0);
        let a_wknd = 1.0 - f_wknd + f_wknd * mean(48.0);
        assert!((office.reduction_factor - (1.0 - (1.0 - a_day) - (1.0 - a_wknd))).abs() < 1e-12);
    }

    #[test]
    fn free_floating_and_warm_months_disable_the_reduction() {
        // θ_set ≤ θ_e: dθ_set = dθ_float = 1 → a_H;red = 1.
        let warm = heating_intermittency(&IntermittencyInput {
            outdoor_c: 21.0,
            ..IntermittencyInput {
                profile: FunctionProfile::of(UsageFunction::Office),
                setpoint_c: 20.0,
                outdoor_c: 0.0,
                annual_outdoor_c: 10.0,
                hours: 744.0,
                time_constant_h: 50.0,
                gains_kwh: 0.0,
                conductance_w_per_k: 100.0,
                ground_conductance_w_per_k: 0.0,
            }
        });
        assert!((warm.reduction_factor - 1.0).abs() < 1e-12);
        // Gains covering all losses: dθ_float = 1 → f_low = 0 → 7.65 gives 1.
        let covered = heating_intermittency(&IntermittencyInput {
            profile: FunctionProfile::of(UsageFunction::Office),
            setpoint_c: 20.0,
            outdoor_c: 0.0,
            annual_outdoor_c: 10.0,
            hours: 744.0,
            time_constant_h: 50.0,
            gains_kwh: 1.0e6,
            conductance_w_per_k: 100.0,
            ground_conductance_w_per_k: 0.0,
        });
        assert!((covered.reduction_factor - 1.0).abs() < 1e-12);
    }

    #[test]
    fn dwelling_levelling_7_78_by_hand() {
        // H_e;spec 1,0 W/m²K, θ 20, θ_e 0, other dwelling f_sp 0,6:
        // (0,8·0,6)·(0,6·1,0)·20 / (0,6 + 2,0).
        let delta = levelling_reduction_k(DwellingType::Other, 1.0, 20.0, 0.0);
        assert!((delta - 0.48 * 0.6 * 20.0 / 2.6).abs() < 1e-12);
        let apartment = levelling_reduction_k(DwellingType::ApartmentBuilding, 1.0, 20.0, 0.0);
        assert!((apartment - 0.4 * 0.5 * 20.0 / 2.5).abs() < 1e-12);
    }

    #[test]
    fn hand_checked_january_balance() {
        let result = valid(&sample());
        let jan = &result.monthly[0];
        let hours = 744.0;
        let tau = 180.0 * 1000.0 * 100.0 / 3600.0 / 120.0;
        assert!((jan.heating.time_constant_h - tau).abs() < 1e-9);
        assert!((jan.cooling.time_constant_h - tau).abs() < 1e-9);
        let a = 1.0 + tau / 15.0;
        let internal = 180.0 * 2.28 * 0.001 * hours;
        assert!((jan.internal_gains_kwh - internal).abs() < 1e-9);
        let window = 0.9 * 0.6 * 10.0 * 0.75 * 60.1 * hours * 0.001
            - 0.5 * 0.04 * 1.2 * 10.0 * 4.14 * 11.0 * hours * 0.001;
        assert!((jan.window_solar_gains_kwh - window).abs() < 1e-9);
        let roof = 0.6 * 0.04 * 0.16 * 50.0 * 28.0 * hours * 0.001
            - 1.0 * 0.04 * 0.16 * 50.0 * 4.14 * 11.0 * hours * 0.001;
        assert!((jan.opaque_solar_gains_kwh - roof).abs() < 1e-9);
        let gains = internal + window + roof;
        // 7.78/7.79 with H_e;spec = 120/100, then 7.59–7.73.
        let setpoint = 20.0 - levelling_reduction_k(DwellingType::Other, 1.2, 20.0, 2.61);
        assert!((jan.heating.setpoint_c - setpoint).abs() < 1e-12);
        let expected = heating_intermittency(&IntermittencyInput {
            profile: FunctionProfile::of(UsageFunction::Residential),
            setpoint_c: setpoint,
            outdoor_c: 2.61,
            annual_outdoor_c: annual_mean_outdoor_temperature_c(),
            hours,
            time_constant_h: tau,
            gains_kwh: gains,
            conductance_w_per_k: 120.0,
            ground_conductance_w_per_k: 0.0,
        });
        let theta = expected.calculation_temperature_c;
        assert!((jan.heating.calculation_temperature_c - theta).abs() < 1e-12);
        assert!(theta < setpoint && setpoint < 20.0);
        let q_tr = 80.0 * (theta - 2.61) * hours / 1000.0;
        let q_ve = 40.0 * (theta - 2.61) * hours / 1000.0;
        assert!((jan.heating.transmission_kwh - q_tr).abs() < 1e-9);
        assert!((jan.heating.ventilation_kwh - q_ve).abs() < 1e-9);
        let gamma = gains / (q_tr + q_ve);
        let eta = (1.0 - gamma.powf(a)) / (1.0 - gamma.powf(a + 1.0));
        assert!((jan.heating.need_kwh - (q_tr + q_ve - eta * gains)).abs() < 1e-9);
        // Cooling uses the standard setpoint and a_C;red = 1 for dwellings.
        assert_eq!(jan.cooling.calculation_temperature_c, 24.0);
        assert!((jan.cooling.transmission_kwh - 80.0 * (24.0 - 2.61) * 0.744).abs() < 1e-9);
        assert_eq!(jan.cooling.need_kwh, 0.0);
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn heating_gate_7_2_stops_need_above_gamma_two() {
        let mut input = sample();
        input.windows[0].area_m2 = 40.0;
        let result = valid(&input);
        let june = &result.monthly[5];
        let gamma = june.heating.gamma.unwrap();
        assert!(gamma > 2.0, "{gamma}");
        assert_eq!(june.heating.need_kwh, 0.0);
        // Below the gate the old formula still applies.
        let jan = &result.monthly[0];
        assert!(jan.heating.gamma.unwrap() <= 2.0);
        assert!(jan.heating.need_kwh > 0.0);
    }

    #[test]
    fn cooling_gate_and_cooling_month() {
        let mut input = sample();
        input.windows[0].area_m2 = 40.0;
        let result = valid(&input);
        let july = &result.monthly[6];
        assert!(july.cooling.heat_transfer_kwh / july.cooling.gains_kwh <= 2.0);
        assert!(july.cooling.need_kwh > 0.0);
        let base = valid(&sample());
        let jan = &base.monthly[0];
        assert!(jan.cooling.heat_transfer_kwh / jan.cooling.gains_kwh > 2.0);
        assert_eq!(jan.cooling.need_kwh, 0.0);
    }

    #[test]
    fn warm_supply_air_uses_7_54_instead_of_failing() {
        // July supply air at 30 °C makes H_C;ve negative (b_v < 0) and the
        // cooling heat transfer negative; a large H_C;g;adj keeps τ_C positive.
        let mut input = sample();
        let july = &mut input.ventilation_flows[0].months[6];
        july.cooling_conductance_w_per_k = Some(400.0);
        july.cooling_supply_temperature_c = Some(30.0);
        let Transmission::Explicit(transmission) = &mut input.transmission else {
            unreachable!()
        };
        transmission.ground = Some(GroundTransfer {
            monthly_conductance_w_per_k: vec![0.0; 12],
            heating_adjusted_conductance_w_per_k: 0.0,
            cooling_adjusted_conductance_w_per_k: 1000.0,
            source_reference: "synthetic".into(),
        });
        let result = valid(&input);
        let july = &result.monthly[6];
        assert!(july.cooling.heat_transfer_kwh < 0.0);
        assert_eq!(july.cooling.utilization, 1.0);
        let expected = july.cooling.gains_kwh - july.cooling.heat_transfer_kwh;
        assert!((july.cooling.need_kwh - expected).abs() < 1e-9);
    }

    #[test]
    fn utility_cooling_need_includes_a_c_red() {
        let mut input = office();
        input.windows[0].area_m2 = 40.0;
        let result = valid(&input);
        let july = &result.monthly[6];
        let f = 48.0 / 168.0;
        let a_red = 1.0 - f + 0.3 * f;
        assert!((july.cooling.reduction_factor - a_red).abs() < 1e-12);
        let raw =
            july.cooling.gains_kwh - july.cooling.utilization * july.cooling.heat_transfer_kwh;
        assert!(raw > 0.0);
        assert!((july.cooling.need_kwh - a_red * raw).abs() < 1e-9);
        // Utility: no levelling, but intermittent heating lowers θ_calc.
        let jan = &result.monthly[0];
        assert_eq!(jan.heating.setpoint_c, 21.0);
        assert!(jan.heating.calculation_temperature_c < 21.0);
    }

    #[test]
    fn supply_temperature_enters_through_b_v() {
        let mut input = sample();
        for row in &mut input.ventilation_flows[0].months {
            row.supply_temperature_c = Some(15.0);
            row.cooling_conductance_w_per_k = Some(10.0);
        }
        let result = valid(&input);
        let jan = &result.monthly[0];
        // 7.20: b_v = (20 − 15)/(20 − 2,61); 7.18 with θ_calc.
        let h_ve = 40.0 * 5.0 / (20.0 - 2.61);
        assert!((jan.heating.ventilation_conductance_w_per_k - h_ve).abs() < 1e-12);
        let theta = jan.heating.calculation_temperature_c;
        assert!((jan.heating.ventilation_kwh - h_ve * (theta - 2.61) * 0.744).abs() < 1e-9);
        // Cooling: conductance 10, supply 15 → b_v = (24 − 15)/(24 − 2,61).
        let h_c = 10.0 * 9.0 / (24.0 - 2.61);
        assert!((jan.cooling.ventilation_conductance_w_per_k - h_c).abs() < 1e-12);
        assert!((jan.cooling.ventilation_kwh - 10.0 * 9.0 * 0.744).abs() < 1e-9);
        // τ_H uses H_H;ve with b_v.
        let tau = 180.0 * 1000.0 * 100.0 / 3600.0 / (80.0 + h_ve);
        assert!((jan.heating.time_constant_h - tau).abs() < 1e-9);
    }

    #[test]
    fn heavier_mass_lowers_heating_need() {
        let light = {
            let mut input = sample();
            input.thermal_mass.floor = MassClass::Light;
            input.thermal_mass.ceiling = CeilingColumn::ClosedOrSuspended;
            valid(&input).annual_heating_need_kwh.unwrap()
        };
        let heavy = valid(&sample()).annual_heating_need_kwh.unwrap();
        assert!(heavy < light);
    }

    #[test]
    fn rejects_incomplete_or_unsupported_input_without_numbers() {
        let mut input = sample();
        input.windows[0].tilt_deg = 200.0;
        input.window_inventory_complete = false;
        if let Transmission::Explicit(transmission) = &mut input.transmission {
            transmission.ground_inventory_confirmed = false;
        }
        input.ventilation_flows[0].months.pop();
        input.ventilation_flows[0].months[0].cooling_conductance_w_per_k = Some(-1.0);
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
            "ventilation_conductance_invalid",
            "element_id_invalid",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    #[test]
    fn rejects_function_inconsistencies() {
        let mut input = sample();
        input.setpoints.heating_c = 21.0;
        input.dwelling_type = None;
        input.windows[0].movable_shading = Some(MovableShading {
            reduction_factor: 0.2,
            control: crate::solar_shading::ShadingControl::ManualUtilityWithoutGlareProtection,
            source_reference: "table 7.5".into(),
        });
        let codes: Vec<_> = assess_monthly_demand(&input)
            .issues
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "setpoints_table_7_13_mismatch",
            "dwelling_type_required",
            "window_shading_control_function_mismatch",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
        let mut utility = office();
        utility.dwelling_type = Some(DwellingType::Other);
        utility.internal_gains = sample().internal_gains;
        let codes: Vec<_> = assess_monthly_demand(&utility)
            .issues
            .iter()
            .map(|item| item.code)
            .collect();
        assert!(codes.contains(&"dwelling_type_not_applicable"));
        assert!(codes.contains(&"internal_gains_function_mismatch"));
    }

    #[test]
    fn rejects_unknown_fields() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["windows"][0]["gValue"] = json!(0.5);
        assert!(serde_json::from_value::<MonthlyDemandInput>(value).is_err());
    }

    #[test]
    fn explicit_ground_terms_enter_transfer_and_time_constants() {
        let mut input = sample();
        let Transmission::Explicit(transmission) = &mut input.transmission else {
            unreachable!()
        };
        transmission.ground = Some(GroundTransfer {
            monthly_conductance_w_per_k: (1..=12).map(|month| 20.0 + f64::from(month)).collect(),
            heating_adjusted_conductance_w_per_k: 30.0,
            cooling_adjusted_conductance_w_per_k: 10.0,
            source_reference: "synthetic annex D".into(),
        });
        let result = valid(&input);
        let annual = annual_mean_outdoor_temperature_c();
        for (index, row) in result.monthly.iter().enumerate() {
            let h_g = 21.0 + index as f64;
            assert_eq!(row.ground_conductance_w_per_k, h_g);
            let theta = row.heating.calculation_temperature_c;
            let expected = (80.0 * (theta - row.outdoor_temperature_c) + h_g * (theta - annual))
                * row.hours
                / 1000.0;
            assert!((row.heating.transmission_kwh - expected).abs() < 1e-9);
            let cooling = (80.0 * (24.0 - row.outdoor_temperature_c) + h_g * (24.0 - annual))
                * row.hours
                / 1000.0;
            assert!((row.cooling.transmission_kwh - cooling).abs() < 1e-9);
        }
        let jan = &result.monthly[0];
        let c = 180.0 * 1000.0 * 100.0 / 3600.0;
        assert!((jan.heating.time_constant_h - c / 150.0).abs() < 1e-9);
        assert!((jan.cooling.time_constant_h - c / 130.0).abs() < 1e-9);
    }

    #[test]
    fn composes_transmission_from_chapter_8_components() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["transmission"] = json!({
            "method": "components",
            "direct": {
                "elements": [{"id": "wall", "areaM2": 100.0, "uValueWPerM2k": 0.2, "sourceReference": "Rc 4.7"}],
                "linearBridges": [{"id": "lb", "lengthM": 20.0, "psiWPerMk": 0.05, "sourceReference": "detail"}]
            },
            "unheated": {"spaces": [{
                "id": "crawl", "reductionFactor": 0.5, "factorSourceReference": "8.4",
                "boundary": {"elements": [{"id": "floor-crawl", "areaM2": 40.0, "uValueWPerM2k": 0.25, "sourceReference": "Rc 3.7"}]}
            }]},
            "groundFloors": [{
                "id": "slab", "areaM2": 67.0, "exposedPerimeterM": 32.92,
                "constructionResistanceM2kPerW": 1.0 / 0.258398,
                "edgeThermalBridges": {"method": "detailed", "bridges": []},
                "sourceReference": "C1 example"
            }],
            "groundInventoryConfirmed": true
        });
        let input: MonthlyDemandInput = serde_json::from_value(value).unwrap();
        let result = valid(&input);
        let summary = result.transmission.as_ref().unwrap();
        assert!((summary.direct_conductance_w_per_k.unwrap() - 21.0).abs() < 1e-12);
        assert!((summary.unheated_conductance_w_per_k.unwrap() - 5.0).abs() < 1e-12);
        assert!((summary.conductance_w_per_k - 26.0).abs() < 1e-12);
        assert!((summary.ground_steady_conductance_w_per_k.unwrap() - 13.163).abs() < 1e-3);
        let Transmission::Components(components) = &input.transmission else {
            unreachable!()
        };
        let slab = crate::ground::slab_coefficients(&components.ground_floors[0]).unwrap();
        let ground = ground_monthly(&[slab], 20.0, 24.0, annual_mean_outdoor_temperature_c());
        assert_eq!(
            summary.ground_monthly_conductance_w_per_k,
            ground.monthly_w_per_k
        );
        let jan = &result.monthly[0];
        let tau = 180.0 * 1000.0 * 100.0 / 3600.0 / (26.0 + ground.heating_adjusted_w_per_k + 40.0);
        assert!((jan.heating.time_constant_h - tau).abs() < 1e-9);
    }

    #[test]
    fn vertical_pipes_add_h_p_per_table_7_1() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["transmission"] = json!({
            "method": "components",
            "direct": {
                "elements": [{"id": "wall", "areaM2": 100.0, "uValueWPerM2k": 0.2, "sourceReference": "Rc 4.7"}]
            },
            "groundFloors": [],
            "groundInventoryConfirmed": true,
            "verticalPipes": [
                {"id": "hwa", "storeys": 3, "insulated": false, "sourceReference": "survey"},
                {"id": "riool", "storeys": 3, "insulated": true, "sharedZones": 2, "sourceReference": "survey"}
            ]
        });
        let input: MonthlyDemandInput = serde_json::from_value(value).unwrap();
        let result = valid(&input);
        let summary = result.transmission.as_ref().unwrap();
        // 7.17: 3·1,8 + 3·0,5/2.
        let pipes = 3.0 * 1.8 + 3.0 * 0.5 / 2.0;
        assert!((summary.vertical_pipe_conductance_w_per_k.unwrap() - pipes).abs() < 1e-12);
        assert!((summary.conductance_w_per_k - (20.0 + pipes)).abs() < 1e-12);
    }

    #[test]
    fn component_errors_are_prefixed_and_block_numbers() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["transmission"] = json!({
            "method": "components",
            "direct": {"elements": [{"id": "wall", "areaM2": -1.0, "uValueWPerM2k": 0.2, "sourceReference": ""}]},
            "unheated": null,
            "groundFloors": [{"id": "slab", "areaM2": 50.0, "exposedPerimeterM": 0.0,
                "constructionResistanceM2kPerW": 3.0, "edgeThermalBridges": {"method": "forfait"},
                "sourceReference": "x"}],
            "groundInventoryConfirmed": true
        });
        let input: MonthlyDemandInput = serde_json::from_value(value).unwrap();
        let result = assess_monthly_demand(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.transmission.is_none());
        assert!(result
            .issues
            .iter()
            .any(|item| item.path.starts_with("transmission.direct.")));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "ground_floor_invalid"));
    }

    #[test]
    fn pitched_roof_uses_interpolated_irradiance_and_sky_factor() {
        let mut input = sample();
        input.opaque_elements[0].tilt_deg = 35.0;
        let result = valid(&input);
        // January south: 30° = 50,5 and 45° = 57,9 W/m², so 35° = 52,966…
        let irradiance = 50.5 + (57.9 - 50.5) / 3.0;
        let hours = 744.0;
        let roof = 0.6 * 0.04 * 0.16 * 50.0 * irradiance * hours * 0.001
            - 0.75 * 0.04 * 0.16 * 50.0 * 4.14 * 11.0 * hours * 0.001;
        assert!((result.monthly[0].opaque_solar_gains_kwh - roof).abs() < 1e-9);
        assert_eq!(sky_view_factor(120.0), 0.0);
    }

    /// Balance invariants over a grid of envelope, glazing and mass variants.
    #[test]
    fn balance_invariants_hold_over_variants() {
        let masses = [
            (
                MassClass::Light,
                MassClass::Light,
                CeilingColumn::ClosedOrSuspended,
            ),
            (
                MassClass::VeryHeavy,
                MassClass::Heavy,
                CeilingColumn::OpenOrNone,
            ),
        ];
        let mut previous_heating: Option<f64> = None;
        for conductance in [20.0, 60.0, 120.0, 240.0] {
            for window_area in [0.5, 10.0, 40.0] {
                for (floor, wall, ceiling) in masses {
                    for base in [sample(), office()] {
                        let mut input = base;
                        if let Transmission::Explicit(transmission) = &mut input.transmission {
                            transmission.conductance_w_per_k = conductance;
                        }
                        input.windows[0].area_m2 = window_area;
                        input.thermal_mass.floor = floor;
                        input.thermal_mass.wall = wall;
                        input.thermal_mass.ceiling = ceiling;
                        let result = valid(&input);
                        for row in &result.monthly {
                            for balance in [&row.heating, &row.cooling] {
                                assert!(balance.need_kwh >= 0.0);
                                assert!((0.0..=1.0 + 1e-12).contains(&balance.utilization));
                                assert!((0.0..=1.0 + 1e-12).contains(&balance.reduction_factor));
                            }
                            assert!(
                                row.heating.need_kwh
                                    <= row.heating.heat_transfer_kwh.max(0.0) + 1e-9
                            );
                            assert!(row.cooling.need_kwh <= row.cooling.gains_kwh + 1e-9);
                            assert!(
                                row.heating.calculation_temperature_c
                                    <= row.heating.setpoint_c + 1e-9
                            );
                        }
                    }
                }
            }
            // More conductance never lowers the heating need.
            let mut input = sample();
            if let Transmission::Explicit(transmission) = &mut input.transmission {
                transmission.conductance_w_per_k = conductance;
            }
            let heating = valid(&input).annual_heating_need_kwh.unwrap();
            if let Some(previous) = previous_heating {
                assert!(heating >= previous - 1e-9);
            }
            previous_heating = Some(heating);
        }
        // More south glazing never lowers the cooling need.
        let mut last = 0.0;
        for window_area in [0.5, 5.0, 10.0, 20.0, 40.0] {
            let mut input = sample();
            input.windows[0].area_m2 = window_area;
            let cooling = valid(&input).annual_cooling_need_kwh.unwrap();
            assert!(cooling >= last - 1e-9, "{window_area}: {cooling} < {last}");
            last = cooling;
        }
    }

    #[test]
    fn obstruction_and_movable_shading_act_on_their_own_balance() {
        use crate::solar_shading::{MovableShading, ShadingControl};
        let declared = valid(&sample());
        let mut minimal = sample();
        minimal.windows[0].obstruction = Obstruction::Minimal;
        let minimal_result = valid(&minimal);
        // Table 17.4 lowers winter heating gains; table 17.5 leaves cooling unchanged.
        assert!(
            minimal_result.annual_heating_need_kwh.unwrap()
                > declared.annual_heating_need_kwh.unwrap()
        );
        assert!(
            (minimal_result.annual_cooling_need_kwh.unwrap()
                - declared.annual_cooling_need_kwh.unwrap())
            .abs()
                < 1e-9
        );
        let mut shaded = minimal.clone();
        shaded.windows[0].area_m2 = 30.0;
        let unshaded_result = valid(&shaded);
        shaded.windows[0].movable_shading = Some(MovableShading {
            reduction_factor: 0.2,
            control: ShadingControl::ManualResidential,
            source_reference: "table 7.5 screen".into(),
        });
        let shaded_result = valid(&shaded);
        assert!(
            shaded_result.annual_cooling_need_kwh.unwrap()
                < unshaded_result.annual_cooling_need_kwh.unwrap()
        );
        // Manual shading in dwellings: f_sh;with = 0 on the heating balance.
        assert!(
            (shaded_result.annual_heating_need_kwh.unwrap()
                - unshaded_result.annual_heating_need_kwh.unwrap())
            .abs()
                < 1e-9
        );
        // Automatic shading not tuned per ISO 52016-3 also reduces heating gains.
        shaded.windows[0].movable_shading.as_mut().unwrap().control = ShadingControl::Automatic;
        let automatic = valid(&shaded);
        assert!(
            automatic.annual_heating_need_kwh.unwrap()
                > unshaded_result.annual_heating_need_kwh.unwrap()
        );
        let july = &shaded_result.monthly[6];
        assert!(july.window_solar_cooling_kwh < july.window_solar_gains_kwh);
    }
}
