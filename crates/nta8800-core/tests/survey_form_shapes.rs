//! The JSON the survey forms write (src/components/BasisopnamePanel/SurveyBasics.tsx)
//! must be read by the kernel. These cases mirror the expectations of
//! src/__tests__/survey-basics-forms.test.tsx: each is a form result that the
//! kernel used to refuse (review 7, 9 October 2026).

use nta8800_core::opname::utility::{assess_utility_survey, UtilitySurvey};
use nta8800_core::opname::{assess_residential_survey, ResidentialSurvey};
use serde_json::{json, Value};

fn utility_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../training-data/nta8800-opname-utility-1985-office.json"
    ))
    .unwrap()
}

fn residential_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../training-data/nta8800-opname-1930-terraced.json"
    ))
    .unwrap()
}

fn utility(value: Value) -> (UtilitySurvey, Vec<String>) {
    let survey: UtilitySurvey =
        serde_json::from_value(value).expect("the kernel reads the form JSON");
    let codes = assess_utility_survey(&survey)
        .issues
        .iter()
        .map(|issue| issue.code.to_string())
        .collect();
    (survey, codes)
}

fn residential(value: Value) -> Vec<String> {
    let survey: ResidentialSurvey =
        serde_json::from_value(value).expect("the kernel reads the form JSON");
    assess_residential_survey(&survey)
        .issues
        .iter()
        .map(|issue| issue.code.to_string())
        .collect()
}

#[test]
fn utility_survey_form_shapes_are_read() {
    let base = utility_fixture();
    let (_, before) = utility(base.clone());

    let mut value = base;
    // Fix 1: unchecking the last solar-control window writes an empty list.
    value["solarControlWindowIds"] = json!([]);
    // Fix 2: an unheated space from the list and a surface bordering it.
    value["envelope"]["unheatedSpaces"] =
        json!([{ "id": "onverwarmd-1", "description": "berging" }]);
    value["envelope"]["surfaces"][1]["boundary"] =
        json!({ "kind": "unheated_space", "spaceId": "onverwarmd-1" });
    // Fix 3: a door with a glass share and its glass type.
    value["envelope"]["doors"][0]["glassFraction"] = json!(0.4);
    value["envelope"]["doors"][0]["glass"] = json!("hr_plus_plus");
    // Fix 4: a declared variant from the select (table 11.5 code).
    value["ventilation"]["declaredVariant"] = json!("d5b");
    // Fix 6: recirculation present, percentage left empty (no field written).
    value["ventilation"]["recirculation"] = json!("present_percent_unknown");
    // Fix 7: the daylight select's default leaves the field out.
    value["lighting"][0]
        .as_object_mut()
        .unwrap()
        .remove("daylight");

    let (survey, after) = utility(value);
    assert!(survey.solar_control_window_ids.is_empty());
    for code in [
        "door_glass_required",
        "unheated_space_unknown",
        "solar_control_window_unknown",
    ] {
        assert!(!after.iter().any(|c| c == code), "{code} in {after:?}");
    }
    // The form shapes add no issue the plain fixture does not have, apart from
    // what the new answers legitimately bring.
    for code in &after {
        assert!(
            before.contains(code) || !code.contains("invalid"),
            "new invalid code {code}"
        );
    }
}

#[test]
fn residential_survey_form_shapes_are_read() {
    let mut value = residential_fixture();
    value["ventilation"]["declaredVariant"] = json!("c4a");
    if value["envelope"]["doors"]
        .as_array()
        .map_or(true, Vec::is_empty)
    {
        let surface = value["envelope"]["surfaces"][0]["id"].clone();
        value["envelope"]["doors"] = json!([{ "id": "deur-1", "surfaceId": surface, "areaM2": 2.0,
            "frame": "wood_or_plastic", "sourceReference": "" }]);
    }
    value["envelope"]["doors"][0]["glassFraction"] = json!(0.3);
    value["envelope"]["doors"][0]["glass"] = json!("double");
    let codes = residential(value);
    assert!(
        !codes.iter().any(|code| code == "door_glass_required"),
        "{codes:?}"
    );
}

