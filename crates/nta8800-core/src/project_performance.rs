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
use crate::norm_versions::NormVersion;
use crate::pv::PvSystem;
use crate::solar_shading::{
    validate_movable_shading, validate_obstruction, MovableShading, Obstruction,
};
use crate::space_cooling::CoolingSystem;
use crate::space_heating_chain::{
    ChainZone, CollectiveConnection, Distribution, DistributionSystem, Generator,
    SpaceHeatingChainInput,
};
use crate::tojuli::ActiveCoolingEvidence;
use crate::{
    direct_boundary_input_zone, input_fingerprint, unheated_zone_input, ProjectInput,
    ThermalBoundary, KERNEL_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NtaCalculationInput {
    /// Edition of NTA 8800; default 2025+C1:2026, the only registrable one.
    #[serde(default, skip_serializing_if = "NormVersion::is_default")]
    pub norm_version: NormVersion,
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
    /// Annex A: dynamic transparent elements (switchable glazing, movable
    /// shutters) linked to project windows by id, with the method A/B
    /// weighting and optional step-2 correction factors.
    #[serde(default)]
    pub dynamic_windows: Vec<ProjectDynamicWindow>,
    /// External obstruction per project window (7.13: `F_sh;obst;wi,k;mi`
    /// per window, p. 183–184; 17.3.2: one situation per window). A window
    /// not listed keeps `windowSolar.obstruction`.
    #[serde(default)]
    pub window_obstructions: Vec<ProjectWindowObstruction>,
    /// Movable sun shading per project window (7.6.6.1.4: g_gl;wi;mi of
    /// 7.42 and F_c of 7.43 per window wi, 2025+C1 p. 196–198). A listed
    /// window takes its own `movableShading` (absent: none); a window not
    /// listed keeps `windowSolar.movableShading`.
    #[serde(default)]
    pub window_shadings: Vec<ProjectWindowShading>,
    /// Glazing details per project window (7.6.6.1.2/7.6.6.1.3: table 7.4
    /// type, fixed louvres of 7.41a/7.41b or the ISO 15099 values of 7.41
    /// for raam wi, 2025+C1 p. 189–193). A window not listed uses its
    /// `gValue` as g_gl;n.
    #[serde(default)]
    pub window_glazings: Vec<ProjectWindowGlazing>,
    /// Humidifiers per zone (chapter 12).
    #[serde(default)]
    pub humidifiers: Vec<crate::space_heating_chain::ZoneHumidifier>,
    /// Adjacent unheated sunrooms (7.30b); per zone in `zoneData`.
    #[serde(default)]
    pub sunrooms: Vec<crate::monthly_demand::Sunroom>,
    /// Vertical pipes through the envelope (7.3.3, H_p of 7.17); per zone in
    /// `zoneData`. `[]` states that there are none; absent means unknown,
    /// for which 7.3.3 prescribes fictitious uninsulated pipes that have to
    /// be entered (gap `vertical_pipes_unknown`).
    #[serde(default)]
    pub vertical_pipes: Option<Vec<crate::monthly_demand::VerticalPipe>>,
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
    /// §13.2.4: further hot-water systems of the building.
    #[serde(default)]
    pub additional_hot_water_systems: Vec<HotWaterSystem>,
    /// §13.7 solar systems for space heating only (SHS).
    #[serde(default)]
    pub space_heating_solar: Vec<crate::solar_thermal::SolarWaterHeater>,
    #[serde(default)]
    pub lighting: Vec<crate::lighting::ZoneLighting>,
    #[serde(default)]
    pub cooling: Option<CoolingSystem>,
    /// §10.2: several cooling systems with the project zone ids they serve.
    #[serde(default)]
    pub cooling_systems: Vec<crate::building_performance::ServedCoolingSystem>,
    /// §9.2/5.20: further heating systems with the project zone ids they
    /// serve; the other zones stay on the main heating system.
    #[serde(default)]
    pub additional_heating_systems: Vec<ProjectHeatingSystem>,
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
    /// 7.3.3 vertical pipes of this zone; `[]` states none, absent falls
    /// back to the project list (single-zone projects only).
    #[serde(default)]
    pub vertical_pipes: Option<Vec<crate::monthly_demand::VerticalPipe>>,
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

/// Annex A input for one project window. The project `uValue` and `gValue`
/// stay the nominal values; the kernel replaces them per month by
/// `U_mi;mn` (A.1) and `g_mi;mn` (A.2), times the step-2 factors (p. 770).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectDynamicWindow {
    pub window_id: String,
    pub dynamic: crate::annex_a::DynamicTransparent,
}

/// Obstruction of one project window, replacing the project-wide
/// `windowSolar.obstruction` for that window.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectWindowObstruction {
    pub window_id: String,
    pub obstruction: Obstruction,
    pub source_reference: String,
}

/// Movable shading of one project window, replacing the project-wide
/// `windowSolar.movableShading` for that window.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectWindowShading {
    pub window_id: String,
    /// `None`: this window has no movable shading, whatever the default.
    #[serde(default)]
    pub movable_shading: Option<MovableShading>,
    pub source_reference: String,
}

/// Glazing details of one project window (§7.6.6.1.2/7.6.6.1.3).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectWindowGlazing {
    pub window_id: String,
    pub glazing: crate::solar_shading::GlazingSolar,
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
    /// Edition of the calculation (`ntaCalculation.normVersion`).
    pub norm_version: NormVersion,
    /// Only a 2025+C1:2026 calculation may be registered; an older edition
    /// gives status `calculated_legacy_edition`.
    pub registration_eligible: bool,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub attest_status: &'static str,
    pub gaps: Vec<InputGap>,
    /// Plausibility findings that do not block the calculation: input the
    /// norm allows but that contradicts other input or a norm default.
    pub warnings: Vec<InputGap>,
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
    /// Label data of Omgevingsregeling art. 5.13.
    pub label_data: Option<crate::label_data::LabelData>,
}

/// A further heating system of the project (§9.2) with the zones it serves.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectHeatingSystem {
    pub zone_ids: Vec<String>,
    pub generator: crate::space_heating_chain::Generator,
    #[serde(default)]
    pub distribution_system: Option<crate::space_heating_chain::DistributionSystem>,
    #[serde(default)]
    pub collective_connection: Option<crate::space_heating_chain::CollectiveConnection>,
    #[serde(default)]
    pub identical_systems: Option<u32>,
    #[serde(default)]
    pub humidifiers: Vec<crate::space_heating_chain::ZoneHumidifier>,
}

/// §6.4/§6.5.2 for the derived calculation zones: one heating chain serves
/// all zones; cooling systems serve all zones or their listed zones.
fn schematisation_checks(input: &BuildingPerformanceInput) -> Vec<crate::zoning::ZoningIssue> {
    use crate::zoning::{check_zone, CalculationZoneLayout, VentilationShare, ZonePart};
    input
        .zone_inputs()
        .into_iter()
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
                cooling_system_ids: input
                    .cooling_list()
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, zones))| {
                        zones.map_or(true, |zones| zones.contains(&demand.zone_id))
                    })
                    .map(|(index, _)| format!("cooling-{index}"))
                    .collect(),
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
        loss_area_ratio: (floor > 0.0)
            .then(|| loss / floor)
            .filter(|ratio| ratio.is_finite()),
        unclassified_surface_count: unclassified,
    })
}

/// Calculation results for the registration checks (A_g, A_ls/A_g, EP2,
/// label class and envelope summary).
fn registration_context(
    project: Option<&ProjectInput>,
    project_value: &Value,
    derived: Option<&BuildingPerformanceInput>,
    performance: Option<&BuildingPerformanceAssessment>,
    label_data: Option<&crate::label_data::LabelData>,
) -> crate::registration::RegistrationContext {
    crate::registration::RegistrationContext {
        usable_floor_area_m2: derived
            .map(|input| input.total_usable_floor_area_m2)
            .or_else(|| project_geometry(project_value).map(|item| item.usable_floor_area_m2))
            .filter(|area| *area > 0.0),
        zone_floor_areas_m2: project
            .map(|project| project.zones.iter().map(|zone| zone.floor_area).collect())
            .unwrap_or_default(),
        loss_area_ratio: project_geometry(project_value).and_then(|item| item.loss_area_ratio),
        residential: project.is_some_and(|project| {
            matches!(
                project.building_function,
                crate::BuildingFunction::Residential
            )
        }),
        label_function: derived.and_then(|input| input.label_function),
        // Omgevingsregeling art. 5.11 lid 4: the dwelling label, and so its
        // registration checks, follow the EMGforf scenario.
        primary_fossil_kwh_per_m2: performance.and_then(|result| {
            result
                .label_primary_fossil_indicator_kwh_per_m2_year
                .or(result.primary_fossil_indicator_kwh_per_m2_year)
        }),
        label_class: performance.and_then(|result| result.indicative_label_class),
        envelope: label_data
            .map(|data| data.envelope.clone())
            .unwrap_or_default(),
        whole_building: single_detached_dwelling(project, project_value, derived),
    }
}

/// The calculation covers the whole building (BRL 9500-W p. 18, 21: the
/// WLC-GWP threshold is per building): one dwelling (Σ N_woon = 1), not in an
/// apartment building, without surfaces against an adjacent conditioned
/// building, and recorded as detached. Party walls of terraced houses are
/// usually not entered as surfaces, so the absence of such surfaces alone is
/// not enough: the table 11.14 detached airtightness types, or a registration
/// building type that says detached, are required.
fn single_detached_dwelling(
    project: Option<&ProjectInput>,
    project_value: &Value,
    derived: Option<&BuildingPerformanceInput>,
) -> bool {
    let Some(derived) = derived else {
        return false;
    };
    let zones = derived.zone_inputs();
    let dwellings: u32 = zones
        .iter()
        .filter_map(|zone| zone.internal_gains.zone_dwellings())
        .sum::<f64>()
        .round() as u32;
    let apartment = zones.iter().any(|zone| {
        zone.dwelling_type == Some(crate::monthly_demand::DwellingType::ApartmentBuilding)
            || zone
                .ventilation
                .as_ref()
                .is_some_and(|ventilation| ventilation.apartment_building)
    });
    let attached = project.is_some_and(|project| {
        project.zones.iter().any(|zone| {
            zone.surfaces.iter().any(|surface| {
                surface
                    .get("thermalBoundary")
                    .and_then(|value| serde_json::from_value::<ThermalBoundary>(value.clone()).ok())
                    .is_some_and(|boundary| {
                        matches!(boundary, ThermalBoundary::AdjacentConditioned)
                    })
            })
        })
    });
    dwellings == 1 && !apartment && !attached && recorded_detached(project_value)
}

/// Table 11.14 airtightness types of the infiltration inputs all detached, or
/// the registration building type names a detached dwelling.
fn recorded_detached(project_value: &Value) -> bool {
    let mut types = Vec::new();
    collect_airtightness_types(project_value, &mut types);
    if !types.is_empty() {
        return types.iter().all(|kind| kind.contains("detached"));
    }
    project_value
        .pointer("/registration/buildingType")
        .and_then(Value::as_str)
        .map(str::to_lowercase)
        .is_some_and(|text| {
            (text.contains("vrijstaand") || text.contains("detached"))
                && !["half", "semi", "twee-onder", "2-onder", "onder-een-kap"]
                    .iter()
                    .any(|part| text.contains(part))
        })
}

fn collect_airtightness_types(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if map.get("method").and_then(Value::as_str) == Some("reference") {
                if let Some(kind) = map.get("buildingType").and_then(Value::as_str) {
                    out.push(kind.to_string());
                }
            }
            for item in map.values() {
                collect_airtightness_types(item, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_airtightness_types(item, out);
            }
        }
        _ => {}
    }
}

/// 8.2.1 and 8.3.3.1: a forfait floor edge (0,5·P of 8.37/8.38) selects
/// the forfait treatment of linear thermal bridges for the whole building,
/// so H_D takes ΔU_for of 8.3 and no ψ-values may be entered; mixing the
/// two methods is not allowed. Returns ΔU_for for the forfait route.
fn forfait_bridge_route(
    project: &ProjectInput,
    nta: &NtaCalculationInput,
    constructions: &HashMap<&str, &Value>,
    gaps: &mut Vec<InputGap>,
) -> Option<f64> {
    let forfait = nta
        .ground_floors
        .iter()
        .any(|floor| matches!(floor.edge_thermal_bridges, EdgeThermalBridges::Forfait));
    if !forfait {
        return None;
    }
    let detailed_edges = nta.ground_floors.iter().any(|floor| {
        matches!(&floor.edge_thermal_bridges, EdgeThermalBridges::Detailed { bridges } if !bridges.is_empty())
    });
    let psi = project.zones.iter().any(|zone| {
        zone.thermal_bridges
            .as_deref()
            .is_some_and(|items| !items.is_empty())
    });
    if detailed_edges || psi {
        gaps.push(gap(
            "thermal_bridge_methods_mixed",
            "ntaCalculation.groundFloors",
        ));
    }
    let boundary = |surface: &Value| {
        surface
            .get("thermalBoundary")
            .and_then(|value| serde_json::from_value::<ThermalBoundary>(value.clone()).ok())
    };
    let surfaces = || project.zones.iter().flat_map(|zone| &zone.surfaces);
    // 8.4: in the forfait route H_U;for = 0 and the partition joins H_D;for
    // with U_iu;equi of C.1.3, which this route does not derive.
    if surfaces().any(|surface| boundary(surface) == Some(ThermalBoundary::UnheatedSpace)) {
        gaps.push(gap(
            "forfait_thermal_bridges_unheated_space_unsupported",
            "ntaCalculation.groundFloors",
        ));
    }
    // 8.3: area-weighted U of the opaque parts bordering outdoor air.
    let (mut sum_au, mut sum_a) = (0.0, 0.0);
    for surface in surfaces().filter(|surface| boundary(surface) == Some(ThermalBoundary::Outdoor))
    {
        let windows: f64 = surface
            .get("windows")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|window| window.get("area").and_then(Value::as_f64))
                    .sum()
            })
            .unwrap_or(0.0);
        let area = surface.get("area").and_then(Value::as_f64).unwrap_or(0.0) - windows;
        let u_value = surface
            .get("constructionId")
            .and_then(Value::as_str)
            .and_then(|id| constructions.get(id))
            .and_then(|item| item.get("uValue"))
            .and_then(Value::as_f64);
        if let (true, Some(u_value)) = (area > 1e-9, u_value) {
            sum_au += area * u_value;
            sum_a += area;
        }
    }
    Some(if sum_a > 0.0 {
        crate::constructions::forfait_bridge_supplement(&[(sum_a, sum_au / sum_a)])
    } else {
        0.0
    })
}

/// Plausibility checks that leave the calculation running: declared values
/// the norm accepts but that contradict a norm default or other input.
fn year_at(value: &Value, pointer: &str) -> Option<u32> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .and_then(|year| u32::try_from(year).ok())
}

fn registration_construction_year(project_value: &Value) -> Option<u32> {
    year_at(project_value, "/registration/constructionYear")
}

fn nta_block_construction_year(project_value: &Value) -> Option<u32> {
    year_at(project_value, "/ntaCalculation/constructionYear")
}

/// The bouwjaar of §5.3.2 (p. 75–76) and of the label data (Omgevingsregeling
/// art. 5.13 onder a): the registration, else the NTA block. The chapter 11 year is not
/// used: table 11.13 (p. 486) also takes a renovatiejaar.
pub(crate) fn resolved_construction_year(project_value: &Value) -> Option<u32> {
    registration_construction_year(project_value)
        .or_else(|| nta_block_construction_year(project_value))
}

/// The chapter 11 bouwjaar (table 11.13): the block-level ventilation, else
/// the first calculation zone that states one.
fn ventilation_construction_year(project_value: &Value) -> Option<u32> {
    year_at(
        project_value,
        "/ntaCalculation/ventilation/constructionYear",
    )
    .or_else(|| {
        project_value
            .pointer("/ntaCalculation/zoneData")
            .and_then(Value::as_array)?
            .iter()
            .find_map(|zone| year_at(zone, "/ventilation/constructionYear"))
    })
}

/// Bounds of the input range checks. They are program choices, not norm
/// values: a value above the warning bound is unusual for a building and is
/// flagged; a value above the blocking bound cannot describe a building and
/// stops the calculation (docs/nta8800-releasenotes.md lists them).
struct RangeBound {
    warn: f64,
    block: f64,
}

/// Zone usable area A_g and surface or window area, m².
const AREA_BOUND: RangeBound = RangeBound {
    warn: 1.0e6,
    block: 1.0e7,
};
const SURFACE_AREA_BOUND: RangeBound = RangeBound {
    warn: 5.0e5,
    block: 1.0e7,
};
/// Thermal transmittance, W/(m²·K).
const U_BOUND: RangeBound = RangeBound {
    warn: 10.0,
    block: 100.0,
};
/// Air permeability q_v10, dm³/(s·m²).
const QV10_BOUND: RangeBound = RangeBound {
    warn: 10.0,
    block: 1000.0,
};
/// Declared monthly use per m² of A_g, kWh/(m²·month).
const MONTHLY_USE_BOUND: RangeBound = RangeBound {
    warn: 5000.0,
    block: 1.0e6,
};
/// Smallest plausible usable area of a zone, m².
const MIN_ZONE_AREA_M2: f64 = 1.0;
/// Smallest plausible usable area per dwelling, m².
const MIN_AREA_PER_DWELLING_M2: f64 = 10.0;
/// Largest plausible A_ls/A_g.
const MAX_LOSS_AREA_RATIO: f64 = 20.0;

#[derive(Default)]
struct RangeFindings {
    blocking: Vec<InputGap>,
    warnings: Vec<InputGap>,
}

impl RangeFindings {
    fn check(&mut self, value: f64, bound: &RangeBound, code: &'static str, path: String) {
        if value > bound.block {
            self.blocking.push(InputGap {
                detail: Some(format!("{value} > {}", bound.block)),
                ..gap(code, path)
            });
        } else if value > bound.warn {
            self.warnings.push(InputGap {
                detail: Some(format!("{value} > {}", bound.warn)),
                ..gap(code, path)
            });
        }
    }
}

/// Largest magnitude any number in a derived building input may have. No NTA
/// quantity of a building within the bounds above comes near it; a larger
/// value can only come from a wrong patch or input. A program choice.
const BUILDING_VALUE_MAGNITUDE_BLOCK: f64 = 1.0e12;

/// Blocking range findings of a building input that does not pass the project
/// route: a maatwerkadvies base given as building input, or a building input
/// after building-target measure patches. Uses the area and declared-use
/// bounds of the project route plus a generic magnitude bound.
pub(crate) fn building_input_range_blocks(input: &Value) -> Vec<InputGap> {
    let mut found = RangeFindings::default();
    let area = input
        .get("totalUsableFloorAreaM2")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    found.check(
        area,
        &AREA_BOUND,
        "area_out_of_range",
        "totalUsableFloorAreaM2".to_string(),
    );
    if area > 0.0 {
        let empty = Vec::new();
        let uses = input
            .get("declaredUses")
            .and_then(Value::as_array)
            .unwrap_or(&empty);
        for (u, item) in uses.iter().enumerate() {
            let months = item
                .get("monthlyKwh")
                .and_then(Value::as_array)
                .unwrap_or(&empty);
            if let Some(max) = months.iter().filter_map(Value::as_f64).reduce(f64::max) {
                found.check(
                    max / area,
                    &MONTHLY_USE_BOUND,
                    "declared_use_out_of_range",
                    format!("declaredUses[{u}].monthlyKwh"),
                );
            }
        }
    }
    let mut blocks = found.blocking;
    if let Some((path, value)) = first_huge_number(input, String::new()) {
        blocks.push(InputGap {
            detail: Some(format!("|{value}| > {BUILDING_VALUE_MAGNITUDE_BLOCK}")),
            ..gap("value_out_of_range", path)
        });
    }
    blocks
}

fn first_huge_number(value: &Value, path: String) -> Option<(String, f64)> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .filter(|n| n.abs() > BUILDING_VALUE_MAGNITUDE_BLOCK)
            .map(|n| (path, n)),
        Value::Array(items) => items
            .iter()
            .enumerate()
            .find_map(|(i, item)| first_huge_number(item, format!("{path}[{i}]"))),
        Value::Object(map) => map.iter().find_map(|(key, item)| {
            let next = if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            };
            first_huge_number(item, next)
        }),
        _ => None,
    }
}

/// Range findings of the project input: (blocking gaps, warnings).
fn input_range_findings(project_value: &Value) -> (Vec<InputGap>, Vec<InputGap>) {
    let mut found = RangeFindings::default();
    let number = |value: &Value, key: &str| value.get(key).and_then(Value::as_f64);
    let empty = Vec::new();
    let zones = project_value
        .get("zones")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    let mut total_area = 0.0;
    for (z, zone) in zones.iter().enumerate() {
        if let Some(area) = number(zone, "floorArea") {
            total_area += area;
            found.check(
                area,
                &AREA_BOUND,
                "zone_area_out_of_range",
                format!("zones[{z}].floorArea"),
            );
        }
        let surfaces = zone
            .get("surfaces")
            .and_then(Value::as_array)
            .unwrap_or(&empty);
        for (s, surface) in surfaces.iter().enumerate() {
            if let Some(area) = number(surface, "area") {
                found.check(
                    area,
                    &SURFACE_AREA_BOUND,
                    "surface_area_out_of_range",
                    format!("zones[{z}].surfaces[{s}].area"),
                );
            }
            let windows = surface
                .get("windows")
                .and_then(Value::as_array)
                .unwrap_or(&empty);
            for (w, window) in windows.iter().enumerate() {
                let path = format!("zones[{z}].surfaces[{s}].windows[{w}]");
                if let Some(area) = number(window, "area") {
                    found.check(
                        area,
                        &SURFACE_AREA_BOUND,
                        "surface_area_out_of_range",
                        format!("{path}.area"),
                    );
                }
                if let Some(u) = number(window, "uValue") {
                    found.check(
                        u,
                        &U_BOUND,
                        "u_value_out_of_range",
                        format!("{path}.uValue"),
                    );
                }
            }
        }
    }
    let constructions = project_value
        .get("constructions")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    for (c, construction) in constructions.iter().enumerate() {
        if let Some(u) = number(construction, "uValue") {
            found.check(
                u,
                &U_BOUND,
                "u_value_out_of_range",
                format!("constructions[{c}].uValue"),
            );
        }
    }
    // q_v10 and the dwelling counts may sit at several places in the NTA
    // block (block level, per zone, per ventilation system).
    let mut keyed = Vec::new();
    if let Some(nta) = project_value.get("ntaCalculation") {
        collect_keys(
            nta,
            "ntaCalculation",
            &["qv10DmPerSM2", "dwellingCount"],
            &mut keyed,
        );
    }
    for (key, path, value) in &keyed {
        match key.as_str() {
            "qv10DmPerSM2" => found.check(*value, &QV10_BOUND, "qv10_out_of_range", path.clone()),
            _ if total_area > 0.0
                && *value > 0.0
                && total_area / *value < MIN_AREA_PER_DWELLING_M2 =>
            {
                found.warnings.push(InputGap {
                    detail: Some(format!("{:.1} m² per dwelling", total_area / *value)),
                    ..gap("dwelling_count_implausible", path.clone())
                });
            }
            _ => {}
        }
    }
    if total_area > 0.0 {
        let uses = project_value
            .pointer("/ntaCalculation/declaredUses")
            .and_then(Value::as_array)
            .unwrap_or(&empty);
        for (u, item) in uses.iter().enumerate() {
            let months = item
                .get("monthlyKwh")
                .and_then(Value::as_array)
                .unwrap_or(&empty);
            if let Some(max) = months.iter().filter_map(Value::as_f64).reduce(f64::max) {
                found.check(
                    max / total_area,
                    &MONTHLY_USE_BOUND,
                    "declared_use_out_of_range",
                    format!("ntaCalculation.declaredUses[{u}].monthlyKwh"),
                );
            }
        }
    }
    for (z, zone) in zones.iter().enumerate() {
        if number(zone, "floorArea").is_some_and(|area| area > 0.0 && area < MIN_ZONE_AREA_M2) {
            found.warnings.push(gap(
                "zone_area_implausible",
                format!("zones[{z}].floorArea"),
            ));
        }
    }
    if let Some(ratio) =
        project_geometry(project_value).and_then(|geometry| geometry.loss_area_ratio)
    {
        if ratio > MAX_LOSS_AREA_RATIO {
            found.warnings.push(InputGap {
                detail: Some(format!("A_ls/A_g {ratio:.1}")),
                ..gap("loss_area_ratio_implausible", "zones")
            });
        }
    }
    (found.blocking, found.warnings)
}

