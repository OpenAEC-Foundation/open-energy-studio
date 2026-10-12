//! Cross-edition delta table (docs/nta8800-normversies.md, "Verschillen per
//! uitgave"): every public case, both examples and ISSO 54 EPW001/EPU001 in
//! all five editions. Run with
//! `cargo test --test edition_deltas -- --ignored --nocapture`.

use nta8800_core::project_performance::assess_project_performance;
use serde_json::{json, Value};

const PROJECTS: [(&str, &str); 19] = [
    (
        "A",
        include_str!("../../../training-data/nta8800-public-comparison-a.json"),
    ),
    (
        "B",
        include_str!("../../../training-data/nta8800-public-comparison-b.json"),
    ),
    (
        "C",
        include_str!("../../../training-data/nta8800-public-comparison-c.json"),
    ),
    (
        "D",
        include_str!("../../../training-data/nta8800-public-comparison-d.json"),
    ),
    (
        "E",
        include_str!("../../../training-data/nta8800-public-comparison-e.json"),
    ),
    (
        "F",
        include_str!("../../../training-data/nta8800-public-comparison-f.json"),
    ),
    (
        "G",
        include_str!("../../../training-data/nta8800-public-comparison-g.json"),
    ),
    (
        "H",
        include_str!("../../../training-data/nta8800-public-comparison-h.json"),
    ),
    (
        "I",
        include_str!("../../../training-data/nta8800-public-comparison-i.json"),
    ),
    (
        "J",
        include_str!("../../../training-data/nta8800-public-comparison-j.json"),
    ),
    (
        "K",
        include_str!("../../../training-data/nta8800-public-comparison-k.json"),
    ),
    (
        "L",
        include_str!("../../../training-data/nta8800-public-comparison-l.json"),
    ),
    (
        "M",
        include_str!("../../../training-data/nta8800-public-comparison-m.json"),
    ),
    (
        "N",
        include_str!("../../../training-data/nta8800-public-comparison-n.json"),
    ),
    (
        "O",
        include_str!("../../../training-data/nta8800-public-comparison-o.json"),
    ),
    (
        "woning",
        include_str!("../../../training-data/nta8800-example-terraced-dwelling.json"),
    ),
    (
        "kantoor",
        include_str!("../../../training-data/nta8800-example-office.json"),
    ),
    (
        "EPW001",
        include_str!("../../../training-data/isso54/EPW001.json"),
    ),
    (
        "EPU001",
        include_str!("../../../training-data/isso54/EPU001.json"),
    ),
];

pub const EDITIONS: [&str; 5] = ["2020+A1", "2022", "2023", "2024", "2025+C1"];

const SERVICES: [&str; 7] = [
    "heating",
    "hotWater",
    "cooling",
    "ventilation",
    "auxiliary",
    "lighting",
    "humidification",
];

#[derive(Debug, Clone)]
pub struct Row {
    pub beng: [f64; 3],
    pub need: f64,
    pub services: Vec<(String, f64)>,
}

