//! Option coverage: every enum option the fixtures reach, in every edition.
//!
//! The options are not listed by hand. Each string leaf of each fixture is
//! replaced by a probe value and deserialized with the kernel's own input
//! types; serde then reports "unknown variant `__probe__`, expected one of
//! …" for a leaf that is an enum, with the complete variant list. A new
//! variant of such an enum is therefore swept without changing this test.
//!
//! Every variant is applied at its position and the input calculated in all
//! five editions. A variant that needs data of its own (an internally tagged
//! block such as `generator.kind`) takes that block from another fixture of
//! the corpus that has it (a donor); without a donor it is counted as
//! unreachable rather than as a failure. Optional top-level blocks of
//! `ntaCalculation` that one project fixture has and another lacks are
//! transplanted as well.
//!
//! Every run must:
//! - not panic;
//! - hand out only finite numbers and never refuse with `non_finite_result`;
//! - give a non-calculated status only with at least one specific code;
//! - emit only codes with a Dutch and an English label in `src/i18n`.
//!
//! `OPTION_COVERAGE_REPORT=1` prints the coverage per position.

use nta8800_core::building_performance::{assess_building_performance, BuildingPerformanceInput};
use nta8800_core::finite::first_non_finite;
use nta8800_core::norm_versions::NormVersion;
use nta8800_core::opname::utility::{assess_utility_survey, UtilitySurvey};
use nta8800_core::opname::{assess_residential_survey, ResidentialSurvey};
use nta8800_core::project_performance::{assess_project_performance, NtaCalculationInput};
use nta8800_core::ProjectInput;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

const PROBE: &str = "__probe__";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Route {
    Project,
    Building,
    Residential,
    Utility,
}

struct Fixture {
    name: &'static str,
    route: Route,
    value: Value,
}

macro_rules! fixture {
    ($name:literal, $route:expr) => {
        (
            $name,
            $route,
            include_str!(concat!("../../../training-data/nta8800-", $name, ".json")),
        )
    };
}

/// The fixtures a user's project can look like, per route.
fn fixtures() -> Vec<Fixture> {
    let sources: [(&str, Route, &str); 25] = [
        fixture!("example-office", Route::Project),
        fixture!("example-terraced-dwelling", Route::Project),
        fixture!("project-performance-synthetic", Route::Project),
        fixture!("public-comparison-a", Route::Project),
        fixture!("public-comparison-b", Route::Project),
        fixture!("public-comparison-c", Route::Project),
        fixture!("public-comparison-d", Route::Project),
        fixture!("public-comparison-e", Route::Project),
        fixture!("public-comparison-f", Route::Project),
        fixture!("building-performance-synthetic", Route::Building),
        fixture!("rvo-voorbeeldwoningen-galerijwoning-1975-1991", Route::Building),
        fixture!("rvo-voorbeeldwoningen-hoekwoning-1946-1964", Route::Building),
        fixture!("rvo-voorbeeldwoningen-portiekwoning-1965-1974", Route::Building),
        fixture!("rvo-voorbeeldwoningen-tussenwoning-1965-1974", Route::Building),
        fixture!("rvo-voorbeeldwoningen-twee-onder-een-kap-1992-2005", Route::Building),
        fixture!("rvo-voorbeeldwoningen-vrijstaand-1975-1991", Route::Building),
        fixture!("opname-1930-terraced", Route::Residential),
        fixture!("opname-1975-apartment", Route::Residential),
        fixture!("opname-2015-detached", Route::Residential),
        fixture!("rvo-voorbeeldwoningen-hoekwoning-1946-1964", Route::Residential),
        fixture!("rvo-voorbeeldwoningen-galerijwoning-1975-1991", Route::Residential),
        fixture!("rvo-voorbeeldwoningen-vrijstaand-1975-1991", Route::Residential),
        fixture!("opname-utility-1970-retail", Route::Utility),
        fixture!("opname-utility-1985-office", Route::Utility),
        fixture!("opname-utility-2005-school", Route::Utility),
    ];
    let mut seen = HashSet::new();
    sources
        .into_iter()
        .map(|(name, route, json)| {
            let mut value: Value = serde_json::from_str(json).unwrap();
            // The RVO files carry the building input and the survey side by side.
            if let Some(inner) = match route {
                Route::Building => value.get("performanceInput").cloned(),
                Route::Residential => value.get("survey").cloned(),
                _ => None,
            } {
                value = inner;
            }
            assert!(seen.insert((name, route as u8)), "{name}");
            assert!(shape_error(route, &value).is_none(), "{name}: {:?}", shape_error(route, &value));
            Fixture { name, route, value }
        })
        .collect()
}

