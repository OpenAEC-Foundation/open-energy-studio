//! NTA 8800:2025+C1:2026 §9.6.8.1.1 equation 9.85 (p. 357–359) for one
//! individual electric heat pump.
//! All coefficients and generator input electricity must be independently supplied.

use crate::final_energy_draft::{as_f64, decimal};
use crate::{input_fingerprint, KERNEL_VERSION};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

pub const DRAFT_SOURCE: &str = "NTA 8800:2025+C1:2026, §9.6.8.1.1, formule 9.85 (p. 357–359)";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatingAuxDraftInput {
    pub generator_id: String,
    pub generator_source_reference: String,
    pub coefficients: MeasuredCoefficients,
    pub input_energy_source_reference: String,
    pub months: Vec<GeneratorElectricityMonth>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeasuredCoefficients {
    /// Annual constant term from measured input, kWh/year; divided by 12 in 9.85.
    pub a_annual_kwh: f64,
    /// Measured coefficient B in kW, not the ambiguous consultation forfait.
    pub b_kw: f64,
    /// Mean compressor modulation C, dimensionless and greater than zero.
    pub c_dimensionless: f64,
    /// Nominal *electrical input* power of this generator, not useful heat power.
    pub nominal_electric_drive_kw: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratorElectricityMonth {
    pub month: u8,
    pub generator_input_electricity_kwh: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatingAuxMeasuredDraftInput {
    pub generator_id: String,
    pub generator_source_reference: String,
    pub measurements: ElectricHeatPumpAuxMeasurements,
    pub input_energy_source_reference: String,
    pub months: Vec<GeneratorElectricityMonth>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ElectricHeatPumpAuxMeasurements {
    pub standby_electronics_w: f64,
    pub delivery_pump_during_compressor_w: f64,
    pub delivery_pump_pre_post_w: f64,
    pub pump_pre_run_seconds: f64,
    pub pump_post_run_seconds: f64,
    pub average_compressor_on_seconds: f64,
    pub mean_compressor_modulation: f64,
    pub nominal_electric_drive_kw: f64,
    pub measurement_source_reference: String,
    pub timing_source_reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatingAuxMeasuredDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub primary_electricity_factor_used: f64,
    pub useful_pump_fraction_used: f64,
    pub derived_coefficients: Option<MeasuredCoefficients>,
    pub auxiliary: Option<HeatingAuxDraftAssessment>,
    pub issues: Vec<AuxIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatingAuxDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub annual_heat_pump_performance_available: bool,
    pub source_pump_and_fan_included: bool,
    pub forfait_used: bool,
    pub monthly_auxiliary_electricity_kwh: Vec<MonthlyAuxiliary>,
    pub annual_auxiliary_electricity_kwh: Option<f64>,
    pub issues: Vec<AuxIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyAuxiliary {
    pub month: u8,
    pub electricity_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuxIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> AuxIssue {
    AuxIssue {
        code,
        path: path.into(),
    }
}

fn number(value: f64, positive: bool, path: &str, issues: &mut Vec<AuxIssue>) -> Option<Decimal> {
    if !value.is_finite() || value < 0.0 || (positive && value == 0.0) {
        issues.push(issue("aux_input_invalid", path));
        return None;
    }
    match decimal(value) {
        Some(value) => Some(value),
        None => {
            issues.push(issue("aux_input_decimal_range", path));
            None
        }
    }
}

/// Consultation 9.86–9.88 with gas-valve and combustion-fan terms zero for an electric compressor.
pub fn assess_heating_aux_measured_draft(
    input: &HeatingAuxMeasuredDraftInput,
) -> HeatingAuxMeasuredDraftAssessment {
    let mut issues = Vec::new();
    let m = &input.measurements;
    if m.measurement_source_reference.trim().is_empty() {
        issues.push(issue(
            "source_required",
            "measurements.measurementSourceReference",
        ));
    }
    if m.timing_source_reference.trim().is_empty() {
        issues.push(issue(
            "source_required",
            "measurements.timingSourceReference",
        ));
    }
    let standby = number(
        m.standby_electronics_w,
        false,
        "measurements.standbyElectronicsW",
        &mut issues,
    );
    let pump_during = number(
        m.delivery_pump_during_compressor_w,
        false,
        "measurements.deliveryPumpDuringCompressorW",
        &mut issues,
    );
    let pump_pre_post = number(
        m.delivery_pump_pre_post_w,
        false,
        "measurements.deliveryPumpPrePostW",
        &mut issues,
    );
    let pre = number(
        m.pump_pre_run_seconds,
        false,
        "measurements.pumpPreRunSeconds",
        &mut issues,
    );
    let post = number(
        m.pump_post_run_seconds,
        false,
        "measurements.pumpPostRunSeconds",
        &mut issues,
    );
    let on = number(
        m.average_compressor_on_seconds,
        true,
        "measurements.averageCompressorOnSeconds",
        &mut issues,
    );
    let modulation = number(
        m.mean_compressor_modulation,
        true,
        "measurements.meanCompressorModulation",
        &mut issues,
    );
    if m.mean_compressor_modulation > 1.0 {
        issues.push(issue(
            "modulation_above_one",
            "measurements.meanCompressorModulation",
        ));
    }
    let nominal = number(
        m.nominal_electric_drive_kw,
        true,
        "measurements.nominalElectricDriveKw",
        &mut issues,
    );
    let mut derived = None;
    let mut auxiliary = None;
    if issues.is_empty() {
        if let (
            Some(standby),
            Some(pump_during),
            Some(pump_pre_post),
            Some(pre),
            Some(post),
            Some(on),
            Some(modulation),
            Some(nominal),
        ) = (
            standby,
            pump_during,
            pump_pre_post,
            pre,
            post,
            on,
            modulation,
            nominal,
        ) {
            let primary_factor = Decimal::new(145, 2);
            let useful_fraction = Decimal::new(5, 1);
            let a = standby
                .checked_mul(Decimal::from(8760))
                .and_then(|value| value.checked_div(Decimal::from(1000)));
            let b = useful_fraction
                .checked_div(primary_factor)
                .and_then(|ratio| Decimal::ONE.checked_sub(ratio))
                .and_then(|factor| {
                    pre.checked_add(post)
                        .and_then(|runtime| runtime.checked_div(on))
                        .and_then(|runtime| pump_pre_post.checked_mul(runtime))
                        .and_then(|pump_run| pump_during.checked_add(pump_run))
                        .and_then(|pump_total| factor.checked_mul(pump_total))
                })
                .and_then(|value| value.checked_div(Decimal::from(1000)));
            if let (
                Some(a),
                Some(b),
                Some(a_annual_kwh),
                Some(b_kw),
                Some(c_dimensionless),
                Some(nominal_electric_drive_kw),
            ) = (
                a,
                b,
                a.and_then(as_f64),
                b.and_then(as_f64),
                as_f64(modulation),
                as_f64(nominal),
            ) {
                if a >= Decimal::ZERO && b >= Decimal::ZERO {
                    let coefficients = MeasuredCoefficients {
                        a_annual_kwh,
                        b_kw,
                        c_dimensionless,
                        nominal_electric_drive_kw,
                        source_reference: format!(
                            "{}; {}",
                            m.measurement_source_reference, m.timing_source_reference
                        ),
                    };
                    let result = assess_heating_aux_draft(&HeatingAuxDraftInput {
                        generator_id: input.generator_id.clone(),
                        generator_source_reference: input.generator_source_reference.clone(),
                        coefficients: coefficients.clone(),
                        input_energy_source_reference: input.input_energy_source_reference.clone(),
                        months: input.months.clone(),
                    });
                    if result.status == "input_valid" {
                        derived = Some(coefficients);
                        auxiliary = Some(result);
                    } else {
                        issues.extend(
                            result
                                .issues
                                .iter()
                                .map(|item| issue(item.code, &item.path)),
                        );
                    }
                } else {
                    issues.push(issue("coefficient_arithmetic_invalid", "measurements"));
                }
            } else {
                issues.push(issue("coefficient_arithmetic_overflow", "measurements"));
            }
        }
    }
    HeatingAuxMeasuredDraftAssessment {
        status: if issues.is_empty() {
            "input_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_9_86_9_88_measured_electric_heat_pump_only",
        draft_source: DRAFT_SOURCE,
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        primary_electricity_factor_used: 1.45,
        useful_pump_fraction_used: 0.5,
        derived_coefficients: derived,
        auxiliary,
        issues,
    }
}

pub fn assess_heating_aux_draft(input: &HeatingAuxDraftInput) -> HeatingAuxDraftAssessment {
    let mut issues = Vec::new();
    if input.generator_id.trim().is_empty() {
        issues.push(issue("generator_id_required", "generatorId"));
    }
    for (path, reference) in [
        (
            "generatorSourceReference",
            input.generator_source_reference.as_str(),
        ),
        (
            "coefficients.sourceReference",
            input.coefficients.source_reference.as_str(),
        ),
        (
            "inputEnergySourceReference",
            input.input_energy_source_reference.as_str(),
        ),
    ] {
        if reference.trim().is_empty() {
            issues.push(issue("source_required", path));
        }
    }
    let coefficients = &input.coefficients;
    let a = number(
        coefficients.a_annual_kwh,
        false,
        "coefficients.aAnnualKwh",
        &mut issues,
    );
    let b = number(coefficients.b_kw, false, "coefficients.bKw", &mut issues);
    let c = number(
        coefficients.c_dimensionless,
        true,
        "coefficients.cDimensionless",
        &mut issues,
    );
    if coefficients.c_dimensionless > 1.0 {
        issues.push(issue("modulation_above_one", "coefficients.cDimensionless"));
    }
    let bnom = number(
        coefficients.nominal_electric_drive_kw,
        true,
        "coefficients.nominalElectricDriveKw",
        &mut issues,
    );
    if input.months.len() != 12 {
        issues.push(issue("twelve_months_required", "months"));
    }
    let mut seen = HashSet::new();
    let mut inputs = Vec::new();
    for (index, month) in input.months.iter().enumerate() {
        let path = format!("months[{index}]");
        if !(1..=12).contains(&month.month) || !seen.insert(month.month) {
            issues.push(issue("month_invalid_or_duplicate", format!("{path}.month")));
        }
        inputs.push(number(
            month.generator_input_electricity_kwh,
            false,
            &format!("{path}.generatorInputElectricityKwh"),
            &mut issues,
        ));
    }
    let mut monthly = Vec::new();
    let mut annual = None;
    if issues.is_empty() {
        if let (Some(a), Some(b), Some(c), Some(bnom)) = (a, b, c, bnom) {
            let denominator = c.checked_mul(bnom);
            let fixed = a.checked_div(Decimal::from(12));
            if let (Some(denominator), Some(fixed)) = (denominator, fixed) {
                let mut sum = Decimal::ZERO;
                for (month, energy) in input.months.iter().zip(inputs) {
                    let result = energy
                        .and_then(|energy| b.checked_mul(energy))
                        .and_then(|variable| variable.checked_div(denominator))
                        .and_then(|variable| fixed.checked_add(variable));
                    match result.and_then(|value| sum.checked_add(value).map(|next| (value, next)))
                    {
                        Some((value, next)) => match as_f64(value) {
                            Some(electricity_kwh) => {
                                monthly.push(MonthlyAuxiliary {
                                    month: month.month,
                                    electricity_kwh,
                                });
                                sum = next;
                            }
                            None => {
                                issues.push(issue("aux_arithmetic_overflow", "months"));
                                break;
                            }
                        },
                        None => {
                            issues.push(issue("aux_arithmetic_overflow", "months"));
                            break;
                        }
                    }
                }
                annual = as_f64(sum);
                if annual.is_none() {
                    issues.push(issue("aux_arithmetic_overflow", "months"));
                }
            } else {
                issues.push(issue("aux_arithmetic_overflow", "coefficients"));
            }
        }
    }
    if !issues.is_empty() {
        monthly.clear();
        annual = None;
    }
    HeatingAuxDraftAssessment {
        status: if issues.is_empty() {
            "input_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_9_85_measured_coefficients_one_individual_generator_only",
        draft_source: DRAFT_SOURCE,
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        annual_heat_pump_performance_available: false,
        source_pump_and_fan_included: false,
        forfait_used: false,
        monthly_auxiliary_electricity_kwh: monthly,
        annual_auxiliary_electricity_kwh: annual,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> HeatingAuxDraftInput {
        HeatingAuxDraftInput {
            generator_id: "hp-1".into(),
            generator_source_reference: "installation schedule".into(),
            coefficients: MeasuredCoefficients {
                a_annual_kwh: 60.0,
                b_kw: 0.1,
                c_dimensionless: 0.5,
                nominal_electric_drive_kw: 2.0,
                source_reference: "synthetic measured coefficient record".into(),
            },
            input_energy_source_reference: "separate generator input electricity".into(),
            months: (1..=12)
                .map(|month| GeneratorElectricityMonth {
                    month,
                    generator_input_electricity_kwh: 100.0,
                })
                .collect(),
        }
    }
    #[test]
    fn hand_case_applies_fixed_and_variable_terms_without_source_pump() {
        let result = assess_heating_aux_draft(&sample());
        assert_eq!(result.status, "input_valid");
        assert_eq!(
            result.monthly_auxiliary_electricity_kwh[0].electricity_kwh,
            15.0
        );
        assert_eq!(result.annual_auxiliary_electricity_kwh, Some(180.0));
        assert!(!result.source_pump_and_fan_included && !result.forfait_used);
        assert!(!result.beng_calculation_available);
    }
    #[test]
    fn zero_input_energy_still_has_standby_term() {
        let mut input = sample();
        input.months[0].generator_input_electricity_kwh = 0.0;
        let result = assess_heating_aux_draft(&input);
        assert_eq!(
            result.monthly_auxiliary_electricity_kwh[0].electricity_kwh,
            5.0
        );
        assert_eq!(result.annual_auxiliary_electricity_kwh, Some(170.0));
    }
    #[test]
    fn incomplete_or_invalid_input_yields_no_partial_electricity() {
        let mut input = sample();
        input.months.pop();
        input.coefficients.c_dimensionless = 0.0;
        let result = assess_heating_aux_draft(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.monthly_auxiliary_electricity_kwh.is_empty());
        assert!(result.annual_auxiliary_electricity_kwh.is_none());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "twelve_months_required"));
    }

    #[test]
    fn measured_components_derive_a_b_c_then_monthly_auxiliary() {
        let input = HeatingAuxMeasuredDraftInput {
            generator_id: "hp-1".into(),
            generator_source_reference: "schedule".into(),
            measurements: ElectricHeatPumpAuxMeasurements {
                standby_electronics_w: 10.0,
                delivery_pump_during_compressor_w: 200.0,
                delivery_pump_pre_post_w: 90.0,
                pump_pre_run_seconds: 300.0,
                pump_post_run_seconds: 300.0,
                average_compressor_on_seconds: 600.0,
                mean_compressor_modulation: 0.5,
                nominal_electric_drive_kw: 2.0,
                measurement_source_reference: "synthetic meter sheet".into(),
                timing_source_reference: "synthetic cycle sheet".into(),
            },
            input_energy_source_reference: "generator energy".into(),
            months: sample().months,
        };
        let result = assess_heating_aux_measured_draft(&input);
        assert_eq!(result.status, "input_valid");
        let coefficients = result.derived_coefficients.unwrap();
        assert_eq!(coefficients.a_annual_kwh, 87.6);
        assert!((coefficients.b_kw - 0.19).abs() < 1e-12);
        assert_eq!(coefficients.c_dimensionless, 0.5);
        let auxiliary = result.auxiliary.unwrap();
        assert!(
            (auxiliary.monthly_auxiliary_electricity_kwh[0].electricity_kwh - 26.3).abs() < 1e-10
        );
        assert!((auxiliary.annual_auxiliary_electricity_kwh.unwrap() - 315.6).abs() < 1e-9);
        assert!(!auxiliary.source_pump_and_fan_included);
    }
    #[test]
    fn measured_components_require_positive_cycle_time_and_complete_months() {
        let mut input = HeatingAuxMeasuredDraftInput {
            generator_id: "hp-1".into(),
            generator_source_reference: "schedule".into(),
            measurements: ElectricHeatPumpAuxMeasurements {
                standby_electronics_w: 10.0,
                delivery_pump_during_compressor_w: 200.0,
                delivery_pump_pre_post_w: 90.0,
                pump_pre_run_seconds: 300.0,
                pump_post_run_seconds: 300.0,
                average_compressor_on_seconds: 0.0,
                mean_compressor_modulation: 0.5,
                nominal_electric_drive_kw: 2.0,
                measurement_source_reference: "meter".into(),
                timing_source_reference: "cycle".into(),
            },
            input_energy_source_reference: "generator energy".into(),
            months: sample().months,
        };
        let result = assess_heating_aux_measured_draft(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.derived_coefficients.is_none() && result.auxiliary.is_none());
        input.measurements.average_compressor_on_seconds = 600.0;
        input.months.pop();
        let result = assess_heating_aux_measured_draft(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.derived_coefficients.is_none() && result.auxiliary.is_none());
    }
}