thread_local! {
    static DROPPED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Removes the member at a gap path such as
/// `ntaCalculation.hotWater.storage[0].electricBoilerInsulatedPipe`.
fn remove_path(value: &mut Value, path: &str) -> bool {
    let mut steps: Vec<(String, Option<usize>)> = Vec::new();
    for part in path.split('.') {
        match part.split_once('[') {
            Some((key, rest)) => {
                steps.push((key.to_string(), None));
                for index in rest.trim_end_matches(']').split("][") {
                    steps.push((String::new(), index.parse().ok()));
                }
            }
            None => steps.push((part.to_string(), None)),
        }
    }
    let Some((last, last_index)) = steps.pop() else {
        return false;
    };
    let mut node = value;
    for (key, index) in steps {
        node = match index {
            Some(i) => match node.get_mut(i) {
                Some(next) => next,
                None => return false,
            },
            None => match node.get_mut(&key) {
                Some(next) => next,
                None => return false,
            },
        };
    }
    match (last_index, node) {
        (None, Value::Object(map)) => map.remove(&last).is_some(),
        (Some(i), Value::Array(items)) if i < items.len() => {
            items.remove(i);
            true
        }
        _ => false,
    }
}

/// The project in `edition`, made comparable: members of a route the edition
/// does not have are dropped, and the dated NEN-EN 14511-2 of a table 9.28
/// test report is set to the one the edition names (2022 p. 316; 2023
/// p. 322). Returns the assessment and what was changed.
fn portable(
    json: &str,
    edition: &str,
) -> (
    nta8800_core::project_performance::ProjectPerformanceAssessment,
    Vec<String>,
) {
    let mut value: Value = serde_json::from_str(json).unwrap();
    value["ntaCalculation"]["normVersion"] = json!(edition);
    let mut dropped = Vec::new();
    // EDITION_DROP=path;path removes members up front (attribution runs).
    if let Ok(paths) = std::env::var("EDITION_DROP") {
        for path in paths.split(';').filter(|path| !path.is_empty()) {
            if remove_path(&mut value, path) {
                dropped.push(format!("{path} (EDITION_DROP)"));
            }
        }
    }
    // EDITION_SET=/json/pointer=value;… sets members up front (probe runs).
    if let Ok(sets) = std::env::var("EDITION_SET") {
        for set in sets.split(';').filter(|set| !set.is_empty()) {
            let (pointer, raw) = set.split_once('=').unwrap();
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            if let Some(Value::Object(map)) = value.pointer_mut(parent) {
                map.insert(key.to_string(), serde_json::from_str(raw).unwrap());
                dropped.push(format!("{pointer} = {raw} (EDITION_SET)"));
            }
        }
    }
    let standard = if matches!(edition, "2020+A1" | "2022") {
        "NEN-EN 14511-2:2007"
    } else {
        "NEN-EN 14511-2:2022"
    };
    if let Some(evidence) = value
        .pointer_mut("/ntaCalculation/generator/forfait/highEfficiencyEvidence/testStandardEdition")
    {
        if evidence != standard {
            *evidence = json!(standard);
            dropped.push(format!("testStandardEdition → {standard}"));
        }
    }
    let mut result = assess_project_performance(&value);
    // Members of a route the edition does not have are refused with
    // route_not_in_edition; drop them so the rest of the input can still be
    // compared across editions, and list what was dropped.
    for _ in 0..8 {
        let routes: Vec<String> = result
            .gaps
            .iter()
            .filter(|gap| gap.code == "route_not_in_edition")
            .map(|gap| gap.path.clone())
            .collect();
        if routes.is_empty() {
            break;
        }
        for path in &routes {
            // A route switched on by a flag (ledFrom2017) is switched off.
            let pointer = format!("/{}", path.replace(['.', '['], "/").replace(']', ""));
            if let Some(flag @ Value::Bool(true)) = value.pointer_mut(&pointer) {
                *flag = Value::Bool(false);
                dropped.push(format!(
                    "{} = false",
                    path.trim_start_matches("ntaCalculation.")
                ));
                continue;
            }
            if remove_path(&mut value, path) {
                dropped.push(path.trim_start_matches("ntaCalculation.").to_string());
            }
        }
        result = assess_project_performance(&value);
    }
    (result, dropped)
}

thread_local! {
    static WARNINGS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn run(json: &str, edition: &str) -> Result<Row, String> {
    let (result, dropped) = portable(json, edition);
    DROPPED.with(|cell| *cell.borrow_mut() = dropped);
    let mut codes: Vec<String> = result
        .warnings
        .iter()
        .map(|warning| warning.code.to_string())
        .collect();
    if let Some(performance) = &result.performance {
        let p = serde_json::to_value(performance).unwrap();
        let mut out = Vec::new();
        collect_codes(&p, &mut out);
        codes.extend(out);
    }
    codes.sort();
    codes.dedup();
    WARNINGS.with(|cell| *cell.borrow_mut() = codes);
    let Some(performance) = result.performance.as_ref() else {
        let codes: Vec<String> = result
            .gaps
            .iter()
            .take(3)
            .map(|gap| format!("{gap:?}"))
            .collect();
        return Err(format!("{}: {}", result.status, codes.join("; ")));
    };
    let p = serde_json::to_value(performance).unwrap();
    if p.pointer("/primaryFossilIndicatorKwhPerM2Year")
        .and_then(Value::as_f64)
        .is_none()
    {
        let gaps: Vec<String> = result
            .gaps
            .iter()
            .take(3)
            .map(|gap| format!("{gap:?}"))
            .collect();
        let issues: String = p
            .pointer("/issues")
            .cloned()
            .unwrap_or(Value::Null)
            .to_string()
            .chars()
            .take(400)
            .collect();
        return Err(format!(
            "{} without BENG: {} | issues {issues}",
            result.status,
            gaps.join("; ")
        ));
    }
    let read = |key: &str| p.pointer(key).and_then(Value::as_f64).unwrap_or(f64::NAN);
    let mut services = Vec::new();
    for service in SERVICES {
        let used: f64 = p["energyByService"]["annual"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|entry| entry["service"] == service)
            .filter_map(|entry| entry["usedKwh"].as_f64())
            .sum();
        services.push((service.to_string(), used));
    }
    Ok(Row {
        beng: [
            read("/needIndicatorKwhPerM2Year"),
            read("/primaryFossilIndicatorKwhPerM2Year"),
            read("/renewableSharePercent"),
        ],
        need: read("/annualHeatingAndCoolingNeedKwh"),
        services,
    })
}

/// Every `code` in the output (issues and warnings of the chapters).
fn collect_codes(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(code)) = map.get("code") {
                out.push(code.clone());
            }
            for item in map.values() {
                collect_codes(item, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_codes(item, out)),
        _ => {}
    }
}

fn leaves(value: &Value, path: String, out: &mut Vec<(String, f64)>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                leaves(item, format!("{path}/{key}"), out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                leaves(item, format!("{path}/{index}"), out);
            }
        }
        Value::Number(number) => out.push((path, number.as_f64().unwrap_or(f64::NAN))),
        _ => {}
    }
}

