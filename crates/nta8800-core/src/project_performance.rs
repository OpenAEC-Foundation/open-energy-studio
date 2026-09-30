//! Adapter from a `.oes` project to the unverified building-performance
//! chain. Geometry, windows, constructions, thermal bridges and unheated
//! spaces come from the project; everything the project model does not hold
//! comes from the strict `ntaCalculation` block, each with a source
//! reference. Missing data is reported as input gaps instead of defaults.

use crate::building_performance::{
    assess_building_performance, BuildingPerformanceAssessment, BuildingPerformanceInput,
    DeclaredRenewableHeat, DeclaredUse, HeatPumpRenewableEvidence, OnSiteProduction,
};
use crate::climate::Orientation;
use crate::ground::SlabOnGround;
use crate::heating_emission::EmissionInput;
use crate::indicators_draft::CalculationScope;
use crate::monthly_demand::{
    ComponentTransmission, InternalGains, MonthlyDemandInput, OpaqueElement, Setpoints,
    ThermalMass, Transmission, VentilationFlow, Window,
};
use crate::pv::PvSystem;
use crate::space_heating_chain::{Distribution, Generator, SpaceHeatingChainInput};
use crate::{
    direct_boundary_input, input_fingerprint, unheated_project_input, ProjectInput,
    ThermalBoundary, KERNEL_VERSION, TARGET_NORM_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NtaCalculationInput {
    pub calculation_scope: CalculationScope,
    pub area_source_reference: String,
    pub setpoints: Setpoints,
    pub thermal_mass: ThermalMass,
    pub internal_gains: InternalGains,
    #[serde(default)]
    pub surface_tilts: Vec<SurfaceTilt>,
    pub window_solar: WindowSolarDefaults,
    #[serde(default)]
    pub ground_floors: Vec<GroundFloorData>,
    pub ventilation_flows: Vec<VentilationFlow>,
    pub emission: EmissionInput,
    pub distribution: Distribution,
    pub generator: Generator,
    #[serde(default)]
    pub heat_pump_renewable: Option<HeatPumpRenewableEvidence>,
    pub bacs_factor: f64,
    pub bacs_source_reference: String,
    pub use_inventory_complete: bool,
    #[serde(default)]
    pub declared_uses: Vec<DeclaredUse>,
    #[serde(default)]
    pub declared_renewable_heat: Vec<DeclaredRenewableHeat>,
    pub production_inventory_complete: bool,
    #[serde(default)]
    pub on_site_production: Vec<OnSiteProduction>,
    #[serde(default)]
    pub pv_systems: Vec<PvSystem>,
    pub demand_uses_fixed_c1_ventilation: bool,
    pub battery_storage_present: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurfaceTilt {
    pub surface_id: String,
    pub tilt_deg: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowSolarDefaults {
    pub frame_fraction: f64,
    pub obstruction_factor: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundFloorData {
    pub surface_id: String,
    pub exposed_perimeter_m: f64,
    pub construction_resistance_m2k_per_w: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputGap {
    pub code: &'static str,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPerformanceAssessment {
    pub status: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub attest_status: &'static str,
    pub gaps: Vec<InputGap>,
    pub derived_input: Option<BuildingPerformanceInput>,
    pub performance: Option<BuildingPerformanceAssessment>,
}

fn gap(code: &'static str, path: impl Into<String>) -> InputGap {
    InputGap {
        code,
        path: path.into(),
        detail: None,
    }
}

fn orientation(value: &str) -> Option<Option<Orientation>> {
    Some(match value {
        "N" => Some(Orientation::North),
        "NE" => Some(Orientation::NorthEast),
        "E" => Some(Orientation::East),
        "SE" => Some(Orientation::SouthEast),
        "S" => Some(Orientation::South),
        "SW" => Some(Orientation::SouthWest),
        "W" => Some(Orientation::West),
        "NW" => Some(Orientation::NorthWest),
        "horizontal" => None,
        _ => return None,
    })
}

pub fn assess_project_performance(project_value: &Value) -> ProjectPerformanceAssessment {
    let fingerprint = input_fingerprint(project_value);
    let mut gaps = Vec::new();
    let derived = derive_input(project_value, &mut gaps);
    let performance = derived.as_ref().map(assess_building_performance);
    let status = match (&derived, &performance) {
        (None, _) => "incomplete",
        (Some(_), Some(result)) if result.status == "calculated_unverified" => {
            "calculated_unverified"
        }
        _ => "invalid",
    };
    ProjectPerformanceAssessment {
        status,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        attest_status: "unattested",
        gaps,
        derived_input: derived,
        performance,
    }
}

fn derive_input(
    project_value: &Value,
    gaps: &mut Vec<InputGap>,
) -> Option<BuildingPerformanceInput> {
    let project: ProjectInput = match serde_json::from_value(project_value.clone()) {
        Ok(project) => project,
        Err(_) => {
            gaps.push(gap("project_shape_invalid", "project"));
            return None;
        }
    };
    let nta: Option<NtaCalculationInput> = match project_value.get("ntaCalculation") {
        None | Some(Value::Null) => {
            gaps.push(gap("nta_calculation_block_missing", "ntaCalculation"));
            None
        }
        Some(value) => {
            match serde_path_to_error::deserialize::<_, NtaCalculationInput>(value.clone()) {
                Ok(block) => Some(block),
                Err(error) => {
                    gaps.push(InputGap {
                        detail: Some(error.to_string()),
                        ..gap("nta_calculation_block_invalid", "ntaCalculation")
                    });
                    None
                }
            }
        }
    };
    if project.zones.len() != 1 {
        gaps.push(gap("single_zone_required", "zones"));
    }
    let nta = nta?;
    let zone = project.zones.first()?;

    let constructions: HashMap<&str, &Value> = project
        .constructions
        .iter()
        .filter_map(|item| Some((item.get("id")?.as_str()?, item)))
        .collect();
    let tilts: HashMap<&str, f64> = nta
        .surface_tilts
        .iter()
        .map(|item| (item.surface_id.as_str(), item.tilt_deg))
        .collect();
    let ground_data: HashMap<&str, &GroundFloorData> = nta
        .ground_floors
        .iter()
        .map(|item| (item.surface_id.as_str(), item))
        .collect();
    let mut used_ground = HashSet::new();
    let mut windows = Vec::new();
    let mut opaque = Vec::new();
    let mut ground_floors = Vec::new();

    for (index, surface) in zone.surfaces.iter().enumerate() {
        let path = format!("zones[0].surfaces[{index}]");
        let id = surface.get("id").and_then(Value::as_str).unwrap_or("");
        let boundary = surface
            .get("thermalBoundary")
            .and_then(|value| serde_json::from_value::<ThermalBoundary>(value.clone()).ok());
        let Some(boundary) = boundary else {
            gaps.push(gap(
                "surface_boundary_missing",
                format!("{path}.thermalBoundary"),
            ));
            continue;
        };
        let surface_type = surface.get("type").and_then(Value::as_str).unwrap_or("");
        match boundary {
            ThermalBoundary::Ground => {
                match ground_data.get(id) {
                    Some(data) => {
                        used_ground.insert(id.to_owned());
                        ground_floors.push(SlabOnGround {
                            id: id.to_owned(),
                            area_m2: surface.get("area").and_then(Value::as_f64).unwrap_or(0.0),
                            exposed_perimeter_m: data.exposed_perimeter_m,
                            construction_resistance_m2k_per_w: data
                                .construction_resistance_m2k_per_w,
                            source_reference: data.source_reference.clone(),
                        });
                    }
                    None => gaps.push(gap("ground_floor_data_missing", format!("{path}.id"))),
                }
                continue;
            }
            ThermalBoundary::Outdoor => {}
            _ => continue,
        }
        // Outdoor surface: solar gains for windows and the opaque rest.
        let orientation_value = surface
            .get("orientation")
            .and_then(Value::as_str)
            .unwrap_or("");
        let Some(surface_orientation) = orientation(orientation_value) else {
            gaps.push(gap(
                "surface_orientation_invalid",
                format!("{path}.orientation"),
            ));
            continue;
        };
        let tilt = match (tilts.get(id), surface_orientation, surface_type) {
            (Some(tilt), _, _) => *tilt,
            (None, None, _) => 0.0,
            (None, Some(_), "wall") => 90.0,
            (None, Some(_), _) => {
                gaps.push(gap("surface_tilt_missing", format!("{path}.id")));
                continue;
            }
        };
        let azimuth_orientation = surface_orientation.unwrap_or(Orientation::South);
        let mut window_area = 0.0;
        for (window_index, window) in surface
            .get("windows")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            let window_path = format!("{path}.windows[{window_index}]");
            let (Some(window_id), Some(area), Some(u_value), Some(g_value)) = (
                window.get("id").and_then(Value::as_str),
                window.get("area").and_then(Value::as_f64),
                window.get("uValue").and_then(Value::as_f64),
                window.get("gValue").and_then(Value::as_f64),
            ) else {
                gaps.push(gap("window_data_missing", window_path));
                continue;
            };
            window_area += area;
            windows.push(Window {
                id: format!("window:{window_id}"),
                area_m2: area,
                orientation: azimuth_orientation,
                tilt_deg: tilt,
                g_perpendicular: g_value,
                frame_fraction: nta.window_solar.frame_fraction,
                u_value_w_per_m2k: u_value,
                obstruction_factor: nta.window_solar.obstruction_factor,
                source_reference: format!(
                    "project:window:{window_id}; {}",
                    nta.window_solar.source_reference
                ),
            });
        }
        let gross = surface.get("area").and_then(Value::as_f64).unwrap_or(0.0);
        let opaque_area = gross - window_area;
        if opaque_area > 1e-9 {
            let construction_id = surface
                .get("constructionId")
                .and_then(Value::as_str)
                .unwrap_or("");
            let u_value = constructions
                .get(construction_id)
                .and_then(|item| item.get("uValue"))
                .and_then(Value::as_f64);
            match u_value {
                Some(u_value) => opaque.push(OpaqueElement {
                    id: format!("surface:{id}:opaque"),
                    area_m2: opaque_area,
                    orientation: azimuth_orientation,
                    tilt_deg: tilt,
                    u_value_w_per_m2k: u_value,
                    source_reference: format!("project:construction:{construction_id}.uValue"),
                }),
                None => gaps.push(gap(
                    "construction_u_value_missing",
                    format!("{path}.constructionId"),
                )),
            }
        }
    }
    for (index, item) in nta.ground_floors.iter().enumerate() {
        if !used_ground.contains(&item.surface_id) {
            gaps.push(gap(
                "ground_floor_data_without_ground_surface",
                format!("ntaCalculation.groundFloors[{index}].surfaceId"),
            ));
        }
    }
    let direct = direct_boundary_input(&project, ThermalBoundary::Outdoor, None);
    if direct.is_none() {
        gaps.push(gap("direct_transmission_unresolved", "zones[0]"));
    }
    let unheated = if project.unheated_spaces.is_empty() {
        None
    } else {
        let resolved = unheated_project_input(&project);
        if resolved.is_none() {
            gaps.push(gap("unheated_transmission_unresolved", "unheatedSpaces"));
        }
        resolved
    };
    if !gaps.is_empty() {
        return None;
    }
    let demand = MonthlyDemandInput {
        zone_id: zone.id.clone(),
        usable_floor_area_m2: zone.floor_area,
        area_source_reference: nta.area_source_reference.clone(),
        setpoints: nta.setpoints,
        transmission: Transmission::Components(ComponentTransmission {
            direct: direct?,
            unheated,
            ground_floors,
            ground_inventory_confirmed: true,
        }),
        ventilation_flows: nta.ventilation_flows,
        thermal_mass: nta.thermal_mass,
        internal_gains: nta.internal_gains,
        window_inventory_complete: true,
        windows,
        opaque_inventory_complete: true,
        opaque_elements: opaque,
    };
    Some(BuildingPerformanceInput {
        calculation_scope: nta.calculation_scope,
        total_usable_floor_area_m2: zone.floor_area,
        area_source_reference: nta.area_source_reference,
        space_heating: SpaceHeatingChainInput {
            demand,
            emission: nta.emission,
            distribution: nta.distribution,
            generator: nta.generator,
        },
        heat_pump_renewable: nta.heat_pump_renewable,
        bacs_factor: nta.bacs_factor,
        bacs_source_reference: nta.bacs_source_reference,
        use_inventory_complete: nta.use_inventory_complete,
        declared_uses: nta.declared_uses,
        declared_renewable_heat: nta.declared_renewable_heat,
        production_inventory_complete: nta.production_inventory_complete,
        on_site_production: nta.on_site_production,
        pv_systems: nta.pv_systems,
        demand_uses_fixed_c1_ventilation: nta.demand_uses_fixed_c1_ventilation,
        battery_storage_present: nta.battery_storage_present,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> Value {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap()
    }

    #[test]
    fn complete_project_reaches_unverified_indicators() {
        let result = assess_project_performance(&project());
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        let derived = result.derived_input.as_ref().unwrap();
        let demand = &derived.space_heating.demand;
        assert_eq!(demand.windows.len(), 2);
        // 4 walls + roof as opaque solar elements.
        assert_eq!(demand.opaque_elements.len(), 5);
        let roof = demand
            .opaque_elements
            .iter()
            .find(|item| item.id == "surface:roof:opaque")
            .unwrap();
        assert_eq!(roof.tilt_deg, 45.0);
        let performance = result.performance.as_ref().unwrap();
        let summary = performance
            .space_heating
            .demand
            .transmission
            .as_ref()
            .unwrap();
        // Walls 110 m² minus 12 m² glass at 0,21, roof 52 m² at 0,16, glass 12 m² at 1,1, bridges 1,5 W/K.
        let expected = 98.0 * 0.21 + 52.0 * 0.16 + 12.0 * 1.1 + 30.0 * 0.05;
        assert!((summary.conductance_w_per_k - expected).abs() < 1e-9);
        assert!(summary.ground_conductance_w_per_k > 0.0);
        assert!(performance
            .primary_fossil_indicator_kwh_per_m2_year
            .is_some());
        assert_eq!(result.attest_status, "unattested");
    }

    #[test]
    fn missing_block_and_data_are_reported_as_gaps() {
        let mut value = project();
        value.as_object_mut().unwrap().remove("ntaCalculation");
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "incomplete");
        assert!(result.performance.is_none());
        assert!(result
            .gaps
            .iter()
            .any(|item| item.code == "nta_calculation_block_missing"));

        let mut value = project();
        value["ntaCalculation"]["surfaceTilts"] = serde_json::json!([]);
        value["ntaCalculation"]["groundFloors"] = serde_json::json!([]);
        value["zones"][0]["surfaces"][1]
            .as_object_mut()
            .unwrap()
            .remove("thermalBoundary");
        let result = assess_project_performance(&value);
        let codes: Vec<_> = result.gaps.iter().map(|item| item.code).collect();
        for code in [
            "surface_tilt_missing",
            "ground_floor_data_missing",
            "surface_boundary_missing",
            "direct_transmission_unresolved",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    #[test]
    fn strict_block_rejects_unknown_fields() {
        let mut value = project();
        value["ntaCalculation"]["heatingSetpoint"] = serde_json::json!(21.0);
        let result = assess_project_performance(&value);
        let gap = result
            .gaps
            .iter()
            .find(|item| item.code == "nta_calculation_block_invalid")
            .unwrap();
        assert!(gap.detail.as_deref().unwrap().contains("heatingSetpoint"));
    }

    #[test]
    fn block_errors_name_the_exact_field() {
        let mut value = project();
        value["ntaCalculation"]["ventilationFlows"][0]["months"][3]["conductanceWPerK"] =
            Value::Null;
        let result = assess_project_performance(&value);
        let detail = result.gaps[0].detail.as_deref().unwrap();
        assert!(
            detail.contains("ventilationFlows[0].months[3].conductanceWPerK"),
            "{detail}"
        );
    }
}
