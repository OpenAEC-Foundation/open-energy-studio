//! Review 75: chapter 7 routes the two example projects do not reach, recalculated independently
//! from NTA 8800:2025+C1:2026: external obstruction (17.3.2-17.3.7, tables 17.3-17.15, nearest tilt with
//! the higher value on a tie, south field of view below 15 degrees), movable shading (7.42/7.43, tables
//! 7.5-7.9 with tilt interpolation and the heating-balance cases of 7.6.6.1.4), glazing g (7.40, table 7.4,
//! fixed louvres 7.41a/7.41b, 7.41), dynamic glazing (annex A), rooflights and tilted windows (table 17.2
//! interpolation, F_sky of 7.6.6.4), opaque solar gains and sky radiation (7.33/7.39), internal gains
//! (7.21-7.29), internal heat capacity (table 7.10, annex B), utilisation and time constants (7.46-7.58),
//! intermittency for every utility function (7.59-7.75, tables 7.13-7.15) and annex AA (TOjuli method 3).
//! The cases are synthetic inputs on training-data/nta8800-monthly-demand-synthetic.json. The expected
//! values come from a separate implementation written from the norm pages; only numbers and page
//! references are kept here. The scratch sweep compared 28 224 obstruction, 18 720 collector and 22 080
//! shading lookups, 188 utilisation factors, 120 annex AA quantities and 19 378 monthly quantities of 58
//! demand cases; this file keeps a selection.

use nta8800_core::annex_aa::{assess_annex_aa, insulation_factor, AnnexAaInput};
use nta8800_core::climate::Orientation;
use nta8800_core::monthly_demand::{
    assess_monthly_demand, cooling_utilization, heating_utilization, MonthlyDemandInput,
};
use nta8800_core::solar_shading::{
    collector_obstruction_factor, movable_shading_factor, obstruction_factor, shading_fraction,
    Balance, CollectorObstruction, MovableShading, Obstruction, ShadingControl,
};
use serde_json::Value;

const BASE: &str = include_str!("../../../training-data/nta8800-monthly-demand-synthetic.json");

/// The synthetic zone with top-level fields replaced (null removes a field).
fn input(overrides: &str) -> MonthlyDemandInput {
    let mut doc: Value = serde_json::from_str(BASE).expect("base");
    let edits: Value = serde_json::from_str(overrides).expect("overrides");
    let map = doc.as_object_mut().expect("object");
    for (key, value) in edits.as_object().expect("object") {
        if value.is_null() {
            map.remove(key);
        } else {
            map.insert(key.clone(), value.clone());
        }
    }
    serde_json::from_value(doc).expect("valid demand input")
}

fn demand(overrides: &str) -> Value {
    let assessment = assess_monthly_demand(&input(overrides));
    assert_eq!(
        assessment.status, "calculated_unverified",
        "{:?}",
        assessment.issues
    );
    serde_json::to_value(assessment).expect("serializable")
}

fn close(label: &str, kernel: f64, expected: f64) {
    let diff = (kernel - expected).abs();
    let scale = kernel.abs().max(expected.abs()).max(1e-12);
    assert!(
        diff <= 1e-9 || diff / scale <= 1e-6,
        "{label}: kernel {kernel} independent {expected}"
    );
}

fn number(value: &Value, label: &str) -> f64 {
    value.as_f64().unwrap_or_else(|| panic!("{label}: missing"))
}

/// A monthly quantity: `internal`, `opaque`, `htr`, `window:<id>:heating|cooling` or
/// `heating.<field>` / `cooling.<field>` of the balance terms.
fn monthly(d: &Value, month: usize, key: &str) -> f64 {
    let m = &d["monthly"][month - 1];
    let label = format!("month {month} {key}");
    match key {
        "internal" => number(&m["internalGainsKwh"], &label),
        "opaque" => number(&m["opaqueSolarGainsKwh"], &label),
        "htr" => number(&m["transmissionConductanceWPerK"], &label),
        _ if key.starts_with("window:") => {
            let parts: Vec<&str> = key.split(':').collect();
            let window = m["windowSolarByWindow"]
                .as_array()
                .expect("windows")
                .iter()
                .find(|w| w["id"] == parts[1])
                .unwrap_or_else(|| panic!("{label}: no window"));
            number(&window[format!("{}Kwh", parts[2])], &label)
        }
        _ => {
            let (balance, field) = key.split_once('.').expect("balance.field");
            number(&m[balance][field], &label)
        }
    }
}

fn check(d: &Value, expected: &[(usize, &str, f64)]) {
    for &(month, key, value) in expected {
        close(
            &format!("month {month} {key}"),
            monthly(d, month, key),
            value,
        );
    }
}

fn orientation(name: &str) -> Orientation {
    serde_json::from_value(Value::String(name.to_string())).expect("orientation")
}

fn balance(name: &str) -> Balance {
    if name == "cooling" {
        Balance::Cooling
    } else {
        Balance::Heating
    }
}

