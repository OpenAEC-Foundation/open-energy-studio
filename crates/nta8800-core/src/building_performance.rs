//! End-to-end unverified energy performance of a single-zone building:
//! space-heating chain + declared other services → `E_EPus` per carrier
//! (5.20/5.21) → on-site electricity self-use and export (5.22–5.26) →
//! primary fossil energy `EPTot` (5.9–5.14, table 5.2) and renewable energy
//! `EPrenTot` (5.29–5.31, 5.39, table 5.4) → indicators with the rounding of
//! `indicators_draft`.
//!
//! Checked against NTA 8800:2025+C1:2026 chapter 5: storage correction
//! 5.14a/5.14b (page 85), `f_BACS` (§5.5.8, page 100) and the operational
//! CO2 emission of §5.5.6.1 with table 5.3 (page 97). External heat, hot
//! water and cold use the forfait factors of tables 5.2–5.4 or annex P
//! values (§5.8, `annex_p`); with a quality declaration EP_Tot and RER are
//! calculated twice (EMGverklaring and EMGforf, §5.3.1). A collective
//! heat-pump source adds `Q_HD;hp;in;bron` at its own factors (5.20,
//! 9.6.8.1.1.2.3). Exported heat is rejected because it is not wired. Non-EP electricity is fixed at 0 as
//! required for the indicators (5.27).

use crate::annex_p::{
    assess_route, source_factors, AnnexPRoute, CollectiveHeatPumpSource, ScenarioFactors,
    SupplyFactors, SystemFunction, SystemResult, COLD_FORFAIT, HEAT_FORFAIT,
};
use crate::annex_q::AnnexQSource;
use crate::bbl_requirements::{
    a0_check, check_mixed as bbl_check_mixed, A0Check, BblCheck, BblFunction, BblFunctionArea,
};
use crate::domestic_hot_water::{
    assess_hot_water, validate_hot_water, HotWaterAssessment, HotWaterContext, HotWaterSystem,
    SolarSpaceHeating,
};
use crate::final_energy_draft::DRAFT_SOURCE;
use crate::forfait_heat_pump_draft::TableSource;
use crate::forfait_heat_pump_monthly_draft::SourceSystem;
use crate::indicators_draft::{
    assess_indicators_draft, AnnualScenario, CalculationScope, IndicatorsDraftAssessment,
    IndicatorsDraftInput, ScenarioKind,
};
use crate::label_class::{indicative_label_class, LabelFunction, LABEL_SOURCE};
use crate::lighting::{
    assess_zone_lighting, validate_lighting, LightingContext, ZoneLighting, ZoneLightingResult,
};
use crate::monthly_demand::{InternalGains, MonthlyDemandInput, UtilityLighting};
use crate::pv::{monthly_yield_kwh, validate_pv, PvSystem};
use crate::space_cooling::{
    assess_cooling, validate_cooling, CoolingAssessment, CoolingContext, CoolingGeneratorKind,
    CoolingSystem, CoolingZoneNeed, FreeCoolingSource,
};
use crate::space_heating_chain::{
    assess_space_heating_chain, Generator, SpaceHeatingChainAssessment, SpaceHeatingChainInput,
};
use crate::tojuli::{assess_tojuli, ActiveCoolingEvidence, TojuliAssessment, TojuliOptions};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Table 5.2 `f_P;del` / `f_P;pr;us` / `f_P;exp` for electricity.
pub const F_P_ELECTRICITY: f64 = 1.45;
/// Table 5.2 `f_P;del` for natural gas and fuel oil.
pub const F_P_GAS: f64 = 1.0;
pub const F_P_OIL: f64 = 1.0;
/// Table 5.2: external heat without a quality declaration (annex P).
pub const F_P_DISTRICT_HEAT_FORFAIT: f64 = 0.9;
/// Table 5.2: biomass appliances of at most 500 kW meeting annex R (bmB).
pub const F_P_BIOMASS_B: f64 = 0.5;
/// Table 5.4: renewable factor for bmB.
pub const F_PREN_BIOMASS_B: f64 = 0.5;
/// Table 5.4.
pub const F_PREN_RENELECT: f64 = 1.45;
/// 5.14a: policy factor on the stored renewable electricity.
pub const STORAGE_CORRECTION_FACTOR: f64 = 0.05;
/// 5.14a: minimum building-bound storage capacity for `f_BAT;cor = 1`, kWh.
pub const STORAGE_MIN_CAPACITY_KWH: f64 = 5.0;
/// Table 5.3 `K_CO2` in kg CO2eq/kWh (reference date January 2025).
pub const K_CO2_ELECTRICITY: f64 = 0.268;
pub const K_CO2_GAS: f64 = 0.218;
pub const K_CO2_OIL: f64 = 0.326;
/// Table 5.3: bmB = 0,5 × 0,104.
pub const K_CO2_BIOMASS_B: f64 = 0.5 * 0.104;
/// Table 5.3: external heat without an annex P declaration.
pub const K_CO2_DISTRICT_HEAT_FORFAIT: f64 = 0.09;
/// Table 5.4: ambient cold.
pub const F_PREN_RENCOLD: f64 = 1.0;
pub const F_PREN_RENHEAT: f64 = 1.0;
/// Table 5.2: external cold without an annex P declaration, `f_P;del;el / 3`.
pub const F_P_DISTRICT_COLD_FORFAIT: f64 = F_P_ELECTRICITY / 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Carrier {
    El,
    Gas,
    Oil,
}

impl Carrier {
    fn primary_factor(self) -> f64 {
        match self {
            Self::El => F_P_ELECTRICITY,
            Self::Gas => F_P_GAS,
            Self::Oil => F_P_OIL,
        }
    }

    fn co2_factor(self) -> f64 {
        match self {
            Self::El => K_CO2_ELECTRICITY,
            Self::Gas => K_CO2_GAS,
            Self::Oil => K_CO2_OIL,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::El => "el",
            Self::Gas => "gas",
            Self::Oil => "oil",
        }
    }
}

/// Energy functions of 5.20/5.21 other than the modelled space heating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    DomesticHotWater,
    DomesticHotWaterAuxiliary,
    VentilationFans,
    SpaceCooling,
    SpaceCoolingAuxiliary,
    Lighting,
    PvAuxiliary,
    /// `E_hum;ci` of 5.20 (steam humidifiers, 12.3); not BACS-corrected.
    Humidification,
}

impl Service {
    /// `f_BACS` applies to cooling and its auxiliary energy (5.20/5.21).
    fn bacs_weighted(self) -> bool {
        matches!(self, Self::SpaceCooling | Self::SpaceCoolingAuxiliary)
    }

