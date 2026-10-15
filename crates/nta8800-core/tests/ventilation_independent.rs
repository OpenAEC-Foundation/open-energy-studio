//! Review 65: chapter 11 routes the two example projects do not reach, recalculated independently
//! from NTA 8800:2025+C1:2026 (ventilative cooling 11.70-11.79, open combustion appliances 11.2.4,
//! AHU coils 11.3.2 with c_a as printed, airflow zones for H >= 15 m, the crawlspace rows of
//! table 11.1, grille preheating 11.123-11.128 and frost protection in dwellings). The expected
//! values come from a separate implementation written from the norm pages; only numbers and page
//! references are kept here.

use nta8800_core::ventilation::{assess_ventilation, VentilationInput};
use serde_json::Value;

fn actual(input: &str) -> Value {
    let input: VentilationInput = serde_json::from_str(input).expect("valid ventilation input");
    let assessment = assess_ventilation(&input);
    assert_eq!(
        assessment.status, "calculated_unverified",
        "{:?}",
        assessment.issues
    );
    serde_json::to_value(assessment.actual.expect("actual run")).expect("serializable")
}

fn close(label: &str, kernel: &Value, expected: f64) {
    let kernel = kernel
        .as_f64()
        .unwrap_or_else(|| panic!("{label}: missing"));
    let diff = (kernel - expected).abs();
    let scale = kernel.abs().max(expected.abs()).max(1e-12);
    assert!(
        diff <= 1e-9 || diff / scale <= 1e-6,
        "{label}: kernel {kernel} independent {expected}"
    );
}

/// Cross ventilation, openings SE (alpha 315), E (alpha 270) and N (alpha 180) on the half-open sector boundaries of 11.77 (p. 480).
#[test]
fn ventilative_cooling_cross_on_sector_boundaries() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 9.0, "category": "residential", "combustionAppliances": [], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": {"openings": [{"id": "raam-zo", "area": {"method": "declared", "netAreaM2": 0.4}, "centreHeightM": 1.8, "openingHeightM": 1.2, "azimuthDeg": 135.0, "tiltDeg": 90.0}, {"id": "raam-o", "area": {"method": "declared", "netAreaM2": 0.3}, "centreHeightM": 2.0, "openingHeightM": 0.6, "azimuthDeg": 90.0, "tiltDeg": 90.0}, {"id": "raam-n", "area": {"method": "discharge", "grossAreaM2": 0.9, "dischargeCoefficient": 0.6, "entryLossCoefficient": 0.5}, "centreHeightM": 1.6, "openingHeightM": 1.1, "azimuthDeg": 0.0, "tiltDeg": 90.0}], "operation": "automatic", "conditionsEvidence": "r65 synthetic"}, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "argII month 5",
        &a["months"][4]["cooling"]["ventilativeCoolingM3PerH"],
        358.1017505548475,
    );
    close(
        "argII month 7",
        &a["months"][6]["cooling"]["ventilativeCoolingM3PerH"],
        560.4869143210805,
    );
    close(
        "pzC zone 1 month 7",
        &a["months"][6]["coolingReferencePressurePa"][0],
        -2.4888240340011123,
    );
    assert_eq!(
        a["months"][6]["coolingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    close(
        "wcondC month 7",
        &a["months"][6]["cooling"]["weightedConductanceWPerK"],
        319.15050777261877,
    );
}

