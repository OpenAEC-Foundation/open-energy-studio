//! Review 66: installation routes the two example projects do not reach, recalculated independently
//! from NTA 8800:2025+C1:2026 (heat pumps forfait and declared, hybrid, booster, external heat with and
//! without a declaration, CHP method 2, biomass, electric boilers, collective boilers with priority, hot-water
//! storage, shower heat recovery, cooling generators of method 3, external cold, PV with a battery). Every case
//! is an ISSO 54 v2.0 deeltest (training-data/reference-suites/isso54-v2.json) run in 2025+C1, optionally
//! with a small edit. The expected values come from a separate implementation written from the norm pages;
//! only numbers and page references are kept here.

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

fn row<'a>(p: &'a Value, service: &str, carrier: &str) -> &'a Value {
    p["energyByService"]["annual"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|row| row["service"] == service && row["carrier"] == carrier)
        .expect("service row")
}

fn renewable(p: &Value, service: &str, month: u64) -> f64 {
    p["energyByService"]["renewable"]
        .as_array()
        .expect("renewable")
        .iter()
        .find(|row| row["service"] == service && row["month"] == month)
        .map_or(0.0, |row| {
            row["renewablePrimaryKwh"].as_f64().expect("number")
        })
}

fn close(label: &str, kernel: f64, expected: f64) {
    let diff = (kernel - expected).abs();
    let scale = kernel.abs().max(expected.abs()).max(1e-12);
    assert!(
        diff <= 1e-9 || diff / scale <= 1e-6,
        "{label}: kernel {kernel} independent {expected}"
    );
}

/// External heat with an annex P declaration on calculated values (9.6.7, p. 355-356): f_prac 0,95 in the
/// delivered energy of chapter 5, renewable 5.39g on Q_H;gen;out (p. 112), auxiliary 9.91 (p. 364-365); the
/// EMGforf run takes the forfait factors and f_prac 1 (5.3.1.3).
#[test]
fn external_heat_declaration_with_calculated_values() {
    let p = performance("EPW204i", r#"[]"#);
    close(
        "EPW204i W_H;gen;aux m1",
        p["spaceHeating"]["monthly"][0]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        23.31147424060753,
    );
    close(
        "EPW204i delivered dh heating",
        row(&p, "heating", "dh")["deliveredKwh"]
            .as_f64()
            .expect("number"),
        4829.62758237179,
    );
    close(
        "EPW204i E_P dh heating (fP 0.5)",
        row(&p, "heating", "dh")["primaryFossilKwh"]
            .as_f64()
            .expect("number"),
        2414.813791185895,
    );
    close(
        "EPW204i E_Pren;dh 5.39g m1",
        renewable(&p, "heating", 1),
        413.47485743568706,
    );
    close(
        "EPW204i E_P;tot",
        p["annualPrimaryFossilKwh"].as_f64().expect("number"),
        7818.489537343233,
    );
    close(
        "EPW204i EMGforf E_P;tot",
        p["externalSupply"]["forfaitPrimaryFossilKwh"]
            .as_f64()
            .expect("number"),
        9533.00732908522,
    );
}

/// Central biomass boiler outside the thermal boundary, table 9.30 0,750 (p. 341), auxiliary 9.91/9.92 with
/// 10 W/kW automatic feed and t_on capped at t_mi (p. 364-365), bmB renewable 5.39 (p. 112).
#[test]
fn biomass_boiler_outside_with_capped_operating_time() {
    let p = performance(
        "EPW204c",
        r#"[{"pointer": "/ntaCalculation/generator/appliance", "value": "central_boiler"}, {"pointer": "/ntaCalculation/generator/location", "value": "outside_thermal_boundary"}, {"pointer": "/ntaCalculation/generator/automaticFuelFeed", "value": true}, {"pointer": "/ntaCalculation/generator/auxiliary", "value": {"electricallyConnectedDevices": 1, "nominalPowerKw": 2.0, "sourceReference": "r66 synthetic: small P_gen so t_on hits t_mi in winter"}}]"#,
    );
    close(
        "bio-boiler-out E_H;bm m1",
        p["spaceHeating"]["monthly"][0]["biomassKwh"]
            .as_f64()
            .expect("number"),
        1810.6928026450066,
    );
    close(
        "bio-boiler-out W_H;aux m1",
        p["spaceHeating"]["monthly"][0]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        22.32,
    );
    close(
        "bio-boiler-out E_Pren;bmB m1",
        renewable(&p, "heating", 1),
        679.0098009918775,
    );
    close(
        "bio-boiler-out E_P;tot",
        p["annualPrimaryFossilKwh"].as_f64().expect("number"),
        8379.095063719344,
    );
}

/// Micro-CHP method 2, table 9.31 HRe 0,83/0,10 (p. 343), electricity 16.12 (p. 686), the monthly balance
/// 5.22-5.26 and the export credit at f_P 1,45 (5.10-5.13, p. 84-92).
#[test]
fn chp_method_2_with_export() {
    let p = performance("EPW204f", r#"[]"#);
    close(
        "EPW204f E_H;gas m1",
        p["spaceHeating"]["monthly"][0]["naturalGasKwh"]
            .as_f64()
            .expect("number"),
        1673.5299168462418,
    );
    close(
        "EPW204f E_el;chp;out;H m1",
        p["spaceHeating"]["monthly"][0]["chpElectricityKwh"]
            .as_f64()
            .expect("number"),
        167.35299168462421,
    );
    close(
        "EPW204f W_H;aux m1",
        p["spaceHeating"]["monthly"][0]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        14.383469285945733,
    );
    close(
        "EPW204f exported el m1",
        p["electricityBalance"][0]["exportedKwh"]
            .as_f64()
            .expect("number"),
        113.7368966053311,
    );
    close(
        "EPW204f E_P;tot",
        p["annualPrimaryFossilKwh"].as_f64().expect("number"),
        10105.301597193567,
    );
}

/// Outdoor-air heat pump with a declared COP 3,137: rounded down to a multiple of 0,05 (9.1, p. 285) and
/// f_prac 0,95 (NEN question 3, kernel reading); auxiliary 9.85 (p. 359-360); renewable 5.31 (p. 103).
#[test]
fn declared_heat_pump_cop() {
    let p = performance(
        "EPW203d",
        r#"[{"pointer": "/ntaCalculation/generator/forfait/source", "value": "outdoor_air"}, {"pointer": "/ntaCalculation/generator/forfait/designSupplyTemperatureC", "value": 35.0}, {"pointer": "/ntaCalculation/generator/forfait/sourceCorrectionFactor", "op": "remove"}, {"pointer": "/ntaCalculation/generator/forfait/sourceCorrectionReference", "op": "remove"}, {"pointer": "/ntaCalculation/generator/forfait/qualityDeclaration", "value": {"declarationReference": "r66 synthetic BCRG", "generationEfficiency": 3.137}}]"#,
    );
    close(
        "hp-air-decl E_H;el m1",
        p["spaceHeating"]["monthly"][0]["generatorElectricityKwh"]
            .as_f64()
            .expect("number"),
        473.5529408403193,
    );
    close(
        "hp-air-decl W_H;aux m1",
        p["spaceHeating"]["monthly"][0]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        33.41618485282008,
    );
    close(
        "hp-air-decl annual ambient heat",
        p["annualHeatPumpAmbientHeatKwh"].as_f64().expect("number"),
        3085.3839018823965,
    );
}

/// Hybrid heat pump: beta 9.56 and table 9.23 (p. 323-324), table 9.27 outdoor air 45 C 2,30 (p. 333),
/// table 9.25 supplementary HR107 LT 0,95 (p. 327), auxiliary 9.85 per appliance (p. 359-360).
#[test]
fn hybrid_heat_pump_single_zone() {
    let p = performance(
        "EPW001",
        r#"[{"pointer": "/ntaCalculation/generator/kind", "value": "hybrid_heat_pump"}, {"pointer": "/ntaCalculation/generator/designContext", "value": "new_build"}, {"pointer": "/ntaCalculation/generator/generators", "value": [{"id": "wp", "class": "heat_pump", "classificationReference": "ISSO 54 v2.0 EP-W203m: warmtepomp", "nominalThermalPowerKw": 10.0, "powerReference": "ISSO 54 v2.0 EP-W203m: 10 kW", "priorityEfficiency": 2.3, "efficiencyReference": "tabel 9.27 (2022) of de kwaliteitsverklaring"}, {"id": "ketel", "class": "other_boiler", "classificationReference": "ISSO 54 v2.0 EP-W203m: HR107-ketel", "nominalThermalPowerKw": 35.0, "powerReference": "ISSO 54 v2.0 EP-W203m: 35 kW", "priorityEfficiency": 0.95, "efficiencyReference": "tabel 9.25 (2022)"}]}, {"pointer": "/ntaCalculation/generator/forfait", "value": {"generatorId": "wp", "classificationSourceReference": "ISSO 54 v2.0 EP-W203m: elektrische lucht/waterwarmtepomp op buitenlucht", "scope": "residential_at_most25_kw", "source": "outdoor_air", "sink": "hydronic", "designSupplyTemperatureC": 45.0, "thermalCapacityKw": 10.0, "capacitySourceReference": "ISSO 54 v2.0 EP-W203m: vermogen warmtepomp 10 kW", "collectiveBuildingInstallation": false}}, {"pointer": "/ntaCalculation/generator/boiler/generatorId", "value": "ketel"}, {"pointer": "/ntaCalculation/generator/boiler/role", "value": "individual_supplementary"}, {"pointer": "/ntaCalculation/generator/boiler/equipmentReference", "value": "ISSO 54 v2.0 EP-W203m: HR107-ketel CW4, 35 kW, bijstook"}, {"pointer": "/ntaCalculation/generator/sourceSystem", "value": "individual"}, {"pointer": "/ntaCalculation/generator/sourceSystemReference", "value": "ISSO 54 v2.0 EP-W203m: individuele installatie binnen de thermische schil"}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "ISSO 54 v2.0 EP-W203m: buitenlucht"}}]"#,
    );
    close(
        "hybrid-1zone Q_hp;out m1",
        p["spaceHeating"]["monthly"][0]["heatPumpOutputKwh"]
            .as_f64()
            .expect("number"),
        616.7290416537185,
    );
    close(
        "hybrid-1zone E_H;gas m1",
        p["spaceHeating"]["monthly"][0]["naturalGasKwh"]
            .as_f64()
            .expect("number"),
        818.8256517063388,
    );
    close(
        "hybrid-1zone W_H;aux m1",
        p["spaceHeating"]["monthly"][0]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        35.413559439386766,
    );
}

/// Three collective boilers by priority (9.56-9.60, table 9.23, p. 322-324), table 9.25 collective HT
/// (p. 328), pilot flames 695 kWh/year x f_gebouw (p. 326), auxiliary 9.91 with f_gebouw (p. 364).
#[test]
fn three_collective_boilers_with_pilot_flames() {
    let p = performance("EPW205c", r#"[]"#);
    close(
        "EPW205c E_H;gas m1",
        p["spaceHeating"]["monthly"][0]["naturalGasKwh"]
            .as_f64()
            .expect("number"),
        1174.654243771543,
    );
    close(
        "EPW205c E_H;gas m6",
        p["spaceHeating"]["monthly"][5]["naturalGasKwh"]
            .as_f64()
            .expect("number"),
        11.424657534246576,
    );
    close(
        "EPW205c W_H;gen;aux m1",
        p["spaceHeating"]["monthly"][0]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        9.60648497287223,
    );
}

/// Indirect VR boiler outside the boundary, table 13.28 0,75 (p. 645), 150 l label C vessel in an unheated
/// space, 13.59 with table 13.9 (p. 567-570), auxiliary 13.181 with 10 W standby (p. 647-648).
#[test]
fn indirect_vr_boiler_outside_with_label_c_vessel() {
    let p = performance(
        "EPW404a",
        r#"[{"pointer": "/ntaCalculation/hotWater/generator", "value": {"kind": "indirect_boiler", "boiler": "vr", "oil": false, "insideBoundary": false, "alsoSpaceHeating": false}}, {"pointer": "/ntaCalculation/hotWater/storage", "value": [{"id": "vat-1", "volumeL": 150.0, "loss": {"method": "label", "label": "c"}, "connectionFactor": 3, "inHeatedZone": false, "sourceReference": "r66 synthetic"}]}]"#,
    );
    close(
        "dhw-ind-out Q_W;sto;ls m1",
        p["hotWater"]["months"][0]["storageLossKwh"]
            .as_f64()
            .expect("number"),
        150.07649090190196,
    );
    close(
        "dhw-ind-out E_W;in m1",
        p["hotWater"]["months"][0]["naturalGasKwh"]
            .as_f64()
            .expect("number"),
        509.1005043003109,
    );
    close(
        "dhw-ind-out W_W;aux m1",
        p["hotWater"]["months"][0]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        7.860007916047756,
    );
}

/// Electric boiler with a measured standby loss: 13.60 rounded up by annex X (2,083 -> 2,2 W/K, p. 568,
/// 1129-1130), 90 C for a boiling-water tap and the zone setpoint of 7.9.4 as ambient (13.58, p. 567).
#[test]
fn electric_boiler_with_measured_standby_and_boiling_tap() {
    let p = performance("EPW404c", r#"[]"#);
    close(
        "EPW404c Q_W;sto;ls m1",
        p["hotWater"]["months"][0]["storageLossKwh"]
            .as_f64()
            .expect("number"),
        119.41750358584353,
    );
    close(
        "EPW404c E_W;in m1",
        p["hotWater"]["months"][0]["electricityKwh"]
            .as_f64()
            .expect("number"),
        351.1663909091747,
    );
}

/// Booster heat pump with the forfait values, 13.162/13.163 (p. 632-633): 45 C held at 44 C, c_W;gen from
/// table 13.27 (individual hot-water heat pump, 13.8.4.7.3 c, p. 643-644).
#[test]
fn booster_heat_pump_forfait() {
    let p = performance("EPW406i", r#"[]"#);
    close(
        "EPW406i E_W;gen;in;prac;BWP;hj m1",
        p["hotWater"]["months"][0]["heatingSystemLoadKwh"]
            .as_f64()
            .expect("number"),
        280.540976745091,
    );
    close(
        "EPW406i W_W;aux;gen m1",
        p["hotWater"]["months"][0]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        67.65988262675725,
    );
}

/// Shower heat recovery with two units, 13.51/13.53 with table 13.8 (p. 563-565) and 13.9 (p. 532).
#[test]
fn two_showers_one_horizontal_one_vertical_recovery() {
    let p = performance("EPW403d", r#"[]"#);
    close(
        "EPW403d Q_W;rcd m1",
        p["hotWater"]["months"][0]["recoveredKwh"]
            .as_f64()
            .expect("number"),
        25.807663053351078,
    );
    close(
        "EPW403d Q_W;em (13.9) m1",
        p["hotWater"]["months"][0]["emissionInputKwh"]
            .as_f64()
            .expect("number"),
        205.94122426998013,
    );
}

/// Compression chiller with a declared EER 4,37: 4,35 (10.1, p. 366) with f_prpr 0,9 (NEN question 12).
#[test]
fn declared_eer_cooling() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/generators/0/generator", "value": {"kind": "compression", "declared": {"value": 4.37, "sourceReference": "r66 synthetic"}}}]"#,
    );
    close(
        "cool-decl E_C;el m7",
        p["cooling"]["months"][6]["electricityKwh"]
            .as_f64()
            .expect("number"),
        115.2930708095567,
    );
}

/// External cold, 10.78 with eta 1 and f_prpr 1 (p. 418-420), f_P;del;dc = 1,45/3 (table 5.2); no control
/// energy for external cold (kernel reading, NEN question 2).
#[test]
fn external_cold() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/generators/0/generator", "value": {"kind": "external_cold"}}, {"pointer": "/ntaCalculation/activeCooling/system", "value": "external_cold_with_cooling_emitter"}]"#,
    );
    close(
        "cool-ext Q_C;gen;ext;dc m7",
        p["cooling"]["months"][6]["districtColdKwh"]
            .as_f64()
            .expect("number"),
        451.3723722194145,
    );
    close(
        "cool-ext W_C;aux m7",
        p["cooling"]["months"][6]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        2.336096883903151,
    );
    close(
        "cool-ext E_P cooling/dc",
        row(&p, "cooling", "dc")["primaryFossilKwh"]
            .as_f64()
            .expect("number"),
        1087.6802333914536,
    );
}

/// Table chiller with a closed cooling tower: 10.80 with the table EER and 10.83 with table 10.33 0,033
/// (p. 420-423), control 10.87 (p. 425).
#[test]
fn chiller_with_closed_cooling_tower() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/generators/0/generator", "value": {"kind": "compression", "heatRejection": "closed_cooling_tower"}}]"#,
    );
    close(
        "cool-hr W_C;aux m7",
        p["cooling"]["months"][6]["auxiliaryElectricityKwh"]
            .as_f64()
            .expect("number"),
        29.636481261557392,
    );
}

