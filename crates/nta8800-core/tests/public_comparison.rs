//! Regression on fictionalised rebuilds of five public BENG reports
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

fn run_edition(value: &Value, edition: &str) -> (Value, [f64; 3]) {
    let mut value = value.clone();
    value["ntaCalculation"]["normVersion"] = Value::from(edition);
    let result = assess_project_performance(&value);
    assert_eq!(
        result.status, "calculated_legacy_edition",
        "{edition}: {:?}",
        result.gaps
    );
    let performance = serde_json::to_value(result.performance.as_ref().unwrap()).unwrap();
    let read = |key: &str| performance.pointer(key).and_then(Value::as_f64).unwrap();
    let indicators = [
        read("/needIndicatorKwhPerM2Year"),
        read("/primaryFossilIndicatorKwhPerM2Year"),
        read("/renewableSharePercent"),
    ];
    (performance, indicators)
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

fn assert_indicators(name: &str, actual: [f64; 3], expected: [f64; 3]) {
    let tolerance = [0.05, 0.05, 0.15];
    for i in 0..3 {
        assert!(
            (actual[i] - expected[i]).abs() <= tolerance[i],
            "{name} BENG {}: {actual:?}",
            i + 1
        );
    }
}

/// Case A was calculated on 13-04-2021 (Uniec 3.0.16), in the designation
/// period of NTA 8800:2020+A1; case B on 22-03-2023 (Uniec 3.1.6.2), in that
/// of NTA 8800:2022. Under their own editions:
/// - 2022 and 2020+A1 take the real height h of (8.47) (2022 p. 236); both
///   reports give 0,10 m for the floor above the crawlspace.
/// - 2020+A1 has no actual pipe length for dwellings (2020 p. 292); the
///   48,84 m of A is the forfait length of the report ("leidinggegevens
///   onbekend"), which 9.36 reproduces.
/// - 2020+A1 (16.4, p. 651) floors K_pk to 5 W/m²: 325 Wp on 9 panels of
///   15,21 m² together gives 192,3 → 190 W/m², the 2 437 kWh of the report.
///
/// A: 2024 35,15 → 2020+A1 36,06 (+0,56 PV, +0,36 the 2023 switch points of
/// emission and ΔT_C;fan); B: 2022 equals 2023, 29,10. What remains against
/// the reports is 10.15, 10.87 and the rest
/// (docs/nta8800-vergelijking-openbare-rapporten.md).
#[test]
fn cases_a_and_b_under_their_own_editions() {
    let with_height = |json: &str| {
        let mut value: Value = serde_json::from_str(json).unwrap();
        value["ntaCalculation"]["groundFloors"][0]["below"]["wallHeightAboveGroundM"] =
            Value::from(0.10);
        value
    };
    let mut a = with_height(CASES[0].json);
    a["ntaCalculation"]["distributionSystem"]
        .as_object_mut()
        .unwrap()
        .remove("actualPipeLengthM")
        .expect("A gives the forfait length");
    a["ntaCalculation"]["pvSystems"][0]["peakPower"] = serde_json::json!({
        "method": "declared_specific",
        "peakPowerWPerM2": 325.0 / 1.69,
        "panelAreaM2": 15.21
    });
    let (a20, a20_indicators) = run_edition(&a, "2020+A1");
    assert_indicators("A 2020+A1", a20_indicators, [94.0, 36.06, 74.4]);
    // 15,21 m² × 190 W/m²: the report's 2 437 kWh within the rounding.
    let pv = a20["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 2437.0).abs() < 5.0, "{pv}");

    let b = with_height(CASES[1].json);
    // The same as under 2023 (`cases_b_and_c_under_nta_8800_2023`): h 0,10
    // instead of the fixed 0,125 m changes nothing at two decimals.
    let (_, b22) = run_edition(&b, "2022");
    assert_indicators("B 2022", b22, [52.96, 29.10, 62.5]);
}

/// Case D: a detached holiday home of 79,70 m², calculated on 30-03-2021
/// with Uniec 3.0.10.0 (NTA 8800:2020+A1); published 86,72 / 39,19 / 83,5.
/// Hot water (3 552 kWh), fans (401 kWh, table 11.23 from 2007) and PV
/// (4 273 kWh on the meter) agree. The heat-pump auxiliary energy follows
/// 9.85 with A 13,0 for a device from 2015 (installation year 2021; the
/// report's 46 kWh is that formula on its own 2 427 kWh). The constant
/// overhang on the east glazing (9,84 m², size not printed) is a
/// per-window obstruction with h_o;⊥ 0,25: the class h_o;⊥ ≤ 0,4 of
/// tables 17.8/17.9 is the only one that gives the report's TOjuli 0,29
/// (kernel 0,27); BENG 1 is then 0,4 % and BENG 2 0,12 below the report.
#[test]
fn case_d_under_nta_8800_2020_a1() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-d.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2020+A1");
    let (performance, indicators) = run_edition(&value, "2020+A1");
    assert_indicators("D 2020+A1", indicators, [86.35, 39.07, 83.6]);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 4273.0).abs() < 1.0, "{pv}");
    assert!((used_kwh(&performance, "hotWater") - 3552.0).abs() < 1.0);
    assert!((used_kwh(&performance, "ventilation") - 401.0).abs() < 1.0);
    // 9.85 with A 13,0 from 2015 (2020 p. 334–336): 13,0 + 0,132 · E / (0,4 · 24).
    let heating = used_kwh(&performance, "heating");
    let auxiliary = used_kwh(&performance, "auxiliary");
    assert!(
        (auxiliary - (13.0 + 0.132 * heating / 9.6)).abs() < 0.5,
        "{auxiliary}"
    );
    // The report: 2 427 kWh for 12 136 kWh heat at COP 5,00.
    assert!((heating - 2420.6).abs() < 1.0, "{heating}");
    // The report: TOjuli 0,29; minimal obstruction on the east glazing gave 0,67.
    let tojuli = performance["tojuli"][0]["maxTojuliK"].as_f64().unwrap();
    assert!((tojuli - 0.27).abs() < 0.005, "{tojuli}");
}