/// Facade + 45 degree roof opening (cross, p. 476); stack term wins in July, 11.75/11.76 as printed; roof opening with the 0,3 screen factor (p. 477).
#[test]
fn ventilative_cooling_cross_roof_stack() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 9.0, "category": "residential", "combustionAppliances": [], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": {"openings": [{"id": "raam-n", "area": {"method": "discharge", "grossAreaM2": 0.9, "dischargeCoefficient": 0.6, "entryLossCoefficient": 0.5}, "centreHeightM": 1.6, "openingHeightM": 1.1, "azimuthDeg": 0.0, "tiltDeg": 90.0}, {"id": "dakraam", "area": {"method": "opening_angle", "maxNetAreaM2": 0.6, "maxAngleDeg": 30.0, "screenUnspecified": true}, "centreHeightM": 7.8, "openingHeightM": 0.9, "azimuthDeg": 180.0, "tiltDeg": 45.0}], "operation": "manual", "conditionsEvidence": "r65 synthetic"}, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "argII month 7",
        &a["months"][6]["cooling"]["ventilativeCoolingM3PerH"],
        173.8933700317317,
    );
    close(
        "argII month 10",
        &a["months"][9]["cooling"]["ventilativeCoolingM3PerH"],
        65.02715079463562,
    );
    close(
        "pzC zone 1 month 7",
        &a["months"][6]["coolingReferencePressurePa"][0],
        -2.4497615340011123,
    );
    assert_eq!(
        a["months"][6]["coolingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    close(
        "wcondC month 7",
        &a["months"][6]["cooling"]["weightedConductanceWPerK"],
        176.3760271235305,
    );
}

/// Single-sided ventilation 11.73/11.74 with h_w;st of 11.70 and R_w;arg of note 6 (p. 476-478).
#[test]
fn ventilative_cooling_single_sided() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 9.0, "category": "residential", "combustionAppliances": [], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": {"openings": [{"id": "raam-z", "area": {"method": "declared", "netAreaM2": 0.5}, "centreHeightM": 1.5, "openingHeightM": 1.0, "azimuthDeg": 180.0, "tiltDeg": 90.0}, {"id": "raam-z2", "area": {"method": "opening_angle", "maxNetAreaM2": 0.8, "maxAngleDeg": 25.0}, "centreHeightM": 4.2, "openingHeightM": 0.8, "azimuthDeg": 180.0, "tiltDeg": 90.0}], "operation": "manual", "conditionsEvidence": "r65 synthetic"}, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "argII month 5",
        &a["months"][4]["cooling"]["ventilativeCoolingM3PerH"],
        83.3730375404344,
    );
    close(
        "argII month 7",
        &a["months"][6]["cooling"]["ventilativeCoolingM3PerH"],
        136.35563947531531,
    );
    close(
        "pzC zone 1 month 7",
        &a["months"][6]["coolingReferencePressurePa"][0],
        -2.4888240340011123,
    );
    assert_eq!(
        a["months"][6]["coolingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
}

/// Tables 11.11/11.12 (p. 483-484), 11.80-11.83: flueless geyser in, flued boiler out, room-sealed boiler ignored.
#[test]
fn open_combustion_flueless_flued_and_sealed() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 9.0, "category": "residential", "combustionAppliances": [{"id": "geiser", "kind": "gas_geyser", "class": "gas_type_a", "nominalInputKw": 20.0, "sourceReference": "r65 synthetic"}, {"id": "ketel", "kind": "gas_central_heating_boiler", "class": "gas_type_b", "nominalInputKw": null, "sourceReference": "r65 synthetic"}, {"id": "hr", "kind": "gas_central_heating_boiler", "class": "room_sealed", "nominalInputKw": 24.0, "sourceReference": "r65 synthetic"}], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "combH month 1",
        &a["months"][0]["heating"]["combustionM3PerH"],
        5.963149192907182,
    );
    close(
        "combC month 1",
        &a["months"][0]["cooling"]["combustionM3PerH"],
        5.862030880750253,
    );
    close(
        "pzH zone 1 month 1",
        &a["months"][0]["heatingReferencePressurePa"][0],
        -6.058474359058096,
    );
    assert_eq!(
        a["months"][0]["heatingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    close(
        "leaH month 1",
        &a["months"][0]["heating"]["infiltrationM3PerH"],
        32.117712913442666,
    );
    close(
        "wcondH month 1",
        &a["months"][0]["heating"]["weightedConductanceWPerK"],
        77.9556673391396,
    );
}

