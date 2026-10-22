//! Comparison harness for a complete twelve-month gas heat-pump draft chain.
//! Submitted expectations and matching values cannot establish independent provenance.

use crate::forfait_heat_pump_monthly_draft::SourceSystem;
use crate::gas_heat_pump_chain_draft::{
    assess_gas_heat_pump_chain_draft, GasHeatPumpChainDraftInput,
};
use crate::reference::ReferenceSource;
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasChainDiagnosticCase {
    pub case_id: String,
    pub norm_version: String,
    pub source: ReferenceSource,
    pub input: GasHeatPumpChainDraftInput,
    pub expected: Vec<GasChainExpectedMetric>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GasChainMetric {
    Equation962UnallocatedInputTerm,
    CollectiveSourceHeat,
    EquipmentAuxiliaryElectricity,
    AnnualEquipmentAuxiliaryElectricity,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasChainExpectedMetric {
    pub metric: GasChainMetric,
    pub month: Option<u8>,
    pub value_kwh: f64,
    pub absolute_tolerance_kwh: f64,
    pub calculation_basis: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasChainDiagnosticComparison {
    pub case_id: String,
    pub source: ReferenceSource,
    pub status: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub case_fingerprint: String,
    pub reference_verified: bool,
    pub carrier_allocation_available: bool,
    pub beng_calculation_available: bool,
    pub all_metrics_within_tolerance: bool,
    pub metrics: Vec<GasChainMetricComparison>,
    pub issues: Vec<GasChainComparisonIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasChainMetricComparison {
    pub metric: GasChainMetric,
    pub month: Option<u8>,
    pub calculation_basis: String,
    pub expected_kwh: f64,
    pub actual_kwh: f64,
    pub absolute_difference_kwh: f64,
    pub absolute_tolerance_kwh: f64,
    pub within_tolerance: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasChainComparisonIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> GasChainComparisonIssue {
    GasChainComparisonIssue {
        code,
        path: path.into(),
    }
}

pub fn compare_gas_heat_pump_chain_diagnostic(
    case: GasChainDiagnosticCase,
) -> GasChainDiagnosticComparison {
    let case_fingerprint =
        input_fingerprint(&serde_json::to_value(&case).expect("case serializes"));
    let assessment = assess_gas_heat_pump_chain_draft(&case.input);
    let mut issues = Vec::new();
    if case.case_id.trim().is_empty() {
        issues.push(issue("case_id_required", "caseId"));
    }
    if case.norm_version != TARGET_NORM_VERSION {
        issues.push(issue("norm_version_mismatch", "normVersion"));
    }
    for (field, value) in [
        ("publisher", &case.source.publisher),
        ("documentId", &case.source.document_id),
        ("edition", &case.source.edition),
        ("usePermission", &case.source.use_permission),
        ("independentReviewer", &case.source.independent_reviewer),
    ] {
        if value.trim().is_empty() {
            issues.push(issue("source_field_required", format!("source.{field}")));
        }
    }
    if assessment.status == "invalid" {
        for problem in &assessment.issues {
            issues.push(issue(problem.code, format!("input.{}", problem.path)));
        }
    }
    let collective = case.input.monthly.source_system != SourceSystem::Individual;
    let mut seen = HashSet::new();
    for (index, metric) in case.expected.iter().enumerate() {
        let path = format!("expected[{index}]");
        let valid_month = match metric.metric {
            GasChainMetric::AnnualEquipmentAuxiliaryElectricity => metric.month.is_none(),
            _ => metric.month.is_some_and(|month| (1..=12).contains(&month)),
        };
        if !valid_month {
            issues.push(issue("metric_month_invalid", format!("{path}.month")));
        } else if !seen.insert((metric.metric, metric.month)) {
            issues.push(issue("metric_duplicate", path.clone()));
        }
        if metric.metric == GasChainMetric::CollectiveSourceHeat && !collective {
            issues.push(issue("metric_not_applicable", path.clone()));
        }
        if !metric.value_kwh.is_finite() {
            issues.push(issue("expected_value_invalid", format!("{path}.valueKwh")));
        }
        if !metric.absolute_tolerance_kwh.is_finite() || metric.absolute_tolerance_kwh < 0.0 {
            issues.push(issue(
                "metric_tolerance_invalid",
                format!("{path}.absoluteToleranceKwh"),
            ));
        }
        if metric.calculation_basis.trim().is_empty() {
            issues.push(issue(
                "calculation_basis_required",
                format!("{path}.calculationBasis"),
            ));
        }
    }
    for month in 1..=12 {
        let required = [
            Some(GasChainMetric::Equation962UnallocatedInputTerm),
            Some(GasChainMetric::EquipmentAuxiliaryElectricity),
            collective.then_some(GasChainMetric::CollectiveSourceHeat),
        ];
        for metric in required.into_iter().flatten() {
            if !seen.contains(&(metric, Some(month))) {
                issues.push(issue(
                    "metric_required",
                    format!("expected.{metric:?}[{month}]"),
                ));
            }
        }
    }
    if !seen.contains(&(GasChainMetric::AnnualEquipmentAuxiliaryElectricity, None)) {
        issues.push(issue(
            "metric_required",
            "expected.annualEquipmentAuxiliaryElectricity",
        ));
    }
    let mut metrics = Vec::new();
    if issues.is_empty() {
        for (index, expected) in case.expected.iter().enumerate() {
            let actual = match expected.metric {
                GasChainMetric::AnnualEquipmentAuxiliaryElectricity => assessment
                    .annual_equipment_auxiliary_electricity_kwh
                    .expect("valid chain has annual electricity"),
                GasChainMetric::Equation962UnallocatedInputTerm => {
                    assessment.monthly[usize::from(expected.month.expect("validated month") - 1)]
                        .equation_962_unallocated_input_term_kwh
                }
                GasChainMetric::CollectiveSourceHeat => {
                    assessment.monthly[usize::from(expected.month.expect("validated month") - 1)]
                        .collective_source_heat_kwh
                }
                GasChainMetric::EquipmentAuxiliaryElectricity => {
                    assessment.monthly[usize::from(expected.month.expect("validated month") - 1)]
                        .equipment_auxiliary_electricity_kwh
                }
            };
            let difference = (actual - expected.value_kwh).abs();
            if !difference.is_finite() {
                issues.push(issue(
                    "metric_difference_overflow",
                    format!("expected[{index}]"),
                ));
                continue;
            }
            metrics.push(GasChainMetricComparison {
                metric: expected.metric,
                month: expected.month,
                calculation_basis: expected.calculation_basis.clone(),
                expected_kwh: expected.value_kwh,
                actual_kwh: actual,
                absolute_difference_kwh: difference,
                absolute_tolerance_kwh: expected.absolute_tolerance_kwh,
                within_tolerance: difference <= expected.absolute_tolerance_kwh,
            });
        }
    }
    if !issues.is_empty() {
        metrics.clear();
    }
    let required_count = if collective { 37 } else { 25 };
    let passed =
        metrics.len() == required_count && metrics.iter().all(|item| item.within_tolerance);
    GasChainDiagnosticComparison {
        case_id: case.case_id,
        source: case.source,
        status: if !issues.is_empty() {
            "invalid_case"
        } else if passed {
            "compared_pass"
        } else {
            "compared_fail"
        },
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: assessment.input_fingerprint,
        case_fingerprint,
        reference_verified: false,
        carrier_allocation_available: false,
        beng_calculation_available: false,
        all_metrics_within_tolerance: passed,
        metrics,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> GasChainDiagnosticCase {
        let mut expected = Vec::new();
        for month in 1..=12 {
            for (metric, value_kwh) in [
                (GasChainMetric::Equation962UnallocatedInputTerm, 625.0),
                (
                    GasChainMetric::EquipmentAuxiliaryElectricity,
                    crate::climate::MONTH_HOURS[(month - 1) as usize] * 0.01 + 1.1,
                ),
            ] {
                expected.push(GasChainExpectedMetric {
                    metric,
                    month: Some(month),
                    value_kwh,
                    absolute_tolerance_kwh: 1e-9,
                    calculation_basis: "synthetic independent arithmetic".into(),
                });
            }
        }
        expected.push(GasChainExpectedMetric {
            metric: GasChainMetric::AnnualEquipmentAuxiliaryElectricity,
            month: None,
            value_kwh: 100.8,
            absolute_tolerance_kwh: 1e-9,
            calculation_basis: "synthetic independent arithmetic".into(),
        });
        GasChainDiagnosticCase {
            case_id: "synthetic-gas-chain".into(),
            norm_version: TARGET_NORM_VERSION.into(),
            source: ReferenceSource {
                publisher: "internal test".into(),
                document_id: "synthetic hand calculation".into(),
                edition: "draft 2026".into(),
                use_permission: "internal test".into(),
                independent_reviewer: "unverified".into(),
            },
            input: serde_json::from_value(json!({
                "monthly":{
                    "forfait":{"generatorId":"ga-1","drive":"absorption","application":"utility",
                        "applicationReference":"office","collectiveBuildingInstallation":false,
                        "externalHeatSupply":false,"thermalCapacityKw":20.0,"capacityReference":"plate",
                        "source":"outdoor_air","sourceReference":"plan","designSupplyTemperatureC":35.0,
                        "designSupplyReference":"design"},
                    "generatorOutputKwh":(1..=12).map(|month|json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
                    "generatorOutputReference":"meter","sourceSystem":"individual","sourceSystemReference":"plan"
                },
                "auxiliary":{"generatorId":"ga-1","drive":"absorption","nominalThermalCapacityKw":20.0,
                    "capacityReference":"plate","standbyElectronicsW":10.0,"burnerAuxiliaryWPerKw":1.0,
                    "solutionPumpWPerKw":0.0,"coefficientsReference":"draft","meanModulation":1.0,
                    "modulationReference":"draft","buildingShare":1.0,"buildingShareReference":"whole building",
                    "forfaitCopUsed":true,"monthHoursReference":"schedule","generatorOutputReference":"meter",
                    "months":(1..=12).map(|month|json!({"month":month,"hours":crate::climate::MONTH_HOURS[(month - 1) as usize],"generatorOutputKwh":1000.0})).collect::<Vec<_>>()}
            })).unwrap(),
            expected,
        }
    }

    #[test]
    fn complete_synthetic_values_compare_without_claiming_verification() {
        let mut case = sample();
        case.expected[0].calculation_basis = "case-specific synthetic arithmetic".into();
        let result = compare_gas_heat_pump_chain_diagnostic(case);
        assert_eq!(result.status, "compared_pass");
        assert_eq!(result.metrics.len(), 25);
        assert_eq!(result.source.document_id, "synthetic hand calculation");
        assert_eq!(
            result.metrics[0].calculation_basis,
            "case-specific synthetic arithmetic"
        );
        let response = serde_json::to_value(&result).unwrap();
        assert_eq!(
            response["metrics"][0]["calculationBasis"],
            "case-specific synthetic arithmetic"
        );
        assert_eq!(
            response["source"]["documentId"],
            "synthetic hand calculation"
        );
        assert!(result.all_metrics_within_tolerance);
        assert!(!result.reference_verified && !result.carrier_allocation_available);
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn collective_case_requires_and_compares_all_twelve_source_heat_terms() {
        let mut case = sample();
        case.input.monthly.forfait.source =
            crate::gas_heat_pump_forfait_draft::GasPumpSource::GroundwaterAquifer;
        case.input.monthly.source_system = SourceSystem::CollectiveGroundwaterSurfaceOrAtLeast15C;
        let source_heat = 1000.0 * (1.0 - 1.0 / 2.1);
        let input_term = 1000.0 / 2.1 - source_heat * 0.022;
        for metric in &mut case.expected {
            if metric.metric == GasChainMetric::Equation962UnallocatedInputTerm {
                metric.value_kwh = input_term;
            }
        }
        let missing = compare_gas_heat_pump_chain_diagnostic(case.clone());
        assert_eq!(missing.status, "invalid_case");
        assert!(missing.metrics.is_empty());
        assert_eq!(
            missing
                .issues
                .iter()
                .filter(|item| item.code == "metric_required")
                .count(),
            12
        );
        for month in 1..=12 {
            case.expected.push(GasChainExpectedMetric {
                metric: GasChainMetric::CollectiveSourceHeat,
                month: Some(month),
                value_kwh: source_heat,
                absolute_tolerance_kwh: 1e-9,
                calculation_basis: "independent synthetic source-heat arithmetic".into(),
            });
        }
        let passed = compare_gas_heat_pump_chain_diagnostic(case.clone());
        assert_eq!(passed.status, "compared_pass");
        assert_eq!(passed.metrics.len(), 37);
        assert_eq!(
            passed.metrics.last().unwrap().calculation_basis,
            "independent synthetic source-heat arithmetic"
        );
        assert!(!passed.reference_verified && !passed.carrier_allocation_available);
        case.expected.last_mut().unwrap().value_kwh += 1.0;
        let failed = compare_gas_heat_pump_chain_diagnostic(case);
        assert_eq!(failed.status, "compared_fail");
        assert_eq!(
            failed
                .metrics
                .iter()
                .filter(|item| !item.within_tolerance)
                .count(),
            1
        );
    }

    #[test]
    fn individual_case_rejects_collective_source_heat_expectation() {
        let mut case = sample();
        case.expected.push(GasChainExpectedMetric {
            metric: GasChainMetric::CollectiveSourceHeat,
            month: Some(1),
            value_kwh: 0.0,
            absolute_tolerance_kwh: 1e-9,
            calculation_basis: "synthetic".into(),
        });
        let result = compare_gas_heat_pump_chain_diagnostic(case);
        assert_eq!(result.status, "invalid_case");
        assert!(result.metrics.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "metric_not_applicable"));
    }

    #[test]
    fn published_repository_fixture_is_explicitly_synthetic_and_complete() {
        let raw =
            include_str!("../../../training-data/nta8800-gas-chain-diagnostic-synthetic.json");
        let case: GasChainDiagnosticCase = serde_json::from_str(raw).unwrap();
        assert!(case.source.document_id.contains("not ISSO 54"));
        let result = compare_gas_heat_pump_chain_diagnostic(case);
        assert_eq!(result.status, "compared_pass");
        assert_eq!(result.metrics.len(), 25);
        assert!(!result.reference_verified);

        let raw = include_str!(
            "../../../training-data/nta8800-gas-chain-diagnostic-collective-synthetic.json"
        );
        let case: GasChainDiagnosticCase = serde_json::from_str(raw).unwrap();
        assert!(case.source.document_id.contains("not ISSO 54"));
        let result = compare_gas_heat_pump_chain_diagnostic(case);
        assert_eq!(result.status, "compared_pass", "{:?}", result.issues);
        assert_eq!(result.metrics.len(), 37);
        assert!(!result.reference_verified);
    }

    #[test]
    fn changed_expectation_fails_and_changes_case_not_input_fingerprint() {
        let baseline = compare_gas_heat_pump_chain_diagnostic(sample());
        let mut changed = sample();
        changed.expected[0].value_kwh = 626.0;
        let result = compare_gas_heat_pump_chain_diagnostic(changed);
        assert_eq!(result.status, "compared_fail");
        assert_eq!(result.input_fingerprint, baseline.input_fingerprint);
        assert_ne!(result.case_fingerprint, baseline.case_fingerprint);
        assert_eq!(
            result
                .metrics
                .iter()
                .filter(|item| !item.within_tolerance)
                .count(),
            1
        );
    }

    #[test]
    fn changed_reference_metadata_changes_case_fingerprint_not_calculation() {
        let baseline = compare_gas_heat_pump_chain_diagnostic(sample());
        let mut source_change = sample();
        source_change.source.document_id = "different submitted document".into();
        let source_result = compare_gas_heat_pump_chain_diagnostic(source_change);
        assert_eq!(source_result.status, "compared_pass");
        assert_eq!(source_result.input_fingerprint, baseline.input_fingerprint);
        assert_ne!(source_result.case_fingerprint, baseline.case_fingerprint);
        assert_eq!(
            source_result.source.document_id,
            "different submitted document"
        );
        assert!(!source_result.reference_verified);

        let mut basis_change = sample();
        basis_change.expected[0].calculation_basis = "different submitted method".into();
        let basis_result = compare_gas_heat_pump_chain_diagnostic(basis_change);
        assert_eq!(basis_result.status, "compared_pass");
        assert_eq!(basis_result.input_fingerprint, baseline.input_fingerprint);
        assert_ne!(basis_result.case_fingerprint, baseline.case_fingerprint);
        assert_eq!(
            basis_result.metrics[0].calculation_basis,
            "different submitted method"
        );
        assert!(!basis_result.reference_verified);
    }

    #[test]
    fn missing_duplicate_or_invalid_input_never_emit_partial_comparisons() {
        let mut case = sample();
        case.expected.pop();
        case.expected.push(case.expected[0].clone());
        case.input.auxiliary.months[0].generator_output_kwh = 900.0;
        let result = compare_gas_heat_pump_chain_diagnostic(case);
        assert_eq!(result.status, "invalid_case");
        assert!(result.metrics.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "metric_required"));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "metric_duplicate"));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "generator_output_mismatch"));
    }

    #[test]
    fn finite_values_with_overflowed_difference_invalidate_case() {
        let mut case = sample();
        case.expected[0].value_kwh = -f64::MAX;
        case.input.monthly.generator_output_kwh[0].energy_kwh = 1e308;
        case.input.auxiliary.months[0].generator_output_kwh = 1e308;
        let result = compare_gas_heat_pump_chain_diagnostic(case);
        assert_eq!(result.status, "invalid_case");
        assert!(result.metrics.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "metric_difference_overflow"));
    }
}
