//! Editions of NTA 8800 on the project route: the default edition keeps its
//! results and fingerprint, an older edition is calculated but never
//! registrable, an edition without a profile is refused.

use nta8800_core::norm_versions::NormVersion;
use nta8800_core::project_performance::{assess_project_performance, ProjectPerformanceAssessment};
use serde_json::{json, Value};

const DWELLING: &str =
    include_str!("../../../training-data/nta8800-example-terraced-dwelling.json");
const OFFICE: &str = include_str!("../../../training-data/nta8800-example-office.json");

fn strip_nulls(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, item)| !item.is_null())
                .map(|(key, item)| (key, strip_nulls(item)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(strip_nulls).collect()),
        other => other,
    }
}

fn project(json: &str, edition: Option<&str>) -> Value {
    let mut value = strip_nulls(serde_json::from_str(json).unwrap());
    if let Some(edition) = edition {
        value["ntaCalculation"]["normVersion"] = json!(edition);
    }
    value
}

fn indicator(result: &ProjectPerformanceAssessment, key: &str) -> Option<f64> {
    let performance = serde_json::to_value(result.performance.as_ref()?).unwrap();
    performance.pointer(key).and_then(Value::as_f64)
}

#[test]
fn default_edition_is_unchanged_and_registrable() {
    for json in [DWELLING, OFFICE] {
        let implicit = assess_project_performance(&project(json, None));
        let explicit = assess_project_performance(&project(json, Some("2025+C1")));
        assert_eq!(
            implicit.status, "calculated_unverified",
            "{:?}",
            implicit.gaps
        );
        assert_eq!(explicit.status, "calculated_unverified");
        assert_eq!(implicit.norm_version, NormVersion::V2025C1);
        assert!(implicit.registration_eligible);
        assert_eq!(implicit.target_norm_version, "NTA 8800:2025+C1:2026");
        // The derived kernel input leaves the default edition out, so its
        // fingerprint does not move.
        assert_eq!(
            implicit.performance.as_ref().unwrap().input_fingerprint,
            explicit.performance.as_ref().unwrap().input_fingerprint
        );
        assert_eq!(
            indicator(&implicit, "/primaryFossilIndicatorKwhPerM2Year"),
            indicator(&explicit, "/primaryFossilIndicatorKwhPerM2Year")
        );
    }
}

#[test]
fn older_edition_is_calculated_but_not_registrable() {
    for json in [DWELLING, OFFICE] {
        let current = assess_project_performance(&project(json, None));
        let legacy = assess_project_performance(&project(json, Some("2024")));
        assert_eq!(
            legacy.status, "calculated_legacy_edition",
            "{:?}",
            legacy.gaps
        );
        assert_eq!(legacy.norm_version, NormVersion::V2024);
        assert!(!legacy.registration_eligible);
        assert_eq!(legacy.target_norm_version, "NTA 8800:2024 met INT-V1:2024");
        let performance = legacy.performance.as_ref().unwrap();
        assert_eq!(
            performance.target_norm_version,
            "NTA 8800:2024 met INT-V1:2024"
        );
        assert!(!performance.registration_eligible);
        // The edition is part of the input and therefore of the fingerprint.
        assert_ne!(legacy.input_fingerprint, current.input_fingerprint);
        assert_ne!(
            performance.input_fingerprint,
            current.performance.as_ref().unwrap().input_fingerprint
        );
        // Indicators new in 2025+C1 are absent.
        assert!(performance.chapter5.is_none());
        assert!(performance.annual_zeb_primary_total_kwh.is_none());
        assert!(performance.annual_final_energy_kwh.is_none());
        assert!(indicator(&legacy, "/primaryFossilIndicatorKwhPerM2Year").is_some());
    }
}

#[test]
fn registration_refuses_an_older_edition() {
    let mut value = project(DWELLING, Some("2024"));
    value["registration"] = json!({});
    let result = assess_project_performance(&value);
    let registration = result.registration.expect("registration assessed");
    assert!(registration
        .issues
        .iter()
        .any(|issue| issue.code == "legacy_edition_not_registrable" && issue.severity == "error"));
    assert!(!registration.ready_for_registration);
    assert!(!registration.dossier_complete);
}

