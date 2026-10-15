//! Review 76: the defaults that the basisopname (ISSO 82.1 / 75.1 survey) derivation applies for
//! unknowns, checked against NTA 8800:2025+C1:2026. Expected values are transcribed from the norm
//! pages (numbers and page references only); the survey fixtures in `training-data` are run through
//! the derivation and the derived kernel input is compared with an independent reading of the norm.
//! Defaults whose only source is ISSO 82.1/75.1 (no NTA counterpart) are not tested here.

use nta8800_core::forfait_envelope::{
    basic_survey_unheated_transfer, forfait_door_u, forfait_window_u, BuildingKind, ElementType,
    ForfaitGlass, ForfaitOpaque, InsulationState,
};
use nta8800_core::lighting::PARASITIC_FORFAIT_KWH_PER_M2;
use nta8800_core::monthly_demand::{
    specific_heat_capacity, CeilingColumn, MassClass, VERTICAL_PIPE_INSULATED_W_PER_K,
    VERTICAL_PIPE_UNINSULATED_W_PER_K,
};
use nta8800_core::opname::envelope::{delta_u_forfait, FRAME_FRACTION_FORFAIT};
use nta8800_core::opname::general::{
    next_year_class_start, thermal_mass, Construction, FloorConstruction, WallConstruction,
};
use nta8800_core::opname::production::{derive_pv, SurveyPv};
use nta8800_core::opname::utility::{derive_utility_input, UtilitySurvey};
use nta8800_core::opname::{derive_residential_input, Recorder, ResidentialSurvey};
use nta8800_core::pv::{PvModuleType, PvMounting};
use nta8800_core::window_u::FrameGroup;
use serde_json::{json, Value};

const TERRACED_1930: &str =
    include_str!("../../../training-data/nta8800-opname-1930-terraced.json");
const DETACHED_2015: &str =
    include_str!("../../../training-data/nta8800-opname-2015-detached.json");
const RETAIL_1970: &str =
    include_str!("../../../training-data/nta8800-opname-utility-1970-retail.json");

fn close(label: &str, kernel: f64, expected: f64) {
    assert!(
        (kernel - expected).abs() <= 1e-9,
        "{label}: kernel {kernel} independent {expected}"
    );
}

fn residential(raw: &str, edit: impl FnOnce(&mut Value)) -> Value {
    let mut value: Value = serde_json::from_str(raw).expect("fixture");
    edit(&mut value);
    let survey: ResidentialSurvey = serde_json::from_value(value).expect("survey");
    let mut recorder = Recorder::default();
    let derived = derive_residential_input(&survey, &mut recorder);
    assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
    derived.expect("derived input")
}

fn utility(raw: &str) -> Value {
    let survey: UtilitySurvey = serde_json::from_str(raw).expect("survey");
    let mut recorder = Recorder::default();
    let derived = derive_utility_input(&survey, &mut recorder);
    assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
    derived.expect("derived input")
}

fn element<'a>(items: &'a Value, id: &str) -> &'a Value {
    items
        .as_array()
        .expect("array")
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("element {id}"))
}

fn collect_pipes(value: &Value, found: &mut Vec<Value>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                if key == "verticalPipes" {
                    found.extend(item.as_array().cloned().unwrap_or_default());
                } else {
                    collect_pipes(item, found);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect_pipes(item, found)),
        _ => {}
    }
}

fn opaque(
    element: ElementType,
    building: BuildingKind,
    year: i32,
    insulation: InsulationState,
    cavity: bool,
) -> ForfaitOpaque {
    ForfaitOpaque {
        element,
        building,
        construction_year: year,
        insulation,
        cavity,
        r_si_override: None,
        towards_unheated_space: false,
        renovation: None,
        survey_table_8_10: false,
    }
}

fn rc(element: ElementType, building: BuildingKind, year: i32) -> f64 {
    let item = opaque(
        element,
        building,
        year,
        InsulationState::AbsentOrUnknown,
        false,
    );
    assert!(item.validate("x").is_empty(), "{element:?} {year}");
    item.calculate().r_c
}