/// Every numeric value under one of `keys`, with its path.
fn collect_keys(value: &Value, path: &str, keys: &[&str], out: &mut Vec<(String, String, f64)>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                let child = format!("{path}.{key}");
                if keys.contains(&key.as_str()) {
                    if let Some(number) = item.as_f64() {
                        out.push((key.clone(), child.clone(), number));
                    }
                }
                collect_keys(item, &child, keys, out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_keys(item, &format!("{path}[{index}]"), keys, out);
            }
        }
        _ => {}
    }
}

fn plausibility_warnings(project_value: &Value) -> Vec<InputGap> {
    use crate::building_performance::{Carrier, Service};
    use crate::monthly_demand::CeilingColumn;
    let mut warnings = Vec::new();

    // One building bouwjaar feeds §5.3.2 (p. 75–76) and the label data
    // (Omgevingsregeling art. 5.13 onder a): the registration year, else the NTA block. A
    // differing NTA block year means one of the two is wrong.
    let registered = registration_construction_year(project_value);
    let block = nta_block_construction_year(project_value);
    if let (Some(registered), Some(block)) = (registered, block) {
        if registered != block {
            warnings.push(InputGap {
                detail: Some(format!(
                    "registration {registered} against ntaCalculation {block}"
                )),
                ..gap(
                    "construction_year_mismatch",
                    "ntaCalculation.constructionYear",
                )
            });
        }
    }
    // Table 11.13 (p. 486) reads a bouwjaar or renovatiejaar, so a later
    // ventilation year is normal; an earlier one contradicts the bouwjaar.
    if let (Some(building), Some(ventilation)) = (
        registered.or(block),
        ventilation_construction_year(project_value),
    ) {
        if ventilation < building {
            warnings.push(InputGap {
                detail: Some(format!(
                    "ventilation (table 11.13) {ventilation} before bouwjaar {building}"
                )),
                ..gap(
                    "ventilation_year_before_construction_year",
                    "ntaCalculation.ventilation.constructionYear",
                )
            });
        }
    }

    let Some(nta) = project_value
        .get("ntaCalculation")
        .filter(|value| !value.is_null())
        .and_then(|value| serde_json::from_value::<NtaCalculationInput>(value.clone()).ok())
    else {
        return warnings;
    };
    let Ok(project) = serde_json::from_value::<ProjectInput>(project_value.clone()) else {
        return warnings;
    };
    let residential = matches!(nta.calculation_scope, CalculationScope::Residential);
    let area: f64 = project.zones.iter().map(|zone| zone.floor_area).sum();
    let dwellings = match &nta.internal_gains {
        InternalGains::Residential { dwelling_count, .. } if *dwelling_count > 0 => {
            f64::from(*dwelling_count)
        }
        _ => 1.0,
    };

    // Chapter 13: a declared fuel use for hot water below the net need
    // Q_W;nd (13.15 or table 13.1) implies η_W above 1 on gross value.
    let need = if residential {
        Some(
            crate::domestic_hot_water::RESIDENTIAL_NEED_PER_OCCUPANT_KWH
                * dwellings
                * crate::monthly_demand::occupants_per_dwelling(area / dwellings),
        )
    } else if !nta.label_functions.is_empty() {
        Some(
            nta.label_functions
                .iter()
                .map(|part| {
                    crate::domestic_hot_water::utility_specific_need(part.function).unwrap_or(0.0)
                        * part.area_m2
                })
                .sum(),
        )
    } else {
        nta.label_function
            .and_then(crate::domestic_hot_water::utility_specific_need)
            .map(|specific| specific * area)
    };
    if let Some(need) = need.filter(|need| *need > 0.0) {
        for (index, item) in nta.declared_uses.iter().enumerate() {
            let declared: f64 = item.monthly_kwh.iter().sum();
            if item.service == Service::DomesticHotWater
                && item.carrier != Carrier::El
                && declared > 0.0
                && need > declared
            {
                warnings.push(InputGap {
                    detail: Some(format!(
                        "Q_W;nd ≈ {need:.0} kWh/yr against {declared:.0} kWh/yr declared fuel: η_W ≈ {:.2} > 1",
                        need / declared
                    )),
                    ..gap(
                        "declared_hot_water_efficiency_above_one",
                        format!("ntaCalculation.declaredUses[{index}].monthlyKwh"),
                    )
                });
            }
        }
    }

    // Chapter 11: a mechanical system without heat recovery moves at least
    // q_V;ODA;req (11.22); a declared conductance below that flow is
    // implausible. Only dwellings: their design flow is continuous, while a
    // utility design flow is time-averaged over the operating hours, which
    // this indicative check does not model. f_ctrl·f_sys takes the lowest
    // residential value of table 11.5 for the system type, so demand
    // control never raises a false warning.
    let lowest_control_factor = project
        .ventilation_systems
        .iter()
        .filter(|system| {
            system
                .get("heatRecoveryEfficiency")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                <= 0.0
        })
        .filter_map(|system| match system.get("type").and_then(Value::as_str) {
            Some("type_b") => Some(RESIDENTIAL_MIN_CONTROL_FACTOR_B),
            Some("type_c" | "type_d") => Some(RESIDENTIAL_MIN_CONTROL_FACTOR_CD),
            _ => None,
        })
        .fold(None, |lowest: Option<f64>, factor| {
            Some(lowest.map_or(factor, |value| value.min(factor)))
        });
    if let Some(control_factor) = lowest_control_factor.filter(|_| residential) {
        for zone in &project.zones {
            let data = nta.zone_data.iter().find(|item| item.zone_id == zone.id);
            let (flows, chapter11, path) = match data {
                Some(item) => (
                    &item.ventilation_flows,
                    item.ventilation.is_some(),
                    format!(
                        "ntaCalculation.zoneData[{}].ventilationFlows",
                        nta.zone_data
                            .iter()
                            .position(|other| other.zone_id == zone.id)
                            .unwrap_or(0)
                    ),
                ),
                None => (
                    &nta.ventilation_flows,
                    nta.ventilation.is_some(),
                    "ntaCalculation.ventilationFlows".to_owned(),
                ),
            };
            if chapter11 || flows.is_empty() {
                continue;
            }
            let function = data
                .and_then(|item| item.usage_function)
                .unwrap_or(nta.usage_function);
            let required = crate::ventilation::indicative_required_flow_m3_per_h(
                crate::zoning::ventilation_function(function),
                zone.floor_area,
                zone.floor_area / dwellings,
            );
            let required = control_factor * required;
            let floor = crate::ventilation::VOLUMETRIC_HEAT_CAPACITY / 3600.0 * required;
            let lowest = (1..=12u8)
                .map(|month| {
                    flows
                        .iter()
                        .flat_map(|flow| &flow.months)
                        .filter(|item| item.month == month)
                        .map(|item| item.conductance_w_per_k)
                        .sum::<f64>()
                })
                .fold(f64::INFINITY, f64::min);
            if lowest < floor {
                warnings.push(InputGap {
                    detail: Some(format!(
                        "H_ve {lowest:.1} W/K is below ρc·q_V;ODA;req ≈ {floor:.1} W/K ({required:.1} m³/h, 11.22 with the lowest table 11.5 f_ctrl·f_sys {control_factor:.2}) of the mechanical system without heat recovery"
                    )),
                    ..gap("declared_ventilation_below_required_flow", path)
                });
            }
        }
    }

    // §5.5.8: f_BACS = 1,0 in a utility building needs every heating and
    // cooling system shown at most 290 kW (or class A/B automation).
    if !residential && nta.bacs.is_none() && nta.bacs_factor < 1.05 {
        warnings.push(InputGap {
            detail: Some(
                "§5.5.8: without the bacs block (system powers ≤ 290 kW or BACS evidence) f_BACS is 1,05".into(),
            ),
            ..gap("bacs_factor_without_capacity_evidence", "ntaCalculation.bacsFactor")
        });
    }

    // Table 7.10 footnote a: utility buildings use the closed-ceiling
    // column unless an open suspended ceiling (≥ 15 % open) is shown.
    if !residential {
        let masses = std::iter::once(("ntaCalculation.thermalMass".to_owned(), &nta.thermal_mass))
            .chain(
                nta.zone_data
                    .iter()
                    .enumerate()
                    .filter_map(|(index, item)| {
                        item.thermal_mass.as_ref().map(|mass| {
                            (
                                format!("ntaCalculation.zoneData[{index}].thermalMass"),
                                mass,
                            )
                        })
                    }),
            );
        for (path, mass) in masses {
            if mass.ceiling == CeilingColumn::OpenOrNone && mass.annex_b_elements.is_empty() {
                warnings.push(InputGap {
                    detail: Some(
                        "Table 7.10 a: utility buildings take the closed or suspended ceiling column unless at least 15 % of a free-hanging ceiling is open".into(),
                    ),
                    ..gap("utility_open_ceiling_requires_evidence", format!("{path}.ceiling"))
                });
            }
        }
    }

    chapter8_warnings(&project, &nta, &mut warnings);
    warnings
}

/// Smallest mean floor width the perimeter check accepts, m: a floor of
/// area A and width w ≥ 1 m has a perimeter of at most 2·A/w + 2·w.
const MIN_FLOOR_WIDTH_M: f64 = 1.0;
/// Relative deviation of a declared sunroom b_U or H_zi;ztu from the value
/// derived from the matching unheated space before a warning.
const SUNROOM_DEVIATION: f64 = 0.10;

/// Chapter 8 plausibility: floor input, the detailed thermal-bridge route
/// without ψ-values and sunroom values that differ from their unheated
/// space.
fn chapter8_warnings(
    project: &ProjectInput,
    nta: &NtaCalculationInput,
    warnings: &mut Vec<InputGap>,
) {
    for (index, floor) in nta.ground_floors.iter().enumerate() {
        let path = format!("ntaCalculation.groundFloors[{index}]");
        // The input is R_si + R_c (8.32/8.43); below R_si the construction
        // has a negative R_c.
        if floor.construction_resistance_m2k_per_w < crate::constructions::R_SI_DOWNWARD {
            warnings.push(InputGap {
                detail: Some(format!(
                    "{:.2} m²K/W is below R_si = 0,17: the input is R_si + R_c of the floor",
                    floor.construction_resistance_m2k_per_w
                )),
                ..gap(
                    "ground_floor_resistance_below_surface_resistance",
                    format!("{path}.constructionResistanceM2kPerW"),
                )
            });
        }
        let area = project
            .zones
            .iter()
            .flat_map(|zone| &zone.surfaces)
            .find(|surface| {
                surface.get("id").and_then(Value::as_str) == Some(floor.surface_id.as_str())
            })
            .and_then(|surface| surface.get("area").and_then(Value::as_f64));
        if let Some(area) = area.filter(|area| *area > 0.0) {
            let limit = 2.0 * area / MIN_FLOOR_WIDTH_M + 2.0 * MIN_FLOOR_WIDTH_M;
            if floor.exposed_perimeter_m > limit {
                warnings.push(InputGap {
                    detail: Some(format!(
                        "P = {:.1} m on A = {area:.1} m² implies a mean floor width below {MIN_FLOOR_WIDTH_M} m (B' = {:.2} m)",
                        floor.exposed_perimeter_m,
                        area / (0.5 * floor.exposed_perimeter_m)
                    )),
                    ..gap("ground_floor_perimeter_implausible", format!("{path}.exposedPerimeterM"))
                });
            }
        }
    }

    // 8.2.1: the detailed route sums ψ·ℓ over the linear thermal bridges of
    // H_D; with none entered H_D carries no bridges at all.
    let forfait = nta
        .ground_floors
        .iter()
        .any(|floor| matches!(floor.edge_thermal_bridges, EdgeThermalBridges::Forfait));
    let outdoor = |item: &Value| {
        item.get("thermalBoundary")
            .and_then(|value| serde_json::from_value::<ThermalBoundary>(value.clone()).ok())
            == Some(ThermalBoundary::Outdoor)
    };
    let outdoor_surfaces = project
        .zones
        .iter()
        .flat_map(|zone| &zone.surfaces)
        .any(&outdoor);
    let outdoor_bridges = project
        .zones
        .iter()
        .flat_map(|zone| zone.thermal_bridges.as_deref().unwrap_or(&[]))
        .any(outdoor);
    if !forfait && outdoor_surfaces && !outdoor_bridges {
        warnings.push(InputGap {
            detail: Some(
                "Detailed thermal-bridge route (8.2.1) without linear thermal bridges to outside air: H_D has no ψ·ℓ; enter the bridges or use the forfait ΔU_for (8.3)".into(),
            ),
            ..gap("detailed_thermal_bridges_none_entered", "zones")
        });
    }

    // 7.30b/8.4.1: a sunroom that is also an unheated space of the project
    // should carry the b_U and H_zi;ztu derived for that space.
    let sunroom_sets = std::iter::once((None, "ntaCalculation.sunrooms".to_owned(), &nta.sunrooms))
        .chain(nta.zone_data.iter().enumerate().map(|(index, item)| {
            (
                Some(item.zone_id.as_str()),
                format!("ntaCalculation.zoneData[{index}].sunrooms"),
                &item.sunrooms,
            )
        }));
    for (zone_id, base, sunrooms) in sunroom_sets {
        if sunrooms.is_empty() {
            continue;
        }
        let Some(assessment) = unheated_zone_input(project, zone_id)
            .map(|input| crate::unheated_transmission::assess_unheated_transmission(&input))
        else {
            continue;
        };
        for (index, room) in sunrooms.iter().enumerate() {
            let Some(space) = assessment.spaces.iter().find(|space| space.id == room.id) else {
                continue;
            };
            let differs = |declared: f64, derived: f64| {
                (declared - derived).abs() > SUNROOM_DEVIATION * derived.abs().max(0.1)
            };
            if differs(room.reduction_factor, space.reduction_factor)
                || differs(
                    room.zone_conductance_w_per_k,
                    space.unreduced_conductance_w_per_k,
                )
            {
                warnings.push(InputGap {
                    detail: Some(format!(
                        "declared b_U {:.3} and H_zi;ztu {:.2} W/K; unheated space {} gives b_U {:.3} and H_zi;ztu {:.2} W/K (8.4.1)",
                        room.reduction_factor,
                        room.zone_conductance_w_per_k,
                        space.id,
                        space.reduction_factor,
                        space.unreduced_conductance_w_per_k
                    )),
                    ..gap("sunroom_values_differ_from_unheated_space", format!("{base}[{index}]"))
                });
            }
        }
    }
}

/// Lowest residential f_ctrl·f_sys of table 11.5 (p. 460) for system B and
/// for systems C and D, so the plausibility floor holds for every control.
const RESIDENTIAL_MIN_CONTROL_FACTOR_B: f64 = 0.57;
const RESIDENTIAL_MIN_CONTROL_FACTOR_CD: f64 = 0.52;

const VERTICAL_PIPES_UNKNOWN_DETAIL: &str = "7.3.3: state the pipes, [] for none; when unknown enter one fictitious uninsulated pipe per storey of the zone (dwelling outside a residential building), one per dwelling (residential building) or one per toilet group with N = H/3 shared by usable area over the zones (utility building)";

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
    if system.unheated_ambient_c.is_none() && system.unheated_reduction_factor.is_none() {
        system.unheated_reduction_factor = single_unheated_reduction(project);
    }
    Some(system)
}

/// `b_U` of the only unheated space of the project, if there is exactly one.
fn single_unheated_reduction(project: &ProjectInput) -> Option<f64> {
    if project.unheated_spaces.len() != 1 {
        return None;
    }
    crate::unheated_project_input(project)
        .map(|input| crate::unheated_transmission::assess_unheated_transmission(&input))
        .and_then(|assessed| assessed.spaces.first().map(|space| space.reduction_factor))
}

/// 7.82 for hot-water pipes and vessels in that unheated space (13.26,
/// 13.58, 13.168), unless the system gives `b_U` itself.
fn with_hot_water_reduction(mut system: HotWaterSystem, project: &ProjectInput) -> HotWaterSystem {
    if system.unheated_reduction_factor.is_none() {
        system.unheated_reduction_factor = single_unheated_reduction(project);
    }
    system
}

/// §5.7.1 evidence the kernel rejects (e.g. an annex AA window it cannot
/// resolve) drops TOjuli for that zone; name the cause at project level.
fn tojuli_evidence_gaps(tojuli: &[crate::tojuli::TojuliAssessment]) -> Vec<InputGap> {
    let mut gaps = Vec::new();
    for zone in tojuli.iter().filter(|zone| zone.status == "invalid") {
        for item in zone
            .issues
            .iter()
            .filter(|item| item.path.starts_with("activeCooling"))
        {
            gaps.push(InputGap {
                detail: Some(format!("TOjuli, rekenzone {}", zone.zone_id)),
                ..gap(item.code, format!("ntaCalculation.{}", item.path))
            });
        }
    }
    gaps
}

/// A refusal of the building calculation (a route the edition lacks, a
/// combination the chapters refuse) names its cause in `performance.issues`
/// with a path in the derived input. Each such issue becomes a project gap
/// at the project input that feeds it, so the refusal is actionable; the
/// derived path stays in `detail`.
fn building_issue_gaps(
    issues: &[crate::building_performance::PerformanceIssue],
    derived: &BuildingPerformanceInput,
    project_value: &Value,
    existing: &[InputGap],
) -> Vec<InputGap> {
    let mut gaps: Vec<InputGap> = Vec::new();
    for issue in issues {
        let path = project_path_for_derived(&issue.path, derived, project_value);
        let known = existing
            .iter()
            .chain(gaps.iter())
            .any(|gap| gap.code == issue.code && gap.path == path);
        if !known {
            gaps.push(InputGap {
                detail: Some(format!("derivedInput.{}", issue.path)),
                ..gap(issue.code, path)
            });
        }
    }
    gaps
}

/// Splits `a.b[2].c` into its first member, the index directly after it and
/// the rest (`.c`, with its leading separator).
fn split_member(path: &str) -> (&str, Option<usize>, &str) {
    let end = path.find(['.', '[']).unwrap_or(path.len());
    let (member, rest) = path.split_at(end);
    if let Some(after) = rest.strip_prefix('[') {
        if let Some(close) = after.find(']') {
            if let Ok(index) = after[..close].parse() {
                return (member, Some(index), &after[close + 1..]);
            }
        }
    }
    (member, None, rest)
}

/// The project input path behind a path in the derived building input.
/// Members the derived input shares with `ntaCalculation` keep their path;
/// the space-heating chain is taken apart into the NTA blocks and zones it
/// was built from. Anything else lands on the NTA input as a whole.
fn project_path_for_derived(
    path: &str,
    derived: &BuildingPerformanceInput,
    project_value: &Value,
) -> String {
    let nta = project_value.get("ntaCalculation");
    let has_member = |member: &str| nta.and_then(|nta| nta.get(member)).is_some();
    let (member, index, rest) = split_member(path);
    match member {
        "spaceHeating" => chain_path(
            rest.strip_prefix('.').unwrap_or(rest),
            &derived.space_heating,
            None,
            project_value,
        ),
        "additionalHeatingSystems" => {
            let system = index.and_then(|index| derived.additional_heating_systems.get(index));
            let base = match index {
                Some(index) => format!("ntaCalculation.additionalHeatingSystems[{index}]"),
                None => "ntaCalculation.additionalHeatingSystems".to_string(),
            };
            match system {
                Some(system) => chain_path(
                    rest.strip_prefix('.').unwrap_or(rest),
                    system,
                    Some(&base),
                    project_value,
                ),
                None => base,
            }
        }
        // Built from the zones (6.6, the usable area and loss area).
        "totalUsableFloorAreaM2" | "lossAreaM2" => "zones".to_string(),
        member if has_member(member) => format!("ntaCalculation.{path}"),
        _ => "ntaCalculation".to_string(),
    }
}

/// Path inside one space-heating chain (`spaceHeating` or an entry of
/// `additionalHeatingSystems`, whose own blocks sit under `system`).
fn chain_path(
    path: &str,
    chain: &crate::space_heating_chain::SpaceHeatingChainInput,
    system: Option<&str>,
    project_value: &Value,
) -> String {
    let (member, index, rest) = split_member(path);
    let own = |block: &str| match system {
        Some(system) => format!("{system}.{block}{rest}"),
        None => format!("ntaCalculation.{block}{rest}"),
    };
    match member {
        // An additional heating system has its own generator, distribution,
        // humidifiers, collective connection and number of identical systems.
        "generator"
        | "distributionSystem"
        | "humidifiers"
        | "collectiveConnection"
        | "identicalSystems" => own(member),
        "demand" | "emission" | "distribution" => {
            zone_path(member, rest, &chain.demand, project_value)
        }
        "additionalZones" => match index.and_then(|index| chain.additional_zones.get(index)) {
            Some(zone) => {
                let (part, _, rest) = split_member(rest.strip_prefix('.').unwrap_or(rest));
                zone_path(part, rest, &zone.demand, project_value)
            }
            None => "zones".to_string(),
        },
        _ => system
            .map(str::to_string)
            .unwrap_or_else(|| "ntaCalculation".to_string()),
    }
}

/// Path of a zone part of the chain (`demand`, `emission`, `distribution`)
/// of the zone `demand` was derived for. A part the zone takes from its
/// `zoneData` entry maps there, the rest to the project block, mirroring
/// `derive_input`; ground floors and windows map by id to their source.
fn zone_path(
    part: &str,
    rest: &str,
    demand: &crate::monthly_demand::MonthlyDemandInput,
    project_value: &Value,
) -> String {
    let zone_id = demand.zone_id.as_str();
    let zone_index = project_value
        .get("zones")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .position(|item| item.get("id").and_then(Value::as_str) == Some(zone_id))
        });
    let zone = zone_index
        .map(|index| format!("zones[{index}]"))
        .unwrap_or_else(|| "zones".to_string());
    let zone_data = project_value
        .pointer("/ntaCalculation/zoneData")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .position(|item| item.get("zoneId").and_then(Value::as_str) == Some(zone_id))
        });
    // The entry of this zone has the member (not absent, not null).
    let zone_has = |member: &str| {
        zone_data
            .and_then(|index| {
                project_value.pointer(&format!("/ntaCalculation/zoneData/{index}/{member}"))
            })
            .is_some_and(|value| !value.is_null())
    };
    let per_zone = |member: &str, tail: &str, from_zone: bool| match (zone_data, from_zone) {
        (Some(index), true) => format!("ntaCalculation.zoneData[{index}].{member}{tail}"),
        _ => format!("ntaCalculation.{member}{tail}"),
    };
    match part {
        "emission" | "distribution" => per_zone(part, rest, zone_has(part)),
        "demand" => {
            let (field, index, tail) = split_member(rest.strip_prefix('.').unwrap_or(rest));
            let indexed = match index {
                Some(index) => format!("[{index}]{tail}"),
                None => tail.to_string(),
            };
            match field {
                // Taken from the zone entry whenever the zone has one.
                "ventilation" | "ventilationFlows" | "sunrooms" | "internalGains"
                | "functionAreas" => per_zone(field, &indexed, zone_data.is_some()),
                // Taken from the zone entry only when it gives them.
                "thermalMass" | "setpoints" | "usageFunction" => {
                    per_zone(field, &indexed, zone_has(field))
                }
                // A zone that overrides the usage function brings its own
                // dwelling type.
                "dwellingType" => per_zone(
                    field,
                    &indexed,
                    zone_has("usageFunction") || zone_has("dwellingType"),
                ),
                "windows" => match index.and_then(|index| demand.windows.get(index)) {
                    Some(window) => window_source_path(window, tail, project_value),
                    None => "ntaCalculation.windowSolar".to_string(),
                },
                "transmission" => {
                    transmission_path(tail, demand, &zone, &per_zone, &zone_has, project_value)
                }
                _ => zone,
            }
        }
        _ => zone,
    }
}

/// Path of a transmission issue: a ground floor maps by its surface id to
/// `ntaCalculation.groundFloors`, vertical pipes to where the zone takes
/// them from, the rest to the surfaces of the zone.
fn transmission_path(
    tail: &str,
    demand: &crate::monthly_demand::MonthlyDemandInput,
    zone: &str,
    per_zone: &dyn Fn(&str, &str, bool) -> String,
    zone_has: &dyn Fn(&str) -> bool,
    project_value: &Value,
) -> String {
    let (member, index, rest) = split_member(tail.strip_prefix('.').unwrap_or(tail));
    match member {
        "groundFloors" => {
            let floor = match (&demand.transmission, index) {
                (crate::monthly_demand::Transmission::Components(components), Some(index)) => {
                    components.ground_floors.get(index)
                }
                _ => None,
            };
            let project_index = floor.and_then(|floor| {
                project_value
                    .pointer("/ntaCalculation/groundFloors")
                    .and_then(Value::as_array)
                    .and_then(|items| {
                        items.iter().position(|item| {
                            item.get("surfaceId").and_then(Value::as_str) == Some(floor.id.as_str())
                        })
                    })
            });
            match project_index {
                Some(index) => format!("ntaCalculation.groundFloors[{index}]{rest}"),
                None => "ntaCalculation.groundFloors".to_string(),
            }
        }
        "verticalPipes" => {
            let indexed = match index {
                Some(index) => format!("[{index}]{rest}"),
                None => rest.to_string(),
            };
            per_zone(member, &indexed, zone_has(member))
        }
        _ => format!("{zone}.surfaces"),
    }
}