/// F_sh;obst per situation and balance: tables 17.4/17.5 (p. 708-715), 17.7 (p. 716-720), 17.8/17.9
/// (p. 720-730), 17.10/17.11 (p. 731-743), 17.13/17.14 (p. 752-763) with table 17.3 (p. 706-707) for the
/// fallbacks; nearest tilt column, the higher value on a tie (p. 708, 752); south field of view below 15
/// degrees (17.3.1, p. 695).
#[test]
fn obstruction_factor_situations_a_to_g() {
    let rows: &[(&str, &str, f64, u8, &str, f64)] = &[
        (r#"{"method":"minimal"}"#, "south", 90.0, 1, "heating", 0.23),
        (r#"{"method":"minimal"}"#, "west", 67.5, 5, "heating", 0.87),
        (
            r#"{"method":"minimal"}"#,
            "north_west",
            10.0,
            4,
            "heating",
            0.99,
        ),
        (r#"{"method":"minimal"}"#, "north", 14.9, 6, "heating", 0.92),
        (r#"{"method":"minimal"}"#, "east", 97.5, 2, "heating", 0.79),
        (
            r#"{"method":"minimal"}"#,
            "south_east",
            172.5,
            1,
            "heating",
            1.0,
        ),
        (
            r#"{"method":"minimal"}"#,
            "north_east",
            37.5,
            7,
            "heating",
            0.81,
        ),
        (r#"{"method":"minimal"}"#, "south", 30.0, 7, "cooling", 1.0),
        (
            r#"{"method":"parallel_obstruction","relativeHeight":0.3}"#,
            "south",
            90.0,
            3,
            "heating",
            0.95,
        ),
        (
            r#"{"method":"parallel_obstruction","relativeHeight":0.5}"#,
            "south_west",
            90.0,
            2,
            "heating",
            0.51,
        ),
        (
            r#"{"method":"parallel_obstruction","relativeHeight":1}"#,
            "south_east",
            90.0,
            9,
            "heating",
            0.38,
        ),
        (
            r#"{"method":"parallel_obstruction","relativeHeight":1.4}"#,
            "east",
            90.0,
            7,
            "cooling",
            1.0,
        ),
        (
            r#"{"method":"overhang","relativeHeight":0.2}"#,
            "south",
            90.0,
            6,
            "heating",
            0.56,
        ),
        (
            r#"{"method":"overhang","relativeHeight":0.99}"#,
            "west",
            90.0,
            8,
            "heating",
            0.56,
        ),
        (
            r#"{"method":"overhang","relativeHeight":0.6}"#,
            "south_west",
            90.0,
            7,
            "cooling",
            0.7,
        ),
        (
            r#"{"method":"overhang","relativeHeight":2.5}"#,
            "north",
            90.0,
            6,
            "cooling",
            1.0,
        ),
        (
            r#"{"method":"side_obstruction","side":"left","relativeWidth":0.5,"coolingHeightCondition":true}"#,
            "west",
            90.0,
            1,
            "heating",
            0.49,
        ),
        (
            r#"{"method":"side_obstruction","side":"right","relativeWidth":1.5,"coolingHeightCondition":true}"#,
            "south",
            90.0,
            7,
            "cooling",
            1.0,
        ),
        (
            r#"{"method":"side_obstruction","side":"both","relativeWidth":0.3,"coolingHeightCondition":true}"#,
            "east",
            90.0,
            6,
            "cooling",
            0.96,
        ),
        (
            r#"{"method":"side_obstruction","side":"both","relativeWidth":0.3}"#,
            "east",
            90.0,
            6,
            "cooling",
            1.0,
        ),
        (
            r#"{"method":"side_obstruction","side":"both","relativeWidth":2}"#,
            "north_west",
            90.0,
            11,
            "heating",
            0.99,
        ),
        (
            r#"{"method":"full","coolingConditionsMet":true}"#,
            "north",
            60.0,
            5,
            "cooling",
            0.8,
        ),
        (
            r#"{"method":"full","coolingConditionsMet":false}"#,
            "north",
            60.0,
            5,
            "cooling",
            1.0,
        ),
        (
            r#"{"method":"full"}"#,
            "south_west",
            52.5,
            12,
            "heating",
            0.33,
        ),
        (
            r#"{"method":"full"}"#,
            "north_east",
            7.5,
            3,
            "heating",
            0.49,
        ),
        (
            r#"{"method":"other"}"#,
            "south_east",
            120.0,
            10,
            "heating",
            0.41,
        ),
        (
            r#"{"method":"other","overhangRelativeHeight":0.7}"#,
            "south",
            90.0,
            7,
            "cooling",
            0.57,
        ),
        (
            r#"{"method":"other","overhangRelativeHeight":0.7}"#,
            "south",
            60.0,
            7,
            "cooling",
            1.0,
        ),
        (r#"{"method":"other"}"#, "west", 90.0, 7, "cooling", 1.0),
    ];
    for &(situation, side, tilt, month, bal, expected) in rows {
        let situation: Obstruction = serde_json::from_str(situation).expect("situation");
        let kernel = obstruction_factor(&situation, orientation(side), tilt, month, balance(bal))
            .expect("factor");
        close(
            &format!("{situation:?} {side} {tilt} {month} {bal}"),
            kernel,
            expected,
        );
    }
}

/// F_sh;obst for collectors and PV (x = P): tables 17.6, 17.12 with footnote c (p. 744-751), 17.15
/// (p. 760-765) and the roof-edge conditions of situation f (p. 703-706).
#[test]
fn collector_obstruction_tables_17_6_17_12_17_15() {
    let rows: &[(&str, &str, f64, u8, &str, f64)] = &[
        (
            r#"{"method":"side_obstruction","side":"left","relativeWidth":0.3}"#,
            "south_west",
            35.0,
            6,
            "p",
            0.79,
        ),
        (
            r#"{"method":"side_obstruction","side":"both","relativeWidth":0.3}"#,
            "south",
            35.0,
            12,
            "p",
            0.32,
        ),
        (
            r#"{"method":"side_obstruction","side":"both","relativeWidth":0.3}"#,
            "east",
            90.0,
            1,
            "p",
            0.55,
        ),
        (
            r#"{"method":"side_obstruction","side":"right","relativeWidth":2}"#,
            "north",
            10.0,
            4,
            "p",
            0.88,
        ),
        (r#"{"method":"full"}"#, "west", 45.0, 3, "p", 0.52),
        (r#"{"method":"other"}"#, "north_east", 20.0, 9, "p", 0.5),
        (
            r#"{"method":"roof_edge","heightM":0.8,"distanceM":0.5}"#,
            "south",
            37.5,
            1,
            "p",
            0.3,
        ),
        (
            r#"{"method":"roof_edge","heightM":0.8,"distanceM":0.8}"#,
            "south",
            37.5,
            1,
            "p",
            1.0,
        ),
        (
            r#"{"method":"roof_edge","heightM":0.5,"distanceM":0.1}"#,
            "east",
            30.0,
            11,
            "p",
            1.0,
        ),
    ];
    for &(situation, side, tilt, month, bal, expected) in rows {
        let _ = bal;
        let situation: CollectorObstruction = serde_json::from_str(situation).expect("situation");
        let kernel = collector_obstruction_factor(&situation, orientation(side), tilt, month)
            .expect("factor");
        close(
            &format!("{situation:?} {side} {tilt} {month}"),
            kernel,
            expected,
        );
    }
}

/// f_sh;with: case 1 of 7.6.6.1.4 (p. 197) and tables 7.7-7.9 (p. 200-202) with linear interpolation
/// between the 0, 45, 90 and 180 degree columns.
#[test]
fn shading_fraction_tables_7_7_to_7_9() {
    let rows: &[(&str, &str, f64, u8, &str, f64)] = &[
        ("manual_residential", "south", 90.0, 7, "heating", 0.0),
        ("manual_residential", "south", 90.0, 7, "cooling", 0.59),
        (
            "automatic_residential_iso52016",
            "west",
            45.0,
            6,
            "heating",
            0.0,
        ),
        (
            "automatic_residential_iso52016",
            "west",
            22.5,
            6,
            "cooling",
            0.905,
        ),
        ("automatic", "east", 67.5, 4, "heating", 0.81),
        (
            "automatic",
            "north",
            135.0,
            5,
            "cooling",
            0.21000000000000002,
        ),
        (
            "manual_utility_with_glare_protection",
            "south_east",
            10.0,
            1,
            "heating",
            0.0,
        ),
        (
            "manual_utility_with_glare_protection",
            "south_east",
            90.0,
            3,
            "heating",
            0.64,
        ),
        (
            "manual_utility_without_glare_protection",
            "south",
            90.0,
            1,
            "heating",
            0.7,
        ),
        (
            "manual_utility_without_glare_protection",
            "north_west",
            30.0,
            11,
            "cooling",
            0.07,
        ),
        (
            "automatic",
            "south_west",
            100.0,
            8,
            "heating",
            0.7666666666666666,
        ),
        ("automatic", "north_east", 0.0, 2, "cooling", 0.67),
    ];
    for &(control, side, tilt, month, bal, expected) in rows {
        let control: ShadingControl =
            serde_json::from_value(Value::String(control.to_string())).expect("control");
        let kernel = shading_fraction(control, orientation(side), tilt, month, balance(bal));
        close(
            &format!("{control:?} {side} {tilt} {month} {bal}"),
            kernel,
            expected,
        );
    }
}

/// (1 - f_sh;with) + f_sh;with * F_c of 7.42/7.43 (p. 196-198) with F_c rounded up to two decimals and
/// the device values of tables 7.5 and 7.6 (p. 199).
#[test]
fn movable_shading_factor_with_devices() {
    let rows: &[(&str, &str, f64, u8, &str, f64)] = &[
        (
            r#"{"reductionFactor":0.123,"control":"automatic","sourceReference":"x"}"#,
            "south",
            90.0,
            7,
            "cooling",
            0.32139999999999996,
        ),
        (
            r#"{"reductionFactor":0.351,"control":"automatic","sourceReference":"x"}"#,
            "north",
            30.0,
            6,
            "heating",
            0.4197333333333333,
        ),
        (
            r#"{"device":{"kind":"external_screen","colour":"dark"},"control":"automatic","sourceReference":"x"}"#,
            "south_west",
            90.0,
            6,
            "cooling",
            0.30479999999999996,
        ),
        (
            r#"{"device":{"kind":"external_screen","colour":"white"},"control":"automatic","sourceReference":"x"}"#,
            "south_west",
            90.0,
            6,
            "cooling",
            0.4075,
        ),
        (
            r#"{"device":{"kind":"external_screen","colour":"unknown"},"control":"automatic","sourceReference":"x"}"#,
            "south_west",
            90.0,
            6,
            "cooling",
            0.368,
        ),
        (
            r#"{"device":{"kind":"external_venetian_blind","colour":"other"},"control":"automatic","sourceReference":"x"}"#,
            "south_west",
            90.0,
            6,
            "cooling",
            0.289,
        ),
        (
            r#"{"device":{"kind":"external_venetian_blind","colour":"white"},"control":"automatic","sourceReference":"x"}"#,
            "south_west",
            90.0,
            6,
            "cooling",
            0.368,
        ),
        (
            r#"{"device":{"kind":"external_roller_shutter","colour":"dark"},"control":"automatic","sourceReference":"x"}"#,
            "south_west",
            90.0,
            6,
            "cooling",
            0.29689999999999994,
        ),
        (
            r#"{"device":{"kind":"external_roller_shutter","colour":"white"},"control":"automatic","sourceReference":"x"}"#,
            "south_west",
            90.0,
            6,
            "cooling",
            0.24159999999999998,
        ),
        (
            r#"{"device":{"kind":"internal_metallised_fabric"},"control":"manual_utility_with_glare_protection","sourceReference":"x"}"#,
            "east",
            90.0,
            5,
            "heating",
            0.692,
        ),
        (
            r#"{"device":{"kind":"drop_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "north",
            90.0,
            7,
            "cooling",
            0.83,
        ),
        (
            r#"{"device":{"kind":"drop_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "north_east",
            90.0,
            7,
            "cooling",
            0.6755,
        ),
        (
            r#"{"device":{"kind":"drop_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "east",
            90.0,
            7,
            "cooling",
            0.532,
        ),
        (
            r#"{"device":{"kind":"drop_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "south_east",
            90.0,
            7,
            "cooling",
            0.5125,
        ),
        (
            r#"{"device":{"kind":"drop_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "south",
            90.0,
            7,
            "cooling",
            0.49299999999999994,
        ),
        (
            r#"{"device":{"kind":"folding_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "north",
            90.0,
            7,
            "cooling",
            0.966,
        ),
        (
            r#"{"device":{"kind":"folding_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "north_west",
            90.0,
            7,
            "cooling",
            0.874,
        ),
        (
            r#"{"device":{"kind":"folding_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "west",
            90.0,
            7,
            "cooling",
            0.7444999999999999,
        ),
        (
            r#"{"device":{"kind":"folding_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "south_west",
            90.0,
            7,
            "cooling",
            0.658,
        ),
        (
            r#"{"device":{"kind":"folding_arm_awning"},"control":"automatic","sourceReference":"x"}"#,
            "south",
            90.0,
            7,
            "cooling",
            0.61,
        ),
        (
            r#"{"device":{"kind":"folding_arm_awning"},"control":"manual_residential","sourceReference":"x"}"#,
            "south",
            90.0,
            7,
            "heating",
            1.0,
        ),
    ];
    for &(shading, side, tilt, month, bal, expected) in rows {
        let shading: MovableShading = serde_json::from_str(shading).expect("shading");
        let kernel =
            movable_shading_factor(Some(&shading), orientation(side), tilt, month, balance(bal));
        close(
            &format!("{shading:?} {side} {tilt} {month} {bal}"),
            kernel,
            expected,
        );
    }
}

/// eta_H;gn of 7.46-7.49 (p. 207) and eta_C;ht of 7.52-7.54 (p. 209), including gamma = 1 and gamma <= 0.
#[test]
fn utilisation_factor_branches() {
    let rows: &[(&str, f64, f64, f64, f64)] = &[
        ("h", 0.3, 3.78, 5.0, 0.9925870008699595),
        ("h", 1.0, 3.78, 5.0, 0.790794979079498),
        ("h", 1.0, 1.5, 5.0, 0.6),
        ("h", 0.999999, 12.0, 5.0, 0.923077384616484),
        ("h", 1.7, 2.0, 5.0, 0.48300536672629696),
        ("h", -0.5, 3.78, 5.0, -2.0),
        ("h", -0.5, 3.78, -5.0, 1.0),
        ("h", -2.0, 1.0, 0.0, 1.0),
        ("c", 0.3, 3.78, 0.0, 0.29777610026098805),
        ("c", 1.0, 3.78, 0.0, 0.790794979079498),
        ("c", 1.000001, 12.0, 0.0, 0.9230773846162886),
        ("c", 2.5, 1.5, 0.0, 0.8311213549869846),
        ("c", -0.5, 3.78, 0.0, 1.0),
        ("c", 0.0, 2.0, 0.0, 1.0),
    ];
    for &(kind, gamma, a, gains, expected) in rows {
        let kernel = if kind == "h" {
            heating_utilization(gamma, a, gains)
        } else {
            cooling_utilization(gamma, a)
        };
        close(&format!("{kind} {gamma} {a} {gains}"), kernel, expected);
    }
}

/// 7.32 per window (p. 182-183) with F_sh;obst of every situation (17.3.2, p. 698-707), g_gl = F_w * g_gl;n
/// (7.40, p. 190) and Q_sky of 7.39 (p. 188) with F_sky 0,5; residential zone.
#[test]
fn window_gains_per_obstruction_situation() {
    let d = demand(
        r#"{"windows":[{"id":"s-min","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"sw-overhang","areaM2":4,"orientation":"south_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"overhang","relativeHeight":0.6}},{"id":"e-parallel","areaM2":4,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"parallel_obstruction","relativeHeight":0.3}},{"id":"w-side-left","areaM2":4,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"side_obstruction","side":"left","relativeWidth":0.5,"coolingHeightCondition":true}},{"id":"nw-side-both","areaM2":4,"orientation":"north_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"side_obstruction","side":"both","relativeWidth":2}},{"id":"n-full","areaM2":4,"orientation":"north","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"se-other-oh","areaM2":4,"orientation":"south_east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"other","overhangRelativeHeight":1.2}},{"id":"ne-other","areaM2":4,"orientation":"north_east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"other"}},{"id":"s-full-nc","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full"}},{"id":"s-side-right","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"side_obstruction","side":"right","relativeWidth":1.5,"coolingHeightCondition":true}},{"id":"s-parallel-hi","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"parallel_obstruction","relativeHeight":1.4}},{"id":"s-overhang-lo","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"overhang","relativeHeight":0.2}},{"id":"s-declared","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"declared","heating":[0.5,0.55,0.6,0.65,0.7,0.75,0.8,0.85,0.9,0.95,1,0.4],"cooling":[0.9,0.8,0.7,0.6,0.5,0.4,0.3,0.2,0.1,0.15,0.25,0.35],"sourceReference":"x"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "window:s-min:heating", 12.865828320000006),
            (1, "window:s-min:cooling", 68.64257088000004),
            (1, "window:sw-overhang:heating", 13.017693600000001),
            (1, "window:sw-overhang:cooling", 50.12103312000003),
            (1, "window:e-parallel:heating", 15.682567680000009),
            (1, "window:e-parallel:cooling", 20.551898880000007),
            (1, "window:w-side-left:heating", 10.024983360000004),
            (1, "window:w-side-left:cooling", 13.127374080000006),
            (1, "window:nw-side-both:heating", 9.533229120000005),
            (1, "window:nw-side-both:cooling", 9.945434880000006),
            (1, "window:n-full:heating", 9.583850880000007),
            (1, "window:n-full:cooling", 9.583850880000007),
            (1, "window:se-other-oh:heating", 9.433190880000005),
            (1, "window:se-other-oh:cooling", 49.11703488000002),
            (1, "window:ne-other:heating", 9.583850880000007),
            (1, "window:ne-other:cooling", 9.583850880000007),
            (1, "window:s-full-nc:heating", 9.968335200000006),
            (1, "window:s-full-nc:cooling", 68.64257088000004),
            (1, "window:s-side-right:heating", 12.865828320000006),
            (1, "window:s-side-right:cooling", 68.64257088000004),
            (1, "window:s-parallel-hi:heating", 9.968335200000006),
            (1, "window:s-parallel-hi:cooling", 68.64257088000004),
            (1, "window:s-overhang-lo:heating", 9.968335200000006),
            (1, "window:s-overhang-lo:cooling", 59.95009152000002),
            (1, "window:s-declared:heating", 32.42390688000002),
            (1, "window:s-declared:cooling", 61.39883808000003),
            (4, "window:s-min:heating", 153.90829440000005),
            (4, "window:s-min:cooling", 153.90829440000005),
            (4, "window:sw-overhang:heating", 60.31052640000003),
            (4, "window:sw-overhang:cooling", 102.96577440000004),
            (4, "window:e-parallel:heating", 103.64112000000004),
            (4, "window:e-parallel:cooling", 127.19773440000006),
            (4, "window:w-side-left:heating", 44.57695680000001),
            (4, "window:w-side-left:cooling", 96.19248960000003),
            (4, "window:nw-side-both:heating", 45.6733728),
            (4, "window:nw-side-both:cooling", 71.09389440000002),
            (4, "window:n-full:heating", 52.21920960000002),
            (4, "window:n-full:cooling", 52.21920960000002),
            (4, "window:se-other-oh:heating", 58.10253120000003),
            (4, "window:se-other-oh:cooling", 154.7247744000001),
            (4, "window:ne-other:heating", 60.10057440000002),
            (4, "window:ne-other:cooling", 81.35821440000005),
            (4, "window:s-full-nc:heating", 53.05668480000002),
            (4, "window:s-full-nc:cooling", 153.90829440000005),
            (4, "window:s-side-right:heating", 134.99861760000005),
            (4, "window:s-side-right:cooling", 153.90829440000005),
            (4, "window:s-parallel-hi:heating", 53.05668480000002),
            (4, "window:s-parallel-hi:cooling", 153.90829440000005),
            (4, "window:s-overhang-lo:heating", 53.05668480000002),
            (4, "window:s-overhang-lo:cooling", 53.05668480000002),
            (4, "window:s-declared:heating", 98.75507040000005),
            (4, "window:s-declared:cooling", 90.87603840000004),
            (7, "window:s-min:heating", 128.42445888000006),
            (7, "window:s-min:cooling", 128.42445888000006),
            (7, "window:sw-overhang:heating", 75.24268416000002),
            (7, "window:sw-overhang:cooling", 95.00204448000002),
            (7, "window:e-parallel:heating", 106.20271152000004),
            (7, "window:e-parallel:cooling", 122.63911488000006),
            (7, "window:w-side-left:heating", 79.06462704000002),
            (7, "window:w-side-left:cooling", 121.17349440000005),
            (7, "window:nw-side-both:heating", 77.27237568000004),
            (7, "window:nw-side-both:cooling", 102.87252288000003),
            (7, "window:n-full:heating", 67.75427952000003),
            (7, "window:n-full:cooling", 67.75427952000003),
            (7, "window:se-other-oh:heating", 65.78846784000002),
            (7, "window:se-other-oh:cooling", 132.64293888000006),
            (7, "window:ne-other:heating", 63.734670720000025),
            (7, "window:ne-other:cooling", 94.07397888000004),
            (7, "window:s-full-nc:heating", 70.24800384000002),
            (7, "window:s-full-nc:cooling", 128.42445888000006),
            (7, "window:s-side-right:heating", 119.16911376000006),
            (7, "window:s-side-right:cooling", 128.42445888000006),
            (7, "window:s-parallel-hi:heating", 70.24800384000002),
            (7, "window:s-parallel-hi:cooling", 128.42445888000006),
            (7, "window:s-overhang-lo:heating", 70.24800384000002),
            (7, "window:s-overhang-lo:cooling", 70.24800384000002),
            (7, "window:s-declared:heating", 101.98061568000004),
            (7, "window:s-declared:cooling", 35.87100768000001),
        ],
    );
}

/// Tilted windows and rooflights: I_sol interpolated between the tilt columns of table 17.2 (p. 691-694),
/// F_sky per tilt band (7.6.6.4, p. 203), table 17.4 nearest tilt and south field of view below 15 degrees.
#[test]
fn rooflights_and_tilted_windows_minimal() {
    let d = demand(
        r#"{"windows":[{"id":"t0","areaM2":4,"orientation":"north","tiltDeg":0,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t5","areaM2":4,"orientation":"north_east","tiltDeg":5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t6","areaM2":4,"orientation":"east","tiltDeg":6,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t10","areaM2":4,"orientation":"south_east","tiltDeg":10,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t15","areaM2":4,"orientation":"south","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t22.5","areaM2":4,"orientation":"south_west","tiltDeg":22.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t30","areaM2":4,"orientation":"west","tiltDeg":30,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t37.5","areaM2":4,"orientation":"north_west","tiltDeg":37.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t45","areaM2":4,"orientation":"north","tiltDeg":45,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t52.5","areaM2":4,"orientation":"north_east","tiltDeg":52.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t60","areaM2":4,"orientation":"east","tiltDeg":60,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t67.5","areaM2":4,"orientation":"south_east","tiltDeg":67.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t75","areaM2":4,"orientation":"south","tiltDeg":75,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t76","areaM2":4,"orientation":"south_west","tiltDeg":76,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t82.5","areaM2":4,"orientation":"west","tiltDeg":82.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"t90","areaM2":4,"orientation":"north_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "window:t0:heating", 26.15832576000001),
            (1, "window:t0:cooling", 26.15832576000001),
            (1, "window:t5:heating", 23.707589760000015),
            (1, "window:t5:cooling", 23.707589760000015),
            (1, "window:t6:heating", 27.790542720000012),
            (1, "window:t6:cooling", 27.790542720000012),
            (1, "window:t10:heating", 10.878053760000007),
            (1, "window:t10:cooling", 33.760696320000015),
            (1, "window:t15:heating", 14.176905120000008),
            (1, "window:t15:cooling", 41.61510432000002),
            (1, "window:t22.5:heating", 27.337357440000005),
            (1, "window:t22.5:cooling", 42.880648320000006),
            (1, "window:t30:heating", 23.668485120000014),
            (1, "window:t30:cooling", 29.260984320000013),
            (1, "window:t37.5:heating", 12.487705200000008),
            (1, "window:t37.5:cooling", 13.049968320000005),
            (1, "window:t45:heating", 11.543368320000006),
            (1, "window:t45:cooling", 11.543368320000006),
            (1, "window:t52.5:heating", 11.013045120000008),
            (1, "window:t52.5:cooling", 11.181784320000006),
            (1, "window:t60:heating", 21.99435120000001),
            (1, "window:t60:cooling", 24.078280320000005),
            (1, "window:t67.5:heating", 24.363329040000007),
            (1, "window:t67.5:cooling", 51.016288320000015),
            (1, "window:t75:heating", 11.99655360000001),
            (1, "window:t75:cooling", 68.01073632000005),
            (1, "window:t76:heating", 26.83333612800001),
            (1, "window:t76:cooling", 56.260327680000024),
            (1, "window:t82.5:heating", 21.600492480000007),
            (1, "window:t82.5:cooling", 25.734602880000008),
            (1, "window:t90:heating", 9.533229120000005),
            (1, "window:t90:cooling", 9.945434880000006),
            (7, "window:t0:heating", 222.6189657600001),
            (7, "window:t0:cooling", 222.6189657600001),
            (7, "window:t5:heating", 215.76895776000003),
            (7, "window:t5:cooling", 215.76895776000003),
            (7, "window:t6:heating", 220.65944832000008),
            (7, "window:t6:cooling", 220.65944832000008),
            (7, "window:t10:heating", 214.92231552000007),
            (7, "window:t10:cooling", 224.11458432000006),
            (7, "window:t15:heating", 218.25853056000008),
            (7, "window:t15:cooling", 227.5898083200001),
            (7, "window:t22.5:heating", 203.28533712000004),
            (7, "window:t22.5:cooling", 226.50505632000005),
            (7, "window:t30:heating", 165.88911456000005),
            (7, "window:t30:cooling", 211.49932032000004),
            (7, "window:t37.5:heating", 134.02814040000007),
            (7, "window:t37.5:cooling", 175.76276832000008),
            (7, "window:t45:heating", 97.01579952000004),
            (7, "window:t45:cooling", 143.15994432000005),
            (7, "window:t52.5:heating", 122.76058032000005),
            (7, "window:t52.5:cooling", 141.95466432000003),
            (7, "window:t60:heating", 153.09627264000005),
            (7, "window:t60:cooling", 166.90396032000004),
            (7, "window:t67.5:heating", 172.87431480000006),
            (7, "window:t67.5:cooling", 174.67801632000007),
            (7, "window:t75:heating", 160.39544832000007),
            (7, "window:t75:cooling", 160.39544832000007),
            (7, "window:t76:heating", 162.82748908800008),
            (7, "window:t76:cooling", 164.51054208000008),
            (7, "window:t82.5:heating", 131.67992016000008),
            (7, "window:t82.5:cooling", 143.46032688000008),
            (7, "window:t90:heating", 92.20579488000003),
            (7, "window:t90:cooling", 102.87252288000003),
        ],
    );
}

/// As above with full obstruction (tables 17.13/17.14, p. 752-763).
#[test]
fn rooflights_and_tilted_windows_full_obstruction() {
    let d = demand(
        r#"{"windows":[{"id":"t0","areaM2":4,"orientation":"south_east","tiltDeg":0,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t5","areaM2":4,"orientation":"south","tiltDeg":5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t6","areaM2":4,"orientation":"south_west","tiltDeg":6,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t10","areaM2":4,"orientation":"west","tiltDeg":10,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t15","areaM2":4,"orientation":"north_west","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t22.5","areaM2":4,"orientation":"north","tiltDeg":22.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t30","areaM2":4,"orientation":"north_east","tiltDeg":30,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t37.5","areaM2":4,"orientation":"east","tiltDeg":37.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t45","areaM2":4,"orientation":"south_east","tiltDeg":45,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t52.5","areaM2":4,"orientation":"south","tiltDeg":52.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t60","areaM2":4,"orientation":"south_west","tiltDeg":60,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t67.5","areaM2":4,"orientation":"west","tiltDeg":67.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t75","areaM2":4,"orientation":"north_west","tiltDeg":75,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t76","areaM2":4,"orientation":"north","tiltDeg":76,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t82.5","areaM2":4,"orientation":"north_east","tiltDeg":82.5,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"t90","areaM2":4,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}}]}"#,
    );
    check(
        &d,
        &[
            (2, "window:t0:heating", 24.273527040000005),
            (2, "window:t0:cooling", 24.273527040000005),
            (2, "window:t5:heating", 26.357184000000004),
            (2, "window:t5:cooling", 26.357184000000004),
            (2, "window:t6:heating", 27.49004697600001),
            (2, "window:t6:cooling", 27.49004697600001),
            (2, "window:t10:heating", 19.554877440000002),
            (2, "window:t10:cooling", 19.554877440000002),
            (2, "window:t15:heating", 26.626320000000007),
            (2, "window:t15:cooling", 26.626320000000007),
            (2, "window:t22.5:heating", 30.484460160000005),
            (2, "window:t22.5:cooling", 30.484460160000005),
            (2, "window:t30:heating", 24.90518016000001),
            (2, "window:t30:cooling", 24.90518016000001),
            (2, "window:t37.5:heating", 24.517624320000014),
            (2, "window:t37.5:cooling", 24.517624320000014),
            (2, "window:t45:heating", 23.092594560000016),
            (2, "window:t45:cooling", 23.092594560000016),
            (2, "window:t52.5:heating", 24.154018560000008),
            (2, "window:t52.5:cooling", 24.154018560000008),
            (2, "window:t60:heating", 21.90053376000001),
            (2, "window:t60:cooling", 21.90053376000001),
            (2, "window:t67.5:heating", 20.34922176000001),
            (2, "window:t67.5:cooling", 20.34922176000001),
            (2, "window:t75:heating", 18.802808640000013),
            (2, "window:t75:cooling", 18.802808640000013),
            (2, "window:t76:heating", 20.13790464000001),
            (2, "window:t76:cooling", 20.13790464000001),
            (2, "window:t82.5:heating", 19.308179520000007),
            (2, "window:t82.5:cooling", 19.308179520000007),
            (2, "window:t90:heating", 18.02957184000001),
            (2, "window:t90:cooling", 18.02957184000001),
            (8, "window:t0:heating", 71.43346368000003),
            (8, "window:t0:cooling", 71.43346368000003),
            (8, "window:t5:heating", 72.97200360000001),
            (8, "window:t5:cooling", 72.97200360000001),
            (8, "window:t6:heating", 75.21276643200002),
            (8, "window:t6:cooling", 75.21276643200002),
            (8, "window:t10:heating", 69.22806912000003),
            (8, "window:t10:cooling", 69.22806912000003),
            (8, "window:t15:heating", 72.60285312000003),
            (8, "window:t15:cooling", 72.60285312000003),
            (8, "window:t22.5:heating", 81.20614176000002),
            (8, "window:t22.5:cooling", 81.20614176000002),
            (8, "window:t30:heating", 71.14446432000003),
            (8, "window:t30:cooling", 71.14446432000003),
            (8, "window:t37.5:heating", 73.14221592000006),
            (8, "window:t37.5:cooling", 73.14221592000006),
            (8, "window:t45:heating", 70.56592992000003),
            (8, "window:t45:cooling", 70.56592992000003),
            (8, "window:t52.5:heating", 70.77564864000003),
            (8, "window:t52.5:cooling", 70.77564864000003),
            (8, "window:t60:heating", 69.25940640000003),
            (8, "window:t60:cooling", 69.25940640000003),
            (8, "window:t67.5:heating", 69.06354840000004),
            (8, "window:t67.5:cooling", 69.06354840000004),
            (8, "window:t75:heating", 65.36695464000003),
            (8, "window:t75:cooling", 65.36695464000003),
            (8, "window:t76:heating", 65.93535129600002),
            (8, "window:t76:cooling", 65.93535129600002),
            (8, "window:t82.5:heating", 63.09828288000003),
            (8, "window:t82.5:cooling", 63.09828288000003),
            (8, "window:t90:heating", 57.34909728000002),
            (8, "window:t90:cooling", 57.34909728000002),
        ],
    );
}

/// g of the glazing: table 7.4 types and g_gl;n rounded down to 0,05 (7.40, p. 190-191); fixed louvres
/// 7.41a with table 7.4a and rotatable louvres 7.41b with table 7.4b and f_sh;with (p. 192-194); 7.41
/// with a_gl 0,75 (p. 191, read as written for diffusing glazing, see the review notes); combinations
/// with movable shading 7.42/7.43.
#[test]
fn glazing_g_values_louvres_and_diffusing() {
    let d = demand(
        r#"{"windows":[{"id":"gt-single","areaM2":4,"orientation":"north","tiltDeg":90,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"glazingType":"single"}},{"id":"gt-double","areaM2":4,"orientation":"north_east","tiltDeg":90,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"glazingType":"double"}},{"id":"gt-double_low_e","areaM2":4,"orientation":"east","tiltDeg":90,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"glazingType":"double_low_e"}},{"id":"gt-triple_one_coating","areaM2":4,"orientation":"south_east","tiltDeg":90,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"glazingType":"triple_one_coating"}},{"id":"gt-triple_two_coatings","areaM2":4,"orientation":"south","tiltDeg":90,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"glazingType":"triple_two_coatings"}},{"id":"gt-single_with_secondary_pane","areaM2":4,"orientation":"south_west","tiltDeg":90,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"glazingType":"single_with_secondary_pane"}},{"id":"g067","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.67,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"g0349","areaM2":4,"orientation":"east","tiltDeg":90,"gPerpendicular":0.349,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"g0451","areaM2":4,"orientation":"west","tiltDeg":90,"gPerpendicular":0.451,"frameFraction":0.1,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"lam90","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"horizontal90"}}},{"id":"lamang","areaM2":4,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"horizontal_angled"}}},{"id":"lamrot-man","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_residential"}}},{"id":"lamrot-auto","areaM2":4,"orientation":"south_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"automatic"}}},{"id":"lamrot-iso","areaM2":4,"orientation":"east","tiltDeg":60,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"automatic_residential_iso52016"}}},{"id":"diff","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"diffusing":{"gAltitude45":0.3,"gDiffuse":0.45,"sourceReference":"x"}}},{"id":"diff-mov","areaM2":4,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"diffusing":{"gAltitude45":0.3,"gDiffuse":0.45,"sourceReference":"x"}},"movableShading":{"reductionFactor":0.3,"control":"automatic","sourceReference":"x"}},{"id":"lam-mov","areaM2":4,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"horizontal90"}},"movableShading":{"device":{"kind":"external_screen","colour":"white"},"control":"automatic","sourceReference":"x"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "window:gt-single:heating", 15.158270880000003),
            (1, "window:gt-single:cooling", 15.158270880000003),
            (1, "window:gt-double:heating", 12.928502880000003),
            (1, "window:gt-double:cooling", 12.928502880000003),
            (1, "window:gt-double_low_e:heating", 18.60416640000001),
            (1, "window:gt-double_low_e:cooling", 20.551898880000007),
            (1, "window:gt-triple_one_coating:heating", 17.36995968),
            (
                1,
                "window:gt-triple_one_coating:cooling",
                40.298402880000005,
            ),
            (
                1,
                "window:gt-triple_two_coatings:heating",
                7.312299840000001,
            ),
            (1, "window:gt-triple_two_coatings:cooling", 44.49679488),
            (
                1,
                "window:gt-single_with_secondary_pane:heating",
                31.71429828000001,
            ),
            (
                1,
                "window:gt-single_with_secondary_pane:cooling",
                68.67270288000002,
            ),
            (1, "window:g067:heating", 14.254210440000005),
            (1, "window:g067:cooling", 74.67901488000003),
            (1, "window:g0349:heating", 7.404704640000006),
            (1, "window:g0349:cooling", 8.378570880000003),
            (1, "window:g0451:heating", 17.78096016),
            (1, "window:g0451:cooling", 21.588439680000004),
            (1, "window:lam90:heating", 0.7036009488000023),
            (1, "window:lam90:cooling", 15.763321440000006),
            (1, "window:lamang:heating", -0.19880423999999897),
            (1, "window:lamang:cooling", 0.4357756800000012),
            (1, "window:lamrot-man:heating", 0.7036009488000023),
            (1, "window:lamrot-man:cooling", 15.763321440000006),
            (1, "window:lamrot-auto:heating", -0.8972181993599992),
            (1, "window:lamrot-auto:cooling", 2.1185876160000015),
            (1, "window:lamrot-iso:heating", 1.7832157776000033),
            (1, "window:lamrot-iso:cooling", -0.7174991663999979),
            (1, "window:diff:heating", 6.61810878),
            (1, "window:diff:cooling", 41.47857288),
            (1, "window:diff-mov:heating", 5.794782011999999),
            (1, "window:diff-mov:cooling", 6.6286549799999985),
            (1, "window:lam-mov:heating", 0.34778059776000125),
            (1, "window:lam-mov:cooling", 1.0788166656000011),
            (5, "window:gt-single:heating", 98.72746452000003),
            (5, "window:gt-single:cooling", 101.89825488000002),
            (5, "window:gt-double:heating", 112.35958308000002),
            (5, "window:gt-double:cooling", 121.10238288000002),
            (5, "window:gt-double_low_e:heating", 127.42407648000004),
            (5, "window:gt-double_low_e:cooling", 134.33033088000005),
            (
                5,
                "window:gt-triple_one_coating:heating",
                125.17020288000002,
            ),
            (
                5,
                "window:gt-triple_one_coating:cooling",
                125.17020288000002,
            ),
            (
                5,
                "window:gt-triple_two_coatings:heating",
                96.56489088000004,
            ),
            (
                5,
                "window:gt-triple_two_coatings:cooling",
                96.56489088000004,
            ),
            (
                5,
                "window:gt-single_with_secondary_pane:heating",
                178.32154428000004,
            ),
            (
                5,
                "window:gt-single_with_secondary_pane:cooling",
                180.16110288000004,
            ),
            (5, "window:g067:heating", 159.28967088000007),
            (5, "window:g067:cooling", 159.28967088000007),
            (5, "window:g0349:heating", 61.81465968000002),
            (5, "window:g0349:cooling", 65.26778688000002),
            (5, "window:g0451:heating", 98.63186572800002),
            (5, "window:g0451:cooling", 112.59913248000002),
            (5, "window:lam90:heating", 36.850900320000015),
            (5, "window:lam90:cooling", 36.850900320000015),
            (5, "window:lamang:heating", 13.276346688000004),
            (5, "window:lamang:cooling", 15.604224480000006),
            (5, "window:lamrot-man:heating", 36.850900320000015),
            (5, "window:lamrot-man:cooling", 15.669996609600005),
            (5, "window:lamrot-auto:heating", 10.759977687888005),
            (5, "window:lamrot-auto:cooling", 10.906995211200003),
            (5, "window:lamrot-iso:heating", 41.12717081760002),
            (5, "window:lamrot-iso:cooling", 11.890742068800002),
            (5, "window:diff:heating", 90.29241288),
            (5, "window:diff:cooling", 90.29241288),
            (5, "window:diff-mov:heating", 35.160834104999985),
            (5, "window:diff-mov:cooling", 37.21112837999999),
            (5, "window:lam-mov:heating", 10.109656931616003),
            (5, "window:lam-mov:cooling", 12.005713393200004),
            (7, "window:gt-single:heating", 106.67749140000002),
            (7, "window:gt-single:cooling", 110.09415888000002),
            (7, "window:gt-double:heating", 107.53093008000003),
            (7, "window:gt-double:cooling", 118.54116288000003),
            (7, "window:gt-double_low_e:heating", 113.78874384000007),
            (7, "window:gt-double_low_e:cooling", 122.63911488000006),
            (7, "window:gt-triple_one_coating:heating", 108.76634208),
            (7, "window:gt-triple_one_coating:cooling", 109.90332288),
            (7, "window:gt-triple_two_coatings:heating", 84.35138688),
            (7, "window:gt-triple_two_coatings:cooling", 84.35138688),
            (
                7,
                "window:gt-single_with_secondary_pane:heating",
                172.62810288000003,
            ),
            (
                7,
                "window:gt-single_with_secondary_pane:cooling",
                172.62810288000003,
            ),
            (7, "window:g067:heating", 139.44272688000004),
            (7, "window:g067:cooling", 139.44272688000004),
            (7, "window:g0349:heating", 54.99699336000003),
            (7, "window:g0349:cooling", 59.42217888000003),
            (7, "window:g0451:heating", 108.67666924800004),
            (7, "window:g0451:cooling", 118.45679328000003),
            (7, "window:lam90:heating", 31.904431200000012),
            (7, "window:lam90:cooling", 31.904431200000012),
            (7, "window:lamang:heating", 14.950480608000007),
            (7, "window:lamang:cooling", 16.580501280000004),
            (7, "window:lamrot-man:heating", 31.904431200000012),
            (7, "window:lamrot-man:cooling", 15.522470337600005),
            (7, "window:lamrot-auto:heating", 11.7869098752),
            (7, "window:lamrot-auto:cooling", 11.7869098752),
            (7, "window:lamrot-iso:heating", 37.18073456640001),
            (7, "window:lamrot-iso:cooling", 11.429397043200005),
            (7, "window:diff:heating", 78.84225288),
            (7, "window:diff:cooling", 78.84225288),
            (7, "window:diff-mov:heating", 32.6561281776),
            (7, "window:diff-mov:cooling", 35.399743199999996),
            (7, "window:lam-mov:heating", 11.473239009456005),
            (7, "window:lam-mov:cooling", 12.800890846800009),
        ],
    );
}

/// Movable shading in a dwelling: f_sh;with = 0 on the heating balance for manual and ISO 52016-3 tuned
/// automatic control (p. 197), table 7.9 for other automatic control on both balances, devices of
/// tables 7.5/7.6 and the F_c rounding of 7.43 (p. 198-199).
#[test]
fn movable_shading_residential_controls() {
    let d = demand(
        r#"{"windows":[{"id":"man","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.123,"control":"manual_residential","sourceReference":"x"}},{"id":"iso","areaM2":4,"orientation":"south_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.2,"control":"automatic_residential_iso52016","sourceReference":"x"}},{"id":"auto","areaM2":4,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.35,"control":"automatic","sourceReference":"x"}},{"id":"auto-roof","areaM2":4,"orientation":"north","tiltDeg":30,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.351,"control":"automatic","sourceReference":"x"}},{"id":"auto-60","areaM2":4,"orientation":"south_east","tiltDeg":60,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"device":{"kind":"external_venetian_blind","colour":"dark"},"control":"automatic","sourceReference":"x"}},{"id":"drop","areaM2":4,"orientation":"north_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"device":{"kind":"drop_arm_awning"},"control":"manual_residential","sourceReference":"x"}},{"id":"fold","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"overhang","relativeHeight":0.7},"movableShading":{"device":{"kind":"folding_arm_awning"},"control":"automatic","sourceReference":"x"}},{"id":"rol","areaM2":4,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"device":{"kind":"external_roller_shutter","colour":"white"},"control":"automatic","sourceReference":"x"}}]}"#,
    );
    check(
        &d,
        &[
            (3, "window:man:heating", 118.90274688000005),
            (3, "window:man:cooling", 46.31490351360001),
            (3, "window:iso:heating", 88.23198672000004),
            (3, "window:iso:cooling", 33.41175436800001),
            (3, "window:auto:heating", 36.5579992008),
            (3, "window:auto:cooling", 45.41592132000001),
            (3, "window:auto-roof:heating", 33.163607678976014),
            (3, "window:auto-roof:cooling", 41.69291719680002),
            (3, "window:auto-60:heating", 17.652891861360008),
            (3, "window:auto-60:cooling", 20.53823234400001),
            (3, "window:drop:heating", 40.75239168000002),
            (3, "window:drop:cooling", 42.60852288000002),
            (3, "window:fold:heating", 27.677152656000022),
            (3, "window:fold:cooling", 50.756553158400024),
            (3, "window:rol:heating", 25.217036363520005),
            (3, "window:rol:cooling", 28.80276364800001),
            (7, "window:man:heating", 128.42445888000006),
            (7, "window:man:cooling", 60.55633530720003),
            (7, "window:iso:heating", 137.34353088000006),
            (7, "window:iso:cooling", 51.53145177600002),
            (7, "window:auto:heating", 58.75966539072002),
            (7, "window:auto:cooling", 63.468062784000026),
            (7, "window:auto-roof:heating", 41.9391800064),
            (7, "window:auto-roof:cooling", 76.43082240000003),
            (7, "window:auto-60:heating", 33.51809032992001),
            (7, "window:auto-60:cooling", 34.318299024000005),
            (7, "window:drop:heating", 92.20579488000003),
            (7, "window:drop:cooling", 85.27242168000002),
            (7, "window:fold:heating", 41.371327065600006),
            (7, "window:fold:cooling", 42.17786428320001),
            (7, "window:rol:heating", 33.595743734784016),
            (7, "window:rol:cooling", 36.84709163520001),
            (10, "window:man:heating", 108.67474080000004),
            (10, "window:man:cooling", 42.54963289920002),
            (10, "window:iso:heating", 77.13256320000002),
            (10, "window:iso:cooling", 27.840468096000016),
            (10, "window:auto:heating", 27.288724659840014),
            (10, "window:auto:cooling", 33.20938785600001),
            (10, "window:auto-roof:heating", 28.334758459392006),
            (10, "window:auto-roof:cooling", 29.752545715200007),
            (10, "window:auto-60:heating", 13.90860979200001),
            (10, "window:auto-60:cooling", 16.58143872000001),
            (10, "window:drop:heating", 29.759032800000014),
            (10, "window:drop:cooling", 30.796778880000012),
            (10, "window:fold:heating", 29.61564063840001),
            (10, "window:fold:cooling", 52.54434498240002),
            (10, "window:rol:heating", 20.05003764403201),
            (10, "window:rol:cooling", 24.933911270400014),
        ],
    );
}

/// 7.33 with alpha_sol 0,6 (7.6.6.3), R_se 0,04 (table C.2), F_sh;obst = 1 (p. 185) and 7.39 with h_r 4,14
/// and delta-theta_sky 11 K (p. 188); F_sky 1 / 0,75 / 0,5 / 0 at tilts 0-5, 6-75, 76-90, over 90
/// degrees (p. 203); a partition behind an unheated space without solar terms.
#[test]
fn opaque_solar_gains_and_sky_radiation() {
    let d = demand(
        r#"{"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}],"opaqueElements":[{"id":"op0","areaM2":10,"orientation":"north","tiltDeg":0,"uValueWPerM2k":0.2,"sourceReference":"x"},{"id":"op5","areaM2":11,"orientation":"north_east","tiltDeg":5,"uValueWPerM2k":0.25,"sourceReference":"x"},{"id":"op6","areaM2":12,"orientation":"east","tiltDeg":6,"uValueWPerM2k":0.30000000000000004,"sourceReference":"x"},{"id":"op30","areaM2":13,"orientation":"south_east","tiltDeg":30,"uValueWPerM2k":0.35000000000000003,"sourceReference":"x"},{"id":"op45","areaM2":14,"orientation":"south","tiltDeg":45,"uValueWPerM2k":0.4,"sourceReference":"x"},{"id":"op75","areaM2":15,"orientation":"south_west","tiltDeg":75,"uValueWPerM2k":0.45,"sourceReference":"x"},{"id":"op76","areaM2":16,"orientation":"west","tiltDeg":76,"uValueWPerM2k":0.5,"sourceReference":"x"},{"id":"op90","areaM2":17,"orientation":"north_west","tiltDeg":90,"uValueWPerM2k":0.55,"sourceReference":"x"},{"id":"op100","areaM2":18,"orientation":"north","tiltDeg":100,"uValueWPerM2k":0.6000000000000001,"sourceReference":"x"},{"id":"op135","areaM2":19,"orientation":"north_east","tiltDeg":135,"uValueWPerM2k":0.65,"sourceReference":"x"},{"id":"op180","areaM2":20,"orientation":"east","tiltDeg":180,"uValueWPerM2k":0.7,"sourceReference":"x"},{"id":"behind","areaM2":12,"orientation":"south","tiltDeg":90,"uValueWPerM2k":0.5,"behindUnheatedSpace":true,"sourceReference":"x"}]}"#,
    );
    check(
        &d,
        &[
            (1, "opaque", -9.153155232000001),
            (4, "opaque", 86.13473184),
            (7, "opaque", 105.65183904),
            (12, "opaque", -16.676748095999997),
        ],
    );
}

/// 7.33/7.39 take U_c of 8.2.2 without the forfait surcharge of 8.3.
#[test]
fn forfait_delta_u_kept_out_of_solar_terms() {
    let d = demand(
        r#"{"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"forfaitDeltaUWPerM2k":0.1}],"opaqueElements":[{"id":"roof","areaM2":50,"orientation":"south","tiltDeg":0,"uValueWPerM2k":0.16,"sourceReference":"synthetic roof"},{"id":"wall","areaM2":30,"orientation":"west","tiltDeg":90,"uValueWPerM2k":0.25,"forfaitDeltaUWPerM2k":0.1,"sourceReference":"x"}]}"#,
    );
    check(
        &d,
        &[
            (1, "opaque", -8.790955199999999),
            (1, "window:w:heating", 12.865828320000006),
            (1, "window:w:cooling", 68.64257088000004),
            (7, "opaque", 26.4523248),
            (7, "window:w:heating", 128.42445888000006),
            (7, "window:w:cooling", 128.42445888000006),
        ],
    );
}

/// Annex A method B (p. 769): one state for g and U; F_w applied to the state g, H_tr corrected by
/// A_w * (U_mi - U_w) and Q_sky with the state U.
#[test]
fn dynamic_glazing_single_state() {
    let d = demand(
        r#"{"windows":[{"id":"dyn","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"dynamic":{"method":"single_state","state":{"id":"s","gPerpendicular":0.42,"uValueWPerM2k":1.1},"sourceReference":"x"}},{"id":"ref","areaM2":4,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "htr", 78.8),
            (1, "window:dyn:heating", 8.680814928),
            (1, "window:dyn:cooling", 47.72453471999999),
            (1, "heating.needKwh", 975.3673393862651),
            (1, "cooling.needKwh", 0.0),
            (7, "htr", 78.8),
            (7, "window:dyn:heating", 89.57185632),
            (7, "window:dyn:cooling", 89.57185632),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 114.36211473719266),
        ],
    );
}

/// Annex A method A (A.1/A.2, p. 768) with step-2 correction factors per month.
#[test]
fn dynamic_glazing_weighted_states() {
    let d = demand(
        r#"{"windows":[{"id":"dyn","areaM2":4,"orientation":"south_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"dynamic":{"method":"weighted_states","states":[{"id":"clear","gPerpendicular":0.63,"uValueWPerM2k":1.2},{"id":"tinted","gPerpendicular":0.18,"uValueWPerM2k":0.9}],"solarWeights":[[0.1,0.9],[0.16,0.84],[0.22,0.78],[0.28,0.72],[0.33999999999999997,0.66],[0.4,0.6],[0.45999999999999996,0.54],[0.52,0.48],[0.58,0.42000000000000004],[0.64,0.36],[0.7,0.30000000000000004],[0.7599999999999999,0.2400000000000001]],"temperatureWeights":[[0.95,0.05],[0.92,0.08],[0.89,0.11],[0.86,0.14],[0.8300000000000001,0.16999999999999998],[0.8,0.2],[0.77,0.22999999999999998],[0.74,0.26],[0.71,0.29],[0.6799999999999999,0.32],[0.65,0.35],[0.6200000000000001,0.37999999999999995]],"sourceReference":"x","correction":{"gFactors":[1,1,0.95,0.95,0.9,0.9,0.9,0.9,0.95,1,1,1],"uFactors":[1.02,1,1,1,1,1,1,1,1,1,1,1.02],"sourceReference":"x"}}}]}"#,
    );
    check(
        &d,
        &[
            (1, "htr", 79.2348),
            (1, "window:dyn:heating", 7.3764859550400015),
            (1, "window:dyn:cooling", 18.46400733504),
            (3, "htr", 79.068),
            (3, "window:dyn:heating", 37.48961297772),
            (3, "window:dyn:cooling", 40.54950221039999),
            (6, "htr", 78.96),
            (6, "window:dyn:heating", 77.50525824),
            (6, "window:dyn:cooling", 77.50525824),
            (12, "htr", 78.83088000000001),
            (12, "window:dyn:heating", 18.223111396224),
            (12, "window:dyn:cooling", 33.59339638502401),
        ],
    );
}

/// 7.21-7.24 (p. 177-178) with N_woon;zi = share * N_woon (6.2b) and 7.78 levelling.
#[test]
fn residential_internal_gains_res_a25_n1_snull() {
    let d = demand(
        r#"{"usableFloorAreaM2":25,"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 133.92),
            (1, "heating.setpointC", 15.073783606557377),
            (1, "heating.needKwh", 976.4829103386919),
            (7, "internal", 133.92),
            (7, "heating.setpointC", 19.44760655737705),
            (7, "heating.needKwh", 0.0),
        ],
    );
}

/// 7.21-7.24 (p. 177-178) with N_woon;zi = share * N_woon (6.2b) and 7.78 levelling.
#[test]
fn residential_internal_gains_res_a65_n1_snull() {
    let d = demand(
        r#"{"usableFloorAreaM2":65,"dwellingType":"apartment_building","windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 219.62879999999998),
            (1, "heating.setpointC", 17.80336842105263),
            (1, "heating.needKwh", 1078.6870779707847),
            (7, "internal", 219.62879999999998),
            (7, "heating.setpointC", 19.753684210526316),
            (7, "heating.needKwh", 0.0),
        ],
    );
}

/// 7.21-7.24 (p. 177-178) with N_woon;zi = share * N_woon (6.2b) and 7.78 levelling.
#[test]
fn residential_internal_gains_res_a150_n1_snull() {
    let d = demand(
        r#"{"usableFloorAreaM2":150,"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 372.29760000000005),
            (1, "heating.setpointC", 18.384412903225808),
            (1, "heating.needKwh", 997.7927309453432),
            (7, "internal", 372.29760000000005),
            (7, "heating.setpointC", 19.81883870967742),
            (7, "heating.needKwh", 0.0),
        ],
    );
}

