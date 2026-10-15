//! Review 77: coverage sweep of NTA 8800:2025+C1:2026 chapters 5-8 and annexes A-I and AA
//! (docs/nta8800-dekking-ch5-8.md). Provisions that the kernel implements but that no earlier
//! independent test exercised are recalculated here from the norm pages: the renewable energy per
//! carrier (5.28-5.39h with table 5.4, p. 103-112), the need indicators (5.3a-5.3g, p. 74-77), BENG 1
//! with the fixed C1 run (5.1, 5.4-5.8 and the fixed q_L of 5.4.2, p. 71, 80-82), the cooling
//! temperature as output variable (7.80/7.81, p. 221), the examples of table 8.1 for 8.3 (p. 225) and
//! tables F.1-F.3 (p. 814-817). Inputs from chapters 7, 9, 10, 13 and 16 (verified in reviews 63-70)
//! are taken from the kernel output where a route needs them; the expected values come from a
//! separate calculation written from the norm pages. Only numbers and page references are kept here.
//! Tolerance 1e-6 relative.
//!
//! Reading kept (not a kernel deviation, NEN question 17): 5.36 for the forfait hot-water heat pump
//! books E_W;gen;in·(1,4·c_source − 1) as in the ISSO 54 BRL 9501 test set; the literal
//! Q_W;gen;out·(1 − 1/η_W;gen;prac) with η including c_W;gen would give 310,28 kWh instead of
//! 967,35 kWh in `hot_water_heat_pump_renewable`.

use nta8800_core::constructions::{
    forfait_bridge_supplement, narrow_cavity_resistance, tubular_cavity_lambda, HeatFlow,
    TubeOrientation,
};
use nta8800_core::monthly_demand::UsageFunction;
use nta8800_core::project_performance::assess_project_performance;
use serde_json::{json, Value};

const EPW001: &str = include_str!("../../../training-data/isso54/EPW001.json");
const SUITE: &str = include_str!("../../../training-data/reference-suites/isso54-v2.json");
const DWELLING: &str =
    include_str!("../../../training-data/nta8800-example-terraced-dwelling.json");
const OFFICE: &str = include_str!("../../../training-data/nta8800-example-office.json");

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

fn in_target_edition(mut project: Value) -> Value {
    let block = project["ntaCalculation"].as_object_mut().expect("block");
    block.remove("normVersion");
    if let Some(emission) = block.get_mut("emission").and_then(Value::as_object_mut) {
        emission.remove("edition2023");
    }
    project
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

fn example(text: &str) -> Value {
    in_target_edition(serde_json::from_str(text).expect("json"))
}

/// The performance block of a calculated assessment without gaps.
fn performance(project: &Value) -> Value {
    let result = assess_project_performance(project);
    assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
    assert!(result.gaps.is_empty(), "{:?}", result.gaps);
    serde_json::to_value(result.performance.expect("performance")).expect("serializable")
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or_else(|| panic!("number: {value}"))
}

fn carrier(p: &Value, key: &str) -> f64 {
    number(&p["chapter5"]["renewableByCarrier"][key])
}

/// 5.28: the carriers of 5.39a-h add up to E_PrenTot.
fn total_is_sum_of_carriers(label: &str, p: &Value) {
    let sum: f64 = p["chapter5"]["renewableByCarrier"]
        .as_object()
        .expect("carriers")
        .values()
        .map(number)
        .sum();
    close(label, number(&p["annualRenewablePrimaryKwh"]), sum);
}

// ---------------------------------------------------------------------------------------------
// 5.6.2 and 5.6.4: renewable energy per carrier (p. 103-112) with table 5.4 (p. 110)
// ---------------------------------------------------------------------------------------------

fn rows<'a>(p: &'a Value, pointer: &str) -> &'a Vec<Value> {
    p.pointer(pointer)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("array {pointer}"))
}

fn field_sum(p: &Value, pointer: &str, field: &str) -> f64 {
    rows(p, pointer).iter().map(|row| number(&row[field])).sum()
}

/// 5.31 per month: Q_H;gen;out·(1 − 1/COP_prac) when COP_prac = Q_H;gen;out/E_H;gen;in ≥ 1 (chapter 9
/// output), else 0 (5.33).
fn heat_pump_ambient(p: &Value) -> Vec<f64> {
    rows(p, "/spaceHeating/monthly")
        .iter()
        .map(|row| {
            let output = number(&row["heatPumpOutputKwh"]);
            let input = number(&row["generatorElectricityKwh"]);
            if output > 0.0 && output / input >= 1.0 {
                output * (1.0 - 1.0 / (output / input))
            } else {
                0.0
            }
        })
        .collect()
}