/// H = 60 m: three airflow zones of table 11.1 (p. 433-436), 11.8-11.10 split, table 11.3 per path height (p. 440).
#[test]
fn open_combustion_in_three_airflow_zones() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 60.0, "category": "residential", "combustionAppliances": [{"id": "sfeer", "kind": "gas_open_atmospheric", "class": "gas_type_a", "nominalInputKw": null, "sourceReference": "r65 synthetic"}, {"id": "bad", "kind": "gas_bath_geyser", "class": "gas_type_b", "nominalInputKw": 40.0, "sourceReference": "r65 synthetic"}], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "combH month 1",
        &a["months"][0]["heating"]["combustionM3PerH"],
        12.772753605507932,
    );
    close(
        "pzH zone 1 month 1",
        &a["months"][0]["heatingReferencePressurePa"][0],
        -7.990166731069252,
    );
    close(
        "pzH zone 2 month 1",
        &a["months"][0]["heatingReferencePressurePa"][1],
        -26.263621612241877,
    );
    close(
        "pzH zone 3 month 1",
        &a["months"][0]["heatingReferencePressurePa"][2],
        -43.10164683756034,
    );
    assert_eq!(
        a["months"][0]["heatingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
    close(
        "leaH month 1",
        &a["months"][0]["heating"]["infiltrationM3PerH"],
        44.09682128769512,
    );
    close(
        "pzC zone 1 month 7",
        &a["months"][6]["coolingReferencePressurePa"][0],
        -3.1562117486744654,
    );
    close(
        "pzC zone 2 month 7",
        &a["months"][6]["coolingReferencePressurePa"][1],
        -8.806193246045552,
    );
    close(
        "pzC zone 3 month 7",
        &a["months"][6]["coolingReferencePressurePa"][2],
        -14.0900443968535,
    );
    assert_eq!(
        a["months"][6]["coolingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
}

/// H < 15 m, built before 1992, floor above a crawlspace: 0,35/0,35/0,15/0,15 x C_lea with C_p;floor -0,20 (p. 430, 440).
#[test]
fn crawlspace_rows_of_table_11_1() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 9.0, "category": "residential", "combustionAppliances": [], "constructionYear": 1980, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": true, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "pzH zone 1 month 1",
        &a["months"][0]["heatingReferencePressurePa"][0],
        -5.849151404339466,
    );
    assert_eq!(
        a["months"][0]["heatingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    close(
        "leaH month 1",
        &a["months"][0]["heating"]["infiltrationM3PerH"],
        40.112459222708715,
    );
    close(
        "pzC zone 1 month 7",
        &a["months"][6]["coolingReferencePressurePa"][0],
        -2.4540919513048323,
    );
    assert_eq!(
        a["months"][6]["coolingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    close(
        "leaC month 7",
        &a["months"][6]["cooling"]["infiltrationM3PerH"],
        25.901339980245947,
    );
}

/// 15 m <= H < 50 m: two airflow zones, H_path 7,5 and 15 + (H - 15)/2 (p. 431-433), 11.6/11.7.
#[test]
fn two_airflow_zones_at_twenty_metres() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 20.0, "category": "residential", "combustionAppliances": [], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "pzH zone 1 month 1",
        &a["months"][0]["heatingReferencePressurePa"][0],
        -8.005791731069252,
    );
    close(
        "pzH zone 2 month 1",
        &a["months"][0]["heatingReferencePressurePa"][1],
        -15.075635354486366,
    );
    assert_eq!(
        a["months"][0]["heatingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    close(
        "leaH month 1",
        &a["months"][0]["heating"]["infiltrationM3PerH"],
        39.526603835176196,
    );
    close(
        "pzC zone 1 month 7",
        &a["months"][6]["coolingReferencePressurePa"][0],
        -3.1405867486744654,
    );
    close(
        "pzC zone 2 month 7",
        &a["months"][6]["coolingReferencePressurePa"][1],
        -5.214483134927519,
    );
    assert_eq!(
        a["months"][6]["coolingReferencePressurePa"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
}

/// 11.123 capped at the maximum rise, 11.125-11.128 with f_pre;nat;elvv from the preheated design flow (p. 509-511).
#[test]
fn grille_preheating_specified_with_preheated_share() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 9.0, "category": "residential", "combustionAppliances": [], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": {"control": {"method": "specified", "maxPowerWPerDm3PerS": 20.0, "maxTemperatureRiseK": 9.0, "switchOnBelowC": 6.0, "maxSupplyTemperatureC": 18.0}, "preheatedDesignFlowM3PerH": 50.0, "sourceReference": "r65 synthetic"}, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": {"naturalSupplyDm3PerS": 20.0, "totalDm3PerS": 20.0, "sourceReference": "r65 synthetic"}, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "grille month 1",
        &a["months"][0]["grillePreheatingElectricityKwh"],
        251.9407660053158,
    );
    close(
        "thsupH month 1",
        &a["months"][0]["heating"]["naturalSupplyTemperatureC"],
        11.61,
    );
    close(
        "grille month 3",
        &a["months"][2]["grillePreheatingElectricityKwh"],
        247.81125318217184,
    );
    close(
        "wcondH month 1",
        &a["months"][0]["heating"]["weightedConductanceWPerK"],
        47.039652907948216,
    );
}

/// 11.124: (theta_int;set;H - 4) - theta_ODA when the control is unknown (p. 509).
#[test]
fn grille_preheating_fallback() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 9.0, "category": "residential", "combustionAppliances": [], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": {"control": {"method": "fallback"}, "preheatedDesignFlowM3PerH": null, "sourceReference": "r65 synthetic"}, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: mechanische afzuiging, gelijkstroomventilator", "heatRecovery": null, "variant": "c1"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "grille month 1",
        &a["months"][0]["grillePreheatingElectricityKwh"],
        539.7578970897887,
    );
    close(
        "thsupH month 1",
        &a["months"][0]["heating"]["naturalSupplyTemperatureC"],
        16.0,
    );
    close(
        "grille month 10",
        &a["months"][9]["grillePreheatingElectricityKwh"],
        207.62433463176038,
    );
}

