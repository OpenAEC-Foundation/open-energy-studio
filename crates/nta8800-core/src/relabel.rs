//! Relabelling (herlabelen): which changes between the original project
//! and the improved one may be counted.
//!
//! Sources (page numbers only, no text): BRL 9500-W draft 14-10-2025
//! §4.2.3/4.2.4 (p. 23–24, original survey date and software version),
//! Bijlage 6a (allowed measures, p. 67) and Bijlage 6b (measures that may
//! not be counted, p. 68); BRL 9500-U has the same appendices. ISSO 82.1
//! explains the clusters.
//!
//! The classification works on the project JSON and is deliberately
//! conservative: geometric changes (areas, heights, added or removed
//! zones, surfaces, windows), changed thermal boundaries and a changed
//! generator or system type are not allowed (6b); one-to-one changes of
//! insulation and installation properties and changes to emission,
//! distribution or control are allowed (6a); everything else needs the
//! adviser's review. Insulation on the inside or of elements outside the
//! thermal zone is 6b and cannot be seen in the data, so allowed
//! insulation changes carry a confirmation note.

use serde::Serialize;
use serde_json::Value;

pub const RELABEL_SOURCE: &str =
    "BRL 9500-W/U (14-10-2025) §4.2.3–4.2.4, Bijlage 6a (p. 67) and 6b (p. 68)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelabelVerdict {
    /// Bijlage 6a.
    Allowed,
    /// Bijlage 6b.
    NotAllowed,
    /// Not classifiable from the data; the adviser decides.
    Review,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelabelChange {
    /// JSON pointer of the change.
    pub path: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub verdict: RelabelVerdict,
    /// Cluster of Bijlage 6a/6b, or why it needs review.
    pub cluster: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelabelAssessment {
    pub source: &'static str,
    /// No change is classified as not allowed.
    pub allowed: bool,
    pub needs_review: bool,
    pub changes: Vec<RelabelChange>,
}

/// Keys whose change says nothing about the building.
const IGNORED_KEYS: &[&str] = &[
    "name",
    "description",
    "notes",
    "color",
    "visible",
    "selected",
    "label",
];

const GEOMETRY_KEYS: &[&str] = &[
    "area",
    "areaM2",
    "grossArea",
    "grossAreaM2",
    "netArea",
    "usableFloorArea",
    "usableFloorAreaM2",
    "totalUsableFloorAreaM2",
    "lossAreaM2",
    "height",
    "heightM",
    "buildingHeightM",
    "width",
    "widthM",
    "length",
    "lengthM",
    "perimeter",
    "perimeterM",
    "exposedPerimeterM",
    "volume",
    "volumeM3",
    "orientation",
    "azimuth",
    "tilt",
    "storeys",
];

const LAYOUT_ARRAYS: &[&str] = &[
    "zones",
    "surfaces",
    "windows",
    "doors",
    "rooms",
    "functions",
    "functionAreas",
    "unheatedSpaces",
];

const BOUNDARY_KEYS: &[&str] = &["thermalBoundary", "boundary", "unheatedSpaceId", "zoneId"];

const SYSTEM_KEYS: &[&str] = &[
    "kind",
    "type",
    "method",
    "variant",
    "generator",
    "generatorType",
    "carrier",
    "source",
    "sink",
    "systemType",
];

const INSTALLATION_MARKERS: &[&str] = &[
    "heating",
    "hotwater",
    "ventilation",
    "cooling",
    "generator",
    "heatpump",
    "pv",
    "production",
    "solar",
    "humidif",
    "lighting",
    "system",
];

const SUBSYSTEM_MARKERS: &[&str] = &["emission", "distribution", "control", "balancing"];

const INSULATION_MARKERS: &[&str] = &[
    "rc",
    "uvalue",
    "u_value",
    "lambda",
    "thickness",
    "insulation",
    "layers",
    "glazing",
    "ggl",
];

const SHADING_MARKERS: &[&str] = &["shading", "overhang", "obstruction", "sunshade"];

const PRODUCTION_ARRAYS: &[&str] = &["pvSystems", "onSiteProduction", "pv", "solarThermal"];

fn segments(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect()
}

fn contains_any(path: &str, markers: &[&str]) -> bool {
    let lower = path.to_ascii_lowercase();
    markers.iter().any(|marker| lower.contains(marker))
}

fn classify(path: &str, before: Option<&Value>, after: Option<&Value>) -> RelabelChange {
    let parts = segments(path);
    let key = parts
        .iter()
        .rev()
        .find(|segment| segment.parse::<usize>().is_err())
        .cloned()
        .unwrap_or_default();
    let last_is_index = parts
        .last()
        .is_some_and(|last| last.parse::<usize>().is_ok());
    let element_added_or_removed = before.is_none() || after.is_none();
    let change = |verdict, cluster, note| RelabelChange {
        path: path.to_string(),
        before: before.cloned(),
        after: after.cloned(),
        verdict,
        cluster,
        note,
    };
    if element_added_or_removed && last_is_index && LAYOUT_ARRAYS.contains(&key.as_str()) {
        return change(
            RelabelVerdict::NotAllowed,
            "geometric change: layout (zones, surfaces, windows added or removed)",
            None,
        );
    }
    if element_added_or_removed && last_is_index && PRODUCTION_ARRAYS.contains(&key.as_str()) {
        return change(
            RelabelVerdict::Review,
            "building-bound production added or removed",
            Some("a one-to-one replacement is allowed (6a), a system change is not (6b)"),
        );
    }
    if GEOMETRY_KEYS.contains(&key.as_str()) {
        return change(
            RelabelVerdict::NotAllowed,
            "geometric change: usable or loss area, dimensions or layout",
            None,
        );
    }
    if BOUNDARY_KEYS.contains(&key.as_str()) {
        return change(
            RelabelVerdict::NotAllowed,
            "geometric change: thermal boundary or zoning",
            None,
        );
    }
    if contains_any(path, SHADING_MARKERS) {
        return change(
            RelabelVerdict::Review,
            "shading, overhang or obstruction",
            Some("allowed for heating and hot water layouts (6a), not for cooling (6b)"),
        );
    }
    if contains_any(path, SUBSYSTEM_MARKERS) {
        return change(
            RelabelVerdict::Allowed,
            "change in distribution, emission or control",
            None,
        );
    }
    let is_text =
        matches!(before, Some(Value::String(_))) || matches!(after, Some(Value::String(_)));
    if contains_any(path, INSTALLATION_MARKERS) {
        if SYSTEM_KEYS.contains(&key.as_str()) && is_text {
            return change(
                RelabelVerdict::NotAllowed,
                "installation: system change",
                None,
            );
        }
        if element_added_or_removed {
            return change(
                RelabelVerdict::Review,
                "installation part added or removed",
                Some("a one-to-one replacement is allowed (6a), a system change is not (6b)"),
            );
        }
        return change(
            RelabelVerdict::Allowed,
            "one-to-one replacement or change: installation properties",
            None,
        );
    }
    if contains_any(path, INSULATION_MARKERS) || contains_any(&key, &["construction"]) {
        return change(
            RelabelVerdict::Allowed,
            "one-to-one replacement or change: insulation properties",
            Some("confirm: not insulation on the inside and not of elements outside the thermal zone (6b)"),
        );
    }
    change(
        RelabelVerdict::Review,
        "not classified",
        Some("decide with Bijlage 6a/6b and ISSO 82.1"),
    )
}

fn diff(path: &str, before: Option<&Value>, after: Option<&Value>, out: &mut Vec<RelabelChange>) {
    match (before, after) {
        (Some(Value::Object(a)), Some(Value::Object(b))) => {
            let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
            keys.sort();
            keys.dedup();
            for key in keys {
                if IGNORED_KEYS.contains(&key.as_str())
                    || key.ends_with("Reference")
                    || (path.is_empty() && key == "registration")
                {
                    continue;
                }
                let child = format!("{path}/{}", key.replace('~', "~0").replace('/', "~1"));
                diff(&child, a.get(key), b.get(key), out);
            }
        }
        (Some(Value::Array(a)), Some(Value::Array(b))) => {
            for index in 0..a.len().max(b.len()) {
                diff(&format!("{path}/{index}"), a.get(index), b.get(index), out);
            }
        }
        (Some(a), Some(b)) if a == b => {}
        (None, None) => {}
        _ => out.push(classify(path, before, after)),
    }
}

/// Classifies every difference between the original and the current
/// project per Bijlage 6a/6b. The registration block is compared only for
/// the survey date, which must stay the original one (§4.2.4).
pub fn assess_relabel(original: &Value, current: &Value) -> RelabelAssessment {
    let mut changes = Vec::new();
    diff("", Some(original), Some(current), &mut changes);
    let survey_date = |project: &Value| project.pointer("/registration/surveyDate").cloned();
    let (before, after) = (survey_date(original), survey_date(current));
    if before.is_some() && before != after {
        changes.push(RelabelChange {
            path: "/registration/surveyDate".into(),
            before,
            after,
            verdict: RelabelVerdict::NotAllowed,
            cluster: "relabel keeps the original survey date",
            note: None,
        });
    }
    RelabelAssessment {
        source: RELABEL_SOURCE,
        allowed: changes
            .iter()
            .all(|change| change.verdict != RelabelVerdict::NotAllowed),
        needs_review: changes
            .iter()
            .any(|change| change.verdict == RelabelVerdict::Review),
        changes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn project() -> Value {
        json!({
            "name": "Woning",
            "registration": {"surveyDate": "2026-01-10", "relabel": false},
            "zones": [{
                "id": "z1",
                "surfaces": [{"id": "gevel", "area": 40.0, "thermalBoundary": "outdoor",
                    "constructionId": "c1", "windows": [{"id": "w1", "area": 2.0, "uValue": 2.8}]}]
            }],
            "constructions": [{"id": "c1", "rcValue": 0.4}],
            "heatingSystems": [{"generator": {"type": "boiler_hr107", "efficiency": 0.9},
                "emission": {"system": "radiators"}}],
            "pvSystems": []
        })
    }

    fn verdict(result: &RelabelAssessment, path: &str) -> RelabelVerdict {
        result
            .changes
            .iter()
            .find(|change| change.path == path)
            .unwrap_or_else(|| panic!("{path}: {:?}", result.changes))
            .verdict
    }

    #[test]
    fn insulation_and_one_to_one_installation_changes_are_allowed() {
        let original = project();
        let mut current = project();
        current["name"] = json!("Woning na isolatie");
        current["constructions"][0]["rcValue"] = json!(3.5);
        current["zones"][0]["surfaces"][0]["windows"][0]["uValue"] = json!(1.1);
        current["heatingSystems"][0]["generator"]["efficiency"] = json!(0.95);
        current["heatingSystems"][0]["emission"]["system"] = json!("floor_heating");
        current["registration"]["relabel"] = json!(true);
        let result = assess_relabel(&original, &current);
        assert!(result.allowed, "{:?}", result.changes);
        assert_eq!(result.changes.len(), 4);
        assert_eq!(
            verdict(&result, "/constructions/0/rcValue"),
            RelabelVerdict::Allowed
        );
        let insulation = result
            .changes
            .iter()
            .find(|change| change.path == "/constructions/0/rcValue")
            .unwrap();
        assert!(insulation.note.is_some());
        assert_eq!(
            verdict(&result, "/heatingSystems/0/emission/system"),
            RelabelVerdict::Allowed
        );
    }

    #[test]
    fn geometry_boundary_system_and_survey_date_changes_are_not_allowed() {
        let original = project();
        let mut current = project();
        current["zones"][0]["surfaces"][0]["area"] = json!(44.0);
        current["zones"][0]["surfaces"][0]["thermalBoundary"] = json!("unheated_space");
        current["zones"][0]["surfaces"][0]["windows"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id": "w2", "area": 1.0, "uValue": 1.1}));
        current["heatingSystems"][0]["generator"]["type"] = json!("heat_pump_air_water");
        current["registration"]["surveyDate"] = json!("2026-09-01");
        let result = assess_relabel(&original, &current);
        assert!(!result.allowed);
        for path in [
            "/zones/0/surfaces/0/area",
            "/zones/0/surfaces/0/thermalBoundary",
            "/zones/0/surfaces/0/windows/1",
            "/heatingSystems/0/generator/type",
            "/registration/surveyDate",
        ] {
            assert_eq!(verdict(&result, path), RelabelVerdict::NotAllowed, "{path}");
        }
    }

    #[test]
    fn added_pv_needs_review() {
        let original = project();
        let mut current = project();
        current["pvSystems"] = json!([{"id": "pv1", "peakPowerKw": 3.0}]);
        let result = assess_relabel(&original, &current);
        assert!(result.allowed);
        assert!(result.needs_review);
        assert_eq!(verdict(&result, "/pvSystems/0"), RelabelVerdict::Review);
    }
}
