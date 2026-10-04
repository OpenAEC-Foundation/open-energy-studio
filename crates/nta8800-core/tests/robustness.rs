//! Robustness: randomly perturbed fixtures must never make the kernel panic
//! or hand out a non-finite number.
//!
//! A reduced, deterministic version of the robustness fuzzer: each trial
//! changes one to three numeric leaves (or empties an array) of a fixture
//! to zero, tiny, huge, negative or extreme values and runs the project or
//! survey assessment under `catch_unwind`. The maatwerkadvies route runs
//! the editor's template measures (training-data/nta8800-mwa-template-
//! measures.json) on the terraced example. `ROBUST_TRIALS` raises the
//! number of trials per fixture for a local run.

use nta8800_core::finite::first_non_finite;
use nta8800_core::maatwerkadvies::assess_maatwerkadvies_json;
use nta8800_core::opname::utility::{assess_utility_survey, UtilitySurvey};
use nta8800_core::opname::{assess_residential_survey, ResidentialSurvey};
use nta8800_core::project_performance::assess_project_performance;
use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

#[derive(Clone, Copy)]
enum Kind {
    Project,
    Residential,
    Utility,
    Maatwerkadvies,
}

const FIXTURES: [(&str, Kind, &str); 9] = [
    (
        "office",
        Kind::Project,
        include_str!("../../../training-data/nta8800-example-office.json"),
    ),
    (
        "terraced",
        Kind::Project,
        include_str!("../../../training-data/nta8800-example-terraced-dwelling.json"),
    ),
    (
        "synthetic",
        Kind::Project,
        include_str!("../../../training-data/nta8800-project-performance-synthetic.json"),
    ),
    (
        "opname-1930",
        Kind::Residential,
        include_str!("../../../training-data/nta8800-opname-1930-terraced.json"),
    ),
    (
        "opname-1975",
        Kind::Residential,
        include_str!("../../../training-data/nta8800-opname-1975-apartment.json"),
    ),
    (
        "opname-2015",
        Kind::Residential,
        include_str!("../../../training-data/nta8800-opname-2015-detached.json"),
    ),
    (
        "utility-1970",
        Kind::Utility,
        include_str!("../../../training-data/nta8800-opname-utility-1970-retail.json"),
    ),
    (
        "utility-1985",
        Kind::Utility,
        include_str!("../../../training-data/nta8800-opname-utility-1985-office.json"),
    ),
    (
        "utility-2005",
        Kind::Utility,
        include_str!("../../../training-data/nta8800-opname-utility-2005-school.json"),
    ),
];

/// xorshift64: deterministic and dependency-free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// JSON pointers of the numeric leaves (false) and arrays (true).
fn leaves(value: &Value, pointer: String, out: &mut Vec<(String, bool)>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                let key = key.replace('~', "~0").replace('/', "~1");
                leaves(item, format!("{pointer}/{key}"), out);
            }
        }
        Value::Array(items) => {
            out.push((pointer.clone(), true));
            for (index, item) in items.iter().enumerate() {
                leaves(item, format!("{pointer}/{index}"), out);
            }
        }
        Value::Number(_) => out.push((pointer, false)),
        _ => {}
    }
}

fn mutation(rng: &mut Rng, base: &Value, pointer: &str, array: bool) -> Value {
    if array {
        return json!([]);
    }
    let current = base.pointer(pointer).cloned().unwrap_or(Value::Null);
    if current.is_u64() || current.is_i64() {
        let v = current.as_i64().unwrap_or(0);
        let choices = [
            0,
            1,
            -1,
            v.saturating_add(1),
            v.saturating_mul(10),
            100_000,
            u32::MAX as i64,
        ];
        return json!(choices[rng.below(choices.len())]);
    }
    let v = current.as_f64().unwrap_or(0.0);
    let choices = [
        0.0,
        1e-9,
        -v,
        -1.0,
        v * 1e3,
        v * 1e6,
        1e12,
        1e300,
        -1e300,
        1.0,
        1e-300,
        5e-324,
        1.0001,
    ];
    json!(choices[rng.below(choices.len())])
}

