//! Review 75: transmission routes outside the ground route that the two example projects do not reach,
//! recalculated independently from NTA 8800:2025+C1:2026: layered and composite constructions with the ΔU
//! corrections (8.4–8.13, annex C), windows and doors (8.14–8.23, tables 8.3, 8.4, G.1, L.1/L.2), ΔU_for
//! (8.3) and forfait ψ (table I.1), unheated spaces with a calculated or declared b_U (8.52–8.59), the forfait
//! route with H_U;for = 0 (8.4.1 note 5, C.10), adjacent heated spaces (8.5), adjacent unheated sunrooms
//! (7.30b, 7.34–7.38) and the TOjuli split per orientation (5.7.2 steps A and 1–5). The expected values come
//! from a separate implementation written from the norm pages; only numbers and page references are kept here.

use nta8800_core::envelope_elements::{assess_envelope, EnvelopeInput};
use nta8800_core::monthly_demand::{assess_monthly_demand, MonthlyDemandInput};
use nta8800_core::project_performance::assess_project_performance;
use nta8800_core::unheated_transmission::{
    assess_unheated_transmission, UnheatedTransmissionInput,
};
use serde_json::{json, Value};

const DWELLING: &str =
    include_str!("../../../training-data/nta8800-example-terraced-dwelling.json");

/// Table 17.1 (p. 690): t_mi in h and θ_e;avg;mi in °C.
const HOURS: [f64; 12] = [
    744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0,
];
const OUTDOOR_C: [f64; 12] = [
    2.61, 4.82, 5.91, 9.32, 14.73, 16.12, 18.05, 18.48, 15.63, 10.40, 7.99, 4.00,
];
/// θ_e;avg;an (17.2).
const ANNUAL_OUTDOOR_C: f64 = 10.67;

fn close(label: &str, kernel: f64, expected: f64) {
    let diff = (kernel - expected).abs();
    let scale = kernel.abs().max(expected.abs()).max(1e-12);
    assert!(
        diff <= 1e-9 || diff / scale <= 1e-6,
        "{label}: kernel {kernel} independent {expected}"
    );
}

fn num(value: &Value, label: &str) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| panic!("{label}: not a number: {value}"))
}

fn envelope(input: Value) -> Value {
    let input: EnvelopeInput = serde_json::from_value(input).expect("envelope input");
    let result = serde_json::to_value(assess_envelope(&input)).expect("serializable");
    assert_eq!(result["status"], "calculated_unverified", "{result}");
    result
}

fn element<'a>(result: &'a Value, id: &str) -> &'a Value {
    result["elements"]
        .as_array()
        .expect("elements")
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("element {id}"))
}

fn check(result: &Value, id: &str, part: &str, expected: &[(&str, f64)]) {
    let item = element(result, id);
    for (field, value) in expected {
        let kernel = if part.is_empty() {
            &item[*field]
        } else {
            &item[part][*field]
        };
        close(&format!("{id}.{field}"), num(kernel, field), *value);
    }
}

fn material(thickness: f64, lambda: f64) -> Value {
    json!({"kind": "material", "thicknessM": thickness,
        "conductivity": {"method": "calculated", "lambdaCalc": lambda, "sourceReference": "r75"}})
}

fn resistance(r: f64, thickness: f64) -> Value {
    json!({"kind": "resistance", "resistanceM2KPerW": r, "thicknessM": thickness, "sourceReference": "r75"})
}

fn opaque(id: &str, construction: Value) -> Value {
    json!({"id": id, "element": {"kind": "opaque", "construction": construction}})
}

/// Timber-frame-like composite of table C.1: board 12 mm λ 0,17, middle layer 140 mm (λ of the first section
/// or 0,035), board 18 mm λ 0,10; the first section takes the area fraction `share`.
fn composite(interruption: &str, lambda: f64, share: f64) -> Value {
    let layers = |middle: f64| {
        json!([
            material(0.012, 0.17),
            material(0.14, middle),
            material(0.018, 0.1)
        ])
    };
    json!({"heatFlow": "horizontal", "build": {"kind": "composite", "interruption": interruption, "sections": [
        {"id": "s1", "area": share, "layers": layers(lambda)},
        {"id": "s2", "area": 1.0 - share, "layers": layers(0.035)}]}})
}

