//! Option coverage: every enum option and every optional field the kernel
//! input types offer, in every edition.
//!
//! The options are not listed by hand; serde lists them:
//! - Each string leaf of each fixture is replaced by a probe value. For an
//!   enum leaf, deserializing gives "unknown variant `__probe__`, expected
//!   one of …" with the complete variant list.
//! - Each object gets a probe key. For a struct with `deny_unknown_fields`,
//!   deserializing gives "unknown field `__probe__`, expected one of …" with
//!   every field, also the optional ones the fixture leaves out.
//!
//! A new variant or a new optional field is therefore swept without changing
//! this test. A variant that needs data of its own (an internally tagged
//! block such as `generator.kind`) and an absent field take their value from
//! a donor: an object anywhere in the corpus (fixtures, other test inputs
//! and the seeds below) with the same tag or field. A variant or field
//! without a donor that deserializes is counted as unreached, not failed;
//! `OPTION_COVERAGE_REPORT=1` lists them, and a seed closes the gap.
//!
//! Every input runs in all five editions and must:
//! - not panic;
//! - hand out only finite numbers and never refuse with `non_finite_result`;
//! - give a non-calculated status only with at least one specific code;
//! - emit only codes with a Dutch and an English label in `src/i18n`.
//!
//! The runs are spread over threads; the edition is thread-local.

use nta8800_core::building_performance::{assess_building_performance, BuildingPerformanceInput};
use nta8800_core::finite::first_non_finite;
use nta8800_core::norm_versions::NormVersion;
use nta8800_core::opname::utility::{assess_utility_survey, UtilitySurvey};
use nta8800_core::opname::{assess_residential_survey, ResidentialSurvey};
use nta8800_core::project_performance::{assess_project_performance, NtaCalculationInput};
use nta8800_core::ProjectInput;
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const PROBE: &str = "__probe__";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

/// The fixtures a user's input can look like, per route.
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
        fixture!(
            "rvo-voorbeeldwoningen-galerijwoning-1975-1991",
            Route::Building
        ),
        fixture!(
            "rvo-voorbeeldwoningen-hoekwoning-1946-1964",
            Route::Building
        ),
        fixture!(
            "rvo-voorbeeldwoningen-portiekwoning-1965-1974",
            Route::Building
        ),
        fixture!(
            "rvo-voorbeeldwoningen-tussenwoning-1965-1974",
            Route::Building
        ),
        fixture!(
            "rvo-voorbeeldwoningen-twee-onder-een-kap-1992-2005",
            Route::Building
        ),
        fixture!(
            "rvo-voorbeeldwoningen-vrijstaand-1975-1991",
            Route::Building
        ),
        fixture!("opname-1930-terraced", Route::Residential),
        fixture!("opname-1975-apartment", Route::Residential),
        fixture!("opname-2015-detached", Route::Residential),
        fixture!(
            "rvo-voorbeeldwoningen-hoekwoning-1946-1964",
            Route::Residential
        ),
        fixture!(
            "rvo-voorbeeldwoningen-galerijwoning-1975-1991",
            Route::Residential
        ),
        fixture!(
            "rvo-voorbeeldwoningen-vrijstaand-1975-1991",
            Route::Residential
        ),
        fixture!("opname-utility-1970-retail", Route::Utility),
        fixture!("opname-utility-1985-office", Route::Utility),
        fixture!("opname-utility-2005-school", Route::Utility),
    ];
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
            assert!(
                shape_error(route, &value).is_none(),
                "{name}: {:?}",
                shape_error(route, &value)
            );
            Fixture { name, route, value }
        })
        .collect()
}

/// Further JSON that only serves as a donor of blocks (generators,
/// distribution systems) for options no fixture has.
fn donor_corpus() -> Vec<Value> {
    let mut corpus: Vec<Value> = [
        include_str!("../../../training-data/nta8800-space-heating-chain-synthetic.json"),
        include_str!("../../../training-data/nta8800-space-heating-chain-annex-q-synthetic.json"),
        include_str!(
            "../../../training-data/nta8800-space-heating-chain-collective-synthetic.json"
        ),
        include_str!(
            "../../../training-data/nta8800-space-heating-chain-product-boiler-synthetic.json"
        ),
        include_str!("../../../training-data/nta8800-hot-water-annex-t-u-synthetic.json"),
        include_str!("../../../training-data/nta8800-ventilation-synthetic.json"),
        include_str!("../../../training-data/nta8800-monthly-demand-synthetic.json"),
        include_str!("../../../training-data/nta8800-annex-p-synthetic.json"),
        include_str!("../../../training-data/nta8800-mwa-template-measures.json"),
        include_str!("../../../training-data/nta8800-gas-chain-diagnostic-synthetic.json"),
    ]
    .iter()
    .map(|json| serde_json::from_str(json).unwrap())
    .collect();
    corpus.push(seeds());
    corpus
}

