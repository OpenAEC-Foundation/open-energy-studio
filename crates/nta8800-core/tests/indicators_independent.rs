//! Review 75: chapter 5 indicators, chapter 6 areas, chapter 12, §5.5.8, annexes M and O and the calculated
//! route of annex P outside the example projects, recalculated independently from NTA 8800:2025+C1:2026.
//! The expected values come from a separate implementation written from the norm pages without first reading
//! the kernel code of the route; only numbers and page references are kept here. Inputs from chapters 7, 9,
//! 11 and 16 (verified in reviews 63-70) are taken from the kernel where a route needs them. Tolerance 1e-6
//! relative.
//!
//! Readings where the text leaves a choice and the kernel's choice is kept (see the review notes in
//! docs/nta8800-verificatiestatus.md): annex M, M.25 with Q_gen;out of M.24 (f_ctr;ls included), M.12 with
//! η_gen;Pn;60 for condensing boilers, M.1 not below 0, M.15 as M.16 + M.19; annex O, the interpolation of
//! the load curve (one test ignored, see `annex_o_on_time_keeps_the_120_s_product`); annex P, 5.48 with
//! 1 − 1/η of P.6.5.4.8 instead of the printed 0,95.

use nta8800_core::annex_m::{boiler_month, BoilerMonth, ProductBoiler};
use nta8800_core::annex_o::{
    auxiliary_constants, monthly_auxiliary_kwh, AppliancePowerMeasurements,
};
use nta8800_core::annex_p::{assess_route, AnnexPRoute, SystemFunction};
use nta8800_core::bbl_requirements::{bbl_limits, bbl_limits_mixed, BblFunction, BblFunctionArea};
use nta8800_core::building_performance::standard_insulation;
use nta8800_core::humidification::{calculate_humidity, HumidityInput};
use nta8800_core::label_class::{renovation_standard, LabelFunction, LabelFunctionArea};
use nta8800_core::project_performance::assess_project_performance;
use nta8800_core::zoning::{check_zone, CalculationZoneLayout};
use serde_json::{json, Value};

const OFFICE: &str = include_str!("../../../training-data/nta8800-example-office.json");
const DWELLING: &str =
    include_str!("../../../training-data/nta8800-example-terraced-dwelling.json");
const EPU001: &str = include_str!("../../../training-data/isso54/EPU001.json");
const EPW001: &str = include_str!("../../../training-data/isso54/EPW001.json");
const SUITE: &str = include_str!("../../../training-data/reference-suites/isso54-v2.json");
const CASE_H: &str = include_str!("../../../training-data/nta8800-public-comparison-h.json");

fn close(label: &str, kernel: f64, expected: f64) {
    let diff = (kernel - expected).abs();
    let scale = kernel.abs().max(expected.abs()).max(1e-12);
    assert!(
        diff <= 1e-9 || diff / scale <= 1e-6,
        "{label}: kernel {kernel} independent {expected}"
    );
}

fn set(doc: &mut Value, pointer: &str, value: Option<Value>) {
    let parts: Vec<String> = pointer
        .split('/')
        .skip(1)
        .map(|part| part.replace("~1", "/").replace("~0", "~"))
        .collect();
    let (last, path) = parts.split_last().expect("pointer");
    let mut current = doc;
    for part in path {
        current = if current.is_array() {
            &mut current[part.parse::<usize>().expect("index")]
        } else {
            current
                .as_object_mut()
                .expect("object")
                .entry(part.clone())
                .or_insert_with(|| Value::Object(Default::default()))
        };
    }
    match (current, value) {
        (Value::Array(items), None) => {
            items.remove(last.parse::<usize>().expect("index"));
        }
        (Value::Array(items), Some(value)) => items[last.parse::<usize>().expect("index")] = value,
        (Value::Object(map), None) => {
            map.remove(last);
        }
        (Value::Object(map), Some(value)) => {
            map.insert(last.clone(), value);
        }
        _ => panic!("pointer {pointer}"),
    }
}