/// Two PV systems, 16.2-16.4a (p. 678-682), table 17.15 full obstruction (p. 760-762) with c_sh held at
/// 0,75 below F 0,80 (kernel reading of table 16.3), export credit 5.13.
#[test]
fn pv_with_full_obstruction() {
    let p = performance("EPW501b", r#"[]"#);
    close(
        "EPW501b E_el;PV;out m6",
        p["electricityBalance"][5]["producedKwh"]
            .as_f64()
            .expect("number"),
        296.44091421696004,
    );
    close(
        "EPW501b E_exp;el m6",
        p["electricityBalance"][5]["exportedKwh"]
            .as_f64()
            .expect("number"),
        262.2325984274863,
    );
    close(
        "EPW501b E_P;tot (5.10)",
        p["annualPrimaryFossilKwh"].as_f64().expect("number"),
        6835.316507603183,
    );
}

/// PV with 5 kWh of building-bound storage: 5.14a/5.14b (p. 84-86) with f_BAT;cor 1.
#[test]
fn pv_with_a_battery() {
    let p = performance(
        "EPW501a",
        r#"[{"pointer": "/ntaCalculation/batteryStoragePresent", "value": true}, {"pointer": "/ntaCalculation/storage", "value": {"buildingBoundElectricalKwh": 5.0, "buildingBoundThermalKwh": 0.0, "sourceReference": "r66 synthetic battery"}}]"#,
    );
    close(
        "pv-bat E_P;BAT,out m6",
        p["energyByService"]["adjustments"][5]["storageCorrectionKwh"]
            .as_f64()
            .expect("number"),
        1.7104157894736849,
    );
    close(
        "pv-bat E_P;BAT annual",
        p["annualStorageCorrectionKwh"].as_f64().expect("number"),
        25.736696582757297,
    );
    close(
        "pv-bat E_P;tot (5.10)",
        p["annualPrimaryFossilKwh"].as_f64().expect("number"),
        6666.068113927736,
    );
}

// Review 67: the routes review 66 left unchecked, recalculated independently in the same way.

fn at(p: &Value, pointer: &str) -> f64 {
    p.pointer(pointer)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("{pointer}"))
}

