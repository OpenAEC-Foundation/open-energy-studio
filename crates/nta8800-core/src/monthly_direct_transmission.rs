//! Signed monthly heat-flow diagnostic for explicit direct-to-outdoor conductance.
//! This is dimensional accounting (W/K × K × h / 1000 = kWh), not the NTA
//! heating/cooling demand, climate profile, ground route or BENG calculation.

use crate::direct_transmission::{assess_direct_transmission, DirectTransmissionInput};
use crate::{KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonthlyDirectInput {
    pub direct: DirectTransmissionInput,
    pub months: Vec<MonthlyConditions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonthlyConditions {
    pub month: u8,
    pub indoor_temperature_c: f64,
    pub outdoor_temperature_c: f64,
    pub hours: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyHeatFlow {
    pub month: u8,
    pub temperature_difference_k: f64,
    pub signed_heat_flow_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthlyIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyDirectAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub direct_conductance_w_per_k: Option<f64>,
    pub annual_signed_heat_flow_kwh: Option<f64>,
    pub monthly: Vec<MonthlyHeatFlow>,
    pub issues: Vec<MonthlyIssue>,
}

fn issue(code: &'static str, path: impl Into<String>) -> MonthlyIssue {
    MonthlyIssue {
        code,
        path: path.into(),
    }
}

pub fn assess_monthly_direct(input: &MonthlyDirectInput) -> MonthlyDirectAssessment {
    let direct = assess_direct_transmission(&input.direct);
    let mut issues = direct
        .issues
        .iter()
        .map(|item| issue(item.code, format!("direct.{}", item.path)))
        .collect::<Vec<_>>();
    if input.months.len() != 12 {
        issues.push(issue("monthly_twelve_months_required", "months"));
    }
    let mut seen = HashSet::new();
    let mut monthly = Vec::with_capacity(input.months.len());
    let mut annual = 0.0;
    for (index, conditions) in input.months.iter().enumerate() {
        let path = format!("months[{index}]");
        if !(1..=12).contains(&conditions.month) || !seen.insert(conditions.month) {
            issues.push(issue("monthly_month_invalid", format!("{path}.month")));
        }
        if !conditions.indoor_temperature_c.is_finite()
            || !conditions.outdoor_temperature_c.is_finite()
        {
            issues.push(issue("monthly_temperature_invalid", path.clone()));
        }
        if !conditions.hours.is_finite() || conditions.hours <= 0.0 {
            issues.push(issue("monthly_hours_invalid", format!("{path}.hours")));
        }
        if let Some(conductance) = direct.total_direct_conductance_w_per_k {
            let delta = conditions.indoor_temperature_c - conditions.outdoor_temperature_c;
            let flow = conductance * delta * conditions.hours / 1000.0;
            if !delta.is_finite() || !flow.is_finite() {
                issues.push(issue("monthly_flow_invalid", path));
            } else {
                annual += flow;
                monthly.push(MonthlyHeatFlow {
                    month: conditions.month,
                    temperature_difference_k: delta,
                    signed_heat_flow_kwh: flow,
                });
            }
        }
    }
    if !annual.is_finite() {
        issues.push(issue("monthly_annual_sum_invalid", "months"));
    }
    let valid = issues.is_empty();
    let input_fingerprint = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(input).expect("typed input serializes"))
    );
    MonthlyDirectAssessment {
        status: if valid { "input_valid" } else { "invalid" },
        scope: "diagnostic_direct_outdoor_signed_heat_flow_only",
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint,
        reference_verified: false,
        beng_calculation_available: false,
        direct_conductance_w_per_k: valid
            .then_some(direct.total_direct_conductance_w_per_k)
            .flatten(),
        annual_signed_heat_flow_kwh: valid.then_some(annual),
        monthly: if valid { monthly } else { Vec::new() },
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direct_transmission::DirectElement;

    fn example() -> MonthlyDirectInput {
        MonthlyDirectInput {
            direct: DirectTransmissionInput {
                elements: vec![DirectElement {
                    id: "wall".into(),
                    area_m2: 10.0,
                    u_value_w_per_m2k: 0.2,
                    source_reference: "drawing".into(),
                }],
                linear_bridges: Vec::new(),
                point_bridges: Vec::new(),
            },
            months: (1..=12)
                .map(|month| MonthlyConditions {
                    month,
                    indoor_temperature_c: 20.0,
                    outdoor_temperature_c: 10.0,
                    hours: 100.0,
                })
                .collect(),
        }
    }

    #[test]
    fn sums_explicit_signed_monthly_flow_without_claiming_demand() {
        let mut input = example();
        input.months[6].outdoor_temperature_c = 30.0;
        let result = assess_monthly_direct(&input);
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.direct_conductance_w_per_k, Some(2.0));
        assert_eq!(result.monthly[0].signed_heat_flow_kwh, 2.0);
        assert_eq!(result.monthly[6].signed_heat_flow_kwh, -2.0);
        assert_eq!(result.annual_signed_heat_flow_kwh, Some(20.0));
        assert!(!result.reference_verified && !result.beng_calculation_available);
    }

    #[test]
    fn incomplete_or_duplicate_months_emit_no_energy() {
        let mut input = example();
        input.months[11].month = 1;
        let result = assess_monthly_direct(&input);
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "monthly_month_invalid"));
        assert!(result.monthly.is_empty());
        assert!(result.annual_signed_heat_flow_kwh.is_none());
        input.months.pop();
        assert!(assess_monthly_direct(&input)
            .issues
            .iter()
            .any(|issue| issue.code == "monthly_twelve_months_required"));
    }

    #[test]
    fn invalid_conductance_and_overflow_emit_no_energy() {
        let mut input = example();
        input.direct.elements[0].u_value_w_per_m2k = 0.0;
        assert!(assess_monthly_direct(&input).monthly.is_empty());
        input.direct.elements[0].u_value_w_per_m2k = 1e307;
        input.months[0].hours = 1e308;
        let result = assess_monthly_direct(&input);
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "monthly_flow_invalid"));
        assert!(result.annual_signed_heat_flow_kwh.is_none());
    }
}
