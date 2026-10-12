//! Review 55: project inputs that carry the routes added in reviews 40–54,
//! so the robustness fuzz (`robustness.rs`), the numeric fuzz
//! (`numeric_fuzz.rs`) and the option sweep (`option_coverage.rs`) reach
//! their members. Each variant is a fixture with the smallest edits that
//! put the route in place; `new_input_fixtures_calculate` checks they
//! calculate as given.

use serde_json::{json, Value};

fn fixture(json: &str) -> Value {
    serde_json::from_str(json).expect("fixture")
}

fn synthetic() -> Value {
    fixture(include_str!(
        "../../../../training-data/nta8800-project-performance-synthetic.json"
    ))
}

/// The synthetic dwelling with a second zone; both border sunroom
/// "atrium" with their own H_zi;ztu and F, so the kernel derives the 7.38
/// bordering-zone list (review 46, shared-sunroom fix, review 54).
pub fn shared_sunroom() -> Value {
    let mut value = synthetic();
    let mut second = value["zones"][0].clone();
    second["id"] = Value::from("z2");
    second["floorArea"] = Value::from(50.0);
    for surface in second["surfaces"].as_array_mut().unwrap() {
        let id = surface["id"].as_str().unwrap().to_owned();
        surface["id"] = Value::from(format!("{id}-2"));
        surface["zoneId"] = Value::from("z2");
        for window in surface["windows"].as_array_mut().unwrap() {
            let window_id = window["id"].as_str().unwrap().to_owned();
            window["id"] = Value::from(format!("{window_id}-2"));
        }
    }
    for bridge in second["thermalBridges"].as_array_mut().unwrap() {
        let id = bridge["id"].as_str().unwrap().to_owned();
        bridge["id"] = Value::from(format!("{id}-2"));
        bridge["zoneId"] = Value::from("z2");
    }
    value["zones"].as_array_mut().unwrap().push(second);
    let sunroom = |h: f64, f: f64| {
        json!([{
            "id": "atrium", "glazingGHeating": 0.55, "glazingGCooling": 0.35,
            "exteriorFrameFraction": 0.25, "reductionFactor": 0.7,
            "zoneConductanceWPerK": h, "distributionFactor": f,
            "surfaces": [
                {"areaM2": 60.0, "absorptance": 0.7, "azimuthDeg": 180.0, "tiltDeg": 0.0}
            ],
            "sourceReference": "review 55 fuzz input"
        }])
    };
    let block = &mut value["ntaCalculation"];
    let entry = |id: &str, h: f64, f: f64, block: &Value| {
        json!({
            "zoneId": id,
            "verticalPipes": [],
            "ventilationFlows": block["ventilationFlows"].clone(),
            "internalGains": block["internalGains"].clone(),
            "sunrooms": sunroom(h, f)
        })
    };
    let entries = vec![entry("z1", 6.0, 0.3, block), entry("z2", 14.0, 0.7, block)];
    block["zoneData"] = Value::from(entries);
    let tilts = block["surfaceTilts"].as_array_mut().unwrap();
    let copies: Vec<Value> = tilts
        .iter()
        .map(|tilt| {
            let mut copy = tilt.clone();
            let id = copy["surfaceId"].as_str().unwrap().to_owned();
            copy["surfaceId"] = Value::from(format!("{id}-2"));
            copy
        })
        .collect();
    tilts.extend(copies);
    let mut floor = block["groundFloors"][0].clone();
    floor["surfaceId"] = Value::from("floor-2");
    block["groundFloors"].as_array_mut().unwrap().push(floor);
    value
}

/// The synthetic dwelling on a collective installation for 600 m² with a
/// hot-water delivery set (review 42: L_zi and the combined row of table
/// 9.16; table 9.25 note b: the boiler of a system over 500 m² takes the
/// outside row) and two identical appliances.
pub fn collective_delivery_set() -> Value {
    let mut value = synthetic();
    let block = &mut value["ntaCalculation"];
    block["distribution"] = json!({
        "method": "calculated", "sourceReference": "review 55 fuzz input"
    });
    block["distributionSystem"] = json!({
        "designTemperatureClass": "70_60",
        "installation": "collective",
        "usageFunction": "residential",
        "collectiveHotWaterDeliverySet": true,
        "connectedStoreys": 5,
        "pipeTransmittance": {
            "method": "forfait",
            "insulation": {"state": "insulated", "period": "from1995"}
        },
        "valvesInsulated": true,
        "actualPipeLengthM": 100.0,
        "unheatedPipeLengthM": 0.0,
        "pump": {"method": "calculated", "heatMeterPresent": true,
            "sourceReference": "review 55 fuzz input"},
        "sourceReference": "review 55 fuzz input"
    });
    block["collectiveConnection"] = json!({
        "connectedUsableAreaM2": 1200.0, "sourceReference": "review 55 fuzz input"
    });
    block["identicalSystems"] = Value::from(2);
    value
}