/// Solar water heater, water heating only, preheater (13.7.2.2, p. 574-594): forfait glazed collector of table
/// 13.14, η_loop 13.72 with H_s;sto;hx of 13.73, H_loop;p of 13.75, storage H of 13.82 from label A, X/Y 13.97/13.99,
/// 13.100 with table 13.15, storage loss 13.102 at the levelled setpoint of 7.76 (13.69), Q_W;ren rounded down
/// (annex X), f_prac;sol 0,95 (13.66); pump 13.76 rounded up to 16 W (annex X: the next higher value) over 1 500 h
/// spread by irradiation (13.108/13.109).
#[test]
fn solar_water_heater_preheater() {
    let p = performance("EPW405a", r#"[]"#);
    close(
        "EPW405a Q_W;ren;prac m6",
        at(&p, "/hotWater/months/5/solarRenewableKwh"),
        190.0,
    );
    close(
        "EPW405a Q_W;ren;prac m7",
        at(&p, "/hotWater/months/6/solarRenewableKwh"),
        209.0,
    );
    close(
        "EPW405a W_W;aux;sol m1",
        at(&p, "/hotWater/months/0/solarAuxiliaryKwh"),
        0.8690975045343672,
    );
    close(
        "EPW405a Q_W;sol;ls;rbl m7",
        at(&p, "/hotWater/months/6/solarRecoverableKwh"),
        20.232297986032997,
    );
    close(
        "EPW405a Q_W;sol;ls;rbl m11",
        at(&p, "/hotWater/months/10/solarRecoverableKwh"),
        0.2566844762080898,
    );
}

/// Solar water heater with an integrated back-up part (SOL_TYPE PAR): 13.93/13.94 with V_bu 100 l of 220 l,
/// back-up storage loss 13.95 at 60 °C and the back-up output 13.105.
#[test]
fn solar_water_heater_integrated_backup() {
    let p = performance("EPW405e", r#"[]"#);
    close(
        "EPW405e Q_W;bu;sto;ls m1",
        at(&p, "/hotWater/months/0/solarBackupStorageLossKwh"),
        17.5214703405296,
    );
    close(
        "EPW405e Q_W;ren;prac m7",
        at(&p, "/hotWater/months/6/solarRenewableKwh"),
        209.0,
    );
    close(
        "EPW405e Q_W;bu;out m1",
        at(&p, "/hotWater/months/0/generatorOutputKwh"),
        232.17035766386078,
    );
    close(
        "EPW405e Q_W;sol;ls;rbl m7",
        at(&p, "/hotWater/months/6/solarRecoverableKwh"),
        36.186095059869444,
    );
}

/// Cooling method 2 (10.5.5, p. 409-416) for a split inverter room unit: PLV of table 10.19 row C (10.68/10.69),
/// f_EER;corr 10.73 with Δϑ_evap 20 K and Δϑ_cond 10 K (table 10.22), ϑ_e;kg of table 10.24, the required outlet
/// ϑ_C;int;inc (10.10) and f_prpr 0,60 (direct condensation).
#[test]
fn cooling_method_2_room_air_conditioner() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/generators/0/generator", "value": {"kind": "room_air_conditioner", "performance": {"method": "en14511", "nominalEer": 3.21, "nominalCapacityKw": 3.5, "nominalEvaporatorOutletC": 27.0, "nominalCondenserInletC": 35.0, "roomUnitType": "split_inverter", "sourceReference": "r67 synthetic"}}}, {"pointer": "/ntaCalculation/cooling/emission/emitter", "value": "fan_coil_or_rac_on_outer_wall"}, {"pointer": "/ntaCalculation/cooling/emission/fanCoilCount", "value": 1}]"#,
    );
    close(
        "rac E_C;el m7",
        at(&p, "/cooling/months/6/electricityKwh"),
        122.86484213685722,
    );
    close(
        "rac E_C;el m8",
        at(&p, "/cooling/months/7/electricityKwh"),
        191.81164614750148,
    );
    close(
        "rac EER m7",
        at(&p, "/cooling/generatorShares/0/monthlyEer/6"),
        3.6442604372252374,
    );
}

/// Cooling method 2 for a water-cooled chiller on a dry cooler: table 10.21 (heat rejection to water), Δϑ_evap 6 K,
/// Δϑ_cond 4 K, ϑ_ref = ϑ_e;kg + 15 (table 10.26), the table 10.8 outlet 17 − Δϑ_int;inc read literally, f_prpr 0,9.
#[test]
fn cooling_method_2_water_cooled_chiller() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/generators/0/generator", "value": {"kind": "compression", "heatRejection": "dry_cooler", "performance": {"method": "en14511", "nominalEer": 4.5, "nominalCapacityKw": 2.0, "nominalEvaporatorOutletC": 7.0, "nominalCondenserInletC": 30.0, "sourceReference": "r67 synthetic"}}}]"#,
    );
    close(
        "wtr E_C;el m8",
        at(&p, "/cooling/months/7/electricityKwh"),
        129.70624911986462,
    );
    close(
        "wtr E_C;el m10",
        at(&p, "/cooling/months/9/electricityKwh"),
        2.9941390824029273,
    );
    close(
        "wtr EER m8",
        at(&p, "/cooling/generatorShares/0/monthlyEer/7"),
        5.488240443536629,
    );
}

/// Measured booster heat pump, annex W (p. 1119-1126): COP line W.11-W.13 through the 24 °C and 40 °C tests after
/// the tap-quantity correction W.14/W.15 (class 3), W.1 with f_prac 0,95 and W.2 (f_C 0) with W.8-W.10, both
/// rounded up (annex X).
#[test]
fn measured_booster_heat_pump() {
    let p = performance(
        "EPW406i",
        r#"[{"pointer": "/ntaCalculation/hotWater/generator", "value": {"kind": "booster_heat_pump", "lowTest": {"sourceTemperatureC": 24.0, "cop": 3.2}, "highTest": {"sourceTemperatureC": 40.0, "cop": 4.1}, "measuredClass": "class3", "standingLossKw": 0.012, "sourceTemperaturesC": [38.0, 38.0, 36.0, 34.0, 30.0, 28.0, 26.0, 26.0, 30.0, 34.0, 36.0, 38.0], "heatSource": {"kind": "heating_system"}, "testReportReference": "r67 synthetic"}}]"#,
    );
    let aux = [
        65.0, 60.0, 65.0, 65.0, 75.0, 75.0, 80.0, 80.0, 70.0, 70.0, 65.0, 65.0,
    ];
    let hj = [
        220.0, 190.0, 220.0, 200.0, 200.0, 190.0, 190.0, 190.0, 190.0, 200.0, 200.0, 220.0,
    ];
    for (m, (aux, hj)) in aux.iter().zip(hj.iter()).enumerate() {
        close(
            &format!("bwp W_W;aux;gen;bwp m{}", m + 1),
            at(&p, &format!("/hotWater/months/{m}/auxiliaryElectricityKwh")),
            *aux,
        );
        close(
            &format!("bwp E_W;gen,in;hj m{}", m + 1),
            at(&p, &format!("/hotWater/months/{m}/heatingSystemLoadKwh")),
            *hj,
        );
    }
}

/// Micro-CHP method 1 (9.6.6.2, p. 346-352): 9.66-9.83 between the stand-by point (P_th;sb 0), CHP_100+Sup_0 and
/// CHP_100+Sup_100 with f_prac 0,95, t = t_H;op of 9.32a, recoverable stand-by loss 9.80; the kernel's documented
/// readings kept (micro_chp INTERPRETATIONS): heat above full load as supplementary heat and f_Hs/Hi 1,11.
#[test]
fn micro_chp_method_1() {
    let p = performance(
        "EPW204g",
        r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "chp", "method1": {"kind": "stirling_engine", "fuel": "natural_gas", "location": "heated_space", "fullLoad": {"thermalPowerKw": 24.0, "thermalEfficiency": 0.92, "electricEfficiency": 0.04, "auxiliaryPowerKw": 0.06}, "chpOnly": {"thermalPowerKw": 5.5, "thermalEfficiency": 0.78, "electricEfficiency": 0.14, "auxiliaryPowerKw": 0.04}, "standbyLossKw": 0.1, "pilotKw": 0.0, "standbyElectricKw": 0.0, "standbyAuxiliaryKw": 0.005, "testReportReference": "r67 synthetic"}, "auxiliary": {"electricallyConnectedDevices": 1, "nominalPowerKw": 24.0, "sourceReference": "r67 synthetic"}, "equipmentReference": "r67 synthetic"}}]"#,
    );
    close(
        "chp1 E_el m2",
        at(&p, "/spaceHeating/monthly/1/chpElectricityKwh"),
        33.83909139807581,
    );
    close(
        "chp1 E_gen;in m2",
        at(&p, "/spaceHeating/monthly/1/chpInputKwh"),
        1035.5687296499302,
    );
    close(
        "chp1 W_gen;aux m2",
        at(&p, "/spaceHeating/monthly/1/chpGeneratorAuxiliaryKwh"),
        1.926317815474853,
    );
    close(
        "chp1 Q_gen;ls;rbl m2",
        at(&p, "/spaceHeating/monthly/1/generatorRecoverableLossKwh"),
        3.249166666666667,
    );
    close(
        "chp1 E_gen;in m1 (excess)",
        at(&p, "/spaceHeating/monthly/0/chpInputKwh"),
        1822.097877640744,
    );
    close(
        "chp1 E_gen;in m5 (stand-by)",
        at(&p, "/spaceHeating/monthly/4/chpInputKwh"),
        82.58400000000002,
    );
}

// Review 68: cooling method 1 and annex Q product values, recalculated independently in the same way.

/// Cooling method 1 (10.5.4, p. 401-409): the five equations of 10.63 for C1-C4 and Δϑ_corr with a measured
/// fifth point, f_EER per bin 10.56/10.57 with T_0;abs 273,16 K (p. 413), ϑ_C;evap;out = ϑ_C;int;inc − 20 K
/// (10.61, direct evaporation in the room), Φ_net per bin 10.60 with table 10.17 at the cooling limit, the
/// month weighting 10.55 with the printed f_t;tot row of table 10.18, and 10.54 with f_prpr 0,9.
#[test]
fn cooling_method_1_with_a_fifth_point() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/generators/0/generator", "value": {"kind": "room_air_conditioner", "performance": {"method": "en14825", "nominalEer": 3.4, "nominalCapacityKw": 3.5, "minimumCapacityKw": 1.0, "testPoints": [{"partLoadPercent": 100, "eer": 3.4, "evaporatorOutletC": 12.0, "condenserInletC": 35.0}, {"partLoadPercent": 74, "eer": 4.9, "evaporatorOutletC": 13.0, "condenserInletC": 30.0}, {"partLoadPercent": 47, "eer": 7.1, "evaporatorOutletC": 14.0, "condenserInletC": 25.0}, {"partLoadPercent": 21, "eer": 8.6, "evaporatorOutletC": 15.0, "condenserInletC": 20.0}], "fifthPoint": {"partLoadPercent": 47, "eer": 5.3, "evaporatorOutletC": 14.0, "condenserInletC": 35.0}, "sourceReference": "r68 synthetic"}}}, {"pointer": "/ntaCalculation/cooling/emission/emitter", "value": "fan_coil_or_rac_on_outer_wall"}, {"pointer": "/ntaCalculation/cooling/emission/fanCoilCount", "value": 1}]"#,
    );
    let expected = [
        0.4603403015901443,
        -1.240043896515591,
        1.0523145526545443,
        -0.05433525700681298,
        39.2434988179669,
    ];
    for (index, value) in expected.iter().enumerate() {
        close(
            &format!("m1 C/dT_corr {index}"),
            at(
                &p,
                &format!("/cooling/generatorShares/0/en14825Coefficients/{index}"),
            ),
            *value,
        );
    }
    close(
        "m1 EER m7",
        at(&p, "/cooling/generatorShares/0/monthlyEer/6"),
        4.533652810681888,
    );
    close(
        "m1 EER m8",
        at(&p, "/cooling/generatorShares/0/monthlyEer/7"),
        4.479986062497868,
    );
    close(
        "m1 E_C;el m8",
        at(&p, "/cooling/months/7/electricityKwh"),
        157.49963783028423,
    );
}