/// C.3 with table C.2 (p. 773, 778), table C.3 (p. 782), C.12, C.15 (p. 785–786), 8.4–8.13 with table 8.2
/// (p. 228–235) and the 3 % rule, R_C of C.2 with β of C.8 (p. 775), R_eq of C.9/C.10 (p. 777).
#[test]
fn layered_constructions_with_corrections() {
    let cavity = |kind: &str| json!({"kind": "air_cavity", "thicknessMm": 40, "ventilation": {"kind": kind}});
    let result = envelope(json!({"elements": [
        opaque("wall", json!({"heatFlow": "horizontal", "build": {"kind": "homogeneous", "layers": [
            material(0.0125, 0.25), material(0.1, 1.0), resistance(4.5, 0.12), cavity("unventilated"), material(0.1, 0.9)]},
            "corrections": {"airVoids": {"level": "weak", "insulationLayer": 2},
                "fasteners": {"fasteners": {"method": "formula", "countPerM2": 4, "lambdaWPerMK": 17,
                    "crossSectionM2": 1.257e-5, "penetrationDepthM": 0.12, "insulationThicknessM": 0.12},
                    "insulationLayer": 2}}})),
        opaque("inverted", json!({"heatFlow": "upward", "build": {"kind": "homogeneous", "layers": [
            material(0.2, 2.0), resistance(3.0, 0.12)]},
            "corrections": {"invertedRoof": {"drainage": "xps_straight_edges", "insulationLayer": 1}}})),
        opaque("strongly", json!({"heatFlow": "horizontal", "build": {"kind": "homogeneous", "layers": [
            material(0.1, 1.0), resistance(3.5, 0.12), cavity("strongly"), material(0.1, 0.9)]}})),
        opaque("weakly", json!({"heatFlow": "horizontal", "build": {"kind": "homogeneous", "layers": [
            material(0.1, 1.0), resistance(3.5, 0.12), cavity("weakly"), material(0.1, 0.9)]}})),
        opaque("attic", json!({"heatFlow": "upward", "build": {"kind": "homogeneous", "layers": [
            material(0.0125, 0.25), resistance(5.0, 0.2),
            {"kind": "ventilated_unheated_space", "separationAreaM2": 40,
             "envelopeParts": [{"areaM2": 30, "uValueWPerM2k": 1.5}, {"areaM2": 12}],
             "airChangeRate": 0.5, "volumeM3": 60}]}})),
        opaque("to_unheated", json!({"heatFlow": "downward", "build": {"kind": "homogeneous", "layers": [
            material(0.2, 2.0), resistance(2.5, 0.1)]}, "unheatedReductionFactor": 0.6})),
        opaque("below_three_percent", json!({"heatFlow": "horizontal", "build": {"kind": "homogeneous", "layers": [
            material(0.2, 0.1), resistance(1.0, 0.05)]},
            "corrections": {"airVoids": {"level": "weak", "insulationLayer": 1}}})),
        opaque("strong_voids_point_fasteners", json!({"heatFlow": "downward", "build": {"kind": "homogeneous", "layers": [
            material(0.2, 2.0), resistance(3.2, 0.13)]},
            "corrections": {"airVoids": {"level": "strong", "insulationLayer": 1},
                "fasteners": {"fasteners": {"method": "point_bridge", "countPerM2": 6, "chiWPerK": 0.002,
                    "sourceReference": "r75"}, "insulationLayer": 1}}})),
        {"id": "rooflight", "projectedAreaM2": 1.2, "element": {"kind": "rooflight", "uRcWPerM2K": 2.4,
            "areaWithUpstandM2": 1.69, "sourceReference": "r75"}},
        {"id": "grille", "element": {"kind": "ventilation_grille"}},
    ]}));
    // R_T 0,13 + ΣR + 0,18 (table C.3) + 0,04; ΔU_a 8.9 and ΔU_fa 8.11/8.12 both above 3 % of U_T.
    check(
        &result,
        "wall",
        "opaque",
        &[
            ("rT", 5.111111111111111),
            ("uT", 0.1956521739130435),
            ("deltaUAirVoids", 0.007751654064272212),
            ("deltaUFasteners", 0.004417202551984878),
            ("deltaU", 0.01216885661625709),
            ("uC", 0.20782103052930057),
            ("rC", 4.641832553486498),
            ("uCRounded", 0.21),
            ("rCRounded", 4.64),
        ],
    );
    // 8.13: p = 2,105, f·x = 0,04, R_si 0,10.
    check(
        &result,
        "inverted",
        "opaque",
        &[
            ("rT", 3.24),
            ("deltaUInvertedRoof", 0.07218792866941014),
            ("deltaU", 0.07218792866941014),
            ("uC", 0.3808299039780521),
            ("rC", 2.485844214317875),
        ],
    );
    // C.3.3 and table C.3 note b: outer leaf and cavity left out, still-air R_se 0,12; R_C with R_se 0,04.
    check(
        &result,
        "strongly",
        "opaque",
        &[
            ("rT", 3.85),
            ("uC", 0.2597402597402597),
            ("rC", 3.68),
            ("exteriorSurfaceResistance", 0.12),
        ],
    );
    // Table C.3, weakly ventilated at A_V = 1 000 mm²/m: 0,15.
    check(
        &result,
        "weakly",
        "opaque",
        &[("rT", 4.031111111111111), ("uC", 0.24807056229327457)],
    );
    // C.15 with U_e;k = 2 for the part without a U (p. 786): R = 40/(45 + 24 + 0,33·0,5·60).
    check(
        &result,
        "attic",
        "opaque",
        &[
            ("rT", 5.6969708491761715),
            ("uC", 0.1755318793924684),
            ("rC", 5.556970849176172),
        ],
    );
    check(
        &result,
        "to_unheated",
        "opaque",
        &[
            ("rT", 2.81),
            ("uC", 0.35587188612099646),
            ("rC", 2.6),
            ("rEquivalent", 4.473333333333334),
        ],
    );
    // ΔU_a = 0,001 < 3 % of U_T: ΔU = 0.
    check(
        &result,
        "below_three_percent",
        "opaque",
        &[
            ("rT", 3.17),
            ("deltaUAirVoids", 0.000995133795738837),
            ("deltaU", 0.0),
            ("uC", 0.31545741324921134),
        ],
    );
    // Table 8.2 level 2 (0,04) and 8.10 n·χ.
    check(
        &result,
        "strong_voids_point_fasteners",
        "opaque",
        &[
            ("rT", 3.51),
            ("deltaUAirVoids", 0.03324648338893353),
            ("deltaUFasteners", 0.012),
            ("deltaU", 0.04524648338893353),
            ("uC", 0.3301467682892184),
            ("rC", 2.8189558949247995),
        ],
    );
    // 8.2.2.1 note 2: U_C = U_rc·A_rc/A_T.
    check(&result, "rooflight", "", &[("uUnrounded", 3.38)]);
    check(
        &result,
        "grille",
        "",
        &[("uUnrounded", 6.2), ("uValue", 6.2)],
    );
}