/// An ISSO 54 v2.0 deeltest in 2025+C1 (no `normVersion`, no 2023 emission block).
fn suite_case(case: &str) -> Value {
    let mut project: Value = serde_json::from_str(EPW001).expect("base");
    let suite: Value = serde_json::from_str(SUITE).expect("suite");
    let wanted = format!("isso54-v2-{case}");
    let entry = suite["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|item| item["caseId"] == wanted.as_str())
        .expect("case");
    for op in entry["projectPatch"].as_array().expect("patch") {
        let pointer = op["pointer"].as_str().expect("pointer");
        let value = (op["op"] != "remove").then(|| op["value"].clone());
        set(&mut project, pointer, value);
    }
    in_target_edition(project)
}

fn in_target_edition(mut project: Value) -> Value {
    let block = project["ntaCalculation"].as_object_mut().expect("block");
    block.remove("normVersion");
    if let Some(emission) = block.get_mut("emission").and_then(Value::as_object_mut) {
        emission.remove("edition2023");
    }
    project
}

fn parse(text: &str) -> Value {
    in_target_edition(serde_json::from_str(text).expect("json"))
}

/// The full assessment as JSON; asserts a calculated result without gaps.
fn run(project: &Value) -> Value {
    let result = assess_project_performance(project);
    assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
    assert!(result.gaps.is_empty(), "{:?}", result.gaps);
    serde_json::to_value(result).expect("serializable")
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or_else(|| panic!("number: {value}"))
}

fn service_used(p: &Value, service: &str, carrier: &str) -> f64 {
    p["performance"]["energyByService"]["annual"]
        .as_array()
        .expect("rows")
        .iter()
        .filter(|row| row["service"] == service && row["carrier"] == carrier)
        .map(|row| number(&row["usedKwh"]))
        .sum()
}

fn steam_humidifier(carrier: &str) -> Value {
    json!([{
        "zoneId": "z1",
        "humidification": {
            "humidifier": {"kind": "steam", "carrier": carrier},
            "rotaryWheel": false,
            "equipmentReference": "review 75"
        }
    }])
}

// ---------------------------------------------------------------------------------------------------
// Chapter 12 (p. 520-524)
// ---------------------------------------------------------------------------------------------------

const SUPPLY_FLOW: [f64; 12] = [
    504.76, 510.0, 498.0, 470.0, 430.0, 400.0, 380.0, 390.0, 420.0, 460.0, 490.0, 505.0,
];
const COOLING_NEED: [f64; 12] = [
    0.0, 0.0, 10.0, 40.0, 120.0, 300.0, 520.0, 480.0, 260.0, 60.0, 5.0, 0.0,
];
const HUMIDITY_FIELDS: [&str; 7] = [
    "humidificationNeedKwh",
    "heatingSystemLoadKwh",
    "steamElectricityKwh",
    "steamFuelKwh",
    "steamOilKwh",
    "recoverableKwh",
    "dehumidificationNeedKwh",
];

fn humidity(edits: Value, expected: &[(&str, [f64; 12])]) {
    let mut input = json!({
        "zoneId": "z1",
        "usableFloorAreaM2": 400.0,
        "functions": [{"function": "general", "areaM2": 400.0}],
        "supplyFlowM3PerH": SUPPLY_FLOW,
        "coolingNeedKwh": COOLING_NEED
    });
    for (key, value) in edits.as_object().expect("edits") {
        input[key] = value.clone();
    }
    let input: HumidityInput = serde_json::from_value(input).expect("humidity input");
    let months = calculate_humidity(&input).expect("humidity");
    let months = serde_json::to_value(months).expect("serializable");
    for field in HUMIDITY_FIELDS {
        let want = expected
            .iter()
            .find(|(name, _)| *name == field)
            .map_or([0.0; 12], |(_, values)| *values);
        for (index, value) in want.iter().enumerate() {
            let got = months[index][field].as_f64().unwrap_or(0.0);
            close(&format!("{field} month {}", index + 1), got, *value);
        }
    }
}

/// 12.1 (h_we 2 538,2, ρ_a 1,205, table 12.1 general), 12.3 (η 0,8), 12.4 ((1 − η)·Q, A_g ≤ 500 m²).
#[test]
fn humidification_steam_electric() {
    humidity(
        json!({"humidification": {"humidifier": {"kind": "steam", "carrier": "electricity"},
            "rotaryWheel": false, "equipmentReference": "x"}}),
        HUM_STEAM_EL,
    );
}

/// Table 12.1 healthcare, η_HU;rvd 0,55 (wheel), η 0,6 for gas; 12.5 with table 12.2 column 12/16 °C.
#[test]
fn humidification_gas_steam_with_wheel_healthcare() {
    humidity(
        json!({"functions": [{"function": "healthcare", "areaM2": 400.0}],
            "humidification": {"humidifier": {"kind": "steam", "carrier": "gas"},
            "rotaryWheel": true, "equipmentReference": "x"},
            "coolingDesign": "from12_to16"}),
        HUM_STEAM_GAS_WHEEL_HEALTHCARE,
    );
}

/// Two functions weighted by area, oil (η 0,6, carrier oil), A_g 800 m² > 500: no recoverable loss (12.2.1);
/// table 12.2 column 6/12 °C.
#[test]
fn humidification_oil_steam_large_mixed_zone() {
    humidity(
        json!({"usableFloorAreaM2": 800.0,
            "functions": [{"function": "general", "areaM2": 600.0}, {"function": "healthcare", "areaM2": 200.0}],
            "humidification": {"humidifier": {"kind": "steam", "carrier": "oil"},
            "rotaryWheel": false, "equipmentReference": "x"},
            "coolingDesign": "from6_to12"}),
        HUM_STEAM_OIL_BIG,
    );
}

/// 12.2.1 with the served installation area above 500 m²; direct expansion counts as 6/12 °C (12.3).
#[test]
fn humidification_large_installation_and_direct_expansion() {
    humidity(
        json!({"installationAreaM2": 1200.0,
            "humidification": {"humidifier": {"kind": "steam", "carrier": "electricity"},
            "rotaryWheel": false, "equipmentReference": "x"},
            "coolingDesign": "direct_expansion"}),
        HUM_STEAM_EL_INSTALLATION_AREA,
    );
}

/// Atomising: E_hum = 0, the latent heat goes to the heating system; residential part has 0 (table 12.1);
/// table 12.2 column 12/18 °C.
#[test]
fn humidification_atomising_with_residential_part() {
    humidity(
        json!({"functions": [{"function": "general", "areaM2": 300.0}, {"function": "residential", "areaM2": 100.0}],
            "humidification": {"humidifier": {"kind": "atomising"}, "rotaryWheel": true, "equipmentReference": "x"},
            "coolingDesign": "from12_to18"}),
        HUM_ATOMISING_MIXED_RES,
    );
}

/// No humidifier: Q_H;hum = 0; floor/wall/ceiling cooling (17/21 °C) does not dehumidify (12.3 note 1).
#[test]
fn dehumidification_surface_cooling_is_zero() {
    humidity(
        json!({"coolingDesign": "from17_to21"}),
        HUM_DEHUM_ONLY_17_21,
    );
}

/// Sport: no humidification (table 12.1 note 1); an unknown design temperature counts as 6/12 °C.
#[test]
fn humidification_sport_and_unknown_cooling_design() {
    humidity(
        json!({"functions": [{"function": "sport", "areaM2": 400.0}],
            "humidification": {"humidifier": {"kind": "steam", "carrier": "electricity"},
            "rotaryWheel": false, "equipmentReference": "x"},
            "coolingDesign": "unknown"}),
        HUM_DEHUM_UNKNOWN_SPORT,
    );
}

/// The example office with an electric steam humidifier: 12.1 with q_V;SUP;dis;out = 504,758 m³/h (the
/// mechanical supply of chapter 11, review 64), E_hum = Q/0,8 per month in the humidification row of 5.20
/// (not corrected with f_BACS), and Q_H;hum;rbl = 0,2·Q_H;hum (A_g 400 m²) in Q_H;ls;rbl of 7.3-7.5
/// (Δη by 7.8.2), which lowers Q_H;nd. E_H;nd of 5.3a leaves the recoverable losses out (5.3b-c, p. 75).
#[test]
fn office_with_steam_humidifier() {
    let mut office = parse(OFFICE);
    office["ntaCalculation"]["humidifiers"] = steam_humidifier("electricity");
    let p = run(&office);
    let monthly = p["performance"]["spaceHeating"]["monthly"]
        .as_array()
        .expect("monthly");
    let demand = p["performance"]["spaceHeating"]["demand"]["monthly"]
        .as_array()
        .expect("demand");
    for (index, (e_hum, need)) in OFFICE_STEAM_E_HUM.iter().zip(OFFICE_STEAM_NEED).enumerate() {
        close(
            &format!("E_hum month {}", index + 1),
            number(&monthly[index]["humidificationElectricityKwh"]),
            *e_hum,
        );
        close(
            &format!("Q_H;nd month {}", index + 1),
            number(&demand[index]["heating"]["needKwh"]),
            need,
        );
    }
    close(
        "humidification row",
        service_used(&p, "humidification", "el"),
        OFFICE_STEAM_E_HUM.iter().sum(),
    );
    close(
        "E_H;nd",
        number(&p["performance"]["chapter5"]["heatingNeedKwhPerM2"]),
        30.82,
    );
}

const OFFICE_STEAM_E_HUM: [f64; 12] = [
    439.5591972947368,
    251.94246674210524,
    343.0705930105263,
    21.441912063157893,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    0.0,
    316.2682029315789,
];
const OFFICE_STEAM_NEED: [f64; 12] = [
    2940.1634331410055,
    1979.0744988914332,
    1459.3642590832108,
    663.0422726349065,
    108.07227496441465,
    0.0,
    0.0,
    0.0,
    73.1059276699832,
    770.509690321227,
    1508.0956485076117,
    2652.834825842863,
];

// ---------------------------------------------------------------------------------------------------
// §5.5.8 BACS factor (p. 99-101) and its use in 5.20/5.21 (p. 89-91)
// ---------------------------------------------------------------------------------------------------

fn office_bacs(edit: impl FnOnce(&mut Value)) -> Value {
    let mut office = parse(OFFICE);
    office["ntaCalculation"]["humidifiers"] = steam_humidifier("electricity");
    edit(&mut office["ntaCalculation"]["bacs"]);
    office
}

fn bacs_factor(project: &Value) -> Value {
    serde_json::to_value(assess_project_performance(project).bacs).expect("bacs")["factor"].clone()
}

/// A utility heating system above 290 kW: 1,05 without BACS, with class D controls or with energy
/// management class C/D; 1,0 with controls ≥ C and management ≥ B; exactly 290 kW is not "meer dan";
/// an undeterminable capacity triggers the test; the 290 kW test is per system (two cooling generators of
/// 200 + 100 kW in one system).
#[test]
fn bacs_factor_rules() {
    let triggered = |edit: &dyn Fn(&mut Value)| {
        bacs_factor(&office_bacs(|bacs| {
            bacs["systems"][0]["generators"][0]["nominalThermalCapacityKw"] = json!(300.0);
            edit(bacs);
        }))
    };
    let evidence = |present: bool, controls: Option<&str>, management: Option<&str>| {
        let mut value = json!({"present": present, "sourceReference": "x"});
        if let Some(class) = controls {
            value["automaticControlsClass"] = json!(class);
        }
        if let Some(class) = management {
            value["energyManagementClass"] = json!(class);
        }
        value
    };
    assert_eq!(
        triggered(&|b| b["bacs"] = evidence(false, None, None)),
        json!(1.05)
    );
    assert_eq!(
        triggered(&|b| b["bacs"] = evidence(true, Some("C"), Some("B"))),
        json!(1.0)
    );
    assert_eq!(
        triggered(&|b| b["bacs"] = evidence(true, Some("B"), Some("C"))),
        json!(1.05)
    );
    assert_eq!(
        triggered(&|b| b["bacs"] = evidence(true, Some("D"), Some("A"))),
        json!(1.05)
    );
    let exact = office_bacs(|bacs| {
        bacs["systems"][0]["generators"]
            .as_array_mut()
            .expect("generators")
            .push(json!({"id": "b2", "nominalThermalCapacityKw": 230.0, "sourceReference": "x"}));
    });
    assert_eq!(bacs_factor(&exact), json!(1.0));
    let unknown = office_bacs(|bacs| {
        bacs["systems"][0]["generators"][0]["nominalThermalCapacityKw"] = Value::Null;
        bacs["bacs"] = evidence(true, Some("D"), Some("A"));
    });
    assert_eq!(bacs_factor(&unknown), json!(1.05));
    let cooling = office_bacs(|bacs| {
        bacs["systems"]
            .as_array_mut()
            .expect("systems")
            .push(json!({
                "id": "koeling", "service": "cooling", "sourceReference": "x",
                "generators": [
                    {"id": "c1", "nominalThermalCapacityKw": 200.0, "sourceReference": "x"},
                    {"id": "c2", "nominalThermalCapacityKw": 100.0, "sourceReference": "x"}
                ],
                "bacs": evidence(true, Some("D"), Some("A"))
            }));
        bacs["systems"][0]["bacs"] = evidence(true, Some("A"), Some("A"));
    });
    assert_eq!(bacs_factor(&cooling), json!(1.05));
}

/// 5.20a/b and 5.21: f_BACS multiplies E_H, E_C, W_H;aux and W_C;aux only; humidification, fans, lighting,
/// hot water and W_W;aux are not corrected. EPU001 with a circulation pump (W_W;aux 365,65 kWh).
#[test]
fn bacs_factor_scope_in_5_20_and_5_21() {
    let office = |factor: f64| {
        let mut project = parse(EPU001);
        project["ntaCalculation"]["bacsFactor"] = json!(factor);
        project["ntaCalculation"]["hotWater"]["circulation"] = json!({
            "insulation": "mm15", "fittingsInsulated": true, "floorCount": 1,
            "pump": {"control": "uncontrolled_or_unknown"}, "sourceReference": "review 75"
        });
        run(&project)
    };
    let (one, corrected) = (office(1.0), office(1.05));
    for (service, carrier, factor) in [
        ("heating", "gas", 1.05),
        ("cooling", "el", 1.05),
        ("ventilation", "el", 1.0),
        ("lighting", "el", 1.0),
        ("hotWater", "gas", 1.0),
    ] {
        close(
            &format!("{service} {carrier}"),
            service_used(&corrected, service, carrier),
            factor * service_used(&one, service, carrier),
        );
    }
    let sum = |p: &Value, part: &str, field: &str| -> f64 {
        p["performance"][part][field]
            .as_array()
            .expect("months")
            .iter()
            .map(|month| number(&month["auxiliaryElectricityKwh"]))
            .sum()
    };
    let w_aux = sum(&corrected, "hotWater", "months");
    close("W_W;aux", w_aux, 365.65177712552565);
    let aux_one = service_used(&one, "auxiliary", "el");
    // Only the part without W_W;aux (H and C auxiliary energy) takes 1,05.
    close(
        "auxiliary row",
        service_used(&corrected, "auxiliary", "el"),
        w_aux + 1.05 * (aux_one - w_aux),
    );
}

// ---------------------------------------------------------------------------------------------------
// Chapter 5 indicators (5.9-5.27, 5.57-5.60, tables 5.2-5.4, rounding of 5.3.1-5.3.5)
// ---------------------------------------------------------------------------------------------------

/// Expected E_PTot, EwePTot, E_PrenTot, EwePrenTot, RER, m_CO2, E_Final, EweFinal, E_EPdel;el, m³aeq
/// (5.19a) and the storage correction (5.14a).
struct ChapterFive {
    primary: f64,
    primary_indicator: f64,
    renewable: f64,
    renewable_indicator: f64,
    share: f64,
    co2: f64,
    final_energy: f64,
    final_indicator: f64,
    delivered_electricity: f64,
    gas_equivalent: f64,
    storage: f64,
}

fn chapter_five(p: &Value, want: &ChapterFive) {
    let perf = &p["performance"];
    let c5 = &perf["chapter5"];
    close(
        "E_PTot",
        number(&perf["annualPrimaryFossilKwh"]),
        want.primary,
    );
    close(
        "EwePTot",
        number(&perf["primaryFossilIndicatorKwhPerM2Year"]),
        want.primary_indicator,
    );
    close(
        "E_PrenTot",
        number(&perf["annualRenewablePrimaryKwh"]),
        want.renewable,
    );
    close(
        "EwePrenTot",
        number(&c5["renewableIndicatorKwhPerM2"]),
        want.renewable_indicator,
    );
    close("RER", number(&perf["renewableSharePercent"]), want.share);
    close("m_CO2", number(&perf["annualCo2Kg"]), want.co2);
    close(
        "E_Final",
        number(&perf["annualFinalEnergyKwh"]),
        want.final_energy,
    );
    close(
        "EweFinal",
        number(&c5["finalEnergyKwhPerM2"]),
        want.final_indicator,
    );
    close(
        "E_EPdel;el",
        number(&c5["deliveredElectricityKwh"]),
        want.delivered_electricity,
    );
    close(
        "E_EPdel;aeq",
        number(&c5["deliveredOtherM3Aeq"]),
        want.gas_equivalent,
    );
    close(
        "E_P;BAT,out;tot",
        number(&perf["annualStorageCorrectionKwh"]),
        want.storage,
    );
}

/// Office with an electric steam humidifier, f_BACS 1,0 and 1,05 (heating system 300 kW).
#[test]
fn indicators_office_steam_humidifier_and_bacs() {
    let mut office = parse(OFFICE);
    office["ntaCalculation"]["humidifiers"] = steam_humidifier("electricity");
    chapter_five(
        &run(&office),
        &ChapterFive {
            primary: 21106.273611150165,
            primary_indicator: 52.77,
            renewable: 19090.589568,
            renewable_indicator: 47.72,
            share: 47.4,
            co2: 4389.089154191455,
            final_energy: 32288.09512541886,
            final_indicator: 80.73,
            delivered_electricity: 6334.726320846423,
            gas_equivalent: 1506.0278084065233,
            storage: 0.0,
        },
    );
    let corrected = office_bacs(|bacs| {
        bacs["systems"][0]["generators"][0]["nominalThermalCapacityKw"] = json!(300.0);
        bacs["bacs"] = json!({"present": true, "automaticControlsClass": "B",
            "energyManagementClass": "C", "sourceReference": "x"});
    });
    chapter_five(
        &run(&corrected),
        &ChapterFive {
            primary: 21859.768938160087,
            primary_indicator: 54.65,
            renewable: 19090.589568,
            renewable_indicator: 47.72,
            share: 46.6,
            co2: 4552.759254141888,
            final_energy: 33036.05310103629,
            final_indicator: 82.6,
            delivered_electricity: 6345.477296227064,
            gas_equivalent: 1581.3291988268495,
            storage: 0.0,
        },
    );
}

/// EPU001 (utility, cooling, combi tap water) with a circulation pump, f_BACS 1,05: no production, so
/// E_P = Σ E_del·f_P;del, RER 0; m³aeq = E_gas·3,6/35,17 (5.19a).
#[test]
fn indicators_utility_without_production() {
    let mut project = parse(EPU001);
    project["ntaCalculation"]["bacsFactor"] = json!(1.05);
    project["ntaCalculation"]["hotWater"]["circulation"] = json!({
        "insulation": "mm15", "fittingsInsulated": true, "floorCount": 1,
        "pump": {"control": "uncontrolled_or_unknown"}, "sourceReference": "review 75"
    });
    chapter_five(
        &run(&project),
        &ChapterFive {
            primary: 20210.388497187254,
            primary_indicator: 210.53,
            renewable: 0.0,
            renewable_indicator: 0.0,
            share: 0.0,
            co2: 4079.456672008428,
            final_energy: 17156.675208823908,
            final_indicator: 178.72,
            delivered_electricity: 6786.029529696318,
            gas_equivalent: 1061.5389378691875,
            storage: 0.0,
        },
    );
}

/// EPW501a (PV) with building-bound storage: f_BAT;cor = 1 from 5 kWh electrical + thermal together
/// (5.14a, p. 85-87), E_P;BAT = MIN(E_pr;el;ren;tot; E_EPus;el)·0,05 per month, not in m_CO2 (5.5.6.1);
/// 4,99 kWh gives f_BAT;cor = 0; thermal storage alone counts as well.
#[test]
fn indicators_storage_correction() {
    let with = |electrical: f64, thermal: f64| {
        let mut project = suite_case("EPW501a");
        project["ntaCalculation"]["storage"] = json!({
            "buildingBoundElectricalKwh": electrical,
            "buildingBoundThermalKwh": thermal,
            "sourceReference": "review 75"
        });
        project["ntaCalculation"]["batteryStoragePresent"] = json!(true);
        run(&project)
    };
    let common = |primary: f64, primary_indicator: f64, share: f64, storage: f64| ChapterFive {
        primary,
        primary_indicator,
        renewable: 3400.5480030720005,
        renewable_indicator: 35.42,
        share,
        co2: 1546.6651679549295,
        final_energy: 9858.907911339082,
        final_indicator: 102.7,
        delivered_electricity: 4.032517774655304,
        gas_equivalent: 956.0565613555135,
        storage,
    };
    chapter_five(
        &with(3.0, 2.0),
        &common(6666.068113927735, 69.44, 33.7, 25.736696582757297),
    );
    chapter_five(
        &with(2.99, 2.0),
        &common(6691.804810510491, 69.71, 33.6, 0.0),
    );
    chapter_five(
        &with(0.0, 6.0),
        &common(6666.068113927735, 69.44, 33.7, 25.736696582757297),
    );
}

// ---------------------------------------------------------------------------------------------------
// 5.3.1.2 renovation standard (table 5.7, p. 73), 5.3.2 standard insulation (p. 75-76), 5.3.1 Bbl weighting
// ---------------------------------------------------------------------------------------------------

#[test]
fn renovation_standard_table_5_7() {
    let one = |function: LabelFunction| {
        renovation_standard(&[LabelFunctionArea {
            function,
            area_m2: 100.0,
        }])
        .expect("limit")
    };
    for (function, limit) in [
        (LabelFunction::AssemblyWithDayCare, 110.0),
        (LabelFunction::AssemblyWithoutDayCare, 100.0),
        (LabelFunction::Cell, 180.0),
        (LabelFunction::HealthcareWithBeds, 270.0),
        (LabelFunction::HealthcareWithoutBeds, 90.0),
        (LabelFunction::Office, 80.0),
        (LabelFunction::Lodging, 150.0),
        (LabelFunction::Education, 100.0),
        (LabelFunction::Sport, 105.0),
        (LabelFunction::Retail, 120.0),
    ] {
        close(&format!("{function:?}"), one(function), limit);
    }
    // Area-weighted, rounded half up to two decimals: 80,005 → 80,01.
    let mixed = renovation_standard(&[
        LabelFunctionArea {
            function: LabelFunction::Office,
            area_m2: 7999.0,
        },
        LabelFunctionArea {
            function: LabelFunction::Retail,
            area_m2: 1.0,
        },
    ])
    .expect("mixed");
    close("80,005", mixed, 80.01);
}

/// 5.3.2: four rows by dwelling type and construction year (≤ 1945), 1,0 as the switch, rounded to a whole
/// number (half up).
#[test]
fn standard_insulation_rows() {
    let cases = [
        (false, 1945, 90.0, 100.0, 60.0_f64),
        (false, 1945, 150.0, 100.0, 60.0 + 105.0 * 0.5),
        (false, 1946, 150.0, 100.0, 63.0),
        (false, 1990, 116.25, 100.0, 50.0),
        (true, 1930, 120.0, 100.0, 95.0 + 70.0 * 0.2),
        (true, 1930, 99.0, 100.0, 95.0),
        (true, 2000, 140.0, 100.0, 45.0 + 45.0 * 0.4),
        (true, 2000, 100.0, 100.0, 45.0),
    ];
    for (apartment, year, loss, usable, expected) in cases {
        let got = standard_insulation(apartment, year, loss, usable).expect("standard");
        close(&format!("{apartment} {year} {loss}"), got, expected.round());
    }
}

/// 5.3.1 (p. 71-72): a utility building with several functions is tested against the limits weighted by
/// A_g, each with the A_ls/A_g of the whole building.
#[test]
fn bbl_limits_weighted_by_area() {
    let parts = [
        (BblFunction::Office, 1000.0),
        (BblFunction::OtherAssembly, 500.0),
        (BblFunction::Education, 250.0),
    ];
    for ratio in [1.2, 1.8, 2.6] {
        let mixed = bbl_limits_mixed(
            &parts
                .iter()
                .map(|(function, area_m2)| BblFunctionArea {
                    function: *function,
                    area_m2: *area_m2,
                })
                .collect::<Vec<_>>(),
            ratio,
            300.0,
        )
        .expect("mixed");
        let total: f64 = parts.iter().map(|(_, area)| area).sum();
        let weighted = |field: fn(&nta8800_core::bbl_requirements::BblLimits) -> f64| -> f64 {
            parts
                .iter()
                .map(|(function, area)| {
                    field(&bbl_limits(*function, ratio, 300.0).expect("one")) * area
                })
                .sum::<f64>()
                / total
        };
        close(
            "BENG 1 limit",
            mixed.energy_need_max_kwh_per_m2,
            weighted(|l| l.energy_need_max_kwh_per_m2),
        );
        close(
            "BENG 2 limit",
            mixed.primary_fossil_max_kwh_per_m2,
            weighted(|l| l.primary_fossil_max_kwh_per_m2),
        );
        close(
            "BENG 3 limit",
            mixed.renewable_share_min_percent,
            weighted(|l| l.renewable_share_min_percent),
        );
    }
}

// ---------------------------------------------------------------------------------------------------
// Chapter 6: 6.6.4 common spaces (p. 158), 6.7.3 f_ls (p. 162), 6.4/6.5.2 zoning (p. 152-157)
// ---------------------------------------------------------------------------------------------------

fn case_h(edit: impl FnOnce(&mut Value)) -> Value {
    let mut project = parse(CASE_H);
    let nta = &mut project["ntaCalculation"];
    nta.as_object_mut().expect("block").remove("bblFunctions");
    nta["zoneData"][0]["functionAreas"] = json!([
        {"function": "office", "areaM2": 14.70},
        {"function": "other_assembly", "areaM2": 26.30},
        {"function": "other_healthcare", "areaM2": 27.60}
    ]);
    nta["zoneData"][0]["ventilation"]["functions"] = nta["zoneData"][0]["functionAreas"].clone();
    edit(nta);
    project
}

fn common_spaces(
    p: &Value,
    building: &[(&str, f64)],
    zones: &[(&str, &[(&str, f64)])],
    renovation: f64,
) {
    let split = &p["commonSpaces"];
    for (function, area) in building {
        let got = split["building"]
            .as_array()
            .expect("building")
            .iter()
            .find(|part| part["function"] == *function)
            .expect("function");
        close(
            &format!("building {function}"),
            number(&got["areaM2"]),
            *area,
        );
    }
    for (zone, parts) in zones {
        let allocated = &split["zones"]
            .as_array()
            .expect("zones")
            .iter()
            .find(|item| item["zoneId"] == *zone)
            .expect("zone")["allocated"];
        for (function, area) in parts.iter() {
            let got = allocated
                .as_array()
                .expect("allocated")
                .iter()
                .find(|part| part["function"] == *function)
                .expect("function");
            close(&format!("{zone} {function}"), number(&got["areaM2"]), *area);
        }
    }
    close(
        "renovation standard",
        number(&p["performance"]["chapter5"]["renovationStandardKwhPerM2"]),
        renovation,
    );
}

/// One common space for all functions: building split over all four, within RZ-1 over its three.
#[test]
fn common_space_for_all_functions() {
    let p = run(&case_h(|nta| {
        nta["zoneData"][0]["commonSpaces"] =
            json!([{"id": "g", "areaM2": 49.20, "sourceReference": "x"}]);
    }));
    common_spaces(
        &p,
        &[
            ("office", 15.741232363950473),
            ("other_assembly", 28.162885113734525),
            ("other_healthcare", 29.55496688741722),
            ("sport", 670.3409156348978),
        ],
        &[(
            "rz1",
            &[
                ("office", 25.242857142857144),
                ("other_assembly", 45.16239067055394),
                ("other_healthcare", 47.394752186588924),
            ],
        )],
        103.69,
    );
}

/// Served by office and sport: in the building pro rata over both, within RZ-1 all to office.
#[test]
fn common_space_for_functions_outside_the_zone() {
    let p = run(&case_h(|nta| {
        nta["zoneData"][0]["commonSpaces"] = json!([{"id": "g", "areaM2": 49.20,
            "servedFunctions": ["office", "sport"], "sourceReference": "x"}]);
        nta["zoneData"][0]["usageFunction"] = json!("office");
    }));
    common_spaces(
        &p,
        &[
            ("office", 15.828827844545028),
            ("other_assembly", 26.3),
            ("other_healthcare", 27.6),
            ("sport", 674.071172155455),
        ],
        &[(
            "rz1",
            &[
                ("office", 63.900000000000006),
                ("other_assembly", 26.3),
                ("other_healthcare", 27.6),
            ],
        )],
        103.73,
    );
}

/// Three common spaces in two zones, each serving functions partly outside its zone.
#[test]
fn common_spaces_in_two_zones() {
    let p = run(&case_h(|nta| {
        nta["zoneData"][0]["commonSpaces"] = json!([
            {"id": "g1", "areaM2": 30.0, "servedFunctions": ["office", "other_assembly"], "sourceReference": "x"},
            {"id": "g2", "areaM2": 19.2, "servedFunctions": ["other_healthcare", "sport"], "sourceReference": "x"}
        ]);
        nta["zoneData"][1]["functionAreas"] = json!([{"function": "sport", "areaM2": 600.0}]);
        nta["zoneData"][1]["commonSpaces"] = json!([
            {"id": "g3", "areaM2": 26.0, "servedFunctions": ["sport", "office"], "sourceReference": "x"}
        ]);
    }));
    common_spaces(
        &p,
        &[
            ("office", 26.07786427644657),
            ("other_assembly", 45.54390243902439),
            ("other_healthcare", 28.44435946462715),
            ("sport", 643.7338738199019),
        ],
        &[
            (
                "rz1",
                &[
                    ("office", 25.456097560975607),
                    ("other_assembly", 45.54390243902439),
                    ("other_healthcare", 46.8),
                ],
            ),
            ("rz2", &[("sport", 626.0)]),
        ],
        103.24,
    );
}

/// 6.7.3: f_ls 1 for outside air and an unheated space, 0,7 for the ground, 0 for an adjacent heated space
/// and an internal partition; the Bbl BENG 1 limit and the standard insulation use A_ls/A_g.
#[test]
fn loss_area_weighting() {
    let mut project = parse(DWELLING);
    let surfaces = project["zones"][0]["surfaces"]
        .as_array_mut()
        .expect("surfaces");
    for (id, boundary, area, kind) in [
        ("garage-wall", "unheated_space", 10.0, "wall"),
        ("party", "adjacent_conditioned", 45.0, "wall"),
        ("inner", "internal", 12.0, "floor"),
    ] {
        let mut surface = json!({"id": id, "name": id, "type": kind, "thermalBoundary": boundary,
            "area": area, "orientation": "E", "constructionId": "c-wall", "zoneId": "z1", "windows": []});
        if boundary == "unheated_space" {
            surface["unheatedSpaceId"] = json!("garage");
        }
        surfaces.push(surface);
    }
    project["unheatedSpaces"] = json!([{"id": "garage", "name": "Garage", "reductionFactor": 0.6,
        "factorSourceReference": "8.4.1"}]);
    let p = run(&project);
    // 30 + 30 + 35,36 + 35,36 + 10 (f_ls 1) + 0,7 × 50.
    let loss = 30.0 + 30.0 + 35.36 + 35.36 + 10.0 + 0.7 * 50.0;
    close("A_ls", number(&p["geometry"]["lossAreaM2"]), loss);
    close(
        "standard insulation",
        number(&p["performance"]["chapter5"]["standardInsulationKwhPerM2"]),
        (43.0 + 40.0 * (loss / 100.0 - 1.0)).round(),
    );
    close(
        "A_ls/A_g",
        number(&p["performance"]["bblCheck"]["lossAreaRatio"]),
        loss / 100.0,
    );
}

fn zone(parts: Value, ventilation: Value) -> Vec<&'static str> {
    let layout: CalculationZoneLayout = serde_json::from_value(json!({
        "id": "z", "parts": parts, "heatingSystemIds": ["h"], "ventilation": ventilation
    }))
    .expect("layout");
    check_zone(&layout)
        .into_iter()
        .map(|issue| issue.code)
        .collect()
}