/// Hand-written donors for optional blocks that no fixture or test input
/// carries; each is the smallest valid block. A block a fixture gains later
/// needs no seed.
fn seeds() -> Value {
    json!([
        // 8.3.3.2 with the depth per wall part (8.42/D.12): `depthM` stays unset.
        { "heatedBasement": {
            "wallDepths": [{ "lengthM": 10.0, "depthM": 1.5 }],
            "wallResistanceM2kPerW": 2.5
        } },
        { "heatedBasement": { "depthM": 1.5, "wallResistanceM2kPerW": 2.5 } },
    ])
}

/// Where deserialization stopped (a JSON pointer into the route's input) and
/// why.
#[derive(Clone, PartialEq, Debug)]
struct ShapeError {
    pointer: String,
    message: String,
}

fn shape<T: DeserializeOwned>(value: &Value, prefix: &str) -> Option<ShapeError> {
    use serde_path_to_error::Segment;
    serde_path_to_error::deserialize::<_, T>(value.clone())
        .err()
        .map(|error| {
            let mut pointer = prefix.to_string();
            for segment in error.path().iter() {
                match segment {
                    Segment::Seq { index } => pointer.push_str(&format!("/{index}")),
                    Segment::Map { key } => pointer.push_str(&format!("/{}", escape(key))),
                    Segment::Enum { variant } => pointer.push_str(&format!("/{}", escape(variant))),
                    Segment::Unknown => break,
                }
            }
            ShapeError {
                pointer,
                message: error.inner().to_string(),
            }
        })
}

/// The deserialization error of the input of a route, if any.
fn shape_error(route: Route, value: &Value) -> Option<ShapeError> {
    match route {
        Route::Project => shape::<ProjectInput>(value, "").or_else(|| {
            value
                .get("ntaCalculation")
                .and_then(|block| shape::<NtaCalculationInput>(block, "/ntaCalculation"))
        }),
        Route::Building => shape::<BuildingPerformanceInput>(value, ""),
        Route::Residential => shape::<ResidentialSurvey>(value, ""),
        Route::Utility => shape::<UtilitySurvey>(value, ""),
    }
}

fn edition_pointer(route: Route) -> &'static str {
    match route {
        Route::Project => "/ntaCalculation/normVersion",
        _ => "/normVersion",
    }
}

fn unescape(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// Sets (or inserts) the value at a JSON pointer whose parent exists.
fn set_pointer(root: &mut Value, pointer: &str, new: Value) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    let key = unescape(key);
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

/// Removes the object member at a JSON pointer.
fn remove_pointer(root: &mut Value, pointer: &str) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    if let Some(Value::Object(map)) = root.pointer_mut(parent) {
        map.remove(&unescape(key));
    }
}

/// Optional members the fixtures fill in, left out one at a time: the
/// default route behind them (`#[serde(default)]`, an absent `Option`).
/// One occurrence per position; `(fixture, pointer, schema)`.
fn omissions(fixtures: &[Fixture]) -> Vec<(usize, String, String)> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (index, fixture) in fixtures.iter().enumerate() {
        let (mut strings, mut objects) = (Vec::new(), Vec::new());
        walk(&fixture.value, String::new(), &mut strings, &mut objects);
        for object in objects {
            let Some(map) = fixture.value.pointer(&object).and_then(Value::as_object) else {
                continue;
            };
            for key in map.keys() {
                let pointer = format!("{object}/{}", escape(key));
                if pointer == edition_pointer(fixture.route) {
                    continue;
                }
                let schema = schema_path(&pointer);
                if seen.insert((fixture.route, schema.clone())) {
                    out.push((index, pointer, schema));
                }
            }
        }
    }
    out
}