/// Cooling method 1 without a fifth point: 10.64 with Δϑ_corr = 0, a condenser inlet limit of 25 °C (10.62) and a
/// minimum part load of 71 % so most bins take 10.57.
#[test]
fn cooling_method_1_without_a_fifth_point() {
    let p = performance(
        "EPW301a",
        r#"[{"pointer": "/ntaCalculation/cooling/generators/0/generator", "value": {"kind": "room_air_conditioner", "performance": {"method": "en14825", "nominalEer": 3.1, "nominalCapacityKw": 3.5, "minimumCapacityKw": 2.5, "testPoints": [{"partLoadPercent": 100, "eer": 3.1, "evaporatorOutletC": 12.0, "condenserInletC": 35.0}, {"partLoadPercent": 74, "eer": 4.4, "evaporatorOutletC": 12.0, "condenserInletC": 30.0}, {"partLoadPercent": 47, "eer": 6.3, "evaporatorOutletC": 12.0, "condenserInletC": 25.0}, {"partLoadPercent": 21, "eer": 7.9, "evaporatorOutletC": 12.0, "condenserInletC": 20.0}], "condenserInletLimitC": 25.0, "sourceReference": "r68 synthetic"}}}, {"pointer": "/ntaCalculation/cooling/emission/emitter", "value": "fan_coil_or_rac_on_outer_wall"}, {"pointer": "/ntaCalculation/cooling/emission/fanCoilCount", "value": 1}]"#,
    );
    let expected = [
        0.16626214298917724,
        -0.4109480605495624,
        0.30033645466484676,
        0.025005936454241994,
        0.0,
    ];
    for (index, value) in expected.iter().enumerate() {
        close(
            &format!("m1b C/dT_corr {index}"),
            at(
                &p,
                &format!("/cooling/generatorShares/0/en14825Coefficients/{index}"),
            ),
            *value,
        );
    }
    close(
        "m1b EER m7",
        at(&p, "/cooling/generatorShares/0/monthlyEer/6"),
        2.681753397252766,
    );
    close(
        "m1b EER m8",
        at(&p, "/cooling/generatorShares/0/monthlyEer/7"),
        2.679769590862541,
    );
    close(
        "m1b E_C;el m8",
        at(&p, "/cooling/months/7/electricityKwh"),
        263.3047948353739,
    );
}

const ANNEX_Q_ON_OFF: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "outdoor_air_water", "maximumPower": {"condition1": {"evaporatorInC": 7.0, "evaporatorOutC": 6.0, "condenserInC": 30.0, "condenserOutC": 35.0, "cop": 4.6, "heatingPowerKw": 6.0}, "condition2": {"evaporatorInC": 7.0, "evaporatorOutC": 6.0, "condenserInC": 47.0, "condenserOutC": 55.0, "cop": 2.9, "heatingPowerKw": 5.6}, "condition3": {"evaporatorInC": -7.0, "evaporatorOutC": -8.0, "condenserInC": 30.0, "condenserOutC": 35.0, "cop": 2.7, "heatingPowerKw": 4.4}, "condition4": {"evaporatorInC": 2.0, "evaporatorOutC": 1.0, "condenserInC": 30.0, "condenserOutC": 35.0, "cop": 3.3, "heatingPowerKw": 4.9}}, "modulation": {"method": "on_off"}, "switchOff": {"minEvaporatorInC": -20.0}, "testReportReference": "r68 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r68 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r68 synthetic"}}]"#;

const ANNEX_Q_UNDERSIZED: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "outdoor_air_water", "maximumPower": {"condition1": {"evaporatorInC": 7.0, "evaporatorOutC": 6.0, "condenserInC": 30.0, "condenserOutC": 35.0, "cop": 4.6, "heatingPowerKw": 2.4}, "condition2": {"evaporatorInC": 7.0, "evaporatorOutC": 6.0, "condenserInC": 47.0, "condenserOutC": 55.0, "cop": 2.9, "heatingPowerKw": 2.2}, "condition3": {"evaporatorInC": -7.0, "evaporatorOutC": -8.0, "condenserInC": 30.0, "condenserOutC": 35.0, "cop": 2.7, "heatingPowerKw": 1.7}}, "modulation": {"method": "on_off"}, "switchOff": {"minEvaporatorInC": -5.0, "minCop": 2.0}, "testReportReference": "r68 synthetic"}, "designSupplyTemperatureC": 45.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r68 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r68 synthetic"}}]"#;

/// Annex Q, on/off outdoor-air/water heat pump (p. 1027-1069): planes Q.51-Q.53 and Q.60-Q.62 through conditions
/// 1-3 at the operative temperatures Q.69/Q.70, dT over all four measured conditions (Q.49/Q.50), f_cor;bu;max from
/// condition 4 (Q.81-Q.83) with Q.32-Q.34, θ_cond;in of table Q.5, P_max Q.37/Q.45, θ_cond;out;max Q.36 with the
/// design flow Q.47/Q.48 (table Q.7; θ_evap;nom −10 °C, kernel reading), COP Q.31/Q.35, demand Q.46 with table Q.6
/// (WHE), Q.29/Q.9, totals Q.1-Q.4; 9.63 with f_prac 0,95 and the same value every month (p. 340). Every bin is
/// covered, so the kernel applies F_H;gen = 1 (its documented reading of Q.1).
#[test]
fn annex_q_on_off_outdoor_air() {
    let p = performance("EPW001", ANNEX_Q_ON_OFF);
    let q = "/spaceHeating/annexQ/annexQ";
    close(
        "q-onoff F",
        at(&p, &format!("{q}/energyFraction")),
        0.99973692,
    );
    close(
        "q-onoff eta",
        at(&p, &format!("{q}/generationEfficiency")),
        4.319731312142112,
    );
    close(
        "q-onoff Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4670.470501957207,
    );
    close(
        "q-onoff Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        1081.1946772775473,
    );
    for (index, power, cop) in [
        (9, 6.165555555555555, 5.248700735824447),
        (14, 5.545238095238095, 3.691025186126918),
        (19, 4.922698412698413, 3.2666135444775524),
        (23, 4.4255555555555555, 2.8202829394104834),
    ] {
        close(
            &format!("q-onoff P_max bin {index}"),
            at(&p, &format!("{q}/bins/{index}/maximumPowerKw")),
            power,
        );
        close(
            &format!("q-onoff COP bin {index}"),
            at(&p, &format!("{q}/bins/{index}/cop")),
            cop,
        );
    }
    close(
        "q-onoff E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        339.83921927704716,
    );
}

/// Annex Q, undersized on/off heat pump without condition 4 (f_cor;bu;max 0,75), switch-off below −5 °C inlet and a
/// minimum COP of 2 (table Q.3), design supply 45 °C: F_H;gen < 1 and the electric backup (COP 1,0) takes the rest.
#[test]
fn annex_q_undersized_with_switch_off_and_backup() {
    let p = performance("EPW001", ANNEX_Q_UNDERSIZED);
    let q = "/spaceHeating/annexQ/annexQ";
    close(
        "q-small F",
        at(&p, &format!("{q}/energyFraction")),
        0.9728849680329282,
    );
    close(
        "q-small eta",
        at(&p, &format!("{q}/generationEfficiency")),
        3.7788288772602825,
    );
    close(
        "q-small COP bin -3",
        at(&p, &format!("{q}/bins/19/cop")),
        2.5427664756156054,
    );
    close(
        "q-small off bin -7",
        at(&p, &format!("{q}/bins/23/switchOffFactor")),
        0.0,
    );
    close(
        "q-small Q_hp m1",
        at(&p, "/spaceHeating/monthly/0/heatPumpOutputKwh"),
        1356.7984235598763,
    );
    close(
        "q-small E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        415.76511761098016,
    );
}
const ANNEX_Q_BRINE_WATER: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "brine_water", "maximumPower": {"condition1": {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 30, "condenserOutC": 35, "cop": 4.5, "heatingPowerKw": 8.0}, "condition2": {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 47, "condenserOutC": 55, "cop": 2.9, "heatingPowerKw": 7.4}, "condition3": {"evaporatorInC": 10, "evaporatorOutC": 7, "condenserInC": 30, "condenserOutC": 35, "cop": 5.8, "heatingPowerKw": 10.2}}, "modulation": {"method": "on_off"}, "switchOff": {}, "sourcePump": {"nominalPowerW": 120.0, "overrunS": 60.0}, "testReportReference": "r69 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r69 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r69 synthetic"}}]"#;

const ANNEX_Q_EXHAUST_AIR: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "exhaust_air_water", "maximumPower": {"condition1": {"evaporatorInC": 20, "evaporatorOutC": 12, "condenserInC": 30, "condenserOutC": 35, "cop": 4.2, "heatingPowerKw": 1.6}, "condition2": {"evaporatorInC": 20, "evaporatorOutC": 12, "condenserInC": 47, "condenserOutC": 55, "cop": 2.6, "heatingPowerKw": 1.35}}, "modulation": {"method": "on_off"}, "switchOff": {}, "testReportReference": "r69 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r69 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": true, "sourceReference": "r69 synthetic"}}]"#;

const ANNEX_Q_COMBINED_AIR: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "combined_air_water", "outdoorAirFraction": 0.6, "maximumPower": {"condition1": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 30, "condenserOutC": 35, "cop": 4.6, "heatingPowerKw": 6.0}, "condition2": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 47, "condenserOutC": 55, "cop": 2.9, "heatingPowerKw": 5.6}, "condition3": {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 30, "condenserOutC": 35, "cop": 2.7, "heatingPowerKw": 4.4}, "condition4": {"evaporatorInC": 2, "evaporatorOutC": 1, "condenserInC": 30, "condenserOutC": 35, "cop": 3.3, "heatingPowerKw": 4.9}}, "modulation": {"method": "on_off"}, "switchOff": {"minEvaporatorInC": -20.0}, "testReportReference": "r69 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r69 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r69 synthetic", "combinedOutdoorAndExhaustAir": true, "outdoorAirHeatFraction": 0.6, "outdoorAirFractionReference": "r69"}}]"#;

const ANNEX_Q_AIR_AIR: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "air_air", "maximumPower": {"condition1": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 20, "condenserOutC": 35, "cop": 4.0, "heatingPowerKw": 5.0}, "condition3": {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 20, "condenserOutC": 32, "cop": 2.6, "heatingPowerKw": 3.6}}, "modulation": {"method": "on_off"}, "switchOff": {"minEvaporatorInC": -15.0}, "testReportReference": "r69 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r69 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r69 synthetic"}}, {"pointer": "/ntaCalculation/emission", "value": {"system": "air_heating", "balancing": "not_applicable", "control": "individual_room_thermostats", "sourceReference": "r69"}}, {"op": "remove", "pointer": "/ntaCalculation/distributionSystem"}, {"pointer": "/ntaCalculation/distribution", "value": {"method": "heated_zone_only_space_heating", "sourceReference": "r69"}}]"#;

