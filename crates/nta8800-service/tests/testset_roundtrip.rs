//! The testset dossier (BRL 9501 §3.3): a deeltest written as a project file
//! and opened again calculates exactly as the reference suite calculates it.
//!
//! Twelve ISSO 54 v2.0 deeltests, dwelling and utility, go the way the app
//! goes: the file's `project` (`ProjectSerializer.deserializeProjectFile`)
//! through `calculate_project_performance`, the operation of the HTTP route
//! and the MCP tool. The answer must equal `compare_reference_case` on the
//! suite case, and its input fingerprint must equal the file's kernel stamp,
//! so the app opens the file without an "input changed" notice.
//!
//! Three of the files are kept in `src/__tests__/fixtures/testset/` for the
//! app-side test (`testset-project-files.test.ts`), which opens them with the
//! app's own code; `NTA_WRITE_TESTSET_FIXTURE=1` rewrites them.

use nta8800_core::reference::{compare_reference_case, ReferenceCase};
use nta8800_service::testset::{self, apply_patch, PatchStep};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const SAMPLE_SERIES: [&str; 12] = [
    "EPW001", "EPW004", "EPW101", "EPW203", "EPW301", "EPW406", "EPW407", "EPW501", "EPU001",
    "EPU002", "EPU303", "EPU602",
];
const FIXTURES: [&str; 3] = ["isso54-v2-EPW001", "isso54-v2-EPW101a", "isso54-v2-EPU001"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The suite case with its project resolved, as `reference_gate` resolves it.
fn resolve(raw: &Value, suite_dir: &Path) -> ReferenceCase {
    let mut raw = raw.clone();
    let object = raw.as_object_mut().unwrap();
    let file = object.remove("projectFile").unwrap();
    let patch: Vec<PatchStep> = serde_json::from_value(
        object
            .remove("projectPatch")
            .unwrap_or(Value::Array(vec![])),
    )
    .unwrap();
    let mut project: Value =
        serde_json::from_slice(&fs::read(suite_dir.join(file.as_str().unwrap())).unwrap()).unwrap();
    apply_patch(&mut project, patch).unwrap();
    object.insert("project".into(), project);
    serde_json::from_value(raw).unwrap()
}

fn recorded(case: ReferenceCase, path: &str) -> f64 {
    compare_reference_case(case)
        .recorded
        .iter()
        .find(|metric| metric.path == path)
        .map(|metric| metric.actual)
        .unwrap_or_else(|| panic!("{path} not recorded"))
}

#[test]
fn opened_project_files_calculate_as_the_suite() {
    let suite_path = root().join("training-data/reference-suites/isso54-v2.json");
    let suite: Value = serde_json::from_slice(&fs::read(&suite_path).unwrap()).unwrap();
    let suite_dir = suite_path.parent().unwrap();
    let cases = suite["cases"].as_array().unwrap();
    let mut checked = Vec::new();
    for series in SAMPLE_SERIES {
        let raw = cases
            .iter()
            .find(|case| {
                case["caseId"]
                    .as_str()
                    .unwrap()
                    .rsplit('-')
                    .next()
                    .unwrap()
                    .starts_with(series)
            })
            .unwrap_or_else(|| panic!("no case of {series}"));
        let case = resolve(raw, suite_dir);
        let case_id = case.case_id.clone();
        let first = testset::calculate(&case.project);
        let text = testset::project_file(&case.project, &first);
        let opened = testset::open_project_file(&text).unwrap();
        assert_eq!(
            opened, case.project,
            "{case_id}: the file holds the suite project"
        );
        let answer = testset::calculate(&opened);
        let stamp: Value = serde_json::from_str::<Value>(&text).unwrap()["kernel"].clone();
        assert_eq!(
            answer["inputFingerprint"], stamp["inputFingerprint"],
            "{case_id}"
        );
        assert_eq!(answer["kernelVersion"], stamp["kernelVersion"], "{case_id}");
        let performance = &answer["performance"];
        for (path, key) in [
            ("beng1", "needIndicatorKwhPerM2Year"),
            ("beng2", "primaryFossilIndicatorKwhPerM2Year"),
            ("beng3", "renewableSharePercent"),
        ] {
            let suite_value = recorded(case.clone(), path);
            let file_value = performance[key].as_f64().unwrap();
            assert_eq!(suite_value, file_value, "{case_id}: {path}");
        }
        if FIXTURES.contains(&case_id.as_str()) {
            check_fixture(&case_id, &text);
        }
        checked.push(case_id);
    }
    // A fixture outside the sample is written or checked here.
    for fixture in FIXTURES {
        if !checked.iter().any(|id| id == fixture) {
            let raw = cases.iter().find(|case| case["caseId"] == fixture).unwrap();
            let case = resolve(raw, suite_dir);
            let text = testset::project_file(&case.project, &testset::calculate(&case.project));
            check_fixture(fixture, &text);
        }
    }
    assert!(checked.iter().filter(|id| id.contains("EPW")).count() >= 8);
    assert!(checked.iter().filter(|id| id.contains("EPU")).count() >= 4);
}

/// The app-side fixture holds the project this generator writes; the kernel
/// stamp may differ after a kernel change, the project may not.
fn check_fixture(case_id: &str, text: &str) {
    let path = root()
        .join("src/__tests__/fixtures/testset")
        .join(format!("{case_id}.oes.json"));
    if std::env::var_os("NTA_WRITE_TESTSET_FIXTURE").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        return;
    }
    let kept: Value = serde_json::from_slice(&fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "{}: {error}; write it with NTA_WRITE_TESTSET_FIXTURE=1",
            path.display()
        )
    }))
    .unwrap();
    let written: Value = serde_json::from_str(text).unwrap();
    assert_eq!(
        kept["project"], written["project"],
        "{case_id}: fixture is stale; rewrite it with NTA_WRITE_TESTSET_FIXTURE=1"
    );
    assert_eq!(kept["type"], written["type"]);
    assert_eq!(kept["version"], written["version"]);
}