/// C.4–C.7 with table C.1 (p. 774–776): R_T' over the sections, R_T'' with λ_j'' to three decimals, a'.
#[test]
fn composite_constructions_table_c1() {
    let result = envelope(json!({"elements": [
        opaque("woody", composite("woody_unshielded", 0.18, 0.12)),
        opaque("other_close_bounds", composite("other", 0.13, 0.1)),
        opaque("metal", composite("metal_one_side_shielded", 50.0, 0.01)),
        opaque("stony", composite("stony_unshielded", 0.9, 0.1)),
        opaque("woody_close_bounds", composite("woody_unshielded", 0.04, 0.5)),
    ]}));
    // λ'' of the middle layer 0,0524 → 0,052 (C.7); R_T' > 1,05·R_T'' so a' = 0,5.
    check(
        &result,
        "woody",
        "opaque",
        &[
            ("rTUpper", 3.3421914681504945),
            ("rTLower", 3.1128959276018104),
            ("weightingFactor", 0.5),
            ("rT", 3.13704371257512),
            ("uC", 0.31877145861608835),
            ("rC", 2.96704371257512),
        ],
    );
    // R_T' ≤ 1,05·R_T'': a' = 0 whatever the class.
    check(
        &result,
        "other_close_bounds",
        "opaque",
        &[
            ("rTUpper", 3.6986314662383726),
            ("rTLower", 3.5316993464052295),
            ("weightingFactor", 0.0),
            ("rT", 3.5316993464052295),
            ("uC", 0.2831498103081336),
        ],
    );
    check(
        &result,
        "metal",
        "opaque",
        &[
            ("rTUpper", 4.039244003566902),
            ("rTLower", 0.6822704782847718),
            ("weightingFactor", 0.5),
            ("rT", 1.7717327738152282),
            ("uC", 0.5644192029290128),
        ],
    );
    check(
        &result,
        "stony",
        "opaque",
        &[
            ("rTUpper", 2.6513904398256543),
            ("rTLower", 1.568129218900675),
            ("weightingFactor", 0.0),
            ("rT", 1.568129218900675),
            ("uC", 0.6377025489653476),
        ],
    );
    check(
        &result,
        "woody_close_bounds",
        "opaque",
        &[
            ("rTUpper", 4.155602339666473),
            ("rTLower", 4.104798761609907),
            ("weightingFactor", 0.0),
            ("rT", 4.104798761609907),
            ("uC", 0.2436173021080816),
        ],
    );
}

fn window(id: &str, method: Value, shutter: Option<Value>) -> Value {
    let mut window = json!({"method": method});
    if let Some(shutter) = shutter {
        window["shutter"] = shutter;
    }
    json!({"id": id, "element": {"kind": "window", "window": window}})
}

fn detailed_glazing(
    glazing: Value,
    area: f64,
    frame_area: f64,
    frame_u: f64,
    perimeter: f64,
    edge: Value,
) -> Value {
    json!({"method": "detailed", "glazing": [{"areaM2": area, "glazing": glazing, "perimeterM": perimeter, "edge": edge}],
        "frameAreaM2": frame_area, "frameUWPerM2K": frame_u, "frameSourceReference": "r75"})
}

/// 8.14–8.23 (p. 235–241), table 8.3 (p. 236), table 8.4 (p. 241), table G.1 (p. 818–819), tables L.1/L.2
/// (p. 867–868) and the rounding of 8.2.2.1 (p. 227).
#[test]
fn windows_doors_and_shutters() {
    let g1 = |layers: &str, coating: &str, gap: u32, gas: &str| json!({"method": "table_g1", "layers": layers, "coating": coating, "gapMm": gap, "gas": gas});
    let improved = json!({"method": "table", "frame": "wood_or_plastic", "spacer": "thermally_improved",
        "lowE": true, "spacerSourceReference": "r75"});
    let declared_edge = json!({"method": "declared", "value": 0.05, "sourceReference": "r75"});
    let hr = detailed_glazing(
        g1("double", "at_most_0_05", 16, "argon"),
        1.3,
        0.52,
        1.6,
        5.2,
        improved,
    );
    let result = envelope(json!({"elements": [
        window("detailed", hr.clone(), None),
        window("frame_table", json!({"method": "frame_table",
            "glazing": {"method": "declared", "value": 1.1, "sourceReference": "r75"}, "frame": "b"}), None),
        window("door", json!({"method": "door_default", "door": "thermally_insulating"}), None),
        window("panel", json!({"method": "detailed",
            "glazing": [{"areaM2": 0.8, "glazing": {"method": "single"}, "perimeterM": 3.6, "edge": {"method": "single_glazing"}}],
            "panels": [{"areaM2": 0.6, "uValueWPerM2k": 0.9, "perimeterM": 3.2, "psiWPerMk": 0.05, "sourceReference": "r75"}],
            "frameAreaM2": 0.5, "frameUWPerM2K": 2.2, "frameSourceReference": "r75"}), None),
        window("shutter_table", json!({"method": "declared", "value": 1.4, "sourceReference": "r75"}),
            Some(json!({"deltaR": {"method": "table", "kind": "internal_metallised"}, "conditionsEvidence": "r75"}))),
        window("shutter_declared", hr,
            Some(json!({"deltaR": {"method": "declared", "deltaRM2KPerW": 0.17, "sourceReference": "r75"},
                "conditionsEvidence": "r75"}))),
        window("triple", detailed_glazing(g1("triple", "at_most_0_10", 12, "argon"), 1.0, 0.4, 1.4, 4.0, declared_edge.clone()), None),
        window("sf6", detailed_glazing(g1("double", "uncoated", 6, "sf6"), 1.0, 0.4, 1.4, 4.0, declared_edge.clone()), None),
        window("krypton", detailed_glazing(g1("double", "at_most_0_20", 8, "krypton"), 1.0, 0.4, 1.4, 4.0, declared_edge), None),
    ]}));
    // U_gl 1,2 (G.1), ψ_gl 0,06 (L.2): (1,3·1,2 + 0,52·1,6 + 5,2·0,06)/1,82.
    check(
        &result,
        "detailed",
        "window",
        &[("uW", 1.4857142857142858), ("uRounded", 1.5)],
    );
    check(&result, "detailed", "", &[("uValue", 1.5)]);
    // Table 8.3 column B between U_gl 1,2 (2,2) and 1,0 (2,1).
    check(
        &result,
        "frame_table",
        "window",
        &[("uW", 2.15), ("uRounded", 2.2)],
    );
    check(&result, "door", "window", &[("uW", 2.0)]);
    // 8.18 with single glazing (8.24, ψ = 0) and a panel.
    check(
        &result,
        "panel",
        "window",
        &[("uW", 3.389473684210526), ("uRounded", 3.4)],
    );
    // 8.22/8.23 with f_shut;with 0,5 and ΔR 0,15 (table 8.4) or 0,17 on the unrounded U_W.
    check(
        &result,
        "shutter_table",
        "window",
        &[
            ("uW", 1.4),
            ("uWShut", 1.1570247933884297),
            ("uEffective", 1.278512396694215),
            ("uRounded", 1.3),
        ],
    );
    check(
        &result,
        "shutter_declared",
        "window",
        &[
            ("uWShut", 1.186131386861314),
            ("uEffective", 1.3359228362878),
            ("uRounded", 1.3),
        ],
    );
    // G.1: triple ε ≤ 0,10 argon 12 mm 0,9; double uncoated SF6 6 mm 3,0; ε ≤ 0,20 krypton 8 mm 1,7.
    check(&result, "triple", "window", &[("uW", 1.1857142857142857)]);
    check(&result, "sf6", "window", &[("uW", 2.685714285714286)]);
    check(&result, "krypton", "window", &[("uW", 1.7571428571428573)]);

    // 8.15–8.17: max(U1; U2) with ψ_gl 0,11 of table L.1 (metal with break, aluminium spacer, low-E).
    let simplified = envelope(
        json!({"elements": [window("simplified", json!({"method": "simplified",
        "glazing": {"method": "declared", "value": 1.1, "sourceReference": "r75"},
        "frameUWPerM2K": 1.8, "frameSourceReference": "r75",
        "edge": {"method": "table", "frame": "metal_with_thermal_break", "spacer": "aluminium_or_steel", "lowE": true}}), None)]}),
    );
    check(
        &simplified,
        "simplified",
        "window",
        &[("uW", 1.585), ("uRounded", 1.6)],
    );
}