/// 6.5.2 a: setpoints (table 7.13, p. 217) differing more than 4 K need the largest function at ≥ 90 %;
/// 6.5.2 c: D_m;int;eff up to a factor 3, or more than 80 % alike; 6.4 d: at least 80 % one ventilation
/// variant.
#[test]
fn zoning_boundaries() {
    let balanced = json!([{"op": "balanced", "areaM2": 100.0}]);
    let parts = |beds: f64, sport: f64, c1: f64, c2: f64| {
        json!([
            {"function": "healthcare_with_beds", "areaM2": beds, "heatCapacityKjPerM2k": c1},
            {"function": "sport", "areaM2": sport, "heatCapacityKjPerM2k": c2}
        ])
    };
    assert!(!zone(parts(90.0, 10.0, 180.0, 180.0), balanced.clone())
        .contains(&"zone_setpoint_spread_exceeds_4_k"));
    assert!(zone(parts(89.0, 11.0, 180.0, 180.0), balanced.clone())
        .contains(&"zone_setpoint_spread_exceeds_4_k"));
    // Factor 3 exactly is allowed; above it only with more than 80 % alike.
    assert!(!zone(parts(90.0, 10.0, 360.0, 120.0), balanced.clone())
        .contains(&"zone_heat_capacity_spread_exceeds_3"));
    assert!(zone(parts(80.0, 20.0, 370.0, 120.0), balanced.clone())
        .contains(&"zone_heat_capacity_spread_exceeds_3"));
    assert!(!zone(parts(81.0, 19.0, 370.0, 120.0), balanced.clone())
        .contains(&"zone_heat_capacity_spread_exceeds_3"));
    let split = |balanced: f64, natural: f64| json!([{"op": "balanced", "areaM2": balanced}, {"op": "natural", "areaM2": natural}]);
    assert!(!zone(parts(90.0, 10.0, 180.0, 180.0), split(80.0, 20.0))
        .contains(&"zone_ventilation_not_80_percent_uniform"));
    assert!(zone(parts(90.0, 10.0, 180.0, 180.0), split(79.0, 21.0))
        .contains(&"zone_ventilation_not_80_percent_uniform"));
}