/// Further JSON that only serves as a donor of blocks (generators,
/// distribution systems) for variants no project fixture has.
fn donor_corpus() -> Vec<Value> {
    [
        include_str!("../../../training-data/nta8800-space-heating-chain-synthetic.json"),
        include_str!("../../../training-data/nta8800-space-heating-chain-annex-q-synthetic.json"),
        include_str!("../../../training-data/nta8800-space-heating-chain-collective-synthetic.json"),
        include_str!("../../../training-data/nta8800-space-heating-chain-product-boiler-synthetic.json"),
        include_str!("../../../training-data/nta8800-hot-water-annex-t-u-synthetic.json"),
        include_str!("../../../training-data/nta8800-ventilation-synthetic.json"),
        include_str!("../../../training-data/nta8800-monthly-demand-synthetic.json"),
        include_str!("../../../training-data/nta8800-annex-p-synthetic.json"),
        include_str!("../../../training-data/nta8800-mwa-template-measures.json"),
    ]
    .iter()
    .map(|json| serde_json::from_str(json).unwrap())
    .collect()
}

fn shape<T: DeserializeOwned>(value: &Value) -> Option<String> {
    serde_path_to_error::deserialize::<_, T>(value.clone())
        .err()
        .map(|error| error.to_string())
}

/// The deserialization error of the input of a route, if any.
fn shape_error(route: Route, value: &Value) -> Option<String> {
    match route {
        Route::Project => shape::<ProjectInput>(value).or_else(|| {
            value
                .get("ntaCalculation")
                .and_then(shape::<NtaCalculationInput>)
        }),
        Route::Building => shape::<BuildingPerformanceInput>(value),
        Route::Residential => shape::<ResidentialSurvey>(value),
        Route::Utility => shape::<UtilitySurvey>(value),
    }
}

fn edition_pointer(route: Route) -> &'static str {
    match route {
        Route::Project => "/ntaCalculation/normVersion",
        _ => "/normVersion",
    }
}

fn set_pointer(root: &mut Value, pointer: &str, new: Value) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    let key = key.replace("~1", "/").replace("~0", "~");
    let target = if parent.is_empty() {
        root
    } else {
        root.pointer_mut(parent).unwrap()
    };
    match target {
        Value::Object(map) => {
            map.insert(key, new);
        }
        Value::Array(items) => items[key.parse::<usize>().unwrap()] = new,
        _ => panic!("{pointer}"),
    }
}

fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// Every string leaf, as a JSON pointer.
fn string_leaves(value: &Value, pointer: String, out: &mut Vec<String>) {
    match value {
        Value::String(_) => out.push(pointer),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                string_leaves(item, format!("{pointer}/{index}"), out);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                string_leaves(item, format!("{pointer}/{}", escape(key)), out);
            }
        }
        _ => {}
    }
}