/// ΔU_for of 8.3 (p. 225) on the rounded U of the opaque parts, and the forfait ψ of table I.1 (p. 821–822)
/// with the default 0,5 and the half for a shared detail (note 9 under figure 8.2, p. 226).
#[test]
fn forfait_supplement_and_forfait_psi() {
    let wall = |id: &str, area: f64, r: f64| {
        let mut item = opaque(
            id,
            json!({"heatFlow": "horizontal", "build": {"kind": "homogeneous", "layers": [
            material(0.1, 1.0), resistance(r, 0.1)]}}),
        );
        item["projectedAreaM2"] = json!(area);
        item["inForfaitSupplement"] = json!(true);
        item
    };
    // U_C 0,2 → 0,20 (A 50); 0,4048… → 0,40 (A 30); 0,3105… → 0,31 (A 20). R_calc 2,909 is rounded down to
    // 2,90 (E.2.1.1, p. 799): U_C 0,3154… → 0,32 (A 0, not in ΔU_for).
    let mut rounded_down = wall("d", 0.0, 2.909);
    rounded_down["inForfaitSupplement"] = json!(false);
    rounded_down
        .as_object_mut()
        .expect("element")
        .remove("projectedAreaM2");
    let result = envelope(json!({"forfaitSupplement": true,
        "elements": [wall("a", 50.0, 4.73), wall("b", 30.0, 2.2), wall("c", 20.0, 2.95), rounded_down]}));
    check(&result, "a", "", &[("uValue", 0.2)]);
    check(&result, "b", "", &[("uValue", 0.4)]);
    check(&result, "c", "", &[("uValue", 0.31)]);
    check(
        &result,
        "d",
        "opaque",
        &[("rT", 3.17), ("uC", 1.0 / 3.17), ("uCRounded", 0.32)],
    );
    close(
        "ΔU_for",
        num(&result["deltaUForfait"], "dU"),
        0.1 - 0.25 * ((10.0 + 12.0 + 6.2) / 100.0 - 0.4),
    );
    // Mean U above 0,8: ΔU_for = 0.
    let poor = envelope(json!({"forfaitSupplement": true, "elements": [wall("p", 10.0, 0.6)]}));
    close("ΔU_for max(0)", num(&poor["deltaUForfait"], "dU"), 0.0);

    let mut plain = wall("a", 50.0, 4.73);
    plain["inForfaitSupplement"] = json!(false);
    let bridges = envelope(json!({"elements": [plain],
        "forfaitBridges": [
            {"id": "p16", "position": 16, "column": "a", "lengthM": 7, "description": "r75"},
            {"id": "p23", "position": 23, "column": "b", "lengthM": 5, "description": "r75"},
            {"id": "p24", "position": 24, "variant": 1, "column": "a", "lengthM": 3, "description": "r75"},
            {"id": "p14", "position": 14, "column": "b", "lengthM": 6, "shared": true, "description": "r75"},
            {"id": "none", "column": "a", "lengthM": 4, "description": "r75"}]}));
    let expected = [
        ("p16", 0.05, 0.35),
        ("p23", 0.36, 1.8),
        ("p24", 0.41, 1.23),
        ("p14", 0.13, 0.39),
        ("none", 0.5, 2.0),
    ];
    for (id, psi, coefficient) in expected {
        let bridge = bridges["bridges"]
            .as_array()
            .expect("bridges")
            .iter()
            .find(|item| item["id"] == id)
            .expect("bridge");
        close(&format!("{id}.ψ"), num(&bridge["psiWPerMk"], "psi"), psi);
        close(
            &format!("{id}.ℓψ"),
            num(&bridge["coefficientWPerK"], "coefficient"),
            coefficient,
        );
    }
}

