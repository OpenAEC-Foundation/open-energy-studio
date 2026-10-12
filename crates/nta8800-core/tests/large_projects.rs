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
use nta8800_core::opname::utility::{assess_utility_survey, UtilitySurvey};
use nta8800_core::project_performance::assess_project_performance;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::{Duration, Instant};

const CASE_H: &str = include_str!("../../../training-data/nta8800-public-comparison-h.json");
const TERRACED: &str =
    include_str!("../../../training-data/nta8800-example-terraced-dwelling.json");
const OFFICE_SURVEY: &str =
    include_str!("../../../training-data/nta8800-opname-utility-1985-office.json");

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
    // One boiler per dwelling (§9.1 identical systems): each serves the
    // area of one dwelling, so none is a system for more than 500 m² (§9.6).
    project["ntaCalculation"]["identicalSystems"] = dwellings.clone();
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

/// Splits every window of the project into `parts` equal windows
/// (`<id>.<n>`), so the glazed area and the result stay those of the
/// original. Window-level entries (obstructions) follow their window.
fn split_windows(project: &mut Value, parts: usize) {
    let mut children: std::collections::HashMap<String, Vec<String>> = Default::default();
    for zone in project["zones"].as_array_mut().unwrap() {
        for surface in zone["surfaces"].as_array_mut().unwrap() {
            let Some(windows) = surface["windows"].as_array_mut() else {
                continue;
            };
            let mut split = Vec::with_capacity(windows.len() * parts);
            for window in windows.iter() {
                let id = window["id"].as_str().unwrap().to_string();
                let area = window["area"].as_f64().unwrap() / parts as f64;
                let ids = children.entry(id.clone()).or_default();
                for part in 0..parts {
                    let mut piece = window.clone();
                    let piece_id = format!("{id}.{part}");
                    piece["id"] = Value::from(piece_id.clone());
                    piece["area"] = Value::from(area);
                    ids.push(piece_id);
                    split.push(piece);
                }
            }
            *windows = split;
        }
    }
    let nta = &mut project["ntaCalculation"];
    if let Some(entries) = nta["windowObstructions"].as_array() {
        let mut split = Vec::new();
        for entry in entries {
            let id = entry["windowId"].as_str().unwrap();
            for child in children.get(id).into_iter().flatten() {
                let mut piece = entry.clone();
                piece["windowId"] = Value::from(child.as_str());
                split.push(piece);
            }
        }
        nta["windowObstructions"] = Value::Array(split);
    }
}

/// Every window to outside gets its own movable shading (7.5, a manual
/// external screen).
fn shade_every_window(project: &mut Value) {
    let shadings: Vec<Value> = project["zones"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|zone| zone["surfaces"].as_array().unwrap())
        .filter(|surface| surface["thermalBoundary"] == "outdoor")
        .flat_map(|surface| surface["windows"].as_array().into_iter().flatten())
        .map(|window| {
            json!({
                "windowId": window["id"],
                "movableShading": {
                    "device": {"kind": "external_screen", "colour": "dark"},
                    "control": "manual_utility_without_glare_protection",
                    "sourceReference": "schaaltest: screen per raam"
                },
                "sourceReference": "schaaltest: zonwering per raam"
            })
        })
        .collect();
    project["ntaCalculation"]["windowShadings"] = Value::Array(shadings);
}

/// The utility building of the scale audit: case H with its common space
/// entered as a common space (6.6.4), repeated to 50 rekenzones, every
/// window split in five (2 250 windows, each with shading and, where case H
/// has one, an obstruction), 1 200 thermal bridges, the PV field split
/// over 30 systems and the second half of the zones on an additional
/// heating system.
pub fn dense_utility_building() -> Value {
    let mut base: Value = serde_json::from_str(CASE_H).unwrap();
    {
        let nta = &mut base["ntaCalculation"];
        let rz1 = &mut nta["zoneData"][0];
        rz1["functionAreas"] = json!([
            {"function": "office", "areaM2": 14.70},
            {"function": "other_assembly", "areaM2": 26.30},
            {"function": "other_healthcare", "areaM2": 27.60}
        ]);
        rz1["commonSpaces"] = json!([{
            "id": "gemeenschappelijk",
            "areaM2": 49.20,
            "sourceReference": "schaaltest: gemeenschappelijke ruimte van geval H"
        }]);
        rz1["ventilation"]["functions"] = rz1["functionAreas"].clone();
        nta.as_object_mut().unwrap().remove("bblFunctions");
    }
    let mut project = scaled(base, 25);
    split_windows(&mut project, 5);
    shade_every_window(&mut project);
    let nta = &mut project["ntaCalculation"];
    let pv = nta["pvSystems"][0].clone();
    let panel_area = pv["peakPower"]["panelAreaM2"].as_f64().unwrap();
    nta["pvSystems"] = Value::Array(
        (0..30)
            .map(|n| {
                let mut system = pv.clone();
                system["id"] = Value::from(format!("pv-{n}"));
                system["peakPower"]["panelAreaM2"] = Value::from(panel_area / 30.0);
                system
            })
            .collect(),
    );
    let zone_ids: Vec<Value> = project["zones"].as_array().unwrap()[25..]
        .iter()
        .map(|zone| zone["id"].clone())
        .collect();
    let generator = project["ntaCalculation"]["generator"].clone();
    let distribution = project["ntaCalculation"]["distributionSystem"].clone();
    project["ntaCalculation"]["additionalHeatingSystems"] = json!([{
        "zoneIds": zone_ids,
        "generator": generator,
        "distributionSystem": distribution,
        "identicalSystems": 1
    }]);
    project
}

