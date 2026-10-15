//! Review 76: construction routes outside review 75, recalculated independently from NTA 8800:2025+C1:2026:
//! tapered roof insulation (C.4.4, C.16–C.22), reflective layers in cavities (C.3.5, tables C.3/C.4) and foil
//! systems (table E.13), λ_calc and R_calc of annex E (E.3–E.10, tables E.1–E.5, E.10–E.17 and the rounding of
//! E.2.1.1), the window materials of table H.1, the forfait values of annex I (tables I.1/I.2 for ψ, figure I.4
//! with tables I.4–I.7 and formulas I.1–I.3 for closed elements, tables I.8–I.12 and I.15/I.16 for windows,
//! doors and panels) and the numerical route of 8.5/8.6 with 8.25–8.29, table 8.5 and the glazing bars of
//! 8.14/8.18. The expected values come from a separate implementation written from the norm pages; only numbers
//! and page references are kept here.

use nta8800_core::constructions::{tapered_part_u, HeatFlow, Layer, TaperShape};
use nta8800_core::envelope_elements::{assess_envelope, EnvelopeInput};
use nta8800_core::forfait_envelope::{
    forfait_door_u, forfait_glazed_door_u, forfait_panel_u, forfait_panel_u_in_edition,
    forfait_psi, forfait_window_u, ForfaitGlass, PanelInsulation, PsiColumn,
};
use nta8800_core::materials::{
    round_lambda_up, round_resistance_down, Conductivity, ForfaitMaterial, WindowMaterial,
};
use nta8800_core::window_u::{
    edge_psi, frame_u_from_l2d, linear_psi, point_bridge_counts, point_chi, FrameGroup,
    PointBridgeMaterial,
};
use serde_json::{json, Value};

/// Kind and (ρ, dry, other) rows of tables E.15–E.17.
type MasonryRows = (&'static str, &'static [(f64, f64, f64)]);
/// Thickness with (no cavity, cavity) × frame groups, outdoor (I.15) and not outdoor (I.16).
type ThinPanelRow = (f64, [[f64; 3]; 2], [[f64; 3]; 2]);

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

fn assess(input: Value) -> Value {
    let input: EnvelopeInput = serde_json::from_value(input).expect("envelope input");
    serde_json::to_value(assess_envelope(&input)).expect("serializable")
}

fn envelope(input: Value) -> Value {
    let result = assess(input);
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
        "conductivity": {"method": "calculated", "lambdaCalc": lambda, "sourceReference": "r76"}})
}

fn resistance(r: f64, thickness: f64) -> Value {
    json!({"kind": "resistance", "resistanceM2KPerW": r, "thicknessM": thickness, "sourceReference": "r76"})
}

fn opaque(id: &str, heat_flow: &str, layers: Value) -> Value {
    json!({"id": id, "element": {"kind": "opaque", "construction": {"heatFlow": heat_flow,
        "build": {"kind": "homogeneous", "layers": layers}}}})
}

fn lambda(conductivity: Value) -> f64 {
    let conductivity: Conductivity = serde_json::from_value(conductivity).expect("conductivity");
    assert!(
        conductivity.validate().is_empty(),
        "{:?}",
        conductivity.validate()
    );
    conductivity.lambda_calc()
}

/// C.16–C.22 (p. 786–789): R_0 = R_T of the base (R_si 0,10 upward, R_se 0,04), R_2 = d_2/λ_calc;t,
/// R_1 = d_1/λ_calc;t, area-weighted U_T; U rounded to 2 decimals (8.2.2.1, p. 227).
#[test]
fn tapered_roof_c16_to_c22() {
    // R_0 = 0,10 + 0,2/2,0 + 2,5 + 0,04 = 2,74; λ_calc;t 0,035.
    let r0 = 2.74;
    close(
        "type 1",
        tapered_part_u(TaperShape::Rectangular, r0, 0.0, 0.10 / 0.035),
        0.2500044200713858,
    );
    close(
        "type 2",
        tapered_part_u(TaperShape::TriangleThickestAtApex, r0, 0.0, 0.20 / 0.035),
        0.233441493927622,
    );
    close(
        "type 3",
        tapered_part_u(TaperShape::TriangleThickestAtBase, r0, 0.0, 0.20 / 0.035),
        0.16090895820324783,
    );
    close(
        "type 4",
        tapered_part_u(TaperShape::TriangleVarying, r0, 0.05 / 0.035, 0.15 / 0.035),
        0.22343258585751577,
    );
    let roof = |slope: f64, conductivity: Value, parts: Value| {
        json!({"kind": "tapered_roof", "roof": {"baseLayers": [material(0.2, 2.0), resistance(2.5, 0.1)],
            "taperConductivity": conductivity, "slopePercent": slope, "parts": parts}})
    };
    let calculated = json!({"method": "calculated", "lambdaCalc": 0.035, "sourceReference": "r76"});
    let four = json!([
        {"shape": "rectangular", "areaM2": 20, "maxThicknessM": 0.10},
        {"shape": "triangle_thickest_at_apex", "areaM2": 8, "maxThicknessM": 0.20},
        {"shape": "triangle_thickest_at_base", "areaM2": 8, "maxThicknessM": 0.20},
        {"shape": "triangle_varying", "areaM2": 6, "intermediateThicknessM": 0.05, "maxThicknessM": 0.15}]);
    // λ_for EPS board 0,040 (table E.10, p. 809) with F_M = F_A = 1.
    let eps = json!({"method": "forfait_insulation", "material": "eps_board", "moisture": "regular",
        "ageing": {"kind": "factory_made"}});
    let result = envelope(json!({"elements": [
        {"id": "four-types", "element": roof(2.0, calculated.clone(), four)},
        {"id": "at-5-percent", "element": roof(5.0, eps, json!([{"shape": "rectangular", "areaM2": 10, "maxThicknessM": 0.12}]))},
    ]}));
    check(
        &result,
        "four-types",
        "",
        &[("uUnrounded", 0.22608303651475642), ("uValue", 0.23)],
    );
    check(
        &result,
        "at-5-percent",
        "",
        &[("uUnrounded", 0.24650042997716537), ("uValue", 0.25)],
    );
    // Above 5 % a numerical method is required (p. 788).
    let steep = assess(
        json!({"elements": [{"id": "steep", "element": roof(5.5, calculated,
        json!([{"shape": "rectangular", "areaM2": 10, "maxThicknessM": 0.12}]))}]}),
    );
    assert!(
        steep["issues"].to_string().contains("above_5_percent"),
        "{steep}"
    );
}

