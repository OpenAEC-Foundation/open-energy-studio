//! Review 75: heat transfer to the ground beyond the plain slab on ground, recalculated independently
//! from NTA 8800:2025+C1:2026 §8.3 (p. 249-264) and annex D (p. 790-796): crawlspace (8.43-8.48,
//! D.13/D.14), unheated basement (8.49, D.15/D.16), heated basement (8.38/8.39, 8.42, 8.45, D.10-D.12),
//! partly sunken floors (8.3.3.1 note 3), edge insulation (D.7-D.9, table D.1), detailed and forfait
//! floor-edge bridges (8.36/8.37), the monthly H_g;an;mi and the seasonal H_H/C;g;adj (D.1-D.4), the
//! ground terms of 7.14/7.15 and 7.57/7.58, and the July balance of TOjuli (5.40, p. 115-119). Every
//! case is the example terraced dwelling with its ground floor replaced; the single-orientation cases
//! turn the north surfaces to the south so the one assessed orientation carries share 1. The expected
//! values come from a separate implementation written from the norm pages; only numbers and page
//! references are kept here.

use nta8800_core::project_performance::assess_project_performance;
use serde_json::Value;

const BASE: &str = include_str!("../../../training-data/nta8800-example-terraced-dwelling.json");
/// Table 17.1 (p. 690): θ_e;avg;mi and t_mi.
const OUTDOOR: [f64; 12] = [
    2.61, 4.82, 5.91, 9.32, 14.73, 16.12, 18.05, 18.48, 15.63, 10.40, 7.99, 4.00,
];
const HOURS: [f64; 12] = [
    744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0,
];
/// θ_e of D.4 (p. 791), used as θ_e;avg;an in D.1-D.3 and 7.14/7.15.
const ANNUAL_MEAN: f64 = 10.67;

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
        .unwrap_or_else(|| panic!("{label}: missing number"))
}

/// The example dwelling with `floors` as its ground floors; `split` divides the 50 m² floor into
/// 30 m² (`floor`) and 20 m² (`floor2`); `one_orientation` turns the north surfaces to the south.
fn performance(floors: &str, split: bool, one_orientation: bool) -> Value {
    let mut project: Value = serde_json::from_str(BASE).expect("base");
    project["ntaCalculation"]["groundFloors"] = serde_json::from_str(floors).expect("floors");
    let surfaces = project["zones"][0]["surfaces"]
        .as_array_mut()
        .expect("surfaces");
    if split {
        let floor = surfaces
            .iter_mut()
            .find(|surface| surface["id"] == "floor")
            .expect("floor");
        floor["area"] = 30.0.into();
        let mut second = floor.clone();
        second["id"] = "floor2".into();
        second["area"] = 20.0.into();
        surfaces.push(second);
    }
    if one_orientation {
        for surface in surfaces.iter_mut() {
            if surface["orientation"] == "N" {
                surface["orientation"] = "S".into();
                for window in surface["windows"].as_array_mut().expect("windows") {
                    window["orientation"] = "S".into();
                }
            }
        }
    }
    let result = assess_project_performance(&project);
    assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
    serde_json::to_value(result.performance.expect("performance")).expect("serializable")
}

struct Expected {
    /// H_g of 8.36-8.39, summed over the floors, W/K.
    steady: f64,
    /// D.1 H_g;an;mi, W/K.
    monthly: [f64; 12],
    /// D.2 H_H;g;adj and D.3 H_C;g;adj, W/K.
    heating_adjusted: f64,
    cooling_adjusted: f64,
    /// 5.40 denominator H_C;D;juli;or + H_gr;an;juli;or + H_C;ve;juli;or at share 1, W/K.
    july_conductance: f64,
    /// Q_C;nd;juli;or of 7.2.2 at share 1 (time constant 7.58 with H_C;g;adj), kWh.
    july_cooling_need: f64,
    tojuli: f64,
}