/// Case E: a detached house of 209,40 m², calculated on 31-01-2023 with
/// Uniec 3.1.5.0 (NTA 8800:2022); published 86,82 / 28,89 / 74,1, TOjuli 0
/// (active cooling). Obstructions differ per window: side obstructions on
/// four windows (width not printed, b_b 0,5 assumed; 0,1–2,0 moves BENG 1 by
/// at most 0,16) and full obstruction on two (cooling conditions not met,
/// table 17.5). No linear bridges are printed, so ΔU_for applies. Two
/// hot-water systems split the need by 13.19a: the heat pump serves the
/// bathroom, a 7 l boiling-water boiler the kitchen. Fans (690 kWh), PV
/// (3 543 kWh on the meter) and the heating auxiliary energy (341 kWh) agree.
#[test]
fn case_e_under_nta_8800_2022() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-e.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2022");
    let obstructions = value["ntaCalculation"]["windowObstructions"]
        .as_array()
        .unwrap();
    assert_eq!(obstructions.len(), 6);
    let (performance, indicators) = run_edition(&value, "2022");
    assert_indicators("E 2022", indicators, [88.77, 31.73, 72.6]);
    assert!((used_kwh(&performance, "ventilation") - 690.0).abs() < 1.0);
    let produced: f64 = performance["electricityBalance"]
        .as_array()
        .unwrap()
        .iter()
        .map(|month| month["producedKwh"].as_f64().unwrap())
        .sum();
    assert!((produced - 3543.0).abs() < 1.0, "{produced}");
    let heating_auxiliary = performance["spaceHeating"]["annualAuxiliaryElectricityKwh"]
        .as_f64()
        .unwrap();
    assert!(
        (heating_auxiliary - 341.0).abs() < 1.0,
        "{heating_auxiliary}"
    );
    assert_eq!(performance["tojuliMaxK"].as_f64(), Some(0.0));
}

/// Case C was calculated in NTA 8800:2023, case B in the 2022 period (2022
/// equals 2023 for B, see above). Under 2023 the
/// table 13.2 difference of case C disappears: η_W;em;k 0,55 for a kitchen
/// pipe of at most 10 mm inner diameter (2023 p. 533) instead of 0,43
/// (2024 p. 527) lowers BENG 2 by 1,35 kWh/(m²·jr). The other 2023 switch
/// points raise BENG 2: ΔT_C;fan 0,7 K instead of 0,4 K for dwellings
/// (11.3.2.7, 2023 p. 496 / 2024 p. 491; case B +0,11, C 0,00), the
/// heating emission of tables 9.2–9.4 (2023 p. 275–280; B +0,13, C +0,24)
/// and the cooling emission of tables 10.2–10.5 (2023 p. 362–364; +0,02
/// each), both from the 2024 inputs with the unknown values of 2023. What
/// remains against the reports is the reading of 10.15 and 10.87 and the
/// building-physics rest (docs/nta8800-vergelijking-openbare-rapporten.md).
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
    // B: BENG 1 unchanged; BENG 2 28,84 → 29,10 (+0,11 ΔT_C;fan, +0,13
    // heating emission, +0,02 cooling emission); the report does not give
    // B's kitchen pipe as ≤ 10 mm.
    let b23 = run(b.json, "2023", false);
    assert!((b23[0] - b.beng1).abs() <= 0.05, "{b23:?}");
    assert!((b23[1] - 29.10).abs() <= 0.05, "{b23:?}");
    assert!((b23[2] - 62.5).abs() <= 0.15, "{b23:?}");
    // C: 31,54 (2024) → 30,44 (2023: −1,35 table 13.2, +0,24 heating and
    // +0,02 cooling emission), published 28,70; BENG 3 69,0 → 69,8,
    // published 70,9.
    let c24 = run(c.json, "2024", false);
    let c23 = run(c.json, "2023", true);
    assert!((c24[1] - c.beng2).abs() <= 0.05, "{c24:?}");
    assert!((c23[1] - 30.44).abs() <= 0.05, "{c23:?}");
    assert!((c23[2] - 69.8).abs() <= 0.15, "{c23:?}");
    assert!((c23[0] - c.beng1).abs() <= 0.05, "{c23:?}");
}
