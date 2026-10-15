//! Review 77: coverage sweep of NTA 8800:2025+C1:2026 chapters 9 and 10 (and annexes J to R). Provisions
//! that the kernel implements but that no earlier independent recalculation (reviews 63-76) reached are
//! recalculated here from the norm pages with a separate implementation (the functions below, written before
//! reading the kernel's formula code) and compared with the kernel at 1e-6 relative. Each case is an ISSO 54 v2.0
//! deeltest (training-data/reference-suites/isso54-v2.json) run in 2025+C1 with a small edit; the derived input
//! taken from the kernel (monthly need, outdoor temperature, calculation temperature and setpoint, Q_H;ht, the
//! emission increment of 9.12, dehumidification, the method-2 monthly EER, heat-pump heat) was itself recalculated
//! in reviews 63-68. Only numbers and page references are kept; the classification is in
//! docs/nta8800-dekking-ch9-10.md.

use nta8800_core::project_performance::assess_project_performance;
use serde_json::Value;

const BASE: &str = include_str!("../../../training-data/isso54/EPW001.json");
const SUITE: &str = include_str!("../../../training-data/reference-suites/isso54-v2.json");

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

/// The case's project in 2025+C1 (default edition; the 2022/2023 emission block is dropped) with the edits.
fn performance(case: &str, edits: &str) -> Value {
    let mut project: Value = serde_json::from_str(BASE).expect("base");
    let suite: Value = serde_json::from_str(SUITE).expect("suite");
    let wanted = format!("isso54-v2-{case}");
    let entry = suite["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|item| item["caseId"] == wanted.as_str())
        .expect("case");
    assert_eq!(entry["projectFile"], "../isso54/EPW001.json");
    for op in entry["projectPatch"].as_array().expect("patch") {
        let pointer = op["pointer"].as_str().expect("pointer");
        let value = (op["op"] != "remove").then(|| op["value"].clone());
        set(&mut project, pointer, value);
    }
    let block = project["ntaCalculation"].as_object_mut().expect("block");
    block.remove("normVersion");
    if let Some(emission) = block.get_mut("emission").and_then(Value::as_object_mut) {
        emission.remove("edition2023");
    }
    let edits: Value = serde_json::from_str(edits).expect("edits");
    for edit in edits.as_array().expect("edits") {
        let pointer = edit["pointer"].as_str().expect("pointer");
        let value = (edit["op"] != "remove").then(|| edit["value"].clone());
        set(&mut project, pointer, value);
    }
    let result = assess_project_performance(&project);
    assert_eq!(
        result.status, "calculated_unverified",
        "{case}: {:?}",
        result.gaps
    );
    serde_json::to_value(result.performance.expect("performance")).expect("serializable")
}

fn at(p: &Value, pointer: &str) -> f64 {
    p.pointer(pointer)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("{pointer}"))
}

fn close(label: &str, kernel: f64, expected: f64) {
    let diff = (kernel - expected).abs();
    let scale = kernel.abs().max(expected.abs()).max(1e-12);
    assert!(
        diff <= 1e-9 || diff / scale <= 1e-6,
        "{label}: kernel {kernel} independent {expected}"
    );
}

// ---------------------------------------------------------------------------------------------------------------
// Derived input taken from the kernel (chapter 7 and 17, recalculated in reviews 63-65).
// ---------------------------------------------------------------------------------------------------------------

fn months(p: &Value, pointer: impl Fn(usize) -> String) -> [f64; 12] {
    std::array::from_fn(|m| at(p, &pointer(m)))
}

struct Derived {
    /// ϑ_e;avg;mi (table 17.1).
    outdoor: [f64; 12],
    /// t_mi (17.2), h.
    hours: [f64; 12],
    /// Q_H;nd;zt,j;ϑH,stook;mi of 9.28 (with chapter 11 input only), kWh.
    heating_limit_need: [f64; 12],
    /// ϑ_int;calc;H (7.9.2) = ϑ_int;op;H (7.9.6).
    calculation_temperature: [f64; 12],
    /// ϑ_int;set;H;zi,mi of 7.76 (levelled).
    levelled_setpoint: [f64; 12],
    /// Q_H;ht of 7.10, kWh.
    heat_transfer: [f64; 12],
    /// Q_C;nd, kWh.
    cooling_need: [f64; 12],
}