/// The answers the form-coverage forms add (src/components/BasisopnamePanel/
/// SurveyCoverageFields.tsx), in the shapes those forms write: the kernel
/// reads them and they bring no code the plain fixture does not have.
fn coverage_answers(value: &mut Value, utility: bool) {
    value["areaSourceReference"] = json!("NEN 2580 meetrapport");
    // §15.5 storage counts only with PV (storage_requires_pv); the form offers it only then.
    if value
        .get("pv")
        .and_then(Value::as_array)
        .is_some_and(|pv| !pv.is_empty())
    {
        value["storage"] =
            json!({ "electricalKwh": 5.0, "thermalKwh": 0.0, "sourceReference": "factuur" });
    }
    let surface = value["envelope"]["surfaces"][0]["id"].clone();
    value["envelope"]["surfaces"][0]["orientation"] = json!("south");
    value["envelope"]["surfaces"][0]["tiltDeg"] = json!(90.0);
    value["envelope"]["surfaces"][0]["cavity"] = json!(true);
    value["envelope"]["panels"] = json!([{ "id": "paneel-1", "surfaceId": surface, "areaM2": 1.0,
        "insulation": { "kind": "known_thickness", "thicknessMm": 40.0 }, "cavity": false,
        "frame": "wood_or_plastic", "sourceReference": "opname" }]);
    // `get_mut`, not `value[...]`: indexing a missing key would insert a null.
    if let Some(window) = value
        .get_mut("envelope")
        .and_then(|envelope| envelope.get_mut("windows"))
        .and_then(|windows| windows.get_mut(0))
    {
        window.as_object_mut().unwrap().remove("shading");
        window["obstruction"] = json!({ "heating": vec![1.0; 12], "cooling": vec![1.0; 12],
            "sourceReference": "berekening" });
    }
    value["heating"]["balanced"] = json!(true);
    value["heating"]["storeys"] = json!(2);
    value["hotWater"]["showerHeatRecovery"] = json!("vertical");
    if let Some(pv) = value.get_mut("pv").and_then(|pv| pv.get_mut(0)) {
        pv["mounting"] = json!("strongly_ventilated");
        pv["moduleType"] = json!("amorphous_multi_junction");
        pv.as_object_mut().unwrap().remove("shading");
        pv["obstructionFactors"] = json!([0.9]);
    }
    if utility {
        value["hotWater"]["circulation"] = json!(true);
        value["hotWater"]["storage"] = json!([{ "id": "vat-1", "volumeL": 150.0, "label": "b",
            "producedFrom2018": true, "connection": "fully_insulated", "inHeatedZone": true,
            "sourceReference": "typeplaat" }]);
        if value.get("cooling").is_some_and(Value::is_object) {
            value["cooling"]["designTemperature"] = json!("t12_to18");
            value["cooling"]["control"] = json!("central_with_room_control");
            value["cooling"]["pipesInsulated"] = json!(true);
        }
        if value["ventilation"]
            .get("ahu")
            .is_some_and(Value::is_object)
        {
            value["ventilation"]["ahu"]["insideThermalZone"] = json!(true);
        }
    } else {
        value["hotWater"]["served"] = json!("kitchen_and_bathroom");
        value["hotWater"]["kitchenLengthM"] = json!(3.0);
        value["hotWater"]["bathroomLengthM"] = json!(6.0);
    }
}

#[test]
fn form_coverage_answers_are_read_by_both_surveys() {
    let (_, before) = utility(utility_fixture());
    let mut value = utility_fixture();
    coverage_answers(&mut value, true);
    let (_, after) = utility(value);
    for code in &after {
        assert!(
            before.contains(code),
            "utility: new code {code} in {after:?}"
        );
    }

    let before = residential(residential_fixture());
    let mut value = residential_fixture();
    coverage_answers(&mut value, false);
    let after = residential(value);
    for code in &after {
        assert!(
            before.contains(code),
            "residential: new code {code} in {after:?}"
        );
    }
}