const ANNEX_Q_GAS_BACKUP: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "outdoor_air_water", "maximumPower": {"condition1": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 30, "condenserOutC": 35, "cop": 4.6, "heatingPowerKw": 2.4}, "condition2": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 47, "condenserOutC": 55, "cop": 2.9, "heatingPowerKw": 2.2}, "condition3": {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 30, "condenserOutC": 35, "cop": 2.7, "heatingPowerKw": 1.7}}, "modulation": {"method": "on_off"}, "switchOff": {"minEvaporatorInC": -5.0, "minCop": 2.0}, "testReportReference": "r69 synthetic"}, "designSupplyTemperatureC": 45.0, "backup": {"kind": "gas_boiler", "boiler": {"generatorId": "hr107", "role": "individual_main", "location": "inside_thermal_boundary", "kind": "hr107", "fuel": "natural_gas", "averageDesignEmissionTemperatureC": 42.5, "emissionCircuit": "direct", "equipmentReference": "r69", "locationReference": "r69", "temperatureAndCircuitReference": "r69", "pilotFlamePresent": false, "installationYear": 2021, "installationYearReference": "r69"}}, "equipmentReference": "r69 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r69 synthetic"}}]"#;

const ANNEX_Q_MODULATING: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "outdoor_air_water", "maximumPower": {"condition1": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 30, "condenserOutC": 35, "cop": 4.6, "heatingPowerKw": 6.0}, "condition2": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 47, "condenserOutC": 55, "cop": 2.9, "heatingPowerKw": 5.6}, "condition3": {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 30, "condenserOutC": 35, "cop": 2.7, "heatingPowerKw": 4.4}, "condition4": {"evaporatorInC": 2, "evaporatorOutC": 1, "condenserInC": 30, "condenserOutC": 35, "cop": 3.3, "heatingPowerKw": 4.9}}, "modulation": {"method": "modulating", "minimumPowerKw": 1.2, "lowRange": [{"evaporatorInC": -10, "evaporatorOutC": -11, "condenserInC": 30, "condenserOutC": 35, "cop": 2.4, "heatingPowerKw": 4.0}, {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 29.5, "condenserOutC": 34, "cop": 2.9, "heatingPowerKw": 3.9}, {"evaporatorInC": 2, "evaporatorOutC": 1, "condenserInC": 26.5, "condenserOutC": 30, "cop": 4.3, "heatingPowerKw": 2.6}, {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 24.5, "condenserOutC": 27, "cop": 5.4, "heatingPowerKw": 1.75}, {"evaporatorInC": 12, "evaporatorOutC": 11, "condenserInC": 22.5, "condenserOutC": 24, "cop": 6.6, "heatingPowerKw": 0.8}], "highRange": [{"evaporatorInC": -10, "evaporatorOutC": -11, "condenserInC": 48, "condenserOutC": 55, "cop": 1.6, "heatingPowerKw": 3.6}, {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 46, "condenserOutC": 52, "cop": 1.9, "heatingPowerKw": 3.5}, {"evaporatorInC": 2, "evaporatorOutC": 1, "condenserInC": 38, "condenserOutC": 42, "cop": 2.9, "heatingPowerKw": 2.3}, {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 33, "condenserOutC": 36, "cop": 3.8, "heatingPowerKw": 1.5}, {"evaporatorInC": 12, "evaporatorOutC": 11, "condenserInC": 28, "condenserOutC": 30, "cop": 4.7, "heatingPowerKw": 0.7}], "condenserPumpModulating": true, "sourcePumpModulating": true}, "switchOff": {"minEvaporatorInC": -20.0}, "testReportReference": "r69 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r69 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r69 synthetic"}}]"#;

const ANNEX_Q_MODULATING_BRINE: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "brine_water", "maximumPower": {"condition1": {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 30, "condenserOutC": 35, "cop": 4.5, "heatingPowerKw": 8.0}, "condition2": {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 47, "condenserOutC": 55, "cop": 2.9, "heatingPowerKw": 7.4}, "condition3": {"evaporatorInC": 10, "evaporatorOutC": 7, "condenserInC": 30, "condenserOutC": 35, "cop": 5.8, "heatingPowerKw": 10.2}}, "modulation": {"method": "modulating", "minimumPowerKw": 1.5, "lowRange": [{"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 30, "condenserOutC": 35, "cop": 4.5, "heatingPowerKw": 8.0}, {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 29.5, "condenserOutC": 34, "cop": 4.8, "heatingPowerKw": 7.0}, {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 27, "condenserOutC": 30, "cop": 5.6, "heatingPowerKw": 4.3}, {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 25, "condenserOutC": 27, "cop": 6.1, "heatingPowerKw": 2.8}, {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 23, "condenserOutC": 24, "cop": 6.5, "heatingPowerKw": 1.2}], "highRange": [{"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 48, "condenserOutC": 55, "cop": 2.8, "heatingPowerKw": 7.4}, {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 46.5, "condenserOutC": 52, "cop": 3.0, "heatingPowerKw": 6.5}, {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 39, "condenserOutC": 42, "cop": 3.7, "heatingPowerKw": 4.0}, {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 34, "condenserOutC": 36, "cop": 4.3, "heatingPowerKw": 2.6}, {"evaporatorInC": 0, "evaporatorOutC": -3, "condenserInC": 29, "condenserOutC": 30, "cop": 5.0, "heatingPowerKw": 1.1}], "condenserPumpModulating": true, "sourcePumpModulating": true}, "switchOff": {}, "sourcePump": {"nominalPowerW": 120.0, "overrunS": 60.0}, "testReportReference": "r69 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r69 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r69 synthetic"}}]"#;

/// Review 69. Annex Q, on/off brine/water heat pump with a source pump (p. 1028-1056): evaporator inlet of table Q.4,
/// f_cor;bu 1 (Q.3.8), θ_evap;nom −1,5 °C (Q.48), source pump Q.5/Q.7/Q.8 with t_on of Q.9 inside the efficiency of
/// Q.2/Q.4; 9.6.3 (p. 340) adds W_H;aux;hp;an / (12 × 3,6) as auxiliary energy every month (literal text).
#[test]
fn annex_q_brine_water_with_source_pump() {
    let p = performance("EPW001", ANNEX_Q_BRINE_WATER);
    let q = "/spaceHeating/annexQ/annexQ";
    close("bw F", at(&p, &format!("{q}/energyFraction")), 0.99973692);
    close(
        "bw eta",
        at(&p, &format!("{q}/generationEfficiency")),
        5.023885721386021,
    );
    close(
        "bw Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4670.470501957207,
    );
    close(
        "bw Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        929.6530138167011,
    );
    close(
        "bw W_aux",
        at(&p, &format!("{q}/sourcePumpKwh")),
        65.77810328085602,
    );
    close(
        "bw P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        8.883555555555555,
    );
    close(
        "bw COP bin 9",
        at(&p, &format!("{q}/bins/9/cop")),
        5.545059452701766,
    );
    close(
        "bw on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        41.52412596784017,
    );
    close(
        "bw P_max bin 16",
        at(&p, &format!("{q}/bins/16/maximumPowerKw")),
        8.49088888888889,
    );
    close(
        "bw COP bin 16",
        at(&p, &format!("{q}/bins/16/cop")),
        5.098887623281012,
    );
    close(
        "bw on hours bin 16",
        at(&p, &format!("{q}/bins/16/onHours")),
        31.027665450491885,
    );
    close(
        "bw P_max bin 21",
        at(&p, &format!("{q}/bins/21/maximumPowerKw")),
        8.307555555555556,
    );
    close(
        "bw COP bin 21",
        at(&p, &format!("{q}/bins/21/cop")),
        4.8417276010557035,
    );
    close(
        "bw on hours bin 21",
        at(&p, &format!("{q}/bins/21/onHours")),
        6.592982202609814,
    );
    close(
        "bw f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.13749975950733773,
    );
    close(
        "bw E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        292.20690875906007,
    );
    close(
        "bw aux m1",
        at(&p, "/spaceHeating/monthly/0/auxiliaryElectricityKwh"),
        5.481508606738001,
    );
}

/// Review 69. Annex Q, exhaust-air-only air/water heat pump: θ_evap;in 20 °C (Q.41), constants from conditions 1 and 2
/// (Q.54-Q.56, Q.63-Q.65), f_cor;bu 1; undersized, so the electric backup (COP 1,0) takes the rest of every month.
#[test]
fn annex_q_exhaust_air_only() {
    let p = performance("EPW001", ANNEX_Q_EXHAUST_AIR);
    let q = "/spaceHeating/annexQ/annexQ";
    close(
        "exh F",
        at(&p, &format!("{q}/energyFraction")),
        0.9923593016639592,
    );
    close(
        "exh eta",
        at(&p, &format!("{q}/generationEfficiency")),
        4.768926489097366,
    );
    close(
        "exh Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4636.0044858245055,
    );
    close(
        "exh Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        972.1274790926753,
    );
    close("exh W_aux", at(&p, &format!("{q}/sourcePumpKwh")), 0.0);
    close(
        "exh P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        1.7000000000000002,
    );
    close(
        "exh COP bin 9",
        at(&p, &format!("{q}/bins/9/cop")),
        4.8751396258776865,
    );
    close(
        "exh on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        216.98934113599717,
    );
    close(
        "exh P_max bin 16",
        at(&p, &format!("{q}/bins/16/maximumPowerKw")),
        1.6555555555555557,
    );
    close(
        "exh COP bin 16",
        at(&p, &format!("{q}/bins/16/cop")),
        4.594811598863697,
    );
    close(
        "exh on hours bin 16",
        at(&p, &format!("{q}/bins/16/onHours")),
        159.13235828158986,
    );
    close(
        "exh P_max bin 21",
        at(&p, &format!("{q}/bins/21/maximumPowerKw")),
        1.625,
    );
    close(
        "exh COP bin 21",
        at(&p, &format!("{q}/bins/21/cop")),
        4.402086080291579,
    );
    close(
        "exh on hours bin 21",
        at(&p, &format!("{q}/bins/21/onHours")),
        29.0,
    );
    close(
        "exh f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.6777152841178543,
    );
    close(
        "exh E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        316.1328426943665,
    );
}

