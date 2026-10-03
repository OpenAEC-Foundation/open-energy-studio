//! Adapter from a `.oes` project to the unverified building-performance
//! chain. Geometry, windows, constructions, thermal bridges and unheated
//! spaces come from the project; everything the project model does not hold
//! comes from the strict `ntaCalculation` block, each with a source
//! reference. Missing data is reported as input gaps instead of defaults.

use crate::bbl_requirements::BblFunction;
use crate::building_performance::{
    assess_building_performance, BuildingPerformanceAssessment, BuildingPerformanceInput,
    DeclaredRenewableHeat, DeclaredUse, EnergyStorage, HeatPumpRenewableEvidence, OnSiteProduction,
};
use crate::climate::Orientation;
use crate::domestic_hot_water::HotWaterSystem;
use crate::ground::{EdgeInsulation, EdgeThermalBridges, SlabOnGround};
use crate::heating_emission::EmissionInput;
use crate::indicators_draft::CalculationScope;
use crate::label_class::LabelFunction;
use crate::monthly_demand::{
    ComponentTransmission, DwellingType, InternalGains, MonthlyDemandInput, OpaqueElement,
    Setpoints, ThermalMass, Transmission, UsageFunction, VentilationFlow, Window,
};
use crate::pv::PvSystem;
use crate::solar_shading::{MovableShading, Obstruction};
use crate::space_cooling::CoolingSystem;
use crate::space_heating_chain::{
    ChainZone, CollectiveConnection, Distribution, DistributionSystem, Generator,
    SpaceHeatingChainInput,
};
use crate::tojuli::ActiveCoolingEvidence;
use crate::{
    direct_boundary_input_zone, input_fingerprint, unheated_zone_input, ProjectInput,
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
    /// Usage function for tables 7.13–7.15.
    pub usage_function: UsageFunction,
    /// 7.78 `f_mod;sp`; required for the residential function.
    #[serde(default)]
    pub dwelling_type: Option<DwellingType>,
    pub setpoints: Setpoints,
    pub thermal_mass: ThermalMass,
    pub internal_gains: InternalGains,
    #[serde(default)]
    pub surface_tilts: Vec<SurfaceTilt>,
    pub window_solar: WindowSolarDefaults,
    /// Humidifiers per zone (chapter 12).
    #[serde(default)]
    pub humidifiers: Vec<crate::space_heating_chain::ZoneHumidifier>,
    /// Adjacent unheated sunrooms (7.30b); per zone in `zoneData`.
    #[serde(default)]
    pub sunrooms: Vec<crate::monthly_demand::Sunroom>,
    /// Vertical pipes through the envelope (7.3.3, H_p of 7.17); per zone in
    /// `zoneData`.
    #[serde(default)]
    pub vertical_pipes: Vec<crate::monthly_demand::VerticalPipe>,
    #[serde(default)]
    pub ground_floors: Vec<GroundFloorData>,
    #[serde(default)]
    pub ventilation_flows: Vec<VentilationFlow>,
    /// Chapter 11 input (single-zone projects; per zone in `zoneData`).
    #[serde(default)]
    pub ventilation: Option<crate::ventilation::VentilationInput>,
    /// Required for every zone when the project has more than one zone.
    #[serde(default)]
    pub zone_data: Vec<ZoneNtaData>,
    pub emission: EmissionInput,
    pub distribution: Distribution,
    pub generator: Generator,
    /// §9.4 hydraulic data; see `SpaceHeatingChainInput`.
    #[serde(default)]
    pub distribution_system: Option<DistributionSystem>,
    /// §9.1: number of identical physical generators the system models.
    #[serde(default)]
    pub identical_systems: Option<u32>,
    #[serde(default)]
    pub collective_connection: Option<CollectiveConnection>,
    #[serde(default)]
    pub heat_pump_renewable: Option<HeatPumpRenewableEvidence>,
    pub bacs_factor: f64,
    pub bacs_source_reference: String,
    /// §5.5.8 systems and BACS evidence; when given, f_BACS is derived and
    /// replaces `bacsFactor`.
    #[serde(default)]
    pub bacs: Option<crate::bacs_draft::BacsDraftInput>,
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
    #[serde(default)]
    pub hot_water: Option<HotWaterSystem>,
    /// §13.7 solar systems for space heating only (SHS).
    #[serde(default)]
    pub space_heating_solar: Vec<crate::solar_thermal::SolarWaterHeater>,
    #[serde(default)]
    pub lighting: Vec<crate::lighting::ZoneLighting>,
    #[serde(default)]
    pub cooling: Option<CoolingSystem>,
    #[serde(default)]
    pub label_function: Option<LabelFunction>,
    /// §5.3.1: use functions of an existing utility building with areas.
    #[serde(default)]
    pub label_functions: Vec<crate::label_class::LabelFunctionArea>,
    /// Construction year for the Standaard voor woningisolatie; falls back
    /// to `registration.constructionYear`.
    #[serde(default)]
    pub construction_year: Option<u32>,
    /// §5.5.7: fossil-fuelled building-bound appliances left out of the
    /// calculation.
    #[serde(default)]
    pub fossil_appliances_outside_calculation: Option<bool>,
    #[serde(default)]
    pub bbl_function: Option<BblFunction>,
    /// Annex AB footnote g: delivery temperature of external heat.
    #[serde(default)]
    pub zeb_heat_delivery_temperature: Option<crate::building_performance::ZebHeatTemperature>,
    /// Bbl art. 4.149 lid 2: several use functions with their areas.
    #[serde(default)]
    pub bbl_functions: Vec<crate::bbl_requirements::BblFunctionArea>,
    #[serde(default)]
    pub active_cooling: Option<ActiveCoolingEvidence>,
    #[serde(default, rename = "permitApplicationAfter20260529")]
    pub permit_application_after_2026_05_29: bool,
    pub demand_uses_fixed_c1_ventilation: bool,
    pub battery_storage_present: bool,
    #[serde(default)]
    pub storage: Option<EnergyStorage>,
    /// External heat, hot-water and cold supply with annex P values.
    #[serde(default)]
    pub external_supply: crate::building_performance::ExternalSupply,
}