fn check(floors: &str, split: bool, expected: &Expected) {
    let p = performance(floors, split, true);
    let demand = &p["spaceHeating"]["demand"];
    let transmission = &demand["transmission"];
    close(
        "H_g",
        num(&transmission["groundSteadyConductanceWPerK"], "H_g"),
        expected.steady,
    );
    close(
        "H_H;g;adj",
        num(&transmission["groundHeatingAdjustedWPerK"], "adjH"),
        expected.heating_adjusted,
    );
    close(
        "H_C;g;adj",
        num(&transmission["groundCoolingAdjustedWPerK"], "adjC"),
        expected.cooling_adjusted,
    );
    close(
        "theta_e;avg;an",
        num(&transmission["annualMeanOutdoorTemperatureC"], "θe"),
        ANNUAL_MEAN,
    );
    let h_tr = num(&transmission["conductanceWPerK"], "H_tr");
    let h_pipes = num(&transmission["verticalPipeConductanceWPerK"], "H_p");
    let capacity = num(&demand["specificHeatCapacityKjPerM2k"], "c") * 1000.0 * 100.0;
    for (index, month) in demand["monthly"]
        .as_array()
        .expect("months")
        .iter()
        .enumerate()
    {
        let h_g = expected.monthly[index];
        close(
            &format!("H_g;an m{}", index + 1),
            num(
                &transmission["groundMonthlyConductanceWPerK"][index],
                "H_g;an",
            ),
            h_g,
        );
        close(
            &format!("month H_g;an m{}", index + 1),
            num(&month["groundConductanceWPerK"], "H_g;an"),
            h_g,
        );
        for (service, adjusted) in [
            ("heating", expected.heating_adjusted),
            ("cooling", expected.cooling_adjusted),
        ] {
            let balance = &month[service];
            let theta = num(&balance["calculationTemperatureC"], "θ_calc");
            // 7.14/7.15: ground term on the annual mean, the rest on the monthly mean.
            let ground = h_g * (theta - ANNUAL_MEAN) * HOURS[index] / 1000.0;
            close(
                &format!("{service} ground Q_tr m{}", index + 1),
                num(&balance["groundTransmissionKwh"], "ground Q"),
                ground,
            );
            close(
                &format!("{service} Q_tr m{}", index + 1),
                num(&balance["transmissionKwh"], "Q_tr"),
                h_tr * (theta - OUTDOOR[index]) * HOURS[index] / 1000.0 + ground,
            );
            // 7.57/7.58: the seasonal H_H/C;g;adj in the time constant.
            let ventilation = num(&balance["ventilationConductanceWPerK"], "H_ve");
            close(
                &format!("{service} tau m{}", index + 1),
                num(&balance["timeConstantH"], "τ"),
                capacity / 3600.0 / (h_tr + adjusted + ventilation),
            );
        }
    }
    // 5.40 with share 1: H_C;p = 0, H_gr;an;juli = H_g;an of July.
    let july = &demand["monthly"][6]["cooling"];
    let orientation = p["tojuli"][0]["orientations"]
        .as_array()
        .expect("orientations")
        .iter()
        .find(|item| item["assessed"] == true)
        .expect("assessed orientation");
    assert_eq!(num(&orientation["share"], "share"), 1.0);
    close(
        "TOjuli H",
        num(&orientation["conductanceWPerK"], "H_or"),
        h_tr - h_pipes + expected.monthly[6] + num(&july["ventilationConductanceWPerK"], "H_ve"),
    );
    close(
        "TOjuli H (independent)",
        num(&orientation["conductanceWPerK"], "H_or"),
        expected.july_conductance,
    );
    close(
        "Q_C;nd;juli;or",
        num(&orientation["coolingNeedJulyKwh"], "Q"),
        expected.july_cooling_need,
    );
    close(
        "TOjuli",
        num(&orientation["tojuliK"], "TOjuli"),
        expected.tojuli,
    );
}

/// Crawlspace, z = 0,5 m class, uninsulated crawlspace floor, default ε, forfait edge 8.37: 8.33/8.34, 8.40, 8.43-8.48, D.13/D.14, β = 0 (p. 251-263, 792-795).
#[test]
fn crawl_forfait() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","below":{"kind":"crawlspace","depthClass":"other","wallResistanceM2kPerW":2.5,"wallUValueWPerM2k":0.35}}]"#,
        false,
        &Expected {
            steady: 13.406953546124383,
            monthly: [
                14.34865148280622,
                14.222487881982245,
                13.877802514465303,
                13.406953546124383,
                12.936104577783464,
                12.59141921026652,
                12.465255609442545,
                12.591419210266519,
                12.936104577783462,
                13.406953546124383,
                13.877802514465303,
                14.222487881982245,
            ],
            heating_adjusted: 21.064034116962166,
            cooling_adjusted: 8.282969399277379,
            july_conductance: 180.80836482223043,
            july_cooling_need: 63.21872366985747,
            tojuli: 0.47,
        },
    );
}