/// Review 69. Annex Q, combined outdoor and exhaust air (f_buitenlucht 0,6): θ_evap;in of Q.43, planes through
/// conditions 1-3, f_cor;bu;max from condition 4 (Q.81-Q.83).
#[test]
fn annex_q_combined_outdoor_and_exhaust_air() {
    let p = performance("EPW001", ANNEX_Q_COMBINED_AIR);
    let q = "/spaceHeating/annexQ/annexQ";
    close("comb F", at(&p, &format!("{q}/energyFraction")), 0.99973692);
    close(
        "comb eta",
        at(&p, &format!("{q}/generationEfficiency")),
        5.167431824322921,
    );
    close(
        "comb Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4670.470501957207,
    );
    close(
        "comb Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        903.8281801752014,
    );
    close("comb W_aux", at(&p, &format!("{q}/sourcePumpKwh")), 0.0);
    close(
        "comb P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        6.759841269841269,
    );
    close(
        "comb COP bin 9",
        at(&p, &format!("{q}/bins/9/cop")),
        5.931674301758937,
    );
    close(
        "comb on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        54.569606771233715,
    );
    close(
        "comb P_max bin 14",
        at(&p, &format!("{q}/bins/14/maximumPowerKw")),
        6.368095238095237,
    );
    close(
        "comb COP bin 14",
        at(&p, &format!("{q}/bins/14/cop")),
        4.486822884643272,
    );
    close(
        "comb on hours bin 14",
        at(&p, &format!("{q}/bins/14/onHours")),
        46.683313536368225,
    );
    close(
        "comb P_max bin 23",
        at(&p, &format!("{q}/bins/23/maximumPowerKw")),
        5.6598412698412695,
    );
    close(
        "comb COP bin 23",
        at(&p, &format!("{q}/bins/23/cop")),
        4.238766499428271,
    );
    close(
        "comb on hours bin 23",
        at(&p, &format!("{q}/bins/23/onHours")),
        5.11689257033558,
    );
    close(
        "comb f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.18907111847193941,
    );
    close(
        "comb E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        284.08969223262875,
    );
}

/// Review 69. Annex Q, air/air heat pump with air heating: constants from conditions 1 and 3 on the evaporator
/// temperature only (Q.57-Q.59, Q.66-Q.68), no condition 4 so f_cor;bu;max 0,75.
#[test]
fn annex_q_air_air() {
    let p = performance("EPW001", ANNEX_Q_AIR_AIR);
    let q = "/spaceHeating/annexQ/annexQ";
    close(
        "aa F",
        at(&p, &format!("{q}/energyFraction")),
        0.9997369200000001,
    );
    close(
        "aa eta",
        at(&p, &format!("{q}/generationEfficiency")),
        3.252730016153208,
    );
    close(
        "aa Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4479.338322845153,
    );
    close(
        "aa Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        1377.101173660449,
    );
    close("aa W_aux", at(&p, &format!("{q}/sourcePumpKwh")), 0.0);
    close(
        "aa P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        5.0,
    );
    close("aa COP bin 9", at(&p, &format!("{q}/bins/9/cop")), 4.0);
    close(
        "aa on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        70.75718562772356,
    );
    close(
        "aa P_max bin 14",
        at(&p, &format!("{q}/bins/14/maximumPowerKw")),
        4.500000000000001,
    );
    close(
        "aa COP bin 14",
        at(&p, &format!("{q}/bins/14/cop")),
        2.6250000000000004,
    );
    close(
        "aa on hours bin 14",
        at(&p, &format!("{q}/bins/14/onHours")),
        63.3595293689384,
    );
    close(
        "aa P_max bin 23",
        at(&p, &format!("{q}/bins/23/maximumPowerKw")),
        3.6000000000000005,
    );
    close(
        "aa COP bin 23",
        at(&p, &format!("{q}/bins/23/cop")),
        2.6000000000000005,
    );
    close(
        "aa on hours bin 23",
        at(&p, &format!("{q}/bins/23/onHours")),
        7.715450372623382,
    );
    close(
        "aa f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.262158050208687,
    );
    close(
        "aa E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        432.76105599965587,
    );
}

/// Review 69. Annex Q with a gas boiler as backup (hybrid): the undersized heat pump of review 68 at 45 °C with
/// switch-off below −5 °C and COP 2; the rest of every month goes to the HR107 boiler (table 9.25, 0,975).
#[test]
fn annex_q_with_a_gas_boiler_backup() {
    let p = performance("EPW001", ANNEX_Q_GAS_BACKUP);
    let q = "/spaceHeating/annexQ/annexQ";
    close(
        "hyb F",
        at(&p, &format!("{q}/energyFraction")),
        0.9728849680329282,
    );
    close(
        "hyb eta",
        at(&p, &format!("{q}/generationEfficiency")),
        3.7788288772602825,
    );
    close(
        "hyb Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4545.026250501353,
    );
    close(
        "hyb Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        1202.7605372267021,
    );
    close("hyb W_aux", at(&p, &format!("{q}/sourcePumpKwh")), 0.0);
    close(
        "hyb P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        2.4574074074074073,
    );
    close(
        "hyb COP bin 9",
        at(&p, &format!("{q}/bins/9/cop")),
        5.012547626077299,
    );
    close(
        "hyb on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        150.11018474969512,
    );
    close(
        "hyb P_max bin 19",
        at(&p, &format!("{q}/bins/19/maximumPowerKw")),
        1.8562962962962963,
    );
    close(
        "hyb COP bin 19",
        at(&p, &format!("{q}/bins/19/cop")),
        2.5427664756156054,
    );
    close(
        "hyb on hours bin 19",
        at(&p, &format!("{q}/bins/19/onHours")),
        45.10450212040932,
    );
    close(
        "hyb P_max bin 23",
        at(&p, &format!("{q}/bins/23/maximumPowerKw")),
        1.6151851851851853,
    );
    close(
        "hyb COP bin 23",
        at(&p, &format!("{q}/bins/23/cop")),
        1.9942431847259137,
    );
    close(
        "hyb on hours bin 23",
        at(&p, &format!("{q}/bins/23/onHours")),
        0.0,
    );
    close(
        "hyb f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.4317507389128007,
    );
    close(
        "hyb E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        377.95013039611615,
    );
    close(
        "hyb gas m1",
        at(&p, "/spaceHeating/monthly/0/naturalGasKwh"),
        38.78460227165539,
    );
}

/// Review 69. Annex Q, modulating outdoor-air/water heat pump with modulating condenser and source fan (Q.2.6): part-load
/// constants Q.71-Q.73 from the LT and MT rows of table Q.13 (f_cor;bu;max at plj = 2, Q.77), utilisation Q.28 and
/// interval table Q.2, COP Q.13-Q.16 with interpolation Q.14, P_pl Q.27, θ_evap;out Q.21, F_water Q.25, t_on Q.10;
/// monthly on-fraction Q.90 with Q.93 and table Q.17.
#[test]
fn annex_q_modulating_outdoor_air() {
    let p = performance("EPW001", ANNEX_Q_MODULATING);
    let q = "/spaceHeating/annexQ/annexQ";
    close("mod F", at(&p, &format!("{q}/energyFraction")), 0.99973692);
    close(
        "mod eta",
        at(&p, &format!("{q}/generationEfficiency")),
        4.2713332886620385,
    );
    close(
        "mod Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4670.470501957207,
    );
    close(
        "mod Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        1093.4455792421188,
    );
    close("mod W_aux", at(&p, &format!("{q}/sourcePumpKwh")), 0.0);
    close(
        "mod P_max bin 4",
        at(&p, &format!("{q}/bins/4/maximumPowerKw")),
        6.788095238095237,
    );
    close(
        "mod COP bin 4",
        at(&p, &format!("{q}/bins/4/cop")),
        6.666544973973457,
    );
    close(
        "mod on hours bin 4",
        at(&p, &format!("{q}/bins/4/onHours")),
        111.57139691904426,
    );
    close(
        "mod P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        6.165555555555555,
    );
    close(
        "mod COP bin 9",
        at(&p, &format!("{q}/bins/9/cop")),
        5.358773380986137,
    );
    close(
        "mod on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        307.40156660932934,
    );
    close(
        "mod P_max bin 14",
        at(&p, &format!("{q}/bins/14/maximumPowerKw")),
        5.545238095238095,
    );
    close(
        "mod COP bin 14",
        at(&p, &format!("{q}/bins/14/cop")),
        3.508704764912016,
    );
    close(
        "mod on hours bin 14",
        at(&p, &format!("{q}/bins/14/onHours")),
        236.0,
    );
    close(
        "mod P_max bin 19",
        at(&p, &format!("{q}/bins/19/maximumPowerKw")),
        4.922698412698413,
    );
    close(
        "mod COP bin 19",
        at(&p, &format!("{q}/bins/19/cop")),
        3.1536248758843524,
    );
    close(
        "mod on hours bin 19",
        at(&p, &format!("{q}/bins/19/onHours")),
        49.0,
    );
    close(
        "mod f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.8312059213840048,
    );
    close(
        "mod E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        343.68990134807154,
    );
}

/// Review 69. Annex Q, modulating brine/water heat pump with a modulating source pump: P_aux (P_pl/P_max)³ (Q.6) over
/// t_ev;on of Q.8 with t_on of Q.10, table Q.12 part-load rows.
#[test]
fn annex_q_modulating_brine_water() {
    let p = performance("EPW001", ANNEX_Q_MODULATING_BRINE);
    let q = "/spaceHeating/annexQ/annexQ";
    close(
        "modbw F",
        at(&p, &format!("{q}/energyFraction")),
        0.99973692,
    );
    close(
        "modbw eta",
        at(&p, &format!("{q}/generationEfficiency")),
        6.316718691709764,
    );
    close(
        "modbw Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4670.470501957207,
    );
    close(
        "modbw Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        739.3823802992622,
    );
    close(
        "modbw W_aux",
        at(&p, &format!("{q}/sourcePumpKwh")),
        2.007883476509976,
    );
    close(
        "modbw P_max bin 4",
        at(&p, &format!("{q}/bins/4/maximumPowerKw")),
        9.620222222222221,
    );
    close(
        "modbw COP bin 4",
        at(&p, &format!("{q}/bins/4/cop")),
        7.599217141162558,
    );
    close(
        "modbw on hours bin 4",
        at(&p, &format!("{q}/bins/4/onHours")),
        89.25711753523541,
    );
    close(
        "modbw P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        8.883555555555555,
    );
    close(
        "modbw COP bin 9",
        at(&p, &format!("{q}/bins/9/cop")),
        6.682721159509616,
    );
    close(
        "modbw on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        245.9212532874635,
    );
    close(
        "modbw P_max bin 14",
        at(&p, &format!("{q}/bins/14/maximumPowerKw")),
        8.612222222222222,
    );
    close(
        "modbw COP bin 14",
        at(&p, &format!("{q}/bins/14/cop")),
        6.045668226498378,
    );
    close(
        "modbw on hours bin 14",
        at(&p, &format!("{q}/bins/14/onHours")),
        198.18919108630226,
    );
    close(
        "modbw P_max bin 19",
        at(&p, &format!("{q}/bins/19/maximumPowerKw")),
        8.535555555555556,
    );
    close(
        "modbw COP bin 19",
        at(&p, &format!("{q}/bins/19/cop")),
        5.531252649247946,
    );
    close(
        "modbw on hours bin 19",
        at(&p, &format!("{q}/bins/19/onHours")),
        49.0,
    );
    close(
        "modbw f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.7333394725254121,
    );
    close(
        "modbw E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        232.40137613404443,
    );
    close(
        "modbw aux m1",
        at(&p, "/spaceHeating/monthly/0/auxiliaryElectricityKwh"),
        0.167323623042498,
    );
}

