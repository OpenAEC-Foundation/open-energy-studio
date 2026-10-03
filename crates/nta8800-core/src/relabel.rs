//! Relabelling (herlabelen): which changes between the original project
//! and the improved one may be counted.
//!
//! Sources (page numbers only, no text): BRL 9500-W draft 14-10-2025
//! §4.2.3/4.2.4 (p. 23–24, original survey date and software version),
//! Bijlage 6a (allowed measures, p. 67) and Bijlage 6b (measures that may
//! not be counted, p. 68); BRL 9500-U draft 14-10-2025 §4.2.3 (p. 18–19)
//! with §4.2.4 (p. 19–20) and Bijlagen 6a/6b (p. 58–60). For utility
//! buildings 6a covers only
//! one-to-one replacements: geometric changes of insulation or
//! installation and changes in distribution, emission or control are 6b.
//! For dwellings 6a lists the subsystem changes per service (p. 67):
//! ventilation and hot water only the emission system; heating and
//! cooling distribution, emission and control. Other subsystem changes need
//! review.
//! Lighting is in neither appendix and always needs review. ISSO 82.1 and
//! 75.1 explain the clusters. The scheme follows the project's
//! `buildingFunction` (residential → W, otherwise U).
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

pub const RELABEL_SOURCE_W: &str =
    "BRL 9500-W (14-10-2025) §4.2.3–4.2.4 (p. 23–24), Bijlage 6a (p. 67) and 6b (p. 68)";
pub const RELABEL_SOURCE_U: &str =
    "BRL 9500-U (14-10-2025) §4.2.3 (p. 18–19), §4.2.4 (p. 19–20), Bijlage 6a (p. 58) and 6b (p. 59–60)";

/// BRL 9500 part that governs the relabel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelabelScheme {
    /// 9500-W, dwellings.
    W,
    /// 9500-U, utility buildings.
    U,
}

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
    pub scheme: RelabelScheme,
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
    "floorArea",
    "floorAreaM2",
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

/// Elements whose layout may change at an equal loss area (W 6a,
/// "geometrische wijziging met betrekking tot isolatie").
const OPENING_ARRAYS: &[&str] = &["windows", "doors", "panels"];

const LIGHTING_MARKERS: &[&str] = &["lighting"];

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

const PRODUCTION_ARRAYS: &[&str] = &[
    "solarPV",
    "pvSystems",
    "onSiteProduction",
    "pv",
    "solarThermal",
    "solarWaterHeaters",
];

/// Project blocks that are not label input: the maatwerkadvies and the
/// basic survey kept with the project, and the registration (compared only
/// for the survey date).
const NON_LABEL_BLOCKS: &[&str] = &["registration", "maatwerkadvies", "basisopname"];

/// Window and glazing properties: one-to-one replacement of glazing (6a,
/// W p. 67, U p. 58).
const GLAZING_KEYS: &[&str] = &["gValue", "gGl", "gglN", "gPerpendicular", "frameFraction"];

/// Keys of the share a generator covers; with an added generator they
/// follow the system change and need review.
const SHARE_MARKERS: &[&str] = &["fraction", "coverage", "share"];

/// Whether a change path belongs to building-bound production (PV or
/// solar thermal); such relabels need photos with shading and proof that
/// the panels serve this building (BRL 9500-W p. 23, U p. 19).
pub fn is_production_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains("solar")
        || segments(path).iter().any(|segment| {
            let segment = segment.to_ascii_lowercase();
            segment == "pv" || segment.starts_with("pvsystem") || segment == "onsiteproduction"
        })
}

/// Service of a subsystem change, from the path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Service {
    Ventilation,
    HotWater,
    Heating,
    Cooling,
}

fn service_of(path: &str) -> Option<Service> {
    let lower = path.to_ascii_lowercase();
    if lower.contains("ventilation") {
        Some(Service::Ventilation)
    } else if lower.contains("hotwater") || lower.contains("tapwater") || lower.contains("dhw") {
        Some(Service::HotWater)
    } else if lower.contains("cooling") {
        Some(Service::Cooling)
    } else if lower.contains("heating") {
        Some(Service::Heating)
    } else {
        None
    }
}

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