/// The project in another edition. Case H enters its emission with the
/// 2023 route; the editions without it take the general one.
fn in_edition(project: &Value, edition: &str) -> Value {
    let mut copy = project.clone();
    copy["ntaCalculation"]["normVersion"] = Value::from(edition);
    if !["2022", "2023"].contains(&edition) {
        if let Some(emission) = copy["ntaCalculation"]["emission"].as_object_mut() {
            emission.remove("edition2023");
        }
    }
    copy
}

/// The 1985 office survey with every window split into `parts` windows.
fn dense_office_survey(parts: usize) -> UtilitySurvey {
    let mut survey: Value = serde_json::from_str(OFFICE_SURVEY).unwrap();
    let windows = survey["envelope"]["windows"].as_array().unwrap().clone();
    let mut split = Vec::new();
    for window in &windows {
        let area = window["areaM2"].as_f64().unwrap() / parts as f64;
        for part in 0..parts {
            let mut piece = window.clone();
            piece["id"] = Value::from(format!("{}.{part}", window["id"].as_str().unwrap()));
            piece["areaM2"] = Value::from(area);
            split.push(piece);
        }
    }
    survey["envelope"]["windows"] = Value::Array(split);
    serde_json::from_value(survey).unwrap()
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
    let dense = dense_utility_building();
    for edition in ["2020+A1", "2022", "2023", "2024", "2025+C1"] {
        row(
            &format!("utiliteit dicht ({edition})"),
            in_edition(&dense, edition),
        );
    }
    for parts in [1usize, 5, 10, 20] {
        let mut project = scaled_case_h(25);
        split_windows(&mut project, parts);
        shade_every_window(&mut project);
        row(&format!("geval H × 25, ramen × {parts}"), project);
    }
    println!();
    println!("| blanks in the NTA block | gaps | time (ms) |");
    println!("|---:|---:|---:|");
    for blanks in [1usize, 10, 50, 200] {
        let project = with_blank_shadings(&dense, blanks);
        let start = Instant::now();
        let result = assess_project_performance(&project);
        println!(
            "| {blanks} | {} | {:.0} |",
            result.gaps.len(),
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
    println!();
    for parts in [1usize, 10, 100] {
        let survey = dense_office_survey(parts);
        let start = Instant::now();
        let result = assess_utility_survey(&survey);
        println!(
            "survey, {} windows: {} in {:.1} ms",
            4 * parts,
            result.status,
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
}

/// The project with the control of `blanks` window shadings cleared.
fn with_blank_shadings(project: &Value, blanks: usize) -> Value {
    let mut copy = project.clone();
    let shadings = copy["ntaCalculation"]["windowShadings"]
        .as_array_mut()
        .unwrap();
    let step = (shadings.len() / blanks.max(1)).max(1);
    for index in 0..blanks {
        shadings[index * step]["movableShading"]["control"] = Value::Null;
    }
    copy
}

fn budget(debug_secs: u64, release_secs: u64) -> Duration {
    Duration::from_secs(if cfg!(debug_assertions) {
        debug_secs
    } else {
        release_secs
    })
}

/// The scale audit's utility building (50 zones, 2 250 windows with
/// shading, 1 200 thermal bridges, 30 PV systems, an additional heating
/// system and a common space) calculates within budget. Measured on
/// 10 October 2026: 3,4 s in a debug build, 0,42 s in release; the budget
/// leaves room for a loaded gate machine.
#[test]
fn dense_utility_building_calculates_within_budget() {
    let project = in_edition(&dense_utility_building(), "2025+C1");
    let run = measure(&project);
    assert_eq!(run.zones, 50);
    assert_eq!(run.windows, 2250);
    assert!(run.status.starts_with("calculated"), "{}", run.status);
    let limit = budget(40, 4);
    assert!(run.elapsed < limit, "dense building took {:?}", run.elapsed);
}

/// The blank search runs on a reduced copy of the NTA block (the list
/// elements that hold a blank) and checks it on the whole block, so its cost
/// follows the blanks rather than the blanks times the block. Two hundred
/// blanks in the dense building: 6,9 s in a debug build, 1,2 s in release
/// (10 October 2026; 19,6 s in release before), 1,6 s in release since
/// review 27 checks each blank on the whole block.
#[test]
fn blanks_in_a_dense_building_are_found_within_budget() {
    let project = with_blank_shadings(&in_edition(&dense_utility_building(), "2025+C1"), 200);
    let start = Instant::now();
    let result = assess_project_performance(&project);
    let elapsed = start.elapsed();
    assert_eq!(result.status, "incomplete");
    assert_eq!(result.gaps.len(), 200, "{:?}", result.gaps);
    assert!(
        result
            .gaps
            .iter()
            .all(|gap| gap.code == "nta_value_missing"
                && gap.path.ends_with("movableShading.control"))
    );
    let limit = budget(60, 6);
    assert!(elapsed < limit, "200 blanks took {elapsed:?}");
}

/// The blank search runs on a reduced copy of the block and checks it on the
/// whole block; on the dense building it reports the same as the search on
/// the whole block (`blank_search_equivalence.rs` covers the fixtures):
/// blanks in the window shadings, leaves spread over every list, and a whole
/// list element blanked.
#[test]
fn dense_building_blank_search_matches_the_whole_block_search() {
    use nta8800_core::project_performance::nta_blank_search;
    let mut block = dense_utility_building()["ntaCalculation"].clone();
    for index in 0..12 {
        block["windowShadings"][index * 37]["movableShading"]["control"] = Value::Null;
    }
    assert_eq!(
        nta_blank_search(&block, false),
        nta_blank_search(&block, true),
        "twelve shading controls"
    );
    let mut numbers = Vec::new();
    collect_number_paths(&block, String::new(), &mut numbers);
    for pointer in numbers.iter().step_by(997) {
        *block.pointer_mut(pointer).unwrap() = Value::Null;
    }
    let spread = nta_blank_search(&block, false);
    assert!(spread.0.len() > 12, "{spread:?}");
    assert_eq!(
        spread,
        nta_blank_search(&block, true),
        "leaves spread over the block"
    );
    block["windowShadings"][5] = Value::Null;
    assert_eq!(
        nta_blank_search(&block, false),
        nta_blank_search(&block, true),
        "a list element blanked"
    );
}

/// The 1985 office survey with 400 windows calculates within budget
/// (0,3 s debug, 35 ms release on 10 October 2026).
#[test]
fn dense_survey_calculates_within_budget() {
    let start = Instant::now();
    let result = assess_utility_survey(&dense_office_survey(100));
    let elapsed = start.elapsed();
    assert!(result.status.starts_with("calculated"), "{}", result.status);
    let limit = budget(15, 2);
    assert!(elapsed < limit, "dense survey took {elapsed:?}");
}

/// Writes the serialized result of every project fixture (as is, with
/// blanks, in every edition) to `$NTA_SNAPSHOT_DIR`, to compare outputs
/// byte by byte before and after a performance change.
#[test]
#[ignore = "snapshot; set NTA_SNAPSHOT_DIR"]
fn snapshot_outputs() {
    let Ok(dir) = std::env::var("NTA_SNAPSHOT_DIR") else {
        return;
    };
    let dir = std::path::Path::new(&dir);
    std::fs::create_dir_all(dir).unwrap();
    let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../training-data");
    let mut names: Vec<_> = std::fs::read_dir(&data)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    names.sort();
    let mut written = 0;
    for path in names {
        let Ok(value) = serde_json::from_str::<Value>(&std::fs::read_to_string(&path).unwrap())
        else {
            continue;
        };
        if value.get("zones").is_none() || value.get("ntaCalculation").is_none() {
            continue;
        }
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let mut variants = vec![("asis".to_string(), value.clone())];
        for edition in ["2020+A1", "2022", "2023", "2024", "2025+C1"] {
            variants.push((format!("ed-{edition}"), in_edition(&value, edition)));
        }
        // Every third leaf number of the NTA block blanked, one at a time
        // for the first 40, and all of them together.
        let mut numbers = Vec::new();
        collect_number_paths(&value["ntaCalculation"], String::new(), &mut numbers);
        let picked: Vec<&String> = numbers.iter().step_by(3).take(40).collect();
        for (index, pointer) in picked.iter().enumerate() {
            let mut copy = value.clone();
            *copy["ntaCalculation"].pointer_mut(pointer).unwrap() = Value::Null;
            variants.push((format!("blank-{index}"), copy));
        }
        let mut all = value.clone();
        for pointer in &picked {
            *all["ntaCalculation"].pointer_mut(pointer).unwrap() = Value::Null;
        }
        variants.push(("blank-all".to_string(), all));
        for (name, project) in variants {
            let result = assess_project_performance(&project);
            let bytes = serde_json::to_vec(&result).unwrap();
            std::fs::write(dir.join(format!("{stem}.{name}.json")), bytes).unwrap();
            written += 1;
        }
    }
    let mut dense = dense_utility_building();
    for index in 0..12 {
        dense["ntaCalculation"]["windowShadings"][index * 37]["movableShading"]["control"] =
            Value::Null;
    }
    let result = assess_project_performance(&dense);
    std::fs::write(
        dir.join("dense.blanks.json"),
        serde_json::to_vec(&result).unwrap(),
    )
    .unwrap();
    println!("{written} outputs written");
}

fn collect_number_paths(value: &Value, pointer: String, out: &mut Vec<String>) {
    match value {
        Value::Number(_) => out.push(pointer),
        Value::Object(map) => {
            for (key, item) in map {
                collect_number_paths(item, format!("{pointer}/{key}"), out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_number_paths(item, format!("{pointer}/{index}"), out);
            }
        }
        _ => {}
    }
}