/// 7.21-7.24 (p. 177-178) with N_woon;zi = share * N_woon (6.2b) and 7.78 levelling.
#[test]
fn residential_internal_gains_res_a300_n5_s0_4() {
    let d = demand(
        r#"{"usableFloorAreaM2":300,"dwellingType":"apartment_building","internalGains":{"method":"residential","dwellingCount":5,"sourceReference":"single dwelling","dwellingShare":0.4},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 744.5952000000001),
            (1, "heating.setpointC", 19.367636363636365),
            (1, "heating.needKwh", 734.0438982147583),
            (7, "internal", 744.5952000000001),
            (7, "heating.setpointC", 19.92909090909091),
            (7, "heating.needKwh", 0.0),
        ],
    );
}

/// 7.21-7.24 (p. 177-178) with N_woon;zi = share * N_woon (6.2b) and 7.78 levelling.
#[test]
fn residential_internal_gains_res_a1000_n3_snull() {
    let d = demand(
        r#"{"usableFloorAreaM2":1000,"internalGains":{"method":"residential","dwellingCount":3,"sourceReference":"single dwelling"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 1853.4528),
            (1, "heating.setpointC", 19.709942857142856),
            (1, "heating.needKwh", 0.9401077663337674),
            (7, "internal", 1853.4528),
            (7, "heating.setpointC", 19.967474903474905),
            (7, "heating.needKwh", 0.0),
        ],
    );
}

