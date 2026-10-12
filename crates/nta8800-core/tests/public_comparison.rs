//! Regression on fictionalised rebuilds of public BENG reports
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
        beng2: 34.37,
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
/// A: 2024 34,37 → 2020+A1 35,29 (+0,56 PV, +0,36 the 2023 switch points of
/// emission and ΔT_C;fan; both with the declared f_ctrl 0,50 of the report);
/// B: 2022 equals 2023, 29,10. What remains against
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
    assert_indicators("A 2020+A1", a20_indicators, [94.0, 35.29, 74.4]);
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
/// four windows, with b_b = distance / width from the printout (2022
/// p. 673: D 3,01, L 1,33, I 0,36, H 0,12; all below 2,5 m), and full
/// obstruction on two (no geometry printed; cooling conditions taken as not
/// met, table 17.5). No linear bridges are printed, so ΔU_for applies. Two
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
    assert_indicators("E 2022", indicators, [88.61, 31.68, 72.6]);
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

/// Case F: a detached house of 231,61 m² with a pitched and a flat roof,
/// calculated on 08-03-2022 with Uniec 3.0.19.4 (NTA 8800:2020+A1);
/// published 74,69 / 2,59 / 97,5. Hot water (3 157 kWh: 5 849 kWh at COP
/// 1,95 and f_prac 0,95), fans (987 kWh with frost protection), the heating
/// pipe length (148,23 m forfait) and the declared heat-pump auxiliary
/// energy (55 kWh) agree. What differs:
/// - PV: Uniec counts 24 × 360 Wp = 8 640 Wp; the kernel takes (16.4) with
///   K_pk floored to 190 W/m² (2020 p. 651), 8 482 Wp, the value the label
///   page of the same report prints. Scaled to 8 640 Wp the kernel gives the
///   report's 7 226 kWh.
/// - Cooling: the literal 10.15 emission loss and the 10.87 control energy
///   (87,6 kWh/year), the known interpretation questions.
/// - BENG 1 is 0,8 % lower. Eight windows have a side obstruction; the
///   printout gives distance, width and height per window, so
///   `windowObstructions` carries b_b = distance / width (2020 p. 669) and,
///   for V0.5 and V0.6 (height ≥ 2,5 m), the cooling condition (p. 672).
///   Without them (minimal obstruction) BENG 1 was 73,49.
///
/// The roller shutters are a table 7.5 device: before this case a device
/// left `reductionFactor` NaN in the derived input and the finite-number
/// guard refused every such project.
#[test]
fn case_f_under_nta_8800_2020_a1() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-f.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2020+A1");
    let (performance, indicators) = run_edition(&value, "2020+A1");
    assert_indicators("F 2020+A1", indicators, [74.06, 3.79, 96.3]);
    assert!((used_kwh(&performance, "hotWater") - 3157.0).abs() < 1.0);
    assert!((used_kwh(&performance, "ventilation") - 987.0).abs() < 1.0);
    let heating = &performance["spaceHeating"];
    assert_eq!(
        heating["annualAuxiliaryElectricityKwh"].as_f64(),
        Some(55.0)
    );
    let pipe = heating["distribution"]["pipeLengthM"].as_f64().unwrap();
    assert!((pipe - 148.23).abs() < 0.01, "{pipe}");
    // Declared COP 4,60 · f_prac 0,95 = 4,37 (report: 13 664 kWh / 4,37 +
    // 200 kWh element = 3 326 kWh on 13 864 kWh heat; the kernel's heat is
    // 1,8 % lower, see BENG 1).
    assert_eq!(heating["generationEfficiency"].as_f64(), Some(4.37));
    assert!((used_kwh(&performance, "heating") - 3264.5).abs() < 1.0);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 7093.7).abs() < 1.0, "{pv}");
    assert!((pv * 8640.0 / (190.0 * 44.64) - 7226.0).abs() < 2.0, "{pv}");
    // 10.87: 0,010 kW in every month with a cooling generator, plus the pump.
    let cooling_auxiliary: f64 = performance["cooling"]["months"]
        .as_array()
        .unwrap()
        .iter()
        .map(|month| month["auxiliaryElectricityKwh"].as_f64().unwrap())
        .sum();
    assert!(
        (87.6..89.0).contains(&cooling_auxiliary),
        "{cooling_auxiliary}"
    );
    assert!((used_kwh(&performance, "cooling") - 146.4).abs() < 1.0);
}

/// A table 7.5/7.6 device leaves `reductionFactor` unset; it must not be
/// written out as a non-finite number.
#[test]
fn shading_device_serialises_without_reduction_factor() {
    use nta8800_core::solar_shading::MovableShading;
    let shading: MovableShading = serde_json::from_value(serde_json::json!({
        "device": {"kind": "external_roller_shutter", "colour": "white"},
        "control": "manual_residential",
        "sourceReference": "test"
    }))
    .unwrap();
    let written = serde_json::to_value(&shading).unwrap();
    assert!(written.get("reductionFactor").is_none(), "{written}");
    let again: MovableShading = serde_json::from_value(written).unwrap();
    assert!(again.reduction_factor.is_nan());
    let declared: MovableShading = serde_json::from_value(serde_json::json!({
        "reductionFactor": 0.3,
        "control": "manual_residential",
        "sourceReference": "test"
    }))
    .unwrap();
    assert_eq!(
        serde_json::to_value(&declared).unwrap()["reductionFactor"],
        0.3
    );
}

