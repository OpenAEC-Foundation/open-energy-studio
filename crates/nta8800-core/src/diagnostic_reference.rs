//! Comparison harness for the isolated direct-transmission diagnostic.
//! A matching submitted value is not proof of independent provenance or NTA compliance.

use crate::direct_transmission::{assess_direct_transmission, DirectTransmissionInput};
use crate::reference::ReferenceSource;
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const METRICS: [&str; 4] = [
    "elementConductanceWPerK",
    "linearBridgeConductanceWPerK",
    "pointBridgeConductanceWPerK",
    "totalDirectConductanceWPerK",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DirectDiagnosticCase {
    pub case_id: String,
    pub norm_version: String,
    pub source: ReferenceSource,
    pub input: DirectTransmissionInput,
    pub expected: Vec<DiagnosticExpectedMetric>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiagnosticExpectedMetric {
    pub path: String,
    pub value: f64,
    pub unit: String,
    pub absolute_tolerance: f64,
    pub calculation_basis: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticComparison {
    pub case_id: String,
    pub status: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub case_fingerprint: String,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub all_metrics_within_tolerance: bool,
    pub metrics: Vec<MetricComparison>,
    pub issues: Vec<ComparisonIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricComparison {
    pub path: String,
    pub expected: f64,
    pub actual: f64,
    pub absolute_difference: f64,
    pub absolute_tolerance: f64,
    pub within_tolerance: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> ComparisonIssue {
    ComparisonIssue {
        code,
        path: path.into(),
    }
}

pub fn compare_direct_diagnostic(case: DirectDiagnosticCase) -> DiagnosticComparison {
    let case_fingerprint = input_fingerprint(
        &serde_json::to_value(&case).expect("diagnostic case serializes as JSON"),
    );
    let assessment = assess_direct_transmission(&case.input);
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
    let mut seen = HashSet::new();
    for (index, metric) in case.expected.iter().enumerate() {
        let path = format!("expected[{index}]");
        if !METRICS.contains(&metric.path.as_str()) || !seen.insert(metric.path.as_str()) {
            issues.push(issue("metric_path_invalid", format!("{path}.path")));
        }
        if metric.unit != "W/K" {
            issues.push(issue("metric_unit_invalid", format!("{path}.unit")));
        }
        if !metric.value.is_finite() {
            issues.push(issue("expected_value_invalid", format!("{path}.value")));
        }
        if !metric.absolute_tolerance.is_finite() || metric.absolute_tolerance < 0.0 {
            issues.push(issue(
                "metric_tolerance_invalid",
                format!("{path}.absoluteTolerance"),
            ));
        }
        if metric.calculation_basis.trim().is_empty() {
            issues.push(issue(
                "calculation_basis_required",
                format!("{path}.calculationBasis"),
            ));
        }
    }
    for metric in METRICS {
        if !seen.contains(metric) {
            issues.push(issue("metric_required", format!("expected.{metric}")));
        }
    }
    let mut comparisons = Vec::new();
    if issues.is_empty() {
        for metric in &case.expected {
            let actual = match metric.path.as_str() {
                "elementConductanceWPerK" => assessment.element_conductance_w_per_k,
                "linearBridgeConductanceWPerK" => assessment.linear_bridge_conductance_w_per_k,
                "pointBridgeConductanceWPerK" => assessment.point_bridge_conductance_w_per_k,
                "totalDirectConductanceWPerK" => assessment.total_direct_conductance_w_per_k,
                _ => None,
            }
            .expect("valid diagnostic and metric path have a value");
            let difference = (actual - metric.value).abs();
            if !difference.is_finite() {
                issues.push(issue(
                    "metric_difference_overflow",
                    format!("expected.{}", metric.path),
                ));
                continue;
            }
            comparisons.push(MetricComparison {
                path: metric.path.clone(),
                expected: metric.value,
                actual,
                absolute_difference: difference,
                absolute_tolerance: metric.absolute_tolerance,
                within_tolerance: difference <= metric.absolute_tolerance,
            });
        }
    }
    if !issues.is_empty() {
        comparisons.clear();
    }
    let passed = !comparisons.is_empty() && comparisons.iter().all(|m| m.within_tolerance);
    DiagnosticComparison {
        case_id: case.case_id,
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
        beng_calculation_available: false,
        all_metrics_within_tolerance: passed,
        metrics: comparisons,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn case() -> DirectDiagnosticCase {
        serde_json::from_value(json!({
            "caseId":"arithmetic-1", "normVersion":TARGET_NORM_VERSION,
            "source":{"publisher":"Independent lab", "documentId":"case-1",
                "edition":"2026", "usePermission":"internal", "independentReviewer":"Reviewer"},
            "input":{"elements":[{"id":"wall", "areaM2":10, "uValueWPerM2k":0.2,
                "sourceReference":"drawing-1"}],
                "linearBridges":[{"id":"edge", "lengthM":3, "psiWPerMk":0.05,
                    "sourceReference":"detail-1"}]},
            "expected":[
                {"path":"elementConductanceWPerK", "value":2, "unit":"W/K", "absoluteTolerance":0,
                    "calculationBasis":"Independent arithmetic"},
                {"path":"linearBridgeConductanceWPerK", "value":0.15, "unit":"W/K", "absoluteTolerance":1e-12,
                    "calculationBasis":"Independent arithmetic"},
                {"path":"pointBridgeConductanceWPerK", "value":0, "unit":"W/K", "absoluteTolerance":0,
                    "calculationBasis":"Independent arithmetic"},
                {"path":"totalDirectConductanceWPerK", "value":2.15, "unit":"W/K", "absoluteTolerance":1e-12,
                    "calculationBasis":"Independent arithmetic"}
            ]
        })).unwrap()
    }

    #[test]
    fn compares_all_four_terms_without_claiming_verification() {
        let result = compare_direct_diagnostic(case());
        assert_eq!(result.status, "compared_pass");
        assert_eq!(result.metrics.len(), 4);
        assert!(result.all_metrics_within_tolerance);
        assert!(!result.reference_verified);
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn catches_wrong_expected_value() {
        let mut case = case();
        case.expected[3].value = 3.0;
        let result = compare_direct_diagnostic(case);
        assert_eq!(result.status, "compared_fail");
        assert!(!result.all_metrics_within_tolerance);
        assert_eq!(
            result
                .metrics
                .iter()
                .filter(|m| !m.within_tolerance)
                .count(),
            1
        );
    }

    #[test]
    fn rejects_missing_metric_and_invalid_input_without_comparing() {
        let mut case = case();
        case.expected.pop();
        case.input.elements[0].u_value_w_per_m2k = 0.0;
        let result = compare_direct_diagnostic(case);
        assert_eq!(result.status, "invalid_case");
        assert!(result.metrics.is_empty());
        assert!(result.issues.iter().any(|i| i.code == "metric_required"));
        assert!(result.issues.iter().any(|i| i.code == "direct_u_invalid"));
    }

    #[test]
    fn case_fingerprint_tracks_expected_values_separately_from_input() {
        let baseline = compare_direct_diagnostic(case());
        let mut changed = case();
        changed.expected[3].value = 2.16;
        let result = compare_direct_diagnostic(changed);
        assert_eq!(result.input_fingerprint, baseline.input_fingerprint);
        assert_ne!(result.case_fingerprint, baseline.case_fingerprint);
        assert!(result.case_fingerprint.starts_with("sha256:"));
    }

    #[test]
    fn rejects_finite_expected_value_when_difference_overflows() {
        let mut candidate = case();
        candidate.input.elements[0].area_m2 = 1e308;
        candidate.input.elements[0].u_value_w_per_m2k = 1.0;
        candidate.expected[0].value = -1e308;
        candidate.expected[3].value = -1e308;
        let result = compare_direct_diagnostic(candidate);
        assert_eq!(result.status, "invalid_case");
        assert!(result.metrics.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "metric_difference_overflow"));
        assert!(!result.all_metrics_within_tolerance);
        assert!(serde_json::to_value(result).is_ok());
    }
}