/// C.13/C.14 with tables C.3 (p. 782) and C.4 (p. 784): bracket values for an effective reflective layer,
/// R_se 0,05 for upward heat flow, footnote b of table C.4 (an upward-facing layer earns no bracket value unless
/// the cavity is hermetically sealed) and the still-air R_se of footnote b (0,22 vertical, 0,10 upward, 0,83
/// downward with a reflective layer); R_C with the R_se of C.2. Table E.13 (p. 811) for foil systems, R rounded
/// down per E.2.1.1 (p. 799).
#[test]
fn reflective_cavities_and_foil_systems() {
    let cavity = |d: f64, ventilation: &str, facing_up: bool, sealed: bool| {
        json!({"kind": "air_cavity", "thicknessMm": d, "ventilation": {"kind": ventilation},
            "reflectiveSurface": true, "reflectiveFacingUp": facing_up, "hermeticallySealed": sealed})
    };
    let wall = |id: &str, layer: Value| {
        opaque(
            id,
            "horizontal",
            json!([
                material(0.1, 1.0),
                resistance(3.0, 0.1),
                layer,
                material(0.1, 0.9)
            ]),
        )
    };
    let roof = |id: &str, layer: Value| {
        opaque(
            id,
            "upward",
            json!([
                material(0.2, 2.0),
                resistance(3.0, 0.1),
                layer,
                material(0.018, 0.13)
            ]),
        )
    };
    let floor = |id: String, layer: Value| {
        opaque(
            &id,
            "downward",
            json!([material(0.2, 2.0), resistance(2.5, 0.1), layer]),
        )
    };
    let foil = |system: Value| json!({"kind": "reflective_foil", "system": system});
    let mut elements = vec![
        wall("v_nv", cavity(40.0, "unventilated", false, false)),
        wall("v_zv", cavity(40.0, "weakly", false, false)),
        wall("v_sv", cavity(40.0, "strongly", false, false)),
        roof("up_nv", cavity(50.0, "unventilated", false, false)),
        roof("up_zv", cavity(50.0, "weakly", false, false)),
        roof("up_facing_up", cavity(50.0, "unventilated", true, false)),
        roof(
            "up_facing_up_sealed",
            cavity(50.0, "unventilated", true, true),
        ),
        roof("up_sv", cavity(50.0, "strongly", false, false)),
        floor("dn_sv".into(), cavity(100.0, "strongly", false, false)),
        wall(
            "foil_layers",
            foil(json!({"kind": "foil_layers", "thicknessM": 0.008})),
        ),
        wall("foil_facing", foil(json!({"kind": "facing"}))),
        wall(
            "foil_two",
            foil(json!({"kind": "two_foils_with_air_layer"})),
        ),
        wall(
            "foil_three",
            foil(json!({"kind": "three_foils_with_air_layers"})),
        ),
    ];
    let rows = [
        (20.0, 0.57, 0.40),
        (25.0, 0.66, 0.44),
        (50.0, 0.99, 0.61),
        (100.0, 1.19, 0.71),
        (300.0, 1.40, 0.81),
    ];
    for (d, _, _) in rows {
        elements.push(floor(
            format!("dn_nv_{d}"),
            cavity(d, "unventilated", false, false),
        ));
        elements.push(floor(
            format!("dn_zv_{d}"),
            cavity(d, "weakly", false, false),
        ));
    }
    let result = envelope(json!({ "elements": elements }));
    let outer = 0.1 / 0.9;
    let board = 0.018 / 0.13;
    // Vertical: 0,13 + 0,1 + 3,0 + R_cav + 0,1/0,9 + 0,04.
    check(
        &result,
        "v_nv",
        "opaque",
        &[
            ("rT", 3.951111111111111),
            ("rC", 3.781111111111111),
            ("exteriorSurfaceResistance", 0.04),
        ],
    );
    check(&result, "v_zv", "opaque", &[("rT", 3.781111111111111)]);
    check(
        &result,
        "v_sv",
        "opaque",
        &[
            ("rT", 3.45),
            ("rC", 3.28),
            ("exteriorSurfaceResistance", 0.22),
        ],
    );
    // Upward: 0,10 + 0,1 + 3,0 + R_cav + 0,018/0,13 + R_se (0,05 with the bracket value).
    check(
        &result,
        "up_nv",
        "opaque",
        &[
            ("rT", 3.7984615384615386),
            ("rC", 3.6484615384615386),
            ("exteriorSurfaceResistance", 0.05),
        ],
    );
    check(&result, "up_zv", "opaque", &[("rT", 3.6984615384615385)]);
    check(
        &result,
        "up_facing_up",
        "opaque",
        &[
            ("rT", 0.10 + 3.1 + 0.16 + board + 0.04),
            ("exteriorSurfaceResistance", 0.04),
        ],
    );
    check(
        &result,
        "up_facing_up_sealed",
        "opaque",
        &[("rT", 3.7984615384615386)],
    );
    check(
        &result,
        "up_sv",
        "opaque",
        &[("rT", 3.3), ("exteriorSurfaceResistance", 0.10)],
    );
    // Downward: 0,17 + 0,1 + 2,5 + R_cav + 0,04; strongly ventilated with R_se 0,83.
    for (d, nv, zv) in rows {
        check(
            &result,
            &format!("dn_nv_{d}"),
            "opaque",
            &[("rT", 0.17 + 2.6 + nv + 0.04)],
        );
        check(
            &result,
            &format!("dn_zv_{d}"),
            "opaque",
            &[("rT", 0.17 + 2.6 + zv + 0.04)],
        );
    }
    check(
        &result,
        "dn_sv",
        "opaque",
        &[
            ("rT", 3.6),
            ("rC", 3.39),
            ("exteriorSurfaceResistance", 0.83),
        ],
    );
    // Table E.13: d/0,03 = 0,2667 rounded down to 0,26; facing 0; 1,80 and 2,90.
    let base = 0.13 + 0.1 + 3.0 + outer + 0.04;
    for (id, r) in [
        ("foil_layers", 0.26),
        ("foil_facing", 0.0),
        ("foil_two", 1.80),
        ("foil_three", 2.90),
    ] {
        check(&result, id, "opaque", &[("rT", base + r)]);
    }
}