/// Case G: a detached dwelling with a heated basement, 369,10 m², calculated
/// on 29-04-2025 with Uniec 3.3.5.0 (NTA 8800:2024); published 64,52 /
/// 29,44 / 70,6. Fans (300 kWh), PV (4 164 kWh on the meter), the heating
/// COP (4,25, table 9.28 ground row at 40 °C) and hot water agree: the
/// generator heat of both systems is the report's 7 317 kWh and the
/// electricity its 5 545 kWh, with the net need of 13.15/13.18 (2024
/// p. 521–522), 856 · (1,28 + 0,01 · 369,10) = 4 255 kWh, and f_sto;dis;ls 2
/// for both vessels (all connections insulated; an electric boiler without
/// test data, 2024 p. 552–553). What differs, as BENG 2 (Δ kWh · 1,45 / A_g):
/// - The C.4c unit has the declared f_ctrl 0,51 of its BCRG
///   gelijkwaardigheidsverklaring (instead of table 11.5: 0,59), and the
///   roof window its dark external screens (table 7.5, per window), as in
///   the report.
/// - The basement walls take the entered 130,45 m² (ΣA_T;bw of K.1.2) in
///   A_ls (6.7.2) and in the 8.38 forfait, so A_ls is the printed 670,80 m²
///   and the BENG 1 limit the printed 64,52.
/// - Heating +70 kWh (+0,28): the heat need of three windows with
///   "overige belemmering" without geometry (§17.3.2g, heating table 17.13).
/// - Heating auxiliary about +5 kWh (+0,02).
/// - Cooling with its auxiliary about −108 kWh (−0,42): the literal 10.15
///   emission loss and the 10.84 regeneration surcharge (ground not shown
///   above 0 °C).
#[test]
fn case_g_under_nta_8800_2024() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-g.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2024");
    let (performance, indicators) = run_edition(&value, "2024");
    assert_indicators("G 2024", indicators, [65.07, 29.31, 72.5]);
    // A_ls with the entered 130,45 m² basement wall (6.7.2, 8.38 ΣA_T;bw):
    // 670,80 m² as printed.
    let limit = performance["bblCheck"]["limits"]["energyNeedMaxKwhPerM2"]
        .as_f64()
        .unwrap();
    assert!((limit - 64.52).abs() < 0.005, "{limit}");
    assert!((used_kwh(&performance, "ventilation") - 300.4).abs() < 1.0);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 4164.0).abs() < 1.0, "{pv}");
    let heating = &performance["spaceHeating"];
    assert_eq!(heating["generationEfficiency"].as_f64(), Some(4.25));
    assert!((used_kwh(&performance, "heating") - 4436.5).abs() < 1.0);
    let need = performance["hotWater"]["annualNetNeedKwh"]
        .as_f64()
        .unwrap();
    assert!(
        (need - 856.0 * (1.28 + 0.01 * 369.10)).abs() < 0.5,
        "{need}"
    );
    let output = performance["hotWater"]["annualGeneratorOutputKwh"]
        .as_f64()
        .unwrap();
    assert!((output - 7317.0).abs() < 1.0, "{output}");
    assert!((used_kwh(&performance, "hotWater") - 5545.0).abs() < 1.0);
}

/// Case H, the first utility case: a building with an office, an assembly
/// and a healthcare function and a gym (sport), 743,80 m² in two zones,
/// calculated on 29-05-2024 with Uniec 3.2.9.2 (NTA 8800:2023 period);
/// published 29,08 / 80,01 / 30,5 against limits 44,94 / 86,22 / 30,4.
/// The common space of 49,20 m² serves all four functions and is shared over
/// them pro rata for the building (Bbl limits, hot water, the lighting zone);
/// with that the kernel's Bbl limits equal the report's. Within RZ-1, where
/// it lies, it goes only to the functions of that zone (6.6.4, 2023 p. 145):
/// RZ-1 is office, assembly and healthcare, all 21 °C (6.5.3, table 7.13),
/// with no sport. Lighting (34 294 kWh, forfait power, manual on/off), hot
/// water (7 961 kWh, table 13.1 per function, COP 1,40, f_sto;dis;ls 4), PV
/// (9 652 kWh) and the heat-pump COP (3,05, table 9.29 outdoor air at 45 °C)
/// agree, and so do the heat (13 301 against 13 366 kWh), its auxiliary
/// energy (318 against 319) and the ambient heat (8 940 against 8 984). BENG 2
/// (Δ kWh · 1,45 / A_g): heating −21 (−0,04), cooling +798 (+1,56; the
/// literal 10.15 emission loss), the 10.87 cooling control energy +88
/// (+0,17) and the forfait fans +85 (+0,17). BENG 1 is 3,9 % higher; the
/// report gives no need per zone or per service to split it.
#[test]
fn case_h_utility_under_nta_8800_2023() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-h.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2023");
    let (performance, indicators) = run_edition(&value, "2023");
    assert_indicators("H 2023", indicators, [30.20, 81.86, 30.0]);
    let limits = &performance["bblCheck"]["limits"];
    let limit = |key: &str| limits[key].as_f64().unwrap();
    assert!((limit("energyNeedMaxKwhPerM2") - 44.94).abs() < 0.005);
    assert!((limit("primaryFossilMaxKwhPerM2") - 86.22).abs() < 0.005);
    assert!((limit("renewableShareMinPercent") - 30.4).abs() < 0.005);
    assert!((used_kwh(&performance, "lighting") - 34294.0).abs() < 1.0);
    assert!((used_kwh(&performance, "hotWater") - 7961.0).abs() < 1.0);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 9652.0).abs() < 1.0, "{pv}");
    assert_eq!(
        performance["spaceHeating"]["generationEfficiency"].as_f64(),
        Some(3.05)
    );
    // RZ-1 holds only its own functions (6.6.4), so its setpoint is 21 °C;
    // the heat and its auxiliary energy then agree with the report.
    let rz1 = &performance["spaceHeating"]["demand"];
    assert!((rz1["annualHeatingNeedKwh"].as_f64().unwrap() - 4457.2).abs() < 1.0);
    let heating = &performance["spaceHeating"];
    let output: f64 = heating["monthly"]
        .as_array()
        .unwrap()
        .iter()
        .map(|month| month["generatorOutputKwh"].as_f64().unwrap())
        .sum();
    assert!((output - 13301.0).abs() < 1.0, "{output}");
    let electricity = heating["annualGeneratorElectricityKwh"].as_f64().unwrap();
    assert!((electricity - 4361.0).abs() < 1.0, "{electricity}");
    let auxiliary = heating["annualAuxiliaryElectricityKwh"].as_f64().unwrap();
    assert!((auxiliary - 318.0).abs() < 1.0, "{auxiliary}");
    let ambient = performance["annualHeatPumpAmbientHeatKwh"]
        .as_f64()
        .unwrap();
    assert!((ambient - 8940.0).abs() < 1.0, "{ambient}");
}