/// Maatwerkadvies input: the template measures on the terraced example.
fn maatwerkadvies_fixture() -> Value {
    let measures: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-mwa-template-measures.json"
    ))
    .expect("template fixture");
    let project: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-example-terraced-dwelling.json"
    ))
    .expect("project fixture");
    json!({
        "base": {"kind": "project", "project": project},
        "measures": measures["terracedDwelling"],
        "packages": [],
        "tariffs": {
            "gasEurPerM3": 1.4,
            "electricityEurPerKwh": 0.3,
            "electricityExportEurPerKwh": 0.05,
            "sourceReference": "robustness test"
        }
    })
}

/// Outcome of one trial.
enum Trial {
    /// The kernel ran and gave finite numbers.
    Ran,
    /// The perturbed input does not deserialize (a negative count in a
    /// `u32`, say); nothing was calculated.
    Skipped,
}

/// Runs one assessment; `Err` describes a panic, a non-finite number in
/// the output, or a result the safety net had to refuse (`non_finite_result`:
/// some computation produced NaN or ±∞ and needs a guard of its own).
fn run(kind: Kind, input: &Value) -> Result<Trial, String> {
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<Trial, String> {
        let problem = match kind {
            Kind::Project => {
                let result = assess_project_performance(input);
                result
                    .gaps
                    .iter()
                    .find(|gap| gap.code == "non_finite_result")
                    .map(|gap| format!("refused: {:?}", gap.detail))
                    .or_else(|| first_non_finite(&result))
            }
            Kind::Residential => {
                let Ok(survey) = serde_json::from_value::<ResidentialSurvey>(input.clone()) else {
                    return Ok(Trial::Skipped);
                };
                let result = assess_residential_survey(&survey);
                refused_survey(&result).or_else(|| first_non_finite(&result))
            }
            Kind::Utility => {
                let Ok(survey) = serde_json::from_value::<UtilitySurvey>(input.clone()) else {
                    return Ok(Trial::Skipped);
                };
                let result = assess_utility_survey(&survey);
                refused_survey(&result).or_else(|| first_non_finite(&result))
            }
            Kind::Maatwerkadvies => {
                let Ok(result) = assess_maatwerkadvies_json(input.clone()) else {
                    return Ok(Trial::Skipped);
                };
                result
                    .issues
                    .iter()
                    .find(|issue| issue.code == "non_finite_result")
                    .map(|issue| format!("refused: {}", issue.path))
                    .or_else(|| first_non_finite(&result))
            }
        };
        match problem {
            Some(problem) => Err(format!("non-finite number: {problem}")),
            None => Ok(Trial::Ran),
        }
    }));
    outcome.unwrap_or_else(|_| Err("panic".into()))
}

fn refused_survey(result: &nta8800_core::opname::OpnameAssessment) -> Option<String> {
    result
        .issues
        .iter()
        .find(|issue| issue.code == "non_finite_result")
        .map(|issue| format!("refused: {}", issue.path))
}

/// Smallest share of the trials of a fixture that must reach the kernel.
/// Perturbations a fixture cannot even deserialize (a negative or huge count
/// in a `u32`) are skipped; if most were skipped, the test would pass
/// without exercising the kernel.
const MIN_EXECUTED_SHARE: f64 = 0.5;