/// Path of an issue on a derived window (`window:<id>`): the obstruction of
/// that window or the project default, its annex A data, the solar block,
/// or the window itself in the zone surfaces.
fn window_source_path(
    window: &crate::monthly_demand::Window,
    tail: &str,
    project_value: &Value,
) -> String {
    let window_id = window.id.strip_prefix("window:").unwrap_or(&window.id);
    let (field, _, rest) = split_member(tail.strip_prefix('.').unwrap_or(tail));
    let listed = |list: &str| {
        project_value
            .pointer(&format!("/ntaCalculation/{list}"))
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().position(|item| {
                    item.get("windowId").and_then(Value::as_str) == Some(window_id)
                })
            })
    };
    let project_window = || {
        let zones = project_value.get("zones").and_then(Value::as_array)?;
        for (zone_index, zone) in zones.iter().enumerate() {
            let surfaces = zone.get("surfaces").and_then(Value::as_array);
            for (surface_index, surface) in surfaces.into_iter().flatten().enumerate() {
                let windows = surface.get("windows").and_then(Value::as_array);
                for (index, item) in windows.into_iter().flatten().enumerate() {
                    if item.get("id").and_then(Value::as_str) == Some(window_id) {
                        return Some(format!(
                            "zones[{zone_index}].surfaces[{surface_index}].windows[{index}]"
                        ));
                    }
                }
            }
        }
        None
    };
    match field {
        "obstruction" => match listed("windowObstructions") {
            Some(index) => format!("ntaCalculation.windowObstructions[{index}].obstruction{rest}"),
            None => format!("ntaCalculation.windowSolar.obstruction{rest}"),
        },
        "dynamic" => match listed("dynamicWindows") {
            Some(index) => format!("ntaCalculation.dynamicWindows[{index}].dynamic{rest}"),
            None => "ntaCalculation.dynamicWindows".to_string(),
        },
        "movableShading" => match listed("windowShadings") {
            Some(index) => format!("ntaCalculation.windowShadings[{index}].movableShading{rest}"),
            None => format!("ntaCalculation.windowSolar.movableShading{rest}"),
        },
        "glazing" => match listed("windowGlazings") {
            Some(index) => format!("ntaCalculation.windowGlazings[{index}].glazing{rest}"),
            None => "ntaCalculation.windowGlazings".to_string(),
        },
        "frameFraction" => format!("ntaCalculation.windowSolar.{field}{rest}"),
        // The window's own data in the project model.
        "areaM2" | "uValueWPerM2k" | "gPerpendicular" | "" => {
            let member = match field {
                "areaM2" => ".area",
                "uValueWPerM2k" => ".uValue",
                "gPerpendicular" => ".gValue",
                _ => "",
            };
            project_window()
                .map(|path| format!("{path}{member}"))
                .unwrap_or_else(|| "ntaCalculation.windowSolar".to_string())
        }
        _ => "ntaCalculation.windowSolar".to_string(),
    }
}

pub fn assess_project_performance(project_value: &Value) -> ProjectPerformanceAssessment {
    // The whole project route (derived constructions, materials, annexes)
    // runs in the chosen edition, not only the building calculation.
    crate::norm_versions::with_version(project_norm_version(project_value), || {
        assess_project_in_edition(project_value)
    })
}

fn assess_project_in_edition(project_value: &Value) -> ProjectPerformanceAssessment {
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
    if let Some(result) = &performance {
        gaps.extend(tojuli_evidence_gaps(&result.tojuli));
    }
    if let (Some(input), Some(result)) = (&derived, &performance) {
        if !result.status.starts_with("calculated") {
            let found = building_issue_gaps(&result.issues, input, project_value, &gaps);
            gaps.extend(found);
        }
    }
    let version = project_norm_version(project_value);
    let status = match (&derived, &performance) {
        (None, _) => "incomplete",
        (Some(_), Some(result)) if result.status == "calculated_unverified" => {
            if version.registration_eligible() {
                "calculated_unverified"
            } else {
                "calculated_legacy_edition"
            }
        }
        _ => "invalid",
    };
    let project: Option<ProjectInput> = serde_json::from_value(project_value.clone()).ok();
    let label_data = project.as_ref().map(|project| {
        let mut data = crate::label_data::label_data(project, derived.as_ref());
        data.indicators = performance
            .as_ref()
            .filter(|result| result.status == "calculated_unverified")
            .map(crate::label_data::LabelIndicators::from_performance);
        // Art. 5.13a lid 1 onder e: the WLC-GWP comes from the registration.
        if let Some(indicators) = data.indicators.as_mut() {
            indicators.elements.wlc_gwp_kg_co2_eq_per_m2 = project_value
                .pointer("/registration/wlcGwp/valueKgCo2EqPerM2Year")
                .and_then(Value::as_f64);
            // Onder k en l: the adviser's statements from the registration.
            indicators.elements.responds_to_external_signals = project_value
                .pointer("/registration/labelStatements/respondsToExternalSignals")
                .and_then(Value::as_bool);
            indicators.elements.low_temperature_heating = project_value
                .pointer("/registration/labelStatements/lowTemperatureHeating")
                .and_then(Value::as_bool);
        }
        data
    });
    let registration = match project_value.get("registration") {
        None | Some(Value::Null) => None,
        Some(block) => {
            match serde_path_to_error::deserialize::<_, crate::registration::Registration>(
                block.clone(),
            ) {
                Ok(registration) => {
                    let context = registration_context(
                        project.as_ref(),
                        project_value,
                        derived.as_ref(),
                        performance
                            .as_ref()
                            .filter(|result| result.status == "calculated_unverified"),
                        label_data.as_ref(),
                    );
                    let mut assessment = crate::registration::assess_project_registration(
                        &registration,
                        project_value,
                        &context,
                    );
                    crate::registration::refuse_legacy_edition(&mut assessment, version);
                    Some(assessment)
                }
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
    let bacs = project_value
        .pointer("/ntaCalculation/bacs")
        .filter(|value| !value.is_null())
        .and_then(|value| {
            serde_json::from_value::<crate::bacs_draft::BacsDraftInput>(value.clone()).ok()
        })
        .map(|input| crate::bacs_draft::assess_bacs_draft(&input));
    let (range_gaps, range_warnings) = input_range_findings(project_value);
    let mut warnings = plausibility_warnings(project_value);
    warnings.extend(range_warnings);
    let blocked_by_range = !range_gaps.is_empty();
    gaps.extend(range_gaps);
    let mut assessment = ProjectPerformanceAssessment {
        status,
        target_norm_version: version.label(),
        norm_version: version,
        registration_eligible: version.registration_eligible(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        attest_status: "unattested",
        registration,
        bacs,
        label_data,
        gaps,
        warnings,
        geometry: project_geometry(project_value),
        schematisation: derived
            .as_ref()
            .map(schematisation_checks)
            .unwrap_or_default(),
        derived_input: derived,
        performance,
    };
    if blocked_by_range {
        withhold_results(&mut assessment);
    }
    refuse_non_finite(assessment)
}

/// `ntaCalculation.normVersion`, or the default edition when absent or
/// unreadable (an unreadable value is reported by `derive_input`).
fn project_norm_version(project_value: &Value) -> NormVersion {
    project_value
        .pointer("/ntaCalculation/normVersion")
        .filter(|value| !value.is_null())
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}

/// Marks the assessment invalid and withholds every computed part.
fn withhold_results(assessment: &mut ProjectPerformanceAssessment) {
    assessment.status = "invalid";
    assessment.performance = None;
    assessment.label_data = None;
    assessment.registration = None;
    assessment.bacs = None;
    assessment.geometry = None;
    assessment.schematisation.clear();
    assessment.derived_input = None;
}

/// Safety net: a result with a non-finite number (NaN, ±∞) is never handed
/// out, because serde_json would write it as `null`. The status becomes
/// `invalid` with `non_finite_result` at the first such path, and the parts
/// that carry computed numbers are withheld.
fn refuse_non_finite(mut assessment: ProjectPerformanceAssessment) -> ProjectPerformanceAssessment {
    let Some(path) = crate::finite::first_non_finite(&assessment) else {
        return assessment;
    };
    withhold_results(&mut assessment);
    assessment.gaps.push(InputGap {
        detail: Some(path),
        ..gap("non_finite_result", "result")
    });
    assessment
}

/// A deserialization error caused by a blank (null) value.
fn is_blank_error(message: &str) -> bool {
    message.starts_with("invalid type: null") || message.starts_with("invalid type: unit value")
}

/// Result of [`blank_paths`]: the blank (null) leaves the kernel cannot
/// accept, and whether the block still fails for another reason once those
/// blanks are filled (a truly absent field, an unknown field, a shape the
/// search cannot fill).
#[derive(Debug, Default, PartialEq)]
struct BlankSearch {
    paths: Vec<String>,
    residual: bool,
}

/// One null leaf of the block.
struct NullLeaf {
    path: String,
    steps: Vec<PathStep>,
    /// A null object member is taken out of the working copy first; an
    /// array element stays in place as null.
    removed: bool,
}

/// A location the repair has given a placeholder value, with the values it
/// may still try there.
#[derive(Clone)]
struct Slot {
    path: String,
    steps: Vec<PathStep>,
    candidates: Vec<Value>,
    next: usize,
    /// Names of the blank members taken out next to this location. A
    /// variant chosen here must know them (`deny_unknown_fields`), so the
    /// variant the input was written for is preferred.
    siblings: Vec<String>,
    /// A member that was never in the block (not a blank), filled only so
    /// the search can go on.
    absent: bool,
    /// Variant names an "unknown variant" error listed for this location:
    /// the location holds an enum (a plain enum field or a tag).
    variants: Vec<String>,
}

impl Slot {
    fn new(path: String, steps: Vec<PathStep>) -> Self {
        Slot {
            path,
            steps,
            // Text first: a blank enum tag then names its variants (an
            // integer would pick a variant by index, blind to the members
            // next to it).
            candidates: vec![
                Value::from(""),
                Value::from(0),
                Value::from(1),
                Value::from(false),
                // A map before a list: internally tagged enums and structs
                // also accept a sequence, which leaves no place to put the
                // tag or members the repair has to fill next.
                Value::Object(serde_json::Map::new()),
                Value::Array(Vec::new()),
            ],
            next: 0,
            siblings: Vec::new(),
            absent: false,
            variants: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Verdict {
    Accept,
    Weak,
    /// Accepted, but a variant that does not know a blank member next to it.
    Poor,
    Refuse,
}

type FirstError = Option<(String, String)>;

trait ErrorMove {
    /// The error is gone, or moved and is not a refusal of `field` itself.
    fn is_none_or_moved(&self, before: &FirstError, field: &str) -> bool;
}

impl ErrorMove for FirstError {
    fn is_none_or_moved(&self, before: &FirstError, field: &str) -> bool {
        match self {
            None => true,
            Some((_, message)) => {
                self != before && !message.starts_with(&format!("unknown field `{field}`"))
            }
        }
    }
}

/// 6.2b (p. 160): with several calculation zones each zone takes
/// N_woon;zi = A_g;zi / Σ A_g;zi × N_woon, unless the input states the share.
fn zone_internal_gains(
    gains: &InternalGains,
    multi_zone: bool,
    zone_area_m2: f64,
    total_area_m2: f64,
) -> InternalGains {
    match gains {
        InternalGains::Residential {
            dwelling_count,
            dwelling_share: None,
            source_reference,
        } if multi_zone && total_area_m2 > 0.0 && zone_area_m2 > 0.0 => {
            InternalGains::Residential {
                dwelling_count: *dwelling_count,
                dwelling_share: Some((zone_area_m2 / total_area_m2).min(1.0)),
                source_reference: format!("{source_reference}; N_woon;zi per 6.2b"),
            }
        }
        other => other.clone(),
    }
}

#[cfg(test)]
thread_local! {
    // Deserializations `blank_paths` made on this thread (tests bound the
    // cost by this count instead of by wall-clock time).
    static BLANK_SEARCH_DESERIALIZATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The null leaves of the block that the kernel does not accept.
///
/// Design: serde gives no type information at runtime, and inside internally
/// tagged enums and flattened structs it buffers the input, so a null there
/// is reported as a unit value with the error path stopped at the enum. The
/// search therefore does not try to read the culprit from the error.
/// Instead it works in two phases:
///
/// 1. Repair: build a copy that deserializes. Null object members are taken
///    out (optional ones then default); every remaining error is resolved by
///    giving a value to the location it points at: the missing member, the
///    null array element, or, when the path stops at an enum, the nulls
///    under it in turn. Values are tried in a fixed order (numbers, text,
///    boolean, list, object, then the variants an enum names) and the first
///    one the kernel does not refuse is kept. Whether a null was allowed is
///    not decided here; a wrong guess only costs another round. Where the
///    path stops at a buffered enum, the object a missing member belongs
///    to is found with a probe value (an object that knows the member
///    refuses the probe, one that does not reports it as unknown), and a
///    blank tag takes the variant that knows the blank members next to it.
///    A required member that was never in the block (absent, not null) is
///    filled too, so the search can go on, but makes the result `residual`.
/// 2. Classify: with a copy that deserializes, every null leaf is put back
///    as null. A set of put-back nulls fails exactly when it contains one the
///    kernel rejects, so the leaves are tested in halving groups: all allowed
///    nulls together cost one deserialization, and each reported leaf was
///    tested on its own.
///
/// If the repair cannot finish (an untagged enum, a value outside the tried
/// set), each null is tested on its own against the partial copy and counts
/// only when it moves the first error to a blank at or above its path; the
/// result is then `residual`.
///
/// Cost: phase 1 is bounded by a few deserializations per null leaf and per
/// filled location (guarded), phase 2 by about 2·k·log2(n) deserializations
/// for k blanks among n nulls.
fn blank_paths<T: serde::de::DeserializeOwned>(block: &Value) -> BlankSearch {
    let first_error = |candidate: &Value| -> FirstError {
        #[cfg(test)]
        BLANK_SEARCH_DESERIALIZATIONS.with(|count| count.set(count.get() + 1));
        serde_path_to_error::deserialize::<_, T>(candidate.clone())
            .err()
            .map(|error| {
                (
                    normal_path(&error.path().to_string()),
                    error.inner().to_string(),
                )
            })
    };
    let mut raw = Vec::new();
    collect_nulls(block, String::new(), Vec::new(), &mut raw);
    let leaves: Vec<NullLeaf> = raw
        .into_iter()
        .map(|(path, steps)| NullLeaf {
            removed: matches!(steps.last(), Some(PathStep::Key(_))),
            path,
            steps,
        })
        .collect();
    let mut work = block.clone();
    for leaf in leaves.iter().filter(|leaf| leaf.removed) {
        remove_member(&mut work, &leaf.steps);
    }
    let mut slots: Vec<Slot> = Vec::new();
    let mut guard = 0;
    let limit = 12 * leaves.len() + 200;
    let mut current = first_error(&work);
    // Phase 1: repair.
    while let Some((at, message)) = current.clone() {
        guard += 1;
        if guard > limit {
            break;
        }
        let progressed = if let Some(field) = missing_field(&message) {
            repair_missing(
                &mut work,
                &mut slots,
                &leaves,
                &at,
                field,
                &first_error,
                &current,
            )
        } else if is_blank_error(&message) {
            repair_blank(&mut work, &mut slots, &leaves, &at, &first_error, &current)
        } else {
            // A placeholder refused late (it passed while an earlier error
            // hid it), or a variant that does not fit its fields: try the
            // next value at the latest filled location at or around the
            // error.
            let mut near: Vec<usize> = (0..slots.len())
                .rev()
                .filter(|&index| {
                    let slot = &slots[index];
                    slot.path == at || is_under(&slot.path, &at) || is_under(&at, &slot.path)
                })
                .collect();
            // "invalid type: sequence" and the like name the refused kind:
            // slots holding a value of that kind go first (a stable sort, so
            // the latest one stays first among them).
            if let Some(kind) = refused_kind(&message) {
                near.sort_by_key(|&index| {
                    walk(&work, &slots[index].steps).map_or(true, |value| json_kind(value) != kind)
                });
            }
            retry(
                &mut work,
                &mut slots,
                &leaves,
                &near,
                &first_error,
                &current,
            )
        };
        if !progressed {
            break;
        }
        current = first_error(&work);
    }
    let settled = current.is_none();
    let absent = slots.iter().any(|slot| slot.absent);
    // Phase 2: classify every null leaf against the repaired copy.
    let mut blank = vec![false; leaves.len()];
    if settled {
        let mut groups = vec![(0..leaves.len()).collect::<Vec<_>>()];
        while let Some(group) = groups.pop() {
            if group.is_empty() {
                continue;
            }
            let mut trial = work.clone();
            for &index in &group {
                set_at(&mut trial, &leaves[index].steps, Value::Null);
            }
            if first_error(&trial).is_none() {
                continue;
            }
            if group.len() == 1 {
                blank[group[0]] = true;
            } else {
                let (left, right) = group.split_at(group.len() / 2);
                groups.push(right.to_vec());
                groups.push(left.to_vec());
            }
        }
    } else {
        // The copy still fails elsewhere: a null counts only when it moves
        // the first error to a blank at (or above) its own path.
        let baseline = first_error(&work);
        for (index, leaf) in leaves.iter().enumerate() {
            let mut trial = work.clone();
            set_at(&mut trial, &leaf.steps, Value::Null);
            let error = first_error(&trial);
            blank[index] = match &error {
                None => false,
                Some((at, message)) => {
                    error != baseline
                        && is_blank_error(message)
                        && (*at == leaf.path || is_under(&leaf.path, at))
                }
            };
        }
    }
    // A blank enum tag decides which members the object around it has, so
    // the nulls next to it (and under them) are not classified against the
    // variant the repair happened to choose: only the tag is reported, and
    // the user fills the rest once the variant is known.
    let tags = blank_tags(&work, &slots, &leaves, &blank, &first_error);
    for (index, leaf) in leaves.iter().enumerate() {
        if blank[index]
            && tags
                .iter()
                .any(|(tag, parent)| leaf.path != *tag && is_under(&leaf.path, parent))
        {
            blank[index] = false;
        }
    }
    BlankSearch {
        paths: leaves
            .into_iter()
            .zip(blank)
            .filter(|(_, blank)| *blank)
            .map(|(leaf, _)| leaf.path)
            .collect(),
        residual: absent || !settled,
    }
}

/// Blank leaves that are enum tags, with the path of the object they tag.
/// A blank leaf holding an enum (it learned variant names) is a tag when
/// another of those variants changes which members its object may or must
/// have (an unknown or missing field); a plain enum field (a fuel, a
/// class) never does. At most eight other variants are tried per leaf.
fn blank_tags<F>(
    work: &Value,
    slots: &[Slot],
    leaves: &[NullLeaf],
    blank: &[bool],
    first_error: &F,
) -> Vec<(String, String)>
where
    F: Fn(&Value) -> FirstError,
{
    let baseline = first_error(work);
    let mut tags = Vec::new();
    for (index, leaf) in leaves.iter().enumerate() {
        if !blank[index] {
            continue;
        }
        let Some(slot) = slots.iter().find(|slot| slot.path == leaf.path) else {
            continue;
        };
        if slot.variants.is_empty() || slot.steps.len() < 2 {
            continue;
        }
        let current = walk(work, &slot.steps).and_then(Value::as_str);
        let shapes_object = slot
            .variants
            .iter()
            .filter(|name| Some(name.as_str()) != current)
            .take(8)
            .any(|name| {
                let mut trial = work.clone();
                set_at(&mut trial, &slot.steps, Value::from(name.as_str()));
                let error = first_error(&trial);
                error != baseline
                    && error.is_some_and(|(_, message)| {
                        message.starts_with("unknown field") || message.starts_with("missing field")
                    })
            });
        if shapes_object {
            let parent = render_path(&slot.steps[..slot.steps.len() - 1]);
            tags.push((leaf.path.clone(), parent));
        }
    }
    tags
}

/// Resolves ``missing field `field` `` at `at`: the removed null member of
/// that name, or a member that was never in the block.
#[allow(clippy::too_many_arguments)]
fn repair_missing<F>(
    work: &mut Value,
    slots: &mut Vec<Slot>,
    leaves: &[NullLeaf],
    at: &str,
    field: &str,
    first_error: &F,
    current: &FirstError,
) -> bool
where
    F: Fn(&Value) -> FirstError,
{
    let filled = |slots: &[Slot], path: &str| slots.iter().any(|slot| slot.path == path);
    let target = join_path(at, field);
    if let Some(leaf) = leaves
        .iter()
        .find(|leaf| leaf.removed && leaf.path == target && !filled(slots, &leaf.path))
    {
        return place(
            work,
            slots,
            leaves,
            &leaf.path,
            &leaf.steps,
            first_error,
            current,
        );
    }
    // Under a tagged enum the path stops early: a removed member with this
    // name deeper under it, when giving it a value clears this error.
    let deeper: Vec<Vec<PathStep>> = leaves
        .iter()
        .filter(|leaf| {
            leaf.removed
                && is_under(&leaf.path, at)
                && matches!(leaf.steps.last(), Some(PathStep::Key(key)) if key == field)
                && !filled(slots, &leaf.path)
        })
        .map(|leaf| leaf.steps.clone())
        .collect();
    for steps in deeper {
        if place_member(work, slots, leaves, &steps, field, first_error, current) {
            return true;
        }
    }
    // Inside a buffered enum the path stops at the enum: the member may
    // belong to a placeholder object the repair put under it (a blank struct
    // or tagged enum). Try the latest such placeholder first.
    let placeholders: Vec<Vec<PathStep>> = slots
        .iter()
        .rev()
        .filter(|slot| is_under(&slot.path, at))
        .filter(|slot| {
            walk(work, &slot.steps).is_some_and(|value| {
                value
                    .as_object()
                    .is_some_and(|map| !map.contains_key(field))
            })
        })
        .map(|slot| slot.steps.clone())
        .collect();
    for steps in placeholders {
        let mut member = steps.clone();
        member.push(PathStep::Key(field.to_string()));
        if filled(slots, &render_path(&member)) {
            continue;
        }
        if place_member(work, slots, leaves, &member, field, first_error, current) {
            return true;
        }
    }
    // Still ambiguous inside a buffered enum: find the object under the
    // error path that knows the member, by giving it a probe value no kernel
    // type accepts. An object that knows the member refuses the probe; one
    // that does not reports it as unknown (or ignores it).
    if let Some(owner) = member_owner(work, at, field, first_error, current) {
        let mut member = owner.clone();
        member.push(PathStep::Key(field.to_string()));
        let path = render_path(&member);
        if !filled(slots, &path) {
            let removed = leaves.iter().any(|leaf| leaf.removed && leaf.path == path);
            let inside_placeholder = slots.iter().any(|slot| {
                let owner_path = render_path(&owner);
                slot.path == owner_path || is_under(&owner_path, &slot.path)
            });
            if place_member(work, slots, leaves, &member, field, first_error, current) {
                if !removed && !inside_placeholder {
                    if let Some(slot) = slots.iter_mut().rev().find(|slot| slot.path == path) {
                        slot.absent = true;
                    }
                }
                return true;
            }
        }
    }
    // A variant chosen for a blank tag next to it may be the wrong one: one
    // that needs members the input never had. Try the next variant first.
    let tags = tag_slots(slots, work, at, false);
    if retry(work, slots, leaves, &tags, first_error, current) {
        return true;
    }
    // Inside a buffered enum the error stops above a nested tagged enum:
    // its variant may be the one that needs the member.
    let deeper_tags = tag_slots(slots, work, at, true);
    if retry(work, slots, leaves, &deeper_tags, first_error, current) {
        return true;
    }
    // A member that was never in the block. Inside a value the repair put
    // there itself it is part of that placeholder, not an absent input.
    let inside_placeholder = slots
        .iter()
        .any(|slot| slot.path == at || is_under(at, &slot.path));
    let Some(steps) = parse_path(at, field) else {
        return false;
    };
    let path = render_path(&steps);
    if filled(slots, &path) || walk_mut(work, &steps[..steps.len() - 1]).is_none() {
        return false;
    }
    let placed = place(work, slots, leaves, &path, &steps, first_error, current);
    if placed && !inside_placeholder {
        if let Some(slot) = slots.last_mut() {
            slot.absent = true;
        }
    }
    placed
}

/// Places a value at `steps`; when that is an object and the error still
/// asks for `field` (a tagged enum whose own tag has the same name, such as
/// a `kind` member that is itself a `kind`-tagged enum), the member is
/// placed inside it too, a few levels deep. Returns whether the error moved.
#[allow(clippy::too_many_arguments)]
fn place_member<F>(
    work: &mut Value,
    slots: &mut Vec<Slot>,
    leaves: &[NullLeaf],
    steps: &[PathStep],
    field: &str,
    first_error: &F,
    current: &FirstError,
) -> bool
where
    F: Fn(&Value) -> FirstError,
{
    let mut trial = work.clone();
    let mut trial_slots = Vec::new();
    let mut at = steps.to_vec();
    for _ in 0..4 {
        let path = render_path(&at);
        if !place(
            &mut trial,
            &mut trial_slots,
            leaves,
            &path,
            &at,
            first_error,
            current,
        ) {
            return false;
        }
        let error = first_error(&trial);
        // The same error can come from another object asking for a member
        // of the same name (nested `kind`-tagged enums under one buffered
        // path): the member placed here still counts when this object knows
        // it, which a probe value there shows.
        let same_name_elsewhere = error == *current
            && error
                .as_ref()
                .and_then(|(_, message)| missing_field(message))
                == Some(field)
            && at
                .last()
                .is_some_and(|step| matches!(step, PathStep::Key(key) if key == field))
            && {
                let mut probe = trial.clone();
                set_at(&mut probe, &at, serde_json::json!({ "\u{1}probe": [] }));
                let probed = first_error(&probe);
                probed != error
                    && probed.is_some_and(|(_, text)| {
                        !text.starts_with(&format!("unknown field `{field}`"))
                    })
            };
        if error.is_none_or_moved(current, field) || same_name_elsewhere {
            *work = trial;
            slots.extend(trial_slots);
            return true;
        }
        let nested = walk(&trial, &at)
            .and_then(Value::as_object)
            .is_some_and(|map| !map.contains_key(field));
        if !nested
            || error
                .as_ref()
                .and_then(|(_, message)| missing_field(message))
                != Some(field)
        {
            return false;
        }
        at.push(PathStep::Key(field.to_string()));
    }
    false
}

/// The object at or under `at` that lacks `field` and knows it: putting a
/// probe value there changes the error to a refusal of the probe rather than
/// an unknown field. Deepest objects first.
fn member_owner<F>(
    work: &Value,
    at: &str,
    field: &str,
    first_error: &F,
    current: &FirstError,
) -> Option<Vec<PathStep>>
where
    F: Fn(&Value) -> FirstError,
{
    let root = parse_path(at, "x")?;
    let root = &root[..root.len() - 1];
    let start = walk(work, root)?;
    let mut objects = Vec::new();
    collect_objects(start, root.to_vec(), &mut objects);
    objects.sort_by_key(|steps| std::cmp::Reverse(steps.len()));
    objects.into_iter().find(|steps| {
        let lacks = walk(work, steps)
            .and_then(Value::as_object)
            .is_some_and(|map| !map.contains_key(field));
        if !lacks {
            return false;
        }
        let mut trial = work.clone();
        let mut member = steps.clone();
        member.push(PathStep::Key(field.to_string()));
        set_at(&mut trial, &member, serde_json::json!({ "\u{1}probe": [] }));
        let error = first_error(&trial);
        error != *current
            && error.as_ref().is_some_and(|(_, message)| {
                !message.starts_with(&format!("unknown field `{field}`"))
                    && missing_field(message) != Some(field)
            })
    })
}

fn collect_objects(value: &Value, steps: Vec<PathStep>, out: &mut Vec<Vec<PathStep>>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                let mut more = steps.clone();
                more.push(PathStep::Key(key.clone()));
                collect_objects(item, more, out);
            }
            out.push(steps);
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let mut more = steps.clone();
                more.push(PathStep::Index(index));
                collect_objects(item, more, out);
            }
        }
        _ => {}
    }
}

/// Slots holding a variant name (latest first) whose enum sits at `at`, or
/// under it when `deeper`.
fn tag_slots(slots: &[Slot], work: &Value, at: &str, deeper: bool) -> Vec<usize> {
    (0..slots.len())
        .rev()
        .filter(|&index| {
            let slot = &slots[index];
            let parent = render_path(&slot.steps[..slot.steps.len().saturating_sub(1)]);
            (if deeper {
                is_under(&parent, at)
            } else {
                parent == at
            }) && walk(work, &slot.steps)
                .is_some_and(|value| value.is_string() || value.is_number())
        })
        .collect()
}

/// Moves the given slots (in order) to their next accepted value until the
/// error moves. Values the repair put next to a slot after it may belong to
/// its previous value (the fields of a variant), so they are taken back
/// first and filled again as needed.
fn retry<F>(
    work: &mut Value,
    slots: &mut Vec<Slot>,
    leaves: &[NullLeaf],
    order: &[usize],
    first_error: &F,
    current: &FirstError,
) -> bool
where
    F: Fn(&Value) -> FirstError,
{
    for &index in order {
        if index >= slots.len() {
            continue;
        }
        let mut trial = work.clone();
        let mut trial_slots = slots.clone();
        reset_after(&mut trial, &mut trial_slots, leaves, index);
        let advanced = advance(&mut trial, &mut trial_slots[index], first_error, current);
        if advanced && first_error(&trial) != *current {
            *work = trial;
            *slots = trial_slots;
            return true;
        }
        // Keep what was learned about this slot, but not the trial values.
        slots[index].next = trial_slots[index].next;
        slots[index].candidates = trial_slots[index].candidates.clone();
    }
    false
}

/// Takes back the values put after slot `index` next to or under its parent.
fn reset_after(work: &mut Value, slots: &mut Vec<Slot>, leaves: &[NullLeaf], index: usize) {
    let parent = render_path(&slots[index].steps[..slots[index].steps.len().saturating_sub(1)]);
    let mut later = index + 1;
    while later < slots.len() {
        if slots[later].path != parent
            && (parent.is_empty() || is_under(&slots[later].path, &parent))
        {
            let slot = slots.remove(later);
            match leaves.iter().find(|leaf| leaf.path == slot.path) {
                Some(leaf) if !leaf.removed => set_at(work, &slot.steps, Value::Null),
                _ => remove_member(work, &slot.steps),
            }
        } else {
            later += 1;
        }
    }
}

/// Resolves a blank error at `at`: the null element there or, when the path
/// stops at an enum, the nulls under it in turn until the error moves.
fn repair_blank<F>(
    work: &mut Value,
    slots: &mut Vec<Slot>,
    leaves: &[NullLeaf],
    at: &str,
    first_error: &F,
    current: &FirstError,
) -> bool
where
    F: Fn(&Value) -> FirstError,
{
    let still_null =
        |work: &Value, leaf: &NullLeaf| walk(work, &leaf.steps).is_some_and(Value::is_null);
    if let Some(leaf) = leaves
        .iter()
        .find(|leaf| leaf.path == at && still_null(work, leaf))
    {
        return place(
            work,
            slots,
            leaves,
            &leaf.path,
            &leaf.steps,
            first_error,
            current,
        );
    }
    for leaf in leaves.iter() {
        if !is_under(&leaf.path, at) || !still_null(work, leaf) {
            continue;
        }
        if place(
            work,
            slots,
            leaves,
            &leaf.path,
            &leaf.steps,
            first_error,
            current,
        ) && first_error(work) != *current
        {
            return true;
        }
    }
    false
}

/// Gives the location a value the kernel accepts and records it as a slot.
/// Returns whether a value was accepted.
fn place<F>(
    work: &mut Value,
    slots: &mut Vec<Slot>,
    leaves: &[NullLeaf],
    path: &str,
    steps: &[PathStep],
    first_error: &F,
    before: &FirstError,
) -> bool
where
    F: Fn(&Value) -> FirstError,
{
    let mut slot = Slot::new(path.to_string(), steps.to_vec());
    let parent = &steps[..steps.len().saturating_sub(1)];
    slot.siblings = leaves
        .iter()
        .filter(|leaf| {
            leaf.removed
                && leaf.path != path
                && leaf.steps.len() == steps.len()
                && render_path(&leaf.steps[..leaf.steps.len() - 1]) == render_path(parent)
        })
        .filter_map(|leaf| match leaf.steps.last() {
            Some(PathStep::Key(key)) => Some(key.clone()),
            _ => None,
        })
        .collect();
    let accepted = advance(work, &mut slot, first_error, before);
    if accepted {
        slots.push(slot);
    }
    accepted
}

/// Tries the slot's next values in order and keeps the first one the kernel
/// does not refuse; a weak acceptance (nothing moved, or a variant whose
/// fields are not all there yet) is kept only when nothing better follows.
fn advance<F>(work: &mut Value, slot: &mut Slot, first_error: &F, before: &FirstError) -> bool
where
    F: Fn(&Value) -> FirstError,
{
    let mut fallback: Option<(Verdict, usize)> = None;
    while slot.next < slot.candidates.len() {
        let index = slot.next;
        slot.next += 1;
        let tried = slot.candidates[index].clone();
        set_at(work, &slot.steps, tried.clone());
        let error = first_error(work);
        let mut verdict = judge(&error, &tried, &slot.path, before, work, first_error);
        if verdict != Verdict::Refuse && !knows_siblings(work, slot, first_error) {
            verdict = Verdict::Poor;
        }
        if verdict == Verdict::Accept {
            return true;
        }
        let rank = |verdict: Verdict| match verdict {
            Verdict::Weak => 1,
            Verdict::Poor => 2,
            _ => 3,
        };
        if verdict != Verdict::Refuse
            && fallback.map_or(true, |(best, _)| rank(verdict) < rank(best))
        {
            fallback = Some((verdict, index));
        }
        if let Some((_, message)) = &error {
            learn(message, slot);
        }
    }
    if let Some((_, index)) = fallback {
        set_at(work, &slot.steps, slot.candidates[index].clone());
        return true;
    }
    false
}

/// Whether the value now at the slot (a variant name) knows every blank
/// member taken out next to it. A member it knows refuses a value of no
/// kernel type; one it does not know is an unknown field, or is silently
/// ignored (a unit variant of an internally tagged enum ignores the rest of
/// the map). The repair works in serde's visiting order, so everything
/// before the slot's parent already deserializes: a refusal of the probe
/// shows at or under that parent, while an error elsewhere is a later one
/// the probe never reached.
fn knows_siblings<F>(work: &Value, slot: &Slot, first_error: &F) -> bool
where
    F: Fn(&Value) -> FirstError,
{
    let parent_steps = &slot.steps[..slot.steps.len().saturating_sub(1)];
    let parent = render_path(parent_steps);
    if slot.siblings.is_empty() {
        return true;
    }
    // Without the probe: a probe that leaves this error as it is was ignored
    // (a unit variant) or never reached, so it shows no knowledge.
    let baseline = first_error(work);
    slot.siblings.iter().all(|key| {
        let mut trial = work.clone();
        let mut steps = parent_steps.to_vec();
        steps.push(PathStep::Key(key.clone()));
        set_at(&mut trial, &steps, serde_json::json!({ "\u{1}probe": [] }));
        let error = first_error(&trial);
        error != baseline
            && error.is_some_and(|(at, message)| {
                // Inside a buffered enum the path stops at the enum, above it.
                (at == parent
                    || is_under(&at, &parent)
                    || (!at.is_empty() && is_under(&parent, &at)))
                    && !message.starts_with(&format!("unknown field `{key}`"))
            })
    })
}

/// Adds the values an error suggests: the variants an enum names, or a list
/// of the length an array needs.
fn learn(message: &str, slot: &mut Slot) {
    let mut suggested = Vec::new();
    if message.starts_with("unknown variant") {
        for name in variants(message) {
            if !slot.variants.contains(&name) {
                slot.variants.push(name);
            }
        }
        suggested.extend(variants(message).into_iter().map(Value::from));
        // An enum: its variants by name, not by index.
        let next = slot.next;
        let mut index = 0;
        slot.candidates.retain(|value| {
            let keep = index < next || !value.is_number();
            index += 1;
            keep
        });
        slot.next = slot.next.min(slot.candidates.len());
    }
    if let Some(rest) = message.split("expected an array of length ").nth(1) {
        if let Some(length) = rest
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .and_then(|digits| digits.parse::<usize>().ok())
        {
            suggested.push(Value::Array(vec![Value::from(0); length]));
        }
    }
    for value in suggested {
        if !slot.candidates.contains(&value) {
            slot.candidates.push(value);
        }
    }
}

/// Whether the error after putting `tried` at `path` refuses that value.
fn judge<F>(
    error: &FirstError,
    tried: &Value,
    path: &str,
    before: &FirstError,
    work: &Value,
    first_error: &F,
) -> Verdict
where
    F: Fn(&Value) -> FirstError,
{
    let Some((at, message)) = error else {
        return Verdict::Accept;
    };
    if at == path || is_under(at, path) {
        // The error is at or inside the value just put there: a struct
        // placeholder still missing its own members is fine, the repair
        // fills those next.
        return if is_blank_error(message) || missing_field(message).is_some() {
            Verdict::Accept
        } else {
            Verdict::Refuse
        };
    }
    if !is_under(path, at) {
        return Verdict::Accept;
    }
    // Above it: an enum or flattened struct that stops the path.
    let kind = match tried {
        Value::Number(_) => "integer",
        Value::String(_) => "string",
        Value::Bool(_) => "boolean",
        Value::Array(_) => "sequence",
        Value::Object(_) => "map",
        Value::Null => "null",
    };
    let refused = message.starts_with(&format!("invalid type: {kind}"))
        || message.starts_with(&format!("invalid value: {kind}"))
        || message.starts_with("invalid length")
        || (tried.is_string()
            && (message.starts_with("unknown variant") || message.starts_with("unknown field")));
    if refused {
        return Verdict::Refuse;
    }
    if let Some(field) = missing_field(message).filter(|_| tried.is_string() && error != before) {
        // A member the object the error points at lacks and knows (it
        // refuses a probe there) is a later error at that level, after the
        // enum deserialized: the variant is fine. Otherwise the variant
        // needs a member the input lacks.
        let later = parse_path(at, field).is_some_and(|steps| {
            let lacks = walk(work, &steps[..steps.len() - 1])
                .and_then(Value::as_object)
                .is_some_and(|map| !map.contains_key(field));
            if !lacks {
                return false;
            }
            let mut trial = work.clone();
            set_at(&mut trial, &steps, serde_json::json!({ "\u{1}probe": [] }));
            let probed = first_error(&trial);
            probed != *error
                && probed.is_some_and(|(_, text)| {
                    !text.starts_with(&format!("unknown field `{field}`"))
                        && missing_field(&text) != Some(field)
                })
        });
        let enum_level = path
            .rsplit_once(['.', '['])
            .map_or("", |(parent, _)| parent);
        if !(later && *at != enum_level) {
            return Verdict::Weak;
        }
    }
    if error == before {
        // Nothing moved: under a buffered enum the value may be refused with
        // the same wording as the blank it replaced, or the error comes from
        // another member of the same name. Text and numbers first, so a
        // scalar member keeps a scalar placeholder.
        return Verdict::Weak;
    }
    Verdict::Accept
}

/// The JSON kind an ``invalid type: <kind>`` or ``invalid value: <kind>``
/// error refuses, in serde's wording.
fn refused_kind(message: &str) -> Option<&'static str> {
    let rest = message
        .strip_prefix("invalid type: ")
        .or_else(|| message.strip_prefix("invalid value: "))?;
    [
        "integer",
        "floating point",
        "string",
        "boolean",
        "sequence",
        "map",
    ]
    .into_iter()
    .find(|kind| rest.starts_with(kind))
    .map(|kind| {
        if kind == "floating point" {
            "integer"
        } else {
            kind
        }
    })
}

/// The kind of a JSON value in serde's wording (numbers as "integer").
fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Number(_) => "integer",
        Value::String(_) => "string",
        Value::Bool(_) => "boolean",
        Value::Array(_) => "sequence",
        Value::Object(_) => "map",
        Value::Null => "null",
    }
}

/// The field named by a ``missing field `X` `` error.
fn missing_field(message: &str) -> Option<&str> {
    message
        .strip_prefix("missing field `")
        .and_then(|rest| rest.split('`').next())
}

/// The variants an `unknown variant` error lists as expected.
fn variants(message: &str) -> Vec<String> {
    let Some(expected) = message.split("expected").nth(1) else {
        return Vec::new();
    };
    expected
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

fn join_path(parent: &str, field: &str) -> String {
    if parent.is_empty() {
        field.to_string()
    } else {
        format!("{parent}.{field}")
    }
}

fn walk<'a>(value: &'a Value, steps: &[PathStep]) -> Option<&'a Value> {
    let mut current = value;
    for step in steps {
        current = match step {
            PathStep::Key(key) => current.get(key.as_str())?,
            PathStep::Index(index) => current.get(*index)?,
        };
    }
    Some(current)
}