/// Case H with the common space as input (6.6.4): the zone gives its own
/// initial function areas and the 49,20 m² common space, and the kernel
/// allocates it, within RZ-1 over office, assembly and healthcare, for the
/// building (Bbl limits) over all four functions. The results equal those of
/// the hand split in the fixture.
#[test]
fn case_h_common_space_input_equals_the_hand_split() {
    let hand: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-h.json"
    ))
    .unwrap();
    let mut input = hand.clone();
    let nta = &mut input["ntaCalculation"];
    let rz1 = &mut nta["zoneData"][0];
    rz1["functionAreas"] = serde_json::json!([
        {"function": "office", "areaM2": 14.70},
        {"function": "other_assembly", "areaM2": 26.30},
        {"function": "other_healthcare", "areaM2": 27.60}
    ]);
    rz1["commonSpaces"] = serde_json::json!([{
        "id": "gemeenschappelijk",
        "areaM2": 49.20,
        "sourceReference": "openbare bron Uniec 3.2.9.2 printout: gemeenschappelijke ruimte 49,20 m2 voor alle functies"
    }]);
    // The kernel writes the allocated areas over the zone's chapter 11
    // functions, whatever they were.
    rz1["ventilation"]["functions"] = rz1["functionAreas"].clone();
    nta.as_object_mut().unwrap().remove("bblFunctions");
    let (_, expected) = run_edition(&hand, "2023");
    let (performance, indicators) = run_edition(&input, "2023");
    for (got, want) in indicators.iter().zip(expected) {
        assert!(
            (got - want).abs() < 0.005,
            "{indicators:?} against {expected:?}"
        );
    }
    let limits = &performance["bblCheck"]["limits"];
    assert!((limits["energyNeedMaxKwhPerM2"].as_f64().unwrap() - 44.94).abs() < 0.005);
    assert!((limits["primaryFossilMaxKwhPerM2"].as_f64().unwrap() - 86.22).abs() < 0.005);
    let mut copy = input.clone();
    copy["ntaCalculation"]["normVersion"] = Value::from("2023");
    let result = assess_project_performance(&copy);
    let split = serde_json::to_value(result.common_spaces.as_ref().unwrap()).unwrap();
    let office = split["zones"][0]["allocated"][0]["areaM2"]
        .as_f64()
        .unwrap();
    assert!((office - 25.24).abs() < 0.01, "{split}");
    let sport = split["building"]
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part["function"] == "sport")
        .unwrap()["areaM2"]
        .as_f64()
        .unwrap();
    assert!((sport - 670.35).abs() < 0.01, "{split}");
    // Review 24: the label functions take the same building split as the
    // Bbl functions when none are given.
    let derived = result.derived_input.as_ref().unwrap();
    assert_eq!(derived.label_functions.len(), 4);
    assert_eq!(derived.bbl_functions.len(), 4);
    for (label, bbl) in derived.label_functions.iter().zip(&derived.bbl_functions) {
        assert!((label.area_m2 - bbl.area_m2).abs() < 1e-9);
    }
}

/// Review 25: own areas plus common space may differ from the zone area by up
/// to 1 % (common_space_area_mismatch, 6.6.3). The lists the kernel fills
/// then still cover the zone exactly (14.4 and 13.19 take A_g of the zone),
/// so a filled list never raises lighting_function_areas_invalid or
/// hot_water_function_areas_invalid on a field the adviser did not enter.
#[test]
fn filled_hot_water_and_lighting_lists_cover_the_zone_within_the_tolerance() {
    let hand: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-h.json"
    ))
    .unwrap();
    let mut input = hand.clone();
    let nta = &mut input["ntaCalculation"];
    let rz1 = &mut nta["zoneData"][0];
    // 0,5 % more own area than the zone holds next to its common space.
    rz1["functionAreas"] = serde_json::json!([
        {"function": "office", "areaM2": 15.29},
        {"function": "other_assembly", "areaM2": 26.30},
        {"function": "other_healthcare", "areaM2": 27.60}
    ]);
    rz1["commonSpaces"] = serde_json::json!([{
        "id": "gemeenschappelijk",
        "areaM2": 49.20,
        "sourceReference": "openbare bron Uniec 3.2.9.2 printout: gemeenschappelijke ruimte 49,20 m2 voor alle functies"
    }]);
    rz1["ventilation"]["functions"] = rz1["functionAreas"].clone();
    nta.as_object_mut().unwrap().remove("bblFunctions");
    nta["hotWater"]["need"]["areas"] = serde_json::json!([]);
    for lighting in nta["lighting"].as_array_mut().unwrap() {
        lighting.as_object_mut().unwrap().remove("functions");
    }
    let (_, indicators) = run_edition(&input, "2023");
    assert!(indicators.iter().all(|value| value.is_finite()));
    let mut copy = input.clone();
    copy["ntaCalculation"]["normVersion"] = Value::from("2023");
    let result = assess_project_performance(&copy);
    let derived = result.derived_input.as_ref().unwrap();
    let derived = serde_json::to_value(derived).unwrap();
    let zone_area = hand["zones"][0]["floorArea"].as_f64().unwrap();
    let lit: f64 = derived["lighting"][0]["functions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|part| part["areaM2"].as_f64().unwrap())
        .sum();
    assert!((lit - zone_area).abs() < 1e-9, "{lit} against {zone_area}");
    let building_area: f64 = hand["zones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|zone| zone["floorArea"].as_f64().unwrap())
        .sum();
    let served: f64 = derived["hotWater"]["need"]["areas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|part| part["areaM2"].as_f64().unwrap())
        .sum();
    assert!(
        (served - building_area).abs() < 1e-9,
        "{served} against {building_area}"
    );
}

/// Review 25: the lists may be omitted only because common spaces fill them.
/// Without common spaces an omitted list is still refused, on its own path.
#[test]
fn omitted_hot_water_and_lighting_lists_without_common_spaces_are_refused() {
    let mut input: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-example-office.json"
    ))
    .unwrap();
    let nta = &mut input["ntaCalculation"];
    nta["lighting"][0]
        .as_object_mut()
        .unwrap()
        .remove("functions");
    nta["hotWater"]["need"]
        .as_object_mut()
        .unwrap()
        .remove("areas");
    let result = assess_project_performance(&input);
    assert_eq!(result.status, "invalid", "{:?}", result.gaps);
    let paths: Vec<(&str, &str)> = result
        .gaps
        .iter()
        .map(|gap| (gap.code, gap.path.as_str()))
        .collect();
    assert!(
        paths.contains(&(
            "lighting_function_areas_invalid",
            "ntaCalculation.lighting[0].functions"
        )),
        "{paths:?}"
    );
    assert!(
        paths.contains(&(
            "hot_water_function_areas_invalid",
            "ntaCalculation.hotWater.need.areas"
        )),
        "{paths:?}"
    );
    let split = result.common_spaces.as_ref();
    assert!(split.is_none());
}

