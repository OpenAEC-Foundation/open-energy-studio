//! Provisional gas-driven heat-pump generator auxiliary electricity, draft §9.6.8.2.
//! Equation 9.91/9.92 only; month hours and generator heat are supplied inputs.

use crate::gas_heat_pump_forfait_draft::GasPumpDrive;
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

const DRAFT_SOURCE: &str = "https://www.internetconsultatie.nl/epg2026/document/14150";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasHeatPumpAuxDraftInput {
    pub generator_id: String,
    pub drive: GasPumpDrive,
    pub nominal_thermal_capacity_kw: f64,
    pub capacity_reference: String,
    pub standby_electronics_w: f64,
    pub burner_auxiliary_w_per_kw: f64,
    pub solution_pump_w_per_kw: f64,
    pub coefficients_reference: String,
    pub mean_modulation: f64,
    pub modulation_reference: String,
    pub building_share: f64,
    pub building_share_reference: String,
    pub forfait_cop_used: bool,
    pub month_hours_reference: String,
    pub generator_output_reference: String,
    pub months: Vec<GasHeatPumpAuxMonthInput>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasHeatPumpAuxMonthInput {
    pub month: u8,
    pub hours: f64,
    pub generator_output_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasHeatPumpAuxDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub monthly: Vec<GasHeatPumpAuxMonthResult>,
    pub annual_auxiliary_electricity_kwh: Option<f64>,
    pub gas_input_energy_available: bool,
    pub source_pump_or_fan_included: bool,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<GasAuxIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasHeatPumpAuxMonthResult {
    pub month: u8,
    pub capped_on_hours: f64,
    pub auxiliary_electricity_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasAuxIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> GasAuxIssue {
    GasAuxIssue {
        code,
        path: path.into(),
    }
}

pub fn assess_gas_heat_pump_aux_draft(
    input: &GasHeatPumpAuxDraftInput,
) -> GasHeatPumpAuxDraftAssessment {
    let mut issues = Vec::new();
    if input.generator_id.trim().is_empty() {
        issues.push(issue("generator_id_required", "generatorId"));
    }
    for (path, reference) in [
        ("capacityReference", &input.capacity_reference),
        ("coefficientsReference", &input.coefficients_reference),
        ("modulationReference", &input.modulation_reference),
        ("buildingShareReference", &input.building_share_reference),
        ("monthHoursReference", &input.month_hours_reference),
        (
            "generatorOutputReference",
            &input.generator_output_reference,
        ),
    ] {
        if reference.trim().is_empty() {
            issues.push(issue("source_required", path));
        }
    }
    let positive = |value: f64| value.is_finite() && value > 0.0;
    let nonnegative = |value: f64| value.is_finite() && value >= 0.0;
    if !positive(input.nominal_thermal_capacity_kw) {
        issues.push(issue("capacity_invalid", "nominalThermalCapacityKw"));
    }
    for (path, value) in [
        ("standbyElectronicsW", input.standby_electronics_w),
        ("burnerAuxiliaryWPerKw", input.burner_auxiliary_w_per_kw),
        ("solutionPumpWPerKw", input.solution_pump_w_per_kw),
    ] {
        if !nonnegative(value) {
            issues.push(issue("coefficient_invalid", path));
        }
    }
    if !positive(input.mean_modulation) || input.mean_modulation > 1.0 {
        issues.push(issue("modulation_invalid", "meanModulation"));
    }
    if !positive(input.building_share) || input.building_share > 1.0 {
        issues.push(issue("building_share_invalid", "buildingShare"));
    }
    if input.drive == GasPumpDrive::GasEngine && input.solution_pump_w_per_kw != 0.0 {
        issues.push(issue("solution_pump_not_applicable", "solutionPumpWPerKw"));
    }
    if input.forfait_cop_used && input.solution_pump_w_per_kw != 0.0 {
        issues.push(issue(
            "forfait_solution_pump_must_be_zero",
            "solutionPumpWPerKw",
        ));
    }
    if input.forfait_cop_used
        && (input.standby_electronics_w != 10.0
            || input.burner_auxiliary_w_per_kw != 1.0
            || input.mean_modulation != 1.0)
    {
        issues.push(issue(
            "forfait_coefficients_mismatch",
            "coefficientsReference",
        ));
    }
    if input.months.len() != 12 {
        issues.push(issue("twelve_months_required", "months"));
    }
    let mut seen = HashSet::new();
    for (index, month) in input.months.iter().enumerate() {
        if !(1..=12).contains(&month.month) || !seen.insert(month.month) {
            issues.push(issue(
                "month_invalid_or_duplicate",
                format!("months[{index}].month"),
            ));
        }
        if !positive(month.hours) || month.hours > 744.0 {
            issues.push(issue(
                "month_hours_invalid",
                format!("months[{index}].hours"),
            ));
        }
        if !nonnegative(month.generator_output_kwh) {
            issues.push(issue(
                "generator_output_invalid",
                format!("months[{index}].generatorOutputKwh"),
            ));
        }
    }
    let mut monthly = Vec::new();
    let mut annual = 0.0_f64;
    if issues.is_empty() {
        for (index, month) in input.months.iter().enumerate() {
            let on_hours = (month.generator_output_kwh * 1.1
                / (input.nominal_thermal_capacity_kw * input.mean_modulation))
                .min(month.hours);
            let electricity = (input.standby_electronics_w * month.hours
                + (input.burner_auxiliary_w_per_kw + input.solution_pump_w_per_kw)
                    * input.nominal_thermal_capacity_kw
                    * on_hours)
                * input.building_share
                / 1000.0;
            if !on_hours.is_finite() || !electricity.is_finite() || electricity < 0.0 {
                issues.push(issue("auxiliary_overflow", format!("months[{index}]")));
                break;
            }
            annual += electricity;
            if !annual.is_finite() {
                issues.push(issue("auxiliary_overflow", "months"));
                break;
            }
            monthly.push(GasHeatPumpAuxMonthResult {
                month: month.month,
                capped_on_hours: on_hours,
                auxiliary_electricity_kwh: electricity,
            });
        }
    }
    if !issues.is_empty() {
        monthly.clear();
    }
    GasHeatPumpAuxDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_9_91_9_92_gas_generator_auxiliary_only",
        draft_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        monthly,
        annual_auxiliary_electricity_kwh: if issues.is_empty() {
            Some(annual)
        } else {
            None
        },
        gas_input_energy_available: false,
        source_pump_or_fan_included: false,
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> GasHeatPumpAuxDraftInput {
        GasHeatPumpAuxDraftInput {
            generator_id: "ga-1".into(),
            drive: GasPumpDrive::Absorption,
            nominal_thermal_capacity_kw: 20.0,
            capacity_reference: "plate".into(),
            standby_electronics_w: 10.0,
            burner_auxiliary_w_per_kw: 1.0,
            solution_pump_w_per_kw: 0.0,
            coefficients_reference: "draft §9.6.8.2.3".into(),
            mean_modulation: 1.0,
            modulation_reference: "draft §9.6.8.2.3".into(),
            building_share: 1.0,
            building_share_reference: "whole building".into(),
            forfait_cop_used: true,
            month_hours_reference: "supplied month-hour schedule".into(),
            generator_output_reference: "supplied generator output".into(),
            months: (1..=12)
                .map(|month| GasHeatPumpAuxMonthInput {
                    month,
                    hours: 730.0,
                    generator_output_kwh: 1000.0,
                })
                .collect(),
        }
    }

    #[test]
    fn equation_991_separates_standby_and_burner_and_caps_runtime() {
        let mut input = sample();
        let result = assess_gas_heat_pump_aux_draft(&input);
        assert_eq!(result.status, "diagnostic_valid");
        assert_eq!(result.monthly[0].capped_on_hours, 55.0);
        assert_eq!(result.monthly[0].auxiliary_electricity_kwh, 8.4);
        assert!((result.annual_auxiliary_electricity_kwh.unwrap() - 100.8).abs() < 1e-10);
        assert!(!result.gas_input_energy_available && !result.beng_calculation_available);
        input.months[0].generator_output_kwh = 20000.0;
        let capped = assess_gas_heat_pump_aux_draft(&input);
        assert_eq!(capped.monthly[0].capped_on_hours, 730.0);
        assert_eq!(capped.monthly[0].auxiliary_electricity_kwh, 21.9);
    }

    #[test]
    fn forfait_absorption_excludes_solution_pump_and_invalid_cases_emit_no_partial_values() {
        let mut input = sample();
        input.solution_pump_w_per_kw = 10.0;
        let forbidden = assess_gas_heat_pump_aux_draft(&input);
        assert_eq!(forbidden.status, "invalid");
        assert!(
            forbidden.monthly.is_empty() && forbidden.annual_auxiliary_electricity_kwh.is_none()
        );
        input.forfait_cop_used = false;
        let measured = assess_gas_heat_pump_aux_draft(&input);
        assert_eq!(measured.monthly[0].auxiliary_electricity_kwh, 19.4);
        input.drive = GasPumpDrive::GasEngine;
        assert!(assess_gas_heat_pump_aux_draft(&input).monthly.is_empty());
        input.drive = GasPumpDrive::Absorption;
        input.months[1].month = 1;
        assert!(assess_gas_heat_pump_aux_draft(&input).monthly.is_empty());
    }
}
