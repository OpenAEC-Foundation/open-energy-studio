//! The blank search of the project route runs on a reduced copy of the NTA
//! block (lists of objects cut down to the elements that hold a null) and
//! checks the outcome on the whole block. These tests hold it to the search
//! on the whole block, which stays in the kernel as the oracle: the same
//! paths, in the same order, and the same residual flag.
//!
//! Covered here: every project fixture as is and with blanks (the variants of
//! the snapshot harness in `large_projects.rs`) and a seeded property test
//! of random blank sets. The dense building with blanks is compared in
//! `large_projects.rs`, next to its builder; the project-form guard
//! (`project_form_guard.rs`) compares every collected form state as well.

use nta8800_core::project_performance::{nta_blank_search, reduced_blank_searches};
use serde_json::Value;

fn training_data() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../training-data")
}

/// Every project fixture: a file with zones and an NTA block.
fn project_fixtures() -> Vec<(String, Value)> {
    let mut paths: Vec<_> = std::fs::read_dir(training_data())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| {
            let value: Value = serde_json::from_str(&std::fs::read_to_string(&path).ok()?).ok()?;
            (value.get("zones").is_some() && value.get("ntaCalculation").is_some()).then(|| {
                (
                    path.file_stem().unwrap().to_string_lossy().to_string(),
                    value,
                )
            })
        })
        .collect()
}

fn assert_same(name: &str, block: &Value) {
    let reduced = nta_blank_search(block, false);
    let whole = nta_blank_search(block, true);
    assert_eq!(
        reduced, whole,
        "{name}: reduced search differs from the whole-block search"
    );
}

fn collect_pointers(value: &Value, pointer: String, numbers_only: bool, out: &mut Vec<String>) {
    match value {
        Value::Number(_) => out.push(pointer),
        Value::String(_) | Value::Bool(_) if !numbers_only => out.push(pointer),
        Value::Object(map) => {
            for (key, item) in map {
                let key = key.replace('~', "~0").replace('/', "~1");
                collect_pointers(item, format!("{pointer}/{key}"), numbers_only, out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_pointers(item, format!("{pointer}/{index}"), numbers_only, out);
            }
        }
        _ => {}
    }
}

/// The snapshot harness's variants: every third leaf number of the NTA block
/// blanked, one at a time for the first 40, and all of them together.
#[test]
fn fixtures_with_blanks_give_the_same_gaps() {
    let fixtures = project_fixtures();
    assert!(fixtures.len() >= 18, "only {} fixtures", fixtures.len());
    let mut compared = 0;
    for (stem, project) in fixtures {
        let block = &project["ntaCalculation"];
        assert_same(&format!("{stem} as is"), block);
        let mut numbers = Vec::new();
        collect_pointers(block, String::new(), true, &mut numbers);
        let picked: Vec<&String> = numbers.iter().step_by(3).take(40).collect();
        let mut all = block.clone();
        for (index, pointer) in picked.iter().enumerate() {
            let mut copy = block.clone();
            *copy.pointer_mut(pointer).unwrap() = Value::Null;
            assert_same(&format!("{stem} blank {index} ({pointer})"), &copy);
            *all.pointer_mut(pointer).unwrap() = Value::Null;
            compared += 1;
        }
        assert_same(&format!("{stem} all blanks"), &all);
        compared += 2;
    }
    assert!(compared > 400, "only {compared} blocks compared");
    // The comparison only means something when the reduced search is used:
    // most blocks with a blank in a list of objects take it.
    let (confirmed, fell_back) = reduced_blank_searches();
    println!("reduced searches: {confirmed} confirmed, {fell_back} fell back");
    assert!(
        confirmed > 100,
        "{confirmed} confirmed, {fell_back} fell back"
    );
}

/// A JSON pointer with its new value, or None to take the member out.
type Edit<'a> = (&'a str, Option<Value>);

/// Case H's NTA block (two zones, lists of objects inside lists of objects)
/// with the edits applied: a pointer and its new value, or None to take the
/// member out.
fn case_h_with(edits: &[Edit]) -> Value {
    let project: Value = serde_json::from_str(
        &std::fs::read_to_string(training_data().join("nta8800-public-comparison-h.json")).unwrap(),
    )
    .unwrap();
    let mut block = project["ntaCalculation"].clone();
    for (pointer, value) in edits {
        match value {
            Some(value) => *block.pointer_mut(pointer).unwrap() = value.clone(),
            None => {
                let (parent, key) = pointer.rsplit_once('/').unwrap();
                block
                    .pointer_mut(parent)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .remove(key)
                    .unwrap();
            }
        }
    }
    block
}