/// Case H with the common space as input and the hot-water and lighting
/// lists left empty: the kernel fills them with the within-zone split, as
/// 13.19 (2023 p. 527) and chapter 14 (t_D, t_N p. 643, P_n;spec p. 647)
/// weight by the use functions lying in the calculation zone (6.6.4, last
/// paragraph, p. 145). The result equals the same within-zone lists entered
/// by hand. Uniec 3.2.9.2 takes the building split for both instead (the
/// fixture); that reading gives slightly more hot water and lighting.
#[test]
fn case_h_common_space_fills_hot_water_and_lighting_within_the_zone() {
    let hand: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-h.json"
    ))
    .unwrap();
    let mut input = hand.clone();
    let nta = &mut input["ntaCalculation"];
    let rz1 = &mut nta["zoneData"][0];
    rz1["functionAreas"] = serde_json::json!([
        {"function": "office", "areaM2": 14.70},
        {"function": "other_assembly", "areaM2": 26.30},
        {"function": "other_healthcare", "areaM2": 27.60}
    ]);
    rz1["commonSpaces"] = serde_json::json!([{
        "id": "gemeenschappelijk",
        "areaM2": 49.20,
        "sourceReference": "openbare bron Uniec 3.2.9.2 printout: gemeenschappelijke ruimte 49,20 m2 voor alle functies"
    }]);
    rz1["ventilation"]["functions"] = rz1["functionAreas"].clone();
    nta.as_object_mut().unwrap().remove("bblFunctions");
    nta["hotWater"]["need"]["areas"] = serde_json::json!([]);
    for lighting in nta["lighting"].as_array_mut().unwrap() {
        lighting.as_object_mut().unwrap().remove("functions");
    }

    // The same within-zone lists entered by hand.
    let mut entered = input.clone();
    let rz1_within = serde_json::json!([
        {"function": "office", "areaM2": 25.24},
        {"function": "assembly_without_day_care", "areaM2": 45.16},
        {"function": "healthcare_without_beds", "areaM2": 47.40}
    ]);
    entered["ntaCalculation"]["lighting"][0]["functions"] = rz1_within.clone();
    entered["ntaCalculation"]["lighting"][1]["functions"] =
        serde_json::json!([{"function": "sport", "areaM2": 626.0}]);
    let mut areas = rz1_within.as_array().unwrap().clone();
    areas.push(serde_json::json!({"function": "sport", "areaM2": 626.0}));
    entered["ntaCalculation"]["hotWater"]["need"]["areas"] = Value::from(areas);
    // Exact within-zone shares of 49,20 m² over 14,70 / 26,30 / 27,60 m².
    let own = [14.70, 26.30, 27.60];
    let total: f64 = own.iter().sum();
    for (index, area) in own.iter().enumerate() {
        let allocated = area + 49.20 * area / total;
        entered["ntaCalculation"]["lighting"][0]["functions"][index]["areaM2"] =
            Value::from(allocated);
        entered["ntaCalculation"]["hotWater"]["need"]["areas"][index]["areaM2"] =
            Value::from(allocated);
    }

    let (filled_performance, filled) = run_edition(&input, "2023");
    let (entered_performance, by_hand) = run_edition(&entered, "2023");
    for (got, want) in filled.iter().zip(by_hand) {
        assert!((got - want).abs() < 1e-9, "{filled:?} against {by_hand:?}");
    }
    for service in ["hotWater", "lighting"] {
        let got = used_kwh(&filled_performance, service);
        let want = used_kwh(&entered_performance, service);
        assert!((got - want).abs() < 1e-6, "{service}: {got} against {want}");
    }

    let mut copy = input.clone();
    copy["ntaCalculation"]["normVersion"] = Value::from("2023");
    let result = assess_project_performance(&copy);
    let split = serde_json::to_value(result.common_spaces.as_ref().unwrap()).unwrap();
    let filled_paths: Vec<&str> = split["filled"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect();
    for path in [
        "bblFunctions",
        "labelFunctions",
        "lighting[0].functions",
        "lighting[1].functions",
        "hotWater.need.areas",
    ] {
        assert!(filled_paths.contains(&path), "{filled_paths:?}");
    }
    let sport = split["withinZones"]
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part["function"] == "sport")
        .unwrap()["areaM2"]
        .as_f64()
        .unwrap();
    assert!((sport - 626.0).abs() < 1e-9, "{split}");

    // Against Uniec's building split (the fixture): less hot water and
    // lighting, since the 44,35 m² sport share of the common space stays
    // with the functions of RZ-1.
    let (hand_performance, _) = run_edition(&hand, "2023");
    let hot_water =
        used_kwh(&filled_performance, "hotWater") - used_kwh(&hand_performance, "hotWater");
    let lighting =
        used_kwh(&filled_performance, "lighting") - used_kwh(&hand_performance, "lighting");
    eprintln!("case H within-zone minus building split: hot water {hot_water:.1} kWh, lighting {lighting:.1} kWh, BENG {filled:?}");
    assert!(hot_water < 0.0 && lighting < 0.0, "{hot_water} {lighting}");
}

