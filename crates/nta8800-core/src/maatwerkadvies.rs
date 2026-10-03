//! Maatwerkadvies (BRL 9500-MWA-W/U) on top of the NTA 8800 calculation.
//!
//! Method books: ISSO 82.2 3e druk (dwellings) and ISSO 75.2 3e druk
//! (utility buildings); requirements: BRL 9500-MWA-W/U §3.1 and §4.2.5.
//!
//! - Current-use layer (ISSO 82.2/75.2 §2.5, table 2.2): standard user
//!   profiles (82.2 tables 2.3–2.6, p. 36–38; 75.2 tables 2.3–2.5, p. 42–43)
//!   or free input for setpoints, reduced heating, `f_mod;sp`, occupants,
//!   internal heat and the hot-water need. The layer only feeds the
//!   actual-use run; the label run keeps NTA 8800 unchanged.
//! - Measures are JSON patches (RFC 6902 subset) on the project or on the
//!   derived building input, with investment, cost source, lifetime,
//!   maintenance and phasing. Packages combine measures (82.2 §4.2.2, p. 52:
//!   at least two packages).
//! - Every variant (current situation, each measure, each package) runs
//!   twice: the NTA 8800 run (label class, BENG 1–3, TOjuli) and the
//!   actual-use run (savings per carrier, primary energy, CO2, energy costs
//!   from the adviser's tariffs; 82.2 §4.2.3, p. 53).
//! - Economics: simple payback (82.2 §5.1, p. 79; §6.2.4, p. 82) and the net
//!   present value demanded by BRL 9500-MWA-W §3.1. The calculation rules of
//!   the ISSO model description (rapport 110293) are not available here; the
//!   NPV uses the adviser's discount rate and energy price change, with
//!   reinvestment at the end of a measure's lifetime and a linear residual
//!   value (interpretation).
//! - Advice: the adviser's chosen package or, failing that, the package
//!   with the highest NPV; warnings (partial execution, TOjuli increase,
//!   fewer than two packages) and specialist notes (82.2 §6.2.3, p. 82).
//! - Fit check (82.2/75.2 chapter 3): annual deviation against measured use
//!   and, with monthly gas readings, the regression lines against the
//!   monthly outdoor temperature (§3.2.3).

use crate::building_performance::{
    assess_building_performance, BuildingPerformanceAssessment, BuildingPerformanceInput,
    HotWaterNeedFit,
};
use crate::climate::OUTDOOR_TEMPERATURE_C;
use crate::indicators_draft::CalculationScope;
use crate::monthly_demand::{occupants_per_dwelling, InternalGains, MonthlyDemandInput, UsageFit};
use crate::project_performance::assess_project_performance;
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

pub const SCOPE: &str = "maatwerkadvies_isso_82_2_75_2_unverified";

/// Groningen-equivalent gross calorific value, 35,17 MJ/m³, in kWh/m³.
pub const GAS_KWH_PER_M3: f64 = 35.17 / 3.6;

pub const INTERPRETATIONS: &[&str] = &[
    "net present value: ISSO rapport 110293 (Modelbeschrijving) is not available; the adviser supplies discount rate, energy price change and horizon, measures are reinvested after their lifetime and the last generation keeps a linear residual value",
    "ISSO 82.2 table 2.5 Q_W;nd;spec is read per occupant per year; the annual need is Q_W;nd;spec × N_p",
    "the hot-water fit rescales the chapter 13 result linearly in the net need (circulation and storage losses fixed)",
    "ISSO 75.2 table 2.4 gives NTA −20 % reduction hours for both energy-conscious and not energy-conscious users; read as +20 % (energy-conscious) and −20 % (not energy-conscious), in line with the weekend column; weekend hours are capped at 48",
    "gas costs use 35,17 MJ/m³ (Groningen equivalent, gross calorific value)",
    "heat from a collective heat-pump source (reported as carrier dh, 5.20) is priced at the district-heat tariff",
    "the best-fit package is the adviser's choice; without one, the package with the highest net present value is proposed and marked as automatic",
    "ISSO 82.2 §2.5.3: the NTA option carries the MWA practice correction for hot water; taken as the average-profile 545 kWh per occupant",
    "ventilation practice factors (82.2 table 2.7, 75.2 table 2.8) apply to the standard profiles or when entered, not to the NTA option (table 2.2 '–'); system B takes the system C value",
    "fit check (82.2 §3.2, Bijlage C.1): heating limit at the knee above the base load (mean of the months ≥ 15 °C); the measured line uses the local temperatures when given, the calculated line the NTA 8800 climate year; electricity is compared as monthly net delivery",
    "NCW phasing: an investment starts in its phase year relative to economics.baseYear; savings of a package start in year 1",
    "EPBD system requirements (Bbl art. 4.248, Omgevingsregeling bijlage VIII) on the standard NTA 8800 run with table 5.2 f_P;del (external heat by forfait); the lighting limit follows the Bbl (75 kWh_prim/m2), ISSO 82.2 table 5.1 prints 17; cooling is not evaluated",
    "utility persons route (75.2 table 2.6): N_p of the building split over the zones by area, q_oc;p 80 W, f_t and q_A from NTA tables 7.2/7.3 unless entered",
    "renovation passport (82.2 §1.10.2/§4.4): the Standaard voor Woningisolatie is the adviser's statement or an entered net heat need limit; gas-free means no natural gas or oil use in the actual-use run",
    "location-specific climate data (82.2 §2.6) is not used: NEN 5060 hourly or KNMI data is not available to the kernel",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MwaBase {
    /// The project (HTTP/Tauri project shape); derived with
    /// `project_performance`.
    Project { project: Value },
    /// A complete building-performance input.
    Building { input: Value },
}

/// ISSO 82.2/75.2 §2.5.2 standard user profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserProfile {
    /// NTA 8800 values (no change).
    Nta,
    EnergyConscious,
    Average,
    NotEnergyConscious,
}