/// Sets `null` at a path like `hotWater.storage[].inHeatedZone`, creating the
/// missing objects on the way. A `[]` step uses the first element of the list,
/// which is created from `item` when the list is empty or absent.
fn set_null(value: &mut Value, path: &str, item: &Value) {
    let mut current = value;
    let steps: Vec<&str> = path.split('.').collect();
    for (index, step) in steps.iter().enumerate() {
        let last = index + 1 == steps.len();
        let (key, list) = match step.strip_suffix("[]") {
            Some(key) => (key, true),
            None => (*step, false),
        };
        if !current.is_object() {
            *current = json!({});
        }
        if last && !list {
            current[key] = Value::Null;
            return;
        }
        if list {
            let empty = current[key]
                .as_array()
                .map_or(true, |items| items.is_empty());
            if empty {
                current[key] = json!([item.clone()]);
            }
            current = &mut current[key][0];
        } else {
            if current[key].is_null() {
                current[key] = json!({});
            }
            current = &mut current[key];
        }
    }
}

/// Review 8: the survey forms may write `null` only where the kernel reads it as
/// "not answered". training-data/nta8800-survey-null-tolerant-paths.json lists
/// those answers; src/__tests__/survey-null-paths.test.tsx requires every other
/// null to be named by `emptyRequiredSurveyFields` first.
#[test]
fn null_tolerant_survey_answers_are_accepted() {
    let tolerant: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-survey-null-tolerant-paths.json"
    ))
    .unwrap();
    let vessel = json!({ "id": "vat-1", "volumeL": 150, "sourceReference": "" });
    for path in tolerant["residentialSurvey"].as_array().unwrap() {
        let mut value = residential_fixture();
        set_null(&mut value, path.as_str().unwrap(), &vessel);
        let survey: Result<ResidentialSurvey, _> = serde_json::from_value(value);
        assert!(survey.is_ok(), "residential {path}: {:?}", survey.err());
    }
    for path in tolerant["utilitySurvey"].as_array().unwrap() {
        let mut value = utility_fixture();
        if path.as_str().unwrap().starts_with("cooling.") && value["cooling"].is_null() {
            // The office fixture has no cooling; the form's water-based template.
            value["cooling"] = json!({
                "generator": "compression", "emitter": "ceiling_cooling",
                "fanCoilCount": 1, "waterBased": true, "sourceReference": "",
            });
        }
        set_null(&mut value, path.as_str().unwrap(), &vessel);
        let survey: Result<UtilitySurvey, _> = serde_json::from_value(value);
        assert!(survey.is_ok(), "utility {path}: {:?}", survey.err());
    }
}

/// The other side of the list: a required number refuses null, which is why
/// `emptyRequiredSurveyFields` must name it before the survey is calculated.
#[test]
fn required_survey_numbers_refuse_null() {
    let vessel = json!({ "id": "vat-1", "volumeL": 150, "sourceReference": "" });
    let mut value = utility_fixture();
    set_null(&mut value, "hotWater.storage[].volumeL", &vessel);
    assert!(serde_json::from_value::<UtilitySurvey>(value).is_err());
    let mut value = residential_fixture();
    value["storage"] = json!({ "electricalKwh": null, "sourceReference": "" });
    assert!(serde_json::from_value::<ResidentialSurvey>(value).is_err());
}

/// Review 8: storage left out by the form (both capacities empty) is read with
/// the kernel's default of 0 kWh.
#[test]
fn storage_without_capacities_is_read() {
    let mut value = residential_fixture();
    value["storage"] = json!({ "sourceReference": "" });
    let survey: Result<ResidentialSurvey, _> = serde_json::from_value(value);
    assert!(survey.is_ok(), "{:?}", survey.err());
}