/// EPW203d (ground-source heat pump below 20 °C, COP ≥ 1 in every heating month): 5.31 per month with
/// f_Pren;renheat 1,0 (5.30, 5.39c, table 5.4).
#[test]
fn heat_pump_space_heating_renewable() {
    let p = performance(&suite_case("EPW203d"));
    let ambient = heat_pump_ambient(&p);
    assert!(ambient[0] > 900.0 && ambient[6] == 0.0);
    let renewable = rows(&p, "/energyByService/renewable");
    for (index, expected) in ambient.iter().enumerate() {
        let kernel: f64 = renewable
            .iter()
            .filter(|row| row["service"] == "heating" && row["month"] == index as u64 + 1)
            .map(|row| number(&row["renewablePrimaryKwh"]))
            .sum();
        close(&format!("5.31 m{}", index + 1), kernel, 1.0 * expected);
    }
    close(
        "5.39c",
        carrier(&p, "heatPumpHeat"),
        1.0 * ambient.iter().sum::<f64>(),
    );
    total_is_sum_of_carriers("5.28 EPW203d", &p);
}

/// EPW203d turned to an outdoor-air heat pump whose source is partly exhaust air (5.32, p. 104): only the
/// declared outdoor share f_H;buitenlucht 0,4 of 5.31 counts; without a declared share it is 0.
#[test]
fn heat_pump_partly_exhaust_air_renewable() {
    let mut project = suite_case("EPW203d");
    set(
        &mut project,
        "/ntaCalculation/generator/forfait/source",
        Some(json!("outdoor_air")),
    );
    set(
        &mut project,
        "/ntaCalculation/generator/forfait/sourceCorrectionFactor",
        None,
    );
    set(
        &mut project,
        "/ntaCalculation/generator/forfait/sourceCorrectionReference",
        None,
    );
    set(
        &mut project,
        "/ntaCalculation/heatPumpRenewable/combinedOutdoorAndExhaustAir",
        Some(json!(true)),
    );
    let p = performance(&project);
    close("5.32 unknown share", carrier(&p, "heatPumpHeat"), 0.0);
    set(
        &mut project,
        "/ntaCalculation/heatPumpRenewable/outdoorAirHeatFraction",
        Some(json!(0.4)),
    );
    set(
        &mut project,
        "/ntaCalculation/heatPumpRenewable/outdoorAirFractionReference",
        Some(json!("r77 declaration")),
    );
    let p = performance(&project);
    let ambient: f64 = heat_pump_ambient(&p).iter().sum();
    assert!(ambient > 0.0);
    close("5.32 share 0,4", carrier(&p, "heatPumpHeat"), 0.4 * ambient);
    total_is_sum_of_carriers("5.28 EPW203d partly exhaust", &p);
}

/// EPW204c: wood stove ≤ 500 kW meeting annex R (bmB), f_Pren 0,5 on Q_H;gen;out (5.30, 5.39f).
#[test]
fn biomass_renewable() {
    let p = performance(&suite_case("EPW204c"));
    let output = field_sum(&p, "/spaceHeating/monthly", "biomassOutputKwh");
    assert!(output > 4000.0);
    close("5.39f", carrier(&p, "biomass"), 0.5 * output);
    total_is_sum_of_carriers("5.28 EPW204c", &p);
}

/// EPW204b: external heat with a quality declaration (dhkwal, f_Pren;dh 0,3 from the declaration) on
/// Q_H;gen;out (5.30, 5.39g).
#[test]
fn external_heat_declared_renewable() {
    let p = performance(&suite_case("EPW204b"));
    let output = field_sum(&p, "/spaceHeating/monthly", "districtHeatKwh");
    assert!(output > 4000.0);
    close("5.39g", carrier(&p, "externalHeat"), 0.3 * output);
    total_is_sum_of_carriers("5.28 EPW204b", &p);
}

/// EPW405a: solar water heater, Q_W;ren;sol;prac of chapter 13 (1 555,15 kWh) with f_Pren;renheat 1,0
/// (5.35, 5.39d).
#[test]
fn solar_water_heater_renewable() {
    let p = performance(&suite_case("EPW405a"));
    let solar = field_sum(&p, "/hotWater/months", "solarRenewableKwh");
    close("Q_W;ren;sol;prac", solar, 1555.15);
    close("5.39d", carrier(&p, "solarHeat"), 1.0 * solar);
    total_is_sum_of_carriers("5.28 EPW405a", &p);
}

