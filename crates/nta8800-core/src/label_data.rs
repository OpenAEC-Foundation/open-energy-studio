//! Summary of the label data of Regeling energieprestatie gebouwen art. 4
//! (p. 5–6): a. the general building data (use function, construction
//! year, usable floor area and, for dwellings, the dwelling type), b. the
//! insulation per element type and c. the installations including the
//! solar water heater, next to the indicators (d) that the building
//! assessment already reports.
//!
//! The summary is derived from the project and the derived kernel input; it
//! adds no calculation.

use crate::building_performance::BuildingPerformanceInput;
use crate::{ProjectInput, ThermalBoundary};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementCategory {
    Facade,
    Roof,
    Floor,
    Glazing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopeSummary {
    pub category: ElementCategory,
    pub area_m2: f64,
    /// Area-weighted U of the elements with a known U, W/(m²·K).
    pub mean_u_w_per_m2k: Option<f64>,
    /// Lowest and highest R_c of the opaque constructions, m²K/W.
    pub min_rc_m2k_per_w: Option<f64>,
    pub max_rc_m2k_per_w: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationSummary {
    /// Kind of the space-heating generator (kernel tag).
    pub heating_generator: Option<String>,
    pub hot_water_generator: Option<String>,
    /// Ventilation system variant per zone (table 11.5).
    pub ventilation_systems: Vec<String>,
    pub cooling_generators: Vec<String>,
    pub pv_system_count: usize,
    pub lighting_zone_count: usize,
    /// Solar water heaters (zonneboiler), counted per appliance.
    pub solar_water_heater_count: u32,
    /// Their use: hot water, combi or space heating (kernel tags).
    pub solar_water_heater_uses: Vec<String>,
}

/// Art. 4 a: general building data.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralData {
    /// Use function (label function of the kernel input, else the project's
    /// building function).
    pub use_function: Option<String>,
    pub construction_year: Option<u32>,
    pub usable_floor_area_m2: Option<f64>,
    /// Dwelling type, for a dwelling or residential building only.
    pub dwelling_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelData {
    pub source: &'static str,
    pub general: GeneralData,
    pub envelope: Vec<EnvelopeSummary>,
    pub installations: InstallationSummary,
}

pub const LABEL_DATA_SOURCE: &str =
    "Regeling energieprestatie gebouwen art. 4 (gegevens op het energielabel)";

#[derive(Default)]
struct Accumulator {
    area: f64,
    ua: f64,
    u_area: f64,
    rc: Vec<f64>,
}

/// Envelope summary: surfaces to outside air, ground and unheated spaces.
pub fn envelope_summary(project: &ProjectInput) -> Vec<EnvelopeSummary> {
    let constructions: HashMap<&str, &Value> = project
        .constructions
        .iter()
        .filter_map(|item| Some((item.get("id")?.as_str()?, item)))
        .collect();
    let order = [
        ElementCategory::Facade,
        ElementCategory::Roof,
        ElementCategory::Floor,
        ElementCategory::Glazing,
    ];
    let mut totals: Vec<Accumulator> = order.iter().map(|_| Accumulator::default()).collect();
    let slot = |category: ElementCategory| order.iter().position(|item| *item == category);
    for surface in project.zones.iter().flat_map(|zone| &zone.surfaces) {
        let boundary: Option<ThermalBoundary> = surface
            .get("thermalBoundary")
            .and_then(|value| serde_json::from_value(value.clone()).ok());
        if !matches!(
            boundary,
            Some(
                ThermalBoundary::Outdoor | ThermalBoundary::Ground | ThermalBoundary::UnheatedSpace
            )
        ) {
            continue;
        }
        let category = match surface.get("type").and_then(Value::as_str) {
            Some("wall") => ElementCategory::Facade,
            Some("roof") => ElementCategory::Roof,
            Some("floor") => ElementCategory::Floor,
            _ => continue,
        };
        let gross = surface.get("area").and_then(Value::as_f64).unwrap_or(0.0);
        let mut window_area = 0.0;
        for window in surface
            .get("windows")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let area = window.get("area").and_then(Value::as_f64).unwrap_or(0.0);
            if !(area.is_finite() && area > 0.0) {
                continue;
            }
            window_area += area;
            let total = &mut totals[slot(ElementCategory::Glazing).unwrap_or(3)];
            total.area += area;
            if let Some(u) = window.get("uValue").and_then(Value::as_f64) {
                total.ua += u * area;
                total.u_area += area;
            }
        }
        let opaque = gross - window_area;
        if !(opaque.is_finite() && opaque > 1e-9) {
            continue;
        }
        let construction = surface
            .get("constructionId")
            .and_then(Value::as_str)
            .and_then(|id| constructions.get(id));
        let total = &mut totals[slot(category).unwrap_or(0)];
        total.area += opaque;
        if let Some(construction) = construction {
            if let Some(u) = construction.get("uValue").and_then(Value::as_f64) {
                total.ua += u * opaque;
                total.u_area += opaque;
            }
            if let Some(rc) = construction.get("rcValue").and_then(Value::as_f64) {
                if rc.is_finite() {
                    total.rc.push(rc);
                }
            }
        }
    }
    order
        .iter()
        .zip(totals)
        .filter(|(_, total)| total.area > 0.0)
        .map(|(category, total)| EnvelopeSummary {
            category: *category,
            area_m2: total.area,
            mean_u_w_per_m2k: (total.u_area > 0.0).then(|| total.ua / total.u_area),
            min_rc_m2k_per_w: total.rc.iter().copied().reduce(f64::min),
            max_rc_m2k_per_w: total.rc.iter().copied().reduce(f64::max),
        })
        .collect()
}

/// Tag of a serde-tagged kernel enum (`kind`, `method` or `type`).
fn tag<T: Serialize>(value: &T) -> Option<String> {
    let value = serde_json::to_value(value).ok()?;
    match &value {
        Value::String(text) => Some(text.clone()),
        Value::Object(map) => ["kind", "method", "type"]
            .iter()
            .find_map(|key| map.get(*key).and_then(Value::as_str).map(str::to_owned))
            .or_else(|| {
                (map.len() == 1)
                    .then(|| map.keys().next().cloned())
                    .flatten()
            }),
        _ => None,
    }
}

pub fn installation_summary(input: &BuildingPerformanceInput) -> InstallationSummary {
    let demands = std::iter::once(&input.space_heating.demand).chain(
        input
            .space_heating
            .additional_zones
            .iter()
            .map(|zone| &zone.demand),
    );
    InstallationSummary {
        heating_generator: tag(&input.space_heating.generator),
        hot_water_generator: input
            .hot_water
            .as_ref()
            .and_then(|system| tag(&system.generator)),
        ventilation_systems: demands
            .filter_map(|demand| demand.ventilation.as_ref())
            .flat_map(|ventilation| match &ventilation.system {
                crate::ventilation::VentilationSystem::Single { unit } => {
                    vec![unit.variant]
                }
                crate::ventilation::VentilationSystem::Combined {
                    decentral, other, ..
                } => vec![decentral.variant, other.variant],
            })
            .filter_map(|variant| tag(&variant))
            .collect(),
        cooling_generators: input
            .cooling
            .as_ref()
            .map(|system| {
                system
                    .generators
                    .iter()
                    .filter_map(|generator| tag(&generator.generator))
                    .collect()
            })
            .unwrap_or_default(),
        pv_system_count: input.pv_systems.len(),
        lighting_zone_count: input.lighting.len(),
        solar_water_heater_count: input
            .hot_water
            .iter()
            .flat_map(|system| &system.solar)
            .map(|heater| heater.count.max(1))
            .sum(),
        solar_water_heater_uses: input
            .hot_water
            .iter()
            .flat_map(|system| &system.solar)
            .filter_map(|heater| tag(&heater.solar_use))
            .collect(),
    }
}

/// Art. 4 a from the project, its registration block and the kernel input.
pub fn general_data(
    project: &ProjectInput,
    derived: Option<&BuildingPerformanceInput>,
) -> GeneralData {
    let registration = project.registration.as_ref();
    let text = |key: &str| {
        registration
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .filter(|value| !value.trim().is_empty())
    };
    let residential = matches!(
        project.building_function,
        crate::BuildingFunction::Residential
    );
    let zone_area: f64 = project.zones.iter().map(|zone| zone.floor_area).sum();
    GeneralData {
        use_function: derived
            .and_then(|input| input.label_function)
            .and_then(|function| tag(&function))
            .or_else(|| tag(&project.building_function)),
        construction_year: registration
            .and_then(|value| value.get("constructionYear"))
            .and_then(Value::as_u64)
            .and_then(|year| u32::try_from(year).ok()),
        usable_floor_area_m2: derived
            .map(|input| input.total_usable_floor_area_m2)
            .or((zone_area > 0.0).then_some(zone_area)),
        dwelling_type: if residential {
            text("buildingType").or_else(|| {
                derived
                    .and_then(|input| input.space_heating.demand.dwelling_type)
                    .and_then(|kind| tag(&kind))
            })
        } else {
            None
        },
    }
}

pub fn label_data(project: &ProjectInput, derived: Option<&BuildingPerformanceInput>) -> LabelData {
    LabelData {
        source: LABEL_DATA_SOURCE,
        general: general_data(project, derived),
        envelope: envelope_summary(project),
        installations: derived.map(installation_summary).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn envelope_is_summarised_per_element_type() {
        let project: ProjectInput = serde_json::from_value(json!({
            "id": "p", "name": "p", "buildingFunction": "residential",
            "constructions": [
                {"id": "wall", "uValue": 0.3, "rcValue": 3.2},
                {"id": "old", "uValue": 1.0, "rcValue": 0.8},
                {"id": "roof", "uValue": 0.2, "rcValue": 4.7}
            ],
            "zones": [{"id": "z", "floorArea": 100.0, "volume": 250.0, "surfaces": [
                {"id": "s1", "type": "wall", "area": 30.0, "constructionId": "wall",
                 "thermalBoundary": "outdoor", "windows": [{"id": "w", "area": 10.0, "uValue": 1.4}]},
                {"id": "s2", "type": "wall", "area": 20.0, "constructionId": "old",
                 "thermalBoundary": "unheated_space", "windows": []},
                {"id": "s3", "type": "roof", "area": 50.0, "constructionId": "roof",
                 "thermalBoundary": "outdoor", "windows": []},
                {"id": "s4", "type": "wall", "area": 40.0, "constructionId": "old",
                 "thermalBoundary": "adjacent_conditioned", "windows": []}
            ]}]
        }))
        .unwrap();
        let summary = envelope_summary(&project);
        assert_eq!(summary.len(), 3);
        let facade = &summary[0];
        assert_eq!(facade.category, ElementCategory::Facade);
        assert!((facade.area_m2 - 40.0).abs() < 1e-12);
        assert!((facade.mean_u_w_per_m2k.unwrap() - (0.3 * 20.0 + 20.0) / 40.0).abs() < 1e-12);
        assert_eq!(facade.min_rc_m2k_per_w, Some(0.8));
        assert_eq!(facade.max_rc_m2k_per_w, Some(3.2));
        assert_eq!(summary[1].category, ElementCategory::Roof);
        let glazing = &summary[2];
        assert_eq!(glazing.category, ElementCategory::Glazing);
        assert_eq!(glazing.mean_u_w_per_m2k, Some(1.4));
    }

    #[test]
    fn general_data_follow_article_4a() {
        let project: ProjectInput = serde_json::from_value(json!({
            "id": "p", "name": "p", "buildingFunction": "residential",
            "registration": {"constructionYear": 1975, "buildingType": "tussenwoning"},
            "zones": [{"id": "z", "floorArea": 96.0, "volume": 250.0, "surfaces": []}]
        }))
        .unwrap();
        let general = general_data(&project, None);
        assert_eq!(general.use_function.as_deref(), Some("residential"));
        assert_eq!(general.construction_year, Some(1975));
        assert_eq!(general.usable_floor_area_m2, Some(96.0));
        assert_eq!(general.dwelling_type.as_deref(), Some("tussenwoning"));
        let mut office = project.clone();
        office.building_function = crate::BuildingFunction::Office;
        assert!(general_data(&office, None).dwelling_type.is_none());
    }
}
