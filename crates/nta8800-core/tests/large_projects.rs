//! Large projects: a deterministic generator scales the public utility case H
//! (two rekenzones, 18 windows) to many zones and checks that the project
//! still calculates, stays finite and scales roughly linearly.
//!
//! Every copy of the zones gets fresh ids (`<id>~<n>`), together with the
//! per-zone entries of `ntaCalculation` that point at those ids (zone data,
//! ground floors, surface tilts, window obstructions, lighting). The gate runs
//! the sizes up to 20 zones; the timing table for 40 and 80 zones is an
//! ignored benchmark:
//!
//! ```text
//! cargo test --release --offline --test large_projects -- --ignored --nocapture
//! ```

use nta8800_core::finite::first_non_finite;
use nta8800_core::project_performance::assess_project_performance;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::{Duration, Instant};

const CASE_H: &str = include_str!("../../../training-data/nta8800-public-comparison-h.json");
const TERRACED: &str =
    include_str!("../../../training-data/nta8800-example-terraced-dwelling.json");

/// Arrays in `ntaCalculation` whose entries belong to one zone, surface or
/// window and are copied with it.
const PER_ZONE_ARRAYS: [&str; 5] = [
    "zoneData",
    "groundFloors",
    "surfaceTilts",
    "windowObstructions",
    "lighting",
];

fn element_ids(project: &Value) -> HashSet<String> {
    let mut ids = HashSet::new();
    for zone in project["zones"].as_array().unwrap() {
        ids.insert(zone["id"].as_str().unwrap().to_string());
        for surface in zone["surfaces"].as_array().unwrap() {
            ids.insert(surface["id"].as_str().unwrap().to_string());
            for window in surface["windows"].as_array().into_iter().flatten() {
                ids.insert(window["id"].as_str().unwrap().to_string());
            }
        }
        for bridge in zone["thermalBridges"].as_array().into_iter().flatten() {
            if let Some(id) = bridge["id"].as_str() {
                ids.insert(id.to_string());
            }
        }
    }
    if let Some(zone_data) = project
        .pointer("/ntaCalculation/zoneData")
        .and_then(Value::as_array)
    {
        for entry in zone_data {
            for pipe in entry["verticalPipes"].as_array().into_iter().flatten() {
                if let Some(id) = pipe["id"].as_str() {
                    ids.insert(id.to_string());
                }
            }
        }
    }
    ids
}