fn derived(p: &Value) -> Derived {
    let d = "/spaceHeating/demand";
    Derived {
        outdoor: months(p, |m| format!("{d}/monthly/{m}/outdoorTemperatureC")),
        hours: months(p, |m| format!("{d}/monthly/{m}/hours")),
        heating_limit_need: months(p, |m| format!("{d}/heatingLimitNeedKwh/{m}")),
        calculation_temperature: months(p, |m| {
            format!("{d}/monthly/{m}/heating/calculationTemperatureC")
        }),
        levelled_setpoint: months(p, |m| format!("{d}/monthly/{m}/heating/setpointC")),
        heat_transfer: months(p, |m| format!("{d}/monthly/{m}/heating/heatTransferKwh")),
        cooling_need: months(p, |m| format!("{d}/monthly/{m}/cooling/needKwh")),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Independent implementation of chapter 9 (p. 285-321) and chapter 10 (p. 366-425), written from the norm pages.
// ---------------------------------------------------------------------------------------------------------------

/// Table 9.15 (p. 308): t_H;zi,mi for ϑ_H,stook 16 down to 8.
const TABLE_9_15: [[f64; 12]; 9] = [
    [
        744.0, 672.0, 733.0, 666.0, 530.0, 414.0, 295.0, 230.0, 444.0, 670.0, 720.0, 744.0,
    ],
    [
        744.0, 661.0, 725.0, 654.0, 491.0, 326.0, 219.0, 172.0, 351.0, 618.0, 720.0, 744.0,
    ],
    [
        744.0, 652.0, 716.0, 637.0, 443.0, 245.0, 141.0, 119.0, 258.0, 588.0, 719.0, 744.0,
    ],
    [
        744.0, 637.0, 709.0, 618.0, 377.0, 174.0, 78.0, 86.0, 185.0, 566.0, 701.0, 744.0,
    ],
    [
        744.0, 629.0, 678.0, 573.0, 293.0, 107.0, 39.0, 64.0, 131.0, 524.0, 663.0, 743.0,
    ],
    [
        743.0, 600.0, 639.0, 514.0, 225.0, 66.0, 18.0, 37.0, 84.0, 464.0, 613.0, 724.0,
    ],
    [
        736.0, 557.0, 605.0, 448.0, 159.0, 42.0, 11.0, 20.0, 71.0, 402.0, 550.0, 711.0,
    ],
    [
        716.0, 533.0, 570.0, 386.0, 76.0, 35.0, 6.0, 10.0, 54.0, 319.0, 461.0, 700.0,
    ],
    [
        697.0, 518.0, 532.0, 316.0, 46.0, 23.0, 1.0, 5.0, 30.0, 254.0, 384.0, 662.0,
    ],
];

/// Table 10.6 (p. 379): t_C;zi,mi for ϑ_koelgrens ≥ 25 down to ≤ 14.
const TABLE_10_6: [[f64; 12]; 12] = [
    [
        0.0, 0.0, 0.0, 0.0, 61.0, 34.0, 71.0, 74.0, 17.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 88.0, 40.0, 91.0, 87.0, 29.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 104.0, 50.0, 111.0, 109.0, 40.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 116.0, 67.0, 143.0, 149.0, 63.0, 3.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 126.0, 86.0, 183.0, 199.0, 77.0, 9.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 12.0, 140.0, 119.0, 229.0, 269.0, 97.0, 14.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 22.0, 155.0, 159.0, 295.0, 353.0, 134.0, 28.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 4.0, 33.0, 183.0, 216.0, 376.0, 450.0, 198.0, 51.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 11.0, 54.0, 214.0, 306.0, 449.0, 514.0, 276.0, 74.0, 0.0, 0.0,
    ],
    [
        0.0, 11.0, 19.0, 66.0, 253.0, 394.0, 525.0, 572.0, 369.0, 126.0, 0.0, 0.0,
    ],
    [
        0.0, 20.0, 28.0, 83.0, 301.0, 475.0, 603.0, 625.0, 462.0, 156.0, 1.0, 0.0,
    ],
    [
        0.0, 35.0, 35.0, 102.0, 367.0, 546.0, 666.0, 658.0, 535.0, 178.0, 19.0, 0.0,
    ],
];

/// Steps 2-5 of 9.28 (p. 305) and steps 2-5 of 10.19 (p. 378): drop the months below 10 % of the largest need,
/// fit y = a + b·x by least squares and return the intercept with the temperature axis.
fn intercept(need: &[f64; 12], outdoor: &[f64; 12]) -> f64 {
    let max = need.iter().cloned().fold(0.0, f64::max);
    let points: Vec<(f64, f64)> = (0..12)
        .filter(|&m| need[m] >= 0.1 * max)
        .map(|m| (outdoor[m], need[m]))
        .collect();
    let n = points.len() as f64;
    let sx: f64 = points.iter().map(|p| p.0).sum();
    let sy: f64 = points.iter().map(|p| p.1).sum();
    let sxx: f64 = points.iter().map(|p| p.0 * p.0).sum();
    let sxy: f64 = points.iter().map(|p| p.0 * p.1).sum();
    let b = (n * sxy - sx * sy) / (n * sxx - sx * sx);
    let a = (sy - b * sx) / n;
    -a / b
}

/// 9.28 step 5: rounded, at most ϑ_int;set;H;stc, within the rows of table 9.15.
fn heating_limit(d: &Derived, stc: f64) -> i32 {
    (intercept(&d.heating_limit_need, &d.outdoor)
        .round()
        .min(stc) as i32)
        .clamp(8, 16)
}

/// 9.32a with 9.32b and 7.62/7.63 (p. 213, 308-309): t_H;zi,mi · f_H;red · f_H;red;pmp;op.
fn heating_hours(
    limit: i32,
    reduced_hours_day: f64,
    reduced_hours_weekend: f64,
    pump: f64,
) -> [f64; 12] {
    let f_day = reduced_hours_day * (7.0 - reduced_hours_weekend / 24.0) / (24.0 * 7.0);
    let f_weekend = reduced_hours_weekend / (24.0 * 7.0);
    let row = TABLE_9_15[(16 - limit) as usize];
    std::array::from_fn(|m| row[m] * (1.0 - f_day - f_weekend) * pump)
}

/// 9.31/9.32 (p. 306) with ϑ_H,e;ontw −10 °C: (ϑ_H,in, ϑ_H,out) per month.
fn supply_return(
    d: &Derived,
    set: f64,
    design: (f64, f64),
    limit: i32,
    increment: f64,
) -> [(f64, f64); 12] {
    let (supply, spread) = design;
    let limit = f64::from(limit);
    std::array::from_fn(|m| {
        let x = (d.outdoor[m] - limit) / (-10.0 - limit);
        let t_in = set.max(set + (supply - set) * x + increment);
        let t_out = set.max((set + (supply - spread - set) * x + increment).min(t_in));
        (t_in, t_out)
    })
}

/// 9.33/9.35 (p. 311-312) and 10.24/10.26 (p. 384-385): π / (ln(d_a/d_i)/(2λ) + 1/(h_a·d_a)).
fn psi(d_inner: f64, d_outer: f64, lambda: f64, h_a: f64) -> f64 {
    std::f64::consts::PI / ((d_outer / d_inner).ln() / (2.0 * lambda) + 1.0 / (h_a * d_outer))
}

/// 9.27a/9.27b and 10.22a/10.22b: L + (0,15 or 0,03)/Ψ · L.
fn with_fittings(length: f64, psi: f64, insulated: bool) -> f64 {
    length + if insulated { 0.03 } else { 0.15 } / psi * length
}

/// 9.37 and 10.27 (p. 312, 386).
fn max_length(storeys: f64, area: f64) -> f64 {
    35.0 + 6.0 * storeys + 0.13 * area / storeys
}

/// 9.43 (p. 315) and 10.30 (p. 388).
fn hydraulic_power(pressure_kpa: f64, flow: f64) -> f64 {
    (pressure_kpa * flow / 3600.0).max(0.01)
}

/// 9.48/9.49 (p. 320) and 10.34/10.35 (p. 391): f_e of a pump without label below 2,5 kW.
fn reference_factor(p_hydr: f64) -> f64 {
    (1.7 * p_hydr + 17.0 * (1.0 - (-0.3 * p_hydr * 1000.0).exp()) * 1e-3) / p_hydr
}

/// 9.46 and 10.38 with β = 1 and C_P1/C_P2 0,25/0,75 (tables 9.22, 10.13).
fn energy_factor(f_e: f64, eei: f64) -> f64 {
    f_e * (0.25 + 0.75 * 1.0) * eei / 0.25
}

struct HeatingPump {
    storeys: f64,
    area: f64,
    building_fraction: f64,
    emitter_kpa: f64,
    heat_meter: bool,
    generator_spread: f64,
    electric_power_kw: Option<f64>,
    eei: Option<f64>,
    balanced: bool,
}

struct PumpResult {
    max_length: f64,
    pressure: f64,
    flow: f64,
    hydraulic: f64,
    epsilon: f64,
    energy: [f64; 12],
}

/// 9.41-9.51 with tables 9.19-9.22 (p. 314-321).
fn heating_pump(
    d: &Derived,
    pump: &HeatingPump,
    design_spread: f64,
    hours: &[f64; 12],
    temperatures: &[(f64, f64); 12],
) -> PumpResult {
    let max_length = max_length(pump.storeys, pump.area);
    let generator = if design_spread <= 10.0 && pump.generator_spread <= 10.0 {
        1000.0 / design_spread.powi(2)
    } else if design_spread <= 10.0 {
        design_spread / pump.generator_spread * 4000.0 / design_spread.powi(2)
    } else {
        4000.0 / design_spread.powi(2)
    };
    let additional = pump.emitter_kpa + if pump.heat_meter { 10.0 } else { 0.0 } + generator;
    let pressure = (1.0 + 0.4) * 0.10 * max_length + additional;
    let need = &d.heating_limit_need;
    let worst = (0..12).fold(0, |best, m| if need[m] > need[best] { m } else { best });
    let spread = temperatures[worst].0 - temperatures[worst].1;
    let flow =
        need[worst] * 3600.0 / (hours[worst] * 4.2 * spread * 1000.0 * pump.building_fraction);
    let hydraulic = hydraulic_power(pressure, flow);
    let f_e = match pump.electric_power_kw {
        Some(power) => power / hydraulic,
        None => reference_factor(hydraulic),
    };
    let epsilon = energy_factor(f_e, pump.eei.unwrap_or(0.23));
    let f_hb = if pump.balanced { 1.0 } else { 1.15 };
    let energy =
        std::array::from_fn(|m| hydraulic * hours[m] * f_hb * epsilon * pump.building_fraction);
    PumpResult {
        max_length,
        pressure,
        flow,
        hydraulic,
        epsilon,
        energy,
    }
}

fn mean(temperatures: &[(f64, f64); 12]) -> [f64; 12] {
    std::array::from_fn(|m| (temperatures[m].0 + temperatures[m].1) / 2.0)
}

/// 10.19 steps 2-6 (p. 378): the cooling limit, within 14-25 °C.
fn cooling_limit(d: &Derived) -> i32 {
    (intercept(&d.cooling_need, &d.outdoor).round() as i32).clamp(14, 25)
}

fn cooling_hours(limit: i32) -> [f64; 12] {
    TABLE_10_6[(25 - limit) as usize]
}

/// 10.15/10.16 (p. 376) with ϑ_C;int;ini 24 °C (10.10) and Δϑ_e,sol 8 K for a dwelling (table 10.5a).
fn cooling_emission_loss(d: &Derived, increment: f64, solar: f64) -> [f64; 12] {
    let inc = 24.0 + increment;
    std::array::from_fn(|m| {
        let comb = d.outdoor[m] + solar;
        if d.cooling_need[m] > 0.0 && inc - comb < 0.0 {
            d.cooling_need[m] * (increment / (inc - comb)).max(0.15)
        } else {
            0.0
        }
    })
}

struct CoolingPump {
    storeys: f64,
    area: f64,
    building_fraction: f64,
    emitter_kpa: f64,
    heat_meter: bool,
    label_kw: Option<f64>,
    eei: Option<f64>,
    balanced: bool,
    dwelling: bool,
}

/// 10.28-10.38 with tables 10.10-10.13 (p. 387-394); no pump energy in a month without cooling need.
fn cooling_pump(
    d: &Derived,
    pump: &CoolingPump,
    design: (f64, f64),
    hours: &[f64; 12],
) -> [f64; 12] {
    let (t_in, t_out) = design;
    let additional = pump.emitter_kpa
        + if pump.heat_meter { 10.0 } else { 0.0 }
        + 360.0 * (1.0 / (t_out - t_in)).powi(2);
    let pressure = (1.0 + 0.4) * 0.1 * max_length(pump.storeys, pump.area) + additional;
    let need = &d.cooling_need;
    let worst = (0..12).fold(0, |best, m| if need[m] > need[best] { m } else { best });
    let flow = need[worst] * 3600.0
        / (hours[worst] * 4.2 * (t_out - t_in) * 1000.0 * pump.building_fraction);
    let hydraulic = hydraulic_power(pressure, flow);
    let f_e = match pump.label_kw {
        Some(power) => power / hydraulic,
        None => reference_factor(hydraulic),
    };
    let epsilon = energy_factor(f_e, pump.eei.unwrap_or(0.23));
    let f_corr = if pump.balanced { 1.0 } else { 1.15 } * if pump.dwelling { 0.10 } else { 1.0 };
    std::array::from_fn(|m| {
        if need[m] > 0.0 {
            hydraulic * hours[m] * f_corr * epsilon * pump.building_fraction
        } else {
            0.0
        }
    })
}

/// 10.21 (p. 381) with L_C;zi = 0: −Ψ_j·(ϑ_C,mean − ϑ_C,amb;j)·L_j·t_C/1000·f_gebouw;C in months with a need.
fn cooling_distribution_loss(
    d: &Derived,
    psi: f64,
    medium: f64,
    ambient: f64,
    length: f64,
    hours: &[f64; 12],
    building_fraction: f64,
) -> [f64; 12] {
    std::array::from_fn(|m| {
        if d.cooling_need[m] > 0.0 {
            -psi * (medium - ambient) * length * hours[m] / 1000.0 * building_fraction
        } else {
            0.0
        }
    })
}

fn add(a: &[f64; 12], b: &[f64; 12]) -> [f64; 12] {
    std::array::from_fn(|m| a[m] + b[m])
}

fn scale(a: &[f64; 12], factor: f64) -> [f64; 12] {
    std::array::from_fn(|m| a[m] * factor)
}

// ---------------------------------------------------------------------------------------------------------------
// Comparison helpers.
// ---------------------------------------------------------------------------------------------------------------

fn series(p: &Value, label: &str, pointer: impl Fn(usize) -> String, expected: &[f64; 12]) {
    for (month, want) in expected.iter().enumerate() {
        close(
            &format!("{label} m{}", month + 1),
            at(p, &pointer(month)),
            *want,
        );
    }
}

fn heating(field: &'static str) -> impl Fn(usize) -> String {
    move |m| format!("/spaceHeating/monthly/{m}/{field}")
}

fn cooling(field: &'static str) -> impl Fn(usize) -> String {
    move |m| format!("/cooling/months/{m}/{field}")
}

/// Joins JSON edit arrays.
fn edits(parts: &[&str]) -> String {
    let mut all = Vec::new();
    for part in parts {
        let value: Value = serde_json::from_str(part).expect("edits");
        all.extend(value.as_array().expect("array").iter().cloned());
    }
    Value::Array(all).to_string()
}

const LABELLED_PUMP: &str = r#"[{"pointer": "/ntaCalculation/distributionSystem/pump", "value": {"method": "calculated",
    "heatMeterPresent": false, "electricPowerKw": 0.05, "energyEfficiencyIndex": 0.3, "sourceReference": "r77 synthetic"}}]"#;