/// Table 11.16 (p. 497) in 11.108 and 11.105/11.106 for a dwelling with D2 and heat recovery.
#[test]
fn frost_protection_in_a_balanced_dwelling() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 9.0, "category": "residential", "combustionAppliances": [], "constructionYear": 2020, "coolingSetpointC": 24.0, "dwellingCount": 1, "fans": {"current": "dc", "manufactureYear": 2020, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 100.0, "function": "residential", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 20.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: C1-afzuigsysteem, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": null, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: balansventilatie met WTW", "heatRecovery": {"bypass": {"coldRecoveryEvidence": null, "kind": "full"}, "constantVolumeControl": false, "efficiency": {"exchanger": "counter_flow_plastic", "method": "table"}, "equipmentReference": "voorbeeld: WTW-unit tegenstroom", "layout": "central", "manufactureYear": 2021, "supplyDuctInsulation": {"kind": "insulated"}, "supplyDuctLengthM": null}, "variant": "d2"}}, "usableFloorAreaM2": 100.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "frost month 1",
        &a["months"][0]["frostProtectionElectricityKwh"],
        7.974686955789474,
    );
    close(
        "frost month 11",
        &a["months"][10]["frostProtectionElectricityKwh"],
        7.717438989473684,
    );
    close(
        "thsupH month 1",
        &a["months"][0]["heating"]["mechanicalSupplyTemperatureC"],
        14.774240000000002,
    );
    close(
        "wcondH month 1",
        &a["months"][0]["heating"]["weightedConductanceWPerK"],
        33.131631514177954,
    );
}

