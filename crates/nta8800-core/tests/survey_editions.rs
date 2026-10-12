//! Every basisopname fixture under every NTA 8800 edition.
//!
//! NTA 8800:2022 and 2020+A1 take the actual height h of the crawlspace wall
//! above ground level in (8.47) (2022 p. 236); from 2023 h is the fixed
//! 0,125 m (p. 240). The survey asks it as H2 (ISSO 82.1 4e druk afb. 8.23,
//! p. 73; ISSO 75.1 4e druk the same §), `floorHeightAboveGroundM` on the floor
//! above the crawlspace. There is no forfait value for an unknown height, so
//! without it the survey names that answer; it never refuses on a field the
//! survey cannot set (`ground_floor_wall_height_required`).

use nta8800_core::opname::utility::{assess_utility_survey, UtilitySurvey};
use nta8800_core::opname::{assess_residential_survey, ResidentialSurvey};
use serde_json::Value;

const EDITIONS: [&str; 5] = ["2025+C1", "2024", "2023", "2022", "2020+A1"];
const MEASURED_EDITIONS: [&str; 2] = ["2022", "2020+A1"];

const RESIDENTIAL: [(&str, &str); 3] = [
    (
        "1930-terraced",
        include_str!("../../../training-data/nta8800-opname-1930-terraced.json"),
    ),
    (
        "1975-apartment",
        include_str!("../../../training-data/nta8800-opname-1975-apartment.json"),
    ),
    (
        "2015-detached",
        include_str!("../../../training-data/nta8800-opname-2015-detached.json"),
    ),
];

const UTILITY: [(&str, &str); 3] = [
    (
        "utility-1970-retail",
        include_str!("../../../training-data/nta8800-opname-utility-1970-retail.json"),
    ),
    (
        "utility-1985-office",
        include_str!("../../../training-data/nta8800-opname-utility-1985-office.json"),
    ),
    (
        "utility-2005-school",
        include_str!("../../../training-data/nta8800-opname-utility-2005-school.json"),
    ),
];

struct Outcome {
    status: String,
    codes: Vec<(String, String)>,
}