const COLLECTIVE: &str = r#"[
    {"pointer": "/ntaCalculation/emission/system", "value": "radiators_or_convectors"},
    {"pointer": "/ntaCalculation/distributionSystem/designTemperatureClass", "value": "55_47"},
    {"pointer": "/ntaCalculation/distributionSystem/installation", "value": "collective"},
    {"pointer": "/ntaCalculation/distributionSystem/connectedStoreys", "value": 4},
    {"pointer": "/ntaCalculation/distributionSystem/pipeTransmittance", "value": {"method": "insulated_in_air",
        "pipeOuterDiameterM": 0.022, "insulatedDiameterM": 0.062, "insulationLambdaWPerMK": 0.035}},
    {"pointer": "/ntaCalculation/distributionSystem/unheatedPipeTransmittance", "value": {"method": "uninsulated",
        "innerDiameterM": 0.02, "outerDiameterM": 0.022, "pipeLambdaWPerMK": 50}},
    {"op": "remove", "pointer": "/ntaCalculation/distributionSystem/unheatedPipeLengthM"},
    {"pointer": "/ntaCalculation/distributionSystem/unheatedAmbientC", "value": [8, 8, 9, 11, 14, 16, 17, 17, 15, 12, 10, 8]},
    {"pointer": "/ntaCalculation/distributionSystem/pump", "value": {"method": "calculated", "heatMeterPresent": true,
        "sourceReference": "r77 synthetic"}},
    {"pointer": "/ntaCalculation/collectiveConnection", "value": {"connectedUsableAreaM2": 2000, "sourceReference": "r77 synthetic"}},
    {"pointer": "/ntaCalculation/generator/boiler/role", "value": "collective"},
    {"pointer": "/ntaCalculation/generator/boiler/location", "value": "outside_thermal_boundary"},
    {"pointer": "/ntaCalculation/generator/auxiliary", "value": {"electricallyConnectedDevices": 1, "nominalPowerKw": 30,
        "sourceReference": "r77 synthetic"}}]"#;

const UNHEATED_AMBIENT: [f64; 12] = [
    8.0, 8.0, 9.0, 11.0, 14.0, 16.0, 17.0, 17.0, 15.0, 12.0, 10.0, 8.0,
];

fn buffer(in_heated_space: bool) -> String {
    format!(
        r#"[{{"pointer": "/ntaCalculation/distributionSystem/bufferVessel", "value": {{"volumeL": 500, "producedFrom2018": true,
        "inHeatedSpace": {in_heated_space}, "sourceReference": "r77 synthetic"}}}}]"#
    )
}

/// Residential zone of EP-W001 (A_g 96 m²): ϑ_int;set;H;stc 20 °C (table 7.13), table 7.15 10 h/day and 0 h/week.
const AREA: f64 = 96.0;