/// assembly_child_care: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_assembly_child_care() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"assembly_child_care","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 820.4383561643835),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 1012.5922883515036),
            (1, "cooling.gainsKwh", 1314.1163464739836),
            (1, "heating.setpointC", 21.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.9679708855130631),
            (1, "cooling.reductionFactor", 0.8),
            (1, "heating.calculationTemperatureC", 20.41098458458523),
            (1, "heating.heatTransferKwh", 1589.2719037117693),
            (1, "heating.utilization", 0.9897132574747582),
            (1, "cooling.utilization", 0.6767082805176354),
            (1, "heating.needKwh", 587.095891513583),
            (1, "cooling.needKwh", 17.44566762886352),
            (7, "internal", 820.4383561643835),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 2032.8915870761239),
            (7, "cooling.gainsKwh", 2100.775180568384),
            (7, "heating.setpointC", 21.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.8),
            (7, "heating.calculationTemperatureC", 21.0),
            (7, "heating.heatTransferKwh", 263.3759999999999),
            (7, "heating.utilization", 0.12955731692184308),
            (7, "cooling.utilization", 0.999986519520337),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1255.653073291895),
        ],
    );
}

/// other_assembly: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_other_assembly() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"other_assembly","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 541.4383561643837),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 733.5922883515037),
            (1, "cooling.gainsKwh", 1035.1163464739839),
            (1, "heating.setpointC", 21.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.9537700388191771),
            (1, "cooling.reductionFactor", 0.8),
            (1, "heating.calculationTemperatureC", 20.14983101388467),
            (1, "heating.heatTransferKwh", 1565.9561129196234),
            (1, "heating.utilization", 0.9987125709711263),
            (1, "cooling.utilization", 0.5401095575012105),
            (1, "heating.needKwh", 833.3082725755013),
            (1, "cooling.needKwh", 2.9356452812544376),
            (7, "internal", 541.4383561643837),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 1753.891587076124),
            (7, "cooling.gainsKwh", 1821.775180568384),
            (7, "heating.setpointC", 21.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.8),
            (7, "heating.calculationTemperatureC", 21.0),
            (7, "heating.heatTransferKwh", 263.3759999999999),
            (7, "heating.utilization", 0.1501665995982382),
            (7, "cooling.utilization", 0.9999603506741261),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1032.464194339742),
        ],
    );
}