/// Crawlspace on sand (z = 0, wall term vanishes), insulated crawlspace floor, given ε, detailed edge 8.36 (p. 253-263).
#[test]
fn crawl_sand_detailed() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":2.67,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":6,"psiWPerMk":0.12,"sourceReference":"r75"},{"lengthM":4,"psiWPerMk":-0.05,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","below":{"kind":"crawlspace","depthClass":"on_sand","floorResistanceM2kPerW":0.5,"wallResistanceM2kPerW":1.3,"wallUValueWPerM2k":0.6,"ventilationOpeningM2PerM":0.003}}]"#,
        false,
        &Expected {
            steady: 10.819053014837813,
            monthly: [
                11.66671999940985,
                11.55315415742655,
                11.242886507123831,
                10.819053014837813,
                10.395219522551796,
                10.084951872249079,
                9.971386030265776,
                10.084951872249079,
                10.395219522551795,
                10.819053014837813,
                11.242886507123831,
                11.55315415742655,
            ],
            heating_adjusted: 17.08026910061323,
            cooling_adjusted: 6.6488765887289,
            july_conductance: 158.0599421433232,
            july_cooling_need: 89.73373566128339,
            tojuli: 0.77,
        },
    );
}

/// Uninsulated floor above a poorly ventilated crawlspace with a high-U wall above ground (8.43, 8.47, 8.48; D.13/D.14). The north orientation is cut off by 7.6 (1/γ_C > 2) in the two-orientation run.
#[test]
fn crawl_uninsulated() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":18,"constructionResistanceM2kPerW":0.32,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","below":{"kind":"crawlspace","depthClass":"other","wallResistanceM2kPerW":0.35,"wallUValueWPerM2k":1.9,"ventilationOpeningM2PerM":0.0005}}]"#,
        false,
        &Expected {
            steady: 44.41572117034779,
            monthly: [
                54.50311896191936,
                53.15166391592781,
                49.45942006613358,
                44.41572117034779,
                39.372022274562,
                35.67977842476776,
                34.32832337877621,
                35.67977842476776,
                39.37202227456199,
                44.41572117034778,
                49.45942006613358,
                53.15166391592781,
            ],
            heating_adjusted: 76.30690351400138,
            cooling_adjusted: 24.640655718350363,
            july_conductance: 202.6714325915641,
            july_cooling_need: 31.924697921112852,
            tojuli: 0.22,
        },
    );
}

/// Unheated basement, forfait n = 0,3, 8.49 with the printed ρ_a·c_a = 1211 J/(m³K) (p. 263); D.15/D.16 with h = 0,125 m, U_x;T read as the wall U and 0,33·n·V (p. 795-796).
#[test]
fn basement_forfait() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","below":{"kind":"unheated_basement","depthClass":"other","wallResistanceM2kPerW":2.5,"wallUValueWPerM2k":0.35,"volumeM3":120}}]"#,
        false,
        &Expected {
            steady: 14.327818698959417,
            monthly: [
                15.902122718137818,
                16.773171398769044,
                16.988989798277018,
                16.4917495508018,
                15.414685779098619,
                14.04639685099217,
                12.753514679781018,
                11.88246599914979,
                11.666647599641818,
                12.163887847117037,
                13.240951618820215,
                14.609240546926667,
            ],
            heating_adjusted: 22.499689556438575,
            cooling_adjusted: 8.85667175531501,
            july_conductance: 181.0966238925689,
            july_cooling_need: 62.74193471196861,
            tojuli: 0.47,
        },
    );
}