// ---------------------------------------------------------------------------------------------------
// §5.7.1 method 3 (p. 114-115)
// ---------------------------------------------------------------------------------------------------

fn small_window_area(window_m2: f64) -> assess::Outcome {
    let mut project = suite_case("EPW301a");
    project["ntaCalculation"]["activeCooling"]["capacity"] = json!({"method": "solar_limitation", "criterion": "small_window_area", "sourceReference": "x"});
    for window in project["zones"][0]["surfaces"][2]["windows"]
        .as_array_mut()
        .expect("windows")
    {
        window["area"] = json!(window_m2);
    }
    assess::outcome(&project)
}

mod assess {
    use super::*;

    pub struct Outcome {
        pub not_met: bool,
        pub tojuli: Option<f64>,
    }

    pub fn outcome(project: &Value) -> Outcome {
        // §5.7.1 (p. 115): a criterion that is not met is reported, and TOjuli
        // is then calculated ("alle andere systemen en situaties").
        let result = assess_project_performance(project);
        let p = serde_json::to_value(result).expect("serializable");
        let not_met = p.to_string().contains("solar_limitation_not_met");
        Outcome {
            not_met,
            tojuli: p["performance"]["tojuliMaxK"].as_f64(),
        }
    }
}

/// A_w < 0,2·A_g;tot (A_g 96 m²): 4 × 4,79 m² = 19,16 m² exempts (TOjuli 0), 4 × 4,81 m² does not.
#[test]
fn tojuli_small_window_area() {
    let small = small_window_area(4.79);
    assert!(!small.not_met);
    assert_eq!(small.tojuli, Some(0.0));
    let large = small_window_area(4.81);
    assert!(large.not_met);
    assert!(
        large.tojuli.is_some_and(|value| value > 0.0),
        "{:?}",
        large.tojuli
    );
}

/// "kleiner is dan 0,2 × A_g;tot": A_w = 19,20 m² at A_g = 96 m² is not smaller, so the exemption does not
/// apply. The kernel compares 19,2 with 0,2 × 96 = 19,200000000000003 in floating point and accepts it.
#[test]
fn tojuli_small_window_area_at_the_boundary() {
    let boundary = small_window_area(4.8);
    assert!(boundary.not_met);
    assert!(
        boundary.tojuli.is_some_and(|value| value > 0.0),
        "{:?}",
        boundary.tojuli
    );
}

// ---------------------------------------------------------------------------------------------------
// Annex O (p. 923-931) with 9.85-9.90 (p. 359-361)
// ---------------------------------------------------------------------------------------------------

const CARRIER_INPUT: [f64; 12] = [
    2400.0, 2000.0, 1700.0, 900.0, 300.0, 60.0, 0.0, 0.0, 120.0, 800.0, 1500.0, 2200.0,
];

fn fan_points() -> Value {
    json!([
        {"modulation": 0.26, "powerW": 8.2}, {"modulation": 0.28, "powerW": 8.6},
        {"modulation": 0.42, "powerW": 12.7}, {"modulation": 0.55, "powerW": 17.3},
        {"modulation": 0.68, "powerW": 26.0}, {"modulation": 0.80, "powerW": 38.3}
    ])
}

fn table_o2_curve() -> Value {
    json!([
        {"timeS": 60.0, "meanLoad": 0.22, "meanPumpModulation": 0.71},
        {"timeS": 120.0, "meanLoad": 0.25, "meanPumpModulation": 0.84},
        {"timeS": 180.0, "meanLoad": 0.28, "meanPumpModulation": 0.90},
        {"timeS": 240.0, "meanLoad": 0.35, "meanPumpModulation": 0.92},
        {"timeS": 300.0, "meanLoad": 0.45, "meanPumpModulation": 0.94}
    ])
}

fn annex_o(measurements: Value, t: f64, mb: f64, a: f64, b: f64, months: [f64; 12]) {
    let m: AppliancePowerMeasurements = serde_json::from_value(measurements).expect("measurements");
    let constants = auxiliary_constants(&m, "m").expect("constants");
    close("t_a;gem", constants.mean_on_time_s, t);
    close("m_b", constants.mean_modulation, mb);
    close("A", constants.a_kwh, a);
    close("B", constants.b_kw, b);
    close("C", constants.c, mb);
    for (index, (input, want)) in CARRIER_INPUT.iter().zip(months).enumerate() {
        close(
            &format!("W_H;gen;aux month {}", index + 1),
            monthly_auxiliary_kwh(&constants, *input),
            want,
        );
    }
}

/// The first running mean already gives ≥ 120 s: t_a;gem = 120/0,9 (O.1.5); modulating fan by the
/// least-squares line O.2 at m_b; no pump; 9.85 with N = 1.
#[test]
fn annex_o_first_point_and_fan_line() {
    annex_o(
        json!({"nominalLoadKw": 30.0, "standbyElectronicsW": 1.5, "gasValveW": 3.0,
            "fan": {"method": "modulating", "points": fan_points()}, "pump": {"method": "none"},
            "loadCurve": [{"timeS": 150.0, "meanLoad": 0.9}, {"timeS": 300.0, "meanLoad": 0.95}],
            "sourceReference": "x"}),
        133.33333333333334,
        0.9,
        13.14,
        0.04241253072978858,
        [
            4.865002731536763,
            4.236668942947302,
            3.765418601505207,
            2.508751024326286,
            1.5662503414420954,
            1.189250068288419,
            1.095,
            1.095,
            1.2835001365768381,
            2.3516675771789206,
            3.451251707210477,
            4.550835837242032,
        ],
    );
}

