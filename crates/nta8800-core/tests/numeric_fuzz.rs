//! Numeric fuzz: boundary and adversarial values for every number of the
//! project inputs, in every edition. It complements `option_coverage.rs`,
//! which reaches every option but keeps the numbers of the fixtures.
//!
//! The inputs are the example projects and the public comparison cases.
//! Four passes:
//! - **boundaries:** per numeric position (a pointer with its array indices
//!   wildcarded), one occurrence gets each adversarial value in turn: 0, −0,
//!   −1, the negated value, 10⁻¹², 10¹², 10³⁰⁰, a fraction for an integer,
//!   and the edges of the domain the member's name implies (a year, a
//!   fraction or factor, an angle);
//! - **every leaf:** each numeric leaf of each fixture once, with one of
//!   those values in rotation, so every number is touched;
//! - **combinations:** a seeded xorshift scales a few real-valued leaves of
//!   one fixture together by factors in [0,5; 2];
//! - **monotonicity:** more heat loss (U values and infiltration up, Rc
//!   down) never lowers the heating need.
//!
//! Every run must keep the rules of the option sweep (`support::check_outcome`):
//! no panic, only finite numbers, a non-calculated result only with codes
//! that have NL/EN labels and with project gaps that route in `gapRoutes.ts`.
//! A calculated result must also be plausible: the energy and heating need
//! are not negative and the renewable share lies in [0, 100] %; in the
//! combination pass, whose inputs stay realistic, the need also stays below
//! 10⁴ kWh/m².

use nta8800_core::norm_versions::NormVersion;
use nta8800_core::project_performance::assess_project_performance;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use support::{check_outcome, checked, label_keys, labelled, panic_message, set_pointer};

mod support;

macro_rules! fixture {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!("../../../training-data/nta8800-", $name, ".json")),
        )
    };
}

fn fixtures() -> Vec<(&'static str, Value)> {
    [
        fixture!("example-office"),
        fixture!("example-terraced-dwelling"),
        fixture!("public-comparison-a"),
        fixture!("public-comparison-b"),
        fixture!("public-comparison-c"),
        fixture!("public-comparison-d"),
        fixture!("public-comparison-e"),
        fixture!("public-comparison-f"),
        fixture!("public-comparison-g"),
        fixture!("public-comparison-h"),
    ]
    .into_iter()
    .map(|(name, json)| (name, serde_json::from_str(json).unwrap()))
    .chain(std::iter::once(declared_window_shading()))
    .collect()
}

/// Case G with its roof-window screen given as a declared `F_c` instead of
/// the table 7.5 device, so the numeric leaf of a per-window shading
/// (`windowShadings[].movableShading.reductionFactor`, 7.43) is fuzzed too.
fn declared_window_shading() -> (&'static str, Value) {
    let mut value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-g.json"
    ))
    .unwrap();
    let shading = &mut value["ntaCalculation"]["windowShadings"][0]["movableShading"];
    let object = shading.as_object_mut().expect("case G has a window shading");
    object.remove("device");
    object.insert("reductionFactor".into(), Value::from(0.25));
    ("public-comparison-g (declared F_c per window)", value)
}

/// A numeric leaf: its pointer, its value and whether it is an integer.
struct Leaf {
    pointer: String,
    value: f64,
    integer: bool,
}