/// Per-zone data for projects with more than one calculation zone. Omitted
/// optional fields fall back to the block-level values.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZoneNtaData {
    pub zone_id: String,
    /// §6.5.3 functions with areas for a mixed zone.
    #[serde(default)]
    pub function_areas: Vec<crate::monthly_demand::UsageFunctionArea>,
    #[serde(default)]
    pub sunrooms: Vec<crate::monthly_demand::Sunroom>,
    /// 7.3.3 vertical pipes of this zone; empty falls back to the project
    /// list.
    #[serde(default)]
    pub vertical_pipes: Vec<crate::monthly_demand::VerticalPipe>,
    #[serde(default)]
    pub ventilation_flows: Vec<VentilationFlow>,
    #[serde(default)]
    pub ventilation: Option<crate::ventilation::VentilationInput>,
    pub internal_gains: InternalGains,
    #[serde(default)]
    pub usage_function: Option<UsageFunction>,
    #[serde(default)]
    pub dwelling_type: Option<DwellingType>,
    #[serde(default)]
    pub setpoints: Option<Setpoints>,
    #[serde(default)]
    pub thermal_mass: Option<ThermalMass>,
    #[serde(default)]
    pub emission: Option<EmissionInput>,
    #[serde(default)]
    pub distribution: Option<Distribution>,
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
    pub obstruction: Obstruction,
    #[serde(default)]
    pub movable_shading: Option<MovableShading>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundFloorData {
    pub surface_id: String,
    pub exposed_perimeter_m: f64,
    pub construction_resistance_m2k_per_w: f64,
    /// Floor-edge bridges of 8.36 or the 8.37 forfait.
    pub edge_thermal_bridges: EdgeThermalBridges,
    #[serde(default)]
    pub edge_insulation: Vec<EdgeInsulation>,
    /// Crawlspace or unheated basement below the floor (8.3.4.2).
    #[serde(default)]
    pub below: Option<crate::ground::FloorBelow>,
    /// Heated room with its floor below ground level (8.3.3.2).
    #[serde(default)]
    pub heated_basement: Option<crate::ground::HeatedBasement>,
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
    /// Envelope geometry derived from the project alone, available even when
    /// the NTA block is incomplete.
    pub geometry: Option<GeometrySummary>,
    pub derived_input: Option<BuildingPerformanceInput>,
    pub performance: Option<BuildingPerformanceAssessment>,
    /// §6.4/§6.5.2 schematisation findings per zone (warnings).
    pub schematisation: Vec<crate::zoning::ZoningIssue>,
    /// Registration data checks (BRL 9500 §4.2.3–4.2.5); `None` when the
    /// project has no registration block.
    pub registration: Option<crate::registration::RegistrationAssessment>,
    /// §5.5.8 assessment when `ntaCalculation.bacs` is given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bacs: Option<crate::bacs_draft::BacsDraftAssessment>,
    /// Label data of Regeling energieprestatie gebouwen art. 4.
    pub label_data: Option<crate::label_data::LabelData>,
}