/// 8.52–8.59 (p. 264–268): b_U from H_ue = H_D;ue + ρc·V_ue/3600 (ρc 1 211), from 1,5·H_D;ue, with other
/// zones in Σ_j, and a declared b_U.
#[test]
fn unheated_spaces_reduction_factor() {
    let element = |id: &str, area: f64, u: f64| json!({"id": id, "areaM2": area, "uValueWPerM2k": u, "sourceReference": "r75"});
    let input: UnheatedTransmissionInput = serde_json::from_value(json!({"spaces": [
        {"id": "flow", "boundary": {"elements": [element("w", 13.0, 0.35)],
            "linearBridges": [{"id": "l", "lengthM": 5, "psiWPerMk": 0.2, "sourceReference": "r75"}],
            "pointBridges": [{"id": "p", "chiWPerK": 0.1, "sourceReference": "r75"}]},
         "outside": {"transmission": {"elements": [element("o1", 30.0, 1.8), element("o2", 5.0, 3.4)],
                "linearBridges": [{"id": "ol", "lengthM": 10, "psiWPerMk": 0.1, "sourceReference": "r75"}]},
            "ventilation": {"method": "flow", "airflowM3PerH": 60, "sourceReference": "r75"},
            "otherZonesConductanceWPerK": 5.0}},
        {"id": "half", "boundary": {"elements": [element("f", 6.0, 0.4)]},
         "outside": {"transmission": {"elements": [element("o", 12.0, 2.0)]}, "ventilation": {"method": "half_of_transmission"}}},
        {"id": "declared", "reductionFactor": 0.7, "factorSourceReference": "r75",
         "boundary": {"elements": [element("c", 20.0, 0.42)]}},
    ]}))
    .expect("unheated input");
    let result = serde_json::to_value(assess_unheated_transmission(&input)).expect("serializable");
    let h_iu = 13.0 * 0.35 + 1.0 + 0.1;
    let h_ue = 54.0 + 17.0 + 1.0 + 1.205 * 1005.0 * 60.0 / 3600.0;
    let b_flow = h_ue / (h_ue + h_iu + 5.0);
    let b_half = 1.5 * 24.0 / (1.5 * 24.0 + 2.4);
    let expected = [
        ("flow", h_iu, b_flow),
        ("half", 2.4, b_half),
        ("declared", 8.4, 0.7),
    ];
    let mut total = 0.0;
    for (id, unreduced, b) in expected {
        let space = result["spaces"]
            .as_array()
            .expect("spaces")
            .iter()
            .find(|item| item["id"] == id)
            .expect("space");
        close(
            &format!("{id}.H_iu"),
            num(&space["unreducedConductanceWPerK"], "h"),
            unreduced,
        );
        close(&format!("{id}.b_U"), num(&space["reductionFactor"], "b"), b);
        close(
            &format!("{id}.H_U"),
            num(&space["reducedConductanceWPerK"], "hu"),
            unreduced * b,
        );
        total += unreduced * b;
    }
    close(
        "ΣH_U",
        num(&result["totalReducedConductanceWPerK"], "total"),
        total,
    );
}

fn dwelling() -> Value {
    serde_json::from_str(DWELLING).expect("example dwelling")
}

/// The example dwelling on the detailed bridge route with a garage (flow), a storage room under part of the
/// floor (1,5·H_D;ue), a party wall to a heated neighbour, ψ-bridges on one, two and no orientations and χ.
fn detailed_project() -> Value {
    let mut p = dwelling();
    let constructions = p["constructions"].as_array_mut().expect("constructions");
    constructions.push(json!({"id": "c-part", "name": "Wand garage", "layers": [], "rcValue": 2.5, "uValue": 0.36}));
    constructions.push(json!({"id": "c-pfloor", "name": "Vloer berging", "layers": [], "rcValue": 2.0, "uValue": 0.43}));
    constructions.push(
        json!({"id": "c-party", "name": "Bouwmuur", "layers": [], "rcValue": 0.5, "uValue": 1.2}),
    );
    let zone = &mut p["zones"][0];
    let surfaces = zone["surfaces"].as_array_mut().expect("surfaces");
    surfaces.push(json!({"id": "wall-E", "name": "Wand garage", "type": "wall", "thermalBoundary": "unheated_space",
        "unheatedSpaceId": "garage", "area": 15, "orientation": "E", "constructionId": "c-part", "zoneId": "z1",
        "windows": [{"id": "win-E", "name": "Raam garage", "area": 2, "uValue": 1.4, "gValue": 0.6,
            "orientation": "E", "surfaceId": "wall-E"}]}));
    surfaces.push(json!({"id": "floor-b", "name": "Vloer berging", "type": "floor", "thermalBoundary": "unheated_space",
        "unheatedSpaceId": "berging", "area": 6, "orientation": "horizontal", "constructionId": "c-pfloor",
        "zoneId": "z1", "windows": []}));
    surfaces.push(json!({"id": "wall-W", "name": "Bouwmuur", "type": "wall", "thermalBoundary": "adjacent_conditioned",
        "area": 25, "orientation": "W", "constructionId": "c-party", "zoneId": "z1", "windows": []}));
    zone["thermalBridges"] = json!([
        {"id": "ridge", "name": "Nok", "psiValue": 0.05, "length": 7, "zoneId": "z1", "thermalBoundary": "outdoor", "orientations": ["N", "S"]},
        {"id": "lintel", "name": "Latei", "psiValue": 0.1, "length": 10, "zoneId": "z1", "thermalBoundary": "outdoor", "orientations": ["S"]},
        {"id": "eave", "name": "Goot", "psiValue": 0.08, "length": 7, "zoneId": "z1", "thermalBoundary": "outdoor", "orientations": ["N", "horizontal"]},
        {"id": "other", "name": "Overig", "psiValue": 0.04, "length": 12, "zoneId": "z1", "thermalBoundary": "outdoor"},
        {"id": "g-edge", "name": "Garage", "psiValue": 0.2, "length": 5, "zoneId": "z1", "thermalBoundary": "unheated_space", "unheatedSpaceId": "garage"}]);
    zone["pointThermalBridges"] = json!([
        {"id": "console", "name": "Console", "chiValue": 0.3, "zoneId": "z1", "thermalBoundary": "outdoor", "orientations": ["N"], "sourceReference": "r75"},
        {"id": "g-point", "name": "Garage", "chiValue": 0.1, "zoneId": "z1", "thermalBoundary": "unheated_space", "unheatedSpaceId": "garage", "sourceReference": "r75"}]);
    zone["pointBridgeInventoryComplete"] = json!(true);
    p["unheatedSpaces"] = json!([
        {"id": "garage", "name": "Garage", "outside": {
            "transmission": {"elements": [
                {"id": "gw", "areaM2": 30, "uValueWPerM2k": 1.8, "sourceReference": "r75"},
                {"id": "gd", "areaM2": 5, "uValueWPerM2k": 3.4, "sourceReference": "r75"}],
             "linearBridges": [{"id": "gb", "lengthM": 10, "psiWPerMk": 0.1, "sourceReference": "r75"}]},
            "ventilation": {"method": "flow", "airflowM3PerH": 60, "sourceReference": "r75"}}},
        {"id": "berging", "name": "Berging", "outside": {
            "transmission": {"elements": [{"id": "bw", "areaM2": 12, "uValueWPerM2k": 2.0, "sourceReference": "r75"}]},
            "ventilation": {"method": "half_of_transmission"}}}]);
    p["ntaCalculation"]["thermalBridgeRoute"] = json!("detailed");
    p["ntaCalculation"]["groundFloors"][0]["edgeThermalBridges"] = json!({"method": "detailed", "bridges": [{"lengthM": 10, "psiWPerMk": 0.1, "sourceReference": "r75"}]});
    p
}