#[test]
fn edition_without_profile_is_refused() {
    for edition in ["2022", "2020+A1"] {
        let result = assess_project_performance(&project(DWELLING, Some(edition)));
        assert_eq!(result.status, "invalid", "{edition}");
        let performance = result.performance.as_ref().unwrap();
        assert!(performance
            .issues
            .iter()
            .any(|issue| issue.code == "edition_not_implemented"));
    }
}

#[test]
fn unknown_edition_is_an_input_gap() {
    let result = assess_project_performance(&project(DWELLING, Some("2019")));
    assert_eq!(result.status, "incomplete");
    assert!(
        result.gaps.iter().any(|gap| gap
            .detail
            .as_deref()
            .is_some_and(|detail| detail.contains("normVersion"))),
        "{:?}",
        result.gaps
    );
}

/// The example projects have no input on the routes that differ, so only
/// the CO2 factors of table 5.3 (2024 p. 94–95 against 2025+C1 p. 96–98)
/// and the indicators new in 2025+C1 change between the editions.
#[test]
fn example_projects_differ_only_where_the_editions_differ() {
    const EXPECTED: [&str; 7] = [
        "annualCo2Kg",
        "co2KgPerM2",
        "annualFinalEnergyKwh",
        "annualFinalEnergyEedKwh",
        "annualZebPrimaryTotalKwh",
        "zebPrimaryTotalIndicatorKwhPerM2",
        "annualZebCo2Kg",
    ];
    for json in [DWELLING, OFFICE] {
        let current = assess_project_performance(&project(json, None));
        let legacy = assess_project_performance(&project(json, Some("2024")));
        let current = serde_json::to_value(current.performance.as_ref().unwrap()).unwrap();
        let legacy = serde_json::to_value(legacy.performance.as_ref().unwrap()).unwrap();
        let (Value::Object(current), Value::Object(legacy)) = (current, legacy) else {
            panic!("performance is an object");
        };
        let changed: Vec<&str> = current
            .iter()
            .filter(|(key, value)| {
                !matches!(
                    key.as_str(),
                    "inputFingerprint"
                        | "targetNormVersion"
                        | "normVersion"
                        | "registrationEligible"
                ) && (value.is_number() || value.is_string() || value.is_boolean())
                    && legacy.get(key.as_str()) != Some(value)
            })
            .map(|(key, _)| key.as_str())
            .collect();
        assert!(
            changed.iter().all(|key| EXPECTED.contains(key)),
            "{changed:?}"
        );
        assert!(changed.contains(&"annualCo2Kg"));
        // BENG 1–3 and the label are the same.
        for key in [
            "needIndicatorKwhPerM2Year",
            "primaryFossilIndicatorKwhPerM2Year",
            "renewableSharePercent",
            "indicativeLabelClass",
        ] {
            assert_eq!(current.get(key), legacy.get(key), "{key}");
        }
    }
}

/// The example office uses the LED-from-2017 column of table 14.3, which
/// NTA 8800:2023 does not have (p. 648); for 2023 it is set to "overig".
fn project_2023_compatible(json: &str, edition: &str) -> Value {
    fn clear(value: &mut Value) {
        match value {
            Value::Object(map) => {
                if let Some(led) = map.get_mut("ledFrom2017") {
                    *led = json!(false);
                }
                map.values_mut().for_each(clear);
            }
            Value::Array(items) => items.iter_mut().for_each(clear),
            _ => {}
        }
    }
    let mut value = project(json, Some(edition));
    clear(&mut value);
    value
}

/// NTA 8800:2023 is calculated as an older edition with its own label.
#[test]
fn edition_2023_is_calculated_but_not_registrable() {
    for json in [DWELLING, OFFICE] {
        let result = assess_project_performance(&project_2023_compatible(json, "2023"));
        assert_eq!(
            result.status, "calculated_legacy_edition",
            "{:?}",
            result.gaps
        );
        assert_eq!(result.norm_version, NormVersion::V2023);
        assert!(!result.registration_eligible);
        assert_eq!(result.target_norm_version, "NTA 8800:2023");
        assert!(result.performance.as_ref().unwrap().chapter5.is_none());
    }
    // Unchanged, the office asks for a 2024 column.
    let office = assess_project_performance(&project(OFFICE, Some("2023")));
    assert_eq!(office.status, "invalid");
    assert!(office
        .performance
        .as_ref()
        .unwrap()
        .issues
        .iter()
        .any(|issue| issue.code == "route_not_in_edition" && issue.path.ends_with("ledFrom2017")));
}