/// serde_path_to_error renders the root as `.`; leaves use an empty parent.
fn normal_path(path: &str) -> String {
    if path == "." {
        String::new()
    } else {
        path.to_string()
    }
}

fn is_under(path: &str, prefix: &str) -> bool {
    prefix.is_empty()
        || path.starts_with(&format!("{prefix}."))
        || path.starts_with(&format!("{prefix}["))
}

fn render_path(steps: &[PathStep]) -> String {
    let mut out = String::new();
    for step in steps {
        match step {
            PathStep::Key(key) => {
                if !out.is_empty() {
                    out.push('.');
                }
                out.push_str(key);
            }
            PathStep::Index(index) => out.push_str(&format!("[{index}]")),
        }
    }
    out
}

/// Steps of `parent` (as rendered by serde_path_to_error) plus `field`.
fn parse_path(parent: &str, field: &str) -> Option<Vec<PathStep>> {
    let mut steps = Vec::new();
    for part in parent.split('.').filter(|part| !part.is_empty()) {
        let (key, rest) = part.split_once('[').unwrap_or((part, ""));
        if !key.is_empty() {
            steps.push(PathStep::Key(key.to_string()));
        }
        for index in rest.split('[').filter(|piece| !piece.is_empty()) {
            steps.push(PathStep::Index(index.trim_end_matches(']').parse().ok()?));
        }
    }
    steps.push(PathStep::Key(field.to_string()));
    Some(steps)
}

fn collect_nulls(
    value: &Value,
    path: String,
    steps: Vec<PathStep>,
    out: &mut Vec<(String, Vec<PathStep>)>,
) {
    match value {
        Value::Null => out.push((path, steps)),
        Value::Object(map) => {
            for (key, item) in map {
                let next = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                let mut more = steps.clone();
                more.push(PathStep::Key(key.clone()));
                collect_nulls(item, next, more, out);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let mut more = steps.clone();
                more.push(PathStep::Index(index));
                collect_nulls(item, format!("{path}[{index}]"), more, out);
            }
        }
        _ => {}
    }
}

#[derive(Clone)]
enum PathStep {
    Key(String),
    Index(usize),
}

fn remove_member(value: &mut Value, steps: &[PathStep]) {
    let Some((last, parents)) = steps.split_last() else {
        return;
    };
    let Some(current) = walk_mut(value, parents) else {
        return;
    };
    if let (PathStep::Key(key), Value::Object(map)) = (last, current) {
        map.remove(key);
    }
}

fn set_at(value: &mut Value, steps: &[PathStep], replacement: Value) {
    let Some((last, parents)) = steps.split_last() else {
        return;
    };
    let Some(current) = walk_mut(value, parents) else {
        return;
    };
    match (last, current) {
        (PathStep::Key(key), Value::Object(map)) => {
            map.insert(key.clone(), replacement);
        }
        (PathStep::Index(index), Value::Array(items)) => {
            if let Some(slot) = items.get_mut(*index) {
                *slot = replacement;
            }
        }
        _ => {}
    }
}