fn assess(project: &Value) -> Value {
    let result = serde_json::to_value(assess_project_performance(project)).expect("serializable");
    assert_eq!(
        result["status"], "calculated_unverified",
        "{}",
        result["gaps"]
    );
    result
}

/// 8.4.2.1 (p. 265): towards the unheated space R_se 0,04 is replaced by R_si of table C.2 (wall 0,13, floor
/// with downward flow 0,17).
fn towards_unheated(u: f64, r_si: f64) -> f64 {
    1.0 / (1.0 / u - 0.04 + r_si)
}

fn orientation<'a>(result: &'a Value, name: &str) -> &'a Value {
    result["performance"]["tojuli"][0]["orientations"]
        .as_array()
        .expect("orientations")
        .iter()
        .find(|item| item["orientation"] == name)
        .expect("orientation")
}

/// H_D (8.1) with ψ·ℓ and χ, H_U (8.52–8.59) with H_D;zi,j;ztu per 8.4.2.1 and H_V;iu = 0, H_A = 0 (8.5),
/// H_tr (7.16) and the monthly heat transfer by transmission (7.14/7.15, p. 169–170).
#[test]
fn detailed_route_conductances_and_monthly_flows() {
    let result = assess(&detailed_project());
    let demand = &result["performance"]["spaceHeating"]["demand"];
    let transmission = &demand["transmission"];
    let h_d = 26.0 * 0.21
        + 4.0 * 1.1
        + 22.0 * 0.21
        + 8.0 * 1.1
        + 2.0 * 35.36 * 0.16
        + 0.05 * 7.0
        + 0.1 * 10.0
        + 0.08 * 7.0
        + 0.04 * 12.0
        + 0.3;
    let h_iu_garage =
        13.0 * towards_unheated(0.36, 0.13) + 2.0 * towards_unheated(1.4, 0.13) + 1.0 + 0.1;
    let h_ue_garage = 54.0 + 17.0 + 1.0 + 1.205 * 1005.0 * 60.0 / 3600.0;
    let h_iu_storage = 6.0 * towards_unheated(0.43, 0.17);
    let h_u = h_iu_garage * h_ue_garage / (h_ue_garage + h_iu_garage)
        + h_iu_storage * 36.0 / (36.0 + h_iu_storage);
    close(
        "H_D",
        num(&transmission["directConductanceWPerK"], "hd"),
        h_d,
    );
    close("H_D value", h_d, 37.2852);
    close(
        "H_U",
        num(&transmission["unheatedConductanceWPerK"], "hu"),
        h_u,
    );
    close("H_U value", h_u, 9.750601203597332);
    let h_tr = h_d + h_u + 3.6;
    close("H_tr", num(&transmission["conductanceWPerK"], "htr"), h_tr);
    // Monthly flows with the kernel's θ_int;calc (7.9) and ground H_g;an (annex D, review 63).
    let ground = transmission["groundMonthlyConductanceWPerK"]
        .as_array()
        .expect("ground");
    for (index, month) in demand["monthly"]
        .as_array()
        .expect("monthly")
        .iter()
        .enumerate()
    {
        let h_g = num(&ground[index], "hg");
        for balance in ["heating", "cooling"] {
            let theta = num(&month[balance]["calculationTemperatureC"], "theta");
            let expected = (h_tr * (theta - OUTDOOR_C[index]) + h_g * (theta - ANNUAL_OUTDOOR_C))
                * 0.001
                * HOURS[index];
            close(
                &format!("Q_{balance};tr month {}", index + 1),
                num(&month[balance]["transmissionKwh"], "q"),
                expected,
            );
        }
    }
}

/// 5.7.2 step A and steps 1–5 (p. 115–118): A_T;or without the AOR partition, the party wall and horizontal
/// parts; ψ·ℓ split equally over the listed orientations, the horizontal half and unlisted bridges pro rata
/// with H_gr;an;juli and H_C;ve;juli. H_U is not in H_C;D;juli (reading of the kernel, vragen-nen 9).
#[test]
fn tojuli_split_detailed_route() {
    let result = assess(&detailed_project());
    let demand = &result["performance"]["spaceHeating"]["demand"];
    let july = &demand["monthly"][6];
    let h_ve = num(&july["cooling"]["ventilationConductanceWPerK"], "hve");
    let h_gr = num(
        &demand["transmission"]["groundMonthlyConductanceWPerK"][6],
        "hg",
    );
    let pool = 0.04 * 12.0 + 0.08 * 7.0 / 2.0;
    let north = 26.0 * 0.21 + 4.0 * 1.1 + 35.36 * 0.16 + 0.05 * 7.0 / 2.0 + 0.08 * 7.0 / 2.0 + 0.3;
    let south = 22.0 * 0.21 + 8.0 * 1.1 + 35.36 * 0.16 + 0.05 * 7.0 / 2.0 + 0.1 * 10.0;
    for (name, h) in [("north", north), ("south", south)] {
        let item = orientation(&result, name);
        close(&format!("{name}.A_T"), num(&item["areaM2"], "a"), 65.36);
        close(&format!("{name}.share"), num(&item["share"], "share"), 0.5);
        close(
            &format!("{name}.H"),
            num(&item["conductanceWPerK"], "h"),
            h + 0.5 * (pool + h_gr + h_ve),
        );
        // 5.40 with the kernel's per-orientation July need, rounded up to 0,01 K.
        let need = num(&item["coolingNeedJulyKwh"], "q");
        let tojuli = (need * 1000.0 / ((h + 0.5 * (pool + h_gr + h_ve)) * 744.0) * 100.0 - 1e-9)
            .ceil()
            / 100.0;
        close(
            &format!("{name}.TOjuli"),
            num(&item["tojuliK"], "to"),
            tojuli.max(0.0),
        );
    }
    for name in ["east", "west"] {
        close(
            &format!("{name}.A_T"),
            num(&orientation(&result, name)["areaM2"], "a"),
            0.0,
        );
    }
}