/// E.3 (p. 798) with F_T of E.7 (p. 800; 1,00 between 5 and 20 °C), F_M of table E.2 (p. 801) or E.8/E.9,
/// F_A of E.10 with table E.5 (p. 803), F_conv, and the rounding up of λ_calc in E.2.1.1 (p. 799); E.4 with the
/// rounding down of R_calc.
#[test]
fn declared_insulation_and_resistance() {
    let declared = |lambda_d: f64, moisture: &str, ageing: Value, extra: Value| {
        let mut value = json!({"method": "declared_insulation", "lambdaDeclared": lambda_d,
            "moisture": moisture, "ageing": ageing, "sourceReference": "r76"});
        for (key, item) in extra.as_object().expect("extra") {
            value[key] = item.clone();
        }
        value
    };
    let factory = json!({"kind": "factory_made"});
    let in_situ = |product: &str, situation: &str, tested: bool| json!({"kind": "in_situ", "product": product, "situation": situation, "practiceTested": tested});
    let temperature = |t: f64, f: f64| json!({"temperature": {"meanTemperatureC": t, "conversionCoefficient": f, "sourceReference": "r76"}});
    let cases = [
        // Table E.2: 0,032 · F_M.
        (
            declared(0.032, "regular", factory.clone(), json!({})),
            0.032,
        ),
        (
            declared(
                0.032,
                "inverted_roof_xps_slope_up_to_1_percent",
                factory.clone(),
                json!({}),
            ),
            0.034,
        ),
        (
            declared(
                0.032,
                "inverted_roof_xps_slope_above_1_percent",
                factory.clone(),
                json!({}),
            ),
            0.033,
        ),
        (
            declared(
                0.032,
                "inverted_roof_xps_green_roof",
                factory.clone(),
                json!({}),
            ),
            0.035,
        ),
        (
            declared(0.032, "inverted_roof_general", factory.clone(), json!({})),
            0.040,
        ),
        (
            declared(0.032, "perimeter_general", factory.clone(), json!({})),
            0.040,
        ),
        (
            declared(
                0.032,
                "perimeter_xps_fully_bonded",
                factory.clone(),
                json!({}),
            ),
            0.032,
        ),
        (
            declared(
                0.032,
                "perimeter_xps_point_bonded",
                factory.clone(),
                json!({}),
            ),
            0.033,
        ),
        // E.7: 0,035·e^(0,0035·(0 − 10)) and 0,035·e^(0,004·(25 − 10)); 12 °C gives F_T = 1.
        (
            declared(0.035, "regular", factory.clone(), temperature(0.0, 0.0035)),
            0.034,
        ),
        (
            declared(0.035, "regular", factory.clone(), temperature(25.0, 0.004)),
            0.038,
        ),
        (
            declared(0.035, "regular", factory.clone(), temperature(12.0, 0.004)),
            0.035,
        ),
        // E.8/E.9 with Ψ_1 = u_1 = 0: 0,040·e^(4·0,02) and 0,040·e^(2,5·0,03).
        (
            declared(
                0.040,
                "regular",
                factory.clone(),
                json!({"moistureConversion": {"basis": "volume", "conversionCoefficient": 4,
                "moistureContent": 0.02, "sourceReference": "r76"}}),
            ),
            0.044,
        ),
        (
            declared(
                0.040,
                "regular",
                factory.clone(),
                json!({"moistureConversion": {"basis": "mass", "conversionCoefficient": 2.5,
                "moistureContent": 0.03, "sourceReference": "r76"}}),
            ),
            0.044,
        ),
        // Table E.5: F_A;iso · F_A;appl.
        (
            declared(
                0.040,
                "regular",
                in_situ("eps_beads", "a", false),
                json!({}),
            ),
            0.042,
        ),
        (
            declared(
                0.030,
                "regular",
                in_situ("polyurethane", "b", false),
                json!({}),
            ),
            0.038,
        ),
        (
            declared(
                0.030,
                "regular",
                in_situ("polyurethane", "b", true),
                json!({}),
            ),
            0.037,
        ),
        (
            declared(
                0.040,
                "regular",
                in_situ("urea_formaldehyde", "a", false),
                json!({}),
            ),
            0.050,
        ),
        (
            declared(0.040, "regular", in_situ("other", "b", false), json!({})),
            0.060,
        ),
        (
            declared(
                0.040,
                "regular",
                in_situ("fibres_and_flakes", "b", false),
                json!({}),
            ),
            0.046,
        ),
        (
            declared(
                0.035,
                "regular",
                factory.clone(),
                json!({"convectionFactor": 1.1}),
            ),
            0.039,
        ),
        // 0,040 · e^(0,0035·20) · 1,25 · 1,05·1,15 · 1,05.
        (
            declared(
                0.040,
                "inverted_roof_general",
                in_situ("polyurethane", "b", true),
                json!({"temperature": {"meanTemperatureC": 30, "conversionCoefficient": 0.0035,
                "sourceReference": "r76"}, "convectionFactor": 1.05}),
            ),
            0.068,
        ),
    ];
    for (index, (input, expected)) in cases.into_iter().enumerate() {
        close(&format!("declared {index}"), lambda(input), expected);
    }
    // E.4: R_D/(F_T·F_M·F_A·F_conv), rounded down to two decimals or three significant figures.
    let declared_r = |r: f64, moisture: &str, ageing: Value| {
        let layer: Layer = serde_json::from_value(json!({"kind": "declared_resistance",
            "resistanceDeclared": r, "moisture": moisture, "ageing": ageing, "sourceReference": "r76"}))
        .expect("layer");
        layer.resistance(HeatFlow::Horizontal).expect("resistance")
    };
    close(
        "E.4 general",
        declared_r(2.5, "inverted_roof_general", factory.clone()),
        2.0,
    );
    close(
        "E.4 XPS ≤ 1 %",
        declared_r(
            3.15,
            "inverted_roof_xps_slope_up_to_1_percent",
            factory.clone(),
        ),
        3.02,
    );
    close(
        "E.4 PU B",
        declared_r(4.0, "regular", in_situ("polyurethane", "b", false)),
        3.16,
    );
    // Rounding rules of E.2.1.1 themselves.
    for (value, expected) in [
        (0.0801, 0.085),
        (0.08, 0.08),
        (0.0341, 0.035),
        (0.2001, 0.21),
        (1.999, 2.0),
        (2.0001, 2.1),
    ] {
        close(&format!("λ {value}"), round_lambda_up(value), expected);
    }
    for (value, expected) in [(2.909, 2.90), (0.2667, 0.26), (3.0288, 3.02)] {
        close(
            &format!("R {value}"),
            round_resistance_down(value),
            expected,
        );
    }
}

/// Tables E.10 (p. 809), E.11 and E.12 (p. 810) in 2025+C1, used through E.3 with the table E.2/E.5 factors.
#[test]
fn forfait_lambda_tables_e10_to_e12() {
    use ForfaitMaterial::*;
    for (material, expected) in [
        (GlassWool, 0.040),
        (StoneWool, 0.040),
        (MineralWoolFlakes, 0.045),
        (EpsBoard, 0.040),
        (EpsWhiteBeads, 0.045),
        (EpsGreyBeads, 0.035),
        (Xps, 0.040),
        (PurPirBoard, 0.030),
        (PurSprayedOpenCell, 0.045),
        (PurSprayedClosedCell, 0.035),
        (PhenolicFoam, 0.030),
        (CellularGlass, 0.045),
        (WoodWool, 0.100),
        (ExpandedPerlite, 0.050),
        (ExpandedCork, 0.050),
        (WoodFibre, 0.045),
        (UreaFormaldehyde, 0.060),
        (CelluloseLoose, 0.045),
        (FlaxBoard, 0.045),
        (FlaxSprayed, 0.050),
        (SheepWool, 0.045),
        (Cotton, 0.045),
        (Coconut, 0.050),
        (Straw, 0.055),
        (HempBoard, 0.045),
        (HempOther, 0.060),
        (Peat, 0.100),
        (Miscanthus, 0.060),
        (GrassWool, 0.045),
        (Reed, 0.100),
        (CementBoundPerlite, 0.150),
        (CementBoundEps, 0.150),
        (Shells, 0.200),
        (ClayGranules, 0.300),
    ] {
        close(&format!("{material:?}"), material.lambda(), expected);
    }
    let forfait = |material: &str, moisture: &str, ageing: Value| {
        lambda(
            json!({"method": "forfait_insulation", "material": material, "moisture": moisture, "ageing": ageing}),
        )
    };
    let in_situ = |product: &str, situation: &str| json!({"kind": "in_situ", "product": product, "situation": situation});
    let factory = json!({"kind": "factory_made"});
    close(
        "flakes B",
        forfait(
            "mineral_wool_flakes",
            "regular",
            in_situ("fibres_and_flakes", "b"),
        ),
        0.052,
    );
    close(
        "white beads A",
        forfait("eps_white_beads", "regular", in_situ("eps_beads", "a")),
        0.048,
    );
    close(
        "closed-cell PU B",
        forfait(
            "pur_sprayed_closed_cell",
            "regular",
            in_situ("polyurethane", "b"),
        ),
        0.045,
    );
    close(
        "UF A",
        forfait(
            "urea_formaldehyde",
            "regular",
            in_situ("urea_formaldehyde", "a"),
        ),
        0.075,
    );
    close(
        "cellulose B",
        forfait(
            "cellulose_loose",
            "regular",
            in_situ("fibres_and_flakes", "b"),
        ),
        0.052,
    );
    close(
        "XPS inverted roof",
        forfait("xps", "inverted_roof_general", factory.clone()),
        0.050,
    );
    close("straw", forfait("straw", "regular", factory.clone()), 0.055);
    close(
        "grass wool",
        forfait("grass_wool", "regular", factory),
        0.045,
    );
}