/// Bivalent appliance (β < 1): 1 800 s (O.1.5); staged pump with post-run (9.87, f_nuttig;p 0,5 over
/// f_P;del;el 1,45).
#[test]
fn annex_o_bivalent_with_staged_pump() {
    annex_o(
        json!({"nominalLoadKw": 6.0, "standbyElectronicsW": 4.0, "gasValveW": 0.0,
            "fan": {"method": "modulating", "points": fan_points()},
            "pump": {"method": "staged", "operationW": 25.0, "prePostRunW": 25.0},
            "pumpPostRunS": 60.0, "bivalent": true, "bivalentMeanLoad": 0.55, "sourceReference": "x"}),
        1800.0,
        0.55,
        35.04,
        0.038129803757249,
        [
            30.650766368908364,
            26.028971974090304,
            22.562626177976753,
            13.319037388340636,
            6.386345796113545,
            3.613269159222709,
            2.92,
            2.92,
            4.306538318445418,
            12.163588789636123,
            20.251728980567727,
            28.339869171499338,
        ],
    );
}

/// O.1.5 (p. 924) and O.2.3.4 (p. 930): the mean load at t_a;gem times t_a;gem equals 120 s. With the
/// load curve of table O.2 the kernel interpolates the product to t = 282,35 s and then the mean load
/// linearly at that t (0,4206), so m_b·t = 118,75 s. Either m_b = 120/t (282,35 s; 0,4250) or a mean load
/// linear in time (283,75 s; 0,42291) keeps the rule; the text does not say how to interpolate.
#[test]
fn annex_o_on_time_keeps_the_120_s_product() {
    let m: AppliancePowerMeasurements = serde_json::from_value(json!({
        "nominalLoadKw": 24.5, "standbyElectronicsW": 2.1, "gasValveW": 4.3,
        "fan": {"method": "modulating", "points": fan_points()},
        "pump": {"method": "modulating", "points": [
            {"modulation": 0.3, "powerW": 12.0}, {"modulation": 0.5, "powerW": 22.0},
            {"modulation": 0.7, "powerW": 31.0}, {"modulation": 0.9, "powerW": 43.0},
            {"modulation": 1.0, "powerW": 47.0}], "prePostRunModulation": 0.6},
        "pumpPostRunS": 120.0, "fanPreRunS": 5.0, "fanPostRunS": 20.0,
        "loadCurve": table_o2_curve(), "sourceReference": "x"
    }))
    .expect("measurements");
    let constants = auxiliary_constants(&m, "m").expect("constants");
    close(
        "m_b·t_a;gem",
        constants.mean_modulation * constants.mean_on_time_s,
        120.0,
    );
}

/// A of 9.86 does not depend on the load curve.
#[test]
fn annex_o_standby_constant() {
    let m: AppliancePowerMeasurements = serde_json::from_value(json!({
        "nominalLoadKw": 24.5, "standbyElectronicsW": 3.4, "gasValveW": 5.0,
        "fan": {"method": "modulating", "points": fan_points()},
        "pump": {"method": "staged", "operationW": 45.0, "prePostRunW": 30.0},
        "pumpPreRunS": 10.0, "pumpPostRunS": 180.0, "fanPostRunS": 10.0,
        "loadCurve": table_o2_curve(), "sourceReference": "x"
    }))
    .expect("measurements");
    let constants = auxiliary_constants(&m, "m").expect("constants");
    close("A", constants.a_kwh, 3.4 * 8760.0 / 1000.0);
}

// ---------------------------------------------------------------------------------------------------
// Annex M (p. 870-879)
// ---------------------------------------------------------------------------------------------------

const M_OUTPUT: [f64; 12] = [
    2900.0, 2300.0, 1900.0, 900.0, 250.0, 40.0, 0.0, 0.0, 120.0, 800.0, 1600.0, 2600.0,
];
const M_HOURS_ON: [f64; 12] = [
    380.0, 330.0, 300.0, 200.0, 90.0, 20.0, 0.0, 0.0, 50.0, 180.0, 280.0, 360.0,
];
const M_RETURN: [f64; 12] = [
    38.0, 36.0, 35.0, 31.0, 27.0, 25.0, 20.0, 20.0, 26.0, 31.0, 35.0, 37.0,
];
const M_OUTDOOR: [f64; 12] = [
    2.61, 4.82, 5.91, 9.32, 14.73, 16.12, 18.05, 18.48, 15.63, 10.4, 7.99, 4.0,
];
const MONTH_HOURS: [f64; 12] = [
    744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0,
];

fn condensing_gas() -> Value {
    json!({
        "technology": "condensing_gas", "fuel": "natural_gas", "placement": "installation_room",
        "draught": "fan_assisted", "control": "wall_hung_room_temperature", "equipmentReference": "x",
        "product": {"nominalPowerKw": 24.0, "intermediatePowerKw": 7.2,
            "fullLoad": {"method": "condensing", "efficiencyAt60": 0.88, "efficiencyAt30": 0.97},
            "partLoadEfficiency": 1.07, "standbyLossFactor": 0.008, "standbyTestTemperatureC": 50.0,
            "auxiliaryStandbyW": 3.0, "auxiliaryIntermediateW": 15.0, "auxiliaryFullW": 40.0,
            "sourceReference": "x"}
    })
}

fn annex_m(boiler: Value, ambient: Option<[f64; 12]>, expected: &[(&str, [f64; 12])]) {
    let boiler: ProductBoiler = serde_json::from_value(boiler).expect("boiler");
    for index in 0..12 {
        let result = boiler_month(
            &boiler,
            BoilerMonth {
                heat_output_kwh: M_OUTPUT[index],
                operating_hours: M_HOURS_ON[index],
                month_hours: MONTH_HOURS[index],
                return_temperature_c: M_RETURN[index],
                outdoor_temperature_c: M_OUTDOOR[index],
                ambient_temperature_c: ambient.map(|values| values[index]),
            },
        );
        let result = serde_json::to_value(result).expect("serializable");
        for (field, values) in expected {
            close(
                &format!("{field} month {}", index + 1),
                number(&result[*field]),
                values[index],
            );
        }
    }
}

/// Condensing gas boiler: M.8, M.10 (30 °C, 0,20 %/°C), M.9/M.11 (f_Hs/Hi 1,11, f_prac 0,95), M.12, M.4/M.5,
/// M.6, M.21-M.23, M.18/M.19 (f_brm 0,3), M.16 (f_gen;env 0,75), M.24 (f_ctr;ls 1,06), M.1.
#[test]
fn annex_m_condensing_gas() {
    annex_m(condensing_gas(), None, M_COND_GAS);
}

/// Standard gas boiler, atmospheric, in the heated space (f_brm 0, ϑ_brm 20), P_int = 0,3·P_n (M.29),
/// floor-standing outdoor-compensated (f_ctr;ls 1,00). In July and August (no output) the stand-by loss is 0
/// at ϑ_Hc;mn = ϑ_brm and literal M.1 would give −0,75·W; the kernel keeps E_in at 0.
#[test]
fn annex_m_standard_gas_heated_space() {
    let mut boiler = condensing_gas();
    boiler["technology"] = json!("gas_oil_standard");
    boiler["draught"] = json!("atmospheric");
    boiler["placement"] = json!("heated_space");
    boiler["control"] = json!("floor_standing_outdoor_compensated");
    boiler["product"]["fullLoad"] = json!({"method": "single", "efficiency": 0.86});
    boiler["product"]["partLoadEfficiency"] = json!(0.84);
    boiler["product"]
        .as_object_mut()
        .expect("product")
        .remove("intermediatePowerKw");
    annex_m(boiler, None, M_STD_GAS);
}

/// Low-temperature oil boiler outdoors with additional tests (M.13/M.14) and own test temperatures, ϑ_brm
/// given (8 and 11 °C), f_brm 1 (no recoverable loss), f_Hs/Hi 1,06.
#[test]
fn annex_m_low_temperature_oil_with_additional_tests() {
    let mut boiler = condensing_gas();
    boiler["technology"] = json!("low_temperature");
    boiler["fuel"] = json!("oil");
    boiler["placement"] = json!("outdoors");
    boiler["control"] = json!("wall_hung_outdoor_compensated");
    boiler["product"]["fullLoad"] = json!({"method": "single", "efficiency": 0.9, "testTemperatureC": 65.0,
        "additionalTest": {"efficiency": 0.92, "testTemperatureC": 45.0}});
    boiler["product"]["partLoadEfficiency"] = json!(0.91);
    boiler["product"]["partLoadTestTemperatureC"] = json!(38.0);
    boiler["product"]["partLoadAdditionalTest"] =
        json!({"efficiency": 0.89, "testTemperatureC": 55.0});
    let ambient = [
        8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 11.0, 11.0, 11.0, 11.0, 11.0, 11.0,
    ];
    annex_m(boiler, Some(ambient), M_LT_OIL_ADD);
}

/// Solid-fuel (wood) boiler under the roof (f_brm 0,2, ϑ_brm 5), table M.2/M.4 solid-fuel rows, f_Hs/Hi 1,08.
#[test]
fn annex_m_wood_under_the_roof() {
    let mut boiler = condensing_gas();
    boiler["technology"] = json!("solid_fuel_standard");
    boiler["fuel"] = json!("wood");
    boiler["placement"] = json!("under_roof");
    boiler["product"]["fullLoad"] = json!({"method": "single", "efficiency": 0.8});
    boiler["product"]["partLoadEfficiency"] = json!(0.78);
    annex_m(boiler, None, M_WOOD);
}

/// Condensing oil boiler: table M.4 0,10 %/°C at 30 °C, f_Hs/Hi 1,06.
#[test]
fn annex_m_condensing_oil() {
    let mut boiler = condensing_gas();
    boiler["technology"] = json!("condensing_oil");
    boiler["fuel"] = json!("oil");
    annex_m(boiler, None, M_COND_OIL);
}

// ---------------------------------------------------------------------------------------------------
// Annex P calculated route (P.7-P.12, P.19-P.25, P.6.4 with table P.0, P.6.5.4, tables P.2/P.3/P.5) with
// §5.8.3 (5.42-5.49), and the collective heat-pump source of 9.6.8.1.1.2.3 in chapter 5
// ---------------------------------------------------------------------------------------------------

const ANNEX_P: &str = include_str!("../../../training-data/nta8800-annex-p-synthetic.json");

struct AnnexPExpected {
    distribution: f64,
    generation: f64,
    primary: f64,
    co2: f64,
    renewable: f64,
    beta: Option<f64>,
    /// id, F_XD;gen;gi, f_XD;gen;gi, K_CO2;gen;gi, f_Pren;XD;gi, Q_XD;gen;gi.
    generators: &'static [(&'static str, f64, f64, f64, f64, f64)],
}