/// Forfait route: H_U;for = 0 and the partition in H_D;for with U_iu;eq = U_C·b_U (8.4.1 note 5, p. 266;
/// C.10, p. 777), b_U from 8.53/8.59 without ψ, and ΔU_for (8.3) over all opaque parts including the
/// partitions with their U_iu;eq.
#[test]
fn forfait_route_with_unheated_partitions() {
    let mut project = detailed_project();
    project["zones"][0]["thermalBridges"] = json!([]);
    project["zones"][0]["pointThermalBridges"] = json!([]);
    project["ntaCalculation"]["thermalBridgeRoute"] = json!("forfait");
    project["ntaCalculation"]["groundFloors"][0]["edgeThermalBridges"] =
        json!({"method": "forfait"});
    let result = assess(&project);
    let h_iu = 13.0 * towards_unheated(0.36, 0.13) + 2.0 * towards_unheated(1.4, 0.13);
    let h_ue = 54.0 + 17.0 + 1.0 + 1.205 * 1005.0 * 60.0 / 3600.0;
    let b_garage = h_ue / (h_ue + h_iu);
    let b_storage = 36.0 / (36.0 + 6.0 * towards_unheated(0.43, 0.17));
    let (u_wall, u_window, u_floor) = (0.36 * b_garage, 1.4 * b_garage, 0.43 * b_storage);
    let opaque = [
        (26.0, 0.21),
        (22.0, 0.21),
        (35.36, 0.16),
        (35.36, 0.16),
        (13.0, u_wall),
        (6.0, u_floor),
    ];
    let area: f64 = opaque.iter().map(|(a, _)| a).sum();
    let au: f64 = opaque.iter().map(|(a, u)| a * u).sum();
    let delta_u = (0.1 - 0.25 * (au / area - 0.4)).max(0.0);
    close("ΔU_for value", delta_u, 0.14888169085878428);
    let all = [
        (26.0, 0.21),
        (4.0, 1.1),
        (22.0, 0.21),
        (8.0, 1.1),
        (35.36, 0.16),
        (35.36, 0.16),
        (2.0, u_window),
        (13.0, u_wall),
        (6.0, u_floor),
    ];
    let h_d: f64 = all.iter().map(|(a, u)| a * (u + delta_u)).sum();
    let transmission = &result["performance"]["spaceHeating"]["demand"]["transmission"];
    close(
        "H_D;for",
        num(&transmission["directConductanceWPerK"], "hd"),
        h_d,
    );
    assert!(
        transmission["unheatedConductanceWPerK"].is_null(),
        "H_U;for = 0"
    );
    close(
        "H_tr",
        num(&transmission["conductanceWPerK"], "htr"),
        h_d + 3.6,
    );
    let elements = result["derivedInput"]["spaceHeating"]["demand"]["opaqueElements"]
        .as_array()
        .expect("opaque");
    for (id, u) in [
        ("surface:wall-E:opaque", u_wall),
        ("window:win-E", u_window),
        ("surface:floor-b:opaque", u_floor),
    ] {
        let item = elements
            .iter()
            .find(|item| item["id"] == id)
            .expect("partition");
        close(
            &format!("{id}.U_iu;eq"),
            num(&item["uValueWPerM2k"], "u"),
            u,
        );
        close(
            &format!("{id}.ΔU_for"),
            num(&item["forfaitDeltaUWPerM2k"], "du"),
            delta_u,
        );
    }
}

/// The example dwelling with the garage of `detailed_project` as an adjacent unheated sunroom on the south.
fn sunroom_project() -> (Value, f64, f64) {
    let mut p = detailed_project();
    let surfaces = p["zones"][0]["surfaces"].as_array_mut().expect("surfaces");
    let wall = surfaces
        .iter_mut()
        .find(|item| item["id"] == "wall-E")
        .expect("wall");
    wall["orientation"] = json!("S");
    wall["unheatedSpaceId"] = json!("serre");
    wall["windows"][0]["orientation"] = json!("S");
    for list in ["thermalBridges", "pointThermalBridges"] {
        for bridge in p["zones"][0][list].as_array_mut().expect("bridges") {
            if bridge["unheatedSpaceId"] == "garage" {
                bridge["unheatedSpaceId"] = json!("serre");
            }
        }
    }
    p["unheatedSpaces"][0]["id"] = json!("serre");
    let h_iu = 13.0 * towards_unheated(0.36, 0.13) + 2.0 * towards_unheated(1.4, 0.13) + 1.0 + 0.1;
    let h_ue = 54.0 + 17.0 + 1.0 + 1.205 * 1005.0 * 60.0 / 3600.0;
    let b = h_ue / (h_ue + h_iu);
    p["ntaCalculation"]["sunrooms"] = json!([{"id": "serre", "glazingGHeating": 0.6, "glazingGCooling": 0.45,
        "exteriorFrameFraction": 0.2, "reductionFactor": b, "zoneConductanceWPerK": h_iu,
        "surfaces": [{"areaM2": 12, "absorptance": 0.6, "azimuthDeg": 180, "tiltDeg": 0},
                     {"areaM2": 13, "absorptance": 0.5, "azimuthDeg": 180, "tiltDeg": 90}],
        "sourceReference": "r75"}]);
    (p, b, h_iu)
}

