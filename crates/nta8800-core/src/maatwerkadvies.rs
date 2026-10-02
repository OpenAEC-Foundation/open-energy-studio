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
    "heat from a collective heat-pump source (dh_hp_source) is priced at the district-heat tariff",
    "the best-fit package is the adviser's choice; without one, the package with the highest net present value is proposed and marked as automatic",
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
    /// ISSO 82.2 table 2.7 / 75.2 table 2.8 practice factors; omitted
    /// fields take the standard values.
    #[serde(default)]
    pub ventilation_practice: crate::ventilation::VentilationPractice,
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
    #[serde(default)]
    pub source_reference: String,
}

fn default_discount() -> f64 {
    0.03
}

impl Default for Economics {
    fn default() -> Self {
        Self {
            discount_rate: default_discount(),
            energy_price_change: 0.0,
            horizon_years: None,
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
    pub issues: Vec<MwaIssue>,
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
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionLine {
    pub slope_m3_per_k: f64,
    pub intercept_m3: f64,
    /// Outdoor temperature where the line reaches zero (heating limit).
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
}

/// Standard profile values for dwellings: ISSO 82.2 tables 2.3–2.6
/// (p. 36–38).
fn residential_profile(profile: UserProfile, apartment_building: bool) -> ResolvedUse {
    match profile {
        UserProfile::Nta => ResolvedUse::default(),
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
    let total_area: f64 = std::iter::once(&input.space_heating.demand)
        .chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
        .map(|demand| demand.usable_floor_area_m2)
        .sum();
    let nta_occupants: f64 = std::iter::once(&input.space_heating.demand)
        .chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
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
                .filter(|_| !residential_zone),
            ventilation_practice: Some(profile.ventilation_practice.clone()),
            source_reference: format!("maatwerkadvies: {}", profile.source_reference),
        });
    };
    fit_zone(&mut input.space_heating.demand);
    for zone in input.space_heating.additional_zones.iter_mut() {
        fit_zone(&mut zone.demand);
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
            "dh" | "dw" | "dh_hp_source" => usage.district_heat_kwh += row.used_kwh,
            "dc" => usage.district_cold_kwh += row.used_kwh,
            _ => {}
        }
    }
    for row in &result.electricity_balance {
        usage.electricity_import_kwh += (row.used_kwh - row.self_used_kwh).max(0.0);
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
    issues: Vec<MwaIssue>,
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
            issues,
        };
    };
    if !issues.is_empty() {
        return RunOutcome {
            label: LabelResult::default(),
            actual: None,
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
            issues,
        };
    }
    let label = label_result(&standard);
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
                    issues,
                };
            }
            energy_use(&result, &input.tariffs)
        }
    };
    RunOutcome {
        label,
        actual: Some(actual),
        issues,
    }
}

/// NTA annual net hot-water need of a building input, kWh.
fn hot_water_need_kwh(input: &BuildingPerformanceInput) -> Option<f64> {
    let system = input.hot_water.as_ref()?;
    let context = crate::domestic_hot_water::HotWaterContext {
        residential: matches!(input.calculation_scope, CalculationScope::Residential),
        usable_floor_area_m2: input.total_usable_floor_area_m2,
        heated_ambient_c: input.space_heating.demand.setpoints.heating_c,
        space_heating: None,
        standard_setpoint_c: None,
        levelled_setpoint_c: None,
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
    let mut value = 0.0;
    for t in 1..=years {
        let t_f = f64::from(t);
        let saving = annual_cost_saving_eur * (1.0 + e).powf(t_f - 1.0);
        value += (saving - maintenance) / (1.0 + r).powf(t_f);
    }
    for measure in measures {
        let lifetime = measure.lifetime_years.max(1.0);
        let mut start = 0.0;
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
        issues: outcome.issues,
    }
}

// --------------------------------------------------------------- fit check

/// Least squares of monthly gas against the outdoor temperature for the
/// months below 15 °C (heating season).
pub fn gas_regression(monthly_m3: &[Option<f64>]) -> Option<RegressionLine> {
    let points: Vec<(f64, f64)> = monthly_m3
        .iter()
        .enumerate()
        .take(12)
        .filter_map(|(index, value)| value.map(|v| (OUTDOOR_TEMPERATURE_C[index], v)))
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
        if !measured.monthly_gas_m3.is_empty() && measured.monthly_gas_m3.len() != 12 {
            issues.push(issue("measured_monthly_length", "measured.monthlyGasM3"));
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
        let calculated: Vec<Option<f64>> = current_use
            .monthly_gas_m3
            .iter()
            .map(|v| Some(*v))
            .collect();
        FitCheck {
            calculated: current_use.clone(),
            gas_deviation_percent: deviation(current_use.gas_m3, measured.annual_gas_m3),
            electricity_deviation_percent: deviation(
                current_use.electricity_import_kwh,
                measured.annual_electricity_kwh,
            ),
            heat_deviation_percent: deviation(
                current_use.district_heat_kwh,
                measured.annual_heat_kwh,
            ),
            measured_gas_line: gas_regression(&measured.monthly_gas_m3),
            calculated_gas_line: gas_regression(&calculated),
        }
    });

    // Advice.
    let mut warnings = Vec::new();
    let mut specialist = Vec::new();
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
            ventilation_practice: Default::default(),
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
                ventilation_practice: Default::default(),
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
