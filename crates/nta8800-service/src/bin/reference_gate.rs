//! Batch numeric comparison for independently supplied NTA reference manifests.
//! A green exit code only means the supplied numbers fit the supplied tolerances.
//!
//! Inputs are single case manifests, or suites (`--suite`) that name a
//! project file per case instead of embedding it. With `--report-dir` the
//! run is also written as `reference-report.json` and `reference-report.md`,
//! with the commit, `KERNEL_VERSION`, the edition per case and the SHA-256 of
//! every input and output: the test record of BRL 9501 §6.2–6.3.

use nta8800_core::reference::{
    compare_reference_case, comparison_status_acceptable, ReferenceCase, ReferenceComparison,
};
use nta8800_core::{KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GateCase {
    file: String,
    /// Suite the case came from; absent for a single manifest.
    #[serde(skip_serializing_if = "Option::is_none")]
    suite: Option<String>,
    /// SHA-256 of the resolved case manifest (project included), as compared.
    input_sha256: String,
    /// SHA-256 of the project file a suite case names.
    #[serde(skip_serializing_if = "Option::is_none")]
    project_file_sha256: Option<String>,
    /// SHA-256 of the comparison below, as serialised.
    output_sha256: String,
    comparison: ReferenceComparison,
    /// Published values, for context only: they never decide the verdict.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    published: Vec<PublishedComparison>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublishedComparison {
    path: String,
    published: f64,
    actual: Option<f64>,
    /// (actual − published) / |published|, when both are known.
    relative_difference: Option<f64>,
    source: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Suite {
    suite_id: String,
    description: String,
    cases: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PatchStep {
    op: PatchOp,
    pointer: String,
    #[serde(default)]
    value: Option<Value>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum PatchOp {
    Set,
    Remove,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublishedValue {
    path: String,
    value: f64,
    source: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SuiteRecord {
    file: String,
    suite_id: String,
    description: String,
    sha256: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// Applies `set`/`remove` steps at JSON pointers. A step whose target (or,
/// for `set`, whose parent) is missing is an error, so a fixture change
/// cannot silently turn a patch into a no-op.
fn apply_patch(project: &mut Value, steps: Vec<PatchStep>) -> Result<(), String> {
    for step in steps {
        let (parent, key) = step
            .pointer
            .rsplit_once('/')
            .ok_or_else(|| format!("Patch pointer must start with '/': {}", step.pointer))?;
        let object = project
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| format!("Patch parent is not an object: {}", step.pointer))?;
        match step.op {
            PatchOp::Set => {
                let value = step
                    .value
                    .ok_or_else(|| format!("Patch set needs a value: {}", step.pointer))?;
                object.insert(key.to_string(), value);
            }
            PatchOp::Remove => {
                object
                    .remove(key)
                    .ok_or_else(|| format!("Patch remove target is missing: {}", step.pointer))?;
            }
        }
    }
    Ok(())
}

/// A suite case: a reference manifest whose `project` is read from
/// `projectFile` (relative to the suite), optionally at `projectPointer` and
/// changed by `projectPatch`; `published` is carried to the report.
struct ResolvedCase {
    case: ReferenceCase,
    project_file_sha256: String,
    published: Vec<PublishedValue>,
}

fn resolve_suite_case(mut raw: Value, suite_dir: &Path) -> Result<ResolvedCase, String> {
    let object = raw
        .as_object_mut()
        .ok_or_else(|| "Suite case must be an object".to_string())?;
    let project_file = object
        .remove("projectFile")
        .and_then(|value| value.as_str().map(str::to_string))
        .ok_or_else(|| "Suite case requires projectFile".to_string())?;
    let pointer = match object.remove("projectPointer") {
        None => String::new(),
        Some(Value::String(pointer)) => pointer,
        Some(_) => return Err("projectPointer must be a string".into()),
    };
    let patch: Vec<PatchStep> = match object.remove("projectPatch") {
        None => Vec::new(),
        Some(value) => serde_json::from_value(value).map_err(|error| error.to_string())?,
    };
    let published: Vec<PublishedValue> = match object.remove("published") {
        None => Vec::new(),
        Some(value) => serde_json::from_value(value).map_err(|error| error.to_string())?,
    };
    if object.contains_key("project") {
        return Err("Suite case gives projectFile, not project".into());
    }
    let bytes = fs::read(suite_dir.join(&project_file))
        .map_err(|error| format!("{project_file}: {error}"))?;
    let document: Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("{project_file}: {error}"))?;
    let mut project = document
        .pointer(&pointer)
        .cloned()
        .ok_or_else(|| format!("{project_file}: no value at {pointer}"))?;
    apply_patch(&mut project, patch)?;
    object.insert("project".into(), project);
    let case: ReferenceCase = serde_json::from_value(raw).map_err(|error| error.to_string())?;
    Ok(ResolvedCase {
        case,
        project_file_sha256: sha256(&bytes),
        published,
    })
}

/// Reads the actual value of a compared metric back from the comparison.
fn published_comparisons(
    comparison: &ReferenceComparison,
    published: Vec<PublishedValue>,
) -> Vec<PublishedComparison> {
    published
        .into_iter()
        .map(|item| {
            let actual = comparison
                .metrics
                .iter()
                .find(|metric| metric.path == item.path)
                .map(|metric| metric.actual);
            let relative_difference = actual
                .filter(|_| item.value != 0.0)
                .map(|actual| (actual - item.value) / item.value.abs());
            PublishedComparison {
                path: item.path,
                published: item.value,
                actual,
                relative_difference,
                source: item.source,
            }
        })
        .collect()
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
    /// Commit the run was made from, as supplied with `--commit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    suites: Vec<SuiteRecord>,
    numeric_comparison_passed: bool,
    /// Cases calculated without expected values (`pending_expectation`):
    /// they pass the run but prove nothing about the result.
    pending_expectation_cases: usize,
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
                let mut compared: HashSet<_> = case
                    .comparison
                    .metrics
                    .iter()
                    .map(|metric| metric.path.as_str())
                    .collect();
                if case.comparison.label_class.is_some() {
                    compared.insert("indicativeLabelClass");
                }
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

#[cfg(test)]
fn run(paths: &[PathBuf], plan_path: Option<&Path>) -> GateReport {
    run_with(paths, &[], plan_path)
}

fn push_case(
    report: &mut GateReport,
    case_ids: &mut HashSet<String>,
    file: String,
    suite: Option<String>,
    resolved: ResolvedCase,
) {
    if !case_ids.insert(resolved.case.case_id.clone()) {
        report.numeric_comparison_passed = false;
        report.errors.push(GateError {
            file,
            error: format!("Duplicate caseId: {}", resolved.case.case_id),
        });
        return;
    }
    let input_sha256 =
        sha256(&serde_json::to_vec(&resolved.case).expect("reference case serializes"));
    let comparison = compare_reference_case(resolved.case);
    report.numeric_comparison_passed &= comparison_status_acceptable(comparison.status);
    if comparison.status == "pending_expectation" {
        report.pending_expectation_cases += 1;
    }
    let output_sha256 =
        sha256(&serde_json::to_vec(&comparison).expect("reference comparison serializes"));
    let published = published_comparisons(&comparison, resolved.published);
    report.cases.push(GateCase {
        file,
        suite,
        input_sha256,
        project_file_sha256: (!resolved.project_file_sha256.is_empty())
            .then_some(resolved.project_file_sha256),
        output_sha256,
        comparison,
        published,
    });
}

fn run_with(paths: &[PathBuf], suites: &[PathBuf], plan_path: Option<&Path>) -> GateReport {
    let mut report = GateReport {
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        commit: None,
        suites: Vec::new(),
        numeric_comparison_passed: !paths.is_empty() || !suites.is_empty(),
        pending_expectation_cases: 0,
        planned_coverage_passed: None,
        coverage_plan_fingerprint: None,
        reference_verified: false,
        attest_status: "unattested",
        cases: Vec::new(),
        errors: Vec::new(),
    };
    let mut case_ids = HashSet::new();
    if paths.is_empty() && suites.is_empty() {
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
            Ok(case) => push_case(
                &mut report,
                &mut case_ids,
                file,
                None,
                ResolvedCase {
                    case,
                    project_file_sha256: String::new(),
                    published: Vec::new(),
                },
            ),
        }
    }
    for path in suites {
        let file = path.display().to_string();
        let suite = fs::read(path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                let suite: Suite =
                    serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
                Ok((suite, sha256(&bytes)))
            });
        let (suite, digest) = match suite {
            Ok(found) => found,
            Err(error) => {
                report.numeric_comparison_passed = false;
                report.errors.push(GateError { file, error });
                continue;
            }
        };
        if suite.cases.is_empty() {
            report.numeric_comparison_passed = false;
            report.errors.push(GateError {
                file: file.clone(),
                error: "Suite has no cases".into(),
            });
        }
        let suite_dir = path.parent().unwrap_or(Path::new("."));
        for (index, raw) in suite.cases.into_iter().enumerate() {
            let case_file = format!("{file}#cases[{index}]");
            match resolve_suite_case(raw, suite_dir) {
                Ok(resolved) => push_case(
                    &mut report,
                    &mut case_ids,
                    case_file,
                    Some(suite.suite_id.clone()),
                    resolved,
                ),
                Err(error) => {
                    report.numeric_comparison_passed = false;
                    report.errors.push(GateError {
                        file: case_file,
                        error,
                    });
                }
            }
        }
        report.suites.push(SuiteRecord {
            file,
            suite_id: suite.suite_id,
            description: suite.description,
            sha256: digest,
        });
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

/// Shown when no commit was supplied (a Dutch word, not a kernel code).
const UNKNOWN_COMMIT: &str = "onbekend";

/// Dutch summary of a run, for the release archive and the attest file.
fn markdown(report: &GateReport) -> String {
    let number = |value: f64| {
        let text = format!("{value:.3}");
        let text = text.trim_end_matches('0').trim_end_matches('.').to_string();
        text.replace('.', ",")
    };
    let verdict = |passed: bool| if passed { "geslaagd" } else { "niet geslaagd" };
    let mut out = String::new();
    out.push_str("# Referentieberekeningen\n\n");
    out.push_str(
        "Numerieke vergelijking van referentiegevallen met hun verwachte waarden en bandbreedtes. \
         Een geslaagde vergelijking is geen onafhankelijke verificatie en geen attest.\n\n",
    );
    out.push_str("| Gegeven | Waarde |\n| --- | --- |\n");
    out.push_str(&format!(
        "| Commit | {} |\n",
        report.commit.as_deref().unwrap_or(UNKNOWN_COMMIT)
    ));
    out.push_str(&format!(
        "| Rekenkernversie | {} |\n",
        report.kernel_version
    ));
    out.push_str(&format!(
        "| Normversie van de kern | {} |\n",
        report.target_norm_version
    ));
    out.push_str(&format!(
        "| Vergelijking | {} ({} gevallen) |\n",
        verdict(report.numeric_comparison_passed),
        report.cases.len()
    ));
    if report.pending_expectation_cases > 0 {
        out.push_str(&format!(
            "| Zonder verwachting | {} gevallen (alleen doorgerekend, geen oordeel) |\n",
            report.pending_expectation_cases
        ));
    }
    if let Some(passed) = report.planned_coverage_passed {
        out.push_str(&format!("| Dekkingsplan | {} |\n", verdict(passed)));
    }
    out.push_str(&format!("| Attest | {} |\n", report.attest_status));
    for suite in &report.suites {
        out.push_str(&format!(
            "\n**Suite {}** (`{}`, {}): {}\n",
            suite.suite_id, suite.file, suite.sha256, suite.description
        ));
    }
    out.push_str("\n## Per geval\n\n");
    out.push_str(
        "| Geval | Uitgave | Status | Grootheid | Verwacht | Berekend | Verschil | Band |\n",
    );
    out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for case in &report.cases {
        let comparison = &case.comparison;
        for metric in &comparison.recorded {
            let band = match metric.relative_tolerance {
                Some(fraction) => format!("{} %", number(fraction * 100.0)),
                None => number(metric.absolute_tolerance),
            };
            out.push_str(&format!(
                "| {} | {} | geen verwachting | `{}` | – | {} | – | {} |\n",
                comparison.case_id,
                comparison.target_norm_version,
                metric.path,
                number(metric.actual),
                band
            ));
        }
        if comparison.metrics.is_empty() && comparison.recorded.is_empty() {
            out.push_str(&format!(
                "| {} | {} | {} | – | – | – | – | – |\n",
                comparison.case_id, comparison.target_norm_version, comparison.status
            ));
        }
        for metric in &comparison.metrics {
            out.push_str(&format!(
                "| {} | {} | {} | `{}` | {} | {} | {} | {}{} |\n",
                comparison.case_id,
                comparison.target_norm_version,
                comparison.status,
                metric.path,
                number(metric.expected),
                number(metric.actual),
                number(metric.absolute_difference),
                number(metric.applied_tolerance),
                if metric.within_tolerance { "" } else { " ✗" }
            ));
        }
    }
    let published: Vec<_> = report
        .cases
        .iter()
        .flat_map(|case| case.published.iter().map(move |item| (case, item)))
        .collect();
    if !published.is_empty() {
        out.push_str("\n## Gepubliceerde waarden (ter vergelijking, niet beoordeeld)\n\n");
        out.push_str(
            "| Geval | Grootheid | Gepubliceerd | Berekend | Relatief verschil | Bron |\n",
        );
        out.push_str("| --- | --- | --- | --- | --- | --- |\n");
        for (case, item) in published {
            out.push_str(&format!(
                "| {} | `{}` | {} | {} | {} | {} |\n",
                case.comparison.case_id,
                item.path,
                number(item.published),
                item.actual.map_or("–".into(), number),
                item.relative_difference
                    .map_or("–".into(), |fraction| format!(
                        "{} %",
                        number(fraction * 100.0)
                    )),
                item.source
            ));
        }
    }
    out.push_str("\n## Vingerafdrukken\n\n| Geval | Invoer (SHA-256) | Projectbestand (SHA-256) | Uitvoer (SHA-256) |\n| --- | --- | --- | --- |\n");
    for case in &report.cases {
        out.push_str(&format!(
            "| {} | `{}` | {} | `{}` |\n",
            case.comparison.case_id,
            case.input_sha256,
            case.project_file_sha256
                .as_deref()
                .map_or("–".into(), |digest| format!("`{digest}`")),
            case.output_sha256
        ));
    }
    if !report.errors.is_empty() {
        out.push_str("\n## Fouten\n\n");
        for error in &report.errors {
            out.push_str(&format!("- `{}`: {}\n", error.file, error.error));
        }
    }
    out
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let mut plan_path = None;
    let mut paths = Vec::new();
    let mut suites = Vec::new();
    let mut report_dir = None;
    let mut commit = None;
    while let Some(arg) = args.next() {
        if arg == "--suite" || arg == "--report-dir" || arg == "--commit" {
            let Some(value) = args.next() else {
                eprintln!("{} requires a value", arg.to_string_lossy());
                return ExitCode::from(2);
            };
            if arg == "--suite" {
                suites.push(PathBuf::from(value));
            } else if arg == "--report-dir" {
                report_dir = Some(PathBuf::from(value));
            } else {
                commit = Some(value.to_string_lossy().into_owned());
            }
        } else if arg == "--plan" {
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
    let mut report = run_with(&paths, &suites, plan_path.as_deref());
    report.commit = commit;
    let success =
        report.numeric_comparison_passed && report.planned_coverage_passed.unwrap_or(true);
    let json = match serde_json::to_string_pretty(&report) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("Could not serialize reference comparison report: {error}");
            return ExitCode::from(2);
        }
    };
    if let Some(dir) = report_dir {
        let written = fs::create_dir_all(&dir)
            .and_then(|()| fs::write(dir.join("reference-report.json"), format!("{json}\n")))
            .and_then(|()| fs::write(dir.join("reference-report.md"), markdown(&report)));
        if let Err(error) = written {
            eprintln!("Could not write the report to {}: {error}", dir.display());
            return ExitCode::from(2);
        }
    } else {
        println!("{json}");
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

    #[test]
    fn coverage_plan_can_require_the_indicative_label_class() {
        let mut reference = case(8.17);
        reference.expected_label_class = Some("A+++".into());
        let fingerprint = compare_reference_case(reference.clone()).manifest_fingerprint;
        let case_path = temporary_case_file(&reference);
        let plan_path = case_path.with_extension("plan.json");
        fs::write(
            &plan_path,
            json!({"targetNormVersion":TARGET_NORM_VERSION,
                "requiredCases":[{"caseId":"synthetic-gate",
                    "manifestFingerprint":fingerprint,
                    "requiredPaths":["beng2","indicativeLabelClass"]}]})
            .to_string(),
        )
        .unwrap();
        let pass = run(std::slice::from_ref(&case_path), Some(&plan_path));
        assert!(pass.numeric_comparison_passed);
        assert_eq!(pass.planned_coverage_passed, Some(true));
        assert!(
            pass.cases[0]
                .comparison
                .label_class
                .as_ref()
                .unwrap()
                .matches
        );

        reference.expected_label_class = None;
        fs::write(&case_path, serde_json::to_vec(&reference).unwrap()).unwrap();
        let missing = run(std::slice::from_ref(&case_path), Some(&plan_path));
        assert_eq!(missing.planned_coverage_passed, Some(false));
        assert!(missing
            .errors
            .iter()
            .any(|error| error.error.contains("Missing compared path")));
        fs::remove_file(case_path).unwrap();
        fs::remove_file(plan_path).unwrap();
    }

    fn suite_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "oes-reference-suite-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("project.json"),
            include_str!("../../../../training-data/nta8800-project-performance-synthetic.json"),
        )
        .unwrap();
        dir
    }

    fn suite_case(value: f64, patch: Value) -> Value {
        json!({
            "caseId":"suite-case", "normVersion":"2025+C1", "projectFile":"project.json",
            "projectPatch": patch,
            "source":{"publisher":"synthetic", "documentId":"internal-2", "edition":"test",
                "usePermission":"internal", "independentReviewer":"test"},
            "expected":[{"path":"beng2", "value":value, "unit":"kWh/m2.year",
                "normReference":"internal test", "absoluteTolerance":0.0,
                "relativeTolerance":0.01}],
            "published":[{"path":"beng2", "value":10.0, "source":"synthetic report"}]
        })
    }

    #[test]
    fn suite_reads_project_files_records_digests_and_published_values() {
        let dir = suite_dir();
        let suite = dir.join("suite.json");
        // A patch that sets a member the project already has keeps BENG 2.
        let patch = json!([{"op":"set", "pointer":"/name", "value":"renamed"}]);
        fs::write(
            &suite,
            json!({"suiteId":"synthetic", "description":"test suite",
                "cases":[suite_case(8.2, patch)]})
            .to_string(),
        )
        .unwrap();
        let report = run_with(&[], std::slice::from_ref(&suite), None);
        assert!(
            report.numeric_comparison_passed,
            "{:?}",
            report.errors.len()
        );
        let case = &report.cases[0];
        assert_eq!(case.suite.as_deref(), Some("synthetic"));
        assert!(case.input_sha256.starts_with("sha256:"));
        assert!(case.output_sha256.starts_with("sha256:"));
        assert!(case
            .project_file_sha256
            .as_deref()
            .unwrap()
            .starts_with("sha256:"));
        // 8,17 against 8,2 is within 1 % of 8,2; the band is the relative one.
        assert!((case.comparison.metrics[0].applied_tolerance - 0.082).abs() < 1e-12);
        let published = &case.published[0];
        assert_eq!(published.published, 10.0);
        let actual = published.actual.unwrap();
        assert!((published.relative_difference.unwrap() - (actual - 10.0) / 10.0).abs() < 1e-12);
        assert_eq!(report.suites[0].suite_id, "synthetic");
        let text = markdown(&report);
        assert!(text.contains("| suite-case | NTA 8800:2025+C1:2026 | compared_pass | `beng2` |"));
        assert!(text.contains("Gepubliceerde waarden"));
        assert!(text.contains(&case.output_sha256));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn suite_errors_fail_the_run() {
        let dir = suite_dir();
        let suite = dir.join("suite.json");
        let missing_target = json!([{"op":"remove", "pointer":"/noSuchMember"}]);
        let mut embedded = suite_case(8.17, json!([]));
        embedded["project"] = json!({});
        fs::write(
            &suite,
            json!({"suiteId":"broken", "description":"test suite",
                "cases":[suite_case(8.17, missing_target), embedded]})
            .to_string(),
        )
        .unwrap();
        let report = run_with(&[], std::slice::from_ref(&suite), None);
        assert!(!report.numeric_comparison_passed);
        assert!(report.cases.is_empty());
        assert!(report.errors[0].error.contains("remove target is missing"));
        assert!(report.errors[1].error.contains("not project"));
        fs::write(
            &suite,
            json!({"suiteId":"empty", "description":"test suite", "cases":[]}).to_string(),
        )
        .unwrap();
        let empty = run_with(&[], std::slice::from_ref(&suite), None);
        assert!(!empty.numeric_comparison_passed);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn committed_suites_pass() {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../training-data/reference-suites");
        let suites = [
            root.join("openbare-gevallen.json"),
            root.join("rvo-voorbeeldwoningen.json"),
        ];
        let report = run_with(&[], &suites, None);
        let failing: Vec<_> = report
            .cases
            .iter()
            .filter(|case| case.comparison.status != "compared_pass")
            .map(|case| &case.comparison.case_id)
            .collect();
        assert!(report.errors.is_empty(), "{}", report.errors[0].error);
        assert!(failing.is_empty(), "{failing:?}");
        assert!(report.numeric_comparison_passed);
        assert_eq!(report.cases.len(), 23);
        // Every edition the kernel calculates in is exercised.
        for edition in [
            "NTA 8800:2025+C1:2026",
            "NTA 8800:2024 met INT-V1:2024",
            "NTA 8800:2023",
            "NTA 8800:2022",
            "NTA 8800:2020+A1:2020",
        ] {
            assert!(
                report
                    .cases
                    .iter()
                    .any(|case| case.comparison.target_norm_version == edition),
                "{edition}"
            );
        }
    }
}