/// Unheated basement on sand, n = 1,0, insulated basement floor, detailed edge; 8.49 with 1211 J/(m³K) (p. 263), D.15/D.16.
#[test]
fn basement_n_detailed() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":6,"psiWPerMk":0.12,"sourceReference":"r75"},{"lengthM":4,"psiWPerMk":-0.05,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","below":{"kind":"unheated_basement","depthClass":"on_sand","floorResistanceM2kPerW":1.2,"wallResistanceM2kPerW":0.8,"wallUValueWPerM2k":0.9,"volumeM3":90,"airChangesPerHour":1}}]"#,
        false,
        &Expected {
            steady: 10.47393657725794,
            monthly: [
                12.52536066603616,
                13.502525231846182,
                13.668181913453225,
                12.977943136799858,
                11.616757824675002,
                9.949354482211618,
                8.422512488479716,
                7.445347922669695,
                7.279691241062653,
                7.969930017716015,
                9.331115329840873,
                10.998518672304263,
            ],
            heating_adjusted: 17.059640033389744,
            cooling_adjusted: 6.211814075696168,
            july_conductance: 156.51106860153715,
            july_cooling_need: 95.79702658299686,
            tojuli: 0.83,
        },
    );
}

/// Heated basement, z = 2,4 m, forfait 8.38 with A_T;bw = z·P and ΔU_for of 8.3 (p. 225, 253-254); U_fl by 8.40 with z (8.3.4.1 note 2, p. 257); 8.45 with d1 = d_bw;equi < d_f;equi; D.10/D.11, β = 1 (p. 794).
#[test]
fn heated_forfait() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","heatedBasement":{"depthM":2.4,"wallResistanceM2kPerW":3}}]"#,
        false,
        &Expected {
            steady: 21.415065294545208,
            monthly: [
                21.03425692132444,
                21.970387754883863,
                22.75772038365667,
                23.18528966558853,
                23.138528756877438,
                22.629967205249876,
                21.795873667765974,
                20.859742834206553,
                20.072410205433744,
                19.644840923501885,
                19.691601832212978,
                20.200163383840543,
            ],
            heating_adjusted: 31.43665685058698,
            cooling_adjusted: 14.178543059422687,
            july_conductance: 190.13898288055384,
            july_cooling_need: 45.94074086834212,
            tojuli: 0.33,
        },
    );
}

/// Heated basement, forfait 8.38 with the entered ΣA_T;bw = 26,5 m² (p. 253-254).
#[test]
fn heated_forfait_area() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","heatedBasement":{"depthM":2.4,"wallResistanceM2kPerW":3,"wallAreaM2":26.5}}]"#,
        false,
        &Expected {
            steady: 22.38264258936396,
            monthly: [
                22.001834216143195,
                22.937965049702612,
                23.725297678475425,
                24.152866960407284,
                24.10610605169619,
                23.59754450006863,
                22.763450962584727,
                21.827320129025306,
                21.039987500252497,
                20.612418218320638,
                20.65917912703173,
                21.167740678659293,
            ],
            heating_adjusted: 32.89320809450224,
            cooling_adjusted: 14.803633317278212,
            july_conductance: 191.1065601753726,
            july_cooling_need: 44.512300456724006,
            tojuli: 0.32,
        },
    );
}

/// Heated basement with two wall parts (z_j = 2,6 and 1,1 m): 8.39 Σz_j·ℓ_j·U_bw;j at each z_j, z of 8.42/D.12 in U_fl and D.10/D.11 (p. 255-260, 794).
#[test]
fn heated_parts_detailed() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":1.2,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":6,"psiWPerMk":0.12,"sourceReference":"r75"},{"lengthM":4,"psiWPerMk":-0.05,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","heatedBasement":{"wallDepths":[{"lengthM":6,"depthM":2.6},{"lengthM":4,"depthM":1.1}],"wallResistanceM2kPerW":0.6}}]"#,
        false,
        &Expected {
            steady: 25.3734203914264,
            monthly: [
                25.95638784512918,
                28.086554378987586,
                29.489738851921544,
                29.7899591175765,
                28.906771398218766,
                27.076825130015322,
                24.79045293772362,
                22.660286403865218,
                21.25710193093126,
                20.956881665276292,
                21.84006938463403,
                23.670015652837485,
            ],
            heating_adjusted: 37.633888050251,
            cooling_adjusted: 16.633437065037402,
            july_conductance: 172.87900905078104,
            july_cooling_need: 50.556604622760915,
            tojuli: 0.4,
        },
    );
}