/// Prints every annual numeric output that differs between two editions for
/// one case: `EDITION_DIFF=K,2020+A1,2022 cargo test --test edition_deltas
/// -- --ignored --nocapture diff`.
#[test]
#[ignore = "prints the outputs that differ between two editions"]
fn diff_two_editions() {
    let Ok(spec) = std::env::var("EDITION_DIFF") else {
        return;
    };
    let parts: Vec<&str> = spec.split(',').collect();
    let json = PROJECTS
        .iter()
        .find(|(name, _)| *name == parts[0])
        .unwrap()
        .1;
    let filter = parts.get(3).copied().unwrap_or("");
    let outputs: Vec<Vec<(String, f64)>> = [parts[1], parts[2]]
        .iter()
        .map(|edition| {
            let (result, dropped) = portable(json, edition);
            println!("{edition}: {} dropped {dropped:?}", result.status);
            let mut out = Vec::new();
            leaves(
                &serde_json::to_value(&result).unwrap(),
                String::new(),
                &mut out,
            );
            out
        })
        .collect();
    let second: std::collections::HashMap<&str, f64> = outputs[1]
        .iter()
        .map(|(path, number)| (path.as_str(), *number))
        .collect();
    for (path, a) in &outputs[0] {
        let monthly = path.contains("/months/") || path.contains("/monthly");
        if !path.contains(filter) || (monthly && parts.get(4) != Some(&"months")) {
            continue;
        }
        if let Some(b) = second.get(path.as_str()) {
            if (a - b).abs() > 1e-6 * a.abs().max(1.0) {
                println!("{path}: {a:.4} -> {b:.4}");
            }
        }
    }
}

/// Prints the issues and gaps of one case in one edition:
/// `EDITION_PROBE=D,2023 cargo test --test edition_deltas -- --ignored probe`.
#[test]
#[ignore = "prints the issues of one case in one edition"]
fn probe_one_edition() {
    let Ok(spec) = std::env::var("EDITION_PROBE") else {
        return;
    };
    let (name, edition) = spec.split_once(',').unwrap();
    let json = PROJECTS.iter().find(|(case, _)| *case == name).unwrap().1;
    let (result, dropped) = portable(json, edition);
    println!("{} dropped {dropped:?}", result.status);
    let value = serde_json::to_value(&result).unwrap();
    for key in [
        "/gaps",
        "/warnings",
        "/performance/issues",
        "/performance/spaceHeating/issues",
    ] {
        println!(
            "{key}: {}",
            value.pointer(key).cloned().unwrap_or(Value::Null)
        );
    }
}

#[test]
#[ignore = "prints the cross-edition delta table"]
fn print_edition_delta_table() {
    for (name, json) in PROJECTS {
        println!("\n== {name}");
        let mut previous: Option<Row> = None;
        let mut previous_codes: Option<Vec<String>> = None;
        for edition in EDITIONS {
            match run(json, edition) {
                Ok(row) => {
                    let services: Vec<String> = row
                        .services
                        .iter()
                        .filter(|(_, kwh)| *kwh != 0.0)
                        .map(|(service, kwh)| format!("{service} {kwh:.0}"))
                        .collect();
                    let mut flags = String::new();
                    if let Some(prev) = &previous {
                        for (i, label) in ["B1", "B2", "B3"].iter().enumerate() {
                            let base = prev.beng[i].abs().max(1.0);
                            let delta = (row.beng[i] - prev.beng[i]) / base * 100.0;
                            if delta.abs() > 0.5 {
                                flags.push_str(&format!(" {label}{delta:+.1}%"));
                            }
                        }
                    }
                    println!(
                        "{edition:8} {:7.2} / {:7.2} / {:6.1} | Q_nd {:7.0} | {}{}",
                        row.beng[0],
                        row.beng[1],
                        row.beng[2],
                        row.need,
                        services.join(", "),
                        if flags.is_empty() {
                            String::new()
                        } else {
                            format!(" |{flags}")
                        }
                    );
                    DROPPED.with(|cell| {
                        let dropped = cell.borrow();
                        if !dropped.is_empty() {
                            println!("         dropped (not in edition): {}", dropped.join(", "));
                        }
                    });
                    let codes = WARNINGS.with(|cell| cell.borrow().clone());
                    if let Some(prev) = &previous_codes {
                        let added: Vec<&String> =
                            codes.iter().filter(|code| !prev.contains(code)).collect();
                        let removed: Vec<&String> =
                            prev.iter().filter(|code| !codes.contains(code)).collect();
                        if !added.is_empty() || !removed.is_empty() {
                            println!("         codes +{added:?} -{removed:?}");
                        }
                    }
                    previous_codes = Some(codes);
                    previous = Some(row);
                }
                Err(error) => {
                    println!("{edition:8} not calculated: {error}");
                    previous = None;
                    previous_codes = None;
                }
            }
        }
    }
}