    fn electric_only(self) -> bool {
        !matches!(
            self,
            Self::DomesticHotWater | Self::SpaceCooling | Self::Humidification
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredUse {
    pub id: String,
    pub service: Service,
    pub carrier: Carrier,
    pub monthly_kwh: Vec<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredRenewableHeat {
    pub id: String,
    /// Ambient heat of a hot-water heat pump (5.35/5.36) determined elsewhere.
    pub monthly_kwh: Vec<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OnSiteProduction {
    pub id: String,
    pub kind: ProductionKind,
    /// `E_pr;el;gi` per month from chapter 16, kWh.
    pub monthly_kwh: Vec<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionKind {
    Pv,
    Pvt,
    Wind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatPumpRenewableEvidence {
    /// 5.31 requires a source temperature below 20 °C.
    pub source_below_20_c: bool,
    /// Exhaust-air sources are not renewable without a declaration (5.32).
    pub exhaust_air_source: bool,
    pub source_reference: String,
    /// Source of both outdoor air and exhaust air: table 9.27 footnote c
    /// uses the outdoor-air row; 5.32 counts only the outdoor-air share.
    #[serde(default)]
    pub combined_outdoor_and_exhaust_air: bool,
    /// `f_H;buitenlucht` from a quality declaration; absent means 0 (5.32).
    #[serde(default)]
    pub outdoor_air_heat_fraction: Option<f64>,
    #[serde(default)]
    pub outdoor_air_fraction_reference: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnergyStorage {
    /// Building-bound electrical storage behind the meter (plug-in batteries
    /// do not count), kWh.
    pub building_bound_electrical_kwh: f64,
    /// Building-bound thermal storage, kWh.
    pub building_bound_thermal_kwh: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildingPerformanceInput {
    pub calculation_scope: CalculationScope,
    pub total_usable_floor_area_m2: f64,
    pub area_source_reference: String,
    pub space_heating: SpaceHeatingChainInput,
    /// Required when the generator is a heat pump.
    #[serde(default)]
    pub heat_pump_renewable: Option<HeatPumpRenewableEvidence>,
    /// `f_BACS` (5.5.8): 1,0, or 1,05 for utility buildings, with source.
    pub bacs_factor: f64,
    pub bacs_source_reference: String,
    pub use_inventory_complete: bool,
    pub declared_uses: Vec<DeclaredUse>,
    #[serde(default)]
    pub declared_renewable_heat: Vec<DeclaredRenewableHeat>,
    pub production_inventory_complete: bool,
    pub on_site_production: Vec<OnSiteProduction>,
    /// PV systems calculated here with 16.2/16.3.
    #[serde(default)]
    pub pv_systems: Vec<PvSystem>,
    /// Use function for the indicative label table (annex X); residential
    /// buildings use annex IX and may omit it.
    #[serde(default)]
    pub label_function: Option<LabelFunction>,
    /// §5.7.1: an active cooling system with demonstrated capacity; then
    /// TOjuli = 0.
    #[serde(default)]
    pub active_cooling: Option<ActiveCoolingEvidence>,
    /// Permit application after 29 May 2026: the A0 designation may apply.
    #[serde(default, rename = "permitApplicationAfter20260529")]
    pub permit_application_after_2026_05_29: bool,
    /// Row of Bbl table 4.148A for the requirement check.
    #[serde(default)]
    pub bbl_function: Option<BblFunction>,
    /// Several use functions of different kinds (Bbl art. 4.149 lid 2);
    /// replaces `bblFunction` when not empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bbl_functions: Vec<BblFunctionArea>,
    /// Loss area `A_ls` in m² for the `A_ls/A_g` ratio of table 4.148A.
    #[serde(default)]
    pub loss_area_m2: Option<f64>,
    #[serde(default)]
    pub loss_area_source_reference: Option<String>,
    /// Space cooling calculated here (chapter 10, method 3).
    #[serde(default)]
    pub cooling: Option<CoolingSystem>,
    /// Utility lighting per calculation zone (chapter 14); dwellings have
    /// `W_L = 0` and leave this empty.
    #[serde(default)]
    pub lighting: Vec<ZoneLighting>,
    /// Domestic hot water calculated here (chapter 13, one generator).
    #[serde(default)]
    pub hot_water: Option<HotWaterSystem>,
    /// Confirms the demand input uses the fixed C1 ventilation system of
    /// §5.4.3 and the fixed internal loads of §5.4.2; only then the need
    /// indicator is shown.
    pub demand_uses_fixed_c1_ventilation: bool,
    /// On-site electrical or thermal storage present (5.14a).
    pub battery_storage_present: bool,
    /// Capacities for `f_BAT;cor`; required when storage is present.
    #[serde(default)]
    pub storage: Option<EnergyStorage>,
    /// External heat, hot-water and cold supply (§5.8, annex P).
    #[serde(default)]
    pub external_supply: ExternalSupply,
}

/// Annex P values per external supply carrier; absent means the forfait
/// values of tables 5.2–5.4.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalSupply {
    /// Carrier dh: external heat for space heating.
    #[serde(default)]
    pub heating: Option<AnnexPRoute>,
    /// Carrier dw: external heat for hot water.
    #[serde(default)]
    pub hot_water: Option<AnnexPRoute>,
    /// Carrier dc: external cold.
    #[serde(default)]
    pub cooling: Option<AnnexPRoute>,
    /// Collective heat-pump source (9.6.8.1.1.2.3).
    #[serde(default)]
    pub collective_heat_pump_source: Option<CollectiveHeatPumpSource>,
}

/// Factors per external carrier for one scenario.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CarrierFactors {
    pub district_heat: SupplyFactors,
    pub district_hot_water: SupplyFactors,
    pub district_cold: SupplyFactors,
    pub heat_pump_source: Option<SupplyFactors>,
}

const FORFAIT_FACTORS: CarrierFactors = CarrierFactors {
    district_heat: HEAT_FORFAIT,
    district_hot_water: HEAT_FORFAIT,
    district_cold: COLD_FORFAIT,
    heat_pump_source: None,
};

/// Resolved annex P values and the EMGforf scenario.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSupplyResult {
    /// Factors of the main calculation (EMGverklaring when declared).
    pub declared: CarrierFactors,
    /// Factors of the EMGforf calculation.
    pub forfait: CarrierFactors,
    pub quality_declaration_used: bool,
    pub heating: Option<SystemResult>,
    pub hot_water: Option<SystemResult>,
    pub cooling: Option<SystemResult>,
    pub heat_pump_source: Option<SystemResult>,
    /// EP_Tot and EP_ren of the EMGforf calculation, kWh.
    pub forfait_primary_fossil_kwh: Option<f64>,
    pub forfait_renewable_primary_kwh: Option<f64>,
    pub forfait_co2_kg: Option<f64>,
}

fn resolve_external(
    input: &BuildingPerformanceInput,
    issues: &mut Vec<PerformanceIssue>,
) -> ExternalSupplyResult {
    let supply = &input.external_supply;
    let mut resolve = |route: &Option<AnnexPRoute>, function, path: &str| {
        route
            .as_ref()
            .and_then(|route| match assess_route(route, function, path) {
                Ok(result) => Some(result),
                Err(found) => {
                    issues.extend(found.into_iter().map(|item| issue(item.code, item.path)));
                    None
                }
            })
    };
    let heating = resolve(
        &supply.heating,
        SystemFunction::Heating,
        "externalSupply.heating",
    );
    let hot_water = resolve(
        &supply.hot_water,
        SystemFunction::HotWater,
        "externalSupply.hotWater",
    );
    let cooling = resolve(
        &supply.cooling,
        SystemFunction::Cooling,
        "externalSupply.cooling",
    );
    let mut source_result = None;
    let source: Option<ScenarioFactors> =
        supply
            .collective_heat_pump_source
            .as_ref()
            .and_then(|source| {
                match source_factors(source, "externalSupply.collectiveHeatPumpSource") {
                    Ok((factors, result)) => {
                        source_result = result;
                        Some(factors)
                    }
                    Err(found) => {
                        issues.extend(found.into_iter().map(|item| issue(item.code, item.path)));
                        None
                    }
                }
            });
    let declared = CarrierFactors {
        district_heat: heating.as_ref().map_or(HEAT_FORFAIT, |item| item.factors),
        district_hot_water: hot_water.as_ref().map_or(HEAT_FORFAIT, |item| item.factors),
        district_cold: cooling.as_ref().map_or(COLD_FORFAIT, |item| item.factors),
        heat_pump_source: source.map(|item| item.declared),
    };
    let forfait = CarrierFactors {
        heat_pump_source: source.map(|item| item.forfait),
        ..FORFAIT_FACTORS
    };
    ExternalSupplyResult {
        declared,
        forfait,
        quality_declaration_used: supply.heating.is_some()
            || supply.hot_water.is_some()
            || supply.cooling.is_some()
            || supply
                .collective_heat_pump_source
                .as_ref()
                .is_some_and(|source| source.annex_p.is_some()),
        heating,
        hot_water,
        cooling,
        heat_pump_source: source_result,
        forfait_primary_fossil_kwh: None,
        forfait_renewable_primary_kwh: None,
        forfait_co2_kg: None,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CarrierMonth {
    pub carrier: &'static str,
    pub month: u8,
    pub used_kwh: f64,
    pub delivered_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElectricityBalanceMonth {
    pub month: u8,
    pub used_kwh: f64,
    pub produced_kwh: f64,
    pub self_used_kwh: f64,
    pub exported_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildingPerformanceAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub chapter_5_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub attest_status: &'static str,
    pub label_available: bool,
    pub carriers: Vec<CarrierMonth>,
    pub electricity_balance: Vec<ElectricityBalanceMonth>,
    pub annual_primary_fossil_kwh: Option<f64>,
    pub annual_renewable_primary_kwh: Option<f64>,
    pub annual_heat_pump_ambient_heat_kwh: Option<f64>,
    pub annual_heating_and_cooling_need_kwh: Option<f64>,
    /// 5.14a `E_P;BAT,out;tot` summed over the year, kWh.
    pub annual_storage_correction_kwh: Option<f64>,
    /// §5.5.6.1 operational emission `m_CO2`, kg CO2eq per year.
    pub annual_co2_kg: Option<f64>,
    /// `m_CO2;spec = m_CO2 / A_g`, kg CO2eq/m².
    pub co2_kg_per_m2: Option<f64>,
    /// BENG 1, only with confirmed C1 ventilation.
    pub need_indicator_kwh_per_m2_year: Option<f64>,
    /// BENG 2.
    pub primary_fossil_indicator_kwh_per_m2_year: Option<f64>,
    /// BENG 3.
    pub renewable_share_percent: Option<f64>,
    /// Class from annex IX/X for the rounded BENG 2; not a registered label.
    pub indicative_label_class: Option<&'static str>,
    pub label_source: &'static str,
    /// TOjuli per zone and orientation (§5.7); requires the component route.
    pub tojuli: Vec<TojuliAssessment>,
    /// Highest TOjuli over all zones and orientations.
    pub tojuli_max_k: Option<f64>,
    pub tojuli_meets_bbl_limit: Option<bool>,
    /// Bbl 4.149 check; only with a function and a loss area.
    pub bbl_check: Option<BblCheck>,
    /// A0 designation check; only with a Bbl check and a qualifying permit date.
    pub a0_check: Option<A0Check>,
    pub space_heating: SpaceHeatingChainAssessment,
    /// Chapter 13, when calculated (generators, solar systems).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hot_water: Option<HotWaterAssessment>,
    /// Chapter 14 per zone, with the 7.28 internal gain for chapter 7.
    pub lighting: Vec<ZoneLightingResult>,
    pub indicators: Option<IndicatorsDraftAssessment>,
    /// Factors of the external supply and the EMGforf totals (§5.8).
    pub external_supply: Option<ExternalSupplyResult>,
    pub issues: Vec<PerformanceIssue>,
}

fn issue(code: &'static str, path: impl Into<String>) -> PerformanceIssue {
    PerformanceIssue {
        code,
        path: path.into(),
    }
}

fn twelve_nonnegative(values: &[f64]) -> bool {
    values.len() == 12
        && values
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0)
}

/// §5.7.1 system class against the calculated cooling generators.
fn active_cooling_matches(
    system: crate::tojuli::ActiveCoolingSystem,
    cooling: &CoolingSystem,
) -> bool {
    use crate::tojuli::ActiveCoolingSystem as A;
    cooling.generators.iter().any(|generator| {
        let kind = &generator.generator;
        match system {
            A::CompressionTable10_29 | A::HeatPumpWithCoolingEmitter => matches!(
                kind,
                CoolingGeneratorKind::Compression { .. }
                    | CoolingGeneratorKind::RoomAirConditioner { .. }
                    | CoolingGeneratorKind::UnknownCollective
                    | CoolingGeneratorKind::GasEngineCompression { .. }
            ),
            A::AbsorptionTable10_30 => matches!(
                kind,
                CoolingGeneratorKind::GasAbsorption { .. }
                    | CoolingGeneratorKind::AbsorptionExternalHeat { .. }
                    | CoolingGeneratorKind::AbsorptionChp { .. }
            ),
            A::FreeCoolingTable10_34 => matches!(
                kind,
                CoolingGeneratorKind::FreeCooling { source, .. }
                    if *source != FreeCoolingSource::DewPointCooling
            ),
            A::DewPointCoolingHumidifiedExhaust => matches!(
                kind,
                CoolingGeneratorKind::FreeCooling {
                    source: FreeCoolingSource::DewPointCooling,
                    ..
                }
            ),
            A::ExternalColdWithCoolingEmitter => {
                matches!(kind, CoolingGeneratorKind::ExternalCold)
            }
            A::SplitUnitsInEveryHabitableRoom => {
                matches!(kind, CoolingGeneratorKind::RoomAirConditioner { .. })
            }
            A::OtherUtility => true,
        }
    })
}

/// Chapter 13 context: scope, `A_g;tot` and the area-weighted heating setpoint.
fn hot_water_context(input: &BuildingPerformanceInput) -> HotWaterContext {
    let zones = std::iter::once(&input.space_heating.demand).chain(
        input
            .space_heating
            .additional_zones
            .iter()
            .map(|zone| &zone.demand),
    );
    let (weighted, area) = zones.fold((0.0, 0.0), |(weighted, area), zone| {
        (
            weighted + zone.setpoints.heating_c * zone.usable_floor_area_m2,
            area + zone.usable_floor_area_m2,
        )
    });
    HotWaterContext {
        residential: matches!(input.calculation_scope, CalculationScope::Residential),
        usable_floor_area_m2: input.total_usable_floor_area_m2,
        heated_ambient_c: if area > 0.0 { weighted / area } else { 20.0 },
        space_heating: None,
    }
}

/// 13.7.2.2.3 inputs of solar combi systems from a chain run without solar
/// gains: `Q_H;nod;out + Q_H;nod;ls`, `f_gebouw;si;H` and the design
/// temperatures of table 9.14 (class of the distribution system, else of a
/// product boiler, else 90/70 as in §9.4).
fn solar_space_heating(
    input: &BuildingPerformanceInput,
    heating: &SpaceHeatingChainAssessment,
) -> Option<SolarSpaceHeating> {
    if heating.monthly.len() != 12 {
        return None;
    }
    let chain = &input.space_heating;
    let zone_area: f64 = std::iter::once(&chain.demand)
        .chain(chain.additional_zones.iter().map(|zone| &zone.demand))
        .map(|demand| demand.usable_floor_area_m2)
        .sum();
    let building_fraction = chain
        .collective_connection
        .as_ref()
        .filter(|connection| connection.connected_usable_area_m2 > 0.0)
        .map_or(1.0, |connection| {
            (zone_area / connection.connected_usable_area_m2).min(1.0)
        });
    let class = chain
        .distribution_system
        .as_ref()
        .and_then(|system| system.design_temperature_class)
        .or(match &chain.generator {
            crate::space_heating_chain::Generator::ProductBoiler(generator) => {
                generator.design_temperature_class
            }
            _ => None,
        })
        .unwrap_or(crate::heating_distribution::DesignTemperatureClass::C90);
    let (supply, spread) = class.design();
    Some(SolarSpaceHeating {
        node_kwh: std::array::from_fn(|index| {
            let row = &heating.monthly[index];
            row.generator_output_kwh + row.solar_gain_kwh
        }),
        building_fraction,
        design_supply_c: supply,
        design_return_c: supply - spread,
    })
}

/// Chapter 10 for all zones on the one cooling system.
fn cooling_assessment(
    input: &BuildingPerformanceInput,
    heating: &SpaceHeatingChainAssessment,
) -> Option<CoolingAssessment> {
    let system = input.cooling.as_ref()?;
    let areas = std::iter::once(&input.space_heating.demand)
        .chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
        .map(|zone| zone.usable_floor_area_m2);
    let zones: Vec<CoolingZoneNeed> = std::iter::once(&heating.demand)
        .chain(&heating.additional_zone_demands)
        .zip(areas)
        .map(|(demand, area)| CoolingZoneNeed {
            usable_floor_area_m2: area,
            need_kwh: std::array::from_fn(|index| demand.monthly[index].cooling.need_kwh),
            ahu_load_kwh: std::array::from_fn(|index| {
                demand
                    .ventilation
                    .as_ref()
                    .and_then(|result| result.months.get(index))
                    .map_or(0.0, |month| month.ahu_cooling_kwh)
            }),
            // 10.19/10.20: without recoverable losses, with the supply-air
            // term of chapter 11.
            limit_need_kwh: Some(std::array::from_fn(|index| {
                let extra = demand
                    .ventilation
                    .as_ref()
                    .and_then(|result| result.months.get(index))
                    .map_or(0.0, |month| month.cooling_limit_air_kwh);
                crate::monthly_demand::cooling_need_with_extra_transfer(
                    &demand.monthly[index].cooling,
                    extra,
                )
            })),
        })
        .collect();
    // 10.84: heat taken from the source by the space-heating heat pump.
    let extraction: [f64; 12] = std::array::from_fn(|index| {
        let row = &heating.monthly[index];
        (row.heat_pump_output_kwh - row.generator_electricity_kwh).max(0.0)
    });
    Some(assess_cooling(
        system,
        CoolingContext {
            zones: &zones,
            residential: matches!(input.calculation_scope, CalculationScope::Residential),
            heat_pump_source_extraction_kwh: if input.space_heating.generator.heat_pump().is_some()
                || input.space_heating.generator.annex_q().is_some()
            {
                extraction
            } else {
                [0.0; 12]
            },
        },
    ))
}

fn validate(input: &BuildingPerformanceInput, issues: &mut Vec<PerformanceIssue>) {
    let collective_source = input
        .space_heating
        .generator
        .heat_pump()
        .is_some_and(|(_, source)| source != SourceSystem::Individual);
    if input.external_supply.collective_heat_pump_source.is_some() && !collective_source {
        issues.push(issue(
            "collective_heat_pump_source_unused",
            "externalSupply.collectiveHeatPumpSource",
        ));
    }
    if let Generator::ExternalHeat(generator) = &input.space_heating.generator {
        if generator.quality_declaration_present != input.external_supply.heating.is_some() {
            // §5.8: a declared supply needs its annex P values, and back.
            issues.push(issue(
                "external_heat_declaration_mismatch",
                "externalSupply.heating",
            ));
        }
    }
    if !input.total_usable_floor_area_m2.is_finite() || input.total_usable_floor_area_m2 <= 0.0 {
        issues.push(issue("usable_floor_area_invalid", "totalUsableFloorAreaM2"));
    }
    if input.area_source_reference.trim().is_empty() {
        issues.push(issue("source_reference_required", "areaSourceReference"));
    }
    if input.bacs_factor != 1.0 && input.bacs_factor != 1.05 {
        issues.push(issue("bacs_factor_invalid", "bacsFactor"));
    } else if input.bacs_factor == 1.05
        && matches!(input.calculation_scope, CalculationScope::Residential)
    {
        // §5.5.8: the 1,05 correction applies to utility buildings only.
        issues.push(issue("bacs_factor_residential_invalid", "bacsFactor"));
    }
    let zone_area: f64 = std::iter::once(&input.space_heating.demand)
        .chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
        .map(|zone| zone.usable_floor_area_m2)
        .sum();
    if (zone_area - input.total_usable_floor_area_m2).abs()
        > 1e-6 * input.total_usable_floor_area_m2.abs().max(1.0)
    {
        issues.push(issue("zone_area_sum_mismatch", "totalUsableFloorAreaM2"));
    }
    let residential = matches!(input.calculation_scope, CalculationScope::Residential);
    if let Some(area) = input.loss_area_m2 {
        if !area.is_finite() || area <= 0.0 {
            issues.push(issue("loss_area_invalid", "lossAreaM2"));
        }
        if input
            .loss_area_source_reference
            .as_deref()
            .map_or(true, |value| value.trim().is_empty())
        {
            issues.push(issue(
                "source_reference_required",
                "lossAreaSourceReference",
            ));
        }
    }
    match input.label_function {
        Some(LabelFunction::Residential) if !residential => {
            issues.push(issue("label_function_scope_mismatch", "labelFunction"));
        }
        Some(function) if residential && function != LabelFunction::Residential => {
            issues.push(issue("label_function_scope_mismatch", "labelFunction"));
        }
        _ => {}
    }
    if input.bacs_source_reference.trim().is_empty() {
        issues.push(issue("source_reference_required", "bacsSourceReference"));
    }
    if !input.use_inventory_complete {
        issues.push(issue("use_inventory_incomplete", "useInventoryComplete"));
    }
    if !input.production_inventory_complete {
        issues.push(issue(
            "production_inventory_incomplete",
            "productionInventoryComplete",
        ));
    }
    match (&input.storage, input.battery_storage_present) {
        (None, true) => issues.push(issue("storage_capacity_required", "storage")),
        (Some(_), false) => issues.push(issue(
            "storage_without_storage_present",
            "batteryStoragePresent",
        )),
        (Some(storage), true) => {
            for (value, field) in [
                (
                    storage.building_bound_electrical_kwh,
                    "storage.buildingBoundElectricalKwh",
                ),
                (
                    storage.building_bound_thermal_kwh,
                    "storage.buildingBoundThermalKwh",
                ),
            ] {
                if !value.is_finite() || value < 0.0 {
                    issues.push(issue("storage_capacity_invalid", field));
                }
            }
            if storage.source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    "storage.sourceReference",
                ));
            }
        }
        (None, false) => {}
    }
    let mut ids = HashSet::new();
    for (index, item) in input.declared_uses.iter().enumerate() {
        let path = format!("declaredUses[{index}]");
        if item.id.trim().is_empty() || !ids.insert(item.id.as_str()) {
            issues.push(issue("id_invalid", format!("{path}.id")));
        }
        if !twelve_nonnegative(&item.monthly_kwh) {
            issues.push(issue(
                "monthly_values_invalid",
                format!("{path}.monthlyKwh"),
            ));
        }
        if item.service.electric_only() && item.carrier != Carrier::El {
            issues.push(issue(
                "service_carrier_must_be_electricity",
                format!("{path}.carrier"),
            ));
        }
        if item.service == Service::Lighting
            && matches!(input.calculation_scope, CalculationScope::Residential)
        {
            // Draft 5.20 note 3: lighting energy is 0 for residential buildings.
            issues.push(issue(
                "residential_lighting_must_be_omitted",
                format!("{path}.service"),
            ));
        }
        if item.source_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                format!("{path}.sourceReference"),
            ));
        }
    }
    for (index, item) in input.declared_renewable_heat.iter().enumerate() {
        let path = format!("declaredRenewableHeat[{index}]");
        if item.id.trim().is_empty() || !ids.insert(item.id.as_str()) {
            issues.push(issue("id_invalid", format!("{path}.id")));
        }
        if !twelve_nonnegative(&item.monthly_kwh) {
            issues.push(issue(
                "monthly_values_invalid",
                format!("{path}.monthlyKwh"),
            ));
        }
        if item.source_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                format!("{path}.sourceReference"),
            ));
        }
    }
    if let Some(system) = &input.cooling {
        issues.extend(
            validate_cooling(system, "cooling")
                .into_iter()
                .map(|item| issue(item.code, item.path)),
        );
        if input.declared_uses.iter().any(|item| {
            matches!(
                item.service,
                Service::SpaceCooling | Service::SpaceCoolingAuxiliary
            )
        }) {
            issues.push(issue("cooling_double_count", "cooling"));
        }
        for (index, generator) in system.generators.iter().enumerate() {
            if matches!(
                generator.generator,
                CoolingGeneratorKind::AbsorptionChp { .. }
            ) {
                // Building CHP is not modelled in the heating chain.
                issues.push(issue(
                    "cooling_chp_unsupported",
                    format!("cooling.generators[{index}].generator"),
                ));
            }
        }
        if let Some(evidence) = &input.active_cooling {
            if !active_cooling_matches(evidence.system, system) {
                issues.push(issue(
                    "active_cooling_system_inconsistent",
                    "activeCooling.system",
                ));
            }
        }
    }
    let chapter_11_zones = std::iter::once(&input.space_heating.demand)
        .chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
        .any(|demand| demand.ventilation.is_some());
    if chapter_11_zones
        && input
            .declared_uses
            .iter()
            .any(|item| item.service == Service::VentilationFans)
    {
        // Chapter 11 already yields the fan energy.
        issues.push(issue("ventilation_fans_double_count", "declaredUses"));
    }
    if !input.pv_systems.is_empty()
        && input
            .on_site_production
            .iter()
            .any(|item| matches!(item.kind, ProductionKind::Pv | ProductionKind::Pvt))
    {
        // One route for solar electricity, so the same array cannot count twice.
        issues.push(issue("pv_route_mixed", "onSiteProduction"));
    }
    if input.hot_water.is_some() && !input.declared_renewable_heat.is_empty() {
        // Ambient heat of a calculated hot-water heat pump is derived here.
        issues.push(issue(
            "hot_water_renewable_double_count",
            "declaredRenewableHeat",
        ));
    }
    if input.active_cooling.is_some()
        && input.cooling.is_none()
        && !input
            .declared_uses
            .iter()
            .any(|item| item.service == Service::SpaceCooling)
    {
        // §5.7.1: TOjuli = 0 needs an active cooling system.
        issues.push(issue(
            "active_cooling_without_cooling_system",
            "activeCooling",
        ));
    }
    if !input.lighting.is_empty() {
        if residential {
            // 14.2.1: W_L;spec = 0 for the indicators of dwellings.
            issues.push(issue("residential_lighting_must_be_omitted", "lighting"));
        }
        if input
            .declared_uses
            .iter()
            .any(|item| item.service == Service::Lighting)
        {
            issues.push(issue("lighting_double_count", "lighting"));
        }
        let zones: Vec<(&str, f64)> = std::iter::once(&input.space_heating.demand)
            .chain(
                input
                    .space_heating
                    .additional_zones
                    .iter()
                    .map(|zone| &zone.demand),
            )
            .map(|zone| (zone.zone_id.as_str(), zone.usable_floor_area_m2))
            .collect();
        let mut seen = HashSet::new();
        for (index, item) in input.lighting.iter().enumerate() {
            let path = format!("lighting[{index}]");
            match zones.iter().find(|zone| zone.0 == item.zone_id) {
                Some((_, area)) if seen.insert(item.zone_id.as_str()) => issues.extend(
                    validate_lighting(item, *area, &path)
                        .into_iter()
                        .map(|found| issue(found.code, found.path)),
                ),
                _ => issues.push(issue("lighting_zone_unknown", format!("{path}.zoneId"))),
            }
        }
        if seen.len() != zones.len() {
            issues.push(issue("lighting_zone_missing", "lighting"));
        }
    }
    if let Some(system) = &input.hot_water {
        issues.extend(
            validate_hot_water(system, hot_water_context(input), "hotWater")
                .into_iter()
                .map(|item| issue(item.code, item.path)),
        );
        if input.declared_uses.iter().any(|item| {
            matches!(
                item.service,
                Service::DomesticHotWater | Service::DomesticHotWaterAuxiliary
            )
        }) {
            issues.push(issue("hot_water_double_count", "hotWater"));
        }
    }
    for (index, system) in input.pv_systems.iter().enumerate() {
        let path = format!("pvSystems[{index}]");
        if !ids.insert(system.id.as_str()) {
            issues.push(issue("id_invalid", format!("{path}.id")));
        }
        issues.extend(
            validate_pv(system, &path)
                .into_iter()
                .map(|item| issue(item.code, item.path)),
        );
    }
    for (index, item) in input.on_site_production.iter().enumerate() {
        let path = format!("onSiteProduction[{index}]");
        if item.id.trim().is_empty() || !ids.insert(item.id.as_str()) {
            issues.push(issue("id_invalid", format!("{path}.id")));
        }
        if !twelve_nonnegative(&item.monthly_kwh) {
            issues.push(issue(
                "monthly_values_invalid",
                format!("{path}.monthlyKwh"),
            ));
        }
        if item.source_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                format!("{path}.sourceReference"),
            ));
        }
    }
    match (
        input.space_heating.generator.heat_pump(),
        &input.heat_pump_renewable,
    ) {
        (Some((forfait, source_system)), evidence) => {
            if source_system != SourceSystem::Individual
                && input.external_supply.collective_heat_pump_source.is_none()
            {
                // 5.20/9.6.8.1.1.2.3: the source needs its class and evidence.
                issues.push(issue(
                    "collective_heat_pump_source_data_required",
                    "externalSupply.collectiveHeatPumpSource",
                ));
            }
            match evidence {
                None => issues.push(issue(
                    "heat_pump_renewable_evidence_required",
                    "heatPumpRenewable",
                )),
                Some(evidence) => {
                    if evidence.source_reference.trim().is_empty() {
                        issues.push(issue(
                            "source_reference_required",
                            "heatPumpRenewable.sourceReference",
                        ));
                    }
                    let source = forfait.source;
                    if evidence.combined_outdoor_and_exhaust_air {
                        // Table 9.27 footnote c: the outdoor-air row applies.
                        if source != TableSource::OutdoorAir || evidence.exhaust_air_source {
                            issues.push(issue(
                                "heat_pump_source_contradiction",
                                "heatPumpRenewable.combinedOutdoorAndExhaustAir",
                            ));
                        }
                    } else if evidence.outdoor_air_heat_fraction.is_some() {
                        issues.push(issue(
                            "outdoor_air_fraction_without_combined_source",
                            "heatPumpRenewable.outdoorAirHeatFraction",
                        ));
                    }
                    if let Some(fraction) = evidence.outdoor_air_heat_fraction {
                        if !(0.0..=1.0).contains(&fraction) {
                            issues.push(issue(
                                "outdoor_air_fraction_invalid",
                                "heatPumpRenewable.outdoorAirHeatFraction",
                            ));
                        }
                        if evidence
                            .outdoor_air_fraction_reference
                            .as_deref()
                            .map_or(true, |value| value.trim().is_empty())
                        {
                            issues.push(issue(
                                "source_reference_required",
                                "heatPumpRenewable.outdoorAirFractionReference",
                            ));
                        }
                    }
                    let exhaust_expected = source == TableSource::ExhaustAir;
                    let warm_source = matches!(
                        source,
                        TableSource::Collective20To40C | TableSource::CollectiveAtLeast40C
                    );
                    if evidence.exhaust_air_source != exhaust_expected
                        || (warm_source && evidence.source_below_20_c)
                    {
                        issues.push(issue(
                            "heat_pump_source_contradiction",
                            "heatPumpRenewable.exhaustAirSource",
                        ));
                    }
                }
            }
        }
        (None, evidence) => match (input.space_heating.generator.annex_q(), evidence) {
            (Some(generator), None) => {
                let _ = generator;
                issues.push(issue(
                    "heat_pump_renewable_evidence_required",
                    "heatPumpRenewable",
                ));
            }
            (Some(generator), Some(evidence)) => {
                if evidence.source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        "heatPumpRenewable.sourceReference",
                    ));
                }
                // 5.31/5.32 against the annex Q source.
                let source = generator.heat_pump.source;
                let consistent = match source {
                    AnnexQSource::ExhaustAirWater => {
                        evidence.exhaust_air_source && !evidence.combined_outdoor_and_exhaust_air
                    }
                    AnnexQSource::CombinedAirWater => {
                        evidence.combined_outdoor_and_exhaust_air && !evidence.exhaust_air_source
                    }
                    _ => !evidence.exhaust_air_source && !evidence.combined_outdoor_and_exhaust_air,
                };
                if !consistent {
                    issues.push(issue(
                        "heat_pump_source_contradiction",
                        "heatPumpRenewable.exhaustAirSource",
                    ));
                }
                if let Some(fraction) = evidence.outdoor_air_heat_fraction {
                    if !(0.0..=1.0).contains(&fraction) {
                        issues.push(issue(
                            "outdoor_air_fraction_invalid",
                            "heatPumpRenewable.outdoorAirHeatFraction",
                        ));
                    }
                }
            }
            (None, Some(_)) => {
                issues.push(issue(
                    "heat_pump_evidence_without_heat_pump",
                    "heatPumpRenewable",
                ));
            }
            (None, None) => {}
        },
    }
}

