//! Testset dossier for the attesting body (BRL 9501 §3.3, p. 5–6): every
//! suite case as a project file the desktop app opens, plus an overview of
//! what the kernel calculates for it.
//!
//! A project file is the app's own envelope (`src/core/io/ProjectSerializer.ts`:
//! `version`, `type`, `project`, `kernel`) with the kernel stamp of the
//! calculation, so the app opens it without a version or input warning. The
//! numbers come from the same operation the HTTP route and the MCP tool run
//! (`calculate_project_performance`), on the project as written to the file.

use crate::operations::{execute, operation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// File format version and type of `ProjectSerializer.ts`.
pub const PROJECT_FILE_VERSION: &str = "1.0";
pub const PROJECT_FILE_TYPE: &str = "open-energy-studio";

/// Energy functions of the overview, in the order of §5.5.3.
pub const SERVICES: [(&str, &str); 7] = [
    ("heating", "verwarming"),
    ("hotWater", "tapwater"),
    ("cooling", "koeling"),
    ("humidification", "bevochtiging"),
    ("ventilation", "ventilatoren"),
    ("lighting", "verlichting"),
    ("auxiliary", "hulpenergie"),
];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchStep {
    pub op: PatchOp,
    pub pointer: String,
    // Preserve an explicitly supplied JSON null. Plain Option<Value> treats
    // null like an absent value, making a valid `set` to null impossible.
    #[serde(default, deserialize_with = "present_patch_value")]
    pub value: Option<Value>,
}

fn present_patch_value<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Value::deserialize(deserializer).map(Some)
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PatchOp {
    Set,
    Remove,
}

/// Applies `set`/`remove` steps at JSON pointers. An object `set` may add a
/// member; an array `set` replaces an existing index. A missing parent or
/// target is an error, so a fixture change cannot silently turn a patch into
/// a no-op.
pub fn apply_patch(project: &mut Value, steps: Vec<PatchStep>) -> Result<(), String> {
    for step in steps {
        if !step.pointer.starts_with('/') {
            return Err(format!(
                "Patch pointer must start with '/': {}",
                step.pointer
            ));
        }
        let (parent, key) = step
            .pointer
            .rsplit_once('/')
            .expect("a pointer starting with '/' has a final token");
        // serde_json resolves escapes in the parent pointer; decode the final
        // token too, otherwise /a~1b silently edits a literal "a~1b" key.
        let mut decoded_key = String::new();
        let mut chars = key.chars();
        while let Some(ch) = chars.next() {
            if ch == '~' {
                match chars.next() {
                    Some('0') => decoded_key.push('~'),
                    Some('1') => decoded_key.push('/'),
                    _ => return Err(format!("Invalid patch pointer escape: {}", step.pointer)),
                }
            } else {
                decoded_key.push(ch);
            }
        }
        let parent_value = project
            .pointer_mut(parent)
            .ok_or_else(|| format!("Patch parent is missing: {}", step.pointer))?;
        match parent_value {
            Value::Object(object) => match step.op {
                PatchOp::Set => {
                    let value = step
                        .value
                        .ok_or_else(|| format!("Patch set needs a value: {}", step.pointer))?;
                    object.insert(decoded_key, value);
                }
                PatchOp::Remove => {
                    object.remove(&decoded_key).ok_or_else(|| {
                        format!("Patch remove target is missing: {}", step.pointer)
                    })?;
                }
            },
            Value::Array(array) => {
                let index = decoded_key
                    .parse::<usize>()
                    .ok()
                    .filter(|index| decoded_key == index.to_string() && *index < array.len());
                let index = index.ok_or_else(|| {
                    format!("Patch array index is invalid or missing: {}", step.pointer)
                })?;
                match step.op {
                    PatchOp::Set => {
                        array[index] = step
                            .value
                            .ok_or_else(|| format!("Patch set needs a value: {}", step.pointer))?;
                    }
                    PatchOp::Remove => {
                        array.remove(index);
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Patch parent is not an object or array: {}",
                    step.pointer
                ));
            }
        }
    }
    Ok(())
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// The kernel answer for a project, through the operation of the HTTP route.
pub fn calculate(project: &Value) -> Value {
    let op = operation("calculate_project_performance")
        .expect("calculate_project_performance is registered");
    execute(op, json!({ "project": project })).body
}

/// The project file the app saves and opens, with the kernel stamp of
/// `assessment`. Pretty-printed like `serializeProject`.
pub fn project_file(project: &Value, assessment: &Value) -> String {
    let mut file = Map::new();
    file.insert("version".into(), json!(PROJECT_FILE_VERSION));
    file.insert("type".into(), json!(PROJECT_FILE_TYPE));
    file.insert("project".into(), project.clone());
    let stamp = ["kernelVersion", "targetNormVersion", "inputFingerprint"]
        .iter()
        .map(|key| {
            assessment
                .get(*key)
                .and_then(Value::as_str)
                .map(|v| (*key, v))
        })
        .collect::<Option<Vec<_>>>();
    if let Some(stamp) = stamp {
        file.insert(
            "kernel".into(),
            Value::Object(
                stamp
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), json!(value)))
                    .collect(),
            ),
        );
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(file)).expect("value serializes");
    text.push('\n');
    text
}

