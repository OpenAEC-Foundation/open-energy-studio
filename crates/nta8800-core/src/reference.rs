//! Administrative checks for independent reference cases.
//! Completeness of a manifest never proves that its expected values are correct.

use crate::{assess_json, input_fingerprint, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceCase {
    pub case_id: String,
    pub norm_version: String,
    pub project: Value,
    pub source: ReferenceSource,
    pub expected: Vec<ExpectedMetric>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSource {
    pub publisher: String,
    pub document_id: String,
    pub edition: String,
    pub use_permission: String,
    pub independent_reviewer: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedMetric {
    pub path: String,
    pub value: f64,
    pub unit: String,
    pub norm_reference: String,
    pub absolute_tolerance: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceAudit {
    pub case_id: String,
    pub target_norm_version: &'static str,
    pub manifest_fingerprint: String,
    pub manifest_complete: bool,
    pub reference_verified: bool,
    pub calculation_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_fingerprint: Option<String>,
    pub issues: Vec<ReferenceIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferenceIssue {
    pub code: &'static str,
    pub path: String,
    pub message: &'static str,
}

fn issue(code: &'static str, path: impl Into<String>, message: &'static str) -> ReferenceIssue {
    ReferenceIssue {
        code,
        path: path.into(),
        message,
    }
}

pub fn audit_reference_case(case: ReferenceCase) -> ReferenceAudit {
    let manifest_fingerprint =
        input_fingerprint(&serde_json::to_value(&case).expect("reference case serializes as JSON"));
    let mut issues = Vec::new();
    if case.case_id.trim().is_empty() {
        issues.push(issue(
            "case_id_required",
            "caseId",
            "Reference case ID is required",
        ));
    }
    if case.norm_version != TARGET_NORM_VERSION {
        issues.push(issue(
            "norm_version_mismatch",
            "normVersion",
            "Reference case targets a different NTA edition",
        ));
    }
    for (field, value) in [
        ("publisher", &case.source.publisher),
        ("documentId", &case.source.document_id),
        ("edition", &case.source.edition),
        ("usePermission", &case.source.use_permission),
        ("independentReviewer", &case.source.independent_reviewer),
    ] {
        if value.trim().is_empty() {
            issues.push(issue(
                "source_field_required",
                format!("source.{field}"),
                "Reference source provenance is required",
            ));
        }
    }
    if case.expected.is_empty() {
        issues.push(issue(
            "expected_metrics_required",
            "expected",
            "Independent expected values are required",
        ));
    }
    let mut metric_paths = HashSet::new();
    for (index, metric) in case.expected.iter().enumerate() {
        let path = format!("expected[{index}]");
        if metric.path.trim().is_empty() || !metric_paths.insert(metric.path.as_str()) {
            issues.push(issue(
                "metric_path_invalid",
                format!("{path}.path"),
                "Metric paths must be nonempty and unique",
            ));
        }
        if !metric.value.is_finite() {
            issues.push(issue(
                "expected_value_invalid",
                format!("{path}.value"),
                "Expected value must be finite",
            ));
        }
        if metric.unit.trim().is_empty() {
            issues.push(issue(
                "metric_unit_required",
                format!("{path}.unit"),
                "Metric unit is required",
            ));
        }
        if metric.norm_reference.trim().is_empty() {
            issues.push(issue(
                "metric_norm_reference_required",
                format!("{path}.normReference"),
                "Metric requires an exact norm reference",
            ));
        }
        if !metric.absolute_tolerance.is_finite() || metric.absolute_tolerance < 0.0 {
            issues.push(issue(
                "metric_tolerance_invalid",
                format!("{path}.absoluteTolerance"),
                "Absolute tolerance must be finite and nonnegative",
            ));
        }
    }
    let input_fingerprint = match assess_json(case.project) {
        Ok(assessment) => {
            if assessment.status == "invalid" {
                issues.push(issue(
                    "project_input_invalid",
                    "project",
                    "Reference project fails structural validation",
                ));
            }
            Some(assessment.input_fingerprint)
        }
        Err(_) => {
            issues.push(issue(
                "project_shape_invalid",
                "project",
                "Reference project shape cannot be parsed",
            ));
            None
        }
    };
    ReferenceAudit {
        case_id: case.case_id,
        target_norm_version: TARGET_NORM_VERSION,
        manifest_fingerprint,
        manifest_complete: issues.is_empty(),
        reference_verified: false,
        calculation_available: false,
        input_fingerprint,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn project() -> Value {
        json!({"id":"p", "name":"Reference input", "buildingFunction":"residential",
            "zones":[{"id":"z", "floorArea":100, "volume":250, "surfaces":[{
                "id":"s", "area":50, "zoneId":"z", "windows":[]}]}]})
    }

    #[test]
    fn refuses_diagnostic_input_without_expected_values_or_provenance() {
        let case: ReferenceCase = serde_json::from_value(json!({
            "caseId":"diagnostic", "normVersion":TARGET_NORM_VERSION,
            "project":project(), "source":{"publisher":"","documentId":"",
                "edition":"","usePermission":"","independentReviewer":""},
            "expected":[]
        }))
        .unwrap();
        let audit = audit_reference_case(case);
        assert!(!audit.manifest_complete);
        assert!(!audit.reference_verified);
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.code == "expected_metrics_required"));
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.code == "source_field_required"));
    }

    #[test]
    fn complete_fields_do_not_claim_independent_verification() {
        let case: ReferenceCase = serde_json::from_value(json!({
            "caseId":"review-candidate", "normVersion":TARGET_NORM_VERSION,
            "project":project(), "source":{"publisher":"ISSO","documentId":"case-1",
                "edition":"2026","usePermission":"internal review","independentReviewer":"Reviewer"},
            "expected":[{"path":"beng1","value":42.0,"unit":"kWh/m2.year",
                "normReference":"NTA 8800:2025+C1:2026 §placeholder",
                "absoluteTolerance":0.1}]
        })).unwrap();
        let audit = audit_reference_case(case);
        assert!(audit.manifest_complete);
        assert!(!audit.reference_verified);
        assert!(!audit.calculation_available);
        assert!(audit.input_fingerprint.unwrap().starts_with("sha256:"));
    }

    #[test]
    fn detects_duplicate_metric_and_invalid_tolerance() {
        let case: ReferenceCase = serde_json::from_value(json!({
            "caseId":"x", "normVersion":TARGET_NORM_VERSION,
            "project":project(), "source":{"publisher":"ISSO","documentId":"x",
                "edition":"2026","usePermission":"internal","independentReviewer":"R"},
            "expected":[
                {"path":"beng1","value":42,"unit":"kWh","normReference":"§1","absoluteTolerance":0},
                {"path":"beng1","value":43,"unit":"kWh","normReference":"§1","absoluteTolerance":-1}
            ]
        }))
        .unwrap();
        let audit = audit_reference_case(case);
        assert!(!audit.manifest_complete);
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.code == "metric_path_invalid"));
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.code == "metric_tolerance_invalid"));
    }

    #[test]
    fn manifest_fingerprint_changes_with_expected_values_and_source() {
        let raw = json!({
            "caseId":"fingerprint-case", "normVersion":TARGET_NORM_VERSION,
            "project":project(), "source":{"publisher":"ISSO","documentId":"case-1",
                "edition":"2026","usePermission":"internal review","independentReviewer":"Reviewer"},
            "expected":[{"path":"beng1","value":42.0,"unit":"kWh/m2.year",
                "normReference":"test source", "absoluteTolerance":0.1}]
        });
        let base: ReferenceCase = serde_json::from_value(raw.clone()).unwrap();
        let project_fingerprint = audit_reference_case(base.clone()).input_fingerprint;
        let manifest_fingerprint = audit_reference_case(base).manifest_fingerprint;
        let mut expected_changed = raw.clone();
        expected_changed["expected"][0]["value"] = json!(43.0);
        let changed = audit_reference_case(serde_json::from_value(expected_changed).unwrap());
        assert_eq!(changed.input_fingerprint, project_fingerprint);
        assert_ne!(changed.manifest_fingerprint, manifest_fingerprint);
        let mut source_changed = raw;
        source_changed["source"]["documentId"] = json!("case-2");
        let changed = audit_reference_case(serde_json::from_value(source_changed).unwrap());
        assert_eq!(changed.input_fingerprint, project_fingerprint);
        assert_ne!(changed.manifest_fingerprint, manifest_fingerprint);
    }
}
