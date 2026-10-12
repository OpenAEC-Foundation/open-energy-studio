//! Every edition a result names is the edition it was calculated in: the
//! source citations, the stamps and the notes of a project run in one edition
//! never name another as "NTA 8800:<edition>" (BRL 9501 traceability). A
//! comparison such as "17.2 equal to 2025+C1" stays allowed: it does not cite.

use nta8800_core::norm_versions::NormVersion;
use nta8800_core::opname::utility::{assess_utility_survey, UtilitySurvey};
use nta8800_core::opname::{assess_residential_survey, ResidentialSurvey};
use nta8800_core::project_performance::assess_project_performance;
use serde_json::{json, Value};
use std::collections::HashSet;

const PROJECTS: &[&str] = &[
    "nta8800-example-terraced-dwelling.json",
    "nta8800-example-office.json",
    "nta8800-evidence-link-project.json",
    "nta8800-public-comparison-a.json",
    "nta8800-public-comparison-b.json",
    "nta8800-public-comparison-c.json",
    "nta8800-public-comparison-d.json",
    "nta8800-public-comparison-e.json",
    "nta8800-public-comparison-f.json",
    "nta8800-public-comparison-g.json",
    "nta8800-public-comparison-h.json",
    "nta8800-public-comparison-i.json",
    "nta8800-public-comparison-j.json",
    "nta8800-public-comparison-k.json",
    "nta8800-public-comparison-l.json",
    "nta8800-public-comparison-m.json",
    "nta8800-public-comparison-n.json",
    "nta8800-public-comparison-o.json",
    "isso54/EPW001.json",
    "isso54/EPU001.json",
];

fn read(name: &str) -> Value {
    let path = format!("{}/../../training-data/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

fn strip_nulls(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, item)| !item.is_null())
                .map(|(key, item)| (key, strip_nulls(item)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(strip_nulls).collect()),
        other => other,
    }
}

/// The year of every "NTA 8800:<year>…" citation in `text`.
fn cited_years(text: &str) -> Vec<&str> {
    text.match_indices("NTA 8800:")
        .filter_map(|(at, marker)| text.get(at + marker.len()..at + marker.len() + 4))
        .collect()
}

fn strings<'a>(value: &'a Value, out: &mut HashSet<&'a str>) {
    match value {
        Value::Object(map) => map.values().for_each(|item| strings(item, out)),
        Value::Array(items) => items.iter().for_each(|item| strings(item, out)),
        Value::String(text) => {
            out.insert(text);
        }
        _ => {}
    }
}

/// Each string of `value` citing another edition than `version`, by path.
/// Strings of the input itself (an adviser's source reference, echoed in the
/// derived input) are the adviser's words and are left out.
fn foreign_citations(
    value: &Value,
    version: NormVersion,
    input: &HashSet<&str>,
    path: &str,
    out: &mut Vec<String>,
) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                foreign_citations(item, version, input, &format!("{path}/{key}"), out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                foreign_citations(item, version, input, &format!("{path}/{index}"), out);
            }
        }
        Value::String(text) if !input.contains(text.as_str()) => {
            let own = cited_years(version.label())[0];
            if cited_years(text).iter().any(|year| *year != own) {
                out.push(format!("{path}: {text}"));
            }
        }
        _ => {}
    }
}

fn run(project: &Value, version: NormVersion) -> Vec<String> {
    let mut project = strip_nulls(project.clone());
    project["ntaCalculation"]["normVersion"] = json!(version);
    let result = serde_json::to_value(assess_project_performance(&project)).unwrap();
    let mut input = HashSet::new();
    strings(&project, &mut input);
    let mut found = Vec::new();
    foreign_citations(&result, version, &input, "", &mut found);
    found
}