/// What a project file holds for the app: the envelope's `project`, refused
/// as `ProjectSerializer.deserializeProjectFile` refuses it.
pub fn open_project_file(text: &str) -> Result<Value, String> {
    let mut file: Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    if file.get("type").and_then(Value::as_str) != Some(PROJECT_FILE_TYPE) {
        return Err("Invalid file format: wrong type".into());
    }
    let major = file
        .get("version")
        .and_then(Value::as_str)
        .and_then(|version| version.split('.').next())
        .and_then(|major| major.parse::<u32>().ok());
    if major != Some(1) {
        return Err("Invalid file format: unsupported version".into());
    }
    file.get_mut("project")
        .filter(|project| project.is_object())
        .map(Value::take)
        .ok_or_else(|| "Invalid file format: missing project data".into())
}

/// One row of the outcome overview.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestsetRow {
    pub case_id: String,
    /// `EP-W001`-style id of the deeltest.
    pub test_id: String,
    pub topic: String,
    pub file: String,
    pub norm_version: String,
    pub status: String,
    pub beng1: Option<f64>,
    pub beng2: Option<f64>,
    pub beng3: Option<f64>,
    pub tojuli_max: Option<f64>,
    /// Delivered energy per energy function, kWh per year, summed over carriers.
    pub delivered_kwh: BTreeMap<String, f64>,
    pub input_sha256: String,
    pub output_sha256: String,
    /// Expected values and verdict, once the case carries expectations.
    pub expected: Vec<ExpectedValue>,
    pub verdict: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpectedValue {
    pub path: String,
    /// A number for a metric, the class letter for `indicativeLabelClass`.
    pub expected: Value,
    pub actual: Value,
    pub within_tolerance: bool,
}

impl ExpectedValue {
    fn expected_text(&self) -> String {
        match &self.expected {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        }
    }
}

/// `isso54-v2-EPW101c` → `EP-W101c`; other ids unchanged.
pub fn test_id(case_id: &str) -> String {
    let id = case_id.rsplit('-').next().unwrap_or(case_id);
    if id.len() > 3 && id.starts_with("EP") {
        format!("{}-{}", &id[..2], &id[2..])
    } else {
        case_id.to_string()
    }
}

