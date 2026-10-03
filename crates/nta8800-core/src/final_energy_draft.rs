//! Provisional final-energy arithmetic from the public 2026 chapter-5 consultation draft.
//! Inputs must already be monthly E_EPus values from an independently established
//! upstream route. This module neither derives those values nor proves the draft
//! survived unchanged in NTA 8800:2025+C1:2026.

use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::str::FromStr;

/// Final-edition citation of §5.9; the name is kept for the output field.
pub const DRAFT_SOURCE: &str = "NTA 8800:2025+C1:2026, §5.9, formules 5.57–5.60 (p. 133)";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FinalEnergyDraftInput {
    pub carrier_inventory_complete: bool,
    pub solar_thermal_inventory_complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_usable_floor_area_m2: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_source_reference: Option<String>,
    pub carriers: Vec<CarrierUse>,
    #[serde(default)]
    pub solar_thermal: Vec<SolarThermalUse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CarrierUse {
    pub carrier_code: String,
    pub source_reference: String,
    pub monthly_epus_kwh: Vec<MonthlyEnergy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SolarThermalUse {
    pub system_id: String,
    pub service: SolarService,
    pub source_reference: String,
    pub monthly_renewable_practice_kwh: Vec<MonthlyEnergy>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SolarService {
    SpaceHeating,
    DomesticHotWater,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonthlyEnergy {
    pub month: u8,
    pub energy_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalEnergyDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub annual_by_carrier_kwh: Vec<CarrierAnnual>,
    pub final_energy_kwh_per_year: Option<f64>,
    pub solar_thermal_kwh_per_year: Option<f64>,
    pub eed_final_energy_kwh_per_year: Option<f64>,
    pub final_energy_indicator_kwh_per_m2_year: Option<f64>,
    pub eed_final_energy_indicator_kwh_per_m2_year: Option<f64>,
    pub issues: Vec<DraftIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CarrierAnnual {
    pub carrier_code: String,
    pub energy_kwh_per_year: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> DraftIssue {
    DraftIssue {
        code,
        path: path.into(),
    }
}

pub(crate) fn decimal(value: f64) -> Option<Decimal> {
    Decimal::from_str(&value.to_string()).ok()
}

pub(crate) fn as_f64(value: Decimal) -> Option<f64> {
    value
        .to_string()
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
}

fn rounded_indicator(total: Decimal, area: Decimal) -> Option<f64> {
    as_f64(
        total
            .checked_div(area)?
            .round_dp_with_strategy(2, RoundingStrategy::ToPositiveInfinity),
    )
}

fn sum_months(
    months: &[MonthlyEnergy],
    path: &str,
    issues: &mut Vec<DraftIssue>,
) -> Option<Decimal> {
    let mut seen = HashSet::new();
    let mut sum = Decimal::ZERO;
    if months.len() != 12 {
        issues.push(issue("twelve_months_required", path));
    }
    for (index, value) in months.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        if !(1..=12).contains(&value.month) || !seen.insert(value.month) {
            issues.push(issue(
                "month_invalid_or_duplicate",
                format!("{item_path}.month"),
            ));
        }
        if !value.energy_kwh.is_finite() || value.energy_kwh < 0.0 {
            issues.push(issue(
                "monthly_energy_invalid",
                format!("{item_path}.energyKwh"),
            ));
        } else if let Some(value) = decimal(value.energy_kwh) {
            if let Some(next) = sum.checked_add(value) {
                sum = next;
            } else {
                issues.push(issue("annual_energy_overflow", path));
            }
        } else {
            issues.push(issue(
                "monthly_energy_decimal_range",
                format!("{item_path}.energyKwh"),
            ));
        }
    }
    if seen.len() == 12 && issues.is_empty() {
        Some(sum)
    } else {
        None
    }
}

pub fn assess_final_energy_draft(input: &FinalEnergyDraftInput) -> FinalEnergyDraftAssessment {
    let fingerprint = input_fingerprint(&json!(input));
    let mut issues = Vec::new();
    let mut seen_carriers = HashSet::new();
    if input.carriers.is_empty() {
        issues.push(issue("carrier_required", "carriers"));
    }
    let area = match (
        input.total_usable_floor_area_m2,
        &input.area_source_reference,
    ) {
        (None, None) => None,
        (Some(value), Some(source))
            if value.is_finite() && value > 0.0 && !source.trim().is_empty() =>
        {
            match decimal(value) {
                Some(area) => Some(area),
                None => {
                    issues.push(issue("area_decimal_range", "totalUsableFloorAreaM2"));
                    None
                }
            }
        }
        (Some(_), Some(source)) if source.trim().is_empty() => {
            issues.push(issue("area_source_required", "areaSourceReference"));
            None
        }
        (Some(_), None) => {
            issues.push(issue("area_source_required", "areaSourceReference"));
            None
        }
        (Some(_), Some(_)) => {
            issues.push(issue("area_invalid", "totalUsableFloorAreaM2"));
            None
        }
        (None, Some(_)) => {
            issues.push(issue("area_without_value", "areaSourceReference"));
            None
        }
    };
    let mut annual_by_carrier = Vec::new();
    let mut annual_decimals = Vec::new();
    for (index, carrier) in input.carriers.iter().enumerate() {
        let path = format!("carriers[{index}]");
        if carrier.carrier_code.trim().is_empty()
            || !seen_carriers.insert(carrier.carrier_code.as_str())
        {
            issues.push(issue("carrier_code_invalid", format!("{path}.carrierCode")));
        }
        if carrier.source_reference.trim().is_empty() {
            issues.push(issue("source_required", format!("{path}.sourceReference")));
        }
        if let Some(sum) = sum_months(
            &carrier.monthly_epus_kwh,
            &format!("{path}.monthlyEpusKwh"),
            &mut issues,
        ) {
            if let Some(value) = as_f64(sum) {
                annual_decimals.push(sum);
                annual_by_carrier.push(CarrierAnnual {
                    carrier_code: carrier.carrier_code.clone(),
                    energy_kwh_per_year: value,
                });
            } else {
                issues.push(issue("annual_energy_range", &path));
            }
        }
    }
    let mut seen_solar = HashSet::new();
    let mut solar_total = Decimal::ZERO;
    for (index, system) in input.solar_thermal.iter().enumerate() {
        let path = format!("solarThermal[{index}]");
        if system.system_id.trim().is_empty()
            || !seen_solar.insert((system.system_id.as_str(), system.service))
        {
            issues.push(issue("solar_system_invalid", format!("{path}.systemId")));
        }
        if system.source_reference.trim().is_empty() {
            issues.push(issue("source_required", format!("{path}.sourceReference")));
        }
        if let Some(sum) = sum_months(
            &system.monthly_renewable_practice_kwh,
            &format!("{path}.monthlyRenewablePracticeKwh"),
            &mut issues,
        ) {
            if let Some(next) = solar_total.checked_add(sum) {
                solar_total = next;
            } else {
                issues.push(issue("solar_energy_overflow", "solarThermal"));
            }
        }
    }
    if !issues.is_empty() {
        annual_by_carrier.clear();
    }
    let mut final_energy = None;
    let mut eed_final_energy = None;
    let mut solar_thermal = None;
    let mut final_indicator = None;
    let mut eed_final_indicator = None;
    if issues.is_empty() && input.carrier_inventory_complete {
        let sum = annual_decimals
            .iter()
            .try_fold(Decimal::ZERO, |total, value| total.checked_add(*value));
        if let Some(sum) = sum {
            final_energy = as_f64(sum);
            if let Some(area) = area {
                final_indicator = rounded_indicator(sum, area);
                if final_indicator.is_none() {
                    issues.push(issue("final_indicator_overflow", "totalUsableFloorAreaM2"));
                }
            }
            if input.solar_thermal_inventory_complete {
                if let Some(eed) = sum.checked_add(solar_total) {
                    solar_thermal = as_f64(solar_total);
                    eed_final_energy = as_f64(eed);
                    if let Some(area) = area {
                        eed_final_indicator = rounded_indicator(eed, area);
                        if eed_final_indicator.is_none() {
                            issues.push(issue("eed_indicator_overflow", "totalUsableFloorAreaM2"));
                        }
                    }
                } else {
                    issues.push(issue("eed_energy_overflow", "solarThermal"));
                }
            }
        } else {
            issues.push(issue("final_energy_overflow", "carriers"));
        }
    }
    if !issues.is_empty() {
        annual_by_carrier.clear();
        final_energy = None;
        solar_thermal = None;
        eed_final_energy = None;
        final_indicator = None;
        eed_final_indicator = None;
    }
    FinalEnergyDraftAssessment {
        status: if !issues.is_empty() {
            "invalid"
        } else if !input.carrier_inventory_complete || !input.solar_thermal_inventory_complete {
            "incomplete"
        } else {
            "input_valid"
        },
        scope: "public_chapter_5_draft_arithmetic_only",
        draft_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        annual_by_carrier_kwh: annual_by_carrier,
        final_energy_kwh_per_year: final_energy,
        solar_thermal_kwh_per_year: solar_thermal,
        eed_final_energy_kwh_per_year: eed_final_energy,
        final_energy_indicator_kwh_per_m2_year: final_indicator,
        eed_final_energy_indicator_kwh_per_m2_year: eed_final_indicator,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn months(value: f64) -> Vec<MonthlyEnergy> {
        (1..=12)
            .map(|month| MonthlyEnergy {
                month,
                energy_kwh: value,
            })
            .collect()
    }

    fn input() -> FinalEnergyDraftInput {
        FinalEnergyDraftInput {
            carrier_inventory_complete: true,
            solar_thermal_inventory_complete: true,
            total_usable_floor_area_m2: Some(100.0),
            area_source_reference: Some("synthetic supplied area".into()),
            carriers: vec![
                CarrierUse {
                    carrier_code: "el".into(),
                    source_reference: "synthetic EPus electricity".into(),
                    monthly_epus_kwh: months(100.0),
                },
                CarrierUse {
                    carrier_code: "gas".into(),
                    source_reference: "synthetic EPus gas".into(),
                    monthly_epus_kwh: months(200.0),
                },
            ],
            solar_thermal: vec![SolarThermalUse {
                system_id: "solar-1".into(),
                service: SolarService::DomesticHotWater,
                source_reference: "synthetic QW renewable practice".into(),
                monthly_renewable_practice_kwh: months(10.0),
            }],
        }
    }

    #[test]
    fn sums_supplied_monthly_carriers_and_solar_without_claiming_final_norm() {
        let result = assess_final_energy_draft(&input());
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.annual_by_carrier_kwh[0].energy_kwh_per_year, 1200.0);
        assert_eq!(result.annual_by_carrier_kwh[1].energy_kwh_per_year, 2400.0);
        assert_eq!(result.final_energy_kwh_per_year, Some(3600.0));
        assert_eq!(result.solar_thermal_kwh_per_year, Some(120.0));
        assert_eq!(result.eed_final_energy_kwh_per_year, Some(3720.0));
        assert_eq!(result.final_energy_indicator_kwh_per_m2_year, Some(36.0));
        assert_eq!(
            result.eed_final_energy_indicator_kwh_per_m2_year,
            Some(37.2)
        );
        assert!(!result.final_edition_verified && !result.reference_verified);
    }

    #[test]
    fn incomplete_inventories_do_not_invent_missing_flows() {
        let mut sample = input();
        sample.solar_thermal_inventory_complete = false;
        let partial = assess_final_energy_draft(&sample);
        assert_eq!(partial.status, "incomplete");
        assert_eq!(partial.final_energy_kwh_per_year, Some(3600.0));
        assert_eq!(partial.eed_final_energy_kwh_per_year, None);
        sample.carrier_inventory_complete = false;
        let incomplete = assess_final_energy_draft(&sample);
        assert_eq!(incomplete.status, "incomplete");
        assert_eq!(incomplete.final_energy_kwh_per_year, None);
    }

    #[test]
    fn duplicate_month_and_overflow_suppress_all_results() {
        let mut sample = input();
        sample.carriers[0].monthly_epus_kwh[1].month = 1;
        let invalid = assess_final_energy_draft(&sample);
        assert_eq!(invalid.status, "invalid");
        assert!(invalid.annual_by_carrier_kwh.is_empty());
        sample.carriers[0].monthly_epus_kwh[1].month = 2;
        sample.carriers[0].monthly_epus_kwh[0].energy_kwh = f64::MAX;
        sample.carriers[0].monthly_epus_kwh[1].energy_kwh = f64::MAX;
        let overflow = assess_final_energy_draft(&sample);
        assert_eq!(overflow.status, "invalid");
        assert!(overflow.final_energy_kwh_per_year.is_none());
    }

    #[test]
    fn decimal_ceiling_handles_exact_and_just_above_cent_boundary() {
        let mut sample = input();
        sample.carriers.truncate(1);
        sample.solar_thermal.clear();
        sample.total_usable_floor_area_m2 = Some(1.0);
        sample.carriers[0].monthly_epus_kwh = months(0.0);
        sample.carriers[0].monthly_epus_kwh[0].energy_kwh = 0.1;
        sample.carriers[0].monthly_epus_kwh[1].energy_kwh = 0.1;
        sample.carriers[0].monthly_epus_kwh[2].energy_kwh = 0.1;
        let exact = assess_final_energy_draft(&sample);
        assert_eq!(exact.final_energy_indicator_kwh_per_m2_year, Some(0.3));
        sample.carriers[0].monthly_epus_kwh[3].energy_kwh = 0.001;
        let above = assess_final_energy_draft(&sample);
        assert_eq!(above.final_energy_indicator_kwh_per_m2_year, Some(0.31));
        sample.total_usable_floor_area_m2 = Some(0.0);
        let invalid = assess_final_energy_draft(&sample);
        assert_eq!(invalid.status, "invalid");
        assert!(invalid.final_energy_indicator_kwh_per_m2_year.is_none());
    }
}
