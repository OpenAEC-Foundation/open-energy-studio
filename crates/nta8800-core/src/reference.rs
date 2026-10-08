//! Administrative checks for independent reference cases.
//! Completeness of a manifest never proves that its expected values are correct.

use crate::norm_versions::{self, NormVersion};
use crate::{assess_json, input_fingerprint, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceCase {
    pub case_id: String,
    /// The edition the expected values belong to: a label as stamped on
    /// results (`NTA 8800:2022`) or an input id (`2022`). The input must
    /// carry the same edition, so the case calculates in it.
    pub norm_version: String,
    /// What `project` holds; absent is a project file (`ntaCalculation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_kind: Option<ReferenceInputKind>,
    /// Full `.oes` project input; `null` when the case is a survey.
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub project: Value,
    /// ISSO basic survey input (EDR "Real B" tests) instead of a project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survey: Option<ReferenceSurvey>,
    pub source: ReferenceSource,
    /// Empty only for a case with `pending` expectations.
    #[serde(default)]
    pub expected: Vec<ExpectedMetric>,
    /// Optional comparison of the calculated, unregistered label class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_label_class: Option<String>,
    /// A test-set case whose expected values are not (yet) in hand, such as
    /// ISSO 54 without its results document: the case is calculated and
    /// the named metrics are recorded without a verdict. It must calculate;
    /// it can never pass a comparison.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<PendingExpectations>,
}

/// The metrics a pending case records, with the band they will be judged
/// against once the expected values are supplied (then they move to
/// `expected`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PendingExpectations {
    /// Why the expected values are missing, e.g. the results document of
    /// the test set is not in hand.
    pub reason: String,
    pub metrics: Vec<PendingMetric>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PendingMetric {
    pub path: String,
    pub unit: String,
    pub norm_reference: String,
    #[serde(default)]
    pub absolute_tolerance: f64,
    /// The band of the test set as a fraction (ISSO 54: 0,01).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_tolerance: Option<f64>,
}

/// A metric calculated for a pending case: no expectation, no verdict.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordedMetric {
    pub path: String,
    pub actual: f64,
    pub unit: String,
    pub absolute_tolerance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_tolerance: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceInputKind {
    /// A project file, calculated with `assess_project_performance`; its
    /// edition is `ntaCalculation.normVersion`.
    Project,
    /// A building-performance input, calculated with
    /// `assess_building_performance`; its edition is `normVersion`.
    BuildingPerformance,
}

impl ReferenceInputKind {
    fn edition_pointer(self) -> &'static str {
        match self {
            ReferenceInputKind::Project => "/ntaCalculation/normVersion",
            ReferenceInputKind::BuildingPerformance => "/normVersion",
        }
    }
}

/// The edition named by a manifest: an edition label or an input id.
pub fn reference_edition(norm_version: &str) -> Option<NormVersion> {
    NormVersion::ALL
        .into_iter()
        .find(|version| version.label() == norm_version || version.id() == norm_version)
}

/// A basisopname as the calculation input of a reference case. The opname
/// layer derives the kernel input; the comparison runs on its performance.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSurvey {
    pub kind: ReferenceSurveyKind,
    pub input: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceSurveyKind {
    /// ISSO 82.1 dwelling survey (`opname/residential`).
    Residential,
    /// ISSO 75.1 utility survey (`opname/utility`).
    Utility,
}

/// Runs the survey through the opname layer and returns the performance when
/// the opname is complete.
fn survey_performance(
    survey: &ReferenceSurvey,
) -> Result<
    (
        String,
        Option<crate::building_performance::BuildingPerformanceAssessment>,
    ),
    &'static str,
