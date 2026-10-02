//! NTA 8800 kernel boundary for Open Energy Studio.
//!
//! The current implementation validates the legacy `.oes` project envelope.
//! It deliberately does not return BENG values until the norm calculation has
//! independent reference cases. Callers must not use the legacy TypeScript
//! monthly model as if it were this kernel.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

pub mod bacs_draft;
pub mod bbl_requirements;
pub mod boiler_forfait_draft;
pub mod building_performance;
pub mod climate;
pub mod declared_dhw;
pub mod declared_heating_table;
pub mod diagnostic_reference;
pub mod direct_transmission;
pub mod domestic_hot_water;
pub mod epus_draft;
pub mod final_energy_draft;
pub mod forfait_heat_pump_draft;
pub mod forfait_heat_pump_monthly_draft;
pub mod gas_collective_source_draft;
pub mod gas_heat_pump_aux_draft;
pub mod gas_heat_pump_chain_draft;
pub mod gas_heat_pump_chain_reference;
pub mod gas_heat_pump_forfait_draft;
pub mod gas_heat_pump_monthly_draft;
pub mod generator_dispatch_draft;
pub mod ground;
pub mod heat_pumps;
pub mod heating_aux_draft;
pub mod heating_distribution;
pub mod heating_emission;
pub mod hybrid_heat_pump_monthly_draft;
pub mod indicators_draft;
pub mod label_class;
pub mod monthly_demand;
pub mod monthly_direct_transmission;
pub mod project_performance;
pub mod pv;
pub mod reference;
pub mod solar_shading;
pub mod space_cooling;
pub mod space_heating_chain;
pub mod tojuli;
pub mod unheated_transmission;
use direct_transmission::{DirectElement, DirectTransmissionInput, LinearBridge, PointBridge};
use heat_pumps::{
    AuxiliaryMeasurementBoundary, HeatPumpInput, HeatSink, HeatSource, InputEnergyCarrier,
    PerformanceEvidenceKind, PerformanceService, SystemLinkRole, SystemLinkTargetKind,
};
use unheated_transmission::{UnheatedSpaceInput, UnheatedTransmissionInput};

