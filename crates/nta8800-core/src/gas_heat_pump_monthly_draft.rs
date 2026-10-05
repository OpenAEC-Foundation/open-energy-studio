//! Gas-driven heat-pump month terms from NTA 8800:2025+C1:2026 formula 9.62
//! (p. 331) and §9.6.8.1.1.2.3 (p. 362).
//! Carrier allocation, gas input and auxiliaries require a separate audit.

use crate::final_energy_draft::MonthlyEnergy;
use crate::forfait_heat_pump_monthly_draft::SourceSystem;
use crate::gas_heat_pump_forfait_draft::{
    assess_gas_heat_pump_forfait_draft, GasHeatPumpForfaitDraftInput, GasPumpSource,
};
use crate::{input_fingerprint, KERNEL_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;

const DRAFT_SOURCE: &str =
    "NTA 8800:2025+C1:2026, formule 9.62 (p. 331) en §9.6.8.1.1.2.3 (p. 362)";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasHeatPumpMonthlyDraftInput {
    pub forfait: GasHeatPumpForfaitDraftInput,
    pub generator_output_kwh: Vec<MonthlyEnergy>,
    pub generator_output_reference: String,
    pub source_system: SourceSystem,
    pub source_system_reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasHeatPumpMonthlyDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub corrected_cop: Option<f64>,
    pub collective_source_correction_factor: Option<f64>,
    pub monthly: Vec<GasHeatPumpMonthlyTerm>,
    pub generator_output_derived: bool,
    pub collective_source_heat_derived: bool,
    pub carrier_allocation_available: bool,
    pub gas_input_energy_available: bool,
    pub auxiliaries_included: bool,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<GasMonthlyIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasHeatPumpMonthlyTerm {
    pub month: u8,
    pub generator_output_kwh: f64,
    pub collective_source_heat_kwh: f64,
    pub uncorrected_input_term_kwh: f64,
    pub collective_source_correction_term_kwh: f64,
    pub equation_962_input_term_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasMonthlyIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> GasMonthlyIssue {
    GasMonthlyIssue {
        code,
        path: path.into(),
    }
}

pub fn assess_gas_heat_pump_monthly_draft(
    input: &GasHeatPumpMonthlyDraftInput,
) -> GasHeatPumpMonthlyDraftAssessment {
    let mut issues = Vec::new();
    let lookup = assess_gas_heat_pump_forfait_draft(&input.forfait);
    if lookup.status == "invalid" || lookup.corrected_cop.is_none() {
        issues.push(issue("forfait_lookup_invalid", "forfait"));
    }
    if input.generator_output_reference.trim().is_empty() {
        issues.push(issue("source_required", "generatorOutputReference"));
    }
    if input.source_system_reference.trim().is_empty() {
        issues.push(issue("source_required", "sourceSystemReference"));
    }
    let compatible = match input.source_system {
        SourceSystem::Individual => true,
        SourceSystem::CollectiveGround => input.forfait.source == GasPumpSource::Ground,
        SourceSystem::CollectiveGroundwaterSurfaceOrAtLeast15C => matches!(
            input.forfait.source,
            GasPumpSource::GroundwaterAquifer | GasPumpSource::SurfaceWater
        ),
    };
    if !compatible {
        issues.push(issue("source_system_mismatch", "sourceSystem"));
    }
    let mut months = [None; 12];
    for (index, item) in input.generator_output_kwh.iter().enumerate() {
        if !(1..=12).contains(&item.month) {
            issues.push(issue(
                "month_out_of_range",
                format!("generatorOutputKwh[{index}].month"),
            ));
            continue;
        }
        let slot = &mut months[usize::from(item.month - 1)];
        if slot.is_some() {
            issues.push(issue(
                "month_duplicate",
                format!("generatorOutputKwh[{index}].month"),
            ));
        } else if !item.energy_kwh.is_finite() || item.energy_kwh < 0.0 {
            issues.push(issue(
                "monthly_heat_invalid",
                format!("generatorOutputKwh[{index}].energyKwh"),
            ));
        } else {
            *slot = Some(item.energy_kwh);
        }
    }
    if months.iter().any(Option::is_none) {
        issues.push(issue("months_incomplete", "generatorOutputKwh"));
    }
    let collective = input.source_system != SourceSystem::Individual;
    let factor = match input.source_system {
        SourceSystem::Individual => 0.0,
        SourceSystem::CollectiveGround => 0.009,
        SourceSystem::CollectiveGroundwaterSurfaceOrAtLeast15C => 0.022,
    };
    let mut monthly = Vec::new();
    if issues.is_empty() {
        let cop = lookup.corrected_cop.expect("valid lookup checked");
        for (index, value) in months.iter().enumerate() {
            let output = value.expect("complete months checked");
            let uncorrected = output / cop;
            let source_heat = if collective {
                output * (1.0 - 1.0 / cop)
            } else {
                0.0
            };
            let correction = source_heat * factor;
            let term = uncorrected - correction;
            if !uncorrected.is_finite()
                || !source_heat.is_finite()
                || source_heat < 0.0
                || source_heat > output
                || !correction.is_finite()
                || !term.is_finite()
                || term < 0.0
            {
                issues.push(issue(
                    "input_term_invalid",
                    format!("generatorOutputKwh[{index}]"),
                ));
                break;
            }
            monthly.push(GasHeatPumpMonthlyTerm {
                month: (index + 1) as u8,
                generator_output_kwh: output,
                collective_source_heat_kwh: source_heat,
                uncorrected_input_term_kwh: uncorrected,
                collective_source_correction_term_kwh: correction,
                equation_962_input_term_kwh: term,
            });
        }
    }
    if !issues.is_empty() {
        monthly.clear();
    }
    GasHeatPumpMonthlyDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_equation_9_62_gas_pump_terms_without_carrier_allocation",
        draft_source: DRAFT_SOURCE,
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        corrected_cop: if issues.is_empty() {
            lookup.corrected_cop
        } else {
            None
        },
        collective_source_correction_factor: if issues.is_empty() {
            Some(factor)
        } else {
            None
        },
        monthly,
        generator_output_derived: false,
        collective_source_heat_derived: collective && issues.is_empty(),
        carrier_allocation_available: false,
        gas_input_energy_available: false,
        auxiliaries_included: false,
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

    fn sample(source: &str, system: SourceSystem) -> GasHeatPumpMonthlyDraftInput {
        serde_json::from_value(json!({
            "forfait":{
                "generatorId":"gwp-1", "drive":"absorption", "application":"utility",
                "applicationReference":"office schedule", "collectiveBuildingInstallation":false,
                "externalHeatSupply":false, "thermalCapacityKw":40.0, "capacityReference":"plate",
                "source":source, "sourceReference":"source plan",
                "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design"
            },
            "generatorOutputKwh":(1..=12).map(|month| json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
            "generatorOutputReference":"monthly heat ledger", "sourceSystem":system,
            "sourceSystemReference":"source design"
        })).unwrap()
    }

    #[test]
    fn individual_air_and_collective_water_keep_962_terms_without_gas_claim() {
        let individual =
            assess_gas_heat_pump_monthly_draft(&sample("outdoor_air", SourceSystem::Individual));
        assert_eq!(individual.status, "diagnostic_valid");
        assert_eq!(individual.corrected_cop, Some(1.6));
        assert!((individual.monthly[0].equation_962_input_term_kwh - 625.0).abs() < 1e-10);
        assert_eq!(individual.monthly[0].collective_source_heat_kwh, 0.0);
        assert!(!individual.gas_input_energy_available && !individual.carrier_allocation_available);
        let collective = assess_gas_heat_pump_monthly_draft(&sample(
            "groundwater_aquifer",
            SourceSystem::CollectiveGroundwaterSurfaceOrAtLeast15C,
        ));
        assert_eq!(collective.corrected_cop, Some(2.1));
        let heat = 1000.0 * (1.0 - 1.0 / 2.1);
        assert!((collective.monthly[0].collective_source_heat_kwh - heat).abs() < 1e-10);
        assert!(
            (collective.monthly[0].collective_source_correction_term_kwh - heat * 0.022).abs()
                < 1e-10
        );
        assert!(
            (collective.monthly[0].equation_962_input_term_kwh - (1000.0 / 2.1 - heat * 0.022))
                .abs()
                < 1e-10
        );
        let mut ground_input = sample("ground", SourceSystem::CollectiveGround);
        ground_input.forfait.drive = crate::gas_heat_pump_forfait_draft::GasPumpDrive::GasEngine;
        let ground = assess_gas_heat_pump_monthly_draft(&ground_input);
        assert_eq!(ground.status, "diagnostic_valid");
        assert_eq!(ground.collective_source_correction_factor, Some(0.009));
        assert!((ground.monthly[0].equation_962_input_term_kwh - 621.625).abs() < 1e-10);
    }

    #[test]
    fn mismatched_source_incomplete_months_and_overflow_emit_no_terms() {
        let mut input = sample("outdoor_air", SourceSystem::CollectiveGround);
        assert!(assess_gas_heat_pump_monthly_draft(&input)
            .monthly
            .is_empty());
        input.source_system = SourceSystem::Individual;
        input.generator_output_kwh.pop();
        assert!(assess_gas_heat_pump_monthly_draft(&input)
            .monthly
            .is_empty());
        input.generator_output_kwh.push(MonthlyEnergy {
            month: 12,
            energy_kwh: 1000.0,
        });
        input.forfait.application =
            crate::gas_heat_pump_forfait_draft::GasPumpApplication::ResidentialCollectiveAtMost25Kw;
        input.forfait.collective_building_installation = true;
        input.forfait.thermal_capacity_kw = 20.0;
        input.forfait.source = GasPumpSource::Ground;
        input.forfait.source_correction_factor = Some(0.1);
        input.forfait.source_correction_reference = Some("supplied source correction".into());
        input.generator_output_kwh[0].energy_kwh = f64::MAX;
        let invalid = assess_gas_heat_pump_monthly_draft(&input);
        assert_eq!(invalid.status, "invalid");
        assert!(invalid.monthly.is_empty() && invalid.corrected_cop.is_none());
    }
}