fn walk_mut<'a>(value: &'a mut Value, steps: &[PathStep]) -> Option<&'a mut Value> {
    let mut current = value;
    for step in steps {
        current = match step {
            PathStep::Key(key) => current.get_mut(key.as_str())?,
            PathStep::Index(index) => current.get_mut(*index)?,
        };
    }
    Some(current)
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
                    // A blank value (null) where the kernel needs a number,
                    // e.g. one month of a 12-month row or one pump power, is
                    // a missing input at its own path, not a malformed block.
                    // Inside internally tagged enums serde reports the null as
                    // a unit value and stops the path at the enum, so the
                    // blank leaves are located in the JSON itself. Whatever
                    // still fails once the blanks are filled is reported as
                    // an invalid block.
                    let search = blank_paths::<NtaCalculationInput>(value);
                    if search.paths.is_empty() || search.residual {
                        gaps.push(InputGap {
                            detail: Some(error.to_string()),
                            ..gap("nta_calculation_block_invalid", "ntaCalculation")
                        });
                    }
                    for path in search.paths {
                        gaps.push(InputGap {
                            detail: Some(error.to_string()),
                            ..gap("nta_value_missing", format!("ntaCalculation.{path}"))
                        });
                    }
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
    let total_zone_area: f64 = project.zones.iter().map(|zone| zone.floor_area).sum();
    if multi_zone
        && nta
            .vertical_pipes
            .as_ref()
            .is_some_and(|pipes| !pipes.is_empty())
    {
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
    let delta_u_forfait = forfait_bridge_route(&project, &nta, &constructions, gaps);
    let mut used_ground = HashSet::new();
    let mut window_shadings: HashMap<&str, (usize, &ProjectWindowShading)> = HashMap::new();
    for (index, item) in nta.window_shadings.iter().enumerate() {
        let path = format!("ntaCalculation.windowShadings[{index}]");
        if window_shadings
            .insert(item.window_id.as_str(), (index, item))
            .is_some()
        {
            gaps.push(gap("window_shading_duplicate", format!("{path}.windowId")));
        }
        if item.source_reference.trim().is_empty() {
            gaps.push(gap(
                "window_shading_reference_required",
                format!("{path}.sourceReference"),
            ));
        }
    }
    let mut used_shading = HashSet::new();
    let mut window_glazings: HashMap<&str, (usize, &ProjectWindowGlazing)> = HashMap::new();
    for (index, item) in nta.window_glazings.iter().enumerate() {
        let path = format!("ntaCalculation.windowGlazings[{index}]");
        if window_glazings
            .insert(item.window_id.as_str(), (index, item))
            .is_some()
        {
            gaps.push(gap("window_glazing_duplicate", format!("{path}.windowId")));
        }
        if item.source_reference.trim().is_empty() {
            gaps.push(gap(
                "window_glazing_reference_required",
                format!("{path}.sourceReference"),
            ));
        }
    }
    let mut used_glazing = HashSet::new();
    let mut dynamic_windows: HashMap<&str, &crate::annex_a::DynamicTransparent> = HashMap::new();
    for (index, item) in nta.dynamic_windows.iter().enumerate() {
        let path = format!("ntaCalculation.dynamicWindows[{index}]");
        if dynamic_windows
            .insert(item.window_id.as_str(), &item.dynamic)
            .is_some()
        {
            gaps.push(gap("dynamic_window_duplicate", format!("{path}.windowId")));
        }
        // §A.2 (p. 767): movable shading of a dynamic window belongs in its
        // states; the project-wide 7.42 shading would count it twice.
        let shaded = match window_shadings.get(item.window_id.as_str()) {
            Some((_, own)) => own.movable_shading.is_some(),
            None => nta.window_solar.movable_shading.is_some(),
        };
        if shaded {
            gaps.push(gap("window_dynamic_and_shading_exclusive", path.clone()));
        }
        for issue in item.dynamic.validate(&format!("{path}.dynamic")) {
            gaps.push(gap(issue.code, issue.path));
        }
    }
    let mut used_dynamic = HashSet::new();
    let mut window_obstructions: HashMap<&str, (usize, &ProjectWindowObstruction)> = HashMap::new();
    for (index, item) in nta.window_obstructions.iter().enumerate() {
        let path = format!("ntaCalculation.windowObstructions[{index}]");
        if window_obstructions
            .insert(item.window_id.as_str(), (index, item))
            .is_some()
        {
            gaps.push(gap(
                "window_obstruction_duplicate",
                format!("{path}.windowId"),
            ));
        }
        if item.source_reference.trim().is_empty() {
            gaps.push(gap(
                "window_obstruction_reference_required",
                format!("{path}.sourceReference"),
            ));
        }
    }
    let mut used_obstruction = HashSet::new();
    // Every window id of the project and whether it sits in an outdoor
    // surface, so a per-window obstruction is only "without window" when the
    // id names no window at all.
    let mut project_windows: HashMap<&str, bool> = HashMap::new();
    for surface in project.zones.iter().flat_map(|zone| &zone.surfaces) {
        let outdoor = matches!(
            surface
                .get("thermalBoundary")
                .and_then(|value| serde_json::from_value::<ThermalBoundary>(value.clone()).ok()),
            Some(ThermalBoundary::Outdoor)
        );
        for window in surface
            .get("windows")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(id) = window.get("id").and_then(Value::as_str) {
                *project_windows.entry(id).or_insert(false) |= outdoor;
            }
        }
    }
    // The project-wide obstruction: errors that do not depend on the tilt at
    // its own path, once; a situation that needs a vertical window at the
    // first non-vertical window that takes the default.
    for (code, suffix) in validate_obstruction(&nta.window_solar.obstruction, 90.0) {
        gaps.push(gap(
            code,
            format!("ntaCalculation.windowSolar.obstruction{suffix}"),
        ));
    }
    let mut default_tilt_reported = false;
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
                                heated_basement: data.heated_basement.clone().map(
                                    |mut basement| {
                                        // 8.38: the walls take ΔU_for of 8.2.1, the
                                        // same surcharge as H_D.
                                        if let (
                                            EdgeThermalBridges::Forfait,
                                            Some(derived),
                                        ) = (&data.edge_thermal_bridges, delta_u_forfait)
                                        {
                                            if basement.forfait_delta_u_w_per_m2k.is_some_and(
                                                |declared| (declared - derived).abs() > 5e-4,
                                            ) {
                                                let index = nta
                                                    .ground_floors
                                                    .iter()
                                                    .position(|item| item.surface_id == id)
                                                    .unwrap_or_default();
                                                gaps.push(InputGap {
                                                    code: "basement_forfait_delta_u_conflict",
                                                    path: format!("ntaCalculation.groundFloors[{index}].heatedBasement.forfaitDeltaUWPerM2k"),
                                                    detail: Some(format!(
                                                        "8.38 takes ΔU_for of 8.2.1 (8.3): {derived:.3} W/(m²K)"
                                                    )),
                                                });
                                            }
                                            basement.forfait_delta_u_w_per_m2k = Some(derived);
                                        }
                                        basement
                                    },
                                ),
                                source_reference: data.source_reference.clone(),
                            });
                            if let Some(code) = ground_floors
                                .last()
                                .and_then(crate::ground::combination_issue)
                            {
                                let index = nta
                                    .ground_floors
                                    .iter()
                                    .position(|item| item.surface_id == id)
                                    .unwrap_or_default();
                                gaps.push(gap(
                                    code,
                                    format!("ntaCalculation.groundFloors[{index}]"),
                                ));
                            }
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
                let dynamic = dynamic_windows.get(window_id).map(|item| (*item).clone());
                if dynamic.is_some() {
                    used_dynamic.insert(window_id.to_owned());
                }
                // A per-window obstruction replaces the project default; an
                // invalid one is a gap at its own path and the window keeps
                // the default, so the demand reports no second error.
                let obstruction = match window_obstructions.get(window_id) {
                    Some((index, item)) => {
                        used_obstruction.insert(window_id.to_owned());
                        let issues = validate_obstruction(&item.obstruction, tilt);
                        for (code, suffix) in &issues {
                            gaps.push(gap(
                                code,
                                format!(
                                    "ntaCalculation.windowObstructions[{index}].obstruction{suffix}"
                                ),
                            ));
                        }
                        if issues.is_empty() {
                            item.obstruction.clone()
                        } else {
                            nta.window_solar.obstruction.clone()
                        }
                    }
                    None => {
                        if !default_tilt_reported
                            && validate_obstruction(&nta.window_solar.obstruction, tilt)
                                .iter()
                                .any(|(code, _)| *code == "obstruction_situation_requires_vertical")
                        {
                            default_tilt_reported = true;
                            gaps.push(InputGap {
                                code: "obstruction_situation_requires_vertical",
                                path: "ntaCalculation.windowSolar.obstruction.method".into(),
                                detail: Some(format!("window {window_id} (tilt {tilt}°)")),
                            });
                        }
                        nta.window_solar.obstruction.clone()
                    }
                };
                // A per-window entry replaces the project-wide shading.
                // An invalid one is a gap at its own path and the window
                // keeps the default, as for obstructions.
                let shading = match window_shadings.get(window_id) {
                    Some((index, item)) => {
                        used_shading.insert(window_id.to_owned());
                        let issues = item
                            .movable_shading
                            .as_ref()
                            .map(validate_movable_shading)
                            .unwrap_or_default();
                        for (code, suffix) in &issues {
                            gaps.push(gap(
                                code,
                                format!(
                                    "ntaCalculation.windowShadings[{index}].movableShading{suffix}"
                                ),
                            ));
                        }
                        if issues.is_empty() {
                            item.movable_shading.clone()
                        } else {
                            nta.window_solar.movable_shading.clone()
                        }
                    }
                    None => nta.window_solar.movable_shading.clone(),
                };
                let mut source_reference = match window_obstructions.get(window_id) {
                    Some((_, item)) => format!(
                        "project:window:{window_id}; {}; obstruction: {}",
                        nta.window_solar.source_reference, item.source_reference
                    ),
                    None => format!(
                        "project:window:{window_id}; {}",
                        nta.window_solar.source_reference
                    ),
                };
                if let Some((_, item)) = window_shadings.get(window_id) {
                    source_reference.push_str(&format!("; shading: {}", item.source_reference));
                }
                // Glazing details replace the window's gValue route; the
                // demand validates them (7.41 values, louvres) and its
                // issues map back to this entry.
                let glazing = window_glazings.get(window_id).map(|(_, item)| {
                    used_glazing.insert(window_id.to_owned());
                    source_reference.push_str(&format!("; glazing: {}", item.source_reference));
                    item.glazing.clone()
                });
                windows.push(Window {
                    id: format!("window:{window_id}"),
                    area_m2: area,
                    orientation: azimuth_orientation,
                    tilt_deg: tilt,
                    g_perpendicular: g_value,
                    frame_fraction: nta.window_solar.frame_fraction,
                    u_value_w_per_m2k: u_value,
                    forfait_delta_u_w_per_m2k: delta_u_forfait,
                    obstruction,
                    movable_shading: shading,
                    dynamic,
                    glazing,
                    source_reference,
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
                        forfait_delta_u_w_per_m2k: delta_u_forfait,
                        source_reference: format!("project:construction:{construction_id}.uValue"),
                    }),
                    None => gaps.push(gap(
                        "construction_u_value_missing",
                        format!("{path}.constructionId"),
                    )),
                }
            }
        }
        let mut direct =
            direct_boundary_input_zone(&project, ThermalBoundary::Outdoor, None, Some(&zone.id));
        if let (Some(direct), Some(delta)) = (direct.as_mut(), delta_u_forfait) {
            // 8.2 and note 7 of 8.3: ΔU_for on every element of H_D.
            for element in &mut direct.elements {
                element.u_value_w_per_m2k += delta;
                element.source_reference = format!(
                    "{} + ΔU_for {delta:.3} (NTA 8800 8.2/8.3)",
                    element.source_reference
                );
            }
        }
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
                vertical_pipes: Some(match data.and_then(|item| item.vertical_pipes.as_ref()) {
                    // A zone "none" next to listed project pipes would drop
                    // those pipes silently; the input has to say which holds.
                    Some(pipes)
                        if pipes.is_empty()
                            && nta
                                .vertical_pipes
                                .as_ref()
                                .is_some_and(|list| !list.is_empty()) =>
                    {
                        let index = nta
                            .zone_data
                            .iter()
                            .position(|item| item.zone_id == zone.id)
                            .unwrap_or_default();
                        gaps.push(InputGap {
                            detail: Some(
                                "7.3.3: zone list [] (none) conflicts with the project-level pipes; remove one of the two".into(),
                            ),
                            ..gap(
                                "vertical_pipes_conflicting",
                                format!("ntaCalculation.zoneData[{index}].verticalPipes"),
                            )
                        });
                        Vec::new()
                    }
                    Some(pipes) => pipes.clone(),
                    None if !multi_zone && nta.vertical_pipes.is_some() => {
                        nta.vertical_pipes.clone().unwrap_or_default()
                    }
                    None => {
                        // 7.3.3: unknown pipes are not "none"; the fictitious
                        // pipes of the norm have to be entered.
                        let path = match nta
                            .zone_data
                            .iter()
                            .position(|item| item.zone_id == zone.id)
                        {
                            Some(index) if multi_zone => {
                                format!("ntaCalculation.zoneData[{index}].verticalPipes")
                            }
                            _ => "ntaCalculation.verticalPipes".into(),
                        };
                        gaps.push(InputGap {
                            detail: Some(VERTICAL_PIPES_UNKNOWN_DETAIL.into()),
                            ..gap("vertical_pipes_unknown", path)
                        });
                        Vec::new()
                    }
                }),
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
            internal_gains: zone_internal_gains(
                data.map(|item| &item.internal_gains)
                    .unwrap_or(&nta.internal_gains),
                multi_zone,
                zone.floor_area,
                total_zone_area,
            ),
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
    for (index, item) in nta.window_obstructions.iter().enumerate() {
        if used_obstruction.contains(&item.window_id) {
            continue;
        }
        // A window with incomplete data already has its own gap.
        let code = match project_windows.get(item.window_id.as_str()) {
            None => "window_obstruction_without_window",
            Some(false) => "window_obstruction_not_outdoor",
            Some(true) => continue,
        };
        gaps.push(gap(
            code,
            format!("ntaCalculation.windowObstructions[{index}].windowId"),
        ));
    }
    for (index, item) in nta.window_shadings.iter().enumerate() {
        if used_shading.contains(&item.window_id) {
            continue;
        }
        // A window with incomplete data already has its own gap.
        let code = match project_windows.get(item.window_id.as_str()) {
            None => "window_shading_without_window",
            Some(false) => "window_shading_not_outdoor",
            Some(true) => continue,
        };
        gaps.push(gap(
            code,
            format!("ntaCalculation.windowShadings[{index}].windowId"),
        ));
    }
    for (index, item) in nta.window_glazings.iter().enumerate() {
        if used_glazing.contains(&item.window_id) {
            continue;
        }
        let code = match project_windows.get(item.window_id.as_str()) {
            None => "window_glazing_without_window",
            Some(false) => "window_glazing_not_outdoor",
            Some(true) => continue,
        };
        gaps.push(gap(
            code,
            format!("ntaCalculation.windowGlazings[{index}].windowId"),
        ));
    }
    for (index, item) in nta.dynamic_windows.iter().enumerate() {
        if !used_dynamic.contains(&item.window_id) {
            gaps.push(gap(
                "dynamic_window_without_window",
                format!("ntaCalculation.dynamicWindows[{index}].windowId"),
            ));
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
    if !gaps.is_empty() || zones.is_empty() {
        return None;
    }
    // §9.2: zones of the further heating systems leave the main chain.
    let mut assigned: HashSet<String> = HashSet::new();
    let mut extra_chains = Vec::new();
    for (index, system) in nta.additional_heating_systems.iter().enumerate() {
        let path = format!("ntaCalculation.additionalHeatingSystems[{index}]");
        let mut served: Vec<ChainZone> = Vec::new();
        if system.zone_ids.is_empty() {
            gaps.push(gap(
                "heating_system_zones_required",
                format!("{path}.zoneIds"),
            ));
        }
        for zone_id in &system.zone_ids {
            if !assigned.insert(zone_id.clone()) {
                gaps.push(gap("heating_system_zone_twice", format!("{path}.zoneIds")));
                continue;
            }
            match zones
                .iter()
                .position(|zone| &zone.demand.zone_id == zone_id)
            {
                Some(position) => served.push(zones.remove(position)),
                None => gaps.push(gap(
                    "heating_system_zone_unknown",
                    format!("{path}.zoneIds"),
                )),
            }
        }
        if served.is_empty() {
            continue;
        }
        let first = served.remove(0);
        extra_chains.push(SpaceHeatingChainInput {
            humidifiers: system.humidifiers.clone(),
            solar_heating_kwh: Vec::new(),
            solar_recoverable_kwh: Vec::new(),
            hot_water_load_kwh: Vec::new(),
            demand: first.demand,
            emission: first.emission,
            distribution: first.distribution,
            additional_zones: served,
            generator: system.generator.clone(),
            distribution_system: with_unheated_reduction(
                system.distribution_system.clone(),
                &project,
            ),
            collective_connection: system.collective_connection.clone(),
            identical_systems: system.identical_systems,
            regeneration_hot_water: None,
        });
    }
    // Remark 1 (p. 323): a rest set beside an annex Q heat pump estimated
    // at β ≥ 1 needs all its nominal powers or none.
    let partial_rest = |generator: &crate::space_heating_chain::Generator,
                        path: &str,
                        gaps: &mut Vec<InputGap>| {
        if let crate::space_heating_chain::Generator::Multiple(set) = generator {
            for item in crate::space_heating_chain::rest_set_partial_power_paths(set) {
                gaps.push(gap("rest_set_power_partial", format!("{path}.{item}")));
            }
        }
    };
    partial_rest(&nta.generator, "ntaCalculation.generator", gaps);
    for (index, system) in nta.additional_heating_systems.iter().enumerate() {
        partial_rest(
            &system.generator,
            &format!("ntaCalculation.additionalHeatingSystems[{index}].generator"),
            gaps,
        );
    }
    if zones.is_empty() && !nta.additional_heating_systems.is_empty() {
        gaps.push(gap(
            "main_heating_system_without_zones",
            "ntaCalculation.additionalHeatingSystems",
        ));
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
    // 5.5.8 is new in 2025+C1: older editions have no f_BACS.
    let (bacs_factor, bacs_source_reference) = if nta.norm_version.profile().bacs_factor_route {
        (bacs_factor, bacs_source_reference)
    } else {
        (
            1.0,
            format!(
                "{} kent geen f_BACS (5.5.8 is nieuw in 2025+C1)",
                nta.norm_version.label()
            ),
        )
    };
    Some(BuildingPerformanceInput {
        norm_version: nta.norm_version,
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
            regeneration_hot_water: None,
        },
        additional_heating_systems: extra_chains,
        heat_pump_renewable: nta.heat_pump_renewable,
        bacs_factor,
        bacs_source_reference,
        use_inventory_complete: nta.use_inventory_complete,
        declared_uses: nta.declared_uses,
        declared_renewable_heat: nta.declared_renewable_heat,
        production_inventory_complete: nta.production_inventory_complete,
        on_site_production: nta.on_site_production,
        pv_systems: nta.pv_systems,
        hot_water: nta
            .hot_water
            .map(|system| with_hot_water_reduction(system, &project)),
        additional_hot_water_systems: nta
            .additional_hot_water_systems
            .into_iter()
            .map(|system| with_hot_water_reduction(system, &project))
            .collect(),
        space_heating_solar: nta.space_heating_solar,
        lighting: nta.lighting,
        cooling: nta.cooling,
        cooling_systems: nta.cooling_systems,
        label_function: nta.label_function,
        label_functions: nta.label_functions.clone(),
        // One building construction year: the NTA block, else the registration, else the
        // chapter 11 bouwjaar of table 11.13 (the same quantity).
        construction_year: resolved_construction_year(project_value),
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
    fn rejected_tojuli_evidence_becomes_a_project_gap() {
        let zone = |status: &'static str, code: &'static str, path: &str| {
            crate::tojuli::TojuliAssessment {
                zone_id: "z1".into(),
                status,
                active_cooling: true,
                orientations: Vec::new(),
                max_tojuli_k: None,
                meets_bbl_limit: None,
                annex_aa: None,
                issues: vec![crate::tojuli::TojuliIssue {
                    code,
                    path: path.into(),
                }],
                warnings: Vec::new(),
            }
        };
        let gaps = tojuli_evidence_gaps(&[
            zone(
                "invalid",
                "annex_aa_window_unknown",
                "activeCooling.capacity.calculation.rooms[0].windows[0].windowId",
            ),
            // Kernel-internal causes stay out of the project gaps.
            zone("invalid", "tojuli_components_required", "transmission"),
        ]);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].code, "annex_aa_window_unknown");
        assert_eq!(
            gaps[0].path,
            "ntaCalculation.activeCooling.capacity.calculation.rooms[0].windows[0].windowId"
        );
        assert_eq!(gaps[0].detail.as_deref(), Some("TOjuli, rekenzone z1"));
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
        // Omgevingsregeling art. 5.13: the label data carries the calculated indicators.
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

    /// Annex A on a project window: method A weights with step-2 factors.
    /// States open (g 0,60, U 1,10) and shutter closed (g 0,10, U 0,70),
    /// solar weights 0,75/0,25 and temperature weights 0,5/0,5:
    /// g = 0,60·0,75 + 0,10·0,25 = 0,475 and U = 0,90 (A.1/A.2); step 2
    /// with g factor 0,9 and a January U factor 1,1 gives g 0,4275 and
    /// U 0,99 in January, U 0,90 in February.
    #[test]
    fn dynamic_window_carries_annex_a_into_the_demand() {
        let base = assess_project_performance(&project());
        let mut value = project();
        let mut u_factors = vec![1.0; 12];
        u_factors[0] = 1.1;
        value["ntaCalculation"]["dynamicWindows"] = serde_json::json!([{
            "windowId": "win-S",
            "dynamic": {
                "method": "weighted_states",
                "states": [
                    {"id": "open", "gPerpendicular": 0.6, "uValueWPerM2k": 1.1},
                    {"id": "closed", "gPerpendicular": 0.1, "uValueWPerM2k": 0.7}
                ],
                "solarWeights": vec![vec![0.75, 0.25]; 12],
                "temperatureWeights": vec![vec![0.5, 0.5]; 12],
                "sourceReference": "hourly simulation report",
                "correction": {
                    "uFactors": u_factors,
                    "gFactors": vec![0.9; 12],
                    "sourceReference": "step-2 comparison"
                }
            }
        }]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        let demand = &result.derived_input.as_ref().unwrap().space_heating.demand;
        let window = demand
            .windows
            .iter()
            .find(|item| item.id == "window:win-S")
            .unwrap();
        assert!(window.dynamic.is_some());
        assert!((window.g_for_month(0) - 0.4275).abs() < 1e-12);
        assert!((window.u_for_month(0) - 0.99).abs() < 1e-12);
        assert!((window.u_for_month(1) - 0.90).abs() < 1e-12);
        // The nominal U stays in H_D; the month adds 8 m²·(U_mi;mn − 1,10).
        assert!(
            (crate::monthly_demand::dynamic_window_correction_w_per_k(demand, 0) + 0.88).abs()
                < 1e-12
        );
        assert!(
            (crate::monthly_demand::dynamic_window_correction_w_per_k(demand, 1) + 1.6).abs()
                < 1e-12
        );
        let other = demand
            .windows
            .iter()
            .find(|item| item.id == "window:win-N")
            .unwrap();
        assert!(other.dynamic.is_none());
        let transmission = |item: &ProjectPerformanceAssessment, month: usize| {
            item.performance
                .as_ref()
                .unwrap()
                .space_heating
                .demand
                .monthly[month]
                .heating
                .transmission_kwh
        };
        // H_D drops by 0,88 W/K in January (U 0,99 instead of 1,10).
        assert!(transmission(&result, 0) < transmission(&base, 0));
    }

    /// 7.13 (p. 183–184) takes `F_sh;obst;wi,k;mi` per window and 17.3.2
    /// one situation per window: an overhang on win-S lowers its factor and
    /// the heating gains, while win-N keeps the project default.
    #[test]
    fn window_obstruction_replaces_the_default_for_that_window_only() {
        let base = assess_project_performance(&project());
        let mut value = project();
        value["ntaCalculation"]["windowObstructions"] = serde_json::json!([{
            "windowId": "win-S",
            "obstruction": {"method": "overhang", "relativeHeight": 0.5},
            "sourceReference": "balcony above the south window"
        }]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        let demand = &result.derived_input.as_ref().unwrap().space_heating.demand;
        let find = |id: &str| demand.windows.iter().find(|item| item.id == id).unwrap();
        let south = find("window:win-S");
        assert!(matches!(
            south.obstruction,
            Obstruction::Overhang { relative_height } if relative_height == 0.5
        ));
        assert!(south
            .source_reference
            .contains("balcony above the south window"));
        assert!(matches!(
            find("window:win-N").obstruction,
            Obstruction::Minimal
        ));
        let factor = |window: &crate::monthly_demand::Window| {
            crate::monthly_demand::window_obstruction(
                window,
                1,
                crate::solar_shading::Balance::Heating,
            )
            .unwrap()
        };
        let base_demand = &base.derived_input.as_ref().unwrap().space_heating.demand;
        let base_south = base_demand
            .windows
            .iter()
            .find(|item| item.id == "window:win-S")
            .unwrap();
        assert!(factor(south) < factor(base_south));
        let solar = |window: &crate::monthly_demand::Window| {
            crate::monthly_demand::window_solar_kwh(
                window,
                1,
                crate::solar_shading::Balance::Heating,
            )
        };
        assert!(solar(south) < solar(base_south));
    }

    /// Listing a window with the same situation as the project default
    /// changes nothing: projects without `windowObstructions` and projects
    /// that repeat the default give identical results.
    #[test]
    fn window_obstruction_equal_to_the_default_changes_nothing() {
        let base = assess_project_performance(&project());
        let mut value = project();
        value["ntaCalculation"]["windowObstructions"] = serde_json::json!([
            {"windowId": "win-S", "obstruction": {"method": "minimal"}, "sourceReference": "survey"},
            {"windowId": "win-N", "obstruction": {"method": "minimal"}, "sourceReference": "survey"}
        ]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, base.status);
        // Every number is equal; only the source references name the list.
        fn numbers(value: Value) -> Value {
            match value {
                Value::String(_) => Value::Null,
                Value::Array(items) => Value::Array(items.into_iter().map(numbers).collect()),
                Value::Object(map) => Value::Object(
                    map.into_iter()
                        .map(|(key, item)| (key, numbers(item)))
                        .collect(),
                ),
                other => other,
            }
        }
        let json = |item: &ProjectPerformanceAssessment| {
            numbers(serde_json::to_value(item.performance.as_ref().unwrap()).unwrap())
        };
        assert_eq!(json(&result), json(&base));
    }

    /// 7.42/7.43 (2025+C1 p. 196–198) give g_gl and F_c per window wi:
    /// screens on win-S only lower its cooling gains, win-N keeps the project
    /// default (none), and an entry without `movableShading` removes the
    /// project-wide shading from that window.
    #[test]
    fn window_shading_replaces_the_default_for_that_window_only() {
        let shading = serde_json::json!({
            "reductionFactor": 0.2, "control": "manual_residential", "sourceReference": "screen"
        });
        let base = assess_project_performance(&project());
        let mut value = project();
        value["ntaCalculation"]["windowShadings"] = serde_json::json!([{
            "windowId": "win-S", "movableShading": shading, "sourceReference": "screens on the south window"
        }]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        let demand = &result.derived_input.as_ref().unwrap().space_heating.demand;
        let find = |id: &str| demand.windows.iter().find(|item| item.id == id).unwrap();
        assert!(find("window:win-S").movable_shading.is_some());
        assert!(find("window:win-S")
            .source_reference
            .contains("screens on the south window"));
        assert!(find("window:win-N").movable_shading.is_none());
        let cooling = |item: &ProjectPerformanceAssessment| {
            item.performance
                .as_ref()
                .unwrap()
                .space_heating
                .demand
                .monthly
                .iter()
                .map(|month| month.cooling.gains_kwh)
                .sum::<f64>()
        };
        assert!(cooling(&result) < cooling(&base));

        // Project-wide shading, removed again for win-N.
        let mut value = project();
        value["ntaCalculation"]["windowSolar"]["movableShading"] = shading.clone();
        value["ntaCalculation"]["windowShadings"] = serde_json::json!([{
            "windowId": "win-N", "sourceReference": "no screen on the north window"
        }]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        let demand = &result.derived_input.as_ref().unwrap().space_heating.demand;
        let find = |id: &str| demand.windows.iter().find(|item| item.id == id).unwrap();
        assert!(find("window:win-S").movable_shading.is_some());
        assert!(find("window:win-N").movable_shading.is_none());
    }

    /// Projects without `windowShadings`, and projects that repeat the
    /// project default per window, give identical numbers.
    #[test]
    fn window_shading_equal_to_the_default_changes_nothing() {
        let base = assess_project_performance(&project());
        let mut value = project();
        value["ntaCalculation"]["windowShadings"] = serde_json::json!([
            {"windowId": "win-S", "sourceReference": "survey"},
            {"windowId": "win-N", "sourceReference": "survey"}
        ]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, base.status);
        fn numbers(value: Value) -> Value {
            match value {
                Value::String(_) => Value::Null,
                Value::Array(items) => Value::Array(items.into_iter().map(numbers).collect()),
                Value::Object(map) => Value::Object(
                    map.into_iter()
                        .map(|(key, item)| (key, numbers(item)))
                        .collect(),
                ),
                other => other,
            }
        }
        let json = |item: &ProjectPerformanceAssessment| {
            numbers(serde_json::to_value(item.performance.as_ref().unwrap()).unwrap())
        };
        assert_eq!(json(&result), json(&base));
    }

    #[test]
    fn window_shading_input_errors_are_gaps() {
        let mut value = project();
        value["ntaCalculation"]["windowShadings"] = serde_json::json!([
            {"windowId": "win-S", "sourceReference": "a"},
            {"windowId": "win-S", "sourceReference": "b"},
            {"windowId": "missing", "sourceReference": "c"},
            {"windowId": "win-N", "movableShading": {
                "reductionFactor": 1.5, "control": "manual_residential", "sourceReference": "x"
            }, "sourceReference": " "}
        ]);
        let result = assess_project_performance(&value);
        let codes: Vec<(&str, &str)> = result
            .gaps
            .iter()
            .map(|gap| (gap.code, gap.path.as_str()))
            .collect();
        for expected in [
            (
                "window_shading_duplicate",
                "ntaCalculation.windowShadings[1].windowId",
            ),
            (
                "window_shading_without_window",
                "ntaCalculation.windowShadings[2].windowId",
            ),
            (
                "window_shading_reference_required",
                "ntaCalculation.windowShadings[3].sourceReference",
            ),
            (
                "window_shading_factor_invalid",
                "ntaCalculation.windowShadings[3].movableShading.reductionFactor",
            ),
        ] {
            assert!(codes.contains(&expected), "{expected:?} in {codes:?}");
        }
        assert_ne!(result.status, "calculated_unverified");
    }

    /// 7.41 (2025+C1 p. 191) per window: the ISSO 54 EP-W011a values
    /// g_gl,alt 0,045 and g_gl,dif 0,2 give 0,75·0,045 + 0,25·0,2 = 0,08375
    /// for that window only.
    #[test]
    fn window_glazing_replaces_the_g_value_route_for_that_window_only() {
        let base = assess_project_performance(&project());
        let mut value = project();
        value["ntaCalculation"]["windowGlazings"] = serde_json::json!([{
            "windowId": "win-S",
            "glazing": {"diffusing": {
                "gAltitude45": 0.045, "gDiffuse": 0.2, "sourceReference": "ISO 15099 calculation"
            }},
            "sourceReference": "closed horizontal louvres"
        }]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        let demand = &result.derived_input.as_ref().unwrap().space_heating.demand;
        let find = |id: &str| demand.windows.iter().find(|item| item.id == id).unwrap();
        let south = find("window:win-S");
        assert!((south.g_gl(0, 0.0) - 0.08375).abs() < 1e-12);
        assert!(south.source_reference.contains("closed horizontal louvres"));
        assert!(find("window:win-N").glazing.is_none());
        let gains = |item: &ProjectPerformanceAssessment| {
            item.performance
                .as_ref()
                .unwrap()
                .space_heating
                .demand
                .monthly
                .iter()
                .map(|month| month.cooling.gains_kwh)
                .sum::<f64>()
        };
        assert!(gains(&result) < gains(&base));
    }

    /// Projects without `windowGlazings`, and entries without glazing
    /// details, give identical numbers.
    #[test]
    fn window_glazing_without_details_changes_nothing() {
        let base = assess_project_performance(&project());
        let mut value = project();
        value["ntaCalculation"]["windowGlazings"] = serde_json::json!([
            {"windowId": "win-S", "glazing": {}, "sourceReference": "survey"},
            {"windowId": "win-N", "glazing": {}, "sourceReference": "survey"}
        ]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, base.status);
        let numbers = |item: &ProjectPerformanceAssessment| {
            let mut json = serde_json::to_value(item.performance.as_ref().unwrap()).unwrap();
            fn strip(value: &mut Value) {
                match value {
                    Value::String(_) => *value = Value::Null,
                    Value::Array(items) => items.iter_mut().for_each(strip),
                    Value::Object(map) => map.values_mut().for_each(strip),
                    _ => {}
                }
            }
            strip(&mut json);
            json
        };
        assert_eq!(numbers(&result), numbers(&base));
    }

    #[test]
    fn window_glazing_input_errors_are_gaps() {
        let mut value = project();
        value["ntaCalculation"]["windowGlazings"] = serde_json::json!([
            {"windowId": "win-S", "glazing": {}, "sourceReference": "a"},
            {"windowId": "win-S", "glazing": {}, "sourceReference": "b"},
            {"windowId": "missing", "glazing": {}, "sourceReference": "c"},
            {"windowId": "win-N", "glazing": {"diffusing": {
                "gAltitude45": 1.5, "gDiffuse": 0.2, "sourceReference": "x"
            }}, "sourceReference": " "}
        ]);
        let result = assess_project_performance(&value);
        let codes: Vec<(&str, &str)> = result
            .gaps
            .iter()
            .map(|gap| (gap.code, gap.path.as_str()))
            .collect();
        for expected in [
            (
                "window_glazing_duplicate",
                "ntaCalculation.windowGlazings[1].windowId",
            ),
            (
                "window_glazing_without_window",
                "ntaCalculation.windowGlazings[2].windowId",
            ),
            (
                "window_glazing_reference_required",
                "ntaCalculation.windowGlazings[3].sourceReference",
            ),
        ] {
            assert!(codes.contains(&expected), "{expected:?} in {codes:?}");
        }
        assert_ne!(result.status, "calculated_unverified");
        // The demand's own check of the 7.41 values points at the entry.
        let mut value = project();
        value["ntaCalculation"]["windowGlazings"] = serde_json::json!([
            {"windowId": "win-N", "glazing": {"diffusing": {
                "gAltitude45": 1.5, "gDiffuse": 0.2, "sourceReference": "x"
            }}, "sourceReference": "y"}
        ]);
        let result = assess_project_performance(&value);
        assert!(
            result.gaps.iter().any(|gap| gap.code == "window_g_invalid"
                && gap.path == "ntaCalculation.windowGlazings[0].glazing.diffusing"),
            "{:?}",
            result.gaps
        );
    }

    #[test]
    fn window_obstruction_input_errors_are_gaps() {
        let mut value = project();
        value["ntaCalculation"]["windowObstructions"] = serde_json::json!([
            {"windowId": "win-S", "obstruction": {"method": "overhang", "relativeHeight": 0.5}, "sourceReference": "a"},
            {"windowId": "win-S", "obstruction": {"method": "minimal"}, "sourceReference": "b"},
            {"windowId": "missing", "obstruction": {"method": "minimal"}, "sourceReference": "c"},
            {"windowId": "win-N", "obstruction": {"method": "overhang", "relativeHeight": -1.0}, "sourceReference": " "}
        ]);
        let result = assess_project_performance(&value);
        let codes: Vec<(&str, &str)> = result
            .gaps
            .iter()
            .map(|gap| (gap.code, gap.path.as_str()))
            .collect();
        for expected in [
            (
                "window_obstruction_duplicate",
                "ntaCalculation.windowObstructions[1].windowId",
            ),
            (
                "window_obstruction_without_window",
                "ntaCalculation.windowObstructions[2].windowId",
            ),
            (
                "obstruction_geometry_invalid",
                "ntaCalculation.windowObstructions[3].obstruction.relativeHeight",
            ),
            (
                "window_obstruction_reference_required",
                "ntaCalculation.windowObstructions[3].sourceReference",
            ),
        ] {
            assert!(codes.contains(&expected), "{expected:?} in {codes:?}");
        }
        assert_eq!(result.status, "incomplete");
    }

    /// Declared obstruction factors need a source; a blank one is a gap at
    /// the input path (per window, with the default kept for that window, and
    /// for the project-wide obstruction), not only an issue deep in the demand.
    #[test]
    fn declared_obstruction_without_source_is_a_gap() {
        let declared = serde_json::json!({
            "method": "declared", "heating": vec![0.9; 12], "cooling": vec![0.8; 12], "sourceReference": " "
        });
        let mut value = project();
        value["ntaCalculation"]["windowObstructions"] = serde_json::json!([
            {"windowId": "win-S", "obstruction": declared, "sourceReference": "survey"}
        ]);
        let result = assess_project_performance(&value);
        assert!(
            result
                .gaps
                .iter()
                .any(|gap| gap.code == "source_reference_required"
                    && gap.path
                        == "ntaCalculation.windowObstructions[0].obstruction.sourceReference"),
            "{:?}",
            result.gaps
        );
        assert_eq!(result.status, "incomplete");
        // One gap: the window keeps the project default, so the demand
        // reports no second error elsewhere.
        assert_eq!(
            result
                .gaps
                .iter()
                .filter(|gap| gap.code == "source_reference_required")
                .count(),
            1
        );

        let mut value = project();
        value["ntaCalculation"]["windowSolar"]["obstruction"] = declared;
        let result = assess_project_performance(&value);
        assert!(
            result
                .gaps
                .iter()
                .any(|gap| gap.code == "source_reference_required"
                    && gap.path == "ntaCalculation.windowSolar.obstruction.sourceReference"),
            "{:?}",
            result.gaps
        );
        assert_ne!(result.status, "calculated_unverified");
    }

    /// "Without window" only for an id that names no window: a window with
    /// incomplete data has its own gap, and a window that no longer borders
    /// outdoor air gets `window_obstruction_not_outdoor`.
    #[test]
    fn obstruction_for_an_existing_window_is_not_without_window() {
        let entry = |id: &str| serde_json::json!({"windowId": id, "obstruction": {"method": "minimal"}, "sourceReference": "s"});
        let mut value = project();
        value["ntaCalculation"]["windowObstructions"] = serde_json::json!([entry("win-N")]);
        value["zones"][0]["surfaces"][0]["windows"][0]
            .as_object_mut()
            .unwrap()
            .remove("gValue");
        let result = assess_project_performance(&value);
        let codes: Vec<&str> = result.gaps.iter().map(|gap| gap.code).collect();
        assert!(codes.contains(&"window_data_missing"), "{codes:?}");
        assert!(
            !codes.contains(&"window_obstruction_without_window"),
            "{codes:?}"
        );

        let mut value = project();
        value["ntaCalculation"]["windowObstructions"] = serde_json::json!([entry("win-N")]);
        value["zones"][0]["surfaces"][0]["thermalBoundary"] = Value::from("unheated_space");
        let result = assess_project_performance(&value);
        assert!(
            result
                .gaps
                .iter()
                .any(|gap| gap.code == "window_obstruction_not_outdoor"
                    && gap.path == "ntaCalculation.windowObstructions[0].windowId"),
            "{:?}",
            result.gaps
        );
        assert!(result
            .gaps
            .iter()
            .all(|gap| gap.code != "window_obstruction_without_window"));
    }

    #[test]
    fn dynamic_window_input_errors_are_gaps() {
        let mut value = project();
        let dynamic = serde_json::json!({
            "method": "single_state",
            "state": {"id": "closed", "gPerpendicular": 0.1, "uValueWPerM2k": 0.7},
            "sourceReference": "product sheet"
        });
        value["ntaCalculation"]["dynamicWindows"] = serde_json::json!([
            {"windowId": "win-S", "dynamic": dynamic},
            {"windowId": "win-S", "dynamic": dynamic},
            {"windowId": "missing", "dynamic": dynamic},
            {"windowId": "win-N", "dynamic": {
                "method": "weighted_states",
                "states": [{"id": "a", "gPerpendicular": 0.5, "uValueWPerM2k": 1.0}],
                "solarWeights": vec![vec![0.5]; 12],
                "temperatureWeights": vec![vec![1.0]; 12],
                "sourceReference": "x"
            }}
        ]);
        let result = assess_project_performance(&value);
        let codes: Vec<(&str, &str)> = result
            .gaps
            .iter()
            .map(|gap| (gap.code, gap.path.as_str()))
            .collect();
        assert!(codes.contains(&(
            "dynamic_window_duplicate",
            "ntaCalculation.dynamicWindows[1].windowId"
        )));
        assert!(codes.contains(&(
            "dynamic_window_without_window",
            "ntaCalculation.dynamicWindows[2].windowId"
        )));
        assert!(codes.contains(&(
            "dynamic_weights_invalid",
            "ntaCalculation.dynamicWindows[3].dynamic.solarWeights"
        )));
        assert_eq!(result.status, "incomplete");
    }

    /// §A.2 (p. 767): a dynamic window with the project-wide movable shading
    /// of 7.42 would count the shaded state twice; a blank form value is a
    /// gap at its own path, not an unreadable block.
    #[test]
    fn dynamic_window_excludes_movable_shading_and_reports_blanks() {
        let mut value = project();
        value["ntaCalculation"]["windowSolar"]["movableShading"] = serde_json::json!({
            "reductionFactor": 0.5,
            "control": "automatic",
            "sourceReference": "screen"
        });
        value["ntaCalculation"]["dynamicWindows"] = serde_json::json!([{
            "windowId": "win-S",
            "dynamic": {
                "method": "single_state",
                "state": {"id": "closed", "gPerpendicular": null, "uValueWPerM2k": 0.7},
                "sourceReference": "product sheet"
            }
        }]);
        let result = assess_project_performance(&value);
        let codes: Vec<(&str, &str)> = result
            .gaps
            .iter()
            .map(|gap| (gap.code, gap.path.as_str()))
            .collect();
        assert!(codes.contains(&(
            "window_dynamic_and_shading_exclusive",
            "ntaCalculation.dynamicWindows[0]"
        )));
        assert!(codes.contains(&(
            "dynamic_value_missing",
            "ntaCalculation.dynamicWindows[0].dynamic.state.gPerpendicular"
        )));
        assert!(!codes
            .iter()
            .any(|(code, _)| *code == "nta_calculation_block_invalid"));
        assert_eq!(result.status, "incomplete");
    }

    /// Omgevingsregeling art. 5.13a lid 1: the label elements the calculation
    /// supplies, and the WLC-GWP taken from the registration.
    #[test]
    fn label_elements_follow_article_5_13a() {
        let mut value: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-terraced-dwelling.json"
        ))
        .unwrap();
        value["registration"] = serde_json::json!({ "wlcGwp": { "valueKgCo2EqPerM2Year": 7.5 } });
        let result = assess_project_performance(&value);
        let performance = result.performance.as_ref().unwrap();
        let elements = &result
            .label_data
            .as_ref()
            .unwrap()
            .indicators
            .as_ref()
            .unwrap()
            .elements;
        assert_eq!(
            elements.operational_co2_kg_per_m2,
            performance.co2_kg_per_m2
        );
        assert_eq!(elements.wlc_gwp_kg_co2_eq_per_m2, Some(7.5));
        assert_eq!(
            elements.final_energy_kwh_per_m2,
            performance
                .chapter5
                .as_ref()
                .map(|item| item.final_energy_kwh_per_m2)
        );
        assert_eq!(
            elements.annual_final_energy_kwh,
            performance.annual_final_energy_kwh
        );
        // The example has a gas boiler and a small PV system.
        assert_eq!(elements.main_energy_carrier, Some("gas"));
        assert!(elements.renewable_production_kwh.unwrap() > 0.0);
        assert!(elements.main_renewable_source.is_some());
        assert_eq!(elements.responds_to_external_signals, None);
        assert_eq!(elements.low_temperature_heating, None);
    }

    /// Art. 5.13a lid 1 onder k en l: the adviser's statements in the
    /// registration reach the label elements; an unanswered one stays `None`.
    #[test]
    fn label_elements_k_and_l_come_from_the_registration() {
        let mut value: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-terraced-dwelling.json"
        ))
        .unwrap();
        value["registration"] = serde_json::json!({
            "labelStatements": {
                "respondsToExternalSignals": false,
                "lowTemperatureHeating": true
            }
        });
        let result = assess_project_performance(&value);
        let elements = &result
            .label_data
            .as_ref()
            .unwrap()
            .indicators
            .as_ref()
            .unwrap()
            .elements;
        assert_eq!(elements.responds_to_external_signals, Some(false));
        assert_eq!(elements.low_temperature_heating, Some(true));
        value["registration"]["labelStatements"] =
            serde_json::json!({ "respondsToExternalSignals": true });
        let result = assess_project_performance(&value);
        let elements = &result
            .label_data
            .as_ref()
            .unwrap()
            .indicators
            .as_ref()
            .unwrap()
            .elements;
        assert_eq!(elements.responds_to_external_signals, Some(true));
        assert_eq!(elements.low_temperature_heating, None);
    }

    /// The example projects of the start screen (src/core/nta/ExampleProjects.ts)
    /// calculate out of the box with plausible indicators.
    #[test]
    fn example_projects_reach_unverified_indicators() {
        let examples = [
            (
                include_str!("../../../training-data/nta8800-example-terraced-dwelling.json"),
                "residential",
                // Gas boiler and gas combi for hot water: BENG 2 of an
                // all-gas dwelling with a small PV system.
                50.0..100.0,
            ),
            (
                include_str!("../../../training-data/nta8800-example-office.json"),
                "office",
                15.0..120.0,
            ),
        ];
        for (json, function, ep2_range) in examples {
            let value: Value = serde_json::from_str(json).unwrap();
            let result = assess_project_performance(&value);
            assert_eq!(
                result.status, "calculated_unverified",
                "{function}: {:?}",
                result.gaps
            );
            // Norm-consistent input: no plausibility findings, one bridge
            // method (forfait: no ψ-values, ΔU_for in H_D), stated pipes,
            // chapter 11 ventilation (BENG 1 from the C1 run) and a
            // calculated hot-water system.
            assert!(
                result.warnings.is_empty(),
                "{function}: {:?}",
                result.warnings
            );
            let derived = result.derived_input.as_ref().unwrap();
            let demand = &derived.space_heating.demand;
            let Transmission::Components(components) = &demand.transmission else {
                panic!("{function}: component transmission expected");
            };
            assert!(components.direct.linear_bridges.is_empty(), "{function}");
            assert!(components.direct.elements[0]
                .source_reference
                .contains("ΔU_for"));
            assert!(!components.vertical_pipes.as_ref().unwrap().is_empty());
            assert!(demand.ventilation.is_some() && demand.ventilation_flows.is_empty());
            assert!(derived.hot_water.is_some());
            assert!(
                !derived
                    .declared_uses
                    .iter()
                    .any(|item| item.service
                        == crate::building_performance::Service::DomesticHotWater)
            );
            let performance = result.performance.as_ref().unwrap();
            assert!(
                performance.need_indicator_kwh_per_m2_year.is_some(),
                "{function}"
            );
            // Bbl art. 4.149b: the TOjuli limit is for woonfuncties only.
            assert!(performance.tojuli_max_k.is_some());
            assert_eq!(
                performance.tojuli_meets_bbl_limit.is_some(),
                function == "residential"
            );
            let ep2 = performance
                .primary_fossil_indicator_kwh_per_m2_year
                .unwrap();
            assert!(ep2_range.contains(&ep2), "{function}: EP2 {ep2}");
            let rer = performance.renewable_share_percent.unwrap();
            assert!((20.0..90.0).contains(&rer), "{function}: RER {rer}");
            let label = performance.indicative_label_class.unwrap();
            assert!(label.starts_with('A'), "{function}: label {label}");
        }
    }

    /// One building bouwjaar (§5.3.2, Omgevingsregeling art. 5.13 onder a): the registration,
    /// else the NTA block; the chapter 11 year (bouw- of renovatiejaar) is
    /// never used and only warned about when it predates the bouwjaar.
    #[test]
    fn construction_year_follows_registration_then_nta_block() {
        let base: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-terraced-dwelling.json"
        ))
        .unwrap();
        let year = |value: &Value| {
            let mut gaps = Vec::new();
            derive_input(value, &mut gaps).and_then(|input| input.construction_year)
        };
        let codes = |value: &Value| {
            plausibility_warnings(value)
                .into_iter()
                .map(|item| item.code)
                .collect::<Vec<_>>()
        };
        assert_eq!(year(&base), Some(2020));
        assert!(codes(&base).is_empty());

        // Without the NTA block year the ventilation year is not a fallback.
        let mut bare = base.clone();
        bare["ntaCalculation"]
            .as_object_mut()
            .unwrap()
            .remove("constructionYear");
        assert_eq!(year(&bare), None);
        assert_eq!(resolved_construction_year(&bare), None);

        // The registration year comes first; a differing block year warns.
        let mut registered = base.clone();
        registered["registration"] = serde_json::json!({"constructionYear": 2010});
        assert_eq!(year(&registered), Some(2010));
        assert!(codes(&registered).contains(&"construction_year_mismatch"));
        registered["registration"]["constructionYear"] = serde_json::json!(2020);
        assert!(!codes(&registered).contains(&"construction_year_mismatch"));

        // A later ventilation year is a renovatiejaar (table 11.13): no
        // warning; an earlier one contradicts the bouwjaar.
        let mut renovated = base.clone();
        renovated["ntaCalculation"]["constructionYear"] = serde_json::json!(1975);
        assert!(!codes(&renovated).contains(&"ventilation_year_before_construction_year"));
        let mut earlier = base.clone();
        earlier["ntaCalculation"]["constructionYear"] = serde_json::json!(2024);
        assert!(codes(&earlier).contains(&"ventilation_year_before_construction_year"));

        // The chapter 11 bouwjaar of a calculation zone is still read.
        let zoned = serde_json::json!({"ntaCalculation": {"zoneData": [
            {"zoneId": "a"},
            {"zoneId": "b", "ventilation": {"constructionYear": 1984}}
        ]}});
        assert_eq!(ventilation_construction_year(&zoned), Some(1984));

        // The label data (art. 4 a) carry the same resolved year as §5.3.2.
        registered["registration"]["constructionYear"] = serde_json::json!(2010);
        let assessment = assess_project_performance(&registered);
        assert_eq!(
            assessment
                .label_data
                .as_ref()
                .and_then(|data| data.general.construction_year),
            Some(2010)
        );
    }

    /// The WLC-GWP area is per building: a terraced dwelling (party walls
    /// not entered as surfaces) is not the whole building; a dwelling
    /// recorded as detached is.
    #[test]
    fn whole_building_needs_a_detached_dwelling() {
        let base: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-terraced-dwelling.json"
        ))
        .unwrap();
        let whole = |value: &Value| {
            let mut gaps = Vec::new();
            let derived = derive_input(value, &mut gaps);
            let project: ProjectInput = serde_json::from_value(value.clone()).unwrap();
            single_detached_dwelling(Some(&project), value, derived.as_ref())
        };
        assert!(!whole(&base));
        let mut terraced = base.clone();
        terraced["registration"] = serde_json::json!({"buildingType": "tussenwoning"});
        assert!(!whole(&terraced));
        let mut semi = base.clone();
        semi["registration"] = serde_json::json!({"buildingType": "half vrijstaande woning"});
        assert!(!whole(&semi));
        let mut detached = base.clone();
        detached["registration"] = serde_json::json!({"buildingType": "vrijstaande woning"});
        assert!(whole(&detached));
        // Table 11.14 types take precedence over the free-text type.
        let mut typed = detached.clone();
        typed["ntaCalculation"]["ventilation"]["infiltration"] = serde_json::json!({
            "method": "reference", "buildingType": "pitched_roof_terraced"
        });
        assert!(!whole(&typed));
        typed["ntaCalculation"]["ventilation"]["infiltration"]["buildingType"] =
            serde_json::json!("pitched_roof_detached");
        assert!(whole(&typed));
    }

    /// §5.5.3: the per-function breakdown adds up to the carriers (5.20),
    /// to EPTot (with the export and storage terms) and to EPrenTot.
    #[test]
    fn example_projects_energy_by_service_reconciles() {
        for json in [
            include_str!("../../../training-data/nta8800-example-terraced-dwelling.json"),
            include_str!("../../../training-data/nta8800-example-office.json"),
        ] {
            let value: Value = serde_json::from_str(json).unwrap();
            let result = assess_project_performance(&value);
            let performance = result.performance.as_ref().unwrap();
            let services = &performance.energy_by_service;
            assert!(!services.months.is_empty());
            for row in &performance.carriers {
                let (used, delivered) = services
                    .months
                    .iter()
                    .filter(|item| item.carrier == row.carrier && item.month == row.month)
                    .fold((0.0, 0.0), |(u, d), item| {
                        (u + item.used_kwh, d + item.delivered_kwh)
                    });
                assert!(
                    (used - row.used_kwh).abs() < 1e-6,
                    "{} {}: {used} vs {}",
                    row.carrier,
                    row.month,
                    row.used_kwh
                );
                assert!((delivered - row.delivered_kwh).abs() < 1e-6);
            }
            let fossil: f64 = services
                .months
                .iter()
                .map(|item| item.primary_fossil_kwh)
                .sum::<f64>()
                - services
                    .adjustments
                    .iter()
                    .map(|item| item.exported_electricity_credit_kwh + item.storage_correction_kwh)
                    .sum::<f64>();
            let expected = performance.annual_primary_fossil_kwh.unwrap();
            assert!((fossil - expected).abs() < 1e-6, "{fossil} vs {expected}");
            let renewable: f64 = services
                .renewable
                .iter()
                .map(|item| item.renewable_primary_kwh)
                .sum::<f64>()
                + services
                    .adjustments
                    .iter()
                    .map(|item| item.renewable_electricity_kwh)
                    .sum::<f64>();
            let expected = performance.annual_renewable_primary_kwh.unwrap();
            assert!(
                (renewable - expected).abs() < 1e-6,
                "{renewable} vs {expected}"
            );
            let annual: f64 = services.annual.iter().map(|item| item.used_kwh).sum();
            let carriers: f64 = performance.carriers.iter().map(|item| item.used_kwh).sum();
            assert!((annual - carriers).abs() < 1e-6);
            assert!(services
                .annual
                .iter()
                .any(|item| item.service == "heating" && item.used_kwh > 0.0));
            assert!(services
                .annual
                .iter()
                .any(|item| item.service == "hotWater" && item.used_kwh > 0.0));
        }
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
        // A blank number is a missing input at its own path.
        assert_eq!(result.gaps[0].code, "nta_value_missing");
        assert_eq!(
            result.gaps[0].path,
            "ntaCalculation.ventilationFlows[0].months[3].conductanceWPerK"
        );
    }

    #[test]
    fn blank_values_inside_tagged_enums_are_missing_inputs() {
        // Internally tagged enums buffer the value, so serde reports a unit
        // value and stops the path at the enum; the blank leaf is found in
        // the JSON itself.
        let mut value = project();
        value["ntaCalculation"]["generator"]["boiler"]["averageDesignEmissionTemperatureC"] =
            Value::Null;
        let result = assess_project_performance(&value);
        let missing: Vec<&str> = result
            .gaps
            .iter()
            .filter(|gap| gap.code == "nta_value_missing")
            .map(|gap| gap.path.as_str())
            .collect();
        assert_eq!(
            missing,
            ["ntaCalculation.generator.boiler.averageDesignEmissionTemperatureC"],
            "{:?}",
            result.gaps
        );
        assert!(!result
            .gaps
            .iter()
            .any(|gap| gap.code == "nta_calculation_block_invalid"));
        // A blank power inside a tagged peak-power enum.
        let mut value = project();
        value["ntaCalculation"]["pvSystems"][0]["peakPower"]["panelPeakPowerW"] = Value::Null;
        let result = assess_project_performance(&value);
        assert!(
            result.gaps.iter().any(|gap| gap.code == "nta_value_missing"
                && gap.path == "ntaCalculation.pvSystems[0].peakPower.panelPeakPowerW"),
            "{:?}",
            result.gaps
        );
        // An optional member that may be null is not reported.
        let mut value = project();
        value["ntaCalculation"]["generator"]["boiler"]["averageDesignEmissionTemperatureC"] =
            Value::Null;
        value["ntaCalculation"]["verticalPipes"] = Value::Null;
        let result = assess_project_performance(&value);
        assert!(!result.gaps.iter().any(
            |gap| gap.path == "ntaCalculation.verticalPipes" && gap.code == "nta_value_missing"
        ));
    }

    /// A probe with nested arrays of internally tagged enums.
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    #[allow(dead_code)]
    struct BlankProbe {
        segments: Vec<ProbeSegment>,
        #[serde(default)]
        note: Option<String>,
    }

    #[derive(serde::Deserialize)]
    #[serde(
        tag = "kind",
        rename_all = "snake_case",
        rename_all_fields = "camelCase",
        deny_unknown_fields
    )]
    #[allow(dead_code)]
    enum ProbeSegment {
        Pipe {
            length_m: f64,
            cover_depth_m: f64,
            temperatures_c: Vec<Option<f64>>,
            #[serde(default)]
            label: Option<String>,
        },
        Group {
            parts: Vec<ProbeSegment>,
        },
    }

    #[test]
    fn blank_search_reports_paired_blanks_and_skips_allowed_nulls() {
        // Months out of operation are null (allowed).
        let months = || {
            (0..12)
                .map(|month| match month {
                    0 | 11 => Value::Null,
                    _ => serde_json::json!(60.0),
                })
                .collect::<Vec<_>>()
        };
        // Two required blanks in one enum; the allowed null months and the
        // optional label are not reported.
        let block = serde_json::json!({
            "segments": [
                {"kind": "pipe", "lengthM": null, "coverDepthM": null,
                 "temperaturesC": months(), "label": null},
                {"kind": "pipe", "lengthM": 12.0, "coverDepthM": null,
                 "temperaturesC": months()}
            ],
            "note": null
        });
        assert_eq!(
            blank_paths::<BlankProbe>(&block).paths,
            [
                "segments[0].coverDepthM",
                "segments[0].lengthM",
                "segments[1].coverDepthM"
            ]
        );
        // Allowed null months plus one real blank in the same enum: only
        // the blank is reported.
        let block = serde_json::json!({"segments": [
            {"kind": "pipe", "lengthM": 5.0, "coverDepthM": null,
             "temperaturesC": months()}
        ]});
        assert_eq!(
            blank_paths::<BlankProbe>(&block).paths,
            ["segments[0].coverDepthM"]
        );
        // Nested arrays of enums: a blank inside a group's part.
        let block = serde_json::json!({"segments": [
            {"kind": "group", "parts": [
                {"kind": "pipe", "lengthM": 1.0, "coverDepthM": 0.6,
                 "temperaturesC": months()},
                {"kind": "pipe", "lengthM": null, "coverDepthM": 0.6,
                 "temperaturesC": months()}
            ]}
        ]});
        assert_eq!(
            blank_paths::<BlankProbe>(&block).paths,
            ["segments[0].parts[1].lengthM"]
        );
        // A null element in a required f64 row is reported, the allowed
        // null row is not.
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct Rows {
            required: Vec<f64>,
            allowed: Vec<Option<f64>>,
        }
        let block = serde_json::json!({"required": [1.0, null, 3.0], "allowed": [null, 2.0]});
        assert_eq!(blank_paths::<Rows>(&block).paths, ["required[1]"]);
        // A complete block has no blanks.
        let block = serde_json::json!({"segments": [
            {"kind": "pipe", "lengthM": 1.0, "coverDepthM": 0.6,
             "temperaturesC": months()}
        ]});
        assert_eq!(blank_paths::<BlankProbe>(&block), BlankSearch::default());
    }

    #[test]
    fn blank_search_covers_enums_defaults_absent_fields_and_flatten() {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        #[allow(dead_code)]
        enum Fuel {
            Gas,
            Oil,
        }
        #[derive(serde::Deserialize)]
        #[serde(
            tag = "kind",
            rename_all = "snake_case",
            rename_all_fields = "camelCase"
        )]
        #[allow(dead_code)]
        enum Generator {
            Boiler { fuel: Fuel, power_kw: f64 },
        }
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct Holder {
            generator: Generator,
        }
        // An enum field and a power field, both blank, inside a tagged enum.
        let block =
            serde_json::json!({"generator": {"kind": "boiler", "fuel": null, "powerKw": null}});
        assert_eq!(
            blank_paths::<Holder>(&block),
            BlankSearch {
                paths: vec!["generator.fuel".into(), "generator.powerKw".into()],
                residual: false
            }
        );

        // A blank `#[serde(default)]` member that is not an Option.
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #[allow(dead_code)]
        struct Defaults {
            #[serde(default)]
            other_loss_kwh: f64,
            #[serde(default)]
            note: Option<String>,
        }
        let block = serde_json::json!({"otherLossKwh": null, "note": null});
        assert_eq!(blank_paths::<Defaults>(&block).paths, ["otherLossKwh"]);

        // A truly absent required field next to an optional null with the
        // same name elsewhere: the absent field is not a blank, the other
        // blank is still found, and the block stays invalid (residual).
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct Sub {
            #[serde(default)]
            x: Option<f64>,
        }
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct Top {
            sub: Sub,
            x: f64,
            y: f64,
        }
        let block = serde_json::json!({"sub": {"x": null}, "y": null});
        assert_eq!(
            blank_paths::<Top>(&block),
            BlankSearch {
                paths: vec!["y".into()],
                residual: true
            }
        );

        // Flattened structs buffer like tagged enums.
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct Inner {
            v: f64,
        }
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct Outer {
            #[serde(flatten)]
            inner: Inner,
            w: Option<f64>,
        }
        let block = serde_json::json!({"v": null, "w": null});
        assert_eq!(blank_paths::<Outer>(&block).paths, ["v"]);

        // Untagged enums report no position; the search degrades to an
        // invalid block instead of a wrong path.
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        #[allow(dead_code)]
        enum Either {
            Number { a: f64 },
            Text { b: String },
        }
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct Wrapper {
            either: Either,
        }
        let search = blank_paths::<Wrapper>(&serde_json::json!({"either": {"a": null}}));
        assert!(
            search.residual || search.paths == ["either.a"],
            "{search:?}"
        );
    }

    /// A blank tag: the variants share a member that is optional in one and
    /// required in the other, so its blank could be reported or not
    /// depending on the variant the repair picks. Only the tag is reported;
    /// a blank next to a filled tag is still classified.
    #[test]
    fn blank_tag_is_reported_alone() {
        #[derive(serde::Deserialize)]
        #[serde(
            tag = "method",
            rename_all = "snake_case",
            rename_all_fields = "camelCase",
            deny_unknown_fields
        )]
        #[allow(dead_code)]
        enum Loss {
            Declared {
                #[serde(default)]
                value_kwh: Option<f64>,
                source: String,
            },
            Measured {
                value_kwh: f64,
            },
        }
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct Holder {
            loss: Loss,
            other: f64,
        }
        for (declared_first, block) in [
            (
                true,
                serde_json::json!({"loss": {"method": null, "valueKwh": null, "source": "s"}, "other": null}),
            ),
            (
                false,
                serde_json::json!({"loss": {"method": null, "valueKwh": null}, "other": null}),
            ),
        ] {
            let search = blank_paths::<Holder>(&block);
            assert_eq!(
                search.paths,
                ["loss.method", "other"],
                "declared_first {declared_first}: {search:?}"
            );
        }
        // With the tag filled the shared member follows its variant.
        let measured =
            serde_json::json!({"loss": {"method": "measured", "valueKwh": null}, "other": 1.0});
        assert_eq!(blank_paths::<Holder>(&measured).paths, ["loss.valueKwh"]);
        let declared = serde_json::json!({"loss": {"method": "declared", "valueKwh": null, "source": "s"}, "other": 1.0});
        assert!(blank_paths::<Holder>(&declared).paths.is_empty());
    }

    #[test]
    fn blank_search_tests_each_allowed_null_once() {
        // 200 rows of allowed nulls inside a tagged enum plus one blank.
        let rows: Vec<Value> = (0..200)
            .map(|_| {
                serde_json::json!({"kind": "pipe", "lengthM": 1.0, "coverDepthM": 0.6,
                    "temperaturesC": vec![Value::Null; 12]})
            })
            .collect();
        let mut block = serde_json::json!({ "segments": rows });
        block["segments"][150]["lengthM"] = Value::Null;
        let before = BLANK_SEARCH_DESERIALIZATIONS.with(|count| count.get());
        let search = blank_paths::<BlankProbe>(&block);
        let used = BLANK_SEARCH_DESERIALIZATIONS.with(|count| count.get()) - before;
        assert_eq!(search.paths, ["segments[150].lengthM"]);
        assert!(!search.residual);
        // 2 401 nulls: linear in the nulls, not quadratic (a per-null retry
        // of every allowed null would need millions).
        assert!(used < 12 * 2401 + 200, "{used} deserializations");
    }

    #[test]
    fn two_blanks_in_one_generator_are_both_missing_inputs() {
        let mut value = project();
        value["ntaCalculation"]["generator"]["boiler"]["averageDesignEmissionTemperatureC"] =
            Value::Null;
        value["ntaCalculation"]["generator"]["boiler"]["fuel"] = Value::Null;
        let result = assess_project_performance(&value);
        let mut missing: Vec<&str> = result
            .gaps
            .iter()
            .filter(|gap| gap.code == "nta_value_missing")
            .map(|gap| gap.path.as_str())
            .collect();
        missing.sort_unstable();
        assert_eq!(
            missing,
            [
                "ntaCalculation.generator.boiler.averageDesignEmissionTemperatureC",
                "ntaCalculation.generator.boiler.fuel"
            ],
            "{:?}",
            result.gaps
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
        // A zone "none" next to listed project pipes is a conflict, not a
        // silent drop of the project pipes.
        let mut conflicting = value.clone();
        conflicting["ntaCalculation"]["zoneData"] = serde_json::json!([{
            "zoneId": "z1",
            "verticalPipes": [],
            "ventilationFlows": value["ntaCalculation"]["ventilationFlows"].clone(),
            "internalGains": value["ntaCalculation"]["internalGains"].clone()
        }]);
        let result = assess_project_performance(&conflicting);
        assert_eq!(result.status, "incomplete");
        assert!(result
            .gaps
            .iter()
            .any(|gap| gap.code == "vertical_pipes_conflicting"
                && gap.path == "ntaCalculation.zoneData[0].verticalPipes"));
        // 7.3.3: an absent list is "unknown", not "none".
        value["ntaCalculation"]
            .as_object_mut()
            .unwrap()
            .remove("verticalPipes");
        let unknown = assess_project_performance(&value);
        assert_eq!(unknown.status, "incomplete");
        assert!(unknown
            .gaps
            .iter()
            .any(|gap| gap.code == "vertical_pipes_unknown"
                && gap.path == "ntaCalculation.verticalPipes"
                && gap.detail.is_some()));
    }

    /// 8.2.1/8.3.3.1: the forfait floor edge selects ΔU_for for the whole
    /// building; ψ-values next to it mix the methods.
    #[test]
    fn forfait_bridges_apply_to_the_whole_building() {
        let mut value = project();
        value["ntaCalculation"]["groundFloors"][0]["edgeThermalBridges"] =
            serde_json::json!({"method": "forfait"});
        let mixed = assess_project_performance(&value);
        assert_eq!(mixed.status, "incomplete");
        assert!(mixed
            .gaps
            .iter()
            .any(|gap| gap.code == "thermal_bridge_methods_mixed"));

        value["zones"][0]["thermalBridges"] = serde_json::json!([]);
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified", "{:?}", result.gaps);
        // 8.3: opaque walls 98 m² at 0,21 and roof 52 m² at 0,16.
        let mean = (98.0 * 0.21 + 52.0 * 0.16) / 150.0;
        let delta = 0.1 - 0.25 * (mean - 0.4);
        // 8.2: ΔU_for on all 162 m² of H_D, glass included (note 7).
        let expected = 98.0 * 0.21 + 52.0 * 0.16 + 12.0 * 1.1 + 162.0 * delta;
        let summary = result
            .performance
            .as_ref()
            .unwrap()
            .space_heating
            .demand
            .transmission
            .clone()
            .unwrap();
        assert!((summary.direct_conductance_w_per_k.unwrap() - expected).abs() < 1e-9);
        // 7.33/7.39 take U_c of 8.2.2: ΔU_for stays out of the solar terms
        // and is carried separately for the TOjuli split of H_D.
        let demand = &result.derived_input.as_ref().unwrap().space_heating.demand;
        let roof = demand
            .opaque_elements
            .iter()
            .find(|item| item.id == "surface:roof:opaque")
            .unwrap();
        assert_eq!(roof.u_value_w_per_m2k, 0.16);
        assert!((roof.forfait_delta_u_w_per_m2k.unwrap() - delta).abs() < 1e-12);
        let mut without = roof.clone();
        without.forfait_delta_u_w_per_m2k = None;
        for month in 1..=12 {
            assert_eq!(
                crate::monthly_demand::opaque_solar_kwh(roof, month),
                crate::monthly_demand::opaque_solar_kwh(&without, month)
            );
        }
        assert!(demand
            .windows
            .iter()
            .all(|window| window.u_value_w_per_m2k == 1.1
                && window.forfait_delta_u_w_per_m2k
                    == Some(roof.forfait_delta_u_w_per_m2k.unwrap())));
        // The TOjuli split still covers H_D including ΔU_for.
        let performance = result.performance.as_ref().unwrap();
        assert!(!performance.tojuli.is_empty());
        assert!(performance
            .tojuli
            .iter()
            .all(|item| item.status != "invalid"));
        assert!(performance.tojuli_max_k.is_some());

        // 8.38: a heated basement takes the derived ΔU_for of 8.2.1; a
        // conflicting declared value is a gap, an absent one is filled in.
        let mut basement = value.clone();
        basement["ntaCalculation"]["groundFloors"][0]["heatedBasement"] = serde_json::json!({
            "depthM": 1.5, "wallResistanceM2kPerW": 2.5
        });
        let derived = assess_project_performance(&basement);
        assert_eq!(
            derived.status, "calculated_unverified",
            "{:?}",
            derived.gaps
        );
        let slab = &derived.derived_input.as_ref().unwrap().space_heating.demand;
        let crate::monthly_demand::Transmission::Components(components) = &slab.transmission else {
            panic!("component transmission");
        };
        let filled = components.ground_floors[0]
            .heated_basement
            .as_ref()
            .unwrap()
            .forfait_delta_u_w_per_m2k
            .unwrap();
        assert!((filled - delta).abs() < 1e-12);
        basement["ntaCalculation"]["groundFloors"][0]["heatedBasement"]["forfaitDeltaUWPerM2k"] =
            Value::from(delta + 0.05);
        let conflict = assess_project_performance(&basement);
        assert!(conflict
            .gaps
            .iter()
            .any(|gap| gap.code == "basement_forfait_delta_u_conflict"
                && gap.path
                    == "ntaCalculation.groundFloors[0].heatedBasement.forfaitDeltaUWPerM2k"));

        // H_U;for = 0 with U_iu;equi (C.1.3) is not derived here.
        value["zones"][0]["surfaces"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "id": "garage-wall", "name": "Wand garage", "type": "wall",
                "thermalBoundary": "unheated_space", "unheatedSpaceId": "garage",
                "area": 10.0, "orientation": "E", "constructionId": "c-wall",
                "zoneId": "z1", "windows": []
            }));
        assert!(assess_project_performance(&value)
            .gaps
            .iter()
            .any(|gap| gap.code == "forfait_thermal_bridges_unheated_space_unsupported"));
    }

    #[test]
    fn chapter8_plausibility_and_ground_combinations() {
        let example = || -> Value {
            serde_json::from_str(include_str!(
                "../../../training-data/nta8800-example-terraced-dwelling.json"
            ))
            .unwrap()
        };
        let codes = |value: &Value| -> Vec<&'static str> {
            plausibility_warnings(value)
                .iter()
                .map(|item| item.code)
                .collect()
        };
        assert!(codes(&example()).is_empty());

        // R_si + R_c below R_si, and P = 120 m on 50 m² (limit 2·50/1 + 2).
        let mut floor = example();
        floor["ntaCalculation"]["groundFloors"][0]["constructionResistanceM2kPerW"] =
            Value::from(0.05);
        floor["ntaCalculation"]["groundFloors"][0]["exposedPerimeterM"] = Value::from(120.0);
        assert_eq!(
            codes(&floor),
            [
                "ground_floor_resistance_below_surface_resistance",
                "ground_floor_perimeter_implausible"
            ]
        );

        // Detailed floor edge without any ψ to outside air.
        let mut detailed = example();
        detailed["ntaCalculation"]["groundFloors"][0]["edgeThermalBridges"] = serde_json::json!({
            "method": "detailed",
            "bridges": [{"lengthM": 10.0, "psiWPerMk": 0.1, "sourceReference": "x"}]
        });
        assert_eq!(codes(&detailed), ["detailed_thermal_bridges_none_entered"]);

        // A crawlspace with edge insulation is a top-level gap.
        let mut crawl = example();
        crawl["ntaCalculation"]["groundFloors"][0]["below"] = serde_json::json!({
            "kind": "crawlspace",
            "floorResistanceM2kPerW": 0.0,
            "depthClass": "other",
            "wallResistanceM2kPerW": 0.35,
            "wallUValueWPerM2k": 1.9
        });
        let fine = assess_project_performance(&crawl);
        assert_eq!(fine.status, "calculated_unverified", "{:?}", fine.gaps);
        crawl["ntaCalculation"]["groundFloors"][0]["edgeInsulation"] = serde_json::json!([{
            "kind": "vertical", "resistanceM2kPerW": 2.0, "thicknessM": 0.1,
            "sourceReference": "x"
        }]);
        let result = assess_project_performance(&crawl);
        assert_eq!(result.status, "incomplete");
        assert!(result
            .gaps
            .iter()
            .any(|gap| gap.code == "ground_floor_edge_insulation_slab_only"
                && gap.path == "ntaCalculation.groundFloors[0]"));
    }

    #[test]
    fn sunroom_values_are_checked_against_the_unheated_space() {
        let mut value: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-terraced-dwelling.json"
        ))
        .unwrap();
        value["constructions"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "id": "c-inner", "name": "Binnenwand", "layers": [], "rcValue": 0.5, "uValue": 1.4
            }));
        value["zones"][0]["surfaces"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "id": "wall-serre", "name": "Wand serre", "type": "wall",
                "thermalBoundary": "unheated_space", "unheatedSpaceId": "serre",
                "area": 10.0, "orientation": "S", "constructionId": "c-inner",
                "zoneId": "z1", "windows": []
            }));
        value["unheatedSpaces"] = serde_json::json!([{
            "id": "serre", "name": "Serre",
            "outside": {
                "transmission": {"elements": [{
                    "id": "glass", "areaM2": 20.0, "uValueWPerM2k": 2.8,
                    "sourceReference": "survey"
                }]},
                "ventilation": {"method": "half_of_transmission"},
                "otherZonesConductanceWPerK": 0.0
            }
        }]);
        let project: ProjectInput = serde_json::from_value(value.clone()).unwrap();
        let derived = crate::unheated_transmission::assess_unheated_transmission(
            &unheated_zone_input(&project, None).unwrap(),
        );
        let space = &derived.spaces[0];
        let sunroom = |b: f64, h: f64| {
            serde_json::json!([{
                "id": "serre", "glazingGHeating": 0.6, "glazingGCooling": 0.6,
                "exteriorFrameFraction": 0.2, "reductionFactor": b,
                "zoneConductanceWPerK": h, "surfaces": [], "sourceReference": "x"
            }])
        };
        value["ntaCalculation"]["sunrooms"] =
            sunroom(space.reduction_factor, space.unreduced_conductance_w_per_k);
        assert!(!plausibility_warnings(&value)
            .iter()
            .any(|item| item.code == "sunroom_values_differ_from_unheated_space"));
        value["ntaCalculation"]["sunrooms"] = sunroom(0.3, 99.0);
        let warnings = plausibility_warnings(&value);
        let warning = warnings
            .iter()
            .find(|item| item.code == "sunroom_values_differ_from_unheated_space")
            .unwrap();
        assert_eq!(warning.path, "ntaCalculation.sunrooms[0]");
    }

    #[test]
    fn plausibility_warnings_leave_the_calculation_running() {
        // The synthetic dwelling declares 1 800 kWh gas for 1 952 kWh need
        // and 35 W/K for a C system: about 152 m³/h (51 W/K) at f_ctrl 1,
        // but demand control (table 11.5, lowest 0,52) may bring it to
        // 26,5 W/K, so 35 W/K is plausible.
        let mut value = project();
        let result = assess_project_performance(&value);
        assert_eq!(result.status, "calculated_unverified");
        let codes: Vec<_> = result.warnings.iter().map(|item| item.code).collect();
        assert_eq!(codes, ["declared_hot_water_efficiency_above_one"]);
        // 20 W/K is below even the demand-controlled flow.
        for flow in value["ntaCalculation"]["ventilationFlows"]
            .as_array_mut()
            .unwrap()
        {
            for month in flow["months"].as_array_mut().unwrap() {
                month["conductanceWPerK"] = Value::from(20.0);
            }
        }
        let low = assess_project_performance(&value);
        let codes: Vec<_> = low.warnings.iter().map(|item| item.code).collect();
        assert_eq!(
            codes,
            [
                "declared_hot_water_efficiency_above_one",
                "declared_ventilation_below_required_flow"
            ]
        );
        assert!(low.warnings[1]
            .detail
            .as_ref()
            .unwrap()
            .contains("26.5 W/K"));

        // Utility: f_BACS 1,0 without the bacs block and an open ceiling.
        let mut office: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-office.json"
        ))
        .unwrap();
        office["ntaCalculation"]
            .as_object_mut()
            .unwrap()
            .remove("bacs");
        office["ntaCalculation"]["thermalMass"]["ceiling"] = Value::from("open_or_none");
        let codes: Vec<_> = plausibility_warnings(&office)
            .iter()
            .map(|item| item.code)
            .collect();
        assert_eq!(
            codes,
            [
                "bacs_factor_without_capacity_evidence",
                "utility_open_ceiling_requires_evidence"
            ]
        );
        // The same declarations in a dwelling are the norm defaults.
        let mut dwelling = project();
        dwelling["ntaCalculation"]["declaredUses"] = serde_json::json!([]);
        dwelling["ntaCalculation"]["ventilationFlows"][0]["months"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .for_each(|month| month["conductanceWPerK"] = Value::from(55.0));
        assert!(plausibility_warnings(&dwelling).is_empty());
    }

    /// Review 9 October 2026: a refused route in the second zone, or in an
    /// additional heating system, has to land on the project input that
    /// feeds it, not on the first zone's or the project's.
    #[test]
    fn refusal_paths_follow_zones_floors_and_systems() {
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
        let block = &mut value["ntaCalculation"];
        // z1 takes emission and thermal mass from the project block, z2 from
        // its own entry; both have their own internal gains.
        let mut z1 = serde_json::json!({
            "zoneId": "z1",
            "verticalPipes": [],
            "ventilationFlows": block["ventilationFlows"].clone(),
            "internalGains": block["internalGains"].clone()
        });
        let mut z2 = z1.clone();
        z2["zoneId"] = Value::from("z2");
        z2["emission"] = block["emission"].clone();
        z2["thermalMass"] = block["thermalMass"].clone();
        z1["zoneId"] = Value::from("z1");
        block["zoneData"] = serde_json::json!([z1, z2]);
        block["surfaceTilts"].as_array_mut().unwrap().push(
            serde_json::json!({"surfaceId": "roof-2", "tiltDeg": 45.0, "sourceReference": "copy"}),
        );
        // The floor of z2 is listed first, so its derived index (0 in z2)
        // differs from its project index (0 here, z1's floor at 1).
        let mut floor = block["groundFloors"][0].clone();
        floor["surfaceId"] = Value::from("floor-2");
        block["groundFloors"]
            .as_array_mut()
            .unwrap()
            .insert(0, floor);
        let window_two = value["zones"][1]["surfaces"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .find_map(|(surface, item)| {
                item["windows"]
                    .as_array()
                    .filter(|windows| !windows.is_empty())
                    .map(|windows| (surface, windows[0]["id"].as_str().unwrap().to_owned()))
            })
            .unwrap();
        value["ntaCalculation"]["windowObstructions"] = serde_json::json!([{
            "windowId": window_two.1,
            "obstruction": {"method": "minimal"},
            "sourceReference": "test"
        }]);
        let mut gaps = Vec::new();
        let derived = derive_input(&value, &mut gaps).unwrap_or_else(|| panic!("{gaps:?}"));
        assert_eq!(
            derived.space_heating.additional_zones[0].demand.zone_id,
            "z2"
        );
        let floor_z1 = value["ntaCalculation"]["groundFloors"]
            .as_array()
            .unwrap()
            .iter()
            .position(|item| item["surfaceId"] != "floor-2")
            .unwrap();
        let window_in_z2 = derived.space_heating.additional_zones[0]
            .demand
            .windows
            .iter()
            .position(|item| item.id == format!("window:{}", window_two.1))
            .unwrap();
        for (path, expected) in [
            (
                "spaceHeating.additionalZones[0].demand.transmission.groundFloors[0].heatedBasement"
                    .to_string(),
                "ntaCalculation.groundFloors[0].heatedBasement".to_string(),
            ),
            (
                "spaceHeating.demand.transmission.groundFloors[0].heatedBasement".to_string(),
                format!("ntaCalculation.groundFloors[{floor_z1}].heatedBasement"),
            ),
            (
                "spaceHeating.additionalZones[0].emission.fans".to_string(),
                "ntaCalculation.zoneData[1].emission.fans".to_string(),
            ),
            (
                "spaceHeating.emission.fans".to_string(),
                "ntaCalculation.emission.fans".to_string(),
            ),
            (
                "spaceHeating.additionalZones[0].demand.internalGains.lighting".to_string(),
                "ntaCalculation.zoneData[1].internalGains.lighting".to_string(),
            ),
            (
                "spaceHeating.demand.internalGains.lighting".to_string(),
                "ntaCalculation.zoneData[0].internalGains.lighting".to_string(),
            ),
            (
                "spaceHeating.additionalZones[0].demand.thermalMass".to_string(),
                "ntaCalculation.zoneData[1].thermalMass".to_string(),
            ),
            (
                "spaceHeating.demand.thermalMass".to_string(),
                "ntaCalculation.thermalMass".to_string(),
            ),
            (
                "spaceHeating.demand.transmission.verticalPipes[0]".to_string(),
                "ntaCalculation.zoneData[0].verticalPipes[0]".to_string(),
            ),
            (
                format!("spaceHeating.additionalZones[0].demand.windows[{window_in_z2}].obstruction.method"),
                "ntaCalculation.windowObstructions[0].obstruction.method".to_string(),
            ),
            (
                format!("spaceHeating.additionalZones[0].demand.windows[{window_in_z2}].gPerpendicular"),
                format!("zones[1].surfaces[{}].windows[0].gValue", window_two.0),
            ),
            (
                "spaceHeating.demand.windows[0].obstruction".to_string(),
                "ntaCalculation.windowSolar.obstruction".to_string(),
            ),
            (
                "spaceHeating.demand.windows[0].frameFraction".to_string(),
                "ntaCalculation.windowSolar.frameFraction".to_string(),
            ),
        ] {
            assert_eq!(project_path_for_derived(&path, &derived, &value), expected, "{path}");
        }
        // An additional heating system serving z2 has its own collective
        // connection and number of identical systems.
        value["ntaCalculation"]["additionalHeatingSystems"] = serde_json::json!([{
            "zoneIds": ["z2"],
            "generator": value["ntaCalculation"]["generator"].clone(),
            "identicalSystems": 2
        }]);
        let mut gaps = Vec::new();
        let derived = derive_input(&value, &mut gaps).unwrap_or_else(|| panic!("{gaps:?}"));
        for (path, expected) in [
            (
                "additionalHeatingSystems[0].identicalSystems",
                "ntaCalculation.additionalHeatingSystems[0].identicalSystems",
            ),
            (
                "additionalHeatingSystems[0].collectiveConnection.x",
                "ntaCalculation.additionalHeatingSystems[0].collectiveConnection.x",
            ),
            (
                "additionalHeatingSystems[0].demand.transmission.groundFloors[0]",
                "ntaCalculation.groundFloors[0]",
            ),
            (
                "additionalHeatingSystems[0].emission.fans",
                "ntaCalculation.zoneData[1].emission.fans",
            ),
            (
                "spaceHeating.identicalSystems",
                "ntaCalculation.identicalSystems",
            ),
        ] {
            assert_eq!(
                project_path_for_derived(path, &derived, &value),
                expected,
                "{path}"
            );
        }
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
                "verticalPipes": [],
                "ventilationFlows": block["ventilationFlows"].clone(),
                "internalGains": block["internalGains"].clone()
            })
        };
        let entries = vec![zone_entry("z1"), zone_entry("z2")];
        block["zoneData"] = Value::from(entries);
        block["surfaceTilts"].as_array_mut().unwrap().push(
            serde_json::json!({"surfaceId": "roof-2", "tiltDeg": 45.0, "sourceReference": "copy"}),
        );
        let mut floor = block["groundFloors"][0].clone();
        floor["surfaceId"] = Value::from("floor-2");
        block["groundFloors"].as_array_mut().unwrap().push(floor);
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
        let need = |performance: &BuildingPerformanceAssessment| -> f64 {
            performance
                .space_heating
                .monthly
                .iter()
                .map(|row| row.heating_need_kwh)
                .sum()
        };
        let one_system = need(performance);

        // §9.2: zone z2 gets its own heating system; z1 stays on the main one.
        let generator = value["ntaCalculation"]["generator"].clone();
        value["ntaCalculation"]["additionalHeatingSystems"] =
            serde_json::json!([{"zoneIds": ["z2"], "generator": generator}]);
        let split = assess_project_performance(&value);
        assert_eq!(split.status, "calculated_unverified", "{:?}", split.gaps);
        let derived = split.derived_input.as_ref().unwrap();
        assert!(derived.space_heating.additional_zones.is_empty());
        assert_eq!(derived.additional_heating_systems.len(), 1);
        assert_eq!(derived.additional_heating_systems[0].demand.zone_id, "z2");
        let performance = split.performance.as_ref().unwrap();
        assert_eq!(performance.tojuli.len(), 2);
        assert!((need(performance) - one_system).abs() < 1e-6 * one_system);

        // Gaps: an unknown zone, a zone in two systems, no zone left for
        // the main system, and a system without zones.
        let codes = |systems: Value| -> Vec<&'static str> {
            let mut copy = value.clone();
            copy["ntaCalculation"]["additionalHeatingSystems"] = systems;
            assess_project_performance(&copy)
                .gaps
                .iter()
                .map(|item| item.code)
                .collect()
        };
        let generator = &value["ntaCalculation"]["generator"];
        let found = codes(serde_json::json!([{"zoneIds": ["nope"], "generator": generator}]));
        assert!(found.contains(&"heating_system_zone_unknown"), "{found:?}");
        let found = codes(serde_json::json!([
            {"zoneIds": ["z2"], "generator": generator},
            {"zoneIds": ["z2"], "generator": generator}
        ]));
        assert!(found.contains(&"heating_system_zone_twice"), "{found:?}");
        let found = codes(serde_json::json!([{"zoneIds": ["z1", "z2"], "generator": generator}]));
        assert!(
            found.contains(&"main_heating_system_without_zones"),
            "{found:?}"
        );
        let found = codes(serde_json::json!([{"zoneIds": [], "generator": generator}]));
        assert!(
            found.contains(&"heating_system_zones_required"),
            "{found:?}"
        );
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

