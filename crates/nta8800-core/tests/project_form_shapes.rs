//! The JSON the project forms added for form coverage write
//! (docs/ui-redesign/formulierdekking.md; NtaZoneData.tsx,
//! NtaCollectiveShare.tsx, the circulation and further-generator forms of
//! NtaPipeGeometry.tsx / NtaSystemSections.tsx) must be read and calculated
//! by the kernel. Each patch mirrors a form template; src/__tests__/
//! nta-form-coverage-project.test.tsx checks that the forms write it.

use nta8800_core::project_performance::assess_project_performance;
use serde_json::{json, Value};

fn office() -> Value {
    serde_json::from_str(include_str!(
        "../../../training-data/nta8800-example-office.json"
    ))
    .unwrap()
}

fn gap_codes(value: &Value) -> (String, Vec<String>) {
    let result = assess_project_performance(value);
    let codes = result
        .gaps
        .iter()
        .map(|gap| format!("{} @ {}", gap.code, gap.path))
        .collect();
    (result.status.to_string(), codes)
}

/// The circulation template of `circulationTemplate()` with its required source filled in.
fn circulation() -> Value {
    json!({ "insulation": "mm15", "fittingsInsulated": true, "floorCount": 2, "lengthM": 30.0,
        "pump": { "control": "constant_pressure" }, "sourceReference": "tekening W-01" })
}

#[test]
fn zone_data_of_the_zone_editor_is_calculated() {
    let base = office();
    let (status, before) = gap_codes(&base);
    let mut value = base;
    // `zoneDataTemplate()` copies the block's internal gains, flows and ventilation.
    let gains = value["ntaCalculation"]["internalGains"].clone();
    let flows = value["ntaCalculation"]["ventilationFlows"].clone();
    let ventilation = value["ntaCalculation"]["ventilation"].clone();
    value["ntaCalculation"]["zoneData"] = json!([{
        "zoneId": "z1",
        "internalGains": gains,
        "ventilationFlows": flows,
        "ventilation": ventilation,
        "usageFunction": "office",
        "functionAreas": [{ "function": "office", "areaM2": 300.0 }, { "function": "retail", "areaM2": 100.0 }],
        "thermalMass": { "floor": "heavy", "wall": "very_heavy", "ceiling": "open_or_none", "sourceReference": "bestek" },
        "sunrooms": [],
    }]);
    let (after_status, after) = gap_codes(&value);
    assert_eq!(after_status, status, "{after:?}");
    assert_eq!(after, before);
}

#[test]
fn circulation_and_further_generators_are_calculated() {
    let base = office();
    let (status, before) = gap_codes(&base);
    let mut value = base;
    value["ntaCalculation"]["hotWater"]["circulation"] = circulation();
    let mut further = value["ntaCalculation"]["hotWater"].clone();
    further["need"]["areas"][0]["areaM2"] = json!(100.0);
    further["circulation"] = circulation();
    further["nominalPowerKw"] = json!(3.0);
    further["additionalGenerators"] = json!([{
        "generator": { "kind": "electric_boiler" }, "nominalPowerKw": 2.0, "equipmentReference": "typeplaat",
    }]);
    value["ntaCalculation"]["hotWater"]["need"]["areas"][0]["areaM2"] = json!(300.0);
    value["ntaCalculation"]["additionalHotWaterSystems"] = json!([further]);
    let (after_status, after) = gap_codes(&value);
    assert_eq!(after_status, status, "{after:?}");
    for gap in &after {
        assert!(before.contains(gap), "new gap {gap} in {after:?}");
    }
}

#[test]
fn collective_pv_swimming_pool_and_lighting_zone_are_read() {
    let base = office();
    let (status, before) = gap_codes(&base);
    let mut value = base;
    value["ntaCalculation"]["pvSystems"][0]["collective"] =
        json!({ "buildingUsableFloorAreaM2": 1200.0, "sourceReference": "NEN 2580 gebouw" });
    value["ntaCalculation"]["lighting"][0]["zoneId"] = json!("z1");
    let (after_status, after) = gap_codes(&value);
    assert_eq!(after_status, status, "{after:?}");
    assert_eq!(after, before);

    // The swimming-pool check only exists for a sport function (§11.2.2.5.1).
    let mut sport = value.clone();
    sport["ntaCalculation"]["ventilation"]["functions"][0] =
        json!({ "function": "sport", "areaM2": 400.0, "swimmingPool": true });
    let (_, codes) = gap_codes(&sport);
    assert!(
        !codes
            .iter()
            .any(|code| code.starts_with("swimming_pool_requires_sport_function")),
        "{codes:?}"
    );
}

/// The office with a second calculation zone of 200 m² (no windows).
fn two_zone_office() -> Value {
    let mut value = office();
    let mut z2 = value["zones"][0].clone();
    z2["id"] = json!("z2");
    z2["floorArea"] = json!(200.0);
    z2["volume"] = json!(700.0);
    let mut wall = z2["surfaces"][0].clone();
    wall["id"] = json!("wall-N2");
    wall["zoneId"] = json!("z2");
    wall["windows"] = json!([]);
    z2["surfaces"] = json!([wall]);
    value["zones"].as_array_mut().unwrap().push(z2);
    value["ntaCalculation"]
        .as_object_mut()
        .unwrap()
        .remove("verticalPipes");
    value
}