fn annex_p(route: Value, function: SystemFunction, want: &AnnexPExpected) {
    let route: AnnexPRoute = serde_json::from_value(route).expect("route");
    let result = assess_route(&route, function, "route").expect("annex P");
    let result = serde_json::to_value(result).expect("serializable");
    close(
        "η_XD;dis",
        number(&result["distributionEfficiency"]),
        want.distribution,
    );
    close(
        "f_XD;gen;tot",
        number(&result["generationPrimaryFactor"]),
        want.generation,
    );
    close(
        "f_P;XD;tot",
        number(&result["factors"]["primaryFactor"]),
        want.primary,
    );
    close(
        "K_CO2;XD;tot",
        number(&result["factors"]["co2KgPerKwh"]),
        want.co2,
    );
    close(
        "f_Pren",
        number(&result["factors"]["renewableFactor"]),
        want.renewable,
    );
    if let Some(beta) = want.beta {
        close("β_HD;gen", number(&result["calculation"]["beta"]), beta);
    }
    let generators = result["generators"].as_array().expect("generators");
    assert_eq!(generators.len(), want.generators.len());
    for (got, (id, fraction, primary, co2, renewable, heat)) in
        generators.iter().zip(want.generators)
    {
        assert_eq!(got["id"], *id);
        close(
            &format!("{id} F"),
            number(&got["energyFraction"]),
            *fraction,
        );
        close(&format!("{id} f"), number(&got["primaryFactor"]), *primary);
        close(&format!("{id} K"), number(&got["co2KgPerKwh"]), *co2);
        close(
            &format!("{id} f_Pren"),
            number(&got["renewableFactor"]),
            *renewable,
        );
        close(&format!("{id} Q"), number(&got["heatKwh"]), *heat);
    }
}

fn fixture(part: &str) -> Value {
    let all: Value = serde_json::from_str(ANNEX_P).expect("fixture");
    all[part].clone()
}

/// The synthetic heat network: aquifer heat pump (table P.5 groundwater < 15 °C at 70 °C: 2,6; electricity
/// with a 20 % direct renewable share, P.4.5), geothermal (P.6.5.4.8: 20·(75 − 40 − 3)/40 = 16), gas peak
/// boiler; η_dis from P.12/P.10; f_P rounded up (P.7), f_Pren rounded down (5.42) with E_prim from the
/// rounded f_P.
#[test]
fn annex_p_heat_network() {
    annex_p(fixture("heating"), SystemFunction::Heating, &P_H1);
}

/// Cold: declared EER 20 ≥ 8 gives f_Pren;CD;gi = 1 (5.49), no rounding of f_Pren;dc stated.
#[test]
fn annex_p_cold_network() {
    annex_p(fixture("cooling"), SystemFunction::Cooling, &P_C1);
}

/// Geothermal alone. 5.48 prints 1 − 1/20 = 0,95 with "η = 20 conform P.6.5.4.8"; the kernel takes
/// 1 − 1/η with the forfait η of P.6.5.4.8 (16 here), giving f_Pren 0,88 instead of 0,89 (NEN question).
#[test]
fn annex_p_geothermal_only() {
    let mut route = fixture("heating");
    route["generators"] = json!([{"id": "geo", "energyFraction": 1.0,
        "kind": {"kind": "geothermal", "sourceTemperatureC": 75.0, "returnTemperatureC": 40.0}}]);
    annex_p(route, SystemFunction::Heating, &P_H2);
}

/// Residual heat (P.6.5.4.7: f_rw;aux;spec 0,07, f_P;del;rw = 0,07·1,45; 5.47: 1 − 0,07) with a gas boiler;
/// Q_in given (P.10 case a); 40 % renewable auxiliary electricity (P.4.5 weighting, 5.42).
#[test]
fn annex_p_residual_heat() {
    let mut route = fixture("heating");
    route["deliveredKwh"] = json!(5.0e6);
    route["distribution"] = json!({"method": "flows", "inputKwh": 5.5e6, "sourceReference": "x"});
    route["auxiliaryElectricityKwh"] = json!(50000.0);
    route["auxiliaryRenewableShare"] = json!(0.4);
    route["generators"] = json!([
        {"id": "rw", "energyFraction": 0.7, "kind": {"kind": "residual_heat"}},
        {"id": "b", "energyFraction": 0.3, "kind": {"kind": "combustion", "carrier": {"kind": "natural_gas"},
            "efficiency": 0.95, "efficiencyReference": "x"}}
    ]);
    annex_p(route, SystemFunction::Heating, &P_H3);
}

/// Gas heat pump on surface water at 52 °C (table P.5 GWP: 1,7) and an HR-107 boiler (table P.3 with
/// table P.4: θ_em;avg 45 °C, direct system: LT, 0,925).
#[test]
fn annex_p_gas_heat_pump_and_table_p3_boiler() {
    let mut route = fixture("heating");
    route["deliveredKwh"] = json!(8.0e6);
    route["distribution"] = json!({"method": "flows", "lossKwh": 1.0e6, "sourceReference": "x"});
    route["auxiliaryElectricityKwh"] = json!(100000.0);
    route["generators"] = json!([
        {"id": "gwp", "energyFraction": 0.8, "kind": {"kind": "heat_pump",
            "efficiency": {"method": "table_p5", "source": "gas_surface_water", "supplyTemperatureC": 52.0},
            "drive": {"kind": "natural_gas"}}},
        {"id": "hr", "energyFraction": 0.2, "kind": {"kind": "boiler", "carrier": {"kind": "natural_gas"},
            "efficiency": {"method": "table_p3", "boiler": "hr107",
                "emission": {"averageDesignTemperatureC": 45.0, "system": "direct"}}}}
    ]);
    annex_p(route, SystemFunction::Heating, &P_H4);
}

/// Energy fractions derived: P_ref = Q_in·3,6/5 400 (P.25), β = 800/3 960 (P.24), table P.2 interpolated,
/// the rest over the boilers by power (P.23); distribution loss 400 × 2 350 kWh (table P.0, 70/40,
/// ground-bound).
#[test]
fn annex_p_derived_fractions_and_small_system_loss() {
    let mut route = fixture("heating");
    route["deliveredKwh"] = json!(5.0e6);
    route["distribution"] = json!({"method": "small_system_forfait", "connections": 400,
        "connectionType": "ground_bound", "designTemperature": "t70_to40"});
    route["auxiliaryElectricityKwh"] = json!(60000.0);
    route["generators"] = json!([
        {"id": "hp", "nominalPowerKw": 800.0, "kind": {"kind": "heat_pump",
            "efficiency": {"method": "table_p5", "source": "electric_outdoor_air", "supplyTemperatureC": 55.0},
            "drive": {"kind": "electricity"}}},
        {"id": "b1", "nominalPowerKw": 1200.0, "kind": {"kind": "combustion", "carrier": {"kind": "natural_gas"},
            "efficiency": 0.95, "efficiencyReference": "x"}},
        {"id": "b2", "nominalPowerKw": 600.0, "kind": {"kind": "combustion", "carrier": {"kind": "natural_gas"},
            "efficiency": 0.95, "efficiencyReference": "x"}}
    ]);
    annex_p(route, SystemFunction::Heating, &P_H5);
}

fn collective_source(edit: impl FnOnce(&mut Value)) -> Value {
    let mut project = suite_case("EPW203d");
    project["ntaCalculation"]["generator"]["sourceSystem"] = json!("collective_ground");
    project["ntaCalculation"]["externalSupply"] = json!({"collectiveHeatPumpSource": {
        "temperatureClass": "below20_c", "supplierReference": "review 75"}});
    edit(&mut project["ntaCalculation"]["externalSupply"]["collectiveHeatPumpSource"]);
    run(&project)
}

/// Q_HD;hp;in;bron = Q_H;gen;out·(1 − 1/COP) = 3 442,30 kWh (p. 362) in 5.20 as carrier dh. Below 20 °C:
/// f_P;del;dh = 1,45/23, K_CO2 = 0,268/23, f_Pren;dh = 0,95 (9.6.8.1.1.2.3 a); renewable only through the
/// source term of 5.30 (5.31 is for individual sources); E_Final includes dh (5.58), 5.18a in GJ.
#[test]
fn collective_heat_pump_source_below_20_c() {
    let p = collective_source(|_| {});
    let source = 3442.3049188096666;
    close(
        "Q_HD;hp;in;bron",
        service_used(&p, "heatPumpSource", "dh"),
        source,
    );
    let electricity = service_used(&p, "heating", "el")
        + service_used(&p, "ventilation", "el")
        + service_used(&p, "auxiliary", "el");
    let gas = service_used(&p, "hotWater", "gas");
    let perf = &p["performance"];
    close(
        "E_PTot",
        number(&perf["annualPrimaryFossilKwh"]),
        electricity * 1.45 + gas + source * 1.45 / 23.0,
    );
    close(
        "E_PrenTot",
        number(&perf["annualRenewablePrimaryKwh"]),
        source * 0.95,
    );
    close(
        "renewable heat pump heat",
        number(&perf["chapter5"]["renewableByCarrier"]["heatPumpHeat"]),
        0.0,
    );
    close(
        "m_CO2",
        number(&perf["annualCo2Kg"]),
        electricity * 0.268 + gas * 0.218 + source * 0.268 / 23.0,
    );
    close(
        "E_Final",
        number(&perf["annualFinalEnergyKwh"]),
        electricity + gas + source,
    );
    close(
        "E_EPdel;dh GJ",
        number(&perf["chapter5"]["deliveredExternalGj"]),
        source * 3.6 / 1000.0,
    );
    close("RER", number(&perf["renewableSharePercent"]), 31.0);
}

/// A declaration for the source below 20 °C: its f_P;del;dh holds in EMGverklaring and EMGforf alike
/// (5.3.1.2, p. 72 and p. 362).
#[test]
fn collective_heat_pump_source_declared() {
    let p = collective_source(|source| {
        source["annexP"] = json!({"method": "declared", "primaryFactor": 0.03, "renewableFactor": 0.9,
            "co2KgPerKwh": 0.005, "declarationReference": "review 75", "measuredOnly": false});
    });
    let perf = &p["performance"];
    close(
        "E_PTot",
        number(&perf["annualPrimaryFossilKwh"]),
        7155.531969909,
    );
    close(
        "E_PrenTot",
        number(&perf["annualRenewablePrimaryKwh"]),
        3442.3049188096666 * 0.9,
    );
    let scenarios = perf["indicators"]["scenarios"]
        .as_array()
        .expect("scenarios");
    assert_eq!(scenarios.len(), 2);
    for scenario in scenarios {
        close(
            "EwePTot per scenario",
            number(&scenario["primaryFossilIndicatorKwhPerM2Year"]),
            74.54,
        );
    }
}