/// Shallow heated basement (z = 0,4 m), weakly insulated floor, detailed edge; 8.39, 8.45 with d1 = d_f;equi (p. 255-260).
#[test]
fn heated_shallow_detailed() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":30,"constructionResistanceM2kPerW":0.3,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":6,"psiWPerMk":0.12,"sourceReference":"r75"},{"lengthM":4,"psiWPerMk":-0.05,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","heatedBasement":{"depthM":0.4,"wallResistanceM2kPerW":4.5}}]"#,
        false,
        &Expected {
            steady: 39.00774017614307,
            monthly: [
                50.70947561072662,
                54.670321372889546,
                54.434391151997374,
                50.06490226020825,
                42.73265571741383,
                34.40232106346178,
                27.30600474155953,
                23.345158979396608,
                23.581089200288776,
                27.950578092077894,
                35.28282463487231,
                43.613159288824384,
            ],
            heating_adjusted: 66.90336255584577,
            cooling_adjusted: 21.688751893168135,
            july_conductance: 175.39456085461694,
            july_cooling_need: 47.497802966011704,
            tojuli: 0.37,
        },
    );
}

/// Slab with horizontal edge insulation R_n = 2,5: D.7 as printed (second exponent 2·d_f;equi/δ) with d' of D.9, β = 2 of table D.1 (p. 792-793).
#[test]
fn edge_horizontal() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":1.2,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","edgeInsulation":[{"kind":"horizontal","resistanceM2kPerW":2.5,"thicknessM":0.08,"sourceReference":"r75"}]}]"#,
        false,
        &Expected {
            steady: 19.222700700581587,
            monthly: [
                15.619676191789685,
                16.91149507995488,
                18.822599647709268,
                20.840910969471643,
                22.42562415650117,
                23.152116590098352,
                22.82572520937349,
                21.533906321208296,
                19.622801753453906,
                17.604490431691534,
                16.019777244662002,
                15.293284811064824,
            ],
            heating_adjusted: 25.157390804743123,
            cooling_adjusted: 14.040643623313162,
            july_conductance: 191.16883442216135,
            july_cooling_need: 44.124563506587606,
            tojuli: 0.32,
        },
    );
}

/// Slab with horizontal edge insulation R_n = 1,5 < 2,0: D.7 still applies, β = 1 (table D.1, p. 792-793).
#[test]
fn edge_horizontal_low() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":0.6,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","edgeInsulation":[{"kind":"horizontal","resistanceM2kPerW":1.5,"thicknessM":0.05,"sourceReference":"r75"}]}]"#,
        false,
        &Expected {
            steady: 22.627547200557917,
            monthly: [
                19.51307168643242,
                20.746978173365804,
                22.4847816124463,
                24.26083897556826,
                25.5992571265718,
                26.141408002760368,
                25.74202271468341,
                24.508116227750026,
                22.770312788669532,
                20.994255425547568,
                19.65583727454403,
                19.113686398355462,
            ],
            heating_adjusted: 30.736574409885016,
            cooling_adjusted: 16.045604021935265,
            july_conductance: 194.0851319274713,
            july_cooling_need: 40.217804128734656,
            tojuli: 0.28,
        },
    );
}

/// Slab with vertical edge insulation: D.8, β = 2; detailed edge (p. 792-793).
#[test]
fn edge_vertical() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":1.2,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":6,"psiWPerMk":0.12,"sourceReference":"r75"},{"lengthM":4,"psiWPerMk":-0.05,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","edgeInsulation":[{"kind":"vertical","resistanceM2kPerW":1,"thicknessM":0.04,"sourceReference":"r75"}]}]"#,
        false,
        &Expected {
            steady: 14.742700700581587,
            monthly: [
                11.949630071740597,
                13.834376351817399,
                15.962507407611092,
                17.763792241334162,
                18.755578036452082,
                18.672116590098348,
                17.535771329422577,
                15.651025049345773,
                13.522893993552081,
                11.72160915982901,
                10.729823364711086,
                10.813284811064824,
            ],
            heating_adjusted: 18.819806033178647,
            cooling_adjusted: 10.971980136703744,
            july_conductance: 165.62432744248,
            july_cooling_need: 66.1599073554737,
            tojuli: 0.54,
        },
    );
}

