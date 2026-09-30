//! Monthly space-heating chain for one zone and one generator:
//! need (chapter 7) → emission (9.3) → distribution (9.4) → generator (9.6)
//! → energy per carrier.
//!
//! With a single generator the table 9.1 dispatch gives `β = 1` and the
//! generator covers the whole node input. Node losses and gains (9.2.3),
//! recoverable system losses (9.2.5), emission fan energy (9.21) and heat
//! pump auxiliaries are not included; they are listed in
//! [`OMITTED_TERMS`]. All results are unverified.

use crate::boiler_forfait_draft::{
    assess_boiler_forfait_monthly_draft, BoilerForfaitDraftInput, BoilerForfaitMonthlyDraftInput,
};
use crate::climate::OUTDOOR_TEMPERATURE_C;
use crate::final_energy_draft::MonthlyEnergy;
use crate::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput;
use crate::forfait_heat_pump_monthly_draft::{
    assess_forfait_heat_pump_monthly_draft, ForfaitHeatPumpMonthlyDraftInput, SourceSystem,
};
use crate::generator_dispatch_draft::{
    Generator as DispatchGenerator, GeneratorDispatchDraftInput,
};
use crate::heating_aux_draft::{
    assess_heating_aux_measured_draft, GeneratorElectricityMonth, HeatingAuxMeasuredDraftInput,
};
use crate::heating_emission::{
    balancing_consistent, monthly_loss_kwh, temperature_increment_k, EmissionInput, EmissionSystem,
    DRAFT_SOURCE,
};
use crate::hybrid_heat_pump_monthly_draft::{
    assess_hybrid_heat_pump_monthly_draft, HybridHeatPumpAuxMeasurements,
    HybridHeatPumpMonthlyDraftInput,
};
use crate::monthly_demand::{assess_monthly_demand, MonthlyDemandAssessment, MonthlyDemandInput};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};