pub const TARGET_NORM_VERSION: &str = "NTA 8800:2025+C1:2026";
pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub id: String,
    pub name: String,
    pub building_function: BuildingFunction,
    pub zones: Vec<ZoneInput>,
    #[serde(default)]
    pub heating_systems: Vec<Value>,
    #[serde(default)]
    pub ventilation_systems: Vec<Value>,
    #[serde(default)]
    pub cooling_systems: Vec<Value>,
    #[serde(default)]
    pub hot_water_systems: Vec<Value>,
    #[serde(default)]
    pub nta_heat_pumps: Vec<HeatPumpInput>,
    #[serde(default)]
    pub constructions: Vec<Value>,
    #[serde(default)]
    pub unheated_spaces: Vec<ProjectUnheatedSpace>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUnheatedSpace {
    pub id: String,
    pub name: String,
    pub reduction_factor: f64,
    pub factor_source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingFunction {
    Residential,
    Office,
    Education,
    Healthcare,
    Retail,
    Industrial,
    Other,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneInput {
    pub id: String,
    pub floor_area: f64,
    pub volume: f64,
    #[serde(default)]
    pub surfaces: Vec<Value>,
    #[serde(default)]
    pub thermal_bridges: Option<Vec<Value>>,
    #[serde(default)]
    pub point_thermal_bridges: Option<Vec<Value>>,
    #[serde(default)]
    pub point_bridge_inventory_complete: Option<bool>,
    #[serde(default)]
    pub air_tightness: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalBoundary {
    Outdoor,
    Ground,
    UnheatedSpace,
    AdjacentConditioned,
    Internal,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: &'static str,
    pub path: String,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub zone_count: usize,
    pub surface_count: usize,
    pub system_count: usize,
    pub heat_pump_count: usize,
    pub classified_heat_pump_count: usize,
    pub auxiliary_component_count: usize,
    pub system_link_count: usize,
    /// Ratios of individual declared operating points, not annual NTA efficiencies.
    pub performance_point_diagnostics: Vec<PerformancePointDiagnostic>,
    /// Declared tap-profile energy ratios, not NTA annual DHW efficiencies.
    pub dhw_test_diagnostics: Vec<DhwTestDiagnostic>,
    pub floor_area_m2: f64,
    /// Simple nested-surface bookkeeping, not NTA-defined Ag/Als or transmission area.
    pub envelope_geometry: Option<EnvelopeGeometrySummary>,
    pub thermal_boundaries: ThermalBoundarySummary,
    pub direct_outdoor_diagnostic: Option<direct_transmission::DirectTransmissionAssessment>,
    pub unheated_transmission_diagnostic:
        Option<unheated_transmission::UnheatedTransmissionAssessment>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformancePointDiagnostic {
    pub heat_pump_path: String,
    pub heat_pump_id: String,
    pub point_id: String,
    pub service: PerformanceService,
    pub input_energy_carrier: InputEnergyCarrier,
    pub instantaneous_useful_to_input_ratio: f64,
    pub reference_verified: bool,
    pub annual_performance_available: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DhwTestDiagnostic {
    pub heat_pump_id: String,
    pub point_id: String,
    pub tap_profile: String,
    pub declaration_norm_version: String,
    pub declared_useful_to_input_ratio: f64,
    pub reference_verified: bool,
    pub annual_performance_available: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThermalBoundarySummary {
    pub surface_count: usize,
    pub classified_surface_count: usize,
    pub bridge_count: usize,
    pub classified_bridge_count: usize,
    pub point_bridge_count: usize,
    pub classified_point_bridge_count: usize,
    pub point_inventory_complete: bool,
    pub complete: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopeGeometrySummary {
    pub gross_surface_area_m2: f64,
    pub window_area_m2: f64,
    pub remaining_opaque_area_m2: f64,
    pub zones: Vec<ZoneGeometrySummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneGeometrySummary {
    pub zone_id: String,
    pub gross_surface_area_m2: f64,
    pub window_area_m2: f64,
    pub remaining_opaque_area_m2: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputAssessment {
    pub status: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub calculation_available: bool,
    pub summary: ProjectSummary,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelCapabilities {
    pub kernel_language: &'static str,
    pub kernel_version: &'static str,
    pub target_norm_version: &'static str,
    pub calculation_available: bool,
    pub attest_status: &'static str,
    pub implemented: &'static [&'static str],
}

pub fn capabilities() -> KernelCapabilities {
    KernelCapabilities {
        kernel_language: "rust",
        kernel_version: KERNEL_VERSION,
        target_norm_version: TARGET_NORM_VERSION,
        calculation_available: false,
        attest_status: "unattested",
        implemented: &[
            "legacy_project_structural_validation",
            "envelope_geometry_input_audit",
            "diagnostic_direct_outdoor_conductance",
            "diagnostic_monthly_direct_outdoor_heat_flow",
            "diagnostic_unheated_space_conductance_with_supplied_factors",
            "project_point_bridge_input_audit",
            "heat_pump_input_taxonomy",
            "heat_pump_input_audit",
            "standalone_heat_pump_input_audit",
            "heat_pump_performance_point_audit",
            "diagnostic_heat_pump_operating_point_ratio",
            "diagnostic_heat_pump_forfait_cop_consultation_tables",
            "diagnostic_gas_engine_and_absorption_heat_pump_cop_consultation_tables_9_27_9_29",
            "diagnostic_gas_heat_pump_generator_auxiliary_consultation_equations_9_91_9_92",
            "diagnostic_gas_heat_pump_monthly_input_terms_consultation_equation_9_62",
            "diagnostic_gas_heat_pump_linked_monthly_input_and_equipment_auxiliary_terms",
            "diagnostic_heat_pump_forfait_monthly_input_consultation_equation_9_62",
            "diagnostic_new_build_heating_generator_dispatch_consultation_tables_9_1_9_23",
            "diagnostic_new_build_hybrid_heat_pump_monthly_chain_consultation_chapter_9",
            "diagnostic_gas_boiler_forfait_efficiency_and_monthly_input_consultation_table_9_25",
            "heat_pump_auxiliary_component_audit",
            "heat_pump_system_link_audit",
            "reference_manifest_field_audit",
            "diagnostic_direct_transmission_reference_comparison",
            "unverified_chapter_7_monthly_heating_cooling_need_single_zone",
            "unverified_space_heating_chain_emission_distribution_single_generator",
            "unverified_single_zone_primary_energy_and_indicators_chapter_5_draft",
            "unverified_space_heating_distribution_9_26_to_9_51_and_auxiliary_9_85_9_91",
            "unverified_storage_correction_5_14a_and_co2_emission_5_5_6_1",
            "unverified_pv_yield_chapter_16",
            "unverified_space_cooling_generation_10_5_with_declared_emission",
            "indicative_label_class_omgevingsregeling_annex_ix_x",
            "bbl_4_149_beng_requirement_check_single_function",
            "unverified_tojuli_per_orientation_5_7",
            "unverified_domestic_hot_water_need_chapter_13_with_declared_efficiencies",
            "unverified_project_performance_adapter_single_zone",
        ],
    }
}

pub fn assess_json(value: Value) -> Result<InputAssessment, String> {
    let input_fingerprint = input_fingerprint(&value);
    let project: ProjectInput = serde_json::from_value(value).map_err(|error| error.to_string())?;
    Ok(assess_project(&project, input_fingerprint))
}

pub(crate) fn input_fingerprint(value: &Value) -> String {
    fn canonical(value: &Value, output: &mut String) {
        match value {
            Value::Array(items) => {
                output.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    canonical(item, output);
                }
                output.push(']');
            }
            Value::Object(fields) => {
                output.push('{');
                let mut keys: Vec<_> = fields.keys().collect();
                keys.sort();
                for (index, key) in keys.into_iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    output.push_str(&serde_json::to_string(key).expect("JSON key serialization"));
                    output.push(':');
                    canonical(&fields[key], output);
                }
                output.push('}');
            }
            _ => output.push_str(&serde_json::to_string(value).expect("JSON value serialization")),
        }
    }
    let mut canonical_input = String::new();
    canonical(value, &mut canonical_input);
    format!("sha256:{:x}", Sha256::digest(canonical_input.as_bytes()))
}

fn assess_project(project: &ProjectInput, input_fingerprint: String) -> InputAssessment {
    let mut issues = Vec::new();
    if project.id.trim().is_empty() {
        issues.push(issue(
            Severity::Error,
            "project_id_required",
            "id",
            "Project ID is required",
        ));
    }
    if project.name.trim().is_empty() {
        issues.push(issue(
            Severity::Error,
            "project_name_required",
            "name",
            "Project name is required",
        ));
    }
    if project.zones.is_empty() {
        issues.push(issue(
            Severity::Error,
            "zone_required",
            "zones",
            "At least one calculation zone is required",
        ));
    }
    audit_system_records(&project.heating_systems, "heatingSystems", &mut issues);
    audit_system_records(
        &project.ventilation_systems,
        "ventilationSystems",
        &mut issues,
    );
    audit_system_records(&project.cooling_systems, "coolingSystems", &mut issues);
    audit_system_records(&project.hot_water_systems, "hotWaterSystems", &mut issues);

    let mut zone_ids = HashSet::new();
    let mut surface_ids = HashSet::new();
    let mut window_ids = HashSet::new();
    let mut thermal_bridge_ids = HashSet::new();
    let mut construction_ids = HashSet::new();
    for (index, construction) in project.constructions.iter().enumerate() {
        let path = format!("constructions[{index}]");
        let id = construction.get("id").and_then(Value::as_str).unwrap_or("");
        if id.trim().is_empty() || !construction_ids.insert(id) {
            issues.push(issue(
                Severity::Error,
                "construction_id_invalid",
                format!("{path}.id"),
                "Construction IDs must be nonempty and unique",
            ));
        }
        for (field, require_positive) in [("rcValue", false), ("uValue", true)] {
            match construction.get(field) {
                Some(raw)
                    if raw.as_f64().is_some_and(|value| {
                        value.is_finite() && value >= 0.0 && (!require_positive || value > 0.0)
                    }) => {}
                Some(_) => issues.push(issue(
                    Severity::Error,
                    "construction_thermal_value_invalid",
                    format!("{path}.{field}"),
                    "Construction Rc must be nonnegative and U must be positive",
                )),
                None => issues.push(issue(
                    Severity::Warning,
                    "construction_thermal_value_missing",
                    format!("{path}.{field}"),
                    "Construction thermal value is missing",
                )),
            }
        }
        if let Some(layers) = construction.get("layers").and_then(Value::as_array) {
            for (layer_index, layer) in layers.iter().enumerate() {
                for field in ["thickness", "lambda"] {
                    if layer
                        .get(field)
                        .and_then(Value::as_f64)
                        .map_or(true, |value| !value.is_finite() || value <= 0.0)
                    {
                        issues.push(issue(
                            Severity::Error,
                            "construction_layer_invalid",
                            format!("{path}.layers[{layer_index}].{field}"),
                            "Construction layer thickness and conductivity must be positive",
                        ));
                    }
                }
            }
        } else if construction.get("layers").is_some() {
            issues.push(issue(
                Severity::Error,
                "construction_layers_invalid",
                format!("{path}.layers"),
                "Construction layers must be an array",
            ));
        }
    }
    let mut floor_area_m2 = 0.0;
    let mut surface_count = 0;
    let mut classified_surface_boundary_count = 0;
    let mut thermal_bridge_count = 0;
    let mut classified_bridge_boundary_count = 0;
    let mut point_bridge_count = 0;
    let mut classified_point_boundary_count = 0;
    let mut boundary_lists_present = true;
    let mut point_lists_present = true;
    let mut zone_geometry = Vec::new();
    let mut geometry_complete = !project.zones.is_empty();
    for (index, zone) in project.zones.iter().enumerate() {
        let mut zone_geometry_complete = true;
        let mut zone_gross_area_m2 = 0.0;
        let mut zone_window_area_m2 = 0.0;
        if zone.id.trim().is_empty() || !zone_ids.insert(zone.id.as_str()) {
            zone_geometry_complete = false;
            issues.push(issue(
                Severity::Error,
                "zone_id_invalid",
                format!("zones[{index}].id"),
                "Zone IDs must be nonempty and unique",
            ));
        }
        if !zone.floor_area.is_finite() || zone.floor_area <= 0.0 {
            issues.push(issue(
                Severity::Error,
                "floor_area_invalid",
                format!("zones[{index}].floorArea"),
                "Zone floor area must be finite and greater than zero",
            ));
        } else {
            floor_area_m2 += zone.floor_area;
            if !floor_area_m2.is_finite() {
                issues.push(issue(
                    Severity::Error,
                    "floor_area_sum_invalid",
                    "zones",
                    "Summed zone floor area is not finite",
                ));
                floor_area_m2 = 0.0;
            }
        }
        if !zone.volume.is_finite() || zone.volume <= 0.0 {
            issues.push(issue(
                Severity::Error,
                "volume_invalid",
                format!("zones[{index}].volume"),
                "Zone volume must be finite and greater than zero",
            ));
        }
        if let Some(airtightness) = zone.air_tightness.as_ref() {
            if airtightness
                .get("qv10")
                .and_then(Value::as_f64)
                .map_or(true, |value| !value.is_finite() || value < 0.0)
            {
                issues.push(issue(
                    Severity::Error,
                    "airtightness_invalid",
                    format!("zones[{index}].airTightness.qv10"),
                    "Air tightness qv10 must be finite and nonnegative",
                ));
            }
        }
        boundary_lists_present &= zone.thermal_bridges.is_some();
        point_lists_present &= zone.point_thermal_bridges.is_some()
            && zone.point_bridge_inventory_complete == Some(true);
        let bridges = zone.thermal_bridges.as_deref().unwrap_or(&[]);
        thermal_bridge_count += bridges.len();
        for (bridge_index, bridge) in bridges.iter().enumerate() {
            let path = format!("zones[{index}].thermalBridges[{bridge_index}]");
            if audit_thermal_boundary(
                bridge.get("thermalBoundary"),
                format!("{path}.thermalBoundary"),
                &mut issues,
            )
            .is_some()
            {
                classified_bridge_boundary_count += 1;
            }
            let id = bridge.get("id").and_then(Value::as_str).unwrap_or("");
            if id.trim().is_empty() || !thermal_bridge_ids.insert(id) {
                issues.push(issue(
                    Severity::Error,
                    "thermal_bridge_id_invalid",
                    format!("{path}.id"),
                    "Thermal bridge IDs must be nonempty and unique",
                ));
            }
            if bridge
                .get("length")
                .and_then(Value::as_f64)
                .map_or(true, |value| !value.is_finite() || value <= 0.0)
            {
                issues.push(issue(
                    Severity::Error,
                    "thermal_bridge_length_invalid",
                    format!("{path}.length"),
                    "Thermal bridge length must be finite and positive",
                ));
            }
            if bridge
                .get("psiValue")
                .and_then(Value::as_f64)
                .map_or(true, |value| !value.is_finite())
            {
                issues.push(issue(
                    Severity::Error,
                    "thermal_bridge_psi_invalid",
                    format!("{path}.psiValue"),
                    "Thermal bridge linear transmittance must be finite",
                ));
            }
            if bridge
                .get("zoneId")
                .and_then(Value::as_str)
                .is_some_and(|id| id != zone.id)
            {
                issues.push(issue(
                    Severity::Error,
                    "thermal_bridge_zone_mismatch",
                    format!("{path}.zoneId"),
                    "Thermal bridge zone ID does not match its containing zone",
                ));
            }
        }
        let point_bridges = zone.point_thermal_bridges.as_deref().unwrap_or(&[]);
        point_bridge_count += point_bridges.len();
        for (bridge_index, bridge) in point_bridges.iter().enumerate() {
            let path = format!("zones[{index}].pointThermalBridges[{bridge_index}]");
            if audit_thermal_boundary(
                bridge.get("thermalBoundary"),
                format!("{path}.thermalBoundary"),
                &mut issues,
            )
            .is_some()
            {
                classified_point_boundary_count += 1;
            }
            let id = bridge.get("id").and_then(Value::as_str).unwrap_or("");
            if id.trim().is_empty() || !thermal_bridge_ids.insert(id) {
                issues.push(issue(
                    Severity::Error,
                    "point_bridge_id_invalid",
                    format!("{path}.id"),
                    "Point bridge IDs must be nonempty and unique across all bridges",
                ));
            }
            if bridge
                .get("chiValue")
                .and_then(Value::as_f64)
                .map_or(true, |value| !value.is_finite())
            {
                issues.push(issue(
                    Severity::Error,
                    "point_bridge_chi_invalid",
                    format!("{path}.chiValue"),
                    "Point bridge transmittance must be finite",
                ));
            }
            if bridge
                .get("sourceReference")
                .and_then(Value::as_str)
                .map_or(true, |value| value.trim().is_empty())
            {
                issues.push(issue(
                    Severity::Error,
                    "point_bridge_source_required",
                    format!("{path}.sourceReference"),
                    "Point bridge requires a traceable source reference",
                ));
            }
            if bridge
                .get("zoneId")
                .and_then(Value::as_str)
                .is_some_and(|id| id != zone.id)
            {
                issues.push(issue(
                    Severity::Error,
                    "point_bridge_zone_mismatch",
                    format!("{path}.zoneId"),
                    "Point bridge zone ID does not match its containing zone",
                ));
            }
        }
        if zone.surfaces.is_empty() {
            zone_geometry_complete = false;
            issues.push(issue(
                Severity::Warning,
                "surfaces_missing",
                format!("zones[{index}].surfaces"),
                "No envelope surfaces have been entered for this zone",
            ));
        }
        surface_count += zone.surfaces.len();
        for (surface_index, surface) in zone.surfaces.iter().enumerate() {
            let path = format!("zones[{index}].surfaces[{surface_index}]");
            if audit_thermal_boundary(
                surface.get("thermalBoundary"),
                format!("{path}.thermalBoundary"),
                &mut issues,
            )
            .is_some()
            {
                classified_surface_boundary_count += 1;
            }
            let surface_id = surface.get("id").and_then(Value::as_str).unwrap_or("");
            if surface_id.trim().is_empty() || !surface_ids.insert(surface_id) {
                zone_geometry_complete = false;
                issues.push(issue(
                    Severity::Error,
                    "surface_id_invalid",
                    format!("{path}.id"),
                    "Surface IDs must be nonempty and unique",
                ));
            }
            let area = surface.get("area").and_then(Value::as_f64);
            if area.map_or(true, |value| !value.is_finite() || value <= 0.0) {
                zone_geometry_complete = false;
                issues.push(issue(
                    Severity::Error,
                    "surface_area_invalid",
                    format!("{path}.area"),
                    "Surface area must be finite and greater than zero",
                ));
            } else if let Some(value) = area {
                zone_gross_area_m2 += value;
                if !zone_gross_area_m2.is_finite() {
                    zone_geometry_complete = false;
                    issues.push(issue(
                        Severity::Error,
                        "surface_area_sum_invalid",
                        format!("zones[{index}].surfaces"),
                        "Summed surface area is not finite",
                    ));
                }
            }
            if surface
                .get("zoneId")
                .and_then(Value::as_str)
                .is_some_and(|id| id != zone.id)
            {
                zone_geometry_complete = false;
                issues.push(issue(
                    Severity::Error,
                    "surface_zone_mismatch",
                    format!("{path}.zoneId"),
                    "Surface zone ID does not match its containing zone",
                ));
            }
            if !construction_ids.is_empty()
                && surface
                    .get("constructionId")
                    .and_then(Value::as_str)
                    .map_or(true, |id| !construction_ids.contains(id))
            {
                issues.push(issue(
                    Severity::Error,
                    "surface_construction_missing",
                    format!("{path}.constructionId"),
                    "Surface construction ID does not resolve to a construction",
                ));
            }
            if let Some(windows) = surface.get("windows") {
                if let Some(windows) = windows.as_array() {
                    let mut window_area = 0.0;
                    for (window_index, window) in windows.iter().enumerate() {
                        let window_path = format!("{path}.windows[{window_index}]");
                        let window_id = window.get("id").and_then(Value::as_str).unwrap_or("");
                        if window_id.trim().is_empty() || !window_ids.insert(window_id) {
                            zone_geometry_complete = false;
                            issues.push(issue(
                                Severity::Error,
                                "window_id_invalid",
                                format!("{window_path}.id"),
                                "Window IDs must be nonempty and unique",
                            ));
                        }
                        match window.get("area").and_then(Value::as_f64) {
                            Some(value) if value.is_finite() && value > 0.0 => {
                                window_area += value;
                                if !window_area.is_finite() {
                                    zone_geometry_complete = false;
                                    issues.push(issue(
                                        Severity::Error,
                                        "window_area_sum_invalid",
                                        format!("{path}.windows"),
                                        "Summed window area is not finite",
                                    ));
                                }
                            }
                            _ => {
                                zone_geometry_complete = false;
                                issues.push(issue(
                                    Severity::Error,
                                    "window_area_invalid",
                                    format!("{window_path}.area"),
                                    "Window area must be finite and greater than zero",
                                ));
                            }
                        }
                        match window.get("uValue") {
                            Some(raw)
                                if raw
                                    .as_f64()
                                    .is_some_and(|value| value.is_finite() && value > 0.0) => {}
                            Some(_) => issues.push(issue(
                                Severity::Error,
                                "window_u_invalid",
                                format!("{window_path}.uValue"),
                                "Window U-value must be finite and positive",
                            )),
                            None => issues.push(issue(
                                Severity::Warning,
                                "window_u_missing",
                                format!("{window_path}.uValue"),
                                "Window U-value is missing",
                            )),
                        }
                        match window.get("gValue") {
                            Some(raw)
                                if raw.as_f64().is_some_and(|value| {
                                    value.is_finite() && (0.0..=1.0).contains(&value)
                                }) => {}
                            Some(_) => issues.push(issue(
                                Severity::Error,
                                "window_g_invalid",
                                format!("{window_path}.gValue"),
                                "Window g-value must be finite and between zero and one",
                            )),
                            None => issues.push(issue(
                                Severity::Warning,
                                "window_g_missing",
                                format!("{window_path}.gValue"),
                                "Window g-value is missing",
                            )),
                        }
                        if window
                            .get("surfaceId")
                            .and_then(Value::as_str)
                            .is_some_and(|id| id != surface_id)
                        {
                            zone_geometry_complete = false;
                            issues.push(issue(
                                Severity::Error,
                                "window_surface_mismatch",
                                format!("{window_path}.surfaceId"),
                                "Window surface ID does not match its containing surface",
                            ));
                        }
                    }
                    zone_window_area_m2 += window_area;
                    if !zone_window_area_m2.is_finite() {
                        zone_geometry_complete = false;
                        issues.push(issue(
                            Severity::Error,
                            "zone_window_area_sum_invalid",
                            format!("zones[{index}].surfaces"),
                            "Summed zone window area is not finite",
                        ));
                    }
                    if area.is_some_and(|value| window_area > value + 1e-9) {
                        zone_geometry_complete = false;
                        issues.push(issue(
                            Severity::Error,
                            "window_area_exceeds_surface",
                            format!("{path}.windows"),
                            "Sum of window areas exceeds gross surface area",
                        ));
                    }
                } else {
                    zone_geometry_complete = false;
                    issues.push(issue(
                        Severity::Error,
                        "windows_invalid",
                        format!("{path}.windows"),
                        "Windows must be an array",
                    ));
                }
            } else {
                zone_geometry_complete = false;
                issues.push(issue(
                    Severity::Warning,
                    "windows_missing",
                    format!("{path}.windows"),
                    "Window list is absent; envelope area bookkeeping is unavailable",
                ));
            }
        }
        if zone_geometry_complete
            && zone_gross_area_m2.is_finite()
            && zone_window_area_m2.is_finite()
            && zone_gross_area_m2 + 1e-9 >= zone_window_area_m2
        {
            zone_geometry.push(ZoneGeometrySummary {
                zone_id: zone.id.clone(),
                gross_surface_area_m2: zone_gross_area_m2,
                window_area_m2: zone_window_area_m2,
                remaining_opaque_area_m2: (zone_gross_area_m2 - zone_window_area_m2).max(0.0),
            });
        } else {
            geometry_complete = false;
        }
    }

    let envelope_geometry = if geometry_complete {
        let gross_surface_area_m2: f64 = zone_geometry
            .iter()
            .map(|zone| zone.gross_surface_area_m2)
            .sum();
        let window_area_m2: f64 = zone_geometry.iter().map(|zone| zone.window_area_m2).sum();
        let remaining_opaque_area_m2: f64 = zone_geometry
            .iter()
            .map(|zone| zone.remaining_opaque_area_m2)
            .sum();
        if gross_surface_area_m2.is_finite()
            && window_area_m2.is_finite()
            && remaining_opaque_area_m2.is_finite()
        {
            Some(EnvelopeGeometrySummary {
                gross_surface_area_m2,
                window_area_m2,
                remaining_opaque_area_m2,
                zones: zone_geometry,
            })
        } else {
            issues.push(issue(
                Severity::Error,
                "project_envelope_area_sum_invalid",
                "zones",
                "Summed project envelope area is not finite",
            ));
            None
        }
    } else {
        None
    };

    let system_count = project.heating_systems.len()
        + project.ventilation_systems.len()
        + project.cooling_systems.len()
        + project.hot_water_systems.len()
        + project.nta_heat_pumps.len();
    let mut heat_pump_count = 0;
    let mut classified_heat_pump_count = 0;
    let mut auxiliary_component_count = 0;
    let mut system_link_count = 0;
    let mut classified_heat_pump_ids = HashSet::new();
    let mut classified_pumps: Vec<(String, HeatPumpInput)> = Vec::new();
    for (index, system) in project.heating_systems.iter().enumerate() {
        let system_type = system.get("type").and_then(Value::as_str);
        if !matches!(system_type, Some("heat_pump_air" | "heat_pump_ground")) {
            if system.get("ntaHeatPump").is_some() {
                issues.push(issue(
                    Severity::Error,
                    "heat_pump_metadata_on_other_system",
                    format!("heatingSystems[{index}].ntaHeatPump"),
                    "Heat pump metadata is attached to a non-heat-pump heating system",
                ));
            }
            continue;
        }
        heat_pump_count += 1;
        let path = format!("heatingSystems[{index}]");
        if system
            .get("cop")
            .and_then(Value::as_f64)
            .map_or(true, |value| !value.is_finite() || value <= 0.0)
        {
            issues.push(issue(
                Severity::Error,
                "heat_pump_cop_invalid",
                format!("{path}.cop"),
                "Legacy heat pump COP must be finite and greater than zero",
            ));
        }
        if system
            .get("coverageFraction")
            .and_then(Value::as_f64)
            .map_or(true, |value| {
                !value.is_finite() || !(0.0..=1.0).contains(&value)
            })
        {
            issues.push(issue(
                Severity::Error,
                "heat_pump_coverage_invalid",
                format!("{path}.coverageFraction"),
                "Legacy heat pump coverage fraction must be between zero and one",
            ));
        }
        let Some(metadata) = system.get("ntaHeatPump") else {
            issues.push(issue(
                Severity::Warning,
                "heat_pump_route_unclassified",
                path,
                "Legacy heat pump has only a scalar COP; source, sink, drive and evidence are required for a future NTA route",
            ));
            continue;
        };
        match serde_json::from_value::<HeatPumpInput>(metadata.clone()) {
            Ok(pump) => {
                classified_heat_pump_count += 1;
                auxiliary_component_count += pump.auxiliary_components.len();
                system_link_count += pump.system_links.len();
                classified_pumps.push((format!("{path}.ntaHeatPump"), pump.clone()));
                if !classified_heat_pump_ids.insert(pump.id.clone()) {
                    issues.push(issue(
                        Severity::Error,
                        "heat_pump_id_duplicate",
                        format!("{path}.ntaHeatPump.id"),
                        "Heat pump IDs must be unique across the project",
                    ));
                }
                for pump_issue in pump.validate_inputs() {
                    issues.push(issue(
                        Severity::Error,
                        pump_issue.code,
                        format!("{path}.ntaHeatPump.{}", pump_issue.field),
                        pump_issue.message,
                    ));
                }
                if !pump.performance_points.is_empty() {
                    issues.push(issue(
                        Severity::Warning,
                        "heat_pump_performance_points_unimplemented",
                        format!("{path}.ntaHeatPump.performancePoints"),
                        "Declared performance points are stored but have no verified NTA calculation route",
                    ));
                }
                if !pump.dhw_test_points.is_empty() {
                    issues.push(issue(Severity::Warning, "heat_pump_dhw_test_route_unimplemented",
                        format!("{path}.ntaHeatPump.dhwTestPoints"),
                        "Declared DHW tap-profile inputs are stored but no verified NTA route is available"));
                }
                if !pump.auxiliary_components.is_empty() {
                    issues.push(issue(
                        Severity::Warning,
                        "heat_pump_auxiliaries_unimplemented",
                        format!("{path}.ntaHeatPump.auxiliaryComponents"),
                        "Declared auxiliary components have no verified NTA calculation route",
                    ));
                }
                if !pump.system_links.is_empty() {
                    issues.push(issue(
                        Severity::Warning,
                        "heat_pump_system_links_unimplemented",
                        format!("{path}.ntaHeatPump.systemLinks"),
                        "Equipment links are stored but have no verified dispatch or energy calculation route",
                    ));
                }
                audit_served_zones(
                    &pump,
                    &format!("{path}.ntaHeatPump"),
                    &zone_ids,
                    false,
                    &mut issues,
                );
                if system.get("id").and_then(Value::as_str) != Some(pump.id.as_str()) {
                    issues.push(issue(
                        Severity::Error,
                        "heat_pump_id_mismatch",
                        format!("{path}.ntaHeatPump.id"),
                        "Heat pump metadata ID must match its heating system ID",
                    ));
                }
                let source_matches_legacy_type = match system_type {
                    Some("heat_pump_air") => {
                        matches!(pump.source, HeatSource::OutdoorAir | HeatSource::ExhaustAir)
                    }
                    Some("heat_pump_ground") => {
                        matches!(pump.source, HeatSource::Ground | HeatSource::Groundwater)
                    }
                    _ => true,
                };
                if !source_matches_legacy_type {
                    issues.push(issue(
                        Severity::Error,
                        "heat_pump_source_mismatch",
                        format!("{path}.ntaHeatPump.source"),
                        "Heat pump source conflicts with the legacy heating system type",
                    ));
                }
                if matches!(pump.sink, HeatSink::DomesticHotWater) {
                    issues.push(issue(
                        Severity::Error,
                        "heating_heat_pump_sink_mismatch",
                        format!("{path}.ntaHeatPump.sink"),
                        "A heating heat pump must include space heating as a sink",
                    ));
                }
            }
            Err(error) => issues.push(issue_with_detail(
                Severity::Error,
                "heat_pump_metadata_invalid",
                format!("{path}.ntaHeatPump"),
                "Heat pump metadata is incomplete or has an unknown value",
                error.to_string(),
            )),
        }
    }
    for (index, system) in project.hot_water_systems.iter().enumerate() {
        if system.get("type").and_then(Value::as_str) != Some("heat_pump")
            && system.get("ntaHeatPump").is_some()
        {
            issues.push(issue(
                Severity::Error,
                "heat_pump_metadata_on_other_system",
                format!("hotWaterSystems[{index}].ntaHeatPump"),
                "Heat pump metadata is attached to a non-heat-pump hot water system",
            ));
        }
        if system.get("type").and_then(Value::as_str) == Some("heat_pump") {
            heat_pump_count += 1;
            let path = format!("hotWaterSystems[{index}]");
            let Some(metadata) = system.get("ntaHeatPump") else {
                issues.push(issue(
                    Severity::Warning,
                    "hot_water_heat_pump_route_unclassified",
                    path,
                    "Legacy hot water heat pump lacks a classified NTA route",
                ));
                continue;
            };
            match serde_json::from_value::<HeatPumpInput>(metadata.clone()) {
                Ok(pump) => {
                    classified_heat_pump_count += 1;
                    auxiliary_component_count += pump.auxiliary_components.len();
                    system_link_count += pump.system_links.len();
                    classified_pumps.push((format!("{path}.ntaHeatPump"), pump.clone()));
                    if !classified_heat_pump_ids.insert(pump.id.clone()) {
                        issues.push(issue(
                            Severity::Error,
                            "heat_pump_id_duplicate",
                            format!("{path}.ntaHeatPump.id"),
                            "Heat pump IDs must be unique across the project",
                        ));
                    }
                    for pump_issue in pump.validate_inputs() {
                        issues.push(issue(
                            Severity::Error,
                            pump_issue.code,
                            format!("{path}.ntaHeatPump.{}", pump_issue.field),
                            pump_issue.message,
                        ));
                    }
                    if !pump.performance_points.is_empty() {
                        issues.push(issue(
                            Severity::Warning,
                            "heat_pump_performance_points_unimplemented",
                            format!("{path}.ntaHeatPump.performancePoints"),
                            "Declared performance points are stored but have no verified NTA calculation route",
                        ));
                    }
                    if !pump.dhw_test_points.is_empty() {
                        issues.push(issue(Severity::Warning, "heat_pump_dhw_test_route_unimplemented",
                            format!("{path}.ntaHeatPump.dhwTestPoints"),
                            "Declared DHW tap-profile inputs are stored but no verified NTA route is available"));
                    }
                    if !pump.auxiliary_components.is_empty() {
                        issues.push(issue(
                            Severity::Warning,
                            "heat_pump_auxiliaries_unimplemented",
                            format!("{path}.ntaHeatPump.auxiliaryComponents"),
                            "Declared auxiliary components have no verified NTA calculation route",
                        ));
                    }
                    if !pump.system_links.is_empty() {
                        issues.push(issue(
                            Severity::Warning,
                            "heat_pump_system_links_unimplemented",
                            format!("{path}.ntaHeatPump.systemLinks"),
                            "Equipment links are stored but have no verified dispatch or energy calculation route",
                        ));
                    }
                    audit_served_zones(
                        &pump,
                        &format!("{path}.ntaHeatPump"),
                        &zone_ids,
                        false,
                        &mut issues,
                    );
                    if system.get("id").and_then(Value::as_str) != Some(pump.id.as_str()) {
                        issues.push(issue(
                            Severity::Error,
                            "heat_pump_id_mismatch",
                            format!("{path}.ntaHeatPump.id"),
                            "Heat pump metadata ID must match its hot water system ID",
                        ));
                    }
                    if !matches!(
                        pump.sink,
                        HeatSink::DomesticHotWater | HeatSink::CombinedHydronicAndHotWater
                    ) {
                        issues.push(issue(
                            Severity::Error,
                            "hot_water_heat_pump_sink_mismatch",
                            format!("{path}.ntaHeatPump.sink"),
                            "Hot water heat pump must include domestic hot water as a sink",
                        ));
                    }
                }
                Err(error) => issues.push(issue_with_detail(
                    Severity::Error,
                    "heat_pump_metadata_invalid",
                    format!("{path}.ntaHeatPump"),
                    "Heat pump metadata is incomplete or has an unknown value",
                    error.to_string(),
                )),
            }
        }
    }
    for (index, pump) in project.nta_heat_pumps.iter().enumerate() {
        heat_pump_count += 1;
        classified_heat_pump_count += 1;
        auxiliary_component_count += pump.auxiliary_components.len();
        system_link_count += pump.system_links.len();
        let path = format!("ntaHeatPumps[{index}]");
        classified_pumps.push((path.clone(), pump.clone()));
        if !classified_heat_pump_ids.insert(pump.id.clone()) {
            issues.push(issue(
                Severity::Error,
                "heat_pump_id_duplicate",
                format!("{path}.id"),
                "Heat pump IDs must be unique across the project",
            ));
        }
        for pump_issue in pump.validate_inputs() {
            issues.push(issue(
                Severity::Error,
                pump_issue.code,
                format!("{path}.{}", pump_issue.field),
                pump_issue.message,
            ));
        }
        audit_served_zones(pump, &path, &zone_ids, true, &mut issues);
        issues.push(issue(
            Severity::Warning,
            "heat_pump_route_unimplemented",
            path,
            "This classified heat pump is stored but has no verified NTA calculation route",
        ));
    }

    for (path, pump) in &classified_pumps {
        if let Some(input) = &pump.forfait_heat_pump_draft {
            issues.push(issue(
                Severity::Warning,
                "forfait_heat_pump_draft_unverified",
                format!("{path}.forfaitHeatPumpDraft"),
                "Consultation-draft COP table input is stored but final-edition applicability and annual performance are unverified",
            ));
            if forfait_heat_pump_draft::source_fallback_applies(input) {
                let (code, message) = if input.source
                    == forfait_heat_pump_draft::TableSource::GroundOrGroundwaterUnknown
                {
                    (
                        "forfait_heat_pump_draft_unknown_ground_source_fallback",
                        "An unknown ground/groundwater source type or temperature uses the ground table row",
                    )
                } else {
                    (
                        "forfait_heat_pump_draft_source_fallback",
                        "A source at or above 20 C without a source quality declaration uses the groundwater-below-15-C table row",
                    )
                };
                issues.push(issue(
                    Severity::Warning,
                    code,
                    format!("{path}.forfaitHeatPumpDraft.source"),
                    message,
                ));
            }
            if input.scope == forfait_heat_pump_draft::TableScope::ResidentialAtMost25Kw
                && project.building_function != BuildingFunction::Residential
            {
                issues.push(issue(
                    Severity::Error,
                    "forfait_heat_pump_draft_building_scope_invalid",
                    format!("{path}.forfaitHeatPumpDraft.scope"),
                    "Residential table 9.27 cannot be selected for a non-residential project",
                ));
            }
            if input.thermal_capacity_kw.is_none()
                || input.capacity_source_reference.is_none()
                || input.collective_building_installation.is_none()
            {
                issues.push(issue(
                    Severity::Warning,
                    "forfait_heat_pump_draft_scope_evidence_missing",
                    format!("{path}.forfaitHeatPumpDraft"),
                    "Thermal capacity, its source and collective-installation status are needed to substantiate the 25 kW table boundary",
                ));
            }
            if project.building_function == BuildingFunction::Residential
                && input.scope == forfait_heat_pump_draft::TableScope::UtilityCollectiveOrOver25Kw
                && input.collective_building_installation == Some(false)
                && input.thermal_capacity_kw.is_some_and(|value| value <= 25.0)
            {
                issues.push(issue(
                    Severity::Error,
                    "forfait_heat_pump_draft_building_scope_invalid",
                    format!("{path}.forfaitHeatPumpDraft.scope"),
                    "An individual residential heat pump of at most 25 kW cannot use table 9.29",
                ));
            }
        }
        if let Some(input) = &pump.gas_heat_pump_forfait_draft {
            issues.push(issue(
                Severity::Warning,
                "gas_heat_pump_forfait_draft_unverified",
                format!("{path}.gasHeatPumpForfaitDraft"),
                "Gas COP consultation-table input is stored; gas input, auxiliaries, BENG and final-edition applicability are unverified",
            ));
            if input.application == gas_heat_pump_forfait_draft::GasPumpApplication::Utility
                && project.building_function == BuildingFunction::Residential
            {
                issues.push(issue(
                    Severity::Error,
                    "gas_heat_pump_forfait_draft_building_scope_invalid",
                    format!("{path}.gasHeatPumpForfaitDraft.application"),
                    "Utility table application cannot be selected for a residential project",
                ));
            }
            if input.application
                == gas_heat_pump_forfait_draft::GasPumpApplication::ResidentialCollectiveAtMost25Kw
                && project.building_function != BuildingFunction::Residential
            {
                issues.push(issue(
                    Severity::Error,
                    "gas_heat_pump_forfait_draft_building_scope_invalid",
                    format!("{path}.gasHeatPumpForfaitDraft.application"),
                    "Residential table 9.27 cannot be selected for a non-residential project",
                ));
            }
        }
        if pump.gas_heat_pump_aux_draft.is_some() {
            issues.push(issue(
                Severity::Warning,
                "gas_heat_pump_aux_draft_unverified",
                format!("{path}.gasHeatPumpAuxDraft"),
                "Gas generator auxiliary input is stored; month hours, coefficients, final-edition applicability and BENG are unverified",
            ));
        }
        if pump.heating_aux_measured_draft.is_some() {
            issues.push(issue(
                Severity::Warning,
                "heating_aux_measured_draft_unverified",
                format!("{path}.heatingAuxMeasuredDraft"),
                "Measured auxiliary inputs are stored for a consultation-draft diagnostic only; no verified NTA route uses them",
            ));
        }
        if let Some(limits) = &pump.declared_operating_limits {
            issues.push(issue(
                Severity::Warning,
                "heat_pump_operating_limits_unimplemented",
                format!("{path}.declaredOperatingLimits"),
                "Declared shutoff limits are stored but no verified dispatch or backup energy route is available",
            ));
            if limits.declaration_norm_version != TARGET_NORM_VERSION {
                issues.push(issue(
                    Severity::Warning,
                    "operating_limits_norm_edition_unverified",
                    format!("{path}.declaredOperatingLimits.declarationNormVersion"),
                    "Operating-limit declaration edition differs from the target norm; applicability is unverified",
                ));
            }
        }
        for (index, point) in pump.dhw_test_points.iter().enumerate() {
            if point.declaration_norm_version != TARGET_NORM_VERSION {
                issues.push(issue(
                    Severity::Warning,
                    "dhw_declaration_norm_edition_unverified",
                    format!("{path}.dhwTestPoints[{index}].declarationNormVersion"),
                    "Declaration edition differs from the target norm; applicability is unverified",
                ));
            }
        }
    }
    audit_system_links(
        project,
        &classified_pumps,
        &classified_heat_pump_ids,
        &mut issues,
    );

    let mut unheated_ids = HashSet::new();
    for (index, space) in project.unheated_spaces.iter().enumerate() {
        let path = format!("unheatedSpaces[{index}]");
        if space.id.trim().is_empty() || !unheated_ids.insert(space.id.as_str()) {
            issues.push(issue(
                Severity::Error,
                "unheated_space_id_invalid",
                format!("{path}.id"),
                "Unheated space IDs must be nonempty and unique",
            ));
        }
        if space.name.trim().is_empty() {
            issues.push(issue(
                Severity::Error,
                "unheated_space_name_required",
                format!("{path}.name"),
                "Unheated space needs a name",
            ));
        }
        if !space.reduction_factor.is_finite() || !(0.0..=1.0).contains(&space.reduction_factor) {
            issues.push(issue(
                Severity::Error,
                "unheated_reduction_factor_invalid",
                format!("{path}.reductionFactor"),
                "Supplied reduction factor must be between zero and one",
            ));
        }
        if space.factor_source_reference.trim().is_empty() {
            issues.push(issue(
                Severity::Error,
                "unheated_factor_source_required",
                format!("{path}.factorSourceReference"),
                "Reduction factor needs a traceable source",
            ));
        }
    }
    let mut referenced_unheated_ids = HashSet::new();
    for (zone_index, zone) in project.zones.iter().enumerate() {
        for (collection, records) in [
            ("surfaces", Some(zone.surfaces.as_slice())),
            ("thermalBridges", zone.thermal_bridges.as_deref()),
            ("pointThermalBridges", zone.point_thermal_bridges.as_deref()),
        ] {
            for (index, record) in records.unwrap_or(&[]).iter().enumerate() {
                if record.get("thermalBoundary").and_then(Value::as_str) == Some("unheated_space") {
                    let id = record
                        .get("unheatedSpaceId")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if id.trim().is_empty() || !unheated_ids.contains(id) {
                        issues.push(issue(
                            Severity::Error,
                            "unheated_space_link_invalid",
                            format!("zones[{zone_index}].{collection}[{index}].unheatedSpaceId"),
                            "Unheated boundary must refer to a declared unheated space",
                        ));
                    } else {
                        referenced_unheated_ids.insert(id);
                    }
                }
            }
        }
    }
    for (index, space) in project.unheated_spaces.iter().enumerate() {
        if !referenced_unheated_ids.contains(space.id.as_str()) {
            issues.push(issue(
                Severity::Error,
                "unheated_space_unused",
                format!("unheatedSpaces[{index}].id"),
                "Unheated space needs at least one linked boundary",
            ));
        }
        if !project
            .zones
            .iter()
            .flat_map(|zone| &zone.surfaces)
            .any(|surface| {
                surface.get("thermalBoundary").and_then(Value::as_str) == Some("unheated_space")
                    && surface.get("unheatedSpaceId").and_then(Value::as_str)
                        == Some(space.id.as_str())
            })
        {
            issues.push(issue(
                Severity::Error,
                "unheated_space_surface_required",
                format!("unheatedSpaces[{index}].id"),
                "A diagnostic space needs at least one linked surface",
            ));
        }
    }

    let status = if issues
        .iter()
        .any(|item| matches!(item.severity, Severity::Error))
    {
        "invalid"
    } else {
        "structurally_valid"
    };
    let performance_point_diagnostics = if status == "structurally_valid" {
        classified_pumps
            .iter()
            .flat_map(|(path, pump)| {
                pump.performance_points
                    .iter()
                    .map(move |point| PerformancePointDiagnostic {
                        heat_pump_path: path.clone(),
                        heat_pump_id: pump.id.clone(),
                        point_id: point.id.clone(),
                        service: point.service,
                        input_energy_carrier: point.input_energy_carrier,
                        instantaneous_useful_to_input_ratio: point.useful_capacity_kw
                            / point.input_power_kw,
                        reference_verified: false,
                        annual_performance_available: false,
                    })
            })
            .collect()
    } else {
        Vec::new()
    };
    let dhw_test_diagnostics = if status == "structurally_valid" {
        classified_pumps
            .iter()
            .flat_map(|(_, pump)| {
                pump.dhw_test_points
                    .iter()
                    .map(move |point| DhwTestDiagnostic {
                        heat_pump_id: pump.id.clone(),
                        point_id: point.id.clone(),
                        tap_profile: point.tap_profile.clone(),
                        declaration_norm_version: point.declaration_norm_version.clone(),
                        declared_useful_to_input_ratio: point.useful_energy_kwh_per_day
                            / point.input_energy_kwh_per_day,
                        reference_verified: false,
                        annual_performance_available: false,
                    })
            })
            .collect()
    } else {
        Vec::new()
    };
    let thermal_boundaries = ThermalBoundarySummary {
        surface_count,
        classified_surface_count: classified_surface_boundary_count,
        bridge_count: thermal_bridge_count,
        classified_bridge_count: classified_bridge_boundary_count,
        point_bridge_count,
        classified_point_bridge_count: classified_point_boundary_count,
        point_inventory_complete: point_lists_present,
        complete: surface_count > 0
            && boundary_lists_present
            && point_lists_present
            && classified_surface_boundary_count == surface_count
            && classified_bridge_boundary_count == thermal_bridge_count
            && classified_point_boundary_count == point_bridge_count,
    };
    let direct_outdoor_diagnostic = if status == "structurally_valid" && thermal_boundaries.complete
    {
        direct_outdoor_input(project).and_then(|input| {
            let assessment = direct_transmission::assess_direct_transmission(&input);
            (assessment.status == "input_valid").then_some(assessment)
        })
    } else {
        None
    };
    let unheated_transmission_diagnostic = if status == "structurally_valid"
        && thermal_boundaries.complete
        && !project.unheated_spaces.is_empty()
    {
        unheated_project_input(project).and_then(|input| {
            let assessment = unheated_transmission::assess_unheated_transmission(&input);
            (assessment.status == "input_valid").then_some(assessment)
        })
    } else {
        None
    };
    InputAssessment {
        status,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint,
        calculation_available: false,
        summary: ProjectSummary {
            zone_count: project.zones.len(),
            surface_count,
            system_count,
            heat_pump_count,
            classified_heat_pump_count,
            auxiliary_component_count,
            system_link_count,
            performance_point_diagnostics,
            dhw_test_diagnostics,
            floor_area_m2,
            envelope_geometry,
            thermal_boundaries,
            direct_outdoor_diagnostic,
            unheated_transmission_diagnostic,
        },
        issues,
    }
}

fn audit_system_records(records: &[Value], collection: &str, issues: &mut Vec<ValidationIssue>) {
    let mut ids = HashSet::new();
    for (index, record) in records.iter().enumerate() {
        let path = format!("{collection}[{index}]");
        if !record.is_object() {
            issues.push(issue(
                Severity::Error,
                "system_record_invalid",
                path,
                "System record must be an object",
            ));
            continue;
        }
        let id = record.get("id").and_then(Value::as_str).unwrap_or("");
        if id.trim().is_empty() || !ids.insert(id) {
            issues.push(issue(
                Severity::Error,
                "system_id_invalid",
                format!("{path}.id"),
                "System IDs must be nonempty and unique within their collection",
            ));
        }
        if record
            .get("type")
            .and_then(Value::as_str)
            .map_or(true, |value| value.trim().is_empty())
        {
            issues.push(issue(
                Severity::Error,
                "system_type_required",
                format!("{path}.type"),
                "System type must be a nonempty string",
            ));
        }
    }
}

fn direct_outdoor_input(project: &ProjectInput) -> Option<DirectTransmissionInput> {
    direct_boundary_input(project, ThermalBoundary::Outdoor, None)
}

pub(crate) fn unheated_project_input(project: &ProjectInput) -> Option<UnheatedTransmissionInput> {
    unheated_zone_input(project, None)
}

/// Unheated-space transmission, optionally limited to one zone; spaces without
/// a bordering surface in that zone are left out.
pub(crate) fn unheated_zone_input(
    project: &ProjectInput,
    zone_id: Option<&str>,
) -> Option<UnheatedTransmissionInput> {
    let spaces = project
        .unheated_spaces
        .iter()
        .filter(|space| {
            zone_id.map_or(true, |zone_id| {
                project
                    .zones
                    .iter()
                    .filter(|zone| zone.id == zone_id)
                    .any(|zone| {
                        zone.surfaces.iter().any(|surface| {
                            surface.get("unheatedSpaceId").and_then(Value::as_str)
                                == Some(space.id.as_str())
                        })
                    })
            })
        })
        .map(|space| {
            Some(UnheatedSpaceInput {
                id: space.id.clone(),
                reduction_factor: space.reduction_factor,
                factor_source_reference: space.factor_source_reference.clone(),
                boundary: direct_boundary_input_zone(
                    project,
                    ThermalBoundary::UnheatedSpace,
                    Some(&space.id),
                    zone_id,
                )?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(UnheatedTransmissionInput { spaces })
}

pub(crate) fn direct_boundary_input(
    project: &ProjectInput,
    target: ThermalBoundary,
    space_id: Option<&str>,
) -> Option<DirectTransmissionInput> {
    direct_boundary_input_zone(project, target, space_id, None)
}

pub(crate) fn direct_boundary_input_zone(
    project: &ProjectInput,
    target: ThermalBoundary,
    space_id: Option<&str>,
    zone_id: Option<&str>,
) -> Option<DirectTransmissionInput> {
    let constructions: HashMap<&str, &Value> = project
        .constructions
        .iter()
        .filter_map(|item| Some((item.get("id")?.as_str()?, item)))
        .collect();
    let mut elements = Vec::new();
    let mut linear_bridges = Vec::new();
    let mut point_bridges = Vec::new();
    for zone in project
        .zones
        .iter()
        .filter(|zone| zone_id.map_or(true, |id| zone.id == id))
    {
        for surface in &zone.surfaces {
            let boundary: ThermalBoundary =
                serde_json::from_value(surface.get("thermalBoundary")?.clone()).ok()?;
            if boundary != target
                || (space_id.is_some()
                    && surface.get("unheatedSpaceId").and_then(Value::as_str) != space_id)
            {
                continue;
            }
            let surface_id = surface.get("id")?.as_str()?;
            let gross_area = surface.get("area")?.as_f64()?;
            let windows = surface.get("windows")?.as_array()?;
            let mut window_area = 0.0;
            for window in windows {
                let window_id = window.get("id")?.as_str()?;
                let area = window.get("area")?.as_f64()?;
                let u_value = window.get("uValue")?.as_f64()?;
                if !area.is_finite() || area <= 0.0 || !u_value.is_finite() || u_value <= 0.0 {
                    return None;
                }
                window_area += area;
                elements.push(DirectElement {
                    id: format!("window:{window_id}"),
                    area_m2: area,
                    u_value_w_per_m2k: u_value,
                    source_reference: format!("project:window:{window_id}.uValue"),
                });
            }
            let opaque_area = gross_area - window_area;
            if !opaque_area.is_finite() || opaque_area < -1e-9 {
                return None;
            }
            if opaque_area > 1e-9 {
                let construction_id = surface.get("constructionId")?.as_str()?;
                let construction = constructions.get(construction_id)?;
                let u_value = construction.get("uValue")?.as_f64()?;
                if !u_value.is_finite() || u_value <= 0.0 {
                    return None;
                }
                elements.push(DirectElement {
                    id: format!("surface:{surface_id}:opaque"),
                    area_m2: opaque_area,
                    u_value_w_per_m2k: u_value,
                    source_reference: format!("project:construction:{construction_id}.uValue"),
                });
            }
        }
        for bridge in zone.thermal_bridges.as_deref()? {
            let boundary: ThermalBoundary =
                serde_json::from_value(bridge.get("thermalBoundary")?.clone()).ok()?;
            if boundary != target
                || (space_id.is_some()
                    && bridge.get("unheatedSpaceId").and_then(Value::as_str) != space_id)
            {
                continue;
            }
            let id = bridge.get("id")?.as_str()?;
            linear_bridges.push(LinearBridge {
                id: format!("bridge:{id}"),
                length_m: bridge.get("length")?.as_f64()?,
                psi_w_per_mk: bridge.get("psiValue")?.as_f64()?,
                source_reference: format!("project:thermalBridge:{id}.psiValue"),
            });
        }
        for bridge in zone.point_thermal_bridges.as_deref()? {
            let boundary: ThermalBoundary =
                serde_json::from_value(bridge.get("thermalBoundary")?.clone()).ok()?;
            if boundary != target
                || (space_id.is_some()
                    && bridge.get("unheatedSpaceId").and_then(Value::as_str) != space_id)
            {
                continue;
            }
            let id = bridge.get("id")?.as_str()?;
            point_bridges.push(PointBridge {
                id: format!("point:{id}"),
                chi_w_per_k: bridge.get("chiValue")?.as_f64()?,
                source_reference: bridge.get("sourceReference")?.as_str()?.to_owned(),
            });
        }
    }
    if elements.is_empty() {
        return None;
    }
    Some(DirectTransmissionInput {
        elements,
        linear_bridges,
        point_bridges,
    })
}

fn audit_system_links(
    project: &ProjectInput,
    pumps: &[(String, HeatPumpInput)],
    heat_pump_ids: &HashSet<String>,
    issues: &mut Vec<ValidationIssue>,
) {
    let heating_ids: HashSet<&str> = project
        .heating_systems
        .iter()
        .filter_map(|system| system.get("id").and_then(Value::as_str))
        .collect();
    let hot_water_ids: HashSet<&str> = project
        .hot_water_systems
        .iter()
        .filter_map(|system| system.get("id").and_then(Value::as_str))
        .collect();
    let ventilation_ids: HashSet<&str> = project
        .ventilation_systems
        .iter()
        .filter_map(|system| system.get("id").and_then(Value::as_str))
        .collect();
    let upstream: HashMap<&str, Vec<&str>> = pumps
        .iter()
        .map(|(_, pump)| {
            (
                pump.id.as_str(),
                pump.system_links
                    .iter()
                    .filter(|link| {
                        link.role == SystemLinkRole::UpstreamHeatPump
                            && link.target_kind == SystemLinkTargetKind::HeatPump
                    })
                    .map(|link| link.target_id.as_str())
                    .collect(),
            )
        })
        .collect();
    for (path, pump) in pumps {
        if pump.source == HeatSource::ExhaustAir
            && !pump
                .system_links
                .iter()
                .any(|link| link.role == SystemLinkRole::SourceVentilation)
        {
            issues.push(issue(
                Severity::Warning,
                "exhaust_air_ventilation_link_missing",
                format!("{path}.systemLinks"),
                "Exhaust-air heat pump has no recorded source ventilation system",
            ));
        }
        for (index, link) in pump.system_links.iter().enumerate() {
            let field = format!("{path}.systemLinks[{index}].targetId");
            let exists = match link.target_kind {
                SystemLinkTargetKind::HeatPump => heat_pump_ids.contains(&link.target_id),
                SystemLinkTargetKind::HeatingSystem => {
                    heating_ids.contains(link.target_id.as_str())
                }
                SystemLinkTargetKind::HotWaterSystem => {
                    hot_water_ids.contains(link.target_id.as_str())
                }
                SystemLinkTargetKind::VentilationSystem => {
                    ventilation_ids.contains(link.target_id.as_str())
                }
            };
            if !exists {
                issues.push(issue(
                    Severity::Error,
                    "system_link_target_missing",
                    field.clone(),
                    "Linked equipment does not exist in the project",
                ));
            }
            if link.role == SystemLinkRole::UpstreamHeatPump
                && link.target_kind == SystemLinkTargetKind::HeatPump
                && reaches_heat_pump(&upstream, link.target_id.as_str(), pump.id.as_str())
            {
                issues.push(issue(
                    Severity::Error,
                    "system_link_cycle",
                    field,
                    "Upstream heat pump links must not form a cycle",
                ));
            }
        }
    }
}

fn reaches_heat_pump(graph: &HashMap<&str, Vec<&str>>, start: &str, goal: &str) -> bool {
    let mut stack = vec![start];
    let mut visited = HashSet::new();
    while let Some(id) = stack.pop() {
        if id == goal {
            return true;
        }
        if visited.insert(id) {
            if let Some(next) = graph.get(id) {
                stack.extend(next.iter().copied());
            }
        }
    }
    false
}

fn audit_served_zones(
    pump: &HeatPumpInput,
    path: &str,
    zone_ids: &HashSet<&str>,
    require_explicit_zones: bool,
    issues: &mut Vec<ValidationIssue>,
) {
    for (index, component) in pump.auxiliary_components.iter().enumerate() {
        if component.measurement_boundary == AuxiliaryMeasurementBoundary::Unknown {
            issues.push(issue(
                Severity::Warning,
                "auxiliary_measurement_boundary_unknown",
                format!("{path}.auxiliaryComponents[{index}].measurementBoundary"),
                "Auxiliary power has no declared boundary relative to performance data",
            ));
        }
    }
    if pump.performance_evidence.kind == PerformanceEvidenceKind::ControlledQualityDeclaration
        && pump.performance_evidence.registry_record.is_none()
    {
        issues.push(issue(
            Severity::Warning,
            "quality_declaration_registry_record_missing",
            format!("{path}.performanceEvidence.registryRecord"),
            "Quality declaration registry identity has not been recorded",
        ));
    }
    if require_explicit_zones
        && !matches!(pump.sink, HeatSink::DomesticHotWater)
        && pump.served_zone_ids.is_empty()
    {
        issues.push(issue(
            Severity::Warning,
            "heat_pump_zones_missing",
            format!("{path}.servedZoneIds"),
            "No served calculation zones have been assigned to this heat pump",
        ));
    }
    let mut seen = HashSet::new();
    for (zone_index, zone_id) in pump.served_zone_ids.iter().enumerate() {
        let field = format!("{path}.servedZoneIds[{zone_index}]");
        if !zone_ids.contains(zone_id.as_str()) {
            issues.push(issue(
                Severity::Error,
                "heat_pump_zone_missing",
                field.clone(),
                "Heat pump references a calculation zone that does not exist",
            ));
        }
        if !seen.insert(zone_id.as_str()) {
            issues.push(issue(
                Severity::Error,
                "heat_pump_zone_duplicate",
                field,
                "A served calculation zone may occur only once per heat pump",
            ));
        }
    }
}

pub fn calculate_beng(_project: &ProjectInput) -> Result<(), &'static str> {
    Err("NTA 8800 calculation is not implemented in the Rust kernel yet")
}

fn issue(
    severity: Severity,
    code: &'static str,
    path: impl Into<String>,
    message: &'static str,
) -> ValidationIssue {
    ValidationIssue {
        severity,
        code,
        path: path.into(),
        message,
        detail: None,
    }
}

fn issue_with_detail(
    severity: Severity,
    code: &'static str,
    path: impl Into<String>,
    message: &'static str,
    detail: String,
) -> ValidationIssue {
    ValidationIssue {
        detail: Some(detail),
        ..issue(severity, code, path, message)
    }
}

fn audit_thermal_boundary(
    value: Option<&Value>,
    path: String,
    issues: &mut Vec<ValidationIssue>,
) -> Option<ThermalBoundary> {
    let value = value?;
    match serde_json::from_value::<ThermalBoundary>(value.clone()) {
        Ok(boundary) => Some(boundary),
        Err(_) => {
            issues.push(issue(
                Severity::Error,
                "thermal_boundary_invalid",
                path,
                "Thermal boundary must be one of the supported explicit categories",
            ));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Value {
        json!({
            "id": "test", "name": "Test house", "buildingFunction": "residential",
            "zones": [{"id": "z1", "floorArea": 100.0, "volume": 250.0,
                "surfaces": [{"id": "wall", "area": 50.0, "zoneId": "z1", "windows": []}]}]
        })
    }

    #[test]
    fn reports_area_and_structure_without_claiming_calculation() {
        let result = assess_json(sample()).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert_eq!(result.summary.floor_area_m2, 100.0);
        assert_eq!(result.summary.surface_count, 1);
        assert!(!result.calculation_available);
        assert_eq!(result.kernel_version, KERNEL_VERSION);
        assert!(result.input_fingerprint.starts_with("sha256:"));
        let geometry = result.summary.envelope_geometry.unwrap();
        assert_eq!(geometry.gross_surface_area_m2, 50.0);
        assert_eq!(geometry.window_area_m2, 0.0);
        assert_eq!(geometry.remaining_opaque_area_m2, 50.0);
    }

    #[test]
    fn historical_declared_dhw_test_ratio_does_not_enable_annual_calculation() {
        let mut value = sample();
        value["ntaHeatPumps"] = json!([serde_json::from_str::<Value>(include_str!(
            "../../../training-data/bcrg-20240123gk-dhw-test-input.json"
        ))
        .unwrap()]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(!result.calculation_available);
        assert_eq!(result.summary.dhw_test_diagnostics.len(), 2);
        let first = &result.summary.dhw_test_diagnostics[0];
        assert!((first.declared_useful_to_input_ratio - 5.865 / 2.520).abs() < 1e-12);
        assert!(!first.reference_verified);
        assert!(!first.annual_performance_available);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "dhw_declaration_norm_edition_unverified"));
    }

    #[test]
    fn hybrid_shutoff_declaration_is_audited_without_dispatch() {
        let mut value = sample();
        value["ntaHeatPumps"] = json!([serde_json::from_str::<Value>(include_str!(
            "../../../training-data/bcrg-20240234gk-hybrid-limit-input.json"
        ))
        .unwrap()]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(!result.calculation_available);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_operating_limits_unimplemented"));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "operating_limits_norm_edition_unverified"));
    }

    #[test]
    fn rejects_finite_zone_areas_whose_project_sum_overflows() {
        let mut value = sample();
        value["zones"] = json!([
            {"id":"z1", "floorArea":1e308, "volume":250.0,
                "surfaces":[{"id":"s1", "area":10.0, "windows":[]}]},
            {"id":"z2", "floorArea":1e308, "volume":250.0,
                "surfaces":[{"id":"s2", "area":10.0, "windows":[]}]}
        ]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "floor_area_sum_invalid"));
        assert!(result.summary.floor_area_m2.is_finite());
        assert!(!result.calculation_available);
    }

    #[test]
    fn rejects_finite_surface_areas_whose_sum_overflows() {
        let mut value = sample();
        value["zones"][0]["surfaces"] = json!([
            {"id":"s1", "area":1e308, "windows":[]},
            {"id":"s2", "area":1e308, "windows":[]}
        ]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "surface_area_sum_invalid"));
        assert!(result.summary.envelope_geometry.is_none());
    }

    #[test]
    fn rejects_overflow_across_zone_geometry_totals() {
        let mut value = sample();
        value["zones"] = json!([
            {"id":"z1", "floorArea":10.0, "volume":25.0,
                "surfaces":[{"id":"s1", "area":1e308, "windows":[]}]},
            {"id":"z2", "floorArea":10.0, "volume":25.0,
                "surfaces":[{"id":"s2", "area":1e308, "windows":[]}]}
        ]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "project_envelope_area_sum_invalid"));
        assert!(result.summary.envelope_geometry.is_none());
    }

    #[test]
    fn keeps_gross_glazing_and_opaque_area_separate_by_zone() {
        let mut value = sample();
        value["zones"][0]["surfaces"][0]["windows"] = json!([
            {"id":"w1", "area":10.0, "surfaceId":"wall"},
            {"id":"w2", "area":5.0, "surfaceId":"wall"}
        ]);
        let result = assess_json(value).unwrap();
        let geometry = result.summary.envelope_geometry.unwrap();
        assert_eq!(geometry.gross_surface_area_m2, 50.0);
        assert_eq!(geometry.window_area_m2, 15.0);
        assert_eq!(geometry.remaining_opaque_area_m2, 35.0);
        assert_eq!(geometry.zones[0].zone_id, "z1");
    }

    #[test]
    fn derives_only_explicit_outdoor_parts_for_a_project_diagnostic() {
        let mut value = sample();
        value["constructions"] = json!([
            {"id":"c1", "rcValue":5.0, "uValue":0.2, "layers":[]},
            {"id":"c2", "rcValue":3.0, "uValue":0.3, "layers":[]}
        ]);
        value["zones"][0]["surfaces"] = json!([
            {"id":"wall", "area":10.0, "constructionId":"c1", "zoneId":"z1",
                "thermalBoundary":"outdoor", "windows":[
                    {"id":"w1", "area":2.0, "uValue":1.1, "gValue":0.4, "surfaceId":"wall"}
                ]},
            {"id":"floor", "area":20.0, "constructionId":"c2", "zoneId":"z1",
                "thermalBoundary":"ground", "windows":[]}
        ]);
        value["zones"][0]["thermalBridges"] = json!([{
            "id":"edge", "length":3.0, "psiValue":0.05, "zoneId":"z1",
            "thermalBoundary":"outdoor"
        }]);
        value["zones"][0]["pointThermalBridges"] = json!([]);
        value["zones"][0]["pointBridgeInventoryComplete"] = json!(true);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(result.summary.thermal_boundaries.complete);
        assert_eq!(
            result.summary.thermal_boundaries.classified_surface_count,
            2
        );
        let diagnostic = result.summary.direct_outdoor_diagnostic.unwrap();
        assert!((diagnostic.element_conductance_w_per_k.unwrap() - 3.8).abs() < 1e-12);
        assert!((diagnostic.linear_bridge_conductance_w_per_k.unwrap() - 0.15).abs() < 1e-12);
        assert!((diagnostic.total_direct_conductance_w_per_k.unwrap() - 3.95).abs() < 1e-12);
        assert!(!diagnostic.reference_verified);
    }

    #[test]
    fn project_unheated_diagnostic_requires_named_space_and_complete_links() {
        let mut value = sample();
        value["constructions"] = json!([{"id":"c1", "rcValue":5.0, "uValue":0.4, "layers":[]}]);
        value["zones"][0]["surfaces"] = json!([{
            "id":"wall", "area":10.0, "constructionId":"c1", "zoneId":"z1",
            "thermalBoundary":"unheated_space", "unheatedSpaceId":"garage", "windows":[]
        }]);
        value["zones"][0]["thermalBridges"] = json!([{
            "id":"edge", "length":2.0, "psiValue":0.1, "zoneId":"z1",
            "thermalBoundary":"unheated_space", "unheatedSpaceId":"garage"
        }]);
        value["zones"][0]["pointThermalBridges"] = json!([]);
        value["zones"][0]["pointBridgeInventoryComplete"] = json!(true);
        value["unheatedSpaces"] = json!([{
            "id":"garage", "name":"Garage", "reductionFactor":0.5, "factorSourceReference":"user-supplied-b-1"
        }]);
        let result = assess_json(value.clone()).unwrap();
        assert_eq!(result.status, "structurally_valid");
        let diagnosis = result.summary.unheated_transmission_diagnostic.unwrap();
        assert!((diagnosis.total_reduced_conductance_w_per_k.unwrap() - 2.1).abs() < 1e-12);
        assert!(!diagnosis.reference_verified);
        assert!(!diagnosis.beng_calculation_available);

        value["zones"][0]["thermalBridges"][0]["unheatedSpaceId"] = json!("missing");
        let broken = assess_json(value.clone()).unwrap();
        assert_eq!(broken.status, "invalid");
        assert!(broken.summary.unheated_transmission_diagnostic.is_none());
        assert!(broken
            .issues
            .iter()
            .any(|issue| issue.code == "unheated_space_link_invalid"));

        value["zones"][0]["thermalBridges"][0]["unheatedSpaceId"] = json!("garage");
        value["unheatedSpaces"][0]["reductionFactor"] = json!(1.2);
        let factor = assess_json(value).unwrap();
        assert_eq!(factor.status, "invalid");
        assert!(factor.summary.unheated_transmission_diagnostic.is_none());
    }

    #[test]
    fn missing_or_unknown_boundary_suppresses_project_diagnostic() {
        let mut value = sample();
        value["zones"][0]["surfaces"][0]["thermalBoundary"] = json!("outdoor");
        let no_bridge_list = assess_json(value.clone()).unwrap();
        assert!(!no_bridge_list.summary.thermal_boundaries.complete);
        assert!(no_bridge_list.summary.direct_outdoor_diagnostic.is_none());

        value["zones"][0]["thermalBridges"] = json!([]);
        value["zones"][0]["surfaces"][0]["thermalBoundary"] = json!("unknown");
        let unknown = assess_json(value).unwrap();
        assert_eq!(unknown.status, "invalid");
        assert!(unknown
            .issues
            .iter()
            .any(|issue| issue.code == "thermal_boundary_invalid"));
        assert!(unknown.summary.direct_outdoor_diagnostic.is_none());
    }

    #[test]
    fn point_bridge_inventory_is_explicit_and_only_outdoor_chi_is_summed() {
        let mut value = sample();
        value["constructions"] = json!([{"id":"c", "rcValue":5, "uValue":0.2, "layers":[]}]);
        value["zones"][0]["surfaces"][0] = json!({
            "id":"wall", "area":10, "zoneId":"z1", "constructionId":"c",
            "thermalBoundary":"outdoor", "windows":[]
        });
        value["zones"][0]["thermalBridges"] = json!([]);
        let missing = assess_json(value.clone()).unwrap();
        assert!(!missing.summary.thermal_boundaries.point_inventory_complete);
        assert!(missing.summary.direct_outdoor_diagnostic.is_none());

        value["zones"][0]["pointThermalBridges"] = json!([
            {"id":"p-out", "chiValue":0.04, "zoneId":"z1",
                "thermalBoundary":"outdoor", "sourceReference":"detail-1"},
            {"id":"p-ground", "chiValue":0.2, "zoneId":"z1",
                "thermalBoundary":"ground", "sourceReference":"detail-2"}
        ]);
        let unconfirmed = assess_json(value.clone()).unwrap();
        assert!(
            !unconfirmed
                .summary
                .thermal_boundaries
                .point_inventory_complete
        );
        assert!(unconfirmed.summary.direct_outdoor_diagnostic.is_none());
        value["zones"][0]["pointBridgeInventoryComplete"] = json!(true);
        let result = assess_json(value.clone()).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert_eq!(result.summary.thermal_boundaries.point_bridge_count, 2);
        let diagnostic = result.summary.direct_outdoor_diagnostic.unwrap();
        assert!((diagnostic.point_bridge_conductance_w_per_k.unwrap() - 0.04).abs() < 1e-12);
        assert!((diagnostic.total_direct_conductance_w_per_k.unwrap() - 2.04).abs() < 1e-12);

        value["zones"][0]["pointThermalBridges"][0]["sourceReference"] = json!("");
        let invalid = assess_json(value).unwrap();
        assert_eq!(invalid.status, "invalid");
        assert!(invalid.summary.direct_outdoor_diagnostic.is_none());
        assert!(invalid
            .issues
            .iter()
            .any(|issue| issue.code == "point_bridge_source_required"));
    }

    #[test]
    fn omits_area_bookkeeping_when_window_list_is_missing() {
        let mut value = sample();
        value["zones"][0]["surfaces"][0]
            .as_object_mut()
            .unwrap()
            .remove("windows");
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(result.summary.envelope_geometry.is_none());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "windows_missing"));
    }

    #[test]
    fn fingerprint_is_stable_across_key_order_and_changes_with_input() {
        let a: Value = serde_json::from_str(
            r#"{"id":"x","name":"A","buildingFunction":"residential","zones":[]}"#,
        )
        .unwrap();
        let b: Value = serde_json::from_str(
            r#"{"zones":[],"buildingFunction":"residential","name":"A","id":"x"}"#,
        )
        .unwrap();
        let original_fingerprint = assess_json(a.clone()).unwrap().input_fingerprint;
        assert_eq!(
            original_fingerprint,
            assess_json(b).unwrap().input_fingerprint
        );
        let mut changed = a;
        changed["name"] = json!("B");
        assert_ne!(
            assess_json(changed).unwrap().input_fingerprint,
            original_fingerprint,
        );
    }

    #[test]
    fn rejects_zero_area_instead_of_dividing_by_one() {
        let mut value = sample();
        value["zones"][0]["floorArea"] = json!(0.0);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "floor_area_invalid"));
    }

    #[test]
    fn rejects_overlapping_window_area_and_broken_geometry_links() {
        let mut value = sample();
        value["zones"][0]["surfaces"][0]["zoneId"] = json!("other-zone");
        value["zones"][0]["surfaces"][0]["windows"] = json!([
            {"id": "w1", "area": 30.0, "surfaceId": "wall"},
            {"id": "w2", "area": 25.0, "surfaceId": "other-wall"}
        ]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        for code in [
            "surface_zone_mismatch",
            "window_surface_mismatch",
            "window_area_exceeds_surface",
        ] {
            assert!(
                result.issues.iter().any(|item| item.code == code),
                "missing {code}"
            );
        }
        assert!(result.summary.envelope_geometry.is_none());
    }

    #[test]
    fn rejects_nonphysical_envelope_properties_without_discarding_area_bookkeeping() {
        let mut value = sample();
        value["constructions"] = json!([{
            "id": "c1", "rcValue": -0.5, "uValue": 0,
            "layers": [{"thickness": 0, "lambda": -0.02}]
        }, {
            "id": "c2", "rcValue": 1.0, "uValue": 0.8, "layers": "bad"
        }]);
        value["zones"][0]["airTightness"] = json!({"qv10": -1});
        value["zones"][0]["thermalBridges"] = json!([{
            "id": "b1", "length": 0, "psiValue": 0.05, "zoneId": "wrong-zone"
        }]);
        value["zones"][0]["surfaces"][0]["windows"] = json!([{
            "id": "w1", "area": 5, "uValue": 0, "gValue": 1.2,
            "surfaceId": "wall"
        }, {
            "id": "w2", "area": 1, "uValue": "bad", "gValue": null,
            "surfaceId": "wall"
        }]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        for code in [
            "construction_thermal_value_invalid",
            "construction_layer_invalid",
            "construction_layers_invalid",
            "airtightness_invalid",
            "thermal_bridge_length_invalid",
            "thermal_bridge_zone_mismatch",
            "window_u_invalid",
            "window_g_invalid",
        ] {
            assert!(
                result.issues.iter().any(|item| item.code == code),
                "missing {code}"
            );
        }
        let area = result.summary.envelope_geometry.unwrap();
        assert_eq!(area.window_area_m2, 6.0);
        assert_eq!(area.remaining_opaque_area_m2, 44.0);
    }

    #[test]
    fn accepts_finite_negative_thermal_bridge_psi_as_an_input_value() {
        let mut value = sample();
        value["zones"][0]["thermalBridges"] = json!([{
            "id": "b1", "length": 10, "psiValue": -0.05, "zoneId": "z1"
        }]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(!result
            .issues
            .iter()
            .any(|item| item.code == "thermal_bridge_psi_invalid"));
    }

    #[test]
    fn accepts_existing_oes_project_shapes_without_treating_them_as_reference_results() {
        let fixtures = [
            (
                "Aalten woning",
                include_str!("../../../training-data/2522-woning-aalten.oes.json"),
                1,
                67.0,
            ),
            (
                "Gouda woning",
                include_str!("../../../training-data/2467-goejanverwelledijk-gouda.oes.json"),
                1,
                133.06,
            ),
            (
                "Kijkduin reddingspost",
                include_str!("../../../training-data/2786-reddingspost-kijkduin.oes.json"),
                2,
                260.0,
            ),
        ];
        for (name, raw_fixture, zone_count, floor_area_m2) in fixtures {
            let fixture: Value = serde_json::from_str(raw_fixture).unwrap();
            let result = assess_json(fixture["project"].clone()).unwrap();
            assert_eq!(result.status, "structurally_valid", "{name}");
            assert_eq!(result.summary.zone_count, zone_count, "{name}");
            assert_eq!(result.summary.surface_count, 7, "{name}");
            assert!(
                (result.summary.floor_area_m2 - floor_area_m2).abs() < 1e-9,
                "{name}"
            );
            assert_eq!(result.summary.heat_pump_count, 2, "{name}");
            assert_eq!(result.summary.classified_heat_pump_count, 0, "{name}");
            assert!(!result.calculation_available, "{name}");
            assert!(
                result
                    .issues
                    .iter()
                    .any(|item| item.code == "heat_pump_route_unclassified"),
                "{name}"
            );
        }
    }

    #[test]
    fn historical_edr_epw001_geometry_matches_published_input_totals_only() {
        // ISSO 54 (12-05-2022), EP-W001, printed p. 5: Ag=96 m²,
        // Ao=247.2 m², window area=24 m², V=259.2 m³.
        // These are input geometry, not the missing independent energy results
        // and not an NTA 8800:2025+C1:2026 reference case.
        let raw = include_str!("../../../training-data/edr-2022-epw001-geometry.json");
        let project: Value = serde_json::from_str(raw).unwrap();
        assert_eq!(project["zones"][0]["volume"], 259.2);
        let assessment = assess_json(project).unwrap();
        assert_eq!(assessment.status, "structurally_valid");
        assert!(!assessment.calculation_available);
        assert_eq!(assessment.summary.floor_area_m2, 96.0);
        let geometry = assessment.summary.envelope_geometry.unwrap();
        assert!((geometry.gross_surface_area_m2 - 247.2).abs() < 1e-9);
        assert_eq!(geometry.window_area_m2, 24.0);
        assert!((geometry.remaining_opaque_area_m2 - 223.2).abs() < 1e-9);
        assert!(assessment.summary.direct_outdoor_diagnostic.is_none());
    }

    #[test]
    fn audits_classified_heat_pump_and_provenance() {
        let mut value = sample();
        value["heatingSystems"] = json!([{
            "id": "hp-1", "type": "heat_pump_air", "cop": 4.1, "coverageFraction": 1.0,
            "ntaHeatPump": {
                "id": "hp-1", "source": "outdoor_air", "sink": "hydronic",
                "drive": "electric_compression", "reversible": true,
                "hybrid": false, "booster": false,
                "performanceEvidence": {
                    "kind": "controlled_quality_declaration", "reference": "BCRG-example-123"
                }
            }
        }]);
        let result = assess_json(value.clone()).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert_eq!(result.summary.classified_heat_pump_count, 1);
        assert!(result.issues.iter().any(|item| item.code
            == "quality_declaration_registry_record_missing"
            && matches!(item.severity, Severity::Warning)));

        value["heatingSystems"][0]["ntaHeatPump"]["performanceEvidence"]["registryRecord"] = json!({
            "registrationNumber":"20260214GK", "productName":"Model A + tank B",
            "manufacturer":"Supplier", "sourceUrl":"https://bcrg.nl/declaration/example"
        });
        let with_registry = assess_json(value.clone()).unwrap();
        assert!(with_registry.issues.is_empty());

        value["heatingSystems"][0]["ntaHeatPump"]["source"] = json!("groundwater");
        value["heatingSystems"][0]["ntaHeatPump"]["performanceEvidence"]["reference"] = json!("");
        let invalid = assess_json(value).unwrap();
        assert_eq!(invalid.status, "invalid");
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_source_mismatch"));
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "quality_declaration_reference_required"));
    }

    #[test]
    fn reports_embedded_auxiliary_components_without_using_them_for_beng() {
        let mut value = sample();
        value["heatingSystems"] = json!([{
            "id":"hp-1", "type":"heat_pump_air", "cop":3.5, "coverageFraction":1.0,
            "ntaHeatPump": {
                "id":"hp-1", "source":"outdoor_air", "sink":"hydronic",
                "drive":"electric_compression", "reversible":false,
                "hybrid":false, "booster":false,
                "performanceEvidence":{"kind":"normative_default", "reference":null},
                "auxiliaryComponents":[{
                    "id":"fan", "kind":"source_fan", "service":"space_heating",
                    "nominalPowerW":75, "energyCarrier":"electricity",
                    "measurementBoundary":"included_in_declared_performance",
                    "evidenceReference":"manual-1"
                }]
            }
        }]);
        let result = assess_json(value.clone()).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert_eq!(result.summary.auxiliary_component_count, 1);
        assert!(!result.calculation_available);
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "heat_pump_auxiliaries_unimplemented"));
        value["heatingSystems"][0]["ntaHeatPump"]["auxiliaryComponents"][0]
            ["measurementBoundary"] = json!("unknown");
        let unknown = assess_json(value).unwrap();
        assert_eq!(unknown.status, "structurally_valid");
        assert!(unknown.issues.iter().any(|issue| issue.code
            == "auxiliary_measurement_boundary_unknown"
            && issue.path
                == "heatingSystems[0].ntaHeatPump.auxiliaryComponents[0].measurementBoundary"));
    }

    #[test]
    fn rejects_nonphysical_legacy_heat_pump_values() {
        let mut value = sample();
        value["heatingSystems"] = json!([{
            "id": "hp-1", "type": "heat_pump_air",
            "cop": 0.0, "coverageFraction": 1.25
        }]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        for code in ["heat_pump_cop_invalid", "heat_pump_coverage_invalid"] {
            assert!(result.issues.iter().any(|issue| issue.code == code));
        }
    }

    #[test]
    fn audits_hot_water_heat_pump_sink() {
        let mut value = sample();
        value["hotWaterSystems"] = json!([{
            "id": "dhw-1", "type": "heat_pump", "efficiency": 2.8,
            "ntaHeatPump": {
                "id": "dhw-1", "source": "exhaust_air", "sink": "indoor_air",
                "drive": "electric_compression", "reversible": false,
                "hybrid": false, "booster": false,
                "performanceEvidence": {"kind": "normative_default", "reference": null}
            }
        }]);
        let invalid = assess_json(value.clone()).unwrap();
        assert_eq!(invalid.status, "invalid");
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "hot_water_heat_pump_sink_mismatch"));

        value["hotWaterSystems"][0]["ntaHeatPump"]["sink"] = json!("domestic_hot_water");
        let valid = assess_json(value).unwrap();
        assert_eq!(valid.status, "structurally_valid");
        assert_eq!(valid.summary.classified_heat_pump_count, 1);
        assert!(valid
            .issues
            .iter()
            .any(|issue| issue.code == "exhaust_air_ventilation_link_missing"));
    }

    #[test]
    fn rejects_domestic_hot_water_only_sink_on_space_heating_system() {
        let mut value = sample();
        value["heatingSystems"] = json!([{
            "id": "hp-1", "type": "heat_pump_air",
            "ntaHeatPump": {
                "id": "hp-1", "source": "outdoor_air", "sink": "domestic_hot_water",
                "drive": "electric_compression", "reversible": false,
                "hybrid": false, "booster": false,
                "performanceEvidence": {"kind": "normative_default", "reference": null}
            }
        }]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heating_heat_pump_sink_mismatch"));
    }

    #[test]
    fn accepts_all_classified_source_categories_as_unimplemented_assets() {
        let mut value = sample();
        let sources = [
            "outdoor_air",
            "exhaust_air",
            "ground",
            "groundwater",
            "surface_water",
            "district_water",
            "waste_heat",
            "other",
        ];
        value["ntaHeatPumps"] = Value::Array(
            sources
                .iter()
                .enumerate()
                .map(|(index, source)| {
                    json!({
                        "id": format!("asset-{index}"), "servedZoneIds": ["z1"],
                        "source": source, "sink": "hydronic", "drive": "electric_compression",
                        "reversible": false, "hybrid": false, "booster": false,
                        "performanceEvidence": {"kind": "normative_default", "reference": null}
                    })
                })
                .collect(),
        );
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert_eq!(result.summary.heat_pump_count, 8);
        assert_eq!(result.summary.classified_heat_pump_count, 8);
        assert_eq!(
            result
                .issues
                .iter()
                .filter(|item| item.code == "heat_pump_route_unimplemented")
                .count(),
            8
        );
        assert!(!result.calculation_available);
    }

    #[test]
    fn saved_residential_forfait_class_is_rejected_in_utility_project() {
        let mut value = sample();
        value["buildingFunction"] = json!("office");
        value["ntaHeatPumps"] = json!([{
            "id":"hp-table", "source":"outdoor_air", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false, "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "forfaitHeatPumpDraft":{
                "generatorId":"hp-table", "classificationSourceReference":"design sheet",
                "scope":"residential_at_most25_kw", "source":"outdoor_air", "sink":"hydronic",
                "designSupplyTemperatureC":35.0, "sourceCorrectionFactor":null,
                "sourceCorrectionReference":null
            }
        }]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "forfait_heat_pump_draft_building_scope_invalid"));
    }

    #[test]
    fn saved_gas_table_input_is_audited_against_building_function() {
        let mut value = sample();
        value["ntaHeatPumps"] = json!([{
            "id":"gwp-1", "source":"outdoor_air", "sink":"hydronic",
            "drive":"gas_engine", "reversible":false, "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "gasHeatPumpForfaitDraft":{
                "generatorId":"gwp-1", "drive":"gas_engine", "application":"utility",
                "applicationReference":"office schedule", "collectiveBuildingInstallation":false,
                "externalHeatSupply":false, "thermalCapacityKw":20.0, "capacityReference":"plate",
                "source":"outdoor_air", "sourceReference":"system design",
                "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design"
            }
        }]);
        let residential = assess_json(value.clone()).unwrap();
        assert_eq!(residential.status, "invalid");
        assert!(residential
            .issues
            .iter()
            .any(|item| item.code == "gas_heat_pump_forfait_draft_building_scope_invalid"));
        value["buildingFunction"] = json!("office");
        value["ntaHeatPumps"][0]["gasHeatPumpAuxDraft"] = json!({
            "generatorId":"gwp-1", "drive":"gas_engine", "nominalThermalCapacityKw":20.0,
            "capacityReference":"plate", "standbyElectronicsW":10.0,
            "burnerAuxiliaryWPerKw":1.0, "solutionPumpWPerKw":0.0,
            "coefficientsReference":"draft 9.6.8.2.3", "meanModulation":1.0,
            "modulationReference":"draft 9.6.8.2.3", "buildingShare":1.0,
            "buildingShareReference":"whole building", "forfaitCopUsed":true,
            "monthHoursReference":"hour schedule", "generatorOutputReference":"generator ledger",
            "months":(1..=12).map(|month| json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0})).collect::<Vec<_>>()
        });
        let utility = assess_json(value).unwrap();
        assert_eq!(utility.status, "structurally_valid");
        assert!(utility
            .issues
            .iter()
            .any(|item| item.code == "gas_heat_pump_forfait_draft_unverified"));
        assert!(utility
            .issues
            .iter()
            .any(|item| item.code == "gas_heat_pump_aux_draft_unverified"));
        assert!(!utility.calculation_available);
    }

    #[test]
    fn saved_residential_collective_gas_table_requires_residential_project() {
        let mut value = sample();
        value["ntaHeatPumps"] = json!([{
            "id":"gwp-home", "source":"ground", "sink":"hydronic",
            "drive":"absorption", "reversible":false, "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "gasHeatPumpForfaitDraft":{
                "generatorId":"gwp-home", "drive":"absorption",
                "application":"residential_collective_at_most25_kw",
                "applicationReference":"collective schedule", "collectiveBuildingInstallation":true,
                "externalHeatSupply":false, "thermalCapacityKw":25.0, "capacityReference":"plate",
                "source":"ground", "sourceReference":"ground loop design",
                "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design",
                "sourceCorrectionFactor":1.0, "sourceCorrectionReference":"no regeneration; design record"
            }
        }]);
        let home = assess_json(value.clone()).unwrap();
        assert_eq!(home.status, "structurally_valid");
        assert!(!home.calculation_available);
        value["buildingFunction"] = json!("office");
        let office = assess_json(value).unwrap();
        assert_eq!(office.status, "invalid");
        assert!(office
            .issues
            .iter()
            .any(|item| item.code == "gas_heat_pump_forfait_draft_building_scope_invalid"));
    }

    #[test]
    fn saved_utility_forfait_class_requires_collective_or_over25_in_residential_project() {
        let mut value = sample();
        value["ntaHeatPumps"] = json!([{
            "id":"hp-table", "source":"outdoor_air", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false, "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "forfaitHeatPumpDraft":{
                "generatorId":"hp-table", "classificationSourceReference":"design sheet",
                "scope":"utility_collective_or_over25_kw", "source":"outdoor_air", "sink":"hydronic",
                "designSupplyTemperatureC":35.0, "sourceCorrectionFactor":null,
                "sourceCorrectionReference":null, "thermalCapacityKw":25.0,
                "capacitySourceReference":"manufacturer sheet", "collectiveBuildingInstallation":false
            }
        }]);
        let result = assess_json(value.clone()).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "forfait_heat_pump_draft_building_scope_invalid"));
        value["ntaHeatPumps"][0]["forfaitHeatPumpDraft"]["thermalCapacityKw"] = json!(25.01);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(!result.calculation_available);
    }

    #[test]
    fn saved_collective_source_without_declaration_warns_about_table_fallback() {
        let mut value = sample();
        value["ntaHeatPumps"] = json!([{
            "id":"hp-collective", "source":"district_water", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false, "hybrid":false,
            "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "forfaitHeatPumpDraft":{
                "generatorId":"hp-collective", "classificationSourceReference":"design sheet",
                "scope":"residential_at_most25_kw", "source":"collective20_to40_c",
                "sink":"hydronic", "designSupplyTemperatureC":35.0,
                "sourceTemperatureC":20.0,
                "sourceTemperatureEvidenceReference":"source temperature design",
                "sourceCorrectionFactor":1.0,
                "sourceCorrectionReference":"source correction design",
                "thermalCapacityKw":8.0,
                "capacitySourceReference":"manufacturer sheet",
                "collectiveBuildingInstallation":false
            }
        }]);
        let result = assess_json(value.clone()).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "forfait_heat_pump_draft_source_fallback"));
        value["ntaHeatPumps"][0]["forfaitHeatPumpDraft"]["sourceQualityDeclarationReference"] =
            json!("source QD-1");
        value["ntaHeatPumps"][0]["forfaitHeatPumpDraft"]["sourceCorrectionFactor"] = Value::Null;
        value["ntaHeatPumps"][0]["forfaitHeatPumpDraft"]["sourceCorrectionReference"] = Value::Null;
        let declared = assess_json(value).unwrap();
        assert_eq!(declared.status, "structurally_valid");
        assert!(!declared
            .issues
            .iter()
            .any(|item| item.code == "forfait_heat_pump_draft_source_fallback"));
    }

    #[test]
    fn saved_unknown_groundwater_temperature_uses_explicit_ground_fallback() {
        let mut value = sample();
        value["ntaHeatPumps"] = json!([{
            "id":"hp-unknown-temp", "source":"groundwater", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false, "hybrid":false,
            "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "forfaitHeatPumpDraft":{
                "generatorId":"hp-unknown-temp",
                "classificationSourceReference":"survey: source temperature unknown",
                "scope":"residential_at_most25_kw",
                "source":"ground_or_groundwater_unknown", "sink":"hydronic",
                "designSupplyTemperatureC":35.0,
                "sourceCorrectionFactor":1.0,
                "sourceCorrectionReference":"no regeneration, design sheet",
                "thermalCapacityKw":8.0,
                "capacitySourceReference":"manufacturer sheet",
                "collectiveBuildingInstallation":false
            }
        }]);
        let result = assess_json(value.clone()).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(result
            .issues
            .iter()
            .any(|item| { item.code == "forfait_heat_pump_draft_unknown_ground_source_fallback" }));
        value["ntaHeatPumps"][0]["source"] = json!("outdoor_air");
        let mismatched = assess_json(value).unwrap();
        assert_eq!(mismatched.status, "invalid");
        assert!(mismatched
            .issues
            .iter()
            .any(|item| item.code == "forfait_heat_pump_draft_scope_invalid"));
    }

    #[test]
    fn audits_explicit_heat_pump_links_without_inventing_dispatch() {
        let mut value = sample();
        value["heatingSystems"] = json!([{"id":"boiler-1", "type":"hr107"}]);
        value["ntaHeatPumps"] = json!([
            {"id":"hp-a", "servedZoneIds":["z1"], "source":"outdoor_air", "sink":"hydronic",
             "drive":"electric_compression", "reversible":false, "hybrid":true, "booster":false,
             "performanceEvidence":{"kind":"normative_default", "reference":null},
             "systemLinks":[
                {"id":"backup", "role":"backup_generator", "targetKind":"heating_system",
                 "targetId":"boiler-1", "evidenceReference":"scheme-1"},
                {"id":"cascade", "role":"upstream_heat_pump", "targetKind":"heat_pump",
                 "targetId":"hp-b", "evidenceReference":"scheme-1"}
             ]},
            {"id":"hp-b", "servedZoneIds":["z1"], "source":"ground", "sink":"hydronic",
             "drive":"electric_compression", "reversible":false, "hybrid":false, "booster":false,
             "performanceEvidence":{"kind":"normative_default", "reference":null}}
        ]);
        let valid = assess_json(value.clone()).unwrap();
        assert_eq!(valid.status, "structurally_valid");
        assert_eq!(valid.summary.system_link_count, 2);
        assert!(!valid.calculation_available);

        value["ntaHeatPumps"][0]["systemLinks"][0]["targetId"] = json!("missing");
        value["ntaHeatPumps"][1]["systemLinks"] = json!([{
            "id":"back", "role":"upstream_heat_pump", "targetKind":"heat_pump",
            "targetId":"hp-a", "evidenceReference":"scheme-1"
        }]);
        let invalid = assess_json(value).unwrap();
        assert_eq!(invalid.status, "invalid");
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "system_link_target_missing"));
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "system_link_cycle"));
    }

    #[test]
    fn exhaust_air_source_link_targets_an_existing_ventilation_system() {
        let mut value = sample();
        value["ventilationSystems"] = json!([{"id":"vent-1","type":"type_d"}]);
        value["ntaHeatPumps"] = json!([{
            "id":"hp-exhaust", "servedZoneIds":["z1"], "source":"exhaust_air",
            "sink":"hydronic", "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default","reference":null}
        }]);
        let missing = assess_json(value.clone()).unwrap();
        assert_eq!(missing.status, "structurally_valid");
        assert!(missing
            .issues
            .iter()
            .any(|issue| issue.code == "exhaust_air_ventilation_link_missing"));

        value["ntaHeatPumps"][0]["systemLinks"] = json!([{
            "id":"source-1", "role":"source_ventilation",
            "targetKind":"ventilation_system", "targetId":"vent-1",
            "evidenceReference":"ventilation scheme 1"
        }]);
        let linked = assess_json(value.clone()).unwrap();
        assert_eq!(linked.status, "structurally_valid");
        assert_eq!(linked.summary.system_link_count, 1);
        assert!(!linked
            .issues
            .iter()
            .any(|issue| issue.code == "exhaust_air_ventilation_link_missing"));

        value["ntaHeatPumps"][0]["systemLinks"][0]["targetId"] = json!("unknown");
        let unknown = assess_json(value.clone()).unwrap();
        assert_eq!(unknown.status, "invalid");
        assert!(unknown
            .issues
            .iter()
            .any(|issue| issue.code == "system_link_target_missing"));

        value["ntaHeatPumps"][0]["systemLinks"][0]["targetId"] = json!("vent-1");
        value["ntaHeatPumps"][0]["systemLinks"][0]["targetKind"] = json!("heat_pump");
        let wrong_kind = assess_json(value).unwrap();
        assert_eq!(wrong_kind.status, "invalid");
        assert!(wrong_kind
            .issues
            .iter()
            .any(|issue| issue.code == "source_ventilation_target_kind_invalid"));
    }

    #[test]
    fn rejects_malformed_and_duplicate_system_records_before_source_matching() {
        let mut value = sample();
        value["ventilationSystems"] = json!([
            null,
            {"id":"vent-1"},
            {"id":"vent-1", "type":"type_d"}
        ]);
        value["ntaHeatPumps"] = json!([{
            "id":"hp", "servedZoneIds":["z1"], "source":"exhaust_air",
            "sink":"hydronic", "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default","reference":null},
            "systemLinks":[{"id":"source", "role":"source_ventilation",
                "targetKind":"ventilation_system", "targetId":"vent-1",
                "evidenceReference":"drawing"}]
        }]);
        let audit = assess_json(value).unwrap();
        assert_eq!(audit.status, "invalid");
        for code in [
            "system_record_invalid",
            "system_type_required",
            "system_id_invalid",
        ] {
            assert!(
                audit.issues.iter().any(|issue| issue.code == code),
                "{code}"
            );
        }
        assert!(!audit.calculation_available);
    }

    #[test]
    fn rejects_duplicate_asset_id_and_unknown_served_zone() {
        let mut value = sample();
        let asset = json!({
            "id": "same", "servedZoneIds": ["missing-zone"],
            "source": "surface_water", "sink": "hydronic", "drive": "electric_compression",
            "reversible": true, "hybrid": false, "booster": false,
            "performanceEvidence": {"kind": "normative_default", "reference": null}
        });
        value["ntaHeatPumps"] = json!([asset.clone(), asset]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_id_duplicate"));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_zone_missing"));
    }

    #[test]
    fn audits_referenced_performance_point_without_using_it_as_a_calculation() {
        let mut value = sample();
        value["ntaHeatPumps"] = json!([{
            "id": "hp", "servedZoneIds": ["z1"], "source": "groundwater",
            "sink": "hydronic", "drive": "electric_compression",
            "reversible": false, "hybrid": false, "booster": false,
            "performanceEvidence": {"kind": "normative_default", "reference": null},
            "performancePoints": [{
                "id": "p1", "service": "space_heating",
                "sourceTemperatureC": 10.0, "sinkTemperatureC": 35.0,
                "usefulCapacityKw": 5.0, "inputPowerKw": 1.5,
                "inputEnergyCarrier": "electricity", "testReference": "lab-report-1"
            }]
        }]);
        let result = assess_json(value.clone()).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(!result.calculation_available);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].code, "heat_pump_route_unimplemented");
        assert_eq!(result.summary.performance_point_diagnostics.len(), 1);
        let point = &result.summary.performance_point_diagnostics[0];
        assert_eq!(point.heat_pump_path, "ntaHeatPumps[0]");
        assert_eq!(point.point_id, "p1");
        assert!((point.instantaneous_useful_to_input_ratio - 10.0 / 3.0).abs() < 1e-12);
        assert!(!point.reference_verified);
        assert!(!point.annual_performance_available);

        value["ntaHeatPumps"][0]["performancePoints"][0]["usefulCapacityKw"] = json!(0);
        let invalid = assess_json(value).unwrap();
        assert_eq!(invalid.status, "invalid");
        assert!(invalid.summary.performance_point_diagnostics.is_empty());
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "performance_point_power_invalid"
                && item.path == "ntaHeatPumps[0].performancePoints[0].usefulCapacityKw"));
    }

    #[test]
    fn flags_embedded_points_as_unimplemented_for_nta_calculation() {
        let mut value = sample();
        value["heatingSystems"] = json!([{
            "id": "hp", "type": "heat_pump_air", "cop": 3.5, "coverageFraction": 1.0,
            "ntaHeatPump": {
                "id": "hp", "source": "outdoor_air", "sink": "hydronic",
                "drive": "electric_compression", "reversible": false,
                "hybrid": false, "booster": false,
                "performanceEvidence": {"kind": "normative_default", "reference": null},
                "performancePoints": [{
                    "id": "p1", "service": "space_heating",
                    "sourceTemperatureC": 7, "sinkTemperatureC": 35,
                    "usefulCapacityKw": 5, "inputPowerKw": 1.5,
                    "inputEnergyCarrier": "electricity", "testReference": "lab-1"
                }]
            }
        }]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "structurally_valid");
        assert!(!result.calculation_available);
        assert!(result.issues.iter().any(|item| item.code
            == "heat_pump_performance_points_unimplemented"
            && item.path == "heatingSystems[0].ntaHeatPump.performancePoints"));
    }

    #[test]
    fn rejects_misspelled_classified_heat_pump_input_in_both_storage_forms() {
        let pump = json!({
            "id": "hp", "source": "outdoor_air", "sink": "hydronic",
            "drive": "electric_compression", "reversible": false,
            "hybrid": false, "booster": false,
            "performanceEvidence": {"kind": "normative_default", "reference": null},
            "performancePoint": []
        });
        let mut standalone = sample();
        standalone["ntaHeatPumps"] = json!([pump.clone()]);
        assert!(assess_json(standalone)
            .unwrap_err()
            .contains("performancePoint"));

        let mut embedded = sample();
        embedded["heatingSystems"] = json!([{
            "id": "hp", "type": "heat_pump_air", "cop": 3.0,
            "coverageFraction": 1.0, "ntaHeatPump": pump
        }]);
        let result = assess_json(embedded).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_metadata_invalid"
                && item.path == "heatingSystems[0].ntaHeatPump"
                && item
                    .detail
                    .as_deref()
                    .is_some_and(|text| text.contains("performancePoint"))));
    }

    #[test]
    fn audits_zone_links_for_embedded_and_standalone_heat_pumps() {
        let mut value = sample();
        let pump = json!({
            "id": "hp-1", "servedZoneIds": ["z1", "z1", "unknown"],
            "source": "outdoor_air", "sink": "hydronic", "drive": "electric_compression",
            "reversible": false, "hybrid": false, "booster": false,
            "performanceEvidence": {"kind": "normative_default", "reference": null}
        });
        value["heatingSystems"] = json!([{
            "id": "hp-1", "type": "heat_pump_air", "ntaHeatPump": pump
        }]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_zone_duplicate"
                && item.path == "heatingSystems[0].ntaHeatPump.servedZoneIds[1]"));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_zone_missing"
                && item.path == "heatingSystems[0].ntaHeatPump.servedZoneIds[2]"));
    }

    #[test]
    fn rejects_heat_pump_metadata_on_unrelated_systems() {
        let mut value = sample();
        value["heatingSystems"] =
            json!([{"id": "boiler", "type": "gas_boiler", "ntaHeatPump": {}}]);
        value["hotWaterSystems"] = json!([{"id": "dhw", "type": "gas_boiler", "ntaHeatPump": {}}]);
        let result = assess_json(value).unwrap();
        assert_eq!(result.status, "invalid");
        assert_eq!(
            result
                .issues
                .iter()
                .filter(|item| item.code == "heat_pump_metadata_on_other_system")
                .count(),
            2
        );
    }
}