/// Case I, the second utility case: an office with an assembly function and
/// a common space (439,04 m², one zone), calculated on 01-04-2021 with
/// Uniec 3.0.9.3 (NTA 8800:2020+A1 period); published 75,26 / −36,81 / 143,8
/// against limits 97,96 / 44,06 / 30,0. The common space is shared over the
/// two functions pro rata; with that the kernel's Bbl limits equal the
/// report's. Hot water (778 kWh, electric instantaneous 0,95), fans
/// (1 411 kWh), PV (30 621 kWh) and the air-to-air heat-pump COP (2,80)
/// agree. The construction is the table 7.10 row 250 to 500 kg/m², 110
/// kJ/(m²K) with the closed ceiling (2020+A1 p. 179–180), as printed; BENG 1
/// is +0,25. BENG 2 (Δ kWh · 1,45 / A_g): cooling +5 486 (+18,12; Q_C;dhum
/// of 12.5 for direct expansion, 2020+A1 p. 500–501, and the literal 10.15
/// emission loss), the 10.87 control energy +88 (+0,29), heating −339
/// (−1,12), its auxiliary energy −4 (−0,01) and lighting +38 (+0,13; P_n
/// rounded up per annex X). Of the cooling, Q_C;dhum is 13 885 kWh cold
/// (+15,29), the 10.15 loss 4 186 kWh (+4,61) and the sensible need 10 456
/// kWh (−1,78 against the report's 12 069 kWh of cold at EER 3,00). BENG 3
/// follows from BENG 2 alone: E_ren (PV 44 401 primary plus ambient heat
/// 8 042 kWh against the report's 8 653) is within 1,2 %, and with the
/// report's E_P the kernel's E_ren gives 144,5 % against 143,8 %.
#[test]
fn case_i_utility_under_nta_8800_2020_a1() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-i.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2020+A1");
    let (performance, indicators) = run_edition(&value, "2020+A1");
    assert_indicators("I 2020+A1", indicators, [75.51, -19.41, 119.4]);
    let limits = &performance["bblCheck"]["limits"];
    let limit = |key: &str| limits[key].as_f64().unwrap();
    assert!((limit("energyNeedMaxKwhPerM2") - 97.96).abs() < 0.005);
    assert!((limit("primaryFossilMaxKwhPerM2") - 44.06).abs() < 0.005);
    assert!((limit("renewableShareMinPercent") - 30.0).abs() < 0.005);
    assert!((used_kwh(&performance, "hotWater") - 778.0).abs() < 1.0);
    assert!((used_kwh(&performance, "ventilation") - 1411.0).abs() < 1.0);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 30621.0).abs() < 1.0, "{pv}");
    assert_eq!(
        performance["spaceHeating"]["generationEfficiency"].as_f64(),
        Some(2.8)
    );
    // The BENG 2 and BENG 3 gaps are cooling: Q_C;dhum (12.5) and the 10.15
    // loss, which Uniec 3.0.9.3 did not count for direct expansion in the room.
    let cooling = |key: &str| -> f64 {
        performance["cooling"]["months"]
            .as_array()
            .unwrap()
            .iter()
            .map(|month| month[key].as_f64().unwrap())
            .sum()
    };
    assert!((cooling("needKwh") - 10456.0).abs() < 1.0);
    assert!((cooling("dehumidificationKwh") - 13885.0).abs() < 1.0);
    assert!((cooling("emissionLossKwh") - 4186.0).abs() < 1.0);
    let renewable = performance["annualRenewablePrimaryKwh"].as_f64().unwrap();
    assert!((renewable - 52443.0).abs() < 1.0, "{renewable}");
    let ambient = performance["annualHeatPumpAmbientHeatKwh"]
        .as_f64()
        .unwrap();
    assert!((ambient - 8042.0).abs() < 1.0, "{ambient}");
}

/// Case J, a retail space of 89,80 m² under apartments, calculated on
/// 06-04-2022 with Uniec 3.0.19.4 (NTA 8800:2020+A1 period); published
/// 63,92 / 46,11 / 30,1 against limits 75,19 / 60,00 / 30,0. Hot water
/// (90 kWh: the exhaust-air heat pump at 1,4 without c_W;gen, table 13.25),
/// the heat-pump auxiliary energy (26 kWh, A 13,0 from 2015), fans (44 kWh),
/// lighting (2 162 kWh) and PV (1 235 kWh) agree. The construction is the
/// table 7.10 row above 750 kg/m², 250 kJ/(m²K) with the closed ceiling
/// (2020+A1 p. 180), as printed; BENG 1 is −0,17. BENG 2 (Δ kWh · 1,45 /
/// A_g): cooling +251 (+4,05; the literal 10.15 emission loss), the 10.87
/// control energy +87 (+1,41) and heating −7 (−0,11). h of (8.47) is not
/// printed and is taken as 0.
#[test]
fn case_j_retail_under_nta_8800_2020_a1() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-j.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2020+A1");
    let (performance, indicators) = run_edition(&value, "2020+A1");
    assert_indicators("J 2020+A1", indicators, [63.75, 51.46, 27.9]);
    let limits = &performance["bblCheck"]["limits"];
    let limit = |key: &str| limits[key].as_f64().unwrap();
    assert!((limit("energyNeedMaxKwhPerM2") - 75.19).abs() < 0.005);
    assert!((limit("primaryFossilMaxKwhPerM2") - 60.0).abs() < 0.005);
    assert!((limit("renewableShareMinPercent") - 30.0).abs() < 0.005);
    assert!((used_kwh(&performance, "hotWater") - 90.0).abs() < 1.0);
    assert!((used_kwh(&performance, "lighting") - 2162.0).abs() < 1.0);
    assert!((used_kwh(&performance, "ventilation") - 44.0).abs() < 1.0);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 1235.0).abs() < 1.0, "{pv}");
    let auxiliary = performance["spaceHeating"]["annualAuxiliaryElectricityKwh"]
        .as_f64()
        .unwrap();
    assert!((auxiliary - 26.0).abs() < 1.0, "{auxiliary}");
}

