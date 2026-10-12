//! Controls on the ISSO 54 v2.0 encodings (training-data/isso54/).
//!
//! EP-W203l-n split the reference dwelling of EP-W001 into two calculation
//! zones of one storey each. The split itself must not change the result:
//! with the reference HR107 of EP-W001 the two-zone dwelling calculates as
//! EP-W001 (review 19, D1: a 2-storey pipe per zone and a 48 m2 dwelling
//! per zone added about 8 kWh/m2 to BENG 2).

mod support;

use nta8800_core::project_performance::assess_project_performance;
use serde_json::Value;

const EPW001: &str = include_str!("../../../training-data/isso54/EPW001.json");
const CONTROL: &str = include_str!("../../../training-data/isso54/EPW001-two-zones-control.json");

fn run_2022(project: &Value) -> Value {
    let mut project = project.clone();
    project["ntaCalculation"]["normVersion"] = Value::from("2022");
    let result = assess_project_performance(&project);
    assert_eq!(
        result.status, "calculated_legacy_edition",
        "{:?}",
        result.gaps
    );
    serde_json::to_value(result.performance.as_ref().unwrap()).unwrap()
}

fn used_kwh(performance: &Value, service: &str) -> f64 {
    performance["energyByService"]["annual"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["service"] == service)
        .map(|row| row["usedKwh"].as_f64().unwrap())
        .sum()
}

#[test]
fn two_zone_split_alone_calculates_as_epw001() {
    let reference: Value = serde_json::from_str(EPW001).unwrap();
    let control: Value = serde_json::from_str(CONTROL).unwrap();
    let mut split = reference.clone();
    for step in control["projectPatch"].as_array().unwrap() {
        let pointer = step["pointer"].as_str().unwrap();
        match step["op"].as_str().unwrap() {
            "set" => support::set_pointer(&mut split, pointer, step["value"].clone()),
            "remove" => support::remove_pointer(&mut split, pointer),
            other => panic!("unknown op {other}"),
        }
    }
    assert_eq!(split["zones"].as_array().unwrap().len(), 2);

    let one = run_2022(&reference);
    let two = run_2022(&split);
    let read = |p: &Value, key: &str| p.pointer(key).and_then(Value::as_f64).unwrap();
    for key in [
        "/needIndicatorKwhPerM2Year",
        "/primaryFossilIndicatorKwhPerM2Year",
    ] {
        let (a, b) = (read(&one, key), read(&two, key));
        assert!(
            (a - b).abs() <= 0.005 * a.abs(),
            "{key}: EP-W001 {a}, two zones {b}"
        );
    }
    let (a, b) = (used_kwh(&one, "ventilation"), used_kwh(&two, "ventilation"));
    assert!(
        (a - b).abs() <= 0.01 * a,
        "ventilation: EP-W001 {a} kWh, two zones {b} kWh"
    );
}