/// Every string leaf and every object, as JSON pointers.
fn walk(value: &Value, pointer: String, strings: &mut Vec<String>, objects: &mut Vec<String>) {
    match value {
        Value::String(_) => strings.push(pointer),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                walk(item, format!("{pointer}/{index}"), strings, objects);
            }
        }
        Value::Object(map) => {
            objects.push(pointer.clone());
            for (key, item) in map {
                walk(item, format!("{pointer}/{}", escape(key)), strings, objects);
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

/// The names serde lists after "unknown {what} `__probe__`, expected".
fn expected_names(message: &str, what: &str) -> Option<Vec<String>> {
    let marker = format!("unknown {what} `{PROBE}`, expected ");
    let rest = message.split(marker.as_str()).nth(1)?;
    let rest = rest.strip_prefix("one of ").unwrap_or(rest);
    let rest = rest.split(" at line ").next().unwrap_or(rest);
    let names: Vec<String> = rest
        .split('`')
        .enumerate()
        .filter(|(index, _)| index % 2 == 1)
        .map(|(_, name)| name.to_string())
        .collect();
    (!names.is_empty()).then_some(names)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Kind {
    Variant,
    Field,
}

/// One option: a variant of an enum position or an optional field of an
/// object position, with the occurrences (fixture, pointer) found.
struct Position {
    route: Route,
    kind: Kind,
    schema: String,
    occurrences: Vec<(usize, String)>,
    options: Vec<String>,
}

fn discover(fixtures: &[Fixture]) -> Vec<Position> {
    let mut positions: BTreeMap<(Route, Kind, String), Position> = BTreeMap::new();
    for (index, fixture) in fixtures.iter().enumerate() {
        let (mut strings, mut objects) = (Vec::new(), Vec::new());
        walk(&fixture.value, String::new(), &mut strings, &mut objects);
        let mut probe = |kind: Kind, pointer: String| {
            let schema = schema_path(&pointer);
            let key = (fixture.route, kind, schema.clone());
            if let Some(position) = positions.get_mut(&key) {
                if position.occurrences.len() < 6 {
                    position.occurrences.push((index, pointer));
                }
                return;
            }
            let mut probed = fixture.value.clone();
            let (target, what) = match kind {
                Kind::Variant => (pointer.clone(), "variant"),
                Kind::Field => (format!("{pointer}/{PROBE}"), "field"),
            };
            set_pointer(&mut probed, &target, Value::from(PROBE));
            let Some(options) = shape_error(fixture.route, &probed)
                .and_then(|error| expected_names(&error.message, what))
            else {
                return;
            };
            positions.insert(
                key,
                Position {
                    route: fixture.route,
                    kind,
                    schema,
                    occurrences: vec![(index, pointer)],
                    options,
                },
            );
        };
        for pointer in strings {
            if pointer != edition_pointer(fixture.route) {
                probe(Kind::Variant, pointer);
            }
        }
        for pointer in objects {
            probe(Kind::Field, pointer);
        }
    }
    positions.into_values().collect()
}

/// Every object anywhere in `value`.
fn objects_in<'a>(value: &'a Value, out: &mut Vec<&'a Map<String, Value>>) {
    match value {
        Value::Object(map) => {
            out.push(map);
            for item in map.values() {
                objects_in(item, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                objects_in(item, out);
            }
        }
        _ => {}
    }
}

/// The edit that puts `variant` at an enum `pointer`: the plain value when
/// it deserializes, otherwise the enclosing object replaced by a donor with
/// the same tag value.
fn variant_edit(
    route: Route,
    base: &Value,
    pointer: &str,
    variant: &str,
    donors: &[&Map<String, Value>],
) -> Option<(String, Value)> {
    let mut plain = base.clone();
    set_pointer(&mut plain, pointer, Value::from(variant));
    if shape_error(route, &plain).is_none() {
        return Some((pointer.to_string(), Value::from(variant)));
    }
    let (parent, key) = pointer.rsplit_once('/')?;
    let key = unescape(key);
    base.pointer(parent)?.as_object()?;
    donors
        .iter()
        .filter(|donor| donor.get(&key).and_then(Value::as_str) == Some(variant))
        .find_map(|donor| {
            let block = Value::Object((*donor).clone());
            let mut candidate = base.clone();
            set_pointer(&mut candidate, parent, block.clone());
            shape_error(route, &candidate)
                .is_none()
                .then(|| (parent.to_string(), block))
        })
}

/// The edit that adds the absent `field` to the object at `pointer`, with
/// the first donor value that deserializes.
fn field_edit(
    route: Route,
    base: &Value,
    pointer: &str,
    field: &str,
    donors: &[&Map<String, Value>],
) -> Option<(String, Value)> {
    let target = format!("{pointer}/{}", escape(field));
    let mut tried = HashSet::new();
    donors
        .iter()
        .filter_map(|donor| donor.get(field))
        .filter(|value| !value.is_null() && tried.insert(value.to_string()))
        .take(12)
        .find_map(|value| {
            let mut candidate = base.clone();
            set_pointer(&mut candidate, &target, value.clone());
            shape_error(route, &candidate)
                .is_none()
                .then(|| (target.clone(), value.clone()))
        })
}

/// The text between `start` and the next `end` in `text`.
fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let rest = text.split_once(start)?.1;
    Some(rest.split_once(end)?.0)
}

/// A placeholder of the type serde says it expected.
fn guess(expected: &str) -> Option<Value> {
    let e = expected.to_ascii_lowercase();
    Some(
        if e.contains("tagged") || e.contains("struct") || e.contains("map") {
            json!({})
        } else if e.contains("sequence") || e.contains("array") || e.contains("tuple") {
            json!([])
        } else if e.contains("variant") || e.contains("one of") || e.contains("enum") {
            Value::from(PROBE)
        } else if e.contains("string") || e.contains("char") {
            Value::from("seed")
        } else if e.contains("bool") {
            Value::from(false)
        } else if [
            "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "integer",
        ]
        .iter()
        .any(|kind| e.contains(kind))
        {
            Value::from(1)
        } else if e.contains("f64") || e.contains("f32") || e.contains("number") {
            Value::from(1.0)
        } else {
            return None;
        },
    )
}

/// Whether a value is of the kind serde names after "invalid type: ".
fn is_kind(value: &Value, got: &str) -> bool {
    match value {
        Value::Number(_) => got.starts_with("floating point") || got.starts_with("integer"),
        Value::String(_) => got.starts_with("string"),
        Value::Bool(_) => got.starts_with("boolean"),
        Value::Array(_) => got.starts_with("sequence"),
        Value::Object(_) => got.starts_with("map"),
        Value::Null => got.starts_with("null") || got.starts_with("unit"),
    }
}

/// Every object at or below `pointer`, as pointers, breadth first.
fn objects_below(value: &Value, pointer: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut queue = std::collections::VecDeque::from([pointer.to_string()]);
    while let Some(current) = queue.pop_front() {
        match value.pointer(&current) {
            Some(Value::Object(map)) => {
                out.push(current.clone());
                for key in map.keys() {
                    queue.push_back(format!("{current}/{}", escape(key)));
                }
            }
            Some(Value::Array(items)) => {
                for index in 0..items.len() {
                    queue.push_back(format!("{current}/{index}"));
                }
            }
            _ => {}
        }
    }
    out
}

thread_local! {
    /// The last deserialization error `fill` met, for the unreached report.
    static FILL_FAILURE: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

/// Builds the smallest input serde accepts with `start` at `target`, led by
/// the deserialization errors: a missing field gets a placeholder, a value
/// of the wrong type one of the expected type, a probe the first listed
/// variant and an array the expected length. `region` bounds the changes
/// (internally tagged enums report their errors at the enum, not deeper).
/// The placeholders are valid shapes, not realistic values: the kernel
/// then refuses or calculates, and either way the rules of the sweep apply.
fn fill(route: Route, base: &Value, region: &str, target: &str, start: Value) -> Option<Value> {
    let mut input = base.clone();
    set_pointer(&mut input, target, start);
    let mut placed: Vec<String> = vec![target.to_string()];
    let mut previous: Option<ShapeError> = None;
    let mut repeats = 0;
    for _ in 0..400 {
        let Some(error) = shape_error(route, &input) else {
            return Some(input);
        };
        let inside = error.pointer.starts_with(region);
        if !inside && !region.starts_with(&error.pointer) {
            return None;
        }
        if previous.as_ref() == Some(&error) {
            repeats += 1;
            if repeats > 2 {
                return None;
            }
        } else {
            repeats = 0;
        }
        previous = Some(error.clone());
        FILL_FAILURE
            .with(|last| *last.borrow_mut() = format!("{} {}", error.pointer, error.message));
        let message = &error.message;
        // The most recent placeholder at or below the error that `accept`s.
        let recent = |input: &Value, accept: &dyn Fn(&Value) -> bool| -> Option<String> {
            let anchor = if inside {
                error.pointer.as_str()
            } else {
                region
            };
            if inside && input.pointer(&error.pointer).is_some_and(accept) {
                return Some(error.pointer.clone());
            }
            placed
                .iter()
                .rev()
                .find(|pointer| {
                    pointer.starts_with(anchor) && input.pointer(pointer).is_some_and(accept)
                })
                .cloned()
        };
        if let Some(field) = between(message, "missing field `", "`") {
            // Inside an internally tagged enum the error names the enum, not
            // the nested struct: try the placeholders made last, then every
            // object below.
            let anchor = if inside {
                error.pointer.as_str()
            } else {
                region
            };
            let mut candidates: Vec<String> = placed
                .iter()
                .rev()
                .filter(|pointer| {
                    pointer.starts_with(anchor)
                        && input.pointer(pointer).is_some_and(Value::is_object)
                })
                .cloned()
                .collect();
            candidates.extend(objects_below(&input, anchor));
            let unknown = format!("unknown field `{field}`");
            let found = candidates.into_iter().find_map(|object| {
                let pointer = format!("{object}/{}", escape(field));
                if input.pointer(&pointer).is_some() {
                    return None;
                }
                let mut trial = input.clone();
                set_pointer(&mut trial, &pointer, Value::from(1));
                let after = shape_error(route, &trial);
                let better = after.as_ref() != Some(&error)
                    && !after.as_ref().is_some_and(|e| e.message.contains(&unknown));
                better.then_some((trial, pointer))
            });
            let (trial, pointer) = found?;
            input = trial;
            placed.push(pointer);
        } else if let Some(given) = between(message, "unknown variant `", "`") {
            let names: Vec<&str> = message
                .split_once("expected ")?
                .1
                .split('`')
                .skip(1)
                .step_by(2)
                .collect();
            let first = Value::from(*names.first()?);
            let pointer = recent(&input, &|value| value.as_str() == Some(given))?;
            set_pointer(&mut input, &pointer, first);
        } else if let Some(rest) = message.strip_prefix("unknown field `") {
            let field = rest.split('`').next()?;
            let anchor = if inside {
                error.pointer.as_str()
            } else {
                region
            };
            let object = objects_below(&input, anchor).into_iter().find(|object| {
                input
                    .pointer(object)
                    .is_some_and(|o| o.get(field).is_some())
            })?;
            input.pointer_mut(&object)?.as_object_mut()?.remove(field);
        } else if message.starts_with("invalid value: map, expected map with a single key") {
            // An externally tagged enum: a unit variant is a plain string.
            let pointer = recent(&input, &|value| {
                value.as_object().is_some_and(Map::is_empty)
            })?;
            set_pointer(&mut input, &pointer, Value::from(PROBE));
        } else if let Some(rest) = message.strip_prefix("invalid type: ") {
            let (got, expected) = rest.split_once(", expected ")?;
            let new = guess(expected)?;
            let pointer = recent(&input, &|value| is_kind(value, got))?;
            if input.pointer(&pointer) == Some(&new) {
                return None;
            }
            set_pointer(&mut input, &pointer, new);
            if !placed.contains(&pointer) {
                placed.push(pointer);
            }
        } else {
            let rest = message.strip_prefix("invalid length ")?;
            let want: usize = rest
                .split_once("expected ")?
                .1
                .split(|c: char| !c.is_ascii_digit())
                .find(|part| !part.is_empty())?
                .parse()
                .ok()?;
            let pointer = recent(&input, &|value| value.is_array())?;
            let items = input.pointer_mut(&pointer)?.as_array_mut()?;
            let item = items.first().cloned().unwrap_or(Value::from(1));
            items.resize(want, item);
            for index in 0..want {
                placed.push(format!("{pointer}/{index}"));
            }
        }
    }
    None
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

/// The path and detail reported with `code`, for the failure message.
fn entries(value: &Value, code: &str, out: &mut Vec<String>) {
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
                entries(item, code, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                entries(item, code, out);
            }
        }
        _ => {}
    }
}

fn checked<T: serde::Serialize>(result: &T) -> Result<Value, String> {
    match first_non_finite(result) {
        Some(path) => Err(format!("non-finite number at {path}")),
        None => Ok(serde_json::to_value(result).unwrap()),
    }
}

/// Runs one input and checks it: `Ok(calculated, codes)` or the broken rule.
fn run(route: Route, input: &Value) -> Result<(bool, BTreeSet<String>), String> {
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(Value, String), String> {
        let shape = |error: serde_json::Error| format!("shape: {error}");
        match route {
            Route::Project => {
                let result = assess_project_performance(input);
                Ok((checked(&result)?, result.status.to_string()))
            }
            Route::Building => {
                let parsed: BuildingPerformanceInput =
                    serde_json::from_value(input.clone()).map_err(shape)?;
                let result = assess_building_performance(&parsed);
                Ok((checked(&result)?, result.status.to_string()))
            }
            Route::Residential => {
                let survey: ResidentialSurvey =
                    serde_json::from_value(input.clone()).map_err(shape)?;
                let result = assess_residential_survey(&survey);
                Ok((checked(&result)?, result.status.to_string()))
            }
            Route::Utility => {
                let survey: UtilitySurvey = serde_json::from_value(input.clone()).map_err(shape)?;
                let result = assess_utility_survey(&survey);
                Ok((checked(&result)?, result.status.to_string()))
            }
        }
    }));
    let (value, status) = match outcome {
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
            let mut detail = Vec::new();
            entries(&value, generic, &mut detail);
            return Err(format!("refused with {generic}: {}", detail.join("; ")));
        }
    }
    let calculated = status.starts_with("calculated");
    if !calculated && found.is_empty() {
        return Err(format!("status {status} without any code"));
    }
    if route == Route::Project && !calculated {
        let gaps = value["gaps"].as_array().cloned().unwrap_or_default();
        if gaps.is_empty() {
            return Err(format!("status {status} without any project gap"));
        }
        let routes = gap_routes();
        for gap in &gaps {
            let path = gap["path"].as_str().unwrap_or("");
            if !routes.routes(path) {
                return Err(format!(
                    "gap {} at {path} has no route in gapRoutes.ts",
                    gap["code"]
                ));
            }
        }
    }
    Ok((calculated, found))
}

