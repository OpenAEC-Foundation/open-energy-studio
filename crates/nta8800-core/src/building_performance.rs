//! End-to-end unverified energy performance of a single-zone building:
//! space-heating chain + declared other services → `E_EPus` per carrier
//! (5.20/5.21) → on-site electricity self-use and export (5.22–5.26) →
//! primary fossil energy `EPTot` (5.9–5.14, table 5.2) and renewable energy
//! `EPrenTot` (5.29–5.31, 5.39, table 5.4) → indicators with the rounding of
//! `indicators_draft`.
//!
//! Source: public 2026 consultation draft of chapter 5, not the verified
//! target edition. Battery storage (5.14a), exported heat, external heat or
//! cold delivery, biomass and collective heat-pump sources are rejected
//! because their draft rules are incomplete or not wired. Non-EP electricity
//! is fixed at 0 as required for the indicators (5.27).

use crate::bbl_requirements::{check as bbl_check, BblCheck, BblFunction};
use crate::domestic_hot_water::{monthly_hot_water, validate_hot_water, HotWaterSystem};
use crate::final_energy_draft::DRAFT_SOURCE;
use crate::forfait_heat_pump_draft::TableSource;
use crate::forfait_heat_pump_monthly_draft::SourceSystem;
use crate::indicators_draft::{
    assess_indicators_draft, AnnualScenario, CalculationScope, IndicatorsDraftAssessment,
    IndicatorsDraftInput, ScenarioKind,
};
use crate::label_class::{indicative_label_class, LabelFunction, LABEL_SOURCE};
use crate::pv::{monthly_yield_kwh, validate_pv, PvSystem};
use crate::space_heating_chain::{
    assess_space_heating_chain, Generator, SpaceHeatingChainAssessment, SpaceHeatingChainInput,
};
use crate::tojuli::{assess_tojuli, TojuliAssessment};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Table 5.2 `f_P;del` / `f_P;pr;us` / `f_P;exp` for electricity.
pub const F_P_ELECTRICITY: f64 = 1.45;
/// Table 5.2 `f_P;del` for natural gas and fuel oil.
pub const F_P_GAS: f64 = 1.0;
pub const F_P_OIL: f64 = 1.0;
/// Table 5.4.
pub const F_PREN_RENELECT: f64 = 1.45;
pub const F_PREN_RENHEAT: f64 = 1.0;

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
}

impl Service {
    /// `f_BACS` applies to cooling and its auxiliary energy (5.20/5.21).
    fn bacs_weighted(self) -> bool {
        matches!(self, Self::SpaceCooling | Self::SpaceCoolingAuxiliary)
    }