/// PV: 5.39 with f_Pren;renelect 1,45 on E_el;PV;out of chapter 16 (EPW501a, the example dwelling and
/// office), 5.39a.
#[test]
fn pv_renewable() {
    for (label, project) in [
        ("EPW501a", suite_case("EPW501a")),
        ("dwelling", example(DWELLING)),
        ("office", example(OFFICE)),
    ] {
        let p = performance(&project);
        let produced: f64 = rows(&p, "/pvSystems")
            .iter()
            .flat_map(|system| system["monthlyKwh"].as_array().expect("months").clone())
            .map(|value| number(&value))
            .sum();
        assert!(produced > 1000.0);
        close(
            &format!("5.39a {label}"),
            carrier(&p, "electricity"),
            1.45 * produced,
        );
        total_is_sum_of_carriers(&format!("5.28 {label}"), &p);
    }
}

/// EPW301a with the chiller replaced by free cooling on a closed ground loop: EER_fc 10 (table 10.34) ≥ 8,
/// so Q_C;gen;out counts with f_Pren;rencold 1,0 (5.34, 5.39e).
#[test]
fn free_cooling_renewable() {
    let mut project = suite_case("EPW301a");
    set(
        &mut project,
        "/ntaCalculation/cooling/generators/0/generator",
        Some(
            json!({"kind": "free_cooling", "source": "closed_ground_loop", "heatPumpSource": false}),
        ),
    );
    set(
        &mut project,
        "/ntaCalculation/activeCooling/system",
        Some(json!("free_cooling_table10_34")),
    );
    let p = performance(&project);
    let cold = field_sum(&p, "/cooling/months", "generatorColdKwh");
    assert!(cold > 1000.0);
    // 10.79: W_fc = Q/EER_fc is auxiliary energy (next to the distribution
    // pump); free cooling has no E_C;el.
    close(
        "E_C;el of free cooling",
        field_sum(&p, "/cooling/months", "electricityKwh"),
        0.0,
    );
    assert!(field_sum(&p, "/cooling/months", "auxiliaryElectricityKwh") >= cold / 10.0 - 1e-9);
    close("5.39e", carrier(&p, "cold"), 1.0 * cold);
    total_is_sum_of_carriers("5.28 free cooling", &p);
}

/// External cold with a quality declaration (dckwal, f_Pren;dc 0,6) on Q_C;gen;out per month and per year
/// (5.34, 5.39h).
#[test]
fn external_cold_declared_renewable() {
    let mut project = suite_case("EPW301a");
    set(
        &mut project,
        "/ntaCalculation/cooling/generators/0/generator",
        Some(json!({"kind": "external_cold"})),
    );
    set(
        &mut project,
        "/ntaCalculation/activeCooling/system",
        Some(json!("external_cold_with_cooling_emitter")),
    );
    set(
        &mut project,
        "/ntaCalculation/externalSupply",
        Some(
            json!({"cooling": {"method": "declared", "primaryFactor": 0.4, "renewableFactor": 0.6,
            "co2KgPerKwh": 0.05, "measuredOnly": true, "declarationReference": "r77"}}),
        ),
    );
    let p = performance(&project);
    let months = rows(&p, "/cooling/months");
    let renewable = rows(&p, "/energyByService/renewable");
    for (index, month) in months.iter().enumerate() {
        let kernel: f64 = renewable
            .iter()
            .filter(|row| row["service"] == "cooling" && row["month"] == index as u64 + 1)
            .map(|row| number(&row["renewablePrimaryKwh"]))
            .sum();
        close(
            &format!("5.34 m{}", index + 1),
            kernel,
            0.6 * number(&month["districtColdKwh"]),
        );
    }
    let cold = field_sum(&p, "/cooling/months", "districtColdKwh");
    assert!(cold > 1000.0);
    close("5.39h", carrier(&p, "externalCold"), 0.6 * cold);
    total_is_sum_of_carriers("5.28 external cold", &p);
}

