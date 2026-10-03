//! Indicator arithmetic of NTA 8800:2025+C1:2026 §5.3.1, formulas 5.1–5.8
//! (p. 72–79), checked against the final edition; the `_draft` name is kept for
//! the API. The caller supplies independently derived annual totals and Ag;tot.
//! No label is calculated.

use crate::final_energy_draft::{as_f64, decimal};

/// Final-edition citation of the §5.3.1 indicators.
const DRAFT_SOURCE: &str = "NTA 8800:2025+C1:2026, §5.3.1, formules 5.1–5.8 (p. 72–79)";
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CalculationScope {
    Residential,
    Utility,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    Ordinary,
    EmgDeclaration,
    EmgForfait,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IndicatorsDraftInput {
    pub calculation_scope: CalculationScope,
    pub total_usable_floor_area_m2: f64,
    pub area_source_reference: String,
    pub annual_need_c1_kwh: f64,
    pub need_source_reference: String,
    pub scenarios: Vec<AnnualScenario>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnnualScenario {
    pub kind: ScenarioKind,
    pub annual_primary_fossil_kwh: f64,
    pub annual_renewable_kwh: f64,
    pub source_reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorsDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub label_available: bool,
    pub need_indicator_kwh_per_m2_year: Option<f64>,
    pub scenarios: Vec<ScenarioIndicator>,
    pub issues: Vec<IndicatorIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioIndicator {
    pub kind: ScenarioKind,
    pub primary_fossil_indicator_kwh_per_m2_year: f64,
    pub renewable_share_percent: f64,
    pub renewable_indicator_kwh_per_m2_year: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> IndicatorIssue {
    IndicatorIssue {
        code,
        path: path.into(),
    }
}

fn parse(
    value: f64,
    nonnegative: bool,
    path: &str,
    issues: &mut Vec<IndicatorIssue>,
) -> Option<Decimal> {
    if !value.is_finite() || (nonnegative && value < 0.0) {
        issues.push(issue("annual_input_invalid", path));
        return None;
    }
    match decimal(value) {
        Some(value) => Some(value),
        None => {
            issues.push(issue("annual_input_decimal_range", path));
            None
        }
    }
}

/// `value / area` rounded up to 0,01 in decimal arithmetic (as the BENG
/// indicators).
pub(crate) fn ceil_per_area(value: f64, area: f64) -> Option<f64> {
    if !value.is_finite() || !area.is_finite() || area <= 0.0 {
        return None;
    }
    rounded_div(
        decimal(value)?,
        decimal(area)?,
        2,
        RoundingStrategy::ToPositiveInfinity,
    )
}

fn rounded_div(
    numerator: Decimal,
    denominator: Decimal,
    digits: u32,
    strategy: RoundingStrategy,
) -> Option<f64> {
    as_f64(
        numerator
            .checked_div(denominator)?
            .round_dp_with_strategy(digits, strategy),
    )
}

pub fn assess_indicators_draft(input: &IndicatorsDraftInput) -> IndicatorsDraftAssessment {
    let mut issues = Vec::new();
    let area = parse(
        input.total_usable_floor_area_m2,
        true,
        "totalUsableFloorAreaM2",
        &mut issues,
    );
    if input.total_usable_floor_area_m2 <= 0.0 {
        issues.push(issue("area_positive_required", "totalUsableFloorAreaM2"));
    }
    let need = parse(
        input.annual_need_c1_kwh,
        true,
        "annualNeedC1Kwh",
        &mut issues,
    );
    if input.area_source_reference.trim().is_empty() {
        issues.push(issue("source_required", "areaSourceReference"));
    }
    if input.need_source_reference.trim().is_empty() {
        issues.push(issue("source_required", "needSourceReference"));
    }
    let kinds: HashSet<_> = input
        .scenarios
        .iter()
        .map(|scenario| scenario.kind)
        .collect();
    if input.scenarios.len() != kinds.len() {
        issues.push(issue("scenario_duplicate", "scenarios"));
    }
    let ordinary = kinds.len() == 1 && kinds.contains(&ScenarioKind::Ordinary);
    let emg_pair = kinds.len() == 2
        && kinds.contains(&ScenarioKind::EmgDeclaration)
        && kinds.contains(&ScenarioKind::EmgForfait);
    if !ordinary && !emg_pair {
        issues.push(issue("ordinary_or_emg_pair_required", "scenarios"));
    }
    let mut parsed = Vec::new();
    for (index, scenario) in input.scenarios.iter().enumerate() {
        let path = format!("scenarios[{index}]");
        if scenario.source_reference.trim().is_empty() {
            issues.push(issue("source_required", format!("{path}.sourceReference")));
        }
        let fossil = parse(
            scenario.annual_primary_fossil_kwh,
            false,
            &format!("{path}.annualPrimaryFossilKwh"),
            &mut issues,
        );
        let renewable = parse(
            scenario.annual_renewable_kwh,
            true,
            &format!("{path}.annualRenewableKwh"),
            &mut issues,
        );
        if let (Some(fossil), Some(renewable)) = (fossil, renewable) {
            match fossil.checked_add(renewable) {
                Some(sum) if sum > Decimal::ZERO => {
                    parsed.push((scenario.kind, fossil, renewable, sum))
                }
                Some(_) => {
                    issues.push(issue("renewable_share_denominator_positive_required", path))
                }
                None => issues.push(issue("annual_sum_overflow", path)),
            }
        }
    }
    let mut need_indicator = None;
    let mut scenarios = Vec::new();
    if issues.is_empty() {
        if let (Some(area), Some(need)) = (area, need) {
            need_indicator = rounded_div(need, area, 2, RoundingStrategy::ToPositiveInfinity);
            for (kind, fossil, renewable, total) in parsed {
                let fossil_indicator =
                    rounded_div(fossil, area, 2, RoundingStrategy::ToPositiveInfinity);
                let renewable_share = renewable
                    .checked_mul(Decimal::from(100))
                    .and_then(|v| rounded_div(v, total, 1, RoundingStrategy::ToNegativeInfinity));
                let renewable_indicator =
                    rounded_div(renewable, area, 2, RoundingStrategy::ToNegativeInfinity);
                if let (
                    Some(primary_fossil_indicator_kwh_per_m2_year),
                    Some(renewable_share_percent),
                    Some(renewable_indicator_kwh_per_m2_year),
                ) = (fossil_indicator, renewable_share, renewable_indicator)
                {
                    scenarios.push(ScenarioIndicator {
                        kind,
                        primary_fossil_indicator_kwh_per_m2_year,
                        renewable_share_percent,
                        renewable_indicator_kwh_per_m2_year,
                    });
                } else {
                    issues.push(issue("indicator_decimal_range", "scenarios"));
                    break;
                }
            }
            if need_indicator.is_none() {
                issues.push(issue("indicator_decimal_range", "annualNeedC1Kwh"));
            }
        }
    }
    if !issues.is_empty() {
        need_indicator = None;
        scenarios.clear();
    }
    IndicatorsDraftAssessment {
        status: if issues.is_empty() {
            "input_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_5_draft_indicators_from_supplied_annual_totals_only",
        draft_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        label_available: false,
        need_indicator_kwh_per_m2_year: need_indicator,
        scenarios,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> IndicatorsDraftInput {
        IndicatorsDraftInput {
            calculation_scope: CalculationScope::Residential,
            total_usable_floor_area_m2: 100.0,
            area_source_reference: "measured Ag".into(),
            annual_need_c1_kwh: 1234.001,
            need_source_reference: "independent C1 need".into(),
            scenarios: vec![AnnualScenario {
                kind: ScenarioKind::Ordinary,
                annual_primary_fossil_kwh: 2131.001,
                annual_renewable_kwh: 1000.0,
                source_reference: "independent annual totals".into(),
            }],
        }
    }
    #[test]
    fn applies_directional_rounding_without_enabling_label() {
        let result = assess_indicators_draft(&sample());
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.need_indicator_kwh_per_m2_year, Some(12.35));
        assert_eq!(
            result.scenarios[0].primary_fossil_indicator_kwh_per_m2_year,
            21.32
        );
        assert_eq!(
            result.scenarios[0].renewable_indicator_kwh_per_m2_year,
            10.0
        );
        assert_eq!(result.scenarios[0].renewable_share_percent, 31.9);
        assert!(!result.reference_verified && !result.label_available);
    }
    #[test]
    fn label_class_follows_the_rounded_indicator_at_class_edges() {
        use crate::label_class::{indicative_label_class, LabelFunction};
        let class_for = |fossil_kwh: f64, function: LabelFunction| {
            let mut input = sample();
            input.scenarios[0].annual_primary_fossil_kwh = fossil_kwh;
            let ep2 = assess_indicators_draft(&input).scenarios[0]
                .primary_fossil_indicator_kwh_per_m2_year;
            (ep2, indicative_label_class(function, ep2).unwrap())
        };
        // Annex IX: exactly on the A/B bound stays A; any excess rounds up
        // to 160,01 and gives B.
        assert_eq!(class_for(16000.0, LabelFunction::Residential), (160.0, "A"));
        assert_eq!(
            class_for(16000.001, LabelFunction::Residential),
            (160.01, "B")
        );
        assert_eq!(class_for(38000.0, LabelFunction::Residential), (380.0, "F"));
        assert_eq!(
            class_for(38000.0001, LabelFunction::Residential),
            (380.01, "G")
        );
        // Annex X office column, A++++ upper bound 40.
        assert_eq!(class_for(4000.0, LabelFunction::Office), (40.0, "A++++"));
        assert_eq!(class_for(4000.0001, LabelFunction::Office), (40.01, "A+++"));
    }

    #[test]
    fn requires_both_emg_scenarios_without_mixing_ordinary() {
        let mut input = sample();
        input.scenarios[0].kind = ScenarioKind::EmgDeclaration;
        assert_eq!(assess_indicators_draft(&input).status, "invalid");
        input.scenarios.push(AnnualScenario {
            kind: ScenarioKind::EmgForfait,
            annual_primary_fossil_kwh: 3000.0,
            annual_renewable_kwh: 500.0,
            source_reference: "forfait factors".into(),
        });
        let result = assess_indicators_draft(&input);
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.scenarios.len(), 2);
    }
    #[test]
    fn invalid_denominator_or_area_never_returns_partial_indicators() {
        let mut input = sample();
        input.scenarios[0].annual_primary_fossil_kwh = -1000.0;
        let result = assess_indicators_draft(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.scenarios.is_empty());
        input.scenarios[0].annual_primary_fossil_kwh = 100.0;
        input.total_usable_floor_area_m2 = 0.0;
        assert!(assess_indicators_draft(&input)
            .need_indicator_kwh_per_m2_year
            .is_none());
    }
}