/// The pointer with array indices replaced, so the same position in another
/// element or fixture counts once.
fn schema_path(pointer: &str) -> String {
    pointer
        .split('/')
        .map(|segment| {
            if !segment.is_empty() && segment.bytes().all(|b| b.is_ascii_digit()) {
                "[]"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// The variants serde lists for an enum leaf replaced by the probe.
fn variants(message: &str) -> Option<Vec<String>> {
    let rest = message.split("unknown variant `__probe__`, expected ").nth(1)?;
    let rest = rest.strip_prefix("one of ").unwrap_or(rest);
    let variants: Vec<String> = rest
        .split('`')
        .enumerate()
        .filter(|(index, _)| index % 2 == 1)
        .map(|(_, variant)| variant.to_string())
        .collect();
    (!variants.is_empty()).then_some(variants)
}

/// One enum position: where it is first found and its variants.
struct Position {
    route: Route,
    schema: String,
    /// (fixture index, pointer) of every occurrence.
    occurrences: Vec<(usize, String)>,
    variants: Vec<String>,
}

fn discover(fixtures: &[Fixture]) -> Vec<Position> {
    let mut positions: BTreeMap<(u8, String), Position> = BTreeMap::new();
    for (index, fixture) in fixtures.iter().enumerate() {
        let mut leaves = Vec::new();
        string_leaves(&fixture.value, String::new(), &mut leaves);
        for pointer in leaves {
            if pointer == edition_pointer(fixture.route) {
                continue;
            }
            let schema = schema_path(&pointer);
            let key = (fixture.route as u8, schema.clone());
            if let Some(position) = positions.get_mut(&key) {
                if position.occurrences.len() < 4 {
                    position.occurrences.push((index, pointer));
                }
                continue;
            }
            let mut probed = fixture.value.clone();
            set_pointer(&mut probed, &pointer, Value::from(PROBE));
            let Some(found) = shape_error(fixture.route, &probed).and_then(|m| variants(&m)) else {
                continue;
            };
            positions.insert(
                key,
                Position {
                    route: fixture.route,
                    schema,
                    occurrences: vec![(index, pointer)],
                    variants: found,
                },
            );
        }
    }
    positions.into_values().collect()
}

/// Every object anywhere in `value`.
fn objects<'a>(value: &'a Value, out: &mut Vec<&'a Map<String, Value>>) {
    match value {
        Value::Object(map) => {
            out.push(map);
            for item in map.values() {
                objects(item, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                objects(item, out);
            }
        }
        _ => {}
    }
}

/// The input with `variant` at `pointer`: the plain replacement when it
/// deserializes, otherwise the enclosing object replaced by a donor object
/// with the same tag value.
fn with_variant(
    route: Route,
    base: &Value,
    pointer: &str,
    variant: &str,
    donors: &[&Map<String, Value>],
) -> Option<Value> {
    let mut plain = base.clone();
    set_pointer(&mut plain, pointer, Value::from(variant));
    if shape_error(route, &plain).is_none() {
        return Some(plain);
    }
    let (parent, key) = pointer.rsplit_once('/')?;
    let key = key.replace("~1", "/").replace("~0", "~");
    base.pointer(parent)?.as_object()?;
    donors
        .iter()
        .filter(|donor| donor.get(&key).and_then(Value::as_str) == Some(variant))
        .find_map(|donor| {
            let mut candidate = base.clone();
            set_pointer(&mut candidate, parent, Value::Object((*donor).clone()));
            shape_error(route, &candidate).is_none().then_some(candidate)
        })
}

fn edition_label(version: NormVersion) -> Value {
    serde_json::to_value(version).unwrap()
}

/// The codes under every `gaps`, `warnings` and `issues` list.
fn codes(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                if matches!(key.as_str(), "gaps" | "warnings" | "issues") {
                    for entry in item.as_array().into_iter().flatten() {
                        if let Some(code) = entry.get("code").and_then(Value::as_str) {
                            out.insert(code.to_string());
                        }
                    }
                }
                codes(item, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                codes(item, out);
            }
        }
        _ => {}
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Outcome {
    Calculated,
    Refused,
}

/// Runs one input and checks it; the error names the broken rule.
fn run(route: Route, input: &Value) -> Result<(Outcome, BTreeSet<String>), String> {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(Value, &'static str, bool), String> {
        Ok(match route {
            Route::Project => {
                let result = assess_project_performance(input);
                if let Some(path) = first_non_finite(&result) {
                    return Err(format!("non-finite number at {path}"));
                }
                let calculated = result.status.starts_with("calculated");
                (serde_json::to_value(&result).unwrap(), result.status, calculated)
            }
            Route::Building => {
                let parsed: BuildingPerformanceInput = serde_json::from_value(input.clone())
                    .map_err(|error| format!("shape: {error}"))?;
                let result = assess_building_performance(&parsed);
                if let Some(path) = first_non_finite(&result) {
                    return Err(format!("non-finite number at {path}"));
                }
                let calculated = result.status.starts_with("calculated");
                (serde_json::to_value(&result).unwrap(), result.status, calculated)
            }
            Route::Residential | Route::Utility => {
                let result = if route == Route::Residential {
                    let survey: ResidentialSurvey = serde_json::from_value(input.clone())
                        .map_err(|error| format!("shape: {error}"))?;
                    assess_residential_survey(&survey)
                } else {
                    let survey: UtilitySurvey = serde_json::from_value(input.clone())
                        .map_err(|error| format!("shape: {error}"))?;
                    assess_utility_survey(&survey)
                };
                if let Some(path) = first_non_finite(&result) {
                    return Err(format!("non-finite number at {path}"));
                }
                let calculated = result.status.starts_with("calculated");
                (serde_json::to_value(&result).unwrap(), result.status, calculated)
            }
        })
    }));
    let (value, status, calculated) = match result {
        Err(panic) => {
            let message = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            return Err(format!("panic: {message}"));
        }
        Ok(result) => result?,
    };
    let mut found = BTreeSet::new();
    codes(&value, &mut found);
    for generic in ["non_finite_result", "kernel_panic"] {
        if found.contains(generic) {
            return Err(format!("refused with {generic}: {}", detail_of(&value, generic)));
        }
    }
    if !calculated && found.is_empty() {
        return Err(format!("status {status} without any code"));
    }
    let outcome = if calculated {
        Outcome::Calculated
    } else {
        Outcome::Refused
    };
    Ok((outcome, found))
}

/// The path and detail reported with `code`, for the failure message.
fn detail_of(value: &Value, code: &str) -> String {
    let mut out = Vec::new();
    find_entries(value, code, &mut out);
    out.join("; ")
}

fn find_entries(value: &Value, code: &str, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if map.get("code").and_then(Value::as_str) == Some(code) {
                out.push(format!(
                    "{} {}",
                    map.get("path").and_then(Value::as_str).unwrap_or(""),
                    map.get("detail").and_then(Value::as_str).unwrap_or("")
                ));
            }
            for item in map.values() {
                find_entries(item, code, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                find_entries(item, code, out);
            }
        }
        _ => {}
    }
}

/// The label keys of `src/i18n`, Dutch and English apart: `nl.ts` and
/// `en.ts` whole, the other files per `export const …Nl` / `…En` table.
fn label_keys() -> (HashSet<String>, HashSet<String>) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/i18n");
    let (mut nl, mut en) = (HashSet::new(), HashSet::new());
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".ts") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let mut table: Option<bool> = match name.as_str() {
            "nl.ts" => Some(true),
            "en.ts" => Some(false),
            _ => None,
        };
        let whole = table.is_some();
        for line in text.lines() {
            if !whole && line.starts_with("export const ") {
                let ident = line["export const ".len()..]
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("");
                table = if ident.ends_with("Nl") {
                    Some(true)
                } else if ident.ends_with("En") {
                    Some(false)
                } else {
                    None
                };
            }
            let Some(dutch) = table else { continue };
            let trimmed = line.trim_start();
            let Some(rest) = trimmed.strip_prefix('\'') else {
                continue;
            };
            let Some((key, after)) = rest.split_once('\'') else {
                continue;
            };
            if after.trim_start().starts_with(':') {
                if dutch {
                    nl.insert(key.to_string());
                } else {
                    en.insert(key.to_string());
                }
            }
        }
    }
    (nl, en)
}