/// The topic of a deeltest from its series (`EPW101c` → `EPW101`).
pub fn topic(case_id: &str, topics: &Map<String, Value>) -> String {
    let id = case_id.rsplit('-').next().unwrap_or(case_id);
    let series: String = id.chars().take(6).collect();
    topics
        .get(&series)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// The overview row for one case. `file_name` is the project file's name in
/// the dossier, `file_text` its content; `comparison` is the reference
/// comparison of the case, for the expected values once they exist.
pub fn row(
    case_id: &str,
    topic: String,
    file_name: &str,
    file_text: &str,
    assessment: &Value,
    comparison: Option<&Value>,
) -> TestsetRow {
    let performance = assessment.get("performance");
    let number = |key: &str| performance.and_then(|p| p.get(key)).and_then(Value::as_f64);
    let mut delivered = BTreeMap::new();
    for item in performance
        .and_then(|p| p.pointer("/energyByService/annual"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let (Some(service), Some(kwh)) = (
            item.get("service").and_then(Value::as_str),
            item.get("deliveredKwh").and_then(Value::as_f64),
        ) else {
            continue;
        };
        *delivered.entry(service.to_string()).or_insert(0.0) += kwh;
    }
    let mut expected: Vec<ExpectedValue> = comparison
        .and_then(|c| c.get("metrics"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|metric| {
            Some(ExpectedValue {
                path: metric.get("path")?.as_str()?.to_string(),
                expected: json!(metric.get("expected")?.as_f64()?),
                actual: json!(metric.get("actual")?.as_f64()?),
                within_tolerance: metric.get("withinTolerance")?.as_bool()?,
            })
        })
        .collect();
    // An expected label class is part of the verdict as the comparison makes it.
    if let Some(class) = comparison.and_then(|c| c.get("labelClass")) {
        let text = |key: &str| class.get(key).and_then(Value::as_str).unwrap_or("");
        expected.push(ExpectedValue {
            path: text("path").to_string(),
            expected: json!(text("expected")),
            actual: json!(text("actual")),
            within_tolerance: class.get("matches").and_then(Value::as_bool) == Some(true),
        });
    }
    let verdict = verdict(
        comparison
            .and_then(|c| c.get("status"))
            .and_then(Value::as_str),
    );
    TestsetRow {
        case_id: case_id.to_string(),
        test_id: test_id(case_id),
        topic,
        file: file_name.to_string(),
        norm_version: assessment
            .get("normVersion")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        status: assessment
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        beng1: number("needIndicatorKwhPerM2Year"),
        beng2: number("primaryFossilIndicatorKwhPerM2Year"),
        beng3: number("renewableSharePercent"),
        tojuli_max: number("tojuliMaxK"),
        delivered_kwh: delivered,
        input_sha256: sha256(file_text.as_bytes()),
        output_sha256: sha256(&serde_json::to_vec(assessment).expect("value serializes")),
        expected,
        verdict,
    }
}

/// The verdict of a case from the status of its reference comparison, so the
/// overview says what the gate says: a label-class mismatch fails the case
/// as a metric outside its band does, and a case with expectations that did
/// not calculate is never shown as still waiting for expected values.
pub fn verdict(status: Option<&str>) -> Option<String> {
    match status? {
        "compared_pass" => Some("binnen de band".into()),
        "compared_fail" => Some("buiten de band".into()),
        "pending_expectation" => None,
        other => Some(format!("niet vergeleken ({other})")),
    }
}

/// The name of a case's project file in `projecten/`. A case id is a plain
/// name, so no file lands outside the dossier or under another case's name.
pub fn project_file_name(case_id: &str) -> Result<String, String> {
    let plain = !case_id.is_empty()
        && !case_id.starts_with('.')
        && case_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if plain {
        Ok(format!("{case_id}.oes.json"))
    } else {
        Err(format!("caseId is not a plain file name: {case_id:?}"))
    }
}

fn decimal(value: Option<f64>, places: usize) -> String {
    value.map_or_else(String::new, |value| format!("{value:.places$}"))
}

/// The overview as CSV (decimal point, `;` separator) for spreadsheets.
pub fn csv(rows: &[TestsetRow]) -> String {
    let mut out = String::from(
        "deeltest;onderwerp;projectbestand;uitgave;status;beng1;beng2;beng3;tojuli_max",
    );
    for (_, label) in SERVICES {
        out.push_str(&format!(";geleverd_{label}_kwh"));
    }
    out.push_str(";invoer_sha256;uitvoer_sha256;verwacht;oordeel\n");
    for row in rows {
        let fields = [
            row.test_id.clone(),
            row.topic.replace(';', ","),
            row.file.clone(),
            row.norm_version.clone(),
            row.status.clone(),
            decimal(row.beng1, 2),
            decimal(row.beng2, 2),
            decimal(row.beng3, 1),
            decimal(row.tojuli_max, 2),
        ];
        out.push_str(&fields.join(";"));
        for (service, _) in SERVICES {
            out.push(';');
            out.push_str(&decimal(row.delivered_kwh.get(service).copied(), 0));
        }
        let expected = row
            .expected
            .iter()
            .map(|value| format!("{}={}", value.path, value.expected_text()))
            .collect::<Vec<_>>()
            .join(" ");
        out.push_str(&format!(
            ";{};{};{};{}\n",
            row.input_sha256,
            row.output_sha256,
            expected,
            row.verdict.as_deref().unwrap_or("")
        ));
    }
    out
}

/// The overview as Markdown (decimal comma) for the testrapport.
pub fn markdown(rows: &[TestsetRow], header: &[(&str, String)]) -> String {
    let comma = |text: String| text.replace('.', ",");
    let mut out = String::from("# Testsetdossier ISSO 54\n\n");
    out.push_str(
        "Elke deeltest als projectbestand dat het programma opent (map `projecten/`), met wat de \
         rekenkern ervoor berekent. De kolommen *verwacht* en *oordeel* blijven leeg zolang de \
         verwachte uitkomsten van de testset ontbreken; ze vullen zich zodra de deeltest in de \
         suite een verwachting krijgt. Geen ISSO-tekst: het onderwerp staat in eigen woorden.\n\n",
    );
    out.push_str("| Gegeven | Waarde |\n| --- | --- |\n");
    for (label, value) in header {
        out.push_str(&format!("| {label} | {value} |\n"));
    }
    out.push_str(&format!("| Deeltesten | {} |\n\n", rows.len()));
    out.push_str(
        "| Deeltest | Onderwerp | Projectbestand | Status | BENG 1 | BENG 2 | BENG 3 | TOjuli",
    );
    for (_, label) in SERVICES {
        out.push_str(&format!(" | {label} (kWh)"));
    }
    out.push_str(" | Verwacht | Oordeel |\n");
    out.push_str(&"| --- ".repeat(8 + SERVICES.len() + 2));
    out.push_str("|\n");
    for row in rows {
        out.push_str(&format!(
            "| {} | {} | `{}` | {} | {} | {} | {} | {}",
            row.test_id,
            row.topic,
            row.file,
            row.status,
            comma(decimal(row.beng1, 2)),
            comma(decimal(row.beng2, 2)),
            comma(decimal(row.beng3, 1)),
            comma(decimal(row.tojuli_max, 2)),
        ));
        for (service, _) in SERVICES {
            out.push_str(&format!(
                " | {}",
                decimal(row.delivered_kwh.get(service).copied(), 0)
            ));
        }
        let expected = row
            .expected
            .iter()
            .map(|value| format!("{} {}", value.path, comma(value.expected_text())))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            " | {} | {} |\n",
            expected,
            row.verdict.as_deref().unwrap_or("")
        ));
    }
    out.push_str("\n## SHA-256 per deeltest\n\n| Deeltest | Projectbestand (invoer) | Antwoord van de rekenkern (uitvoer) |\n| --- | --- | --- |\n");
    for row in rows {
        out.push_str(&format!(
            "| {} | `{}` | `{}` |\n",
            row.test_id, row.input_sha256, row.output_sha256
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_decodes_json_pointer_tokens_without_touching_similarly_named_keys() {
        let mut project = json!({"a/b": {"c~d": 1}, "a~1b": {"c~0d": 2}});
        let steps: Vec<PatchStep> = serde_json::from_value(json!([
            {"op": "set", "pointer": "/a~1b/c~0d", "value": 3},
            {"op": "remove", "pointer": "/a~1b/c~0d"}
        ]))
        .unwrap();
        apply_patch(&mut project, steps).unwrap();
        assert_eq!(project, json!({"a/b": {}, "a~1b": {"c~0d": 2}}));

        let invalid: Vec<PatchStep> = serde_json::from_value(json!([
            {"op": "set", "pointer": "/a~2b", "value": 4}
        ]))
        .unwrap();
        assert!(apply_patch(&mut project, invalid).is_err());
        assert!(project.get("a~2b").is_none());
    }

    #[test]
    fn patch_replaces_or_removes_array_items_only_at_existing_indices() {
        let mut project = json!({"items": ["a", "b", "c"]});
        let steps: Vec<PatchStep> = serde_json::from_value(json!([
            {"op": "set", "pointer": "/items/1", "value": "B"},
            {"op": "remove", "pointer": "/items/0"}
        ]))
        .unwrap();
        apply_patch(&mut project, steps).unwrap();
        assert_eq!(project, json!({"items": ["B", "c"]}));

        for token in ["02", "+1", "2", "-", "x"] {
            let steps: Vec<PatchStep> = serde_json::from_value(json!([
                {"op": "set", "pointer": format!("/items/{token}"), "value": "wrong"}
            ]))
            .unwrap();
            assert!(apply_patch(&mut project, steps).is_err(), "{token}");
            assert_eq!(project, json!({"items": ["B", "c"]}));
        }
    }

    #[test]
    fn patch_set_distinguishes_explicit_null_from_missing_value() {
        let mut project = json!({"optional": 3});
        let null_step: Vec<PatchStep> = serde_json::from_value(json!([
            {"op": "set", "pointer": "/optional", "value": null}
        ]))
        .unwrap();
        apply_patch(&mut project, null_step).unwrap();
        assert_eq!(project, json!({"optional": null}));

        let missing_step: Vec<PatchStep> = serde_json::from_value(json!([
            {"op": "set", "pointer": "/optional"}
        ]))
        .unwrap();
        assert!(apply_patch(&mut project, missing_step).is_err());
        assert_eq!(project, json!({"optional": null}));
    }

    #[test]
    fn test_ids_and_topics() {
        assert_eq!(test_id("isso54-v2-EPW101c"), "EP-W101c");
        assert_eq!(test_id("isso54-v2-EPU001"), "EP-U001");
        let topics = json!({"EPW101": "Ventilatiesystemen"});
        assert_eq!(
            topic("isso54-v2-EPW101c", topics.as_object().unwrap()),
            "Ventilatiesystemen"
        );
    }

    #[test]
    fn a_project_file_opens_as_the_app_opens_it() {
        let project = json!({"id": "p", "ntaCalculation": {"normVersion": "2022"}});
        let assessment =
            json!({"kernelVersion": "0.4.0", "targetNormVersion": "x", "inputFingerprint": "f"});
        let text = project_file(&project, &assessment);
        let file: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(file["type"], PROJECT_FILE_TYPE);
        assert_eq!(file["kernel"]["inputFingerprint"], "f");
        assert_eq!(open_project_file(&text).unwrap(), project);
        assert!(open_project_file(r#"{"type":"other","project":{}}"#).is_err());
        assert!(
            open_project_file(r#"{"type":"open-energy-studio","version":"2.0","project":{}}"#)
                .is_err()
        );
    }

    #[test]
    fn expected_values_give_a_verdict() {
        let assessment = json!({"status": "calculated_legacy_edition", "normVersion": "2022",
            "performance": {"needIndicatorKwhPerM2Year": 50.0, "energyByService": {"annual": [
                {"service": "heating", "carrier": "gas", "deliveredKwh": 100.0},
                {"service": "heating", "carrier": "el", "deliveredKwh": 5.0}]}}});
        let comparison = json!({"status": "compared_pass",
            "metrics": [{"path": "beng1", "expected": 50.2, "actual": 50.0, "withinTolerance": true}]});
        let row = row(
            "isso54-v2-EPW001",
            "t".into(),
            "f.oes.json",
            "{}",
            &assessment,
            Some(&comparison),
        );
        assert_eq!(row.delivered_kwh["heating"], 105.0);
        assert_eq!(row.verdict.as_deref(), Some("binnen de band"));
        let pending = super::row(
            "isso54-v2-EPW001",
            "t".into(),
            "f.oes.json",
            "{}",
            &assessment,
            None,
        );
        assert!(pending.verdict.is_none() && pending.expected.is_empty());
        assert!(csv(&[pending]).lines().nth(1).unwrap().ends_with(";;"));
    }

    // Review 32: the verdict is the comparison's, not a recount of the metrics.
    #[test]
    fn a_label_class_mismatch_fails_the_case() {
        let assessment = json!({"status": "calculated_legacy_edition", "performance": {}});
        let comparison = json!({"status": "compared_fail",
            "metrics": [{"path": "beng1", "expected": 50.0, "actual": 50.0, "withinTolerance": true}],
            "labelClass": {"path": "indicativeLabelClass", "expected": "A", "actual": "B", "matches": false}});
        let row = row(
            "c",
            String::new(),
            "f",
            "{}",
            &assessment,
            Some(&comparison),
        );
        assert_eq!(row.verdict.as_deref(), Some("buiten de band"));
        assert_eq!(row.expected[1].expected, json!("A"));
        assert!(!row.expected[1].within_tolerance);
        assert!(csv(std::slice::from_ref(&row)).contains("indicativeLabelClass=A"));
        assert!(markdown(&[row], &[]).contains("indicativeLabelClass A"));
    }

    #[test]
    fn a_case_with_expectations_that_did_not_calculate_is_not_pending() {
        let assessment = json!({"status": "incomplete"});
        let comparison = json!({"status": "calculation_unavailable", "metrics": []});
        let row = row(
            "c",
            String::new(),
            "f",
            "{}",
            &assessment,
            Some(&comparison),
        );
        assert_eq!(
            row.verdict.as_deref(),
            Some("niet vergeleken (calculation_unavailable)")
        );
        let pending = json!({"status": "pending_expectation", "metrics": []});
        assert!(
            super::row("c", String::new(), "f", "{}", &assessment, Some(&pending))
                .verdict
                .is_none()
        );
    }

    #[test]
    fn project_file_names_stay_in_the_dossier() {
        assert_eq!(
            project_file_name("isso54-v2-EPW101c").unwrap(),
            "isso54-v2-EPW101c.oes.json"
        );
        for bad in ["", "../x", "a/b", "a\\b", ".hidden", "a b"] {
            assert!(project_file_name(bad).is_err(), "{bad:?}");
        }
    }
}