/// §6.4/§6.5.2 for the derived calculation zones: one heating chain and at
/// most one cooling system serve all zones in this kernel.
fn schematisation_checks(input: &BuildingPerformanceInput) -> Vec<crate::zoning::ZoningIssue> {
    use crate::zoning::{check_zone, CalculationZoneLayout, VentilationShare, ZonePart};
    std::iter::once(&input.space_heating.demand)
        .chain(
            input
                .space_heating
                .additional_zones
                .iter()
                .map(|zone| &zone.demand),
        )
        .flat_map(|demand| {
            let capacity = demand
                .thermal_mass
                .specific_capacity_kj_per_m2k(demand.usable_floor_area_m2);
            let parts = if demand.function_areas.is_empty() {
                vec![ZonePart {
                    function: demand.usage_function,
                    area_m2: demand.usable_floor_area_m2,
                    heat_capacity_kj_per_m2k: capacity,
                }]
            } else {
                demand
                    .function_areas
                    .iter()
                    .map(|part| ZonePart {
                        function: part.function,
                        area_m2: part.area_m2,
                        heat_capacity_kj_per_m2k: capacity,
                    })
                    .collect()
            };
            let ventilation = match &demand.ventilation {
                Some(ventilation) => match &ventilation.system {
                    crate::ventilation::VentilationSystem::Single { unit } => {
                        vec![VentilationShare {
                            op: unit.variant.op().into(),
                            area_m2: demand.usable_floor_area_m2,
                        }]
                    }
                    crate::ventilation::VentilationSystem::Combined {
                        decentral_area_m2,
                        total_residence_area_m2,
                        other,
                        ..
                    } => {
                        let share = decentral_area_m2 / total_residence_area_m2;
                        vec![
                            VentilationShare {
                                op: crate::zoning::VentSysOpInput::Decentral,
                                area_m2: share * demand.usable_floor_area_m2,
                            },
                            VentilationShare {
                                op: other.variant.op().into(),
                                area_m2: (1.0 - share) * demand.usable_floor_area_m2,
                            },
                        ]
                    }
                },
                None => Vec::new(),
            };
            check_zone(&CalculationZoneLayout {
                id: demand.zone_id.clone(),
                parts,
                heating_system_ids: vec!["space-heating".into()],
                cooling_system_ids: Vec::new(),
                humidification_system_ids: Vec::new(),
                ventilation,
                residence_areas_open: false,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometrySummary {
    /// `A_g;tot`: sum of the zone usable floor areas, m².
    pub usable_floor_area_m2: f64,
    /// `A_ls` (6.3, 6.7.3): outdoor air and unheated spaces weighted 1,
    /// ground and crawlspace 0,7, m².
    pub loss_area_m2: f64,
    /// Unweighted envelope area to outdoor air, ground and unheated
    /// spaces (`A_o` in ISSO 54), m².
    pub envelope_area_m2: f64,
    pub loss_area_ratio: Option<f64>,
    /// Surfaces without a thermal boundary are not counted.
    pub unclassified_surface_count: usize,
}

/// 6.7.3 weighting factor `f_ls`; `None` for boundaries that do not count.
fn loss_area_weight(boundary: ThermalBoundary) -> Option<f64> {
    match boundary {
        ThermalBoundary::Outdoor | ThermalBoundary::UnheatedSpace => Some(1.0),
        ThermalBoundary::Ground => Some(0.7),
        ThermalBoundary::AdjacentConditioned | ThermalBoundary::Internal => None,
    }
}

/// Geometry of the project without any NTA block; `None` for an unreadable project.
pub fn project_geometry(project_value: &Value) -> Option<GeometrySummary> {
    let project: ProjectInput = serde_json::from_value(project_value.clone()).ok()?;
    let mut floor = 0.0;
    let mut loss = 0.0;
    let mut envelope = 0.0;
    let mut unclassified = 0;
    for zone in &project.zones {
        floor += zone.floor_area;
        for surface in &zone.surfaces {
            match surface
                .get("thermalBoundary")
                .and_then(|value| serde_json::from_value::<ThermalBoundary>(value.clone()).ok())
            {
                Some(boundary) => {
                    if let Some(weight) = loss_area_weight(boundary) {
                        let area = surface.get("area").and_then(Value::as_f64).unwrap_or(0.0);
                        loss += weight * area;
                        envelope += area;
                    }
                }
                None => unclassified += 1,
            }
        }
    }
    Some(GeometrySummary {
        usable_floor_area_m2: floor,
        loss_area_m2: loss,
        envelope_area_m2: envelope,
        loss_area_ratio: (floor > 0.0).then(|| loss / floor),
        unclassified_surface_count: unclassified,
    })
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

/// 7.82: with exactly one unheated space in the project, its declared or
/// derived `b_U` sets ϑ_ztu of the pipes and vessels there, unless the
/// distribution system gives ϑ_ztu or `b_U` itself.
fn with_unheated_reduction(
    system: Option<crate::space_heating_chain::DistributionSystem>,
    project: &ProjectInput,
) -> Option<crate::space_heating_chain::DistributionSystem> {
    let mut system = system?;
    if system.unheated_ambient_c.is_none()
        && system.unheated_reduction_factor.is_none()
        && project.unheated_spaces.len() == 1
    {
        system.unheated_reduction_factor = crate::unheated_project_input(project)
            .map(|input| crate::unheated_transmission::assess_unheated_transmission(&input))
            .and_then(|assessed| assessed.spaces.first().map(|space| space.reduction_factor));
    }
    Some(system)
}

pub fn assess_project_performance(project_value: &Value) -> ProjectPerformanceAssessment {
    // The maatwerkadvies definition (measures, tariffs) and the kept
    // basisopname survey do not change the energy performance of the
    // project route and stay out of the fingerprint.
    let fingerprint = match (
        project_value.get("maatwerkadvies"),
        project_value.get("basisopname"),
    ) {
        (None, None) => input_fingerprint(project_value),
        _ => {
            let mut stripped = project_value.clone();
            if let Some(map) = stripped.as_object_mut() {
                map.remove("maatwerkadvies");
                map.remove("basisopname");
            }
            input_fingerprint(&stripped)
        }
    };
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
    let project: Option<ProjectInput> = serde_json::from_value(project_value.clone()).ok();
    let registration = match project_value.get("registration") {
        None | Some(Value::Null) => None,
        Some(block) => {
            match serde_path_to_error::deserialize::<_, crate::registration::Registration>(
                block.clone(),
            ) {
                Ok(registration) => Some(crate::registration::assess_project_registration(
                    &registration,
                    project_value,
                )),
                Err(error) => {
                    gaps.push(InputGap {
                        detail: Some(error.to_string()),
                        ..gap("registration_block_invalid", "registration")
                    });
                    None
                }
            }
        }
    };
    let label_data = project.as_ref().map(|project| {
        let mut data = crate::label_data::label_data(project, derived.as_ref());
        data.indicators = performance
            .as_ref()
            .filter(|result| result.status == "calculated_unverified")
            .map(crate::label_data::LabelIndicators::from_performance);
        data
    });
    let bacs = project_value
        .pointer("/ntaCalculation/bacs")
        .filter(|value| !value.is_null())
        .and_then(|value| {
            serde_json::from_value::<crate::bacs_draft::BacsDraftInput>(value.clone()).ok()
        })
        .map(|input| crate::bacs_draft::assess_bacs_draft(&input));
    ProjectPerformanceAssessment {
        status,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        attest_status: "unattested",
        registration,
        bacs,
        label_data,
        gaps,
        geometry: project_geometry(project_value),
        schematisation: derived
            .as_ref()
            .map(schematisation_checks)
            .unwrap_or_default(),
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
    if project.zones.is_empty() {
        gaps.push(gap("zone_required", "zones"));
    }
    let nta = nta?;
    let multi_zone = project.zones.len() > 1;
    if multi_zone && !nta.vertical_pipes.is_empty() {
        gaps.push(gap(
            "vertical_pipes_per_zone_required",
            "ntaCalculation.verticalPipes",
        ));
    }
    let zone_data: HashMap<&str, &ZoneNtaData> = nta
        .zone_data
        .iter()
        .map(|item| (item.zone_id.as_str(), item))
        .collect();
    for (index, item) in nta.zone_data.iter().enumerate() {
        if !project.zones.iter().any(|zone| zone.id == item.zone_id) {
            gaps.push(gap(
                "zone_data_without_zone",
                format!("ntaCalculation.zoneData[{index}].zoneId"),
            ));
        }
    }
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
    let mut loss_area = 0.0;
    let mut total_area = 0.0;
    let mut zones = Vec::new();

    for (zone_index, zone) in project.zones.iter().enumerate() {
        let zone_path = format!("zones[{zone_index}]");
        total_area += zone.floor_area;
        let data = zone_data.get(zone.id.as_str()).copied();
        if multi_zone && data.is_none() {
            gaps.push(gap("zone_data_missing", format!("{zone_path}.id")));
        }
        let mut windows = Vec::new();
        let mut opaque = Vec::new();
        let mut ground_floors = Vec::new();
        for (index, surface) in zone.surfaces.iter().enumerate() {
            let path = format!("{zone_path}.surfaces[{index}]");
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
            if let Some(weight) = loss_area_weight(boundary) {
                loss_area += weight * surface.get("area").and_then(Value::as_f64).unwrap_or(0.0);
            }
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
                                edge_thermal_bridges: data.edge_thermal_bridges.clone(),
                                edge_insulation: data.edge_insulation.clone(),
                                below: data.below.clone(),
                                heated_basement: data.heated_basement.clone(),
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
                // A horizontal floor bordering outdoor air faces down (180°).
                (None, None, "floor") => 180.0,
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
                    obstruction: nta.window_solar.obstruction.clone(),
                    movable_shading: nta.window_solar.movable_shading.clone(),
                    dynamic: None,
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
        let direct =
            direct_boundary_input_zone(&project, ThermalBoundary::Outdoor, None, Some(&zone.id));
        if direct.is_none() {
            gaps.push(gap("direct_transmission_unresolved", zone_path.clone()));
        }
        let unheated = if project.unheated_spaces.is_empty() {
            None
        } else {
            let resolved = unheated_zone_input(&project, Some(&zone.id));
            if resolved.is_none() {
                gaps.push(gap("unheated_transmission_unresolved", zone_path.clone()));
            }
            resolved.filter(|input| !input.spaces.is_empty())
        };
        let Some(direct) = direct else { continue };
        let demand = MonthlyDemandInput {
            usage_fit: None,
            zone_id: zone.id.clone(),
            usable_floor_area_m2: zone.floor_area,
            area_source_reference: nta.area_source_reference.clone(),
            usage_function: data
                .and_then(|item| item.usage_function)
                .unwrap_or(nta.usage_function),
            function_areas: data
                .map(|item| item.function_areas.clone())
                .unwrap_or_default(),
            sunrooms: match data {
                Some(item) => item.sunrooms.clone(),
                None => nta.sunrooms.clone(),
            },
            // A zone that overrides the usage function brings its own dwelling type.
            dwelling_type: match data {
                Some(item) if item.usage_function.is_some() => item.dwelling_type,
                Some(item) => item.dwelling_type.or(nta.dwelling_type),
                None => nta.dwelling_type,
            },
            setpoints: data
                .and_then(|item| item.setpoints.clone())
                .unwrap_or_else(|| nta.setpoints.clone()),
            transmission: Transmission::Components(ComponentTransmission {
                direct,
                unheated,
                ground_floors,
                ground_inventory_confirmed: true,
                // The project list serves a single zone only; a multi-zone
                // project gives the pipes per zone (no double counting).
                vertical_pipes: match data {
                    Some(item) if !item.vertical_pipes.is_empty() => item.vertical_pipes.clone(),
                    _ if !multi_zone => nta.vertical_pipes.clone(),
                    _ => Vec::new(),
                },
            }),
            ventilation_flows: data
                .map(|item| item.ventilation_flows.clone())
                .unwrap_or_else(|| nta.ventilation_flows.clone()),
            ventilation: match data {
                Some(item) => item.ventilation.clone(),
                None => nta.ventilation.clone(),
            },
            thermal_mass: data
                .and_then(|item| item.thermal_mass.clone())
                .unwrap_or_else(|| nta.thermal_mass.clone()),
            internal_gains: data
                .map(|item| item.internal_gains.clone())
                .unwrap_or_else(|| nta.internal_gains.clone()),
            window_inventory_complete: true,
            windows,
            opaque_inventory_complete: true,
            opaque_elements: opaque,
        };
        zones.push(ChainZone {
            demand,
            emission: data
                .and_then(|item| item.emission.clone())
                .unwrap_or_else(|| nta.emission.clone()),
            distribution: data
                .and_then(|item| item.distribution.clone())
                .unwrap_or_else(|| nta.distribution.clone()),
        });
    }
    for (index, item) in nta.ground_floors.iter().enumerate() {
        if !used_ground.contains(&item.surface_id) {
            gaps.push(gap(
                "ground_floor_data_without_ground_surface",
                format!("ntaCalculation.groundFloors[{index}].surfaceId"),
            ));
        }
    }
    if !gaps.is_empty() || zones.is_empty() {
        return None;
    }
    let primary = zones.remove(0);
    // §5.5.8: a derived f_BACS replaces the declared value.
    let (bacs_factor, bacs_source_reference) = match &nta.bacs {
        None => (nta.bacs_factor, nta.bacs_source_reference.clone()),
        Some(bacs)
            if matches!(
                bacs.building_use,
                crate::bacs_draft::BuildingUse::Residential
            ) != matches!(nta.calculation_scope, CalculationScope::Residential) =>
        {
            // §5.5.8: the building use follows the calculation.
            gaps.push(gap(
                "bacs_building_use_mismatch",
                "ntaCalculation.bacs.buildingUse",
            ));
            return None;
        }
        Some(bacs) => match crate::bacs_draft::assess_bacs_draft(bacs).factor {
            Some(factor) => (
                factor,
                "NTA 8800 §5.5.8 (derived from ntaCalculation.bacs)".into(),
            ),
            None => {
                gaps.push(gap("bacs_factor_undetermined", "ntaCalculation.bacs"));
                return None;
            }
        },
    };
    Some(BuildingPerformanceInput {
        hot_water_need_fit: None,
        calculation_scope: nta.calculation_scope,
        total_usable_floor_area_m2: total_area,
        area_source_reference: nta.area_source_reference,
        space_heating: SpaceHeatingChainInput {
            humidifiers: nta.humidifiers.clone(),
            solar_heating_kwh: Vec::new(),
            solar_recoverable_kwh: Vec::new(),
            hot_water_load_kwh: Vec::new(),
            demand: primary.demand,
            emission: primary.emission,
            distribution: primary.distribution,
            additional_zones: zones,
            generator: nta.generator,
            distribution_system: with_unheated_reduction(nta.distribution_system, &project),
            collective_connection: nta.collective_connection,
            identical_systems: nta.identical_systems,
        },
        heat_pump_renewable: nta.heat_pump_renewable,
        bacs_factor,
        bacs_source_reference,
        use_inventory_complete: nta.use_inventory_complete,
        declared_uses: nta.declared_uses,
        declared_renewable_heat: nta.declared_renewable_heat,
        production_inventory_complete: nta.production_inventory_complete,
        on_site_production: nta.on_site_production,
        pv_systems: nta.pv_systems,
        hot_water: nta.hot_water,
        space_heating_solar: nta.space_heating_solar,
        lighting: nta.lighting,
        cooling: nta.cooling,
        label_function: nta.label_function,
        label_functions: nta.label_functions.clone(),
        construction_year: nta.construction_year.or_else(|| {
            project_value
                .pointer("/registration/constructionYear")
                .and_then(Value::as_u64)
                .and_then(|year| u32::try_from(year).ok())
        }),
        fossil_appliances_outside_calculation: nta.fossil_appliances_outside_calculation,
        bbl_function: nta.bbl_function,
        bbl_functions: nta.bbl_functions.clone(),
        zeb_heat_delivery_temperature: nta.zeb_heat_delivery_temperature,
        active_cooling: nta.active_cooling,
        permit_application_after_2026_05_29: nta.permit_application_after_2026_05_29,
        loss_area_m2: Some(loss_area),
        loss_area_source_reference: Some(
            "derived: gross project surfaces bordering outdoor air, ground or unheated space"
                .into(),
        ),
        demand_uses_fixed_c1_ventilation: nta.demand_uses_fixed_c1_ventilation,
        battery_storage_present: nta.battery_storage_present,
        storage: nta.storage,
        external_supply: nta.external_supply,
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
    fn bacs_block_derives_the_factor_of_5_5_8() {
        let mut value = project();
        value["ntaCalculation"]["bacs"] = serde_json::json!({
            "buildingUse": "utility",
            "systemInventoryComplete": true,
            "systems": [{
                "id": "heating", "service": "heating", "sourceReference": "plant room",
                "generators": [{"id": "boiler", "nominalThermalCapacityKw": 400.0,
                                "sourceReference": "type plate"}]
            }],
            "bacs": {"present": false, "sourceReference": "inspection"}
        });
        // The synthetic project is a dwelling: a utility BACS block is a
        // contradiction (§5.5.8 follows the calculation).
        let mismatch = assess_project_performance(&value);
        assert!(mismatch
            .gaps
            .iter()
            .any(|gap| gap.code == "bacs_building_use_mismatch"));
        // As a dwelling the factor is 1,0 whatever the capacity (5.5.8).
        value["ntaCalculation"]["bacs"]["buildingUse"] = serde_json::json!("residential");
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        assert_eq!(result.derived_input.as_ref().unwrap().bacs_factor, 1.0);
        assert_eq!(result.bacs.as_ref().unwrap().factor, Some(1.0));
    }

    #[test]
    fn complete_project_reaches_unverified_indicators() {
        let result = assess_project_performance(&project());
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        // Regeling art. 4: the label data carries the calculated indicators.
        let label = result
            .label_data
            .as_ref()
            .unwrap()
            .indicators
            .as_ref()
            .unwrap();
        assert_eq!(
            label.primary_fossil_kwh_per_m2,
            result
                .performance
                .as_ref()
                .unwrap()
                .primary_fossil_indicator_kwh_per_m2_year
        );
        assert!(label.heating_need_kwh_per_m2.is_some());
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
        assert!(summary.ground_steady_conductance_w_per_k.unwrap() > 0.0);
        assert!(performance
            .primary_fossil_indicator_kwh_per_m2_year
            .is_some());
        // Walls 110 + roof 52 + 0,7 · ground floor 50 m² (6.7.3).
        assert!((derived.loss_area_m2.unwrap() - 197.0).abs() < 1e-9);
        assert_eq!(performance.tojuli.len(), 1);
        assert_eq!(performance.tojuli[0].status, "calculated_unverified");
        assert!(performance.tojuli_max_k.is_some());
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
    fn registration_and_label_data_are_reported_and_fingerprinted() {
        let mut value = project();
        let plain = assess_project_performance(&value);
        assert!(plain.registration.is_none());
        let label = plain.label_data.expect("label data");
        assert!(!label.envelope.is_empty());
        assert!(label.installations.heating_generator.is_some());
        value["registration"] = serde_json::json!({
            "purpose": "existing_building",
            "surveyDate": "2026-03-15",
            "registrationDate": "2026-07-01"
        });
        let registered = assess_project_performance(&value);
        assert_ne!(registered.input_fingerprint, plain.input_fingerprint);
        assert_eq!(registered.status, plain.status);
        let registration = registered.registration.unwrap();
        assert_eq!(registration.valid_until.as_deref(), Some("2036-03-15"));
        assert!(registration
            .issues
            .iter()
            .any(|item| item.code == "registration_deadline_exceeded"));
        value["registration"]["unknownField"] = serde_json::json!(1);
        let broken = assess_project_performance(&value);
        assert_eq!(broken.status, plain.status);
        assert!(broken.registration.is_none());
        assert!(broken
            .gaps
            .iter()
            .any(|item| item.code == "registration_block_invalid"));
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

    #[test]
    fn vertical_pipes_reach_the_zone_transmission() {
        let conductance = |value: &Value| {
            let result = assess_project_performance(value);
            assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
            result
                .performance
                .unwrap()
                .space_heating
                .demand
                .transmission
                .unwrap()
                .conductance_w_per_k
        };
        let mut value = project();
        let base = conductance(&value);
        value["ntaCalculation"]["verticalPipes"] = serde_json::json!([{
            "id": "standleiding", "storeys": 2, "insulated": false,
            "sourceReference": "survey"
        }]);
        // Table 7.1: 1,8 W/K per storey (7.17).
        assert!((conductance(&value) - base - 3.6).abs() < 1e-9);
    }

    #[test]
    fn two_zones_need_zone_data_and_sum_areas() {
        let mut value = project();
        let mut second = value["zones"][0].clone();
        second["id"] = Value::from("z2");
        second["floorArea"] = Value::from(50.0);
        for surface in second["surfaces"].as_array_mut().unwrap() {
            let id = surface["id"].as_str().unwrap().to_owned();
            surface["id"] = Value::from(format!("{id}-2"));
            surface["zoneId"] = Value::from("z2");
            for window in surface["windows"].as_array_mut().unwrap() {
                let window_id = window["id"].as_str().unwrap().to_owned();
                window["id"] = Value::from(format!("{window_id}-2"));
            }
        }
        for bridge in second["thermalBridges"].as_array_mut().unwrap() {
            bridge["id"] = Value::from("tb1-2");
            bridge["zoneId"] = Value::from("z2");
        }
        value["zones"].as_array_mut().unwrap().push(second);
        let missing = assess_project_performance(&value);
        let codes: Vec<_> = missing.gaps.iter().map(|item| item.code).collect();
        assert!(codes.contains(&"zone_data_missing"), "{codes:?}");
        // Supply per-zone data, tilt and ground data for the copied surfaces.
        let block = &mut value["ntaCalculation"];
        let zone_entry = |id: &str| {
            serde_json::json!({
                "zoneId": id,
                "ventilationFlows": block["ventilationFlows"].clone(),
                "internalGains": block["internalGains"].clone()
            })
        };
        let entries = vec![zone_entry("z1"), zone_entry("z2")];
        block["zoneData"] = Value::from(entries);
        block["surfaceTilts"].as_array_mut().unwrap().push(
            serde_json::json!({"surfaceId": "roof-2", "tiltDeg": 45.0, "sourceReference": "copy"}),
        );
        block["groundFloors"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "surfaceId": "floor-2", "exposedPerimeterM": 20.0,
                "constructionResistanceM2kPerW": 3.87,
                "edgeThermalBridges": {"method": "forfait"}, "sourceReference": "copy"
            }));
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        let derived = result.derived_input.as_ref().unwrap();
        assert_eq!(derived.total_usable_floor_area_m2, 150.0);
        assert_eq!(derived.space_heating.additional_zones.len(), 1);
        let performance = result.performance.as_ref().unwrap();
        assert_eq!(performance.tojuli.len(), 2);
        // Each zone keeps only its own envelope.
        let zone_two = &performance.space_heating.additional_zone_demands[0];
        let first = performance
            .space_heating
            .demand
            .transmission
            .as_ref()
            .unwrap();
        let second = zone_two.transmission.as_ref().unwrap();
        assert!((first.conductance_w_per_k - second.conductance_w_per_k).abs() < 1e-9);
    }

    /// ISSO 54 v2.0 (2022), EP-W001, p. 5: A_g = 96 m² and A_o = 247,2 m²
    /// are the only values of the EDR test set published in the document
    /// itself; the official tolerance is 1 %.
    #[test]
    fn edr_epw001_geometry_matches_published_areas() {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/edr-2022-epw001-project.json"
        ))
        .unwrap();
        let result = assess_project_performance(&project);
        assert_eq!(result.status, "incomplete");
        let geometry = result.geometry.unwrap();
        assert!((geometry.usable_floor_area_m2 - 96.0).abs() <= 0.01 * 96.0);
        // p. 5 publishes the envelope A_o = 247,2 m²; A_ls weights the
        // ground floor with f_ls = 0,7 (6.7.3): 199,2 + 0,7 · 48 = 232,8 m².
        assert!((geometry.envelope_area_m2 - 247.2).abs() <= 0.01 * 247.2);
        assert!((geometry.loss_area_m2 - 232.8).abs() < 1e-9);
        assert!((geometry.loss_area_ratio.unwrap() - 2.425).abs() < 1e-9);
        assert_eq!(geometry.unclassified_surface_count, 0);
    }

    #[test]
    fn floor_over_outdoor_air_faces_down() {
        let mut value = project();
        value["zones"][0]["surfaces"][5]["thermalBoundary"] = Value::from("outdoor");
        value["ntaCalculation"]["groundFloors"] = serde_json::json!([]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        let floor = result
            .derived_input
            .unwrap()
            .space_heating
            .demand
            .opaque_elements
            .into_iter()
            .find(|item| item.id == "surface:floor:opaque")
            .unwrap();
        assert_eq!(floor.tilt_deg, 180.0);
    }
}
