//! Relabelling (herlabelen): which changes between the original project
//! and the improved one may be counted.
//!
//! Sources (page numbers only, no text): BRL 9500-W 29-05-2026
//! §4.2.3/4.2.4 (p. 23–24, original survey date and software version),
//! Bijlage 6a (allowed measures, p. 67) and Bijlage 6b (measures that may
//! not be counted, p. 68); BRL 9500-U 29-05-2026 §4.2.3 (p. 18–19)
//! with §4.2.4 (p. 19–20) and Bijlagen 6a/6b (p. 58–60). For utility
//! buildings 6a covers only
//! one-to-one replacements: geometric changes of insulation or
//! installation and changes in distribution, emission or control are 6b.
//! For dwellings 6a lists the subsystem changes per service (p. 67):
//! ventilation the emission system and distribution; hot water only the
//! emission system; heating and cooling distribution, emission and
//! control. Other subsystem changes need review.
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
//!
//! A relabel is calculated with the method and software version of the
//! original survey (W §4.2.4 p. 23–24, U §4.2.4 p. 19–20), so the current
//! project must keep the original's NTA 8800 edition
//! (`ntaCalculation.normVersion`); a different edition is not allowed.

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::norm_versions::{self, NormVersion};

pub const RELABEL_SOURCE_W: &str =
    "BRL 9500-W (29-05-2026) §4.2.3–4.2.4 (p. 23–24), Bijlage 6a (p. 67) and 6b (p. 68)";
pub const RELABEL_SOURCE_U: &str =
    "BRL 9500-U (29-05-2026) §4.2.3 (p. 18–19), §4.2.4 (p. 19–20), Bijlage 6a (p. 58) and 6b (p. 59–60)";

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
    /// [`label_input_hash`] of the original project.
    pub original_label_input_hash: String,
    /// [`label_input_hash`] of the compared project; a registration whose
    /// project hashes differently has an out-of-date comparison.
    pub current_label_input_hash: String,
    /// Edition of the original project: the relabel is assessed and
    /// calculated in it (W §4.2.4 p. 23–24, U p. 19–20).
    pub norm_version: NormVersion,
    /// Label of [`Self::norm_version`].
    pub target_norm_version: &'static str,
    /// Edition of the current project; must equal `normVersion`.
    pub current_norm_version: NormVersion,
}

const NORM_VERSION_PATH: &str = "/ntaCalculation/normVersion";