fn lighting_context(input: &BuildingPerformanceInput) -> LightingContext {
    let window_area: f64 = std::iter::once(&input.space_heating.demand)
        .chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
        .flat_map(|zone| zone.windows.iter())
        .map(|window| window.area_m2)
        .sum();
    LightingContext {
        total_usable_floor_area_m2: input.total_usable_floor_area_m2,
        total_window_area_m2: window_area,
    }
}

/// 7.28: replace `UtilityLighting::Chapter14` by the internal gain of the
/// zone's chapter 14 lighting, when that lighting is valid.
fn with_lighting_gains(input: &BuildingPerformanceInput) -> SpaceHeatingChainInput {
    let mut chain = input.space_heating.clone();
    let context = lighting_context(input);
    let resolve = |demand: &mut MonthlyDemandInput| {
        if let InternalGains::Utility { lighting, .. } = &mut demand.internal_gains {
            if matches!(lighting, UtilityLighting::Chapter14) {
                let zone = input
                    .lighting
                    .iter()
                    .find(|item| item.zone_id == demand.zone_id);
                if let Some(zone) = zone {
                    if validate_lighting(zone, demand.usable_floor_area_m2, "").is_empty() {
                        *lighting = UtilityLighting::Resolved {
                            gain_w: assess_zone_lighting(zone, context).internal_gain_w,
                        };
                    }
                }
            }
        }
    };
    resolve(&mut chain.demand);
    for zone in &mut chain.additional_zones {
        resolve(&mut zone.demand);
    }
    chain
}