const LABEL_PREFIXES: [&str; 7] = [
    "nta.gap.",
    "nta.warning.",
    "kernel.issue.",
    "opname.issue.",
    "opname.warning.",
    "mwa.issue.",
    "registration.issue.",
];

fn labelled(keys: &HashSet<String>, code: &str) -> bool {
    LABEL_PREFIXES
        .iter()
        .any(|prefix| keys.contains(&format!("{prefix}{code}")))
}

#[derive(Default)]
struct Tally {
    runs: usize,
    calculated: usize,
    refused: usize,
    unreachable: BTreeSet<String>,
    failures: Vec<String>,
    codes: BTreeSet<String>,
}

impl Tally {
    fn record(&mut self, label: &str, route: Route, input: &Value) {
        self.runs += 1;
        match run(route, input) {
            Ok((outcome, codes)) => {
                match outcome {
                    Outcome::Calculated => self.calculated += 1,
                    Outcome::Refused => self.refused += 1,
                }
                self.codes.extend(codes);
            }
            Err(problem) => {
                if self.failures.len() < 200 {
                    self.failures.push(format!("{label}: {problem}"));
                }
            }
        }
    }
}

fn with_edition(route: Route, input: &Value, version: NormVersion) -> Value {
    let mut input = input.clone();
    if route == Route::Project && input.get("ntaCalculation").is_none() {
        return input;
    }
    set_pointer(&mut input, edition_pointer(route), edition_label(version));
    input
}

