//! A null object member in the NTA block means the same as an absent one
//! (review 9): a field the form clears keeps the kernel default, and the
//! input fingerprint, which ignores null members, stays truthful. For every
//! scalar member of the example projects whose removal is accepted, setting
//! it to null must give the same status, the same fingerprint and the same
//! indicators as removing it. The one deliberate exception is
//! `nominalThermalCapacityKw` (`EXPLICIT_NULL_MEMBERS`).

use nta8800_core::project_performance::assess_project_performance;
use nta8800_core::EXPLICIT_NULL_MEMBERS;
use serde_json::Value;

fn outcome(project: &Value) -> (String, String, String) {
    let result = serde_json::to_value(assess_project_performance(project)).unwrap();
    (
        result["status"].as_str().unwrap_or("").to_string(),
        result["inputFingerprint"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        result["performance"]["indicators"].to_string(),
    )
}

/// JSON pointers (and keys) of the scalar object members under a value.
fn scalar_members(value: &Value, pointer: String, out: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                let child = format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1"));
                if item.is_object() || item.is_array() {
                    scalar_members(item, child, out);
                } else if !item.is_null() {
                    out.push((child, key.clone()));
                }
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                scalar_members(item, format!("{pointer}/{index}"), out);
            }
        }
        _ => {}
    }
}

fn remove(project: &mut Value, pointer: &str) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    let key = key.replace("~1", "/").replace("~0", "~");
    project
        .pointer_mut(parent)
        .and_then(Value::as_object_mut)
        .unwrap()
        .remove(&key);
}

fn check(fixture: &str) {
    let path = format!(
        "{}/../../training-data/{fixture}",
        env!("CARGO_MANIFEST_DIR")
    );
    let project: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let base = outcome(&project);
    let mut members = Vec::new();
    scalar_members(
        &project["ntaCalculation"],
        "/ntaCalculation".into(),
        &mut members,
    );
    let mut checked = 0;
    let mut differ = Vec::new();
    for (pointer, key) in members {
        if EXPLICIT_NULL_MEMBERS.contains(&key.as_str()) {
            continue;
        }
        let mut absent = project.clone();
        remove(&mut absent, &pointer);
        let removed = outcome(&absent);
        // Only members whose removal the kernel accepts as before (a default
        // or an optional member): a required member is a blank either way.
        if removed.0 != base.0 {
            continue;
        }
        let mut blank = project.clone();
        *blank.pointer_mut(&pointer).unwrap() = Value::Null;
        let nulled = outcome(&blank);
        checked += 1;
        if nulled != removed {
            differ.push(format!(
                "{pointer}: null {:?} vs absent {:?}",
                nulled.0, removed.0
            ));
        }
    }
    assert!(checked >= 5, "{fixture}: only {checked} members checked");
    assert!(
        differ.is_empty(),
        "{fixture}: null differs from absent:\n{}",
        differ.join("\n")
    );
}

#[test]
fn null_members_read_as_absent_in_the_office() {
    check("nta8800-example-office.json");
}

#[test]
fn null_members_read_as_absent_in_the_terraced_dwelling() {
    check("nta8800-example-terraced-dwelling.json");
}

/// The defaulted leaves review 9 found the forms can blank, set to null where
/// their parent object exists in a fixture: each must read as absent.
const REVIEW9_LEAVES: [&str; 12] = [
    "/ntaCalculation/pvSystems/0/mounting",
    "/ntaCalculation/verticalPipes/0/sharedZones",
    "/ntaCalculation/cooling/emission/fanCoilCount",
    "/ntaCalculation/hotWater/solar/0/count",
    "/ntaCalculation/spaceHeatingSolar/0/count",
    "/ntaCalculation/sunrooms/0/distributionFactor",
    "/ntaCalculation/ventilation/fans/buildingShare",
    "/ntaCalculation/ventilation/installedCapacity/naturalSupplyDm3PerS",
    "/ntaCalculation/ventilation/dwellingCount",
    "/ntaCalculation/ventilation/apartmentBuilding",
    "/ntaCalculation/hotWater/boilingWaterTap",
    "/ntaCalculation/permitApplicationAfter20260529",
];

#[test]
fn review9_defaulted_leaves_read_as_absent() {
    let mut checked = 0;
    for fixture in [
        "nta8800-example-office.json",
        "nta8800-example-terraced-dwelling.json",
    ] {
        let path = format!(
            "{}/../../training-data/{fixture}",
            env!("CARGO_MANIFEST_DIR")
        );
        let project: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        for pointer in REVIEW9_LEAVES {
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            if !project.pointer(parent).is_some_and(Value::is_object) {
                continue;
            }
            let mut absent = project.clone();
            remove(&mut absent, pointer);
            let mut blank = absent.clone();
            blank
                .pointer_mut(parent)
                .and_then(Value::as_object_mut)
                .unwrap()
                .insert(key.to_string(), Value::Null);
            let absent_outcome = outcome(&absent);
            assert_eq!(outcome(&blank), absent_outcome, "{fixture}: {pointer}");
            if absent_outcome.0.starts_with("calculated") {
                checked += 1;
            }
        }
    }
    assert!(checked >= 6, "only {checked} review 9 leaves had a parent");
}