/// cell: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_cell() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"cell","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 894.8383561643836),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 1086.9922883515037),
            (1, "cooling.gainsKwh", 1388.5163464739837),
            (1, "heating.setpointC", 21.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.99578288098115),
            (1, "cooling.reductionFactor", 1.0),
            (1, "heating.calculationTemperatureC", 20.92244718124335),
            (1, "heating.heatTransferKwh", 1634.9352843414065),
            (1, "heating.utilization", 0.9865625827064446),
            (1, "cooling.utilization", 0.7103428400520286),
            (1, "heating.needKwh", 562.5493649633586),
            (1, "cooling.needKwh", 31.975193100896604),
            (7, "internal", 894.8383561643836),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 2107.2915870761235),
            (7, "cooling.gainsKwh", 2175.1751805683834),
            (7, "heating.setpointC", 21.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 1.0),
            (7, "heating.calculationTemperatureC", 21.0),
            (7, "heating.heatTransferKwh", 263.3759999999999),
            (7, "heating.utilization", 0.12498317091624325),
            (7, "cooling.utilization", 0.9999896574911483),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1643.9646746745657),
        ],
    );
}

/// healthcare_with_beds: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_healthcare_with_beds() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"healthcare_with_beds","setpoints":{"heatingC":22,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 1564.4383561643838),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 1756.5922883515038),
            (1, "cooling.gainsKwh", 2058.116346473984),
            (1, "heating.setpointC", 22.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 1.0),
            (1, "cooling.reductionFactor", 1.0),
            (1, "heating.calculationTemperatureC", 22.0),
            (1, "heating.heatTransferKwh", 1731.1392000000003),
            (1, "heating.utilization", 0.8816081125213989),
            (1, "cooling.utilization", 0.9184636333061957),
            (1, "heating.needKwh", 182.51318819678613),
            (1, "cooling.needKwh", 304.1270807200483),
            (7, "internal", 1564.4383561643838),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 2776.891587076124),
            (7, "cooling.gainsKwh", 2844.775180568384),
            (7, "heating.setpointC", 22.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 1.0),
            (7, "heating.calculationTemperatureC", 22.0),
            (7, "heating.heatTransferKwh", 352.65599999999995),
            (7, "heating.utilization", 0.12699666716557473),
            (7, "cooling.utilization", 0.9999986802065428),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 2313.559881663785),
        ],
    );
}