/// Renames every string that is one of `ids` to `<id>~<copy>`.
fn rename(value: &Value, ids: &HashSet<String>, copy: usize) -> Value {
    match value {
        Value::String(text) if ids.contains(text) => Value::String(format!("{text}~{copy}")),
        Value::Array(items) => Value::Array(items.iter().map(|v| rename(v, ids, copy)).collect()),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, v)| (key.clone(), rename(v, ids, copy)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Areas in the project-wide blocks (hot-water need per function, PV panel
/// area, the Bbl functions) that grow with the building.
const PROJECT_AREA_KEYS: [&str; 3] = ["areaM2", "panelAreaM2", "usableFloorAreaM2"];

fn scale_areas(value: &mut Value, factor: f64) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                match v {
                    Value::Number(n) if PROJECT_AREA_KEYS.contains(&key.as_str()) => {
                        *v = Value::from(n.as_f64().unwrap() * factor);
                    }
                    _ => scale_areas(v, factor),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|v| scale_areas(v, factor)),
        _ => {}
    }
}

/// Case H with its zones repeated `copies` times (copy 0 is the original).
/// The project-wide areas grow with it, so the per-m² indicators stay those
/// of case H.
pub fn scaled_case_h(copies: usize) -> Value {
    scaled(serde_json::from_str(CASE_H).unwrap(), copies)
}

/// The terraced example dwelling repeated as `copies` dwellings, one rekenzone
/// each. A project with more than one zone needs zone data; each copy gets
/// the dwelling's own ventilation flows, internal gains and ventilation.
pub fn scaled_dwellings(copies: usize) -> Value {
    let mut base: Value = serde_json::from_str(TERRACED).unwrap();
    if copies > 1 {
        let block = &base["ntaCalculation"];
        let entry = json!({
            "zoneId": base["zones"][0]["id"],
            "verticalPipes": block["verticalPipes"],
            "ventilationFlows": block["ventilationFlows"],
            "internalGains": block["internalGains"],
            "ventilation": block["ventilation"],
        });
        base["ntaCalculation"]["zoneData"] = json!([entry]);
        base["ntaCalculation"]
            .as_object_mut()
            .unwrap()
            .remove("verticalPipes");
    }
    let mut project = scaled(base, copies);
    if copies == 1 {
        return project;
    }
    // 6.2b (p. 160): dwellingCount is N_woon of the building; each zone takes
    // its share by area.
    let dwellings = Value::from(copies as u64);
    project["ntaCalculation"]["internalGains"]["dwellingCount"] = dwellings.clone();
    for entry in project["ntaCalculation"]["zoneData"]
        .as_array_mut()
        .into_iter()
        .flatten()
    {
        entry["internalGains"]["dwellingCount"] = dwellings.clone();
    }
    project
}

fn scaled(base: Value, copies: usize) -> Value {
    let ids = element_ids(&base);
    let mut project = base.clone();
    for copy in 1..copies {
        let zones = base["zones"].as_array().unwrap();
        let target = project["zones"].as_array_mut().unwrap();
        target.extend(zones.iter().map(|zone| rename(zone, &ids, copy)));
        for key in PER_ZONE_ARRAYS {
            let Some(entries) = base["ntaCalculation"][key].as_array() else {
                continue;
            };
            let renamed: Vec<Value> = entries.iter().map(|e| rename(e, &ids, copy)).collect();
            project["ntaCalculation"][key]
                .as_array_mut()
                .unwrap()
                .extend(renamed);
        }
    }
    if copies > 1 {
        let nta = project["ntaCalculation"].as_object_mut().unwrap();
        for (key, block) in nta.iter_mut() {
            if !PER_ZONE_ARRAYS.contains(&key.as_str()) {
                scale_areas(block, copies as f64);
            }
        }
    }
    project
}

struct Run {
    zones: usize,
    windows: usize,
    status: String,
    elapsed: Duration,
    output_bytes: usize,
    indicators: [f64; 3],
}

fn window_count(project: &Value) -> usize {
    project["zones"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|zone| zone["surfaces"].as_array().unwrap())
        .map(|surface| surface["windows"].as_array().map_or(0, Vec::len))
        .sum()
}

fn run(copies: usize) -> Run {
    measure(&scaled_case_h(copies))
}

fn measure(project: &Value) -> Run {
    let start = Instant::now();
    let result = assess_project_performance(project);
    let elapsed = start.elapsed();
    let output = serde_json::to_value(&result).unwrap();
    assert!(first_non_finite(&output).is_none(), "non-finite output");
    let output_bytes = serde_json::to_vec(&output).unwrap().len();
    let read = |key: &str| {
        output
            .pointer(&format!("/performance{key}"))
            .and_then(Value::as_f64)
            .unwrap_or(f64::NAN)
    };
    Run {
        zones: project["zones"].as_array().unwrap().len(),
        windows: window_count(project),
        status: result.status.to_string(),
        elapsed,
        output_bytes,
        indicators: [
            read("/needIndicatorKwhPerM2Year"),
            read("/primaryFossilIndicatorKwhPerM2Year"),
            read("/renewableSharePercent"),
        ],
    }
}

/// Peak resident memory of this test process in kB (Linux `VmHWM`), or 0.
fn peak_memory_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("VmHWM:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|kb| kb.parse().ok())
        })
        .unwrap_or(0)
}

fn used_kwh(project: &Value, service: &str) -> f64 {
    let result = assess_project_performance(project);
    let performance = serde_json::to_value(result.performance.as_ref().unwrap()).unwrap();
    performance["energyByService"]["annual"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["service"] == service)
        .filter_map(|row| row["usedKwh"].as_f64())
        .sum()
}