/// Random blanks in the example projects: every reported path must be one
/// the kernel rejects as null on its own, and every such blank must be
/// reported. Ground truth per leaf: the otherwise complete block fails with
/// only that leaf null.
#[cfg(test)]
mod blank_fuzz {
    use super::*;
    use std::collections::{BTreeSet, HashMap};

    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    /// Nodes below the root as (rendered path, JSON pointer, scalar):
    /// scalar leaves, and the objects and arrays (array elements included)
    /// that a user can blank as a whole.
    fn nodes(value: &Value, path: String, pointer: String, out: &mut Vec<(String, String, bool)>) {
        let children: Vec<(String, String, &Value)> = match value {
            Value::Object(map) => map
                .iter()
                .map(|(key, item)| {
                    let next = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    let next_pointer =
                        format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1"));
                    (next, next_pointer, item)
                })
                .collect(),
            Value::Array(items) => items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    (
                        format!("{path}[{index}]"),
                        format!("{pointer}/{index}"),
                        item,
                    )
                })
                .collect(),
            _ => Vec::new(),
        };
        for (next, next_pointer, item) in children {
            match item {
                Value::Null => {}
                Value::Object(_) | Value::Array(_) => {
                    out.push((next.clone(), next_pointer.clone(), false));
                    nodes(item, next, next_pointer, out);
                }
                _ => out.push((next, next_pointer, true)),
            }
        }
    }

    fn deserializes(value: &Value) -> bool {
        serde_json::from_value::<NtaCalculationInput>(value.clone()).is_ok()
    }

    fn training(file: &str) -> Value {
        let text = std::fs::read_to_string(format!(
            "{}/../../training-data/{file}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn block(file: &str) -> Value {
        let block = training(file)["ntaCalculation"].clone();
        assert!(deserializes(&block), "{file} must deserialize");
        block
    }

    /// The office block with a multiple-generator set (two preferences of
    /// its own generator) and the calculated annex P heat network of the
    /// annex P fixture: tagged enums inside arrays inside tagged enums.
    fn multiple_and_annex_p_block() -> Value {
        use crate::space_heating_chain::{MultipleGenerators, PreferredGenerator};
        let mut block = block("nta8800-example-office.json");
        let own: Generator = serde_json::from_value(block["generator"].clone()).unwrap();
        let set = Generator::Multiple(Box::new(MultipleGenerators {
            generators: vec![
                PreferredGenerator {
                    preference: 1,
                    nominal_power_kw: 40.0,
                    generator: own.clone(),
                },
                PreferredGenerator {
                    preference: 2,
                    nominal_power_kw: 20.0,
                    generator: own,
                },
            ],
            added_preferred_generator: false,
            estimated_beta: Vec::new(),
            source_reference: "design".into(),
        }));
        block["generator"] = without_nulls(serde_json::to_value(set).unwrap());
        let annex_p = training("nta8800-annex-p-synthetic.json");
        block["externalSupply"] = serde_json::json!({ "heating": annex_p["heating"].clone() });
        assert!(
            deserializes(&block),
            "multiple + annex P block must deserialize"
        );
        block
    }

    /// Serialized defaults (`None`) as absent members, as the app sends them.
    fn without_nulls(value: Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .filter(|(_, item)| !item.is_null())
                    .map(|(key, item)| (key, without_nulls(item)))
                    .collect(),
            ),
            Value::Array(items) => Value::Array(items.into_iter().map(without_nulls).collect()),
            other => other,
        }
    }

    fn under(path: &str, ancestor: &str) -> bool {
        path.len() > ancestor.len()
            && path.starts_with(ancestor)
            && matches!(path.as_bytes()[ancestor.len()], b'.' | b'[')
    }

    /// Enum tags of the block, from serde alone: a string leaf is a tag when
    /// an unknown value names variants and another of those variants
    /// changes which members its object may or must have. Maps the tag path
    /// to the path of the object it tags.
    fn tags(block: &Value, all_nodes: &[(String, String, bool)]) -> HashMap<String, String> {
        let error = |value: &Value| {
            serde_path_to_error::deserialize::<_, NtaCalculationInput>(value.clone())
                .err()
                .map(|error| error.inner().to_string())
        };
        let mut found = HashMap::new();
        for (path, pointer, scalar) in all_nodes {
            let Some(current) = block.pointer(pointer).and_then(Value::as_str) else {
                continue;
            };
            let Some(dot) = path.rfind('.') else {
                continue;
            };
            if !scalar || path.ends_with(']') {
                continue;
            }
            let mut probe = block.clone();
            *probe.pointer_mut(pointer).unwrap() = Value::from("\u{1}none");
            let Some(message) = error(&probe).filter(|m| m.starts_with("unknown variant")) else {
                continue;
            };
            let is_tag = super::variants(&message)
                .into_iter()
                .filter(|name| name != current)
                .take(8)
                .any(|name| {
                    let mut trial = block.clone();
                    *trial.pointer_mut(pointer).unwrap() = Value::from(name);
                    error(&trial).is_some_and(|m| {
                        m.starts_with("unknown field") || m.starts_with("missing field")
                    })
                });
            if is_tag {
                found.insert(path.clone(), path[..dot].to_string());
            }
        }
        found
    }

    /// The tag-blank rule: next to a blank tag only the tag itself is
    /// expected; the blanks under the object it tags are not classified.
    fn drop_under_blank_tags(
        expected: BTreeSet<String>,
        tags: &HashMap<String, String>,
    ) -> BTreeSet<String> {
        let blank_tags: Vec<(&String, &String)> = expected
            .iter()
            .filter_map(|path| tags.get(path).map(|parent| (path, parent)))
            .collect();
        expected
            .iter()
            .filter(|path| {
                !blank_tags
                    .iter()
                    .any(|(tag, parent)| *path != *tag && under(path, parent))
            })
            .cloned()
            .collect()
    }

    fn counted<F: FnOnce() -> BlankSearch>(search: F) -> (BlankSearch, usize) {
        let before = BLANK_SEARCH_DESERIALIZATIONS.with(|count| count.get());
        let result = search();
        let used = BLANK_SEARCH_DESERIALIZATIONS.with(|count| count.get()) - before;
        (result, used)
    }

    /// `BLANK_FUZZ_TRIALS` and `BLANK_FUZZ_SEED` widen a local run.
    fn check(name: &str, block: Value, seed: u64, trials: usize) {
        let trials = std::env::var("BLANK_FUZZ_TRIALS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(trials);
        let seed = std::env::var("BLANK_FUZZ_SEED")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map_or(seed, |extra| {
                seed ^ extra.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            });
        let mut all_nodes = Vec::new();
        nodes(&block, String::new(), String::new(), &mut all_nodes);
        // Ground truth from serde alone: a node is required when the
        // otherwise valid block fails with only that node blank.
        let required: HashMap<String, bool> = all_nodes
            .iter()
            .map(|(path, pointer, _)| {
                let mut trial = block.clone();
                *trial.pointer_mut(pointer).unwrap() = Value::Null;
                (path.clone(), !deserializes(&trial))
            })
            .collect();
        let tags = tags(&block, &all_nodes);
        let mut rng = Rng(seed);
        let (mut false_positive, mut missed, mut residual) = (Vec::new(), Vec::new(), Vec::new());
        let mut worst = 0usize;
        for trial_index in 0..trials {
            let count = 1 + rng.below(2);
            let mut picked: Vec<&(String, String, bool)> = Vec::new();
            while picked.len() < count {
                let node = &all_nodes[rng.below(all_nodes.len())];
                // A node inside (or around) one already blanked disappears
                // with it.
                if !picked.iter().any(|other| {
                    other.0 == node.0 || under(&node.0, &other.0) || under(&other.0, &node.0)
                }) {
                    picked.push(node);
                }
            }
            let mut trial = block.clone();
            for (_, pointer, _) in &picked {
                *trial.pointer_mut(pointer).unwrap() = Value::Null;
            }
            let (search, used) = counted(|| blank_paths::<NtaCalculationInput>(&trial));
            worst = worst.max(used);
            let found: BTreeSet<String> = search.paths.into_iter().collect();
            let expected: BTreeSet<String> = drop_under_blank_tags(
                picked
                    .iter()
                    .filter(|(path, _, _)| required[path])
                    .map(|(path, _, _)| path.clone())
                    .collect(),
                &tags,
            );
            false_positive.extend(found.difference(&expected).cloned());
            missed.extend(expected.difference(&found).cloned());
            // Only blanks were introduced: once they are filled the block
            // has no other fault.
            if search.residual {
                residual.push((
                    trial_index,
                    picked.iter().map(|node| node.0.clone()).collect::<Vec<_>>(),
                ));
            }
        }
        assert!(
            false_positive.is_empty() && missed.is_empty(),
            "{name}: false positives {false_positive:?}, missed {missed:?}"
        );
        assert!(residual.is_empty(), "{name}: residual after {residual:?}");

        // Every scalar leaf blank at once.
        let mut all = block.clone();
        let scalars: Vec<&(String, String, bool)> =
            all_nodes.iter().filter(|node| node.2).collect();
        for (_, pointer, _) in &scalars {
            *all.pointer_mut(pointer).unwrap() = Value::Null;
        }
        let (search, used) = counted(|| blank_paths::<NtaCalculationInput>(&all));
        let found: BTreeSet<String> = search.paths.into_iter().collect();
        let expected: BTreeSet<String> = drop_under_blank_tags(
            scalars
                .iter()
                .filter(|(path, _, _)| required[path])
                .map(|(path, _, _)| path.clone())
                .collect(),
            &tags,
        );
        let wrong: Vec<_> = found.difference(&expected).collect();
        let missed: Vec<_> = expected.difference(&found).collect();
        assert!(
            wrong.is_empty(),
            "{name}: all blank, false positives {wrong:?}"
        );
        assert!(missed.is_empty(), "{name}: all blank, missed {missed:?}");
        assert!(!search.residual, "{name}: all blank left a residual");
        // Cost bound instead of wall-clock time: phase 1 is guarded at
        // 12·n + 200 rounds of a few deserializations, phase 2 at about
        // 2·k·log2(n) for k blanks among n nulls.
        let n = scalars.len();
        assert!(
            used < 40 * n + 1000,
            "{name}: all blank took {used} deserializations for {n} nulls"
        );
        assert!(
            worst < 2000,
            "{name}: a random trial took {worst} deserializations"
        );
        eprintln!(
            "{name}: {trials} trials over {} nodes ({} scalar), worst {worst} deserializations; all {n} blank: {used}",
            all_nodes.len(),
            n
        );
    }

    #[test]
    fn random_blanks_in_the_terraced_dwelling() {
        check(
            "terraced dwelling",
            block("nta8800-example-terraced-dwelling.json"),
            0x9E37_79B9_7F4A_7C15,
            300,
        );
    }

    #[test]
    fn random_blanks_in_the_office() {
        check(
            "office",
            block("nta8800-example-office.json"),
            0xD1B5_4A32_D192_ED03,
            300,
        );
    }

    #[test]
    fn random_blanks_in_a_multiple_set_with_an_annex_p_network() {
        check(
            "multiple + annex P",
            multiple_and_annex_p_block(),
            0x2545_F491_4F6C_DD1D,
            300,
        );
    }
}