/// The prefixes of `src/core/nta/gapRoutes.ts`: the first member of every
/// `GAP_ROUTES` prefix and every member of `NTA_INPUT_ROUTES` (below
/// `ntaCalculation`). A gap path routes when its first member is one of the
/// former, or when it sits in the NTA input and its member is one of the
/// latter; the UI then opens the step page with that section.
struct GapRoutes {
    project: HashSet<String>,
    nta: HashSet<String>,
}

impl GapRoutes {
    fn routes(&self, path: &str) -> bool {
        let first =
            |path: &str| -> String { path.split(['.', '[']).next().unwrap_or("").to_string() };
        match path.strip_prefix("ntaCalculation") {
            Some("") => true,
            Some(rest) if rest.starts_with('.') => self.nta.contains(&first(&rest[1..])),
            _ => self.project.contains(&first(path)),
        }
    }
}

fn gap_routes() -> &'static GapRoutes {
    static ROUTES: std::sync::OnceLock<GapRoutes> = std::sync::OnceLock::new();
    ROUTES.get_or_init(|| {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/core/nta/gapRoutes.ts");
        let text = std::fs::read_to_string(file).unwrap();
        let mut project = HashSet::new();
        let mut nta = HashSet::new();
        let mut in_nta = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("export const NTA_INPUT_ROUTES") {
                in_nta = true;
                continue;
            }
            if in_nta {
                if line.starts_with("};") {
                    in_nta = false;
                    continue;
                }
                let key = line
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c: char| c == '\'' || c.is_whitespace());
                let member = key.split(['.', '[']).next().unwrap_or("");
                if !member.is_empty() {
                    nta.insert(member.to_string());
                }
            } else if let Some(rest) = line.split("prefix: '").nth(1) {
                let prefix = rest.split('\'').next().unwrap_or("");
                let member = prefix.split(['.', '[']).next().unwrap_or("");
                project.insert(member.to_string());
            }
        }
        assert!(nta.contains("generator") && project.contains("zones"));
        GapRoutes { project, nta }
    })
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
        let whole = match name.as_str() {
            "nl.ts" => Some(true),
            "en.ts" => Some(false),
            _ => None,
        };
        let mut table = whole;
        for line in text.lines() {
            if whole.is_none() && line.starts_with("export const ") {
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
            let Some(rest) = line.trim_start().strip_prefix('\'') else {
                continue;
            };
            let Some((key, after)) = rest.split_once('\'') else {
                continue;
            };
            if after.trim_start().starts_with(':') {
                if dutch { &mut nl } else { &mut en }.insert(key.to_string());
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

/// One input: a fixture with edits, run in every edition.
struct Job {
    label: String,
    fixture: usize,
    /// A value set at a pointer, or `None` to remove the member there.
    edits: Vec<(String, Option<Value>)>,
}

#[derive(Default)]
struct Tally {
    runs: usize,
    calculated: usize,
    failures: Vec<String>,
    codes: BTreeSet<String>,
}

fn run_jobs(fixtures: &[Fixture], jobs: &[Job]) -> Tally {
    let next = AtomicUsize::new(0);
    let tally = Mutex::new(Tally::default());
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(16);
    // The kernel's own panics are expected to be caught; keep them quiet.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(job) = jobs.get(index) else { break };
                let fixture = &fixtures[job.fixture];
                let mut input = fixture.value.clone();
                for (pointer, value) in &job.edits {
                    match value {
                        Some(value) => set_pointer(&mut input, pointer, value.clone()),
                        None => remove_pointer(&mut input, pointer),
                    }
                }
                let mut local = Tally::default();
                for version in NormVersion::ALL {
                    let mut edition = input.clone();
                    let pointer = edition_pointer(fixture.route);
                    let parent = pointer.rsplit_once('/').unwrap().0;
                    // Without its block (`ntaCalculation` left out) the input
                    // has no edition to set.
                    if parent.is_empty() || edition.pointer(parent).is_some() {
                        set_pointer(
                            &mut edition,
                            pointer,
                            serde_json::to_value(version).unwrap(),
                        );
                    }
                    local.runs += 1;
                    match run(fixture.route, &edition) {
                        Ok((calculated, codes)) => {
                            local.calculated += usize::from(calculated);
                            local.codes.extend(codes);
                        }
                        Err(problem) => local
                            .failures
                            .push(format!("{} {version:?}: {problem}", job.label)),
                    }
                }
                let mut tally = tally.lock().unwrap();
                tally.runs += local.runs;
                tally.calculated += local.calculated;
                tally.codes.extend(local.codes);
                tally.failures.extend(local.failures);
            });
        }
    });
    std::panic::set_hook(hook);
    tally.into_inner().unwrap()
}