/// Distribution of EP-W202b (20 m in an unheated space, uninsulated valves) with an additional labelled pump
/// (50 W, EEI 0,3). Heating limit 9.28 steps 2-5 (p. 304-305), table 9.15 and 9.32a/9.32b with table 7.15 and
/// table 9.X (p. 308-309), 9.30-9.32 with table 9.14 (45/40) and the emission increment of 9.12 (p. 306-307),
/// Ψ 0,3 of table 9.16, 9.27/9.27a (p. 303-304) and 9.26 with 13 °C in the unheated space (p. 302, 306); pump
/// 9.41-9.47 and 9.51 with tables 9.19-9.22 (p. 314-321), 9.39/9.40 with table 9.18 (p. 313-314).
#[test]
fn heating_distribution_in_an_unheated_space_with_a_labelled_pump() {
    let p = performance("EPW202b", LABELLED_PUMP);
    let d = derived(&p);
    let increment = at(&p, "/spaceHeating/emissionTemperatureIncrementK");
    let limit = heating_limit(&d, 20.0);
    let hours = heating_hours(limit, 10.0, 0.0, 0.10);
    let temperatures = supply_return(&d, 20.0, (45.0, 5.0), limit, increment);
    let medium = mean(&temperatures);
    let unheated = with_fittings(20.0, 0.3, false);
    let loss: [f64; 12] =
        std::array::from_fn(|m| 0.3 * (medium[m] - 13.0) * unheated * hours[m] / 1000.0);
    let pump = heating_pump(
        &d,
        &HeatingPump {
            storeys: 2.0,
            area: AREA,
            building_fraction: 1.0,
            emitter_kpa: 4.5,
            heat_meter: false,
            generator_spread: 20.0,
            electric_power_kw: Some(0.05),
            eei: Some(0.3),
            balanced: false,
        },
        5.0,
        &hours,
        &temperatures,
    );
    let s = "/spaceHeating/distribution";
    close(
        "heating limit",
        at(&p, &format!("{s}/zones/0/heatingLimitC")),
        f64::from(limit),
    );
    series(
        &p,
        "t_H,op",
        |m| format!("{s}/zones/0/operatingHours/{m}"),
        &hours,
    );
    series(
        &p,
        "theta_H,mean",
        |m| format!("{s}/zones/0/meanMediumTemperatureC/{m}"),
        &medium,
    );
    series(&p, "Q_H;dis;ls", heating("distributionLossKwh"), &loss);
    close(
        "L_max",
        at(&p, &format!("{s}/pump/maxPipeLengthM")),
        pump.max_length,
    );
    close(
        "delta p",
        at(&p, &format!("{s}/pump/pressureKpa")),
        pump.pressure,
    );
    close(
        "V",
        at(&p, &format!("{s}/pump/designFlowM3PerH")),
        pump.flow,
    );
    close(
        "P_hydr",
        at(&p, &format!("{s}/pump/hydraulicPowerKw")),
        pump.hydraulic,
    );
    close(
        "epsilon",
        at(&p, &format!("{s}/pump/energyFactor")),
        pump.epsilon,
    );
    series(
        &p,
        "W_H;aux;dis",
        heating("distributionAuxiliaryElectricityKwh"),
        &pump.energy,
    );
    series(
        &p,
        "Q_H;dis;aux;rvd",
        heating("distributionAuxiliaryToMediumKwh"),
        &scale(&pump.energy, 1.0 - 0.25),
    );
    series(
        &p,
        "Q_H;dis;aux;rbl",
        |m| format!("/spaceHeating/zoneRecoverableLosses/0/monthlyKwh/{m}"),
        &scale(&pump.energy, 0.25),
    );
}

/// A collective installation for 2 000 m² (f_gebouw 0,048) with calculated Ψ: 9.33 in the zone and 9.35 for the
/// uninsulated pipes in the unheated space (p. 311-312), forfait lengths 9.36/9.37 on A_g;gebouw;H and 15 % in the
/// unheated space (p. 303, 312), 9.27b, a given ϑ_ztu, 55/47 with radiators, f_H;red;pmp;op 1,0, and a pump
/// without label: 9.43-9.46, 9.48/9.49 with EEI 0,23 and table 9.21 with heat meter and generator (p. 315-320).
#[test]
fn collective_heating_distribution_with_calculated_psi_and_reference_pump() {
    let p = performance("EPW001", COLLECTIVE);
    let d = derived(&p);
    let increment = at(&p, "/spaceHeating/emissionTemperatureIncrementK");
    let limit = heating_limit(&d, 20.0);
    let hours = heating_hours(limit, 10.0, 0.0, 1.0);
    let temperatures = supply_return(&d, 20.0, (55.0, 8.0), limit, increment);
    let medium = mean(&temperatures);
    let fraction = AREA / 2000.0;
    let psi_zone = psi(0.022, 0.062, 0.035, 8.0);
    let psi_unheated = psi(0.020, 0.022, 50.0, 8.0);
    let length = 0.64 * 2000.0;
    let unheated = with_fittings(0.15 * length, psi_unheated, true);
    let loss: [f64; 12] = std::array::from_fn(|m| {
        psi_unheated * (medium[m] - UNHEATED_AMBIENT[m]) * unheated * hours[m] / 1000.0 * fraction
    });
    let pump = heating_pump(
        &d,
        &HeatingPump {
            storeys: 4.0,
            area: 2000.0,
            building_fraction: fraction,
            emitter_kpa: 2.0,
            heat_meter: true,
            generator_spread: 20.0,
            electric_power_kw: None,
            eei: None,
            balanced: false,
        },
        8.0,
        &hours,
        &temperatures,
    );
    let s = "/spaceHeating/distribution";
    close("psi 9.33", at(&p, &format!("{s}/psiZoneWPerMk")), psi_zone);
    close(
        "psi 9.35",
        at(&p, &format!("{s}/psiUnheatedWPerMk")),
        psi_unheated,
    );
    close("L_si 9.36", at(&p, &format!("{s}/pipeLengthM")), length);
    close(
        "L_si;j 15 %",
        at(&p, &format!("{s}/unheatedPipeLengthM")),
        0.15 * length,
    );
    close(
        "f_gebouw",
        at(&p, &format!("{s}/buildingFraction")),
        fraction,
    );
    series(
        &p,
        "t_H,op",
        |m| format!("{s}/zones/0/operatingHours/{m}"),
        &hours,
    );
    series(
        &p,
        "theta_H,mean",
        |m| format!("{s}/zones/0/meanMediumTemperatureC/{m}"),
        &medium,
    );
    series(&p, "Q_H;dis;ls", heating("distributionLossKwh"), &loss);
    close(
        "L_max",
        at(&p, &format!("{s}/pump/maxPipeLengthM")),
        pump.max_length,
    );
    close(
        "delta p",
        at(&p, &format!("{s}/pump/pressureKpa")),
        pump.pressure,
    );
    close(
        "V",
        at(&p, &format!("{s}/pump/designFlowM3PerH")),
        pump.flow,
    );
    close(
        "P_hydr",
        at(&p, &format!("{s}/pump/hydraulicPowerKw")),
        pump.hydraulic,
    );
    close(
        "epsilon",
        at(&p, &format!("{s}/pump/energyFactor")),
        pump.epsilon,
    );
    series(
        &p,
        "W_H;aux;dis",
        heating("distributionAuxiliaryElectricityKwh"),
        &pump.energy,
    );
}

/// Heating pipes that also carry hot water (p. 303): the in-zone length counts, t_H,op is the whole month and the
/// zone ambient is ϑ_int;op;H = ϑ_int;calc;H (7.9.6, p. 221); 9.38 (p. 313) makes the in-zone loss recoverable.
#[test]
fn heating_pipes_shared_with_hot_water() {
    let p = performance(
        "EPW001",
        r#"[{"pointer": "/ntaCalculation/distributionSystem/pipesAlsoForHotWater", "value": true}]"#,
    );
    let d = derived(&p);
    let increment = at(&p, "/spaceHeating/emissionTemperatureIncrementK");
    let limit = heating_limit(&d, 20.0);
    let medium = mean(&supply_return(&d, 20.0, (45.0, 5.0), limit, increment));
    let in_zone = with_fittings(0.64 * AREA, 0.3, true);
    let loss: [f64; 12] = std::array::from_fn(|m| {
        0.3 * (medium[m] - d.calculation_temperature[m]) * in_zone * d.hours[m] / 1000.0
    });
    series(&p, "Q_H;dis;ls", heating("distributionLossKwh"), &loss);
    series(
        &p,
        "Q_H;dis;ls;rbl",
        |m| format!("/spaceHeating/zoneRecoverableLosses/0/monthlyKwh/{m}"),
        &loss,
    );
}

