//! Lookup of electric heat-pump COP values in NTA 8800:2025+C1:2026 tables
//! 9.27 (p. 333–334) and 9.29 (p. 337–338). The values were first taken from
//! the public consultation draft and checked against the final edition. This
//! is not a generator-energy calculation.

use crate::{input_fingerprint, KERNEL_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Final-edition citation of the tables; the name is kept for the output field.
pub const DRAFT_SOURCE: &str =
    "NTA 8800:2025+C1:2026, tabellen 9.27 (p. 333–334) en 9.29 (p. 337–338)";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableScope {
    #[serde(rename = "residential_at_most25_kw")]
    ResidentialAtMost25Kw,
    #[serde(rename = "utility_collective_or_over25_kw")]
    UtilityCollectiveOrOver25Kw,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableSource {
    Ground,
    /// The source is known to be ground or groundwater, but its type and/or temperature is unknown.
    GroundOrGroundwaterUnknown,
    #[serde(rename = "groundwater_below15_c")]
    GroundwaterBelow15C,
    OutdoorAir,
    ExhaustAir,
    SurfaceWater,
    #[serde(rename = "collective15_to20_c")]
    Collective15To20C,
    #[serde(rename = "collective20_to40_c")]
    Collective20To40C,
    #[serde(rename = "collective_at_least40_c")]
    CollectiveAtLeast40C,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableSink {
    Hydronic,
    IndoorAir,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableRowVariant {
    #[default]
    Base,
    #[serde(rename = "table_9_28_high_efficiency")]
    Table928HighEfficiency,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum HighEfficiencyTestCondition {
    #[serde(rename = "b0_w45")]
    B0W45,
    #[serde(rename = "b0_w35")]
    B0W35,
    #[serde(rename = "w10_w45")]
    W10W45,
    #[serde(rename = "w10_w35")]
    W10W35,
    #[serde(rename = "a7_wet6_w45")]
    A7Wet6W45,
    #[serde(rename = "a7_wet6_w35")]
    A7Wet6W35,
    #[serde(rename = "a_minus7_wet_minus8_w45")]
    AMinus7WetMinus8W45,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HighEfficiencyTestPoint {
    pub condition: HighEfficiencyTestCondition,
    pub measured_cop: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HighEfficiencyEvidence {
    pub product_reference: String,
    pub test_report_reference: String,
    pub test_standard_edition: String,
    pub points: Vec<HighEfficiencyTestPoint>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForfaitHeatPumpDraftInput {
    pub generator_id: String,
    pub classification_source_reference: String,
    pub scope: TableScope,
    pub source: TableSource,
    pub sink: TableSink,
    /// Required for hydronic delivery; table classes end at 70 °C.
    pub design_supply_temperature_c: Option<f64>,
    /// Supplied Annex-V correction for residential ground/groundwater, including
    /// an explicit 1.0 when source regeneration is inapplicable.
    pub source_correction_factor: Option<f64>,
    pub source_correction_reference: Option<String>,
    /// Declared thermal generator capacity for choosing the 25 kW table boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thermal_capacity_kw: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity_source_reference: Option<String>,
    /// Whether the generator is part of a collective building installation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collective_building_installation: Option<bool>,
    #[serde(default, skip_serializing_if = "is_base_row")]
    pub row_variant: TableRowVariant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high_efficiency_evidence: Option<HighEfficiencyEvidence>,
    /// Supplied source temperature used to classify groundwater and collective-source rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_temperature_c: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_temperature_evidence_reference: Option<String>,
    /// Claimed source quality declaration; its authenticity is not checked here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_quality_declaration_reference: Option<String>,
    /// A quality declaration (kwaliteitsverklaring, e.g. BCRG) whose
    /// efficiency replaces the table 9.27/9.28/9.29 value (§9.1, p. 285).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_declaration: Option<HeatPumpQualityDeclaration>,
    /// Build year of the device. Only NTA 8800:2020+A1 uses it: the 9.85
    /// forfait A is 13,0 kWh from 2015 and 87,6 kWh before or unknown
    /// (2020 p. 336); later editions give heat pumps one A of 43,8 kWh.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation_year: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation_year_reference: Option<String>,
}

/// Declared values of a heat pump for space heating (§9.1, p. 285): the
/// declared `η_H;gen` (COP) replaces the table value and is rounded down to
/// a multiple of 0,05 (electric) — the table `c_source` of footnote a still
/// applies to ground and groundwater sources. The declared energy fraction
/// and auxiliary energy are applied by the space-heating chain.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatPumpQualityDeclaration {
    /// Declaration number and issuer, e.g. "BCRG 91849/03".
    pub declaration_reference: String,
    /// Declared `η_H;gen;si;hp` (COP) for the design supply temperature and
    /// annual heat demand of this system (interpolated in the declaration).
    pub generation_efficiency: f64,
    /// Declared `F_H;gen;si,gpref`; omitted means 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_fraction: Option<f64>,
    /// Declared `W_H;aux` of the appliance in kWh per year; replaces the
    /// 9.85 forfait auxiliary energy. Omitted keeps the forfait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary_kwh_per_year: Option<f64>,
}

/// Round down to a multiple of `step` (§9.1, p. 285).
fn round_down_to(value: f64, step: f64) -> f64 {
    ((value / step) + 1e-9).floor() * step
}

fn is_base_row(row: &TableRowVariant) -> bool {
    *row == TableRowVariant::Base
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForfaitHeatPumpDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub consultation_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub table: &'static str,
    pub row_variant: TableRowVariant,
    pub selected_source: TableSource,
    pub source_fallback_applied: bool,
    pub source_fallback_reason: Option<&'static str>,
    pub temperature_band: Option<&'static str>,
    pub table_cop: Option<f64>,
    pub corrected_cop: Option<f64>,
    pub final_edition_verified: bool,
    pub applicability_verified: bool,
    pub annual_performance_available: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<TableIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableIssue {
    pub code: &'static str,
    pub path: &'static str,
}

fn issue(code: &'static str, path: &'static str) -> TableIssue {
    TableIssue { code, path }
}

const BANDS: [&str; 9] = [
    "≤30 °C",
    ">30–35 °C",
    ">35–40 °C",
    ">40–45 °C",
    ">45–50 °C",
    ">50–55 °C",
    ">55–60 °C",
    ">60–65 °C",
    ">65–70 °C",
];

fn band_index(temperature_c: f64) -> Option<usize> {
    [30.0, 35.0, 40.0, 45.0, 50.0, 55.0, 60.0, 65.0, 70.0]
        .iter()
        .position(|upper| temperature_c <= *upper)
}

fn table_values(scope: TableScope, source: TableSource) -> Option<[Option<f64>; 9]> {
    use TableScope::{ResidentialAtMost25Kw as R, UtilityCollectiveOrOver25Kw as U};
    use TableSource::*;
    let full = match (scope, source) {
        (R, Ground) => [4.0, 3.8, 3.6, 3.4, 3.2, 3.0, 2.4, 2.3, 2.1],
        (_, GroundOrGroundwaterUnknown) => return None,
        (R, GroundwaterBelow15C) => [4.7, 4.5, 4.3, 4.1, 3.9, 3.7, 2.9, 2.7, 2.6],
        (R, OutdoorAir) => [2.4, 2.4, 2.35, 2.3, 2.3, 2.25, 1.8, 1.7, 1.6],
        (R, ExhaustAir) => [3.9, 3.65, 3.55, 3.45, 3.35, 3.25, 2.6, 2.4, 2.3],
        (R, Collective15To20C) => [5.1, 4.8, 4.5, 4.3, 4.1, 3.8, 3.0, 2.8, 2.6],
        (R, Collective20To40C) => [5.4, 5.1, 4.8, 4.5, 4.2, 4.0, 3.1, 2.9, 2.7],
        (U, Ground) => [3.55, 3.4, 3.25, 3.1, 2.95, 2.8, 2.2, 2.1, 2.0],
        (U, GroundwaterBelow15C) => [5.0, 4.7, 4.45, 4.2, 3.9, 3.6, 2.8, 2.7, 2.5],
        (U, OutdoorAir) => [3.4, 3.25, 3.15, 3.05, 2.9, 2.8, 2.2, 2.1, 2.0],
        (U, ExhaustAir) => [4.65, 4.5, 4.35, 4.2, 4.05, 3.9, 3.1, 2.9, 2.7],
        (U, SurfaceWater) => [4.3, 4.1, 3.9, 3.7, 3.5, 3.3, 2.6, 2.5, 2.3],
        (U, Collective15To20C) => [5.4, 5.0, 4.7, 4.4, 4.1, 3.7, 2.9, 2.7, 2.6],
        (U, Collective20To40C) => [5.8, 5.3, 4.9, 4.6, 4.2, 3.9, 3.2, 2.9, 2.7],
        (R, SurfaceWater) => return None,
        (_, CollectiveAtLeast40C) => {
            return Some([
                None,
                None,
                None,
                Some(if scope == R { 8.3 } else { 9.2 }),
                Some(if scope == R { 6.8 } else { 7.3 }),
                Some(if scope == R { 5.7 } else { 6.2 }),
                Some(if scope == R { 4.9 } else { 5.2 }),
                Some(if scope == R { 4.3 } else { 4.6 }),
                Some(if scope == R { 3.8 } else { 4.1 }),
            ])
        }
    };
    Some(full.map(Some))
}

/// Table 9.29 (utility, collective or > 25 kW) at 65 °C < θ_sup ≤ 70 °C,
/// used by 13.8.4.10 for a series of electric heat pumps.
pub fn utility_cop_65_to_70(source: TableSource) -> Option<f64> {
    table_values(TableScope::UtilityCollectiveOrOver25Kw, source)?[8]
}

fn high_row_values(source: TableSource) -> Option<[f64; 6]> {
    match source {
        TableSource::Ground => Some([4.55, 4.4, 4.25, 4.1, 3.9, 3.7]),
        TableSource::GroundwaterBelow15C => Some([5.2, 5.0, 4.8, 4.6, 4.4, 4.2]),
        TableSource::OutdoorAir => Some([3.5, 3.35, 3.15, 3.0, 2.8, 2.6]),
        _ => None,
    }
}

fn high_test_limits(source: TableSource) -> Option<&'static [(HighEfficiencyTestCondition, f64)]> {
    use HighEfficiencyTestCondition::*;
    match source {
        TableSource::Ground => Some(&[(B0W45, 3.0), (B0W35, 3.5)]),
        TableSource::GroundwaterBelow15C => Some(&[(W10W45, 3.75), (W10W35, 4.4)]),
        TableSource::OutdoorAir => Some(&[
            (A7Wet6W45, 2.75),
            (A7Wet6W35, 2.85),
            (AMinus7WetMinus8W45, 1.9),
        ]),
        _ => None,
    }
}

fn high_evidence_issues(input: &ForfaitHeatPumpDraftInput) -> Vec<TableIssue> {
    let mut issues = Vec::new();
    if input.scope != TableScope::ResidentialAtMost25Kw
        || input.sink != TableSink::Hydronic
        || high_row_values(input.source).is_none()
    {
        issues.push(issue("high_row_unavailable", "rowVariant"));
    }
    let Some(evidence) = &input.high_efficiency_evidence else {
        issues.push(issue(
            "high_test_evidence_required",
            "highEfficiencyEvidence",
        ));
        return issues;
    };
    if evidence.product_reference.trim().is_empty() {
        issues.push(issue(
            "source_required",
            "highEfficiencyEvidence.productReference",
        ));
    }
    if evidence.test_report_reference.trim().is_empty() {
        issues.push(issue(
            "source_required",
            "highEfficiencyEvidence.testReportReference",
        ));
    }
    if evidence.test_standard_edition != "NEN-EN 14511-2:2022" {
        issues.push(issue(
            "high_test_standard_invalid",
            "highEfficiencyEvidence.testStandardEdition",
        ));
    }
    if let Some(limits) = high_test_limits(input.source) {
        let mut seen = Vec::new();
        for point in &evidence.points {
            if seen.contains(&point.condition) {
                issues.push(issue(
                    "high_test_condition_duplicate",
                    "highEfficiencyEvidence.points",
                ));
            }
            seen.push(point.condition);
            match limits
                .iter()
                .find(|(condition, _)| *condition == point.condition)
            {
                Some((_, minimum))
                    if point.measured_cop.is_finite() && point.measured_cop > *minimum => {}
                Some(_) => issues.push(issue(
                    "high_test_cop_below_requirement",
                    "highEfficiencyEvidence.points",
                )),
                None => issues.push(issue(
                    "high_test_condition_unexpected",
                    "highEfficiencyEvidence.points",
                )),
            }
        }
        for (condition, _) in limits {
            if !seen.contains(condition) {
                issues.push(issue(
                    "high_test_condition_missing",
                    "highEfficiencyEvidence.points",
                ));
            }
        }
    }
    issues
}

fn source_classification_issues(input: &ForfaitHeatPumpDraftInput) -> Vec<TableIssue> {
    use TableSource::*;
    let bounds = match input.source {
        GroundwaterBelow15C => Some((f64::NEG_INFINITY, 15.0)),
        Collective15To20C => Some((15.0, 20.0)),
        Collective20To40C => Some((20.0, 40.0)),
        CollectiveAtLeast40C => Some((40.0, f64::INFINITY)),
        _ => None,
    };
    let Some((lower, upper)) = bounds else {
        return Vec::new();
    };
    // NTA 8800:2022 tables 9.27/9.29 (p. 314–317): "grondwater" without a
    // source temperature; the warmer source rows do not exist.
    if crate::norm_versions::profile().heat_pump_tables_2022 {
        return if input.source == GroundwaterBelow15C {
            Vec::new()
        } else {
            vec![issue("route_not_in_edition", "source")]
        };
    }
    let mut issues = Vec::new();
    if input.source_temperature_c.map_or(true, |value| {
        !value.is_finite() || value < lower || value >= upper
    }) {
        issues.push(issue(
            "source_temperature_class_mismatch",
            "sourceTemperatureC",
        ));
    }
    if input
        .source_temperature_evidence_reference
        .as_deref()
        .map_or(true, |value| value.trim().is_empty())
    {
        issues.push(issue(
            "source_required",
            "sourceTemperatureEvidenceReference",
        ));
    }
    issues
}

pub fn source_fallback_applies(input: &ForfaitHeatPumpDraftInput) -> bool {
    input.source == TableSource::GroundOrGroundwaterUnknown
        || matches!(
            input.source,
            TableSource::Collective20To40C | TableSource::CollectiveAtLeast40C
        ) && input
            .source_quality_declaration_reference
            .as_deref()
            .map_or(true, |value| value.trim().is_empty())
}

pub fn assess_forfait_heat_pump_draft(
    input: &ForfaitHeatPumpDraftInput,
) -> ForfaitHeatPumpDraftAssessment {
    let mut issues = Vec::new();
    if input.generator_id.trim().is_empty() {
        issues.push(issue("generator_id_required", "generatorId"));
    }
    if input.classification_source_reference.trim().is_empty() {
        issues.push(issue("source_required", "classificationSourceReference"));
    }
    issues.extend(source_classification_issues(input));
    if input
        .installation_year
        .is_some_and(|year| !(1900..=2026).contains(&year))
    {
        issues.push(issue("installation_year_invalid", "installationYear"));
    }
    if input.installation_year.is_some()
        && input
            .installation_year_reference
            .as_deref()
            .map_or(true, |reference| reference.trim().is_empty())
    {
        issues.push(issue(
            "installation_year_reference_required",
            "installationYearReference",
        ));
    }
    if input.installation_year.is_none() && input.installation_year_reference.is_some() {
        issues.push(issue(
            "installation_year_reference_without_year",
            "installationYearReference",
        ));
    }
    let source_fallback_applied = source_fallback_applies(input);
    let source_fallback_reason = if input.source == TableSource::GroundOrGroundwaterUnknown {
        Some("ground_or_groundwater_unknown")
    } else if source_fallback_applied {
        Some("source_quality_declaration_missing")
    } else {
        None
    };
    let selected_source = if input.source == TableSource::GroundOrGroundwaterUnknown {
        TableSource::Ground
    } else if source_fallback_applied {
        TableSource::GroundwaterBelow15C
    } else {
        input.source
    };
    match input.row_variant {
        TableRowVariant::Base if input.high_efficiency_evidence.is_some() => {
            issues.push(issue(
                "high_test_evidence_not_applicable",
                "highEfficiencyEvidence",
            ));
        }
        TableRowVariant::Table928HighEfficiency => {
            issues.extend(high_evidence_issues(input));
        }
        _ => {}
    }
    let capacity_evidence_present = input.thermal_capacity_kw.is_some()
        || input.capacity_source_reference.is_some()
        || input.collective_building_installation.is_some();
    if capacity_evidence_present {
        if input
            .thermal_capacity_kw
            .map_or(true, |value| !value.is_finite() || value <= 0.0)
        {
            issues.push(issue("thermal_capacity_invalid", "thermalCapacityKw"));
        }
        if input
            .capacity_source_reference
            .as_deref()
            .map_or(true, |value| value.trim().is_empty())
        {
            issues.push(issue("source_required", "capacitySourceReference"));
        }
        if input.collective_building_installation.is_none() {
            issues.push(issue(
                "system_arrangement_required",
                "collectiveBuildingInstallation",
            ));
        }
        // NTA 8800:2022 (p. 314, 317): table 9.27 for dwellings, 9.29 for
        // utility buildings, without the 25 kW or collective boundary.
        if input.scope == TableScope::ResidentialAtMost25Kw
            && !crate::norm_versions::profile().heat_pump_tables_2022
            && (input.thermal_capacity_kw.is_some_and(|value| value > 25.0)
                || input.collective_building_installation == Some(true))
        {
            issues.push(issue("table_scope_capacity_mismatch", "scope"));
        }
    }
    let table = match input.scope {
        TableScope::ResidentialAtMost25Kw => "9.27",
        TableScope::UtilityCollectiveOrOver25Kw => "9.29",
    };
    let mut selected_band = None;
    let mut table_cop = None;
    let mut corrected_cop = None;
    if input.sink == TableSink::IndoorAir {
        if input.source != TableSource::OutdoorAir {
            issues.push(issue("air_to_air_source_unsupported", "source"));
        }
        if input.design_supply_temperature_c.is_some() {
            issues.push(issue(
                "temperature_not_applicable",
                "designSupplyTemperatureC",
            ));
        }
        if input.source_correction_factor.is_some() || input.source_correction_reference.is_some() {
            issues.push(issue(
                "source_correction_not_applicable",
                "sourceCorrectionFactor",
            ));
        }
        if issues.is_empty() {
            table_cop = Some(
                input
                    .quality_declaration
                    .as_ref()
                    .filter(|item| {
                        item.generation_efficiency.is_finite() && item.generation_efficiency > 0.0
                    })
                    .map_or(2.8, |item| round_down_to(item.generation_efficiency, 0.05)),
            );
            corrected_cop = table_cop;
        }
    } else {
        let index = match input.design_supply_temperature_c {
            Some(value) if value.is_finite() && value > 0.0 => band_index(value),
            _ => None,
        };
        if index.is_none() {
            issues.push(issue(
                "supply_temperature_invalid",
                "designSupplyTemperatureC",
            ));
        }
        // NTA 8800:2022 p. 313–317: the tables end at 55 °C; above it annex Q
        // applies.
        let index = if crate::norm_versions::profile().heat_pump_tables_2022
            && index.is_some_and(|value| value > 5)
        {
            issues.push(issue("route_not_in_edition", "designSupplyTemperatureC"));
            None
        } else {
            index
        };
        let correction_required = input.scope == TableScope::ResidentialAtMost25Kw
            && matches!(
                selected_source,
                TableSource::Ground | TableSource::GroundwaterBelow15C
            );
        if correction_required {
            if input
                .source_correction_factor
                .map_or(true, |value| !value.is_finite() || value <= 0.0)
            {
                issues.push(issue("source_correction_invalid", "sourceCorrectionFactor"));
            }
            if input
                .source_correction_reference
                .as_deref()
                .map_or(true, |value| value.trim().is_empty())
            {
                issues.push(issue("source_required", "sourceCorrectionReference"));
            }
        } else if input.source_correction_factor.is_some()
            || input.source_correction_reference.is_some()
        {
            issues.push(issue(
                "source_correction_not_applicable",
                "sourceCorrectionFactor",
            ));
        }
        if let Some(index) = index {
            selected_band = Some(BANDS[index]);
            let selected_value = match input.row_variant {
                TableRowVariant::Base => {
                    table_values(input.scope, selected_source).and_then(|row| row[index])
                }
                TableRowVariant::Table928HighEfficiency => {
                    high_row_values(input.source).and_then(|row| row.get(index).copied())
                }
            };
            if let Some(declaration) = &input.quality_declaration {
                if declaration.declaration_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_required",
                        "qualityDeclaration.declarationReference",
                    ));
                }
                if !declaration.generation_efficiency.is_finite()
                    || declaration.generation_efficiency <= 0.0
                    || declaration.generation_efficiency > 15.0
                {
                    issues.push(issue(
                        "heat_pump_declared_efficiency_invalid",
                        "qualityDeclaration.generationEfficiency",
                    ));
                }
                if declaration
                    .energy_fraction
                    .is_some_and(|value| !value.is_finite() || value <= 0.0 || value > 1.0)
                {
                    issues.push(issue(
                        "heat_pump_declared_fraction_invalid",
                        "qualityDeclaration.energyFraction",
                    ));
                }
                if declaration
                    .auxiliary_kwh_per_year
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
                {
                    issues.push(issue(
                        "heat_pump_declared_auxiliary_invalid",
                        "qualityDeclaration.auxiliaryKwhPerYear",
                    ));
                }
            }
            // §9.1 (p. 285): a declared value replaces the table cell and is
            // rounded down to a multiple of 0,05 for electric generators.
            let selected_value = match &input.quality_declaration {
                Some(declaration) => selected_value
                    .map(|_| round_down_to(declaration.generation_efficiency, 0.05))
                    .or(Some(round_down_to(declaration.generation_efficiency, 0.05))),
                None => selected_value,
            };
            match selected_value {
                Some(value) if issues.is_empty() => {
                    let corrected = value * input.source_correction_factor.unwrap_or(1.0);
                    if corrected.is_finite() && corrected > 0.0 {
                        table_cop = Some(value);
                        corrected_cop = Some(corrected);
                    } else {
                        issues.push(issue("cop_overflow", "sourceCorrectionFactor"));
                    }
                }
                None => issues.push(issue("table_cell_unavailable", "source")),
                _ => {}
            }
        }
    }
    ForfaitHeatPumpDraftAssessment {
        status: if issues.is_empty() {
            "input_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_electric_heat_pump_forfait_lookup_only",
        consultation_source: DRAFT_SOURCE,
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        table,
        row_variant: input.row_variant,
        selected_source,
        source_fallback_applied,
        source_fallback_reason,
        temperature_band: selected_band,
        table_cop,
        corrected_cop,
        final_edition_verified: false,
        applicability_verified: false,
        annual_performance_available: false,
        beng_calculation_available: false,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(scope: TableScope, source: TableSource, temp: f64) -> ForfaitHeatPumpDraftInput {
        ForfaitHeatPumpDraftInput {
            installation_year: None,
            installation_year_reference: None,
            generator_id: "hp".into(),
            classification_source_reference: "system design".into(),
            scope,
            source,
            sink: TableSink::Hydronic,
            design_supply_temperature_c: Some(temp),
            source_correction_factor: None,
            source_correction_reference: None,
            thermal_capacity_kw: None,
            capacity_source_reference: None,
            collective_building_installation: None,
            row_variant: TableRowVariant::Base,
            high_efficiency_evidence: None,
            source_temperature_c: None,
            source_temperature_evidence_reference: None,
            source_quality_declaration_reference: None,
            quality_declaration: None,
        }
    }

    #[test]
    fn tables_9_27_and_9_29_of_2022() {
        use crate::norm_versions::{with_version, NormVersion};
        let codes = |edition, input: &ForfaitHeatPumpDraftInput| {
            with_version(edition, || {
                assess_forfait_heat_pump_draft(input)
                    .issues
                    .iter()
                    .map(|item| (item.code, item.path))
                    .collect::<Vec<_>>()
            })
        };
        let cop = |edition, input: &ForfaitHeatPumpDraftInput| {
            with_version(edition, || assess_forfait_heat_pump_draft(input).table_cop)
        };
        // 2022 p. 314: the outdoor-air row is the same up to 55 °C.
        let air = example(
            TableScope::UtilityCollectiveOrOver25Kw,
            TableSource::OutdoorAir,
            55.0,
        );
        assert_eq!(cop(NormVersion::V2022, &air), Some(2.8));
        assert_eq!(cop(NormVersion::V2022, &air), cop(NormVersion::V2023, &air));
        // Above 55 °C annex Q (2022 p. 313); 2023 p. 323 has 2,2 at 60 °C.
        let hot = example(
            TableScope::UtilityCollectiveOrOver25Kw,
            TableSource::OutdoorAir,
            60.0,
        );
        assert_eq!(cop(NormVersion::V2023, &hot), Some(2.2));
        assert_eq!(
            codes(NormVersion::V2022, &hot),
            vec![("route_not_in_edition", "designSupplyTemperatureC")]
        );
        // No source-temperature rows in 2022 (2023 p. 319–324).
        let mut warm = example(
            TableScope::UtilityCollectiveOrOver25Kw,
            TableSource::Collective20To40C,
            45.0,
        );
        warm.source_temperature_c = Some(25.0);
        warm.source_temperature_evidence_reference = Some("design".into());
        warm.source_quality_declaration_reference = Some("declaration".into());
        assert!(codes(NormVersion::V2023, &warm).is_empty());
        assert_eq!(
            codes(NormVersion::V2022, &warm),
            vec![("route_not_in_edition", "source")]
        );
        // "Grondwater" without a temperature, and table 9.27 for every
        // dwelling: no 25 kW or collective boundary in 2022 (p. 314).
        let mut ground = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::GroundwaterBelow15C,
            35.0,
        );
        ground.source_correction_factor = Some(1.0);
        ground.source_correction_reference = Some("annex V".into());
        ground.thermal_capacity_kw = Some(40.0);
        ground.capacity_source_reference = Some("type plate".into());
        ground.collective_building_installation = Some(true);
        assert!(codes(NormVersion::V2022, &ground).is_empty());
        assert_eq!(cop(NormVersion::V2022, &ground), Some(4.5));
        assert!(codes(NormVersion::V2023, &ground)
            .iter()
            .any(|(code, _)| *code == "table_scope_capacity_mismatch"));
    }

    #[test]
    fn capacity_and_collective_evidence_enforce_residential_table_boundary() {
        let mut input = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::OutdoorAir,
            35.0,
        );
        input.thermal_capacity_kw = Some(25.0);
        input.capacity_source_reference = Some("manufacturer rated thermal capacity".into());
        input.collective_building_installation = Some(false);
        assert_eq!(assess_forfait_heat_pump_draft(&input).status, "input_valid");
        input.thermal_capacity_kw = Some(25.01);
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "table_scope_capacity_mismatch"));
        input.thermal_capacity_kw = Some(25.0);
        input.collective_building_installation = Some(true);
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "table_scope_capacity_mismatch"));
        input.scope = TableScope::UtilityCollectiveOrOver25Kw;
        assert_eq!(assess_forfait_heat_pump_draft(&input).status, "input_valid");
        input.capacity_source_reference = None;
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "source_required" && item.path == "capacitySourceReference"));
    }

    fn evidence(points: &[(HighEfficiencyTestCondition, f64)]) -> HighEfficiencyEvidence {
        HighEfficiencyEvidence {
            product_reference: "tested model A".into(),
            test_report_reference: "EN 14511 lab report p4".into(),
            test_standard_edition: "NEN-EN 14511-2:2022".into(),
            points: points
                .iter()
                .map(|(condition, measured_cop)| HighEfficiencyTestPoint {
                    condition: *condition,
                    measured_cop: *measured_cop,
                })
                .collect(),
        }
    }

    #[test]
    fn high_row_requires_all_strictly_above_928_test_limits() {
        use HighEfficiencyTestCondition::*;
        let mut input = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::OutdoorAir,
            35.0,
        );
        input.row_variant = TableRowVariant::Table928HighEfficiency;
        input.high_efficiency_evidence = Some(evidence(&[
            (A7Wet6W45, 2.76),
            (A7Wet6W35, 2.86),
            (AMinus7WetMinus8W45, 1.91),
        ]));
        let result = assess_forfait_heat_pump_draft(&input);
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.table_cop, Some(3.35));
        assert_eq!(result.row_variant, TableRowVariant::Table928HighEfficiency);
        assert!(!result.applicability_verified);
        input.high_efficiency_evidence.as_mut().unwrap().points[0].measured_cop = 2.75;
        let result = assess_forfait_heat_pump_draft(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.corrected_cop.is_none());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "high_test_cop_below_requirement"));
        input
            .high_efficiency_evidence
            .as_mut()
            .unwrap()
            .points
            .pop();
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "high_test_condition_missing"));
        input
            .high_efficiency_evidence
            .as_mut()
            .unwrap()
            .points
            .push(HighEfficiencyTestPoint {
                condition: A7Wet6W35,
                measured_cop: 3.0,
            });
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "high_test_condition_duplicate"));
    }

    #[test]
    fn high_row_has_no_value_above55_or_without_eligible_source() {
        use HighEfficiencyTestCondition::*;
        let mut input = example(TableScope::ResidentialAtMost25Kw, TableSource::Ground, 55.0);
        input.row_variant = TableRowVariant::Table928HighEfficiency;
        input.source_correction_factor = Some(1.0);
        input.source_correction_reference = Some("no regeneration".into());
        input.high_efficiency_evidence = Some(evidence(&[(B0W45, 3.01), (B0W35, 3.51)]));
        assert_eq!(
            assess_forfait_heat_pump_draft(&input).corrected_cop,
            Some(3.7)
        );
        input.design_supply_temperature_c = Some(55.01);
        let result = assess_forfait_heat_pump_draft(&input);
        assert!(result.corrected_cop.is_none());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "table_cell_unavailable"));
        input.design_supply_temperature_c = Some(35.0);
        input.scope = TableScope::UtilityCollectiveOrOver25Kw;
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "high_row_unavailable"));
        input.scope = TableScope::ResidentialAtMost25Kw;
        input.source = TableSource::ExhaustAir;
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "high_row_unavailable"));
    }

    #[test]
    fn published_boundaries_select_class_without_interpolation() {
        let mut input = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::OutdoorAir,
            35.0,
        );
        let low = assess_forfait_heat_pump_draft(&input);
        assert_eq!(low.table_cop, Some(2.4));
        assert_eq!(low.temperature_band, Some(">30–35 °C"));
        input.design_supply_temperature_c = Some(35.01);
        assert_eq!(assess_forfait_heat_pump_draft(&input).table_cop, Some(2.35));
        input.design_supply_temperature_c = Some(70.0);
        assert_eq!(assess_forfait_heat_pump_draft(&input).table_cop, Some(1.6));
        input.design_supply_temperature_c = Some(70.01);
        assert!(assess_forfait_heat_pump_draft(&input)
            .corrected_cop
            .is_none());
    }

    #[test]
    fn utility_surface_water_and_hot_collective_use_their_own_rows() {
        let utility = example(
            TableScope::UtilityCollectiveOrOver25Kw,
            TableSource::SurfaceWater,
            45.0,
        );
        assert_eq!(
            assess_forfait_heat_pump_draft(&utility).corrected_cop,
            Some(3.7)
        );
        let mut hot = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::CollectiveAtLeast40C,
            40.0,
        );
        hot.source_temperature_c = Some(40.0);
        hot.source_temperature_evidence_reference = Some("design temperature schedule".into());
        hot.source_quality_declaration_reference = Some("source quality declaration QD-1".into());
        assert!(assess_forfait_heat_pump_draft(&hot).corrected_cop.is_none());
        hot.design_supply_temperature_c = Some(40.01);
        assert_eq!(
            assess_forfait_heat_pump_draft(&hot).corrected_cop,
            Some(8.3)
        );
    }

    #[test]
    fn collective_source_temperature_and_declaration_boundaries_are_explicit() {
        let mut input = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::Collective20To40C,
            45.0,
        );
        input.source_temperature_c = Some(20.0);
        input.source_temperature_evidence_reference = Some("measured source design".into());
        let fallback = assess_forfait_heat_pump_draft(&input);
        assert!(fallback.source_fallback_applied);
        assert_eq!(fallback.selected_source, TableSource::GroundwaterBelow15C);
        assert!(fallback.corrected_cop.is_none());
        assert!(fallback
            .issues
            .iter()
            .any(|item| item.code == "source_correction_invalid"));
        input.source_correction_factor = Some(1.0);
        input.source_correction_reference = Some("no regeneration".into());
        assert_eq!(
            assess_forfait_heat_pump_draft(&input).corrected_cop,
            Some(4.1)
        );
        input.source_quality_declaration_reference = Some("quality declaration QD-1".into());
        input.source_correction_factor = None;
        input.source_correction_reference = None;
        assert_eq!(
            assess_forfait_heat_pump_draft(&input).corrected_cop,
            Some(4.5)
        );
        input.source_temperature_c = Some(19.99);
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "source_temperature_class_mismatch"));
        input.source_temperature_c = Some(40.0);
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "source_temperature_class_mismatch"));
        input.source = TableSource::CollectiveAtLeast40C;
        assert_eq!(
            assess_forfait_heat_pump_draft(&input).corrected_cop,
            Some(8.3)
        );
        input.source = TableSource::GroundwaterBelow15C;
        input.source_quality_declaration_reference = None;
        input.source_temperature_c = Some(14.99);
        input.source_correction_factor = Some(1.0);
        input.source_correction_reference = Some("no regeneration".into());
        assert_eq!(
            assess_forfait_heat_pump_draft(&input).corrected_cop,
            Some(4.1)
        );
        input.scope = TableScope::UtilityCollectiveOrOver25Kw;
        input.source = TableSource::Collective20To40C;
        input.source_temperature_c = Some(20.0);
        input.source_correction_factor = None;
        input.source_correction_reference = None;
        let fallback = assess_forfait_heat_pump_draft(&input);
        assert_eq!(fallback.corrected_cop, Some(4.2));
        assert!(fallback.source_fallback_applied);
    }

    #[test]
    fn unknown_ground_or_groundwater_class_uses_ground_row_without_temperature() {
        let mut input = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::GroundOrGroundwaterUnknown,
            35.0,
        );
        let missing_correction = assess_forfait_heat_pump_draft(&input);
        assert_eq!(missing_correction.selected_source, TableSource::Ground);
        assert_eq!(
            missing_correction.source_fallback_reason,
            Some("ground_or_groundwater_unknown")
        );
        assert!(missing_correction.corrected_cop.is_none());
        assert!(missing_correction
            .issues
            .iter()
            .any(|item| item.code == "source_correction_invalid"));
        input.source_correction_factor = Some(1.0);
        input.source_correction_reference = Some("no regeneration, source plan".into());
        let residential = assess_forfait_heat_pump_draft(&input);
        assert_eq!(residential.corrected_cop, Some(3.8));
        assert!(residential.source_fallback_applied);
        input.scope = TableScope::UtilityCollectiveOrOver25Kw;
        input.source_correction_factor = None;
        input.source_correction_reference = None;
        assert_eq!(
            assess_forfait_heat_pump_draft(&input).corrected_cop,
            Some(3.4)
        );
    }

    #[test]
    fn ground_correction_must_be_explicit_and_traceable() {
        let mut input = example(TableScope::ResidentialAtMost25Kw, TableSource::Ground, 30.0);
        assert!(assess_forfait_heat_pump_draft(&input)
            .corrected_cop
            .is_none());
        input.source_correction_factor = Some(1.0);
        input.source_correction_reference = Some("No regeneration, design sheet".into());
        assert_eq!(
            assess_forfait_heat_pump_draft(&input).corrected_cop,
            Some(4.0)
        );
        input.source_correction_factor = Some(f64::MAX);
        assert!(assess_forfait_heat_pump_draft(&input)
            .corrected_cop
            .is_none());
    }

    #[test]
    fn air_to_air_and_unavailable_residential_surface_water_stay_scoped() {
        let mut input = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::OutdoorAir,
            45.0,
        );
        input.sink = TableSink::IndoorAir;
        input.design_supply_temperature_c = None;
        assert_eq!(
            assess_forfait_heat_pump_draft(&input).corrected_cop,
            Some(2.8)
        );
        input.source = TableSource::SurfaceWater;
        assert!(assess_forfait_heat_pump_draft(&input)
            .corrected_cop
            .is_none());
        input.sink = TableSink::Hydronic;
        input.design_supply_temperature_c = Some(45.0);
        assert!(assess_forfait_heat_pump_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "table_cell_unavailable"));
    }

    /// §9.1 (p. 285): a declared COP replaces the table cell, rounded down
    /// to 0,05; invalid declarations are refused.
    #[test]
    fn quality_declaration_replaces_the_table_cop() {
        let mut input = example(
            TableScope::ResidentialAtMost25Kw,
            TableSource::OutdoorAir,
            35.0,
        );
        let table = assess_forfait_heat_pump_draft(&input)
            .corrected_cop
            .unwrap();
        input.quality_declaration = Some(HeatPumpQualityDeclaration {
            declaration_reference: "BCRG 0000/01".into(),
            generation_efficiency: 4.349,
            energy_fraction: Some(1.0),
            auxiliary_kwh_per_year: Some(30.0),
        });
        let declared = assess_forfait_heat_pump_draft(&input);
        assert_eq!(declared.status, "input_valid");
        assert!((declared.corrected_cop.unwrap() - 4.30).abs() < 1e-9);
        assert!(declared.corrected_cop.unwrap() > table);
        input
            .quality_declaration
            .as_mut()
            .unwrap()
            .declaration_reference = " ".into();
        input.quality_declaration.as_mut().unwrap().energy_fraction = Some(1.2);
        let invalid = assess_forfait_heat_pump_draft(&input);
        let codes: Vec<&str> = invalid.issues.iter().map(|item| item.code).collect();
        assert!(codes.contains(&"source_required"));
        assert!(codes.contains(&"heat_pump_declared_fraction_invalid"));
    }
}