/// Review 27: list shapes the reduction has to map back. A null in the
/// first element (always kept), in a nested list's later element, in every
/// element of a list, a whole element or object blanked, an empty list, and
/// a blank enum tag inside a nested list.
#[test]
fn list_shapes_give_the_same_gaps() {
    let null = || Some(Value::Null);
    let cases: Vec<(&str, Vec<Edit>)> = vec![
        (
            "first element",
            vec![("/zoneData/0/functionAreas/0/areaM2", null())],
        ),
        (
            "first elements of nested lists",
            vec![
                ("/lighting/0/lightingZones/0/areaM2", null()),
                ("/lighting/1/lightingZones/0/areaM2", null()),
            ],
        ),
        (
            "whole object in a nested list",
            vec![("/lighting/0/lightingZones/1/power", null())],
        ),
        (
            "whole list element",
            vec![("/lighting/0/lightingZones/1", null())],
        ),
        (
            "every element of a list",
            vec![
                ("/zoneData/0/functionAreas/0/areaM2", null()),
                ("/zoneData/0/functionAreas/1/areaM2", null()),
                ("/zoneData/0/functionAreas/2/areaM2", null()),
            ],
        ),
        (
            "empty list next to a blank",
            vec![
                ("/zoneData/0/functionAreas", Some(Value::Array(Vec::new()))),
                ("/lighting/1/lightingZones/0/areaM2", null()),
            ],
        ),
        (
            "blank tag in a nested list",
            vec![("/lighting/0/lightingZones/1/power/method", null())],
        ),
    ];
    for (name, edits) in cases {
        let before = reduced_blank_searches();
        let block = case_h_with(&edits);
        assert_eq!(
            nta_blank_search(&block, false),
            nta_blank_search(&block, true),
            "{name}"
        );
        // Every case cuts a list and is confirmed on the whole block.
        assert_eq!(
            reduced_blank_searches(),
            (before.0 + 1, before.1),
            "{name}: not a confirmed reduced search"
        );
    }
}

/// Review 27: a dropped element (it holds no null) that fails on its own
/// must not slip past the reduced copy. The whole block then fails before
/// any blank is put back, the confirmation refuses the reduced result and the
/// search runs on the whole block.
#[test]
fn a_dropped_element_that_fails_on_its_own_falls_back() {
    let blank = ("/lighting/1/lightingZones/0/areaM2", Some(Value::Null));
    let faults: Vec<(&str, Edit)> = vec![
        (
            "unknown member",
            ("/lighting/0/lightingZones/1/bogus", Some(Value::from(1))),
        ),
        (
            "required member left out",
            ("/lighting/0/lightingZones/1/id", None),
        ),
        (
            "wrong type",
            (
                "/lighting/0/lightingZones/1/areaM2",
                Some(Value::from("68")),
            ),
        ),
        (
            "unknown enum variant",
            (
                "/lighting/0/lightingZones/1/power/method",
                Some(Value::from("magic")),
            ),
        ),
    ];
    for (name, fault) in faults {
        let mut edits = vec![blank.clone()];
        if fault.0.ends_with("/bogus") {
            // A new member: added to its object rather than replaced.
            let mut block = case_h_with(&edits);
            block
                .pointer_mut("/lighting/0/lightingZones/1")
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("bogus".into(), Value::from(1));
            let before = reduced_blank_searches();
            let reduced = nta_blank_search(&block, false);
            let after = reduced_blank_searches();
            assert_eq!(reduced, nta_blank_search(&block, true), "{name}");
            assert_eq!(after.1, before.1 + 1, "{name}: no fallback");
            assert!(reduced.1, "{name}: not residual");
            continue;
        }
        edits.push(fault);
        let block = case_h_with(&edits);
        let before = reduced_blank_searches();
        let reduced = nta_blank_search(&block, false);
        let after = reduced_blank_searches();
        assert_eq!(reduced, nta_blank_search(&block, true), "{name}");
        assert_eq!(after.1, before.1 + 1, "{name}: no fallback");
        // The whole block still fails without the blank: residual.
        assert!(reduced.1, "{name}: not residual");
    }
}

/// A small deterministic generator (xorshift64*), so the property test needs
/// no crate and repeats exactly.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

/// Every location in the block: leaves, objects and list elements (a whole
/// object or element blanked is a null too).
fn all_pointers(value: &Value, pointer: String, out: &mut Vec<String>) {
    if !pointer.is_empty() {
        out.push(pointer.clone());
    }
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                let key = key.replace('~', "~0").replace('/', "~1");
                all_pointers(item, format!("{pointer}/{key}"), out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                all_pointers(item, format!("{pointer}/{index}"), out);
            }
        }
        _ => {}
    }
}