/// Case K, a sports hall with an office and an assembly function (1 896,98
/// m², two zones), calculated on 11-12-2022 with Uniec 3.1.5.0 (NTA 8800:2022
/// period); published 29,68 / −0,79 / 101,0 against limits 50,34 / 84,87 /
/// 30,0. The common space is shared over the two functions of its zone pro
/// rata; with that the kernel's Bbl limits and A_ls/A_g (1,99) equal the
/// report's. No linear thermal bridges are printed, so the forfait route of
/// 8.2/8.3 and 8.37 (2022 p. 200–201). Both zones are the 250–500 kg/m² row
/// of table 7.10 (2022 p. 181): 110 and, without a ceiling, 180 kJ/(m²K).
/// Hot water (27 769 kWh, electric boilers), fans (2 955 kWh), PV on the
/// meter (72 115 kWh) and the ground heat-pump COP (3,40, table 9.29 at
/// 35 °C) agree. BENG 2 (Δ kWh · 1,45 / A_g): heating +109 (+0,08) with
/// 0,9 % more heat, its auxiliary energy +7 (+0,01), free cooling and its
/// auxiliaries +27 (+0,02) and lighting −286 (−0,22).
#[test]
fn case_k_sports_hall_under_nta_8800_2022() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-k.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2022");
    let (performance, indicators) = run_edition(&value, "2022");
    assert_indicators("K 2022", indicators, [29.57, -0.9, 101.1]);
    let limits = &performance["bblCheck"]["limits"];
    let limit = |key: &str| limits[key].as_f64().unwrap();
    assert!((limit("energyNeedMaxKwhPerM2") - 50.34).abs() < 0.005);
    assert!((limit("primaryFossilMaxKwhPerM2") - 84.87).abs() < 0.005);
    assert!((limit("renewableShareMinPercent") - 30.0).abs() < 0.005);
    let ratio = performance["bblCheck"]["lossAreaRatio"].as_f64().unwrap();
    assert!((ratio - 1.99).abs() < 0.005, "{ratio}");
    assert!((used_kwh(&performance, "hotWater") - 27769.0).abs() < 1.0);
    assert!((used_kwh(&performance, "ventilation") - 2955.0).abs() < 1.0);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 72115.0).abs() < 1.0, "{pv}");
    assert_eq!(
        performance["spaceHeating"]["generationEfficiency"].as_f64(),
        Some(3.4)
    );
}

/// Case L, a primary school with a gym (1 653,96 m², two zones, education,
/// office, assembly and sport), calculated on 02-01-2023 with Uniec 3.1.5.0
/// (NTA 8800:2022 period); published 111,52 / 25,36 / 76,9 against limits
/// 140,15 / 74,99 / 36,1. Two common spaces are shared pro rata, one over
/// the three functions of its zone and the entrance over all four; with
/// that the kernel's limits and A_ls/A_g (1,96) equal the report's. Hot water
/// (10 233 kWh over two systems), lighting (21 501 kWh, LED L80 with
/// constant-illuminance compensation, 2022 table 14.4) and PV (46 017 kWh)
/// agree. BENG 2 (Δ kWh · 1,45 / A_g) is 6,73 lower: heating −6 236
/// (−5,47) and its auxiliary energy −307 (−0,27), free cooling −921
/// (−0,81) and fans −209 (−0,18). The report's heating (87 979 kWh heat)
/// with the AHU, flow control and the emitters of rooms above 8 m could not
/// be reproduced from the printout.
#[test]
fn case_l_school_under_nta_8800_2022() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-l.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2022");
    let (performance, indicators) = run_edition(&value, "2022");
    assert_indicators("L 2022", indicators, [111.44, 18.63, 80.2]);
    let limits = &performance["bblCheck"]["limits"];
    let limit = |key: &str| limits[key].as_f64().unwrap();
    assert!((limit("energyNeedMaxKwhPerM2") - 140.15).abs() < 0.005);
    assert!((limit("primaryFossilMaxKwhPerM2") - 74.99).abs() < 0.005);
    assert!((limit("renewableShareMinPercent") - 36.13).abs() < 0.005);
    let ratio = performance["bblCheck"]["lossAreaRatio"].as_f64().unwrap();
    assert!((ratio - 1.96).abs() < 0.005, "{ratio}");
    assert!((used_kwh(&performance, "hotWater") - 10233.0).abs() < 1.0);
    assert!((used_kwh(&performance, "lighting") - 21501.0).abs() < 1.0);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 46017.0).abs() < 0.5, "{pv}");
    assert_eq!(
        performance["spaceHeating"]["generationEfficiency"].as_f64(),
        Some(3.1)
    );
}

/// Case M, a small detached house with a pitched roof (43,44 m², A_ls/A_g
/// 4,40), calculated on 15-09-2022 with Uniec 3.1.3.1 (NTA 8800:2022);
/// published 171,29 / 26,25 / 88,3 against limits 174,95 / 30,00 / 50,0,
/// TOjuli 0 (active cooling). Forfait outdoor-air heat pump (table 9.28 row,
/// COP 3,15 at 40 °C, auxiliary 155 kWh), its hot water with an integrated
/// vessel (COP 1,40, 964 kWh), C.4c with the declared f_ctrl 0,51, PV (3 066
/// kWh on the meter) and the Bbl limits agree; heating is +11 kWh. BENG 2
/// (Δ kWh · 1,45 / A_g) is 21,60 higher, all cooling: the literal 10.15
/// emission loss (cooling electricity 1 496 against 949 kWh, +18,26; the
/// report's 2 846 kWh cold is the kernel's 2 449 kWh need plus 16 %) and the
/// 10.87 control energy (+87,6 kWh, +2,92). On this small floor area the
/// two interpretations take BENG 2 over its limit of 30. BENG 3 follows from
/// BENG 2 alone: E_ren is 8 649 against the report's 8 625 kWh (ambient heat
/// of heating 3 817 against 3 793, of hot water 386 in both, cooling 0 in
/// both, PV 4 446 in both); with the report's E_P (1 140 kWh) the kernel's
/// E_ren gives 88,35 % against 88,3 %.
#[test]
fn case_m_small_dwelling_with_cooling_under_nta_8800_2022() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-m.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2022");
    let (performance, indicators) = run_edition(&value, "2022");
    assert_indicators("M 2022", indicators, [170.87, 47.85, 80.6]);
    let limits = &performance["bblCheck"]["limits"];
    assert!((limits["energyNeedMaxKwhPerM2"].as_f64().unwrap() - 174.95).abs() < 0.005);
    let ratio = performance["bblCheck"]["lossAreaRatio"].as_f64().unwrap();
    assert!((ratio - 4.40).abs() < 0.005, "{ratio}");
    assert_eq!(
        performance["spaceHeating"]["generationEfficiency"].as_f64(),
        Some(3.15)
    );
    assert!((used_kwh(&performance, "heating") - 1764.0).abs() < 15.0);
    assert!((used_kwh(&performance, "hotWater") - 964.0).abs() < 1.0);
    assert!((used_kwh(&performance, "ventilation") - 11.0).abs() < 0.5);
    let heating_auxiliary = performance["spaceHeating"]["annualAuxiliaryElectricityKwh"]
        .as_f64()
        .unwrap();
    assert!(
        (heating_auxiliary - 155.0).abs() < 1.0,
        "{heating_auxiliary}"
    );
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 3066.0).abs() < 1.0, "{pv}");
    let need: f64 = performance["cooling"]["months"]
        .as_array()
        .unwrap()
        .iter()
        .map(|month| month["needKwh"].as_f64().unwrap())
        .sum();
    assert!((need - 2449.0).abs() < 1.0, "{need}");
    assert_eq!(performance["tojuliMaxK"].as_f64(), Some(0.0));
    // E_ren per term (report: heating 3 793, hot water 386, cooling 0,
    // electricity 4 446, total 8 625 kWh).
    let read = |key: &str| performance[key].as_f64().unwrap();
    let ambient = read("annualHeatPumpAmbientHeatKwh");
    assert!((ambient - 3817.0).abs() < 1.0, "{ambient}");
    let hot_water_ambient = performance["hotWater"]["annualGeneratorOutputKwh"]
        .as_f64()
        .unwrap()
        - used_kwh(&performance, "hotWater");
    assert!(
        (hot_water_ambient - 386.0).abs() < 1.0,
        "{hot_water_ambient}"
    );
    let renewable = read("annualRenewablePrimaryKwh");
    assert!((renewable - 8649.0).abs() < 1.0, "{renewable}");
    // With the report's E_P of 1 140 kWh the kernel's E_ren gives 88,3 %.
    let share = renewable / (renewable + 1140.0) * 100.0;
    assert!((share - 88.3).abs() < 0.1, "{share}");
}