/// AHU with heating and cooling coil, ducts in situation 2: table 11.15, table 11.19, f_lea;ahu 1,02, f_ins;ahu 0,02, 11.115/11.119 with c_a as printed (NEN question 7), 11.116/11.120 (p. 474, 496, 502-508).
#[test]
fn ahu_coils_with_ducts_outside_the_zone() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 7.5, "category": "utility", "combustionAppliances": [], "constructionYear": 2021, "coolingSetpointC": 24.0, "dwellingCount": 0, "fans": {"current": "dc", "manufactureYear": 2021, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 400.0, "function": "office", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 21.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: D2 balansventilatie met WTW, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: D2 balansventilatie met WTW, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": {"insideThermalZone": false, "supplyDuctsOutside": "situation2", "heatingCoil": true, "coolingCoil": true}, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: balansventilatie met WTW", "heatRecovery": {"bypass": {"coldRecoveryEvidence": null, "kind": "full"}, "constantVolumeControl": false, "efficiency": {"exchanger": "counter_flow_plastic", "method": "table"}, "equipmentReference": "voorbeeld: WTW-unit tegenstroom", "layout": "central", "manufactureYear": 2021, "supplyDuctInsulation": {"kind": "insulated"}, "supplyDuctLengthM": null}, "variant": "d2"}}, "usableFloorAreaM2": 400.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "ahuH month 1",
        &a["months"][0]["ahuHeatingKwh"],
        110.88089151899065,
    );
    close(
        "ahuC month 7",
        &a["months"][6]["ahuCoolingKwh"],
        16.750032637309403,
    );
    close(
        "thsupC month 1",
        &a["months"][0]["cooling"]["mechanicalSupplyTemperatureC"],
        0.48999999999999977,
    );
    close(
        "thsupH month 7",
        &a["months"][6]["heating"]["mechanicalSupplyTemperatureC"],
        19.45528,
    );
    close(
        "wcondH month 1",
        &a["months"][0]["heating"]["weightedConductanceWPerK"],
        67.42692429404809,
    );
}

/// AHU with heating coil only, ducts in situation 3 (table 11.19, p. 508).
#[test]
fn ahu_heating_coil_only_situation_3() {
    let a = actual(
        r#"{"apartmentBuilding": false, "buildingHeightM": 7.5, "category": "utility", "combustionAppliances": [], "constructionYear": 2021, "coolingSetpointC": 24.0, "dwellingCount": 0, "fans": {"current": "dc", "manufactureYear": 2021, "method": "forfait"}, "floorAboveCrawlspace": false, "flowReduction": {"collective": false, "evidenceReference": null, "flowControlPercent": null, "recirculationPercent": null}, "functions": [{"areaM2": 400.0, "function": "office", "swimmingPool": false}], "grillePreheating": null, "heatingSetpointC": 21.0, "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "voorbeeld: D2 balansventilatie met WTW, gemeten qv10 0,4 dm³/(s·m²)"}, "installedCapacity": null, "maximumCapacityForCooling": null, "overventilation": null, "sourceReference": "voorbeeld: D2 balansventilatie met WTW, gemeten qv10 0,4 dm³/(s·m²)", "system": {"kind": "single", "unit": {"airHandlingUnit": {"insideThermalZone": false, "supplyDuctsOutside": "situation3", "heatingCoil": true, "coolingCoil": false}, "ducts": "luka_a_b_c", "equipmentReference": "voorbeeld: balansventilatie met WTW", "heatRecovery": {"bypass": {"coldRecoveryEvidence": null, "kind": "full"}, "constantVolumeControl": false, "efficiency": {"exchanger": "counter_flow_plastic", "method": "table"}, "equipmentReference": "voorbeeld: WTW-unit tegenstroom", "layout": "central", "manufactureYear": 2021, "supplyDuctInsulation": {"kind": "insulated"}, "supplyDuctLengthM": null}, "variant": "d2"}}, "usableFloorAreaM2": 400.0, "ventilativeCooling": null, "wholeDwellingAreaM2": null, "zoneId": "z1"}"#,
    );
    close(
        "ahuH month 1",
        &a["months"][0]["ahuHeatingKwh"],
        178.6644852717519,
    );
    close(
        "thsupH month 1",
        &a["months"][0]["heating"]["mechanicalSupplyTemperatureC"],
        18.0,
    );
    close(
        "thsupC month 7",
        &a["months"][6]["cooling"]["mechanicalSupplyTemperatureC"],
        18.11,
    );
}