/// Edition of a project (`ntaCalculation.normVersion`, default 2025+C1).
pub fn project_norm_version(project: &Value) -> NormVersion {
    project
        .pointer(NORM_VERSION_PATH)
        .filter(|value| !value.is_null())
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
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
    // Survey: the ventilation system (natural, C, D, ...).
    "principle",
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

/// Size, tilt and orientation of building-bound production in the NTA
/// input (`pvSystems`, solar collectors) and the legacy `solarThermal`.
const PRODUCTION_GEOMETRY_KEYS: &[&str] = &[
    "tiltDeg",
    "azimuthDeg",
    "moduleAreaM2",
    "collectorAreaM2",
    "collectorArea",
    "peakPower",
    // Survey: PV panel area.
    "panelAreaM2",
];

/// Project blocks that are not label input: the maatwerkadvies and the
/// basic survey kept with the project, and the registration (compared only
/// for the survey date).
/// `importLog` records which tool read data in (BRL 9501 §4.3.1); it is
/// not label input.
const NON_LABEL_BLOCKS: &[&str] = &["registration", "maatwerkadvies", "basisopname", "importLog"];

/// Window and glazing properties: one-to-one replacement of glazing (6a,
/// W p. 67, U p. 58).
const GLAZING_KEYS: &[&str] = &[
    "gValue",
    "gGl",
    "gglN",
    "gPerpendicular",
    "frameFraction",
    // Survey: glass and frame kind of a window or door.
    "glass",
    "frame",
];

/// The basic survey of a project whose label input it is: an existing
/// building (not new build) without NTA input of its own (no zones, no NTA
/// block), as the app's question flow makes it. Its answers are what a
/// relabel compares and hashes; the reasons for defaults (`inklapRedenen`)
/// explain the survey and are left out.
pub fn survey_label_input(project: &Value) -> Option<Value> {
    let purpose = project
        .pointer("/registration/purpose")
        .and_then(Value::as_str);
    if matches!(purpose, Some("delivery" | "bbl_check")) {
        return None;
    }
    let own_input = project
        .get("zones")
        .and_then(Value::as_array)
        .is_some_and(|zones| !zones.is_empty())
        || project
            .get("ntaCalculation")
            .is_some_and(|nta| !nta.is_null());
    if own_input {
        return None;
    }
    let mut survey = project.pointer("/basisopname/survey")?.clone();
    survey.as_object_mut()?.remove("inklapRedenen");
    Some(survey)
}

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
    let parts = segments(path);
    // Space heating's own subsystems sit directly in the NTA block
    // (`emission`, `distribution`, `distributionSystem`) and per zone in
    // `zoneData[i]`, without "heating" in their path.
    let nta_heating = parts.first().map(String::as_str) == Some("ntaCalculation")
        && match parts.get(1).map(String::as_str) {
            Some("emission" | "distribution" | "distributionSystem") => true,
            Some("zoneData") => matches!(
                parts.get(3).map(String::as_str),
                Some("emission" | "distribution")
            ),
            _ => false,
        };
    if nta_heating {
        Some(Service::Heating)
    } else if lower.contains("ventilation") {
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
    if is_production_path(path)
        && parts.iter().any(|segment| {
            GEOMETRY_KEYS.contains(&segment.as_str())
                || PRODUCTION_GEOMETRY_KEYS.contains(&segment.as_str())
        })
    {
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
                // Bijlage 6a (p. 67) per service: ventilation the emission
                // system and distribution, hot water only the emission
                // system, heating and cooling also distribution and control.
                let lower = path.to_ascii_lowercase();
                let emission = lower.contains("emission");
                let distribution = lower.contains("distribution");
                let listed = match service_of(path) {
                    Some(Service::Ventilation) => emission || distribution,
                    Some(Service::HotWater) => emission,
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
                        Some("Bijlage 6a lists emission and distribution for ventilation and only the emission system for hot water; the adviser decides"),
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
    let version = project_norm_version(original);
    norm_versions::with_version(version, || assess_relabel_in_edition(original, current))
}

fn assess_relabel_in_edition(original: &Value, current: &Value) -> RelabelAssessment {
    // The app leaves null members out of the NTA block before a kernel
    // call; a null and an absent member are the same input here.
    let original = &without_null_members(original);
    let current = &without_null_members(current);
    let (original_version, current_version) = (
        project_norm_version(original),
        project_norm_version(current),
    );
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
    // A survey project: its label input is the survey.
    let (survey_before, survey_after) = (survey_label_input(original), survey_label_input(current));
    if survey_before.is_some() || survey_after.is_some() {
        diff(
            scheme,
            "/basisopname/survey",
            survey_before.as_ref(),
            survey_after.as_ref(),
            &mut changes,
        );
    }
    review_shares_next_to_added_generators(&mut changes);
    // The edition is compared as an edition (absent is 2025+C1), not as
    // text: the method of the original survey applies.
    changes.retain(|change| change.path != NORM_VERSION_PATH);
    if original_version != current_version {
        changes.push(RelabelChange {
            path: NORM_VERSION_PATH.into(),
            before: Some(Value::String(original_version.id().into())),
            after: Some(Value::String(current_version.id().into())),
            verdict: RelabelVerdict::NotAllowed,
            cluster: "relabel uses the method and software version of the original survey: NTA 8800 edition changed",
            note: None,
        });
    }
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
        original_label_input_hash: label_input_hash(original),
        current_label_input_hash: label_input_hash(current),
        norm_version: original_version,
        target_norm_version: original_version.label(),
        current_norm_version: current_version,
    }
}

/// Copy of `value` without null object members, at any depth; null array
/// elements stay, as in the app's `withoutNulls`. Numbers are normalised
/// the way JavaScript reads them: a float with an integral value becomes an
/// integer (`100.0` is `100`) and `-0` becomes `0`, so a file written by
/// other tooling compares and hashes like the same file read by the app.
fn without_null_members(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(_, item)| !item.is_null())
                .map(|(key, item)| (key.clone(), without_null_members(item)))
                .collect::<Map<String, Value>>(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(without_null_members).collect()),
        Value::Number(number) => Value::Number(normalised_number(number)),
        other => other.clone(),
    }
}

/// Integral floats within the exactly representable range become integers;
/// `-0` becomes `0`.
fn normalised_number(number: &serde_json::Number) -> serde_json::Number {
    const EXACT: f64 = 9_007_199_254_740_992.0; // 2^53
    match number.as_f64() {
        Some(value) if number.is_f64() && value.fract() == 0.0 && value.abs() <= EXACT => {
            serde_json::Number::from(value as i64)
        }
        _ => number.clone(),
    }
}