    fn electric_only(self) -> bool {
        !matches!(self, Self::DomesticHotWater | Self::SpaceCooling)
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
    /// `f_BACS` (5.5.8): 1,0 or 1,05 with source.
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
    /// §5.7.1: a zone with sufficient active cooling may use TOjuli = 0.
    #[serde(default)]
    pub active_cooling_present: bool,
    /// Row of Bbl table 4.148A for the requirement check.
    #[serde(default)]
    pub bbl_function: Option<BblFunction>,
    /// Loss area `A_ls` in m² for the `A_ls/A_g` ratio of table 4.148A.
    #[serde(default)]
    pub loss_area_m2: Option<f64>,
    #[serde(default)]
    pub loss_area_source_reference: Option<String>,
    /// Domestic hot water calculated here (chapter 13, partial).
    #[serde(default)]
    pub hot_water: Option<HotWaterSystem>,
    /// Confirms the demand input uses the fixed C1 ventilation system and
    /// fixed internal loads of §5.4.3; only then the need indicator is shown.
    pub demand_uses_fixed_c1_ventilation: bool,
    pub battery_storage_present: bool,
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
    pub space_heating: SpaceHeatingChainAssessment,
    pub indicators: Option<IndicatorsDraftAssessment>,
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

fn validate(input: &BuildingPerformanceInput, issues: &mut Vec<PerformanceIssue>) {
    if !input.total_usable_floor_area_m2.is_finite() || input.total_usable_floor_area_m2 <= 0.0 {
        issues.push(issue("usable_floor_area_invalid", "totalUsableFloorAreaM2"));
    }
    if input.area_source_reference.trim().is_empty() {
        issues.push(issue("source_reference_required", "areaSourceReference"));
    }
    if input.bacs_factor != 1.0 && input.bacs_factor != 1.05 {
        issues.push(issue("bacs_factor_invalid", "bacsFactor"));
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
    if input.battery_storage_present {
        // 5.14a contains an unresolved placeholder in the public draft.
        issues.push(issue(
            "battery_storage_unsupported",
            "batteryStoragePresent",
        ));
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
    if let Some(system) = &input.hot_water {
        issues.extend(
            validate_hot_water(system, "hotWater")
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
    match (&input.space_heating.generator, &input.heat_pump_renewable) {
        (Generator::HeatPumpForfait(generator), evidence) => {
            if generator.source_system != SourceSystem::Individual {
                // 5.20/5.30 collective source terms need the dh factor route.
                issues.push(issue(
                    "collective_heat_pump_source_unsupported",
                    "spaceHeating.generator.sourceSystem",
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
                    let source = generator.forfait.source;
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
        (Generator::GasBoiler(_), Some(_)) => {
            issues.push(issue(
                "heat_pump_evidence_without_heat_pump",
                "heatPumpRenewable",
            ));
        }
        (Generator::GasBoiler(_), None) => {}
    }
}

pub fn assess_building_performance(
    input: &BuildingPerformanceInput,
) -> BuildingPerformanceAssessment {
    let fingerprint =
        input_fingerprint(&serde_json::to_value(input).expect("typed input serializes"));
    let heating = assess_space_heating_chain(&input.space_heating);
    let mut issues: Vec<PerformanceIssue> = heating
        .issues
        .iter()
        .map(|item| issue(item.code, format!("spaceHeating.{}", item.path)))
        .collect();
    validate(input, &mut issues);

    let mut carriers = Vec::new();
    let mut balance = Vec::new();
    let mut totals = None;
    if issues.is_empty() {
        totals = Some(compute(input, &heating, &mut carriers, &mut balance));
    }
    let mut indicators = None;
    if let Some((fossil, renewable, _, need)) = totals {
        let result = assess_indicators_draft(&IndicatorsDraftInput {
            calculation_scope: input.calculation_scope,
            total_usable_floor_area_m2: input.total_usable_floor_area_m2,
            area_source_reference: input.area_source_reference.clone(),
            annual_need_c1_kwh: need,
            need_source_reference: "derived by monthly_demand".into(),
            scenarios: vec![AnnualScenario {
                kind: ScenarioKind::Ordinary,
                annual_primary_fossil_kwh: fossil,
                annual_renewable_kwh: renewable,
                source_reference: "derived by building_performance".into(),
            }],
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
    let need_indicator = indicators
        .as_ref()
        .filter(|_| valid && input.demand_uses_fixed_c1_ventilation)
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
    let tojuli: Vec<TojuliAssessment> = if valid {
        zone_demands()
            .map(|zone| assess_tojuli(zone, input.active_cooling_present))
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
        annual_primary_fossil_kwh: totals.map(|item| item.0),
        annual_renewable_primary_kwh: totals.map(|item| item.1),
        annual_heat_pump_ambient_heat_kwh: totals.map(|item| item.2),
        annual_heating_and_cooling_need_kwh: totals.map(|item| item.3),
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
        bbl_check: match (input.bbl_function, input.loss_area_m2, scenario) {
            (Some(function), Some(area), Some(item)) => bbl_check(
                function,
                area / input.total_usable_floor_area_m2,
                heating_capacity,
                need_indicator,
                Some(item.primary_fossil_indicator_kwh_per_m2_year),
                Some(item.renewable_share_percent),
            ),
            _ => None,
        },
        space_heating: heating,
        indicators: indicators.filter(|_| valid),
        issues,
    }
}

/// Returns (EPTot, EPrenTot, heat-pump ambient heat, Q_H+C;nd) in kWh.
fn compute(
    input: &BuildingPerformanceInput,
    heating: &SpaceHeatingChainAssessment,
    carriers: &mut Vec<CarrierMonth>,
    balance: &mut Vec<ElectricityBalanceMonth>,
) -> (f64, f64, f64, f64) {
    let bacs = input.bacs_factor;
    let mut fossil = 0.0;
    let mut renewable = 0.0;
    let mut ambient_total = 0.0;
    let heat_pump_renewable =
        matches!(input.space_heating.generator, Generator::HeatPumpForfait(_))
            && input
                .heat_pump_renewable
                .as_ref()
                .is_some_and(|evidence| evidence.source_below_20_c && !evidence.exhaust_air_source);
    let cop = heating.generation_efficiency.unwrap_or(0.0);
    let pv_yields: Vec<[f64; 12]> = input.pv_systems.iter().map(monthly_yield_kwh).collect();
    let hot_water = input.hot_water.as_ref().map(|system| {
        (
            system.carrier,
            monthly_hot_water(system, input.total_usable_floor_area_m2),
        )
    });
    for index in 0..12 {
        let month = (index + 1) as u8;
        let row = &heating.monthly[index];
        // 5.20/5.21: space heating and its auxiliary energy weighted by f_BACS.
        let mut used_el =
            bacs * (row.generator_electricity_kwh + row.auxiliary_electricity_kwh.unwrap_or(0.0));
        let mut used_gas = bacs * row.natural_gas_kwh;
        let mut used_oil = 0.0;
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
        let mut hot_water_ambient = 0.0;
        if let Some((carrier, months)) = &hot_water {
            let row = &months[index];
            match carrier {
                Carrier::El => used_el += row.carrier_input_kwh,
                Carrier::Gas => used_gas += row.carrier_input_kwh,
                Carrier::Oil => used_oil += row.carrier_input_kwh,
            }
            used_el += row.auxiliary_electricity_kwh;
            hot_water_ambient = row.ambient_heat_kwh;
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
            carriers.push(CarrierMonth {
                carrier: carrier.code(),
                month,
                used_kwh: used,
                delivered_kwh: delivered,
            });
        }
        // 5.10 and 5.13: exported electricity is subtracted at f_P;exp;el.
        fossil -= exported * F_P_ELECTRICITY;
        balance.push(ElectricityBalanceMonth {
            month,
            used_kwh: used_el,
            produced_kwh: produced,
            self_used_kwh: self_used,
            exported_kwh: exported,
        });

        // 5.30/5.31: ambient heat of the space-heating heat pump.
        let ambient = if heat_pump_renewable && cop >= 1.0 {
            row.generator_output_kwh * (1.0 - 1.0 / cop)
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
        renewable += (ambient + declared_heat + hot_water_ambient) * F_PREN_RENHEAT
            + produced * F_PREN_RENELECT;
    }
    let need = std::iter::once(&heating.demand)
        .chain(&heating.additional_zone_demands)
        .map(|demand| {
            demand.annual_heating_need_kwh.unwrap_or(0.0)
                + demand.annual_cooling_need_kwh.unwrap_or(0.0)
        })
        .sum();
    (fossil, renewable, ambient_total, need)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space_heating_chain::HeatPumpGenerator;
    use serde_json::json;

    fn chain() -> SpaceHeatingChainInput {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-space-heating-chain-synthetic.json"
        ))
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
            "battery_storage_unsupported",
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
        let base = assess_building_performance(&sample);
        sample.pv_systems.push(PvSystem {
            id: "roof-pv".into(),
            peak_power_kw: 3.0,
            azimuth_deg: 180.0,
            tilt_deg: 35.0,
            performance_factor: 0.80,
            shading_correction: 1.0,
            obstruction_factor: 1.0,
            source_reference: "datasheet".into(),
        });
        let result = assess_building_performance(&sample);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let pv: f64 = monthly_yield_kwh(&sample.pv_systems[0]).iter().sum();
        let delta = result.annual_renewable_primary_kwh.unwrap()
            - base.annual_renewable_primary_kwh.unwrap();
        assert!((delta - pv * 1.45).abs() < 1e-6);
        let fossil_delta =
            base.annual_primary_fossil_kwh.unwrap() - result.annual_primary_fossil_kwh.unwrap();
        assert!((fossil_delta - pv * 1.45).abs() < 1e-6);
    }

    #[test]
    fn calculated_hot_water_replaces_declared_use_and_blocks_double_count() {
        use crate::domestic_hot_water::HotWaterNeed;
        let mut sample = input();
        let system = HotWaterSystem {
            need: HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "one dwelling".into(),
            },
            emission_efficiency: 0.9,
            distribution_efficiency: 1.0,
            generation_efficiency: 0.8,
            carrier: Carrier::Gas,
            shower_heat_recovery_kwh: Vec::new(),
            auxiliary_electricity_kwh: Vec::new(),
            renewable_heat_pump: false,
            efficiency_source_reference: "synthetic".into(),
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
        let expected =
            result.space_heating.annual_natural_gas_kwh.unwrap() + 856.0 * 2.28 / 0.9 / 0.8;
        assert!((gas - expected).abs() < 1e-6);
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
}