#[test]
fn perturbed_fixtures_never_panic_or_give_non_finite_numbers() {
    let trials: usize = std::env::var("ROBUST_TRIALS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(30);
    let seed: u64 = std::env::var("ROBUST_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0x1234_5678_9abc_def1);
    // Silence the caught panics of this thread only. The hook is process
    // wide, so panics of tests running in parallel still reach the
    // previous hook and keep their messages.
    let previous = Arc::new(std::panic::take_hook());
    if std::env::var("ROBUST_SHOW").is_err() {
        let fuzz_thread = std::thread::current().id();
        let others = Arc::clone(&previous);
        std::panic::set_hook(Box::new(move |info| {
            if std::thread::current().id() != fuzz_thread {
                others(info);
            }
        }));
    } else {
        let all = Arc::clone(&previous);
        std::panic::set_hook(Box::new(move |info| all(info)));
    }
    let mut fixtures: Vec<(&str, Kind, Value, usize)> = FIXTURES
        .iter()
        .map(|(name, kind, text)| {
            (
                *name,
                *kind,
                serde_json::from_str(text).expect("fixture"),
                trials,
            )
        })
        .collect();
    // Each maatwerkadvies trial runs a dozen variants; fewer trials suffice.
    fixtures.push((
        "maatwerkadvies",
        Kind::Maatwerkadvies,
        maatwerkadvies_fixture(),
        (trials / 3).max(5),
    ));
    let mut failures = Vec::new();
    for (index, (name, kind, base, trials)) in fixtures.iter().enumerate() {
        match run(*kind, base) {
            Ok(Trial::Ran) => {}
            Ok(Trial::Skipped) => failures.push(format!("{name} baseline does not deserialize")),
            Err(error) => failures.push(format!("{name} baseline: {error}")),
        }
        let mut points = Vec::new();
        leaves(base, String::new(), &mut points);
        let mut rng = Rng(seed ^ (index as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut executed = 0usize;
        for _ in 0..*trials {
            let mut input = base.clone();
            let mut changes = Vec::new();
            for _ in 0..1 + rng.below(3) {
                let (pointer, array) = &points[rng.below(points.len())];
                let value = mutation(&mut rng, base, pointer, *array);
                if let Some(slot) = input.pointer_mut(pointer) {
                    *slot = value.clone();
                }
                changes.push(format!("{pointer}={value}"));
            }
            match run(*kind, &input) {
                Ok(Trial::Ran) => executed += 1,
                Ok(Trial::Skipped) => {}
                Err(error) => failures.push(format!("{name} {changes:?}: {error}")),
            }
        }
        let share = executed as f64 / *trials as f64;
        if share < MIN_EXECUTED_SHARE {
            failures.push(format!(
                "{name}: only {executed} of {trials} trials reached the kernel"
            ));
        }
    }
    let _ = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| previous(info)));
    for failure in &failures {
        eprintln!("{failure}");
    }
    assert!(
        failures.is_empty(),
        "{} robustness failure(s)",
        failures.len()
    );
}

#[test]
fn unbounded_storey_counts_are_refused_before_allocating() {
    let mut residential: Value = serde_json::from_str(FIXTURES[4].2).unwrap();
    residential["heating"]["storeys"] = json!(u32::MAX);
    if let Some(map) = residential.as_object_mut() {
        map.remove("verticalPipes");
        map.remove("storeys");
    }
    let survey: ResidentialSurvey = serde_json::from_value(residential).unwrap();
    let result = assess_residential_survey(&survey);
    assert_eq!(result.status, "invalid");
    assert!(result
        .issues
        .iter()
        .any(|issue| issue.code == "storeys_out_of_range" && issue.path == "heating.storeys"));

    let mut utility: Value = serde_json::from_str(FIXTURES[7].2).unwrap();
    utility["storeys"] = json!(u32::MAX);
    utility["toiletStacks"] = json!(u32::MAX);
    if let Some(map) = utility.as_object_mut() {
        map.remove("verticalPipes");
    }
    let survey: UtilitySurvey = serde_json::from_value(utility).unwrap();
    let result = assess_utility_survey(&survey);
    assert_eq!(result.status, "invalid");
    let codes: Vec<_> = result
        .issues
        .iter()
        .map(|issue| (issue.code, issue.path.as_str()))
        .collect();
    assert!(
        codes.contains(&("storeys_out_of_range", "storeys")),
        "{codes:?}"
    );
    assert!(
        codes.contains(&("count_out_of_range", "toiletStacks")),
        "{codes:?}"
    );
}

#[test]
fn extreme_project_values_block_and_unusual_ones_warn() {
    let base: Value = serde_json::from_str(FIXTURES[1].2).unwrap();
    let mut huge = base.clone();
    huge["zones"][0]["floorArea"] = json!(1e12);
    let result = assess_project_performance(&huge);
    assert_eq!(result.status, "invalid");
    assert!(result.performance.is_none());
    assert!(result
        .gaps
        .iter()
        .any(|gap| gap.code == "zone_area_out_of_range"));

    let mut leaky = base.clone();
    let pointer = "/ntaCalculation/ventilation/infiltration/qv10DmPerSM2";
    *leaky.pointer_mut(pointer).unwrap() = json!(20.0);
    let result = assess_project_performance(&leaky);
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.code == "qv10_out_of_range"));
    assert_eq!(result.status, "calculated_unverified");
}