/// Slab with horizontal and vertical edge insulation: the smaller of D.7 and D.8 (p. 793).
#[test]
fn edge_both() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":1.2,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","edgeInsulation":[{"kind":"horizontal","resistanceM2kPerW":2.5,"thicknessM":0.08,"sourceReference":"r75"},{"kind":"vertical","resistanceM2kPerW":1,"thicknessM":0.04,"sourceReference":"r75"}]}]"#,
        false,
        &Expected {
            steady: 19.222700700581587,
            monthly: [
                15.619676191789685,
                16.91149507995488,
                18.822599647709268,
                20.840910969471643,
                22.42562415650117,
                23.152116590098352,
                22.82572520937349,
                21.533906321208296,
                19.622801753453906,
                17.604490431691534,
                16.019777244662002,
                15.293284811064824,
            ],
            heating_adjusted: 25.157390804743123,
            cooling_adjusted: 14.040643623313162,
            july_conductance: 191.16883442216135,
            july_cooling_need: 44.124563506587606,
            tojuli: 0.32,
        },
    );
}

/// Crawlspace with a well-insulated crawlspace floor: U_bf by 8.41 (d_bf;equi + 0,5·z ≥ B'_f, p. 257, 260).
#[test]
fn crawl_841() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":40,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","below":{"kind":"crawlspace","depthClass":"other","floorResistanceM2kPerW":4,"wallResistanceM2kPerW":2.5,"wallUValueWPerM2k":0.35}}]"#,
        false,
        &Expected {
            steady: 28.655907970901794,
            monthly: [
                30.177028793269866,
                29.973237245298016,
                29.416468382085828,
                28.655907970901794,
                27.895347559717756,
                27.338578696505568,
                27.13478714853372,
                27.338578696505568,
                27.895347559717756,
                28.655907970901794,
                29.416468382085828,
                29.973237245298016,
            ],
            heating_adjusted: 44.561726115148325,
            cooling_adjusted: 17.901515287445505,
            july_conductance: 195.4778963613216,
            july_cooling_need: 38.82399727101949,
            tojuli: 0.27,
        },
    );
}

/// Deep heated basement on a narrow floor: U_fl by 8.41 with z = 3,0 m (p. 257); detailed edge.
#[test]
fn heated_841() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":40,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":6,"psiWPerMk":0.12,"sourceReference":"r75"},{"lengthM":4,"psiWPerMk":-0.05,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","heatedBasement":{"depthM":3,"wallResistanceM2kPerW":3.5}}]"#,
        false,
        &Expected {
            steady: 33.28910017961719,
            monthly: [
                37.25554445993816,
                40.14843630035232,
                41.20337456660189,
                40.137689402180584,
                37.23693028628092,
                33.278353281445455,
                29.322655899296233,
                26.429764058882068,
                25.3748257926325,
                26.440510957053803,
                29.341270072953463,
                33.29984707778895,
            ],
            heating_adjusted: 52.10774870204004,
            cooling_adjusted: 20.64950383699984,
            july_conductance: 177.41121201235364,
            july_cooling_need: 43.32316936823349,
            tojuli: 0.33,
        },
    );
}

/// Unheated basement with n = 0 (no 8.49 or 0,33·n·V term): 8.43-8.47, D.15/D.16 with h·P·U_xw and (A_f + z·P)·λ/δ (p. 258-263, 795-796).
#[test]
fn basement_n0() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","below":{"kind":"unheated_basement","depthClass":"other","wallResistanceM2kPerW":2.5,"wallUValueWPerM2k":0.35,"volumeM3":120,"airChangesPerHour":0}}]"#,
        false,
        &Expected {
            steady: 13.135905696191454,
            monthly: [
                13.504965861349929,
                14.150555914945455,
                14.52427126182579,
                14.525975176595223,
                14.155211096667314,
                13.511324957841216,
                12.76684553103298,
                12.12125547743745,
                11.747540130557121,
                11.745836215787685,
                12.116600295715592,
                12.760486434541692,
            ],
            heating_adjusted: 19.77106370127927,
            cooling_adjusted: 8.487644063025515,
            july_conductance: 181.10995474382088,
            july_cooling_need: 62.55313002331877,
            tojuli: 0.47,
        },
    );
}