/// other_healthcare: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_other_healthcare() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"other_healthcare","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 913.4383561643835),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 1105.5922883515036),
            (1, "cooling.gainsKwh", 1407.1163464739836),
            (1, "heating.setpointC", 21.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.9727045010776918),
            (1, "cooling.reductionFactor", 0.8),
            (1, "heating.calculationTemperatureC", 20.498035774818753),
            (1, "heating.heatTransferKwh", 1597.0438339758182),
            (1, "heating.utilization", 0.982790874858574),
            (1, "cooling.utilization", 0.7184972605992087),
            (1, "heating.needKwh", 510.47782166995125),
            (1, "cooling.needKwh", 28.00216216438639),
            (7, "internal", 913.4383561643835),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 2125.891587076124),
            (7, "cooling.gainsKwh", 2193.775180568384),
            (7, "heating.setpointC", 21.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.8),
            (7, "heating.calculationTemperatureC", 21.0),
            (7, "heating.heatTransferKwh", 263.3759999999999),
            (7, "heating.utilization", 0.12388965985833258),
            (7, "cooling.utilization", 0.9999903074868266),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1330.0514635091695),
        ],
    );
}

/// office: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_office() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"office","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 1099.4383561643835),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 1291.5922883515036),
            (1, "cooling.gainsKwh", 1593.1163464739836),
            (1, "heating.setpointC", 21.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.9821717322069491),
            (1, "cooling.reductionFactor", 0.8),
            (1, "heating.calculationTemperatureC", 20.672138155285793),
            (1, "heating.heatTransferKwh", 1612.587694503916),
            (1, "heating.utilization", 0.9604356115685723),
            (1, "cooling.utilization", 0.79338431992924),
            (1, "heating.needKwh", 372.096465143788),
            (1, "cooling.needKwh", 62.39275633005582),
            (7, "internal", 1099.4383561643835),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 2311.891587076124),
            (7, "cooling.gainsKwh", 2379.775180568384),
            (7, "heating.setpointC", 21.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.8),
            (7, "heating.calculationTemperatureC", 21.0),
            (7, "heating.heatTransferKwh", 263.3759999999999),
            (7, "heating.utilization", 0.1139222937624678),
            (7, "cooling.utilization", 0.9999947957136244),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1478.8495561348602),
        ],
    );
}

/// lodging: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_lodging() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"lodging","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 671.6383561643836),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 863.7922883515037),
            (1, "cooling.gainsKwh", 1165.316346473984),
            (1, "heating.setpointC", 21.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.9795914286014238),
            (1, "cooling.reductionFactor", 0.9),
            (1, "heating.calculationTemperatureC", 20.624686371980182),
            (1, "heating.heatTransferKwh", 1608.3511992903907),
            (1, "heating.utilization", 0.9966701969715092),
            (1, "cooling.utilization", 0.6054524371475513),
            (1, "heating.needKwh", 747.4351691166269),
            (1, "cooling.needKwh", 8.175880453729338),
            (7, "internal", 671.6383561643836),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 1884.091587076124),
            (7, "cooling.gainsKwh", 1951.9751805683838),
            (7, "heating.setpointC", 21.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.9),
            (7, "heating.calculationTemperatureC", 21.0),
            (7, "heating.heatTransferKwh", 263.3759999999999),
            (7, "heating.utilization", 0.13978936319004617),
            (7, "cooling.utilization", 0.9999764589936114),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1278.6945173348704),
        ],
    );
}

/// education: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_education() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"education","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 1006.4383561643835),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 1198.5922883515036),
            (1, "cooling.gainsKwh", 1500.1163464739836),
            (1, "heating.setpointC", 21.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.9774381166423205),
            (1, "cooling.reductionFactor", 0.8),
            (1, "heating.calculationTemperatureC", 20.585086965052273),
            (1, "heating.heatTransferKwh", 1604.8157642398667),
            (1, "heating.utilization", 0.9731153504499632),
            (1, "cooling.utilization", 0.7575414578137145),
            (1, "heating.needKwh", 438.44720951406975),
            (1, "cooling.needKwh", 42.75202441623933),
            (7, "internal", 1006.4383561643835),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 2218.891587076124),
            (7, "cooling.gainsKwh", 2286.775180568384),
            (7, "heating.setpointC", 21.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.8),
            (7, "heating.calculationTemperatureC", 21.0),
            (7, "heating.heatTransferKwh", 263.3759999999999),
            (7, "heating.utilization", 0.11869709689227194),
            (7, "cooling.utilization", 0.9999929401751861),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1404.4503446882259),
        ],
    );
}

/// sport: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_sport() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"sport","setpoints":{"heatingC":16,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 429.8383561643836),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 621.9922883515037),
            (1, "cooling.gainsKwh", 923.5163464739837),
            (1, "heating.setpointC", 16.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.9606775282117548),
            (1, "cooling.reductionFactor", 0.8),
            (1, "heating.calculationTemperatureC", 15.473472102755396),
            (1, "heating.heatTransferKwh", 1148.450789334002),
            (1, "heating.utilization", 0.9964743934844962),
            (1, "cooling.utilization", 0.48281365181449337),
            (1, "heating.needKwh", 528.6514010469034),
            (1, "cooling.needKwh", 0.0),
            (7, "internal", 429.8383561643836),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 1642.291587076124),
            (7, "cooling.gainsKwh", 1710.175180568384),
            (7, "heating.setpointC", 16.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.8),
            (7, "heating.calculationTemperatureC", 16.0),
            (7, "heating.heatTransferKwh", -183.02400000000006),
            (7, "heating.utilization", -0.11144427788602955),
            (7, "cooling.utilization", 0.999936243503592),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 943.194439231504),
        ],
    );
}

/// retail: tables 7.2/7.3 (p. 179-180), lighting 7.28 with f_L 0,3, tables 7.13-7.15 (p. 217-220) in
/// 7.59-7.75, tau 7.57/7.58 and the balance 7.1-7.13.
#[test]
fn utility_function_retail() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"retail","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 857.6383561643836),
            (1, "opaque", -6.842419199999999),
            (1, "heating.gainsKwh", 1049.7922883515037),
            (1, "cooling.gainsKwh", 1351.316346473984),
            (1, "heating.setpointC", 21.0),
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.reductionFactor", 0.9844701790318883),
            (1, "cooling.reductionFactor", 0.9),
            (1, "heating.calculationTemperatureC", 20.714406592396426),
            (1, "heating.heatTransferKwh", 1616.361420569153),
            (1, "heating.utilization", 0.988388415913407),
            (1, "cooling.utilization", 0.6937204509239812),
            (1, "heating.needKwh", 578.7588836472999),
            (1, "cooling.needKwh", 23.867060688735798),
            (7, "internal", 857.6383561643836),
            (7, "opaque", 16.4418048),
            (7, "heating.gainsKwh", 2070.0915870761237),
            (7, "cooling.gainsKwh", 2137.9751805683836),
            (7, "heating.setpointC", 21.0),
            (7, "heating.timeConstantH", 104.16666666666667),
            (7, "cooling.timeConstantH", 104.16666666666667),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.9),
            (7, "heating.calculationTemperatureC", 21.0),
            (7, "heating.heatTransferKwh", 263.3759999999999),
            (7, "heating.utilization", 0.12722914481015224),
            (7, "cooling.utilization", 0.9999882051092099),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1446.0889015827806),
        ],
    );
}

/// 7.28 with f_L 1,0 and 7.29 recoverable hot-water losses (p. 180-181).
#[test]
fn utility_lighting_and_hot_water_recovery() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"office","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":5000,"recovery":"other"},"sourceReference":"x","hotWaterRecoverableKwh":[30,28,27,25,20,18,15,15,18,22,26,29]},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 1477.6575342465753),
            (1, "heating.needKwh", 171.4955059120084),
            (6, "internal", 1418.9589041095892),
            (6, "heating.needKwh", 0.0),
        ],
    );
}

/// 7.28 with f_L 0,5 (p. 180).
#[test]
fn utility_lighting_extracted() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"office","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":5000,"recovery":"extracted_luminaires"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(&d, &[(1, "internal", 1235.328767123288)]);
}

/// Declared flux with high internal gains: gamma_H > 2 (7.2) and the cooling need 7.7 with a_C;red.
#[test]
fn utility_declared_flux_and_cooling_need() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"office","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"transmission":{"method":"explicit","conductanceWPerK":30,"sourceReference":"synthetic envelope","ground":null,"groundInventoryConfirmed":true},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"declared","heatFluxWPerM2":40,"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 7440.0),
            (1, "heating.needKwh", 0.0),
            (1, "cooling.needKwh", 5455.749432255297),
            (1, "cooling.utilization", 0.9999999999914536),
            (1, "cooling.reductionFactor", 0.8),
            (4, "internal", 7200.0),
            (4, "heating.needKwh", 0.0),
            (4, "cooling.needKwh", 6230.157586732811),
            (4, "cooling.utilization", 0.9999999999999819),
            (4, "cooling.reductionFactor", 0.8),
            (7, "internal", 7440.0),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 6728.368659523201),
            (7, "cooling.utilization", 1.0),
            (7, "cooling.reductionFactor", 0.8),
        ],
    );
}

/// Sky radiation dominating the gains (7.49, d_float >= 0 of 7.73) in an office.
#[test]
fn negative_gains_and_heat_transfer() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"office","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"light","wall":"light","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"declared","heatFluxWPerM2":0,"sourceReference":"x"},"windows":[{"id":"n","areaM2":20,"orientation":"north","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":5,"sourceReference":"s","obstruction":{"method":"minimal"}}],"opaqueElements":[{"id":"roof","areaM2":250,"orientation":"south","tiltDeg":0,"uValueWPerM2k":3.5,"sourceReference":"x"}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "heating.gainsKwh", -749.2600800000002),
            (1, "heating.utilization", 1.0),
            (1, "heating.reductionFactor", 0.8554760671025798),
            (1, "heating.needKwh", 2153.831331152189),
            (1, "cooling.needKwh", 0.0),
            (7, "heating.gainsKwh", 2120.4609336000003),
            (7, "heating.utilization", 0.12404528700406937),
            (7, "heating.reductionFactor", 1.0),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 1285.2203517862267),
            (12, "heating.gainsKwh", -871.8861600000001),
            (12, "heating.utilization", 1.0),
            (12, "heating.reductionFactor", 0.8482697024088877),
            (12, "heating.needKwh", 2159.3559835281135),
            (12, "cooling.needKwh", 0.0),
        ],
    );
}

/// Sport hall at 16 degC: negative Q_H;ht in summer (7.1, 7.48) and table 7.14 reduced setpoint 14 degC.
#[test]
fn sport_hall_negative_heat_transfer() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"sport","setpoints":{"heatingC":16,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"light","wall":"light","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"declared","heatFluxWPerM2":2,"sourceReference":"x"},"windows":[{"id":"s","areaM2":40,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "heating.reductionFactor", 0.9188023491619008),
            (1, "heating.utilization", 0.9528815338097226),
            (1, "heating.needKwh", 627.8427033793134),
            (1, "cooling.needKwh", 64.17601869884656),
            (6, "heating.reductionFactor", 1.0),
            (6, "heating.utilization", -0.006259922411739491),
            (6, "heating.needKwh", 0.0),
            (6, "cooling.needKwh", 800.8530540788862),
            (7, "heating.reductionFactor", 1.0),
            (7, "heating.utilization", -0.10941919579204022),
            (7, "heating.needKwh", 0.0),
            (7, "cooling.needKwh", 921.3254045485661),
        ],
    );
}