#[cfg(test)]
mod refusal_gaps {
    use super::*;

    fn office() -> Value {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-office.json"
        ))
        .unwrap()
    }

    fn office_in(edition: &str) -> ProjectPerformanceAssessment {
        let mut office = office();
        office["ntaCalculation"]["normVersion"] = Value::from(edition);
        assess_project_performance(&office)
    }

    /// A route the edition lacks refuses the building calculation; the
    /// project names it as a gap at the project input, not only in
    /// `performance.issues`.
    #[test]
    fn refused_routes_are_project_gaps() {
        let result = office_in("2020+A1");
        assert_eq!(result.status, "invalid");
        let found: Vec<(&str, &str, Option<&str>)> = result
            .gaps
            .iter()
            .map(|gap| (gap.code, gap.path.as_str(), gap.detail.as_deref()))
            .collect();
        for expected in [
            (
                "route_not_in_edition",
                "ntaCalculation.lighting[0].lightingZones[0].power.ledFrom2017",
            ),
            (
                "route_not_in_edition",
                "ntaCalculation.pvSystems[0].peakPower.panelPeakPowerW",
            ),
            (
                "lighting_gain_requires_chapter_14",
                "ntaCalculation.internalGains.lighting",
            ),
        ] {
            assert!(
                found
                    .iter()
                    .any(|(code, path, _)| (*code, *path) == expected),
                "{expected:?} in {found:?}"
            );
        }
        // The derived path stays in the detail.
        assert!(found.iter().any(|(_, _, detail)| *detail
            == Some("derivedInput.spaceHeating.demand.internalGains.lighting")));
        // A calculated result gets no gaps from the building issues.
        assert!(office_in("2025+C1").status.starts_with("calculated"));
    }

    #[test]
    fn derived_paths_map_to_their_project_input() {
        let project = office();
        let mut gaps = Vec::new();
        let derived = derive_input(&project, &mut gaps).unwrap();
        let zone = &derived.space_heating.demand.zone_id;
        let zone_index = project["zones"]
            .as_array()
            .unwrap()
            .iter()
            .position(|item| item["id"].as_str() == Some(zone))
            .unwrap();
        for (path, expected) in [
            (
                "spaceHeating.generator.boiler.x",
                "ntaCalculation.generator.boiler.x".to_string(),
            ),
            (
                "spaceHeating.emission.fans",
                "ntaCalculation.emission.fans".to_string(),
            ),
            (
                "spaceHeating.distributionSystem.pump",
                "ntaCalculation.distributionSystem.pump".to_string(),
            ),
            (
                "spaceHeating.demand.thermalMass",
                "ntaCalculation.thermalMass".to_string(),
            ),
            (
                "spaceHeating.demand.transmission.groundFloors[0].heatedBasement",
                "ntaCalculation.groundFloors[0].heatedBasement".to_string(),
            ),
            (
                "spaceHeating.demand.transmission.elements[3]",
                format!("zones[{zone_index}].surfaces"),
            ),
            (
                "spaceHeating.demand.windows[1].movableShading",
                "ntaCalculation.windowSolar.movableShading".to_string(),
            ),
            (
                "hotWater.generator",
                "ntaCalculation.hotWater.generator".to_string(),
            ),
            ("totalUsableFloorAreaM2", "zones".to_string()),
            ("notAProjectMember", "ntaCalculation".to_string()),
        ] {
            assert_eq!(
                project_path_for_derived(path, &derived, &project),
                expected,
                "{path}"
            );
        }
    }
}