/// Case N, a supermarket (2 064,26 m² retail over a basement entrance,
/// ground floor and first floor, above a strongly ventilated car park),
/// calculated on 02-02-2021 with Uniec 3.0.7.1 (NTA 8800:2020+A1);
/// published 55,98 / −27,18 / 126,9 against limits 95,31 / 60,00 / 30,0.
/// The strongly ventilated spaces count as outside (6.3, 2020+A1 p. 136)
/// without solar gain, as the report gives them no orientation. The basement
/// walls (116,07 m², z 3,65 m) are the heated-basement walls of the entrance
/// floor; with them in A_ls (f_ls 0,7) the kernel's A_ls/A_g (2,64) and Bbl
/// limits equal the report's. Table 7.10 row 250–500 kg/m², open ceiling
/// (180 kJ/(m²K), 2020+A1 p. 180). Hot water (3 175 kWh), the VRF COP 2,80
/// and PV on the meter (146 257 kWh, panel area 1,63 m² taken back from the
/// yield) agree. The supply-air reheating serves a 6–8 m room, so its
/// emission takes 9.3.3.7 (2020+A1 p. 277–280): Δθ_ctr,1 2,5 K of table 9.10
/// (the report prints the same), Δθ_str of 9.19 with the highest table 9.8
/// gradient (supply position unknown) at h_R 7 m, together 4,0 K; 9.16 caps
/// the loss at 0,15 in every month, so h_R and the gradient within the class
/// do not matter. BENG 2 (Δ kWh · 1,45 / A_g): heating +3 836 (+2,69; 13,1 %
/// more heat: need 80 201 + capped loss 12 030 + AHU 394 against the
/// report's 81 882), its auxiliaries +108 (+0,08), cooling +1 526 (+1,07; the
/// literal 10.15 loss of 3 026 kWh cold), the 10.87 control energy +88
/// (+0,06), lighting +1 407 (+0,99; P_n rounded up per annex X, 17 546 →
/// 18 000 W) and fans −2 881 (−2,02; the report takes q_V;ODA;req of the
/// cooling balance in 11.139, the kernel that of the heating balance).
#[test]
fn case_n_supermarket_under_nta_8800_2020_a1() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-n.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2020+A1");
    let (performance, indicators) = run_edition(&value, "2020+A1");
    // Review 44: the basement walls (31,80 m at z_j 3,65 m, the rest at 0)
    // take U_bw;j of 8.45 at 3,65 m, not at the mean z 1,66 m (8.39).
    assert_indicators("N 2020+A1", indicators, [54.52, -24.34, 122.7]);
    assert_eq!(
        performance["spaceHeating"]["emissionTemperatureIncrementK"].as_f64(),
        Some(4.0)
    );
    // 9.16: the emission loss is capped at 0,15 of the need in every month.
    for month in performance["spaceHeating"]["monthly"].as_array().unwrap() {
        let need = month["heatingNeedKwh"].as_f64().unwrap();
        let loss = month["emissionLossKwh"].as_f64().unwrap();
        assert!((loss - 0.15 * need).abs() < 1e-6, "{loss} {need}");
    }
    assert!((used_kwh(&performance, "heating") - 33007.0).abs() < 1.0);
    // Heat need of the actual system against fixed C1 (5.4.2). The report
    // implies about 71 000-74 000 kWh; see the doc section "Warmtebehoefte
    // van het werkelijke systeem".
    let demand = &performance["spaceHeating"]["demand"];
    let need = demand["annualHeatingNeedKwh"].as_f64().unwrap();
    assert!((need - 80023.0).abs() < 1.0, "{need}");
    let c1 = demand["fixedC1"]["annualHeatingNeedKwh"].as_f64().unwrap();
    assert!((c1 - 93511.0).abs() < 1.0, "{c1}");
    // Infiltration without a measurement: 11.86 gives 0,5 · 1,2 · 0,7 =
    // 0,42 dm³/(s·m²), 11.85 q_v1 = 0,42/10^0,67 · A_g · 3,6. In January
    // the mass balance of 11.5 (table 11.1, H 9,40 m) lets in 629,5 m³/h
    // next to 876,1 m³/h mechanical supply at 18 °C (table 11.15).
    let ventilation = &demand["ventilation"];
    let q1 = ventilation["infiltrationQV1M3PerH"].as_f64().unwrap();
    assert!((q1 - 667.29).abs() < 0.01, "{q1}");
    let january = &ventilation["months"][0]["heating"];
    let infiltration = january["infiltrationM3PerH"].as_f64().unwrap();
    assert!((infiltration - 629.5).abs() < 0.1, "{infiltration}");
    let supply = january["mechanicalSupplyM3PerH"].as_f64().unwrap();
    assert!((supply - 876.1).abs() < 0.1, "{supply}");
    assert_eq!(january["mechanicalSupplyTemperatureC"].as_f64(), Some(18.0));
    // 7.28 with f_L 0,5: the lighting heat of the actual system is
    // 0,5 · 60 961 kWh / 8 760 h = 3 479 W (1,69 W/m²); the C1 run takes the
    // fixed q_L of 3 W/m² for retail (5.4.2).
    let lighting_gain = performance["lighting"][0]["internalGainW"]
        .as_f64()
        .unwrap();
    assert!((lighting_gain - 3479.5).abs() < 0.1, "{lighting_gain}");
    let limits = &performance["bblCheck"]["limits"];
    let limit = |key: &str| limits[key].as_f64().unwrap();
    assert!((limit("energyNeedMaxKwhPerM2") - 95.31).abs() < 0.005);
    assert!((limit("primaryFossilMaxKwhPerM2") - 60.0).abs() < 0.005);
    assert!((limit("renewableShareMinPercent") - 30.0).abs() < 0.005);
    let ratio = performance["bblCheck"]["lossAreaRatio"].as_f64().unwrap();
    assert!((ratio - 2.64).abs() < 0.005, "{ratio}");
    assert_eq!(
        performance["spaceHeating"]["generationEfficiency"].as_f64(),
        Some(2.8)
    );
    assert!((used_kwh(&performance, "hotWater") - 3175.0).abs() < 1.0);
    assert!((used_kwh(&performance, "lighting") - 60961.0).abs() < 1.0);
    assert!((used_kwh(&performance, "ventilation") - 2283.0).abs() < 1.0);
    let pv: f64 = performance["pvSystems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|system| system["annualKwh"].as_f64().unwrap())
        .sum();
    assert!((pv - 146257.0).abs() < 1.0, "{pv}");
    let installed = performance["lighting"][0]["lightingZones"][0]["installedPowerW"]
        .as_f64()
        .unwrap();
    assert_eq!(installed, 18000.0);
}

