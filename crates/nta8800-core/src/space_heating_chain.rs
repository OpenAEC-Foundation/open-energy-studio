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
use crate::heating_emission::{
    balancing_consistent, monthly_loss_kwh, temperature_increment_k, EmissionInput, EmissionSystem,
    DRAFT_SOURCE,
};
use crate::monthly_demand::{assess_monthly_demand, MonthlyDemandAssessment, MonthlyDemandInput};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};

pub const OMITTED_TERMS: &[&str] = &[
    "9.2.3 node losses and node gains (including solar thermal)",
    "9.2.5 recoverable system losses fed back to the zone",
    "9.21 emission fan energy for fan-assisted emitters",
    "heat pump auxiliary energy (source pump/fan, standby)",
    "multiple generators, hybrid operation and domestic hot water priority",
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Generator {
    GasBoiler(GasBoilerGenerator),
    HeatPumpForfait(HeatPumpGenerator),
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
    pub natural_gas_kwh: f64,
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
                natural_gas_kwh: 0.0,
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
                }
                if result.monthly.len() != 12 && issues.is_empty() {
                    issues.push(issue("generator_result_incomplete", "generator"));
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
}