/// EP-W201c (six fan convectors, 10 W each, table 9.11) with two indirect air heaters with warm-air return and an
/// unknown motor (table 9.13, highest value 0,008·Q_h;b): 9.21-9.23 with t_H,op of 9.32a (p. 298-301) and the
/// Q_h;b estimate of p. 300 (ϑ_int;calc;H read as the printed 21 °C, Q_H;ht of January from 7.10).
#[test]
fn emission_fans_and_indirect_air_heaters() {
    let p = performance(
        "EPW201c",
        r#"[{"pointer": "/ntaCalculation/emission/airHeaters", "value": {"kind": {"kind": "indirect",
            "roomHeightAbove8M": false, "warmAirReturn": true}, "count": 2, "sourceReference": "r77 synthetic"}}]"#,
    );
    let d = derived(&p);
    let limit = heating_limit(&d, 20.0);
    let hours = heating_hours(limit, 10.0, 0.0, 0.10);
    let transmission =
        d.heat_transfer[0] / (0.001 * d.hours[0]) * ((21.0 + 10.0) / (21.0 - d.outdoor[0]));
    let warm_up = (AREA.sqrt() * 4.0 * 3.0 + AREA) * 5.0;
    let design_load = transmission + warm_up;
    let energy: [f64; 12] =
        std::array::from_fn(|m| (10.0 * 6.0 * hours[m] + 0.008 * design_load * hours[m]) / 1000.0);
    series(
        &p,
        "W_H;em;aux",
        heating("emissionFanElectricityKwh"),
        &energy,
    );
}

/// 9.2.3.5 (p. 291-292) with 13.58 (p. 567): f_sto;bac;acc 1,0, H_sto;ls = S_sto;ls/45 with label C (table 13.9,
/// p. 570), ϑ_sto;set the highest ϑ_H,in of 9.31, t = t_H,op of 9.32a, scaled by f_gebouw.
fn buffer_loss(p: &Value, ambient: &[f64; 12]) -> [f64; 12] {
    let d = derived(p);
    let increment = at(p, "/spaceHeating/emissionTemperatureIncrementK");
    let limit = heating_limit(&d, 20.0);
    let hours = heating_hours(limit, 10.0, 0.0, 1.0);
    let temperatures = supply_return(&d, 20.0, (55.0, 8.0), limit, increment);
    let transfer = (14.33 + 7.13 * 500f64.powf(0.4)) / 45.0;
    std::array::from_fn(|m| {
        (AREA / 2000.0 * 1.0 * hours[m] / 1000.0 * transfer * (temperatures[m].0 - ambient[m]))
            .max(0.0)
    })
}

/// Collective buffer vessel in an unheated space at the given ϑ_ztu.
#[test]
fn collective_buffer_vessel_in_an_unheated_space() {
    let p = performance("EPW001", &edits(&[COLLECTIVE, &buffer(false)]));
    series(
        &p,
        "Q_H;nod;ls",
        heating("nodeLossKwh"),
        &buffer_loss(&p, &UNHEATED_AMBIENT),
    );
}

/// The same vessel in a heated space: p. 292 sets ϑ_sto;amb to ϑ_int;set;H;zi,mi of 7.9.4 (the levelled setpoint
/// of 7.76); the kernel takes ϑ_int;op;H (the calculation temperature of 7.9.2).
#[test]

fn collective_buffer_vessel_in_a_heated_space() {
    let p = performance("EPW001", &edits(&[COLLECTIVE, &buffer(true)]));
    let d = derived(&p);
    series(
        &p,
        "Q_H;nod;ls",
        heating("nodeLossKwh"),
        &buffer_loss(&p, &d.levelled_setpoint),
    );
}

const COOLING_FAN_COILS: &str = r#"[
    {"pointer": "/ntaCalculation/cooling/emission/emitter", "value": "fan_coil_or_rac_on_outer_wall"},
    {"pointer": "/ntaCalculation/cooling/emission/fanCoilCount", "value": 3},
    {"pointer": "/ntaCalculation/cooling/distribution/designTemperature", "value": "t6_to12_or_unknown"}]"#;

/// Δϑ_int;inc (10.11, p. 374-375) of fan convectors on the outer wall (table 10.35 −1,4), no balancing (table 10.4
/// −0,6) and unknown control (table 10.5 −2,5).
const FAN_COIL_INCREMENT: f64 = -1.4 - 0.6 - 2.5;

/// Table 10.8 (p. 380) with 10.23c (p. 384) read as the mean of the in and out columns.
fn cooling_medium(design: (f64, f64), increment: f64) -> f64 {
    ((design.0 - increment) + (design.1 - increment)) / 2.0
}

/// 10.87 (p. 425): 0,010 kW in every hour of the month.
fn control(d: &Derived) -> [f64; 12] {
    scale(&d.hours, 0.010)
}

/// EP-W301a with three fan convectors on the outer wall, 6/12, 15 m uninsulated pipe in an unconditioned space at
/// 19 °C (10.23b) and a heat meter: cooling limit 10.19 steps 2-6 (p. 377-378) with table 10.6, 10.10/10.11,
/// 10.15/10.16 with table 10.5a (p. 374-376), 10.17/10.18 with table 10.7 (p. 376-379), 10.21/10.22/10.22a with
/// table 10.9 (p. 381-386), the pump 10.28-10.32, 10.34/10.35, 10.37/10.38 with tables 10.10-10.13
/// (p. 387-393), 10.45 with table 10.14 (p. 394) and 10.87 (p. 425).
#[test]
fn cooling_water_distribution_in_a_dwelling() {
    let p = performance(
        "EPW301a",
        &edits(&[
            COOLING_FAN_COILS,
            r#"[{"pointer": "/ntaCalculation/cooling/distribution/pipe", "value": {"kind": "uninsulated"}},
            {"pointer": "/ntaCalculation/cooling/distribution/fittingsInsulated", "value": false},
            {"pointer": "/ntaCalculation/cooling/distribution/unconditionedPipeLengthM", "value": 15},
            {"pointer": "/ntaCalculation/cooling/distribution/pump/heatMeter", "value": true}]"#,
        ]),
    );
    let d = derived(&p);
    let limit = cooling_limit(&d);
    let hours = cooling_hours(limit);
    let emission = cooling_emission_loss(&d, FAN_COIL_INCREMENT, 8.0);
    let fans = scale(&hours, 10.0 * 3.0 / 1000.0);
    let medium = cooling_medium((6.0, 12.0), FAN_COIL_INCREMENT);
    // Table 10.9: uninsulated, A_g ≤ 200 m²: 1,0.
    let length = with_fittings(15.0, 1.0, false);
    let loss = cooling_distribution_loss(&d, 1.0, medium, 19.0, length, &hours, 1.0);
    let pump = cooling_pump(
        &d,
        &CoolingPump {
            storeys: 2.0,
            area: AREA,
            building_fraction: 1.0,
            emitter_kpa: 2.0,
            heat_meter: true,
            label_kw: None,
            eei: None,
            balanced: false,
            dwelling: true,
        },
        (6.0, 12.0),
        &hours,
    );
    close(
        "cooling limit",
        at(&p, "/cooling/coolingLimitC"),
        f64::from(limit),
    );
    close(
        "delta theta_int;inc",
        at(&p, "/cooling/internalTemperatureShiftK"),
        FAN_COIL_INCREMENT,
    );
    series(&p, "t_C", cooling("operatingHours"), &hours);
    series(&p, "Q_C;em;ls", cooling("emissionLossKwh"), &emission);
    series(&p, "Q_C;dis;ls", cooling("distributionLossKwh"), &loss);
    series(
        &p,
        "Q_C;dis;rvd",
        cooling("pumpRecoveredKwh"),
        &scale(&pump, 1.0 - 0.10),
    );
    series(
        &p,
        "W_C;aux",
        cooling("auxiliaryElectricityKwh"),
        &add(&add(&fans, &pump), &control(&d)),
    );
}