/// Table I.4 (p. 828): before 1965, insulation unknown/absent or (post-)insulated, with or without cavity.
#[test]
fn table_i4_before_1965() {
    let rows = [
        (ElementType::Facade, true, 0.35, 0.85),
        (ElementType::Facade, false, 0.19, 0.69),
        (ElementType::Floor, true, 0.33, 0.83),
        (ElementType::Floor, false, 0.15, 0.65),
        (ElementType::Roof, true, 0.35, 0.85),
        (ElementType::Roof, false, 0.22, 0.72),
    ];
    for (element, cavity, absent, insulated) in rows {
        for (state, expected) in [
            (InsulationState::AbsentOrUnknown, absent),
            (InsulationState::PresentUnknownThickness, insulated),
        ] {
            let result = opaque(element, BuildingKind::Regular, 1964, state, cavity).calculate();
            close(
                &format!("I.4 {element:?} cavity {cavity}"),
                result.r_c,
                expected,
            );
            assert_eq!(result.route, "I.2.1.2");
        }
    }
    // 1965 belongs to table I.5 ("van voor 1965", figure I.4 "Bouwjaar < 1965").
    let result = opaque(
        ElementType::Facade,
        BuildingKind::Regular,
        1965,
        InsulationState::AbsentOrUnknown,
        true,
    )
    .calculate();
    assert_eq!((result.r_c, result.route), (0.43, "I.2.1.3"));
}

/// Table I.5 (p. 829): regular buildings from 1965, "van X tot Y" = X ≤ year < Y.
#[test]
fn table_i5_year_classes() {
    let facade = [
        (1965, 0.43),
        (1975, 1.3),
        (1983, 1.3),
        (1988, 2.0),
        (1992, 2.5),
        (2014, 3.5),
        (2015, 4.5),
        (2021, 4.7),
    ];
    let floor = [
        (1965, 0.17),
        (1975, 0.52),
        (1983, 1.3),
        (1988, 1.3),
        (1992, 2.5),
        (2014, 3.5),
        (2021, 3.7),
    ];
    let roof = [
        (1965, 0.86),
        (1975, 1.3),
        (1983, 1.3),
        (1988, 2.0),
        (1992, 2.5),
        (2014, 3.5),
        (2015, 6.0),
        (2021, 6.3),
    ];
    for (element, rows) in [
        (ElementType::Facade, &facade[..]),
        (ElementType::Floor, &floor[..]),
        (ElementType::Roof, &roof[..]),
    ] {
        for (index, (from, expected)) in rows.iter().enumerate() {
            close(
                &format!("I.5 {element:?} {from}"),
                rc(element, BuildingKind::Regular, *from),
                *expected,
            );
            if let Some((next, _)) = rows.get(index + 1) {
                close(
                    &format!("I.5 {element:?} {}", next - 1),
                    rc(element, BuildingKind::Regular, next - 1),
                    *expected,
                );
            }
        }
        // Attic floors take the floor row (table I.5 "zoldervloeren").
        if element == ElementType::Floor {
            close(
                "I.5 attic floor 1980",
                rc(ElementType::AtticFloor, BuildingKind::Regular, 1980),
                0.52,
            );
        }
    }
}

/// Table I.6 (p. 830): caravans from 1965.
#[test]
fn table_i6_caravans() {
    let cases = [
        (ElementType::Facade, 1982, 0.19),
        (ElementType::Facade, 1983, 1.3),
        (ElementType::Facade, 1991, 1.3),
        (ElementType::Facade, 1992, 2.0),
        (ElementType::Facade, 2014, 2.5),
        (ElementType::Facade, 2021, 2.6),
        (ElementType::Floor, 1965, 0.17),
        (ElementType::Floor, 1983, 1.3),
        (ElementType::Floor, 2020, 2.5),
        (ElementType::Roof, 1982, 0.22),
        (ElementType::Roof, 1992, 2.0),
        (ElementType::Roof, 2021, 2.6),
    ];
    for (element, year, expected) in cases {
        close(
            &format!("I.6 {element:?} {year}"),
            rc(element, BuildingKind::Caravan, year),
            expected,
        );
    }
}