/// From 20 °C (or surface water, unknown): tables 5.2-5.4 (f_P;del 0,9, K 0,09, f_Pren 0) and Q_H;hp;in = 0
/// (5.33).
#[test]
fn collective_heat_pump_source_from_20_c() {
    let p = collective_source(|source| {
        source["temperatureClass"] = json!("at_least20_c_or_surface_water_or_unknown");
    });
    let perf = &p["performance"];
    close(
        "E_PTot",
        number(&perf["annualPrimaryFossilKwh"]),
        10150.33724927341,
    );
    close("E_PrenTot", number(&perf["annualRenewablePrimaryKwh"]), 0.0);
    close("m_CO2", number(&perf["annualCo2Kg"]), 1764.150015401711);
}

/// 5.60 (p. 133): E_Final;EED = E_Final + Q_W;ren;sol;prac (EPW405a, solar water heater 1 555,15 kWh);
/// both indicators rounded up to 0,01 (5.3h/5.3i).
#[test]
fn final_energy_eed_adds_solar_heat() {
    let p = run(&suite_case("EPW405a"));
    let perf = &p["performance"];
    let solar = number(&perf["hotWater"]["annualSolarRenewableKwh"]);
    close("Q_W;ren;sol", solar, 1555.1499999999999);
    let final_energy = number(&perf["annualFinalEnergyKwh"]);
    close(
        "E_Final;EED",
        number(&perf["annualFinalEnergyEedKwh"]),
        final_energy + solar,
    );
    close(
        "EweFinal;EED",
        number(&perf["chapter5"]["finalEnergyEedKwhPerM2"]),
        96.26,
    );
    close(
        "EweFinal",
        number(&perf["chapter5"]["finalEnergyKwhPerM2"]),
        80.06,
    );
}

// Expected values of the independent recalculation (chapter 12).
const HUM_STEAM_EL: &[(&str, [f64; 12])] = &[
    (
        "humidificationNeedKwh",
        [
            351.64882449977773,
            203.64718908333333,
            270.78194453333333,
            15.972328555555558,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            253.13591984722223,
        ],
    ),
    (
        "steamElectricityKwh",
        [
            439.56103062472215,
            254.55898635416665,
            338.47743066666663,
            19.965410694444447,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            316.41989980902775,
        ],
    ),
    (
        "recoverableKwh",
        [
            70.32976489995553,
            40.72943781666666,
            54.156388906666656,
            3.194465711111111,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            50.62718396944443,
        ],
    ),
];
const HUM_STEAM_GAS_WHEEL_HEALTHCARE: &[(&str, [f64; 12])] = &[
    (
        "humidificationNeedKwh",
        [
            326.13284272205,
            245.676502575,
            287.4942676725,
            158.1260527,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            69.31395878749998,
            281.8818632874999,
        ],
    ),
    (
        "steamFuelKwh",
        [
            543.5547378700834,
            409.460837625,
            479.15711278749995,
            263.5434211666667,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            115.52326464583331,
            469.80310547916656,
        ],
    ),
    (
        "recoverableKwh",
        [
            130.45313708882,
            98.27060103000001,
            114.997707069,
            63.25042108,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            27.725583514999997,
            112.75274531499997,
        ],
    ),
    (
        "dehumidificationNeedKwh",
        [
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 46.8, 206.4, 0.0, 0.0, 0.0, 0.0,
        ],
    ),
];
const HUM_STEAM_OIL_BIG: &[(&str, [f64; 12])] = &[
    (
        "humidificationNeedKwh",
        [
            444.92153099819444,
            289.2223376875,
            362.8054959958333,
            99.82705347222223,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            38.50775488194444,
            346.4529750451389,
        ],
    ),
    (
        "steamOilKwh",
        [
            741.5358849969907,
            482.0372294791667,
            604.6758266597222,
            166.37842245370373,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            64.17959146990741,
            577.4216250752315,
        ],
    ),
    (
        "dehumidificationNeedKwh",
        [
            0.0,
            0.0,
            0.0,
            0.0,
            24.0,
            345.0,
            956.8000000000001,
            1032.0,
            496.59999999999997,
            0.0,
            0.0,
            0.0,
        ],
    ),
];
const HUM_STEAM_EL_INSTALLATION_AREA: &[(&str, [f64; 12])] = &[
    (
        "humidificationNeedKwh",
        [
            351.64882449977773,
            203.64718908333333,
            270.78194453333333,
            15.972328555555558,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            253.13591984722223,
        ],
    ),
    (
        "steamElectricityKwh",
        [
            439.56103062472215,
            254.55898635416665,
            338.47743066666663,
            19.965410694444447,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            316.41989980902775,
        ],
    ),
    (
        "dehumidificationNeedKwh",
        [
            0.0,
            0.0,
            0.0,
            0.0,
            24.0,
            345.0,
            956.8000000000001,
            1032.0,
            496.59999999999997,
            0.0,
            0.0,
            0.0,
        ],
    ),
];
const HUM_ATOMISING_MIXED_RES: &[(&str, [f64; 12])] = &[
    (
        "humidificationNeedKwh",
        [
            118.68147826867497,
            68.730926315625,
            91.38890627999999,
            5.390660887499999,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            85.43337294843748,
        ],
    ),
    (
        "heatingSystemLoadKwh",
        [
            118.68147826867497,
            68.730926315625,
            91.38890627999999,
            5.390660887499999,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            85.43337294843748,
        ],
    ),
    (
        "dehumidificationNeedKwh",
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 9.6, 0.0, 0.0, 0.0, 0.0],
    ),
];
const HUM_DEHUM_ONLY_17_21: &[(&str, [f64; 12])] = &[];
const HUM_DEHUM_UNKNOWN_SPORT: &[(&str, [f64; 12])] = &[(
    "dehumidificationNeedKwh",
    [
        0.0,
        0.0,
        0.0,
        0.0,
        24.0,
        345.0,
        956.8000000000001,
        1032.0,
        496.59999999999997,
        0.0,
        0.0,
        0.0,
    ],
)];