/// Writes `value` as JSON with object keys sorted, independent of how the
/// map was built.
fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(key.clone()).to_string());
                out.push(':');
                write_canonical(&map[key], out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Value::Number(number) => out.push_str(&canonical_number(&number.to_string())),
        other => out.push_str(&other.to_string()),
    }
}

/// A number as `<digits>e<exponent>` with integer digits and no leading or
/// trailing zeros (`0` for zero), so Rust's and JavaScript's shortest
/// round-trip forms (`1e-7`, `0.0000001`, `1e21`, `1000000000000000000000`)
/// give the same text. The app's `labelInputSha256` writes the same form.
fn canonical_number(text: &str) -> String {
    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(at) => (
            &unsigned[..at],
            unsigned[at + 1..].parse::<i64>().unwrap_or(0),
        ),
        None => (unsigned, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let mut exponent = exponent - fraction.len() as i64;
    let digits = format!("{whole}{fraction}");
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return "0".into();
    }
    let trimmed = digits.trim_end_matches('0');
    exponent += (digits.len() - trimmed.len()) as i64;
    format!("{}{trimmed}e{exponent}", if negative { "-" } else { "" })
}

/// SHA-256 (lowercase hex) of a project's label input: the project without
/// the registration, maatwerkadvies and basic survey, null members left
/// out, object keys sorted by their UTF-8 bytes and numbers in
/// [`canonical_number`] form, so key order, nulls and number notation do
/// not change it. The app's `labelInputSha256` computes the same hash;
/// `training-data/nta8800-label-hash-cases.json` holds shared cases.
pub fn label_input_hash(project: &Value) -> String {
    let mut input = without_null_members(project);
    let survey = survey_label_input(&input);
    if let Value::Object(map) = &mut input {
        for block in NON_LABEL_BLOCKS {
            map.remove(*block);
        }
        // A survey project: the survey answers are label input.
        if let Some(survey) = survey {
            map.insert(
                "basisopname".into(),
                serde_json::json!({ "survey": survey }),
            );
        }
    }
    let mut text = String::new();
    write_canonical(&input, &mut text);
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nta_heating_subsystems_follow_bijlage_6a() {
        // W p. 67: heating distribution, emission and control are 6a; the
        // NTA block keeps them without "heating" in the path.
        let original: Value = serde_json::from_str(
            &std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../training-data/nta8800-example-terraced-dwelling.json"
            ))
            .unwrap(),
        )
        .unwrap();
        let mut current = original.clone();
        let nta = &mut current["ntaCalculation"];
        nta["emission"]["sourceReference"] = json!("offerte");
        nta["emission"]["edited"] = json!(true);
        nta["distribution"]["edited"] = json!(true);
        nta["hotWater"]["emission"]["edited"] = json!(true);
        nta["hotWater"]["generator"]["edited"] = json!(true);
        nta["ventilation"]["fans"] = json!("changed");
        let result = assess_relabel(&original, &current);
        assert_eq!(
            verdict(&result, "/ntaCalculation/emission/edited"),
            RelabelVerdict::Allowed
        );
        assert_eq!(
            verdict(&result, "/ntaCalculation/distribution/edited"),
            RelabelVerdict::Allowed
        );
        assert_eq!(
            verdict(&result, "/ntaCalculation/hotWater/emission/edited"),
            RelabelVerdict::Allowed
        );
        assert_eq!(
            service_of("/ntaCalculation/zoneData/0/emission/kind"),
            Some(Service::Heating)
        );
        assert_eq!(
            service_of("/ntaCalculation/distributionSystem/pump"),
            Some(Service::Heating)
        );
        assert_eq!(
            service_of("/ntaCalculation/ventilation/system/unit"),
            Some(Service::Ventilation)
        );
        assert_eq!(
            service_of("/ntaCalculation/hotWater/distribution/loop"),
            Some(Service::HotWater)
        );
        assert_eq!(
            service_of("/ntaCalculation/coolingSystems/0/emission"),
            Some(Service::Cooling)
        );
        assert_eq!(
            service_of("/ntaCalculation/additionalHeatingSystems/0/emission"),
            Some(Service::Heating)
        );
    }

    #[test]
    fn nta_production_geometry_needs_review() {
        let original = json!({"ntaCalculation": {"pvSystems": [{"id": "pv", "tiltDeg": 30.0,
            "azimuthDeg": 180.0, "peakPower": {"method": "panels", "panelPeakPowerW": 400.0}}]},
            "solarThermal": [{"collectorArea": 4.0}]});
        let mut current = original.clone();
        current["ntaCalculation"]["pvSystems"][0]["tiltDeg"] = json!(40.0);
        current["ntaCalculation"]["pvSystems"][0]["azimuthDeg"] = json!(200.0);
        current["ntaCalculation"]["pvSystems"][0]["peakPower"]["panelPeakPowerW"] = json!(450.0);
        current["solarThermal"][0]["collectorArea"] = json!(6.0);
        let result = assess_relabel(&original, &current);
        for path in [
            "/ntaCalculation/pvSystems/0/tiltDeg",
            "/ntaCalculation/pvSystems/0/azimuthDeg",
            "/ntaCalculation/pvSystems/0/peakPower/panelPeakPowerW",
            "/solarThermal/0/collectorArea",
        ] {
            assert_eq!(verdict(&result, path), RelabelVerdict::Review, "{path}");
        }
    }

    #[test]
    fn survey_projects_compare_their_survey_answers() {
        let survey = json!({
            "envelope": {
                "surfaces": [{"id": "dak", "element": "roof", "grossAreaM2": 40,
                    "insulation": {"kind": "none_or_unknown"}}],
                "windows": [{"id": "r1", "surfaceId": "gevel", "areaM2": 9, "glass": "double", "frame": "wood_or_plastic"}]
            },
            "heating": {"generator": {"kind": "boiler", "boilerType": "hr107"}, "emitters": "radiators", "control": "unknown"},
            "ventilation": {"principle": "natural"},
            "pv": [{"id": "pv1", "panelAreaM2": 8, "azimuthDeg": 180, "tiltDeg": 30}],
            "inklapRedenen": {"heating.generator": "typeplaat onleesbaar"}
        });
        let original = json!({"buildingFunction": "residential", "zones": [],
            "basisopname": {"kind": "residential", "survey": survey}});
        let mut current = original.clone();
        let s = &mut current["basisopname"]["survey"];
        s["envelope"]["surfaces"][0]["insulation"] =
            json!({"kind": "thickness", "thicknessMm": 120});
        s["envelope"]["windows"][0]["glass"] = json!("hr_plus_plus");
        s["heating"]["control"] = json!("weather_compensated");
        s["ventilation"]["principle"] = json!("mechanical_exhaust");
        s["pv"][0]["panelAreaM2"] = json!(16);
        s["inklapRedenen"] = json!({});
        let result = assess_relabel(&original, &current);
        let verdict = |path: &str| {
            result
                .changes
                .iter()
                .find(|change| change.path == format!("/basisopname/survey{path}"))
                .map(|change| change.verdict)
        };
        assert_eq!(
            verdict("/envelope/surfaces/0/insulation/kind"),
            Some(RelabelVerdict::Allowed)
        );
        assert_eq!(
            verdict("/envelope/surfaces/0/insulation/thicknessMm"),
            Some(RelabelVerdict::Allowed)
        );
        assert_eq!(
            verdict("/envelope/windows/0/glass"),
            Some(RelabelVerdict::Allowed)
        );
        assert_eq!(verdict("/heating/control"), Some(RelabelVerdict::Allowed));
        assert_eq!(
            verdict("/ventilation/principle"),
            Some(RelabelVerdict::NotAllowed)
        );
        assert_eq!(verdict("/pv/0/panelAreaM2"), Some(RelabelVerdict::Review));
        assert!(result
            .changes
            .iter()
            .all(|change| !change.path.contains("inklapRedenen")));
        assert!(!result.allowed);
        assert_ne!(
            result.original_label_input_hash,
            result.current_label_input_hash
        );
        // The reasons for defaults are not label input.
        let mut reasons = original.clone();
        reasons["basisopname"]["survey"]["inklapRedenen"] = json!({});
        assert_eq!(label_input_hash(&original), label_input_hash(&reasons));
        // New build: the survey kept with the project is not label input.
        let mut delivery = current.clone();
        delivery["registration"] = json!({"purpose": "delivery"});
        let mut delivery_original = original.clone();
        delivery_original["registration"] = json!({"purpose": "delivery"});
        assert!(assess_relabel(&delivery_original, &delivery)
            .changes
            .is_empty());
    }

    #[test]
    fn label_input_hash_ignores_key_order_nulls_and_registration() {
        let a = json!({"b": 1.5, "a": {"y": null, "x": [1, null]}, "registration": {"x": 1}});
        let b: Value = serde_json::from_str(r#"{"a":{"x":[1,null]},"b":1.5}"#).unwrap();
        assert_eq!(label_input_hash(&a), label_input_hash(&b));
        let c = json!({"a": {"x": [1, 2]}, "b": 1.5});
        assert_ne!(label_input_hash(&a), label_input_hash(&c));
    }

    #[test]
    fn label_input_hash_matches_the_shared_cases() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../training-data/nta8800-label-hash-cases.json"
        );
        let mut fixture: Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let write = std::env::var_os("LABEL_HASH_WRITE_FIXTURE").is_some();
        for case in fixture["cases"].as_array_mut().unwrap() {
            let hash = label_input_hash(&case["project"]);
            if write {
                case["sha256"] = json!(hash);
            } else {
                assert_eq!(case["sha256"], json!(hash), "{}", case["name"]);
            }
        }
        if write {
            std::fs::write(path, serde_json::to_string_pretty(&fixture).unwrap() + "\n").unwrap();
        }
    }

    #[test]
    fn canonical_numbers_ignore_notation() {
        for (text, expected) in [
            ("0", "0"),
            ("-0.0", "0"),
            ("100", "1e2"),
            ("100.0", "1e2"),
            ("1e-7", "1e-7"),
            ("0.0000001", "1e-7"),
            ("1e21", "1e21"),
            ("1e+21", "1e21"),
            ("1000000000000000000000", "1e21"),
            ("-12.50", "-125e-1"),
        ] {
            assert_eq!(canonical_number(text), expected, "{text}");
        }
    }

    #[test]
    fn integral_floats_and_negative_zero_are_not_changes() {
        // Files written by other tooling keep `100.0` and `-0.0`; the app
        // reads them as 100 and 0.
        let original: Value =
            serde_json::from_str(r#"{"zones":[{"floorArea":100.0,"offset":-0.0,"u":0.25}]}"#)
                .unwrap();
        let current: Value =
            serde_json::from_str(r#"{"zones":[{"floorArea":100,"offset":0,"u":0.25}]}"#).unwrap();
        let result = assess_relabel(&original, &current);
        assert!(result.changes.is_empty(), "{:?}", result.changes);
        assert_eq!(label_input_hash(&original), label_input_hash(&current));
        assert_eq!(
            result.original_label_input_hash,
            result.current_label_input_hash
        );
        let changed: Value =
            serde_json::from_str(r#"{"zones":[{"floorArea":100.5,"offset":0,"u":0.25}]}"#).unwrap();
        assert_ne!(label_input_hash(&original), label_input_hash(&changed));
    }

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
        // W 6a p. 67: ventilation emission and distribution, hot water
        // only the emission system.
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
            RelabelVerdict::Allowed
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

    #[test]
    fn import_log_is_not_label_input() {
        let project = serde_json::json!({"id": "p", "zones": [{"id": "z", "floorArea": 10.0}]});
        let mut logged = project.clone();
        logged["importLog"] = serde_json::json!([{"tool": "UNIEC3"}]);
        assert_eq!(label_input_hash(&project), label_input_hash(&logged));
    }

    /// A relabel keeps the original's edition (W §4.2.4 p. 23–24): another
    /// edition is not allowed; an absent and an explicit 2025+C1 are equal.
    #[test]
    fn relabel_keeps_the_original_edition() {
        let original = json!({
            "buildingFunction": "residential",
            "ntaCalculation": {"normVersion": "2024", "calculationScope": "residential"}
        });
        let mut same = original.clone();
        same["ntaCalculation"]["calculationScope"] = json!("residential");
        let result = assess_relabel(&original, &same);
        assert!(result.allowed);
        assert_eq!(result.norm_version, NormVersion::V2024);
        assert_eq!(result.current_norm_version, NormVersion::V2024);
        assert_eq!(result.target_norm_version, "NTA 8800:2024 met INT-V1:2024");

        let mut newer = original.clone();
        newer["ntaCalculation"]
            .as_object_mut()
            .unwrap()
            .remove("normVersion");
        let result = assess_relabel(&original, &newer);
        assert!(!result.allowed);
        let change = result
            .changes
            .iter()
            .find(|change| change.path == "/ntaCalculation/normVersion")
            .unwrap();
        assert_eq!(change.verdict, RelabelVerdict::NotAllowed);
        assert_eq!(change.before, Some(json!("2024")));
        assert_eq!(change.after, Some(json!("2025+C1")));
        assert_eq!(result.current_norm_version, NormVersion::V2025C1);

        let plain = json!({"buildingFunction": "residential", "ntaCalculation": {}});
        let explicit = json!({"buildingFunction": "residential",
            "ntaCalculation": {"normVersion": "2025+C1"}});
        let result = assess_relabel(&plain, &explicit);
        assert!(result.changes.is_empty(), "{:?}", result.changes);
        assert_eq!(result.norm_version, NormVersion::V2025C1);
    }
}