/// Table I.7 (p. 831–832): floating buildings from 1965 (NTA route, not ISSO table 8.10).
#[test]
fn table_i7_floating_buildings() {
    let old = BuildingKind::Floating {
        new_berth_since_2018: false,
    };
    let new = BuildingKind::Floating {
        new_berth_since_2018: true,
    };
    let cases = [
        (ElementType::Facade, old, 1980, 0.43),
        (ElementType::Facade, old, 1983, 1.3),
        (ElementType::Facade, old, 2015, 4.5),
        (ElementType::Facade, old, 2018, 3.5),
        (ElementType::Facade, old, 2021, 3.7),
        (ElementType::Facade, new, 2018, 4.5),
        (ElementType::Facade, new, 2021, 4.7),
        (ElementType::FloatingHull, old, 1991, 0.17),
        (ElementType::FloatingHull, old, 1992, 2.5),
        (ElementType::FloatingHull, old, 2014, 2.5),
        (ElementType::FloatingHull, old, 2015, 3.5),
        (ElementType::FloatingHull, old, 2018, 2.5),
        (ElementType::FloatingHull, old, 2021, 2.6),
        (ElementType::FloatingHull, new, 2018, 3.5),
        (ElementType::FloatingHull, new, 2021, 3.7),
        (ElementType::Roof, old, 1982, 0.35),
        (ElementType::Roof, old, 1988, 2.0),
        (ElementType::Roof, old, 2015, 6.0),
        (ElementType::Roof, old, 2018, 4.5),
        (ElementType::Roof, old, 2021, 4.7),
        (ElementType::Roof, new, 2018, 6.0),
        (ElementType::Roof, new, 2021, 6.3),
    ];
    for (element, building, year, expected) in cases {
        close(
            &format!("I.7 {element:?} {building:?} {year}"),
            rc(element, building, year),
            expected,
        );
    }
}

/// I.1 (U rounded to two decimals, p. 827) and I.2 with R_ad, λ 0,045, the air layer for at most
/// 30 mm, reed (I.3) and thermal cushions (p. 832–833); R_si/R_se from table C.2.
#[test]
fn formulas_i1_to_i3_known_thickness() {
    let known = |mm: f64| InsulationState::KnownThickness {
        thickness_mm: mm,
        thickness_proven: false,
        known_lambda_equivalent: None,
        reed_thickness_m: None,
        thermal_cushions: false,
    };
    let facade = opaque(
        ElementType::Facade,
        BuildingKind::Regular,
        1980,
        known(43.0),
        true,
    )
    .calculate();
    // 43 mm rounds to 40 mm; above 30 mm no air layer.
    close("I.2 facade", facade.r_c, 0.04 / 0.045 + 0.36);
    close(
        "I.1 facade",
        facade.u_c,
        (100.0_f64 / (0.04 / 0.045 + 0.36 + 0.13 + 0.04)).round() / 100.0,
    );
    let thin = opaque(
        ElementType::Roof,
        BuildingKind::Regular,
        1980,
        known(20.0),
        true,
    )
    .calculate();
    close(
        "I.2 roof with air layer",
        thin.r_c,
        0.02 / 0.045 + 0.22 + 0.13,
    );
    let floor = opaque(
        ElementType::Floor,
        BuildingKind::Regular,
        1980,
        known(30.0),
        true,
    )
    .calculate();
    close(
        "I.2 floor with air layer",
        floor.r_c,
        0.03 / 0.045 + 0.15 + 0.18,
    );
    let reed = opaque(
        ElementType::Roof,
        BuildingKind::Regular,
        1980,
        InsulationState::KnownThickness {
            thickness_mm: 0.0,
            thickness_proven: true,
            known_lambda_equivalent: None,
            reed_thickness_m: Some(0.27),
            thermal_cushions: false,
        },
        false,
    )
    .calculate();
    // Reed thickness floored to a multiple of 0,05 m: 0,25 / 0,105.
    close("I.3 reed", reed.r_c, 0.22 + 0.25 / 0.105);
    let cushions = opaque(
        ElementType::Floor,
        BuildingKind::Regular,
        1980,
        InsulationState::KnownThickness {
            thickness_mm: 0.0,
            thickness_proven: true,
            known_lambda_equivalent: None,
            reed_thickness_m: None,
            thermal_cushions: true,
        },
        false,
    )
    .calculate();
    close("thermal cushions", cushions.r_c, 0.15 + 1.8);
}