> {
    match survey.kind {
        ReferenceSurveyKind::Residential => {
            let input: crate::opname::ResidentialSurvey =
                serde_json::from_value(survey.input.clone()).map_err(|_| "survey_shape_invalid")?;
            let assessment = crate::opname::assess_residential_survey(&input);
            Ok((assessment.status.to_string(), assessment.performance))
        }
        ReferenceSurveyKind::Utility => {
            let input: crate::opname::utility::UtilitySurvey =
                serde_json::from_value(survey.input.clone()).map_err(|_| "survey_shape_invalid")?;
            let assessment = crate::opname::utility::assess_utility_survey(&input);
            Ok((assessment.status.to_string(), assessment.performance))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSource {
    pub publisher: String,
    pub document_id: String,
    pub edition: String,
    pub use_permission: String,
    pub independent_reviewer: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedMetric {
    pub path: String,
    pub value: f64,
    pub unit: String,
    pub norm_reference: String,
    pub absolute_tolerance: f64,
    /// Band as a fraction of the expected value (0,01 is 1 %). With both
    /// tolerances the wider one applies, as test sets give a percentage
    /// per subtest next to an absolute floor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_tolerance: Option<f64>,
}

impl ExpectedMetric {
    /// The band this metric is judged against: the wider of the absolute
    /// tolerance and the relative tolerance times |expected|.
    pub fn applied_tolerance(&self) -> f64 {
        let relative = self
            .relative_tolerance
            .map_or(0.0, |fraction| fraction * self.value.abs());
        self.absolute_tolerance.max(relative)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceAudit {
    pub case_id: String,
    /// Label of the case's edition; the kernel's own edition when the
    /// manifest names none it knows.
    pub target_norm_version: &'static str,
    pub manifest_fingerprint: String,
    pub manifest_complete: bool,
    pub reference_verified: bool,
    pub calculation_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_fingerprint: Option<String>,
    pub issues: Vec<ReferenceIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReferenceIssue {
    pub code: &'static str,
    pub path: String,
    pub message: &'static str,
}

/// Numeric comparison only. A matching submitted expectation is never proof
/// that the source is independent or that the kernel is attested.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceComparison {
    pub status: &'static str,
    pub case_id: String,
    /// Label of the edition the case was calculated in.
    pub target_norm_version: &'static str,
    pub manifest_fingerprint: String,
    pub input_fingerprint: Option<String>,
    pub calculation_available: bool,
    pub reference_verified: bool,
    pub attest_status: &'static str,
    pub metrics: Vec<MetricComparison>,
    /// The metrics of a pending case (status `pending_expectation`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recorded: Vec<RecordedMetric>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_class: Option<LabelClassComparison>,
    pub issues: Vec<ReferenceIssue>,
}

/// Statuses that do not fail a batch: a passed comparison, or a pending
/// case that calculated and recorded every named metric.
pub fn comparison_status_acceptable(status: &str) -> bool {
    matches!(status, "compared_pass" | "pending_expectation")
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelClassComparison {
    pub path: &'static str,
    pub expected: String,
    pub actual: String,
    pub matches: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricComparison {
    pub path: String,
    pub expected: f64,
    pub actual: f64,
    pub unit: String,
    pub absolute_difference: f64,
    pub absolute_tolerance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_tolerance: Option<f64>,
    /// The band actually applied: the wider of both tolerances.
    pub applied_tolerance: f64,
    pub within_tolerance: bool,
}

/// Output allowlist: input and metadata paths must never be compared to values
/// supplied in the same case manifest. A missing row is not assumed to be zero.
fn metric_unit(path: &str) -> Option<&'static str> {
    match path {
        "beng1" | "beng2" | "labelPrimaryFossil" | "heatingNeedPerM2" | "finalEnergyPerM2" => {
            return Some("kWh/m2.year")
        }
        "beng3" | "labelRenewableShare" => return Some("%"),
        "tojuliMax" => return Some("K"),
        "chapter5HeatingNeed" => return Some("kWh/m2.year"),
        "annualPrimaryFossil" | "annualRenewablePrimary" => return Some("kWh"),
        "annualCO2" => return Some("kg CO2eq"),
        _ => {}
    }
    let parts: Vec<_> = path.split('/').collect();
    if let [kind @ ("heatingMonth" | "coolingMonth"), month, field] = parts.as_slice() {
        let Ok(parsed_month) = month.parse::<u8>() else {
            return None;
        };
        if !(1..=12).contains(&parsed_month) || parsed_month.to_string() != *month {
            return None;
        }
        return match (*kind, *field) {
            ("heatingMonth", "heatingNeedKwh")
            | ("heatingMonth", "emissionInputKwh")
            | ("heatingMonth", "distributionLossKwh")
            | ("heatingMonth", "generatorOutputKwh")
            | ("heatingMonth", "heatPumpOutputKwh")
            | ("heatingMonth", "generatorElectricityKwh")
            | ("heatingMonth", "auxiliaryElectricityKwh")
            | ("heatingMonth", "naturalGasKwh")
            | ("heatingMonth", "districtHeatKwh")
            | ("heatingMonth", "collectiveSourceHeatKwh")
            | ("coolingMonth", "needKwh")
            | ("coolingMonth", "emissionLossKwh")
            | ("coolingMonth", "distributionLossKwh")
            | ("coolingMonth", "boosterExtractionKwh")
            | ("coolingMonth", "dehumidificationKwh")
            | ("coolingMonth", "ahuCoolingKwh")
            | ("coolingMonth", "generatorColdKwh")
            | ("coolingMonth", "electricityKwh")
            | ("coolingMonth", "naturalGasKwh")
            | ("coolingMonth", "districtHeatKwh")
            | ("coolingMonth", "districtColdKwh")
            | ("coolingMonth", "auxiliaryElectricityKwh")
            | ("coolingMonth", "ambientColdKwh") => Some("kWh"),
            _ => None,
        };
    }
    let (service, carrier, field) = match parts.as_slice() {
        ["serviceAnnual", service, carrier, field] => (*service, *carrier, *field),
        ["serviceMonth", service, carrier, month, field] => {
            let Ok(parsed_month) = month.parse::<u8>() else {
                return None;
            };
            if !(1..=12).contains(&parsed_month) || parsed_month.to_string() != *month {
                return None;
            }
            (*service, *carrier, *field)
        }
        _ => return None,
    };
    if !crate::building_performance::ENERGY_FUNCTIONS.contains(&service)
        || !crate::building_performance::BREAKDOWN_CARRIERS.contains(&carrier)
    {
        return None;
    }
    match field {
        "usedKwh" | "deliveredKwh" | "primaryFossilKwh" => Some("kWh"),
        _ => None,
    }
}

fn actual_metric(
    result: &crate::building_performance::BuildingPerformanceAssessment,
    path: &str,
) -> Option<f64> {
    match path {
        "beng1" => return result.need_indicator_kwh_per_m2_year,
        "beng2" => return result.primary_fossil_indicator_kwh_per_m2_year,
        "beng3" => return result.renewable_share_percent,
        "labelPrimaryFossil" => {
            return result.label_primary_fossil_indicator_kwh_per_m2_year;
        }
        "labelRenewableShare" => return result.label_renewable_share_percent,
        // ISSO 54 posts QH;nd;net and EweFinal (chapter 5 per m²).
        "heatingNeedPerM2" => {
            return result
                .chapter5
                .as_ref()
                .map(|chapter| chapter.heating_need_kwh_per_m2)
        }
        "finalEnergyPerM2" => {
            return result
                .chapter5
                .as_ref()
                .map(|chapter| chapter.final_energy_kwh_per_m2)
        }
        "tojuliMax" => return result.tojuli_max_k,
        "chapter5HeatingNeed" => {
            return result
                .chapter5
                .as_ref()
                .map(|indicators| indicators.heating_need_kwh_per_m2);
        }
        "annualPrimaryFossil" => return result.annual_primary_fossil_kwh,
        "annualRenewablePrimary" => return result.annual_renewable_primary_kwh,
        "annualCO2" => return result.annual_co2_kg,
        _ => {}
    }
    let parts: Vec<_> = path.split('/').collect();
    if let ["heatingMonth", month, field] = parts.as_slice() {
        let month = month.parse::<u8>().ok()?;
        let row = result
            .space_heating
            .monthly
            .iter()
            .find(|row| row.month == month)?;
        return match *field {
            "heatingNeedKwh" => Some(row.heating_need_kwh),
            "emissionInputKwh" => Some(row.emission_input_kwh),
            "distributionLossKwh" => Some(row.distribution_loss_kwh),
            "generatorOutputKwh" => Some(row.generator_output_kwh),
            "heatPumpOutputKwh" => Some(row.heat_pump_output_kwh),
            "generatorElectricityKwh" => Some(row.generator_electricity_kwh),
            "auxiliaryElectricityKwh" => row.auxiliary_electricity_kwh,
            "naturalGasKwh" => Some(row.natural_gas_kwh),
            "districtHeatKwh" => Some(row.district_heat_kwh),
            "collectiveSourceHeatKwh" => Some(row.collective_source_heat_kwh),
            _ => None,
        };
    }
    if let ["coolingMonth", month, field] = parts.as_slice() {
        let month = month.parse::<u8>().ok()?;
        let row = result
            .cooling
            .as_ref()?
            .months
            .iter()
            .find(|row| row.month == month)?;
        return match *field {
            "needKwh" => Some(row.need_kwh),
            "emissionLossKwh" => Some(row.emission_loss_kwh),
            "distributionLossKwh" => Some(row.distribution_loss_kwh),
            "boosterExtractionKwh" => Some(row.booster_extraction_kwh),
            "dehumidificationKwh" => Some(row.dehumidification_kwh),
            "ahuCoolingKwh" => Some(row.ahu_cooling_kwh),
            "generatorColdKwh" => Some(row.generator_cold_kwh),
            "electricityKwh" => Some(row.electricity_kwh),
            "naturalGasKwh" => Some(row.natural_gas_kwh),
            "districtHeatKwh" => Some(row.district_heat_kwh),
            "districtColdKwh" => Some(row.district_cold_kwh),
            "auxiliaryElectricityKwh" => Some(row.auxiliary_electricity_kwh),
            "ambientColdKwh" => Some(row.ambient_cold_kwh),
            _ => None,
        };
    }
    let row_value = |used: f64, delivered: f64, fossil: f64, field: &str| match field {
        "usedKwh" => Some(used),
        "deliveredKwh" => Some(delivered),
        "primaryFossilKwh" => Some(fossil),
        _ => None,
    };
    match parts.as_slice() {
        ["serviceAnnual", service, carrier, field] => result
            .energy_by_service
            .annual
            .iter()
            .find(|row| row.service == *service && row.carrier == *carrier)
            .and_then(|row| {
                row_value(
                    row.used_kwh,
                    row.delivered_kwh,
                    row.primary_fossil_kwh,
                    field,
                )
            }),
        ["serviceMonth", service, carrier, month, field] => {
            let month = month.parse::<u8>().ok()?;
            result
                .energy_by_service
                .months
                .iter()
                .find(|row| {
                    row.service == *service && row.carrier == *carrier && row.month == month
                })
                .and_then(|row| {
                    row_value(
                        row.used_kwh,
                        row.delivered_kwh,
                        row.primary_fossil_kwh,
                        field,
                    )
                })
        }
        _ => None,
    }
}

pub fn compare_reference_case(case: ReferenceCase) -> ReferenceComparison {
    let audit = audit_reference_case(case.clone());
    let mut result = ReferenceComparison {
        status: "invalid_case",
        case_id: audit.case_id,
        target_norm_version: audit.target_norm_version,
        manifest_fingerprint: audit.manifest_fingerprint,
        input_fingerprint: audit.input_fingerprint,
        calculation_available: false,
        reference_verified: false,
        attest_status: "unattested",
        metrics: Vec::new(),
        recorded: Vec::new(),
        label_class: None,
        issues: audit.issues,
    };
    if !audit.manifest_complete {
        return result;
    }
    let pending_metrics = case.pending.as_ref().map_or(&[][..], |item| &item.metrics);
    let named = case
        .expected
        .iter()
        .enumerate()
        .map(|(index, metric)| (format!("expected[{index}]"), &metric.path, &metric.unit))
        .chain(pending_metrics.iter().enumerate().map(|(index, metric)| {
            (
                format!("pending.metrics[{index}]"),
                &metric.path,
                &metric.unit,
            )
        }));
    for (at, path, unit_given) in named {
        let unit = metric_unit(path);
        if unit.is_none() {
            result.issues.push(issue(
                "metric_path_unsupported",
                format!("{at}.path"),
                "Only allowlisted numeric performance output paths can be compared",
            ));
        } else if unit != Some(unit_given.as_str()) {
            result.issues.push(issue(
                "metric_unit_mismatch",
                format!("{at}.unit"),
                "Metric unit does not match the kernel output unit",
            ));
        }
    }
    if !result.issues.is_empty() {
        return result;
    }
    let calculated = if let Some(survey) = &case.survey {
        match survey_performance(survey) {
            Ok((status, performance)) if status == "calculated_unverified" => performance,
            Ok(_) | Err(_) => {
                result.status = "calculation_unavailable";
                result.issues.push(issue(
                    "survey_calculation_unavailable",
                    "survey",
                    "Survey does not yield a complete unverified Rust calculation",
                ));
                return result;
            }
        }
    } else {
        match case.input_kind.unwrap_or(ReferenceInputKind::Project) {
            ReferenceInputKind::Project => {
                let assessment =
                    crate::project_performance::assess_project_performance(&case.project);
                result.input_fingerprint = Some(assessment.input_fingerprint);
                // An older edition calculates with the legacy status; the
                // comparison is the same, a registration is never implied.
                if matches!(
                    assessment.status,
                    "calculated_unverified" | "calculated_legacy_edition"
                ) {
                    assessment.performance
                } else {
                    None
                }
            }
            ReferenceInputKind::BuildingPerformance => {
                match serde_json::from_value::<crate::building_performance::BuildingPerformanceInput>(
                    case.project.clone(),
                ) {
                    Ok(input) => {
                        let assessment =
                            crate::building_performance::assess_building_performance(&input);
                        (assessment.status == "calculated_unverified").then_some(assessment)
                    }
                    Err(_) => None,
                }
            }
        }
    };
    let Some(performance) = calculated else {
        result.status = "calculation_unavailable";
        result.issues.push(issue(
            "project_calculation_unavailable",
            "project",
            "Project does not yield a complete unverified Rust calculation",
        ));
        return result;
    };
    result.calculation_available = true;
    if case.pending.is_some() {
        for (index, metric) in pending_metrics.iter().enumerate() {
            match record_pending_metric(metric, index, actual_metric(&performance, &metric.path)) {
                Ok(recorded) => result.recorded.push(recorded),
                Err(failure) => {
                    result.status = "calculation_unavailable";
                    result.issues.push(failure);
                    result.recorded.clear();
                    return result;
                }
            }
        }
        result.status = "pending_expectation";
        return result;
    }
    let mut metrics = Vec::with_capacity(case.expected.len());
    for (index, expected) in case.expected.iter().enumerate() {
        let Some(actual) = actual_metric(&performance, &expected.path) else {
            result.status = "calculation_unavailable";
            result.issues.push(issue(
                "metric_calculation_unavailable",
                format!("expected[{index}].path"),
                "Requested metric is unavailable for this project",
            ));
            return result;
        };
        let difference = (actual - expected.value).abs();
        if !difference.is_finite() {
            result.status = "invalid_case";
            result.issues.push(issue(
                "metric_difference_overflow",
                format!("expected[{index}].value"),
                "Absolute difference is not finite",
            ));
            return result;
        }
        let applied_tolerance = expected.applied_tolerance();
        metrics.push(MetricComparison {
            path: expected.path.clone(),
            expected: expected.value,
            actual,
            unit: expected.unit.clone(),
            absolute_difference: difference,
            absolute_tolerance: expected.absolute_tolerance,
            relative_tolerance: expected.relative_tolerance,
            applied_tolerance,
            within_tolerance: difference <= applied_tolerance,
        });
    }
    if let Some(expected) = case.expected_label_class {
        let Some(actual) = performance.indicative_label_class else {
            result.status = "calculation_unavailable";
            result.issues.push(issue(
                "label_class_calculation_unavailable",
                "expectedLabelClass",
                "Indicative label class is unavailable for this project",
            ));
            return result;
        };
        result.label_class = Some(LabelClassComparison {
            path: "indicativeLabelClass",
            matches: expected == actual,
            expected,
            actual: actual.into(),
        });
    }
    result.status = if metrics.iter().all(|metric| metric.within_tolerance)
        && result
            .label_class
            .as_ref()
            .map_or(true, |class| class.matches)
    {
        "compared_pass"
    } else {
        "compared_fail"
    };
    result.metrics = metrics;
    result
}

/// One recorded value of a pending case. A missing or non-finite value is a
/// failed calculation, never a recorded result.
fn record_pending_metric(
    metric: &PendingMetric,
    index: usize,
    actual: Option<f64>,
) -> Result<RecordedMetric, ReferenceIssue> {
    let path = format!("pending.metrics[{index}].path");
    let Some(actual) = actual else {
        return Err(issue(
            "metric_calculation_unavailable",
            path,
            "Requested metric is unavailable for this project",
        ));
    };
    if !actual.is_finite() {
        return Err(issue(
            "metric_not_finite",
            path,
            "Requested metric is not a finite number",
        ));
    }
    Ok(RecordedMetric {
        path: metric.path.clone(),
        actual,
        unit: metric.unit.clone(),
        absolute_tolerance: metric.absolute_tolerance,
        relative_tolerance: metric.relative_tolerance,
    })
}

fn issue(code: &'static str, path: impl Into<String>, message: &'static str) -> ReferenceIssue {
    ReferenceIssue {
        code,
        path: path.into(),
        message,
    }
}

pub fn audit_reference_case(case: ReferenceCase) -> ReferenceAudit {
    let manifest_fingerprint =
        input_fingerprint(&serde_json::to_value(&case).expect("reference case serializes as JSON"));
    let mut issues = Vec::new();
    if case.case_id.trim().is_empty() {
        issues.push(issue(
            "case_id_required",
            "caseId",
            "Reference case ID is required",
        ));
    }
    let kind = case.input_kind.unwrap_or(ReferenceInputKind::Project);
    let edition = reference_edition(&case.norm_version);
    match edition {
        None => issues.push(issue(
            "norm_version_mismatch",
            "normVersion",
            "Reference case targets a different NTA edition",
        )),
        // The input itself selects the edition it calculates in; a case
        // whose input names another edition would compare across editions.
        Some(version)
            if norm_versions::request::edition_at(&case.project, kind.edition_pointer())
                != version =>
        {
            issues.push(issue(
                "norm_version_mismatch",
                format!("project{}", kind.edition_pointer().replace('/', ".")),
                "Reference input calculates in a different NTA edition than the case",
            ))
        }
        Some(_) => {}
    }
    for (field, value) in [
        ("publisher", &case.source.publisher),
        ("documentId", &case.source.document_id),
        ("edition", &case.source.edition),
        ("usePermission", &case.source.use_permission),
        ("independentReviewer", &case.source.independent_reviewer),
    ] {
        if value.trim().is_empty() {
            issues.push(issue(
                "source_field_required",
                format!("source.{field}"),
                "Reference source provenance is required",
            ));
        }
    }
    match &case.pending {
        None if case.expected.is_empty() => issues.push(issue(
            "expected_metrics_required",
            "expected",
            "Independent expected values are required",
        )),
        None => {}
        // A pending case records, it never compares: expected values or a
        // label class next to it would make its status ambiguous.
        Some(_) if !case.expected.is_empty() || case.expected_label_class.is_some() => {
            issues.push(issue(
                "pending_with_expected_values",
                "pending",
                "A pending case gives no expected values; move the metrics to expected",
            ))
        }
        Some(pending) => {
            if pending.reason.trim().is_empty() {
                issues.push(issue(
                    "pending_reason_required",
                    "pending.reason",
                    "A pending case states why its expected values are missing",
                ));
            }
            if pending.metrics.is_empty() {
                issues.push(issue(
                    "pending_metrics_required",
                    "pending.metrics",
                    "A pending case names the metrics it records",
                ));
            }
            let mut paths = HashSet::new();
            for (index, metric) in pending.metrics.iter().enumerate() {
                let path = format!("pending.metrics[{index}]");
                if metric.path.trim().is_empty() || !paths.insert(metric.path.as_str()) {
                    issues.push(issue(
                        "metric_path_invalid",
                        format!("{path}.path"),
                        "Metric paths must be nonempty and unique",
                    ));
                }
                if metric.norm_reference.trim().is_empty() {
                    issues.push(issue(
                        "metric_norm_reference_required",
                        format!("{path}.normReference"),
                        "Metric requires an exact norm reference",
                    ));
                }
                if !metric.absolute_tolerance.is_finite() || metric.absolute_tolerance < 0.0 {
                    issues.push(issue(
                        "metric_tolerance_invalid",
                        format!("{path}.absoluteTolerance"),
                        "Absolute tolerance must be finite and nonnegative",
                    ));
                }
                if metric.relative_tolerance.is_some_and(|fraction| {
                    !fraction.is_finite() || !(0.0..=1.0).contains(&fraction)
                }) {
                    issues.push(issue(
                        "metric_tolerance_invalid",
                        format!("{path}.relativeTolerance"),
                        "Relative tolerance must be a finite fraction between 0 and 1",
                    ));
                }
            }
        }
    }
    if let Some(expected) = &case.expected_label_class {
        if expected.trim() != expected || crate::label_class::class_rank(expected).is_none() {
            issues.push(issue(
                "expected_label_class_invalid",
                "expectedLabelClass",
                "Expected label class must be a recognized class without surrounding whitespace",
            ));
        }
    }
    let mut metric_paths = HashSet::new();
    for (index, metric) in case.expected.iter().enumerate() {
        let path = format!("expected[{index}]");
        if metric.path.trim().is_empty() || !metric_paths.insert(metric.path.as_str()) {
            issues.push(issue(
                "metric_path_invalid",
                format!("{path}.path"),
                "Metric paths must be nonempty and unique",
            ));
        }
        if !metric.value.is_finite() {
            issues.push(issue(
                "expected_value_invalid",
                format!("{path}.value"),
                "Expected value must be finite",
            ));
        }
        if metric.unit.trim().is_empty() {
            issues.push(issue(
                "metric_unit_required",
                format!("{path}.unit"),
                "Metric unit is required",
            ));
        }
        if metric.norm_reference.trim().is_empty() {
            issues.push(issue(
                "metric_norm_reference_required",
                format!("{path}.normReference"),
                "Metric requires an exact norm reference",
            ));
        }
        if !metric.absolute_tolerance.is_finite() || metric.absolute_tolerance < 0.0 {
            issues.push(issue(
                "metric_tolerance_invalid",
                format!("{path}.absoluteTolerance"),
                "Absolute tolerance must be finite and nonnegative",
            ));
        }
        if metric
            .relative_tolerance
            .is_some_and(|fraction| !fraction.is_finite() || !(0.0..=1.0).contains(&fraction))
        {
            issues.push(issue(
                "metric_tolerance_invalid",
                format!("{path}.relativeTolerance"),
                "Relative tolerance must be a finite fraction between 0 and 1",
            ));
        }
    }
    let input_fingerprint = match (&case.survey, case.project.is_null()) {
        (Some(_), false) => {
            issues.push(issue(
                "calculation_input_ambiguous",
                "survey",
                "A reference case carries either a project or a survey, not both",
            ));
            None
        }
        (Some(survey), true) => {
            if survey_performance(survey).is_err() {
                issues.push(issue(
                    "survey_shape_invalid",
                    "survey.input",
                    "Reference survey shape cannot be parsed",
                ));
            }
            Some(input_fingerprint(&survey.input))
        }
        (None, true) => {
            issues.push(issue(
                "calculation_input_required",
                "project",
                "A reference case needs a project or a survey as calculation input",
            ));
            None
        }
        (None, false) => match kind {
            ReferenceInputKind::BuildingPerformance => {
                match serde_json::from_value::<crate::building_performance::BuildingPerformanceInput>(
                    case.project.clone(),
                ) {
                    Ok(_) => Some(input_fingerprint(&case.project)),
                    Err(_) => {
                        issues.push(issue(
                            "project_shape_invalid",
                            "project",
                            "Reference project shape cannot be parsed",
                        ));
                        None
                    }
                }
            }
            ReferenceInputKind::Project => match assess_json(case.project) {
                Ok(assessment) => {
                    if assessment.status == "invalid" {
                        issues.push(issue(
                            "project_input_invalid",
                            "project",
                            "Reference project fails structural validation",
                        ));
                    }
                    Some(assessment.input_fingerprint)
                }
                Err(_) => {
                    issues.push(issue(
                        "project_shape_invalid",
                        "project",
                        "Reference project shape cannot be parsed",
                    ));
                    None
                }
            },
        },
    };
    ReferenceAudit {
        case_id: case.case_id,
        target_norm_version: edition.map_or(TARGET_NORM_VERSION, NormVersion::label),
        manifest_fingerprint,
        manifest_complete: issues.is_empty(),
        reference_verified: false,
        calculation_available: false,
        input_fingerprint,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn project() -> Value {
        json!({"id":"p", "name":"Reference input", "buildingFunction":"residential",
            "zones":[{"id":"z", "floorArea":100, "volume":250, "surfaces":[{
                "id":"s", "area":50, "zoneId":"z", "windows":[]}]}]})
    }

    fn comparison_case(project: Value, path: &str, value: f64, unit: &str) -> ReferenceCase {
        ReferenceCase {
            case_id: "synthetic-comparison".into(),
            norm_version: TARGET_NORM_VERSION.into(),
            input_kind: None,
            project,
            survey: None,
            source: ReferenceSource {
                publisher: "synthetic internal test".into(),
                document_id: "internal-1".into(),
                edition: "test".into(),
                use_permission: "internal".into(),
                independent_reviewer: "test fixture".into(),
            },
            expected: vec![ExpectedMetric {
                path: path.into(),
                value,
                unit: unit.into(),
                norm_reference: "internal arithmetic test".into(),
                absolute_tolerance: 0.0,
                relative_tolerance: None,
            }],
            expected_label_class: None,
            pending: None,
        }
    }

    #[test]
    fn compares_only_calculated_output_and_never_verifies_reference() {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let assessment = crate::project_performance::assess_project_performance(&project);
        assert_eq!(
            assessment.status, "calculated_unverified",
            "{:?}",
            assessment.gaps
        );
        let actual = assessment
            .performance
            .unwrap()
            .primary_fossil_indicator_kwh_per_m2_year
            .unwrap();
        let pass = compare_reference_case(comparison_case(
            project.clone(),
            "beng2",
            actual,
            "kWh/m2.year",
        ));
        assert_eq!(pass.status, "compared_pass");
        assert_eq!(pass.metrics.len(), 1);
        assert!(!pass.reference_verified);
        assert_eq!(pass.attest_status, "unattested");

        let fail = compare_reference_case(comparison_case(
            project,
            "beng2",
            actual + 1.0,
            "kWh/m2.year",
        ));
        assert_eq!(fail.status, "compared_fail");
        assert!(!fail.metrics[0].within_tolerance);
    }

    #[test]
    fn compares_indicative_label_class_without_issuing_a_label() {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let mut reference = comparison_case(project, "beng2", 8.17, "kWh/m2.year");
        reference.expected_label_class = Some("A+++".into());
        let pass = compare_reference_case(reference.clone());
        assert_eq!(pass.status, "compared_pass");
        let class = pass.label_class.unwrap();
        assert_eq!(class.actual, "A+++");
        assert!(class.matches);
        assert!(!pass.reference_verified);

        reference.expected_label_class = Some("B".into());
        let mismatch = compare_reference_case(reference.clone());
        assert_eq!(mismatch.status, "compared_fail");
        assert!(!mismatch.label_class.unwrap().matches);

        reference.expected_label_class = Some("Z".into());
        assert!(!audit_reference_case(reference.clone()).manifest_complete);
        let invalid = compare_reference_case(reference);
        assert_eq!(invalid.status, "invalid_case");
        assert!(invalid.metrics.is_empty());
        assert!(invalid.label_class.is_none());
    }

    #[test]
    fn compares_label_scenario_indicators_separately_from_beng() {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let assessment = crate::project_performance::assess_project_performance(&project);
        let performance = assessment.performance.unwrap();
        let fossil = performance
            .label_primary_fossil_indicator_kwh_per_m2_year
            .unwrap();
        let renewable = performance.label_renewable_share_percent.unwrap();
        let mut case = comparison_case(project, "labelPrimaryFossil", fossil, "kWh/m2.year");
        case.expected.push(ExpectedMetric {
            path: "labelRenewableShare".into(),
            value: renewable,
            unit: "%".into(),
            norm_reference: "internal test".into(),
            absolute_tolerance: 0.0,
            relative_tolerance: None,
        });
        let pass = compare_reference_case(case.clone());
        assert_eq!(pass.status, "compared_pass");
        assert_eq!(pass.metrics.len(), 2);

        case.expected[0].value += 1.0;
        let mismatch = compare_reference_case(case.clone());
        assert_eq!(mismatch.status, "compared_fail");
        assert!(!mismatch.metrics[0].within_tolerance);

        case.expected[0].unit = "kWh".into();
        let invalid = compare_reference_case(case);
        assert_eq!(invalid.status, "invalid_case");
        assert!(invalid.metrics.is_empty());
    }

    #[test]
    fn compares_cooling_month_only_when_cooling_is_calculated() {
        let mut project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let missing = compare_reference_case(comparison_case(
            project.clone(),
            "coolingMonth/7/generatorColdKwh",
            0.0,
            "kWh",
        ));
        assert_eq!(missing.status, "calculation_unavailable");
        assert!(missing.metrics.is_empty());

        project["ntaCalculation"]["cooling"] = json!({
            "emission": {
                "emitter": "other_or_unknown",
                "balancing": "not_applicable",
                "control": "central_with_room_control",
                "sourceReference": "synthetic test"
            },
            "generators": [{
                "id": "cold",
                "generator": {"kind": "compression"},
                "equipmentReference": "synthetic test"
            }]
        });
        let assessment = crate::project_performance::assess_project_performance(&project);
        assert_eq!(
            assessment.status, "calculated_unverified",
            "{:?}",
            assessment.gaps
        );
        let performance = assessment.performance.unwrap();
        let july = performance
            .cooling
            .unwrap()
            .months
            .into_iter()
            .find(|month| month.month == 7)
            .unwrap();
        let mut case = comparison_case(
            project,
            "coolingMonth/7/generatorColdKwh",
            july.generator_cold_kwh,
            "kWh",
        );
        case.expected.push(ExpectedMetric {
            path: "coolingMonth/7/electricityKwh".into(),
            value: july.electricity_kwh,
            unit: "kWh".into(),
            norm_reference: "internal test".into(),
            absolute_tolerance: 0.0,
            relative_tolerance: None,
        });
        let pass = compare_reference_case(case.clone());
        assert_eq!(pass.status, "compared_pass");
        assert_eq!(pass.metrics.len(), 2);

        case.expected[1].value += 1.0;
        assert_eq!(compare_reference_case(case.clone()).status, "compared_fail");
        case.expected[0].path = "coolingMonth/07/generatorColdKwh".into();
        let invalid = compare_reference_case(case);
        assert_eq!(invalid.status, "invalid_case");
        assert!(invalid.metrics.is_empty());
    }

    #[test]
    fn comparison_rejects_input_paths_units_and_incomplete_projects() {
        let unsupported = compare_reference_case(comparison_case(
            project(),
            "derivedInput/floorArea",
            100.0,
            "m2",
        ));
        assert_eq!(unsupported.status, "invalid_case");
        assert!(unsupported.metrics.is_empty());
        let wrong_unit = compare_reference_case(comparison_case(project(), "beng2", 1.0, "kWh"));
        assert_eq!(wrong_unit.status, "invalid_case");
        let incomplete =
            compare_reference_case(comparison_case(project(), "beng2", 1.0, "kWh/m2.year"));
        assert_eq!(incomplete.status, "calculation_unavailable");
        assert!(!incomplete.calculation_available);
    }

    #[test]
    fn compares_service_and_carrier_month_without_array_index_assumptions() {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let assessment = crate::project_performance::assess_project_performance(&project);
        let performance = assessment.performance.unwrap();
        let row = performance.energy_by_service.months.first().unwrap();
        let path = format!(
            "serviceMonth/{}/{}/{}/usedKwh",
            row.service, row.carrier, row.month
        );
        let result =
            compare_reference_case(comparison_case(project.clone(), &path, row.used_kwh, "kWh"));
        assert_eq!(result.status, "compared_pass");
        let annual = performance.energy_by_service.annual.first().unwrap();
        let path = format!(
            "serviceAnnual/{}/{}/deliveredKwh",
            annual.service, annual.carrier
        );
        let result = compare_reference_case(comparison_case(
            project.clone(),
            &path,
            annual.delivered_kwh,
            "kWh",
        ));
        assert_eq!(result.status, "compared_pass");

        for path in [
            "serviceMonth/heating/el/0/usedKwh",
            "serviceMonth/heating/el/01/usedKwh",
            "serviceMonth/heating/el/13/usedKwh",
            "serviceMonth/heating/el/1/inputFingerprint",
            "serviceAnnual/unknown/el/usedKwh",
        ] {
            let result = compare_reference_case(comparison_case(project.clone(), path, 0.0, "kWh"));
            assert_eq!(result.status, "invalid_case", "{path}");
            assert!(result.metrics.is_empty());
        }
        let wrong_unit = compare_reference_case(comparison_case(project, "annualCO2", 0.0, "kWh"));
        assert_eq!(wrong_unit.status, "invalid_case");
    }

    #[test]
    fn compares_heating_chain_month_and_rejects_noncanonical_month() {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let assessment = crate::project_performance::assess_project_performance(&project);
        let performance = assessment.performance.unwrap();
        let month = performance.space_heating.monthly.first().unwrap();
        let path = format!("heatingMonth/{}/generatorOutputKwh", month.month);
        let pass = compare_reference_case(comparison_case(
            project.clone(),
            &path,
            month.generator_output_kwh,
            "kWh",
        ));
        assert_eq!(pass.status, "compared_pass");
        let fail = compare_reference_case(comparison_case(
            project.clone(),
            &path,
            month.generator_output_kwh + 1.0,
            "kWh",
        ));
        assert_eq!(fail.status, "compared_fail");
        assert!(!fail.reference_verified);
        for invalid in [
            "heatingMonth/00/generatorOutputKwh",
            "heatingMonth/13/generatorOutputKwh",
            "heatingMonth/1/inputFingerprint",
        ] {
            let result =
                compare_reference_case(comparison_case(project.clone(), invalid, 0.0, "kWh"));
            assert_eq!(result.status, "invalid_case", "{invalid}");
            assert!(result.metrics.is_empty());
        }
    }

    #[test]
    fn refuses_diagnostic_input_without_expected_values_or_provenance() {
        let case: ReferenceCase = serde_json::from_value(json!({
            "caseId":"diagnostic", "normVersion":TARGET_NORM_VERSION,
            "project":project(), "source":{"publisher":"","documentId":"",
                "edition":"","usePermission":"","independentReviewer":""},
            "expected":[]
        }))
        .unwrap();
        let audit = audit_reference_case(case);
        assert!(!audit.manifest_complete);
        assert!(!audit.reference_verified);
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.code == "expected_metrics_required"));
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.code == "source_field_required"));
    }

    #[test]
    fn complete_fields_do_not_claim_independent_verification() {
        let case: ReferenceCase = serde_json::from_value(json!({
            "caseId":"review-candidate", "normVersion":TARGET_NORM_VERSION,
            "project":project(), "source":{"publisher":"ISSO","documentId":"case-1",
                "edition":"2026","usePermission":"internal review","independentReviewer":"Reviewer"},
            "expected":[{"path":"beng1","value":42.0,"unit":"kWh/m2.year",
                "normReference":"NTA 8800:2025+C1:2026 §placeholder",
                "absoluteTolerance":0.1}]
        })).unwrap();
        let audit = audit_reference_case(case);
        assert!(audit.manifest_complete);
        assert!(!audit.reference_verified);
        assert!(!audit.calculation_available);
        assert!(audit.input_fingerprint.unwrap().starts_with("sha256:"));
    }

    #[test]
    fn detects_duplicate_metric_and_invalid_tolerance() {
        let case: ReferenceCase = serde_json::from_value(json!({
            "caseId":"x", "normVersion":TARGET_NORM_VERSION,
            "project":project(), "source":{"publisher":"ISSO","documentId":"x",
                "edition":"2026","usePermission":"internal","independentReviewer":"R"},
            "expected":[
                {"path":"beng1","value":42,"unit":"kWh","normReference":"§1","absoluteTolerance":0},
                {"path":"beng1","value":43,"unit":"kWh","normReference":"§1","absoluteTolerance":-1}
            ]
        }))
        .unwrap();
        let audit = audit_reference_case(case);
        assert!(!audit.manifest_complete);
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.code == "metric_path_invalid"));
        assert!(audit
            .issues
            .iter()
            .any(|issue| issue.code == "metric_tolerance_invalid"));
    }

    #[test]
    fn manifest_fingerprint_changes_with_expected_values_and_source() {
        let raw = json!({
            "caseId":"fingerprint-case", "normVersion":TARGET_NORM_VERSION,
            "project":project(), "source":{"publisher":"ISSO","documentId":"case-1",
                "edition":"2026","usePermission":"internal review","independentReviewer":"Reviewer"},
            "expected":[{"path":"beng1","value":42.0,"unit":"kWh/m2.year",
                "normReference":"test source", "absoluteTolerance":0.1}]
        });
        let base: ReferenceCase = serde_json::from_value(raw.clone()).unwrap();
        let project_fingerprint = audit_reference_case(base.clone()).input_fingerprint;
        let manifest_fingerprint = audit_reference_case(base).manifest_fingerprint;
        let mut expected_changed = raw.clone();
        expected_changed["expected"][0]["value"] = json!(43.0);
        let changed = audit_reference_case(serde_json::from_value(expected_changed).unwrap());
        assert_eq!(changed.input_fingerprint, project_fingerprint);
        assert_ne!(changed.manifest_fingerprint, manifest_fingerprint);
        let mut source_changed = raw;
        source_changed["source"]["documentId"] = json!("case-2");
        let changed = audit_reference_case(serde_json::from_value(source_changed).unwrap());
        assert_eq!(changed.input_fingerprint, project_fingerprint);
        assert_ne!(changed.manifest_fingerprint, manifest_fingerprint);
    }

    fn survey_case(path: &str, value: f64, unit: &str) -> ReferenceCase {
        let input: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-opname-1930-terraced.json"
        ))
        .unwrap();
        serde_json::from_value(json!({
            "caseId":"synthetic-survey", "normVersion":TARGET_NORM_VERSION,
            "survey":{"kind":"residential", "input":input},
            "source":{"publisher":"synthetic", "documentId":"internal-1", "edition":"test",
                "usePermission":"internal", "independentReviewer":"test"},
            "expected":[{"path":path, "value":value, "unit":unit,
                "normReference":"internal test", "absoluteTolerance":0.5}]
        }))
        .unwrap()
    }

    fn synthetic_project() -> Value {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap()
    }

    #[test]
    fn survey_case_runs_the_opname_layer_and_compares_chapter_5_posts() {
        let audit = audit_reference_case(survey_case("heatingNeedPerM2", 1.0, "kWh/m2.year"));
        assert!(audit.manifest_complete, "{:?}", audit.issues);
        assert!(audit.input_fingerprint.is_some());
        let comparison =
            compare_reference_case(survey_case("heatingNeedPerM2", 1.0, "kWh/m2.year"));
        assert_eq!(comparison.status, "compared_fail");
        assert!(comparison.calculation_available);
        assert!(!comparison.reference_verified);
        let actual = comparison.metrics[0].actual;
        assert!(actual > 50.0, "1930 terraced house heating need {actual}");
        let matched =
            compare_reference_case(survey_case("heatingNeedPerM2", actual, "kWh/m2.year"));
        assert_eq!(matched.status, "compared_pass");
        let final_energy =
            compare_reference_case(survey_case("finalEnergyPerM2", 0.0, "kWh/m2.year"));
        assert!(final_energy.metrics[0].actual > 0.0);
    }

    #[test]
    fn survey_and_project_are_mutually_exclusive_and_one_is_required() {
        let mut both = survey_case("beng2", 1.0, "kWh/m2.year");
        both.project = project();
        let audit = audit_reference_case(both);
        assert!(audit
            .issues
            .iter()
            .any(|item| item.code == "calculation_input_ambiguous"));
        let mut neither = survey_case("beng2", 1.0, "kWh/m2.year");
        neither.survey = None;
        let audit = audit_reference_case(neither);
        assert!(audit
            .issues
            .iter()
            .any(|item| item.code == "calculation_input_required"));
        let mut broken = survey_case("beng2", 1.0, "kWh/m2.year");
        broken.survey.as_mut().unwrap().input = json!({"id": "x"});
        let audit = audit_reference_case(broken);
        assert!(audit
            .issues
            .iter()
            .any(|item| item.code == "survey_shape_invalid"));
    }

    #[test]
    fn relative_tolerance_widens_the_band_and_the_wider_applies() {
        let project = synthetic_project();
        let beng2 = crate::project_performance::assess_project_performance(&project)
            .performance
            .unwrap()
            .primary_fossil_indicator_kwh_per_m2_year
            .unwrap();
        // 1 % above the result: outside an absolute band of 0, inside 1 %.
        let off = beng2 * 1.01;
        let mut case = comparison_case(project, "beng2", off, "kWh/m2.year");
        assert_eq!(compare_reference_case(case.clone()).status, "compared_fail");
        case.expected[0].relative_tolerance = Some(0.0101);
        let pass = compare_reference_case(case.clone());
        assert_eq!(pass.status, "compared_pass", "{:?}", pass.issues);
        let metric = &pass.metrics[0];
        assert!((metric.applied_tolerance - 0.0101 * off).abs() < 1e-12);
        assert_eq!(metric.relative_tolerance, Some(0.0101));
        // An absolute tolerance wider than the relative band wins.
        case.expected[0].relative_tolerance = Some(0.0001);
        case.expected[0].absolute_tolerance = 1.0;
        let wide = compare_reference_case(case.clone());
        assert_eq!(wide.status, "compared_pass");
        assert_eq!(wide.metrics[0].applied_tolerance, 1.0);
        // Without a relative tolerance the manifest and its fingerprint are
        // those of before relative tolerances existed.
        let plain =
            serde_json::to_value(comparison_case(json!({}), "beng2", 1.0, "kWh/m2.year")).unwrap();
        assert!(plain["expected"][0].get("relativeTolerance").is_none());
        assert!(plain.get("inputKind").is_none());
    }

    #[test]
    fn relative_tolerance_must_be_a_fraction() {
        for bad in [-0.01, 1.5, f64::NAN] {
            let mut case = comparison_case(project(), "beng1", 1.0, "kWh/m2.year");
            case.expected[0].relative_tolerance = Some(bad);
            let audit = audit_reference_case(case);
            assert!(!audit.manifest_complete, "{bad}");
            assert!(audit
                .issues
                .iter()
                .any(|issue| issue.path == "expected[0].relativeTolerance"));
        }
    }

    #[test]
    fn older_edition_compares_in_that_edition_only() {
        // Public case E is a project calculated in NTA 8800:2022.
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-public-comparison-e.json"
        ))
        .unwrap();
        assert_eq!(project["ntaCalculation"]["normVersion"], "2022");
        let beng1 = crate::project_performance::assess_project_performance(&project)
            .performance
            .unwrap()
            .need_indicator_kwh_per_m2_year
            .unwrap();
        let mut case = comparison_case(project.clone(), "beng1", beng1, "kWh/m2.year");
        // The kernel's own edition does not match an input in 2022.
        let mismatch = audit_reference_case(case.clone());
        assert!(mismatch
            .issues
            .iter()
            .any(|issue| issue.code == "norm_version_mismatch"
                && issue.path == "project.ntaCalculation.normVersion"));
        for label in ["NTA 8800:2022", "2022"] {
            case.norm_version = label.into();
            let pass = compare_reference_case(case.clone());
            assert_eq!(pass.status, "compared_pass", "{label}: {:?}", pass.issues);
            assert_eq!(pass.target_norm_version, "NTA 8800:2022");
            assert!(!pass.reference_verified);
        }
        case.norm_version = "NTA 8800:2019".into();
        assert!(!audit_reference_case(case).manifest_complete);
    }

    #[test]
    fn building_performance_input_compares_the_chapter_five_need() {
        let doc: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-rvo-voorbeeldwoningen-tussenwoning-1965-1974.json"
        ))
        .unwrap();
        let input = doc["performanceInput"].clone();
        let recorded = doc["regression"]["performanceInputHeatingNeedKwhPerM2"]
            .as_f64()
            .unwrap();
        let mut case = comparison_case(input, "chapter5HeatingNeed", recorded, "kWh/m2.year");
        case.input_kind = Some(ReferenceInputKind::BuildingPerformance);
        case.expected[0].relative_tolerance = Some(0.005);
        let audit = audit_reference_case(case.clone());
        assert!(audit.manifest_complete, "{:?}", audit.issues);
        let pass = compare_reference_case(case.clone());
        assert_eq!(pass.status, "compared_pass", "{:?}", pass.issues);
        // A project file under the building kind does not parse.
        case.project = project();
        assert!(!audit_reference_case(case).manifest_complete);
    }

    fn pending_case(project: Value) -> ReferenceCase {
        let mut case = comparison_case(project, "beng2", 0.0, "kWh/m2.year");
        case.expected.clear();
        case.pending = Some(PendingExpectations {
            reason: "results document of the test set not in hand".into(),
            metrics: vec![
                PendingMetric {
                    path: "heatingMonth/1/heatingNeedKwh".into(),
                    unit: "kWh".into(),
                    norm_reference: "heating need, January (7.2)".into(),
                    absolute_tolerance: 0.5,
                    relative_tolerance: None,
                },
                PendingMetric {
                    path: "beng2".into(),
                    unit: "kWh/m2.year".into(),
                    norm_reference: "BENG 2 (5.2)".into(),
                    absolute_tolerance: 0.0,
                    relative_tolerance: Some(0.01),
                },
            ],
        });
        case
    }

    #[test]
    fn pending_case_records_without_a_verdict_and_must_calculate() {
        let calculating: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let case = pending_case(calculating);
        let audit = audit_reference_case(case.clone());
        assert!(audit.manifest_complete, "{:?}", audit.issues);
        let recorded = compare_reference_case(case.clone());
        assert_eq!(
            recorded.status, "pending_expectation",
            "{:?}",
            recorded.issues
        );
        assert!(comparison_status_acceptable(recorded.status));
        assert!(recorded.metrics.is_empty());
        assert_eq!(recorded.recorded.len(), 2);
        assert_eq!(recorded.recorded[1].path, "beng2");
        assert!(recorded.recorded[1].actual.is_finite());
        assert_eq!(recorded.recorded[1].relative_tolerance, Some(0.01));
        assert!(!recorded.reference_verified);

        // A pending case that does not calculate fails like any other.
        let mut unavailable = case.clone();
        unavailable.project = project();
        let failed = compare_reference_case(unavailable);
        assert_eq!(failed.status, "calculation_unavailable");
        assert!(!comparison_status_acceptable(failed.status));
        assert!(failed.recorded.is_empty());

        // A recorded value must be finite; NaN or infinity is a failure.
        let metric = &case.pending.as_ref().unwrap().metrics[1];
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let refused = record_pending_metric(metric, 1, Some(bad)).unwrap_err();
            assert_eq!(refused.code, "metric_not_finite");
            assert_eq!(refused.path, "pending.metrics[1].path");
        }
        assert_eq!(
            record_pending_metric(metric, 1, None).unwrap_err().code,
            "metric_calculation_unavailable"
        );
        assert_eq!(
            record_pending_metric(metric, 1, Some(31.5)).unwrap().actual,
            31.5
        );

        // An output path outside the allowlist is refused before calculating.
        let mut unsupported = case.clone();
        unsupported.pending.as_mut().unwrap().metrics[0].path = "name".into();
        assert_eq!(compare_reference_case(unsupported).status, "invalid_case");
    }

    #[test]
    fn pending_case_rejects_expected_values_and_incomplete_pending_blocks() {
        let codes = |case: ReferenceCase| -> Vec<&'static str> {
            audit_reference_case(case)
                .issues
                .into_iter()
                .map(|issue| issue.code)
                .collect()
        };
        let mut with_expected = pending_case(project());
        with_expected.expected = comparison_case(project(), "beng2", 1.0, "kWh/m2.year").expected;
        assert!(codes(with_expected).contains(&"pending_with_expected_values"));
        let mut with_class = pending_case(project());
        with_class.expected_label_class = Some("A".into());
        assert!(codes(with_class).contains(&"pending_with_expected_values"));

        let mut blank = pending_case(project());
        let pending = blank.pending.as_mut().unwrap();
        pending.reason = " ".into();
        pending.metrics[1].path = pending.metrics[0].path.clone();
        pending.metrics[0].relative_tolerance = Some(1.5);
        let found = codes(blank);
        for code in [
            "pending_reason_required",
            "metric_path_invalid",
            "metric_tolerance_invalid",
        ] {
            assert!(found.contains(&code), "{code}: {found:?}");
        }
        let mut empty = pending_case(project());
        empty.pending.as_mut().unwrap().metrics.clear();
        assert!(codes(empty).contains(&"pending_metrics_required"));

        // Without a pending block, an empty expectation list stays refused.
        let mut neither = pending_case(project());
        neither.pending = None;
        assert!(codes(neither).contains(&"expected_metrics_required"));
    }
}