/// Partly sunken floor split in two parts (8.3.3.1 note 3, p. 254): slab 30 m² + heated basement 20 m²; D.1 on the summed heat flows of D.4.
#[test]
fn split_slab_basement() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":7,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic"},{"surfaceId":"floor2","exposedPerimeterM":9,"constructionResistanceM2kPerW":2.2,"edgeThermalBridges":{"method":"forfait"},"heatedBasement":{"depthM":1.6,"wallResistanceM2kPerW":2},"sourceReference":"r75"}]"#,
        true,
        &Expected {
            steady: 25.177372388954538,
            monthly: [
                26.21157529798464,
                27.413038309756022,
                28.015456443502856,
                27.85741224678183,
                26.981253534472753,
                25.621746325980343,
                24.14316947992443,
                22.94170646815305,
                22.339288334406213,
                22.497332531127242,
                23.37349124343632,
                24.732998451928733,
            ],
            heating_adjusted: 38.1969536806145,
            cooling_adjusted: 16.13848475047254,
            july_conductance: 192.48627869271232,
            july_cooling_need: 42.733624890684155,
            tojuli: 0.3,
        },
    );
}

/// Two ground floors, slab and crawlspace with different β (0 and 1) summed in D.1 (p. 790-792).
#[test]
fn split_slab_crawl() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":7,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic"},{"surfaceId":"floor2","exposedPerimeterM":9,"constructionResistanceM2kPerW":2.2,"edgeThermalBridges":{"method":"forfait"},"below":{"kind":"crawlspace","depthClass":"other","wallResistanceM2kPerW":1,"wallUValueWPerM2k":0.8},"sourceReference":"r75"}]"#,
        true,
        &Expected {
            steady: 19.022499458808106,
            monthly: [
                21.429086380362094,
                21.444292003475187,
                20.810580270009023,
                19.697753727179983,
                18.403993348455035,
                17.2759611825129,
                16.61591253725412,
                16.600706914141025,
                17.23441864760719,
                18.34724519043623,
                19.641005569161177,
                20.76903773510331,
            ],
            heating_adjusted: 30.71967338694789,
            cooling_adjusted: 11.394872310314982,
            july_conductance: 184.95902175004198,
            july_cooling_need: 54.886192406465284,
            tojuli: 0.4,
        },
    );
}

/// Heated room below ground level above a crawlspace (one reading, see NEN question): 8.38 walls at the heated depth with d1 against d_bf;equi of the crawlspace, U_fl of 8.43 with U_x;T = 0 (p. 254-262); periodic terms D.13/D.14 (β = 0) plus the wall terms of D.10/D.11 (β = 1).
#[test]
fn combo_crawl_forfait() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","heatedBasement":{"depthM":1.2,"wallResistanceM2kPerW":2},"below":{"kind":"crawlspace","depthClass":"other","wallResistanceM2kPerW":2,"wallUValueWPerM2k":0.45}}]"#,
        false,
        &Expected {
            steady: 19.036234638609233,
            monthly: [
                20.940032678984753,
                21.227701180904486,
                20.92816799257638,
                20.121692789919273,
                19.02436995220086,
                17.93022624762402,
                17.132436598233713,
                16.84476809631398,
                17.144301284642086,
                17.950776487299194,
                19.048099325017606,
                20.142243029594447,
            ],
            heating_adjusted: 30.16664801094182,
            cooling_adjusted: 11.649954292833241,
            july_conductance: 185.4755458110216,
            july_cooling_need: 53.895830396448446,
            tojuli: 0.4,
        },
    );
}