/// Tables I.8, I.9 (p. 834–835) and I.10 (p. 835).
#[test]
fn tables_i8_to_i10_windows_and_doors() {
    let exterior = [
        (ForfaitGlass::TripleHr, [1.4, 1.9, 2.7]),
        (ForfaitGlass::HrPlusPlus, [1.8, 2.3, 3.1]),
        (ForfaitGlass::HrPlus, [2.0, 2.5, 3.3]),
        (ForfaitGlass::HrCoatedDouble, [2.3, 2.8, 3.6]),
        (ForfaitGlass::DoubleUncoated, [2.9, 3.3, 4.1]),
        (ForfaitGlass::SecondaryWindow, [2.9, 3.3, 4.1]),
        (ForfaitGlass::Single, [5.1, 5.4, 6.2]),
    ];
    let interior = [
        (ForfaitGlass::TripleHr, [1.3, 1.6, 2.2]),
        (ForfaitGlass::HrPlusPlus, [1.5, 1.9, 2.4]),
        (ForfaitGlass::HrPlus, [1.7, 2.1, 2.6]),
        (ForfaitGlass::HrCoatedDouble, [1.9, 2.2, 2.7]),
        (ForfaitGlass::DoubleUncoated, [2.3, 2.5, 3.0]),
        (ForfaitGlass::SecondaryWindow, [2.3, 2.5, 3.0]),
        (ForfaitGlass::Single, [3.5, 3.6, 4.0]),
    ];
    let frames = [
        FrameGroup::WoodOrPlastic,
        FrameGroup::MetalWithThermalBreak,
        FrameGroup::MetalWithoutThermalBreak,
    ];
    for (outdoor, table) in [(true, &exterior), (false, &interior)] {
        for (glass, row) in table.iter() {
            for (frame, expected) in frames.iter().zip(row) {
                close(
                    &format!("I.8/I.9 {glass:?} {frame:?} {outdoor}"),
                    forfait_window_u(*glass, *frame, outdoor),
                    *expected,
                );
            }
        }
    }
    close("I.10 insulated outdoor", forfait_door_u(true, true), 2.0);
    close("I.10 insulated adjacent", forfait_door_u(true, false), 1.7);
    close("I.10 uninsulated outdoor", forfait_door_u(false, true), 3.4);
    close(
        "I.10 uninsulated adjacent",
        forfait_door_u(false, false),
        2.7,
    );
}

/// 1930 terraced survey end to end: table I.4 values, U by I.1 with R_si 0,13/0,10 and R_se 0,04
/// (C.2), a door with less than 65 % glass split into a single-glass window part (I.8) and an
/// uninsulated door part (I.10, I.2.2.1), ΔU_for (8.3, table 8.1, p. 225) and the crawlspace
/// rules of I.2.3 (p. 840): R_bf 0 for an unknown/absent insulation, z 0,5 m.
#[test]
fn survey_1930_envelope_follows_annex_i() {
    let input = residential(TERRACED_1930, |_| {});
    let demand = &input["spaceHeating"]["demand"];
    let direct = &demand["transmission"]["direct"]["elements"];
    let u = |r_c: f64, r_si: f64| (100.0 / (r_c + r_si + 0.04_f64)).round() / 100.0;
    let front = u(0.35, 0.13);
    let roof = u(0.72, 0.10);
    // 40 mm (ISSO table 8.26 width, an ISSO value) through I.2: no air layer above 30 mm.
    let back = u(0.04 / 0.045 + 0.36, 0.13);
    close(
        "front facade",
        element(direct, "voorgevel")["uValueWPerM2k"]
            .as_f64()
            .unwrap(),
        front,
    );
    close(
        "back facade",
        element(direct, "achtergevel")["uValueWPerM2k"]
            .as_f64()
            .unwrap(),
        back,
    );
    close(
        "roof",
        element(direct, "dak-zuid")["uValueWPerM2k"]
            .as_f64()
            .unwrap(),
        roof,
    );
    close(
        "door part",
        element(direct, "voordeur")["uValueWPerM2k"]
            .as_f64()
            .unwrap(),
        3.4,
    );
    close(
        "door part area",
        element(direct, "voordeur")["areaM2"].as_f64().unwrap(),
        0.8 * 2.0,
    );
    close(
        "door window part",
        element(direct, "voordeur-glass")["uValueWPerM2k"]
            .as_f64()
            .unwrap(),
        5.1,
    );
    // 8.3 over the opaque planes (doors and the crawlspace floor excluded): Ū > 0,8 gives 0.
    let mean = (19.0 * front + 22.0 * back + 70.0 * roof) / 111.0;
    assert!(mean > 0.8, "{mean}");
    close("ΔU_for", delta_u_forfait(mean), 0.0);
    let floor = element(&demand["transmission"]["groundFloors"], "vloer");
    close(
        "R_si + R_c",
        floor["constructionResistanceM2kPerW"].as_f64().unwrap(),
        0.15 + 0.17,
    );
    assert_eq!(floor["below"]["floorResistanceM2kPerW"], 0.0);
    assert_eq!(floor["below"]["depthClass"], "other");
}