/// Random blank sets (1 to 8 locations, leaves and whole objects or list
/// elements) in fixtures with lists of objects, compared case by case.
#[test]
fn random_blank_sets_give_the_same_gaps() {
    let bases: Vec<(String, Value)> = project_fixtures()
        .into_iter()
        .filter(|(stem, _)| {
            [
                "nta8800-example-office",
                "nta8800-example-terraced-dwelling",
                "nta8800-public-comparison-c",
                "nta8800-public-comparison-h",
                "nta8800-public-comparison-k",
                "nta8800-public-comparison-n",
                "nta8800-evidence-link-project",
            ]
            .contains(&stem.as_str())
        })
        .map(|(stem, project)| (stem, project["ntaCalculation"].clone()))
        .collect();
    assert_eq!(bases.len(), 7);
    let pointers: Vec<Vec<String>> = bases
        .iter()
        .map(|(_, block)| {
            let mut out = Vec::new();
            all_pointers(block, String::new(), &mut out);
            out
        })
        .collect();
    let mut rng = Rng(0x0A11_C0DE_2026_1010);
    let cases = 600;
    let mut with_gaps = 0;
    for case in 0..cases {
        let base = rng.below(bases.len());
        let (stem, block) = &bases[base];
        let mut copy = block.clone();
        let count = 1 + rng.below(8);
        let mut chosen = Vec::new();
        for _ in 0..count {
            let pointer = &pointers[base][rng.below(pointers[base].len())];
            if let Some(slot) = copy.pointer_mut(pointer) {
                *slot = Value::Null;
                chosen.push(pointer.clone());
            }
        }
        let reduced = nta_blank_search(&copy, false);
        let whole = nta_blank_search(&copy, true);
        assert_eq!(
            reduced, whole,
            "case {case}: {stem} with {chosen:?} blanked"
        );
        if !whole.0.is_empty() {
            with_gaps += 1;
        }
    }
    assert!(
        with_gaps > cases / 4,
        "only {with_gaps} cases reported gaps"
    );
    let (confirmed, fell_back) = reduced_blank_searches();
    println!("reduced searches: {confirmed} confirmed, {fell_back} fell back");
    assert!(
        confirmed > 100,
        "{confirmed} confirmed, {fell_back} fell back"
    );
}

/// Review 27: the reduced blank search rests on each list element of the
/// NTA input deserializing on its own. Derived serde code does; a custom
/// `Deserialize`, `deserialize_with`, `untagged`, `flatten` or `from`/
/// `try_from` might not. The confirmation on the whole block catches a
/// wrong result (the search then runs on the whole block), but a new
/// construct of that kind should be looked at: this test lists the ones in
/// the kernel source outside test modules and fails on any that is not known.
#[test]
fn custom_deserialization_in_the_kernel_is_known() {
    // (file, construct, count); both are scalars, neither is around a list.
    let known = [
        // An explicit null for the BACS capacity (one f64 field).
        ("bacs_draft.rs", "deserialize_with", 1),
        // A survey answer that is a flag or a kind (survey route only).
        ("opname/utility.rs", "untagged", 1),
    ];
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    let mut stack = vec![source.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs")
                && !path.to_string_lossy().ends_with("_tests.rs")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut found = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        // Test modules are left out: from a `#[cfg(test)]` before a `mod`.
        let end = (0..lines.len())
            .find(|&index| {
                lines[index].trim() == "#[cfg(test)]"
                    && lines.get(index + 1).is_some_and(|next| {
                        let next = next.trim_start();
                        next.starts_with("mod ") || next.starts_with("pub mod ")
                    })
            })
            .unwrap_or(lines.len());
        let name = path
            .strip_prefix(&source)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        for construct in [
            "impl<'de>",
            "deserialize_with",
            "untagged",
            "flatten",
            "from =",
            "try_from =",
        ] {
            let count = lines[..end]
                .iter()
                .filter(|line| {
                    let line = line.trim_start();
                    let attribute = line.starts_with("#[serde(");
                    match construct {
                        "impl<'de>" | "deserialize_with" => line.contains(construct),
                        "from =" => {
                            attribute && (line.contains("(from =") || line.contains(" from ="))
                        }
                        _ => attribute && line.contains(construct),
                    }
                })
                .count();
            if count > 0 {
                found.push((name.clone(), construct, count));
            }
        }
    }
    let known: Vec<(String, &str, usize)> = known
        .iter()
        .map(|(file, construct, count)| (file.to_string(), *construct, *count))
        .collect();
    assert_eq!(
        found, known,
        "custom deserialization changed: check that every list element still \
         deserializes on its own (see `blank_paths` in project_performance.rs) \
         and update this list"
    );
}