fn leaves(value: &Value, pointer: String, out: &mut Vec<Leaf>) {
    match value {
        Value::Number(number) => out.push(Leaf {
            pointer,
            value: number.as_f64().unwrap(),
            integer: number.is_i64() || number.is_u64(),
        }),
        Value::Object(map) => {
            for (key, item) in map {
                leaves(item, format!("{pointer}/{}", support::escape(key)), out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                leaves(item, format!("{pointer}/{index}"), out);
            }
        }
        _ => {}
    }
}

/// The pointer with its array indices wildcarded.
fn position(pointer: &str) -> String {
    pointer
        .split('/')
        .map(|segment| {
            if !segment.is_empty() && segment.bytes().all(|b| b.is_ascii_digit()) {
                "*"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// The adversarial values for a leaf, labelled.
fn adversarial(leaf: &Leaf) -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = vec![
        ("0".into(), Value::from(0)),
        ("-0".into(), Value::from(-0.0_f64)),
        ("-1".into(), Value::from(-1)),
        ("1e-12".into(), Value::from(1e-12)),
        ("1e12".into(), Value::from(1e12)),
        ("1e300".into(), Value::from(1e300)),
    ];
    if leaf.value != 0.0 {
        out.push(("negated".into(), Value::from(-leaf.value)));
    }
    if leaf.integer {
        out.push(("fraction".into(), Value::from(leaf.value + 0.5)));
    }
    let member = leaf
        .pointer
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let declared_control_factor = leaf
        .pointer
        .to_ascii_lowercase()
        .contains("/declaredcontrolfactor/");
    let edges: &[f64] = if declared_control_factor {
        // f_ctrl from a declaration: 0 < f_ctrl ≤ 2 (table 11.5 values lie
        // in that range; the kernel's bound).
        &[2.0, 2.0 + 1e-9, 1e-9, -1e-9]
    } else if member.ends_with("year") && !member.ends_with("peryear") {
        &[1899.0, 1900.0, 2100.0, 2101.0]
    } else if member.contains("fraction")
        || member.contains("factor")
        || member.contains("efficiency")
        || member == "gvalue"
        || member.contains("share")
    {
        &[1.0, 1.0 + 1e-9, -1e-9]
    } else if member == "azimuthdeg" {
        &[360.0, 360.5, -0.5]
    } else if member == "tiltdeg" {
        &[90.0, 180.0, 180.5, -0.5]
    } else if member.ends_with('c') && member.contains("setpoint")
        || member.ends_with("temperaturec")
    {
        &[-273.15, -300.0, 150.0]
    } else {
        &[]
    };
    for edge in edges {
        out.push((format!("edge {edge}"), Value::from(*edge)));
    }
    out
}

/// A tiny deterministic generator (xorshift64*), so the combinations are the
/// same on every run without a dependency.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// Uniform in [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// What a pass expects of a calculated result beyond the shared rules.
#[derive(Clone, Copy, PartialEq)]
enum Plausible {
    /// Need and share within their sign and range.
    Signs,
    /// Also a need below 10⁴ kWh/m² (realistic inputs only).
    Bounded,
}

struct Job {
    label: String,
    fixture: usize,
    edits: Vec<(String, Value)>,
    plausible: Plausible,
    /// The editions to run; the single-leaf pass rotates over them, the
    /// other passes run all five.
    editions: Vec<NormVersion>,
}

/// The heating need of a calculated result, per edition, for the
/// monotonicity pass.
type Needs = BTreeMap<(usize, String, NormVersion), f64>;

#[derive(Default)]
struct Tally {
    runs: usize,
    calculated: usize,
    failures: Vec<String>,
    codes: BTreeSet<String>,
    needs: Needs,
    /// The labels of the adversarial inputs that calculated, for
    /// `NUMERIC_FUZZ_REPORT=1`.
    accepted: BTreeSet<String>,
}

fn indicator(value: &Value, name: &str) -> Option<f64> {
    value
        .pointer(&format!("/labelData/indicators/{name}"))
        .and_then(Value::as_f64)
}

/// The plausibility rules for a calculated result.
fn implausible(value: &Value, plausible: Plausible) -> Option<String> {
    for name in ["energyNeedKwhPerM2", "heatingNeedKwhPerM2"] {
        if let Some(need) = indicator(value, name) {
            if need < 0.0 {
                return Some(format!("{name} {need} < 0"));
            }
            if plausible == Plausible::Bounded && need > 1e4 {
                return Some(format!("{name} {need} > 1e4 for a realistic input"));
            }
        }
    }
    // (5.3): RER_PrenTot = E_PrenTot / (E_PTot + E_PrenTot) × 100 %, rounded
    // down to 0,1 % (2025+C1 p. 73–74). E_PTot can be negative (export), so
    // the share can exceed 100 %; it must match the formula, and E_PrenTot
    // is never negative.
    let element = |name: &str| {
        value
            .pointer(&format!("/labelData/indicators/elements/{name}"))
            .and_then(Value::as_f64)
    };
    if let (Some(share), Some(renewable), Some(fossil)) = (
        indicator(value, "renewableSharePercent"),
        element("annualRenewablePrimaryKwh"),
        element("annualPrimaryFossilKwh"),
    ) {
        if renewable < -1e-6 {
            return Some(format!("annualRenewablePrimaryKwh {renewable} < 0"));
        }
        let total = fossil + renewable;
        if total > 1e-6 * renewable.abs().max(1.0) {
            let expected = (renewable / total * 1000.0 + 1e-9).floor() / 10.0;
            if (share - expected).abs() > 0.1 + 1e-9 {
                return Some(format!(
                    "renewableSharePercent {share} ≠ (5.3) {expected} \
                     (E_Pren {renewable}, E_PTot {fossil})"
                ));
            }
        }
    }
    None
}

fn run_one(input: &Value, plausible: Plausible) -> Result<(bool, BTreeSet<String>, Value), String> {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let result = assess_project_performance(input);
        Ok::<_, String>((checked(&result)?, result.status.to_string()))
    }));
    let (value, status) = match outcome {
        Err(panic) => return Err(format!("panic: {}", panic_message(&*panic))),
        Ok(result) => result?,
    };
    let (calculated, codes) = check_outcome(true, &value, &status)?;
    if calculated {
        if let Some(problem) = implausible(&value, plausible) {
            return Err(format!("implausible: {problem}"));
        }
    }
    Ok((calculated, codes, value))
}

fn run_jobs(fixtures: &[(&str, Value)], jobs: &[Job]) -> Tally {
    let next = AtomicUsize::new(0);
    let tally = Mutex::new(Tally::default());
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(16);
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(job) = jobs.get(index) else { break };
                let mut input = fixtures[job.fixture].1.clone();
                for (pointer, value) in &job.edits {
                    set_pointer(&mut input, pointer, value.clone());
                }
                let mut local = Tally::default();
                for &version in &job.editions {
                    let mut edition = input.clone();
                    if edition.get("ntaCalculation").is_some() {
                        set_pointer(
                            &mut edition,
                            "/ntaCalculation/normVersion",
                            serde_json::to_value(version).unwrap(),
                        );
                    }
                    local.runs += 1;
                    match run_one(&edition, job.plausible) {
                        Ok((calculated, codes, value)) => {
                            local.calculated += usize::from(calculated);
                            local.codes.extend(codes);
                            if calculated && job.plausible == Plausible::Signs {
                                local.accepted.insert(job.label.clone());
                            }
                            if calculated {
                                if let Some(need) = indicator(&value, "heatingNeedKwhPerM2") {
                                    local
                                        .needs
                                        .insert((job.fixture, job.label.clone(), version), need);
                                }
                            }
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
                tally.needs.extend(local.needs);
                tally.accepted.extend(local.accepted);
            });
        }
    });
    std::panic::set_hook(hook);
    tally.into_inner().unwrap()
}