/// How an option was reached.
enum Reach {
    /// The fixture already has it; the unchanged fixture covers it.
    Present,
    /// The value comes from a donor in the corpus.
    Donor(usize, String, Value),
    /// The value was built from the deserialization errors.
    Filled(usize, String, Value),
    /// With the last deserialization error of the fill.
    Unreached(String),
}

fn reach(
    position: &Position,
    option_index: usize,
    fixtures: &[Fixture],
    donors: &[&Map<String, Value>],
) -> Reach {
    let option = &position.options[option_index];
    let count = position.occurrences.len();
    // Rotate over the occurrences, so the options of one position spread
    // over the fixtures that have it.
    let order: Vec<&(usize, String)> = (0..count)
        .map(|offset| &position.occurrences[(option_index + offset) % count])
        .collect();
    for (fixture_index, pointer) in &order {
        let value = &fixtures[*fixture_index].value;
        let present = match position.kind {
            Kind::Variant => {
                value.pointer(pointer).and_then(Value::as_str) == Some(option.as_str())
            }
            Kind::Field => value.pointer(pointer).and_then(|o| o.get(option)).is_some(),
        };
        if present {
            return Reach::Present;
        }
    }
    for (fixture_index, pointer) in &order {
        let fixture = &fixtures[*fixture_index];
        let made = match position.kind {
            Kind::Variant => variant_edit(fixture.route, &fixture.value, pointer, option, donors),
            Kind::Field => field_edit(fixture.route, &fixture.value, pointer, option, donors),
        };
        if let Some((pointer, value)) = made {
            return Reach::Donor(*fixture_index, pointer, value);
        }
    }
    for (fixture_index, pointer) in &order {
        let fixture = &fixtures[*fixture_index];
        let (region, target, start) = match position.kind {
            Kind::Variant => {
                let Some((parent, key)) = pointer.rsplit_once('/') else {
                    continue;
                };
                let mut start = Map::new();
                start.insert(unescape(key), Value::from(option.clone()));
                (parent.to_string(), parent.to_string(), Value::Object(start))
            }
            Kind::Field => {
                let target = format!("{pointer}/{}", escape(option));
                (target.clone(), target, Value::from(1))
            }
        };
        if let Some(filled) = fill(fixture.route, &fixture.value, &region, &target, start) {
            let value = filled.pointer(&region).cloned().unwrap_or(Value::Null);
            return Reach::Filled(*fixture_index, region, value);
        }
    }
    Reach::Unreached(FILL_FAILURE.with(|last| last.borrow().clone()))
}