fn run(utility: bool, mut value: Value, edition: &str) -> Outcome {
    value["normVersion"] = Value::from(edition);
    let assessment = if utility {
        let survey: UtilitySurvey = serde_json::from_value(value).expect("utility survey");
        serde_json::to_value(assess_utility_survey(&survey)).unwrap()
    } else {
        let survey: ResidentialSurvey = serde_json::from_value(value).expect("residential survey");
        serde_json::to_value(assess_residential_survey(&survey)).unwrap()
    };
    let mut codes = Vec::new();
    collect_codes(&assessment, &mut codes);
    Outcome {
        status: assessment["status"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        codes,
    }
}

/// Every `{code, path}` anywhere in the assessment (survey issues and the
/// issues of the building calculation behind it).
fn collect_codes(value: &Value, out: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) => {
            if let Some(code) = map.get("code").and_then(Value::as_str) {
                let path = map
                    .get("path")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                out.push((code.to_string(), path));
            }
            for item in map.values() {
                collect_codes(item, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_codes(item, out)),
        _ => {}
    }
}

/// Indexes of floors above a crawlspace or unheated cellar.
fn crawl_floors(value: &Value) -> Vec<usize> {
    value["envelope"]["surfaces"]
        .as_array()
        .map(|surfaces| {
            surfaces
                .iter()
                .enumerate()
                .filter(|(_, surface)| {
                    matches!(
                        surface["boundary"]["kind"].as_str(),
                        Some("crawlspace" | "unheated_cellar")
                    )
                })
                .map(|(index, _)| index)
                .collect()
        })
        .unwrap_or_default()
}

fn calculated(status: &str) -> bool {
    status.starts_with("calculated")
}

fn fixtures() -> impl Iterator<Item = (bool, &'static str, Value)> {
    RESIDENTIAL
        .iter()
        .map(|(name, json)| (false, *name, serde_json::from_str(json).unwrap()))
        .chain(
            UTILITY
                .iter()
                .map(|(name, json)| (true, *name, serde_json::from_str(json).unwrap())),
        )
}

/// With H2 answered, every fixture calculates in every edition (2025+C1 as
/// `calculated_*`, older editions as the legacy status), and no edition
/// refuses on the wall height.
#[test]
fn every_survey_calculates_in_every_edition_with_h2() {
    for (utility, name, base) in fixtures() {
        let mut value = base.clone();
        for index in crawl_floors(&value) {
            value["envelope"]["surfaces"][index]["floorHeightAboveGroundM"] = Value::from(0.3);
        }
        for edition in EDITIONS {
            let outcome = run(utility, value.clone(), edition);
            assert!(
                calculated(&outcome.status),
                "{name} in {edition}: {} {:?}",
                outcome.status,
                outcome.codes
            );
            assert!(
                !outcome
                    .codes
                    .iter()
                    .any(|(code, _)| code == "ground_floor_wall_height_required"),
                "{name} in {edition}: {:?}",
                outcome.codes
            );
        }
    }
}

/// Without H2, a survey with a crawlspace floor names the survey answer in
/// 2022 and 2020+A1, and calculates in 2023 and later. Surveys without a
/// crawlspace calculate everywhere.
#[test]
fn missing_h2_names_the_survey_answer_only_where_the_edition_needs_it() {
    for (utility, name, value) in fixtures() {
        let floors = crawl_floors(&value);
        for edition in EDITIONS {
            let outcome = run(utility, value.clone(), edition);
            assert!(
                !outcome
                    .codes
                    .iter()
                    .any(|(code, _)| code == "ground_floor_wall_height_required"),
                "{name} in {edition}: {:?}",
                outcome.codes
            );
            if floors.is_empty() || !MEASURED_EDITIONS.contains(&edition) {
                assert!(
                    calculated(&outcome.status),
                    "{name} in {edition}: {} {:?}",
                    outcome.status,
                    outcome.codes
                );
                continue;
            }
            assert!(!calculated(&outcome.status), "{name} in {edition}");
            for index in &floors {
                let expected = format!("envelope.surfaces[{index}].floorHeightAboveGroundM");
                assert!(
                    outcome
                        .codes
                        .iter()
                        .any(|(code, path)| code == "floor_height_above_ground_required"
                            && path == &expected),
                    "{name} in {edition}: {:?}",
                    outcome.codes
                );
            }
        }
    }
}

/// From 2023 h is fixed: an answered H2 changes nothing.
#[test]
fn h2_does_not_change_the_result_from_2023() {
    for (utility, name, base) in fixtures() {
        let floors = crawl_floors(&base);
        if floors.is_empty() {
            continue;
        }
        let mut answered = base.clone();
        for index in &floors {
            answered["envelope"]["surfaces"][*index]["floorHeightAboveGroundM"] = Value::from(0.8);
        }
        for edition in ["2025+C1", "2024", "2023"] {
            let ep = |value: &Value| {
                let mut value = value.clone();
                value["normVersion"] = Value::from(edition);
                let assessment = if utility {
                    let survey: UtilitySurvey = serde_json::from_value(value).unwrap();
                    serde_json::to_value(assess_utility_survey(&survey)).unwrap()
                } else {
                    let survey: ResidentialSurvey = serde_json::from_value(value).unwrap();
                    serde_json::to_value(assess_residential_survey(&survey)).unwrap()
                };
                assessment["performance"].to_string()
            };
            assert_eq!(ep(&base), ep(&answered), "{name} in {edition}");
        }
    }
}

/// In 2022 a higher crawlspace wall above ground (8.47) loses more heat.
#[test]
fn h2_raises_the_heat_loss_in_2022() {
    let base: Value = serde_json::from_str(RESIDENTIAL[0].1).unwrap();
    let floors = crawl_floors(&base);
    assert!(!floors.is_empty());
    let heat = |height: f64| {
        let mut value = base.clone();
        for index in &floors {
            value["envelope"]["surfaces"][*index]["floorHeightAboveGroundM"] = Value::from(height);
        }
        value["normVersion"] = Value::from("2022");
        let survey: ResidentialSurvey = serde_json::from_value(value).unwrap();
        let assessment = serde_json::to_value(assess_residential_survey(&survey)).unwrap();
        assessment["performance"]["annualHeatingAndCoolingNeedKwh"]
            .as_f64()
            .unwrap_or_else(|| panic!("{}", assessment["performance"]))
    };
    assert!(heat(0.8) > heat(0.1));
}

/// ISSO 82.1 7e druk §8.4 (p. 74–75): a façade part against the ground with
/// H1 goes with its floor, which then calculates as the floor of a heated
/// room below ground level (NTA 8800 8.3.3.2) in every edition, on a floor
/// to the ground and on a floor above a crawlspace (afb. 8.9 and 8.11).
#[test]
fn a_facade_part_against_the_ground_calculates_with_its_floor_in_every_edition() {
    for (name, text) in [RESIDENTIAL[0], RESIDENTIAL[2]] {
        let base: Value = serde_json::from_str(text).unwrap();
        let surfaces = base["envelope"]["surfaces"].as_array().unwrap();
        let floor = surfaces
            .iter()
            .find(|surface| {
                surface["element"] == "floor"
                    && matches!(
                        surface["boundary"]["kind"].as_str(),
                        Some("ground" | "crawlspace" | "unheated_cellar")
                    )
            })
            .unwrap_or_else(|| panic!("{name}: no floor to the ground"))["id"]
            .clone();
        for edition in EDITIONS {
            let mut without = base.clone();
            if MEASURED_EDITIONS.contains(&edition) {
                for index in crawl_floors(&without) {
                    without["envelope"]["surfaces"][index]["floorHeightAboveGroundM"] =
                        Value::from(0.5);
                }
            }
            let mut with = without.clone();
            with["envelope"]["surfaces"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "id": "kelderwand", "element": "facade", "boundary": {"kind": "ground"},
                    "grossAreaM2": 20.0, "cavity": false, "insulation": {"kind": "none_or_unknown"},
                    "depthBelowGroundM": 1.2, "groundFloorId": floor,
                    "sourceReference": "synthetic survey: wall against the ground",
                }));
            let heat = |value: &Value| {
                let mut value = value.clone();
                value["normVersion"] = Value::from(edition);
                let survey: ResidentialSurvey = serde_json::from_value(value).unwrap();
                let assessment = serde_json::to_value(assess_residential_survey(&survey)).unwrap();
                let mut codes = Vec::new();
                collect_codes(&assessment, &mut codes);
                assert!(
                    calculated(assessment["status"].as_str().unwrap_or_default()),
                    "{name} in {edition}: {} {codes:?}",
                    assessment["status"]
                );
                assessment["performance"]["annualHeatingAndCoolingNeedKwh"]
                    .as_f64()
                    .unwrap()
            };
            assert!(heat(&with) > heat(&without), "{name} in {edition}");
        }
    }
}