/// Optional top-level blocks of `ntaCalculation` from one project fixture
/// placed in another that lacks them.
fn transplants(fixtures: &[Fixture]) -> Vec<(String, Value)> {
    let projects: Vec<&Fixture> = fixtures
        .iter()
        .filter(|fixture| fixture.route == Route::Project)
        .collect();
    let mut keys: BTreeMap<String, &Value> = BTreeMap::new();
    for fixture in &projects {
        for (key, value) in fixture.value["ntaCalculation"].as_object().unwrap() {
            keys.entry(key.clone()).or_insert(value);
        }
    }
    let mut out = Vec::new();
    for (key, block) in keys {
        for fixture in &projects {
            if fixture.value["ntaCalculation"].get(&key).is_some() {
                continue;
            }
            let mut input = fixture.value.clone();
            input["ntaCalculation"][&key] = block.clone();
            if shape_error(Route::Project, &input).is_none() {
                out.push((format!("{} + ntaCalculation.{key}", fixture.name), input));
                break;
            }
        }
    }
    out
}

#[test]
fn every_option_in_every_edition_calculates_or_refuses_with_a_labelled_code() {
    let fixtures = fixtures();
    let corpus = donor_corpus();
    let mut donors = Vec::new();
    for fixture in &fixtures {
        objects(&fixture.value, &mut donors);
    }
    for value in &corpus {
        objects(value, &mut donors);
    }
    let positions = discover(&fixtures);
    let report = std::env::var_os("OPTION_COVERAGE_REPORT").is_some();
    let mut tally = Tally::default();
    let mut swept = 0usize;
    for fixture in &fixtures {
        for version in NormVersion::ALL {
            let input = with_edition(fixture.route, &fixture.value, version);
            tally.record(&format!("{} {version:?}", fixture.name), fixture.route, &input);
        }
    }
    for position in &positions {
        let mut reached = 0;
        for (index, variant) in position.variants.iter().enumerate() {
            // Rotate over the fixtures that have this position.
            let (fixture_index, pointer) =
                &position.occurrences[index % position.occurrences.len()];
            let fixture = &fixtures[*fixture_index];
            let current = fixture.value.pointer(pointer).and_then(Value::as_str);
            if current == Some(variant.as_str()) && position.variants.len() > 1 {
                // The unchanged fixture already ran in every edition.
                reached += 1;
                swept += 1;
                continue;
            }
            let Some(input) =
                with_variant(fixture.route, &fixture.value, pointer, variant, &donors)
            else {
                tally
                    .unreachable
                    .insert(format!("{:?} {} = {variant}", position.route, position.schema));
                continue;
            };
            reached += 1;
            swept += 1;
            for version in NormVersion::ALL {
                tally.record(
                    &format!("{} {} = {variant} {version:?}", fixture.name, pointer),
                    fixture.route,
                    &with_edition(fixture.route, &input, version),
                );
            }
        }
        if report {
            println!(
                "{:?} {} [{}/{}] {}",
                position.route,
                position.schema,
                reached,
                position.variants.len(),
                position.variants.join(", ")
            );
        }
    }
    for (label, input) in transplants(&fixtures) {
        for version in NormVersion::ALL {
            tally.record(
                &format!("{label} {version:?}"),
                Route::Project,
                &with_edition(Route::Project, &input, version),
            );
        }
    }
    let (nl, en) = label_keys();
    let unlabelled: Vec<&String> = tally
        .codes
        .iter()
        .filter(|code| !labelled(&nl, code) || !labelled(&en, code))
        .collect();
    let variant_count: usize = positions.iter().map(|p| p.variants.len()).sum();
    println!(
        "option coverage: {} positions, {variant_count} variants, {swept} swept, {} unreachable; {} runs ({} calculated, {} refused); {} codes",
        positions.len(),
        tally.unreachable.len(),
        tally.runs,
        tally.calculated,
        tally.refused,
        tally.codes.len()
    );
    if report {
        for item in &tally.unreachable {
            println!("unreachable: {item}");
        }
    }
    assert!(tally.failures.is_empty(), "{:#?}", tally.failures);
    assert!(unlabelled.is_empty(), "codes without NL/EN label: {unlabelled:?}");
}