/// `f` over `items` on all threads, results in input order.
fn parallel<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::with_capacity(items.len()));
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(16);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(item) = items.get(index) else { break };
                let result = f(item);
                results.lock().unwrap().push((index, result));
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, result)| result).collect()
}

#[test]
fn every_option_in_every_edition_calculates_or_refuses_with_a_labelled_code() {
    let fixtures = fixtures();
    let corpus = donor_corpus();
    let mut donors = Vec::new();
    for fixture in &fixtures {
        objects_in(&fixture.value, &mut donors);
    }
    for value in &corpus {
        objects_in(value, &mut donors);
    }
    let positions = discover(&fixtures);
    let report = std::env::var_os("OPTION_COVERAGE_REPORT").is_some();
    let options: Vec<(usize, usize)> = positions
        .iter()
        .enumerate()
        .flat_map(|(position, item)| (0..item.options.len()).map(move |option| (position, option)))
        .collect();
    let reached = parallel(&options, |&(position, option)| {
        reach(&positions[position], option, &fixtures, &donors)
    });
    let mut jobs: Vec<Job> = (0..fixtures.len())
        .map(|index| Job {
            label: fixtures[index].name.to_string(),
            fixture: index,
            edits: Vec::new(),
        })
        .collect();
    // [present, donor, filled, unreached] for variants and fields.
    let mut counts = [[0usize; 4]; 2];
    let mut unreached = Vec::new();
    for (&(position, option), how) in options.iter().zip(reached) {
        let position = &positions[position];
        let option = &position.options[option];
        let row = &mut counts[position.kind as usize];
        let (fixture, pointer, value, how) = match how {
            Reach::Present => {
                row[0] += 1;
                continue;
            }
            Reach::Unreached(reason) => {
                row[3] += 1;
                unreached.push(format!(
                    "{:?} {:?} {} {option} — {reason}",
                    position.route, position.kind, position.schema
                ));
                continue;
            }
            Reach::Donor(fixture, pointer, value) => {
                row[1] += 1;
                (fixture, pointer, value, "donor")
            }
            Reach::Filled(fixture, pointer, value) => {
                row[2] += 1;
                (fixture, pointer, value, "filled")
            }
        };
        jobs.push(Job {
            label: format!(
                "{} {} {option} ({how}) at {pointer}",
                fixtures[fixture].name, position.schema
            ),
            fixture,
            edits: vec![(pointer, Some(value))],
        });
    }
    let candidates = omissions(&fixtures);
    let optional = parallel(&candidates, |(fixture, pointer, _)| {
        let route = fixtures[*fixture].route;
        let mut input = fixtures[*fixture].value.clone();
        remove_pointer(&mut input, pointer);
        shape_error(route, &input).is_none()
    });
    let mut omitted = 0;
    for ((fixture, pointer, schema), optional) in candidates.into_iter().zip(optional) {
        if optional {
            omitted += 1;
            jobs.push(Job {
                label: format!("{} without {schema}", fixtures[fixture].name),
                fixture,
                edits: vec![(pointer, None)],
            });
        }
    }
    let tally = run_jobs(&fixtures, &jobs);
    let (nl, en) = label_keys();
    let unlabelled: Vec<&String> = tally
        .codes
        .iter()
        .filter(|code| !labelled(&nl, code) || !labelled(&en, code))
        .collect();
    let line = |name: &str, row: [usize; 4]| {
        format!(
            "{name} {} (present {}, donor {}, filled {}, unreached {})",
            row.iter().sum::<usize>(),
            row[0],
            row[1],
            row[2],
            row[3]
        )
    };
    println!(
        "option coverage: {} positions; {}; {}; {omitted} optional members left out; {} inputs × 5 editions = {} runs ({} calculated); {} distinct codes",
        positions.len(),
        line("enum variants", counts[0]),
        line("optional fields", counts[1]),
        jobs.len(),
        tally.runs,
        tally.calculated,
        tally.codes.len()
    );
    if report {
        for position in &positions {
            println!(
                "{:?} {:?} {}: {}",
                position.route,
                position.kind,
                position.schema,
                position.options.join(", ")
            );
        }
        for item in &unreached {
            println!("unreached: {item}");
        }
    }
    assert!(tally.failures.is_empty(), "{:#?}", tally.failures);
    assert!(
        unlabelled.is_empty(),
        "codes without NL/EN label: {unlabelled:?}"
    );
    // Coverage may grow but not silently shrink: a fixture change that
    // drops options shows here.
    let reached = |row: [usize; 4]| row[0] + row[1] + row[2];
    assert!(reached(counts[0]) >= MIN_VARIANTS, "{:?}", counts[0]);
    assert!(reached(counts[1]) >= MIN_FIELDS, "{:?}", counts[1]);
    assert!(omitted >= MIN_OMITTED, "{omitted}");
}

/// Floors of the reached options and left-out members, just under the
/// counts of 2026-10-08 (1203 variants, 1130 fields, 299 members).
const MIN_VARIANTS: usize = 1190;
const MIN_FIELDS: usize = 1110;
const MIN_OMITTED: usize = 290;
