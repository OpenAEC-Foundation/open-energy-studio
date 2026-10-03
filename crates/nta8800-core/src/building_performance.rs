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
use crate::label_class::{
    indicative_label_class, indicative_label_class_mixed, renovation_standard, LabelFunction,
    LabelFunctionArea, LABEL_SOURCE,
};
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
    /// §5.3.1: use functions of an existing utility building with their
    /// areas, for area-weighted label class bounds and the table 5.7
    /// renovatiestandaard; replaces `labelFunction` when not empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub label_functions: Vec<LabelFunctionArea>,
    /// Construction year, for the Standaard voor woningisolatie (§5.3.2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub construction_year: Option<u32>,
    /// §5.5.7: building-bound appliances burning fossil fuel that were
    /// left out of the calculation by simplification (they still count).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fossil_appliances_outside_calculation: Option<bool>,
    /// Annex AB table AB.2/AB.3 footnote g: delivery temperature of external
    /// heat; unknown means ≥ 60 °C.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zeb_heat_delivery_temperature: Option<ZebHeatTemperature>,
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
    /// Space cooling calculated here (chapter 10), one system serving every
    /// calculation zone.
    #[serde(default)]
    pub cooling: Option<CoolingSystem>,
    /// §10.2: several cooling systems, each serving the listed calculation
    /// zones; exclusive with `cooling`. Zones that no system serves are not
    /// cooled (no cooling energy, no active cooling for TOjuli).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cooling_systems: Vec<ServedCoolingSystem>,
    /// Utility lighting per calculation zone (chapter 14); dwellings have
    /// `W_L = 0` and leave this empty.
    #[serde(default)]
    pub lighting: Vec<ZoneLighting>,
    /// Domestic hot water calculated here (chapter 13, one generator).
    #[serde(default)]
    pub hot_water: Option<HotWaterSystem>,
    /// §13.7 solar systems for space heating only (SHS) that are not part
    /// of the hot-water system.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub space_heating_solar: Vec<crate::solar_thermal::SolarWaterHeater>,
    /// Maatwerkadvies only (ISSO 82.2/75.2 table 2.2, Q_W;nd;spec): the
    /// actual annual hot-water need. Never part of a label calculation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hot_water_need_fit: Option<HotWaterNeedFit>,
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
    /// f_prac of 9.84 (dh), 13.152 (dw) and f_prpr of 10.78 (dc): 1 for
    /// the forfait factor or measured-only annex P values, 0,95 for
    /// calculated annex P values.
    pub practice_heat: f64,
    pub practice_hot_water: f64,
    pub practice_cold: f64,
}

const FORFAIT_FACTORS: CarrierFactors = CarrierFactors {
    district_heat: HEAT_FORFAIT,
    district_hot_water: HEAT_FORFAIT,
    district_cold: COLD_FORFAIT,
    heat_pump_source: None,
    practice_heat: 1.0,
    practice_hot_water: 1.0,
    practice_cold: 1.0,
};

/// 9.84 / 13.152 / 10.78: 1,0 for the forfait factor (no route) or an annex
/// P route on measured values only, 0,95 for calculated values. A quality
/// declaration counts as calculated unless it states measured values only.
fn external_practice_factor(route: Option<&AnnexPRoute>) -> f64 {
    match route {
        None | Some(AnnexPRoute::Measured(_)) => 1.0,
        Some(AnnexPRoute::Declared { measured_only, .. }) if *measured_only => 1.0,
        Some(_) => 0.95,
    }
}

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
        practice_heat: external_practice_factor(heating.as_ref().and(supply.heating.as_ref())),
        practice_hot_water: external_practice_factor(
            hot_water.as_ref().and(supply.hot_water.as_ref()),
        ),
        practice_cold: external_practice_factor(cooling.as_ref().and(supply.cooling.as_ref())),
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

/// Chapter 5 indicators reported on the label and in the record.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterFiveIndicators {
    /// 5.3a E_H;nd (actual ventilation, without recoverable losses),
    /// kWh/m² per year, rounded up to 0,01.
    pub heating_need_kwh_per_m2: f64,
    /// 5.3d E_C;nd, rounded up to 0,01.
    pub cooling_need_kwh_per_m2: f64,
    /// 5.3g E_H+C;nd.
    pub heating_and_cooling_need_kwh_per_m2: f64,
    /// §5.3.2 Standaard voor woningisolatie (dwellings with a known
    /// construction year and loss area), rounded to a whole number.
    pub standard_insulation_kwh_per_m2: Option<f64>,
    pub meets_standard_insulation: Option<bool>,
    /// §5.3.1.3 EwePrenTot, rounded down to 0,01 (EMGverklaring when a
    /// quality declaration is used).
    pub renewable_indicator_kwh_per_m2: f64,
    /// §5.3.1.3 EwePrenTot;EMGforf, reported next to it with a quality
    /// declaration.
    pub renewable_indicator_forfait_kwh_per_m2: Option<f64>,
    /// 5.3h EweFinal, rounded up to 0,01.
    pub final_energy_kwh_per_m2: f64,
    /// 5.3i EweFinal;EED, rounded up to 0,01.
    pub final_energy_eed_kwh_per_m2: f64,
    /// 5.17a/5.17b delivered electricity.
    pub delivered_electricity_kwh: f64,
    pub delivered_electricity_kwh_per_m2: f64,
    /// 5.18a/5.18b external heat and cold, GJ.
    pub delivered_external_gj: f64,
    pub delivered_external_gj_per_m2: f64,
    /// 5.19a/5.19b other carriers in natural gas equivalents, m³aeq.
    pub delivered_other_m3_aeq: f64,
    pub delivered_other_m3_aeq_per_m2: f64,
    /// 5.39a–h.
    pub renewable_by_carrier: RenewableByCarrier,
    /// §5.5.7 locally carbon-emission free: no gas or oil used by
    /// building-bound systems, including appliances outside the
    /// calculation. `None` when that has not been stated.
    pub locally_carbon_free: Option<bool>,
    /// §5.3.1.2 table 5.7 for utility buildings, rounded to 0,01.
    pub renovation_standard_kwh_per_m2: Option<f64>,
    pub meets_renovation_standard: Option<bool>,
}

/// Σ N_woon over the zones, `None` without residential internal gains.
fn dwelling_count(input: &BuildingPerformanceInput) -> Option<u32> {
    let mut total = None;
    for zone in std::iter::once(&input.space_heating.demand).chain(
        input
            .space_heating
            .additional_zones
            .iter()
            .map(|zone| &zone.demand),
    ) {
        if let crate::monthly_demand::InternalGains::Residential { dwelling_count, .. } =
            &zone.internal_gains
        {
            total = Some(total.unwrap_or(0) + *dwelling_count);
        }
    }
    total
}

/// §5.3.2: Standaard voor woningisolatie, kWh/m² per year.
pub fn standard_insulation(
    apartment_building: bool,
    construction_year: u32,
    loss_area_m2: f64,
    usable_area_m2: f64,
) -> Option<f64> {
    use rust_decimal::{Decimal, RoundingStrategy};
    if !(loss_area_m2.is_finite() && usable_area_m2.is_finite())
        || loss_area_m2 <= 0.0
        || usable_area_m2 <= 0.0
    {
        return None;
    }
    let (base, slope) = match (apartment_building, construction_year <= 1945) {
        (false, true) => (60, 105),
        (false, false) => (43, 40),
        (true, true) => (95, 70),
        (true, false) => (45, 45),
    };
    // Decimal arithmetic so that an exact half rounds up.
    let ratio = crate::final_energy_draft::decimal(loss_area_m2)?
        .checked_div(crate::final_energy_draft::decimal(usable_area_m2)?)?;
    let value = if ratio < Decimal::ONE {
        Decimal::from(base)
    } else {
        Decimal::from(base) + Decimal::from(slope) * (ratio - Decimal::ONE)
    };
    // "Rekenkundig afronden op een geheel getal".
    use rust_decimal::prelude::ToPrimitive;
    value
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_f64()
}

/// Annex AB footnote g: ϑ_aflever;warmte of external heat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ZebHeatTemperature {
    AtLeast60,
    From40To60,
    From20To40,
}

impl ZebHeatTemperature {
    /// Table AB.2 f_P,ZEB;weeg;dh/dw.
    fn primary_weight(self) -> f64 {
        match self {
            Self::AtLeast60 => 0.45,
            Self::From40To60 => 0.29,
            Self::From20To40 => 0.14,
        }
    }
    /// Table AB.3 K_CO2;ZEB;del;dh/dw, kg/kWh.
    fn co2(self) -> f64 {
        match self {
            Self::AtLeast60 => 0.09,
            Self::From40To60 => 0.072,
            Self::From20To40 => 0.055,
        }
    }
}

/// Table AB.1 f_du;el,ren: dwellings, education, other utility.
const ZEB_DIRECT_USE: [[f64; 12]; 3] = [
    [
        0.75, 0.75, 0.5, 0.25, 0.25, 0.15, 0.15, 0.15, 0.25, 0.50, 0.75, 0.75,
    ],
    [
        0.55, 0.55, 0.35, 0.20, 0.20, 0.15, 0.01, 0.01, 0.20, 0.35, 0.55, 0.55,
    ],
    [
        0.55, 0.55, 0.35, 0.20, 0.20, 0.15, 0.15, 0.15, 0.20, 0.35, 0.55, 0.55,
    ],
];

/// Table AB.1 f_du;el,ren per month; for several use functions weighted by
/// usable area (text under table AB.1). The Bbl function list decides when
/// given, otherwise the zones' usage functions.
fn zeb_direct_use_fraction(input: &BuildingPerformanceInput) -> [f64; 12] {
    use crate::bbl_requirements::BblFunction as B;
    // (dwellings, education, other) areas.
    let mut areas = [0.0_f64; 3];
    if !input.bbl_functions.is_empty() {
        for part in &input.bbl_functions {
            let column = match part.function {
                B::ResidentialBuilding
                | B::OtherResidential
                | B::Caravan
                | B::FloatingBuildingAfter2018Berth
                | B::FloatingBuildingOtherBerth => 0,
                B::Education => 1,
                _ => 2,
            };
            areas[column] += part.area_m2.max(0.0);
        }
    } else if matches!(input.calculation_scope, CalculationScope::Residential) {
        areas[0] = 1.0;
    } else {
        for zone in std::iter::once(&input.space_heating.demand).chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        ) {
            let column = match zone.usage_function {
                crate::monthly_demand::UsageFunction::Residential => 0,
                crate::monthly_demand::UsageFunction::Education => 1,
                _ => 2,
            };
            areas[column] += zone.usable_floor_area_m2.max(0.0);
        }
        if areas.iter().sum::<f64>() <= 0.0 {
            let column = if input.label_function == Some(LabelFunction::Education) {
                1
            } else {
                2
            };
            areas[column] = 1.0;
        }
    }
    let total: f64 = areas.iter().sum();
    std::array::from_fn(|month| {
        (0..3)
            .map(|column| areas[column] / total * ZEB_DIRECT_USE[column][month])
            .sum()
    })
}