/// The monotonicity edits of one fixture: every leaf whose member name
/// matches, scaled together. Raising U or infiltration, or lowering Rc,
/// raises the transmission or ventilation loss Q_ht; the monthly heating
/// need Q_H;nd = Q_ht − η_gn·Q_gn does not fall when Q_ht rises
/// (η_gn·γ grows with γ and stays ≤ 1, and a larger H shortens τ), so the
/// heating need may only stay or grow.
const MONOTONE: [(&str, &str, f64); 5] = [
    ("window U up", "/zones/*/surfaces/*/windows/*/uValue", 1.25),
    ("construction U up", "/constructions/*/uValue", 1.25),
    ("construction Rc down", "/constructions/*/rcValue", 0.8),
    ("qv10 up", "/zones/*/airTightness/qv10", 1.25),
    (
        "infiltration up",
        "/ntaCalculation/ventilation/infiltration/qv10DmPerSM2",
        1.25,
    ),
];

#[test]
fn every_number_at_its_edges_calculates_plausibly_or_refuses_with_a_routed_gap() {
    let fixtures = fixtures();
    let all: Vec<Vec<Leaf>> = fixtures
        .iter()
        .map(|(_, value)| {
            let mut out = Vec::new();
            leaves(value, String::new(), &mut out);
            out
        })
        .collect();
    let mut jobs = Vec::new();

    // Boundaries: one occurrence per position, rotating over the fixtures
    // that have it, with every adversarial value.
    let mut occurrences: BTreeMap<String, Vec<(usize, usize)>> = BTreeMap::new();
    for (fixture, leaves) in all.iter().enumerate() {
        for (index, leaf) in leaves.iter().enumerate() {
            occurrences
                .entry(position(&leaf.pointer))
                .or_default()
                .push((fixture, index));
        }
    }
    let positions = occurrences.len();
    for (rank, (position, places)) in occurrences.iter().enumerate() {
        let (fixture, index) = places[rank % places.len()];
        let leaf = &all[fixture][index];
        for (name, value) in adversarial(leaf) {
            jobs.push(Job {
                label: format!("{} {position} = {name}", fixtures[fixture].0),
                fixture,
                edits: vec![(leaf.pointer.clone(), value)],
                plausible: Plausible::Signs,
                editions: NormVersion::ALL.to_vec(),
            });
        }
    }
    let boundary_jobs = jobs.len();

    // Every leaf once, with a rotating adversarial value in a rotating
    // edition: the boundary pass above already runs each position in all
    // five editions.
    let mut leaf_count = 0;
    for (fixture, leaves) in all.iter().enumerate() {
        for (index, leaf) in leaves.iter().enumerate() {
            leaf_count += 1;
            let values = adversarial(leaf);
            let (name, value) = values[(index * 7 + fixture) % values.len()].clone();
            let edition = NormVersion::ALL[(index + fixture) % NormVersion::ALL.len()];
            jobs.push(Job {
                label: format!("{} {} = {name}", fixtures[fixture].0, leaf.pointer),
                fixture,
                edits: vec![(leaf.pointer.clone(), value)],
                plausible: Plausible::Signs,
                editions: vec![edition],
            });
        }
    }
    let leaf_jobs = jobs.len() - boundary_jobs;

    // Combinations: a few real-valued leaves scaled together. Integers
    // (counts, years) stay, so the input stays realistic.
    let mut random = XorShift(0x5EED_8800_2026_1009);
    for combination in 0..COMBINATIONS {
        let fixture = random.below(fixtures.len());
        let reals: Vec<&Leaf> = all[fixture].iter().filter(|leaf| !leaf.integer).collect();
        let count = 2 + random.below(3);
        let mut edits = Vec::new();
        let mut names = Vec::new();
        for _ in 0..count {
            let leaf = reals[random.below(reals.len())];
            let factor = 0.5 + 1.5 * random.unit();
            edits.push((leaf.pointer.clone(), Value::from(leaf.value * factor)));
            names.push(format!("{}×{factor:.3}", leaf.pointer));
        }
        jobs.push(Job {
            label: format!(
                "{} combination {combination}: {}",
                fixtures[fixture].0,
                names.join(", ")
            ),
            fixture,
            edits,
            plausible: Plausible::Bounded,
            editions: NormVersion::ALL.to_vec(),
        });
    }

    // Monotonicity: the unchanged fixture and each loss-raising edit.
    let mut monotone_jobs = 0;
    for (fixture, leaves) in all.iter().enumerate() {
        jobs.push(Job {
            label: "baseline".into(),
            fixture,
            edits: Vec::new(),
            plausible: Plausible::Bounded,
            editions: NormVersion::ALL.to_vec(),
        });
        for (name, pattern, factor) in MONOTONE {
            let edits: Vec<(String, Value)> = leaves
                .iter()
                .filter(|leaf| position(&leaf.pointer) == pattern && leaf.value > 0.0)
                .map(|leaf| (leaf.pointer.clone(), Value::from(leaf.value * factor)))
                .collect();
            if !edits.is_empty() {
                monotone_jobs += 1;
                jobs.push(Job {
                    label: name.into(),
                    fixture,
                    edits,
                    plausible: Plausible::Bounded,
                    editions: NormVersion::ALL.to_vec(),
                });
            }
        }
    }

    let tally = run_jobs(&fixtures, &jobs);

    let mut failures = tally.failures;
    for ((fixture, label, version), need) in &tally.needs {
        if label == "baseline" || !MONOTONE.iter().any(|(name, ..)| name == label) {
            continue;
        }
        let Some(base) = tally
            .needs
            .get(&(*fixture, "baseline".to_string(), *version))
        else {
            continue;
        };
        // The indicator is rounded to two decimals.
        if *need < base - 0.011 {
            failures.push(format!(
                "{} {version:?}: {label} lowers the heating need {base} → {need}",
                fixtures[*fixture].0
            ));
        }
    }
    let (nl, en) = label_keys();
    let unlabelled: Vec<&String> = tally
        .codes
        .iter()
        .filter(|code| !labelled(&nl, code) || !labelled(&en, code))
        .collect();
    println!(
        "numeric fuzz: {} fixtures, {leaf_count} numeric leaves in {positions} positions; \
         {boundary_jobs} boundary inputs (5 editions), {leaf_jobs} single-leaf inputs (1 \
         edition each, rotating), {COMBINATIONS} combinations and {monotone_jobs} \
         monotonicity edits (5 editions); {} inputs, {} runs ({} calculated); {} distinct codes",
        fixtures.len(),
        jobs.len(),
        tally.runs,
        tally.calculated,
        tally.codes.len()
    );
    if std::env::var_os("NUMERIC_FUZZ_REPORT").is_some() {
        for label in &tally.accepted {
            println!("calculated: {label}");
        }
    }
    failures.sort();
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert!(
        unlabelled.is_empty(),
        "codes without NL/EN label: {unlabelled:?}"
    );
    // Coverage may grow but not silently shrink.
    assert!(positions >= 110, "{positions}");
    for required in [
        "/ntaCalculation/ventilation/system/unit/declaredControlFactor/value",
        "/ntaCalculation/windowShadings/*/movableShading/reductionFactor",
    ] {
        assert!(
            occurrences.contains_key(required),
            "the fuzz no longer reaches {required}"
        );
    }
    assert!(leaf_count >= 1700, "{leaf_count}");
    assert!(monotone_jobs >= 20, "{monotone_jobs}");
}

const COMBINATIONS: usize = 200;