/// Hot-water heat pumps (5.36-5.38, p. 107-108): the outdoor-air heat pump of EPW406d (forfait, table
/// 13.25 η 1,4) books E_W;gen;in·(1,4 − 1) (reading of NEN question 17); a partly exhaust-air source with an
/// outdoor share 0,4 takes that fraction (5.37); an exhaust-air heat pump without a declared outdoor share
/// books 0 (f_W;buitenlucht = 0); a booster heat pump books 0 (5.38).
#[test]
fn hot_water_heat_pump_renewable() {
    let mut outdoor = suite_case("EPW406d");
    set(
        &mut outdoor,
        "/ntaCalculation/hotWater/generator/exhaustAirSource",
        Some(json!(false)),
    );
    set(&mut outdoor, "/ntaCalculation/hotWater/exhaustAir", None);
    let p = performance(&outdoor);
    let electricity = field_sum(&p, "/hotWater/months", "electricityKwh");
    // E_W;gen;in 2 418,37 kWh at the time of review 77; the literal 5.36 would give 310,28 kWh.
    assert!(electricity > 2000.0);
    close(
        "5.36",
        carrier(&p, "heatPumpHeat"),
        (1.4 - 1.0) * electricity,
    );
    total_is_sum_of_carriers("5.28 hot-water heat pump", &p);

    let mut partly = suite_case("EPW406d");
    set(
        &mut partly,
        "/ntaCalculation/hotWater/generator/outdoorAirFraction",
        Some(json!(0.4)),
    );
    let p = performance(&partly);
    let electricity = field_sum(&p, "/hotWater/months", "electricityKwh");
    close(
        "5.37",
        carrier(&p, "heatPumpHeat"),
        0.4 * (1.4 - 1.0) * electricity,
    );

    let p = performance(&suite_case("EPW406d"));
    close("exhaust air, share 0", carrier(&p, "heatPumpHeat"), 0.0);
    let p = performance(&suite_case("EPW406i"));
    close("5.38 booster", carrier(&p, "heatPumpHeat"), 0.0);
    close(
        "5.38 booster total",
        number(&p["annualRenewablePrimaryKwh"]),
        0.0,
    );
}

// ---------------------------------------------------------------------------------------------
// 5.3.2-5.3.4 need indicators (p. 74-77), 5.3.1.1 and 5.4 BENG 1 (p. 71, 80-82)
// ---------------------------------------------------------------------------------------------

/// 7.46-7.51 (p. 210-211): η_H;gn with a_H = 1 + τ_H/15.
fn heating_utilisation(gains: f64, transfer: f64, tau: f64) -> f64 {
    let a = 1.0 + tau / 15.0;
    let gamma = gains / transfer;
    if gamma > 0.0 && gamma != 1.0 {
        (1.0 - gamma.powf(a)) / (1.0 - gamma.powf(a + 1.0))
    } else if gamma == 1.0 {
        a / (a + 1.0)
    } else if gains > 0.0 {
        1.0 / gamma
    } else {
        1.0
    }
}

/// 7.52-7.56 (p. 212): η_C;ht with a_C = 1 + τ_C/15.
fn cooling_utilisation(gains: f64, transfer: f64, tau: f64) -> f64 {
    let a = 1.0 + tau / 15.0;
    let gamma = gains / transfer;
    if gamma > 0.0 && gamma != 1.0 {
        (1.0 - gamma.powf(-a)) / (1.0 - gamma.powf(-(a + 1.0)))
    } else if gamma == 1.0 {
        a / (a + 1.0)
    } else {
        1.0
    }
}

/// 5.3b/c and 5.3e/f (p. 75-76): the monthly balance of 7.2 without Δη·Q and the recoverable system
/// losses, from Q_ht, Q_gn, τ and a_C;red of chapter 7: heating 7.1/7.2 and max(0, Q_H;ht − η_H;gn·Q_H;gn),
/// cooling 7.6 and max(0, a_C;red·(Q_C;gn − η_C;ht·Q_C;ht)).
fn net_needs(p: &Value) -> (f64, f64) {
    let mut heating = 0.0;
    let mut cooling = 0.0;
    for month in rows(p, "/spaceHeating/demand/monthly") {
        let h = &month["heating"];
        let (gains, transfer) = (number(&h["gainsKwh"]), number(&h["heatTransferKwh"]));
        if !(transfer <= 0.0 && gains > 0.0) && gains / transfer <= 2.0 {
            let eta = heating_utilisation(gains, transfer, number(&h["timeConstantH"]));
            heating += (transfer - eta * gains).max(0.0);
        }
        let c = &month["cooling"];
        let (gains, transfer) = (number(&c["gainsKwh"]), number(&c["heatTransferKwh"]));
        if transfer / gains <= 2.0 {
            let eta = cooling_utilisation(gains, transfer, number(&c["timeConstantH"]));
            cooling += (number(&c["reductionFactor"]) * (gains - eta * transfer)).max(0.0);
        }
    }
    (heating, cooling)
}