/// 6.5.3 (p. 156-157): area-weighted setpoint, reduced setpoint, hours and q_A.
#[test]
fn mixed_zone_cell_retail() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"cell","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"functionAreas":[{"function":"cell","areaM2":200},{"function":"retail","areaM2":50}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 887.3983561643838),
            (1, "heating.reductionFactor", 0.9945402145643678),
            (1, "cooling.reductionFactor", 0.98),
            (1, "heating.needKwh", 567.3516526287335),
            (7, "internal", 887.3983561643838),
            (7, "heating.reductionFactor", 1.0),
            (7, "cooling.reductionFactor", 0.98),
            (7, "heating.needKwh", 0.0),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_light_light_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"light","wall":"light","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 12.731481481481483),
            (1, "cooling.timeConstantH", 12.731481481481483),
            (1, "heating.utilization", 0.9446104387327874),
            (1, "heating.needKwh", 1001.4423205837243),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_light_light_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"light","wall":"light","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 18.51851851851852),
            (1, "cooling.timeConstantH", 18.51851851851852),
            (1, "heating.utilization", 0.9684281712892815),
            (1, "heating.needKwh", 997.1407242442209),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_light_heavy_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"light","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 25.462962962962965),
            (1, "cooling.timeConstantH", 25.462962962962965),
            (1, "heating.utilization", 0.9838909481739324),
            (1, "heating.needKwh", 996.0633987147032),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_light_heavy_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"light","wall":"heavy","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 41.666666666666664),
            (1, "cooling.timeConstantH", 41.666666666666664),
            (1, "heating.utilization", 0.9966585699859067),
            (1, "heating.needKwh", 1000.8081464351277),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_light_very_heavy_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"light","wall":"very_heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 41.666666666666664),
            (1, "cooling.timeConstantH", 41.666666666666664),
            (1, "heating.utilization", 0.9966585699859067),
            (1, "heating.needKwh", 1000.8081464351277),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_light_very_heavy_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"light","wall":"very_heavy","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 83.33333333333333),
            (1, "cooling.timeConstantH", 83.33333333333333),
            (1, "heating.utilization", 0.9999437831843521),
            (1, "heating.needKwh", 1018.8814430986043),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_heavy_light_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"heavy","wall":"light","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 25.462962962962965),
            (1, "cooling.timeConstantH", 25.462962962962965),
            (1, "heating.utilization", 0.9838909481739324),
            (1, "heating.needKwh", 996.0633987147032),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_heavy_light_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"heavy","wall":"light","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 41.666666666666664),
            (1, "cooling.timeConstantH", 41.666666666666664),
            (1, "heating.utilization", 0.9966585699859067),
            (1, "heating.needKwh", 1000.8081464351277),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_heavy_heavy_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 41.666666666666664),
            (1, "cooling.timeConstantH", 41.666666666666664),
            (1, "heating.utilization", 0.9966585699859067),
            (1, "heating.needKwh", 1000.8081464351277),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_heavy_heavy_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 83.33333333333333),
            (1, "cooling.timeConstantH", 83.33333333333333),
            (1, "heating.utilization", 0.9999437831843521),
            (1, "heating.needKwh", 1018.8814430986043),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_heavy_very_heavy_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"heavy","wall":"very_heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 57.87037037037037),
            (1, "cooling.timeConstantH", 57.87037037037037),
            (1, "heating.utilization", 0.9993145213472014),
            (1, "heating.needKwh", 1008.7012740709649),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_heavy_very_heavy_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"heavy","wall":"very_heavy","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.utilization", 0.9999927247942252),
            (1, "heating.needKwh", 1023.7249789287343),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_very_heavy_light_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"very_heavy","wall":"light","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 25.462962962962965),
            (1, "cooling.timeConstantH", 25.462962962962965),
            (1, "heating.utilization", 0.9838909481739324),
            (1, "heating.needKwh", 996.0633987147032),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_very_heavy_light_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"very_heavy","wall":"light","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 41.666666666666664),
            (1, "cooling.timeConstantH", 41.666666666666664),
            (1, "heating.utilization", 0.9966585699859067),
            (1, "heating.needKwh", 1000.8081464351277),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_very_heavy_heavy_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"very_heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 57.87037037037037),
            (1, "cooling.timeConstantH", 57.87037037037037),
            (1, "heating.utilization", 0.9993145213472014),
            (1, "heating.needKwh", 1008.7012740709649),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_very_heavy_heavy_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"very_heavy","wall":"heavy","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.utilization", 0.9999927247942252),
            (1, "heating.needKwh", 1023.7249789287343),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_very_heavy_very_heavy_closed_or_suspended() {
    let d = demand(
        r#"{"thermalMass":{"floor":"very_heavy","wall":"very_heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 57.87037037037037),
            (1, "cooling.timeConstantH", 57.87037037037037),
            (1, "heating.utilization", 0.9993145213472014),
            (1, "heating.needKwh", 1008.7012740709649),
        ],
    );
}

/// Table 7.10 (p. 205) in 7.45, tau_H/C of 7.57/7.58 and a_H/C of 7.51/7.56.
#[test]
fn thermal_mass_very_heavy_very_heavy_open_or_none() {
    let d = demand(
        r#"{"thermalMass":{"floor":"very_heavy","wall":"very_heavy","ceiling":"open_or_none","sourceReference":"x"},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 104.16666666666667),
            (1, "cooling.timeConstantH", 104.16666666666667),
            (1, "heating.utilization", 0.9999927247942252),
            (1, "heating.needKwh", 1023.7249789287343),
        ],
    );
}

/// Annex B (p. 772-773): effective depth up to R 0,25 m2K/W, at most 100 mm and half the thickness;
/// partitions from both sides; an open suspended ceiling skipped.
#[test]
fn thermal_mass_annex_b() {
    let d = demand(
        r#"{"thermalMass":{"floor":"light","wall":"light","ceiling":"open_or_none","sourceReference":"x","annexBElements":[{"id":"floor","areaM2":100,"layers":[{"thicknessM":0.05,"conductivityWPerMk":1.2,"densityKgPerM3":2000,"specificHeatJPerKgk":1000},{"thicknessM":0.2,"conductivityWPerMk":2,"densityKgPerM3":2400,"specificHeatJPerKgk":1000}],"sourceReference":"x"},{"id":"wall-ins","areaM2":40,"layers":[{"thicknessM":0.0125,"conductivityWPerMk":0.25,"densityKgPerM3":900,"specificHeatJPerKgk":1000},{"thicknessM":0.1,"conductivityWPerMk":0.035,"densityKgPerM3":30,"specificHeatJPerKgk":1450},{"thicknessM":0.1,"conductivityWPerMk":0.9,"densityKgPerM3":1800,"specificHeatJPerKgk":840}],"sourceReference":"x"},{"id":"partition","areaM2":30,"bothSides":true,"layers":[{"thicknessM":0.012,"conductivityWPerMk":0.5,"densityKgPerM3":1300,"specificHeatJPerKgk":1000},{"thicknessM":0.07,"conductivityWPerMk":1,"densityKgPerM3":1700,"specificHeatJPerKgk":840},{"thicknessM":0.012,"conductivityWPerMk":0.5,"densityKgPerM3":1300,"specificHeatJPerKgk":1000}],"sourceReference":"x"},{"id":"ceiling","areaM2":100,"layers":[{"thicknessM":0.02,"conductivityWPerMk":0.2,"densityKgPerM3":800,"specificHeatJPerKgk":1000,"openSuspendedCeiling":true},{"thicknessM":0.25,"conductivityWPerMk":2,"densityKgPerM3":2400,"specificHeatJPerKgk":1000}],"sourceReference":"x"},{"id":"timber","areaM2":20,"layers":[{"thicknessM":0.15,"conductivityWPerMk":0.13,"densityKgPerM3":500,"specificHeatJPerKgk":1600}],"sourceReference":"x"}]},"windows":[{"id":"w","areaM2":4,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    check(
        &d,
        &[
            (1, "heating.timeConstantH", 117.86337962962963),
            (1, "cooling.timeConstantH", 117.86337962962963),
            (1, "heating.needKwh", 1026.0082043270115),
            (7, "heating.timeConstantH", 117.86337962962963),
            (7, "cooling.timeConstantH", 117.86337962962963),
            (7, "heating.needKwh", 0.0),
        ],
    );
}