/// 8.3 and the example values of table 8.1 (p. 225).
#[test]
fn delta_u_for_formula_8_3() {
    close("Ū 0,8", delta_u_forfait(0.8), 0.0);
    close("Ū 0,6", delta_u_forfait(0.6), 0.05);
    close("Ū 0,4", delta_u_forfait(0.4), 0.1);
    close("Ū 0,2", delta_u_forfait(0.2), 0.15);
    close("Ū 1,5", delta_u_forfait(1.5), 0.0);
}

/// 7.6.6.2 method B (p. 203): F_fr 0,25 when the frame fraction is unknown.
#[test]
fn frame_fraction_7_6_6_2() {
    close("constant", FRAME_FRACTION_FORFAIT, 0.25);
    let input = residential(TERRACED_1930, |_| {});
    let windows = input["spaceHeating"]["demand"]["windows"]
        .as_array()
        .unwrap();
    assert!(!windows.is_empty());
    for window in windows {
        assert_eq!(window["frameFraction"], 0.25, "{window}");
    }
}

/// I.8 (p. 841): H_ue = 5·A_T;iu, so b = 1 / (1 + U/5) (note 2).
#[test]
fn formula_i8_unheated_space() {
    close("H_ue", basic_survey_unheated_transfer(12.0), 60.0);
    let (area, u) = (12.0, 1.6);
    let h_ue = basic_survey_unheated_transfer(area);
    close("b", h_ue / (h_ue + area * u), 1.0 / (1.0 + u / 5.0));
}

/// Table 7.1 (p. 172): 1,8 W/K per storey uninsulated, 0,5 insulated; an unknown pipe is an
/// uninsulated fictitious pipe (7.3.3, p. 172–173).
#[test]
fn vertical_pipes_table_7_1() {
    close("uninsulated", VERTICAL_PIPE_UNINSULATED_W_PER_K, 1.8);
    close("insulated", VERTICAL_PIPE_INSULATED_W_PER_K, 0.5);
    let mut pipes = Vec::new();
    collect_pipes(&residential(TERRACED_1930, |_| {}), &mut pipes);
    assert!(!pipes.is_empty());
    assert!(pipes.iter().all(|pipe| pipe["insulated"] == false));
    let mut pipes = Vec::new();
    collect_pipes(&utility(RETAIL_1970), &mut pipes);
    assert_eq!(pipes.len(), 1, "one toilet group");
    assert_eq!(pipes[0]["insulated"], false);
}

/// 7.17a (p. 173): the fictitious pipe per toilet group of a utility building runs through
/// N = H/3 storeys, rounded down, at least 1, with H the building height of 11.2.1.2. The retail
/// fixture has H = 6 m, so N = 2. The derivation hands H to the kernel (`buildingHeightM`, which
/// applies 7.17a) and spreads the pipe over all zones by usable area (`areaShare`, summing to 1).
#[test]
fn utility_fictitious_pipe_storeys_7_17a() {
    let survey: Value = serde_json::from_str(RETAIL_1970).unwrap();
    let height = survey["buildingHeightM"].as_f64().unwrap();
    let expected = ((height / 3.0).floor() as u64).max(1);
    assert_eq!(expected, 2);
    let mut pipes = Vec::new();
    collect_pipes(&utility(RETAIL_1970), &mut pipes);
    assert!(!pipes.is_empty());
    for pipe in &pipes {
        let h = pipe["buildingHeightM"].as_f64().unwrap();
        assert_eq!(((h / 3.0).floor() as u64).max(1), expected);
        assert!(pipe
            .get("storeys")
            .map_or(true, |value| value.as_u64() == Some(0)));
    }
    let share: f64 = pipes
        .iter()
        .map(|pipe| pipe["areaShare"].as_f64().unwrap())
        .sum();
    assert!((share - 1.0).abs() < 1e-9, "{share}");
}