const COLLECTIVE_FREE_COOLING: &str = r#"[
    {"pointer": "/ntaCalculation/cooling/collective", "value": {"buildingUsableFloorAreaM2": 3000, "sourceReference": "r77 synthetic"}},
    {"pointer": "/ntaCalculation/cooling/distribution/pipe", "value": {"kind": "calculated", "geometry": {"method": "insulated_in_air",
        "pipeOuterDiameterM": 0.028, "insulatedDiameterM": 0.068, "insulationLambdaWPerMK": 0.04}, "sourceReference": "r77 synthetic"}},
    {"pointer": "/ntaCalculation/cooling/distribution/fittingsInsulated", "value": true},
    {"pointer": "/ntaCalculation/cooling/distribution/pipeLengthM", "value": 1500},
    {"op": "remove", "pointer": "/ntaCalculation/cooling/distribution/unconditionedPipeLengthM"},
    {"pointer": "/ntaCalculation/cooling/distribution/unconditionedAmbientC", "value": 22},
    {"pointer": "/ntaCalculation/cooling/distribution/pump", "value": {"hydraulicallyBalanced": true, "floorCount": 5,
        "heatMeter": false, "individualDwellingInstallation": false, "sourceReference": "r77 synthetic"}},
    {"pointer": "/ntaCalculation/cooling/generators", "value": [
        {"id": "wko", "generator": {"kind": "free_cooling", "source": "aquifer_from2013"}, "capacityKw": 30, "equipmentReference": "r77 synthetic"},
        {"id": "chiller", "generator": {"kind": "compression", "heatRejection": "closed_cooling_tower"}, "capacityKw": 70,
            "equipmentReference": "r77 synthetic"}]}]"#;

/// Free cooling (30 kW, aquifer from 2013) before a 70 kW chiller with a closed cooling tower, collective for
/// 3 000 m²: returns (cold delivered to the generators per 10.8, free-cooling cold, chiller electricity 10.76,
/// free-cooling energy 10.86, the other auxiliary energy of 10.79).
fn free_cooling_case(p: &Value) -> [[f64; 12]; 5] {
    let d = derived(p);
    let limit = cooling_limit(&d);
    let hours = cooling_hours(limit);
    let fraction = AREA / 3000.0;
    let emission = cooling_emission_loss(&d, FAN_COIL_INCREMENT, 8.0);
    let fans = scale(&hours, 10.0 * 3.0 / 1000.0);
    let pipe = psi(0.028, 0.068, 0.04, 8.0);
    let length = with_fittings(0.15 * 1500.0, pipe, true);
    let medium = cooling_medium((6.0, 12.0), FAN_COIL_INCREMENT);
    let loss = cooling_distribution_loss(&d, pipe, medium, 22.0, length, &hours, fraction);
    let pump = cooling_pump(
        &d,
        &CoolingPump {
            storeys: 5.0,
            area: 3000.0,
            building_fraction: fraction,
            emitter_kpa: 2.0,
            heat_meter: false,
            label_kw: None,
            eei: None,
            balanced: true,
            dwelling: false,
        },
        (6.0, 12.0),
        &hours,
    );
    // Dehumidification of chapter 12 (fan convectors) is derived input here.
    let dehumidification = months(p, |m| format!("/cooling/months/{m}/dehumidificationKwh"));
    let generated: [f64; 12] = std::array::from_fn(|m| {
        d.cooling_need[m] + emission[m] + loss[m] + (1.0 - 0.10) * pump[m] + dehumidification[m]
    });
    // 10.49/10.50: β = 30/100 = 0,3 (rounded up to one decimal); table 10.16 gives 0,68 (July-September) and
    // 0,93 (other months); 10.51 leaves the rest to the chiller.
    let free: [f64; 12] =
        std::array::from_fn(|m| generated[m] * if (6..=8).contains(&m) { 0.68 } else { 0.93 });
    let chiller = add(&generated, &scale(&free, -1.0));
    // Table 10.29 EER 3,00 with f_prpr 1 (10.76); table 10.34 EER_fc 23 (10.86); 10.80 without f_C;PL and f_EER;corr
    // in method 3 and table 10.33 0,033 kW/kW (10.83).
    let chiller_electricity = scale(&chiller, 1.0 / 3.0);
    let free_energy = scale(&free, 1.0 / 23.0);
    let condenser = scale(&chiller, (1.0 + 1.0 / 3.0) * 0.033);
    let other_auxiliary = add(&add(&add(&fans, &pump), &control(&d)), &condenser);
    [
        generated,
        free,
        chiller_electricity,
        free_energy,
        other_auxiliary,
    ]
}

/// A collective cooling installation for 3 000 m² (f_gebouw;C 0,032) with free cooling from an aquifer (table 10.15
/// priority 1) before a chiller: β of 10.49/10.50 and the fractions 10.51 of table 10.16 (p. 400-401), 10.52, 10.76
/// with table 10.29 and f_prpr 1 (p. 417-419), 10.86 with table 10.34 (p. 423-424), 10.80 and 10.83 with table
/// 10.33 (p. 420-422), Ψ of 10.24 with 15 % of the actual length unconditioned (p. 382-384), and the pump above
/// its 0,01 kW floor (10.30, 10.32).
#[test]
fn collective_cooling_with_free_cooling_before_a_chiller() {
    let p = performance(
        "EPW301a",
        &edits(&[COOLING_FAN_COILS, COLLECTIVE_FREE_COOLING]),
    );
    let [generated, free, chiller, free_energy, other] = free_cooling_case(&p);
    close("beta", at(&p, "/cooling/generatorShares/0/beta"), 0.3);
    close(
        "f jul-sep",
        at(&p, "/cooling/generatorShares/0/shareJulyToSeptember"),
        0.68,
    );
    close(
        "f other",
        at(&p, "/cooling/generatorShares/0/shareOtherMonths"),
        0.93,
    );
    series(&p, "Q_C;gen;in", cooling("generatorColdKwh"), &generated);
    series(&p, "free cooling", cooling("ambientColdKwh"), &free);
    // Electricity of the cooling function in total (generator and auxiliary energy together).
    let total = add(&add(&chiller, &free_energy), &other);
    for (month, want) in total.iter().enumerate() {
        let kernel = at(&p, &format!("/cooling/months/{month}/electricityKwh"))
            + at(
                &p,
                &format!("/cooling/months/{month}/auxiliaryElectricityKwh"),
            );
        close(&format!("E_C;el + W_C;aux m{}", month + 1), kernel, *want);
    }
}

/// 10.79 (p. 420) books W_fc;el;in of 10.86 as auxiliary energy of the generation; 10.76 only covers compression
/// machines. The kernel adds the free-cooling pump energy to E_C;el instead (the total is the same).
#[test]

fn free_cooling_energy_is_auxiliary_energy() {
    let p = performance(
        "EPW301a",
        &edits(&[COOLING_FAN_COILS, COLLECTIVE_FREE_COOLING]),
    );
    let [_, _, chiller, free_energy, other] = free_cooling_case(&p);
    series(&p, "E_C;el", cooling("electricityKwh"), &chiller);
    series(
        &p,
        "W_C;aux",
        cooling("auxiliaryElectricityKwh"),
        &add(&other, &free_energy),
    );
}

