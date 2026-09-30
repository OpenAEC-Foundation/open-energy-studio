//! Checks supplied EN 16147 tap-profile test inputs without deriving NTA annual performance.

use crate::{heat_pumps::HeatPumpInput, input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclaredDhwAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub heat_pump_id: String,
    pub reference_verified: bool,
    pub annual_performance_available: bool,
    pub beng_calculation_available: bool,
    pub profiles: Vec<DhwProfileDiagnostic>,
    pub issues: Vec<DhwIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DhwProfileDiagnostic {
    pub point_id: String,
    pub tap_profile: String,
    pub declaration_norm_version: String,
    pub declaration_edition_matches_target: bool,
    pub source_reference: String,
    pub useful_energy_kwh_per_day: f64,
    pub input_energy_kwh_per_day: f64,
    pub raw_useful_to_input_ratio: f64,
    pub practice_factor_input: f64,
    pub nominal_capacity_kw: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DhwIssue {
    pub code: &'static str,
    pub path: String,
}

pub fn assess_declared_dhw(pump: &HeatPumpInput) -> DeclaredDhwAssessment {
    let mut issues: Vec<_> = pump
        .validate_inputs()
        .into_iter()
        .map(|issue| DhwIssue {
            code: issue.code,
            path: issue.field,
        })
        .collect();
    if pump.dhw_test_points.is_empty() {
        issues.push(DhwIssue {
            code: "dhw_test_points_required",
            path: "dhwTestPoints".into(),
        });
    }
    let valid = issues.is_empty();
    DeclaredDhwAssessment {
        status: if valid { "input_valid" } else { "invalid" },
        scope: "declared_dhw_test_input_only",
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(pump)),
        heat_pump_id: pump.id.clone(),
        reference_verified: false,
        annual_performance_available: false,
        beng_calculation_available: false,
        profiles: if valid {
            pump.dhw_test_points
                .iter()
                .map(|point| DhwProfileDiagnostic {
                    point_id: point.id.clone(),
                    tap_profile: point.tap_profile.clone(),
                    declaration_norm_version: point.declaration_norm_version.clone(),
                    declaration_edition_matches_target: point.declaration_norm_version
                        == TARGET_NORM_VERSION,
                    source_reference: point.source_reference.clone(),
                    useful_energy_kwh_per_day: point.useful_energy_kwh_per_day,
                    input_energy_kwh_per_day: point.input_energy_kwh_per_day,
                    raw_useful_to_input_ratio: point.useful_energy_kwh_per_day
                        / point.input_energy_kwh_per_day,
                    practice_factor_input: point.practice_factor,
                    nominal_capacity_kw: point.nominal_capacity_kw,
                })
                .collect()
        } else {
            Vec::new()
        },
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bcrg_20260143gg_reports_raw_test_ratio_only() {
        let pump: HeatPumpInput = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20260143gg-dhw-test-input.json"
        ))
        .unwrap();
        let result = assess_declared_dhw(&pump);
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.profiles.len(), 2);
        assert!((result.profiles[0].raw_useful_to_input_ratio - 5.876 / 1.644).abs() < 1e-12);
        assert_eq!(result.profiles[0].practice_factor_input, 0.95);
        assert!(!result.profiles[0].declaration_edition_matches_target);
        assert!(!result.annual_performance_available);
    }

    #[test]
    fn invalid_points_never_emit_ratios() {
        let mut pump: HeatPumpInput = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20260143gg-dhw-test-input.json"
        ))
        .unwrap();
        pump.dhw_test_points[0].input_energy_kwh_per_day = 0.0;
        let result = assess_declared_dhw(&pump);
        assert_eq!(result.status, "invalid");
        assert!(result.profiles.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "dhw_test_value_invalid"));
    }
}