/// Table 11.13 (p. 486): year classes < 1970, 1970–1980, 1980–1990, 1990–2000, 2000–2010, ≥ 2010.
/// The survey's "next year class" for an unknown renovation year starts on these boundaries.
#[test]
fn infiltration_year_classes_table_11_13() {
    for (year, next) in [
        (1930, 1970),
        (1969, 1970),
        (1970, 1980),
        (1979, 1980),
        (1980, 1990),
        (1999, 2000),
        (2000, 2010),
        (2009, 2010),
    ] {
        assert_eq!(next_year_class_start(year), next, "{year}");
    }
}

/// Table 11.9 (p. 472): an unknown duct airtightness class is passed as "unknown" (f_lea;du
/// 1,10); 11.3.2.2 (p. 499): bypass of a dwelling's unit before 2010 is 0 % when not
/// established and 70 % when established; from 2010 100 %.
#[test]
fn ventilation_unknowns_table_11_9_and_11_3_2_2() {
    let unit =
        |input: &Value| input["spaceHeating"]["demand"]["ventilation"]["system"]["unit"].clone();
    let base = unit(&residential(DETACHED_2015, |_| {}));
    assert_eq!(base["ducts"], "unknown");
    assert_eq!(base["heatRecovery"]["bypass"]["kind"], "full");
    let old_unknown = unit(&residential(DETACHED_2015, |survey| {
        survey["ventilation"]["unitManufactureYear"] = json!(2005);
        survey["ventilation"]["bypassPresent"] = Value::Null;
    }));
    assert_eq!(old_unknown["heatRecovery"]["bypass"]["kind"], "none");
    let old_present = unit(&residential(DETACHED_2015, |survey| {
        survey["ventilation"]["unitManufactureYear"] = json!(2009);
    }));
    assert_eq!(old_present["heatRecovery"]["bypass"]["kind"], "partial");
    close(
        "70 %",
        old_present["heatRecovery"]["bypass"]["fraction"]
            .as_f64()
            .unwrap(),
        0.7,
    );
}

/// Table 9.14 note (p. 307): an unknown design temperature class is 90/70 (a conventional
/// boiler on radiators, the 1930 survey); table 9.16 (p. 310): pipes of unknown insulation take
/// the highest value of their category (uninsulated).
#[test]
fn heating_unknowns_chapter_9() {
    let input = residential(TERRACED_1930, |_| {});
    let heating = &input["spaceHeating"];
    assert_eq!(
        heating["distributionSystem"]["designTemperatureClass"],
        "90_70"
    );
    close(
        "mean emission temperature 90/70",
        heating["generator"]["boiler"]["averageDesignEmissionTemperatureC"]
            .as_f64()
            .unwrap(),
        (90.0 + 70.0) / 2.0,
    );
    assert_eq!(
        heating["distributionSystem"]["pipeTransmittance"]["insulation"]["state"],
        "uninsulated"
    );
}

/// Table 13.25 (p. 641): type unknown 0,30, the same as a water heater or combi without Gaskeur;
/// c_W;gen with an unknown application class is class 4.
#[test]
fn gas_appliance_unknowns_table_13_25() {
    let input = residential(TERRACED_1930, |_| {});
    assert_eq!(
        input["hotWater"]["generator"]["appliance"],
        "without_gaskeur"
    );
    let input = residential(TERRACED_1930, |survey| {
        survey["hotWater"]["generator"] =
            json!({"kind": "gas_appliance", "applianceType": "combi", "gaskeur": "gaskeur"});
    });
    assert_eq!(input["hotWater"]["generator"]["appliance"], "combi_gaskeur");
    assert_eq!(input["hotWater"]["generator"]["measuredClass"], "class4");
}