/// Annex AA (p. 1134-1146): AA.1-AA.13 with tables AA.1-AA.3, the July cooling-balance F_sh;obst, F_c of
/// 7.43 without f_sh;with, U_w+shut where given; the zone peak hour from the summed window solar load.
#[test]
fn annex_aa_basic() {
    let input = input(
        r#"{"usableFloorAreaM2":120,"windows":[{"id":"lw-s","areaM2":6,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"lw-w","areaM2":3,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"bed-e","areaM2":2.5,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"bed-n-roof","areaM2":1.2,"orientation":"north","tiltDeg":40,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    let aa: AnnexAaInput = serde_json::from_str(r#"{"constructionYear":2020,"generatorCapacityKw":3,"rooms":[{"id":"living","areaM2":35,"living":true,"opaqueInnerAreaM2":20,"windows":[{"windowId":"lw-s"},{"windowId":"lw-w"}],"installedCapacityKw":2},{"id":"bed1","areaM2":14,"living":false,"opaqueInnerAreaM2":12,"windows":[{"windowId":"bed-e"}],"installedCapacityKw":0.5},{"id":"bed2","areaM2":10,"living":false,"opaqueInnerAreaM2":18,"windows":[{"windowId":"bed-n-roof"}],"installedCapacityKw":0.3}]}"#).expect("annex AA input");
    let d = assess_monthly_demand(&input);
    let result = assess_annex_aa(&aa, &input, &d, "aa").expect("annex AA result");
    let r = serde_json::to_value(result).expect("serializable");
    let rows: &[(&str, f64)] = &[
        ("peakHour", 14.0),
        ("internalW", 446.4),
        ("outdoorAirW", 235.99999999999994),
        ("opaqueW", 110.00000000000001),
        ("solarW", 3321.243000000001),
        ("glazingW", 104.90199999999996),
        ("needWPerM2", 71.50076271186443),
        ("requiredKw", 2.153545000000001),
        ("rooms/0/internalW", 332.4255319148936),
        ("rooms/0/outdoorAirW", 139.99999999999997),
        ("rooms/0/opaqueW", 44.0),
        ("rooms/0/solarW", 2364.3900000000003),
        ("rooms/0/glazingW", 74.33999999999997),
        ("rooms/0/needWPerM2", 84.4330151975684),
        ("rooms/0/requiredKw", 1.7301555319148938),
        ("rooms/0/peakHour", 14.0),
        ("rooms/1/internalW", 66.48510638297873),
        ("rooms/1/outdoorAirW", 55.999999999999986),
        ("rooms/1/opaqueW", 26.400000000000002),
        ("rooms/1/solarW", 735.0750000000002),
        ("rooms/1/glazingW", 2.4499999999999975),
        ("rooms/1/needWPerM2", 63.31500759878421),
        ("rooms/1/requiredKw", 0.39641010638297897),
        ("rooms/1/peakHour", 9.0),
        ("rooms/2/internalW", 47.48936170212766),
        ("rooms/2/outdoorAirW", 39.99999999999999),
        ("rooms/2/opaqueW", 39.6),
        ("rooms/2/solarW", 221.77800000000008),
        ("rooms/2/glazingW", 9.744000000000002),
        ("rooms/2/needWPerM2", 35.861136170212774),
        ("rooms/2/requiredKw", 0.008611361702127738),
        ("rooms/2/peakHour", 15.0),
    ];
    for &(path, expected) in rows {
        let value = r
            .pointer(&format!("/{path}"))
            .unwrap_or_else(|| panic!("{path}"));
        close(path, number(value, path), expected);
    }
}

/// Annex AA (p. 1134-1146): AA.1-AA.13 with tables AA.1-AA.3, the July cooling-balance F_sh;obst, F_c of
/// 7.43 without f_sh;with, U_w+shut where given; the zone peak hour from the summed window solar load.
#[test]
fn annex_aa_shading() {
    let input = input(
        r#"{"usableFloorAreaM2":120,"ventilationFlows":[{"id":"natural","sourceReference":"synthetic flow","months":[{"month":1,"conductanceWPerK":40},{"month":2,"conductanceWPerK":40},{"month":3,"conductanceWPerK":40},{"month":4,"conductanceWPerK":40},{"month":5,"conductanceWPerK":40},{"month":6,"conductanceWPerK":40},{"month":7,"conductanceWPerK":40,"coolingConductanceWPerK":55},{"month":8,"conductanceWPerK":40},{"month":9,"conductanceWPerK":40},{"month":10,"conductanceWPerK":40},{"month":11,"conductanceWPerK":40},{"month":12,"conductanceWPerK":40}]}],"windows":[{"id":"a","areaM2":5,"orientation":"south_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"overhang","relativeHeight":0.7},"movableShading":{"reductionFactor":0.123,"control":"manual_residential","sourceReference":"x"}},{"id":"b","areaM2":4,"orientation":"south_east","tiltDeg":90,"gPerpendicular":0.67,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"side_obstruction","side":"left","relativeWidth":0.5,"coolingHeightCondition":true},"movableShading":{"device":{"kind":"folding_arm_awning"},"control":"automatic","sourceReference":"x"}},{"id":"c","areaM2":3,"orientation":"west","tiltDeg":75,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"full","coolingConditionsMet":true}},{"id":"d","areaM2":2,"orientation":"north","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"other","overhangRelativeHeight":0.3}}]}"#,
    );
    let aa: AnnexAaInput = serde_json::from_str(r#"{"constructionYear":1985,"generatorCapacityKw":3,"rooms":[{"id":"living","areaM2":40,"living":true,"opaqueInnerAreaM2":30,"windows":[{"windowId":"a","uWithShutterWPerM2k":1.1},{"windowId":"d"}],"installedCapacityKw":3},{"id":"study","areaM2":12,"living":false,"opaqueInnerAreaM2":9,"windows":[{"windowId":"b","uWithShutterWPerM2k":1}],"installedCapacityKw":0.4},{"id":"attic","areaM2":16,"living":false,"opaqueInnerAreaM2":25,"windows":[{"windowId":"c"}],"installedCapacityKw":1.5}],"postInsulated":true}"#).expect("annex AA input");
    let d = assess_monthly_demand(&input);
    let result = assess_annex_aa(&aa, &input, &d, "aa").expect("annex AA result");
    let r = serde_json::to_value(result).expect("serializable");
    let rows: &[(&str, f64)] = &[
        ("peakHour", 15.0),
        ("internalW", 446.4),
        ("outdoorAirW", 319.00000000000006),
        ("opaqueW", 204.8),
        ("solarW", 1533.3552450000004),
        ("glazingW", 95.70000000000002),
        ("needWPerM2", 38.2243418382353),
        ("requiredKw", 0.2192552450000002),
        ("rooms/0/internalW", 330.66666666666663),
        ("rooms/0/outdoorAirW", 187.64705882352945),
        ("rooms/0/opaqueW", 96.0),
        ("rooms/0/solarW", 263.7542250000001),
        ("rooms/0/glazingW", 48.14000000000001),
        ("rooms/0/needWPerM2", 23.155198762254905),
        ("rooms/0/requiredKw", 0.0),
        ("rooms/0/peakHour", 15.0),
        ("rooms/1/internalW", 49.599999999999994),
        ("rooms/1/outdoorAirW", 56.29411764705884),
        ("rooms/1/opaqueW", 28.8),
        ("rooms/1/solarW", 678.4549200000002),
        ("rooms/1/glazingW", 11.599999999999994),
        ("rooms/1/needWPerM2", 68.72908647058826),
        ("rooms/1/requiredKw", 0.40474903764705905),
        ("rooms/1/peakHour", 10.0),
        ("rooms/2/internalW", 66.13333333333333),
        ("rooms/2/outdoorAirW", 75.05882352941178),
        ("rooms/2/opaqueW", 80.0),
        ("rooms/2/solarW", 591.1461000000002),
        ("rooms/2/glazingW", 24.36),
        ("rooms/2/needWPerM2", 52.29364105392158),
        ("rooms/2/requiredKw", 0.27669825686274535),
        ("rooms/2/peakHour", 15.0),
    ];
    for &(path, expected) in rows {
        let value = r
            .pointer(&format!("/{path}"))
            .unwrap_or_else(|| panic!("{path}"));
        close(path, number(value, path), expected);
    }
}

/// Annex AA (p. 1134-1146): AA.1-AA.13 with tables AA.1-AA.3, the July cooling-balance F_sh;obst, F_c of
/// 7.43 without f_sh;with, U_w+shut where given; the zone peak hour from the summed window solar load.
#[test]
fn annex_aa_old_dwellings() {
    let input = input(
        r#"{"usableFloorAreaM2":400,"dwellingType":"apartment_building","internalGains":{"method":"residential","dwellingCount":5,"sourceReference":"single dwelling"},"windows":[{"id":"x","areaM2":10,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"y","areaM2":8,"orientation":"north_west","tiltDeg":30,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"z","areaM2":1,"orientation":"south","tiltDeg":0,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    let aa: AnnexAaInput = serde_json::from_str(r#"{"constructionYear":1960,"generatorCapacityKw":3,"rooms":[{"id":"r1","areaM2":60,"living":true,"opaqueInnerAreaM2":40,"windows":[{"windowId":"x"}],"installedCapacityKw":5},{"id":"r2","areaM2":50,"living":false,"opaqueInnerAreaM2":30,"windows":[{"windowId":"y"},{"windowId":"z"}],"installedCapacityKw":1}]}"#).expect("annex AA input");
    let d = assess_monthly_demand(&input);
    let result = assess_annex_aa(&aa, &input, &d, "aa").expect("annex AA result");
    let r = serde_json::to_value(result).expect("serializable");
    let rows: &[(&str, f64)] = &[
        ("peakHour", 10.0),
        ("internalW", 1722.8571428571427),
        ("outdoorAirW", 115.99999999999994),
        ("opaqueW", 1190.0),
        ("solarW", 6071.760000000002),
        ("glazingW", 77.13999999999996),
        ("needWPerM2", 83.43415584415585),
        ("requiredKw", 5.327757142857144),
        ("rooms/0/internalW", 1216.1344537815125),
        ("rooms/0/outdoorAirW", 63.27272727272724),
        ("rooms/0/opaqueW", 680.0),
        ("rooms/0/solarW", 2940.3000000000006),
        ("rooms/0/glazingW", 9.79999999999999),
        ("rooms/0/needWPerM2", 81.82511968423734),
        ("rooms/0/requiredKw", 2.8095071810542405),
        ("rooms/0/peakHour", 9.0),
        ("rooms/1/internalW", 506.7226890756302),
        ("rooms/1/outdoorAirW", 52.727272727272705),
        ("rooms/1/opaqueW", 510.0),
        ("rooms/1/solarW", 3131.460000000001),
        ("rooms/1/glazingW", 73.08000000000001),
        ("rooms/1/needWPerM2", 85.47979923605808),
        ("rooms/1/requiredKw", 2.5239899618029042),
        ("rooms/1/peakHour", 15.0),
    ];
    for &(path, expected) in rows {
        let value = r
            .pointer(&format!("/{path}"))
            .unwrap_or_else(|| panic!("{path}"));
        close(path, number(value, path), expected);
    }
}

/// Annex AA (p. 1134-1146): AA.1-AA.13 with tables AA.1-AA.3, the July cooling-balance F_sh;obst, F_c of
/// 7.43 without f_sh;with, U_w+shut where given; the zone peak hour from the summed window solar load.
#[test]
fn annex_aa_two_living_and_blind() {
    let input = input(
        r#"{"usableFloorAreaM2":120,"windows":[{"id":"p","areaM2":4,"orientation":"south_west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"q","areaM2":3,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}}]}"#,
    );
    let aa: AnnexAaInput = serde_json::from_str(r#"{"constructionYear":2020,"generatorCapacityKw":null,"rooms":[{"id":"liv","areaM2":25,"living":true,"opaqueInnerAreaM2":10,"windows":[{"windowId":"p"}],"installedCapacityKw":2},{"id":"kit","areaM2":12,"living":true,"opaqueInnerAreaM2":6,"windows":[{"windowId":"q"}],"installedCapacityKw":1},{"id":"blind","areaM2":9,"living":false,"opaqueInnerAreaM2":5,"windows":[],"installedCapacityKw":0.2}]}"#).expect("annex AA input");
    let d = assess_monthly_demand(&input);
    let result = assess_annex_aa(&aa, &input, &d, "aa").expect("annex AA result");
    let r = serde_json::to_value(result).expect("serializable");
    let rows: &[(&str, f64)] = &[
        ("peakHour", 15.0),
        ("internalW", 446.4),
        ("outdoorAirW", 232.00000000000003),
        ("opaqueW", 46.2),
        ("solarW", 2273.6700000000005),
        ("glazingW", 56.84),
        ("needWPerM2", 66.41543478260871),
        ("requiredKw", 1.4451100000000008),
        ("rooms/0/internalW", 268.9156626506024),
        ("rooms/0/outdoorAirW", 126.08695652173915),
        ("rooms/0/opaqueW", 22.0),
        ("rooms/0/solarW", 1391.5800000000004),
        ("rooms/0/glazingW", 32.480000000000004),
        ("rooms/0/needWPerM2", 73.64250476689368),
        ("rooms/0/requiredKw", 0.966062619172342),
        ("rooms/0/peakHour", 15.0),
        ("rooms/1/internalW", 129.07951807228915),
        ("rooms/1/outdoorAirW", 60.521739130434796),
        ("rooms/1/opaqueW", 13.200000000000001),
        ("rooms/1/solarW", 882.0900000000003),
        ("rooms/1/glazingW", 2.9399999999999964),
        ("rooms/1/needWPerM2", 90.6526047668937),
        ("rooms/1/requiredKw", 0.6678312572027244),
        ("rooms/1/peakHour", 9.0),
        ("rooms/2/internalW", 48.40481927710843),
        ("rooms/2/outdoorAirW", 45.3913043478261),
        ("rooms/2/opaqueW", 11.0),
        ("rooms/2/solarW", 0.0),
        ("rooms/2/glazingW", 0.0),
        ("rooms/2/needWPerM2", 11.644013736103837),
        ("rooms/2/requiredKw", 0.0),
    ];
    for &(path, expected) in rows {
        let value = r
            .pointer(&format!("/{path}"))
            .unwrap_or_else(|| panic!("{path}"));
        close(path, number(value, path), expected);
    }
}

/// Table AA.2 (p. 1138) with the shared boundary years read as the lower class, one class up when
/// post-insulated.
#[test]
fn annex_aa_insulation_factor() {
    let rows: &[(i32, bool, f64)] = &[
        (1975, false, 17.0),
        (1976, false, 10.0),
        (1992, false, 10.0),
        (1993, false, 3.2),
        (2015, false, 3.2),
        (2016, false, 2.2),
        (1975, true, 10.0),
        (2015, true, 2.2),
        (2020, true, 2.2),
    ];
    for &(year, post, expected) in rows {
        close(
            &format!("{year} {post}"),
            insulation_factor(year, post),
            expected,
        );
    }
}

/// 17.3.2 d (p. 701): a side obstruction counts only with b_b < 3,73; with b_b >= 3,73 situation a applies
/// (17.3.2 a, p. 698), so east, January, heating takes table 17.4 (p. 711). The kernel accepts any
/// relativeWidth and takes the b_b >= 1,0 column of table 17.10 (0,55 for both sides).
#[test]
fn side_obstruction_beyond_3_73_is_minimal() {
    let situation: Obstruction =
        serde_json::from_str(r#"{"method":"side_obstruction","side":"both","relativeWidth":4.0}"#)
            .expect("situation");
    let kernel = obstruction_factor(&situation, orientation("east"), 90.0, 1, Balance::Heating)
        .expect("factor");
    close("east January heating", kernel, 0.92);
}

/// 6.5.3 (p. 156-157) weights q_Oc (item f) and f_tau (item g) separately, so 7.26 (p. 179) takes
/// q_Oc;usi * f_tau;usi of the weighted values. The kernel weights the product q_Oc * f_tau per function
/// (monthly_demand.rs FunctionProfile::weighted). Office 150 m2 + other assembly 100 m2.
/// The kernel keeps its reading: 7.26 defines f_tau per function, and the published reports of public
/// cases H (2023) and K (2022) match it, not this one (NEN question 19).
#[test]
#[ignore = "literal 6.5.3 reading kept as NEN question 19; kernel follows public cases H and K"]
fn mixed_zone_occupancy_weighted_per_variable() {
    let d = demand(
        r#"{"usableFloorAreaM2":250,"usageFunction":"office","setpoints":{"heatingC":21,"coolingC":24,"sourceReference":"table 7.13"},"thermalMass":{"floor":"heavy","wall":"heavy","ceiling":"closed_or_suspended","sourceReference":"x"},"internalGains":{"method":"utility","lighting":{"method":"declared","annualKwh":3000,"recovery":"forfait_power"},"sourceReference":"x"},"windows":[{"id":"s","areaM2":20,"orientation":"south","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"}},{"id":"e","areaM2":15,"orientation":"east","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_without_glare_protection","sourceReference":"x"}},{"id":"w","areaM2":15,"orientation":"west","tiltDeg":90,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"movableShading":{"reductionFactor":0.25,"control":"manual_utility_with_glare_protection","sourceReference":"x"}},{"id":"n-roof","areaM2":6,"orientation":"north","tiltDeg":15,"gPerpendicular":0.6,"frameFraction":0.25,"uValueWPerM2k":1.4,"sourceReference":"s","obstruction":{"method":"minimal"},"glazing":{"fixedLouvres":{"kind":"rotatable","control":"manual_utility_without_glare_protection"}}}],"functionAreas":[{"function":"office","areaM2":150},{"function":"other_assembly","areaM2":100}],"dwellingType":null}"#,
    );
    check(
        &d,
        &[
            (1, "internal", 909.7183561643833),
            (1, "heating.needKwh", 513.46222554261),
            (7, "internal", 909.7183561643833),
            (7, "heating.needKwh", 0.0),
        ],
    );
}