/// EP-W301a (floor cooling 17/21; Δϑ_int;inc −1,7 −0,6 −2,5) with 10 m insulated pipe (table 10.9 0,3) in an
/// unconditioned space at 19 °C and a labelled pump (40 W, EEI 0,27, 10.33, p. 391). Table 10.8 read literally gives
/// a medium of 23,8 °C, above the ambient, so 10.21 gives a negative loss; the kernel keeps the printed sign.
#[test]
fn floor_cooling_with_a_labelled_pump() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/distribution/unconditionedPipeLengthM", "value": 10},
        {"pointer": "/ntaCalculation/cooling/distribution/pump/labelPowerKw", "value": 0.04},
        {"pointer": "/ntaCalculation/cooling/distribution/pump/energyEfficiencyIndex", "value": 0.27}]"#,
    );
    let d = derived(&p);
    let hours = cooling_hours(cooling_limit(&d));
    let increment = -1.7 - 0.6 - 2.5;
    let medium = cooling_medium((17.0, 21.0), increment);
    let length = with_fittings(10.0, 0.3, true);
    let loss = cooling_distribution_loss(&d, 0.3, medium, 19.0, length, &hours, 1.0);
    let pump = cooling_pump(
        &d,
        &CoolingPump {
            storeys: 2.0,
            area: AREA,
            building_fraction: 1.0,
            emitter_kpa: 4.5,
            heat_meter: false,
            label_kw: Some(0.04),
            eei: Some(0.27),
            balanced: false,
            dwelling: true,
        },
        (17.0, 21.0),
        &hours,
    );
    series(&p, "Q_C;dis;ls", cooling("distributionLossKwh"), &loss);
    series(
        &p,
        "Q_C;dis;rvd",
        cooling("pumpRecoveredKwh"),
        &scale(&pump, 1.0 - 0.10),
    );
    series(
        &p,
        "W_C;aux",
        cooling("auxiliaryElectricityKwh"),
        &add(&pump, &control(&d)),
    );
}

/// 9.34 (p. 311) and 10.25 (p. 385): π / (½·(ln(d_a/d_i)/λ_D + ln(4z/d_a)/λ_em)).
fn psi_embedded(d_inner: f64, d_outer: f64, lambda: f64, lambda_embedding: f64, depth: f64) -> f64 {
    std::f64::consts::PI
        / (0.5
            * ((d_outer / d_inner).ln() / lambda + (4.0 * depth / d_outer).ln() / lambda_embedding))
}

/// Embedded insulated pipes in the unheated part of EP-W202b (heating, 9.34 in 9.26) and in the unconditioned
/// part of EP-W301a (cooling, 10.25 in 10.21).
#[test]
fn embedded_pipes_in_heating_and_cooling() {
    let embedded = psi_embedded(0.02, 0.04, 0.04, 2.0, 0.03);
    let p = performance(
        "EPW202b",
        r#"[{"pointer": "/ntaCalculation/distributionSystem/unheatedPipeTransmittance", "value": {"method": "insulated_embedded",
            "pipeOuterDiameterM": 0.02, "insulatedDiameterM": 0.04, "insulationLambdaWPerMK": 0.04,
            "embeddingLambdaWPerMK": 2.0, "depthM": 0.03}}]"#,
    );
    close(
        "psi 9.34",
        at(&p, "/spaceHeating/distribution/psiUnheatedWPerMk"),
        embedded,
    );
    let d = derived(&p);
    let increment = at(&p, "/spaceHeating/emissionTemperatureIncrementK");
    let limit = heating_limit(&d, 20.0);
    let hours = heating_hours(limit, 10.0, 0.0, 0.10);
    let medium = mean(&supply_return(&d, 20.0, (45.0, 5.0), limit, increment));
    let length = with_fittings(20.0, embedded, false);
    let loss: [f64; 12] =
        std::array::from_fn(|m| embedded * (medium[m] - 13.0) * length * hours[m] / 1000.0);
    series(&p, "Q_H;dis;ls", heating("distributionLossKwh"), &loss);

    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/distribution/unconditionedPipeLengthM", "value": 10},
        {"pointer": "/ntaCalculation/cooling/distribution/unconditionedAmbientC", "value": 26},
        {"pointer": "/ntaCalculation/cooling/distribution/pipe", "value": {"kind": "calculated", "geometry": {"method": "insulated_embedded",
            "pipeOuterDiameterM": 0.02, "insulatedDiameterM": 0.04, "insulationLambdaWPerMK": 0.04,
            "embeddingLambdaWPerMK": 2.0, "depthM": 0.03}, "sourceReference": "r77 synthetic"}}]"#,
    );
    let d = derived(&p);
    let hours = cooling_hours(cooling_limit(&d));
    let medium = cooling_medium((17.0, 21.0), -1.7 - 0.6 - 2.5);
    let length = with_fittings(10.0, embedded, true);
    let loss = cooling_distribution_loss(&d, embedded, medium, 26.0, length, &hours, 1.0);
    series(&p, "Q_C;dis;ls", cooling("distributionLossKwh"), &loss);
}

/// A booster heat pump that extracts 60 kWh per month from the cooling distribution (Q_C;HP of 10.6, given): 10.7
/// and 10.8 lower Q_C;gen;in by that amount, 10.9 limits the extraction to the cooling demand (p. 371-373).
#[test]
fn booster_heat_pump_extraction_from_the_cooling_distribution() {
    let p = performance(
        "EPW301a",
        &edits(&[
            COOLING_FAN_COILS,
            r#"[{"pointer": "/ntaCalculation/cooling/distribution/pipe", "value": {"kind": "uninsulated"}},
            {"pointer": "/ntaCalculation/cooling/distribution/fittingsInsulated", "value": false},
            {"pointer": "/ntaCalculation/cooling/distribution/unconditionedPipeLengthM", "value": 15},
            {"pointer": "/ntaCalculation/cooling/distribution/pump/heatMeter", "value": true},
            {"pointer": "/ntaCalculation/cooling/boosterHeatPumpExtractionKwh", "value": [60, 60, 60, 60, 60, 60, 60, 60, 60, 60, 60, 60]}]"#,
        ]),
    );
    let d = derived(&p);
    let hours = cooling_hours(cooling_limit(&d));
    let emission = cooling_emission_loss(&d, FAN_COIL_INCREMENT, 8.0);
    let medium = cooling_medium((6.0, 12.0), FAN_COIL_INCREMENT);
    let loss = cooling_distribution_loss(
        &d,
        1.0,
        medium,
        19.0,
        with_fittings(15.0, 1.0, false),
        &hours,
        1.0,
    );
    let pump = cooling_pump(
        &d,
        &CoolingPump {
            storeys: 2.0,
            area: AREA,
            building_fraction: 1.0,
            emitter_kpa: 2.0,
            heat_meter: true,
            label_kw: None,
            eei: None,
            balanced: false,
            dwelling: true,
        },
        (6.0, 12.0),
        &hours,
    );
    let dehumidification = months(&p, |m| format!("/cooling/months/{m}/dehumidificationKwh"));
    let demand: [f64; 12] = std::array::from_fn(|m| {
        d.cooling_need[m] + emission[m] + loss[m] + (1.0 - 0.10) * pump[m] + dehumidification[m]
    });
    let generator: [f64; 12] = std::array::from_fn(|m| (demand[m] - 60.0).max(0.0));
    let extraction: [f64; 12] = std::array::from_fn(|m| demand[m].min(60.0));
    series(&p, "Q_C;gen;in", cooling("generatorColdKwh"), &generator);
    series(&p, "Q_C;HP", cooling("boosterExtractionKwh"), &extraction);
}