fn round_up_hundredth(value: f64) -> f64 {
    (value * 100.0 - 1e-9).ceil() / 100.0
}

/// 5.3a/5.3d rounded up to 0,01 and 5.3g as the sum of the rounded indicators.
#[test]
fn need_indicators() {
    for (label, project, area) in [
        ("office", example(OFFICE), 400.0),
        ("dwelling", example(DWELLING), 100.0),
    ] {
        let p = performance(&project);
        let (heating, cooling) = net_needs(&p);
        assert!(heating > 1000.0 && cooling > 100.0);
        let demand = &p["spaceHeating"]["demand"];
        close(
            &format!("5.3b {label}"),
            number(&demand["annualHeatingNeedWithoutRecoverableKwh"]),
            heating,
        );
        close(
            &format!("5.3e {label}"),
            number(&demand["annualCoolingNeedWithoutRecoverableKwh"]),
            cooling,
        );
        let (e_h, e_c) = (
            round_up_hundredth(heating / area),
            round_up_hundredth(cooling / area),
        );
        let c5 = &p["chapter5"];
        close(
            &format!("5.3a {label}"),
            number(&c5["heatingNeedKwhPerM2"]),
            e_h,
        );
        close(
            &format!("5.3d {label}"),
            number(&c5["coolingNeedKwhPerM2"]),
            e_c,
        );
        close(
            &format!("5.3g {label}"),
            number(&c5["heatingAndCoolingNeedKwhPerM2"]),
            e_h + e_c,
        );
    }
}

/// 5.1 with 5.4-5.8: the monthly needs of the fixed C1 run summed over the zones and the year and divided
/// by A_g;tot, rounded up to 0,01. The example dwelling has C1 of its own (§5.4.3.1), so its run is the
/// 5.3b/5.3e balance; the office runs C1 with the fixed q_L of §5.4.2.
#[test]
fn beng_one_from_the_fixed_c1_run() {
    for (label, project, area) in [
        ("office", example(OFFICE), 400.0),
        ("dwelling", example(DWELLING), 100.0),
    ] {
        let p = performance(&project);
        let c1 = &p["spaceHeating"]["demand"]["fixedC1"];
        let total = if c1.is_null() {
            let (heating, cooling) = net_needs(&p);
            heating + cooling
        } else {
            ["monthlyHeatingNeedKwh", "monthlyCoolingNeedKwh"]
                .iter()
                .map(|key| {
                    c1[*key]
                        .as_array()
                        .expect("months")
                        .iter()
                        .map(number)
                        .sum::<f64>()
                })
                .sum()
        };
        assert!(total > 1000.0);
        close(
            &format!("5.4 {label}"),
            number(&p["annualHeatingAndCoolingNeedKwh"]),
            total,
        );
        close(
            &format!("5.1 {label}"),
            number(&p["needIndicatorKwhPerM2Year"]),
            round_up_hundredth(total / area),
        );
    }
}

/// §5.4.2 (p. 80-82): the fixed q_L per utility function for the BENG 1 run, W/m².
#[test]
fn fixed_lighting_flux_of_the_c1_run() {
    use UsageFunction::*;
    for (function, expected) in [
        (AssemblyChildCare, 2.5),
        (OtherAssembly, 2.5),
        (Cell, 2.25),
        (HealthcareWithBeds, 2.5),
        (OtherHealthcare, 1.25),
        (Office, 1.25),
        (Lodging, 1.75),
        (Education, 1.0),
        (Sport, 3.0),
        (Retail, 3.0),
    ] {
        close(
            &format!("q_L {function:?}"),
            function.fixed_lighting_flux_w_per_m2(),
            expected,
        );
    }
}

// ---------------------------------------------------------------------------------------------
// 7.9.6: calculated cooling temperature as output variable (7.80/7.81, p. 221)
// ---------------------------------------------------------------------------------------------