/// Annex AB monthly terms: (E_P,ZEB;Tot, m_CO2;ZEB) in kWh and kg.
#[allow(clippy::too_many_arguments)]
fn zeb_month(
    f_du: f64,
    battery: bool,
    used_el: f64,
    renewable_el: f64,
    chp_el: f64,
    fuels: f64,
    oil: f64,
    biomass: f64,
    heat: f64,
    heat_temperature: ZebHeatTemperature,
    cold: f64,
    source_heat: f64,
) -> (f64, f64) {
    // AB.65, AB.67, AB.68.
    let direct_ren = (f_du * renewable_el)
        .max(0.3 * used_el)
        .min(used_el)
        .min(renewable_el);
    // AB.70–AB.73 with η_BAT 0,85.
    let battery_in = if battery {
        (0.3 * renewable_el)
            .min(used_el - direct_ren)
            .min(renewable_el - direct_ren)
            .max(0.0)
    } else {
        0.0
    };
    let battery_out = 0.85 * battery_in;
    // AB.66, AB.69.
    let direct_nren = (0.5 * chp_el).min((used_el - direct_ren - battery_out).max(0.0));
    // AB.15, AB.61, AB.62.
    let delivered_el = (used_el - direct_ren - direct_nren - battery_out).max(0.0);
    let exported_ren = (renewable_el - direct_ren - battery_in).max(0.0);
    // AB.10–AB.14 with table AB.2 (exported CHP electricity weighs 0).
    let primary = delivered_el * 1.35
        + fuels
        + oil
        + biomass
        + heat * heat_temperature.primary_weight()
        + (cold + source_heat) * 0.04
        - exported_ren;
    // AB.3 with table AB.3.
    let co2 = delivered_el * 0.268
        + fuels * 0.218
        + oil * 0.326
        + biomass * 0.104
        + heat * heat_temperature.co2()
        + cold * 0.027
        + source_heat * 0.012
        - exported_ren * 0.268 / 1.35;
    (primary, co2)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalEnergyCarrier {
    pub carrier: &'static str,
    pub annual_kwh: f64,
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
    /// 5.57/5.58: final energy `E_Final` per carrier (Σ E_EPus;ci), kWh.
    pub final_energy_by_carrier: Vec<FinalEnergyCarrier>,
    /// Annex AB (informative) E_P,ZEB;Tot;an, kWh.
    pub annual_zeb_primary_total_kwh: Option<f64>,
    /// AB.1 EweP,ZEB;Tot, kWh/m², rounded up to 0,01.
    pub zeb_primary_total_indicator_kwh_per_m2: Option<f64>,
    /// AB.3 m_CO2;ZEB, kg CO2eq per year.
    pub annual_zeb_co2_kg: Option<f64>,
    /// 5.57 `E_Final`, kWh; own electricity production is not netted.
    pub annual_final_energy_kwh: Option<f64>,
    /// 5.60 `E_Final;EED`: `E_Final` plus the solar thermal yields
    /// Q_H;ren;sol,prac and Q_W;ren;sol,prac, kWh.
    pub annual_final_energy_eed_kwh: Option<f64>,
    /// `m_CO2;spec = m_CO2 / A_g`, kg CO2eq/m².
    pub co2_kg_per_m2: Option<f64>,
    /// Chapter 5 label and record indicators (5.3a–i, 5.17–5.19, 5.39a–h,
    /// 5.5.7, table 5.7).
    pub chapter5: Option<ChapterFiveIndicators>,
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
    /// 13.184: the part of the space-heating carriers used for hot water
    /// made by the heating system (13.8.4.9.3); reported only, the totals
    /// already contain it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hot_water_from_heating: Vec<HotWaterFromHeatingMonth>,
    /// Standalone space-heating solar systems (`spaceHeatingSolar`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standalone_solar: Option<crate::domestic_hot_water::StandaloneSolarHeating>,
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

/// A cooling system with the calculation zones it serves (§10.2).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServedCoolingSystem {
    /// `zoneId` of the calculation zones (monthly demand inputs).
    pub zone_ids: Vec<String>,
    pub system: CoolingSystem,
}

impl BuildingPerformanceInput {
    /// The cooling systems with the zone ids they serve (`None`: every zone).
    pub fn cooling_list(&self) -> Vec<(&CoolingSystem, Option<&[String]>)> {
        let mut list: Vec<(&CoolingSystem, Option<&[String]>)> =
            self.cooling.iter().map(|system| (system, None)).collect();
        list.extend(
            self.cooling_systems
                .iter()
                .map(|served| (&served.system, Some(served.zone_ids.as_slice()))),
        );
        list
    }

    /// Whether any cooling system is calculated.
    pub fn has_cooling(&self) -> bool {
        self.cooling.is_some() || !self.cooling_systems.is_empty()
    }

    /// Whether a calculated cooling system serves this zone.
    pub fn zone_cooled(&self, zone_id: &str) -> bool {
        self.cooling.is_some()
            || self
                .cooling_systems
                .iter()
                .any(|served| served.zone_ids.iter().any(|id| id == zone_id))
    }

    fn zone_ids(&self) -> Vec<&str> {
        std::iter::once(&self.space_heating.demand)
            .chain(
                self.space_heating
                    .additional_zones
                    .iter()
                    .map(|zone| &zone.demand),
            )
            .map(|demand| demand.zone_id.as_str())
            .collect()
    }
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
/// Maatwerkadvies: actual annual net hot-water need `Q_W;nd`, kWh.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HotWaterNeedFit {
    pub annual_need_kwh: f64,
    pub source_reference: String,
}

/// Rescales a chapter 13 result to a fitted annual net need. The
/// need-dependent terms (net need, recovered heat, emission input and
/// delivery-set conversion) scale with `Q_W;nd`; circulation and storage
/// losses stay; generator output, carrier input, auxiliary and ambient
/// energy follow the generator output (linearised maatwerkadvies route,
/// not chapter 13 itself).
pub fn fit_hot_water_need(result: &mut HotWaterAssessment, annual_need_kwh: f64) {
    let base: f64 = result.months.iter().map(|month| month.net_need_kwh).sum();
    if base <= 0.0 || !annual_need_kwh.is_finite() || annual_need_kwh < 0.0 {
        return;
    }
    let factor = annual_need_kwh / base;
    let mut ratios = Vec::with_capacity(result.months.len());
    for month in result.months.iter_mut() {
        let emission = month.emission_input_kwh * (factor - 1.0);
        let conversion = month.conversion_loss_kwh * (factor - 1.0);
        let distribution = if month.distribution_efficiency > 0.0 {
            emission / month.distribution_efficiency
        } else {
            emission
        };
        let old_output = month.generator_output_kwh;
        let new_output = (old_output + distribution + conversion).max(0.0);
        let ratio = if old_output > 0.0 {
            new_output / old_output
        } else {
            factor
        };
        month.net_need_kwh *= factor;
        month.recovered_kwh *= factor;
        month.emission_input_kwh *= factor;
        month.conversion_loss_kwh *= factor;
        month.generator_output_kwh = new_output;
        month.carrier_input_kwh *= ratio;
        month.auxiliary_electricity_kwh *= ratio;
        month.ambient_heat_kwh *= ratio;
        // 13.3: the carriers booked per generator follow the output.
        month.electricity_kwh *= ratio;
        month.natural_gas_kwh *= ratio;
        month.oil_kwh *= ratio;
        month.district_heat_kwh *= ratio;
        ratios.push(ratio);
    }
    for generator in result.generators.iter_mut() {
        for (value, ratio) in generator.monthly_output_kwh.iter_mut().zip(&ratios) {
            *value *= ratio;
        }
    }
    result.annual_net_need_kwh = annual_need_kwh;
    result.annual_generator_output_kwh = result
        .months
        .iter()
        .map(|month| month.generator_output_kwh)
        .sum();
}

/// True when a zone carries a maatwerkadvies usage fit (ISSO 82.2/75.2).
pub fn usage_fit_applied(input: &BuildingPerformanceInput) -> bool {
    input.hot_water_need_fit.is_some()
        || input.space_heating.demand.usage_fit.is_some()
        || input
            .space_heating
            .additional_zones
            .iter()
            .any(|zone| zone.demand.usage_fit.is_some())
}

fn hot_water_context(input: &BuildingPerformanceInput) -> HotWaterContext {
    let zones = std::iter::once(&input.space_heating.demand).chain(
        input
            .space_heating
            .additional_zones
            .iter()
            .map(|zone| &zone.demand),
    );
    let (weighted, standard, area) =
        zones.fold((0.0, 0.0, 0.0), |(weighted, standard, area), zone| {
            (
                weighted + zone.setpoints.heating_c * zone.usable_floor_area_m2,
                standard + zone.usage_function.heating_setpoint_c() * zone.usable_floor_area_m2,
                area + zone.usable_floor_area_m2,
            )
        });
    HotWaterContext {
        residential: matches!(input.calculation_scope, CalculationScope::Residential),
        usable_floor_area_m2: input.total_usable_floor_area_m2,
        heated_ambient_c: if area > 0.0 { weighted / area } else { 20.0 },
        space_heating: None,
        // 13.69a/13.137a: table 7.13.
        standard_setpoint_c: (area > 0.0).then(|| standard / area),
        levelled_setpoint_c: None,
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

/// Chapter 10: one system for every zone, or several systems each for its
/// own zones (§10.2), combined into the building total.
fn cooling_assessment(
    input: &BuildingPerformanceInput,
    heating: &SpaceHeatingChainAssessment,
) -> Option<CoolingAssessment> {
    if !input.has_cooling() {
        return None;
    }
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
    let extraction = if input.space_heating.generator.heat_pump().is_some()
        || input.space_heating.generator.annex_q().is_some()
    {
        extraction
    } else {
        [0.0; 12]
    };
    let residential = matches!(input.calculation_scope, CalculationScope::Residential);
    if let Some(system) = &input.cooling {
        return Some(assess_cooling(
            system,
            CoolingContext {
                zones: &zones,
                residential,
                heat_pump_source_extraction_kwh: extraction,
            },
        ));
    }
    // §10.2: each system for its own zones. The 10.84 source extraction of
    // the space-heating heat pump goes to the first system only, so it is
    // not counted twice.
    let ids = input.zone_ids();
    let parts: Vec<crate::space_cooling::ServedCoolingResult> = input
        .cooling_systems
        .iter()
        .enumerate()
        .map(|(system_index, served)| {
            let zone_indexes: Vec<usize> = ids
                .iter()
                .enumerate()
                .filter(|(_, id)| served.zone_ids.iter().any(|zone| zone == *id))
                .map(|(index, _)| index)
                .collect();
            let own: Vec<CoolingZoneNeed> =
                zone_indexes.iter().map(|index| zones[*index]).collect();
            let assessment = assess_cooling(
                &served.system,
                CoolingContext {
                    zones: &own,
                    residential,
                    heat_pump_source_extraction_kwh: if system_index == 0 {
                        extraction
                    } else {
                        [0.0; 12]
                    },
                },
            );
            crate::space_cooling::ServedCoolingResult {
                zone_indexes,
                assessment,
            }
        })
        .collect();
    Some(crate::space_cooling::combine_cooling(parts, zones.len()))
}

fn validate(input: &BuildingPerformanceInput, issues: &mut Vec<PerformanceIssue>) {
    // 13.156a: E_W;gen;in;PFHRD needs the heating gas of the same combi.
    if let Some(system) = &input.hot_water {
        let pfhrd = std::iter::once(&system.generator)
            .chain(system.additional_generators.iter().map(|unit| &unit.generator))
            .any(|generator| {
                matches!(generator, crate::domestic_hot_water::HotWaterGenerator::MeasuredTwoProfiles(test)
                    if test.pfhrd.is_some())
            });
        if pfhrd && !heating_by_gas_boiler(&input.space_heating.generator) {
            issues.push(issue(
                "pfhrd_requires_gas_boiler_heating",
                "hotWater.generator.pfhrd",
            ));
        }
    }
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
    if input.cooling.is_some() && !input.cooling_systems.is_empty() {
        issues.push(issue(
            "cooling_and_cooling_systems_exclusive",
            "coolingSystems",
        ));
    }
    let zone_ids = input.zone_ids();
    let mut served_zones: Vec<&str> = Vec::new();
    for (index, served) in input.cooling_systems.iter().enumerate() {
        let path = format!("coolingSystems[{index}]");
        if served.zone_ids.is_empty() {
            issues.push(issue("cooling_zones_required", format!("{path}.zoneIds")));
        }
        for (zone_index, zone) in served.zone_ids.iter().enumerate() {
            let zone_path = format!("{path}.zoneIds[{zone_index}]");
            if !zone_ids.contains(&zone.as_str()) {
                issues.push(issue("cooling_zone_unknown", zone_path));
            } else if served_zones.contains(&zone.as_str()) {
                // §10.2: a zone belongs to one cooling system.
                issues.push(issue("cooling_zone_served_twice", zone_path));
            } else {
                served_zones.push(zone.as_str());
            }
        }
    }
    let systems = input.cooling_list();
    for (index, (system, served)) in systems.iter().enumerate() {
        let path = if served.is_some() {
            let offset = usize::from(input.cooling.is_some());
            format!("coolingSystems[{}].system", index - offset)
        } else {
            "cooling".to_string()
        };
        issues.extend(
            validate_cooling(system, &path)
                .into_iter()
                .map(|item| issue(item.code, item.path)),
        );
    }
    if !systems.is_empty() {
        if input.declared_uses.iter().any(|item| {
            matches!(
                item.service,
                Service::SpaceCooling | Service::SpaceCoolingAuxiliary
            )
        }) {
            issues.push(issue("cooling_double_count", "cooling"));
        }
        if let Some(evidence) = &input.active_cooling {
            if !systems
                .iter()
                .any(|(system, _)| active_cooling_matches(evidence.system, system))
            {
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
        && !input.has_cooling()
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
    issues.extend(
        crate::domestic_hot_water::validate_standalone_solar(
            &input.space_heating_solar,
            "spaceHeatingSolar",
        )
        .into_iter()
        .map(|item| issue(item.code, item.path)),
    );
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
        // 16.5/16.17: wind energy cannot yet be determined, E_el;wind = 0.
        if item.kind == ProductionKind::Wind && item.monthly_kwh.iter().any(|value| *value > 0.0) {
            issues.push(issue(
                "wind_production_not_determinable",
                format!("{path}.monthlyKwh"),
            ));
        }
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
            // Gas-driven heat pumps (tables 9.27/9.29) also need 5.31 evidence.
            (None, Some(evidence)) if input.space_heating.generator.has_gas_heat_pump() => {
                if evidence.source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        "heatPumpRenewable.sourceReference",
                    ));
                }
            }
            (None, None) if input.space_heating.generator.has_gas_heat_pump() => {
                issues.push(issue(
                    "heat_pump_renewable_evidence_required",
                    "heatPumpRenewable",
                ));
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

/// One month of 13.184.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotWaterFromHeatingMonth {
    pub month: u8,
    /// `E_W;gen;in;conv;hj / (E_W;gen;in;conv;hj + Q_H;nod;in)`; the node
    /// input already contains the hot-water load, so the note to 13.184
    /// drops it from the denominator.
    pub share: f64,
    pub natural_gas_kwh: f64,
    pub oil_kwh: f64,
    pub biomass_kwh: f64,
    pub district_heat_kwh: f64,
    pub electricity_kwh: f64,
}

/// 13.184 for each month with a hot-water load on the heating system.
fn hot_water_from_heating(heating: &SpaceHeatingChainAssessment) -> Vec<HotWaterFromHeatingMonth> {
    if heating
        .monthly
        .iter()
        .all(|row| row.hot_water_load_kwh <= 0.0)
    {
        return Vec::new();
    }
    heating
        .monthly
        .iter()
        .map(|row| {
            let share = if row.generator_output_kwh > 0.0 {
                (row.hot_water_load_kwh / row.generator_output_kwh).min(1.0)
            } else {
                0.0
            };
            HotWaterFromHeatingMonth {
                month: row.month,
                share,
                natural_gas_kwh: share * row.natural_gas_kwh,
                oil_kwh: share * row.oil_kwh,
                biomass_kwh: share * row.biomass_kwh,
                district_heat_kwh: share * row.district_heat_kwh,
                electricity_kwh: share * row.generator_electricity_kwh,
            }
        })
        .collect()
}

/// Inputs of the final hot-water run from the space-heating chain:
/// the gas of the space-heating generator for PFHRD (13.156a, taken as the
/// combi appliance) and the chapter 11 flows and extract temperatures for
/// mixed-air heat pumps (13.153h/i; `ϑ_ETA;dis;out` as the zone's levelled
/// heating setpoint, ducts inside the thermal zone).
/// 13.156a: the heating gas belongs to the combi only when a gas boiler
/// heats the building (the PFHRD-tested combi appliance).
fn heating_by_gas_boiler(generator: &Generator) -> bool {
    match generator {
        Generator::GasBoiler(_) => true,
        Generator::ProductBoiler(boiler) => {
            boiler.boiler.fuel == crate::annex_m::BoilerFuel::NaturalGas
        }
        _ => false,
    }
}

fn hot_water_extras(
    input: &BuildingPerformanceInput,
    heating: &SpaceHeatingChainAssessment,
) -> crate::domestic_hot_water::HotWaterExtras {
    let zones: Vec<&crate::monthly_demand::MonthlyDemandAssessment> =
        std::iter::once(&heating.demand)
            .chain(&heating.additional_zone_demands)
            .collect();
    let mut flow = [0.0; 12];
    let mut weighted = [0.0; 12];
    let mut any = false;
    for zone in &zones {
        let Some(ventilation) = &zone.ventilation else {
            continue;
        };
        if ventilation.months.len() != 12 || zone.monthly.len() != 12 {
            continue;
        }
        any = true;
        for index in 0..12 {
            let q = ventilation.months[index]
                .heating
                .required_outdoor_air_m3_per_h;
            flow[index] += q;
            weighted[index] += q * zone.monthly[index].heating.setpoint_c;
        }
    }
    // §13.8.4.8 (p. 650): a micro-CHP that heats and makes hot water.
    let combi_chp =
        input
            .space_heating
            .generator
            .micro_chp()
            .filter(|_| heating.monthly.len() == 12)
            .filter(|_| {
                input.hot_water.as_ref().is_some_and(|system| {
                    std::iter::once(&system.generator)
                    .chain(system.additional_generators.iter().map(|unit| &unit.generator))
                    .any(|generator| {
                        matches!(generator, crate::domestic_hot_water::HotWaterGenerator::Chp(chp)
                            if chp.also_space_heating && chp.method1.is_some())
                    })
                })
            })
            .map(|product| crate::domestic_hot_water::CombiChpHeating {
                product: product.clone(),
                thermal_output_kwh: std::array::from_fn(|index| {
                    heating.monthly[index].chp_thermal_output_kwh
                }),
                operating_hours: std::array::from_fn(|index| {
                    heating.monthly[index].chp_operating_hours
                }),
            });
    crate::domestic_hot_water::HotWaterExtras {
        combi_chp,
        combi_space_heating_gas_kwh: heating
            .annual_natural_gas_kwh
            .filter(|gas| *gas > 0.0)
            .filter(|_| heating_by_gas_boiler(&input.space_heating.generator)),
        mixed_air: any.then(|| crate::domestic_hot_water::MixedAirVentilation {
            required_outdoor_air_m3_per_h: flow,
            extract_air_c: std::array::from_fn(|index| {
                if flow[index] > 0.0 {
                    weighted[index] / flow[index]
                } else {
                    20.0
                }
            }),
        }),
    }
}

/// Replaces the chain's micro-CHP input, electricity and generator auxiliary
/// energy by the heating share of the joint combi evaluation. The heating
/// need and its 9.7 recoverable losses keep the chain's own values.
fn apply_combi_chp_heating(
    heating: &mut SpaceHeatingChainAssessment,
    shares: &[crate::domestic_hot_water::CombiChpHeatingMonth],
    fuel: crate::micro_chp::MicroChpFuel,
) {
    for (row, share) in heating.monthly.iter_mut().zip(shares) {
        if row.chp_thermal_output_kwh <= 0.0 && share.input_kwh <= 0.0 {
            continue;
        }
        let delta = share.input_kwh - row.chp_input_kwh;
        match fuel {
            crate::micro_chp::MicroChpFuel::NaturalGas => row.natural_gas_kwh += delta,
            crate::micro_chp::MicroChpFuel::Oil => row.oil_kwh += delta,
        }
        row.chp_input_kwh = share.input_kwh;
        row.chp_electricity_kwh = share.electricity_kwh;
        if let (Some(total), Some(auxiliary)) = (row.auxiliary_electricity_kwh, share.auxiliary_kwh)
        {
            row.auxiliary_electricity_kwh =
                Some(total - row.chp_generator_auxiliary_kwh + auxiliary);
            row.chp_generator_auxiliary_kwh = auxiliary;
        }
    }
    let sum = |field: fn(&crate::space_heating_chain::ChainMonth) -> f64| {
        heating.monthly.iter().map(field).sum::<f64>()
    };
    if heating.annual_natural_gas_kwh.is_some() {
        heating.annual_natural_gas_kwh = Some(sum(|row| row.natural_gas_kwh));
    }
    if heating.annual_auxiliary_electricity_kwh.is_some() {
        heating.annual_auxiliary_electricity_kwh =
            Some(sum(|row| row.auxiliary_electricity_kwh.unwrap_or(0.0)));
    }
}

/// Area-weighted `ϑ_int;set;H;zi,mi` (7.76) of the chain's zones.
fn levelled_setpoint(
    input: &BuildingPerformanceInput,
    heating: &SpaceHeatingChainAssessment,
) -> Option<[f64; 12]> {
    let areas = std::iter::once(&input.space_heating.demand)
        .chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
        .map(|zone| zone.usable_floor_area_m2);
    let zones: Vec<(&crate::monthly_demand::MonthlyDemandAssessment, f64)> =
        std::iter::once(&heating.demand)
            .chain(&heating.additional_zone_demands)
            .zip(areas)
            .filter(|(zone, _)| zone.monthly.len() == 12)
            .collect();
    let area: f64 = zones.iter().map(|(_, area)| area).sum();
    (area > 0.0).then(|| {
        std::array::from_fn(|index| {
            zones
                .iter()
                .map(|(zone, area)| zone.monthly[index].heating.setpoint_c * area)
                .sum::<f64>()
                / area
        })
    })
}

/// 13.148/13.149 for 11.2.2.1.2: the flow `q_ve;hp;W;zi;mi` and time
/// fraction `f_W;t;hp-on;mi` of an exhaust-air heat pump for hot water fill
/// empty overventilation fields of the zones with a chapter 11 input. A
/// hot-water-only heat pump (`f_combi = 0`) also creates the
/// overventilation when the zone has none. The hot water does not depend
/// on the ventilation, so it is assessed first and the chain follows.
fn apply_exhaust_air_hot_water(
    result: &HotWaterAssessment,
    total_area: f64,
    chain: &mut SpaceHeatingChainInput,
) {
    let Some(exhaust) = &result.exhaust_air else {
        return;
    };
    // Q.5.3 derives f_H;t;hp-on for the primary zone of an annex Q chain.
    let annex_q = matches!(
        chain.generator,
        crate::space_heating_chain::Generator::HeatPumpAnnexQ(_)
    );
    let apply = |demand: &mut MonthlyDemandInput, primary: bool| {
        let area = demand.usable_floor_area_m2;
        let Some(ventilation) = demand.ventilation.as_mut() else {
            return;
        };
        // 11.22: q_V;ODA;req without overventilation.
        let mut plain = ventilation.clone();
        plain.overventilation = None;
        let Ok(base) = crate::ventilation::calculate_ventilation(&plain) else {
            return;
        };
        let required: Vec<f64> = base
            .months
            .iter()
            .map(|month| month.heating.required_outdoor_air_m3_per_h)
            .collect();
        if required.len() != 12 {
            return;
        }
        let share = area / total_area;
        // 13.148a with a declared flow, else 13.148 for a hot-water-only
        // heat pump; a combi heat pump needs the declaration.
        let flow: Option<Vec<f64>> = match exhaust.declared_flow_m3_per_h {
            Some(declared) => Some(
                required
                    .iter()
                    .map(|q| (declared * share).max(*q))
                    .collect(),
            ),
            None if exhaust.hot_water_only => Some(
                required
                    .iter()
                    .map(|q| ((44.0 * share).max(area * 0.44) * 3.6).max(*q))
                    .collect(),
            ),
            None => None,
        };
        match ventilation.overventilation.as_mut() {
            Some(over) => {
                if over.hot_water_time_fraction.is_empty() {
                    over.hot_water_time_fraction = exhaust.time_fraction.clone();
                }
                if over.hot_water_flow_m3_per_h.is_empty() {
                    if let Some(flow) = flow {
                        over.hot_water_flow_m3_per_h = flow;
                    }
                }
            }
            None => {
                // A combi heat pump also carries f_W and q_W into chapter 11
                // (11.23); without a declared flow its hot-water flow is the
                // required flow itself. On an annex Q chain the heating
                // fraction stays empty, so Q.5.3 derives it.
                let flow = flow.unwrap_or_else(|| required.clone());
                ventilation.overventilation = Some(crate::ventilation::Overventilation {
                    heating_time_fraction: if !exhaust.hot_water_only && annex_q && primary {
                        Vec::new()
                    } else {
                        vec![0.0; 12]
                    },
                    hot_water_time_fraction: exhaust.time_fraction.clone(),
                    heating_flow_m3_per_h: None,
                    heating_area_share: 1.0,
                    hot_water_flow_m3_per_h: flow,
                    source_reference: "NTA 8800 13.148/13.149".into(),
                });
            }
        }
    };
    apply(&mut chain.demand, true);
    for zone in &mut chain.additional_zones {
        apply(&mut zone.demand, false);
    }
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
    // 13.185: hot water made with heat from the space-heating system.
    if result
        .months
        .iter()
        .any(|month| month.heating_system_load_kwh > 0.0)
    {
        chain.hot_water_load_kwh = result
            .months
            .iter()
            .map(|month| month.heating_system_load_kwh)
            .collect();
    }
    apply_exhaust_air_hot_water(&result, total_area, &mut chain);
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
            .any(|heater| heater.solar_use != crate::solar_thermal::SolarUse::WaterHeating)
    });
    let standalone_needed = !input.space_heating_solar.is_empty();
    // 13.26/13.58: circulation and vessel losses (and with them the 7.29
    // gains) use the levelled ϑ_int;set;H;zi,mi of 7.9.4, which does not
    // depend on the internal gains; the first run supplies it.
    let levelled = levelled_setpoint(input, &heating);
    let levelling_matters = input.hot_water.is_some()
        && levelled.is_some()
        && hot_water_context.levelled_setpoint_c != levelled;
    hot_water_context.levelled_setpoint_c = levelled;
    let mut standalone_solar = None;
    let mut standalone_solar_issue = None;
    if combi || standalone_needed || levelling_matters {
        let space_heating = solar_space_heating(input, &heating);
        if combi {
            hot_water_context.space_heating = space_heating;
        }
        chain_input = with_hot_water_gains(input, hot_water_context, with_lighting_gains(input));
        if standalone_needed
            && crate::domestic_hot_water::validate_standalone_solar(
                &input.space_heating_solar,
                "spaceHeatingSolar",
            )
            .is_empty()
        {
            if let Some(space_heating) = space_heating {
                let mut context = hot_water_context;
                context.space_heating = Some(space_heating);
                match crate::domestic_hot_water::assess_standalone_solar(
                    &input.space_heating_solar,
                    context,
                    "spaceHeatingSolar",
                ) {
                    Ok(result) => {
                        // 9.2.3.4: the node gain adds to that of solar combis.
                        let mut node = chain_input.solar_heating_kwh.clone();
                        node.resize(12, 0.0);
                        for (value, add) in node.iter_mut().zip(&result.space_heating_kwh) {
                            *value += add;
                        }
                        chain_input.solar_heating_kwh = node;
                        // 13.68 Q_H;sol;ls;rbl into 7.3–7.8.
                        chain_input.solar_recoverable_kwh = result.recoverable_kwh.clone();
                        standalone_solar = Some(result);
                    }
                    Err(error) => standalone_solar_issue = Some(error),
                }
            }
        }
        heating = assess_space_heating_chain(&chain_input);
    }
    let mut issues: Vec<PerformanceIssue> = heating
        .issues
        .iter()
        .map(|item| issue(item.code, format!("spaceHeating.{}", item.path)))
        .collect();
    validate(input, &mut issues);
    if let Some(error) = standalone_solar_issue {
        issues.push(issue(error.code, error.path));
    }

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
    // 13.69/13.137b: the levelled setpoint of 7.76 from the chain's demand.
    hot_water_context.levelled_setpoint_c = levelled_setpoint(input, &heating);
    let extras = hot_water_extras(input, &heating);
    let hot_water_from_heating = hot_water_from_heating(&heating);
    let hot_water = match (&input.hot_water, issues.is_empty()) {
        (Some(system), true) => match crate::domestic_hot_water::assess_hot_water_with(
            system,
            hot_water_context,
            &extras,
        ) {
            Ok(mut result) => {
                if let Some(fit) = &input.hot_water_need_fit {
                    fit_hot_water_need(&mut result, fit.annual_need_kwh);
                }
                Some(result)
            }
            Err(error) => {
                issues.push(issue(error.code, error.path));
                None
            }
        },
        _ => None,
    };
    // §13.8.4.8 (p. 650): the combi micro-CHP's joint input replaces the
    // chain's own heating booking with its heating share.
    if let (Some(result), Some(product)) = (&hot_water, input.space_heating.generator.micro_chp()) {
        if let Some(shares) = &result.combi_chp_heating {
            apply_combi_chp_heating(&mut heating, shares, product.fuel);
        }
    }
    let mut external = resolve_external(input, &mut issues);
    let mut forfait_totals = None;
    if issues.is_empty() {
        totals = Some(compute(
            input,
            &heating,
            cooling.as_ref(),
            hot_water.as_ref(),
            standalone_solar.as_ref(),
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
                standalone_solar.as_ref(),
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
                        // §10.2: only zones a calculated system serves (or
                        // every zone with declared cooling) are cooled.
                        active_cooling: input
                            .active_cooling
                            .as_ref()
                            .filter(|_| !input.has_cooling() || input.zone_cooled(&zone.zone_id)),
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
    // 5.57/5.58: E_Final per carrier from E_EPus; 5.60 adds the solar yields.
    let mut final_by_carrier: Vec<FinalEnergyCarrier> = Vec::new();
    for row in &carriers {
        match final_by_carrier
            .iter_mut()
            .find(|item| item.carrier == row.carrier)
        {
            Some(item) => item.annual_kwh += row.used_kwh,
            None => final_by_carrier.push(FinalEnergyCarrier {
                carrier: row.carrier,
                annual_kwh: row.used_kwh,
            }),
        }
    }
    let final_energy = totals.is_some().then(|| {
        final_by_carrier
            .iter()
            .map(|item| item.annual_kwh)
            .sum::<f64>()
    });
    let solar_yield: f64 = heating
        .monthly
        .iter()
        .map(|row| row.solar_gain_kwh)
        .sum::<f64>()
        + hot_water.as_ref().map_or(0.0, |result| {
            result
                .months
                .iter()
                .map(|month| month.solar_renewable_kwh)
                .sum::<f64>()
        });
    let area = input.total_usable_floor_area_m2;
    let label_functions: Vec<LabelFunctionArea> = if input.label_functions.is_empty() {
        input
            .label_function
            .map(|function| LabelFunctionArea {
                function,
                area_m2: area,
            })
            .into_iter()
            .collect()
    } else {
        input.label_functions.clone()
    };
    let chapter5 = match (totals, scenario, final_energy) {
        (Some(item), Some(indicator), Some(final_kwh)) => {
            use crate::indicators_draft::ceil_per_area;
            // 5.3b/5.3c and 5.3e/5.3f: Q_nd;net per zone by 7.2 without the
            // recoverable losses, with the actual ventilation.
            let assessed =
                || std::iter::once(&heating.demand).chain(&heating.additional_zone_demands);
            let heating_net: f64 = assessed()
                .map(|zone| {
                    zone.annual_heating_need_without_recoverable_kwh
                        .unwrap_or(0.0)
                })
                .sum();
            let cooling_net: f64 = assessed()
                .map(|zone| {
                    zone.annual_cooling_need_without_recoverable_kwh
                        .unwrap_or(0.0)
                })
                .sum();
            let heating_indicator = ceil_per_area(heating_net, area).unwrap_or(f64::NAN);
            let cooling_indicator = ceil_per_area(cooling_net, area).unwrap_or(f64::NAN);
            // §5.3.2: set per dwelling (N_woon = 1).
            let single_dwelling = dwelling_count(input) == Some(1);
            let standard = match (
                residential && single_dwelling,
                input.construction_year,
                input.loss_area_m2,
            ) {
                (true, Some(year), Some(loss)) => standard_insulation(
                    input.space_heating.demand.dwelling_type
                        == Some(crate::monthly_demand::DwellingType::ApartmentBuilding),
                    year,
                    loss,
                    area,
                ),
                _ => None,
            };
            // 5.17–5.19 from E_EPdel per carrier.
            let delivered = |codes: &[&str]| -> f64 {
                carriers
                    .iter()
                    .filter(|row| codes.contains(&row.carrier))
                    .map(|row| row.delivered_kwh)
                    .sum()
            };
            let electricity = delivered(&["el"]);
            let external = delivered(&["dh", "dw", "dc"]) * 3.6 / 1000.0;
            // 5.19a names ci ≠ el, dh; dw and dc are in 5.18a already.
            let other = delivered(&["gas", "oil", "bm"]) * 3.6 / 35.17;
            let renovation = if residential {
                None
            } else {
                renovation_standard(&label_functions)
            };
            let ep2 = indicator.primary_fossil_indicator_kwh_per_m2_year;
            Some(ChapterFiveIndicators {
                heating_need_kwh_per_m2: heating_indicator,
                cooling_need_kwh_per_m2: cooling_indicator,
                heating_and_cooling_need_kwh_per_m2: ((heating_indicator + cooling_indicator)
                    * 100.0)
                    .round()
                    / 100.0,
                meets_standard_insulation: standard.map(|limit| heating_indicator <= limit),
                standard_insulation_kwh_per_m2: standard,
                renewable_indicator_kwh_per_m2: indicator.renewable_indicator_kwh_per_m2_year,
                renewable_indicator_forfait_kwh_per_m2: indicators.as_ref().and_then(|result| {
                    result
                        .scenarios
                        .iter()
                        .find(|item| item.kind == crate::indicators_draft::ScenarioKind::EmgForfait)
                        .map(|item| item.renewable_indicator_kwh_per_m2_year)
                }),
                final_energy_kwh_per_m2: ceil_per_area(final_kwh, area).unwrap_or(f64::NAN),
                final_energy_eed_kwh_per_m2: ceil_per_area(final_kwh + solar_yield, area)
                    .unwrap_or(f64::NAN),
                delivered_electricity_kwh: electricity,
                delivered_electricity_kwh_per_m2: electricity / area,
                delivered_external_gj: external,
                delivered_external_gj_per_m2: external / area,
                delivered_other_m3_aeq: other,
                delivered_other_m3_aeq_per_m2: other / area,
                renewable_by_carrier: item.renewable_by,
                // Gas or oil in the calculation settles it; otherwise the
                // appliances outside the calculation decide.
                locally_carbon_free: if on_site_fossil_use {
                    Some(false)
                } else {
                    input
                        .fossil_appliances_outside_calculation
                        .map(|outside| !outside)
                },
                meets_renovation_standard: renovation.map(|limit| ep2 <= limit),
                renovation_standard_kwh_per_m2: renovation,
            })
        }
        _ => None,
    };
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
        annual_zeb_primary_total_kwh: totals.map(|item| item.zeb_primary),
        zeb_primary_total_indicator_kwh_per_m2: totals.and_then(|item| {
            crate::indicators_draft::ceil_per_area(
                item.zeb_primary,
                input.total_usable_floor_area_m2,
            )
        }),
        annual_zeb_co2_kg: totals.map(|item| item.zeb_co2),
        final_energy_by_carrier: final_by_carrier,
        annual_final_energy_kwh: final_energy,
        annual_final_energy_eed_kwh: final_energy.map(|value| value + solar_yield),
        carriers,
        electricity_balance: balance,
        annual_primary_fossil_kwh: totals.map(|item| item.fossil),
        annual_renewable_primary_kwh: totals.map(|item| item.renewable),
        annual_heat_pump_ambient_heat_kwh: totals.map(|item| item.ambient),
        annual_heating_and_cooling_need_kwh: totals.map(|item| item.need),
        annual_storage_correction_kwh: totals.map(|item| item.storage_correction),
        annual_co2_kg: totals.map(|item| item.co2_kg),
        co2_kg_per_m2: totals.map(|item| item.co2_kg / input.total_usable_floor_area_m2),
        chapter5,
        need_indicator_kwh_per_m2_year: need_indicator,
        primary_fossil_indicator_kwh_per_m2_year: scenario
            .map(|item| item.primary_fossil_indicator_kwh_per_m2_year),
        renewable_share_percent: scenario.map(|item| item.renewable_share_percent),
        // A maatwerkadvies run with actual-use parameters gives no label.
        indicative_label_class: scenario
            .filter(|_| !usage_fit_applied(input))
            .and_then(|item| {
                let ep2 = item.primary_fossil_indicator_kwh_per_m2_year;
                match input.calculation_scope {
                    CalculationScope::Residential => {
                        indicative_label_class(LabelFunction::Residential, ep2)
                    }
                    // §5.3.1: area-weighted bounds for several functions.
                    CalculationScope::Utility => {
                        indicative_label_class_mixed(&label_functions, ep2)
                    }
                }
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
        hot_water_from_heating,
        standalone_solar,
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
    /// Annex AB E_P,ZEB;Tot;an, kWh.
    zeb_primary: f64,
    /// Annex AB m_CO2;ZEB, kg CO2eq.
    zeb_co2: f64,
    /// 5.39a–h.
    renewable_by: RenewableByCarrier,
}

/// 5.39a–h: annual renewable primary energy per carrier ri, kWh.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewableByCarrier {
    /// 5.39a renelect: PV (and other local renewable electricity).
    pub electricity: f64,
    /// 5.39c renheat from heat pumps (space heating and hot water) and
    /// declared renewable heat.
    pub heat_pump_heat: f64,
    /// 5.39d renheat from solar collectors and PVT.
    pub solar_heat: f64,
    /// 5.39e rencold.
    pub cold: f64,
    /// 5.39f biomass.
    pub biomass: f64,
    /// 5.39g external heat (dh, dw and the collective heat-pump source).
    pub external_heat: f64,
    /// 5.39h external cold.
    pub external_cold: f64,
}

impl RenewableByCarrier {
    pub fn total(&self) -> f64 {
        self.electricity
            + self.heat_pump_heat
            + self.solar_heat
            + self.cold
            + self.biomass
            + self.external_heat
            + self.external_cold
    }

    fn add(&mut self, other: &Self) {
        self.electricity += other.electricity;
        self.heat_pump_heat += other.heat_pump_heat;
        self.solar_heat += other.solar_heat;
        self.cold += other.cold;
        self.biomass += other.biomass;
        self.external_heat += other.external_heat;
        self.external_cold += other.external_cold;
    }
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
    standalone_solar: Option<&crate::domestic_hot_water::StandaloneSolarHeating>,
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
    let mut renewable_by = RenewableByCarrier::default();
    let mut ambient_total = 0.0;
    let mut zeb_primary = 0.0;
    let mut zeb_co2 = 0.0;
    let zeb_temperature = input
        .zeb_heat_delivery_temperature
        .unwrap_or(ZebHeatTemperature::AtLeast60);
    // AB.71: f_BAT;el;corr = 1 at 5 kWh or more electrical storage.
    let zeb_battery = input.battery_storage_present
        && input
            .storage
            .as_ref()
            .is_some_and(|storage| storage.building_bound_electrical_kwh >= 5.0);
    let zeb_direct_use = zeb_direct_use_fraction(input);
    // 5.31 for electric and gas-driven heat pumps (COP ≥ 1, source < 20 °C).
    let heat_pump_renewable = (input.space_heating.generator.heat_pump().is_some()
        || input.space_heating.generator.annex_q().is_some()
        || input.space_heating.generator.has_gas_heat_pump())
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
        // 9.84: E = Q/(η·f_prac) with η_H;gen;equiv;dh = 1.
        let mut used_dh = bacs * row.district_heat_kwh / factors.practice_heat;
        // 5.39g: the renewable share counts Q_H;gen;out of external heat and
        // Q_C;gen;out of absorption chillers on it, without f_BACS.
        let mut renewable_dh_basis = row.district_heat_kwh;
        // 5.39h: Q_C;gen;out of external cold, without f_BACS.
        let mut renewable_dc_basis = 0.0;
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
        // CHP electricity of absorption cooling (ε_chp;el) is reported only:
        // 16.11–16.13 credit CHP production of heating and hot-water systems.
        if let Some(assessment) = cooling {
            let month_row = &assessment.months[index];
            used_el += bacs * (month_row.electricity_kwh + month_row.auxiliary_electricity_kwh);
            // Table 10.30 with 9.65: CHP fuel (natural gas, gross value).
            used_gas += bacs * (month_row.natural_gas_kwh + month_row.chp_heat_kwh);
            used_dh += bacs * month_row.district_heat_kwh;
            renewable_dh_basis += month_row.district_heat_cold_kwh;
            // 10.78: f_prpr of external cold.
            used_dc += bacs * month_row.district_cold_kwh / factors.practice_cold;
            renewable_dc_basis += month_row.district_cold_kwh;
            ambient_cold = month_row.ambient_cold_kwh;
        }
        // Chapter 14 lighting (electricity, months by t_mi/t_an).
        used_el += lighting
            .iter()
            .map(|zone| zone.monthly_kwh[index])
            .sum::<f64>();
        let mut hot_water_ambient = 0.0;
        let mut hot_water_chp = 0.0;
        // 5.39d: solar heat for hot water and the space-heating node.
        let mut solar_heat = row.solar_gain_kwh;
        // 13.67: pump energy of standalone space-heating solar systems.
        if let Some(solar) = standalone_solar {
            used_el += solar.auxiliary_kwh.get(index).copied().unwrap_or(0.0);
        }
        if let Some(months) = &hot_water {
            let row = &months[index];
            // 13.1/13.3 per generator carrier; table 5.2 or annex P for
            // external heat (dw).
            used_el += row.electricity_kwh;
            used_gas += row.natural_gas_kwh;
            used_oil += row.oil_kwh;
            // 13.152: f_prac;gi of external heat for hot water.
            used_dw += row.district_heat_kwh / factors.practice_hot_water;
            used_el += row.auxiliary_electricity_kwh;
            hot_water_ambient = row.ambient_heat_kwh;
            hot_water_chp = row.chp_electricity_kwh;
            solar_heat += row.solar_renewable_kwh;
        }
        // 5.24/5.25 with E_nEPus;el = 0 (5.27): self-use capped at EP use.
        let produced_renewable: f64 = input
            .on_site_production
            .iter()
            .map(|item| item.monthly_kwh[index])
            .sum::<f64>()
            + pv_yields.iter().map(|yields| yields[index]).sum::<f64>();
        // 16.11–16.13/16.16: heating and hot-water CHP electricity is own
        // production, not renewable and outside 5.14a (5.14b).
        // 16.12/5.24 carry no f_BACS (a penalty on use only, 5.5.8).
        let chp_electricity = row.chp_electricity_kwh + hot_water_chp;
        let produced = produced_renewable + chp_electricity;
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
        }
        // 5.20 books Q_HD;hp;in;bron as carrier dh (its own factors above).
        let reported_dh = used_dh + if source.is_some() { source_heat } else { 0.0 };
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
        if reported_dh > 0.0 {
            carriers.push(CarrierMonth {
                carrier: "dh",
                month,
                used_kwh: reported_dh,
                delivered_kwh: reported_dh,
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
        // Annex AB (informative): ZEB indicator next to chapter 5.
        let (zeb_p, zeb_c) = zeb_month(
            zeb_direct_use[index],
            zeb_battery,
            used_el,
            produced_renewable,
            chp_electricity,
            used_gas,
            used_oil,
            used_bm,
            used_dh + used_dw,
            zeb_temperature,
            used_dc,
            source_heat,
        );
        zeb_primary += zeb_p;
        zeb_co2 += zeb_c;
        // 5.10 and 5.13: exported electricity is subtracted at f_P;exp;el.
        fossil -= exported * F_P_ELECTRICITY;
        co2 -= exported * K_CO2_ELECTRICITY;
        // 5.14a/5.14b: renewable production only (CHP excluded); the
        // correction is left out of the CO2 emission (§5.5.6.1).
        let correction =
            produced_renewable.min(used_el) * STORAGE_CORRECTION_FACTOR * storage_factor;
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
        // 5.39a–h per carrier ri; their sum is EPrenTot (5.28).
        let month_renewable = RenewableByCarrier {
            electricity: produced_renewable * F_PREN_RENELECT,
            heat_pump_heat: (ambient + declared_heat + hot_water_ambient) * F_PREN_RENHEAT,
            solar_heat: solar_heat * F_PREN_RENHEAT,
            cold: ambient_cold * F_PREN_RENCOLD,
            biomass: biomass_heat * F_PREN_BIOMASS_B,
            external_heat: renewable_dh_basis * dh.renewable_factor
                + used_dw * dw.renewable_factor
                + source.map_or(0.0, |item| source_heat * item.renewable_factor),
            external_cold: renewable_dc_basis * dc.renewable_factor,
        };
        renewable += month_renewable.total();
        renewable_by.add(&month_renewable);
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
        zeb_primary,
        zeb_co2,
        renewable_by,
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
            measured_only: true,
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
        // 9.84: a declaration on calculated values has f_prac = 0,95.
        if let Some(AnnexPRoute::Declared { measured_only, .. }) =
            sample.external_supply.heating.as_mut()
        {
            *measured_only = false;
        }
        let calculated = assess_building_performance(&sample);
        let ep_calculated = calculated.annual_primary_fossil_kwh.unwrap();
        assert!((ep_calculated - (ep + heat * 0.42 * (1.0 / 0.95 - 1.0))).abs() < 1e-6);
        // The EMG forfait scenario keeps f_prac = 1 (fixed factor 0,9).
        assert!(
            (calculated
                .external_supply
                .as_ref()
                .unwrap()
                .forfait_primary_fossil_kwh
                .unwrap()
                - ep_forfait)
                .abs()
                < 1e-6
        );
        if let Some(AnnexPRoute::Declared { measured_only, .. }) =
            sample.external_supply.heating.as_mut()
        {
            *measured_only = true;
        }
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
            regeneration: None,
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
        // 5.20: reported under carrier dh (no other external heat here).
        let listed: f64 = result
            .carriers
            .iter()
            .filter(|item| item.carrier == "dh")
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
            regeneration: None,
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
    fn heating_chp_electricity_is_non_renewable_own_production() {
        use crate::space_heating_chain::{ChpGenerator, OtherGeneratorAuxiliary};
        let mut sample = input();
        sample.on_site_production.clear();
        let base = assess_building_performance(&sample);
        sample.space_heating.generator = Generator::Chp(ChpGenerator {
            chp: Some(crate::space_cooling::ChpClass {
                power_kw: 50.0,
                built_after_2006: true,
                hre_declared: false,
                low_temperature: false,
            }),
            method1: None,
            auxiliary: Some(OtherGeneratorAuxiliary {
                electrically_connected_devices: 1,
                nominal_power_kw: Some(80.0),
                source_reference: "datasheet".into(),
            }),
            equipment_reference: "CHP datasheet".into(),
        });
        // Air heating: no hydronic pump input needed for this booking test.
        sample.space_heating.emission.system = crate::heating_emission::EmissionSystem::AirHeating;
        sample.space_heating.emission.balancing =
            crate::heating_emission::HydronicBalancing::NotApplicable;
        let chp = assess_building_performance(&sample);
        assert_eq!(chp.status, "calculated_unverified", "{:?}", chp.issues);
        let produced: f64 = chp
            .electricity_balance
            .iter()
            .map(|month| month.produced_kwh)
            .sum();
        let expected: f64 = chp
            .space_heating
            .monthly
            .iter()
            .map(|row| row.chp_electricity_kwh)
            .sum();
        assert!(expected > 0.0);
        assert!((produced - expected).abs() < 1e-6);
        // 5.14b: CHP electricity is not renewable.
        assert_eq!(
            chp.annual_renewable_primary_kwh,
            base.annual_renewable_primary_kwh
                .map(|_| chp.annual_renewable_primary_kwh.unwrap())
        );
        assert!(
            chp.annual_renewable_primary_kwh.unwrap()
                <= base.annual_renewable_primary_kwh.unwrap() + 1e-9
        );
    }

    #[test]
    fn zeb_direct_use_is_area_weighted_for_mixed_use() {
        use crate::bbl_requirements::{BblFunction, BblFunctionArea};
        let mut sample = input();
        sample.bbl_functions = vec![
            BblFunctionArea {
                function: BblFunction::ResidentialBuilding,
                area_m2: 50.0,
            },
            BblFunctionArea {
                function: BblFunction::Education,
                area_m2: 50.0,
            },
        ];
        // Text under table AB.1: July 0,5·0,15 + 0,5·0,01; January 0,65.
        let f = zeb_direct_use_fraction(&sample);
        assert!((f[6] - 0.08).abs() < 1e-12);
        assert!((f[0] - 0.65).abs() < 1e-12);
    }

    #[test]
    fn pfhrd_needs_a_gas_boiler_heating_the_building() {
        // 13.156a: the heating gas belongs to the combi only for a gas
        // boiler; other heating gives no PFHRD gas.
        let sample = input();
        assert!(matches!(
            sample.space_heating.generator,
            Generator::GasBoiler(_)
        ));
        assert!(heating_by_gas_boiler(&sample.space_heating.generator));
        let gas = assess_building_performance(&sample);
        assert!(hot_water_extras(&sample, &gas.space_heating)
            .combi_space_heating_gas_kwh
            .is_some());
        let mut electric = sample.clone();
        electric.space_heating.generator = Generator::ElectricResistance(
            crate::space_heating_chain::ElectricResistanceGenerator {
                equipment_reference: "panel heaters".into(),
                auxiliary: None,
            },
        );
        assert!(!heating_by_gas_boiler(&electric.space_heating.generator));
        assert!(hot_water_extras(&electric, &gas.space_heating)
            .combi_space_heating_gas_kwh
            .is_none());
    }

    #[test]
    fn zeb_month_follows_annex_ab() {
        // January dwelling: 300 kWh use, 100 kWh PV, gas 1000 kWh.
        // AB.65: MAX[0,75·100; 0,3·300] = 90 ≤ 100 → 90; no battery.
        let (p, c) = zeb_month(
            0.75,
            false,
            300.0,
            100.0,
            0.0,
            1000.0,
            0.0,
            0.0,
            0.0,
            ZebHeatTemperature::AtLeast60,
            0.0,
            0.0,
        );
        let delivered = 300.0 - 90.0;
        let exported = 100.0 - 90.0;
        assert!((p - (delivered * 1.35 + 1000.0 - exported)).abs() < 1e-9);
        assert!((c - (delivered * 0.268 + 1000.0 * 0.218 - exported * 0.268 / 1.35)).abs() < 1e-9);
        // July with a battery: f_du 0,15; direct = MAX[60; 90] → min(400) = 90,
        // BAT_in = min(0,3·400; 300 − 90; 400 − 90) = 120, out 102.
        let (p, _) = zeb_month(
            0.15,
            true,
            300.0,
            400.0,
            0.0,
            0.0,
            0.0,
            0.0,
            50.0,
            ZebHeatTemperature::From40To60,
            0.0,
            0.0,
        );
        let delivered = 300.0 - 90.0 - 102.0;
        let exported = 400.0 - 90.0 - 120.0;
        assert!((p - (delivered * 1.35 + 50.0 * 0.29 - exported)).abs() < 1e-9);
    }

    #[test]
    fn standard_insulation_follows_5_3_2() {
        // Ground-bound after 1945: 43 below A_ls/A_g 1,0, then 40 per unit.
        assert_eq!(standard_insulation(false, 1975, 80.0, 100.0), Some(43.0));
        assert_eq!(standard_insulation(false, 1975, 150.0, 100.0), Some(63.0));
        // An exact half rounds up: 60 + 105·(68/56 − 1) = 82,5 → 83 (in f64
        // the value lands just below 82,5).
        assert_eq!(standard_insulation(false, 1930, 68.0, 56.0), Some(83.0));
        // Up to 1945: 60 + 105·(x − 1).
        assert_eq!(standard_insulation(false, 1945, 120.0, 100.0), Some(81.0));
        // Apartment building: 95/70 up to 1945, 45/45 after.
        assert_eq!(standard_insulation(true, 1930, 200.0, 100.0), Some(165.0));
        assert_eq!(standard_insulation(true, 2000, 133.0, 100.0), Some(60.0));
        assert_eq!(standard_insulation(true, 2000, 0.0, 100.0), None);
    }

    #[test]
    fn chapter_5_indicators_are_reported() {
        let mut sample = input();
        sample.construction_year = Some(1975);
        sample.loss_area_m2 = Some(150.0);
        sample.loss_area_source_reference = Some("synthetic plan".into());
        sample.fossil_appliances_outside_calculation = Some(false);
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let indicators = result.chapter5.as_ref().unwrap();
        // 5.28 = Σ 5.39a–h.
        assert!(
            (indicators.renewable_by_carrier.total()
                - result.annual_renewable_primary_kwh.unwrap())
            .abs()
                < 1e-6
        );
        assert!(indicators.renewable_by_carrier.electricity > 0.0);
        // 5.3a/5.3b: need without recoverable losses, rounded up to 0,01.
        let net: f64 = std::iter::once(&result.space_heating.demand)
            .chain(&result.space_heating.additional_zone_demands)
            .map(|zone| zone.annual_heating_need_without_recoverable_kwh.unwrap())
            .sum();
        let expected = (net / 100.0 * 100.0).ceil() / 100.0;
        assert!((indicators.heating_need_kwh_per_m2 - expected).abs() < 1e-9);
        // §5.3.2 ground-bound dwelling after 1945, A_ls/A_g 1,5: 63.
        assert_eq!(indicators.standard_insulation_kwh_per_m2, Some(63.0));
        assert_eq!(
            indicators.meets_standard_insulation,
            Some(indicators.heating_need_kwh_per_m2 <= 63.0)
        );
        // 5.17a: delivered electricity; 5.19a gas in m³aeq.
        let delivered = |code: &str| -> f64 {
            result
                .carriers
                .iter()
                .filter(|row| row.carrier == code)
                .map(|row| row.delivered_kwh)
                .sum()
        };
        assert!((indicators.delivered_electricity_kwh - delivered("el")).abs() < 1e-9);
        assert!((indicators.delivered_other_m3_aeq - delivered("gas") * 3.6 / 35.17).abs() < 1e-9);
        assert_eq!(indicators.delivered_external_gj, 0.0);
        // §5.5.7: gas is burnt on site.
        assert_eq!(indicators.locally_carbon_free, Some(false));
        // 5.3h: E_Final per m², rounded up.
        let final_kwh = result.annual_final_energy_kwh.unwrap();
        assert!(indicators.final_energy_kwh_per_m2 >= final_kwh / 100.0);
        assert!(indicators.final_energy_kwh_per_m2 - final_kwh / 100.0 < 0.01);
        // Table 5.7 applies to utility buildings only.
        assert!(indicators.renovation_standard_kwh_per_m2.is_none());
        assert_eq!(
            indicators.renewable_indicator_kwh_per_m2,
            result.indicators.as_ref().unwrap().scenarios[0].renewable_indicator_kwh_per_m2_year
        );
    }

    #[test]
    fn final_energy_sums_carriers_and_wind_is_zero() {
        let mut sample = input();
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        // 5.57/5.58: Σ E_EPus per carrier, own production not netted.
        let used: f64 = result.carriers.iter().map(|row| row.used_kwh).sum();
        assert!((result.annual_final_energy_kwh.unwrap() - used).abs() < 1e-9);
        let by_carrier: f64 = result
            .final_energy_by_carrier
            .iter()
            .map(|item| item.annual_kwh)
            .sum();
        assert!((by_carrier - used).abs() < 1e-9);
        // No solar thermal: E_Final;EED = E_Final (5.60).
        assert_eq!(
            result.annual_final_energy_eed_kwh,
            result.annual_final_energy_kwh
        );
        // 16.17: declared wind production is not allowed.
        sample.on_site_production[0].kind = ProductionKind::Wind;
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "wind_production_not_determinable"));
    }

    #[test]
    fn calculated_pv_adds_to_declared_production() {
        let mut sample = input();
        // Mixing both PV routes is rejected; start without declared PV.
        sample.on_site_production.clear();
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
            obstruction: None,
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
    fn combi_micro_chp_is_evaluated_once_and_split_by_output() {
        use crate::domestic_hot_water::{
            HotWaterChp, HotWaterEmission, HotWaterGenerator, HotWaterNeed, ServedTaps,
        };
        use crate::micro_chp::{
            hot_water_operating_hours, micro_chp_month, ChpTestPoint, MicroChp, MicroChpFuel,
            MicroChpLocation, MicroChpType,
        };
        let product = MicroChp {
            kind: MicroChpType::StirlingEngine,
            fuel: MicroChpFuel::NaturalGas,
            location: MicroChpLocation::InstallationRoom,
            hydraulics: None,
            full_load: ChpTestPoint {
                thermal_power_kw: 20.0,
                electric_power_kw: None,
                thermal_efficiency: None,
                electric_efficiency: None,
                auxiliary_power_kw: Some(0.10),
            },
            chp_only: ChpTestPoint {
                thermal_power_kw: 6.0,
                electric_power_kw: Some(1.0),
                thermal_efficiency: Some(0.80),
                electric_efficiency: None,
                auxiliary_power_kw: Some(0.06),
            },
            standby_loss_kw: None,
            pilot_kw: None,
            standby_electric_kw: None,
            standby_auxiliary_kw: Some(0.01),
            net_production_measured: false,
            storage: None,
            test_report_reference: "EN 50465 report".into(),
        };
        let mut sample = input();
        sample.on_site_production.clear();
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        sample.space_heating.generator = Generator::Chp(crate::space_heating_chain::ChpGenerator {
            chp: None,
            method1: Some(product.clone()),
            auxiliary: None,
            equipment_reference: "micro-CHP".into(),
        });
        // Air heating: no hydronic pump input needed for this booking test.
        sample.space_heating.emission.system = crate::heating_emission::EmissionSystem::AirHeating;
        sample.space_heating.emission.balancing =
            crate::heating_emission::HydronicBalancing::NotApplicable;
        sample.hot_water = Some(HotWaterSystem {
            declared_share: None,
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
            generator: HotWaterGenerator::Chp(Box::new(HotWaterChp {
                chp: None,
                method1: Some(product.clone()),
                also_space_heating: true,
                auxiliary: None,
                equipment_reference: "micro-CHP".into(),
            })),
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
        let hot_water = result.hot_water.as_ref().unwrap();
        assert!(hot_water.combi_chp_heating.is_some());
        // January: one 9.6.6.2 month on Q_H + Q_W with t_H;op + t_W;op.
        let row = &result.space_heating.monthly[0];
        let q_h = row.chp_thermal_output_kwh;
        let q_w = hot_water.months[0].generator_output_kwh;
        let t_w = hot_water_operating_hours(&product, q_w, 1.0, 1.0, 744.0);
        let joint = micro_chp_month(
            &product,
            q_h + q_w,
            (row.chp_operating_hours + t_w).min(744.0),
            744.0,
            1.0,
            false,
        )
        .unwrap();
        let heating_share = q_h / (q_h + q_w);
        assert!((row.chp_input_kwh - joint.input_kwh * heating_share).abs() < 1e-6);
        assert!((row.natural_gas_kwh - joint.input_kwh * heating_share).abs() < 1e-6);
        assert!((row.chp_electricity_kwh - joint.electricity_kwh * heating_share).abs() < 1e-6);
        // The stand-by loss is booked once: less than two separate months.
        let alone_h =
            micro_chp_month(&product, q_h, row.chp_operating_hours, 744.0, 1.0, false).unwrap();
        let alone_w = micro_chp_month(&product, q_w, t_w, 744.0, 1.0, false).unwrap();
        assert!(joint.input_kwh < alone_h.input_kwh + alone_w.input_kwh);
    }

    #[test]
    fn hot_water_chp_electricity_is_own_production() {
        use crate::domestic_hot_water::{
            HotWaterChp, HotWaterEmission, HotWaterGenerator, HotWaterNeed, ServedTaps,
        };
        let mut sample = input();
        sample.on_site_production.clear();
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        sample.hot_water = Some(HotWaterSystem {
            declared_share: None,
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
            generator: HotWaterGenerator::Chp(Box::new(HotWaterChp {
                chp: Some(crate::space_cooling::ChpClass {
                    power_kw: 50.0,
                    built_after_2006: true,
                    hre_declared: false,
                    low_temperature: false,
                }),
                method1: None,
                also_space_heating: false,
                auxiliary: None,
                equipment_reference: "CHP plate".into(),
            })),
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
        let hot_water = result.hot_water.as_ref().unwrap();
        let expected: f64 = result
            .space_heating
            .monthly
            .iter()
            .map(|row| row.chp_electricity_kwh)
            .sum::<f64>()
            + hot_water
                .months
                .iter()
                .map(|month| month.chp_electricity_kwh)
                .sum::<f64>();
        assert!(expected > 0.0);
        let produced: f64 = result
            .electricity_balance
            .iter()
            .map(|month| month.produced_kwh)
            .sum();
        assert!((produced - expected).abs() < 1e-6);
    }

    #[test]
    fn calculated_hot_water_replaces_declared_use_and_blocks_double_count() {
        use crate::domestic_hot_water::{
            GasAppliance, HotWaterEmission, HotWaterGenerator, HotWaterNeed, ServedTaps,
        };
        let mut sample = input();
        let system = HotWaterSystem {
            declared_share: None,
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
            declared_share: None,
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

    fn hot_water_system(generator: crate::domestic_hot_water::HotWaterGenerator) -> HotWaterSystem {
        use crate::domestic_hot_water::{HotWaterEmission, HotWaterNeed, ServedTaps};
        HotWaterSystem {
            declared_share: None,
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
            generator,
            nominal_power_kw: None,
            exhaust_air: None,
            additional_generators: Vec::new(),
            series: None,
            solar: Vec::new(),
            collective: None,
            equipment_reference: "plate".into(),
        }
    }

    #[test]
    fn solar_space_heating_only_feeds_the_node_and_no_hot_water() {
        use crate::domestic_hot_water::{GasAppliance, HotWaterGenerator};
        let mut sample = input();
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        let mut system = hot_water_system(HotWaterGenerator::GasAppliance {
            appliance: GasAppliance::CombiGaskeurHrCw,
            measured_class: Some(crate::domestic_hot_water::ApplicationClass::Class1),
            kitchen_only: false,
            declared: None,
            annex_t: None,
            annex_t_conditions: None,
        });
        sample.hot_water = Some(system.clone());
        let plain = assess_building_performance(&sample);
        system.solar = vec![serde_json::from_value(serde_json::json!({
            "id": "roof", "solarUse": "space_heating",
            "method": {"method": "calculated", "solarType": "preheater",
                "collectors": {"moduleAreaM2": 2.0, "moduleCount": 2, "orientation": "south",
                    "tiltDeg": 45.0, "obstruction": {"method": "minimal"},
                    "efficiency": {"method": "forfait", "collector": "glazed"},
                    "loopPipes": {"method": "forfait"}},
                "storage": {"totalVolumeL": 200.0, "loss": {"method": "label", "label": "b"}}},
            "sourceReference": "datasheet"
        }))
        .unwrap()];
        sample.hot_water = Some(system);
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let hot_water = result.hot_water.as_ref().unwrap();
        // SOL_USE = SHS: f_W;use = 0, f_H;use = 1 (13.85).
        assert_eq!(hot_water.annual_solar_renewable_kwh, 0.0);
        let march = &result.space_heating.monthly[2];
        assert!(march.solar_gain_kwh > 0.0);
        assert!(march.generator_output_kwh < plain.space_heating.monthly[2].generator_output_kwh);
    }

    #[test]
    fn standalone_space_heating_solar_feeds_the_node_without_a_hot_water_system() {
        let mut sample = input();
        let plain = assess_building_performance(&sample);
        let heater: crate::solar_thermal::SolarWaterHeater =
            serde_json::from_value(serde_json::json!({
                "id": "roof", "solarUse": "space_heating",
                "method": {"method": "calculated", "solarType": "preheater",
                    "collectors": {"moduleAreaM2": 2.0, "moduleCount": 2, "orientation": "south",
                        "tiltDeg": 45.0, "obstruction": {"method": "minimal"},
                        "efficiency": {"method": "forfait", "collector": "glazed"},
                        "loopPipes": {"method": "forfait"}},
                    "storage": {"totalVolumeL": 200.0, "loss": {"method": "label", "label": "b"}}},
                "sourceReference": "datasheet"
            }))
            .unwrap();
        assert!(sample.hot_water.is_none());
        sample.space_heating_solar = vec![heater.clone()];
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let solar = result.standalone_solar.as_ref().unwrap();
        let march = &result.space_heating.monthly[2];
        assert!(solar.space_heating_kwh[2] > 0.0);
        // 9.2.3.4: MIN(node, solar) reaches the node.
        assert!(march.solar_gain_kwh > 0.0);
        assert!(march.solar_gain_kwh <= solar.space_heating_kwh[2] + 1e-9);
        assert!(march.generator_output_kwh < plain.space_heating.monthly[2].generator_output_kwh);
        // 13.67: the pump energy is electricity.
        assert!(solar.auxiliary_kwh.iter().sum::<f64>() > 0.0);
        // 13.68: Q_H;sol;ls;rbl joins Q_H;ls;rbl (7.3) and lowers the need.
        assert!(solar.recoverable_kwh[0] > 0.0);
        assert!(
            result.space_heating.monthly[0].heating_need_kwh
                < plain.space_heating.monthly[0].heating_need_kwh
        );
        // Only space-heating systems may stand alone.
        let mut water = heater;
        water.solar_use = crate::solar_thermal::SolarUse::WaterHeating;
        sample.space_heating_solar = vec![water];
        assert!(assess_building_performance(&sample)
            .issues
            .iter()
            .any(|item| item.code == "solar_standalone_space_heating_only"));
    }

    #[test]
    fn hot_water_from_the_heating_system_is_split_per_13_184() {
        use crate::domestic_hot_water::HotWaterGenerator;
        let mut sample = input();
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        sample.hot_water = Some(hot_water_system(HotWaterGenerator::HeatingSystem));
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.space_heating.monthly[0];
        let split = &result.hot_water_from_heating[0];
        let share = jan.hot_water_load_kwh / jan.generator_output_kwh;
        assert!((split.share - share).abs() < 1e-12);
        assert!((split.natural_gas_kwh - share * jan.natural_gas_kwh).abs() < 1e-9);
        // Without a hot-water load on the node there is nothing to split.
        sample.hot_water = Some(hot_water_system(HotWaterGenerator::ElectricInstantaneous));
        assert!(assess_building_performance(&sample)
            .hot_water_from_heating
            .is_empty());
    }

    #[test]
    fn heating_system_hot_water_loads_the_space_heating_node() {
        use crate::domestic_hot_water::HotWaterGenerator;
        let mut sample = input();
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        sample.hot_water = Some(hot_water_system(HotWaterGenerator::ElectricInstantaneous));
        let plain = assess_building_performance(&sample);
        assert_eq!(plain.status, "calculated_unverified", "{:?}", plain.issues);
        sample.hot_water = Some(hot_water_system(HotWaterGenerator::HeatingSystem));
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let hot_water = result.hot_water.as_ref().unwrap();
        let jan = &result.space_heating.monthly[0];
        let load = hot_water.months[0].heating_system_load_kwh;
        assert!(load > 0.0);
        // 13.185: the node supplies E_W;gen;in;conv;hj; hot water itself has
        // no carrier.
        assert!((jan.hot_water_load_kwh - load).abs() < 1e-9);
        assert_eq!(hot_water.months[0].carrier_input_kwh, 0.0);
        let extra = jan.generator_output_kwh - plain.space_heating.monthly[0].generator_output_kwh;
        // The electric instantaneous heater adds recoverable losses (13.179)
        // to the plain run, so the difference is at least the load.
        assert!(extra >= load - 1e-6);
    }

    #[test]
    fn exhaust_air_hot_water_heat_pump_feeds_13_148_and_13_149_into_chapter_11() {
        use crate::domestic_hot_water::{ExhaustAirUse, HotWaterGenerator};
        let mut sample = input();
        sample.declared_uses.retain(|item| {
            !matches!(
                item.service,
                Service::DomesticHotWater | Service::VentilationFans
            )
        });
        let demand = &mut sample.space_heating.demand;
        let ventilation: crate::ventilation::VentilationInput =
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
                "system": {"kind": "single", "unit": {"variant": "c1", "ducts": "luka_a_b_c", "equipmentReference": "synthetic"}},
                "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "synthetic"},
                "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020},
                "sourceReference": "synthetic"
            }))
            .unwrap();
        demand.ventilation_flows.clear();
        demand.ventilation = Some(ventilation.clone());
        let area = demand.usable_floor_area_m2;
        let mut system = hot_water_system(HotWaterGenerator::HeatPump {
            exhaust_air_source: true,
            source_correction: None,
            measured_class: None,
            outdoor_air_fraction: None,
        });
        system.exhaust_air = Some(ExhaustAirUse {
            ventilation_suitable: true,
            heating_time_fraction: Vec::new(),
            declared_flow_m3_per_h: None,
        });
        sample.hot_water = Some(system);
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let hot_water = result.hot_water.as_ref().unwrap();
        let f_w = hot_water.exhaust_air.as_ref().unwrap().time_fraction[0];
        assert!(f_w > 0.0 && f_w < 1.0);
        // 13.148 forfait (one zone): MAX(MAX(44; 0,44·A)·3,6; q_ODA;req).
        let base = crate::ventilation::calculate_ventilation(&ventilation)
            .unwrap()
            .months[0]
            .heating
            .required_outdoor_air_m3_per_h;
        let q_w = ((44.0_f64).max(0.44 * area) * 3.6).max(base);
        // 11.23: (1 − f_H − f_W)·q + f_W·q_W with f_H = 0.
        let expected = (1.0 - f_w) * base + f_w * q_w;
        let actual = result
            .space_heating
            .demand
            .ventilation
            .as_ref()
            .unwrap()
            .months[0]
            .heating
            .required_outdoor_air_m3_per_h;
        assert!((actual - expected).abs() < 1e-9, "{actual} vs {expected}");
    }

    #[test]
    fn combi_exhaust_air_heat_pump_creates_the_overventilation_block() {
        use crate::domestic_hot_water::{ExhaustAirUse, HotWaterGenerator};
        let mut sample = input();
        sample.declared_uses.retain(|item| {
            !matches!(
                item.service,
                Service::DomesticHotWater | Service::VentilationFans
            )
        });
        let demand = &mut sample.space_heating.demand;
        let ventilation: crate::ventilation::VentilationInput =
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
                "system": {"kind": "single", "unit": {"variant": "c1", "ducts": "luka_a_b_c", "equipmentReference": "synthetic"}},
                "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "synthetic"},
                "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020},
                "sourceReference": "synthetic"
            }))
            .unwrap();
        demand.ventilation_flows.clear();
        demand.ventilation = Some(ventilation);
        let mut system = hot_water_system(HotWaterGenerator::HeatPump {
            exhaust_air_source: true,
            source_correction: None,
            measured_class: None,
            outdoor_air_fraction: None,
        });
        // A combi heat pump (f_combi = 1 in October–March).
        let mut combi = vec![0.0; 12];
        for month in [0, 1, 2, 9, 10, 11] {
            combi[month] = 1.0;
        }
        system.exhaust_air = Some(ExhaustAirUse {
            ventilation_suitable: true,
            heating_time_fraction: combi,
            declared_flow_m3_per_h: Some(250.0),
        });
        sample.hot_water = Some(system);
        let mut chain = sample.space_heating.clone();
        let context = hot_water_context(&sample);
        let result = crate::domestic_hot_water::assess_hot_water(
            sample.hot_water.as_ref().unwrap(),
            context,
        )
        .unwrap();
        let exhaust = result.exhaust_air.as_ref().unwrap();
        assert!(!exhaust.hot_water_only);
        apply_exhaust_air_hot_water(&result, chain.demand.usable_floor_area_m2, &mut chain);
        let over = chain
            .demand
            .ventilation
            .as_ref()
            .unwrap()
            .overventilation
            .as_ref()
            .expect("created for the combi heat pump");
        assert_eq!(over.hot_water_time_fraction, exhaust.time_fraction);
        assert_eq!(over.heating_time_fraction, vec![0.0; 12]);
        // 13.148a with the declared flow.
        assert!(over
            .hot_water_flow_m3_per_h
            .iter()
            .all(|q| *q >= 250.0 - 1e-9));
        let full = assess_building_performance(&sample);
        assert_eq!(full.status, "calculated_unverified", "{:?}", full.issues);
    }

    #[test]
    fn hot_water_losses_use_the_levelled_setpoint() {
        use crate::domestic_hot_water::{
            Circulation, CirculationPump, HotWaterGenerator, PipeInsulation, PumpControl,
        };
        let mut sample = input();
        sample
            .declared_uses
            .retain(|item| item.service != Service::DomesticHotWater);
        let mut system = hot_water_system(HotWaterGenerator::ElectricInstantaneous);
        system.circulation = Some(Circulation {
            length_m: Some(20.0),
            unheated_length_m: Some(0.0),
            outer_diameter_mm: Some(22.0),
            insulation: PipeInsulation::None,
            fittings_insulated: false,
            declared_psi_w_per_mk: Some(0.3),
            connected_dwellings: Some(1),
            floor_count: 1,
            sport_hall_area_m2: 0.0,
            unheated_ambient_c: None,
            pump: CirculationPump {
                control: PumpControl::UncontrolledOrUnknown,
                label_power_kw: None,
                energy_efficiency_index: None,
            },
            source_reference: "drawing".into(),
        });
        sample.hot_water = Some(system.clone());
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        // 13.26 with ϑ_W;amb = ϑ_int;set;H;zi,mi of 7.9.4 (levelled).
        let levelled = result.space_heating.demand.monthly[0].heating.setpoint_c;
        let loss = result.hot_water.as_ref().unwrap().months[0].circulation_loss_kwh;
        let equivalent = 20.0 + 0.15 / 0.3 * 20.0;
        let expected = 744.0 / 1000.0 * 0.3 * (62.5 - levelled) * equivalent;
        assert!((loss - expected).abs() < 1e-9, "{loss} vs {expected}");
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
            declared_share: None,
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
    fn several_cooling_systems_serve_their_own_zones() {
        use crate::space_heating_chain::ChainZone;
        let mut sample = input();
        let mut second = sample.space_heating.demand.clone();
        second.zone_id = "z2".into();
        sample.space_heating.additional_zones.push(ChainZone {
            demand: second,
            emission: sample.space_heating.emission.clone(),
            distribution: sample.space_heating.distribution.clone(),
        });
        sample.total_usable_floor_area_m2 *= 2.0;
        let first_zone = sample.space_heating.demand.zone_id.clone();
        let compression = || {
            cooling_system(CoolingGeneratorKind::Compression {
                heat_rejection: None,
                declared: None,
                performance: None,
            })
        };
        // One system for every zone.
        sample.cooling = Some(compression());
        let all = assess_building_performance(&sample);
        assert_eq!(all.status, "calculated_unverified", "{:?}", all.issues);
        let all_cooling = cooling_assessment(&sample, &all.space_heating).unwrap();
        let need = |assessment: &CoolingAssessment| -> f64 {
            assessment.months.iter().map(|row| row.need_kwh).sum()
        };
        // §10.2: a system serving only the first zone carries only its need;
        // the second zone is not cooled.
        sample.cooling = None;
        sample.cooling_systems = vec![ServedCoolingSystem {
            zone_ids: vec![first_zone.clone()],
            system: compression(),
        }];
        let one = assess_building_performance(&sample);
        assert_eq!(one.status, "calculated_unverified", "{:?}", one.issues);
        let one_cooling = cooling_assessment(&sample, &one.space_heating).unwrap();
        let zone_need: f64 = one
            .space_heating
            .demand
            .monthly
            .iter()
            .map(|row| row.cooling.need_kwh)
            .sum();
        assert!((need(&one_cooling) - zone_need).abs() < 1e-6);
        assert!(need(&one_cooling) < need(&all_cooling));
        assert_eq!(one_cooling.systems.len(), 1);
        assert_eq!(one_cooling.zone_booster_extraction_kwh.len(), 2);
        // Two systems: the needs add up to the one-system total.
        sample.cooling_systems.push(ServedCoolingSystem {
            zone_ids: vec!["z2".into()],
            system: compression(),
        });
        let two = assess_building_performance(&sample);
        assert_eq!(two.status, "calculated_unverified", "{:?}", two.issues);
        let two_cooling = cooling_assessment(&sample, &two.space_heating).unwrap();
        assert!((need(&two_cooling) - need(&all_cooling)).abs() < 1e-6);
        assert!(two_cooling
            .generator_shares
            .iter()
            .any(|item| item.id.starts_with("system1:")));
        // Validation: unknown zone, a zone twice and both inputs at once.
        sample.cooling_systems[1].zone_ids = vec![first_zone.clone(), "nope".into()];
        sample.cooling = Some(compression());
        let codes: Vec<&str> = assess_building_performance(&sample)
            .issues
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "cooling_zone_served_twice",
            "cooling_zone_unknown",
            "cooling_and_cooling_systems_exclusive",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
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
    fn external_cold_uses_forfait_factor_and_chp_absorption_books_gas_and_electricity() {
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
            rating: None,
        }));
        let chp = assess_building_performance(&sample);
        assert_eq!(chp.status, "calculated_unverified", "{:?}", chp.issues);
        let cooling = cooling_assessment(&sample, &chp.space_heating).unwrap();
        // Table 9.31, 20–200 kW after 2006 HT: ε_th 0,49, ε_el 0,30; table
        // 10.30: fuel = cold / (1,00·0,49).
        let july = &cooling.months[6];
        assert!(july.chp_heat_kwh > 0.0);
        assert!((july.chp_heat_kwh - july.generator_cold_kwh / 0.49).abs() < 1e-6);
        assert!((july.chp_electricity_kwh - 0.30 * july.chp_heat_kwh).abs() < 1e-9);
        let gas = |result: &BuildingPerformanceAssessment| -> f64 {
            result
                .carriers
                .iter()
                .filter(|item| item.carrier == "gas")
                .map(|item| item.used_kwh)
                .sum()
        };
        let fuel: f64 = cooling.months.iter().map(|month| month.chp_heat_kwh).sum();
        let bacs = sample.bacs_factor;
        assert!((gas(&chp) - gas(&base) - bacs * fuel).abs() < 1e-6);
        // 16.11–16.13 credit only heating and hot-water CHP: the absorber's
        // CHP electricity is reported but not produced in the balance.
        let produced: f64 = chp
            .electricity_balance
            .iter()
            .map(|month| month.produced_kwh)
            .sum();
        let base_produced: f64 = base
            .electricity_balance
            .iter()
            .map(|month| month.produced_kwh)
            .sum();
        let chp_el: f64 = cooling
            .months
            .iter()
            .map(|month| month.chp_electricity_kwh)
            .sum();
        assert!(chp_el > 0.0);
        assert!((produced - base_produced).abs() < 1e-6);
        assert_eq!(
            chp.annual_renewable_primary_kwh,
            base.annual_renewable_primary_kwh
        );
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
            burning_hours_factor: None,
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
            obstruction: None,
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