const ANNEX_Q_INTERVAL_5: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "outdoor_air_water", "maximumPower": {"condition1": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 30, "condenserOutC": 35, "cop": 4.6, "heatingPowerKw": 6.0}, "condition2": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 47, "condenserOutC": 55, "cop": 2.9, "heatingPowerKw": 5.6}, "condition3": {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 30, "condenserOutC": 35, "cop": 2.7, "heatingPowerKw": 4.4}, "condition4": {"evaporatorInC": 2, "evaporatorOutC": 1, "condenserInC": 30, "condenserOutC": 35, "cop": 3.3, "heatingPowerKw": 4.9}}, "modulation": {"method": "modulating", "minimumPowerKw": 0.5, "lowRange": [{"evaporatorInC": -10, "evaporatorOutC": -11, "condenserInC": 30, "condenserOutC": 35, "cop": 2.4, "heatingPowerKw": 4.0}, {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 29.5, "condenserOutC": 34, "cop": 2.9, "heatingPowerKw": 3.9}, {"evaporatorInC": 2, "evaporatorOutC": 1, "condenserInC": 26.5, "condenserOutC": 30, "cop": 4.3, "heatingPowerKw": 2.6}, {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 24.5, "condenserOutC": 27, "cop": 5.4, "heatingPowerKw": 1.75}, {"evaporatorInC": 12, "evaporatorOutC": 11, "condenserInC": 22.5, "condenserOutC": 24, "cop": 6.6, "heatingPowerKw": 0.8}], "highRange": [{"evaporatorInC": -10, "evaporatorOutC": -11, "condenserInC": 48, "condenserOutC": 55, "cop": 1.6, "heatingPowerKw": 3.6}, {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 46, "condenserOutC": 52, "cop": 1.9, "heatingPowerKw": 3.5}, {"evaporatorInC": 2, "evaporatorOutC": 1, "condenserInC": 38, "condenserOutC": 42, "cop": 2.9, "heatingPowerKw": 2.3}, {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 33, "condenserOutC": 36, "cop": 3.8, "heatingPowerKw": 1.5}, {"evaporatorInC": 12, "evaporatorOutC": 11, "condenserInC": 28, "condenserOutC": 30, "cop": 4.7, "heatingPowerKw": 0.7}], "condenserPumpModulating": true, "sourcePumpModulating": true}, "switchOff": {"minEvaporatorInC": -20.0}, "testReportReference": "r70 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r70 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r70 synthetic"}}]"#;

const ANNEX_Q_WATER_WATER_FORFAIT_PUMP: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "water_water", "maximumPower": {"condition1": {"evaporatorInC": 10, "evaporatorOutC": 7, "condenserInC": 30, "condenserOutC": 35, "cop": 5.6, "heatingPowerKw": 9.0}, "condition2": {"evaporatorInC": 10, "evaporatorOutC": 7, "condenserInC": 47, "condenserOutC": 55, "cop": 3.4, "heatingPowerKw": 8.3}, "condition3": {"evaporatorInC": 15, "evaporatorOutC": 12, "condenserInC": 30, "condenserOutC": 35, "cop": 6.3, "heatingPowerKw": 10.1}}, "modulation": {"method": "on_off"}, "switchOff": {}, "sourcePump": {"nominalPowerW": null, "overrunS": 60.0}, "testReportReference": "r70 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r70 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r70 synthetic"}}]"#;

const ANNEX_Q_MODULATING_AIR_AIR: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "air_air", "maximumPower": {"condition1": {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 20, "condenserOutC": 35, "cop": 4.0, "heatingPowerKw": 5.0}, "condition3": {"evaporatorInC": -7, "evaporatorOutC": -8, "condenserInC": 20, "condenserOutC": 32, "cop": 2.6, "heatingPowerKw": 3.6}}, "modulation": {"method": "modulating", "minimumPowerKw": 0.6, "lowRange": [{"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 20, "condenserOutC": 35, "cop": 4.0, "heatingPowerKw": 5.0}, {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 20, "condenserOutC": 33, "cop": 4.3, "heatingPowerKw": 4.4}, {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 20, "condenserOutC": 28, "cop": 5.2, "heatingPowerKw": 2.7}, {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 20, "condenserOutC": 25.5, "cop": 5.6, "heatingPowerKw": 1.75}, {"evaporatorInC": 7, "evaporatorOutC": 6, "condenserInC": 20, "condenserOutC": 22.5, "cop": 5.0, "heatingPowerKw": 0.75}], "highRange": null, "condenserPumpModulating": false, "sourcePumpModulating": false}, "switchOff": {"minEvaporatorInC": -15.0}, "testReportReference": "r70 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r70 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": false, "sourceReference": "r70 synthetic"}}, {"pointer": "/ntaCalculation/emission", "value": {"system": "air_heating", "balancing": "not_applicable", "control": "individual_room_thermostats", "sourceReference": "r70"}}, {"op": "remove", "pointer": "/ntaCalculation/distributionSystem"}, {"pointer": "/ntaCalculation/distribution", "value": {"method": "heated_zone_only_space_heating", "sourceReference": "r70"}}]"#;

const ANNEX_Q_OVERVENTILATION: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "exhaust_air_water", "maximumPower": {"condition1": {"evaporatorInC": 20, "evaporatorOutC": 12, "condenserInC": 30, "condenserOutC": 35, "cop": 4.2, "heatingPowerKw": 1.6}, "condition2": {"evaporatorInC": 20, "evaporatorOutC": 12, "condenserInC": 47, "condenserOutC": 55, "cop": 2.6, "heatingPowerKw": 1.35}}, "modulation": {"method": "on_off"}, "switchOff": {}, "testReportReference": "r70 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r70 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": true, "sourceReference": "r70 synthetic"}}, {"pointer": "/ntaCalculation/ventilation/overventilation", "value": {"heatingFlowM3PerH": 230.0, "hotWaterTimeFraction": [0.25, 0.25, 0.3, 0.3, 0.35, 0.35, 0.35, 0.35, 0.3, 0.3, 0.25, 0.4], "hotWaterFlowM3PerH": [250.0, 250.0, 250.0, 250.0, 250.0, 250.0, 250.0, 250.0, 250.0, 250.0, 250.0, 250.0], "sourceReference": "r70 synthetic"}}]"#;

const OVERVENTILATION_LOW_HOT_WATER_FLOW: &str = r#"[{"pointer": "/ntaCalculation/generator", "value": {"kind": "heat_pump_annex_q", "heatPump": {"source": "exhaust_air_water", "maximumPower": {"condition1": {"evaporatorInC": 20, "evaporatorOutC": 12, "condenserInC": 30, "condenserOutC": 35, "cop": 4.2, "heatingPowerKw": 1.6}, "condition2": {"evaporatorInC": 20, "evaporatorOutC": 12, "condenserInC": 47, "condenserOutC": 55, "cop": 2.6, "heatingPowerKw": 1.35}}, "modulation": {"method": "on_off"}, "switchOff": {}, "testReportReference": "r70 synthetic"}, "designSupplyTemperatureC": 35.0, "backup": {"kind": "electric_resistance"}, "equipmentReference": "r70 synthetic"}}, {"pointer": "/ntaCalculation/heatPumpRenewable", "value": {"sourceBelow20C": true, "exhaustAirSource": true, "sourceReference": "r70 synthetic"}}, {"pointer": "/ntaCalculation/ventilation/overventilation", "value": {"heatingFlowM3PerH": 200.0, "hotWaterTimeFraction": [0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3], "hotWaterFlowM3PerH": [100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0], "sourceReference": "r70 synthetic", "heatingTimeFraction": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]}}]"#;

/// Review 70. Annex Q, modulating outdoor-air/water heat pump with P_H;hp;min 0,5 kW, so B < 0,15 (table Q.2, interval 5)
/// in the mild bins: P_pl = P_min (Q.27), t_on of Q.10, COP of the 15 % row (Q.16 names plj = 5, which is not measured;
/// kernel reading plj = 4, p. 1034/1064).
#[test]
fn annex_q_modulating_in_interval_5() {
    let p = performance("EPW001", ANNEX_Q_INTERVAL_5);
    let q = "/spaceHeating/annexQ/annexQ";
    close("int5 F", at(&p, &format!("{q}/energyFraction")), 0.99973692);
    close(
        "int5 eta",
        at(&p, &format!("{q}/generationEfficiency")),
        4.258388858591388,
    );
    close(
        "int5 Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4670.470501957207,
    );
    close(
        "int5 Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        1096.7693785255037,
    );
    close("int5 W_aux", at(&p, &format!("{q}/sourcePumpKwh")), 0.0);
    close(
        "int5 P_max bin 0",
        at(&p, &format!("{q}/bins/0/maximumPowerKw")),
        7.245238095238094,
    );
    close(
        "int5 COP bin 0",
        at(&p, &format!("{q}/bins/0/cop")),
        7.164476639831449,
    );
    close(
        "int5 on hours bin 0",
        at(&p, &format!("{q}/bins/0/onHours")),
        0.0,
    );
    close(
        "int5 P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        6.165555555555555,
    );
    close(
        "int5 COP bin 9",
        at(&p, &format!("{q}/bins/9/cop")),
        5.3185460626253915,
    );
    close(
        "int5 on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        456.0,
    );
    close(
        "int5 P_max bin 10",
        at(&p, &format!("{q}/bins/10/maximumPowerKw")),
        6.0423809523809515,
    );
    close(
        "int5 COP bin 10",
        at(&p, &format!("{q}/bins/10/cop")),
        4.904903120074015,
    );
    close(
        "int5 on hours bin 10",
        at(&p, &format!("{q}/bins/10/onHours")),
        358.0,
    );
    close(
        "int5 P_max bin 26",
        at(&p, &format!("{q}/bins/26/maximumPowerKw")),
        4.051587301587301,
    );
    close(
        "int5 COP bin 26",
        at(&p, &format!("{q}/bins/26/cop")),
        0.7168202876551018,
    );
    close(
        "int5 on hours bin 26",
        at(&p, &format!("{q}/bins/26/onHours")),
        0.0,
    );
    close(
        "int5 f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.9886745088971003,
    );
    close(
        "int5 E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        344.73463212342403,
    );
}