/// 7.29 with 13.13/13.14: an empty `hotWaterRecoverableKwh` of a utility
/// zone takes the recoverable hot-water losses of the building's system,
/// split by usable floor area, when that system is valid.
fn with_hot_water_gains(
    input: &BuildingPerformanceInput,
    context: HotWaterContext,
    mut chain: SpaceHeatingChainInput,
) -> SpaceHeatingChainInput {
    let Some(system) = &input.hot_water else {
        return chain;
    };
    if !validate_hot_water(system, context, "hotWater").is_empty() {
        return chain;
    }
    let Ok(result) = assess_hot_water(system, context) else {
        return chain;
    };
    let total_area: f64 = std::iter::once(&chain.demand)
        .chain(chain.additional_zones.iter().map(|zone| &zone.demand))
        .map(|demand| demand.usable_floor_area_m2)
        .sum();
    if total_area <= 0.0 {
        return chain;
    }
    // 9.2.3.4: solar combi systems supply the space-heating node.
    if result
        .months
        .iter()
        .any(|month| month.solar_space_heating_kwh > 0.0)
    {
        chain.solar_heating_kwh = result
            .months
            .iter()
            .map(|month| month.solar_space_heating_kwh)
            .collect();
    }
    let fill = |demand: &mut MonthlyDemandInput| {
        let share = demand.usable_floor_area_m2 / total_area;
        if let InternalGains::Utility {
            hot_water_recoverable_kwh,
            ..
        } = &mut demand.internal_gains
        {
            if hot_water_recoverable_kwh.is_empty() {
                *hot_water_recoverable_kwh = result
                    .months
                    .iter()
                    .map(|month| month.recoverable_loss_kwh * share)
                    .collect();
            }
        }
    };
    fill(&mut chain.demand);
    for zone in &mut chain.additional_zones {
        fill(&mut zone.demand);
    }
    chain
}