/// `zoneDataTemplate(draft, zone)`: the block's gains and flows, and its
/// chapter 11 ventilation with this zone's id and floor area (one function
/// gets the zone area).
fn zone_template(value: &Value, zone: &str, area: f64) -> Value {
    let n = &value["ntaCalculation"];
    let mut out = json!({ "zoneId": zone, "internalGains": n["internalGains"].clone() });
    if n["ventilationFlows"].is_array() {
        out["ventilationFlows"] = n["ventilationFlows"].clone();
    }
    if !n["ventilation"].is_null() {
        let mut ventilation = n["ventilation"].clone();
        ventilation["zoneId"] = json!(zone);
        ventilation["usableFloorAreaM2"] = json!(area);
        if let Some(functions) = ventilation["functions"].as_array_mut() {
            for item in functions {
                item["areaM2"] = json!(area);
            }
        }
        out["ventilation"] = ventilation;
    }
    out
}

#[test]
fn zone_data_copies_get_the_zone_area() {
    let value = two_zone_office();
    let z1_area = value["zones"][0]["floorArea"].as_f64().unwrap();
    let mut with = value.clone();
    with["ntaCalculation"]["zoneData"] = json!([
        zone_template(&value, "z1", z1_area),
        zone_template(&value, "z2", 200.0)
    ]);
    let (_, gaps) = gap_codes(&with);
    for gap in &gaps {
        assert!(
            !gap.starts_with("ventilation_area_mismatch")
                && !gap.starts_with("nta_calculation_block_invalid"),
            "{gaps:?}"
        );
    }
}

#[test]
fn zone_usage_switch_leaves_no_stale_dwelling_type() {
    let house: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-example-terraced-dwelling.json"
    ))
    .unwrap();
    let area = house["zones"][0]["floorArea"].as_f64().unwrap();
    let mut base = house.clone();
    base["ntaCalculation"]["zoneData"] = json!([zone_template(&house, "z1", area)]);
    let (status, before) = gap_codes(&base);
    // Own residential use: the form prefills the block's dwelling type.
    let mut own = base.clone();
    own["ntaCalculation"]["zoneData"][0]["usageFunction"] = json!("residential");
    own["ntaCalculation"]["zoneData"][0]["dwellingType"] =
        house["ntaCalculation"]["dwellingType"].clone();
    let (own_status, after) = gap_codes(&own);
    assert_eq!(own_status, status, "{after:?}");
    assert!(
        after
            .iter()
            .all(|gap| !gap.starts_with("dwelling_type_") && before.contains(gap)),
        "{after:?}"
    );
}

#[test]
fn a_second_further_generator_ends_the_series() {
    let base = office();
    let (status, _) = gap_codes(&base);
    let mut value = base;
    value["ntaCalculation"]["hotWater"]["nominalPowerKw"] = json!(3.0);
    value["ntaCalculation"]["hotWater"]["additionalGenerators"] = json!([
        { "generator": { "kind": "electric_boiler" }, "nominalPowerKw": 2.0, "equipmentReference": "x" },
        { "generator": { "kind": "electric_boiler" }, "nominalPowerKw": 2.0, "equipmentReference": "y" }
    ]);
    // The add handler clears the series (`series: null`).
    value["ntaCalculation"]["hotWater"]["series"] = Value::Null;
    let (after_status, after) = gap_codes(&value);
    assert_eq!(after_status, status, "{after:?}");
    assert!(
        after
            .iter()
            .all(|gap| !gap.starts_with("hot_water_series_requires_two_generators")),
        "{after:?}"
    );
}

/// Review 9: a pipe added on the residential route keeps a blank `storeys`
/// when the office takes the utility route. A null member reads as absent,
/// so with H filled in the pipe calculates and nothing points at the hidden
/// storeys field.
#[test]
fn hidden_blank_storeys_on_the_utility_route_reads_as_absent() {
    let base = office();
    let (status, before) = gap_codes(&base);
    let mut value = base;
    value["ntaCalculation"]["verticalPipes"][0]["storeys"] = Value::Null;
    let (after_status, after) = gap_codes(&value);
    assert_eq!(after_status, status);
    assert_eq!(after, before);
    assert!(
        !after.iter().any(|gap| gap.contains("storeys")),
        "{after:?}"
    );
}

/// Review 9: the form syncs a utility pipe to `buildingHeightM` (blank when
/// not yet entered) and drops `storeys`, so the gap lands on the shown field.
#[test]
fn blank_building_height_is_named_on_the_utility_route() {
    let mut value = office();
    value["ntaCalculation"]["verticalPipes"][0]["buildingHeightM"] = Value::Null;
    let (_, gaps) = gap_codes(&value);
    assert!(
        gaps.iter()
            .any(|gap| gap.ends_with("verticalPipes[0].buildingHeightM")),
        "{gaps:?}"
    );
    assert!(!gaps.iter().any(|gap| gap.contains("storeys")), "{gaps:?}");
}