/// Tables E.14–E.17 (p. 812–813) rounded up per E.2.2.3.3 (p. 812), E.5 with tables E.3/E.4 (p. 802), E.6
/// with table E.1 (p. 799) and table H.1 (p. 820).
#[test]
fn masonry_other_materials_and_window_materials() {
    let table = |kind: &str, density: f64, environment: &str, joints: &str| {
        lambda(
            json!({"method": "masonry_table", "kind": kind, "densityKgM3": density,
            "environment": environment, "joints": joints}),
        )
    };
    // Table E.14: ρ, glued dry, glued other, mortar dry, mortar other.
    let brick: [(f64, [f64; 4]); 19] = [
        (600.0, [0.160, 0.300, 0.190, 0.320]),
        (700.0, [0.190, 0.360, 0.220, 0.380]),
        (800.0, [0.220, 0.420, 0.260, 0.440]),
        (900.0, [0.260, 0.480, 0.300, 0.500]),
        (1000.0, [0.290, 0.540, 0.340, 0.570]),
        (1100.0, [0.320, 0.600, 0.380, 0.630]),
        (1200.0, [0.350, 0.660, 0.420, 0.700]),
        (1300.0, [0.390, 0.720, 0.460, 0.760]),
        (1400.0, [0.430, 0.800, 0.510, 0.850]),
        (1500.0, [0.460, 0.860, 0.540, 0.910]),
        (1600.0, [0.500, 0.940, 0.600, 0.990]),
        (1700.0, [0.550, 1.030, 0.650, 1.080]),
        (1800.0, [0.590, 1.110, 0.700, 1.160]),
        (1900.0, [0.640, 1.210, 0.760, 1.270]),
        (2000.0, [0.680, 1.290, 0.810, 1.350]),
        (2100.0, [0.740, 1.390, 0.870, 1.460]),
        (2200.0, [0.790, 1.490, 0.940, 1.560]),
        (2300.0, [0.850, 1.590, 1.000, 1.670]),
        (2400.0, [0.900, 1.690, 1.070, 1.780]),
    ];
    for (density, values) in brick {
        let combos = [
            ("dry_indoor", "glued"),
            ("other", "glued"),
            ("dry_indoor", "mortar"),
            ("other", "mortar"),
        ];
        for ((environment, joints), expected) in combos.into_iter().zip(values) {
            close(
                &format!("E.14 {density} {environment} {joints}"),
                table("brick", density, environment, joints),
                expected,
            );
        }
    }
    // Tables E.15–E.17 after rounding up (1,033 → 1,04; 2,018 → 2,1; 0,119 → 0,120; 0,211 → 0,22).
    let rounded: [MasonryRows; 3] = [
        (
            "concrete",
            &[
                (1600.0, 1.04, 1.17),
                (1700.0, 1.10, 1.24),
                (1800.0, 1.19, 1.34),
                (1900.0, 1.28, 1.45),
                (2000.0, 1.40, 1.58),
                (2100.0, 1.53, 1.72),
                (2200.0, 1.67, 1.88),
                (2300.0, 1.84, 2.1),
                (2400.0, 2.1, 2.3),
            ],
        ),
        (
            "calcium_silicate",
            &[
                (1750.0, 0.87, 1.53),
                (1850.0, 1.00, 1.75),
                (2200.0, 1.57, 2.8),
            ],
        ),
        (
            "aerated_concrete",
            &[
                (400.0, 0.120, 0.130),
                (500.0, 0.145, 0.155),
                (600.0, 0.175, 0.190),
                (700.0, 0.195, 0.22),
                (800.0, 0.23, 0.25),
            ],
        ),
    ];
    for (kind, rows) in rounded {
        for &(density, dry, other) in rows {
            for joints in ["mortar", "glued"] {
                close(
                    &format!("{kind} {density} dry {joints}"),
                    table(kind, density, "dry_indoor", joints),
                    dry,
                );
                close(
                    &format!("{kind} {density} other {joints}"),
                    table(kind, density, "other", joints),
                    other,
                );
            }
        }
    }
    // E.5 with F_M of tables E.3/E.4; the unit and joint λ_calc are rounded before weighting (see the
    // calcium-silicate case, which gives 1,02 when only the weighted value is rounded).
    let declared = |kind: &str, unit: f64, mortar: f64, joints: f64, environment: &str| {
        lambda(
            json!({"method": "masonry_declared", "kind": kind, "unitLambdaDeclared": unit,
            "mortarLambdaDeclared": mortar, "jointAreaFraction": joints, "environment": environment,
            "sourceReference": "r76"}),
        )
    };
    close(
        "E.5 brick other",
        declared("brick", 0.5, 0.8, 0.15, "other"),
        1.08,
    );
    close(
        "E.5 brick dry",
        declared("brick", 0.5, 0.8, 0.15, "dry_indoor"),
        0.60,
    );
    close(
        "E.5 aerated other glued",
        declared("aerated_concrete", 0.13, 0.8, 0.0, "other"),
        0.155,
    );
    close(
        "E.5 calcium silicate dry",
        declared("calcium_silicate", 0.9, 0.8, 0.1, "dry_indoor"),
        1.03,
    );
    close(
        "E.5 concrete other",
        declared("concrete", 1.1, 0.8, 0.12, "other"),
        1.46,
    );
    // E.6 with table E.1 at the tabulated densities.
    let other = |lambda_d: f64, class: &str, density: f64| {
        lambda(
            json!({"method": "declared_other", "lambdaDeclared": lambda_d, "class": class,
            "densityKgM3": density, "sourceReference": "r76"}),
        )
    };
    for (lambda_d, class, density, expected) in [
        (1.5, "inorganic", 2500.0, 1.88),
        (1.5, "inorganic", 2300.0, 1.88),
        (1.5, "inorganic", 1900.0, 1.88),
        (0.6, "inorganic", 1600.0, 0.78),
        (0.6, "inorganic", 1300.0, 0.78),
        (0.3, "inorganic", 1000.0, 0.41),
        (0.12, "inorganic", 700.0, 0.17),
        (0.12, "inorganic", 400.0, 0.17),
        (1.0, "glass", 2500.0, 1.00),
        (0.13, "organic", 1000.0, 0.16),
        (0.13, "organic", 500.0, 0.16),
        (0.1, "organic", 400.0, 0.125),
        (0.1, "organic", 300.0, 0.125),
        (0.2, "plastic", 1500.0, 0.20),
    ] {
        close(
            &format!("E.6 {class} {density}"),
            other(lambda_d, class, density),
            expected,
        );
    }
    use WindowMaterial::*;
    for (material, expected) in [
        (RedCopper, 380.0),
        (Aluminium, 160.0),
        (Copper, 120.0),
        (Steel, 50.0),
        (StainlessSteel, 17.0),
        (Pvc, 0.17),
        (Hardwood, 0.18),
        (Softwood, 0.13),
        (GlassFibrePolyester, 0.30),
        (SodaLimeGlass, 1.0),
        (Acrylic, 0.20),
        (Pmma, 0.18),
        (Polycarbonate, 0.20),
        (Polyamide, 0.25),
        (Polyamide66GlassFibre, 0.30),
        (PolyethyleneHd, 0.52),
        (PolyethyleneLd, 0.33),
        (Polypropylene, 0.22),
        (PolypropyleneGlassFibre, 0.25),
        (PolyurethaneBreak, 0.25),
        (PvcRigid, 0.17),
        (Neoprene, 0.23),
        (Epdm, 0.25),
        (SiliconeSeal, 0.35),
        (PvcFlexible, 0.14),
        (Mohair, 0.14),
        (FoamRubber, 0.06),
        (PurFoamSealant, 0.30),
        (Butyl, 0.24),
        (Polysulfide, 0.40),
        (SiliconeSealant, 0.35),
        (Polyisobutylene, 0.20),
        (PolyesterResin, 0.19),
        (SilicaGel, 0.13),
        (SiliconeFoamLd, 0.12),
        (SiliconeFoamMd, 0.17),
    ] {
        close(&format!("H.1 {material:?}"), material.lambda(), expected);
    }
}