/// Case O, a healthcare function without a bed area (451,50 m², one zone)
/// on the ground floor of an apartment building, calculated on 24-02-2023
/// with Uniec 3.1.5.0 (NTA 8800:2022); published 87,26 / 49,49 / 45,1
/// against limits 90,58 / 50,00 / 40,0, label A+++. The first public case on
/// external heat delivery: a quality declaration for heating and hot water
/// (f_P;del 0,35, f_P;ren 0,60, k_CO2 0,068, calculated values so f_prac
/// 0,95 in 9.84 and 13.152). Hot water (2 079 kWh heat), fans (1 485 kWh),
/// lighting (13 129 kWh), auxiliary energy (389 + 88 kWh), PV on the meter
/// (4 619 kWh), the limits and A_ls/A_g (1,82, with the wall to the bicycle
/// store at f_ls 1 and the floor at 0,7) agree. BENG 2 is 1,74 lower, all
/// heating: 16 134 against 18 374 kWh heat (Δ · 0,35 / A_g). The kernel's
/// AHU heating coil takes 279 kWh with c_a as printed in 11.119 (0,000 027 9
/// kWh/(kg·K), 2022 p. 487); with the physical 0,000 279 it is 2 794 kWh
/// and BENG 2 49,80 — see question 7 to NEN.
#[test]
fn case_o_healthcare_on_district_heat_under_nta_8800_2022() {
    let value: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-public-comparison-o.json"
    ))
    .unwrap();
    assert_eq!(value["ntaCalculation"]["normVersion"], "2022");
    let (performance, indicators) = run_edition(&value, "2022");
    assert_indicators("O 2022", indicators, [88.25, 47.75, 44.2]);
    assert_eq!(performance["indicativeLabelClass"], "A+++");
    let limits = &performance["bblCheck"]["limits"];
    let limit = |key: &str| limits[key].as_f64().unwrap();
    assert!((limit("energyNeedMaxKwhPerM2") - 90.58).abs() < 0.005);
    assert!((limit("primaryFossilMaxKwhPerM2") - 50.0).abs() < 0.005);
    assert!((limit("renewableShareMinPercent") - 40.0).abs() < 0.005);
    let ratio = performance["bblCheck"]["lossAreaRatio"].as_f64().unwrap();
    assert!((ratio - 820.13 / 451.5).abs() < 0.0005, "{ratio}");
    let declared = &performance["externalSupply"]["declared"];
    assert_eq!(declared["practiceHeat"].as_f64(), Some(0.95));
    assert_eq!(declared["practiceHotWater"].as_f64(), Some(0.95));
    assert_eq!(
        declared["districtHeat"]["primaryFactor"].as_f64(),
        Some(0.35)
    );
    assert!((used_kwh(&performance, "heating") - 16134.0).abs() < 1.0);
    assert!((used_kwh(&performance, "hotWater") - 2079.0).abs() < 1.0);
    assert!((used_kwh(&performance, "ventilation") - 1485.0).abs() < 1.0);
    assert!((used_kwh(&performance, "lighting") - 13129.0).abs() < 1.0);
    let auxiliary = performance["spaceHeating"]["annualAuxiliaryElectricityKwh"]
        .as_f64()
        .unwrap();
    assert!((auxiliary - 389.0).abs() < 1.0, "{auxiliary}");
    assert!((used_kwh(&performance, "auxiliary") - 476.4).abs() < 1.0);
    let pv = performance["pvSystems"][0]["annualKwh"].as_f64().unwrap();
    assert!((pv - 4619.0).abs() < 1.0, "{pv}");
    // The forfait pipe length of 9.36 with 15 % outside the heated zone is
    // the report's 245,62 + 43,34 m.
    let pipe = performance["spaceHeating"]["distribution"]["pipeLengthM"]
        .as_f64()
        .unwrap();
    assert!((pipe - 288.96).abs() < 0.01, "{pipe}");
    let coil: f64 = performance["spaceHeating"]["monthly"]
        .as_array()
        .unwrap()
        .iter()
        .map(|month| month["ahuHeatingLoadKwh"].as_f64().unwrap())
        .sum();
    assert!((coil - 279.4).abs() < 0.5, "{coil}");
}