/// The floor cooling of EP-W301a on free cooling from a closed ground loop (table 10.34: 10) that is also the
/// source of the ground-source heat pump of EP-W203d: 10.84 compares the cold returned to the ground with
/// Σ(Q_H;gen;out − E_H;gen;in) of the heat pump and 10.85 adds 0,7·Σ(…) − ΣQ_hr over 10·ΣREG in April to
/// September (p. 423-424). The heat-pump heat and electricity are derived input (review 66). Without a
/// compression machine no 10.87 control energy is counted (kernel reading, question 2 to NEN).
#[test]
fn free_cooling_with_ground_regeneration_surcharge() {
    let p = performance(
        "EPW203d",
        r#"[{"pointer":"/ntaCalculation/activeCooling","value":{"system":"free_cooling_table10_34","capacity":{"method":"dynamic_cooling_load","sourceReference":"EP-W301a"},"sourceReference":"EP-W301a"}},{"pointer":"/ntaCalculation/cooling","value":{"emission":{"emitter":"floor_cooling","balancing":"none_or_unknown","control":"unknown_or_other","sourceReference":"EP-W301a emission"},"distribution":{"designTemperature":"t17_to21","pipe":{"kind":"insulated_from1995"},"fittingsInsulated":true,"pump":{"hydraulicallyBalanced":false,"floorCount":2,"heatMeter":false,"individualDwellingInstallation":true,"sourceReference":"EP-W301a pump"},"unconditionedPipeLengthM":0,"sourceReference":"EP-W301a distribution"},"generators":[{"id":"bodem","generator":{"kind":"free_cooling","source":"closed_ground_loop","heatPumpSource":true},"equipmentReference":"r77 synthetic"}]}}]"#,
    );
    let d = derived(&p);
    let hours = cooling_hours(cooling_limit(&d));
    let emission = cooling_emission_loss(&d, -1.7 - 0.6 - 2.5, 8.0);
    let pump = cooling_pump(
        &d,
        &CoolingPump {
            storeys: 2.0,
            area: AREA,
            building_fraction: 1.0,
            emitter_kpa: 4.5,
            heat_meter: false,
            label_kw: None,
            eei: None,
            balanced: false,
            dwelling: true,
        },
        (17.0, 21.0),
        &hours,
    );
    let returned: [f64; 12] =
        std::array::from_fn(|m| d.cooling_need[m] + emission[m] + (1.0 - 0.10) * pump[m]);
    let extracted: f64 = (0..12)
        .map(|m| {
            at(&p, &format!("/spaceHeating/monthly/{m}/heatPumpOutputKwh"))
                - at(
                    &p,
                    &format!("/spaceHeating/monthly/{m}/generatorElectricityKwh"),
                )
        })
        .sum();
    let returned_total: f64 = returned.iter().sum();
    assert!(returned_total / extracted < 0.7, "10.84 applies");
    let surcharge = (0.7 * extracted - returned_total) / (10.0 * 6.0);
    let free: [f64; 12] = std::array::from_fn(|m| {
        returned[m] / 10.0 + if (3..=8).contains(&m) { surcharge } else { 0.0 }
    });
    series(&p, "Q_C;gen;in", cooling("generatorColdKwh"), &returned);
    let total = add(&free, &pump);
    for (month, want) in total.iter().enumerate() {
        let kernel = at(&p, &format!("/cooling/months/{month}/electricityKwh"))
            + at(
                &p,
                &format!("/cooling/months/{month}/auxiliaryElectricityKwh"),
            );
        close(&format!("W_fc + W_C;aux;dis m{}", month + 1), kernel, *want);
    }
}

/// The water-cooled chiller of review 67 (method 2, NEN-EN 14511, on a dry cooler) on the floor cooling of EP-W301a:
/// 10.80 with EER_n·f_C;PL·f_EER;corr (the monthly EER of the kernel, recalculated in review 67, without f_prpr
/// 0,9), 10.82 with table 10.31 (dry, silencer unknown: 0,054) and table 10.32 (1), 10.83 with table 10.33
/// (0,045), 10.87 (p. 420-425).
#[test]
fn water_cooled_chiller_condenser_auxiliaries() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/generators/0/generator", "value": {"kind": "compression",
            "heatRejection": "dry_cooler", "performance": {"method": "en14511", "nominalEer": 4.5, "nominalCapacityKw": 2.0,
            "nominalEvaporatorOutletC": 7.0, "nominalCondenserInletC": 30.0, "sourceReference": "r67 synthetic"}}}]"#,
    );
    let d = derived(&p);
    let hours = cooling_hours(cooling_limit(&d));
    let pump = cooling_pump(
        &d,
        &CoolingPump {
            storeys: 2.0,
            area: AREA,
            building_fraction: 1.0,
            emitter_kpa: 4.5,
            heat_meter: false,
            label_kw: None,
            eei: None,
            balanced: false,
            dwelling: true,
        },
        (17.0, 21.0),
        &hours,
    );
    let generated = months(&p, |m| format!("/cooling/months/{m}/generatorColdKwh"));
    let condenser: [f64; 12] = std::array::from_fn(|m| {
        if generated[m] > 0.0 {
            let eer = at(&p, &format!("/cooling/generatorShares/0/monthlyEer/{m}")) / 0.9;
            generated[m] * (1.0 + 1.0 / eer) * (0.054 * 1.0 * 1.0 + 0.045)
        } else {
            0.0
        }
    });
    series(
        &p,
        "W_C;aux",
        cooling("auxiliaryElectricityKwh"),
        &add(&add(&pump, &condenser), &control(&d)),
    );
}

/// Three direct-fired air heaters with an unknown fan type (table 9.12, highest value 0,022·Q_h;b) in 9.23 (p. 299-300).
#[test]
fn direct_fired_air_heaters() {
    let p = performance(
        "EPW001",
        r#"[{"pointer": "/ntaCalculation/emission/airHeaters", "value": {"kind": {"kind": "direct"}, "count": 3,
            "sourceReference": "r77 synthetic"}}]"#,
    );
    let d = derived(&p);
    let hours = heating_hours(heating_limit(&d, 20.0), 10.0, 0.0, 0.10);
    let transmission =
        d.heat_transfer[0] / (0.001 * d.hours[0]) * ((21.0 + 10.0) / (21.0 - d.outdoor[0]));
    let design_load = transmission + (AREA.sqrt() * 4.0 * 3.0 + AREA) * 5.0;
    let energy: [f64; 12] = std::array::from_fn(|m| 0.022 * design_load * hours[m] / 1000.0);
    series(
        &p,
        "W_H;em;aux",
        heating("emissionFanElectricityKwh"),
        &energy,
    );
}

/// The collective installation of the second case on 100 000 m²: P_H;hydr;dis above 2,5 kW, so f_e follows 9.50
/// with b = 2 and EEI is 0,25 (p. 319-320).
#[test]
fn large_collective_pump_above_2_5_kw() {
    let p = performance(
        "EPW001",
        &edits(&[
            COLLECTIVE,
            r#"[{"pointer": "/ntaCalculation/collectiveConnection", "value": {"connectedUsableAreaM2": 100000,
                "sourceReference": "r77 synthetic"}}]"#,
        ]),
    );
    let d = derived(&p);
    let increment = at(&p, "/spaceHeating/emissionTemperatureIncrementK");
    let limit = heating_limit(&d, 20.0);
    let hours = heating_hours(limit, 10.0, 0.0, 1.0);
    let temperatures = supply_return(&d, 20.0, (55.0, 8.0), limit, increment);
    let fraction = AREA / 100000.0;
    let reference = heating_pump(
        &d,
        &HeatingPump {
            storeys: 4.0,
            area: 100000.0,
            building_fraction: fraction,
            emitter_kpa: 2.0,
            heat_meter: true,
            generator_spread: 20.0,
            electric_power_kw: None,
            eei: None,
            balanced: false,
        },
        8.0,
        &hours,
        &temperatures,
    );
    assert!(reference.hydraulic >= 2.5);
    // 9.50 and EEI 0,25 replace 9.48/9.49 and 0,23 above 2,5 kW.
    let f_e = (1.25 + (0.2 / reference.hydraulic).powf(0.5)) * 2.0;
    let epsilon = energy_factor(f_e, 0.25);
    let energy: [f64; 12] =
        std::array::from_fn(|m| reference.hydraulic * hours[m] * 1.15 * epsilon * fraction);
    let s = "/spaceHeating/distribution";
    close(
        "P_hydr",
        at(&p, &format!("{s}/pump/hydraulicPowerKw")),
        reference.hydraulic,
    );
    close(
        "epsilon",
        at(&p, &format!("{s}/pump/energyFactor")),
        epsilon,
    );
    series(
        &p,
        "W_H;aux;dis",
        heating("distributionAuxiliaryElectricityKwh"),
        &energy,
    );
}