/// ISSO 82.2/75.2 table 2.2: user-dependent parameters. A profile sets the
/// standard values; explicit fields (free input) take precedence.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageProfile {
    pub profile: UserProfile,
    #[serde(default)]
    pub heating_setpoint_c: Option<f64>,
    #[serde(default)]
    pub cooling_setpoint_c: Option<f64>,
    #[serde(default)]
    pub reduced_setpoint_c: Option<f64>,
    #[serde(default)]
    pub day_reduction_h: Option<f64>,
    #[serde(default)]
    pub weekend_reduction_h: Option<f64>,
    /// f_mod;sp (dwellings).
    #[serde(default)]
    pub spatial_fraction: Option<f64>,
    /// N_p of the building (dwellings); split over the zones by area.
    #[serde(default)]
    pub occupants: Option<f64>,
    /// q_int;tot per person, W (dwellings).
    #[serde(default)]
    pub internal_gain_per_person_w: Option<f64>,
    /// q_Oc·f_τ + q_A, W/m² (utility).
    #[serde(default)]
    pub occupancy_appliance_w_per_m2: Option<f64>,
    /// Q_W;nd;spec per occupant, kWh/year (dwellings).
    #[serde(default)]
    pub hot_water_need_per_person_kwh: Option<f64>,
    /// Total annual net hot-water need, kWh; overrides the other hot-water
    /// inputs.
    #[serde(default)]
    pub annual_hot_water_need_kwh: Option<f64>,
    /// Utility: N_p;usi of the building (ISSO 75.2 table 2.6, p. 44); split
    /// over the zones by area. With it, q_Oc·f_τ becomes N_p·q_oc;p·f_t/A_g.
    #[serde(default)]
    pub persons: Option<f64>,
    /// Utility: q_oc;p;usi, W per person (standard 80 W).
    #[serde(default)]
    pub heat_per_person_w: Option<f64>,
    /// Utility: f_t;usi occupancy time fraction (standard table 7.2).
    #[serde(default)]
    pub occupancy_time_fraction: Option<f64>,
    /// Utility: q_A, W/m² (standard table 7.3).
    #[serde(default)]
    pub appliance_w_per_m2: Option<f64>,
    /// Utility: factor on the table 14.1 burning hours (ISSO 75.2 table
    /// 2.7, p. 44: 0,8 / 1,0 / 1,2 by profile).
    #[serde(default)]
    pub lighting_hours_factor: Option<f64>,
    /// ISSO 82.2 table 2.7 / 75.2 table 2.8 practice factors; omitted
    /// fields take the standard values.
    #[serde(default)]
    pub ventilation_practice: Option<crate::ventilation::VentilationPractice>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchTarget {
    Project,
    Building,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum PatchOperation {
    Replace { path: String, value: Value },
    Add { path: String, value: Value },
    Remove { path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasureCategory {
    Insulation,
    Glazing,
    Airtightness,
    Ventilation,
    HeatRecovery,
    Heating,
    HeatPump,
    HotWater,
    Cooling,
    Pv,
    SolarThermal,
    Lighting,
    Control,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Measure {
    pub id: String,
    pub name: String,
    pub category: MeasureCategory,
    pub target: PatchTarget,
    pub patch: Vec<PatchOperation>,
    pub investment_eur: f64,
    /// BRL 9500-MWA-W §4.2.5: the cost figures used must be justified.
    pub cost_source: String,
    pub lifetime_years: f64,
    /// Change in annual maintenance costs, €/year (negative = saving).
    #[serde(default)]
    pub maintenance_eur_per_year: f64,
    /// Planned year of execution (phasing, natural replacement moment).
    #[serde(default)]
    pub phase_year: Option<u32>,
    #[serde(default)]
    pub specialist_note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub measure_ids: Vec<String>,
    /// Warning when the package is executed only partly or later
    /// (BRL 9500-MWA-W §3.1).
    #[serde(default)]
    pub partial_execution_warning: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tariffs {
    pub gas_eur_per_m3: f64,
    pub electricity_eur_per_kwh: f64,
    /// Compensation for exported electricity, €/kWh.
    #[serde(default)]
    pub electricity_export_eur_per_kwh: f64,
    #[serde(default)]
    pub district_heat_eur_per_kwh: f64,
    #[serde(default)]
    pub district_cold_eur_per_kwh: f64,
    #[serde(default)]
    pub oil_eur_per_kwh: f64,
    #[serde(default)]
    pub biomass_eur_per_kwh: f64,
    /// Fixed gas connection charge, €/year, while gas is used.
    #[serde(default)]
    pub gas_fixed_eur_per_year: f64,
    /// Fixed heat connection charge, €/year, while district heat is used.
    #[serde(default)]
    pub heat_fixed_eur_per_year: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Economics {
    /// Real discount rate per year (fraction).
    #[serde(default = "default_discount")]
    pub discount_rate: f64,
    /// Annual change of energy prices (fraction).
    #[serde(default)]
    pub energy_price_change: f64,
    /// Horizon in years; defaults to the longest lifetime in the variant.
    #[serde(default)]
    pub horizon_years: Option<f64>,
    /// Calendar year of t = 0; with it, a measure's `phaseYear` delays its
    /// investment (and replacements) in the NCW.
    #[serde(default)]
    pub base_year: Option<u32>,
    /// Annual change of maintenance costs (fraction).
    #[serde(default)]
    pub maintenance_price_change: f64,
    #[serde(default)]
    pub source_reference: String,
}

/// ISSO 75.2 table 2.6 (p. 44): standard q_oc;p;usi, W per person.
pub const UTILITY_HEAT_PER_PERSON_W: f64 = 80.0;

fn default_discount() -> f64 {
    0.03
}

impl Default for Economics {
    fn default() -> Self {
        Self {
            discount_rate: default_discount(),
            energy_price_change: 0.0,
            horizon_years: None,
            base_year: None,
            maintenance_price_change: 0.0,
            source_reference: String::new(),
        }
    }
}

/// Measured use for the fit check (ISSO 82.2/75.2 §2.7, chapter 3).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeasuredUse {
    #[serde(default)]
    pub annual_gas_m3: Option<f64>,
    /// Delivered (imported) electricity, kWh/year.
    #[serde(default)]
    pub annual_electricity_kwh: Option<f64>,
    #[serde(default)]
    pub annual_heat_kwh: Option<f64>,
    /// Monthly gas readings, m³ (January–December); `null` for no reading.
    #[serde(default)]
    pub monthly_gas_m3: Vec<Option<f64>>,
    /// Monthly delivered electricity, kWh (§3.2: monthly readings, e.g.
    /// smart meter); `null` for no reading.
    #[serde(default)]
    pub monthly_electricity_kwh: Vec<Option<f64>>,
    /// Monthly delivered heat (external heat), kWh.
    #[serde(default)]
    pub monthly_heat_kwh: Vec<Option<f64>>,
    /// Local monthly mean outdoor temperature of the metered period, °C
    /// (§3.2.1 step 1); `null` falls back to table 17.1 of NTA 8800.
    #[serde(default)]
    pub monthly_outdoor_temperature_c: Vec<Option<f64>>,
    #[serde(default)]
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdviceNote {
    pub text: String,
    #[serde(default)]
    pub package_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaatwerkadviesInput {
    pub base: MwaBase,
    /// Current use (fit); `None` keeps the NTA 8800 values.
    #[serde(default)]
    pub current_use: Option<UsageProfile>,
    /// Use after the measures (BRL 9500-MWA-W §4.2.5 note); defaults to the
    /// current use.
    #[serde(default)]
    pub future_use: Option<UsageProfile>,
    pub measures: Vec<Measure>,
    pub packages: Vec<Package>,
    pub tariffs: Tariffs,
    #[serde(default)]
    pub economics: Economics,
    #[serde(default)]
    pub measured: Option<MeasuredUse>,
    #[serde(default)]
    pub advised_package_id: Option<String>,
    #[serde(default)]
    pub advice_motivation: Option<String>,
    #[serde(default)]
    pub notes: Vec<AdviceNote>,
    /// ISSO 82.2 §1.10.2 / §4.4: the three cumulative steps of a
    /// renovation passport, each a package.
    #[serde(default)]
    pub renovation_passport: Option<RenovationPassportInput>,
}

/// ISSO 82.2 §1.10.2 and §4.4.2 (p. 22–23, 75–77).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenovationPassportInput {
    /// Step 1: limit the heat and cold demand (insulation, airtightness,
    /// ventilation, overheating).
    pub demand_package_id: String,
    /// Step 2: sustainable heating, hot water and cooling (natural-gas-free
    /// main heating).
    pub systems_package_id: String,
    /// Step 3: building-bound renewable production and storage.
    pub production_package_id: String,
    /// The pre-war insulation standard is used because façade insulation is
    /// technically impossible; a hybrid heat pump is then allowed.
    #[serde(default)]
    pub prewar_standard: bool,
    /// Required with `prewarStandard`.
    #[serde(default)]
    pub prewar_motivation: Option<String>,
    /// The adviser's statement that step 1 meets the Standaard voor
    /// Woningisolatie (RVO); used when no limit value is given.
    #[serde(default)]
    pub insulation_standard_met: Option<bool>,
    /// Optional limit of the standard as a net heat need, kWh/m²·yr,
    /// checked against the step 1 label run (BENG 1 indicator).
    #[serde(default)]
    pub insulation_standard_max_need_kwh_per_m2: Option<f64>,
    /// Measures in step 1 that limit the risk of overheating.
    #[serde(default)]
    pub overheating_measure_ids: Vec<String>,
    /// Storage capacity matched to the production was considered (§4.4.2
    /// step 3).
    #[serde(default)]
    pub storage_considered: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MwaIssue {
    pub code: &'static str,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

fn issue(code: &'static str, path: impl Into<String>) -> MwaIssue {
    MwaIssue {
        code,
        path: path.into(),
        detail: None,
    }
}

/// Annual energy per carrier of one run.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnergyUse {
    pub gas_kwh: f64,
    pub gas_m3: f64,
    pub electricity_import_kwh: f64,
    pub electricity_export_kwh: f64,
    pub electricity_produced_kwh: f64,
    pub district_heat_kwh: f64,
    pub district_cold_kwh: f64,
    pub oil_kwh: f64,
    pub biomass_kwh: f64,
    pub primary_fossil_kwh: f64,
    pub co2_kg: f64,
    pub energy_cost_eur: f64,
    /// Monthly gas, m³.
    pub monthly_gas_m3: Vec<f64>,
    /// Monthly delivered electricity (use minus self-used production), kWh.
    pub monthly_electricity_import_kwh: Vec<f64>,
    /// Monthly delivered external heat, kWh.
    pub monthly_heat_kwh: Vec<f64>,
    /// Monthly exported electricity, kWh.
    pub monthly_electricity_export_kwh: Vec<f64>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelResult {
    pub label_class: Option<&'static str>,
    pub need_indicator_kwh_per_m2: Option<f64>,
    pub primary_fossil_indicator_kwh_per_m2: Option<f64>,
    pub renewable_share_percent: Option<f64>,
    pub tojuli_max_k: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Savings {
    pub gas_m3: f64,
    pub electricity_kwh: f64,
    pub heat_kwh: f64,
    pub primary_fossil_kwh: f64,
    pub co2_kg: f64,
    pub energy_cost_eur: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantResult {
    pub id: String,
    pub name: String,
    pub kind: &'static str,
    pub measure_ids: Vec<String>,
    pub valid: bool,
    pub label: LabelResult,
    pub actual_use: Option<EnergyUse>,
    /// Against the current situation (positive = saving).
    pub savings: Option<Savings>,
    pub investment_eur: f64,
    pub maintenance_eur_per_year: f64,
    pub simple_payback_years: Option<f64>,
    pub net_present_value_eur: Option<f64>,
    pub horizon_years: f64,
    pub phasing: Vec<PhaseStep>,
    /// EPBD system requirements on the standard run (ISSO 82.2 §5.2).
    pub system_checks: Vec<SystemPerformanceCheck>,
    pub issues: Vec<MwaIssue>,
}

/// Bbl art. 4.248 (table 4.248) with Omgevingsregeling art. 5.2 and
/// bijlage VIII: energy performance of one technical building system,
/// from the standard NTA 8800 run of a variant (ISSO 82.2 §5.2).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPerformanceCheck {
    pub system: &'static str,
    pub value: Option<f64>,
    pub limit: Option<f64>,
    pub unit: &'static str,
    /// `None` when the system is absent, has no limit for this function or
    /// cannot be evaluated.
    pub meets: Option<bool>,
    pub note: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseStep {
    pub year: Option<u32>,
    pub measure_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FitCheck {
    pub calculated: EnergyUse,
    pub gas_deviation_percent: Option<f64>,
    pub electricity_deviation_percent: Option<f64>,
    pub heat_deviation_percent: Option<f64>,
    /// Monthly gas against outdoor temperature (§3.2.3): slope m³/K and
    /// intercept m³ of months with readings below 15 °C.
    pub measured_gas_line: Option<RegressionLine>,
    pub calculated_gas_line: Option<RegressionLine>,
    /// The same for delivered heat (§3.2.6/3.2.7 with heat instead of
    /// gas), kWh per month; slope in kWh/K.
    pub measured_heat_line: Option<RegressionLine>,
    pub calculated_heat_line: Option<RegressionLine>,
    /// Base load (months at or above 15 °C, §3.2.4): mean measured and
    /// calculated gas, m³/month, and heat, kWh/month.
    pub measured_gas_base_load: Option<f64>,
    pub calculated_gas_base_load: Option<f64>,
    pub measured_heat_base_load: Option<f64>,
    pub calculated_heat_base_load: Option<f64>,
    /// Monthly electricity, measured against calculated (§3.2.5), %; the
    /// fit is on the shape, not on single months.
    pub monthly_electricity_deviation_percent: Vec<Option<f64>>,
    /// Mean monthly measured and calculated electricity, kWh.
    pub measured_electricity_monthly_mean_kwh: Option<f64>,
    pub calculated_electricity_monthly_mean_kwh: Option<f64>,
    /// Electricity lines (all-electric heat pumps, Bijlage C.2 example 2),
    /// on the monthly net delivery (delivered minus exported, C.3).
    pub measured_electricity_line: Option<RegressionLine>,
    pub calculated_electricity_line: Option<RegressionLine>,
    pub measured_electricity_base_load: Option<f64>,
    pub calculated_electricity_base_load: Option<f64>,
    /// ISSO 82.2 Bijlage C.1 (p. 105) fit criteria.
    pub criteria: FitCriteria,
}

/// ISSO 82.2 Bijlage C.1: annual use per carrier within 5 %, slope within
/// 5 %, heating limit within 1 °C, base line within 5 %. `None` when the
/// measurement or the calculated line is missing.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FitCriteria {
    pub annual_gas: Option<bool>,
    pub annual_electricity: Option<bool>,
    pub annual_heat: Option<bool>,
    pub gas_slope: Option<bool>,
    pub gas_heating_limit: Option<bool>,
    pub gas_base_line: Option<bool>,
    pub heat_slope: Option<bool>,
    pub heat_heating_limit: Option<bool>,
    pub heat_base_line: Option<bool>,
    pub electricity_slope: Option<bool>,
    pub electricity_heating_limit: Option<bool>,
    pub electricity_base_line: Option<bool>,
    /// All evaluated criteria met; `None` when none could be evaluated.
    pub within_criteria: Option<bool>,
}

const FIT_TOLERANCE_PERCENT: f64 = 5.0;
const FIT_TOLERANCE_LIMIT_K: f64 = 1.0;

fn within_percent(calculated: Option<f64>, measured: Option<f64>) -> Option<bool> {
    match (calculated, measured) {
        (Some(c), Some(m)) if m.abs() > 1e-9 => {
            Some(((c - m) / m * 100.0).abs() <= FIT_TOLERANCE_PERCENT + 1e-9)
        }
        (Some(c), Some(_)) => Some(c.abs() <= 1e-9),
        _ => None,
    }
}

fn within_kelvin(calculated: Option<f64>, measured: Option<f64>) -> Option<bool> {
    match (calculated, measured) {
        (Some(c), Some(m)) => Some((c - m).abs() <= FIT_TOLERANCE_LIMIT_K + 1e-9),
        _ => None,
    }
}

/// §3.2.7: the heating limit is the knee where the line reaches the base
/// load (hot water and cooking), θ = (a − base)/(−b); without a base load
/// the zero crossing.
fn with_base(line: Option<RegressionLine>, base: Option<f64>) -> Option<RegressionLine> {
    line.map(|mut line| {
        let level = base.unwrap_or(0.0);
        line.heating_limit_c =
            (line.slope_m3_per_k < 0.0).then(|| (line.intercept_m3 - level) / -line.slope_m3_per_k);
        line
    })
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionLine {
    /// Slope per K in the unit of the series (m³ gas, kWh heat or
    /// electricity per month).
    pub slope_m3_per_k: f64,
    pub intercept_m3: f64,
    /// Heating limit: outdoor temperature where the line meets the base
    /// load (§3.2.7), or zero without a base load.
    pub heating_limit_c: Option<f64>,
    pub points: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advice {
    pub package_id: Option<String>,
    pub chosen_by: &'static str,
    pub motivation: Option<String>,
    pub warnings: Vec<String>,
    pub specialist_notes: Vec<String>,
    pub notes: Vec<String>,
}

/// One requirement of the renovation passport.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassportRequirement {
    pub code: &'static str,
    /// `None` when it depends on a statement that was not given.
    pub met: Option<bool>,
    pub detail: Option<String>,
}

/// ISSO 82.2 §1.10 / §4.4: stacked steps (step 2 includes step 1, step 3
/// includes steps 1 and 2) with their NTA 8800 labels and actual-use
/// energy, and the requirements for registration.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenovationPassport {
    pub steps: Vec<VariantResult>,
    pub requirements: Vec<PassportRequirement>,
    /// All requirements met; `None` while a statement is missing.
    pub eligible: Option<bool>,
    /// Points the advice must cover (§1.10.2), paraphrased.
    pub required_statements: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaatwerkadviesAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub attest_status: &'static str,
    pub current: Option<VariantResult>,
    pub measures: Vec<VariantResult>,
    pub packages: Vec<VariantResult>,
    pub fit_check: Option<FitCheck>,
    pub advice: Option<Advice>,
    pub renovation_passport: Option<RenovationPassport>,
    pub interpretations: Vec<&'static str>,
    pub issues: Vec<MwaIssue>,
}

// ---------------------------------------------------------------- patches

fn split_pointer(path: &str) -> Option<(String, String)> {
    if !path.starts_with('/') {
        return None;
    }
    let index = path.rfind('/')?;
    let parent = path[..index].to_string();
    let last = path[index + 1..].replace("~1", "/").replace("~0", "~");
    Some((parent, last))
}

/// Applies one RFC 6902 operation (replace, add, remove).
pub fn apply_patch(target: &mut Value, operation: &PatchOperation) -> Result<(), String> {
    match operation {
        PatchOperation::Replace { path, value } => {
            let slot = target
                .pointer_mut(path)
                .ok_or_else(|| format!("path not found: {path}"))?;
            *slot = value.clone();
            Ok(())
        }
        PatchOperation::Add { path, value } => {
            if path.is_empty() {
                *target = value.clone();
                return Ok(());
            }
            let (parent, last) = split_pointer(path).ok_or_else(|| format!("bad path: {path}"))?;
            let container = target
                .pointer_mut(&parent)
                .ok_or_else(|| format!("parent not found: {path}"))?;
            match container {
                Value::Object(map) => {
                    map.insert(last, value.clone());
                    Ok(())
                }
                Value::Array(items) => {
                    if last == "-" {
                        items.push(value.clone());
                        return Ok(());
                    }
                    let index: usize = last.parse().map_err(|_| format!("bad index: {path}"))?;
                    if index > items.len() {
                        return Err(format!("index out of range: {path}"));
                    }
                    items.insert(index, value.clone());
                    Ok(())
                }
                _ => Err(format!("parent is not a container: {path}")),
            }
        }
        PatchOperation::Remove { path } => {
            let (parent, last) = split_pointer(path).ok_or_else(|| format!("bad path: {path}"))?;
            let container = target
                .pointer_mut(&parent)
                .ok_or_else(|| format!("parent not found: {path}"))?;
            match container {
                Value::Object(map) => map
                    .remove(&last)
                    .map(|_| ())
                    .ok_or_else(|| format!("path not found: {path}")),
                Value::Array(items) => {
                    let index: usize = last.parse().map_err(|_| format!("bad index: {path}"))?;
                    if index >= items.len() {
                        return Err(format!("index out of range: {path}"));
                    }
                    items.remove(index);
                    Ok(())
                }
                _ => Err(format!("parent is not a container: {path}")),
            }
        }
    }
}

// --------------------------------------------------------------- profiles

/// Resolved actual-use parameters.
#[derive(Debug, Clone, Default)]
struct ResolvedUse {
    heating_setpoint_c: Option<f64>,
    cooling_setpoint_c: Option<f64>,
    reduced_setpoint_c: Option<f64>,
    day_reduction_h: Option<f64>,
    weekend_reduction_h: Option<f64>,
    spatial_fraction: Option<f64>,
    occupants: Option<f64>,
    internal_gain_per_person_w: Option<f64>,
    occupancy_appliance_w_per_m2: Option<f64>,
    hot_water_per_person_kwh: Option<f64>,
    hot_water_factor: Option<f64>,
    annual_hot_water_kwh: Option<f64>,
    lighting_hours_factor: Option<f64>,
}

/// Standard profile values for dwellings: ISSO 82.2 tables 2.3–2.6
/// (p. 36–38).
fn residential_profile(profile: UserProfile, apartment_building: bool) -> ResolvedUse {
    match profile {
        // §2.5.3: the NTA option carries the MWA practice correction for hot
        // water; taken as the average-profile value (interpretation).
        UserProfile::Nta => ResolvedUse {
            hot_water_per_person_kwh: Some(545.0),
            ..ResolvedUse::default()
        },
        UserProfile::EnergyConscious => ResolvedUse {
            heating_setpoint_c: Some(18.0),
            cooling_setpoint_c: Some(26.0),
            reduced_setpoint_c: Some(14.0),
            day_reduction_h: Some(12.0),
            weekend_reduction_h: Some(8.0),
            spatial_fraction: Some(0.8),
            internal_gain_per_person_w: Some(152.0),
            hot_water_per_person_kwh: Some(409.0),
            ..ResolvedUse::default()
        },
        UserProfile::Average => ResolvedUse {
            heating_setpoint_c: Some(20.0),
            cooling_setpoint_c: Some(24.0),
            reduced_setpoint_c: Some(16.0),
            day_reduction_h: Some(10.0),
            weekend_reduction_h: Some(0.0),
            spatial_fraction: Some(if apartment_building { 0.5 } else { 0.6 }),
            internal_gain_per_person_w: Some(180.0),
            hot_water_per_person_kwh: Some(545.0),
            ..ResolvedUse::default()
        },
        UserProfile::NotEnergyConscious => ResolvedUse {
            heating_setpoint_c: Some(22.0),
            cooling_setpoint_c: Some(22.0),
            reduced_setpoint_c: Some(18.0),
            day_reduction_h: Some(8.0),
            weekend_reduction_h: Some(0.0),
            spatial_fraction: Some(0.0),
            internal_gain_per_person_w: Some(208.0),
            hot_water_per_person_kwh: Some(681.0),
            ..ResolvedUse::default()
        },
    }
}

/// Standard profile values for utility buildings relative to NTA 8800:
/// ISSO 75.2 tables 2.3–2.5 (p. 42–43).
fn utility_profile(profile: UserProfile, demand: &MonthlyDemandInput) -> ResolvedUse {
    let base = crate::monthly_demand::function_profile(demand);
    let (sign, hours_factor, weekend_shift, hot_water) = match profile {
        UserProfile::Nta | UserProfile::Average => return ResolvedUse::default(),
        UserProfile::EnergyConscious => (-1.0, 1.2, 24.0, 0.8),
        UserProfile::NotEnergyConscious => (1.0, 0.8, -24.0, 1.2),
    };
    ResolvedUse {
        heating_setpoint_c: Some(base.heating_setpoint_c + 2.0 * sign),
        cooling_setpoint_c: Some(base.cooling_setpoint_c - 2.0 * sign),
        reduced_setpoint_c: Some(base.reduced_setpoint_c + 2.0 * sign),
        day_reduction_h: Some((base.day_reduction_h * hours_factor).clamp(0.0, 24.0)),
        weekend_reduction_h: Some((base.weekend_reduction_h + weekend_shift).clamp(0.0, 48.0)),
        hot_water_factor: Some(hot_water),
        // 75.2 table 2.7: NTA burning hours −20 % / +20 %.
        lighting_hours_factor: Some(match profile {
            UserProfile::EnergyConscious => 0.8,
            _ => 1.2,
        }),
        ..ResolvedUse::default()
    }
}

fn resolve_use(profile: &UsageProfile, input: &BuildingPerformanceInput) -> ResolvedUse {
    let residential = matches!(input.calculation_scope, CalculationScope::Residential);
    let demand = &input.space_heating.demand;
    let mut resolved = if residential {
        residential_profile(
            profile.profile,
            demand.dwelling_type == Some(crate::monthly_demand::DwellingType::ApartmentBuilding),
        )
    } else {
        utility_profile(profile.profile, demand)
    };
    let set = |slot: &mut Option<f64>, value: Option<f64>| {
        if value.is_some() {
            *slot = value;
        }
    };
    set(&mut resolved.heating_setpoint_c, profile.heating_setpoint_c);
    set(&mut resolved.cooling_setpoint_c, profile.cooling_setpoint_c);
    set(&mut resolved.reduced_setpoint_c, profile.reduced_setpoint_c);
    set(&mut resolved.day_reduction_h, profile.day_reduction_h);
    set(
        &mut resolved.weekend_reduction_h,
        profile.weekend_reduction_h,
    );
    set(&mut resolved.spatial_fraction, profile.spatial_fraction);
    set(&mut resolved.occupants, profile.occupants);
    set(
        &mut resolved.internal_gain_per_person_w,
        profile.internal_gain_per_person_w,
    );
    set(
        &mut resolved.occupancy_appliance_w_per_m2,
        profile.occupancy_appliance_w_per_m2,
    );
    set(
        &mut resolved.hot_water_per_person_kwh,
        profile.hot_water_need_per_person_kwh,
    );
    set(
        &mut resolved.annual_hot_water_kwh,
        profile.annual_hot_water_need_kwh,
    );
    set(
        &mut resolved.lighting_hours_factor,
        profile.lighting_hours_factor,
    );
    resolved
}

fn zone_occupants(demand: &MonthlyDemandInput) -> Option<f64> {
    match &demand.internal_gains {
        InternalGains::Residential { dwelling_count, .. } if *dwelling_count > 0 => {
            let dwellings = f64::from(*dwelling_count);
            Some(dwellings * occupants_per_dwelling(demand.usable_floor_area_m2 / dwellings))
        }
        _ => None,
    }
}

/// Applies the actual-use layer to a building input (never to a label run).
fn apply_use(input: &mut BuildingPerformanceInput, profile: &UsageProfile) {
    let resolved = resolve_use(profile, input);
    let nta_hot_water_need = hot_water_need_kwh(input);
    let residential = matches!(input.calculation_scope, CalculationScope::Residential);
    let total_area: f64 = input
        .zone_inputs()
        .into_iter()
        .map(|demand| demand.usable_floor_area_m2)
        .sum();
    let nta_occupants: f64 = input
        .zone_inputs()
        .into_iter()
        .filter_map(zone_occupants)
        .sum();
    let fit_zone = |demand: &mut MonthlyDemandInput| {
        if let Some(value) = resolved.heating_setpoint_c {
            demand.setpoints.heating_c = value;
        }
        if let Some(value) = resolved.cooling_setpoint_c {
            demand.setpoints.cooling_c = value;
        }
        let share = if total_area > 0.0 {
            demand.usable_floor_area_m2 / total_area
        } else {
            0.0
        };
        let residential_zone = demand.usage_function.is_residential();
        // ISSO 75.2 table 2.6: N_p·q_oc;p·f_t/A_g + q_A per zone; a free
        // q_Oc·f_τ + q_A takes precedence.
        let per_person = (!residential_zone
            && (profile.persons.is_some() || profile.appliance_w_per_m2.is_some()))
        .then(|| {
            let (f_tau, table_q_a) = crate::monthly_demand::occupancy_time_and_appliances(demand);
            let q_a = profile.appliance_w_per_m2.unwrap_or(table_q_a);
            let area = demand.usable_floor_area_m2;
            let occupancy = match profile.persons {
                Some(total) if area > 0.0 => {
                    total
                        * share
                        * profile
                            .heat_per_person_w
                            .unwrap_or(UTILITY_HEAT_PER_PERSON_W)
                        * profile.occupancy_time_fraction.unwrap_or(f_tau)
                        / area
                }
                _ => {
                    crate::monthly_demand::function_profile(demand).occupancy_appliance_w_per_m2
                        - table_q_a
                }
            };
            occupancy + q_a
        });
        demand.usage_fit = Some(UsageFit {
            reduced_setpoint_c: resolved.reduced_setpoint_c,
            day_reduction_h: resolved.day_reduction_h,
            weekend_reduction_h: resolved.weekend_reduction_h,
            cooling_weekend_reduction_h: None,
            spatial_fraction: resolved.spatial_fraction.filter(|_| residential_zone),
            occupants: resolved
                .occupants
                .map(|total| total * share)
                .filter(|_| residential_zone),
            internal_gain_per_person_w: resolved
                .internal_gain_per_person_w
                .filter(|_| residential_zone),
            occupancy_appliance_w_per_m2: resolved
                .occupancy_appliance_w_per_m2
                .or(per_person)
                .filter(|_| !residential_zone),
            // ISSO 82.2 table 2.2: no practice factors under the NTA 8800
            // option ("–"); standard values for the standard profiles.
            ventilation_practice: profile.ventilation_practice.clone().or_else(|| {
                (profile.profile != UserProfile::Nta)
                    .then(crate::ventilation::VentilationPractice::default)
            }),
            source_reference: format!("maatwerkadvies: {}", profile.source_reference),
        });
    };
    fit_zone(&mut input.space_heating.demand);
    for zone in input.space_heating.additional_zones.iter_mut() {
        fit_zone(&mut zone.demand);
    }
    // §9.2: the zones of further heating systems take the same fit.
    for system in input.additional_heating_systems.iter_mut() {
        fit_zone(&mut system.demand);
        for zone in system.additional_zones.iter_mut() {
            fit_zone(&mut zone.demand);
        }
    }
    // ISSO 75.2 table 2.7: burning hours of chapter 14.
    if let Some(factor) = resolved.lighting_hours_factor.filter(|_| !residential) {
        for zone in input.lighting.iter_mut() {
            zone.burning_hours_factor = Some(factor);
        }
    }
    // Hot water: Q_W;nd;spec × N_p (dwellings) or a factor on the NTA need.
    let annual = resolved.annual_hot_water_kwh.or_else(|| {
        if residential {
            resolved
                .hot_water_per_person_kwh
                .map(|per_person| per_person * resolved.occupants.unwrap_or(nta_occupants))
        } else {
            None
        }
    });
    let factor = resolved.hot_water_factor;
    let target = match (annual, factor) {
        (Some(value), _) => Some(value),
        (None, Some(factor)) => nta_hot_water_need.map(|nta| factor * nta),
        _ => None,
    };
    if let (Some(target), Some(_)) = (target, &input.hot_water) {
        input.hot_water_need_fit = Some(HotWaterNeedFit {
            annual_need_kwh: target,
            source_reference: format!("maatwerkadvies: {}", profile.source_reference),
        });
    }
}

// ------------------------------------------------------------------ runs

fn energy_use(result: &BuildingPerformanceAssessment, tariffs: &Tariffs) -> EnergyUse {
    let mut usage = EnergyUse {
        monthly_gas_m3: vec![0.0; 12],
        monthly_electricity_import_kwh: vec![0.0; 12],
        monthly_heat_kwh: vec![0.0; 12],
        monthly_electricity_export_kwh: vec![0.0; 12],
        ..EnergyUse::default()
    };
    for row in &result.carriers {
        match row.carrier {
            "gas" => {
                usage.gas_kwh += row.used_kwh;
                let month = usize::from(row.month.saturating_sub(1)).min(11);
                usage.monthly_gas_m3[month] += row.used_kwh / GAS_KWH_PER_M3;
            }
            "oil" => usage.oil_kwh += row.used_kwh,
            "bm" => usage.biomass_kwh += row.used_kwh,
            "dh" | "dw" => {
                usage.district_heat_kwh += row.used_kwh;
                let month = usize::from(row.month.saturating_sub(1)).min(11);
                usage.monthly_heat_kwh[month] += row.used_kwh;
            }
            "dc" => usage.district_cold_kwh += row.used_kwh,
            _ => {}
        }
    }
    for row in &result.electricity_balance {
        let imported = (row.used_kwh - row.self_used_kwh).max(0.0);
        let month = usize::from(row.month.saturating_sub(1)).min(11);
        usage.monthly_electricity_import_kwh[month] += imported;
        usage.monthly_electricity_export_kwh[month] += row.exported_kwh;
        usage.electricity_import_kwh += imported;
        usage.electricity_export_kwh += row.exported_kwh;
        usage.electricity_produced_kwh += row.produced_kwh;
    }
    usage.gas_m3 = usage.gas_kwh / GAS_KWH_PER_M3;
    usage.primary_fossil_kwh = result.annual_primary_fossil_kwh.unwrap_or(0.0);
    usage.co2_kg = result.annual_co2_kg.unwrap_or(0.0);
    usage.energy_cost_eur = usage.gas_m3 * tariffs.gas_eur_per_m3
        + if usage.gas_kwh > 0.0 {
            tariffs.gas_fixed_eur_per_year
        } else {
            0.0
        }
        + usage.electricity_import_kwh * tariffs.electricity_eur_per_kwh
        - usage.electricity_export_kwh * tariffs.electricity_export_eur_per_kwh
        + usage.district_heat_kwh * tariffs.district_heat_eur_per_kwh
        + if usage.district_heat_kwh > 0.0 {
            tariffs.heat_fixed_eur_per_year
        } else {
            0.0
        }
        + usage.district_cold_kwh * tariffs.district_cold_eur_per_kwh
        + usage.oil_kwh * tariffs.oil_eur_per_kwh
        + usage.biomass_kwh * tariffs.biomass_eur_per_kwh;
    usage
}

fn label_result(result: &BuildingPerformanceAssessment) -> LabelResult {
    LabelResult {
        label_class: result.indicative_label_class,
        need_indicator_kwh_per_m2: result.need_indicator_kwh_per_m2_year,
        primary_fossil_indicator_kwh_per_m2: result.primary_fossil_indicator_kwh_per_m2_year,
        renewable_share_percent: result.renewable_share_percent,
        tojuli_max_k: result.tojuli_max_k,
    }
}

struct RunOutcome {
    label: LabelResult,
    actual: Option<EnergyUse>,
    system_checks: Vec<SystemPerformanceCheck>,
    issues: Vec<MwaIssue>,
}

/// Bbl table 4.248 limits.
const LIMIT_HEATING: f64 = 1.31;
const LIMIT_COOLING: f64 = 1.33;
const LIMIT_VENTILATION_KWH_PER_M3H: f64 = 3.8;
const LIMIT_HOT_WATER: f64 = 3.45;
const LIMIT_LIGHTING_KWH_PER_M2: f64 = 75.0;

/// Omgevingsregeling bijlage VIII on a standard NTA 8800 result. Primary
/// energy uses the table 5.2 f_P;del factors (external heat by forfait);
/// CHP credits (E_H;WKK, E_W;WKK) are zero because the chain has no CHP
/// for heating or hot water.
pub fn system_performance_checks(
    input: &BuildingPerformanceInput,
    result: &BuildingPerformanceAssessment,
) -> Vec<SystemPerformanceCheck> {
    use crate::building_performance::{
        F_P_BIOMASS_B, F_P_DISTRICT_HEAT_FORFAIT, F_P_ELECTRICITY, F_P_GAS, F_P_OIL,
    };
    let residential = matches!(input.calculation_scope, CalculationScope::Residential);
    let check = |system: &'static str,
                 value: Option<f64>,
                 limit: Option<f64>,
                 unit: &'static str,
                 note: Option<&'static str>| SystemPerformanceCheck {
        system,
        value,
        limit,
        unit,
        meets: match (value, limit) {
            (Some(value), Some(limit)) => Some(value <= limit + 1e-9),
            _ => None,
        },
        note,
    };
    let ratio = |energy: f64, need: f64| (need > 0.0 && energy > 0.0).then(|| energy / need);
    // 1. Space heating: (E_H − E_H;WKK) / Q_H;nd without recoverable losses.
    let heating_energy: f64 = result
        .space_heating
        .monthly
        .iter()
        .map(|row| {
            F_P_GAS * row.natural_gas_kwh
                + F_P_OIL * row.oil_kwh
                + F_P_BIOMASS_B * row.biomass_kwh
                + row.biomass_class_c_kwh
                + F_P_DISTRICT_HEAT_FORFAIT * row.district_heat_kwh
                + F_P_ELECTRICITY
                    * (row.generator_electricity_kwh + row.auxiliary_electricity_kwh.unwrap_or(0.0))
        })
        .sum();
    let zones = || {
        std::iter::once(&result.space_heating.demand)
            .chain(&result.space_heating.additional_zone_demands)
    };
    let heating_need: f64 = zones()
        .filter_map(|zone| zone.annual_heating_need_without_recoverable_kwh)
        .sum();
    let mut checks = vec![check(
        "space_heating",
        ratio(heating_energy, heating_need),
        Some(LIMIT_HEATING),
        "-",
        None,
    )];
    // 2. Space cooling: the chapter 10 result is not part of the output.
    checks.push(check(
        "space_cooling",
        None,
        input.has_cooling().then_some(LIMIT_COOLING),
        "-",
        input
            .has_cooling()
            .then_some("chapter 10 primary energy per system is not reported; check separately"),
    ));
    // 3. Hot water: (E_W − E_W;WKK) / Q_W;nd.
    let hot_water = result.hot_water.as_ref().map(|water| {
        let energy: f64 = water
            .months
            .iter()
            .map(|row| {
                F_P_ELECTRICITY * (row.electricity_kwh + row.auxiliary_electricity_kwh)
                    + F_P_GAS * row.natural_gas_kwh
                    + F_P_OIL * row.oil_kwh
                    + F_P_DISTRICT_HEAT_FORFAIT * row.district_heat_kwh
            })
            .sum();
        ratio(energy, water.annual_net_need_kwh)
    });
    checks.push(check(
        "hot_water",
        hot_water.flatten(),
        hot_water.map(|_| LIMIT_HOT_WATER),
        "-",
        None,
    ));
    // 4. Ventilation (utility): E_V / q_V;ODA;req in kWh/(m³/h).
    let ventilation = zones().filter_map(|zone| zone.ventilation.as_ref()).fold(
        None::<(f64, f64)>,
        |total, item| {
            let energy: f64 = item
                .months
                .iter()
                .map(|row| {
                    F_P_ELECTRICITY
                        * (row.fan_electricity_kwh
                            + row.frost_protection_electricity_kwh
                            + row.grille_preheating_electricity_kwh)
                })
                .sum();
            let flow = item
                .months
                .iter()
                .map(|row| row.heating.required_outdoor_air_m3_per_h)
                .sum::<f64>()
                / item.months.len().max(1) as f64;
            let (e, q) = total.unwrap_or((0.0, 0.0));
            Some((e + energy, q + flow))
        },
    );
    checks.push(check(
        "ventilation",
        ventilation.and_then(|(energy, flow)| ratio(energy, flow)),
        (!residential && ventilation.is_some()).then_some(LIMIT_VENTILATION_KWH_PER_M3H),
        "kWh/(m3/h)",
        None,
    ));
    // 5. Built-in lighting (utility): E_L / A_g in kWh_prim/m².
    let lighting: f64 = result.lighting.iter().map(|zone| zone.annual_kwh).sum();
    checks.push(check(
        "lighting",
        (!result.lighting.is_empty() && input.total_usable_floor_area_m2 > 0.0)
            .then(|| F_P_ELECTRICITY * lighting / input.total_usable_floor_area_m2),
        (!residential && !result.lighting.is_empty()).then_some(LIMIT_LIGHTING_KWH_PER_M2),
        "kWh/m2",
        Some("Bbl table 4.248 gives 75 kWh_prim/m2; ISSO 82.2 table 5.1 prints 17"),
    ));
    checks
}

/// Builds the building input of a variant from the base and the measures.
fn variant_input(
    base: &MwaBase,
    measures: &[&Measure],
    issues: &mut Vec<MwaIssue>,
) -> Option<BuildingPerformanceInput> {
    let mut building_value = match base {
        MwaBase::Project { project } => {
            let mut project = project.clone();
            for measure in measures.iter().filter(|m| m.target == PatchTarget::Project) {
                for (index, operation) in measure.patch.iter().enumerate() {
                    if let Err(message) = apply_patch(&mut project, operation) {
                        issues.push(MwaIssue {
                            code: "measure_patch_failed",
                            path: format!("measures[{}].patch[{index}]", measure.id),
                            detail: Some(message),
                        });
                    }
                }
            }
            let assessed = assess_project_performance(&project);
            let Some(derived) = assessed.derived_input else {
                for gap in assessed.gaps.iter().take(20) {
                    issues.push(MwaIssue {
                        code: "project_input_incomplete",
                        path: gap.path.clone(),
                        detail: Some(gap.code.to_string()),
                    });
                }
                if assessed.gaps.is_empty() {
                    issues.push(issue("project_input_incomplete", "base.project"));
                }
                return None;
            };
            serde_json::to_value(derived).expect("typed input serializes")
        }
        MwaBase::Building { input } => {
            if measures.iter().any(|m| m.target == PatchTarget::Project) {
                issues.push(issue("project_patch_without_project", "measures"));
            }
            input.clone()
        }
    };
    for measure in measures
        .iter()
        .filter(|m| m.target == PatchTarget::Building)
    {
        for (index, operation) in measure.patch.iter().enumerate() {
            if let Err(message) = apply_patch(&mut building_value, operation) {
                issues.push(MwaIssue {
                    code: "measure_patch_failed",
                    path: format!("measures[{}].patch[{index}]", measure.id),
                    detail: Some(message),
                });
            }
        }
    }
    match serde_json::from_value::<BuildingPerformanceInput>(building_value) {
        Ok(input) => Some(input),
        Err(message) => {
            issues.push(MwaIssue {
                code: "building_input_invalid",
                path: "base".into(),
                detail: Some(message.to_string()),
            });
            None
        }
    }
}

fn run_variant(
    input: &MaatwerkadviesInput,
    measures: &[&Measure],
    usage: Option<&UsageProfile>,
) -> RunOutcome {
    let mut issues = Vec::new();
    let Some(building) = variant_input(&input.base, measures, &mut issues) else {
        return RunOutcome {
            label: LabelResult::default(),
            actual: None,
            system_checks: Vec::new(),
            issues,
        };
    };
    if !issues.is_empty() {
        return RunOutcome {
            label: LabelResult::default(),
            actual: None,
            system_checks: Vec::new(),
            issues,
        };
    }
    // NTA 8800 run: label and indicators.
    let standard = assess_building_performance(&building);
    if standard.status != "calculated_unverified" {
        for item in standard.issues.iter().take(20) {
            issues.push(MwaIssue {
                code: "calculation_invalid",
                path: item.path.clone(),
                detail: Some(item.code.to_string()),
            });
        }
        return RunOutcome {
            label: LabelResult::default(),
            actual: None,
            system_checks: Vec::new(),
            issues,
        };
    }
    let label = label_result(&standard);
    let system_checks = system_performance_checks(&building, &standard);
    // Actual-use run.
    let actual = match usage {
        None => energy_use(&standard, &input.tariffs),
        Some(profile) => {
            let mut fitted = building.clone();
            apply_use(&mut fitted, profile);
            let result = assess_building_performance(&fitted);
            if result.status != "calculated_unverified" {
                for item in result.issues.iter().take(20) {
                    issues.push(MwaIssue {
                        code: "actual_use_invalid",
                        path: item.path.clone(),
                        detail: Some(item.code.to_string()),
                    });
                }
                return RunOutcome {
                    label,
                    actual: None,
                    system_checks,
                    issues,
                };
            }
            energy_use(&result, &input.tariffs)
        }
    };
    RunOutcome {
        label,
        actual: Some(actual),
        system_checks,
        issues,
    }
}

/// NTA annual net hot-water need of a building input, kWh.
fn hot_water_need_kwh(input: &BuildingPerformanceInput) -> Option<f64> {
    if !input.additional_hot_water_systems.is_empty() {
        // §13.2.4: all systems together.
        return crate::building_performance::hot_water_annual_need_kwh(input);
    }
    let system = input.hot_water.as_ref()?;
    let context = crate::domestic_hot_water::HotWaterContext {
        residential: matches!(input.calculation_scope, CalculationScope::Residential),
        usable_floor_area_m2: input.total_usable_floor_area_m2,
        heated_ambient_c: input.space_heating.demand.setpoints.heating_c,
        space_heating: None,
        standard_setpoint_c: None,
        levelled_setpoint_c: None,
        need_fraction: None,
    };
    crate::domestic_hot_water::assess_hot_water(system, context)
        .ok()
        .map(|result| result.annual_net_need_kwh)
}

// -------------------------------------------------------------- economics

/// Net present value of a set of measures with an annual saving.
pub fn net_present_value(
    measures: &[&Measure],
    annual_cost_saving_eur: f64,
    economics: &Economics,
    horizon_years: f64,
) -> f64 {
    let r = economics.discount_rate;
    let e = economics.energy_price_change;
    let years = horizon_years.max(0.0).floor() as u32;
    let maintenance: f64 = measures.iter().map(|m| m.maintenance_eur_per_year).sum();
    let m = economics.maintenance_price_change;
    let mut value = 0.0;
    for t in 1..=years {
        let t_f = f64::from(t);
        let saving = annual_cost_saving_eur * (1.0 + e).powf(t_f - 1.0);
        let upkeep = maintenance * (1.0 + m).powf(t_f - 1.0);
        value += (saving - upkeep) / (1.0 + r).powf(t_f);
    }
    for measure in measures {
        let lifetime = measure.lifetime_years.max(1.0);
        // Phasing: the investment starts in its phase year.
        let mut start = match (measure.phase_year, economics.base_year) {
            (Some(year), Some(base)) if year > base => f64::from(year - base),
            _ => 0.0,
        };
        while start < horizon_years - 1e-9 {
            value -= measure.investment_eur / (1.0 + r).powf(start);
            start += lifetime;
        }
        // Linear residual value of the last generation at the horizon.
        let last_start = start - lifetime;
        let used = horizon_years - last_start;
        if used < lifetime {
            let residual = measure.investment_eur * (1.0 - used / lifetime);
            value += residual / (1.0 + r).powf(horizon_years);
        }
    }
    value
}

fn savings(current: &EnergyUse, variant: &EnergyUse) -> Savings {
    Savings {
        gas_m3: current.gas_m3 - variant.gas_m3,
        electricity_kwh: (current.electricity_import_kwh - current.electricity_export_kwh)
            - (variant.electricity_import_kwh - variant.electricity_export_kwh),
        heat_kwh: current.district_heat_kwh - variant.district_heat_kwh,
        primary_fossil_kwh: current.primary_fossil_kwh - variant.primary_fossil_kwh,
        co2_kg: current.co2_kg - variant.co2_kg,
        energy_cost_eur: current.energy_cost_eur - variant.energy_cost_eur,
    }
}

fn phasing(measures: &[&Measure]) -> Vec<PhaseStep> {
    let mut years: Vec<Option<u32>> = measures.iter().map(|m| m.phase_year).collect();
    years.sort();
    years.dedup();
    years
        .into_iter()
        .map(|year| PhaseStep {
            year,
            measure_ids: measures
                .iter()
                .filter(|m| m.phase_year == year)
                .map(|m| m.id.clone())
                .collect(),
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn variant_result(
    input: &MaatwerkadviesInput,
    id: &str,
    name: &str,
    kind: &'static str,
    measures: &[&Measure],
    current: Option<&EnergyUse>,
) -> VariantResult {
    let usage = input.future_use.as_ref().or(input.current_use.as_ref());
    let outcome = run_variant(input, measures, usage);
    let investment: f64 = measures.iter().map(|m| m.investment_eur).sum();
    let maintenance: f64 = measures.iter().map(|m| m.maintenance_eur_per_year).sum();
    let horizon = input.economics.horizon_years.unwrap_or_else(|| {
        measures
            .iter()
            .map(|m| m.lifetime_years)
            .fold(0.0_f64, f64::max)
    });
    let savings = match (current, &outcome.actual) {
        (Some(current), Some(actual)) => Some(savings(current, actual)),
        _ => None,
    };
    let net_saving = savings
        .as_ref()
        .map(|item| item.energy_cost_eur - maintenance);
    let simple_payback = net_saving
        .filter(|value| *value > 0.0 && investment > 0.0)
        .map(|value| investment / value);
    let npv = savings
        .as_ref()
        .map(|item| net_present_value(measures, item.energy_cost_eur, &input.economics, horizon));
    VariantResult {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        measure_ids: measures.iter().map(|m| m.id.clone()).collect(),
        valid: outcome.issues.is_empty() && outcome.actual.is_some(),
        label: outcome.label,
        actual_use: outcome.actual,
        savings,
        investment_eur: investment,
        maintenance_eur_per_year: maintenance,
        simple_payback_years: simple_payback,
        net_present_value_eur: npv,
        horizon_years: horizon,
        phasing: phasing(measures),
        system_checks: outcome.system_checks,
        issues: outcome.issues,
    }
}

/// Bijlage C.1 criteria on a fit check.
fn fit_criteria(check: &FitCheck, measured: Option<&MeasuredUse>) -> FitCriteria {
    let annual = |given: Option<f64>, monthly: &[Option<f64>]| {
        given.or_else(|| {
            (monthly.len() == 12 && monthly.iter().all(Option::is_some))
                .then(|| monthly.iter().map(|value| value.unwrap_or(0.0)).sum())
        })
    };
    let measured_gas = measured.and_then(|m| annual(m.annual_gas_m3, &m.monthly_gas_m3));
    let measured_el =
        measured.and_then(|m| annual(m.annual_electricity_kwh, &m.monthly_electricity_kwh));
    let measured_heat = measured.and_then(|m| annual(m.annual_heat_kwh, &m.monthly_heat_kwh));
    let slope = |a: &Option<RegressionLine>, b: &Option<RegressionLine>| {
        within_percent(a.map(|l| l.slope_m3_per_k), b.map(|l| l.slope_m3_per_k))
    };
    let limit = |a: &Option<RegressionLine>, b: &Option<RegressionLine>| {
        within_kelvin(
            a.and_then(|l| l.heating_limit_c),
            b.and_then(|l| l.heating_limit_c),
        )
    };
    let calculated = &check.calculated;
    let mut criteria = FitCriteria {
        annual_gas: within_percent(measured_gas.map(|_| calculated.gas_m3), measured_gas),
        annual_electricity: within_percent(
            measured_el
                .map(|_| calculated.electricity_import_kwh - calculated.electricity_export_kwh),
            measured_el,
        ),
        annual_heat: within_percent(
            measured_heat.map(|_| calculated.district_heat_kwh),
            measured_heat,
        ),
        gas_slope: slope(&check.calculated_gas_line, &check.measured_gas_line),
        gas_heating_limit: limit(&check.calculated_gas_line, &check.measured_gas_line),
        gas_base_line: within_percent(check.calculated_gas_base_load, check.measured_gas_base_load),
        heat_slope: slope(&check.calculated_heat_line, &check.measured_heat_line),
        heat_heating_limit: limit(&check.calculated_heat_line, &check.measured_heat_line),
        heat_base_line: within_percent(
            check.calculated_heat_base_load,
            check.measured_heat_base_load,
        ),
        electricity_slope: slope(
            &check.calculated_electricity_line,
            &check.measured_electricity_line,
        ),
        electricity_heating_limit: limit(
            &check.calculated_electricity_line,
            &check.measured_electricity_line,
        ),
        electricity_base_line: within_percent(
            check.calculated_electricity_base_load,
            check.measured_electricity_base_load,
        ),
        within_criteria: None,
    };
    let all = [
        criteria.annual_gas,
        criteria.annual_electricity,
        criteria.annual_heat,
        criteria.gas_slope,
        criteria.gas_heating_limit,
        criteria.gas_base_line,
        criteria.heat_slope,
        criteria.heat_heating_limit,
        criteria.heat_base_line,
        criteria.electricity_slope,
        criteria.electricity_heating_limit,
        criteria.electricity_base_line,
    ];
    criteria.within_criteria = if all.contains(&Some(false)) {
        Some(false)
    } else if all.iter().any(Option::is_some) {
        Some(true)
    } else {
        None
    };
    criteria
}

// ------------------------------------------------------ renovation passport

/// §1.10.2: statements the advice must contain (paraphrased).
const PASSPORT_STATEMENTS: &[&str] = &[
    "TO-juli, GTO and ATG only indicate the overheating risk and are limited for existing buildings",
    "the owner is pointed to risks the calculation may not show, such as the need for solar shading or ventilation",
    "the overheating calculation uses outdated climate data and gives no guarantee",
    "the building may be more sensitive to overheating in practice; further dynamic studies may be advised",
    "the stacked labels follow the standard NTA 8800 calculation and can differ from the passport's energy use",
];

fn renovation_passport(
    input: &MaatwerkadviesInput,
    passport: &RenovationPassportInput,
    current: &EnergyUse,
) -> RenovationPassport {
    let package = |id: &str| input.packages.iter().find(|item| item.id == id);
    let measures_of = |ids: &[&str]| -> Vec<&Measure> {
        let mut out: Vec<&Measure> = Vec::new();
        for id in ids {
            if let Some(item) = package(id) {
                for measure_id in &item.measure_ids {
                    if let Some(measure) = input.measures.iter().find(|m| &m.id == measure_id) {
                        if !out.iter().any(|m| m.id == measure.id) {
                            out.push(measure);
                        }
                    }
                }
            }
        }
        out
    };
    let ids = [
        passport.demand_package_id.as_str(),
        passport.systems_package_id.as_str(),
        passport.production_package_id.as_str(),
    ];
    let names = [
        "Stap 1: beperken warmte- en koudevraag",
        "Stap 2: duurzame verwarming, tapwater en koeling",
        "Stap 3: gebouwgebonden opwekking en opslag",
    ];
    let steps: Vec<VariantResult> = (0..3)
        .map(|step| {
            let measures = measures_of(&ids[..=step]);
            variant_result(
                input,
                &format!("passport-step-{}", step + 1),
                names[step],
                "passport_step",
                &measures,
                Some(current),
            )
        })
        .collect();
    let mut requirements = Vec::new();
    let mut require = |code: &'static str, met: Option<bool>, detail: Option<String>| {
        requirements.push(PassportRequirement { code, met, detail })
    };
    let missing: Vec<&str> = ids
        .iter()
        .copied()
        .filter(|id| package(id).is_none())
        .collect();
    require(
        "three_steps_present",
        Some(missing.is_empty() && steps.iter().all(|step| step.valid)),
        (!missing.is_empty()).then(|| format!("unknown packages: {}", missing.join(", "))),
    );
    // Step 1: insulation standard (post-1945, or pre-war with motivation).
    let insulation = match passport.insulation_standard_max_need_kwh_per_m2 {
        Some(limit) => steps[0]
            .label
            .need_indicator_kwh_per_m2
            .map(|need| need <= limit + 1e-9),
        None => passport.insulation_standard_met,
    };
    require(
        "insulation_standard",
        insulation,
        passport
            .insulation_standard_max_need_kwh_per_m2
            .map(|limit| {
                format!(
                    "step 1 net heat need {:?} kWh/m2 against {limit} kWh/m2",
                    steps[0].label.need_indicator_kwh_per_m2
                )
            }),
    );
    if passport.prewar_standard {
        require(
            "prewar_standard_motivated",
            Some(
                passport
                    .prewar_motivation
                    .as_ref()
                    .is_some_and(|text| !text.trim().is_empty()),
            ),
            None,
        );
    }
    let step_one = package(&passport.demand_package_id);
    let overheating = !passport.overheating_measure_ids.is_empty()
        && passport
            .overheating_measure_ids
            .iter()
            .all(|id| step_one.is_some_and(|item| item.measure_ids.iter().any(|m| m == id)));
    require("overheating_measures", Some(overheating), None);
    // Step 2/3: no combustion of natural gas or oil on site (biomass and
    // biogas excepted); a hybrid heat pump is allowed with the pre-war
    // standard.
    let fossil = |step: &VariantResult| {
        step.actual_use
            .as_ref()
            .map(|use_| use_.gas_kwh > 1e-6 || use_.oil_kwh > 1e-6)
    };
    require(
        "natural_gas_free_main_heating",
        if passport.prewar_standard {
            Some(true)
        } else {
            fossil(&steps[1]).map(|uses| !uses)
        },
        passport
            .prewar_standard
            .then(|| "pre-war standard: hybrid heat pump allowed".to_string()),
    );
    require(
        "emission_free_result",
        if passport.prewar_standard {
            None
        } else {
            fossil(&steps[2]).map(|uses| !uses)
        },
        passport
            .prewar_standard
            .then(|| "pre-war standard: hybrid allowed; adviser judgement".to_string()),
    );
    // Step 3: renewable production added and storage considered.
    let production = match (&steps[1].actual_use, &steps[2].actual_use) {
        (Some(before), Some(after)) => {
            let solar = measures_of(&ids[2..]).iter().any(|measure| {
                matches!(
                    measure.category,
                    MeasureCategory::Pv | MeasureCategory::SolarThermal
                )
            });
            Some(after.electricity_produced_kwh > before.electricity_produced_kwh + 1e-6 || solar)
        }
        _ => None,
    };
    require("renewable_production", production, None);
    require(
        "storage_considered",
        Some(passport.storage_considered),
        None,
    );
    let eligible = if requirements.iter().any(|item| item.met == Some(false)) {
        Some(false)
    } else if requirements.iter().all(|item| item.met.is_some()) {
        Some(true)
    } else {
        None
    };
    RenovationPassport {
        steps,
        requirements,
        eligible,
        required_statements: PASSPORT_STATEMENTS.to_vec(),
    }
}

// --------------------------------------------------------------- fit check

/// Least squares of monthly gas against the outdoor temperature for the
/// months below 15 °C (heating season).
pub fn gas_regression(monthly_m3: &[Option<f64>]) -> Option<RegressionLine> {
    regression(monthly_m3, &[])
}

/// Month temperature: the local value when given, else table 17.1.
fn month_temperature(local: &[Option<f64>], index: usize) -> f64 {
    local
        .get(index)
        .copied()
        .flatten()
        .unwrap_or(OUTDOOR_TEMPERATURE_C[index])
}

/// Least squares of a monthly series against the (local) outdoor
/// temperature for the months below 15 °C; the units follow the series.
pub fn regression(monthly: &[Option<f64>], local_c: &[Option<f64>]) -> Option<RegressionLine> {
    let points: Vec<(f64, f64)> = monthly
        .iter()
        .enumerate()
        .take(12)
        .filter_map(|(index, value)| value.map(|v| (month_temperature(local_c, index), v)))
        .filter(|(theta, _)| *theta < 15.0)
        .collect();
    if points.len() < 2 {
        return None;
    }
    let n = points.len() as f64;
    let mean_x = points.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y = points.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|p| (p.0 - mean_x).powi(2)).sum();
    if sxx <= 0.0 {
        return None;
    }
    let sxy: f64 = points.iter().map(|p| (p.0 - mean_x) * (p.1 - mean_y)).sum();
    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    Some(RegressionLine {
        slope_m3_per_k: slope,
        intercept_m3: intercept,
        heating_limit_c: (slope < 0.0).then(|| -intercept / slope),
        points: points.len(),
    })
}

/// Mean of the months at or above 15 °C (outside the heating season).
fn base_load(monthly: &[Option<f64>], local_c: &[Option<f64>]) -> Option<f64> {
    let values: Vec<f64> = monthly
        .iter()
        .enumerate()
        .take(12)
        .filter(|(index, _)| month_temperature(local_c, *index) >= 15.0)
        .filter_map(|(_, value)| *value)
        .collect();
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn mean_of(values: &[Option<f64>]) -> Option<f64> {
    let present: Vec<f64> = values.iter().take(12).filter_map(|value| *value).collect();
    (!present.is_empty()).then(|| present.iter().sum::<f64>() / present.len() as f64)
}

fn deviation(calculated: f64, measured: Option<f64>) -> Option<f64> {
    measured
        .filter(|value| *value > 0.0)
        .map(|value| (calculated - value) / value * 100.0)
}

// ----------------------------------------------------------------- driver

fn validate(input: &MaatwerkadviesInput, issues: &mut Vec<MwaIssue>) {
    let mut ids = HashSet::new();
    for (index, measure) in input.measures.iter().enumerate() {
        let path = format!("measures[{index}]");
        if measure.id.trim().is_empty() || !ids.insert(measure.id.as_str()) {
            issues.push(issue("measure_id_invalid", format!("{path}.id")));
        }
        if !measure.investment_eur.is_finite() || measure.investment_eur < 0.0 {
            issues.push(issue(
                "measure_investment_invalid",
                format!("{path}.investmentEur"),
            ));
        }
        if !measure.lifetime_years.is_finite() || measure.lifetime_years <= 0.0 {
            issues.push(issue(
                "measure_lifetime_invalid",
                format!("{path}.lifetimeYears"),
            ));
        }
        if !measure.maintenance_eur_per_year.is_finite() {
            issues.push(issue(
                "measure_maintenance_invalid",
                format!("{path}.maintenanceEurPerYear"),
            ));
        }
        if measure.cost_source.trim().is_empty() {
            // BRL 9500-MWA-W §4.2.5: cost figures must be accounted for.
            issues.push(issue(
                "measure_cost_source_required",
                format!("{path}.costSource"),
            ));
        }
        if measure.patch.is_empty() {
            issues.push(issue("measure_patch_required", format!("{path}.patch")));
        }
    }
    let mut package_ids = HashSet::new();
    for (index, package) in input.packages.iter().enumerate() {
        let path = format!("packages[{index}]");
        if package.id.trim().is_empty() || !package_ids.insert(package.id.as_str()) {
            issues.push(issue("package_id_invalid", format!("{path}.id")));
        }
        if package.measure_ids.is_empty() {
            issues.push(issue(
                "package_measures_required",
                format!("{path}.measureIds"),
            ));
        }
        for (m_index, id) in package.measure_ids.iter().enumerate() {
            if !ids.contains(id.as_str()) {
                issues.push(issue(
                    "package_measure_unknown",
                    format!("{path}.measureIds[{m_index}]"),
                ));
            }
        }
    }
    if let Some(id) = &input.advised_package_id {
        if !package_ids.contains(id.as_str()) {
            issues.push(issue("advised_package_unknown", "advisedPackageId"));
        }
    }
    let tariffs = &input.tariffs;
    for (value, path) in [
        (tariffs.gas_eur_per_m3, "tariffs.gasEurPerM3"),
        (
            tariffs.electricity_eur_per_kwh,
            "tariffs.electricityEurPerKwh",
        ),
        (
            tariffs.electricity_export_eur_per_kwh,
            "tariffs.electricityExportEurPerKwh",
        ),
        (
            tariffs.district_heat_eur_per_kwh,
            "tariffs.districtHeatEurPerKwh",
        ),
        (
            tariffs.district_cold_eur_per_kwh,
            "tariffs.districtColdEurPerKwh",
        ),
        (tariffs.oil_eur_per_kwh, "tariffs.oilEurPerKwh"),
        (tariffs.biomass_eur_per_kwh, "tariffs.biomassEurPerKwh"),
        (tariffs.gas_fixed_eur_per_year, "tariffs.gasFixedEurPerYear"),
        (
            tariffs.heat_fixed_eur_per_year,
            "tariffs.heatFixedEurPerYear",
        ),
    ] {
        if !value.is_finite() || value < 0.0 {
            issues.push(issue("tariff_invalid", path));
        }
    }
    if tariffs.source_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            "tariffs.sourceReference",
        ));
    }
    let economics = &input.economics;
    if !economics.discount_rate.is_finite() || !(-0.5..=1.0).contains(&economics.discount_rate) {
        issues.push(issue("economics_invalid", "economics.discountRate"));
    }
    if !economics.energy_price_change.is_finite()
        || !(-0.5..=1.0).contains(&economics.energy_price_change)
    {
        issues.push(issue("economics_invalid", "economics.energyPriceChange"));
    }
    if economics
        .horizon_years
        .is_some_and(|years| !years.is_finite() || !(1.0..=100.0).contains(&years))
    {
        issues.push(issue("economics_invalid", "economics.horizonYears"));
    }
    for (name, profile) in [
        ("currentUse", &input.current_use),
        ("futureUse", &input.future_use),
    ] {
        if let Some(profile) = profile {
            if profile.source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{name}.sourceReference"),
                ));
            }
        }
    }
    if let Some(measured) = &input.measured {
        // Electricity is the net delivery per month (Bijlage C.3) and may be
        // negative with on-site production.
        for (field, values, signed) in [
            ("monthlyGasM3", &measured.monthly_gas_m3, false),
            (
                "monthlyElectricityKwh",
                &measured.monthly_electricity_kwh,
                true,
            ),
            ("monthlyHeatKwh", &measured.monthly_heat_kwh, false),
        ] {
            if !values.is_empty() && values.len() != 12 {
                issues.push(issue(
                    "measured_monthly_length",
                    format!("measured.{field}"),
                ));
            }
            if values
                .iter()
                .flatten()
                .any(|value| !value.is_finite() || (!signed && *value < 0.0))
            {
                issues.push(issue("measured_value_invalid", format!("measured.{field}")));
            }
        }
        let temperatures = &measured.monthly_outdoor_temperature_c;
        if !temperatures.is_empty() && temperatures.len() != 12 {
            issues.push(issue(
                "measured_monthly_length",
                "measured.monthlyOutdoorTemperatureC",
            ));
        }
    }
}

pub fn assess_maatwerkadvies(input: &MaatwerkadviesInput) -> MaatwerkadviesAssessment {
    let fingerprint =
        input_fingerprint(&serde_json::to_value(input).expect("typed input serializes"));
    let mut issues = Vec::new();
    validate(input, &mut issues);
    let empty = |issues: Vec<MwaIssue>| MaatwerkadviesAssessment {
        status: "invalid",
        scope: SCOPE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint.clone(),
        attest_status: "unattested",
        current: None,
        measures: Vec::new(),
        packages: Vec::new(),
        fit_check: None,
        advice: None,
        renovation_passport: None,
        interpretations: INTERPRETATIONS.to_vec(),
        issues,
    };
    if !issues.is_empty() {
        return empty(issues);
    }
    // Current situation with the current use.
    let current_outcome = run_variant(input, &[], input.current_use.as_ref());
    if !current_outcome.issues.is_empty() || current_outcome.actual.is_none() {
        let mut issues = current_outcome.issues;
        issues.push(issue("current_situation_invalid", "base"));
        return empty(issues);
    }
    let current_use = current_outcome.actual.clone().expect("checked");
    let current = VariantResult {
        id: "current".into(),
        name: "Huidige situatie".into(),
        kind: "current",
        measure_ids: Vec::new(),
        valid: true,
        label: current_outcome.label.clone(),
        actual_use: Some(current_use.clone()),
        savings: None,
        investment_eur: 0.0,
        maintenance_eur_per_year: 0.0,
        simple_payback_years: None,
        net_present_value_eur: None,
        horizon_years: 0.0,
        phasing: Vec::new(),
        system_checks: current_outcome.system_checks.clone(),
        issues: Vec::new(),
    };
    let measure_results: Vec<VariantResult> = input
        .measures
        .iter()
        .map(|measure| {
            variant_result(
                input,
                &measure.id,
                &measure.name,
                "measure",
                &[measure],
                Some(&current_use),
            )
        })
        .collect();
    let package_results: Vec<VariantResult> = input
        .packages
        .iter()
        .map(|package| {
            let measures: Vec<&Measure> = package
                .measure_ids
                .iter()
                .filter_map(|id| input.measures.iter().find(|m| &m.id == id))
                .collect();
            variant_result(
                input,
                &package.id,
                &package.name,
                "package",
                &measures,
                Some(&current_use),
            )
        })
        .collect();

    // Fit check against measured use (current use, chapter 3).
    let fit_check = input.measured.as_ref().map(|measured| {
        let wrap =
            |values: &[f64]| -> Vec<Option<f64>> { values.iter().map(|v| Some(*v)).collect() };
        let calculated_gas = wrap(&current_use.monthly_gas_m3);
        let calculated_heat = wrap(&current_use.monthly_heat_kwh);
        let calculated_el = wrap(&current_use.monthly_electricity_import_kwh);
        let local = &measured.monthly_outdoor_temperature_c;
        // C.3: electricity is compared as the monthly net delivery.
        let calculated_net: Vec<Option<f64>> = (0..12)
            .map(|index| {
                Some(
                    current_use.monthly_electricity_import_kwh[index]
                        - current_use.monthly_electricity_export_kwh[index],
                )
            })
            .collect();
        let measured_gas_base = base_load(&measured.monthly_gas_m3, local);
        let calculated_gas_base = base_load(&calculated_gas, &[]);
        let measured_heat_base = base_load(&measured.monthly_heat_kwh, local);
        let has_heat = current_use.district_heat_kwh > 0.0;
        let calculated_heat_base = has_heat.then(|| base_load(&calculated_heat, &[])).flatten();
        let calculated_heat_line = has_heat
            .then(|| regression(&calculated_heat, &[]))
            .flatten();
        let measured_el_base = base_load(&measured.monthly_electricity_kwh, local);
        let calculated_el_base = base_load(&calculated_net, &[]);
        // Annual values from complete monthly readings when not given.
        let annual = |given: Option<f64>, monthly: &[Option<f64>]| {
            given.or_else(|| {
                (monthly.len() == 12 && monthly.iter().all(Option::is_some))
                    .then(|| monthly.iter().map(|value| value.unwrap_or(0.0)).sum())
            })
        };
        FitCheck {
            calculated: current_use.clone(),
            gas_deviation_percent: deviation(
                current_use.gas_m3,
                annual(measured.annual_gas_m3, &measured.monthly_gas_m3),
            ),
            electricity_deviation_percent: deviation(
                current_use.electricity_import_kwh,
                annual(
                    measured.annual_electricity_kwh,
                    &measured.monthly_electricity_kwh,
                ),
            ),
            heat_deviation_percent: deviation(
                current_use.district_heat_kwh,
                annual(measured.annual_heat_kwh, &measured.monthly_heat_kwh),
            ),
            // The measured line uses the local temperatures of the metered
            // period, the calculated line the NTA 8800 climate year.
            measured_gas_line: with_base(
                regression(&measured.monthly_gas_m3, local),
                measured_gas_base,
            ),
            calculated_gas_line: with_base(regression(&calculated_gas, &[]), calculated_gas_base),
            measured_heat_line: with_base(
                regression(&measured.monthly_heat_kwh, local),
                measured_heat_base,
            ),
            calculated_heat_line: with_base(calculated_heat_line, calculated_heat_base),
            measured_gas_base_load: measured_gas_base,
            calculated_gas_base_load: calculated_gas_base,
            measured_heat_base_load: measured_heat_base,
            calculated_heat_base_load: calculated_heat_base,
            measured_electricity_line: with_base(
                regression(&measured.monthly_electricity_kwh, local),
                measured_el_base,
            ),
            calculated_electricity_line: with_base(
                regression(&calculated_net, &[]),
                calculated_el_base,
            ),
            measured_electricity_base_load: measured_el_base,
            calculated_electricity_base_load: calculated_el_base,
            criteria: FitCriteria::default(),
            monthly_electricity_deviation_percent: (0..12)
                .map(|index| {
                    deviation(
                        current_use.monthly_electricity_import_kwh[index],
                        measured
                            .monthly_electricity_kwh
                            .get(index)
                            .copied()
                            .flatten(),
                    )
                })
                .collect(),
            measured_electricity_monthly_mean_kwh: mean_of(&measured.monthly_electricity_kwh),
            calculated_electricity_monthly_mean_kwh: mean_of(&calculated_el),
        }
    });

    // ISSO 82.2 table 2.7: practice factors need the chapter 11 route; zones
    // with declared ventilation flows cannot be scaled per flow part.
    let practice_requested = [input.current_use.as_ref(), input.future_use.as_ref()]
        .into_iter()
        .flatten()
        .any(|profile| {
            profile.ventilation_practice.is_some() || profile.profile != UserProfile::Nta
        });
    let practice_gap = practice_requested && {
        let mut scratch = Vec::new();
        variant_input(&input.base, &[], &mut scratch).is_some_and(|building| {
            std::iter::once(&building.space_heating.demand)
                .chain(
                    building
                        .space_heating
                        .additional_zones
                        .iter()
                        .map(|zone| &zone.demand),
                )
                .any(|demand| demand.ventilation.is_none())
        })
    };
    let renovation_passport = input
        .renovation_passport
        .as_ref()
        .map(|passport| renovation_passport(input, passport, &current_use));

    let fit_check = fit_check.map(|mut check| {
        check.criteria = fit_criteria(&check, input.measured.as_ref());
        check
    });

    // Advice.
    let mut warnings = Vec::new();
    let mut specialist = Vec::new();
    if practice_gap {
        warnings.push(
            "ventilation_practice_not_applied: de praktijkfactoren voor ventilatie (ISSO 82.2 tabel 2.7) vragen de route van hoofdstuk 11; zones met opgegeven ventilatiestromen worden niet gecorrigeerd".into(),
        );
    }
    if input.packages.len() < 2 {
        warnings.push(
            "ISSO 82.2/75.2 §4.2.2: combineer de maatregelen in minimaal twee pakketten".into(),
        );
    }
    for (package, result) in input.packages.iter().zip(&package_results) {
        if let Some(text) = &package.partial_execution_warning {
            warnings.push(format!("{}: {text}", package.name));
        }
        if !result.valid {
            warnings.push(format!("{}: berekening ongeldig", package.name));
        }
        if let (Some(after), Some(before)) = (result.label.tojuli_max_k, current.label.tojuli_max_k)
        {
            if after > before + 1e-9 {
                warnings.push(format!(
                    "{}: TOjuli stijgt van {before:.2} naar {after:.2}; het risico op oververhitting neemt toe (ISSO 82.2 §4.2.3)",
                    package.name
                ));
            }
        }
        for check in result
            .system_checks
            .iter()
            .filter(|c| c.meets == Some(false))
        {
            warnings.push(format!(
                "{}: systeemeis Bbl art. 4.248 voor {} niet gehaald ({:.2} > {:.2} {}); geldt bij vervanging of verbetering van het systeem (ISSO 82.2 §5.2)",
                package.name,
                check.system,
                check.value.unwrap_or(0.0),
                check.limit.unwrap_or(0.0),
                check.unit
            ));
        }
        if result
            .savings
            .as_ref()
            .is_some_and(|s| s.energy_cost_eur < 0.0)
        {
            warnings.push(format!("{}: de energiekosten nemen toe", package.name));
        }
    }
    for measure in &input.measures {
        if let Some(note) = &measure.specialist_note {
            specialist.push(format!("{}: {note}", measure.name));
        }
        match measure.category {
            MeasureCategory::HeatPump | MeasureCategory::Cooling => specialist.push(format!(
                "{}: vermogensbepaling door een specialist (ISSO 82.2 §6.2.3)",
                measure.name
            )),
            MeasureCategory::Insulation => specialist.push(format!(
                "{}: samenstelling van het isolatiepakket bij condensatierisico laten uitwerken door een specialist (ISSO 82.2 §6.2.3)",
                measure.name
            )),
            _ => {}
        }
    }
    let (package_id, chosen_by) = match &input.advised_package_id {
        Some(id) => (Some(id.clone()), "adviser"),
        None => (
            package_results
                .iter()
                .filter(|result| result.valid)
                .filter_map(|result| result.net_present_value_eur.map(|npv| (npv, &result.id)))
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, id)| id.clone()),
            "automatic_highest_npv",
        ),
    };
    let advice = Advice {
        package_id,
        chosen_by,
        motivation: input.advice_motivation.clone(),
        warnings,
        specialist_notes: specialist,
        notes: input
            .notes
            .iter()
            .map(|note| match &note.package_id {
                Some(id) => format!("[{id}] {}", note.text),
                None => note.text.clone(),
            })
            .collect(),
    };
    let all_valid =
        measure_results.iter().all(|r| r.valid) && package_results.iter().all(|r| r.valid);
    MaatwerkadviesAssessment {
        status: if all_valid {
            "calculated_unverified"
        } else {
            "partially_calculated"
        },
        scope: SCOPE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        attest_status: "unattested",
        current: Some(current),
        measures: measure_results,
        packages: package_results,
        fit_check,
        advice: Some(advice),
        renovation_passport,
        interpretations: INTERPRETATIONS.to_vec(),
        issues,
    }
}

pub fn assess_maatwerkadvies_json(value: Value) -> Result<MaatwerkadviesAssessment, String> {
    let input: MaatwerkadviesInput =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    Ok(assess_maatwerkadvies(&input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn building() -> Value {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-building-performance-synthetic.json"
        ))
        .unwrap()
    }

    fn tariffs() -> Tariffs {
        Tariffs {
            gas_eur_per_m3: 1.40,
            electricity_eur_per_kwh: 0.30,
            electricity_export_eur_per_kwh: 0.05,
            district_heat_eur_per_kwh: 0.0,
            district_cold_eur_per_kwh: 0.0,
            oil_eur_per_kwh: 0.0,
            biomass_eur_per_kwh: 0.0,
            gas_fixed_eur_per_year: 0.0,
            heat_fixed_eur_per_year: 0.0,
            source_reference: "test tariffs".into(),
        }
    }

    #[test]
    fn json_patch_replace_add_remove() {
        let mut value = json!({"a": {"b": [1, 2]}, "c": 1});
        apply_patch(
            &mut value,
            &PatchOperation::Replace {
                path: "/c".into(),
                value: json!(5),
            },
        )
        .unwrap();
        apply_patch(
            &mut value,
            &PatchOperation::Add {
                path: "/a/b/-".into(),
                value: json!(3),
            },
        )
        .unwrap();
        apply_patch(
            &mut value,
            &PatchOperation::Add {
                path: "/a/d".into(),
                value: json!("x"),
            },
        )
        .unwrap();
        apply_patch(
            &mut value,
            &PatchOperation::Remove {
                path: "/a/b/0".into(),
            },
        )
        .unwrap();
        assert_eq!(value, json!({"a": {"b": [2, 3], "d": "x"}, "c": 5}));
        assert!(apply_patch(
            &mut value,
            &PatchOperation::Replace {
                path: "/missing".into(),
                value: json!(1)
            }
        )
        .is_err());
    }

    #[test]
    fn net_present_value_with_reinvestment_and_residual() {
        let measure = Measure {
            id: "m".into(),
            name: "m".into(),
            category: MeasureCategory::Other,
            target: PatchTarget::Building,
            patch: vec![],
            investment_eur: 1000.0,
            cost_source: "x".into(),
            lifetime_years: 10.0,
            maintenance_eur_per_year: 0.0,
            phase_year: None,
            specialist_note: None,
        };
        let economics = Economics {
            discount_rate: 0.0,
            energy_price_change: 0.0,
            horizon_years: None,
            base_year: None,
            maintenance_price_change: 0.0,
            source_reference: String::new(),
        };
        // r = 0: 15 × 100 − 1000 − 1000 (year 10) + 500 residual.
        let npv = net_present_value(&[&measure], 100.0, &economics, 15.0);
        assert!((npv - (1500.0 - 2000.0 + 500.0)).abs() < 1e-9);
        // r = 5 %, horizon = lifetime: annuity factor.
        let economics = Economics {
            discount_rate: 0.05,
            ..economics
        };
        let npv = net_present_value(&[&measure], 200.0, &economics, 10.0);
        let annuity = (1.0 - 1.05_f64.powi(-10)) / 0.05;
        assert!((npv - (200.0 * annuity - 1000.0)).abs() < 1e-6);
    }

    #[test]
    fn gas_regression_finds_slope_and_heating_limit() {
        // Synthetic 100 − 6·θ m³ in the heating months.
        let monthly: Vec<Option<f64>> = (0..12)
            .map(|index| Some((100.0 - 6.0 * OUTDOOR_TEMPERATURE_C[index]).max(0.0)))
            .collect();
        let line = gas_regression(&monthly).unwrap();
        assert!((line.slope_m3_per_k + 6.0).abs() < 1e-9);
        assert!((line.intercept_m3 - 100.0).abs() < 1e-9);
        assert!((line.heating_limit_c.unwrap() - 100.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn residential_profiles_follow_isso_82_2() {
        let energy = residential_profile(UserProfile::EnergyConscious, false);
        assert_eq!(energy.heating_setpoint_c, Some(18.0));
        assert_eq!(energy.hot_water_per_person_kwh, Some(409.0));
        let average = residential_profile(UserProfile::Average, true);
        assert_eq!(average.spatial_fraction, Some(0.5));
        assert_eq!(average.internal_gain_per_person_w, Some(180.0));
        let wasteful = residential_profile(UserProfile::NotEnergyConscious, false);
        assert_eq!(wasteful.spatial_fraction, Some(0.0));
        assert_eq!(wasteful.hot_water_per_person_kwh, Some(681.0));
    }

    fn mwa(base: Value) -> MaatwerkadviesInput {
        serde_json::from_value(json!({
            "base": {"kind": "building", "input": base},
            "measures": [],
            "packages": [],
            "tariffs": serde_json::to_value(tariffs()).unwrap()
        }))
        .unwrap()
    }

    #[test]
    fn variants_report_savings_and_payback() {
        let base = building();
        let heating = base["spaceHeating"]["demand"]["setpoints"]["heatingC"].clone();
        assert!(heating.is_number());
        let mut input = mwa(base.clone());
        // A measure that lowers the transmission: halve every window area
        // is too input-specific; replace the setpoint through the use
        // instead and add a PV-free no-op measure for the plumbing.
        input.measures = vec![Measure {
            id: "noop".into(),
            name: "Geen wijziging".into(),
            category: MeasureCategory::Other,
            target: PatchTarget::Building,
            patch: vec![PatchOperation::Replace {
                path: "/areaSourceReference".into(),
                value: json!("patched"),
            }],
            investment_eur: 1000.0,
            cost_source: "offerte".into(),
            lifetime_years: 20.0,
            maintenance_eur_per_year: 0.0,
            phase_year: Some(2027),
            specialist_note: None,
        }];
        input.packages = vec![Package {
            id: "p1".into(),
            name: "Pakket 1".into(),
            measure_ids: vec!["noop".into()],
            partial_execution_warning: Some("let op".into()),
        }];
        let result = assess_maatwerkadvies(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let current = result.current.as_ref().unwrap();
        let use_ = current.actual_use.as_ref().unwrap();
        assert!(use_.energy_cost_eur > 0.0);
        let package = &result.packages[0];
        let savings = package.savings.as_ref().unwrap();
        assert!(savings.energy_cost_eur.abs() < 1e-6);
        assert!(package.simple_payback_years.is_none());
        let npv = package.net_present_value_eur.unwrap();
        assert!((npv + 1000.0).abs() < 1e-6);
        let advice = result.advice.as_ref().unwrap();
        assert_eq!(advice.package_id.as_deref(), Some("p1"));
        assert!(advice.warnings.iter().any(|w| w.contains("minimaal twee")));
        assert!(advice.warnings.iter().any(|w| w.contains("let op")));
        assert_eq!(package.phasing[0].year, Some(2027));
    }

    #[test]
    fn current_use_lowers_gas_and_keeps_the_label() {
        let base = building();
        let plain = assess_maatwerkadvies(&mwa(base.clone()));
        assert_eq!(plain.status, "calculated_unverified", "{:?}", plain.issues);
        let mut input = mwa(base);
        let residential = matches!(
            serde_json::from_value::<BuildingPerformanceInput>(match &input.base {
                MwaBase::Building { input } => input.clone(),
                MwaBase::Project { .. } => unreachable!(),
            })
            .unwrap()
            .calculation_scope,
            CalculationScope::Residential
        );
        input.current_use = Some(UsageProfile {
            ventilation_practice: None,
            persons: None,
            heat_per_person_w: None,
            occupancy_time_fraction: None,
            appliance_w_per_m2: None,
            lighting_hours_factor: None,
            profile: UserProfile::EnergyConscious,
            heating_setpoint_c: None,
            cooling_setpoint_c: None,
            reduced_setpoint_c: None,
            day_reduction_h: None,
            weekend_reduction_h: None,
            spatial_fraction: None,
            occupants: None,
            internal_gain_per_person_w: None,
            occupancy_appliance_w_per_m2: None,
            hot_water_need_per_person_kwh: None,
            annual_hot_water_need_kwh: None,
            source_reference: "bewonersgesprek".into(),
        });
        let fitted = assess_maatwerkadvies(&input);
        assert_eq!(
            fitted.status, "calculated_unverified",
            "{:?}",
            fitted.issues
        );
        let before = plain.current.as_ref().unwrap();
        let after = fitted.current.as_ref().unwrap();
        // The label run is the same NTA 8800 run.
        assert_eq!(before.label.label_class, after.label.label_class);
        assert_eq!(
            before.label.primary_fossil_indicator_kwh_per_m2,
            after.label.primary_fossil_indicator_kwh_per_m2
        );
        let gas_before = before.actual_use.as_ref().unwrap();
        let gas_after = after.actual_use.as_ref().unwrap();
        let total = |u: &EnergyUse| u.gas_kwh + u.electricity_import_kwh + u.district_heat_kwh;
        if residential {
            assert!(total(gas_after) < total(gas_before));
        } else {
            assert!(total(gas_after) <= total(gas_before) + 1e-6);
        }
    }

    fn profile(kind: UserProfile) -> UsageProfile {
        UsageProfile {
            profile: kind,
            heating_setpoint_c: None,
            cooling_setpoint_c: None,
            reduced_setpoint_c: None,
            day_reduction_h: None,
            weekend_reduction_h: None,
            spatial_fraction: None,
            occupants: None,
            internal_gain_per_person_w: None,
            occupancy_appliance_w_per_m2: None,
            hot_water_need_per_person_kwh: None,
            annual_hot_water_need_kwh: None,
            persons: None,
            heat_per_person_w: None,
            occupancy_time_fraction: None,
            appliance_w_per_m2: None,
            lighting_hours_factor: None,
            ventilation_practice: None,
            source_reference: "gesprek".into(),
        }
    }

    #[test]
    fn utility_persons_and_lighting_hours_follow_isso_75_2() {
        use crate::lighting::{
            Daylight, InstalledPower, LightingZone, Occupancy, ParasiticPower, SwitchControl,
            UseArea, ZoneLighting,
        };
        let mut building: BuildingPerformanceInput = serde_json::from_value(building()).unwrap();
        building.calculation_scope = CalculationScope::Utility;
        building.space_heating.demand.usage_function = crate::monthly_demand::UsageFunction::Office;
        building.space_heating.demand.function_areas.clear();
        let area = building.space_heating.demand.usable_floor_area_m2;
        building.lighting = vec![ZoneLighting {
            zone_id: building.space_heating.demand.zone_id.clone(),
            functions: vec![UseArea {
                function: crate::label_class::LabelFunction::Office,
                area_m2: area,
            }],
            lighting_zones: vec![LightingZone {
                id: "lz".into(),
                area_m2: area,
                power: InstalledPower::Forfait {
                    led_from_2017: false,
                },
                parasitic: ParasiticPower::Forfait,
                occupancy: Occupancy {
                    control: SwitchControl::ManualOrUnknown,
                    central_on_control: true,
                    large_office_group: false,
                },
                daylight: Daylight::None,
                extracted_luminaires: false,
            }],
            source_reference: "plan".into(),
            burning_hours_factor: None,
        }];
        // 75.2 table 2.6: N_p·80 W·f_t/A_g + q_A (office f_τ 0,30, q_A 4).
        let mut use_ = profile(UserProfile::EnergyConscious);
        use_.persons = Some(12.0);
        let mut fitted = building.clone();
        apply_use(&mut fitted, &use_);
        let fit = fitted.space_heating.demand.usage_fit.as_ref().unwrap();
        let expected = 12.0 * 80.0 * 0.30 / area + 4.0;
        assert!((fit.occupancy_appliance_w_per_m2.unwrap() - expected).abs() < 1e-9);
        // 75.2 table 2.7: energy-conscious burning hours −20 %.
        assert_eq!(fitted.lighting[0].burning_hours_factor, Some(0.8));
        // ISSO 82.2 table 2.2: standard profiles take the practice factors.
        assert!(fit.ventilation_practice.is_some());
        // The NTA option keeps the NTA burning hours and no practice factors.
        let mut nta = building.clone();
        apply_use(&mut nta, &profile(UserProfile::Nta));
        assert_eq!(nta.lighting[0].burning_hours_factor, None);
        assert!(nta
            .space_heating
            .demand
            .usage_fit
            .as_ref()
            .unwrap()
            .ventilation_practice
            .is_none());
    }

    #[test]
    fn system_checks_follow_omgevingsregeling_bijlage_viii() {
        use crate::building_performance::{F_P_DISTRICT_HEAT_FORFAIT, F_P_ELECTRICITY};
        let input: BuildingPerformanceInput = serde_json::from_value(building()).unwrap();
        let result = crate::building_performance::assess_building_performance(&input);
        let checks = system_performance_checks(&input, &result);
        let heating = checks.iter().find(|c| c.system == "space_heating").unwrap();
        let energy: f64 = result
            .space_heating
            .monthly
            .iter()
            .map(|row| {
                row.natural_gas_kwh
                    + row.oil_kwh
                    + 0.5 * row.biomass_kwh
                    + F_P_DISTRICT_HEAT_FORFAIT * row.district_heat_kwh
                    + F_P_ELECTRICITY
                        * (row.generator_electricity_kwh
                            + row.auxiliary_electricity_kwh.unwrap_or(0.0))
            })
            .sum();
        let need = result
            .space_heating
            .demand
            .annual_heating_need_without_recoverable_kwh
            .unwrap();
        assert!((heating.value.unwrap() - energy / need).abs() < 1e-9);
        assert_eq!(heating.limit, Some(1.31));
        assert_eq!(heating.meets, Some(energy / need <= 1.31));
        // Dwellings: no ventilation or lighting limit.
        let ventilation = checks.iter().find(|c| c.system == "ventilation").unwrap();
        assert_eq!(ventilation.limit, None);
        let assessed = assess_maatwerkadvies(&mwa(building()));
        assert_eq!(assessed.current.unwrap().system_checks.len(), checks.len());
    }

    #[test]
    fn fit_check_uses_monthly_series_and_isso_criteria() {
        // §3.2.7: heating limit at the knee above the base load.
        let line = RegressionLine {
            slope_m3_per_k: -10.0,
            intercept_m3: 200.0,
            heating_limit_c: None,
            points: 6,
        };
        let knee = with_base(Some(line), Some(30.0)).unwrap();
        assert!((knee.heating_limit_c.unwrap() - 17.0).abs() < 1e-12);
        assert!(
            (with_base(Some(line), None)
                .unwrap()
                .heating_limit_c
                .unwrap()
                - 20.0)
                .abs()
                < 1e-12
        );
        // Measured series equal to the calculated ones meet every criterion.
        let plain = assess_maatwerkadvies(&mwa(building()));
        let current = plain.current.unwrap().actual_use.unwrap();
        let mut input = mwa(building());
        input.measured = Some(MeasuredUse {
            monthly_gas_m3: current.monthly_gas_m3.iter().map(|v| Some(*v)).collect(),
            monthly_electricity_kwh: (0..12)
                .map(|index| {
                    Some(
                        current.monthly_electricity_import_kwh[index]
                            - current.monthly_electricity_export_kwh[index],
                    )
                })
                .collect(),
            source_reference: "slimme meter".into(),
            ..MeasuredUse::default()
        });
        let fitted = assess_maatwerkadvies(&input);
        let check = fitted.fit_check.unwrap();
        assert_eq!(check.criteria.annual_gas, Some(true));
        assert_eq!(check.criteria.gas_slope, Some(true));
        assert_eq!(check.criteria.gas_heating_limit, Some(true));
        assert_eq!(check.criteria.within_criteria, Some(true));
        assert!(check.monthly_electricity_deviation_percent.len() == 12);
        // 20 % more gas fails the annual criterion; a warmer local climate
        // moves the measured heating limit.
        let mut more = input.clone();
        let measured = more.measured.as_mut().unwrap();
        for value in measured.monthly_gas_m3.iter_mut().flatten() {
            *value *= 1.2;
        }
        measured.monthly_outdoor_temperature_c = (0..12)
            .map(|index| Some(crate::climate::OUTDOOR_TEMPERATURE_C[index] + 2.0))
            .collect();
        let check = assess_maatwerkadvies(&more).fit_check.unwrap();
        assert_eq!(check.criteria.annual_gas, Some(false));
        assert_eq!(check.criteria.within_criteria, Some(false));
    }

    #[test]
    fn renovation_passport_stacks_the_three_steps() {
        let mut input = mwa(building());
        let noop = |id: &str, category: MeasureCategory| Measure {
            id: id.into(),
            name: id.into(),
            category,
            target: PatchTarget::Building,
            patch: vec![PatchOperation::Replace {
                path: "/areaSourceReference".into(),
                value: json!(id),
            }],
            investment_eur: 100.0,
            cost_source: "offerte".into(),
            lifetime_years: 20.0,
            maintenance_eur_per_year: 0.0,
            phase_year: None,
            specialist_note: None,
        };
        input.measures = vec![
            noop("isolatie", MeasureCategory::Insulation),
            noop("zonwering", MeasureCategory::Other),
            noop("wp", MeasureCategory::HeatPump),
            noop("pv", MeasureCategory::Pv),
        ];
        let package = |id: &str, measures: &[&str]| Package {
            id: id.into(),
            name: id.into(),
            measure_ids: measures.iter().map(|m| m.to_string()).collect(),
            partial_execution_warning: None,
        };
        input.packages = vec![
            package("stap1", &["isolatie", "zonwering"]),
            package("stap2", &["wp"]),
            package("stap3", &["pv"]),
        ];
        input.renovation_passport = Some(RenovationPassportInput {
            demand_package_id: "stap1".into(),
            systems_package_id: "stap2".into(),
            production_package_id: "stap3".into(),
            prewar_standard: false,
            prewar_motivation: None,
            insulation_standard_met: Some(true),
            insulation_standard_max_need_kwh_per_m2: None,
            overheating_measure_ids: vec!["zonwering".into()],
            storage_considered: true,
        });
        let result = assess_maatwerkadvies(&input);
        let passport = result.renovation_passport.unwrap();
        assert_eq!(passport.steps.len(), 3);
        // Stacked: step 3 contains the measures of steps 1 and 2.
        assert_eq!(passport.steps[2].measure_ids.len(), 4);
        let met = |code: &str| {
            passport
                .requirements
                .iter()
                .find(|item| item.code == code)
                .unwrap()
                .met
        };
        assert_eq!(met("three_steps_present"), Some(true));
        assert_eq!(met("overheating_measures"), Some(true));
        assert_eq!(met("renewable_production"), Some(true));
        // The no-op "heat pump" leaves the gas boiler in place.
        assert_eq!(met("natural_gas_free_main_heating"), Some(false));
        assert_eq!(passport.eligible, Some(false));
    }

    #[test]
    fn npv_discounts_phased_investments_and_escalates_maintenance() {
        let measure = Measure {
            id: "m".into(),
            name: "m".into(),
            category: MeasureCategory::Other,
            target: PatchTarget::Building,
            patch: Vec::new(),
            investment_eur: 1000.0,
            cost_source: "offerte".into(),
            lifetime_years: 30.0,
            maintenance_eur_per_year: 10.0,
            phase_year: Some(2030),
            specialist_note: None,
        };
        let economics = Economics {
            discount_rate: 0.05,
            energy_price_change: 0.0,
            horizon_years: None,
            base_year: Some(2026),
            maintenance_price_change: 0.02,
            source_reference: String::new(),
        };
        let horizon = 10.0;
        let npv = net_present_value(&[&measure], 0.0, &economics, horizon);
        let upkeep: f64 = (1..=10)
            .map(|t| 10.0 * 1.02_f64.powi(t - 1) / 1.05_f64.powi(t))
            .sum();
        // Invested in year 4; residual (30 − 6)/30 at the horizon.
        let expected =
            -upkeep - 1000.0 / 1.05_f64.powi(4) + 1000.0 * (1.0 - 6.0 / 30.0) / 1.05_f64.powi(10);
        assert!((npv - expected).abs() < 1e-9, "{npv} vs {expected}");
    }

    fn project() -> Value {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap()
    }

    #[test]
    fn project_measure_saves_energy_and_costs() {
        let mut input = mwa(json!(null));
        input.base = MwaBase::Project { project: project() };
        let insulation = |id: &str, u: f64, rc: f64| Measure {
            id: id.into(),
            name: format!("Gevelisolatie {id}"),
            category: MeasureCategory::Insulation,
            target: PatchTarget::Project,
            patch: vec![
                PatchOperation::Replace {
                    path: "/constructions/0/uValue".into(),
                    value: json!(u),
                },
                PatchOperation::Replace {
                    path: "/constructions/0/rcValue".into(),
                    value: json!(rc),
                },
            ],
            investment_eur: 8000.0,
            cost_source: "kostenkentallen RVO".into(),
            lifetime_years: 40.0,
            maintenance_eur_per_year: 0.0,
            phase_year: None,
            specialist_note: None,
        };
        input.measures = vec![insulation("a", 0.15, 6.5), insulation("b", 0.12, 8.0)];
        input.packages = vec![
            Package {
                id: "p1".into(),
                name: "Basis".into(),
                measure_ids: vec!["a".into()],
                partial_execution_warning: None,
            },
            Package {
                id: "p2".into(),
                name: "Ambitieus".into(),
                measure_ids: vec!["b".into()],
                partial_execution_warning: None,
            },
        ];
        let result = assess_maatwerkadvies(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let current = result.current.as_ref().unwrap().actual_use.clone().unwrap();
        let p1 = &result.packages[0];
        let p2 = &result.packages[1];
        let s1 = p1.savings.as_ref().unwrap();
        let s2 = p2.savings.as_ref().unwrap();
        assert!(s1.primary_fossil_kwh > 0.0, "{s1:?}");
        assert!(s2.primary_fossil_kwh > s1.primary_fossil_kwh);
        assert!(s1.energy_cost_eur > 0.0);
        assert!(
            (s1.energy_cost_eur
                - (current.energy_cost_eur - p1.actual_use.as_ref().unwrap().energy_cost_eur))
                .abs()
                < 1e-9
        );
        let payback = p1.simple_payback_years.unwrap();
        assert!((payback - 8000.0 / s1.energy_cost_eur).abs() < 1e-9);
        // The label indicators improve as well.
        let before = result
            .current
            .as_ref()
            .unwrap()
            .label
            .primary_fossil_indicator_kwh_per_m2;
        assert!(p2.label.primary_fossil_indicator_kwh_per_m2 < before);
        let advice = result.advice.as_ref().unwrap();
        assert_eq!(advice.chosen_by, "automatic_highest_npv");
        assert!(advice
            .specialist_notes
            .iter()
            .any(|n| n.contains("isolatiepakket")));
    }

    #[test]
    fn fitted_run_has_no_label_class() {
        let mut building: BuildingPerformanceInput = serde_json::from_value(building()).unwrap();
        let plain = assess_building_performance(&building);
        assert!(plain.indicative_label_class.is_some());
        apply_use(
            &mut building,
            &UsageProfile {
                ventilation_practice: None,
                persons: None,
                heat_per_person_w: None,
                occupancy_time_fraction: None,
                appliance_w_per_m2: None,
                lighting_hours_factor: None,
                profile: UserProfile::NotEnergyConscious,
                heating_setpoint_c: None,
                cooling_setpoint_c: None,
                reduced_setpoint_c: None,
                day_reduction_h: None,
                weekend_reduction_h: None,
                spatial_fraction: None,
                occupants: Some(4.0),
                internal_gain_per_person_w: None,
                occupancy_appliance_w_per_m2: None,
                hot_water_need_per_person_kwh: None,
                annual_hot_water_need_kwh: None,
                source_reference: "test".into(),
            },
        );
        assert_eq!(building.space_heating.demand.setpoints.heating_c, 22.0);
        let fitted = assess_building_performance(&building);
        assert_eq!(
            fitted.status, "calculated_unverified",
            "{:?}",
            fitted.issues
        );
        assert!(fitted.indicative_label_class.is_none());
        assert!(
            fitted.annual_primary_fossil_kwh.unwrap() > plain.annual_primary_fossil_kwh.unwrap()
        );
    }

    #[test]
    fn hot_water_fit_scales_need_dependent_terms() {
        use crate::domestic_hot_water::{HotWaterAssessment, HotWaterMonth};
        let month = HotWaterMonth {
            month: 1,
            net_need_kwh: 100.0,
            emission_input_kwh: 120.0,
            circulation_loss_kwh: 30.0,
            distribution_efficiency: 0.8,
            generator_output_kwh: 150.0,
            carrier_input_kwh: 200.0,
            auxiliary_electricity_kwh: 10.0,
            ..HotWaterMonth::default()
        };
        let mut result = HotWaterAssessment {
            exhaust_air: None,
            combi_chp_heating: None,
            annual_net_need_kwh: 100.0,
            emission_efficiency: 100.0 / 120.0,
            annual_generator_output_kwh: 150.0,
            months: vec![month],
            generators: Vec::new(),
            annual_solar_renewable_kwh: 0.0,
            annual_solar_space_heating_kwh: 0.0,
        };
        crate::building_performance::fit_hot_water_need(&mut result, 50.0);
        let fitted = &result.months[0];
        assert!((fitted.net_need_kwh - 50.0).abs() < 1e-9);
        assert!((fitted.emission_input_kwh - 60.0).abs() < 1e-9);
        // Output loses 60 / 0,8 = 75; carrier input follows the output.
        assert!((fitted.generator_output_kwh - 75.0).abs() < 1e-9);
        assert!((fitted.carrier_input_kwh - 100.0).abs() < 1e-9);
        assert!((result.annual_net_need_kwh - 50.0).abs() < 1e-9);
    }

    #[test]
    fn validation_requires_cost_source_and_known_measures() {
        let mut input = mwa(building());
        input.measures = vec![Measure {
            id: "m".into(),
            name: "m".into(),
            category: MeasureCategory::Other,
            target: PatchTarget::Building,
            patch: vec![],
            investment_eur: -1.0,
            cost_source: String::new(),
            lifetime_years: 0.0,
            maintenance_eur_per_year: 0.0,
            phase_year: None,
            specialist_note: None,
        }];
        input.packages = vec![Package {
            id: "p".into(),
            name: "p".into(),
            measure_ids: vec!["x".into()],
            partial_execution_warning: None,
        }];
        let result = assess_maatwerkadvies(&input);
        let codes: Vec<_> = result.issues.iter().map(|i| i.code).collect();
        for code in [
            "measure_cost_source_required",
            "measure_investment_invalid",
            "measure_lifetime_invalid",
            "measure_patch_required",
            "package_measure_unknown",
        ] {
            assert!(codes.contains(&code), "{code}");
        }
    }
}