/// The synthetic dwelling as an upper-floor dwelling without a floor on the
/// ground, on the building's forfait thermal-bridge route (review 44).
pub fn forfait_bridges_upper_floor() -> Value {
    let mut value = synthetic();
    value["zones"][0]["surfaces"][5]["thermalBoundary"] = Value::from("adjacent_conditioned");
    value["zones"][0]["thermalBridges"] = json!([]);
    value["ntaCalculation"]["groundFloors"] = json!([]);
    value["ntaCalculation"]["thermalBridgeRoute"] = Value::from("forfait");
    value
}

/// The office with its boiler serving 800 m² (one appliance for more than
/// 500 m², review 42) and an oil-fired steam humidifier (review 47).
pub fn office_steam_oil_large_boiler() -> Value {
    let mut office = fixture(include_str!(
        "../../../../training-data/nta8800-example-office.json"
    ));
    office["ntaCalculation"]["collectiveConnection"] = json!({
        "connectedUsableAreaM2": 800.0, "sourceReference": "review 55 fuzz input"
    });
    office["ntaCalculation"]["humidifiers"] = json!([{
        "zoneId": "z1",
        "humidification": {
            "humidifier": {"kind": "steam", "carrier": "oil"},
            "rotaryWheel": false,
            "equipmentReference": "review 55 fuzz input"
        }
    }]);
    office
}

/// Case G (2024) with TOjuli method 3 on shaded glazing: every window has
/// an external screen of table 7.5, which counts in 2024 only (review 40).
pub fn solar_limitation_screens_2024() -> Value {
    let mut value = fixture(include_str!(
        "../../../../training-data/nta8800-public-comparison-g.json"
    ));
    let windows: Vec<String> = value["zones"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|zone| zone["surfaces"].as_array().unwrap().iter())
        .flat_map(|surface| surface["windows"].as_array().unwrap().iter())
        .map(|window| window["id"].as_str().unwrap().to_owned())
        .collect();
    let block = &mut value["ntaCalculation"];
    block["normVersion"] = Value::from("2024");
    block["windowShadings"] = Value::from(
        windows
            .iter()
            .map(|id| {
                json!({
                    "windowId": id,
                    "movableShading": {
                        "device": {"kind": "external_screen", "colour": "dark"},
                        "control": "manual_residential",
                        "sourceReference": "review 55 fuzz input"
                    },
                    "sourceReference": "review 55 fuzz input"
                })
            })
            .collect::<Vec<_>>(),
    );
    block["activeCooling"]["capacity"] = json!({
        "method": "solar_limitation", "criterion": "shaded_glazing",
        "sourceReference": "review 55 fuzz input"
    });
    value
}

/// Every review 55 project input with a name.
pub fn new_input_projects() -> Vec<(&'static str, Value)> {
    vec![
        ("synthetic (shared sunroom, two zones)", shared_sunroom()),
        (
            "synthetic (collective, delivery set, 1200 m²)",
            collective_delivery_set(),
        ),
        (
            "synthetic (upper floor, forfait bridges)",
            forfait_bridges_upper_floor(),
        ),
        (
            "example-office (800 m² boiler, oil steam)",
            office_steam_oil_large_boiler(),
        ),
        (
            "public-comparison-g (2024 TOjuli method 3, screens)",
            solar_limitation_screens_2024(),
        ),
        (
            "public-comparison-n (heated basement, wall parts)",
            fixture(include_str!(
                "../../../../training-data/nta8800-public-comparison-n.json"
            )),
        ),
    ]
}

/// Layered constructions with a weakly ventilated narrow cavity (F.2,
/// review 51) for the envelope route.
pub fn narrow_cavity_layer() -> Value {
    json!({
        "kind": "narrow_cavity", "thicknessMm": 10.0, "widthMm": 5.0, "weaklyVentilated": true
    })
}