pub const OMITTED_TERMS: &[&str] = &[
    "9.2.3 node losses and node gains (including solar thermal)",
    "9.2.5 recoverable system losses fed back to the zone",
    "9.21 emission fan energy for fan-assisted emitters",
    "source pump/fan energy and heat pump auxiliaries without measured powers",
    "more than two generators, product-specific hybrid switching and domestic hot water priority",
    "9.6.8.2 auxiliary energy of external heat supply and other generators",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpaceHeatingChainInput {
    pub demand: MonthlyDemandInput,
    pub emission: EmissionInput,
    pub distribution: Distribution,
    /// Further calculation zones served by the same generator (9.2, sum over zones).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_zones: Vec<ChainZone>,
    pub generator: Generator,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChainZone {
    pub demand: MonthlyDemandInput,
    pub emission: EmissionInput,
    pub distribution: Distribution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Distribution {
    /// 9.4.1: piping only in heated zones and only for space heating, so the
    /// in-zone length is set to 0 and the loss is neglected.
    HeatedZoneOnlySpaceHeating {
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Monthly `Q_H;dis;ls` determined elsewhere, kWh.
    Declared {
        #[serde(rename = "monthlyLossKwh")]
        monthly_loss_kwh: Vec<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

// Input data read once per calculation; variant size does not matter here.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Generator {
    GasBoiler(GasBoilerGenerator),
    HeatPumpForfait(HeatPumpGenerator),
    /// Heat pump with a supplementary boiler, split by table 9.1/9.23 (new build).
    HybridHeatPump(Box<HybridGenerator>),
    /// External heat supply (9.6.7): heat is the energy carrier `dh`.
    ExternalHeat(ExternalHeatGenerator),
    /// Local or central electric resistance heating, COP 1,0 (table 9.27).
    ElectricResistance(ElectricResistanceGenerator),
    /// Solid-biomass stove or boiler, forfait efficiency of table 9.30.
    Biomass(BiomassGenerator),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ElectricResistanceGenerator {
    pub equipment_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomassAppliance {
    FreestandingWoodStove,
    InsertStove,
    PelletStove,
    AccumulatingStove,
    CentralBoiler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomassLocation {
    InsideThermalBoundary,
    OutsideThermalBoundary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BiomassGenerator {
    pub appliance: BiomassAppliance,
    pub location: BiomassLocation,
    /// Table 9.30 and the bmB factors apply to appliances of at most 500 kW
    /// that meet the combustion and emission limits of annex R.
    pub annex_r_compliant_at_most_500_kw: bool,
    pub annex_r_reference: String,
    pub equipment_reference: String,
}

/// Table 9.30 forfait efficiency; `None` where the table has no value.
pub fn biomass_efficiency(appliance: BiomassAppliance, location: BiomassLocation) -> Option<f64> {
    use BiomassAppliance::*;
    match (appliance, location) {
        (
            FreestandingWoodStove | InsertStove | AccumulatingStove,
            BiomassLocation::InsideThermalBoundary,
        ) => Some(0.600),
        (PelletStove, BiomassLocation::InsideThermalBoundary) => Some(0.725),
        (CentralBoiler, BiomassLocation::InsideThermalBoundary) => Some(0.800),
        (CentralBoiler, BiomassLocation::OutsideThermalBoundary) => Some(0.750),
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalHeatGenerator {
    /// Invoice, contract or other proof of external supply (9.6.7.1).
    pub supplier_reference: String,
    /// A quality declaration (annex P) needs a paired forfait scenario and is
    /// not supported yet; only the fixed factor route is calculated.
    pub quality_declaration_present: bool,
}

impl Generator {
    /// The electric heat pump of this generator, if any.
    pub fn heat_pump(&self) -> Option<(&ForfaitHeatPumpDraftInput, SourceSystem)> {
        match self {
            Self::GasBoiler(_) => None,
            Self::HeatPumpForfait(generator) => Some((&generator.forfait, generator.source_system)),
            Self::HybridHeatPump(generator) => Some((&generator.forfait, generator.source_system)),
            Self::ExternalHeat(_) | Self::ElectricResistance(_) | Self::Biomass(_) => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HybridGenerator {
    /// Only the new-build installed-power route of the draft is supported.
    pub design_context: String,
    /// Heat pump and boiler with class, power and priority efficiency (table 9.1).
    pub generators: Vec<DispatchGenerator>,
    pub forfait: ForfaitHeatPumpDraftInput,
    pub boiler: BoilerForfaitDraftInput,
    pub source_system: SourceSystem,
    pub source_system_reference: String,
    #[serde(default)]
    pub declared_operating_limits_present: bool,
    #[serde(default)]
    pub heat_pump_auxiliary_measurements: Option<HybridHeatPumpAuxMeasurements>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasBoilerGenerator {
    pub boiler: BoilerForfaitDraftInput,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatPumpGenerator {
    pub forfait: ForfaitHeatPumpDraftInput,
    pub source_system: SourceSystem,
    pub source_system_reference: String,
    /// Measured auxiliary powers of an individual heat pump (9.85–9.88).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary_measurements: Option<HybridHeatPumpAuxMeasurements>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainMonth {
    pub month: u8,
    pub heating_need_kwh: f64,
    pub emission_loss_kwh: f64,
    pub emission_input_kwh: f64,
    pub distribution_loss_kwh: f64,
    pub generator_output_kwh: f64,
    /// Part of the generator output delivered by an electric heat pump.
    pub heat_pump_output_kwh: f64,
    pub natural_gas_kwh: f64,
    /// Delivered external heat, carrier `dh` (9.84).
    pub district_heat_kwh: f64,
    /// Solid biomass input, carrier `bm` (9.64).
    pub biomass_kwh: f64,
    pub generator_electricity_kwh: f64,
    pub auxiliary_electricity_kwh: Option<f64>,
    pub collective_source_heat_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceHeatingChainAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub chapter_9_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub omitted_terms: &'static [&'static str],
    pub emission_temperature_increment_k: Option<f64>,
    pub generation_efficiency: Option<f64>,
    pub monthly: Vec<ChainMonth>,
    pub annual_natural_gas_kwh: Option<f64>,
    pub annual_generator_electricity_kwh: Option<f64>,
    pub annual_auxiliary_electricity_kwh: Option<f64>,
    pub annual_collective_source_heat_kwh: Option<f64>,
    pub annual_district_heat_kwh: Option<f64>,
    pub annual_biomass_kwh: Option<f64>,
    pub demand: MonthlyDemandAssessment,
    pub additional_zone_demands: Vec<MonthlyDemandAssessment>,
    pub issues: Vec<ChainIssue>,
}

fn issue(code: &'static str, path: impl Into<String>) -> ChainIssue {
    ChainIssue {
        code,
        path: path.into(),
    }
}

/// Validates one zone and returns its monthly (need, emission loss,
/// emission input, distribution loss) terms, or `None` when invalid.
fn zone_terms(
    demand_input: &MonthlyDemandInput,
    demand: &MonthlyDemandAssessment,
    emission: &EmissionInput,
    distribution: &Distribution,
    prefix: &str,
    issues: &mut Vec<ChainIssue>,
) -> Option<Vec<[f64; 4]>> {
    let prior = issues.len();
    issues.extend(
        demand
            .issues
            .iter()
            .map(|item| issue(item.code, format!("{prefix}demand.{}", item.path))),
    );
    if emission.source_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            format!("{prefix}emission.sourceReference"),
        ));
    }
    if !balancing_consistent(emission) {
        issues.push(issue(
            "emission_balancing_inconsistent",
            format!("{prefix}emission.balancing"),
        ));
    }
    if emission.system == EmissionSystem::FanAssistedRadiatorsOrConvectors {
        // 9.21 fan energy is required for this emitter and is not modelled.
        issues.push(issue(
            "emission_fan_energy_unsupported",
            format!("{prefix}emission.system"),
        ));
    }
    let losses = match distribution {
        Distribution::HeatedZoneOnlySpaceHeating { source_reference } => {
            if source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{prefix}distribution.sourceReference"),
                ));
            }
            vec![0.0; 12]
        }
        Distribution::Declared {
            monthly_loss_kwh,
            source_reference,
        } => {
            if source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{prefix}distribution.sourceReference"),
                ));
            }
            if monthly_loss_kwh.len() != 12
                || monthly_loss_kwh
                    .iter()
                    .any(|value| !value.is_finite() || *value < 0.0)
            {
                issues.push(issue(
                    "distribution_monthly_loss_invalid",
                    format!("{prefix}distribution.monthlyLossKwh"),
                ));
            }
            monthly_loss_kwh.clone()
        }
    };
    if issues.len() != prior {
        return None;
    }
    let increment = temperature_increment_k(emission);
    let setpoint = demand_input.setpoints.heating_c;
    Some(
        demand
            .monthly
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let need = row.heating.need_kwh;
                // 9.10 and 9.16.
                let loss =
                    monthly_loss_kwh(need, setpoint, OUTDOOR_TEMPERATURE_C[index], increment);
                // 9.24/9.25: no recoverable auxiliary energy; only in heating months.
                let distribution_loss = if need > 0.0 { losses[index] } else { 0.0 };
                // 9.9
                [need, loss, need + loss, distribution_loss]
            })
            .collect(),
    )
}

pub fn assess_space_heating_chain(input: &SpaceHeatingChainInput) -> SpaceHeatingChainAssessment {
    let fingerprint =
        input_fingerprint(&serde_json::to_value(input).expect("typed input serializes"));
    let mut issues: Vec<ChainIssue> = Vec::new();
    let demand = assess_monthly_demand(&input.demand);
    let primary = zone_terms(
        &input.demand,
        &demand,
        &input.emission,
        &input.distribution,
        "",
        &mut issues,
    );
    let additional_zone_demands: Vec<MonthlyDemandAssessment> = input
        .additional_zones
        .iter()
        .map(|zone| assess_monthly_demand(&zone.demand))
        .collect();
    let mut zones = vec![primary];
    for (index, (zone, assessed)) in input
        .additional_zones
        .iter()
        .zip(&additional_zone_demands)
        .enumerate()
    {
        zones.push(zone_terms(
            &zone.demand,
            assessed,
            &zone.emission,
            &zone.distribution,
            &format!("additionalZones[{index}]."),
            &mut issues,
        ));
    }
    let mut zone_ids = std::collections::HashSet::new();
    for (index, id) in std::iter::once(&input.demand.zone_id)
        .chain(
            input
                .additional_zones
                .iter()
                .map(|zone| &zone.demand.zone_id),
        )
        .enumerate()
    {
        if !zone_ids.insert(id.as_str()) {
            issues.push(issue("zone_id_duplicate", format!("zones[{index}].zoneId")));
        }
    }

    // A single increment is only meaningful for one zone.
    let increment =
        (input.additional_zones.is_empty()).then(|| temperature_increment_k(&input.emission));
    let mut monthly = Vec::with_capacity(12);
    let mut generation_efficiency = None;
    if issues.is_empty() {
        let mut outputs = Vec::with_capacity(12);
        for index in 0..12 {
            let mut totals = [0.0; 4];
            for zone in zones.iter().flatten() {
                for (total, value) in totals.iter_mut().zip(zone[index]) {
                    *total += value;
                }
            }
            let [need, emission_loss, emission_input, distribution_loss] = totals;
            // 9.2 node: generator output covers all zones, clamped at 0.
            let generator_output = (emission_input + distribution_loss).max(0.0);
            let month = index as u8 + 1;
            outputs.push(MonthlyEnergy {
                month,
                energy_kwh: generator_output,
            });
            monthly.push(ChainMonth {
                month,
                heating_need_kwh: need,
                emission_loss_kwh: emission_loss,
                emission_input_kwh: emission_input,
                distribution_loss_kwh: distribution_loss,
                generator_output_kwh: generator_output,
                heat_pump_output_kwh: 0.0,
                natural_gas_kwh: 0.0,
                district_heat_kwh: 0.0,
                biomass_kwh: 0.0,
                generator_electricity_kwh: 0.0,
                auxiliary_electricity_kwh: None,
                collective_source_heat_kwh: 0.0,
            });
        }
        match &input.generator {
            Generator::GasBoiler(generator) => {
                let result = assess_boiler_forfait_monthly_draft(&BoilerForfaitMonthlyDraftInput {
                    boiler: generator.boiler.clone(),
                    generator_output_kwh: outputs,
                    generator_output_reference: "derived by space_heating_chain".into(),
                });
                issues.extend(
                    result
                        .issues
                        .iter()
                        .map(|item| issue(item.code, format!("generator.boiler.{}", item.path))),
                );
                generation_efficiency = result.generation_efficiency;
                for (row, boiler) in monthly.iter_mut().zip(&result.monthly) {
                    row.natural_gas_kwh = boiler.input_natural_gas_kwh;
                    row.auxiliary_electricity_kwh = boiler.auxiliary_electricity_kwh;
                }
                if result.monthly.len() != 12 && issues.is_empty() {
                    issues.push(issue("generator_result_incomplete", "generator"));
                }
            }
            Generator::HeatPumpForfait(generator) => {
                let result =
                    assess_forfait_heat_pump_monthly_draft(&ForfaitHeatPumpMonthlyDraftInput {
                        forfait: generator.forfait.clone(),
                        generator_output_kwh: outputs,
                        generator_output_reference: "derived by space_heating_chain".into(),
                        source_system: generator.source_system,
                        source_system_reference: generator.source_system_reference.clone(),
                    });
                issues.extend(
                    result
                        .issues
                        .iter()
                        .map(|item| issue(item.code, format!("generator.{}", item.path))),
                );
                generation_efficiency = result.corrected_cop;
                for (row, pump) in monthly.iter_mut().zip(&result.monthly) {
                    row.generator_electricity_kwh = pump.generator_input_electricity_kwh;
                    row.collective_source_heat_kwh = pump.collective_source_heat_kwh;
                    row.heat_pump_output_kwh = pump.generator_output_kwh;
                }
                if let (Some(aux), true) = (
                    &generator.auxiliary_measurements,
                    result.monthly.len() == 12,
                ) {
                    if aux.generator_id != generator.forfait.generator_id {
                        issues.push(issue(
                            "heat_pump_auxiliary_generator_mismatch",
                            "generator.auxiliaryMeasurements.generatorId",
                        ));
                    } else if generator.forfait.collective_building_installation == Some(true) {
                        issues.push(issue(
                            "heat_pump_auxiliary_individual_only",
                            "generator.auxiliaryMeasurements",
                        ));
                    } else {
                        let assessed =
                            assess_heating_aux_measured_draft(&HeatingAuxMeasuredDraftInput {
                                generator_id: aux.generator_id.clone(),
                                generator_source_reference: aux.generator_source_reference.clone(),
                                measurements: aux.measurements.clone(),
                                input_energy_source_reference: format!(
                                    "heat_pump_monthly_sha256:{}",
                                    result.input_fingerprint
                                ),
                                months: result
                                    .monthly
                                    .iter()
                                    .map(|month| GeneratorElectricityMonth {
                                        month: month.month,
                                        generator_input_electricity_kwh: month
                                            .generator_input_electricity_kwh,
                                    })
                                    .collect(),
                            });
                        match assessed
                            .auxiliary
                            .as_ref()
                            .filter(|_| assessed.status != "invalid")
                        {
                            None => issues.extend(assessed.issues.iter().map(|item| {
                                issue(
                                    item.code,
                                    format!("generator.auxiliaryMeasurements.{}", item.path),
                                )
                            })),
                            Some(aux) => {
                                for row in monthly.iter_mut() {
                                    row.auxiliary_electricity_kwh = aux
                                        .monthly_auxiliary_electricity_kwh
                                        .iter()
                                        .find(|item| item.month == row.month)
                                        .map(|item| item.electricity_kwh);
                                }
                            }
                        }
                    }
                }
                if result.monthly.len() != 12 && issues.is_empty() {
                    issues.push(issue("generator_result_incomplete", "generator"));
                }
            }
            Generator::HybridHeatPump(generator) => {
                let result =
                    assess_hybrid_heat_pump_monthly_draft(&HybridHeatPumpMonthlyDraftInput {
                        dispatch: GeneratorDispatchDraftInput {
                            node_input_kwh: outputs,
                            node_input_reference: "derived by space_heating_chain".into(),
                            design_context: generator.design_context.clone(),
                            generators: generator.generators.clone(),
                        },
                        forfait: generator.forfait.clone(),
                        boiler: generator.boiler.clone(),
                        source_system: generator.source_system,
                        source_system_reference: generator.source_system_reference.clone(),
                        declared_operating_limits_present: generator
                            .declared_operating_limits_present,
                        heat_pump_auxiliary_measurements: generator
                            .heat_pump_auxiliary_measurements
                            .clone(),
                    });
                issues.extend(
                    result
                        .issues
                        .iter()
                        .map(|item| issue(item.code, format!("generator.{}", item.path))),
                );
                if let (Some(pump), Some(boiler)) = (&result.heat_pump, &result.boiler) {
                    generation_efficiency = pump.corrected_cop;
                    let pump_aux = result
                        .heat_pump_auxiliary
                        .as_ref()
                        .and_then(|aux| aux.auxiliary.as_ref())
                        .map(|aux| &aux.monthly_auxiliary_electricity_kwh);
                    for (index, row) in monthly.iter_mut().enumerate() {
                        let pump_month = &pump.monthly[index];
                        let boiler_month = &boiler.monthly[index];
                        row.generator_electricity_kwh = pump_month.generator_input_electricity_kwh;
                        row.collective_source_heat_kwh = pump_month.collective_source_heat_kwh;
                        row.heat_pump_output_kwh = pump_month.generator_output_kwh;
                        row.natural_gas_kwh = boiler_month.input_natural_gas_kwh;
                        let pump_aux_month = pump_aux
                            .and_then(|months| months.iter().find(|item| item.month == row.month))
                            .map_or(0.0, |item| item.electricity_kwh);
                        row.auxiliary_electricity_kwh = boiler_month
                            .auxiliary_electricity_kwh
                            .map(|value| value + pump_aux_month);
                    }
                } else if issues.is_empty() {
                    issues.push(issue("generator_result_incomplete", "generator"));
                }
            }
            Generator::ExternalHeat(generator) => {
                if generator.supplier_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        "generator.supplierReference",
                    ));
                }
                if generator.quality_declaration_present {
                    issues.push(issue(
                        "external_heat_declaration_unsupported",
                        "generator.qualityDeclarationPresent",
                    ));
                }
                // 9.84 with η = 1,0 and f_prac = 1 for the fixed factor 0,9.
                generation_efficiency = Some(1.0);
                for row in monthly.iter_mut() {
                    row.district_heat_kwh = row.generator_output_kwh;
                }
            }
            Generator::ElectricResistance(generator) => {
                if generator.equipment_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        "generator.equipmentReference",
                    ));
                }
                // Table 9.27: electric heating COP 1,0.
                generation_efficiency = Some(1.0);
                for row in monthly.iter_mut() {
                    row.generator_electricity_kwh = row.generator_output_kwh;
                }
            }
            Generator::Biomass(generator) => {
                for (value, field) in [
                    (&generator.annex_r_reference, "generator.annexRReference"),
                    (
                        &generator.equipment_reference,
                        "generator.equipmentReference",
                    ),
                ] {
                    if value.trim().is_empty() {
                        issues.push(issue("source_reference_required", field));
                    }
                }
                if !generator.annex_r_compliant_at_most_500_kw {
                    issues.push(issue(
                        "biomass_class_unsupported",
                        "generator.annexRCompliantAtMost500Kw",
                    ));
                }
                match biomass_efficiency(generator.appliance, generator.location) {
                    None => issues.push(issue(
                        "biomass_efficiency_unavailable",
                        "generator.location",
                    )),
                    Some(efficiency) => {
                        // 9.64 with f_prac = 1,0.
                        generation_efficiency = Some(efficiency);
                        for row in monthly.iter_mut() {
                            row.biomass_kwh = row.generator_output_kwh / efficiency;
                        }
                    }
                }
            }
        }
    }
    let valid = issues.is_empty();
    if !valid {
        monthly.clear();
    }
    let sum = |field: fn(&ChainMonth) -> f64| valid.then(|| monthly.iter().map(field).sum::<f64>());
    let auxiliary = if valid
        && monthly
            .iter()
            .all(|row| row.auxiliary_electricity_kwh.is_some())
    {
        Some(
            monthly
                .iter()
                .map(|row| row.auxiliary_electricity_kwh.unwrap_or(0.0))
                .sum(),
        )
    } else {
        None
    };
    SpaceHeatingChainAssessment {
        status: if valid {
            "calculated_unverified"
        } else {
            "invalid"
        },
        scope: "nta8800_space_heating_zones_single_generator_unverified",
        chapter_9_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        omitted_terms: OMITTED_TERMS,
        emission_temperature_increment_k: increment.filter(|_| valid),
        generation_efficiency: generation_efficiency.filter(|_| valid),
        annual_natural_gas_kwh: sum(|row| row.natural_gas_kwh),
        annual_generator_electricity_kwh: sum(|row| row.generator_electricity_kwh),
        annual_auxiliary_electricity_kwh: auxiliary,
        annual_collective_source_heat_kwh: sum(|row| row.collective_source_heat_kwh),
        annual_district_heat_kwh: sum(|row| row.district_heat_kwh),
        annual_biomass_kwh: sum(|row| row.biomass_kwh),
        monthly,
        demand,
        additional_zone_demands,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forfait_heat_pump_draft::{TableRowVariant, TableScope, TableSink, TableSource};
    use serde_json::json;

    fn demand() -> MonthlyDemandInput {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-monthly-demand-synthetic.json"
        ))
        .unwrap()
    }

    fn emission() -> EmissionInput {
        serde_json::from_value(json!({
            "system": "radiators_or_convectors", "balancing": "none_or_unknown",
            "control": "main_room_thermostat", "sourceReference": "installation survey"
        }))
        .unwrap()
    }

    fn boiler_chain() -> SpaceHeatingChainInput {
        SpaceHeatingChainInput {
            demand: demand(),
            emission: emission(),
            distribution: Distribution::HeatedZoneOnlySpaceHeating {
                source_reference: "pipes inside envelope".into(),
            },
            additional_zones: Vec::new(),
            generator: serde_json::from_value(json!({
                "kind": "gas_boiler",
                "boiler": {
                    "generatorId":"boiler", "role":"individual_main",
                    "location":"inside_thermal_boundary", "kind":"hr107", "fuel":"natural_gas",
                    "averageDesignEmissionTemperatureC":45.0, "emissionCircuit":"direct",
                    "equipmentReference":"type plate", "locationReference":"building plan",
                    "temperatureAndCircuitReference":"system design", "pilotFlamePresent":false,
                    "installationYear": 2020, "installationYearReference": "invoice"
                }
            }))
            .unwrap(),
        }
    }

    fn heat_pump() -> ForfaitHeatPumpDraftInput {
        ForfaitHeatPumpDraftInput {
            generator_id: "hp".into(),
            classification_source_reference: "system design".into(),
            scope: TableScope::ResidentialAtMost25Kw,
            source: TableSource::OutdoorAir,
            sink: TableSink::Hydronic,
            design_supply_temperature_c: Some(35.0),
            source_correction_factor: None,
            source_correction_reference: None,
            thermal_capacity_kw: Some(8.0),
            capacity_source_reference: Some("rated capacity".into()),
            collective_building_installation: Some(false),
            row_variant: TableRowVariant::Base,
            high_efficiency_evidence: None,
            source_temperature_c: None,
            source_temperature_evidence_reference: None,
            source_quality_declaration_reference: None,
        }
    }

    #[test]
    fn boiler_chain_carries_need_through_emission_to_gas() {
        let result = assess_space_heating_chain(&boiler_chain());
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        let need = result.demand.monthly[0].heating.need_kwh;
        let loss = need * (3.55_f64 / (23.55 - 2.61)).min(0.15);
        assert!((jan.emission_loss_kwh - loss).abs() < 1e-9);
        assert_eq!(jan.distribution_loss_kwh, 0.0);
        assert!((jan.generator_output_kwh - (need + loss)).abs() < 1e-9);
        let efficiency = result.generation_efficiency.unwrap();
        assert!((jan.natural_gas_kwh - (need + loss) / efficiency).abs() < 1e-9);
        assert!(result.annual_natural_gas_kwh.unwrap() > 0.0);
        assert_eq!(result.annual_generator_electricity_kwh, Some(0.0));
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn heat_pump_chain_uses_forfait_cop() {
        let mut input = boiler_chain();
        input.generator = Generator::HeatPumpForfait(HeatPumpGenerator {
            forfait: heat_pump(),
            source_system: SourceSystem::Individual,
            source_system_reference: "own outdoor unit".into(),
            auxiliary_measurements: None,
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let cop = result.generation_efficiency.unwrap();
        assert!(cop > 1.0);
        let jan = &result.monthly[0];
        assert!((jan.generator_electricity_kwh - jan.generator_output_kwh / cop).abs() < 1e-9);
        assert_eq!(result.annual_natural_gas_kwh, Some(0.0));
    }

    #[test]
    fn declared_distribution_loss_is_added_in_heating_months_only() {
        let mut input = boiler_chain();
        input.distribution = Distribution::Declared {
            monthly_loss_kwh: vec![10.0; 12],
            source_reference: "pipe calculation".into(),
        };
        let result = assess_space_heating_chain(&input);
        for row in &result.monthly {
            let expected = if row.heating_need_kwh > 0.0 {
                10.0
            } else {
                0.0
            };
            assert_eq!(row.distribution_loss_kwh, expected);
        }
    }

    #[test]
    fn rejects_fan_assisted_and_inconsistent_balancing_without_numbers() {
        let mut input = boiler_chain();
        input.emission.system = EmissionSystem::FanAssistedRadiatorsOrConvectors;
        let result = assess_space_heating_chain(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.monthly.is_empty());
        assert!(result.annual_natural_gas_kwh.is_none());
        input.emission.system = EmissionSystem::AirHeating;
        let result = assess_space_heating_chain(&input);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "emission_balancing_inconsistent"));
    }

    #[test]
    fn demand_errors_are_prefixed() {
        let mut input = boiler_chain();
        input.demand.usable_floor_area_m2 = 0.0;
        let result = assess_space_heating_chain(&input);
        assert!(result
            .issues
            .iter()
            .any(|item| item.path == "demand.usableFloorAreaM2"));
    }

    #[test]
    fn second_zone_adds_to_generator_output_with_its_own_emission() {
        let single = assess_space_heating_chain(&boiler_chain());
        let mut input = boiler_chain();
        let mut second = demand();
        second.zone_id = "rz-2".into();
        let mut floor = emission();
        floor.system = EmissionSystem::FloorHeating;
        input.additional_zones.push(ChainZone {
            demand: second,
            emission: floor,
            distribution: Distribution::HeatedZoneOnlySpaceHeating {
                source_reference: "inside".into(),
            },
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert_eq!(result.additional_zone_demands.len(), 1);
        assert!(result.emission_temperature_increment_k.is_none());
        let jan = &result.monthly[0];
        let need = single.monthly[0].heating_need_kwh;
        assert!((jan.heating_need_kwh - 2.0 * need).abs() < 1e-9);
        // Floor heating (3,5 K) is capped as well in January: 0,15 · need.
        assert!((jan.emission_loss_kwh - 2.0 * 0.15 * need).abs() < 1e-9);
        let duplicate = {
            let mut copy = input.clone();
            copy.additional_zones[0].demand.zone_id = "rz-1".into();
            assess_space_heating_chain(&copy)
        };
        assert!(duplicate
            .issues
            .iter()
            .any(|item| item.code == "zone_id_duplicate"));
        let mut broken = input;
        broken.additional_zones[0].emission.source_reference.clear();
        let result = assess_space_heating_chain(&broken);
        assert!(result
            .issues
            .iter()
            .any(|item| item.path == "additionalZones[0].emission.sourceReference"));
    }

    #[test]
    fn hybrid_generator_splits_output_between_heat_pump_and_boiler() {
        use crate::forfait_heat_pump_draft::assess_forfait_heat_pump_draft;
        let mut pump = heat_pump();
        pump.thermal_capacity_kw = Some(4.0);
        let cop = assess_forfait_heat_pump_draft(&pump).corrected_cop.unwrap();
        let mut input = boiler_chain();
        let Generator::GasBoiler(boiler) = &input.generator else {
            unreachable!()
        };
        let mut boiler = boiler.boiler.clone();
        boiler.role = crate::boiler_forfait_draft::BoilerRole::IndividualSupplementary;
        let boiler_efficiency = crate::boiler_forfait_draft::assess_boiler_forfait_draft(&boiler)
            .generation_efficiency
            .unwrap();
        input.generator = serde_json::from_value(json!({
            "kind": "hybrid_heat_pump",
            "designContext": "new_build",
            "generators": [
                {"id": "hp", "class": "heat_pump", "classificationReference": "design",
                 "nominalThermalPowerKw": 4.0, "powerReference": "plate",
                 "priorityEfficiency": cop, "efficiencyReference": "table"},
                {"id": "boiler", "class": "other_boiler", "classificationReference": "design",
                 "nominalThermalPowerKw": 6.0, "powerReference": "plate",
                 "priorityEfficiency": boiler_efficiency, "efficiencyReference": "table 9.25"}
            ],
            "forfait": pump,
            "boiler": boiler,
            "sourceSystem": "individual",
            "sourceSystemReference": "own unit"
        }))
        .unwrap();
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        assert!(
            jan.heat_pump_output_kwh > 0.0 && jan.heat_pump_output_kwh < jan.generator_output_kwh
        );
        let boiler_heat = jan.generator_output_kwh - jan.heat_pump_output_kwh;
        assert!((jan.natural_gas_kwh - boiler_heat / boiler_efficiency).abs() < 1e-6);
        assert!((jan.generator_electricity_kwh - jan.heat_pump_output_kwh / cop).abs() < 1e-6);
        assert_eq!(result.generation_efficiency, Some(cop));
        assert!(input.generator.heat_pump().is_some());
    }

    #[test]
    fn single_heat_pump_can_add_measured_auxiliary_energy() {
        use crate::heating_aux_draft::ElectricHeatPumpAuxMeasurements;
        let mut input = boiler_chain();
        input.generator = Generator::HeatPumpForfait(HeatPumpGenerator {
            forfait: heat_pump(),
            source_system: SourceSystem::Individual,
            source_system_reference: "own unit".into(),
            auxiliary_measurements: Some(HybridHeatPumpAuxMeasurements {
                generator_id: "hp".into(),
                generator_source_reference: "plate".into(),
                measurements: ElectricHeatPumpAuxMeasurements {
                    standby_electronics_w: 10.0,
                    delivery_pump_during_compressor_w: 200.0,
                    delivery_pump_pre_post_w: 90.0,
                    pump_pre_run_seconds: 300.0,
                    pump_post_run_seconds: 300.0,
                    average_compressor_on_seconds: 600.0,
                    mean_compressor_modulation: 0.5,
                    nominal_electric_drive_kw: 2.0,
                    measurement_source_reference: "measured".into(),
                    timing_source_reference: "measured".into(),
                },
            }),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert!(result.annual_auxiliary_electricity_kwh.unwrap() > 0.0);
        let Generator::HeatPumpForfait(generator) = &mut input.generator else {
            unreachable!()
        };
        generator
            .auxiliary_measurements
            .as_mut()
            .unwrap()
            .generator_id = "other".into();
        let result = assess_space_heating_chain(&input);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_auxiliary_generator_mismatch"));
    }

    #[test]
    fn external_heat_delivers_generator_output_as_dh() {
        let mut input = boiler_chain();
        input.generator = Generator::ExternalHeat(ExternalHeatGenerator {
            supplier_reference: "heat supply contract".into(),
            quality_declaration_present: false,
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        assert_eq!(jan.district_heat_kwh, jan.generator_output_kwh);
        assert_eq!(jan.natural_gas_kwh, 0.0);
        assert_eq!(result.generation_efficiency, Some(1.0));
        input.generator = Generator::ExternalHeat(ExternalHeatGenerator {
            supplier_reference: String::new(),
            quality_declaration_present: true,
        });
        let codes: Vec<_> = assess_space_heating_chain(&input)
            .issues
            .iter()
            .map(|item| item.code)
            .collect();
        assert!(codes.contains(&"external_heat_declaration_unsupported"));
        assert!(codes.contains(&"source_reference_required"));
    }

    #[test]
    fn electric_resistance_and_biomass_generators() {
        let mut input = boiler_chain();
        input.generator = Generator::ElectricResistance(ElectricResistanceGenerator {
            equipment_reference: "panel heaters".into(),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert_eq!(
            result.monthly[0].generator_electricity_kwh,
            result.monthly[0].generator_output_kwh
        );
        assert!(input.generator.heat_pump().is_none());

        input.generator = Generator::Biomass(BiomassGenerator {
            appliance: BiomassAppliance::CentralBoiler,
            location: BiomassLocation::OutsideThermalBoundary,
            annex_r_compliant_at_most_500_kw: true,
            annex_r_reference: "type test".into(),
            equipment_reference: "plate".into(),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        assert!((jan.biomass_kwh - jan.generator_output_kwh / 0.75).abs() < 1e-9);
        assert_eq!(
            biomass_efficiency(
                BiomassAppliance::PelletStove,
                BiomassLocation::InsideThermalBoundary
            ),
            Some(0.725)
        );
        assert_eq!(
            biomass_efficiency(
                BiomassAppliance::PelletStove,
                BiomassLocation::OutsideThermalBoundary
            ),
            None
        );

        input.generator = Generator::Biomass(BiomassGenerator {
            appliance: BiomassAppliance::PelletStove,
            location: BiomassLocation::OutsideThermalBoundary,
            annex_r_compliant_at_most_500_kw: false,
            annex_r_reference: "x".into(),
            equipment_reference: "x".into(),
        });
        let codes: Vec<_> = assess_space_heating_chain(&input)
            .issues
            .iter()
            .map(|item| item.code)
            .collect();
        assert!(codes.contains(&"biomass_class_unsupported"));
        assert!(codes.contains(&"biomass_efficiency_unavailable"));
    }
}