pub fn assess_building_performance(
    input: &BuildingPerformanceInput,
) -> BuildingPerformanceAssessment {
    let fingerprint =
        input_fingerprint(&serde_json::to_value(input).expect("typed input serializes"));
    // The chain input with chapter 14 lighting (7.28) and hot-water (7.29)
    // gains filled in; TOjuli uses the same zone inputs.
    let mut hot_water_context = hot_water_context(input);
    let mut chain_input =
        with_hot_water_gains(input, hot_water_context, with_lighting_gains(input));
    let mut heating = assess_space_heating_chain(&chain_input);
    // 13.7.2.2.3: solar combi systems need the node output of a run without
    // solar gains; the second run takes their node gain (9.2.3.4).
    let combi = input.hot_water.as_ref().is_some_and(|system| {
        system
            .solar
            .iter()
            .any(|heater| heater.solar_use == crate::solar_thermal::SolarUse::Combi)
    });
    if combi {
        if let Some(space_heating) = solar_space_heating(input, &heating) {
            hot_water_context.space_heating = Some(space_heating);
            chain_input =
                with_hot_water_gains(input, hot_water_context, with_lighting_gains(input));
            heating = assess_space_heating_chain(&chain_input);
        }
    }
    let mut issues: Vec<PerformanceIssue> = heating
        .issues
        .iter()
        .map(|item| issue(item.code, format!("spaceHeating.{}", item.path)))
        .collect();
    validate(input, &mut issues);

    let mut carriers = Vec::new();
    let mut balance = Vec::new();
    let mut totals = None;
    let cooling = if issues.is_empty() {
        cooling_assessment(input, &heating)
    } else {
        None
    };
    let lighting: Vec<ZoneLightingResult> = if issues.is_empty() {
        let context = lighting_context(input);
        input
            .lighting
            .iter()
            .map(|zone| assess_zone_lighting(zone, context))
            .collect()
    } else {
        Vec::new()
    };
    let hot_water = match (&input.hot_water, issues.is_empty()) {
        (Some(system), true) => match assess_hot_water(system, hot_water_context) {
            Ok(result) => Some(result),
            Err(error) => {
                issues.push(issue(error.code, error.path));
                None
            }
        },
        _ => None,
    };
    let mut external = resolve_external(input, &mut issues);
    let mut forfait_totals = None;
    if issues.is_empty() {
        totals = Some(compute(
            input,
            &heating,
            cooling.as_ref(),
            hot_water.as_ref(),
            &lighting,
            external.declared,
            &mut carriers,
            &mut balance,
        ));
        if external.quality_declaration_used {
            // §5.3.1: EwePTot;EMGforf and RERPrenTot;EMGforf.
            forfait_totals = Some(compute(
                input,
                &heating,
                cooling.as_ref(),
                hot_water.as_ref(),
                &lighting,
                external.forfait,
                &mut Vec::new(),
                &mut Vec::new(),
            ));
        }
    }
    external.forfait_primary_fossil_kwh = forfait_totals.map(|item| item.fossil);
    external.forfait_renewable_primary_kwh = forfait_totals.map(|item| item.renewable);
    external.forfait_co2_kg = forfait_totals.map(|item| item.co2_kg);
    let mut indicators = None;
    if let Some(Totals {
        fossil,
        renewable,
        need,
        ..
    }) = totals
    {
        let mut scenarios = vec![AnnualScenario {
            kind: if forfait_totals.is_some() {
                ScenarioKind::EmgDeclaration
            } else {
                ScenarioKind::Ordinary
            },
            annual_primary_fossil_kwh: fossil,
            annual_renewable_kwh: renewable,
            source_reference: "derived by building_performance".into(),
        }];
        if let Some(forfait) = forfait_totals {
            scenarios.push(AnnualScenario {
                kind: ScenarioKind::EmgForfait,
                annual_primary_fossil_kwh: forfait.fossil,
                annual_renewable_kwh: forfait.renewable,
                source_reference: "derived by building_performance (tables 5.2–5.4)".into(),
            });
        }
        let result = assess_indicators_draft(&IndicatorsDraftInput {
            calculation_scope: input.calculation_scope,
            total_usable_floor_area_m2: input.total_usable_floor_area_m2,
            area_source_reference: input.area_source_reference.clone(),
            annual_need_c1_kwh: need,
            need_source_reference: "derived by monthly_demand".into(),
            scenarios,
        });
        issues.extend(
            result
                .issues
                .iter()
                .map(|item| issue(item.code, format!("indicators.{}", item.path))),
        );
        indicators = Some(result);
    }
    let valid = issues.is_empty();
    if !valid {
        carriers.clear();
        balance.clear();
    }
    let scenario = indicators
        .as_ref()
        .filter(|_| valid)
        .and_then(|result| result.scenarios.first());
    let totals = totals.filter(|_| valid);
    let need_from_fixed_c1 = totals.is_some_and(|item| item.need_from_fixed_c1);
    let need_indicator = indicators
        .as_ref()
        .filter(|_| valid && (input.demand_uses_fixed_c1_ventilation || need_from_fixed_c1))
        .and_then(|result| result.need_indicator_kwh_per_m2_year);
    // Bbl 4.149 paragraph 4: capacity weighted by usable floor area.
    let zone_demands = || {
        std::iter::once(&input.space_heating.demand).chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
    };
    let heating_capacity = {
        let assessed = std::iter::once(&heating.demand).chain(&heating.additional_zone_demands);
        let mut weighted = 0.0;
        let mut area = 0.0;
        for (demand, zone) in assessed.zip(zone_demands()) {
            weighted += demand
                .specific_heat_capacity_kj_per_m2k
                .unwrap_or(f64::INFINITY)
                * zone.usable_floor_area_m2;
            area += zone.usable_floor_area_m2;
        }
        if area > 0.0 {
            weighted / area
        } else {
            f64::INFINITY
        }
    };
    let residential = matches!(input.calculation_scope, CalculationScope::Residential);
    // a_C;red follows the zone's usage function (monthly_demand, 7.74/7.75).
    let tojuli: Vec<TojuliAssessment> = if valid {
        std::iter::once(&chain_input.demand)
            .chain(chain_input.additional_zones.iter().map(|zone| &zone.demand))
            .enumerate()
            .map(|(zone_index, zone)| {
                assess_tojuli(
                    zone,
                    TojuliOptions {
                        residential,
                        active_cooling: input.active_cooling.as_ref(),
                        // 10.6 per zone, July.
                        booster_heat_pump_july_kwh: cooling
                            .as_ref()
                            .map_or(0.0, |item| item.zone_booster_extraction_kwh[zone_index][6]),
                        // §5.7.2 step B: July recoverable losses of the zone.
                        heating_recoverable_july_kwh: heating
                            .zone_recoverable_losses
                            .iter()
                            .find(|item| item.zone_id == zone.zone_id)
                            .and_then(|item| item.monthly_kwh.get(6).copied())
                            .unwrap_or(0.0),
                        // Q_C;ls;rbl: chapter 10 has no recoverable cooling
                        // losses (L_C;zi = 0, pump heat goes to the load).
                        cooling_recoverable_july_kwh: 0.0,
                    },
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let tojuli_complete = !tojuli.is_empty()
        && tojuli
            .iter()
            .all(|item| item.status == "calculated_unverified" && item.max_tojuli_k.is_some());
    let tojuli_max = tojuli_complete.then(|| {
        tojuli
            .iter()
            .filter_map(|item| item.max_tojuli_k)
            .fold(0.0_f64, f64::max)
    });
    let bbl_functions: Vec<BblFunctionArea> = if input.bbl_functions.is_empty() {
        input
            .bbl_function
            .map(|function| BblFunctionArea {
                function,
                area_m2: input.total_usable_floor_area_m2,
            })
            .into_iter()
            .collect()
    } else {
        input.bbl_functions.clone()
    };
    let bbl = match (bbl_functions.is_empty(), input.loss_area_m2, scenario) {
        (false, Some(area), Some(item)) => bbl_check_mixed(
            &bbl_functions,
            area / input.total_usable_floor_area_m2,
            heating_capacity,
            need_indicator,
            Some(item.primary_fossil_indicator_kwh_per_m2_year),
            Some(item.renewable_share_percent),
        ),
        _ => None,
    };
    // Condition d of the A0 designation: no gas or oil burnt on site.
    let on_site_fossil_use = carriers
        .iter()
        .any(|item| matches!(item.carrier, "gas" | "oil") && item.used_kwh > 0.0);
    BuildingPerformanceAssessment {
        status: if valid {
            "calculated_unverified"
        } else {
            "invalid"
        },
        scope: "nta8800_single_zone_building_performance_unverified",
        chapter_5_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        final_edition_verified: false,
        reference_verified: false,
        attest_status: "unattested",
        label_available: false,
        carriers,
        electricity_balance: balance,
        annual_primary_fossil_kwh: totals.map(|item| item.fossil),
        annual_renewable_primary_kwh: totals.map(|item| item.renewable),
        annual_heat_pump_ambient_heat_kwh: totals.map(|item| item.ambient),
        annual_heating_and_cooling_need_kwh: totals.map(|item| item.need),
        annual_storage_correction_kwh: totals.map(|item| item.storage_correction),
        annual_co2_kg: totals.map(|item| item.co2_kg),
        co2_kg_per_m2: totals.map(|item| item.co2_kg / input.total_usable_floor_area_m2),
        need_indicator_kwh_per_m2_year: need_indicator,
        primary_fossil_indicator_kwh_per_m2_year: scenario
            .map(|item| item.primary_fossil_indicator_kwh_per_m2_year),
        renewable_share_percent: scenario.map(|item| item.renewable_share_percent),
        indicative_label_class: scenario.and_then(|item| {
            let function = match input.calculation_scope {
                CalculationScope::Residential => Some(LabelFunction::Residential),
                CalculationScope::Utility => input.label_function,
            };
            indicative_label_class(function?, item.primary_fossil_indicator_kwh_per_m2_year)
        }),
        label_source: LABEL_SOURCE,
        tojuli_max_k: tojuli_max,
        tojuli_meets_bbl_limit: tojuli_max.map(|value| value <= crate::tojuli::TOJULI_LIMIT_K),
        tojuli,
        a0_check: bbl
            .as_ref()
            .filter(|_| input.permit_application_after_2026_05_29)
            .map(|check| {
                a0_check(
                    check,
                    scenario.map(|item| item.primary_fossil_indicator_kwh_per_m2_year),
                    on_site_fossil_use,
                )
            }),
        bbl_check: bbl,
        space_heating: heating,
        hot_water,
        lighting: if valid { lighting } else { Vec::new() },
        indicators: indicators.filter(|_| valid),
        external_supply: valid.then_some(external),
        issues,
    }
}

#[derive(Debug, Clone, Copy)]
struct Totals {
    /// EPTot, kWh.
    fossil: f64,
    /// EPrenTot, kWh.
    renewable: f64,
    /// Heat-pump ambient heat, kWh.
    ambient: f64,
    /// Q_H+C;nd for BENG 1, kWh: the §5.4.2 C1 run when every zone has
    /// one, otherwise the need of the supplied ventilation.
    need: f64,
    need_from_fixed_c1: bool,
    /// 5.14a, kWh.
    storage_correction: f64,
    /// §5.5.6.1, kg CO2eq.
    co2_kg: f64,
}

/// 5.14a: `f_BAT;cor`.
fn storage_correction_factor(input: &BuildingPerformanceInput) -> f64 {
    match (&input.storage, input.battery_storage_present) {
        (Some(storage), true)
            if storage.building_bound_electrical_kwh + storage.building_bound_thermal_kwh
                >= STORAGE_MIN_CAPACITY_KWH =>
        {
            1.0
        }
        _ => 0.0,
    }
}

#[allow(clippy::too_many_arguments)]
fn compute(
    input: &BuildingPerformanceInput,
    heating: &SpaceHeatingChainAssessment,
    cooling: Option<&CoolingAssessment>,
    hot_water: Option<&HotWaterAssessment>,
    lighting: &[ZoneLightingResult],
    factors: CarrierFactors,
    carriers: &mut Vec<CarrierMonth>,
    balance: &mut Vec<ElectricityBalanceMonth>,
) -> Totals {
    let bacs = input.bacs_factor;
    let storage_factor = storage_correction_factor(input);
    let mut storage_correction = 0.0;
    let mut co2 = 0.0;
    let mut fossil = 0.0;
    let mut renewable = 0.0;
    let mut ambient_total = 0.0;
    let heat_pump_renewable = (input.space_heating.generator.heat_pump().is_some()
        || input.space_heating.generator.annex_q().is_some())
        && input
            .heat_pump_renewable
            .as_ref()
            .is_some_and(|evidence| evidence.source_below_20_c && !evidence.exhaust_air_source);
    // 5.32: only the outdoor-air share of a combined source is renewable.
    let outdoor_share = input
        .heat_pump_renewable
        .as_ref()
        .filter(|evidence| evidence.combined_outdoor_and_exhaust_air)
        .map_or(1.0, |evidence| {
            evidence.outdoor_air_heat_fraction.unwrap_or(0.0)
        });
    let cop = heating.generation_efficiency.unwrap_or(0.0);
    // 9.6.8.1.1.2.3: heat taken from a collective heat-pump source.
    let source = factors.heat_pump_source.filter(|_| {
        input
            .space_heating
            .generator
            .heat_pump()
            .is_some_and(|(_, system)| system != SourceSystem::Individual)
    });
    let pv_yields: Vec<[f64; 12]> = input
        .pv_systems
        .iter()
        .map(|system| monthly_yield_kwh(system, input.total_usable_floor_area_m2))
        .collect();
    let hot_water = hot_water.map(|result| &result.months);
    for index in 0..12 {
        let month = (index + 1) as u8;
        let row = &heating.monthly[index];
        // 5.20/5.21: space heating and its auxiliary energy weighted by f_BACS.
        let mut used_el =
            bacs * (row.generator_electricity_kwh + row.auxiliary_electricity_kwh.unwrap_or(0.0));
        let mut used_gas = bacs * row.natural_gas_kwh;
        let mut used_oil = bacs * row.oil_kwh;
        // Table 5.4: forfait external heat has f_Pren = 0, so it only adds EPTot.
        // 5.20: f_BACS applies to space heating on every carrier.
        let mut used_dh = bacs * row.district_heat_kwh;
        // 5.39g: the renewable share counts Q_H;gen;out of external heat and
        // Q_C;gen;out of absorption chillers on it, without f_BACS.
        let mut renewable_dh_basis = row.district_heat_kwh;
        let mut used_dc = 0.0;
        let mut used_dw = 0.0;
        for item in &input.declared_uses {
            let factor = if item.service.bacs_weighted() {
                bacs
            } else {
                1.0
            };
            let value = factor * item.monthly_kwh[index];
            match item.carrier {
                Carrier::El => used_el += value,
                Carrier::Gas => used_gas += value,
                Carrier::Oil => used_oil += value,
            }
        }
        // Chapter 12: steam humidifiers (12.3), carrier electricity or gas.
        used_el += heating.monthly[index].humidification_electricity_kwh;
        used_gas += heating.monthly[index].humidification_fuel_kwh;
        // Chapter 11: fans (11.132), frost protection (11.105) and grille
        // preheating (11.125); not weighted by f_BACS.
        for demand in std::iter::once(&heating.demand).chain(&heating.additional_zone_demands) {
            if let Some(ventilation) = &demand.ventilation {
                let row = &ventilation.months[index];
                used_el += row.fan_electricity_kwh
                    + row.frost_protection_electricity_kwh
                    + row.grille_preheating_electricity_kwh;
            }
        }
        // Chapter 10 cooling and its auxiliaries, weighted by f_BACS (5.20/5.21).
        let mut ambient_cold = 0.0;
        if let Some(assessment) = cooling {
            let month_row = &assessment.months[index];
            used_el += bacs * (month_row.electricity_kwh + month_row.auxiliary_electricity_kwh);
            used_gas += bacs * month_row.natural_gas_kwh;
            used_dh += bacs * month_row.district_heat_kwh;
            renewable_dh_basis += month_row.district_heat_cold_kwh;
            used_dc += bacs * month_row.district_cold_kwh;
            ambient_cold = month_row.ambient_cold_kwh;
        }
        // Chapter 14 lighting (electricity, months by t_mi/t_an).
        used_el += lighting
            .iter()
            .map(|zone| zone.monthly_kwh[index])
            .sum::<f64>();
        let mut hot_water_ambient = 0.0;
        // 5.39d: solar heat for hot water and the space-heating node.
        let mut solar_heat = row.solar_gain_kwh;
        if let Some(months) = &hot_water {
            let row = &months[index];
            // 13.1/13.3 per generator carrier; table 5.2 or annex P for
            // external heat (dw).
            used_el += row.electricity_kwh;
            used_gas += row.natural_gas_kwh;
            used_oil += row.oil_kwh;
            used_dw += row.district_heat_kwh;
            used_el += row.auxiliary_electricity_kwh;
            hot_water_ambient = row.ambient_heat_kwh;
            solar_heat += row.solar_renewable_kwh;
        }
        // 5.24/5.25 with E_nEPus;el = 0 (5.27): self-use capped at EP use.
        let produced: f64 = input
            .on_site_production
            .iter()
            .map(|item| item.monthly_kwh[index])
            .sum::<f64>()
            + pv_yields.iter().map(|yields| yields[index]).sum::<f64>();
        let self_used = produced.min(used_el);
        // 5.26 summed over producers.
        let exported = produced - self_used;
        // 5.16: delivered electricity is use minus self-used production.
        let delivered_el = used_el - self_used;
        for (carrier, used, delivered) in [
            (Carrier::El, used_el, delivered_el),
            (Carrier::Gas, used_gas, used_gas),
            (Carrier::Oil, used_oil, used_oil),
        ] {
            fossil += delivered * carrier.primary_factor();
            co2 += delivered * carrier.co2_factor();
            carriers.push(CarrierMonth {
                carrier: carrier.code(),
                month,
                used_kwh: used,
                delivered_kwh: delivered,
            });
        }
        // Tables 5.2/5.3 or annex P (§5.8) for external heat and hot water.
        let dh = factors.district_heat;
        let dw = factors.district_hot_water;
        fossil += used_dh * dh.primary_factor + used_dw * dw.primary_factor;
        co2 += used_dh * dh.co2_kg_per_kwh + used_dw * dw.co2_kg_per_kwh;
        // 5.20 and 9.6.8.1.1.2.3: Q_HD;hp;in;bron at the source factors,
        // not weighted by f_BACS.
        let source_heat = match source {
            Some(_) if cop >= 1.0 => row.heat_pump_output_kwh * (1.0 - 1.0 / cop),
            _ => 0.0,
        };
        if let Some(source) = source {
            fossil += source_heat * source.primary_factor;
            co2 += source_heat * source.co2_kg_per_kwh;
            if source_heat > 0.0 {
                carriers.push(CarrierMonth {
                    carrier: "dh_hp_source",
                    month,
                    used_kwh: source_heat,
                    delivered_kwh: source_heat,
                });
            }
        }
        let used_bm = bacs * row.biomass_kwh;
        fossil += used_bm * F_P_BIOMASS_B;
        co2 += used_bm * K_CO2_BIOMASS_B;
        if used_bm > 0.0 {
            carriers.push(CarrierMonth {
                carrier: "bm",
                month,
                used_kwh: used_bm,
                delivered_kwh: used_bm,
            });
        }
        if used_dh > 0.0 {
            carriers.push(CarrierMonth {
                carrier: "dh",
                month,
                used_kwh: used_dh,
                delivered_kwh: used_dh,
            });
        }
        if used_dw > 0.0 {
            carriers.push(CarrierMonth {
                carrier: "dw",
                month,
                used_kwh: used_dw,
                delivered_kwh: used_dw,
            });
        }
        // Tables 5.2–5.4 or annex P: external cold.
        let dc = factors.district_cold;
        fossil += used_dc * dc.primary_factor;
        co2 += used_dc * dc.co2_kg_per_kwh;
        if used_dc > 0.0 {
            carriers.push(CarrierMonth {
                carrier: "dc",
                month,
                used_kwh: used_dc,
                delivered_kwh: used_dc,
            });
        }
        // 5.10 and 5.13: exported electricity is subtracted at f_P;exp;el.
        fossil -= exported * F_P_ELECTRICITY;
        co2 -= exported * K_CO2_ELECTRICITY;
        // 5.14a/5.14b: all modelled producers are renewable (no CHP); the
        // correction is left out of the CO2 emission (§5.5.6.1).
        let correction = produced.min(used_el) * STORAGE_CORRECTION_FACTOR * storage_factor;
        fossil -= correction;
        storage_correction += correction;
        balance.push(ElectricityBalanceMonth {
            month,
            used_kwh: used_el,
            produced_kwh: produced,
            self_used_kwh: self_used,
            exported_kwh: exported,
        });

        // 5.30/5.31: ambient heat of the space-heating heat pump.
        // A collective source counts through f_Pren;dh;hp;in;bron instead.
        let ambient = if heat_pump_renewable && source.is_none() && cop >= 1.0 {
            row.heat_pump_output_kwh * (1.0 - 1.0 / cop) * outdoor_share
        } else {
            0.0
        };
        ambient_total += ambient;
        let declared_heat: f64 = input
            .declared_renewable_heat
            .iter()
            .map(|item| item.monthly_kwh[index])
            .sum();
        // 5.29 and 5.39.
        // 5.30: biomass counts its delivered heat with f_Pren;bmB.
        let biomass_heat = if row.biomass_kwh > 0.0 {
            row.generator_output_kwh
        } else {
            0.0
        };
        // 5.39: external supply at f_Pren;dX and the collective source.
        renewable += renewable_dh_basis * dh.renewable_factor
            + used_dw * dw.renewable_factor
            + used_dc * dc.renewable_factor
            + source.map_or(0.0, |item| source_heat * item.renewable_factor);
        renewable += (ambient + declared_heat + hot_water_ambient + solar_heat) * F_PREN_RENHEAT
            + biomass_heat * F_PREN_BIOMASS_B
            + ambient_cold * F_PREN_RENCOLD
            + produced * F_PREN_RENELECT;
    }
    let zone_demands = || std::iter::once(&heating.demand).chain(&heating.additional_zone_demands);
    // §5.4.2: the fixed C1 run, when chapter 11 supplied it for every zone.
    let fixed_c1_need: Option<f64> = zone_demands()
        .map(|demand| {
            demand
                .fixed_c1
                .as_ref()
                .filter(|run| run.status == "calculated_unverified")
                .map(|run| {
                    run.annual_heating_need_kwh.unwrap_or(0.0)
                        + run.annual_cooling_need_kwh.unwrap_or(0.0)
                })
        })
        .sum();
    let need = fixed_c1_need.unwrap_or_else(|| {
        zone_demands()
            // §5.4.2: without the recoverable system losses.
            .map(|demand| {
                demand
                    .annual_heating_need_without_recoverable_kwh
                    .unwrap_or(0.0)
                    + demand
                        .annual_cooling_need_without_recoverable_kwh
                        .unwrap_or(0.0)
            })
            .sum()
    });
    Totals {
        fossil,
        renewable,
        ambient: ambient_total,
        need,
        need_from_fixed_c1: fixed_c1_need.is_some(),
        storage_correction,
        co2_kg: co2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annex_p::{AnnexPRoute, CollectiveHeatPumpSource};
    use crate::space_heating_chain::HeatPumpGenerator;
    use serde_json::json;

    fn chain() -> SpaceHeatingChainInput {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-space-heating-chain-synthetic.json"
        ))
        .unwrap()
    }

    /// Hydronic system whose pump is outside the plot (test fixture only).
    fn none_on_site_system() -> crate::space_heating_chain::DistributionSystem {
        serde_json::from_value(json!({
            "installation": "individual",
            "usageFunction": "residential",
            "connectedStoreys": 1,
            "pipeTransmittance": {"method": "forfait", "insulation": {"state": "unknown"}},
            "valvesInsulated": false,
            "pump": {"method": "none_on_site", "sourceReference": "pump in the supplier's station"},
            "sourceReference": "synthetic"
        }))
        .unwrap()
    }

    fn input() -> BuildingPerformanceInput {
        serde_json::from_value(json!({
            "calculationScope": "residential",
            "totalUsableFloorAreaM2": 100.0,
            "areaSourceReference": "synthetic plan",
            "spaceHeating": chain(),
            "bacsFactor": 1.0,
            "bacsSourceReference": "residential, no BACS",
            "useInventoryComplete": true,
            "declaredUses": [
                {"id": "dhw", "service": "domestic_hot_water", "carrier": "gas",
                 "monthlyKwh": vec![150.0; 12], "sourceReference": "synthetic chapter 13"},
                {"id": "fans", "service": "ventilation_fans", "carrier": "el",
                 "monthlyKwh": vec![20.0; 12], "sourceReference": "synthetic chapter 11"}
            ],
            "productionInventoryComplete": true,
            "onSiteProduction": [
                {"id": "pv", "kind": "pv",
                 "monthlyKwh": [20.0, 40.0, 80.0, 110.0, 140.0, 140.0, 140.0, 120.0, 90.0, 60.0, 25.0, 15.0],
                 "sourceReference": "synthetic chapter 16"}
            ],
            "demandUsesFixedC1Ventilation": false,
            "batteryStoragePresent": false
        }))
        .unwrap()
    }

    #[test]
    fn primary_energy_follows_5_10_with_pv_self_use_and_export() {
        let sample = input();
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let heating = &result.space_heating;
        let mut expected = 0.0;
        let pv: [f64; 12] = [
            20.0, 40.0, 80.0, 110.0, 140.0, 140.0, 140.0, 120.0, 90.0, 60.0, 25.0, 15.0,
        ];
        for (index, row) in heating.monthly.iter().enumerate() {
            let el = row.auxiliary_electricity_kwh.unwrap() + 20.0;
            let gas = row.natural_gas_kwh + 150.0;
            let self_used = pv[index].min(el);
            let exported = pv[index] - self_used;
            expected += (el - self_used) * 1.45 + gas * 1.0 - exported * 1.45;
            let month = &result.electricity_balance[index];
            assert!((month.self_used_kwh - self_used).abs() < 1e-9);
            assert!((month.exported_kwh - exported).abs() < 1e-9);
        }
        let fossil = result.annual_primary_fossil_kwh.unwrap();
        assert!((fossil - expected).abs() < 1e-6);
        // Equal electricity factors make the net result month-independent.
        let el_total: f64 = heating
            .monthly
            .iter()
            .map(|row| row.auxiliary_electricity_kwh.unwrap() + 20.0)
            .sum();
        let gas_total = heating.annual_natural_gas_kwh.unwrap() + 1800.0;
        let pv_total: f64 = pv.iter().sum();
        assert!((fossil - ((el_total - pv_total) * 1.45 + gas_total)).abs() < 1e-6);
        let renewable = result.annual_renewable_primary_kwh.unwrap();
        assert!((renewable - pv_total * 1.45).abs() < 1e-9);
        // BENG 2 rounded up to 0,01 and BENG 3 down to 0,1.
        let beng2 = result.primary_fossil_indicator_kwh_per_m2_year.unwrap();
        assert!(beng2 >= fossil / 100.0 && beng2 - fossil / 100.0 < 0.01);
        let share = 100.0 * renewable / (fossil + renewable);
        let beng3 = result.renewable_share_percent.unwrap();
        assert!(beng3 <= share && share - beng3 < 0.1);
        // Annex IX on the rounded BENG 2.
        let expected_class = indicative_label_class(LabelFunction::Residential, beng2).unwrap();
        assert_eq!(result.indicative_label_class, Some(expected_class));
        // Without confirmed C1 ventilation no need indicator is reported.
        assert!(result.need_indicator_kwh_per_m2_year.is_none());
        assert!(!result.label_available);
    }

    #[test]
    fn need_indicator_only_with_confirmed_c1() {
        let mut sample = input();
        sample.demand_uses_fixed_c1_ventilation = true;
        let result = assess_building_performance(&sample);
        let need = result.annual_heating_and_cooling_need_kwh.unwrap();
        let beng1 = result.need_indicator_kwh_per_m2_year.unwrap();
        assert!(beng1 >= need / 100.0 && beng1 - need / 100.0 < 0.01);
    }

    #[test]
    fn chapter_11_ventilation_gives_beng1_and_fan_energy() {
        let mut sample = input();
        let without = assess_building_performance(&sample);
        let demand = &mut sample.space_heating.demand;
        demand.ventilation_flows.clear();
        demand.ventilation = Some(
            serde_json::from_value(serde_json::json!({
                "zoneId": demand.zone_id,
                "usableFloorAreaM2": demand.usable_floor_area_m2,
                "category": "residential",
                "functions": [{"function": "residential", "areaM2": demand.usable_floor_area_m2}],
                "dwellingCount": 1,
                "buildingHeightM": 9.0,
                "constructionYear": 2020,
                "heatingSetpointC": demand.setpoints.heating_c,
                "coolingSetpointC": demand.setpoints.cooling_c,
                "system": {"kind": "single", "unit": {"variant": "c4a", "ducts": "luka_a_b_c", "equipmentReference": "synthetic"}},
                "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "synthetic"},
                "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020},
                "sourceReference": "synthetic"
            }))
            .unwrap(),
        );
        sample
            .declared_uses
            .retain(|item| item.service != Service::VentilationFans);
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert!(!sample.demand_uses_fixed_c1_ventilation);
        let beng1 = result.need_indicator_kwh_per_m2_year.unwrap();
        let fixed = result.space_heating.demand.fixed_c1.as_ref().unwrap();
        let need = fixed.annual_heating_need_kwh.unwrap() + fixed.annual_cooling_need_kwh.unwrap();
        assert!((result.annual_heating_and_cooling_need_kwh.unwrap() - need).abs() < 1e-9);
        assert!(beng1 > 0.0);
        let fans = result
            .space_heating
            .demand
            .ventilation
            .as_ref()
            .unwrap()
            .annual_fan_electricity_kwh;
        assert!(fans > 0.0);
        assert!(without.need_indicator_kwh_per_m2_year.is_none());
        // Declared fan energy next to chapter 11 counts twice.
        sample.declared_uses.push(DeclaredUse {
            id: "fans".into(),
            service: Service::VentilationFans,
            carrier: Carrier::El,
            monthly_kwh: vec![10.0; 12],
            source_reference: "x".into(),
        });
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "ventilation_fans_double_count"));
    }

    fn external_heat_sample(declared: bool) -> BuildingPerformanceInput {
        use crate::space_heating_chain::ExternalHeatGenerator;
        let mut sample = input();
        sample.space_heating.generator = Generator::ExternalHeat(ExternalHeatGenerator {
            supplier_reference: "contract".into(),
            quality_declaration_present: declared,
            auxiliary: Some(crate::space_heating_chain::OtherGeneratorAuxiliary {
                electrically_connected_devices: 1,
                nominal_power_kw: None,
                source_reference: "delivery set".into(),
            }),
        });
        sample.space_heating.distribution_system = Some(none_on_site_system());
        sample
    }

    #[test]
    fn annex_p_fixture_resolves_all_carriers() {
        let supply: ExternalSupply = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-annex-p-synthetic.json"
        ))
        .unwrap();
        let mut sample = input();
        sample.external_supply = supply;
        let mut issues = Vec::new();
        let result = resolve_external(&sample, &mut issues);
        assert!(issues.is_empty(), "{issues:?}");
        assert!(result.quality_declaration_used);
        let heat = result.declared.district_heat;
        assert!(heat.primary_factor > 0.0 && heat.primary_factor < 0.9);
        assert!(heat.renewable_factor > 0.5);
        assert_eq!(result.declared.district_hot_water.primary_factor, 0.55);
        // EER 20 ≥ 8: fully renewable generator (5.49).
        let cold = result.cooling.as_ref().unwrap();
        assert_eq!(cold.generators[0].renewable_factor, 1.0);
        assert_eq!(result.forfait.district_heat, HEAT_FORFAIT);
        assert_eq!(result.forfait.district_cold, COLD_FORFAIT);
    }

    #[test]
    fn annex_p_declaration_gives_two_scenarios() {
        let forfait = assess_building_performance(&external_heat_sample(false));
        let mut sample = external_heat_sample(true);
        let mismatch = assess_building_performance(&sample);
        assert!(mismatch
            .issues
            .iter()
            .any(|item| item.code == "external_heat_declaration_mismatch"));
        sample.external_supply.heating = Some(AnnexPRoute::Declared {
            primary_factor: 0.42,
            renewable_factor: 0.35,
            co2_kg_per_kwh: 0.05,
            declaration_reference: "BCRG EMG-verklaring".into(),
        });
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let heat = result.space_heating.annual_district_heat_kwh.unwrap();
        let ep = result.annual_primary_fossil_kwh.unwrap();
        let ep_forfait = forfait.annual_primary_fossil_kwh.unwrap();
        assert!((ep - (ep_forfait - heat * 0.9 + heat * 0.42)).abs() < 1e-6);
        let ren = result.annual_renewable_primary_kwh.unwrap();
        let ren_forfait = forfait.annual_renewable_primary_kwh.unwrap();
        assert!((ren - (ren_forfait + heat * 0.35)).abs() < 1e-6);
        let co2 = result.annual_co2_kg.unwrap();
        assert!((co2 - (forfait.annual_co2_kg.unwrap() - heat * 0.09 + heat * 0.05)).abs() < 1e-6);
        let external = result.external_supply.as_ref().unwrap();
        assert!(external.quality_declaration_used);
        assert!((external.forfait_primary_fossil_kwh.unwrap() - ep_forfait).abs() < 1e-6);
        let kinds: Vec<_> = result
            .indicators
            .as_ref()
            .unwrap()
            .scenarios
            .iter()
            .map(|item| item.kind)
            .collect();
        assert_eq!(
            kinds,
            [ScenarioKind::EmgDeclaration, ScenarioKind::EmgForfait]
        );
        assert!(!forfait.external_supply.unwrap().quality_declaration_used);
        // 5.39g: the renewable share counts Q_H;gen;out, not weighted by f_BACS.
        sample.calculation_scope = CalculationScope::Utility;
        let plain = assess_building_performance(&sample);
        sample.bacs_factor = 1.05;
        let weighted = assess_building_performance(&sample);
        assert_eq!(weighted.status, "calculated_unverified");
        assert!(
            (weighted.annual_renewable_primary_kwh.unwrap()
                - plain.annual_renewable_primary_kwh.unwrap())
            .abs()
                < 1e-6
        );
        assert!(
            weighted.annual_primary_fossil_kwh.unwrap() > plain.annual_primary_fossil_kwh.unwrap()
        );
    }

    #[test]
    fn collective_heat_pump_source_uses_its_own_factors() {
        use crate::annex_p::SourceTemperatureClass;
        let mut sample = input();
        let forfait = crate::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput {
            generator_id: "hp".into(),
            classification_source_reference: "system design".into(),
            scope: crate::forfait_heat_pump_draft::TableScope::ResidentialAtMost25Kw,
            source: TableSource::Ground,
            sink: crate::forfait_heat_pump_draft::TableSink::Hydronic,
            design_supply_temperature_c: Some(35.0),
            source_correction_factor: Some(1.0),
            source_correction_reference: Some("table V.1".into()),
            thermal_capacity_kw: Some(8.0),
            capacity_source_reference: Some("rated".into()),
            collective_building_installation: Some(false),
            row_variant: crate::forfait_heat_pump_draft::TableRowVariant::Base,
            high_efficiency_evidence: None,
            source_temperature_c: None,
            source_temperature_evidence_reference: None,
            source_quality_declaration_reference: None,
        };
        sample.space_heating.generator = Generator::HeatPumpForfait(HeatPumpGenerator {
            forfait,
            source_system: SourceSystem::CollectiveGround,
            source_system_reference: "district ground loop".into(),
            auxiliary_measurements: None,
            auxiliary: None,
        });
        sample.heat_pump_renewable = Some(HeatPumpRenewableEvidence {
            source_below_20_c: true,
            exhaust_air_source: false,
            source_reference: "ground loop".into(),
            combined_outdoor_and_exhaust_air: false,
            outdoor_air_heat_fraction: None,
            outdoor_air_fraction_reference: None,
        });
        let missing = assess_building_performance(&sample);
        assert!(missing
            .issues
            .iter()
            .any(|item| item.code == "collective_heat_pump_source_data_required"));
        sample.external_supply.collective_heat_pump_source = Some(CollectiveHeatPumpSource {
            temperature_class: SourceTemperatureClass::Below20C,
            supplier_reference: "invoice".into(),
            annex_p: None,
        });
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let cop = result.space_heating.generation_efficiency.unwrap();
        let source: f64 = result
            .space_heating
            .monthly
            .iter()
            .map(|row| row.heat_pump_output_kwh * (1.0 - 1.0 / cop))
            .sum();
        assert!(source > 0.0);
        let listed: f64 = result
            .carriers
            .iter()
            .filter(|item| item.carrier == "dh_hp_source")
            .map(|item| item.used_kwh)
            .sum();
        assert!((listed - source).abs() < 1e-6);
        // The ambient heat counts through f_Pren;dh;hp;in;bron (0,95).
        assert_eq!(result.annual_heat_pump_ambient_heat_kwh, Some(0.0));
        // Below 20 °C: no paired forfait scenario changes (one value in both).
        assert!(!result.external_supply.unwrap().quality_declaration_used);
    }

    #[test]
    fn heat_pump_ambient_heat_counts_as_renewable() {
        let mut sample = input();
        let forfait = crate::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput {
            generator_id: "hp".into(),
            classification_source_reference: "system design".into(),
            scope: crate::forfait_heat_pump_draft::TableScope::ResidentialAtMost25Kw,
            source: TableSource::OutdoorAir,
            sink: crate::forfait_heat_pump_draft::TableSink::Hydronic,
            design_supply_temperature_c: Some(35.0),
            source_correction_factor: None,
            source_correction_reference: None,
            thermal_capacity_kw: Some(8.0),
            capacity_source_reference: Some("rated".into()),
            collective_building_installation: Some(false),
            row_variant: crate::forfait_heat_pump_draft::TableRowVariant::Base,
            high_efficiency_evidence: None,
            source_temperature_c: None,
            source_temperature_evidence_reference: None,
            source_quality_declaration_reference: None,
        };
        sample.space_heating.generator = Generator::HeatPumpForfait(HeatPumpGenerator {
            forfait,
            source_system: SourceSystem::Individual,
            source_system_reference: "own unit".into(),
            auxiliary_measurements: None,
            auxiliary: None,
        });
        let missing = assess_building_performance(&sample);
        assert!(missing
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_renewable_evidence_required"));
        sample.heat_pump_renewable = Some(HeatPumpRenewableEvidence {
            source_below_20_c: true,
            exhaust_air_source: false,
            source_reference: "outdoor air".into(),
            combined_outdoor_and_exhaust_air: false,
            outdoor_air_heat_fraction: None,
            outdoor_air_fraction_reference: None,
        });
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let cop = result.space_heating.generation_efficiency.unwrap();
        let output: f64 = result
            .space_heating
            .monthly
            .iter()
            .map(|row| row.generator_output_kwh)
            .sum();
        let ambient = result.annual_heat_pump_ambient_heat_kwh.unwrap();
        assert!((ambient - output * (1.0 - 1.0 / cop)).abs() < 1e-6);
        assert_eq!(result.space_heating.annual_natural_gas_kwh, Some(0.0));
        sample
            .heat_pump_renewable
            .as_mut()
            .unwrap()
            .exhaust_air_source = true;
        let contradiction = assess_building_performance(&sample);
        assert!(contradiction
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_source_contradiction"));

        // 5.32: outdoor air plus exhaust air counts only the outdoor share.
        let evidence = sample.heat_pump_renewable.as_mut().unwrap();
        evidence.exhaust_air_source = false;
        evidence.combined_outdoor_and_exhaust_air = true;
        let unknown = assess_building_performance(&sample);
        assert_eq!(unknown.annual_heat_pump_ambient_heat_kwh, Some(0.0));
        let evidence = sample.heat_pump_renewable.as_mut().unwrap();
        evidence.outdoor_air_heat_fraction = Some(0.4);
        evidence.outdoor_air_fraction_reference = Some("quality declaration".into());
        let combined = assess_building_performance(&sample);
        let ambient_combined = combined.annual_heat_pump_ambient_heat_kwh.unwrap();
        assert!((ambient_combined - 0.4 * ambient).abs() < 1e-6);
    }

    #[test]
    fn rejects_unsupported_or_incomplete_input_without_numbers() {
        let mut sample = input();
        sample.battery_storage_present = true;
        sample.production_inventory_complete = false;
        sample.declared_uses[1].carrier = Carrier::Gas;
        sample.declared_uses.push(DeclaredUse {
            id: "light".into(),
            service: Service::Lighting,
            carrier: Carrier::El,
            monthly_kwh: vec![1.0; 12],
            source_reference: "x".into(),
        });
        let result = assess_building_performance(&sample);
        assert_eq!(result.status, "invalid");
        assert!(result.annual_primary_fossil_kwh.is_none());
        assert!(result.primary_fossil_indicator_kwh_per_m2_year.is_none());
        assert!(result.carriers.is_empty());
        let codes: Vec<_> = result.issues.iter().map(|item| item.code).collect();
        for code in [
            "storage_capacity_required",
            "production_inventory_incomplete",
            "service_carrier_must_be_electricity",
            "residential_lighting_must_be_omitted",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    #[test]
    fn calculated_pv_adds_to_declared_production() {
        let mut sample = input();
        // Mixing both PV routes is rejected; use wind as declared production.
        sample.on_site_production[0].kind = ProductionKind::Wind;
        let base = assess_building_performance(&sample);
        sample.pv_systems.push(PvSystem {
            id: "roof-pv".into(),
            peak_power: crate::pv::PeakPower::Panels {
                panel_peak_power_w: 300.0,
                panel_count: 10,
            },
            azimuth_deg: 180.0,
            tilt_deg: 35.0,
            mounting: crate::pv::PvMounting::ModeratelyVentilated,
            obstruction_factors: vec![1.0],
            collective: None,
            source_reference: "datasheet".into(),
        });
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let pv: f64 = monthly_yield_kwh(&sample.pv_systems[0], 100.0).iter().sum();
        let delta = result.annual_renewable_primary_kwh.unwrap()
            - base.annual_renewable_primary_kwh.unwrap();
        assert!((delta - pv * 1.45).abs() < 1e-6);
        let fossil_delta =
            base.annual_primary_fossil_kwh.unwrap() - result.annual_primary_fossil_kwh.unwrap();
        assert!((fossil_delta - pv * 1.45).abs() < 1e-6);
    }

    #[test]
    fn calculated_hot_water_replaces_declared_use_and_blocks_double_count() {
        use crate::domestic_hot_water::{
            GasAppliance, HotWaterEmission, HotWaterGenerator, HotWaterNeed, ServedTaps,
        };
        let mut sample = input();
        let system = HotWaterSystem {
            need: HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "one dwelling".into(),
            },
            emission: HotWaterEmission::Residential {
                served: ServedTaps::KitchenAndBathroom,
                kitchen_length_m: Some(1.0),
                bathroom_length_m: Some(1.0),
                source_reference: "drawing".into(),
            },
            shower_heat_recovery: None,
            circulation: None,
            storage: Vec::new(),
            delivery_sets: None,
            boiling_water_tap: false,
            generator: HotWaterGenerator::GasAppliance {
                appliance: GasAppliance::CombiGaskeurHrCw,
                measured_class: Some(crate::domestic_hot_water::ApplicationClass::Class1),
                kitchen_only: false,
                declared: None,
                annex_t: None,
                annex_t_conditions: None,
            },
            nominal_power_kw: None,
            exhaust_air: None,
            additional_generators: Vec::new(),
            series: None,
            solar: Vec::new(),
            collective: None,
            equipment_reference: "plate".into(),
        };
        sample.hot_water = Some(system.clone());
        let doubled = assess_building_performance(&sample);
        assert!(doubled
            .issues
            .iter()
            .any(|item| item.code == "hot_water_double_count"));
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let gas: f64 = result
            .carriers
            .iter()
            .filter(|item| item.carrier == "gas")
            .map(|item| item.used_kwh)
            .sum();
        // Short draw-off lines (η_em = 1) and class 1 (c_W;gen = 1).
        let expected = result.space_heating.annual_natural_gas_kwh.unwrap() + 856.0 * 2.28 / 0.675;
        assert!((gas - expected).abs() < 1e-6);
    }

    #[test]
    fn solar_water_heating_counts_as_renewable_and_combi_feeds_the_node() {
        use crate::domestic_hot_water::{
            GasAppliance, HotWaterEmission, HotWaterGenerator, HotWaterNeed, ServedTaps,
        };
        let mut sample = input();
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        let mut system = HotWaterSystem {
            need: HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "one dwelling".into(),
            },
            emission: HotWaterEmission::Residential {
                served: ServedTaps::KitchenAndBathroom,
                kitchen_length_m: Some(1.0),
                bathroom_length_m: Some(1.0),
                source_reference: "drawing".into(),
            },
            shower_heat_recovery: None,
            circulation: None,
            storage: Vec::new(),
            delivery_sets: None,
            boiling_water_tap: false,
            generator: HotWaterGenerator::GasAppliance {
                appliance: GasAppliance::CombiGaskeurHrCw,
                measured_class: Some(crate::domestic_hot_water::ApplicationClass::Class1),
                kitchen_only: false,
                declared: None,
                annex_t: None,
                annex_t_conditions: None,
            },
            nominal_power_kw: None,
            exhaust_air: None,
            additional_generators: Vec::new(),
            series: None,
            solar: Vec::new(),
            collective: None,
            equipment_reference: "plate".into(),
        };
        sample.hot_water = Some(system.clone());
        let plain = assess_building_performance(&sample);
        assert_eq!(plain.status, "calculated_unverified", "{:?}", plain.issues);
        let heater: crate::solar_thermal::SolarWaterHeater =
            serde_json::from_value(serde_json::json!({
                "id": "roof", "solarUse": "water_heating",
                "method": {"method": "calculated", "solarType": "preheater",
                    "collectors": {"moduleAreaM2": 2.0, "moduleCount": 2, "orientation": "south",
                        "tiltDeg": 45.0, "obstruction": {"method": "minimal"},
                        "efficiency": {"method": "forfait", "collector": "glazed"},
                        "loopPipes": {"method": "forfait"}},
                    "storage": {"totalVolumeL": 200.0, "loss": {"method": "label", "label": "b"}}},
                "sourceReference": "datasheet"
            }))
            .unwrap();
        system.solar = vec![heater.clone()];
        sample.hot_water = Some(system.clone());
        let solar = assess_building_performance(&sample);
        assert_eq!(solar.status, "calculated_unverified", "{:?}", solar.issues);
        let hot_water = solar.hot_water.as_ref().unwrap();
        let solar_heat = hot_water.annual_solar_renewable_kwh;
        assert!(solar_heat > 0.0);
        // 5.39d: f_Pren;renheat on the solar heat; the pump adds electricity.
        let gain = solar.annual_renewable_primary_kwh.unwrap()
            - plain.annual_renewable_primary_kwh.unwrap();
        assert!((gain - solar_heat * F_PREN_RENHEAT).abs() < 1e-6);
        // A combi system also supplies the space-heating node (9.2.3.4).
        let mut combi = heater;
        combi.solar_use = crate::solar_thermal::SolarUse::Combi;
        system.solar = vec![combi];
        sample.hot_water = Some(system);
        let combi = assess_building_performance(&sample);
        assert_eq!(combi.status, "calculated_unverified", "{:?}", combi.issues);
        let march = &combi.space_heating.monthly[2];
        assert!(march.solar_gain_kwh > 0.0);
        assert!(march.generator_output_kwh < plain.space_heating.monthly[2].generator_output_kwh);
        assert!(
            (march.solar_gain_kwh
                - combi.hot_water.as_ref().unwrap().months[2].solar_space_heating_kwh)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn recoverable_hot_water_losses_feed_utility_internal_gains() {
        use crate::domestic_hot_water::{
            HotWaterEmission, HotWaterGenerator, HotWaterNeed, StorageLabel, StorageLoss,
            StorageVessel, UtilityArea,
        };
        use crate::monthly_demand::{LightingRecovery, UsageFunction};
        let mut sample = input();
        sample.calculation_scope = CalculationScope::Utility;
        sample.label_function = Some(LabelFunction::Office);
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        let demand = &mut sample.space_heating.demand;
        demand.usage_function = UsageFunction::Office;
        demand.dwelling_type = None;
        demand.setpoints.heating_c = 21.0;
        demand.internal_gains = InternalGains::Utility {
            lighting: UtilityLighting::Declared {
                annual_kwh: 0.0,
                recovery: LightingRecovery::Other,
            },
            hot_water_recoverable_kwh: Vec::new(),
            source_reference: "tables 7.2/7.3".into(),
        };
        let area = demand.usable_floor_area_m2;
        sample.hot_water = Some(HotWaterSystem {
            need: HotWaterNeed::Utility {
                areas: vec![UtilityArea {
                    function: LabelFunction::Office,
                    area_m2: area,
                }],
                source_reference: "plan".into(),
            },
            emission: HotWaterEmission::Utility {
                mean_length_m: 2.0,
                source_reference: "plan".into(),
            },
            shower_heat_recovery: None,
            circulation: None,
            storage: vec![StorageVessel {
                id: "boiler".into(),
                volume_l: 80.0,
                loss: StorageLoss::Label {
                    label: StorageLabel::C,
                },
                connection_factor: 1,
                in_heated_zone: true,
                unheated_ambient_c: None,
                source_reference: "label".into(),
            }],
            delivery_sets: None,
            boiling_water_tap: false,
            generator: HotWaterGenerator::ElectricBoiler,
            nominal_power_kw: None,
            exhaust_air: None,
            additional_generators: Vec::new(),
            series: None,
            solar: Vec::new(),
            collective: None,
            equipment_reference: "plate".into(),
        });
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        // 13.63 with table 13.9 label C: S_sto = 14,33 + 7,13·80^0,4 W.
        let storage = (14.33 + 7.13 * 80f64.powf(0.4)) * 744.0 / 1000.0;
        // 7.25–7.29 for an office: 5·0,30 + 4 W/m².
        let expected = 5.5 * area * 744.0 / 1000.0 + storage;
        let january = result.space_heating.demand.monthly[0].internal_gains_kwh;
        assert!((january - expected).abs() < 1e-9, "{january} vs {expected}");
    }

    #[test]
    fn bbl_check_uses_loss_area_ratio_and_available_indicators() {
        let mut sample = input();
        sample.bbl_function = Some(BblFunction::OtherResidential);
        sample.loss_area_m2 = Some(200.0);
        sample.loss_area_source_reference = Some("envelope inventory".into());
        let result = assess_building_performance(&sample);
        let check = result.bbl_check.as_ref().unwrap();
        assert!((check.loss_area_ratio - 2.0).abs() < 1e-12);
        // Synthetic mass class very heavy/light with open ceiling: D_m 180,
        // so paragraph 4 adds 5 kWh/m².
        assert!((check.limits.energy_need_max_kwh_per_m2 - 75.0).abs() < 1e-12);
        assert_eq!(check.energy_need_meets, None);
        assert_eq!(
            check.primary_fossil_meets,
            Some(result.primary_fossil_indicator_kwh_per_m2_year.unwrap() <= 30.0)
        );
        sample.loss_area_source_reference = None;
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "source_reference_required"));
    }

    #[test]
    fn tojuli_needs_component_transmission() {
        let result = assess_building_performance(&input());
        let tojuli = &result.tojuli[0];
        assert_eq!(tojuli.status, "invalid");
        assert!(result.tojuli_max_k.is_none());
        assert_eq!(tojuli.issues[0].code, "tojuli_components_required");
    }

    #[test]
    fn external_heat_uses_forfait_primary_factor() {
        use crate::space_heating_chain::ExternalHeatGenerator;
        let mut sample = input();
        let base = assess_building_performance(&sample);
        sample.space_heating.generator = Generator::ExternalHeat(ExternalHeatGenerator {
            supplier_reference: "contract".into(),
            quality_declaration_present: false,
            auxiliary: Some(crate::space_heating_chain::OtherGeneratorAuxiliary {
                electrically_connected_devices: 1,
                nominal_power_kw: None,
                source_reference: "delivery set".into(),
            }),
        });
        sample.space_heating.distribution_system = Some(none_on_site_system());
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let heat: f64 = result.space_heating.annual_district_heat_kwh.unwrap();
        let base_gas = base.space_heating.annual_natural_gas_kwh.unwrap();
        let base_aux = base.space_heating.annual_auxiliary_electricity_kwh.unwrap();
        let aux = result
            .space_heating
            .annual_auxiliary_electricity_kwh
            .unwrap();
        let expected = base.annual_primary_fossil_kwh.unwrap() - base_gas - base_aux * 1.45
            + heat * 0.9
            + aux * 1.45;
        assert!((result.annual_primary_fossil_kwh.unwrap() - expected).abs() < 1e-6);
        assert!(result.carriers.iter().any(|item| item.carrier == "dh"));
        assert_eq!(
            result.annual_renewable_primary_kwh,
            base.annual_renewable_primary_kwh
        );
    }

    #[test]
    fn a0_check_only_with_permit_date_and_fails_with_gas() {
        let mut sample = input();
        sample.bbl_function = Some(BblFunction::OtherResidential);
        sample.loss_area_m2 = Some(200.0);
        sample.loss_area_source_reference = Some("envelope".into());
        assert!(assess_building_performance(&sample).a0_check.is_none());
        sample.permit_application_after_2026_05_29 = true;
        let result = assess_building_performance(&sample);
        let a0 = result.a0_check.as_ref().unwrap();
        assert!(!a0.no_on_site_fossil_combustion);
        assert_eq!(a0.eligible, Some(false));
    }

    #[test]
    fn biomass_uses_bm_b_factors() {
        use crate::space_heating_chain::{BiomassAppliance, BiomassGenerator, BiomassLocation};
        let mut sample = input();
        let base = assess_building_performance(&sample);
        sample.space_heating.generator = Generator::Biomass(BiomassGenerator {
            appliance: BiomassAppliance::CentralBoiler,
            location: BiomassLocation::InsideThermalBoundary,
            annex_r_compliant_at_most_500_kw: true,
            annex_r_reference: "type test".into(),
            equipment_reference: "plate".into(),
            sole_heating_in_served_rooms: None,
            automatic_fuel_feed: true,
            auxiliary: Some(crate::space_heating_chain::OtherGeneratorAuxiliary {
                electrically_connected_devices: 1,
                nominal_power_kw: Some(15.0),
                source_reference: "plate".into(),
            }),
        });
        sample.space_heating.distribution_system = Some(none_on_site_system());
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let bm = result.space_heating.annual_biomass_kwh.unwrap();
        let heat: f64 = result
            .space_heating
            .monthly
            .iter()
            .map(|row| row.generator_output_kwh)
            .sum();
        assert!((bm - heat / 0.8).abs() < 1e-6);
        let renewable_delta = result.annual_renewable_primary_kwh.unwrap()
            - base.annual_renewable_primary_kwh.unwrap();
        assert!((renewable_delta - heat * 0.5).abs() < 1e-6);
        assert!(result.carriers.iter().any(|item| item.carrier == "bm"));
    }

    fn cooling_system(kind: crate::space_cooling::CoolingGeneratorKind) -> CoolingSystem {
        use crate::space_cooling::{
            CoolingBalancing, CoolingControl, CoolingEmission, CoolingEmitter, CoolingGenerator,
        };
        CoolingSystem {
            emission: CoolingEmission {
                emitter: CoolingEmitter::OtherOrUnknown,
                balancing: CoolingBalancing::NotApplicable,
                control: CoolingControl::CentralWithRoomControl,
                fan_coil_count: 0,
                source_reference: "design".into(),
            },
            distribution: None,
            generators: vec![CoolingGenerator {
                id: "cold".into(),
                generator: kind,
                capacity_kw: None,
                equipment_reference: "plate".into(),
            }],
            booster_heat_pump_extraction_kwh: Vec::new(),
            collective: None,
        }
    }

    #[test]
    fn cooling_chain_adds_electricity_and_ambient_cold() {
        let mut sample = input();
        let base = assess_building_performance(&sample);
        sample.cooling = Some(cooling_system(CoolingGeneratorKind::FreeCooling {
            source: FreeCoolingSource::ClosedGroundLoop,
            heat_pump_source: false,
            ground_above_zero_demonstrated: false,
        }));
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let need = result.space_heating.demand.annual_cooling_need_kwh.unwrap();
        assert!(need > 0.0);
        let cooling = cooling_assessment(&sample, &result.space_heating).unwrap();
        let cold: f64 = cooling
            .months
            .iter()
            .map(|row| row.generator_cold_kwh)
            .sum();
        assert!(cold > need);
        let renewable_delta = result.annual_renewable_primary_kwh.unwrap()
            - base.annual_renewable_primary_kwh.unwrap();
        assert!((renewable_delta - cold).abs() < 1e-6);
        let el = |r: &BuildingPerformanceAssessment| -> f64 {
            r.carriers
                .iter()
                .filter(|item| item.carrier == "el")
                .map(|item| item.used_kwh)
                .sum()
        };
        assert!((el(&result) - el(&base) - cold / 10.0).abs() < 1e-6);
        sample.declared_uses.push(DeclaredUse {
            id: "cool".into(),
            service: Service::SpaceCooling,
            carrier: Carrier::El,
            monthly_kwh: vec![1.0; 12],
            source_reference: "x".into(),
        });
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "cooling_double_count"));
    }

    #[test]
    fn external_cold_uses_forfait_factor_and_chp_absorption_is_rejected() {
        let mut sample = input();
        let base = assess_building_performance(&sample);
        sample.cooling = Some(cooling_system(CoolingGeneratorKind::ExternalCold));
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let cold: f64 = result
            .carriers
            .iter()
            .filter(|item| item.carrier == "dc")
            .map(|item| item.used_kwh)
            .sum();
        assert!(cold > 0.0);
        let delta =
            result.annual_primary_fossil_kwh.unwrap() - base.annual_primary_fossil_kwh.unwrap();
        assert!((delta - cold * 1.45 / 3.0).abs() < 1e-6);
        assert_eq!(
            result.annual_renewable_primary_kwh,
            base.annual_renewable_primary_kwh
        );
        sample.cooling = Some(cooling_system(CoolingGeneratorKind::AbsorptionChp {
            chp: crate::space_cooling::ChpClass {
                power_kw: 50.0,
                built_after_2006: true,
                hre_declared: false,
                low_temperature: false,
            },
            heat_rejection: None,
        }));
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "cooling_chp_unsupported"));
    }

    #[test]
    fn active_cooling_must_match_the_calculated_generator() {
        let mut sample = input();
        sample.cooling = Some(cooling_system(CoolingGeneratorKind::ExternalCold));
        sample.active_cooling = Some(ActiveCoolingEvidence {
            system: crate::tojuli::ActiveCoolingSystem::SplitUnitsInEveryHabitableRoom,
            capacity: crate::tojuli::CoolingCapacityEvidence::DynamicCoolingLoad {
                source_reference: "cooling load calculation".into(),
            },
            source_reference: "design".into(),
        });
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "active_cooling_system_inconsistent"));
        sample.active_cooling.as_mut().unwrap().system =
            crate::tojuli::ActiveCoolingSystem::ExternalColdWithCoolingEmitter;
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert_eq!(result.tojuli_max_k, Some(0.0));
    }
    #[test]
    fn review_fixes_bacs_on_district_heat_and_consistency_checks() {
        use crate::space_heating_chain::ExternalHeatGenerator;
        let mut sample = input();
        sample.space_heating.generator = Generator::ExternalHeat(ExternalHeatGenerator {
            supplier_reference: "contract".into(),
            quality_declaration_present: false,
            auxiliary: Some(crate::space_heating_chain::OtherGeneratorAuxiliary {
                electrically_connected_devices: 1,
                nominal_power_kw: None,
                source_reference: "delivery set".into(),
            }),
        });
        sample.space_heating.distribution_system = Some(none_on_site_system());
        sample.bacs_factor = 1.05;
        // §5.5.8: 1,05 is for utility buildings only.
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "bacs_factor_residential_invalid"));
        sample.calculation_scope = CalculationScope::Utility;
        let weighted = assess_building_performance(&sample);
        sample.bacs_factor = 1.0;
        let plain = assess_building_performance(&sample);
        let heat = plain.space_heating.annual_district_heat_kwh.unwrap();
        let aux = plain
            .space_heating
            .annual_auxiliary_electricity_kwh
            .unwrap();
        let delta =
            weighted.annual_primary_fossil_kwh.unwrap() - plain.annual_primary_fossil_kwh.unwrap();
        // f_BACS weights the heat and the auxiliary energy of space heating.
        assert!(
            (delta - 0.05 * (heat * 0.9 + aux * 1.45)).abs() < 1e-6,
            "{delta}"
        );

        let mut cooled = input();
        cooled.active_cooling = Some(ActiveCoolingEvidence {
            system: crate::tojuli::ActiveCoolingSystem::SplitUnitsInEveryHabitableRoom,
            capacity: crate::tojuli::CoolingCapacityEvidence::DynamicCoolingLoad {
                source_reference: "cooling load calculation".into(),
            },
            source_reference: "design".into(),
        });
        assert!(assess_building_performance(&cooled)
            .issues
            .iter()
            .any(|item| item.code == "active_cooling_without_cooling_system"));
    }

    #[test]
    fn utility_lighting_adds_electricity_and_is_rejected_for_dwellings() {
        use crate::lighting::{
            Daylight, InstalledPower, LightingZone, Occupancy, ParasiticPower, SwitchControl,
            UseArea, ZoneLighting,
        };
        let mut sample = input();
        let zone_id = sample.space_heating.demand.zone_id.clone();
        let lighting = ZoneLighting {
            zone_id,
            functions: vec![UseArea {
                function: LabelFunction::Office,
                area_m2: 100.0,
            }],
            lighting_zones: vec![LightingZone {
                id: "open-plan".into(),
                area_m2: 100.0,
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
            source_reference: "lighting plan".into(),
        };
        sample.lighting = vec![lighting];
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "residential_lighting_must_be_omitted"));
        sample.calculation_scope = CalculationScope::Utility;
        sample.label_function = Some(LabelFunction::Office);
        let mut base_input = sample.clone();
        base_input.lighting.clear();
        let base = assess_building_performance(&base_input);
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        // 16 W/m² · 100 m² · (2 200 + 300) h + 2,5 kWh/m² · 100 m².
        let expected = 1600.0 * 2500.0 / 1000.0 + 250.0;
        assert!((result.lighting[0].annual_kwh - expected).abs() < 1e-9);
        let el = |r: &BuildingPerformanceAssessment| -> f64 {
            r.carriers
                .iter()
                .filter(|item| item.carrier == "el")
                .map(|item| item.used_kwh)
                .sum()
        };
        assert!((el(&result) - el(&base) - expected).abs() < 1e-6);

        // 7.28: the chapter 14 gain enters the utility internal gains.
        let demand = &mut sample.space_heating.demand;
        demand.usage_function = crate::monthly_demand::UsageFunction::Office;
        demand.dwelling_type = None;
        demand.setpoints.heating_c = 21.0;
        demand.internal_gains = InternalGains::Utility {
            lighting: UtilityLighting::Chapter14,
            hot_water_recoverable_kwh: Vec::new(),
            source_reference: "tables 7.2/7.3".into(),
        };
        let area = demand.usable_floor_area_m2;
        let coupled = assess_building_performance(&sample);
        assert_eq!(
            coupled.status, "calculated_unverified",
            "{:?}",
            coupled.issues
        );
        let gain_w = coupled.lighting[0].internal_gain_w;
        assert!((gain_w - 0.3 * expected * 1000.0 / 8760.0).abs() < 1e-9);
        let january = coupled.space_heating.demand.monthly[0].internal_gains_kwh;
        assert!((january - (5.5 * area + gain_w) * 744.0 / 1000.0).abs() < 1e-9);
    }

    #[test]
    fn mixed_pv_routes_are_rejected() {
        let mut sample = input();
        sample.pv_systems.push(PvSystem {
            id: "roof-pv".into(),
            peak_power: crate::pv::PeakPower::Panels {
                panel_peak_power_w: 300.0,
                panel_count: 10,
            },
            azimuth_deg: 180.0,
            tilt_deg: 35.0,
            mounting: crate::pv::PvMounting::ModeratelyVentilated,
            obstruction_factors: vec![1.0],
            collective: None,
            source_reference: "datasheet".into(),
        });
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "pv_route_mixed"));
    }
    #[test]
    fn storage_correction_follows_5_14a() {
        let mut sample = input();
        let base = assess_building_performance(&sample);
        sample.battery_storage_present = true;
        sample.storage = Some(EnergyStorage {
            building_bound_electrical_kwh: 3.0,
            building_bound_thermal_kwh: 2.5,
            source_reference: "installation".into(),
        });
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let expected: f64 = result
            .electricity_balance
            .iter()
            .map(|row| row.produced_kwh.min(row.used_kwh) * 0.05)
            .sum();
        assert!(expected > 0.0);
        assert!((result.annual_storage_correction_kwh.unwrap() - expected).abs() < 1e-9);
        assert!(
            (base.annual_primary_fossil_kwh.unwrap()
                - result.annual_primary_fossil_kwh.unwrap()
                - expected)
                .abs()
                < 1e-9
        );
        // The correction is not part of the CO2 emission.
        assert_eq!(result.annual_co2_kg, base.annual_co2_kg);
        // Below 5 kWh in total: f_BAT;cor = 0, no error.
        sample.storage.as_mut().unwrap().building_bound_thermal_kwh = 1.0;
        let small = assess_building_performance(&sample);
        assert_eq!(small.annual_storage_correction_kwh, Some(0.0));
        assert_eq!(
            small.annual_primary_fossil_kwh,
            base.annual_primary_fossil_kwh
        );
    }

    #[test]
    fn co2_emission_uses_table_5_3() {
        let result = assess_building_performance(&input());
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let mut expected = 0.0;
        for row in &result.carriers {
            expected += row.delivered_kwh
                * match row.carrier {
                    "el" => 0.268,
                    "gas" => 0.218,
                    "oil" => 0.326,
                    other => panic!("unexpected carrier {other}"),
                };
        }
        let exported: f64 = result
            .electricity_balance
            .iter()
            .map(|row| row.exported_kwh)
            .sum();
        expected -= exported * 0.268;
        assert!((result.annual_co2_kg.unwrap() - expected).abs() < 1e-9);
        assert!((result.co2_kg_per_m2.unwrap() - expected / 100.0).abs() < 1e-9);
    }
}