/// Repeating every zone n times keeps the energy need per m² (BENG 1) and
/// multiplies the posts that are calculated per zone (heating, ventilation,
/// lighting) by exactly n. Hot water, cooling and auxiliary energy grow
/// less than n: they contain amounts per system rather than per m², such as
/// the storage-vessel loss and the 87,6 kWh control energy of 10.87.
#[test]
fn scaled_utility_building_scales_its_zone_posts_linearly() {
    let base = run(1);
    assert!(base.status.starts_with("calculated"), "{}", base.status);
    let base_project = scaled_case_h(1);
    for copies in [5, 10] {
        let scaled = run(copies);
        assert_eq!(scaled.zones, 2 * copies);
        assert!(
            scaled.status.starts_with("calculated"),
            "{copies} copies: {}",
            scaled.status
        );
        assert!(
            (base.indicators[0] - scaled.indicators[0]).abs() < 0.01,
            "{copies} copies: BENG 1 {} against {}",
            scaled.indicators[0],
            base.indicators[0]
        );
        let project = scaled_case_h(copies);
        for service in ["heating", "ventilation", "lighting"] {
            let one = used_kwh(&base_project, service);
            let many = used_kwh(&project, service);
            assert!(
                (many - copies as f64 * one).abs() < 1e-6 * many.max(1.0),
                "{copies} copies, {service}: {many} against {copies} × {one}"
            );
        }
    }
}

/// Twenty zones and 180 windows calculate within a generous budget in a
/// debug build; the release timings are in the ignored benchmark.
#[test]
fn twenty_zones_calculate_within_budget() {
    let scaled = run(10);
    assert_eq!(scaled.zones, 20);
    assert_eq!(scaled.windows, 180);
    let budget = if cfg!(debug_assertions) {
        Duration::from_secs(30)
    } else {
        Duration::from_secs(3)
    };
    assert!(
        scaled.elapsed < budget,
        "20 zones took {:?}",
        scaled.elapsed
    );
}

/// Fifty dwellings of one rekenzone each calculate, and the need per m²
/// and the heating per dwelling are those of the single dwelling.
#[test]
fn fifty_dwellings_calculate_like_one() {
    let one = measure(&scaled_dwellings(1));
    let fifty = measure(&scaled_dwellings(50));
    assert_eq!(fifty.zones, 50);
    assert!(fifty.status.starts_with("calculated"), "{}", fifty.status);
    assert!(
        (one.indicators[0] - fifty.indicators[0]).abs() < 0.01,
        "BENG 1 {} against {}",
        fifty.indicators[0],
        one.indicators[0]
    );
    let heating_one = used_kwh(&scaled_dwellings(1), "heating");
    let heating_fifty = used_kwh(&scaled_dwellings(50), "heating");
    assert!(
        (heating_fifty - 50.0 * heating_one).abs() < 1e-6 * heating_fifty,
        "{heating_fifty} against 50 × {heating_one}"
    );
}

#[test]
#[ignore = "benchmark; run with --release --ignored --nocapture"]
fn benchmark_large_projects() {
    println!("| project | zones | windows | status | time (ms) | output (kB) | peak memory (MB) |");
    println!("|---|---:|---:|---|---:|---:|---:|");
    let row = |name: &str, project: Value| {
        let run = measure(&project);
        println!(
            "| {name} | {} | {} | {} | {:.1} | {} | {} |",
            run.zones,
            run.windows,
            run.status,
            run.elapsed.as_secs_f64() * 1000.0,
            run.output_bytes / 1024,
            peak_memory_kb() / 1024
        );
    };
    for copies in [1, 10, 20, 40, 100] {
        row("utiliteit (geval H)", scaled_case_h(copies));
    }
    for copies in [1, 10, 50, 100, 200] {
        row("woningen (rijwoning)", scaled_dwellings(copies));
    }
}