/// θ_int;op;C = θ_e;avg + (Q_C;nd + Q_C;gn)/(H_C;ht·0,001·t) with H_C;ht = Q_C;ht/((θ_int;set;C −
/// θ_e;avg)·0,001·t), as printed (the sign of Q_C;nd included); output only, no effect on the indicators.
#[test]
fn cooling_operative_temperature() {
    let p = performance(&example(OFFICE));
    for (index, month) in rows(&p, "/spaceHeating/demand/monthly").iter().enumerate() {
        let c = &month["cooling"];
        let (outdoor, hours) = (
            number(&month["outdoorTemperatureC"]),
            number(&month["hours"]),
        );
        let transfer =
            number(&c["heatTransferKwh"]) / ((number(&c["setpointC"]) - outdoor) * 0.001 * hours);
        let expected =
            outdoor + (number(&c["needKwh"]) + number(&c["gainsKwh"])) / (transfer * 0.001 * hours);
        close(
            &format!("7.80 m{}", index + 1),
            number(&c["operativeTemperatureC"]),
            expected,
        );
    }
}

// ---------------------------------------------------------------------------------------------
// 8.3 with table 8.1 (p. 225) and annex F (p. 814-817)
// ---------------------------------------------------------------------------------------------

/// Table 8.1 gives example values of 8.3: mean U 0,8 → 0,0; 0,6 → 0,05; 0,4 → 0,1; the mean is area
/// weighted and the supplement never negative.
#[test]
fn table_8_1_examples_of_formula_8_3() {
    close("U 0,8", forfait_bridge_supplement(&[(12.0, 0.8)]), 0.0);
    close("U 0,6", forfait_bridge_supplement(&[(12.0, 0.6)]), 0.05);
    close("U 0,4", forfait_bridge_supplement(&[(12.0, 0.4)]), 0.1);
    close(
        "U weighted 0,4",
        forfait_bridge_supplement(&[(30.0, 0.2), (10.0, 1.0)]),
        0.1,
    );
    close("U 0,2", forfait_bridge_supplement(&[(5.0, 0.2)]), 0.15);
    close("U 1,2", forfait_bridge_supplement(&[(5.0, 1.2)]), 0.0);
}

/// Table F.1 (rows d, columns d/b; the 25 mm row also for 25 to 500 mm), tables F.2 (horizontal) and
/// F.3 (vertical tubes; rows b, columns d) with 1,2 upward and 0,8 downward for d > 10 mm (p. 815-817).
/// Between the printed points the kernel interpolates linearly (the annex gives no rule).
#[test]
fn annex_f_air_cavities() {
    close("F.1 2/10", narrow_cavity_resistance(2.0, 0.2), 0.07);
    close("F.1 15/1", narrow_cavity_resistance(15.0, 15.0), 0.22);
    close("F.1 25/0,1", narrow_cavity_resistance(25.0, 250.0), 0.18);
    close("F.1 7/0,3", narrow_cavity_resistance(7.0, 7.0 / 0.3), 0.14);
    close("F.1 100/0,5", narrow_cavity_resistance(100.0, 200.0), 0.22);
    close("F.1 5/4", narrow_cavity_resistance(5.0, 1.25), 0.135);
    close(
        "F.2 b5 d60",
        tubular_cavity_lambda(60.0, 5.0, TubeOrientation::Horizontal, HeatFlow::Horizontal),
        0.2242,
    );
    close(
        "F.2 b30 d40",
        tubular_cavity_lambda(
            40.0,
            30.0,
            TubeOrientation::Horizontal,
            HeatFlow::Horizontal,
        ),
        0.242,
    );
    close(
        "F.3 b80 d80",
        tubular_cavity_lambda(80.0, 80.0, TubeOrientation::Vertical, HeatFlow::Horizontal),
        0.391,
    );
    close(
        "F.3 b10 d10",
        tubular_cavity_lambda(10.0, 10.0, TubeOrientation::Vertical, HeatFlow::Horizontal),
        0.059,
    );
    close(
        "F.2 b20 d20 upward",
        tubular_cavity_lambda(20.0, 20.0, TubeOrientation::Horizontal, HeatFlow::Upward),
        1.2 * 0.133,
    );
    close(
        "F.3 b50 d30 downward",
        tubular_cavity_lambda(30.0, 50.0, TubeOrientation::Vertical, HeatFlow::Downward),
        0.8 * 0.157,
    );
    close(
        "F.2 b40 d10 upward (d not above 10)",
        tubular_cavity_lambda(10.0, 40.0, TubeOrientation::Horizontal, HeatFlow::Upward),
        0.079,
    );
}
