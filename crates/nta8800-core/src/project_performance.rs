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
    /// Annex A: dynamic transparent elements (switchable glazing, movable
    /// shutters) linked to project windows by id, with the method A/B
    /// weighting and optional step-2 correction factors.
    #[serde(default)]
    pub dynamic_windows: Vec<ProjectDynamicWindow>,
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
    /// Label data of Regeling energieprestatie gebouwen art. 4.
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
        loss_area_ratio: (floor > 0.0).then(|| loss / floor),
        unclassified_surface_count: unclassified,
    })
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
fn plausibility_warnings(project_value: &Value) -> Vec<InputGap> {
    use crate::building_performance::{Carrier, Service};
    use crate::monthly_demand::CeilingColumn;
    let mut warnings = Vec::new();
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
    warnings
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
        warnings: plausibility_warnings(project_value),
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
        if nta.window_solar.movable_shading.is_some() {
            gaps.push(gap("window_dynamic_and_shading_exclusive", path.clone()));
        }
        for issue in item.dynamic.validate(&format!("{path}.dynamic")) {
            gaps.push(gap(issue.code, issue.path));
        }
    }
    let mut used_dynamic = HashSet::new();
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
                windows.push(Window {
                    id: format!("window:{window_id}"),
                    area_m2: area,
                    orientation: azimuth_orientation,
                    tilt_deg: tilt,
                    g_perpendicular: g_value,
                    frame_fraction: nta.window_solar.frame_fraction,
                    u_value_w_per_m2k: u_value,
                    forfait_delta_u_w_per_m2k: delta_u_forfait,
                    obstruction: nta.window_solar.obstruction.clone(),
                    movable_shading: nta.window_solar.movable_shading.clone(),
                    dynamic,
                    glazing: None,
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
