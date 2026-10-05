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
    for edition in ["2023", "2022", "2020+A1"] {
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