/// Answers H2 (h 0,3 m) on every floor above a crawlspace.
fn answer_h2(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map
                .get("boundary")
                .and_then(|boundary| boundary.get("kind"))
                .is_some_and(|kind| kind == "crawlspace")
            {
                map.insert("floorHeightAboveGroundM".into(), json!(0.3));
            }
            map.values_mut().for_each(answer_h2);
        }
        Value::Array(items) => items.iter_mut().for_each(answer_h2),
        _ => {}
    }
}

/// A basisopname in every edition: the survey's own defaults and notes too.
/// The residential surveys also answer H2 (the crawlspace wall height),
/// which 2022 and 2020+A1 cite with their own source.
#[test]
fn a_survey_names_only_the_edition_it_was_calculated_in() {
    const SURVEYS: [(&str, bool); 6] = [
        ("nta8800-opname-1930-terraced.json", false),
        ("nta8800-opname-1975-apartment.json", false),
        ("nta8800-opname-2015-detached.json", false),
        ("nta8800-opname-utility-1970-retail.json", true),
        ("nta8800-opname-utility-1985-office.json", true),
        ("nta8800-opname-utility-2005-school.json", true),
    ];
    let mut found = Vec::new();
    for (name, utility) in SURVEYS {
        for version in NormVersion::ALL {
            let mut survey = read(name);
            survey["normVersion"] = json!(version);
            answer_h2(&mut survey);
            let result = if utility {
                let survey: UtilitySurvey = serde_json::from_value(survey.clone()).unwrap();
                serde_json::to_value(assess_utility_survey(&survey)).unwrap()
            } else {
                let survey: ResidentialSurvey = serde_json::from_value(survey.clone()).unwrap();
                serde_json::to_value(assess_residential_survey(&survey)).unwrap()
            };
            let mut input = HashSet::new();
            strings(&survey, &mut input);
            let mut items = Vec::new();
            foreign_citations(&result, version, &input, "", &mut items);
            found.extend(
                items
                    .into_iter()
                    .map(|item| format!("{name} in {}: {item}", version.id())),
            );
            if version == NormVersion::V2020A1 && name.ends_with("1930-terraced.json") {
                // The H2 default was recorded, with the 2020+A1 source.
                assert!(result
                    .to_string()
                    .contains("NTA 8800:2020+A1:2020, hoofdstuk 8 (h van de kruipruimtewand)"));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_project_names_only_the_edition_it_was_calculated_in() {
    let mut found = Vec::new();
    for name in PROJECTS {
        let project = read(name);
        for version in NormVersion::ALL {
            found.extend(
                run(&project, version)
                    .into_iter()
                    .map(|item| format!("{name} in {}: {item}", version.id())),
            );
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// The ISSO 54 deeltests cover generators the projects above do not (gas
/// heat pumps, micro-CHP, collective sources, solar water heaters, …); they
/// run in their own edition, 2022.
#[test]
fn isso54_deeltests_name_only_their_own_edition() {
    let suite = read("reference-suites/isso54-v2.json");
    let mut found = Vec::new();
    for case in suite["cases"].as_array().unwrap() {
        let file = case["projectFile"]
            .as_str()
            .unwrap()
            .trim_start_matches("../");
        let mut project = read(file);
        for step in case["projectPatch"].as_array().unwrap() {
            let pointer = step["pointer"].as_str().unwrap();
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            let object = project
                .pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap();
            match step["op"].as_str().unwrap() {
                "set" => {
                    object.insert(key.to_string(), step["value"].clone());
                }
                _ => {
                    object.remove(key);
                }
            }
        }
        let version: NormVersion = serde_json::from_value(case["normVersion"].clone()).unwrap();
        found.extend(
            run(&project, version)
                .into_iter()
                .map(|item| format!("{}: {item}", case["caseId"].as_str().unwrap())),
        );
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn citation_years_are_read_from_the_label() {
    assert_eq!(
        cited_years("NTA 8800:2022, hoofdstuk 9; NTA 8800:2024 met INT-V1:2024"),
        ["2022", "2024"]
    );
    assert!(cited_years("17.2 equal to 2025+C1").is_empty());
}