/// Tables I.1 (p. 821–822) and I.2 (p. 823–824), columns A and B, in the order of the rows per position.
#[test]
fn forfait_psi_tables_i1_and_i2() {
    let rows: [(u16, u8, f64, f64); 54] = [
        (1, 0, 0.27, 0.41),
        (2, 0, 0.45, 0.68),
        (3, 0, 0.60, 0.90),
        (4, 0, 0.00, 0.00),
        (5, 0, 0.15, 0.25),
        (6, 0, 0.09, 0.19),
        (7, 0, 0.10, 0.20),
        (8, 0, 0.10, 0.20),
        (9, 0, 0.14, 0.24),
        (10, 0, 0.09, 0.19),
        (11, 0, 0.15, 0.25),
        (12, 0, 0.00, 0.00),
        (13, 0, 0.16, 0.26),
        (14, 0, 0.03, 0.13),
        (15, 0, 0.13, 0.23),
        (16, 0, 0.05, 0.15),
        (17, 0, 0.06, 0.09),
        (18, 0, 0.50, 0.75),
        (19, 0, 0.13, 0.23),
        (20, 0, 0.12, 0.22),
        (21, 0, 0.14, 0.24),
        (22, 0, 0.12, 0.22),
        (23, 0, 0.24, 0.36),
        (24, 0, 0.13, 0.23),
        (24, 1, 0.41, 0.62),
        (50, 0, 0.61, 0.92),
        (51, 0, 0.64, 0.96),
        (52, 0, 0.64, 0.96),
        (53, 0, 0.00, 0.00),
        (54, 0, 0.15, 0.25),
        (55, 0, 0.09, 0.19),
        (56, 0, 0.10, 0.20),
        (57, 0, 0.00, 0.00),
        (58, 0, 0.70, 1.05),
        (58, 1, 0.13, 0.23),
        (59, 0, 0.70, 1.05),
        (59, 1, 0.35, 0.53),
        (60, 0, 0.16, 0.26),
        (61, 0, 0.16, 0.26),
        (62, 0, 0.39, 0.59),
        (63, 0, 0.31, 0.47),
        (64, 0, 0.00, 0.00),
        (65, 0, 0.36, 0.54),
        (66, 0, 0.33, 0.50),
        (67, 0, 0.78, 1.17),
        (68, 0, 0.16, 0.26),
        (69, 0, 0.33, 0.50),
        (70, 0, 0.19, 0.29),
        (71, 0, 0.19, 0.29),
        (72, 0, 0.44, 0.66),
        (73, 0, 0.84, 1.26),
        (73, 1, 0.27, 0.41),
        (74, 0, 0.84, 1.26),
        (74, 1, 0.38, 0.57),
    ];
    for (position, variant, a, b) in rows {
        for (column, expected) in [(PsiColumn::A, a), (PsiColumn::B, b)] {
            let kernel = forfait_psi(position, variant, column)
                .unwrap_or_else(|| panic!("position {position}/{variant} {column:?}"));
            close(
                &format!("ψ {position}/{variant} {column:?}"),
                kernel,
                expected,
            );
        }
    }
    // Position 69 separates two zones (note under table I.2): half of ℓ·ψ per zone.
    let result = envelope(
        json!({"elements": [{"id": "grille", "element": {"kind": "ventilation_grille"}}],
        "forfaitBridges": [
        {"id": "p69", "position": 69, "column": "b", "lengthM": 8, "shared": true, "description": "r76"},
        {"id": "p73", "position": 73, "variant": 1, "column": "a", "lengthM": 5, "description": "r76"}]}),
    );
    for (id, psi, coefficient) in [("p69", 0.50, 2.0), ("p73", 0.27, 1.35)] {
        let bridge = result["bridges"]
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

fn forfait_opaque(
    id: &str,
    element: &str,
    building: &str,
    year: i32,
    insulation: Value,
    cavity: bool,
) -> Value {
    let building = match building {
        "floating" => json!({"kind": "floating", "newBerthSince2018": false}),
        "floating_new" => json!({"kind": "floating", "newBerthSince2018": true}),
        kind => json!({ "kind": kind }),
    };
    json!({"id": id, "element": {"kind": "forfait_opaque", "element": {"element": element, "building": building,
        "constructionYear": year, "insulation": insulation, "cavity": cavity}}})
}

/// (element, building, build year, insulation present with unknown thickness, cavity, R_c, R_si).
type ForfaitCase = (&'static str, &'static str, i32, bool, bool, f64, f64);

/// Figure I.4 (p. 827) with tables I.4 (p. 828), I.5 (p. 829), I.6 (p. 830) and I.7 (p. 831–832);
/// U_C = 1/(R_c + R_si + R_se) of I.1 (p. 826–827) rounded to 2 decimals, R_si of table C.2 by the usual
/// heat-flow direction (façade 0,13, floor 0,17, roof and attic floor 0,10).
#[test]
fn forfait_closed_elements_by_year() {
    let mut cases: Vec<ForfaitCase> = vec![
        // Table I.4, columns "onbekend of afwezig" and "(na)geïsoleerd".
        ("facade", "regular", 1950, false, true, 0.35, 0.13),
        ("facade", "regular", 1964, false, false, 0.19, 0.13),
        ("facade", "regular", 1950, true, true, 0.85, 0.13),
        ("facade", "regular", 1950, true, false, 0.69, 0.13),
        ("floor", "regular", 1950, false, true, 0.33, 0.17),
        ("floor", "regular", 1950, false, false, 0.15, 0.17),
        ("floor", "regular", 1950, true, true, 0.83, 0.17),
        ("floor", "regular", 1950, true, false, 0.65, 0.17),
        ("attic_floor", "regular", 1950, true, true, 0.83, 0.10),
        ("roof", "regular", 1950, false, true, 0.35, 0.10),
        ("roof", "regular", 1950, false, false, 0.22, 0.10),
        ("roof", "regular", 1950, true, true, 0.85, 0.10),
        ("roof", "regular", 1950, true, false, 0.72, 0.10),
        ("facade", "caravan", 1960, false, false, 0.19, 0.13),
        ("roof", "floating", 1960, true, true, 0.85, 0.10),
        // Present with an undeterminable thickness from 1965: table I.5.
        ("facade", "regular", 1990, true, true, 2.0, 0.13),
        ("attic_floor", "regular", 1980, false, false, 0.52, 0.10),
    ];
    // Table I.5 by build-year class ("van 1965 tot 1975" includes 1965, excludes 1975).
    for (year, r_c) in [
        (1965, 0.43),
        (1974, 0.43),
        (1975, 1.3),
        (1983, 1.3),
        (1987, 1.3),
        (1988, 2.0),
        (1991, 2.0),
        (1992, 2.5),
        (2013, 2.5),
        (2014, 3.5),
        (2015, 4.5),
        (2020, 4.5),
        (2021, 4.7),
        (2025, 4.7),
    ] {
        cases.push(("facade", "regular", year, false, false, r_c, 0.13));
    }
    for (year, r_c) in [
        (1965, 0.17),
        (1975, 0.52),
        (1983, 1.3),
        (1988, 1.3),
        (1992, 2.5),
        (2014, 3.5),
        (2020, 3.5),
        (2021, 3.7),
    ] {
        cases.push(("floor", "regular", year, false, false, r_c, 0.17));
    }
    for (year, r_c) in [
        (1965, 0.86),
        (1975, 1.3),
        (1983, 1.3),
        (1988, 2.0),
        (1992, 2.5),
        (2014, 3.5),
        (2015, 6.0),
        (2021, 6.3),
    ] {
        cases.push(("roof", "regular", year, false, false, r_c, 0.10));
    }
    // Table I.6 (caravans; 0,19 for façades 1965–1983).
    for (element, year, r_c, r_si) in [
        ("facade", 1965, 0.19, 0.13),
        ("facade", 1982, 0.19, 0.13),
        ("facade", 1983, 1.3, 0.13),
        ("facade", 1991, 1.3, 0.13),
        ("facade", 1992, 2.0, 0.13),
        ("facade", 2014, 2.5, 0.13),
        ("facade", 2021, 2.6, 0.13),
        ("floor", 1970, 0.17, 0.17),
        ("floor", 1983, 1.3, 0.17),
        ("floor", 1992, 2.0, 0.17),
        ("floor", 2014, 2.5, 0.17),
        ("floor", 2021, 2.6, 0.17),
        ("roof", 1970, 0.22, 0.10),
        ("roof", 1983, 1.3, 0.10),
        ("roof", 1992, 2.0, 0.10),
        ("roof", 2014, 2.5, 0.10),
        ("roof", 2021, 2.6, 0.10),
    ] {
        cases.push((element, "caravan", year, false, false, r_c, r_si));
    }
    // Table I.7 (floating buildings), with the rows for a berth that is new on 1 January 2018.
    for (element, year, building, r_c, r_si) in [
        ("facade", 1965, "floating", 0.43, 0.13),
        ("facade", 1975, "floating", 0.43, 0.13),
        ("facade", 1983, "floating", 1.3, 0.13),
        ("facade", 1988, "floating", 2.0, 0.13),
        ("facade", 1992, "floating", 2.5, 0.13),
        ("facade", 2014, "floating", 3.5, 0.13),
        ("facade", 2015, "floating", 4.5, 0.13),
        ("facade", 2017, "floating_new", 4.5, 0.13),
        ("facade", 2018, "floating", 3.5, 0.13),
        ("facade", 2021, "floating", 3.7, 0.13),
        ("facade", 2018, "floating_new", 4.5, 0.13),
        ("facade", 2021, "floating_new", 4.7, 0.13),
        ("roof", 1965, "floating", 0.35, 0.10),
        ("roof", 1975, "floating", 0.35, 0.10),
        ("roof", 1983, "floating", 1.3, 0.10),
        ("roof", 1988, "floating", 2.0, 0.10),
        ("roof", 1992, "floating", 2.5, 0.10),
        ("roof", 2014, "floating", 3.5, 0.10),
        ("roof", 2015, "floating", 6.0, 0.10),
        ("roof", 2018, "floating", 4.5, 0.10),
        ("roof", 2021, "floating", 4.7, 0.10),
        ("roof", 2018, "floating_new", 6.0, 0.10),
        ("roof", 2021, "floating_new", 6.3, 0.10),
    ] {
        cases.push((element, building, year, false, false, r_c, r_si));
    }
    let insulation = |present: bool| {
        if present {
            json!({"kind": "present_unknown_thickness"})
        } else {
            json!({"kind": "absent_or_unknown"})
        }
    };
    let elements: Vec<Value> = cases
        .iter()
        .enumerate()
        .map(
            |(index, &(element, building, year, present, cavity, _, _))| {
                forfait_opaque(
                    &format!("c{index}"),
                    element,
                    building,
                    year,
                    insulation(present),
                    cavity,
                )
            },
        )
        .collect();
    let result = envelope(json!({ "elements": elements }));
    for (index, &(kind, building, year, _, _, r_c, r_si)) in cases.iter().enumerate() {
        let u = (100.0 / (r_c + r_si + 0.04)).round() / 100.0;
        let item = element(&result, &format!("c{index}"));
        let label = format!("{kind} {building} {year}");
        close(
            &format!("{label} R_c"),
            num(&item["forfait"]["rC"], "rC"),
            r_c,
        );
        close(
            &format!("{label} U_C"),
            num(&item["forfait"]["uC"], "uC"),
            u,
        );
        close(&format!("{label} U"), num(&item["uValue"], "u"), u);
    }
    // Hull of a floating building (table I.7): R_c only.
    let hull = [
        (1965, "floating", 0.17),
        (1975, "floating", 0.17),
        (1983, "floating", 0.17),
        (1988, "floating", 0.17),
        (1992, "floating", 2.5),
        (2014, "floating", 2.5),
        (2015, "floating", 3.5),
        (2018, "floating", 2.5),
        (2021, "floating", 2.6),
        (2018, "floating_new", 3.5),
        (2021, "floating_new", 3.7),
    ];
    let elements: Vec<Value> = hull
        .iter()
        .enumerate()
        .map(|(index, &(year, building, _))| {
            forfait_opaque(
                &format!("h{index}"),
                "floating_hull",
                building,
                year,
                insulation(false),
                false,
            )
        })
        .collect();
    let result = envelope(json!({ "elements": elements }));
    for (index, (year, building, r_c)) in hull.into_iter().enumerate() {
        let item = element(&result, &format!("h{index}"));
        close(
            &format!("hull {building} {year}"),
            num(&item["forfait"]["rC"], "rC"),
            r_c,
        );
    }
}

/// I.2 (p. 832–833) for a known insulation thickness: d_iso rounded to 10 mm (25 → 30) unless proven, λ_equi;ntr 0,045 or a
/// known higher value, R_ad 0,36/0,15/0,22, the cavity supplement 0,16/0,13/0,18 up to 30 mm, reed thatch I.3
/// with d rounded down to 0,05 m and thermal cushions (R_ad 0,15 + 1,8, no other thickness); U_C of I.1 rounded to 2 decimals. Towards an
/// unheated space R_se becomes the R_si of table C.2 (8.4.2.1, p. 266).
#[test]
fn forfait_closed_elements_known_thickness() {
    let known = |mm: f64, extra: Value| {
        let mut insulation = json!({"kind": "known_thickness", "thicknessMm": mm});
        for (key, item) in extra.as_object().expect("extra") {
            insulation[key] = item.clone();
        }
        insulation
    };
    // (id, element, insulation, cavity, R_c, R_si + R_se)
    let cases = [
        (
            "facade_104",
            "facade",
            known(104.0, json!({})),
            true,
            2.582222222222222,
            0.17,
        ),
        (
            "facade_24_cavity",
            "facade",
            known(24.0, json!({})),
            true,
            0.9644444444444445,
            0.17,
        ),
        (
            "facade_25_cavity",
            "facade",
            known(25.0, json!({})),
            true,
            1.1866666666666665,
            0.17,
        ),
        (
            "facade_104_proven",
            "facade",
            known(104.0, json!({"thicknessProven": true})),
            false,
            2.6711111111111108,
            0.17,
        ),
        (
            "facade_lambda_050",
            "facade",
            known(100.0, json!({"knownLambdaEquivalent": 0.05})),
            false,
            2.36,
            0.17,
        ),
        (
            "facade_lambda_040",
            "facade",
            known(100.0, json!({"knownLambdaEquivalent": 0.04})),
            false,
            2.582222222222222,
            0.17,
        ),
        (
            "roof_reed",
            "roof",
            known(150.0, json!({"reedThicknessM": 0.27})),
            false,
            5.934285714285714,
            0.14,
        ),
        (
            "roof_30_cavity",
            "roof",
            known(30.0, json!({})),
            true,
            1.0166666666666666,
            0.14,
        ),
        (
            "floor_80",
            "floor",
            known(80.0, json!({})),
            false,
            1.9277777777777778,
            0.21,
        ),
        (
            "floor_cushions",
            "floor",
            known(0.0, json!({"thermalCushions": true})),
            false,
            1.95,
            0.21,
        ),
        (
            "floor_30_cavity",
            "floor",
            known(30.0, json!({})),
            true,
            0.03 / 0.045 + 0.15 + 0.18,
            0.21,
        ),
        (
            "floor_40_cavity",
            "floor",
            known(40.0, json!({})),
            true,
            1.038888888888889,
            0.21,
        ),
        (
            "attic_120",
            "attic_floor",
            known(120.0, json!({})),
            false,
            2.8166666666666664,
            0.14,
        ),
    ];
    let mut elements: Vec<Value> = cases
        .iter()
        .map(|(id, element, insulation, cavity, _, _)| {
            forfait_opaque(id, element, "regular", 1990, insulation.clone(), *cavity)
        })
        .collect();
    let mut unheated = forfait_opaque(
        "towards_unheated",
        "facade",
        "regular",
        1980,
        json!({"kind": "absent_or_unknown"}),
        false,
    );
    unheated["element"]["element"]["towardsUnheatedSpace"] = json!(true);
    elements.push(unheated);
    let mut override_rsi = forfait_opaque(
        "r_si_override",
        "facade",
        "regular",
        1980,
        json!({"kind": "absent_or_unknown"}),
        false,
    );
    override_rsi["element"]["element"]["rSiOverride"] = json!(0.17);
    elements.push(override_rsi);
    let result = envelope(json!({ "elements": elements }));
    for (id, _, _, _, r_c, surfaces) in cases {
        let item = element(&result, id);
        close(&format!("{id} R_c"), num(&item["forfait"]["rC"], "rC"), r_c);
        close(
            &format!("{id} U_C"),
            num(&item["forfait"]["uC"], "uC"),
            (100.0 / (r_c + surfaces)).round() / 100.0,
        );
    }
    close(
        "towards unheated",
        num(&element(&result, "towards_unheated")["forfait"]["uC"], "uC"),
        0.64,
    );
    close(
        "R_si override",
        num(&element(&result, "r_si_override")["forfait"]["uC"], "uC"),
        0.66,
    );
}

/// Figure I.4 (p. 827): "Is er isolatie aanwezig? nee" leads to I.2.1.2 (table I.4, column "isolatie onbekend of
/// afwezig") for every build year; only "onbekend" goes through the build-year question to table I.5 from 1965.
/// A façade with cavity from 1970 that is known to be uninsulated: R_c 0,35, U_C 1/(0,35 + 0,17) = 1,92.
#[test]
fn forfait_known_absent_insulation_from_1965_uses_table_i4() {
    let result = envelope(
        json!({"elements": [forfait_opaque("absent", "facade", "regular",
        1970, json!({"kind": "absent"}), true)]}),
    );
    check(&result, "absent", "forfait", &[("rC", 0.35), ("uC", 1.92)]);
}

/// Tables I.8 and I.9 (p. 834–835), I.10 (p. 835), I.2.2.1 for doors under 65 % glass, tables I.11/I.12
/// (p. 836–837) and I.15/I.16 (p. 838–840) with the thickness rounded to 10 mm.
#[test]
fn forfait_windows_doors_and_panels() {
    use ForfaitGlass::*;
    let frames = [
        FrameGroup::WoodOrPlastic,
        FrameGroup::MetalWithThermalBreak,
        FrameGroup::MetalWithoutThermalBreak,
    ];
    let windows: [(ForfaitGlass, [f64; 3], [f64; 3]); 7] = [
        (TripleHr, [1.4, 1.9, 2.7], [1.3, 1.6, 2.2]),
        (HrPlusPlus, [1.8, 2.3, 3.1], [1.5, 1.9, 2.4]),
        (HrPlus, [2.0, 2.5, 3.3], [1.7, 2.1, 2.6]),
        (HrCoatedDouble, [2.3, 2.8, 3.6], [1.9, 2.2, 2.7]),
        (DoubleUncoated, [2.9, 3.3, 4.1], [2.3, 2.5, 3.0]),
        (SecondaryWindow, [2.9, 3.3, 4.1], [2.3, 2.5, 3.0]),
        (Single, [5.1, 5.4, 6.2], [3.5, 3.6, 4.0]),
    ];
    for (glass, exterior, interior) in windows {
        for (index, frame) in frames.into_iter().enumerate() {
            close(
                &format!("I.8 {glass:?} {frame:?}"),
                forfait_window_u(glass, frame, true),
                exterior[index],
            );
            close(
                &format!("I.9 {glass:?} {frame:?}"),
                forfait_window_u(glass, frame, false),
                interior[index],
            );
        }
    }
    close("I.10 insulated outside", forfait_door_u(true, true), 2.0);
    close("I.10 insulated inside", forfait_door_u(true, false), 1.7);
    close("I.10 uninsulated outside", forfait_door_u(false, true), 3.4);
    close("I.10 uninsulated inside", forfait_door_u(false, false), 2.7);
    close(
        "door 40 % HR++",
        forfait_glazed_door_u(0.4, HrPlusPlus, FrameGroup::WoodOrPlastic, true, true),
        0.4 * 1.8 + 0.6 * 2.0,
    );
    close(
        "door 30 % single inside",
        forfait_glazed_door_u(0.3, Single, FrameGroup::MetalWithThermalBreak, false, false),
        0.3 * 3.6 + 0.7 * 2.7,
    );
    // At least 65 % glass: table I.8 itself (I.2.2.1).
    close(
        "door 70 % HR++",
        forfait_glazed_door_u(0.7, HrPlusPlus, FrameGroup::WoodOrPlastic, true, true),
        1.8,
    );
    // Tables I.11/I.12: (insulation, cavity) rows.
    let unknown: [(PanelInsulation, bool, [f64; 3], [f64; 3]); 4] = [
        (
            PanelInsulation::AbsentOrUnknown,
            false,
            [3.7, 4.1, 4.9],
            [2.8, 3.0, 3.4],
        ),
        (
            PanelInsulation::AbsentOrUnknown,
            true,
            [2.5, 2.8, 3.6],
            [2.0, 2.3, 2.7],
        ),
        (
            PanelInsulation::PresentUnknownThickness,
            false,
            [1.5, 1.9, 2.7],
            [1.3, 1.6, 2.2],
        ),
        (
            PanelInsulation::PresentUnknownThickness,
            true,
            [1.4, 1.7, 2.5],
            [1.2, 1.5, 2.1],
        ),
    ];
    for (insulation, cavity, exterior, interior) in unknown {
        for (index, frame) in frames.into_iter().enumerate() {
            for (outside, expected) in [(true, exterior[index]), (false, interior[index])] {
                let label = format!("I.11/I.12 {insulation:?} {cavity} {frame:?} {outside}");
                close(
                    &label,
                    forfait_panel_u(insulation, cavity, frame, outside).expect("panel"),
                    expected,
                );
                // 2025+C1 has no build-year tables for panels (those were I.13/I.14 of 2023).
                close(
                    &label,
                    forfait_panel_u_in_edition(insulation, cavity, frame, outside, Some(1990))
                        .expect("panel"),
                    expected,
                );
            }
        }
    }
    // Tables I.15/I.16: thickness with (no cavity, cavity) for 10–30 mm.
    let thin: [ThinPanelRow; 3] = [
        (
            10.0,
            [[2.0, 2.4, 3.2], [1.7, 2.0, 2.8]],
            [[1.7, 2.0, 2.5], [1.5, 1.7, 2.3]],
        ),
        (
            20.0,
            [[1.5, 1.9, 2.7], [1.4, 1.7, 2.5]],
            [[1.3, 1.6, 2.2], [1.2, 1.5, 2.1]],
        ),
        (
            30.0,
            [[1.3, 1.6, 2.4], [1.2, 1.5, 2.3]],
            [[1.2, 1.4, 2.0], [1.1, 1.4, 1.9]],
        ),
    ];
    let panel = |mm: f64, cavity: bool, frame: FrameGroup, outside: bool| {
        forfait_panel_u(
            PanelInsulation::KnownThickness { thickness_mm: mm },
            cavity,
            frame,
            outside,
        )
        .unwrap_or_else(|| panic!("panel {mm}"))
    };
    for (mm, exterior, interior) in thin {
        for (cavity_index, cavity) in [false, true].into_iter().enumerate() {
            for (index, frame) in frames.into_iter().enumerate() {
                close(
                    &format!("I.15 {mm} {cavity} {frame:?}"),
                    panel(mm, cavity, frame, true),
                    exterior[cavity_index][index],
                );
                close(
                    &format!("I.16 {mm} {cavity} {frame:?}"),
                    panel(mm, cavity, frame, false),
                    interior[cavity_index][index],
                );
            }
        }
    }
    let thick: [(f64, [f64; 3], [f64; 3]); 27] = [
        (40.0, [1.1, 1.5, 2.3], [1.0, 1.3, 1.9]),
        (50.0, [1.0, 1.4, 2.2], [1.0, 1.2, 1.8]),
        (60.0, [0.98, 1.3, 2.1], [0.9, 1.2, 1.8]),
        (70.0, [0.93, 1.3, 2.1], [0.9, 1.2, 1.8]),
        (80.0, [0.90, 1.2, 2.0], [0.8, 1.1, 1.7]),
        (90.0, [0.87, 1.2, 2.0], [0.8, 1.1, 1.7]),
        (100.0, [0.84, 1.2, 2.0], [0.8, 1.1, 1.7]),
        (110.0, [0.82, 1.2, 2.0], [0.8, 1.1, 1.7]),
        (120.0, [0.80, 1.2, 2.0], [0.8, 1.0, 1.7]),
        (130.0, [0.79, 1.1, 1.9], [0.7, 1.0, 1.7]),
        (140.0, [0.78, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (150.0, [0.77, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (160.0, [0.76, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (170.0, [0.75, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (180.0, [0.74, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (190.0, [0.73, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (200.0, [0.73, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (210.0, [0.72, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (220.0, [0.71, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (230.0, [0.71, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (240.0, [0.71, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (250.0, [0.70, 1.1, 1.9], [0.7, 1.0, 1.6]),
        (260.0, [0.70, 1.0, 1.8], [0.7, 1.0, 1.6]),
        (270.0, [0.69, 1.0, 1.8], [0.7, 1.0, 1.6]),
        (280.0, [0.69, 1.0, 1.8], [0.7, 1.0, 1.6]),
        (290.0, [0.69, 1.0, 1.8], [0.6, 0.9, 1.6]),
        (300.0, [0.69, 1.0, 1.8], [0.6, 0.9, 1.6]),
    ];
    for (mm, exterior, interior) in thick {
        for (index, frame) in frames.into_iter().enumerate() {
            for cavity in [false, true] {
                close(
                    &format!("I.15 {mm} {frame:?}"),
                    panel(mm, cavity, frame, true),
                    exterior[index],
                );
                close(
                    &format!("I.16 {mm} {frame:?}"),
                    panel(mm, cavity, frame, false),
                    interior[index],
                );
            }
        }
    }
    // 104 mm rounds to 100 mm, 156 mm to 160 mm.
    close(
        "I.15 104 mm",
        panel(104.0, false, FrameGroup::WoodOrPlastic, true),
        0.84,
    );
    close(
        "I.16 156 mm",
        panel(156.0, false, FrameGroup::MetalWithThermalBreak, false),
        1.0,
    );
}

/// 8.5 (p. 228) U_T = L_C/A_con plus ΔU; 8.25–8.29 (p. 243–247) and table 8.5 (p. 247); 8.14 and 8.18
/// (p. 235–238) with the glazing bars ℓ_gb·ψ_gb.
#[test]
fn numerical_route_and_glazing_bars() {
    let result = envelope(json!({"elements": [
        {"id": "numerical", "element": {"kind": "numerical", "couplingWPerK": 12.3, "constructionAreaM2": 40,
            "deltaUWPerM2K": 0.01, "sourceReference": "r76"}},
        {"id": "bars_8_14", "element": {"kind": "window", "window": {"method": {"method": "detailed",
            "glazing": [{"areaM2": 1.2, "glazing": {"method": "declared", "value": 1.1, "sourceReference": "r76"},
                "perimeterM": 4.6, "edge": {"method": "declared", "value": 0.06, "sourceReference": "r76"}}],
            "frameAreaM2": 0.5, "frameUWPerM2K": 1.5, "frameSourceReference": "r76",
            "glazingBars": {"lengthM": 2.4, "psiWPerMk": 0.04, "sourceReference": "r76"}}}}},
        {"id": "bars_8_18", "element": {"kind": "window", "window": {"method": {"method": "detailed",
            "glazing": [{"areaM2": 1.2, "glazing": {"method": "declared", "value": 1.1, "sourceReference": "r76"},
                "perimeterM": 4.6, "edge": {"method": "declared", "value": 0.06, "sourceReference": "r76"}}],
            "panels": [{"areaM2": 0.3, "uValueWPerM2k": 0.5, "perimeterM": 2.2, "psiWPerMk": 0.03,
                "sourceReference": "r76"}],
            "frameAreaM2": 0.5, "frameUWPerM2K": 1.5, "frameSourceReference": "r76",
            "glazingBars": {"lengthM": 2.4, "psiWPerMk": 0.04, "sourceReference": "r76"}}}}},
    ]}));
    check(
        &result,
        "numerical",
        "",
        &[("uUnrounded", 0.3175), ("uValue", 0.32)],
    );
    check(
        &result,
        "bars_8_14",
        "window",
        &[("uW", 1.4364705882352942), ("uRounded", 1.4)],
    );
    check(
        &result,
        "bars_8_18",
        "window",
        &[("uW", 1.3289999999999997), ("uRounded", 1.3)],
    );
    // 8.25 as written: U_p = λ_p/d_p = 0,035/0,024 (R_T = R_p, p. 243).
    close(
        "U_fr",
        frame_u_from_l2d(0.42, 0.024, 0.19, 0.1),
        1.4291666666666663,
    );
    close(
        "ψ (8.26)",
        linear_psi(0.55, &[(1.2, 0.2), (1.0, 0.25)]),
        0.06,
    );
    close("ψ_gl (8.27)", edge_psi(0.36, 1.4, 0.11, 1.1, 0.19), -0.003);
    close(
        "χ (8.29)",
        point_chi(2.1, &[(4.0, 0.2), (2.0, 0.25)], &[(2.0, 0.05)]),
        0.7,
    );
    // 8.2.4.1: at least once per 20 m², R_c ≥ 3,5 and a cross-section above table 8.5.
    use PointBridgeMaterial::*;
    for (material, area, per_m2, r_c, counts) in [
        (Concrete, 0.05, 0.06, 3.6, true),
        (Concrete, 0.040, 0.06, 3.6, false),
        (Concrete, 0.05, 0.049, 3.6, false),
        (Concrete, 0.05, 0.05, 3.5, true),
        (Concrete, 0.05, 0.06, 3.49, false),
        (StainlessSteel, 0.0061, 0.1, 4.0, true),
        (StainlessSteel, 0.006, 0.1, 4.0, false),
        (Steel, 0.0021, 0.1, 4.0, true),
        (Steel, 0.002, 0.1, 4.0, false),
        (Aluminium, 0.0007, 0.1, 4.0, true),
        (Aluminium, 0.0006, 0.1, 4.0, false),
    ] {
        assert_eq!(
            point_bridge_counts(material, area, per_m2, r_c),
            counts,
            "{material:?} {area} {per_m2} {r_c}"
        );
    }
}
