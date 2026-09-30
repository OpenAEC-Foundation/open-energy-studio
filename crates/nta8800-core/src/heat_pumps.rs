//! Input taxonomy for heat pump routes. This describes equipment and evidence;
//! it does not imply that an NTA 8800 calculation route is implemented.

use crate::forfait_heat_pump_draft::{
    assess_forfait_heat_pump_draft, ForfaitHeatPumpDraftInput, TableSink, TableSource,
};
use crate::gas_heat_pump_aux_draft::{assess_gas_heat_pump_aux_draft, GasHeatPumpAuxDraftInput};
use crate::gas_heat_pump_forfait_draft::{
    assess_gas_heat_pump_forfait_draft, GasHeatPumpForfaitDraftInput, GasPumpDrive, GasPumpSource,
};
use crate::heating_aux_draft::{assess_heating_aux_measured_draft, HeatingAuxMeasuredDraftInput};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatSource {
    OutdoorAir,
    ExhaustAir,
    Ground,
    Groundwater,
    SurfaceWater,
    DistrictWater,
    WasteHeat,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatSink {
    IndoorAir,
    Hydronic,
    DomesticHotWater,
    CombinedHydronicAndHotWater,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatPumpDrive {
    ElectricCompression,
    GasEngine,
    Absorption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceEvidenceKind {
    NormativeDefault,
    ControlledQualityDeclaration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformanceEvidence {
    pub kind: PerformanceEvidenceKind,
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry_record: Option<DeclarationRegistryRecord>,
}

/// User-entered identity of a controlled declaration. Presence does not
/// authenticate the registry entry or establish applicability to the project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclarationRegistryRecord {
    pub registration_number: String,
    pub product_name: String,
    pub manufacturer: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceService {
    SpaceHeating,
    DomesticHotWater,
    SpaceCooling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputEnergyCarrier {
    Electricity,
    Gas,
    DistrictHeat,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuxiliaryKind {
    SourcePump,
    SourceFan,
    IndoorFan,
    DistributionPump,
    ControlsStandby,
    Defrost,
    BackupHeater,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuxiliaryMeasurementBoundary {
    IncludedInDeclaredPerformance,
    Additional,
    Unknown,
}

/// Declared auxiliary device at a stated rated operating point. Annual run
/// hours, part load and NTA weighting are not inferred from this record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuxiliaryComponent {
    pub id: String,
    pub kind: AuxiliaryKind,
    pub service: PerformanceService,
    pub nominal_power_w: f64,
    pub energy_carrier: InputEnergyCarrier,
    pub measurement_boundary: AuxiliaryMeasurementBoundary,
    pub evidence_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemLinkRole {
    BackupGenerator,
    UpstreamHeatPump,
    SharedSource,
    SourceVentilation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemLinkTargetKind {
    HeatPump,
    HeatingSystem,
    HotWaterSystem,
    VentilationSystem,
}

/// Explicit equipment relationship. Dispatch and energy shares are not inferred.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemLink {
    pub id: String,
    pub role: SystemLinkRole,
    pub target_kind: SystemLinkTargetKind,
    pub target_id: String,
    pub evidence_reference: String,
}

/// An observed or declared operating point. No NTA interpolation or weighting is implied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformancePoint {
    pub id: String,
    pub service: PerformanceService,
    pub source_temperature_c: f64,
    pub sink_temperature_c: f64,
    pub useful_capacity_kw: f64,
    pub input_power_kw: f64,
    pub input_energy_carrier: InputEnergyCarrier,
    pub test_reference: String,
}

/// Declared EN 16147 tap-profile input, not an NTA 8800 practice efficiency.
/// A declaration can use a different norm edition from this kernel's target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DhwTestPoint {
    pub id: String,
    pub tap_profile: String,
    pub useful_energy_kwh_per_day: f64,
    pub input_energy_kwh_per_day: f64,
    pub nominal_capacity_kw: f64,
    pub practice_factor: f64,
    pub test_setpoint_c: f64,
    pub design_setpoint_c: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_air_flow_m3_per_hour: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_air_dry_bulb_c: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_air_wet_bulb_c: Option<f64>,
    pub declaration_norm_version: String,
    pub source_reference: String,
}

/// Product-specific shutoff thresholds, not an NTA dispatch or backup calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredOperatingLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_operating_cop: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum_supply_temperature_c: Option<f64>,
    pub declaration_norm_version: String,
    pub source_reference: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatPumpInput {
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub served_zone_ids: Vec<String>,
    pub source: HeatSource,
    pub sink: HeatSink,
    pub drive: HeatPumpDrive,
    pub reversible: bool,
    pub hybrid: bool,
    pub booster: bool,
    pub performance_evidence: PerformanceEvidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub performance_points: Vec<PerformancePoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dhw_test_points: Vec<DhwTestPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_operating_limits: Option<DeclaredOperatingLimits>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub auxiliary_components: Vec<AuxiliaryComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heating_aux_measured_draft: Option<HeatingAuxMeasuredDraftInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forfait_heat_pump_draft: Option<ForfaitHeatPumpDraftInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_heat_pump_forfait_draft: Option<GasHeatPumpForfaitDraftInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_heat_pump_aux_draft: Option<GasHeatPumpAuxDraftInput>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub system_links: Vec<SystemLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatPumpIssue {
    pub code: &'static str,
    pub field: String,
    pub message: &'static str,
}

impl HeatPumpInput {
    pub fn validate_inputs(&self) -> Vec<HeatPumpIssue> {
        let mut issues = Vec::new();
        if self.id.trim().is_empty() {
            issues.push(HeatPumpIssue {
                code: "heat_pump_id_required",
                field: "id".into(),
                message: "Heat pump ID is required",
            });
        }
        if let Some(input) = &self.heating_aux_measured_draft {
            if input.generator_id != self.id
                || self.drive != HeatPumpDrive::ElectricCompression
                || self.sink != HeatSink::Hydronic
                || self.hybrid
                || self.booster
            {
                issues.push(HeatPumpIssue {
                    code: "heating_aux_measured_draft_scope_invalid",
                    field: "heatingAuxMeasuredDraft".into(),
                    message: "Measured auxiliary draft requires the same individual electric hydronic generator",
                });
            }
            for item in assess_heating_aux_measured_draft(input).issues {
                issues.push(HeatPumpIssue {
                    code: "heating_aux_measured_draft_input_invalid",
                    field: format!("heatingAuxMeasuredDraft.{}", item.path),
                    message: "Measured auxiliary draft input is invalid",
                });
            }
        }
        if let Some(input) = &self.forfait_heat_pump_draft {
            let sink_matches = matches!(
                (self.sink, input.sink),
                (HeatSink::Hydronic, TableSink::Hydronic)
                    | (HeatSink::IndoorAir, TableSink::IndoorAir)
            );
            let source_matches = match input.source {
                TableSource::Ground => self.source == HeatSource::Ground,
                TableSource::GroundOrGroundwaterUnknown => {
                    matches!(self.source, HeatSource::Groundwater | HeatSource::Other)
                }
                TableSource::GroundwaterBelow15C => self.source == HeatSource::Groundwater,
                TableSource::OutdoorAir => self.source == HeatSource::OutdoorAir,
                TableSource::ExhaustAir => self.source == HeatSource::ExhaustAir,
                TableSource::SurfaceWater => self.source == HeatSource::SurfaceWater,
                TableSource::Collective15To20C
                | TableSource::Collective20To40C
                | TableSource::CollectiveAtLeast40C => {
                    !matches!(self.source, HeatSource::OutdoorAir | HeatSource::ExhaustAir)
                }
            };
            if input.generator_id != self.id
                || self.drive != HeatPumpDrive::ElectricCompression
                || self.hybrid
                || self.booster
                || !sink_matches
                || !source_matches
            {
                issues.push(HeatPumpIssue {
                    code: "forfait_heat_pump_draft_scope_invalid",
                    field: "forfaitHeatPumpDraft".into(),
                    message: "Forfait COP draft classification conflicts with this individual electric generator",
                });
            }
            for item in assess_forfait_heat_pump_draft(input).issues {
                issues.push(HeatPumpIssue {
                    code: "forfait_heat_pump_draft_input_invalid",
                    field: format!("forfaitHeatPumpDraft.{}", item.path),
                    message: "Forfait COP draft input is invalid or outside the represented table",
                });
            }
        }
        if let Some(input) = &self.gas_heat_pump_forfait_draft {
            let drive_matches = matches!(
                (self.drive, input.drive),
                (HeatPumpDrive::GasEngine, GasPumpDrive::GasEngine)
                    | (HeatPumpDrive::Absorption, GasPumpDrive::Absorption)
            );
            let source_matches = matches!(
                (self.source, input.source),
                (HeatSource::Ground, GasPumpSource::Ground)
                    | (HeatSource::OutdoorAir, GasPumpSource::OutdoorAir)
                    | (HeatSource::ExhaustAir, GasPumpSource::ExhaustAir)
                    | (HeatSource::Groundwater, GasPumpSource::GroundwaterAquifer)
                    | (HeatSource::SurfaceWater, GasPumpSource::SurfaceWater)
            );
            if input.generator_id != self.id
                || !drive_matches
                || !source_matches
                || self.sink != HeatSink::Hydronic
                || self.booster
            {
                issues.push(HeatPumpIssue {
                    code: "gas_heat_pump_forfait_draft_scope_invalid",
                    field: "gasHeatPumpForfaitDraft".into(),
                    message: "Gas COP draft classification conflicts with this hydronic generator",
                });
            }
            for item in assess_gas_heat_pump_forfait_draft(input).issues {
                issues.push(HeatPumpIssue {
                    code: "gas_heat_pump_forfait_draft_input_invalid",
                    field: format!("gasHeatPumpForfaitDraft.{}", item.path),
                    message: "Gas COP draft input is invalid or outside the represented table",
                });
            }
        }
        if let Some(input) = &self.gas_heat_pump_aux_draft {
            let drive_matches = matches!(
                (self.drive, input.drive),
                (HeatPumpDrive::GasEngine, GasPumpDrive::GasEngine)
                    | (HeatPumpDrive::Absorption, GasPumpDrive::Absorption)
            );
            if input.generator_id != self.id
                || !drive_matches
                || self.sink != HeatSink::Hydronic
                || self.booster
            {
                issues.push(HeatPumpIssue {
                    code: "gas_heat_pump_aux_draft_scope_invalid",
                    field: "gasHeatPumpAuxDraft".into(),
                    message: "Gas auxiliary draft input conflicts with this hydronic generator",
                });
            }
            if input.forfait_cop_used
                && self
                    .gas_heat_pump_forfait_draft
                    .as_ref()
                    .map_or(true, |forfait| {
                        forfait.thermal_capacity_kw != input.nominal_thermal_capacity_kw
                    })
            {
                issues.push(HeatPumpIssue {
                    code: "gas_heat_pump_aux_draft_forfait_mismatch",
                    field: "gasHeatPumpAuxDraft.nominalThermalCapacityKw".into(),
                    message: "Forfait auxiliary input requires a matching saved gas COP table selection and capacity",
                });
            }
            for item in assess_gas_heat_pump_aux_draft(input).issues {
                issues.push(HeatPumpIssue {
                    code: "gas_heat_pump_aux_draft_input_invalid",
                    field: format!("gasHeatPumpAuxDraft.{}", item.path),
                    message: "Gas auxiliary draft input is invalid or outside its diagnostic scope",
                });
            }
        }
        if self.performance_evidence.kind == PerformanceEvidenceKind::ControlledQualityDeclaration
            && self
                .performance_evidence
                .reference
                .as_ref()
                .map_or(true, |reference| reference.trim().is_empty())
        {
            issues.push(HeatPumpIssue {
                code: "quality_declaration_reference_required",
                field: "performanceEvidence.reference".into(),
                message: "A controlled quality declaration requires a traceable reference",
            });
        }
        if self.performance_evidence.kind == PerformanceEvidenceKind::NormativeDefault
            && self.performance_evidence.registry_record.is_some()
        {
            issues.push(HeatPumpIssue {
                code: "quality_declaration_record_without_declaration",
                field: "performanceEvidence.registryRecord".into(),
                message: "A registry record belongs to a controlled declaration, not a normative default",
            });
        }
        if let Some(record) = &self.performance_evidence.registry_record {
            for (field, value) in [
                ("registrationNumber", &record.registration_number),
                ("productName", &record.product_name),
                ("manufacturer", &record.manufacturer),
            ] {
                if value.trim().is_empty() {
                    issues.push(HeatPumpIssue {
                        code: "quality_declaration_registry_field_required",
                        field: format!("performanceEvidence.registryRecord.{field}"),
                        message: "Registry identity fields must be nonempty",
                    });
                }
            }
            if record
                .source_url
                .strip_prefix("https://")
                .map_or(true, |rest| {
                    rest.split('/').next().map_or(true, str::is_empty)
                })
                || record.source_url.chars().any(char::is_whitespace)
            {
                issues.push(HeatPumpIssue {
                    code: "quality_declaration_source_url_invalid",
                    field: "performanceEvidence.registryRecord.sourceUrl".into(),
                    message: "Registry source URL must be an HTTPS URL without whitespace",
                });
            }
        }
        let mut point_ids = HashSet::new();
        for (index, point) in self.performance_points.iter().enumerate() {
            let path = format!("performancePoints[{index}]");
            if point.id.trim().is_empty() || !point_ids.insert(point.id.as_str()) {
                issues.push(HeatPumpIssue {
                    code: "performance_point_id_invalid",
                    field: format!("{path}.id"),
                    message: "Performance point IDs must be nonempty and unique per heat pump",
                });
            }
            if self.performance_points[..index].iter().any(|previous| {
                previous.service == point.service
                    && previous.source_temperature_c == point.source_temperature_c
                    && previous.sink_temperature_c == point.sink_temperature_c
                    && previous.useful_capacity_kw == point.useful_capacity_kw
                    && previous.input_energy_carrier == point.input_energy_carrier
            }) {
                issues.push(HeatPumpIssue {
                    code: "performance_point_condition_duplicate",
                    field: format!("{path}.sourceTemperatureC"),
                    message: "Operating point conditions and useful capacity must be unique per service and input carrier",
                });
            }
            for (field, value) in [
                ("sourceTemperatureC", point.source_temperature_c),
                ("sinkTemperatureC", point.sink_temperature_c),
            ] {
                if !value.is_finite() {
                    issues.push(HeatPumpIssue {
                        code: "performance_point_temperature_invalid",
                        field: format!("{path}.{field}"),
                        message: "Performance point temperature must be finite",
                    });
                }
            }
            for (field, value) in [
                ("usefulCapacityKw", point.useful_capacity_kw),
                ("inputPowerKw", point.input_power_kw),
            ] {
                if !value.is_finite() || value <= 0.0 {
                    issues.push(HeatPumpIssue {
                        code: "performance_point_power_invalid",
                        field: format!("{path}.{field}"),
                        message: "Performance point powers must be finite and greater than zero",
                    });
                }
            }
            if point.useful_capacity_kw.is_finite()
                && point.useful_capacity_kw > 0.0
                && point.input_power_kw.is_finite()
                && point.input_power_kw > 0.0
                && !(point.useful_capacity_kw / point.input_power_kw).is_finite()
            {
                issues.push(HeatPumpIssue {
                    code: "performance_point_ratio_overflow",
                    field: format!("{path}.inputPowerKw"),
                    message: "Performance point useful-to-input power ratio must be finite",
                });
            }
            if point.test_reference.trim().is_empty() {
                issues.push(HeatPumpIssue {
                    code: "performance_point_reference_required",
                    field: format!("{path}.testReference"),
                    message: "Performance point requires a traceable test or declaration reference",
                });
            }
            if self.drive == HeatPumpDrive::ElectricCompression
                && point.input_energy_carrier != InputEnergyCarrier::Electricity
            {
                issues.push(HeatPumpIssue {
                    code: "performance_point_carrier_mismatch",
                    field: format!("{path}.inputEnergyCarrier"),
                    message: "Electric compression performance input must use electricity",
                });
            }
            if !self.supports_service(point.service) {
                issues.push(HeatPumpIssue {
                    code: "performance_point_service_mismatch",
                    field: format!("{path}.service"),
                    message: "Performance point service conflicts with heat pump sink or reversible flag",
                });
            }
        }
        let mut auxiliary_ids = HashSet::new();
        let mut dhw_ids = HashSet::new();
        let mut tap_profiles = HashSet::new();
        if !self.dhw_test_points.is_empty()
            && self.performance_evidence.kind
                != PerformanceEvidenceKind::ControlledQualityDeclaration
        {
            issues.push(HeatPumpIssue {
                code: "dhw_test_declaration_required",
                field: "dhwTestPoints".into(),
                message: "Declared DHW test points require controlled quality declaration evidence",
            });
        }
        for (index, point) in self.dhw_test_points.iter().enumerate() {
            let path = format!("dhwTestPoints[{index}]");
            if point.id.trim().is_empty() || !dhw_ids.insert(point.id.as_str()) {
                issues.push(HeatPumpIssue {
                    code: "dhw_test_id_invalid",
                    field: format!("{path}.id"),
                    message: "DHW test point IDs must be nonempty and unique",
                });
            }
            if point.tap_profile.trim().is_empty()
                || !tap_profiles.insert(point.tap_profile.as_str())
            {
                issues.push(HeatPumpIssue {
                    code: "dhw_tap_profile_invalid",
                    field: format!("{path}.tapProfile"),
                    message: "DHW tap profiles must be nonempty and unique per heat pump",
                });
            }
            if !matches!(
                self.sink,
                HeatSink::DomesticHotWater | HeatSink::CombinedHydronicAndHotWater
            ) {
                issues.push(HeatPumpIssue {
                    code: "dhw_test_sink_mismatch",
                    field: path.clone(),
                    message: "DHW test points require a domestic hot water sink",
                });
            }
            for (field, value) in [
                ("usefulEnergyKwhPerDay", point.useful_energy_kwh_per_day),
                ("inputEnergyKwhPerDay", point.input_energy_kwh_per_day),
                ("nominalCapacityKw", point.nominal_capacity_kw),
                ("practiceFactor", point.practice_factor),
            ] {
                if !value.is_finite() || value <= 0.0 {
                    issues.push(HeatPumpIssue { code: "dhw_test_value_invalid", field: format!("{path}.{field}"),
                        message: "DHW test energies, capacity and practice factor must be finite and positive" });
                }
            }
            if !point.test_setpoint_c.is_finite() || !point.design_setpoint_c.is_finite() {
                issues.push(HeatPumpIssue {
                    code: "dhw_test_temperature_invalid",
                    field: path.clone(),
                    message: "DHW test and design setpoints must be finite",
                });
            }
            if point
                .source_air_flow_m3_per_hour
                .is_some_and(|value| !value.is_finite() || value <= 0.0)
                || point
                    .source_air_dry_bulb_c
                    .is_some_and(|value| !value.is_finite())
                || point
                    .source_air_wet_bulb_c
                    .is_some_and(|value| !value.is_finite())
            {
                issues.push(HeatPumpIssue {
                    code: "dhw_test_source_air_invalid",
                    field: path.clone(),
                    message: "Declared source-air flow must be positive and temperatures finite",
                });
            }
            if point.declaration_norm_version.trim().is_empty()
                || point.source_reference.trim().is_empty()
            {
                issues.push(HeatPumpIssue {
                    code: "dhw_test_provenance_required",
                    field: path.clone(),
                    message: "DHW test point needs declaration norm version and traceable source",
                });
            }
            if point.useful_energy_kwh_per_day.is_finite()
                && point.input_energy_kwh_per_day > 0.0
                && !(point.useful_energy_kwh_per_day / point.input_energy_kwh_per_day).is_finite()
            {
                issues.push(HeatPumpIssue {
                    code: "dhw_test_ratio_overflow",
                    field: format!("{path}.inputEnergyKwhPerDay"),
                    message: "DHW declared test energy ratio must be finite",
                });
            }
        }
        if let Some(limits) = &self.declared_operating_limits {
            if self.performance_evidence.kind
                != PerformanceEvidenceKind::ControlledQualityDeclaration
            {
                issues.push(HeatPumpIssue {
                    code: "operating_limits_declaration_required",
                    field: "declaredOperatingLimits".into(),
                    message:
                        "Declared operating limits require controlled quality declaration evidence",
                });
            }
            if limits.minimum_operating_cop.is_none()
                && limits.maximum_supply_temperature_c.is_none()
            {
                issues.push(HeatPumpIssue {
                    code: "operating_limits_value_required",
                    field: "declaredOperatingLimits".into(),
                    message: "At least one declared operating limit is required",
                });
            }
            if limits
                .minimum_operating_cop
                .is_some_and(|value| !value.is_finite() || value <= 0.0)
            {
                issues.push(HeatPumpIssue {
                    code: "operating_limit_cop_invalid",
                    field: "declaredOperatingLimits.minimumOperatingCop".into(),
                    message: "Declared minimum operating COP must be finite and positive",
                });
            }
            if limits
                .maximum_supply_temperature_c
                .is_some_and(|value| !value.is_finite() || value <= 0.0)
            {
                issues.push(HeatPumpIssue {
                    code: "operating_limit_supply_temperature_invalid",
                    field: "declaredOperatingLimits.maximumSupplyTemperatureC".into(),
                    message: "Declared maximum supply temperature must be finite and positive",
                });
            }
            if limits.declaration_norm_version.trim().is_empty()
                || limits.source_reference.trim().is_empty()
            {
                issues.push(HeatPumpIssue {
                    code: "operating_limits_provenance_required",
                    field: "declaredOperatingLimits".into(),
                    message: "Declared operating limits require norm edition and source reference",
                });
            }
        }
        for (index, component) in self.auxiliary_components.iter().enumerate() {
            let path = format!("auxiliaryComponents[{index}]");
            if component.id.trim().is_empty() || !auxiliary_ids.insert(component.id.as_str()) {
                issues.push(HeatPumpIssue {
                    code: "auxiliary_id_invalid",
                    field: format!("{path}.id"),
                    message: "Auxiliary component IDs must be nonempty and unique per heat pump",
                });
            }
            if !component.nominal_power_w.is_finite() || component.nominal_power_w <= 0.0 {
                issues.push(HeatPumpIssue {
                    code: "auxiliary_power_invalid",
                    field: format!("{path}.nominalPowerW"),
                    message: "Auxiliary nominal power must be finite and positive",
                });
            }
            if component.evidence_reference.trim().is_empty() {
                issues.push(HeatPumpIssue {
                    code: "auxiliary_reference_required",
                    field: format!("{path}.evidenceReference"),
                    message: "Auxiliary component needs a traceable source reference",
                });
            }
            if !self.supports_service(component.service) {
                issues.push(HeatPumpIssue {
                    code: "auxiliary_service_mismatch",
                    field: format!("{path}.service"),
                    message: "Auxiliary component service conflicts with heat pump sink or reversible flag",
                });
            }
        }
        let mut link_ids = HashSet::new();
        for (index, link) in self.system_links.iter().enumerate() {
            let path = format!("systemLinks[{index}]");
            if link.id.trim().is_empty() || !link_ids.insert(link.id.as_str()) {
                issues.push(HeatPumpIssue {
                    code: "system_link_id_invalid",
                    field: format!("{path}.id"),
                    message: "System link IDs must be nonempty and unique per heat pump",
                });
            }
            if link.target_id.trim().is_empty() || link.target_id == self.id {
                issues.push(HeatPumpIssue {
                    code: "system_link_target_invalid",
                    field: format!("{path}.targetId"),
                    message: "System link target must be nonempty and cannot refer to itself",
                });
            }
            if link.evidence_reference.trim().is_empty() {
                issues.push(HeatPumpIssue {
                    code: "system_link_reference_required",
                    field: format!("{path}.evidenceReference"),
                    message: "System link requires a traceable source reference",
                });
            }
            if matches!(
                link.role,
                SystemLinkRole::UpstreamHeatPump | SystemLinkRole::SharedSource
            ) && link.target_kind != SystemLinkTargetKind::HeatPump
            {
                issues.push(HeatPumpIssue {
                    code: "system_link_target_kind_invalid",
                    field: format!("{path}.targetKind"),
                    message: "Upstream and shared-source links must target a heat pump",
                });
            }
            if link.role == SystemLinkRole::SourceVentilation
                && link.target_kind != SystemLinkTargetKind::VentilationSystem
            {
                issues.push(HeatPumpIssue {
                    code: "source_ventilation_target_kind_invalid",
                    field: format!("{path}.targetKind"),
                    message: "Source ventilation link must target a ventilation system",
                });
            }
            if link.target_kind == SystemLinkTargetKind::VentilationSystem
                && link.role != SystemLinkRole::SourceVentilation
            {
                issues.push(HeatPumpIssue {
                    code: "ventilation_link_role_invalid",
                    field: format!("{path}.role"),
                    message: "Ventilation systems can only be linked as a heat pump source",
                });
            }
            if link.role == SystemLinkRole::SourceVentilation
                && self.source != HeatSource::ExhaustAir
            {
                issues.push(HeatPumpIssue {
                    code: "source_ventilation_source_mismatch",
                    field: format!("{path}.role"),
                    message: "Source ventilation link requires an exhaust-air heat source",
                });
            }
        }
        issues
    }

    fn supports_service(&self, service: PerformanceService) -> bool {
        match service {
            PerformanceService::DomesticHotWater => matches!(
                self.sink,
                HeatSink::DomesticHotWater | HeatSink::CombinedHydronicAndHotWater
            ),
            PerformanceService::SpaceHeating => !matches!(self.sink, HeatSink::DomesticHotWater),
            PerformanceService::SpaceCooling => {
                self.reversible && !matches!(self.sink, HeatSink::DomesticHotWater)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_bcrg_dhw_inputs_remain_distinct_from_practice_efficiency() {
        let pump: HeatPumpInput = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20240123gk-dhw-test-input.json"
        ))
        .unwrap();
        assert!(pump.validate_inputs().is_empty());
        assert_eq!(pump.dhw_test_points.len(), 2);
        assert_eq!(pump.dhw_test_points[0].tap_profile, "M");
        assert!(
            (pump.dhw_test_points[0].useful_energy_kwh_per_day
                / pump.dhw_test_points[0].input_energy_kwh_per_day
                - 5.865 / 2.520)
                .abs()
                < 1e-12
        );
        assert_ne!(
            pump.dhw_test_points[0].useful_energy_kwh_per_day
                / pump.dhw_test_points[0].input_energy_kwh_per_day,
            2.095
        );
        assert_eq!(
            pump.dhw_test_points[0].declaration_norm_version,
            "NTA 8800:2020"
        );
    }

    #[test]
    fn recent_exhaust_air_declaration_preserves_source_air_conditions_without_attesting() {
        let pump: HeatPumpInput = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20260044gk-dhw-test-input.json"
        ))
        .unwrap();
        assert!(pump.validate_inputs().is_empty());
        assert_eq!(pump.source, HeatSource::ExhaustAir);
        assert_eq!(
            pump.dhw_test_points[0].source_air_flow_m3_per_hour,
            Some(159.0)
        );
        assert_eq!(
            pump.dhw_test_points[1].source_air_flow_m3_per_hour,
            Some(230.0)
        );
        assert_eq!(pump.dhw_test_points[0].source_air_dry_bulb_c, Some(20.0));
        assert_eq!(pump.dhw_test_points[0].source_air_wet_bulb_c, Some(12.0));
        assert_eq!(
            pump.dhw_test_points[0].declaration_norm_version,
            "NTA 8800:2024"
        );
        let mut invalid = pump;
        invalid.dhw_test_points[0].source_air_flow_m3_per_hour = Some(0.0);
        assert!(invalid
            .validate_inputs()
            .iter()
            .any(|issue| issue.code == "dhw_test_source_air_invalid"));
    }

    #[test]
    fn published_2025_water_water_dhw_input_is_not_the_practice_efficiency() {
        let pump: HeatPumpInput = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20260143gg-dhw-test-input.json"
        ))
        .unwrap();
        assert!(pump.validate_inputs().is_empty());
        assert_eq!(pump.source, HeatSource::Groundwater);
        assert_eq!(pump.sink, HeatSink::CombinedHydronicAndHotWater);
        assert_eq!(pump.dhw_test_points[0].tap_profile, "M");
        assert_eq!(pump.dhw_test_points[1].tap_profile, "L");
        assert_eq!(
            pump.dhw_test_points[0].declaration_norm_version,
            "NTA 8800:2025"
        );
        let raw_ratio = pump.dhw_test_points[0].useful_energy_kwh_per_day
            / pump.dhw_test_points[0].input_energy_kwh_per_day;
        assert!((raw_ratio - 3.395).abs() > 0.1);
    }

    #[test]
    fn hybrid_declaration_shutoff_limits_are_preserved_without_dispatch() {
        let mut pump: HeatPumpInput = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20240234gk-hybrid-limit-input.json"
        ))
        .unwrap();
        assert!(pump.validate_inputs().is_empty());
        assert!(pump.hybrid);
        let limits = pump.declared_operating_limits.as_ref().unwrap();
        assert_eq!(limits.minimum_operating_cop, Some(2.0));
        assert_eq!(limits.maximum_supply_temperature_c, Some(55.0));
        pump.declared_operating_limits
            .as_mut()
            .unwrap()
            .minimum_operating_cop = Some(0.0);
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|issue| issue.code == "operating_limit_cop_invalid"));
        pump.declared_operating_limits
            .as_mut()
            .unwrap()
            .minimum_operating_cop = Some(2.0);
        pump.performance_evidence.kind = PerformanceEvidenceKind::NormativeDefault;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|issue| issue.code == "operating_limits_declaration_required"));
    }

    #[test]
    fn dhw_test_points_reject_duplicate_profile_wrong_sink_and_malformed_values() {
        let mut pump: HeatPumpInput = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20240123gk-dhw-test-input.json"
        ))
        .unwrap();
        pump.sink = HeatSink::Hydronic;
        pump.dhw_test_points[1].tap_profile = "M".into();
        pump.dhw_test_points[0].input_energy_kwh_per_day = 0.0;
        let issues = pump.validate_inputs();
        for code in [
            "dhw_test_sink_mismatch",
            "dhw_tap_profile_invalid",
            "dhw_test_value_invalid",
        ] {
            assert!(
                issues.iter().any(|issue| issue.code == code),
                "missing {code}"
            );
        }
        pump.performance_evidence.kind = PerformanceEvidenceKind::NormativeDefault;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|issue| issue.code == "dhw_test_declaration_required"));
        let mut value = serde_json::to_value(pump).unwrap();
        value["dhwTestPoints"][0]["usefulEnergKwhPerDay"] =
            value["dhwTestPoints"][0]["usefulEnergyKwhPerDay"].take();
        assert!(serde_json::from_value::<HeatPumpInput>(value).is_err());
    }

    #[test]
    fn declaration_requires_traceable_reference() {
        let pump = HeatPumpInput {
            id: "hp-1".into(),
            served_zone_ids: Vec::new(),
            source: HeatSource::Groundwater,
            sink: HeatSink::CombinedHydronicAndHotWater,
            drive: HeatPumpDrive::ElectricCompression,
            reversible: true,
            hybrid: false,
            booster: false,
            performance_evidence: PerformanceEvidence {
                kind: PerformanceEvidenceKind::ControlledQualityDeclaration,
                reference: None,
                registry_record: None,
            },
            performance_points: Vec::new(),
            dhw_test_points: Vec::new(),
            declared_operating_limits: None,
            auxiliary_components: Vec::new(),
            heating_aux_measured_draft: None,
            forfait_heat_pump_draft: None,
            gas_heat_pump_forfait_draft: None,
            gas_heat_pump_aux_draft: None,
            system_links: Vec::new(),
        };
        assert_eq!(
            pump.validate_inputs()[0].code,
            "quality_declaration_reference_required"
        );
    }

    #[test]
    fn saved_measured_auxiliary_input_is_checked_against_generator_and_months() {
        let mut pump: HeatPumpInput = serde_json::from_value(serde_json::json!({
            "id": "hp-1", "source": "outdoor_air", "sink": "hydronic",
            "drive": "electric_compression", "reversible": false, "hybrid": false, "booster": false,
            "performanceEvidence": {"kind": "normative_default", "reference": null},
            "heatingAuxMeasuredDraft": {
                "generatorId": "hp-1", "generatorSourceReference": "schedule",
                "measurements": {
                    "standbyElectronicsW": 10.0, "deliveryPumpDuringCompressorW": 200.0,
                    "deliveryPumpPrePostW": 90.0, "pumpPreRunSeconds": 300.0,
                    "pumpPostRunSeconds": 300.0, "averageCompressorOnSeconds": 600.0,
                    "meanCompressorModulation": 0.5, "nominalElectricDriveKw": 2.0,
                    "measurementSourceReference": "meter", "timingSourceReference": "cycle"
                },
                "inputEnergySourceReference": "monthly meter",
                "months": (1..=12).map(|month| serde_json::json!({
                    "month": month, "generatorInputElectricityKwh": 100.0
                })).collect::<Vec<_>>()
            }
        }))
        .unwrap();
        assert!(pump.validate_inputs().is_empty());
        pump.heating_aux_measured_draft
            .as_mut()
            .unwrap()
            .generator_id = "other".into();
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "heating_aux_measured_draft_scope_invalid"));
        pump.heating_aux_measured_draft
            .as_mut()
            .unwrap()
            .generator_id = "hp-1".into();
        pump.heating_aux_measured_draft
            .as_mut()
            .unwrap()
            .months
            .pop();
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "heating_aux_measured_draft_input_invalid"));
    }

    #[test]
    fn saved_forfait_table_input_must_match_generator_and_source() {
        let mut pump: HeatPumpInput = serde_json::from_value(serde_json::json!({
            "id":"hp-table", "source":"outdoor_air", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false, "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "forfaitHeatPumpDraft":{
                "generatorId":"hp-table", "classificationSourceReference":"design sheet",
                "scope":"residential_at_most25_kw", "source":"outdoor_air", "sink":"hydronic",
                "designSupplyTemperatureC":35.0, "sourceCorrectionFactor":null,
                "sourceCorrectionReference":null
            }
        }))
        .unwrap();
        assert!(pump.validate_inputs().is_empty());
        pump.source = HeatSource::Ground;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "forfait_heat_pump_draft_scope_invalid"));
        pump.source = HeatSource::OutdoorAir;
        pump.forfait_heat_pump_draft
            .as_mut()
            .unwrap()
            .design_supply_temperature_c = Some(71.0);
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "forfait_heat_pump_draft_input_invalid"));
        pump.forfait_heat_pump_draft
            .as_mut()
            .unwrap()
            .design_supply_temperature_c = Some(35.0);
        pump.forfait_heat_pump_draft.as_mut().unwrap().source = TableSource::Collective15To20C;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "forfait_heat_pump_draft_scope_invalid"));
    }

    #[test]
    fn saved_gas_table_input_must_match_generator_drive_source_and_sink() {
        let mut pump: HeatPumpInput = serde_json::from_value(serde_json::json!({
            "id":"gwp-1", "source":"exhaust_air", "sink":"hydronic",
            "drive":"absorption", "reversible":false, "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "gasHeatPumpForfaitDraft":{
                "generatorId":"gwp-1", "drive":"absorption", "application":"utility",
                "applicationReference":"building schedule", "collectiveBuildingInstallation":false,
                "externalHeatSupply":false, "thermalCapacityKw":40.0, "capacityReference":"plate",
                "source":"exhaust_air", "sourceReference":"system design",
                "designSupplyTemperatureC":40.0, "designSupplyReference":"heating design"
            }
        }))
        .unwrap();
        assert!(pump.validate_inputs().is_empty());
        pump.drive = HeatPumpDrive::GasEngine;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "gas_heat_pump_forfait_draft_scope_invalid"));
        pump.drive = HeatPumpDrive::Absorption;
        pump.source = HeatSource::OutdoorAir;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "gas_heat_pump_forfait_draft_scope_invalid"));
        pump.source = HeatSource::ExhaustAir;
        pump.gas_heat_pump_forfait_draft
            .as_mut()
            .unwrap()
            .design_supply_temperature_c = 56.0;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "gas_heat_pump_forfait_draft_input_invalid"));
    }

    #[test]
    fn saved_gas_aux_months_require_matching_forfait_and_generator() {
        let mut pump: HeatPumpInput = serde_json::from_value(serde_json::json!({
            "id":"ga-1", "source":"outdoor_air", "sink":"hydronic", "drive":"absorption",
            "reversible":false, "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default","reference":null},
            "gasHeatPumpForfaitDraft":{
                "generatorId":"ga-1", "drive":"absorption", "application":"utility",
                "applicationReference":"building schedule", "collectiveBuildingInstallation":false,
                "externalHeatSupply":false, "thermalCapacityKw":20.0, "capacityReference":"plate",
                "source":"outdoor_air", "sourceReference":"source design",
                "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design"
            },
            "gasHeatPumpAuxDraft":{
                "generatorId":"ga-1", "drive":"absorption", "nominalThermalCapacityKw":20.0,
                "capacityReference":"plate", "standbyElectronicsW":10.0,
                "burnerAuxiliaryWPerKw":1.0, "solutionPumpWPerKw":0.0,
                "coefficientsReference":"draft 9.6.8.2.3", "meanModulation":1.0,
                "modulationReference":"draft 9.6.8.2.3", "buildingShare":1.0,
                "buildingShareReference":"whole building", "forfaitCopUsed":true,
                "monthHoursReference":"hour schedule", "generatorOutputReference":"generator ledger",
                "months":(1..=12).map(|month| serde_json::json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0})).collect::<Vec<_>>()
            }
        })).unwrap();
        assert!(pump.validate_inputs().is_empty());
        pump.gas_heat_pump_aux_draft
            .as_mut()
            .unwrap()
            .nominal_thermal_capacity_kw = 21.0;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "gas_heat_pump_aux_draft_forfait_mismatch"));
        pump.gas_heat_pump_aux_draft
            .as_mut()
            .unwrap()
            .nominal_thermal_capacity_kw = 20.0;
        pump.gas_heat_pump_forfait_draft = None;
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "gas_heat_pump_aux_draft_forfait_mismatch"));
        pump.gas_heat_pump_aux_draft.as_mut().unwrap().generator_id = "other".into();
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "gas_heat_pump_aux_draft_scope_invalid"));
    }

    #[test]
    fn preserves_and_checks_optional_declaration_registry_identity() {
        let input = serde_json::json!({
            "id":"hp-declared", "source":"outdoor_air", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{
                "kind":"controlled_quality_declaration", "reference":"20260214GK",
                "registryRecord":{
                    "registrationNumber":"20260214GK", "productName":"Model A + tank B",
                    "manufacturer":"Supplier", "sourceUrl":"https://bcrg.nl/declaration/example"
                }
            }
        });
        let pump: HeatPumpInput = serde_json::from_value(input.clone()).unwrap();
        assert!(pump.validate_inputs().is_empty());
        assert_eq!(serde_json::to_value(&pump).unwrap(), input);
        let mut invalid = input;
        invalid["performanceEvidence"]["registryRecord"]["sourceUrl"] =
            serde_json::json!("http://example.test");
        let pump: HeatPumpInput = serde_json::from_value(invalid).unwrap();
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "quality_declaration_source_url_invalid"));
        let mut incomplete = serde_json::to_value(&pump).unwrap();
        incomplete["performanceEvidence"]["registryRecord"]["sourceUrl"] =
            serde_json::json!("https://");
        let pump: HeatPumpInput = serde_json::from_value(incomplete).unwrap();
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "quality_declaration_source_url_invalid"));
    }

    #[test]
    fn rejects_gas_performance_input_for_electric_compression() {
        let pump: HeatPumpInput = serde_json::from_value(serde_json::json!({
            "id":"electric-hp", "source":"outdoor_air", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default","reference":null},
            "performancePoints":[{"id":"a7w35", "service":"space_heating",
                "sourceTemperatureC":7.0, "sinkTemperatureC":35.0,
                "usefulCapacityKw":5.0, "inputPowerKw":1.5,
                "inputEnergyCarrier":"gas", "testReference":"test-sheet"}]
        }))
        .unwrap();
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|issue| issue.code == "performance_point_carrier_mismatch"
                && issue.field == "performancePoints[0].inputEnergyCarrier"));
    }

    #[test]
    fn rejects_performance_point_with_overflowing_power_ratio() {
        let pump: HeatPumpInput = serde_json::from_value(serde_json::json!({
            "id":"hp-overflow", "source":"outdoor_air", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default","reference":null},
            "performancePoints":[{"id":"p1", "service":"space_heating",
                "sourceTemperatureC":7, "sinkTemperatureC":35,
                "usefulCapacityKw":1e308, "inputPowerKw":1e-308,
                "inputEnergyCarrier":"electricity", "testReference":"lab-1"}]
        }))
        .unwrap();
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|issue| issue.code == "performance_point_ratio_overflow"
                && issue.field == "performancePoints[0].inputPowerKw"));
    }

    #[test]
    fn rejects_two_points_with_the_same_operating_condition() {
        let pump: HeatPumpInput = serde_json::from_value(serde_json::json!({
            "id":"hp-duplicate", "source":"outdoor_air", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default","reference":null},
            "performancePoints":[
                {"id":"p1", "service":"space_heating", "sourceTemperatureC":7,
                 "sinkTemperatureC":35, "usefulCapacityKw":5, "inputPowerKw":2,
                 "inputEnergyCarrier":"electricity", "testReference":"lab-1"},
                {"id":"p2", "service":"space_heating", "sourceTemperatureC":7,
                 "sinkTemperatureC":35, "usefulCapacityKw":5, "inputPowerKw":2.2,
                 "inputEnergyCarrier":"electricity", "testReference":"lab-2"}
            ]
        }))
        .unwrap();
        assert!(pump.validate_inputs().iter().any(|issue| issue.code
            == "performance_point_condition_duplicate"
            && issue.field == "performancePoints[1].sourceTemperatureC"));
        let mut part_load = pump;
        part_load.performance_points[1].useful_capacity_kw = 3.0;
        assert!(part_load.validate_inputs().is_empty());
    }

    #[test]
    fn roundtrips_absorption_and_booster_options() {
        let input = serde_json::json!({
            "id": "hp-2", "source": "waste_heat", "sink": "domestic_hot_water",
            "drive": "absorption", "reversible": false, "hybrid": true, "booster": true,
            "performanceEvidence": {"kind": "normative_default", "reference": null}
        });
        let pump: HeatPumpInput = serde_json::from_value(input.clone()).unwrap();
        assert!(pump.validate_inputs().is_empty());
        assert_eq!(serde_json::to_value(pump).unwrap(), input);
    }

    #[test]
    fn rejects_untraceable_or_physically_invalid_operating_points() {
        let input = serde_json::json!({
            "id": "hp", "source": "groundwater", "sink": "hydronic",
            "drive": "electric_compression", "reversible": false,
            "hybrid": false, "booster": false,
            "performanceEvidence": {"kind": "normative_default", "reference": null},
            "performancePoints": [{
                "id": "p1", "service": "space_cooling", "sourceTemperatureC": 12,
                "sinkTemperatureC": 35, "usefulCapacityKw": 0,
                "inputPowerKw": -1, "inputEnergyCarrier": "electricity",
                "testReference": ""
            }]
        });
        let pump: HeatPumpInput = serde_json::from_value(input).unwrap();
        let codes: HashSet<_> = pump
            .validate_inputs()
            .into_iter()
            .map(|issue| issue.code)
            .collect();
        assert!(codes.contains("performance_point_power_invalid"));
        assert!(codes.contains("performance_point_reference_required"));
        assert!(codes.contains("performance_point_service_mismatch"));
    }

    #[test]
    fn rejects_misspelled_fields_in_heat_pump_and_operating_point() {
        let mut input = serde_json::json!({
            "id": "hp", "source": "outdoor_air", "sink": "hydronic",
            "drive": "electric_compression", "reversible": false,
            "hybrid": false, "booster": false,
            "performanceEvidence": {"kind": "normative_default", "reference": null}
        });
        input["performancePoint"] = serde_json::json!([]);
        assert!(serde_json::from_value::<HeatPumpInput>(input.clone()).is_err());
        input.as_object_mut().unwrap().remove("performancePoint");
        input["performancePoints"] = serde_json::json!([{
            "id": "p1", "service": "space_heating", "sourceTemperatureC": 7,
            "sinkTemperatureC": 35, "usefulCapacityKw": 5,
            "inputPowerKW": 1.5, "inputEnergyCarrier": "electricity",
            "testReference": "lab-1"
        }]);
        assert!(serde_json::from_value::<HeatPumpInput>(input).is_err());
    }

    #[test]
    fn stores_auxiliary_power_with_service_and_measurement_boundary() {
        let input = serde_json::json!({
            "id":"hp", "source":"ground", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "auxiliaryComponents":[{
                "id":"source-pump", "kind":"source_pump", "service":"space_heating",
                "nominalPowerW":80.0, "energyCarrier":"electricity",
                "measurementBoundary":"additional", "evidenceReference":"datasheet-1"
            }]
        });
        let pump: HeatPumpInput = serde_json::from_value(input.clone()).unwrap();
        assert!(pump.validate_inputs().is_empty());
        assert_eq!(serde_json::to_value(pump).unwrap(), input);
    }

    #[test]
    fn rejects_self_link_and_misspelled_target_field() {
        let mut value = serde_json::json!({
            "id":"hp", "source":"ground", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false,
            "hybrid":true, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "systemLinks":[{"id":"backup", "role":"backup_generator",
                "targetKind":"heating_system", "targetId":"hp",
                "evidenceReference":"scheme-1"}]
        });
        let pump: HeatPumpInput = serde_json::from_value(value.clone()).unwrap();
        assert!(pump
            .validate_inputs()
            .iter()
            .any(|item| item.code == "system_link_target_invalid"));
        value["systemLinks"][0]["targetID"] = value["systemLinks"][0]["targetId"].take();
        assert!(serde_json::from_value::<HeatPumpInput>(value).is_err());
    }

    #[test]
    fn rejects_untraceable_and_conflicting_auxiliaries() {
        let input = serde_json::json!({
            "id":"hp", "source":"outdoor_air", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "auxiliaryComponents":[
                {"id":"fan", "kind":"source_fan", "service":"space_heating",
                 "nominalPowerW":0, "energyCarrier":"electricity",
                 "measurementBoundary":"unknown", "evidenceReference":""},
                {"id":"fan", "kind":"source_fan", "service":"space_cooling",
                 "nominalPowerW":70, "energyCarrier":"electricity",
                 "measurementBoundary":"additional", "evidenceReference":"datasheet"}
            ]
        });
        let pump: HeatPumpInput = serde_json::from_value(input).unwrap();
        let codes: HashSet<_> = pump
            .validate_inputs()
            .iter()
            .map(|issue| issue.code)
            .collect();
        for code in [
            "auxiliary_id_invalid",
            "auxiliary_power_invalid",
            "auxiliary_reference_required",
            "auxiliary_service_mismatch",
        ] {
            assert!(codes.contains(code), "missing {code}");
        }
    }

    #[test]
    fn refuses_misspelled_auxiliary_power_field() {
        let input = serde_json::json!({
            "id":"hp", "source":"ground", "sink":"hydronic",
            "drive":"electric_compression", "reversible":false,
            "hybrid":false, "booster":false,
            "performanceEvidence":{"kind":"normative_default", "reference":null},
            "auxiliaryComponents":[{
                "id":"pump", "kind":"source_pump", "service":"space_heating",
                "nominalPowerKW":0.08, "energyCarrier":"electricity",
                "measurementBoundary":"additional", "evidenceReference":"datasheet"
            }]
        });
        assert!(serde_json::from_value::<HeatPumpInput>(input).is_err());
    }
}
