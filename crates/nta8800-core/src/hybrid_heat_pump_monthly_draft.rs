//! One auditable Rust chain from supplied heating-node energy through draft
//! generator dispatch to draft electric heat-pump and gas-boiler input.

use crate::boiler_forfait_draft::{
    assess_boiler_forfait_draft, assess_boiler_forfait_monthly_draft, BoilerForfaitDraftInput,
    BoilerForfaitMonthlyDraftAssessment, BoilerForfaitMonthlyDraftInput, BoilerRole,
};
use crate::forfait_heat_pump_draft::{assess_forfait_heat_pump_draft, ForfaitHeatPumpDraftInput};
use crate::forfait_heat_pump_monthly_draft::{
    assess_forfait_heat_pump_monthly_draft, ForfaitHeatPumpMonthlyDraftAssessment,
    ForfaitHeatPumpMonthlyDraftInput, SourceSystem,
};
use crate::generator_dispatch_draft::{
    assess_generator_dispatch_draft, GeneratorClass, GeneratorDispatchDraftAssessment,
    GeneratorDispatchDraftInput,
};
use crate::heating_aux_draft::{
    assess_heating_aux_measured_draft, ElectricHeatPumpAuxMeasurements, GeneratorElectricityMonth,
    HeatingAuxMeasuredDraftAssessment, HeatingAuxMeasuredDraftInput,
};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HybridHeatPumpMonthlyDraftInput {
    pub dispatch: GeneratorDispatchDraftInput,
    pub forfait: ForfaitHeatPumpDraftInput,
    pub boiler: BoilerForfaitDraftInput,
    pub source_system: SourceSystem,
    pub source_system_reference: String,
    pub declared_operating_limits_present: bool,
    #[serde(default)]
    pub heat_pump_auxiliary_measurements: Option<HybridHeatPumpAuxMeasurements>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HybridHeatPumpAuxMeasurements {
    pub generator_id: String,
    pub generator_source_reference: String,
    pub measurements: ElectricHeatPumpAuxMeasurements,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HybridIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HybridHeatPumpMonthlyDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub kernel_version: &'static str,
    pub target_norm_version: &'static str,
    pub input_fingerprint: String,
    pub dispatch: Option<GeneratorDispatchDraftAssessment>,
    pub heat_pump: Option<ForfaitHeatPumpMonthlyDraftAssessment>,
    pub boiler: Option<BoilerForfaitMonthlyDraftAssessment>,
    pub heat_pump_auxiliary: Option<HeatingAuxMeasuredDraftAssessment>,
    pub generator_output_derived: bool,
    pub boiler_input_energy_available: bool,
    pub boiler_auxiliary_electricity_available: bool,
    pub heat_pump_auxiliary_electricity_available: bool,
    pub beng_calculation_available: bool,
    pub final_edition_verified: bool,
    pub issues: Vec<HybridIssue>,
}

fn issue(code: &'static str, path: impl Into<String>) -> HybridIssue {
    HybridIssue {
        code,
        path: path.into(),
    }
}

pub fn assess_hybrid_heat_pump_monthly_draft(
    input: &HybridHeatPumpMonthlyDraftInput,
) -> HybridHeatPumpMonthlyDraftAssessment {
    let mut issues = Vec::new();
    let lookup = assess_forfait_heat_pump_draft(&input.forfait);
    let boiler_lookup = assess_boiler_forfait_draft(&input.boiler);
    if lookup.status == "invalid" || lookup.corrected_cop.is_none() {
        issues.push(issue("forfait_lookup_invalid", "forfait"));
    }
    if input
        .forfait
        .design_supply_temperature_c
        .is_some_and(|temperature| temperature > 55.0)
    {
        issues.push(issue(
            "hybrid_above55_requires_annex_q",
            "forfait.designSupplyTemperatureC",
        ));
    }
    if input.declared_operating_limits_present {
        issues.push(issue(
            "declared_operating_limits_require_product_dispatch",
            "declaredOperatingLimitsPresent",
        ));
    }
    if let Some(aux) = &input.heat_pump_auxiliary_measurements {
        if aux.generator_id != input.forfait.generator_id {
            issues.push(issue(
                "heat_pump_auxiliary_generator_mismatch",
                "heatPumpAuxiliaryMeasurements.generatorId",
            ));
        }
        if input.forfait.collective_building_installation == Some(true) {
            issues.push(issue(
                "heat_pump_auxiliary_individual_only",
                "heatPumpAuxiliaryMeasurements",
            ));
        }
    }
    if boiler_lookup.status == "invalid" || boiler_lookup.generation_efficiency.is_none() {
        issues.push(issue("boiler_lookup_invalid", "boiler"));
    }
    if input.boiler.role != BoilerRole::IndividualSupplementary {
        issues.push(issue(
            "boiler_must_be_individual_supplementary",
            "boiler.role",
        ));
    }
    if input.dispatch.generators.len() != 2 {
        issues.push(issue(
            "hybrid_requires_two_generators",
            "dispatch.generators",
        ));
    }
    let boiler = input
        .dispatch
        .generators
        .iter()
        .find(|generator| generator.id == input.boiler.generator_id);
    match boiler {
        None => issues.push(issue("boiler_generator_missing", "dispatch.generators")),
        Some(generator) => {
            if generator.class != GeneratorClass::OtherBoiler {
                issues.push(issue(
                    "boiler_generator_class_invalid",
                    "dispatch.generators",
                ));
            }
            if let Some(eta) = boiler_lookup.generation_efficiency {
                if (generator.priority_efficiency - eta).abs() > 1e-9 * eta.max(1.0) {
                    issues.push(issue("boiler_efficiency_mismatch", "dispatch.generators"));
                }
            }
        }
    }
    let pump = input
        .dispatch
        .generators
        .iter()
        .find(|generator| generator.id == input.forfait.generator_id);
    match pump {
        None => issues.push(issue("heat_pump_generator_missing", "dispatch.generators")),
        Some(generator) => {
            if !matches!(
                generator.class,
                GeneratorClass::HeatPump | GeneratorClass::ExhaustAirHeatPumpWithoutOverventilation
            ) {
                issues.push(issue(
                    "heat_pump_generator_class_invalid",
                    "dispatch.generators",
                ));
            }
            if let Some(capacity) = input.forfait.thermal_capacity_kw {
                if (generator.nominal_thermal_power_kw - capacity).abs() > 1e-9 * capacity.max(1.0)
                {
                    issues.push(issue(
                        "heat_pump_capacity_mismatch",
                        "forfait.thermalCapacityKw",
                    ));
                }
            } else {
                issues.push(issue(
                    "heat_pump_capacity_required",
                    "forfait.thermalCapacityKw",
                ));
            }
            if let Some(cop) = lookup.corrected_cop {
                if (generator.priority_efficiency - cop).abs() > 1e-9 * cop.max(1.0) {
                    issues.push(issue(
                        "heat_pump_efficiency_mismatch",
                        "dispatch.generators",
                    ));
                }
            }
        }
    }
    let dispatch = assess_generator_dispatch_draft(&input.dispatch);
    if dispatch.status == "invalid" {
        issues.push(issue("dispatch_invalid", "dispatch"));
    }
    let mut heat_pump = None;
    let mut boiler_result = None;
    let mut heat_pump_auxiliary = None;
    if issues.is_empty() {
        let output = dispatch
            .monthly
            .iter()
            .map(|month| {
                let delivered = month
                    .generators
                    .iter()
                    .find(|generator| generator.generator_id == input.forfait.generator_id)
                    .expect("generator exists in valid dispatch")
                    .delivered_heat_kwh;
                crate::final_energy_draft::MonthlyEnergy {
                    month: month.month,
                    energy_kwh: delivered,
                }
            })
            .collect();
        let monthly_input = ForfaitHeatPumpMonthlyDraftInput {
            forfait: input.forfait.clone(),
            generator_output_kwh: output,
            generator_output_reference: format!("dispatch_sha256:{}", dispatch.input_fingerprint),
            source_system: input.source_system,
            source_system_reference: input.source_system_reference.clone(),
        };
        let assessed = assess_forfait_heat_pump_monthly_draft(&monthly_input);
        if assessed.status == "invalid" {
            issues.push(issue("heat_pump_monthly_invalid", "heatPump"));
        } else {
            heat_pump = Some(assessed);
        }
        if let (Some(aux), Some(pump_result)) =
            (&input.heat_pump_auxiliary_measurements, &heat_pump)
        {
            let months = pump_result
                .monthly
                .iter()
                .map(|month| GeneratorElectricityMonth {
                    month: month.month,
                    generator_input_electricity_kwh: month.generator_input_electricity_kwh,
                })
                .collect();
            let aux_input = HeatingAuxMeasuredDraftInput {
                generator_id: aux.generator_id.clone(),
                generator_source_reference: aux.generator_source_reference.clone(),
                measurements: aux.measurements.clone(),
                input_energy_source_reference: format!(
                    "heat_pump_monthly_sha256:{}",
                    pump_result.input_fingerprint
                ),
                months,
            };
            let assessed_aux = assess_heating_aux_measured_draft(&aux_input);
            if assessed_aux.status == "invalid" {
                issues.push(issue(
                    "heat_pump_auxiliary_invalid",
                    "heatPumpAuxiliaryMeasurements",
                ));
            } else {
                heat_pump_auxiliary = Some(assessed_aux);
            }
        }
        let boiler_output = dispatch
            .monthly
            .iter()
            .map(|month| {
                let delivered = month
                    .generators
                    .iter()
                    .find(|generator| generator.generator_id == input.boiler.generator_id)
                    .expect("boiler exists in valid two-generator dispatch")
                    .delivered_heat_kwh;
                crate::final_energy_draft::MonthlyEnergy {
                    month: month.month,
                    energy_kwh: delivered,
                }
            })
            .collect();
        let boiler_monthly_input = BoilerForfaitMonthlyDraftInput {
            boiler: input.boiler.clone(),
            generator_output_kwh: boiler_output,
            generator_output_reference: format!("dispatch_sha256:{}", dispatch.input_fingerprint),
        };
        let assessed_boiler = assess_boiler_forfait_monthly_draft(&boiler_monthly_input);
        if assessed_boiler.status == "invalid" {
            issues.push(issue("boiler_monthly_invalid", "boiler"));
        } else {
            boiler_result = Some(assessed_boiler);
        }
    }
    if !issues.is_empty() {
        heat_pump = None;
        boiler_result = None;
        heat_pump_auxiliary = None;
    }
    HybridHeatPumpMonthlyDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_new_build_dispatch_electric_heat_pump_gas_boiler_and_optional_measured_aux",
        kernel_version: KERNEL_VERSION,
        target_norm_version: TARGET_NORM_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        dispatch: if issues.is_empty() {
            Some(dispatch)
        } else {
            None
        },
        heat_pump,
        boiler: boiler_result,
        heat_pump_auxiliary,
        generator_output_derived: issues.is_empty(),
        boiler_input_energy_available: issues.is_empty(),
        boiler_auxiliary_electricity_available: issues.is_empty(),
        heat_pump_auxiliary_electricity_available: issues.is_empty() && input.heat_pump_auxiliary_measurements.is_some(),
        beng_calculation_available: false,
        final_edition_verified: false,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hybrid_chain_derives_both_generator_inputs_and_rejects_wrong_table_context() {
        let input: HybridHeatPumpMonthlyDraftInput = serde_json::from_value(json!({
            "dispatch": {
                "nodeInputKwh": (1..=12).map(|month| json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
                "nodeInputReference":"node balance", "designContext":"new_build",
                "generators":[
                    {"id":"hp", "class":"heat_pump", "classificationReference":"design schedule", "nominalThermalPowerKw":4.0,
                     "powerReference":"plate", "priorityEfficiency":4.8, "efficiencyReference":"table 9.27"},
                    {"id":"boiler", "class":"other_boiler", "classificationReference":"design schedule", "nominalThermalPowerKw":6.0,
                     "powerReference":"plate", "priorityEfficiency":0.95, "efficiencyReference":"table 9.25"}
                ]
            },
            "forfait": {"generatorId":"hp", "classificationSourceReference":"source design",
                "scope":"residential_at_most25_kw", "source":"collective15_to20_c",
                "sink":"hydronic", "designSupplyTemperatureC":35.0,
                "thermalCapacityKw":4.0,"capacitySourceReference":"plate",
                "collectiveBuildingInstallation":false,
                "sourceCorrectionFactor":null,"sourceCorrectionReference":null,
                "sourceTemperatureC":15.0,"sourceTemperatureEvidenceReference":"source meter"},
            "boiler": {"generatorId":"boiler", "role":"individual_supplementary",
                "location":"inside_thermal_boundary", "kind":"hr107", "fuel":"natural_gas",
                "averageDesignEmissionTemperatureC":45.0, "emissionCircuit":"direct",
                "equipmentReference":"type plate", "locationReference":"building plan",
                "temperatureAndCircuitReference":"system design", "pilotFlamePresent":false},
            "sourceSystem":"collective_groundwater_surface_or_at_least15_c",
            "sourceSystemReference":"source design",
            "declaredOperatingLimitsPresent":false
        })).unwrap();
        let result = assess_hybrid_heat_pump_monthly_draft(&input);
        assert_eq!(result.status, "diagnostic_valid", "{}", json!(result));
        let heat_pump = result.heat_pump.unwrap();
        assert!((heat_pump.monthly[0].generator_output_kwh - 750.0).abs() < 1e-9);
        assert!(
            (heat_pump.monthly[0].generator_input_electricity_kwh
                - (750.0 / 4.8 - 750.0 * (1.0 - 1.0 / 4.8) * 0.022))
                .abs()
                < 1e-9
        );
        assert!(result.boiler_input_energy_available);
        assert!(result.boiler_auxiliary_electricity_available);
        assert!(
            (result.boiler.unwrap().monthly[0].input_natural_gas_kwh - 250.0 / 0.95).abs() < 1e-9
        );
        let mut individual = input.clone();
        individual.heat_pump_auxiliary_measurements = Some(HybridHeatPumpAuxMeasurements {
            generator_id: "hp".into(),
            generator_source_reference: "heat-pump plate".into(),
            measurements: ElectricHeatPumpAuxMeasurements {
                standby_electronics_w: 10.0,
                delivery_pump_during_compressor_w: 200.0,
                delivery_pump_pre_post_w: 90.0,
                pump_pre_run_seconds: 300.0,
                pump_post_run_seconds: 300.0,
                average_compressor_on_seconds: 600.0,
                mean_compressor_modulation: 0.5,
                nominal_electric_drive_kw: 2.0,
                measurement_source_reference: "measured powers".into(),
                timing_source_reference: "measured cycle".into(),
            },
        });
        let linked = assess_hybrid_heat_pump_monthly_draft(&individual);
        assert_eq!(linked.status, "diagnostic_valid", "{}", json!(linked));
        assert!(linked.heat_pump_auxiliary_electricity_available);
        let hp_electricity =
            linked.heat_pump.as_ref().unwrap().monthly[0].generator_input_electricity_kwh;
        let aux_electricity = linked
            .heat_pump_auxiliary
            .as_ref()
            .unwrap()
            .auxiliary
            .as_ref()
            .unwrap()
            .monthly_auxiliary_electricity_kwh[0]
            .electricity_kwh;
        assert!(
            (aux_electricity - (87.6 / 12.0 + 0.19 * hp_electricity / (0.5 * 2.0))).abs() < 1e-9
        );
        individual
            .heat_pump_auxiliary_measurements
            .as_mut()
            .unwrap()
            .generator_id = "other".into();
        let rejected_aux = assess_hybrid_heat_pump_monthly_draft(&individual);
        assert_eq!(rejected_aux.status, "invalid");
        assert!(rejected_aux.dispatch.is_none() && rejected_aux.heat_pump_auxiliary.is_none());
        let mut high_temperature = input;
        high_temperature.forfait.design_supply_temperature_c = Some(56.0);
        let rejected = assess_hybrid_heat_pump_monthly_draft(&high_temperature);
        assert_eq!(rejected.status, "invalid");
        assert!(rejected
            .issues
            .iter()
            .any(|item| item.code == "hybrid_above55_requires_annex_q"));
        assert!(rejected.heat_pump.is_none());
    }
}