fn classify(
    scheme: RelabelScheme,
    path: &str,
    before: Option<&Value>,
    after: Option<&Value>,
) -> RelabelChange {
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
    // An opening (window, door, panel) added, removed or resized inside a
    // surface: W 6a allows a layout change of insulation at an equal loss
    // area per orientation (p. 67); U 6b excludes it (p. 59).
    let in_opening = parts
        .iter()
        .any(|segment| OPENING_ARRAYS.contains(&segment.as_str()));
    let opening_layout =
        (element_added_or_removed && last_is_index && OPENING_ARRAYS.contains(&key.as_str()))
            || (in_opening && GEOMETRY_KEYS.contains(&key.as_str()) && key != "orientation");
    if opening_layout {
        return match scheme {
            RelabelScheme::W => change(
                RelabelVerdict::Review,
                "geometric change of insulation: layout of windows, doors or panels",
                Some("allowed (6a) when the loss area per orientation stays equal, otherwise 6b"),
            ),
            RelabelScheme::U => change(
                RelabelVerdict::NotAllowed,
                "geometric change of insulation: layout of windows, doors or panels",
                None,
            ),
        };
    }
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
    // PV or solar thermal resized, tilted or turned: building-bound
    // production, 6a for a one-to-one replacement and 6b for a system
    // change (W p. 67–68, U p. 58–59); not an area of the building.
    if is_production_path(path) && GEOMETRY_KEYS.contains(&key.as_str()) {
        return change(
            RelabelVerdict::Review,
            "building-bound production: size, tilt or orientation changed",
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
        return match scheme {
            RelabelScheme::W => change(
                RelabelVerdict::Review,
                "shading, overhang or obstruction",
                Some("listed under cooling in both 6a (p. 67) and 6b (p. 68); the adviser decides"),
            ),
            RelabelScheme::U => change(
                RelabelVerdict::NotAllowed,
                "geometric change of the installation: shading, overhang or obstruction",
                None,
            ),
        };
    }
    if contains_any(path, LIGHTING_MARKERS) {
        return change(
            RelabelVerdict::Review,
            "lighting",
            Some("lighting is not listed in Bijlage 6a or 6b; the adviser decides"),
        );
    }
    if contains_any(path, SUBSYSTEM_MARKERS) {
        return match scheme {
            RelabelScheme::W => {
                // Bijlage 6a (p. 67) per service: ventilation and hot water
                // only the emission system; heating and cooling also
                // distribution and control.
                let lower = path.to_ascii_lowercase();
                let emission = lower.contains("emission");
                let listed = match service_of(path) {
                    Some(Service::Ventilation | Service::HotWater) => emission,
                    Some(Service::Heating | Service::Cooling) => true,
                    None => false,
                };
                if listed {
                    change(
                        RelabelVerdict::Allowed,
                        "change in distribution, emission or control",
                        None,
                    )
                } else {
                    change(
                        RelabelVerdict::Review,
                        "change in distribution, emission or control not listed for this service",
                        Some("Bijlage 6a lists only the emission system for ventilation and hot water; the adviser decides"),
                    )
                }
            }
            RelabelScheme::U => change(
                RelabelVerdict::NotAllowed,
                "change in distribution, emission or control",
                None,
            ),
        };
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
    if in_opening && GLAZING_KEYS.contains(&key.as_str()) {
        return change(
            RelabelVerdict::Allowed,
            "one-to-one replacement or change: glazing properties",
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

fn diff(
    scheme: RelabelScheme,
    path: &str,
    before: Option<&Value>,
    after: Option<&Value>,
    out: &mut Vec<RelabelChange>,
) {
    match (before, after) {
        (Some(Value::Object(a)), Some(Value::Object(b))) => {
            let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
            keys.sort();
            keys.dedup();
            for key in keys {
                if IGNORED_KEYS.contains(&key.as_str())
                    || key.ends_with("Reference")
                    || (path.is_empty() && NON_LABEL_BLOCKS.contains(&key.as_str()))
                {
                    continue;
                }
                let child = format!("{path}/{}", key.replace('~', "~0").replace('/', "~1"));
                diff(scheme, &child, a.get(key), b.get(key), out);
            }
        }
        (Some(Value::Array(a)), Some(Value::Array(b))) => {
            for index in 0..a.len().max(b.len()) {
                diff(
                    scheme,
                    &format!("{path}/{index}"),
                    a.get(index),
                    b.get(index),
                    out,
                );
            }
        }
        (Some(a), Some(b)) if a == b => {}
        (None, None) => {}
        _ => out.push(classify(scheme, path, before, after)),
    }
}

/// A generator added next to an existing one (a hybrid heat pump beside the
/// boiler) is a system change (6b) or at least needs review; the shares the
/// generators cover then follow that change and cannot be counted as a
/// one-to-one change on their own (W p. 67–68).
fn review_shares_next_to_added_generators(changes: &mut [RelabelChange]) {
    let top = |path: &str| segments(path).into_iter().next().unwrap_or_default();
    let added: Vec<String> = changes
        .iter()
        .filter(|change| {
            (change.before.is_none() || change.after.is_none())
                && contains_any(&change.path, INSTALLATION_MARKERS)
        })
        .map(|change| top(&change.path))
        .collect();
    if added.is_empty() {
        return;
    }
    for change in changes.iter_mut() {
        let key = segments(&change.path)
            .into_iter()
            .rev()
            .find(|segment| segment.parse::<usize>().is_err())
            .unwrap_or_default();
        if change.verdict == RelabelVerdict::Allowed
            && contains_any(&key, SHARE_MARKERS)
            && added.contains(&top(&change.path))
        {
            change.verdict = RelabelVerdict::Review;
            change.cluster = "installation: share of a generator next to an added generator";
            change.note =
                Some("follows the added generator, a system change (6b); the adviser decides");
        }
    }
}

/// Classifies every difference between the original and the current
/// project per Bijlage 6a/6b. The registration block is compared only for
/// the survey date, which must stay the original one (§4.2.4).
pub fn assess_relabel(original: &Value, current: &Value) -> RelabelAssessment {
    let scheme = match current
        .get("buildingFunction")
        .or_else(|| original.get("buildingFunction"))
        .and_then(Value::as_str)
    {
        Some(function) if function != "residential" => RelabelScheme::U,
        _ => RelabelScheme::W,
    };
    let mut changes = Vec::new();
    diff(scheme, "", Some(original), Some(current), &mut changes);
    review_shares_next_to_added_generators(&mut changes);
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
        source: match scheme {
            RelabelScheme::W => RELABEL_SOURCE_W,
            RelabelScheme::U => RELABEL_SOURCE_U,
        },
        scheme,
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
            "/heatingSystems/0/generator/type",
            "/registration/surveyDate",
        ] {
            assert_eq!(verdict(&result, path), RelabelVerdict::NotAllowed, "{path}");
        }
    }

    #[test]
    fn opening_layout_shading_and_lighting_by_scheme() {
        // W 6a: a panel replaced by glazing at an equal loss area is a
        // layout change of insulation (p. 67): review, not rejected.
        let mut original = project();
        original["windowSolar"] = json!({"obstruction": {"method": "minimal"}});
        let mut current = original.clone();
        current["zones"][0]["surfaces"][0]["windows"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id": "w2", "area": 1.0, "uValue": 1.1}));
        current["zones"][0]["surfaces"][0]["windows"][0]["area"] = json!(2.5);
        current["windowSolar"]["obstruction"]["method"] = json!("overhang");
        current["lighting"] = json!([{"powerWPerM2": 8.0}]);
        current["heatingSystems"][0]["distribution"] = json!({"pipesInsulated": true});
        let w = assess_relabel(&original, &current);
        assert_eq!(w.scheme, RelabelScheme::W);
        assert!(w.allowed, "{:?}", w.changes);
        assert_eq!(
            verdict(&w, "/zones/0/surfaces/0/windows/1"),
            RelabelVerdict::Review
        );
        assert_eq!(
            verdict(&w, "/zones/0/surfaces/0/windows/0/area"),
            RelabelVerdict::Review
        );
        assert_eq!(
            verdict(&w, "/windowSolar/obstruction/method"),
            RelabelVerdict::Review
        );
        assert_eq!(verdict(&w, "/lighting"), RelabelVerdict::Review);
        assert_eq!(
            verdict(&w, "/heatingSystems/0/distribution"),
            RelabelVerdict::Allowed
        );
        // U: 6a covers one-to-one replacements only (p. 58); geometric and
        // distribution or emission changes are 6b (p. 59–60).
        let mut office = original.clone();
        office["buildingFunction"] = json!("office");
        current["buildingFunction"] = json!("office");
        let u = assess_relabel(&office, &current);
        assert_eq!(u.scheme, RelabelScheme::U);
        assert!(u.source.contains("9500-U"));
        assert!(!u.allowed);
        for path in [
            "/zones/0/surfaces/0/windows/1",
            "/windowSolar/obstruction/method",
            "/heatingSystems/0/distribution",
        ] {
            assert_eq!(verdict(&u, path), RelabelVerdict::NotAllowed, "{path}");
        }
        assert_eq!(verdict(&u, "/lighting"), RelabelVerdict::Review);
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

    #[test]
    fn floor_area_alone_is_a_geometric_change() {
        // W 6b p. 68, U 6b p. 59: a changed usable floor area.
        let mut original = project();
        original["zones"][0]["floorArea"] = json!(100.0);
        let mut current = original.clone();
        current["zones"][0]["floorArea"] = json!(104.0);
        let result = assess_relabel(&original, &current);
        assert!(!result.allowed);
        assert_eq!(
            verdict(&result, "/zones/0/floorArea"),
            RelabelVerdict::NotAllowed
        );
    }

    #[test]
    fn pv_size_and_position_need_review_and_glazing_g_is_allowed() {
        let mut original = project();
        original["solarPV"] =
            json!([{"id": "pv1", "area": 10.0, "tilt": 30.0, "orientation": "S"}]);
        let mut current = original.clone();
        current["solarPV"][0]["area"] = json!(14.0);
        current["solarPV"][0]["tilt"] = json!(35.0);
        current["solarPV"][0]["orientation"] = json!("SW");
        current["zones"][0]["surfaces"][0]["windows"][0]["gValue"] = json!(0.4);
        let result = assess_relabel(&original, &current);
        assert!(result.allowed, "{:?}", result.changes);
        for path in [
            "/solarPV/0/area",
            "/solarPV/0/tilt",
            "/solarPV/0/orientation",
        ] {
            assert_eq!(verdict(&result, path), RelabelVerdict::Review, "{path}");
        }
        assert_eq!(
            verdict(&result, "/zones/0/surfaces/0/windows/0/gValue"),
            RelabelVerdict::Allowed
        );
        assert!(is_production_path("/solarPV/0/area"));
        assert!(!is_production_path("/zones/0/surfaces/0/windows/0/gValue"));
    }

    #[test]
    fn dwelling_subsystem_changes_follow_the_service() {
        // W 6a p. 67: ventilation and hot water only the emission system.
        let mut original = project();
        original["ventilationSystems"] =
            json!([{"distribution": {"ducts": "a"}, "emission": {"grilles": "a"}}]);
        original["hotWaterSystems"] =
            json!([{"distribution": {"loop": false}, "emission": {"taps": 1}}]);
        original["coolingSystems"] = json!([{"control": {"kind": "a"}}]);
        let mut current = original.clone();
        current["ventilationSystems"][0]["distribution"]["ducts"] = json!("b");
        current["ventilationSystems"][0]["emission"]["grilles"] = json!("b");
        current["hotWaterSystems"][0]["distribution"]["loop"] = json!(true);
        current["hotWaterSystems"][0]["emission"]["taps"] = json!(2);
        current["coolingSystems"][0]["control"]["kind"] = json!("b");
        let result = assess_relabel(&original, &current);
        assert_eq!(
            verdict(&result, "/ventilationSystems/0/distribution/ducts"),
            RelabelVerdict::Review
        );
        assert_eq!(
            verdict(&result, "/ventilationSystems/0/emission/grilles"),
            RelabelVerdict::Allowed
        );
        assert_eq!(
            verdict(&result, "/hotWaterSystems/0/distribution/loop"),
            RelabelVerdict::Review
        );
        assert_eq!(
            verdict(&result, "/hotWaterSystems/0/emission/taps"),
            RelabelVerdict::Allowed
        );
        assert_eq!(
            verdict(&result, "/coolingSystems/0/control/kind"),
            RelabelVerdict::Allowed
        );
    }

    #[test]
    fn advice_and_survey_blocks_are_not_compared() {
        let mut original = project();
        original["maatwerkadvies"] = json!({"measures": []});
        original["basisopname"] = json!({"kind": "residential", "survey": {"a": 1}});
        let mut current = original.clone();
        current["maatwerkadvies"]["measures"] = json!([{"id": "m1"}]);
        current["basisopname"]["survey"]["a"] = json!(2);
        let result = assess_relabel(&original, &current);
        assert!(result.changes.is_empty(), "{:?}", result.changes);
    }

    #[test]
    fn share_next_to_an_added_generator_needs_review() {
        let mut original = project();
        original["heatingSystems"][0]["coverageFraction"] = json!(1.0);
        let mut current = original.clone();
        current["heatingSystems"][0]["coverageFraction"] = json!(0.4);
        current["heatingSystems"]
            .as_array_mut()
            .unwrap()
            .push(json!({"generator": {"type": "heat_pump_hybrid"}, "coverageFraction": 0.6}));
        let result = assess_relabel(&original, &current);
        assert_eq!(
            verdict(&result, "/heatingSystems/1"),
            RelabelVerdict::Review
        );
        assert_eq!(
            verdict(&result, "/heatingSystems/0/coverageFraction"),
            RelabelVerdict::Review
        );
        // Without an added generator a share change stays one-to-one.
        let mut alone = original.clone();
        alone["heatingSystems"][0]["coverageFraction"] = json!(0.9);
        let result = assess_relabel(&original, &alone);
        assert_eq!(
            verdict(&result, "/heatingSystems/0/coverageFraction"),
            RelabelVerdict::Allowed
        );
    }

    #[test]
    fn utility_source_cites_the_relabel_paragraph() {
        assert!(RELABEL_SOURCE_U.contains("§4.2.4 (p. 19–20)"));
    }
}