/// Review 70. Annex Q, on/off water/water heat pump without a declared source-pump power: forfait 50 W per kW evaporator
/// power (Q.4.4, p. 1065), the evaporator power taken at condition 1 as P·(1 − 1/COP); source pump Q.5/Q.7/Q.8.
#[test]
fn annex_q_water_water_with_forfait_source_pump() {
    let p = performance("EPW001", ANNEX_Q_WATER_WATER_FORFAIT_PUMP);
    let q = "/spaceHeating/annexQ/annexQ";
    close("wwf F", at(&p, &format!("{q}/energyFraction")), 0.99973692);
    close(
        "wwf eta",
        at(&p, &format!("{q}/generationEfficiency")),
        5.0502793079953,
    );
    close(
        "wwf Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4670.470501957207,
    );
    close(
        "wwf Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        924.7944949427247,
    );
    close(
        "wwf W_aux",
        at(&p, &format!("{q}/sourcePumpKwh")),
        193.06423798522155,
    );
    close(
        "wwf P_max bin 6",
        at(&p, &format!("{q}/bins/6/maximumPowerKw")),
        9.340925925925932,
    );
    close(
        "wwf COP bin 6",
        at(&p, &format!("{q}/bins/6/cop")),
        6.704475335953742,
    );
    close(
        "wwf on hours bin 6",
        at(&p, &format!("{q}/bins/6/onHours")),
        24.505206375175167,
    );
    close(
        "wwf P_max bin 16",
        at(&p, &format!("{q}/bins/16/maximumPowerKw")),
        9.162037037037043,
    );
    close(
        "wwf COP bin 16",
        at(&p, &format!("{q}/bins/16/cop")),
        6.146302622462773,
    );
    close(
        "wwf on hours bin 16",
        at(&p, &format!("{q}/bins/16/onHours")),
        28.754790965890095,
    );
    close(
        "wwf P_max bin 26",
        at(&p, &format!("{q}/bins/26/maximumPowerKw")),
        8.987037037037044,
    );
    close(
        "wwf COP bin 26",
        at(&p, &format!("{q}/bins/26/cop")),
        5.60026409839552,
    );
    close(
        "wwf on hours bin 26",
        at(&p, &format!("{q}/bins/26/onHours")),
        0.0,
    );
    close(
        "wwf f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.12877037123645274,
    );
    close(
        "wwf E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        290.679787607176,
    );
    close(
        "wwf aux m1",
        at(&p, "/spaceHeating/monthly/0/auxiliaryElectricityKwh"),
        16.088686498768464,
    );
}

/// Review 70. Annex Q, modulating air/air heat pump: part-load constants Q.74-Q.78 from the t1 rows, θ_cond;out;pl of
/// Q.24A with a condenser inlet of 20 °C, θ_evap;out;pl of Q.22 (on/off fan), and the Carnot cap Q.12A/Q.12B with
/// F_C;max of Q.73A/Q.73B (p. 1034-1037, 1053-1054); the cap applies in most bins below 8 °C.
#[test]
fn annex_q_modulating_air_air_with_carnot_cap() {
    let p = performance("EPW001", ANNEX_Q_MODULATING_AIR_AIR);
    let q = "/spaceHeating/annexQ/annexQ";
    close(
        "aamod F",
        at(&p, &format!("{q}/energyFraction")),
        0.9997369200000001,
    );
    close(
        "aamod eta",
        at(&p, &format!("{q}/generationEfficiency")),
        4.176822441943842,
    );
    close(
        "aamod Q_pr",
        at(&p, &format!("{q}/deliveredKwh")),
        4479.338322845153,
    );
    close(
        "aamod Q_el",
        at(&p, &format!("{q}/electricityKwh")),
        1072.4272781776483,
    );
    close("aamod W_aux", at(&p, &format!("{q}/sourcePumpKwh")), 0.0);
    close(
        "aamod P_max bin 0",
        at(&p, &format!("{q}/bins/0/maximumPowerKw")),
        5.9,
    );
    close(
        "aamod COP bin 0",
        at(&p, &format!("{q}/bins/0/cop")),
        5.944915254237288,
    );
    close(
        "aamod on hours bin 0",
        at(&p, &format!("{q}/bins/0/onHours")),
        0.0,
    );
    close(
        "aamod P_max bin 9",
        at(&p, &format!("{q}/bins/9/maximumPowerKw")),
        5.0,
    );
    close(
        "aamod COP bin 9",
        at(&p, &format!("{q}/bins/9/cop")),
        4.929170200918047,
    );
    close(
        "aamod on hours bin 9",
        at(&p, &format!("{q}/bins/9/onHours")),
        455.99999999999994,
    );
    close(
        "aamod P_max bin 13",
        at(&p, &format!("{q}/bins/13/maximumPowerKw")),
        4.6000000000000005,
    );
    close(
        "aamod COP bin 13",
        at(&p, &format!("{q}/bins/13/cop")),
        3.9351347538786636,
    );
    close(
        "aamod on hours bin 13",
        at(&p, &format!("{q}/bins/13/onHours")),
        304.0,
    );
    close(
        "aamod P_max bin 17",
        at(&p, &format!("{q}/bins/17/maximumPowerKw")),
        4.2,
    );
    close(
        "aamod COP bin 17",
        at(&p, &format!("{q}/bins/17/cop")),
        3.3746613921329813,
    );
    close(
        "aamod on hours bin 17",
        at(&p, &format!("{q}/bins/17/onHours")),
        142.0,
    );
    close(
        "aamod P_max bin 25",
        at(&p, &format!("{q}/bins/25/maximumPowerKw")),
        3.4000000000000004,
    );
    close(
        "aamod COP bin 25",
        at(&p, &format!("{q}/bins/25/cop")),
        2.5621002377495037,
    );
    close(
        "aamod on hours bin 25",
        at(&p, &format!("{q}/bins/25/onHours")),
        4.0,
    );
    close(
        "aamod f_on m1",
        at(&p, &format!("{q}/monthlyOnFraction/0")),
        0.9831572570666868,
    );
    close(
        "aamod E_H;el m1",
        at(&p, "/spaceHeating/monthly/0/generatorElectricityKwh"),
        337.0157329496474,
    );
}

/// Review 70. Exhaust-air heat pump with overventilation for heating (q_V;hp;H 230 m³/h) next to a hot-water fraction
/// and flow: the Q.5.3 recalculation (steps 1-3, Q.84, Q.90/Q.91 with table Q.17, Q.96/Q.97, p. 1065-1072) and the
/// outdoor-air requirement of 11.23/11.23a (p. 449-450) per month. Where Q_H;node,in(2) is 0, f(3) = f(2).
#[test]
fn annex_q_exhaust_air_overventilation_recalculation() {
    let p = performance("EPW001", ANNEX_Q_OVERVENTILATION);
    close(
        "ov f3 m1",
        at(&p, "/spaceHeating/annexQ/exhaustAirHeatingTimeFraction/0"),
        0.7156240408303428,
    );
    close(
        "ov q_ODA m1",
        at(
            &p,
            "/spaceHeating/demand/ventilation/months/0/heating/requiredOutdoorAirM3PerH",
        ),
        232.34588647034104,
    );
    close(
        "ov f3 m2",
        at(&p, "/spaceHeating/annexQ/exhaustAirHeatingTimeFraction/1"),
        0.6219903945707567,
    );
    close(
        "ov q_ODA m2",
        at(
            &p,
            "/spaceHeating/demand/ventilation/months/1/heating/requiredOutdoorAirM3PerH",
        ),
        225.11658048523776,
    );
    close(
        "ov f3 m4",
        at(&p, "/spaceHeating/annexQ/exhaustAirHeatingTimeFraction/3"),
        0.3980582624956521,
    );
    close(
        "ov q_ODA m4",
        at(
            &p,
            "/spaceHeating/demand/ventilation/months/3/heating/requiredOutdoorAirM3PerH",
        ),
        212.68755519740117,
    );
    close(
        "ov f3 m7",
        at(&p, "/spaceHeating/annexQ/exhaustAirHeatingTimeFraction/6"),
        0.0464056123678672,
    );
    close(
        "ov q_ODA m7",
        at(
            &p,
            "/spaceHeating/demand/ventilation/months/6/heating/requiredOutdoorAirM3PerH",
        ),
        190.397430374693,
    );
    close(
        "ov f3 m11",
        at(&p, "/spaceHeating/annexQ/exhaustAirHeatingTimeFraction/10"),
        0.500631368054177,
    );
    close(
        "ov q_ODA m11",
        at(
            &p,
            "/spaceHeating/demand/ventilation/months/10/heating/requiredOutdoorAirM3PerH",
        ),
        215.7466416674082,
    );
    close(
        "ov f3 m12",
        at(&p, "/spaceHeating/annexQ/exhaustAirHeatingTimeFraction/11"),
        0.6,
    );
    close(
        "ov q_ODA m12",
        at(
            &p,
            "/spaceHeating/demand/ventilation/months/11/heating/requiredOutdoorAirM3PerH",
        ),
        238.0,
    );
}

/// Review 70. 11.23 takes q_V;hp;W;zi,mi of 13.8.2.4, which 13.148/13.148a never set below q_V;ODA;req of 11.22
/// (p. 449, 615): an entered hot-water flow of 100 m³/h below the 152,8 m³/h requirement may not lower the outdoor air.
#[test]
fn an_entered_hot_water_flow_below_the_requirement_does_not_lower_the_outdoor_air() {
    let p = performance("EPW001", OVERVENTILATION_LOW_HOT_WATER_FLOW);
    close(
        "low q_W m1 heating",
        at(
            &p,
            "/spaceHeating/demand/ventilation/months/0/heating/requiredOutdoorAirM3PerH",
        ),
        152.79157894736846,
    );
    close(
        "low q_W m7 heating",
        at(
            &p,
            "/spaceHeating/demand/ventilation/months/6/heating/requiredOutdoorAirM3PerH",
        ),
        152.79157894736846,
    );
}