// Expected values of the independent recalculation (annex M, kernel readings named in the header).
const M_COND_GAS: &[(&str, [f64; 12])] = &[
    (
        "loadRatio",
        [
            0.33706140350877195,
            0.30782828282828284,
            0.2797222222222222,
            0.19875,
            0.12268518518518519,
            0.08833333333333335,
            0.0,
            0.0,
            0.106,
            0.1962962962962963,
            0.2523809523809524,
            0.3189814814814815,
        ],
    ),
    (
        "fullLoadEfficiency",
        [
            0.946, 0.952, 0.955, 0.967, 0.979, 0.985, 1.0, 1.0, 0.982, 0.967, 0.955, 0.949,
        ],
    ),
    (
        "partLoadEfficiency",
        [
            1.054, 1.058, 1.06, 1.068, 1.076, 1.08, 1.09, 1.09, 1.078, 1.068, 1.06, 1.056,
        ],
    ),
    (
        "generationLossKwh",
        [
            468.70236125926544,
            327.8861181484633,
            286.3197984010193,
            168.79072926427108,
            92.2985778066803,
            61.38203104801901,
            30.758265914029064,
            30.758265914029064,
            73.61834834465495,
            164.03660019327833,
            257.2823047749499,
            396.6946588157899,
        ],
    ),
    (
        "auxiliaryElectricityKwh",
        [
            7.294976190476191,
            6.068261904761905,
            5.588666666666667,
            3.7500000000000004,
            2.6736666666666666,
            2.2306666666666666,
            2.232,
            2.232,
            2.3720000000000003,
            3.6453333333333333,
            4.986666666666667,
            6.79604761904762,
        ],
    ),
    (
        "auxiliaryToMediumKwh",
        [
            5.4712321428571435,
            4.551196428571429,
            4.1915,
            2.8125000000000004,
            2.00525,
            1.673,
            1.6740000000000002,
            1.6740000000000002,
            1.7790000000000004,
            2.734,
            3.74,
            5.097035714285715,
        ],
    ),
    (
        "inputKwh",
        [
            3537.2311291164087,
            2761.334921719892,
            2296.1282984010195,
            1119.978229264271,
            355.2933278066803,
            102.109031048019,
            29.084265914029064,
            29.084265914029064,
            199.03934834465497,
            1009.3026001932783,
            1949.54230477495,
            3147.5976231015043,
        ],
    ),
    (
        "recoverableToSpaceKwh",
        [
            41.770023535667505,
            32.74663127874925,
            28.22542943044416,
            14.791265598996059,
            5.113881114684547,
            1.2418618944206168,
            0.3906,
            0.3906,
            2.767845003464066,
            13.359447372429786,
            26.303585246192327,
            37.6430589298243,
        ],
    ),
];
const M_STD_GAS: &[(&str, [f64; 12])] = &[
    (
        "loadRatio",
        [
            0.31798245614035087,
            0.2904040404040404,
            0.2638888888888889,
            0.1875,
            0.11574074074074074,
            0.08333333333333333,
            0.0,
            0.0,
            0.1,
            0.18518518518518517,
            0.23809523809523808,
            0.30092592592592593,
        ],
    ),
    (
        "fullLoadEfficiency",
        [
            0.86, 0.86, 0.86, 0.86, 0.86, 0.86, 0.86, 0.86, 0.86, 0.86, 0.86, 0.86,
        ],
    ),
    (
        "partLoadEfficiency",
        [
            0.8448, 0.8456, 0.846, 0.8476, 0.8492, 0.85, 0.852, 0.852, 0.8496, 0.8476, 0.846,
            0.8452,
        ],
    ),
    (
        "generationLossKwh",
        [
            1155.3289046137609,
            919.976881794715,
            776.7705116253907,
            384.9399069316205,
            123.98170743713574,
            34.830911497969495,
            0.0,
            0.0,
            69.5698822871822,
            349.9094088828552,
            664.3744279749961,
            1043.267583953593,
        ],
    ),
    (
        "auxiliaryElectricityKwh",
        [
            7.036047619047618,
            5.849333333333333,
            5.398666666666666,
            3.66,
            2.6486666666666667,
            2.2266666666666666,
            2.232,
            2.232,
            2.3600000000000003,
            3.5653333333333332,
            4.826666666666667,
            6.563904761904762,
        ],
    ),
    (
        "auxiliaryToMediumKwh",
        [
            5.277035714285714,
            4.387,
            4.0489999999999995,
            2.745,
            1.9865,
            1.67,
            1.6740000000000002,
            1.6740000000000002,
            1.7700000000000002,
            2.674,
            3.62,
            4.922928571428572,
        ],
    ),
    (
        "inputKwh",
        [
            4050.0518688994753,
            3215.589881794715,
            2672.7215116253906,
            1282.1949069316206,
            371.9952074371357,
            73.1609114979695,
            0.0,
            0.0,
            187.7998822871822,
            1147.2354088828552,
            2260.7544279749964,
            3638.344655382164,
        ],
    ),
    (
        "recoverableToSpaceKwh",
        [
            27.93148818551651,
            21.079419743807154,
            17.801182335473737,
            8.357887232129546,
            2.565810281000798,
            0.8344544152936875,
            0.558,
            0.558,
            1.4622272570169834,
            7.589931842249925,
            16.56141462421993,
            24.72620797463982,
        ],
    ),
];
const M_LT_OIL_ADD: &[(&str, [f64; 12])] = &[
    (
        "loadRatio",
        [
            0.3275219298245614,
            0.2991161616161616,
            0.2718055555555556,
            0.193125,
            0.11921296296296297,
            0.08583333333333334,
            0.0,
            0.0,
            0.10300000000000001,
            0.19074074074074074,
            0.24523809523809523,
            0.30995370370370373,
        ],
    ),
    (
        "fullLoadEfficiency",
        [
            0.927,
            0.929,
            0.93,
            0.934,
            0.9380000000000001,
            0.9400000000000001,
            0.9450000000000001,
            0.9450000000000001,
            0.9390000000000001,
            0.934,
            0.93,
            0.928,
        ],
    ),
    (
        "partLoadEfficiency",
        [
            0.91,
            0.9123529411764706,
            0.9135294117647059,
            0.9182352941176471,
            0.9229411764705883,
            0.9252941176470588,
            0.9311764705882354,
            0.9311764705882354,
            0.9241176470588236,
            0.9182352941176471,
            0.9135294117647059,
            0.9111764705882354,
        ],
    ),
    (
        "generationLossKwh",
        [
            754.0689614266475,
            603.1336166376542,
            531.8189691429585,
            300.40475256603713,
            149.05334448265512,
            92.07464371232831,
            39.32017872402836,
            39.32017872402836,
            95.97651653233089,
            267.5535237697394,
            453.32261248031796,
            675.0760983123678,
        ],
    ),
    (
        "auxiliaryElectricityKwh",
        [
            7.165511904761905,
            5.964333333333333,
            5.493666666666667,
            3.7050000000000005,
            2.6611666666666665,
            2.228666666666667,
            2.232,
            2.232,
            2.3660000000000005,
            3.6053333333333333,
            4.906666666666667,
            6.679976190476191,
        ],
    ),
    (
        "auxiliaryToMediumKwh",
        [
            5.374133928571428,
            4.47325,
            4.12025,
            2.7787500000000005,
            1.9958749999999998,
            1.6715,
            1.6740000000000002,
            1.6740000000000002,
            1.7745000000000004,
            2.7039999999999997,
            3.6800000000000006,
            5.009982142857144,
        ],
    ),
    (
        "inputKwh",
        [
            3735.694827498076,
            2967.660366637654,
            2484.6987191429585,
            1224.6260025660372,
            404.5574694826551,
            131.60314371232832,
            37.64617872402836,
            37.64617872402836,
            217.8020165323309,
            1088.8495237697393,
            2097.642612480318,
            3348.0661161695107,
        ],
    ),
    (
        "recoverableToSpaceKwh",
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    ),
];
const M_WOOD: &[(&str, [f64; 12])] = &[
    (
        "loadRatio",
        [
            0.33706140350877195,
            0.30782828282828284,
            0.2797222222222222,
            0.19875,
            0.12268518518518519,
            0.08833333333333335,
            0.0,
            0.0,
            0.106,
            0.1962962962962963,
            0.2523809523809524,
            0.3189814814814815,
        ],
    ),
    (
        "fullLoadEfficiency",
        [0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8, 0.8],
    ),
    (
        "partLoadEfficiency",
        [
            0.7928000000000001,
            0.7936000000000001,
            0.794,
            0.7956,
            0.7972,
            0.798,
            0.8,
            0.8,
            0.7976,
            0.7956,
            0.794,
            0.7932,
        ],
    ),
    (
        "generationLossKwh",
        [
            1439.6364082206399,
            1150.6670734820214,
            996.3017303460165,
            543.2216976638501,
            243.84301348220396,
            135.37526329354276,
            85.34868474753661,
            85.34868474753661,
            176.79690123869673,
            506.59166100011396,
            864.4944876044018,
            1304.7157285793915,
        ],
    ),
    (
        "auxiliaryElectricityKwh",
        [
            7.294976190476191,
            6.068261904761905,
            5.588666666666667,
            3.7500000000000004,
            2.6736666666666666,
            2.2306666666666666,
            2.232,
            2.232,
            2.3720000000000003,
            3.6453333333333333,
            4.986666666666667,
            6.79604761904762,
        ],
    ),
    (
        "auxiliaryToMediumKwh",
        [
            5.4712321428571435,
            4.551196428571429,
            4.1915,
            2.8125000000000004,
            2.00525,
            1.673,
            1.6740000000000002,
            1.6740000000000002,
            1.7790000000000004,
            2.734,
            3.74,
            5.097035714285715,
        ],
    ),
    (
        "inputKwh",
        [
            4508.165176077783,
            3584.11587705345,
            3006.1102303460166,
            1494.4091976638501,
            506.83776348220397,
            176.10226329354276,
            83.6746847475366,
            83.6746847475366,
            302.21790123869675,
            1351.857661000114,
            2556.7544876044017,
            4055.618692865106,
        ],
    ),
    (
        "recoverableToSpaceKwh",
        [
            71.53786604571695,
            57.49663850028438,
            50.22931228070176,
            28.128380473105825,
            10.533166186883191,
            2.418458717212675,
            0.4464000000000001,
            0.4464000000000001,
            5.7152918363217164,
            25.36960909246191,
            46.834807017543866,
            65.24452556026046,
        ],
    ),
];
const M_COND_OIL: &[(&str, [f64; 12])] = &[
    (
        "loadRatio",
        [
            0.33706140350877195,
            0.30782828282828284,
            0.2797222222222222,
            0.19875,
            0.12268518518518519,
            0.08833333333333335,
            0.0,
            0.0,
            0.106,
            0.1962962962962963,
            0.2523809523809524,
            0.3189814814814815,
        ],
    ),
    (
        "fullLoadEfficiency",
        [
            0.946, 0.952, 0.955, 0.967, 0.979, 0.985, 1.0, 1.0, 0.982, 0.967, 0.955, 0.949,
        ],
    ),
    (
        "partLoadEfficiency",
        [
            1.062,
            1.064,
            1.0650000000000002,
            1.0690000000000002,
            1.073,
            1.075,
            1.08,
            1.08,
            1.074,
            1.0690000000000002,
            1.0650000000000002,
            1.0630000000000002,
        ],
    ),
    (
        "generationLossKwh",
        [
            288.4579311073495,
            189.32433538954564,
            172.74873782084336,
            117.28221448850687,
            76.97235888387489,
            56.91090928797203,
            29.372758440424143,
            29.372758440424143,
            65.06282695653343,
            117.62060523931905,
            160.9150981974576,
            237.13535070641484,
        ],
    ),
    (
        "auxiliaryElectricityKwh",
        [
            7.294976190476191,
            6.068261904761905,
            5.588666666666667,
            3.7500000000000004,
            2.6736666666666666,
            2.2306666666666666,
            2.232,
            2.232,
            2.3720000000000003,
            3.6453333333333333,
            4.986666666666667,
            6.79604761904762,
        ],
    ),
    (
        "auxiliaryToMediumKwh",
        [
            5.4712321428571435,
            4.551196428571429,
            4.1915,
            2.8125000000000004,
            2.00525,
            1.673,
            1.6740000000000002,
            1.6740000000000002,
            1.7790000000000004,
            2.734,
            3.74,
            5.097035714285715,
        ],
    ),
    (
        "inputKwh",
        [
            3356.9866989644925,
            2622.773138960974,
            2182.5572378208435,
            1068.469714488507,
            339.9671088838749,
            97.63790928797204,
            27.698758440424143,
            27.698758440424143,
            190.48382695653345,
            962.886605239319,
            1853.1750981974576,
            2988.0383149921295,
        ],
    ),
    (
        "recoverableToSpaceKwh",
        [
            39.94599638691371,
            31.319393195622407,
            26.99806849513886,
            14.154553184626867,
            4.9046023107197785,
            1.203506253530799,
            0.3906,
            0.3906,
            2.6618654988035217,
            12.786406199497513,
            25.158048373240717,
            36.00099809214453,
        ],
    ),
];

// Expected values of the independent recalculation (annex P).
const P_H1: AnnexPExpected = AnnexPExpected {
    distribution: 0.8333333333333334,
    generation: 0.45701522435897435,
    primary: 0.58,
    co2: 0.1120173076923077,
    renewable: 0.58,
    beta: None,
    generators: &[
        (
            "aquifer-heat-pump",
            0.6,
            0.4461538461538461,
            0.08246153846153847,
            0.6153846153846154,
            8640000.0,
        ),
        ("geothermal", 0.25, 0.090625, 0.01675, 0.9375, 3600000.0),
        (
            "peak-boiler",
            0.15,
            1.1111111111111112,
            0.24222222222222223,
            0.0,
            2160000.0,
        ),
    ],
};
const P_C1: AnnexPExpected = AnnexPExpected {
    distribution: 1.0,
    generation: 0.0725,
    primary: 0.1,
    co2: 0.018425,
    renewable: 0.9090909090909091,
    beta: None,
    generators: &[("aquifer-cold", 1.0, 0.0725, 0.0134, 1.0, 800000.0)],
};
const P_H2: AnnexPExpected = AnnexPExpected {
    distribution: 0.8333333333333334,
    generation: 0.090625,
    primary: 0.14,
    co2: 0.02412,
    renewable: 0.88,
    beta: None,
    generators: &[("geo", 1.0, 0.090625, 0.01675, 0.9375, 14400000.0)],
};
const P_H3: AnnexPExpected = AnnexPExpected {
    distribution: 0.9090909090909091,
    generation: 0.3868394736842105,
    primary: 0.44,
    co2: 0.0917795157894737,
    renewable: 0.62,
    beta: None,
    generators: &[
        (
            "rw",
            0.7,
            0.1015,
            0.018760000000000002,
            0.9299999999999999,
            3849999.9999999995,
        ),
        (
            "b",
            0.3,
            1.0526315789473684,
            0.22947368421052633,
            0.0,
            1650000.0,
        ),
    ],
};
const P_H4: AnnexPExpected = AnnexPExpected {
    distribution: 0.8888888888888888,
    generation: 0.6868044515103339,
    primary: 0.8,
    co2: 0.17178879173290937,
    renewable: 0.31,
    beta: None,
    generators: &[
        (
            "gwp",
            0.8,
            0.5882352941176471,
            0.12823529411764706,
            0.4117647058823529,
            7200000.0,
        ),
        (
            "hr",
            0.2,
            1.081081081081081,
            0.23567567567567566,
            0.0,
            1800000.0,
        ),
    ],
};
const P_H5: AnnexPExpected = AnnexPExpected {
    distribution: 0.8417508417508418,
    generation: 0.676776980329612,
    primary: 0.83,
    co2: 0.16414698947368422,
    renewable: 0.39,
    beta: Some(0.20202020202020202),
    generators: &[
        (
            "hp",
            0.7028282828282828,
            0.5178571428571429,
            0.09571428571428572,
            0.6428571428571428,
            4174800.0,
        ),
        (
            "b1",
            0.19811447811447813,
            1.0526315789473684,
            0.22947368421052633,
            0.0,
            1176800.0,
        ),
        (
            "b2",
            0.09905723905723907,
            1.0526315789473684,
            0.22947368421052633,
            0.0,
            588400.0,
        ),
    ],
};