/// 7.30b, 7.34–7.37 (p. 182–187) with I_sol of table 17.2 (horizontal and vertical south, p. 691–692):
/// gains (1 − b_U)·F·f_gn;max;H·Q_H;sol;u, f_gn;max with θ_int;set;H;stc 20 °C and capped at 1, no cap for
/// cooling.
#[test]
fn sunroom_gains_one_zone() {
    let (project, b, h_iu) = sunroom_project();
    close("b_U value", b, 0.919047682939721);
    close("H_zi;ztu value", h_iu, 8.119805203072323);
    let result = assess(&project);
    let heating = [
        7.816016299374052,
        6.162458157053456,
        6.332815966542864,
        4.6453312476941635,
        2.3686259860667773,
        1.6876297042184785,
        0.8764365603093385,
        0.6831710623949716,
        1.9007581977924617,
        4.314764604599822,
        5.22382287310926,
        7.191274340999704,
    ];
    let cooling = [
        12.841323579194043,
        15.442167652176542,
        29.42760859122623,
        42.673852033691986,
        48.35687954832842,
        46.99128131184932,
        45.27799740692606,
        45.773220421782604,
        35.39868154709251,
        24.985345400166253,
        13.297035905234637,
        9.789544273543452,
    ];
    let monthly = result["performance"]["spaceHeating"]["demand"]["monthly"]
        .as_array()
        .expect("monthly");
    for (index, month) in monthly.iter().enumerate() {
        close(
            &format!("sunroom H {}", index + 1),
            num(&month["sunroomGainsKwh"], "h"),
            heating[index],
        );
        close(
            &format!("sunroom C {}", index + 1),
            num(&month["sunroomCoolingGainsKwh"], "c"),
            cooling[index],
        );
    }
}

/// 7.36/7.38 (p. 187): a second zone (H 12 W/K, θ_set 21 °C) borders the same sunroom; F = 0,4 for this zone.
#[test]
fn sunroom_gains_shared() {
    let (project, _, h_iu) = sunroom_project();
    let result = assess(&project);
    let mut demand = result["derivedInput"]["spaceHeating"]["demand"].clone();
    demand["sunrooms"][0]["distributionFactor"] = json!(0.4);
    demand["sunrooms"][0]["borderingZones"] = json!([
        {"zoneId": "z1", "conductanceWPerK": h_iu, "heatingSetpointC": 20.0},
        {"zoneId": "z2", "conductanceWPerK": 12.0, "heatingSetpointC": 21.0}]);
    let input: MonthlyDemandInput = serde_json::from_value(demand).expect("demand input");
    let assessed = serde_json::to_value(assess_monthly_demand(&input)).expect("serializable");
    assert_eq!(assessed["status"], "calculated_unverified");
    let heating = [
        6.848705908903489,
        6.347884605300182,
        6.542446576883253,
        4.861330076343627,
        2.613350831336485,
        1.9298125132848136,
        1.1343714804050484,
        0.942816925916941,
        2.1410542200346403,
        4.542260647460918,
        5.434700423236013,
        5.221090279223175,
    ];
    let cooling = [
        5.1365294316776176,
        6.176867060870617,
        11.771043436490494,
        17.069540813476795,
        19.34275181933137,
        18.79651252473973,
        18.111198962770423,
        18.309288168713042,
        14.159472618837004,
        9.994138160066502,
        5.318814362093855,
        3.9158177094173814,
    ];
    for (index, month) in assessed["monthly"]
        .as_array()
        .expect("monthly")
        .iter()
        .enumerate()
    {
        close(
            &format!("shared H {}", index + 1),
            num(&month["sunroomGainsKwh"], "h"),
            heating[index],
        );
        close(
            &format!("shared C {}", index + 1),
            num(&month["sunroomCoolingGainsKwh"], "c"),
            cooling[index],
        );
    }
}

/// 5.7.2 step A (p. 115): horizontal elements are not summed in A_T;or but go pro rata (steps 3–4, p. 117).
/// A floor over outdoor air (tilt 180°) is horizontal; the kernel puts it under south. The flat roof (tilt 0°)
/// is pooled correctly.
#[test]
fn tojuli_floor_over_outdoor_air_is_horizontal() {
    let mut project = detailed_project();
    let surfaces = project["zones"][0]["surfaces"]
        .as_array_mut()
        .expect("surfaces");
    surfaces.push(json!({"id": "flat", "name": "Plat dak", "type": "roof", "thermalBoundary": "outdoor", "area": 5,
        "orientation": "horizontal", "constructionId": "c-roof", "zoneId": "z1", "windows": []}));
    surfaces.push(json!({"id": "cantilever", "name": "Vloer overstek", "type": "floor", "thermalBoundary": "outdoor",
        "area": 4, "orientation": "horizontal", "constructionId": "c-floor", "zoneId": "z1", "windows": []}));
    let result = assess(&project);
    let demand = &result["performance"]["spaceHeating"]["demand"];
    let h_ve = num(
        &demand["monthly"][6]["cooling"]["ventilationConductanceWPerK"],
        "hve",
    );
    let h_gr = num(
        &demand["transmission"]["groundMonthlyConductanceWPerK"][6],
        "hg",
    );
    let pool = 0.04 * 12.0 + 0.08 * 7.0 / 2.0 + 5.0 * 0.16 + 4.0 * 0.26;
    let north = 26.0 * 0.21 + 4.0 * 1.1 + 35.36 * 0.16 + 0.05 * 7.0 / 2.0 + 0.08 * 7.0 / 2.0 + 0.3;
    let south = 22.0 * 0.21 + 8.0 * 1.1 + 35.36 * 0.16 + 0.05 * 7.0 / 2.0 + 0.1 * 10.0;
    for (name, h) in [("north", north), ("south", south)] {
        let item = orientation(&result, name);
        close(&format!("{name}.A_T"), num(&item["areaM2"], "a"), 65.36);
        close(
            &format!("{name}.H"),
            num(&item["conductanceWPerK"], "h"),
            h + 0.5 * (pool + h_gr + h_ve),
        );
    }
}