/// Table 16.1 (p. 680): K_pk by placement year ("voor 2001", "2001 t/m 2010", ...); table 16.2
/// (p. 681): building integration not determinable → not ventilated (f_perf 0,76).
#[test]
fn pv_tables_16_1_and_16_2() {
    let pv = |module: &str, year: i32| -> SurveyPv {
        serde_json::from_value(json!({
            "id": "pv", "panelAreaM2": 10.0, "moduleType": module, "installationYear": year,
            "azimuthDeg": 180.0, "tiltDeg": 35.0, "mounting": "unknown", "sourceReference": "test"
        }))
        .unwrap()
    };
    let cases = [
        ("monocrystalline", 2000, 125.0),
        ("monocrystalline", 2001, 135.0),
        ("monocrystalline", 2010, 135.0),
        ("monocrystalline", 2011, 150.0),
        ("monocrystalline", 2014, 150.0),
        ("monocrystalline", 2015, 165.0),
        ("monocrystalline", 2017, 165.0),
        ("monocrystalline", 2018, 175.0),
        ("polycrystalline", 2000, 115.0),
        ("polycrystalline", 2001, 125.0),
        ("polycrystalline", 2011, 140.0),
        ("polycrystalline", 2015, 155.0),
        ("polycrystalline", 2018, 165.0),
    ];
    for (module, year, expected) in cases {
        let mut recorder = Recorder::default();
        let value = derive_pv(&pv(module, year), 1990, &mut recorder);
        let kind: PvModuleType =
            serde_json::from_value(value["peakPower"]["moduleType"].clone()).unwrap();
        close(
            &format!("16.1 {module} {year}"),
            kind.peak_power_w_per_m2(),
            expected,
        );
        let mounting: PvMounting = serde_json::from_value(value["mounting"].clone()).unwrap();
        close("16.2 unknown", mounting.performance_factor(), 0.76);
    }
}

/// Table 10.8 text (p. 380): an unknown design temperature of a water-based cooling emission
/// system is 6/12 (the retail survey).
#[test]
fn cooling_design_temperature_table_10_8() {
    let input = utility(RETAIL_1970);
    assert_eq!(
        input["cooling"]["distribution"]["designTemperature"],
        "t6_to12_or_unknown"
    );
}

/// 14.14 (p. 661–662): forfait parasitic energy 1,0 + 1,5 kWh/m²; with the forfait power of
/// table 14.3 the daylight effect is left out (14.24, F_D = 1, p. 667).
#[test]
fn lighting_forfaits_chapter_14() {
    close("W_pe + W_pc", PARASITIC_FORFAIT_KWH_PER_M2, 2.5);
    let input = utility(RETAIL_1970);
    let zone = &input["lighting"][0]["lightingZones"][0];
    assert_eq!(zone["power"]["method"], "forfait");
    assert_eq!(zone["parasitic"]["method"], "forfait");
    assert_eq!(zone["daylight"]["method"], "none");
}

/// Table 7.10 (p. 205) with footnotes b and c: dwellings use "geen of open plafond" unless the
/// top of a floor is heavier than the underside of the floor above.
#[test]
fn thermal_mass_table_7_10() {
    let construction = |floor, lighter_ceiling| Construction {
        floor,
        wall: WallConstruction::Heavy,
        lighter_ceiling,
        closed_or_suspended_ceiling: false,
        source_reference: "test".into(),
    };
    let (floor, wall, ceiling) = thermal_mass(&construction(FloorConstruction::Light, false));
    assert_eq!(ceiling, CeilingColumn::OpenOrNone);
    close(
        "light/heavy open",
        specific_heat_capacity(floor, wall, ceiling),
        180.0,
    );
    let (floor, wall, ceiling) = thermal_mass(&construction(FloorConstruction::VeryHeavy, true));
    assert_eq!(ceiling, CeilingColumn::ClosedOrSuspended);
    assert_eq!((floor, wall), (MassClass::VeryHeavy, MassClass::Heavy));
    close(
        "very heavy/heavy closed",
        specific_heat_capacity(floor, wall, ceiling),
        250.0,
    );
}
