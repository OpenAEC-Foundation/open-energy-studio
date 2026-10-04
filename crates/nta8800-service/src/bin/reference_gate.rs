//! Batch numeric comparison for independently supplied NTA reference manifests.
//! A green exit code only means the supplied numbers fit the supplied tolerances.

use nta8800_core::reference::{compare_reference_case, ReferenceCase, ReferenceComparison};
use nta8800_core::{KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GateCase {
    file: String,
    comparison: ReferenceComparison,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GateError {
    file: String,
    error: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GateReport {
    target_norm_version: &'static str,
    kernel_version: &'static str,
    numeric_comparison_passed: bool,
    /// None means that no independently agreed coverage plan was supplied.
    planned_coverage_passed: Option<bool>,
    /// SHA-256 of the exact plan bytes, for release-level traceability only.
    coverage_plan_fingerprint: Option<String>,
    reference_verified: bool,
    attest_status: &'static str,
    cases: Vec<GateCase>,
    errors: Vec<GateError>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CoveragePlan {
    target_norm_version: String,
    required_cases: Vec<RequiredCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RequiredCase {
    case_id: String,
    manifest_fingerprint: String,
    required_paths: Vec<String>,
}

fn check_coverage(plan: CoveragePlan, report: &mut GateReport) -> bool {
    let mut passed = true;
    if plan.target_norm_version != TARGET_NORM_VERSION {
        report.errors.push(GateError {
            file: String::new(),
            error: format!("Coverage plan targetNormVersion must be {TARGET_NORM_VERSION}"),
        });
        passed = false;
    }
    if plan.required_cases.is_empty() {
        report.errors.push(GateError {
            file: String::new(),
            error: "Coverage plan requires at least one case".into(),
        });
        passed = false;
    }
    let mut ids = HashSet::new();
    for required in &plan.required_cases {
        if required.case_id.trim().is_empty() || !ids.insert(required.case_id.as_str()) {
            report.errors.push(GateError {
                file: String::new(),
                error: format!("Blank or duplicate planned caseId: {}", required.case_id),
            });
            passed = false;
        }
        if required.manifest_fingerprint.len() != 71
            || !required.manifest_fingerprint.starts_with("sha256:")
            || !required.manifest_fingerprint[7..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            report.errors.push(GateError {
                file: String::new(),
                error: format!(
                    "Invalid planned manifestFingerprint for caseId: {}",
                    required.case_id
                ),
            });
            passed = false;
        }
        let mut paths = HashSet::new();
        if required.required_paths.is_empty() {
            report.errors.push(GateError {
                file: String::new(),
                error: format!("No requiredPaths for caseId: {}", required.case_id),
            });
            passed = false;
        }
        for path in &required.required_paths {
            if path.trim().is_empty() || !paths.insert(path.as_str()) {
                report.errors.push(GateError {
                    file: String::new(),
                    error: format!(
                        "Blank or duplicate required path for caseId: {}",
                        required.case_id
                    ),
                });
                passed = false;
            }
        }
        match report
            .cases
            .iter()
            .find(|case| case.comparison.case_id == required.case_id)
        {
            None => {
                report.errors.push(GateError {
                    file: String::new(),
                    error: format!("Missing planned caseId: {}", required.case_id),
                });
                passed = false;
            }
            Some(case) => {
                if case.comparison.manifest_fingerprint != required.manifest_fingerprint {
                    report.errors.push(GateError {
                        file: case.file.clone(),
                        error: format!(
                            "Manifest fingerprint differs for caseId: {}",
                            required.case_id
                        ),
                    });
                    passed = false;
                }
                let compared: HashSet<_> = case
                    .comparison
                    .metrics
                    .iter()
                    .map(|metric| metric.path.as_str())
                    .collect();
                for path in &required.required_paths {
                    if !compared.contains(path.as_str()) {
                        report.errors.push(GateError {
                            file: case.file.clone(),
                            error: format!(
                                "Missing compared path for caseId {}: {path}",
                                required.case_id
                            ),
                        });
                        passed = false;
                    }
                }
            }
        }
    }
    for case in &report.cases {
        if !ids.contains(case.comparison.case_id.as_str()) {
            report.errors.push(GateError {
                file: case.file.clone(),
                error: format!("Unplanned caseId: {}", case.comparison.case_id),
            });
            passed = false;
        }
    }
    passed
}

fn run(paths: &[PathBuf], plan_path: Option<&Path>) -> GateReport {
    let mut report = GateReport {
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        numeric_comparison_passed: !paths.is_empty(),
        planned_coverage_passed: None,
        coverage_plan_fingerprint: None,
        reference_verified: false,
        attest_status: "unattested",
        cases: Vec::new(),
        errors: Vec::new(),
    };
    let mut case_ids = HashSet::new();
    if paths.is_empty() {
        report.errors.push(GateError {
            file: String::new(),
            error: "At least one reference-case JSON file is required".into(),
        });
    }
    for path in paths {
        let file = path.display().to_string();
        let case = fs::read_to_string(path)
            .map_err(|error| error.to_string())
            .and_then(|content| {
                serde_json::from_str::<ReferenceCase>(&content).map_err(|error| error.to_string())
            });
        match case {
            Err(error) => {
                report.numeric_comparison_passed = false;
                report.errors.push(GateError { file, error });
            }
            Ok(case) if !case_ids.insert(case.case_id.clone()) => {
                report.numeric_comparison_passed = false;
                report.errors.push(GateError {
                    file,
                    error: format!("Duplicate caseId: {}", case.case_id),
                });
            }
            Ok(case) => {
                let comparison = compare_reference_case(case);
                report.numeric_comparison_passed &= comparison.status == "compared_pass";
                report.cases.push(GateCase { file, comparison });
            }
        }
    }
    if let Some(path) = plan_path {
        let plan = fs::read(path)
            .map_err(|error| error.to_string())
            .and_then(|content| {
                report.coverage_plan_fingerprint =
                    Some(format!("sha256:{:x}", Sha256::digest(&content)));
                serde_json::from_slice::<CoveragePlan>(&content).map_err(|error| error.to_string())
            });
        report.planned_coverage_passed = Some(match plan {
            Ok(plan) => check_coverage(plan, &mut report),
            Err(error) => {
                report.errors.push(GateError {
                    file: path.display().to_string(),
                    error,
                });
                false
            }
        });
    }
    report
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let mut plan_path = None;
    let mut paths = Vec::new();
    while let Some(arg) = args.next() {
        if arg == "--plan" {
            if plan_path.is_some() {
                eprintln!("--plan can only be supplied once");
                return ExitCode::from(2);
            }
            let Some(path) = args.next() else {
                eprintln!("--plan requires a JSON file path");
                return ExitCode::from(2);
            };
            plan_path = Some(PathBuf::from(path));
        } else {
            paths.push(PathBuf::from(arg));
        }
    }
    let report = run(&paths, plan_path.as_deref());
    let success =
        report.numeric_comparison_passed && report.planned_coverage_passed.unwrap_or(true);
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("Could not serialize reference comparison report: {error}");
            return ExitCode::from(2);
        }
    }
    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_case_file(case: &ReferenceCase) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "oes-reference-gate-{}-{nonce}.json",
            std::process::id()
        ));
        fs::write(&path, serde_json::to_vec(case).unwrap()).unwrap();
        path
    }

    fn case(value: f64) -> ReferenceCase {
        let project: Value = serde_json::from_str(include_str!(
            "../../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        serde_json::from_value(json!({
            "caseId":"synthetic-gate", "normVersion":TARGET_NORM_VERSION,
            "project":project,
            "source":{"publisher":"synthetic", "documentId":"internal-1", "edition":"test",
                "usePermission":"internal", "independentReviewer":"test"},
            "expected":[{"path":"beng2", "value":value, "unit":"kWh/m2.year",
                "normReference":"internal test", "absoluteTolerance":0.0}]
        }))
        .unwrap()
    }

    #[test]
    fn no_cases_cannot_pass() {
        let report = run(&[], None);
        assert!(!report.numeric_comparison_passed);
        assert!(!report.reference_verified);
        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn comparison_status_never_grants_reference_verification() {
        let valid = case(8.17);
        let comparison = compare_reference_case(valid);
        assert_eq!(comparison.status, "compared_pass");
        assert!(!comparison.reference_verified);
        assert_eq!(comparison.attest_status, "unattested");
        let mismatch = compare_reference_case(case(9.17));
        assert_eq!(mismatch.status, "compared_fail");
    }

    #[test]
    fn batch_requires_unique_cases_and_all_numeric_comparisons_to_pass() {
        let passing = temporary_case_file(&case(8.17));
        let failing = temporary_case_file(&case(9.17));
        let pass = run(std::slice::from_ref(&passing), None);
        assert!(pass.numeric_comparison_passed);
        assert_eq!(pass.cases.len(), 1);
        assert!(!pass.reference_verified);
        let duplicate = run(&[passing.clone(), failing.clone()], None);
        assert!(!duplicate.numeric_comparison_passed);
        assert_eq!(duplicate.errors.len(), 1);
        assert!(duplicate.errors[0].error.contains("Duplicate caseId"));
        let fail = run(std::slice::from_ref(&failing), None);
        assert!(!fail.numeric_comparison_passed);
        assert_eq!(fail.cases[0].comparison.status, "compared_fail");
        fs::remove_file(passing).unwrap();
        fs::remove_file(failing).unwrap();
    }

    #[test]
    fn coverage_plan_requires_every_case_and_compared_metric() {
        let passing = temporary_case_file(&case(8.17));
        let fingerprint = compare_reference_case(case(8.17)).manifest_fingerprint;
        let plan_path = passing.with_extension("plan.json");
        let plan = |case_id: &str, paths: Vec<&str>| {
            json!({"targetNormVersion":TARGET_NORM_VERSION,
                "requiredCases":[{"caseId":case_id,"manifestFingerprint":fingerprint,
                    "requiredPaths":paths}]})
        };
        fs::write(
            &plan_path,
            plan("synthetic-gate", vec!["beng2"]).to_string(),
        )
        .unwrap();
        let report = run(std::slice::from_ref(&passing), Some(&plan_path));
        assert!(report.numeric_comparison_passed);
        assert_eq!(report.planned_coverage_passed, Some(true));
        assert!(report
            .coverage_plan_fingerprint
            .as_ref()
            .unwrap()
            .starts_with("sha256:"));

        let mut altered = case(8.17);
        altered.source.document_id = "changed-with-same-value".into();
        fs::write(&passing, serde_json::to_vec(&altered).unwrap()).unwrap();
        let changed_case = run(std::slice::from_ref(&passing), Some(&plan_path));
        assert!(changed_case.numeric_comparison_passed);
        assert_eq!(changed_case.planned_coverage_passed, Some(false));
        assert!(changed_case
            .errors
            .iter()
            .any(|error| error.error.contains("Manifest fingerprint differs")));
        fs::write(&passing, serde_json::to_vec(&case(8.17)).unwrap()).unwrap();

        fs::write(
            &plan_path,
            plan("synthetic-gate", vec!["beng2", "beng1"]).to_string(),
        )
        .unwrap();
        let missing_path = run(std::slice::from_ref(&passing), Some(&plan_path));
        assert!(missing_path.numeric_comparison_passed);
        assert_eq!(missing_path.planned_coverage_passed, Some(false));
        assert!(missing_path.errors[0]
            .error
            .contains("Missing compared path"));

        fs::write(&plan_path, plan("other-case", vec!["beng2"]).to_string()).unwrap();
        let missing_case = run(std::slice::from_ref(&passing), Some(&plan_path));
        assert_eq!(missing_case.planned_coverage_passed, Some(false));
        assert!(missing_case
            .errors
            .iter()
            .any(|error| error.error.contains("Missing planned caseId")));
        assert!(missing_case
            .errors
            .iter()
            .any(|error| error.error.contains("Unplanned caseId")));

        fs::write(
            &plan_path,
            json!({"targetNormVersion":"obsolete",
            "requiredCases":[{"caseId":"synthetic-gate","manifestFingerprint":fingerprint,
                "requiredPaths":["beng2"]}]})
            .to_string(),
        )
        .unwrap();
        let wrong_version = run(std::slice::from_ref(&passing), Some(&plan_path));
        assert_eq!(wrong_version.planned_coverage_passed, Some(false));

        fs::write(
            &plan_path,
            plan("synthetic-gate", vec!["beng2", "beng2"]).to_string(),
        )
        .unwrap();
        let duplicate_path = run(std::slice::from_ref(&passing), Some(&plan_path));
        assert_eq!(duplicate_path.planned_coverage_passed, Some(false));

        let mut invalid_fingerprint = plan("synthetic-gate", vec!["beng2"]);
        invalid_fingerprint["requiredCases"][0]["manifestFingerprint"] = json!("sha256:bad");
        fs::write(&plan_path, invalid_fingerprint.to_string()).unwrap();
        let invalid = run(std::slice::from_ref(&passing), Some(&plan_path));
        assert_eq!(invalid.planned_coverage_passed, Some(false));
        assert!(invalid
            .errors
            .iter()
            .any(|error| error.error.contains("Invalid planned manifestFingerprint")));

        fs::write(&plan_path, "{}").unwrap();
        let malformed = run(std::slice::from_ref(&passing), Some(&plan_path));
        assert_eq!(malformed.planned_coverage_passed, Some(false));
        fs::remove_file(passing).unwrap();
        fs::remove_file(plan_path).unwrap();
    }
}
