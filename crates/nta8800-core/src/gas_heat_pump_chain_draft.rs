//! Coupled consultation diagnostics for gas heat-pump 9.62 and 9.91/9.92 terms.
//! Supplied monthly heat must match exactly across both inputs; no carrier allocation.

use crate::gas_heat_pump_aux_draft::{assess_gas_heat_pump_aux_draft, GasHeatPumpAuxDraftInput};
use crate::gas_heat_pump_monthly_draft::{
    assess_gas_heat_pump_monthly_draft, GasHeatPumpMonthlyDraftInput,
};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasHeatPumpChainDraftInput {
    pub monthly: GasHeatPumpMonthlyDraftInput,
    pub auxiliary: GasHeatPumpAuxDraftInput,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasHeatPumpChainDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub monthly: Vec<GasHeatPumpChainMonth>,
    pub annual_equipment_auxiliary_electricity_kwh: Option<f64>,
    pub generator_output_derived: bool,
    pub carrier_allocation_available: bool,
    pub gas_input_energy_available: bool,
    pub source_pump_or_fan_included: bool,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<ChainIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasHeatPumpChainMonth {
    pub month: u8,
    pub supplied_generator_output_kwh: f64,
    pub collective_source_heat_kwh: f64,
    pub equation_962_unallocated_input_term_kwh: f64,
    pub capped_auxiliary_on_hours: f64,
    pub equipment_auxiliary_electricity_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> ChainIssue {
    ChainIssue {
        code,
        path: path.into(),
    }
}

pub fn assess_gas_heat_pump_chain_draft(
    input: &GasHeatPumpChainDraftInput,
) -> GasHeatPumpChainDraftAssessment {
    let mut issues = Vec::new();
    let forfait = &input.monthly.forfait;
    let auxiliary = &input.auxiliary;
    if forfait.generator_id != auxiliary.generator_id || forfait.drive != auxiliary.drive {
        issues.push(issue("generator_mismatch", "auxiliary.generatorId"));
    }
    if forfait.thermal_capacity_kw != auxiliary.nominal_thermal_capacity_kw {
        issues.push(issue(
            "capacity_mismatch",
            "auxiliary.nominalThermalCapacityKw",
        ));
    }
    if !auxiliary.forfait_cop_used {
        issues.push(issue("forfait_basis_required", "auxiliary.forfaitCopUsed"));
    }
    if input.monthly.generator_output_reference != auxiliary.generator_output_reference {
        issues.push(issue(
            "generator_output_reference_mismatch",
            "auxiliary.generatorOutputReference",
        ));
    }
    let monthly_assessment = assess_gas_heat_pump_monthly_draft(&input.monthly);
    if monthly_assessment.status == "invalid" {
        issues.push(issue("monthly_terms_invalid", "monthly"));
    }
    let auxiliary_assessment = assess_gas_heat_pump_aux_draft(auxiliary);
    if auxiliary_assessment.status == "invalid" {
        issues.push(issue("auxiliary_invalid", "auxiliary"));
    }
    if monthly_assessment.status == "diagnostic_valid"
        && auxiliary_assessment.status == "diagnostic_valid"
    {
        for term in &monthly_assessment.monthly {
            let (position, aux) = auxiliary
                .months
                .iter()
                .enumerate()
                .find(|(_, item)| item.month == term.month)
                .expect("twelve unique valid auxiliary months");
            if aux.generator_output_kwh != term.generator_output_kwh {
                issues.push(issue(
                    "generator_output_mismatch",
                    format!("auxiliary.months[{position}].generatorOutputKwh"),
                ));
            }
        }
    }
    let mut monthly = Vec::new();
    if issues.is_empty() {
        for term in &monthly_assessment.monthly {
            let aux = auxiliary_assessment
                .monthly
                .iter()
                .find(|item| item.month == term.month)
                .expect("twelve unique valid auxiliary months");
            monthly.push(GasHeatPumpChainMonth {
                month: term.month,
                supplied_generator_output_kwh: term.generator_output_kwh,
                collective_source_heat_kwh: term.collective_source_heat_kwh,
                equation_962_unallocated_input_term_kwh: term.equation_962_input_term_kwh,
                capped_auxiliary_on_hours: aux.capped_on_hours,
                equipment_auxiliary_electricity_kwh: aux.auxiliary_electricity_kwh,
            });
        }
    }
    GasHeatPumpChainDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_gas_9_62_and_9_91_linked_terms_only",
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        monthly,
        annual_equipment_auxiliary_electricity_kwh: if issues.is_empty() {
            auxiliary_assessment.annual_auxiliary_electricity_kwh
        } else {
            None
        },
        generator_output_derived: false,
        carrier_allocation_available: false,
        gas_input_energy_available: false,
        source_pump_or_fan_included: false,
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> GasHeatPumpChainDraftInput {
        serde_json::from_value(json!({
            "monthly":{
                "forfait":{
                    "generatorId":"ga-1", "drive":"absorption", "application":"utility",
                    "applicationReference":"office design", "collectiveBuildingInstallation":false,
                    "externalHeatSupply":false, "thermalCapacityKw":20.0, "capacityReference":"plate",
                    "source":"outdoor_air", "sourceReference":"source plan",
                    "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design"
                },
                "generatorOutputKwh":(1..=12).map(|month| json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
                "generatorOutputReference":"heat ledger", "sourceSystem":"individual",
                "sourceSystemReference":"source plan"
            },
            "auxiliary":{
                "generatorId":"ga-1", "drive":"absorption", "nominalThermalCapacityKw":20.0,
                "capacityReference":"plate", "standbyElectronicsW":10.0,
                "burnerAuxiliaryWPerKw":1.0, "solutionPumpWPerKw":0.0,
                "coefficientsReference":"draft 9.6.8.2.3", "meanModulation":1.0,
                "modulationReference":"draft 9.6.8.2.3", "buildingShare":1.0,
                "buildingShareReference":"whole building", "forfaitCopUsed":true,
                "monthHoursReference":"hour schedule", "generatorOutputReference":"heat ledger",
                "months":(1..=12).map(|month| json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0})).collect::<Vec<_>>()
            }
        })).unwrap()
    }

    #[test]
    fn linked_terms_preserve_distinct_energy_roles() {
        let result = assess_gas_heat_pump_chain_draft(&sample());
        assert_eq!(result.status, "diagnostic_valid");
        assert_eq!(
            result.monthly[0].equation_962_unallocated_input_term_kwh,
            625.0
        );
        assert_eq!(result.monthly[0].equipment_auxiliary_electricity_kwh, 8.4);
        assert!((result.annual_equipment_auxiliary_electricity_kwh.unwrap() - 100.8).abs() < 1e-10);
        assert!(!result.carrier_allocation_available && !result.gas_input_energy_available);
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn mismatched_heat_capacity_and_reference_suppress_both_terms() {
        let mut input = sample();
        input.auxiliary.months[0].generator_output_kwh = 900.0;
        let mismatch = assess_gas_heat_pump_chain_draft(&input);
        assert_eq!(mismatch.status, "invalid");
        assert!(
            mismatch.monthly.is_empty()
                && mismatch
                    .annual_equipment_auxiliary_electricity_kwh
                    .is_none()
        );
        assert!(mismatch
            .issues
            .iter()
            .any(|item| item.code == "generator_output_mismatch"));
        input.auxiliary.months[0].generator_output_kwh = 1000.0;
        input.auxiliary.nominal_thermal_capacity_kw = 21.0;
        input.auxiliary.generator_output_reference = "another ledger".into();
        let mismatch = assess_gas_heat_pump_chain_draft(&input);
        assert!(mismatch
            .issues
            .iter()
            .any(|item| item.code == "capacity_mismatch"));
        assert!(mismatch
            .issues
            .iter()
            .any(|item| item.code == "generator_output_reference_mismatch"));
        assert!(mismatch.monthly.is_empty());
    }
}