fn changed_keys(a: &ProjectPerformanceAssessment, b: &ProjectPerformanceAssessment) -> Vec<String> {
    let a = serde_json::to_value(a.performance.as_ref().unwrap()).unwrap();
    let b = serde_json::to_value(b.performance.as_ref().unwrap()).unwrap();
    a.as_object()
        .unwrap()
        .iter()
        .filter(|(key, value)| {
            !matches!(
                key.as_str(),
                "inputFingerprint" | "targetNormVersion" | "normVersion"
            ) && (value.is_number() || value.is_string() || value.is_boolean())
                && b.get(key.as_str()) != Some(value)
        })
        .map(|(key, _)| key.clone())
        .collect()
}

/// 2023 against 2024 on the example projects (the office without the LED
/// column in both). The dwelling has no input on a route that differs. The
/// office changes only TOjuli, through ΔT_C;fan of utility buildings: 1,5 K
/// in 2023 (p. 496) against 0,7 K in 2024 (p. 491).
#[test]
fn example_projects_2023_differ_from_2024_only_where_the_editions_differ() {
    let run = |json, edition| assess_project_performance(&project_2023_compatible(json, edition));
    assert!(changed_keys(&run(DWELLING, "2024"), &run(DWELLING, "2023")).is_empty());
    assert_eq!(
        changed_keys(&run(OFFICE, "2024"), &run(OFFICE, "2023")),
        ["tojuliMaxK"]
    );
}

/// Table 13.2 diameter rows exist in NTA 8800:2023 only (p. 533; 2024
/// p. 527 and 2025+C1 p. 543 have one kitchen row).
#[test]
fn kitchen_pipe_diameter_is_a_2023_route() {
    let with_diameter = |edition: &str| {
        let mut value = project(DWELLING, Some(edition));
        let emission = &mut value["ntaCalculation"]["hotWater"]["emission"];
        assert!(emission.is_object(), "{emission}");
        emission["kitchenPipeDiameter"] = json!("up_to_10_mm");
        assess_project_performance(&value)
    };
    assert_eq!(with_diameter("2023").status, "calculated_legacy_edition");
    for edition in ["2024", "2025+C1"] {
        let result = with_diameter(edition);
        assert_eq!(result.status, "invalid", "{edition}");
        assert!(result
            .performance
            .as_ref()
            .unwrap()
            .issues
            .iter()
            .any(|issue| issue.code == "route_not_in_edition"
                && issue.path.ends_with("emission.kitchenPipeDiameter")));
    }
}

/// Tables 9.2–9.10 of NTA 8800:2023 (p. 273–285): the 2023 description of
/// the heating emission is accepted under 2023 only, and there it replaces
/// the forfait derived from the 2024 fields.
#[test]
fn heating_emission_description_is_a_2023_route() {
    let described = |edition: &str| {
        let mut value = project(DWELLING, Some(edition));
        value["ntaCalculation"]["emission"]["edition2023"] = json!({
            "kind": {"type": "surface", "control": "room",
                     "system": "floor_wet_or_unknown", "insulation": "double_insulation"},
            "certifiedControl": true,
            "roomAutomation": "network_with_override_and_adaptive",
            "pipeSystem": "two_pipe",
            "balancing": "dynamic_full"
        });
        assess_project_performance(&value)
    };
    let plain = assess_project_performance(&project(DWELLING, Some("2023")));
    let with_2023 = described("2023");
    assert_eq!(
        with_2023.status, "calculated_legacy_edition",
        "{:?}",
        with_2023.gaps
    );
    // Forfait (1,6 + 1,7)/2 + 2,5 − 0,3 + 0,7 = 4,55 K (capped at 0,15 by
    // 9.16) against the description 0 + 1,5 + (0,7 + 0,1)/2 − 0,2 + 0 − 1,2
    // = 0,5 K.
    let beng2 = |result: &ProjectPerformanceAssessment| {
        indicator(result, "/primaryFossilIndicatorKwhPerM2Year").unwrap()
    };
    assert!(beng2(&with_2023) < beng2(&plain));
    for edition in ["2024", "2025+C1"] {
        let result = described(edition);
        assert_eq!(result.status, "invalid", "{edition}");
        assert!(result
            .performance
            .as_ref()
            .unwrap()
            .issues
            .iter()
            .any(|issue| issue.code == "route_not_in_edition"
                && issue.path.ends_with("emission.edition2023")));
    }
}
