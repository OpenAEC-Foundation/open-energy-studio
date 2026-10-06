//! Regression on fictionalised rebuilds of three public BENG reports
//! (docs/nta8800-vergelijking-openbare-rapporten.md). The asserted values
//! are this kernel's own results, recorded on 2026-10-05 after the input
//! corrections of the line-by-line reconciliation and f_prac 0,95 on declared
//! heat-pump efficiencies; the published
//! values come from older NTA 8800 editions and are listed for context only.
//! This is not an official reference test.

use nta8800_core::project_performance::assess_project_performance;
use serde_json::Value;

struct Case {
    json: &'static str,
    name: &'static str,
    beng1: f64,
    beng2: f64,
    beng3: f64,
}

const CASES: [Case; 3] = [
    Case {
        json: include_str!("../../../training-data/nta8800-public-comparison-a.json"),
        name: "A (published 92,99 / 25,19 / 80,4)",
        beng1: 94.0,
        beng2: 35.15,
        beng3: 74.9,
    },
    Case {
        json: include_str!("../../../training-data/nta8800-public-comparison-b.json"),
        name: "B (published 54,61 / 27,10 / 64,8)",
        beng1: 52.96,
        beng2: 28.84,
        beng3: 62.6,
    },
    Case {
        json: include_str!("../../../training-data/nta8800-public-comparison-c.json"),
        name: "C (published 64,47 / 28,70 / 70,9)",
        beng1: 64.7,
        beng2: 31.54,
        beng3: 69.0,
    },
];

fn close(actual: Option<f64>, expected: f64, tolerance: f64) -> bool {
    actual.is_some_and(|value| (value - expected).abs() <= tolerance)
}

#[test]
fn public_comparison_cases_keep_their_recorded_results() {
    for case in &CASES {
        let value: Value = serde_json::from_str(case.json).unwrap();
        let result = assess_project_performance(&value);
        assert_eq!(
            result.status, "calculated_unverified",
            "{}: {:?}",
            case.name, result.gaps
        );
        let performance = result.performance.as_ref().expect(case.name);
        let indicators = serde_json::to_value(performance).unwrap();
        let read = |key: &str| indicators.pointer(key).and_then(Value::as_f64);
        let beng1 = read("/needIndicatorKwhPerM2Year");
        let beng2 = read("/primaryFossilIndicatorKwhPerM2Year");
        let beng3 = read("/renewableSharePercent");
        assert!(
            close(beng1, case.beng1, 0.05),
            "{} BENG 1: {beng1:?}",
            case.name
        );
        assert!(
            close(beng2, case.beng2, 0.05),
            "{} BENG 2: {beng2:?}",
            case.name
        );
        assert!(
            close(beng3, case.beng3, 0.15),
            "{} BENG 3: {beng3:?}",
            case.name
        );
    }
}

/// 6.2b (p. 160): one dwelling over two zones counts its occupants once, so
/// the residential gains add up to those of the whole dwelling.
#[test]
fn one_dwelling_over_two_zones_shares_its_occupants() {
    let value: Value = serde_json::from_str(CASES[2].json).unwrap();
    let result = assess_project_performance(&value);
    let derived = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
    let mut shares = Vec::new();
    let mut collect = |gains: &Value| {
        if let Some(share) = gains.pointer("/dwellingShare").and_then(Value::as_f64) {
            shares.push(share);
        }
    };
    if let Some(gains) = derived.pointer("/spaceHeating/demand/internalGains") {
        collect(gains);
    }
    if let Some(zones) = derived
        .pointer("/spaceHeating/additionalZones")
        .and_then(Value::as_array)
    {
        for zone in zones {
            if let Some(gains) = zone.pointer("/demand/internalGains") {
                collect(gains);
            }
        }
    }
    assert_eq!(shares.len(), 2, "{shares:?}");
    assert!(
        (shares.iter().sum::<f64>() - 1.0).abs() < 1e-9,
        "{shares:?}"
    );
}

/// Cases B and C were published in NTA 8800:2023. Under that edition the
/// table 13.2 difference of case C disappears: η_W;em;k 0,55 for a kitchen
/// pipe of at most 10 mm inner diameter (2023 p. 533) instead of 0,43
/// (2024 p. 527) lowers BENG 2 by 1,35 kWh/(m²·jr). What remains against
/// the reports is the reading of 10.15 and 10.87 and the building-physics
/// rest (docs/nta8800-vergelijking-openbare-rapporten.md).
#[test]
fn cases_b_and_c_under_nta_8800_2023() {
    fn run(json: &str, edition: &str, kitchen_10_mm: bool) -> [f64; 3] {
        let mut value: Value = serde_json::from_str(json).unwrap();
        value["ntaCalculation"]["normVersion"] = Value::from(edition);
        if kitchen_10_mm {
            let emission = &mut value["ntaCalculation"]["hotWater"]["emission"];
            assert!(emission.get("kitchenLengthM").is_some(), "{emission}");
            emission["kitchenPipeDiameter"] = Value::from("up_to_10_mm");
        }
        let result = assess_project_performance(&value);
        assert_eq!(
            result.status, "calculated_legacy_edition",
            "{edition}: {:?}",
            result.gaps
        );
        let indicators = serde_json::to_value(result.performance.as_ref().unwrap()).unwrap();
        let read = |key: &str| indicators.pointer(key).and_then(Value::as_f64).unwrap();
        [
            read("/needIndicatorKwhPerM2Year"),
            read("/primaryFossilIndicatorKwhPerM2Year"),
            read("/renewableSharePercent"),
        ]
    }
    let [b, c] = [&CASES[1], &CASES[2]];
    // B: the 2023 switch points move BENG 2 by +0,11 (BENG 1 unchanged);
    // the report does not give B's kitchen pipe as ≤ 10 mm.
    let b23 = run(b.json, "2023", false);
    assert!((b23[0] - b.beng1).abs() <= 0.05, "{b23:?}");
    assert!((b23[1] - 28.95).abs() <= 0.05, "{b23:?}");
    assert!((b23[2] - 62.5).abs() <= 0.15, "{b23:?}");
    // C: 31,54 (2024) → 30,19 (2023), published 28,70; BENG 3 69,0 → 69,9,
    // published 70,9.
    let c24 = run(c.json, "2024", false);
    let c23 = run(c.json, "2023", true);
    assert!((c24[1] - c.beng2).abs() <= 0.05, "{c24:?}");
    assert!((c24[1] - c23[1] - 1.35).abs() <= 0.02, "{c24:?} {c23:?}");
    assert!((c23[2] - 69.9).abs() <= 0.15, "{c23:?}");
    assert!((c23[0] - c.beng1).abs() <= 0.05, "{c23:?}");
}