/// As above, detailed edge 8.39, crawlspace on sand with an insulated floor.
#[test]
fn combo_crawl_detailed() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":10,"psiWPerMk":0.1,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","heatedBasement":{"depthM":1.2,"wallResistanceM2kPerW":2},"below":{"kind":"crawlspace","depthClass":"on_sand","floorResistanceM2kPerW":0.3,"wallResistanceM2kPerW":2,"wallUValueWPerM2k":0.45,"ventilationOpeningM2PerM":0.002}}]"#,
        false,
        &Expected {
            steady: 13.104320629924256,
            monthly: [
                14.638944661060352,
                14.976073101801216,
                14.81166697927169,
                14.189778781234294,
                13.277042948135593,
                12.318026309357336,
                11.569696598788159,
                11.232568158047297,
                11.396974280576822,
                12.018862478614217,
                12.931598311712916,
                13.890614950491177,
            ],
            heating_adjusted: 20.89131289716068,
            cooling_adjusted: 7.966070224273,
            july_conductance: 159.6582527118456,
            july_cooling_need: 84.27548698823705,
            tojuli: 0.71,
        },
    );
}

/// Heated room below ground level above an unheated basement, n = 0 (same reading).
#[test]
fn combo_basement_n0() {
    check(
        r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":3.87,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":10,"psiWPerMk":0.1,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","heatedBasement":{"depthM":0.8,"wallResistanceM2kPerW":1},"below":{"kind":"unheated_basement","depthClass":"other","wallResistanceM2kPerW":1,"wallUValueWPerM2k":0.8,"volumeM3":100,"airChangesPerHour":0}}]"#,
        false,
        &Expected {
            steady: 14.04963439838974,
            monthly: [
                15.627244331134579,
                16.698168274431012,
                17.05941970451571,
                16.614201592432902,
                15.481809771770873,
                13.965667716391632,
                12.472024465644902,
                11.40110052234847,
                11.039849092263768,
                11.485067204346578,
                12.617459025008605,
                14.133601080387852,
            ],
            heating_adjusted: 21.983500854955928,
            cooling_adjusted: 8.718763709304932,
            july_conductance: 160.56058057870231,
            july_cooling_need: 81.3652687748081,
            tojuli: 0.69,
        },
    );
}

/// Two orientations (share 0,5 each, p. 116-118): the orientation balance takes share·H_gr;an;juli in
/// the heat transfer and share·H_C;g;adj in the time constant of 7.58. The solar gain per orientation
/// was fitted on the crawlspace case; these are the predictions for other ground floors with that same
/// solar gain. In the uninsulated crawlspace case the north orientation falls under 7.6 (1/γ_C > 2).
#[test]
fn two_orientations_split_ground_by_share() {
    for (floors, north, south) in [
        (
            r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":1.2,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":6,"psiWPerMk":0.12,"sourceReference":"r75"},{"lengthM":4,"psiWPerMk":-0.05,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","heatedBasement":{"wallDepths":[{"lengthM":6,"depthM":2.6},{"lengthM":4,"depthM":1.1}],"wallResistanceM2kPerW":0.6}}]"#,
            2.515425385172364,
            50.65562991090633,
        ),
        (
            r#"[{"surfaceId":"floor","exposedPerimeterM":10,"constructionResistanceM2kPerW":1.2,"edgeThermalBridges":{"method":"detailed","bridges":[{"lengthM":6,"psiWPerMk":0.12,"sourceReference":"r75"},{"lengthM":4,"psiWPerMk":-0.05,"sourceReference":"r75"}]},"sourceReference":"r75 synthetic","edgeInsulation":[{"kind":"vertical","resistanceM2kPerW":1,"thicknessM":0.04,"sourceReference":"r75"}]}]"#,
            3.4685310680325756,
            63.60561054405321,
        ),
        (
            r#"[{"surfaceId":"floor","exposedPerimeterM":18,"constructionResistanceM2kPerW":0.32,"edgeThermalBridges":{"method":"forfait"},"sourceReference":"r75 synthetic","below":{"kind":"crawlspace","depthClass":"other","wallResistanceM2kPerW":0.35,"wallUValueWPerM2k":1.9,"ventilationOpeningM2PerM":0.0005}}]"#,
            0.0,
            32.77554964976372,
        ),
    ] {
        let p = performance(floors, false, false);
        let orientations = p["tojuli"][0]["orientations"]
            .as_array()
            .expect("orientations");
        for (name, expected) in [("north", north), ("south", south)] {
            let item = orientations
                .iter()
                .find(|item| item["orientation"] == name)
                .expect("orientation");
            close(
                &format!("Q_C;nd;juli {name}"),
                num(&item["coolingNeedJulyKwh"], "Q"),
                expected,
            );
        }
    }
}
