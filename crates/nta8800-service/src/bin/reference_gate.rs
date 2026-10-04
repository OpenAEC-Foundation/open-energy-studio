//! Batch numeric comparison for independently supplied NTA reference manifests.
//! A green exit code only means the supplied numbers fit the supplied tolerances.

use nta8800_core::reference::{compare_reference_case, ReferenceCase, ReferenceComparison};
use nta8800_core::{KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
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
    reference_verified: bool,
    attest_status: &'static str,
    cases: Vec<GateCase>,
    errors: Vec<GateError>,
}

fn run(paths: &[PathBuf]) -> GateReport {
    let mut report = GateReport {
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        numeric_comparison_passed: !paths.is_empty(),
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
    report
}

fn main() -> ExitCode {
    let paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let report = run(&paths);
    let success = report.numeric_comparison_passed;
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
        let report = run(&[]);
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
        let pass = run(std::slice::from_ref(&passing));
        assert!(pass.numeric_comparison_passed);
        assert_eq!(pass.cases.len(), 1);
        assert!(!pass.reference_verified);
        let duplicate = run(&[passing.clone(), failing.clone()]);
        assert!(!duplicate.numeric_comparison_passed);
        assert_eq!(duplicate.errors.len(), 1);
        assert!(duplicate.errors[0].error.contains("Duplicate caseId"));
        let fail = run(std::slice::from_ref(&failing));
        assert!(!fail.numeric_comparison_passed);
        assert_eq!(fail.cases[0].comparison.status, "compared_fail");
        fs::remove_file(passing).unwrap();
        fs::remove_file(failing).unwrap();
    }
}
