//! Provisional monthly E_EPus composition from consultation equations 5.20–5.21.
//! Each service input is supplied by the caller; no installation route is inferred.

use crate::bacs_draft::{assess_bacs_draft, BacsDraftInput};
use crate::final_energy_draft::{
    as_f64, assess_final_energy_draft, decimal, CarrierUse, FinalEnergyDraftAssessment,
    FinalEnergyDraftInput, MonthlyEnergy, SolarThermalUse, DRAFT_SOURCE,
};
use crate::gas_collective_source_draft::{
    assess_gas_collective_source_draft, GasCollectiveSourceDraftInput,
};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

const CARRIERS: [&str; 7] = ["el", "gas", "bm", "dh", "dw", "dc", "oil"];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EpusDraftInput {
    pub bacs_factor: f64,
    pub bacs_source_reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bacs_evidence: Option<BacsDraftInput>,
    pub carrier_inventory_complete: bool,
    pub solar_thermal_inventory_complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_usable_floor_area_m2: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_source_reference: Option<String>,
    pub carriers: Vec<CarrierServiceInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_collective_source_evidence: Option<GasCollectiveSourceDraftInput>,
    #[serde(default)]
    pub solar_thermal: Vec<SolarThermalUse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CarrierServiceInput {
    pub carrier_code: String,
    pub source_reference: String,
    pub months: Vec<MonthlyServiceUse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonthlyServiceUse {
    pub month: u8,
    pub space_heating_kwh: f64,
    pub humidification_kwh: f64,
    pub ventilation_kwh: f64,
    pub lighting_kwh: f64,
    pub space_cooling_kwh: f64,
    pub dehumidification_kwh: f64,
    pub domestic_hot_water_kwh: f64,
    pub collective_source_heat_kwh: f64,
    pub auxiliary: AuxiliaryUse,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuxiliaryUse {
    pub space_heating_kwh: f64,
    pub humidification_kwh: f64,
    pub space_cooling_kwh: f64,
    pub dehumidification_kwh: f64,
    pub domestic_hot_water_kwh: f64,
    pub solar_pv_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpusDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub monthly_by_carrier_kwh: Vec<MonthlyCarrierResult>,
    pub gas_collective_source_derived: bool,
    pub gas_collective_source_fingerprint: Option<String>,
    pub final_energy: Option<FinalEnergyDraftAssessment>,
    pub issues: Vec<EpusIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyCarrierResult {
    pub carrier_code: String,
    pub month: u8,
    pub energy_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpusIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> EpusIssue {
    EpusIssue {
        code,
        path: path.into(),
    }
}

fn term(value: f64, path: &str, issues: &mut Vec<EpusIssue>) -> Option<Decimal> {
    if !value.is_finite() || value < 0.0 {
        issues.push(issue("service_energy_invalid", path));
        None
    } else if let Some(value) = decimal(value) {
        Some(value)
    } else {
        issues.push(issue("service_energy_decimal_range", path));
        None
    }
}

fn compose_month(
    row: &MonthlyServiceUse,
    path: &str,
    carrier: &str,
    bacs: Decimal,
    issues: &mut Vec<EpusIssue>,
) -> Option<Decimal> {
    let values = [
        ("spaceHeatingKwh", row.space_heating_kwh),
        ("humidificationKwh", row.humidification_kwh),
        ("ventilationKwh", row.ventilation_kwh),
        ("lightingKwh", row.lighting_kwh),
        ("spaceCoolingKwh", row.space_cooling_kwh),
        ("dehumidificationKwh", row.dehumidification_kwh),
        ("domesticHotWaterKwh", row.domestic_hot_water_kwh),
        ("collectiveSourceHeatKwh", row.collective_source_heat_kwh),
        ("auxiliary.spaceHeatingKwh", row.auxiliary.space_heating_kwh),
        (
            "auxiliary.humidificationKwh",
            row.auxiliary.humidification_kwh,
        ),
        ("auxiliary.spaceCoolingKwh", row.auxiliary.space_cooling_kwh),
        (
            "auxiliary.dehumidificationKwh",
            row.auxiliary.dehumidification_kwh,
        ),
        (
            "auxiliary.domesticHotWaterKwh",
            row.auxiliary.domestic_hot_water_kwh,
        ),
        ("auxiliary.solarPvKwh", row.auxiliary.solar_pv_kwh),
    ];
    let mut parsed = Vec::with_capacity(values.len());
    for (field, value) in values {
        parsed.push(term(value, &format!("{path}.{field}"), issues)?);
    }
    if carrier != "el"
        && (row.ventilation_kwh != 0.0
            || row.lighting_kwh != 0.0
            || parsed[8..].iter().any(|value| !value.is_zero()))
    {
        issues.push(issue("electric_service_carrier_required", path));
    }
    if carrier != "dh" && row.collective_source_heat_kwh != 0.0 {
        issues.push(issue(
            "collective_source_carrier_required",
            format!("{path}.collectiveSourceHeatKwh"),
        ));
    }
    if row.dehumidification_kwh != 0.0
        || row.auxiliary.humidification_kwh != 0.0
        || row.auxiliary.dehumidification_kwh != 0.0
    {
        issues.push(issue("draft_zero_term_required", path));
    }
    let weighted = [0usize, 4, 8, 10];
    let mut sum = Decimal::ZERO;
    for (index, value) in parsed.iter().enumerate() {
        let contribution = if weighted.contains(&index) {
            value.checked_mul(bacs)
        } else {
            Some(*value)
        };
        sum = sum.checked_add(contribution?)?;
    }
    Some(sum)
}

pub fn assess_epus_draft(input: &EpusDraftInput) -> EpusDraftAssessment {
    let fingerprint = input_fingerprint(&json!(input));
    let mut issues = Vec::new();
    let bacs = if input.bacs_factor == 1.0 || input.bacs_factor == 1.05 {
        decimal(input.bacs_factor)
    } else {
        issues.push(issue("bacs_factor_outside_draft_values", "bacsFactor"));
        None
    };
    if input.bacs_source_reference.trim().is_empty() {
        issues.push(issue("bacs_source_required", "bacsSourceReference"));
    }
    if let Some(evidence) = &input.bacs_evidence {
        let assessment = assess_bacs_draft(evidence);
        match assessment.factor {
            Some(factor) if factor == input.bacs_factor => {}
            Some(_) => issues.push(issue("bacs_factor_evidence_mismatch", "bacsFactor")),
            None => issues.push(issue("bacs_evidence_incomplete_or_invalid", "bacsEvidence")),
        }
    }
    if input.carriers.is_empty() {
        issues.push(issue("carrier_required", "carriers"));
    }
    let derived_source = input
        .gas_collective_source_evidence
        .as_ref()
        .and_then(|evidence| {
            let assessment = assess_gas_collective_source_draft(evidence);
            if assessment.status != "diagnostic_valid" {
                issues.push(issue(
                    "gas_collective_source_evidence_invalid",
                    "gasCollectiveSourceEvidence",
                ));
                None
            } else if !input.carriers.iter().any(|item| item.carrier_code == "dh") {
                issues.push(issue(
                    "gas_collective_source_dh_carrier_required",
                    "carriers",
                ));
                None
            } else {
                Some(assessment)
            }
        });
    let mut seen_carriers = HashSet::new();
    let mut monthly_results = Vec::new();
    let mut final_carriers = Vec::new();
    for (carrier_index, carrier) in input.carriers.iter().enumerate() {
        let path = format!("carriers[{carrier_index}]");
        if !CARRIERS.contains(&carrier.carrier_code.as_str())
            || !seen_carriers.insert(carrier.carrier_code.as_str())
        {
            issues.push(issue(
                "carrier_invalid_or_duplicate",
                format!("{path}.carrierCode"),
            ));
        }
        if carrier.source_reference.trim().is_empty() {
            issues.push(issue("source_required", format!("{path}.sourceReference")));
        }
        if carrier.months.len() != 12 {
            issues.push(issue("twelve_months_required", format!("{path}.months")));
        }
        let mut seen_months = HashSet::new();
        let mut monthly = Vec::new();
        for (month_index, row) in carrier.months.iter().enumerate() {
            let row_path = format!("{path}.months[{month_index}]");
            if !(1..=12).contains(&row.month) || !seen_months.insert(row.month) {
                issues.push(issue(
                    "month_invalid_or_duplicate",
                    format!("{row_path}.month"),
                ));
            }
            if let Some(bacs) = bacs {
                let mut composed = row.clone();
                if carrier.carrier_code == "dh" {
                    if let Some(source) = &derived_source {
                        if row.collective_source_heat_kwh != 0.0 {
                            issues.push(issue(
                                "gas_collective_source_double_count",
                                format!("{row_path}.collectiveSourceHeatKwh"),
                            ));
                        } else if let Some(derived_month) =
                            source.monthly.iter().find(|item| item.month == row.month)
                        {
                            composed.collective_source_heat_kwh =
                                derived_month.delivered_source_heat_dh_kwh;
                        }
                    }
                }
                let prior_issue_count = issues.len();
                match compose_month(
                    &composed,
                    &row_path,
                    &carrier.carrier_code,
                    bacs,
                    &mut issues,
                )
                .and_then(as_f64)
                {
                    Some(value) => {
                        monthly.push(MonthlyEnergy {
                            month: row.month,
                            energy_kwh: value,
                        });
                        monthly_results.push(MonthlyCarrierResult {
                            carrier_code: carrier.carrier_code.clone(),
                            month: row.month,
                            energy_kwh: value,
                        });
                    }
                    None if issues.len() == prior_issue_count => {
                        issues.push(issue("monthly_composition_overflow", row_path));
                    }
                    None => {}
                }
            }
        }
        final_carriers.push(CarrierUse {
            carrier_code: carrier.carrier_code.clone(),
            source_reference: carrier.source_reference.clone(),
            monthly_epus_kwh: monthly,
        });
    }
    let final_energy = if issues.is_empty() {
        let result = assess_final_energy_draft(&FinalEnergyDraftInput {
            carrier_inventory_complete: input.carrier_inventory_complete,
            solar_thermal_inventory_complete: input.solar_thermal_inventory_complete,
            total_usable_floor_area_m2: input.total_usable_floor_area_m2,
            area_source_reference: input.area_source_reference.clone(),
            carriers: final_carriers,
            solar_thermal: input.solar_thermal.clone(),
        });
        if result.status == "invalid" {
            issues.extend(
                result
                    .issues
                    .iter()
                    .map(|item| issue(item.code, &item.path)),
            );
            None
        } else {
            Some(result)
        }
    } else {
        None
    };
    if !issues.is_empty() {
        monthly_results.clear();
    }
    EpusDraftAssessment {
        status: if !issues.is_empty() {
            "invalid"
        } else if final_energy
            .as_ref()
            .is_some_and(|result| result.status == "incomplete")
        {
            "incomplete"
        } else {
            "input_valid"
        },
        scope: "public_chapter_5_draft_service_composition_only",
        draft_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        monthly_by_carrier_kwh: monthly_results,
        gas_collective_source_derived: derived_source.is_some() && issues.is_empty(),
        gas_collective_source_fingerprint: if issues.is_empty() {
            derived_source
                .as_ref()
                .map(|source| source.input_fingerprint.clone())
        } else {
            None
        },
        final_energy,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bacs_draft::{
        BacsClass, BacsEvidence, BuildingUse, Generator, SystemService, ThermalSystem,
    };
    use serde_json::{json, Value};

    fn month(month: u8) -> MonthlyServiceUse {
        MonthlyServiceUse {
            month,
            space_heating_kwh: 100.0,
            humidification_kwh: 0.0,
            ventilation_kwh: 5.0,
            lighting_kwh: 0.0,
            space_cooling_kwh: 20.0,
            dehumidification_kwh: 0.0,
            domestic_hot_water_kwh: 30.0,
            collective_source_heat_kwh: 0.0,
            auxiliary: AuxiliaryUse {
                space_heating_kwh: 10.0,
                humidification_kwh: 0.0,
                space_cooling_kwh: 2.0,
                dehumidification_kwh: 0.0,
                domestic_hot_water_kwh: 3.0,
                solar_pv_kwh: 1.0,
            },
        }
    }

    fn input() -> EpusDraftInput {
        EpusDraftInput {
            bacs_evidence: None,
            bacs_factor: 1.05,
            bacs_source_reference: "synthetic draft BACS factor".into(),
            carrier_inventory_complete: true,
            solar_thermal_inventory_complete: true,
            total_usable_floor_area_m2: Some(100.0),
            area_source_reference: Some("synthetic area".into()),
            carriers: vec![CarrierServiceInput {
                carrier_code: "el".into(),
                source_reference: "synthetic service inputs".into(),
                months: (1..=12).map(month).collect(),
            }],
            gas_collective_source_evidence: None,
            solar_thermal: Vec::new(),
        }
    }

    fn collective_source_evidence() -> GasCollectiveSourceDraftInput {
        let mut chain: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-gas-chain-diagnostic-synthetic.json"
        ))
        .unwrap();
        let mut chain = chain["input"].take();
        chain["monthly"]["forfait"]["source"] = json!("groundwater_aquifer");
        chain["monthly"]["sourceSystem"] = json!("collective_groundwater_surface_or_at_least15_c");
        serde_json::from_value(json!({
            "chain": chain,
            "sourceTemperatureClass": "below20_c",
            "sourceTemperatureReference": "source design",
            "noQualityDeclarationConfirmed": true,
            "noQualityDeclarationReference": "register check"
        }))
        .unwrap()
    }

    fn empty_dh_month(month: u8) -> MonthlyServiceUse {
        MonthlyServiceUse {
            month,
            space_heating_kwh: 0.0,
            humidification_kwh: 0.0,
            ventilation_kwh: 0.0,
            lighting_kwh: 0.0,
            space_cooling_kwh: 0.0,
            dehumidification_kwh: 0.0,
            domestic_hot_water_kwh: 0.0,
            collective_source_heat_kwh: 0.0,
            auxiliary: AuxiliaryUse {
                space_heating_kwh: 0.0,
                humidification_kwh: 0.0,
                space_cooling_kwh: 0.0,
                dehumidification_kwh: 0.0,
                domestic_hot_water_kwh: 0.0,
                solar_pv_kwh: 0.0,
            },
        }
    }

    #[test]
    fn derives_collective_gas_source_once_on_dh_without_weighting_it_by_bacs() {
        let mut sample = input();
        sample.gas_collective_source_evidence = Some(collective_source_evidence());
        sample.carriers.push(CarrierServiceInput {
            carrier_code: "dh".into(),
            source_reference: "source invoice".into(),
            months: (1..=12).map(empty_dh_month).collect(),
        });
        let result = assess_epus_draft(&sample);
        assert_eq!(
            result.status,
            "input_valid",
            "{:?}",
            result
                .issues
                .iter()
                .map(|issue| issue.code)
                .collect::<Vec<_>>()
        );
        assert!(result.gas_collective_source_derived);
        assert!(result.gas_collective_source_fingerprint.is_some());
        let dh = result
            .monthly_by_carrier_kwh
            .iter()
            .filter(|item| item.carrier_code == "dh")
            .collect::<Vec<_>>();
        assert_eq!(dh.len(), 12);
        assert!((dh[0].energy_kwh - 1000.0 * (1.0 - 1.0 / 2.1)).abs() < 1e-8);
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn rejects_duplicate_dh_source_or_missing_dh_carrier_without_partial_energy() {
        let mut sample = input();
        sample.gas_collective_source_evidence = Some(collective_source_evidence());
        let missing = assess_epus_draft(&sample);
        assert_eq!(missing.status, "invalid");
        assert!(missing.monthly_by_carrier_kwh.is_empty());
        assert!(missing
            .issues
            .iter()
            .any(|issue| issue.code == "gas_collective_source_dh_carrier_required"));
        sample.carriers.push(CarrierServiceInput {
            carrier_code: "dh".into(),
            source_reference: "source invoice".into(),
            months: (1..=12).map(empty_dh_month).collect(),
        });
        sample.carriers[1].months[0].collective_source_heat_kwh = 100.0;
        let duplicate = assess_epus_draft(&sample);
        assert_eq!(duplicate.status, "invalid");
        assert!(duplicate.monthly_by_carrier_kwh.is_empty());
        assert!(duplicate.final_energy.is_none());
        assert!(duplicate
            .issues
            .iter()
            .any(|issue| issue.code == "gas_collective_source_double_count"));
    }

    #[test]
    fn composes_bacs_weighted_services_and_annual_final_energy() {
        let result = assess_epus_draft(&input());
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.monthly_by_carrier_kwh[0].energy_kwh, 177.6);
        let final_energy = result.final_energy.unwrap();
        assert_eq!(final_energy.final_energy_kwh_per_year, Some(2131.2));
        assert_eq!(
            final_energy.final_energy_indicator_kwh_per_m2_year,
            Some(21.32)
        );
        assert!(!result.final_edition_verified && !result.reference_verified);
    }

    #[test]
    fn rejects_wrong_carrier_missing_month_and_forbidden_draft_terms_without_partial_result() {
        let mut sample = input();
        sample.carriers[0].carrier_code = "gas".into();
        sample.carriers[0].months.pop();
        sample.carriers[0].months[0].dehumidification_kwh = 1.0;
        let result = assess_epus_draft(&sample);
        assert_eq!(result.status, "invalid");
        assert!(result.monthly_by_carrier_kwh.is_empty());
        assert!(result.final_energy.is_none());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "electric_service_carrier_required"));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "twelve_months_required"));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "draft_zero_term_required"));
    }

    #[test]
    fn bacs_evidence_must_agree_with_supplied_factor() {
        let mut sample = input();
        sample.bacs_evidence = Some(BacsDraftInput {
            building_use: BuildingUse::Utility,
            system_inventory_complete: true,
            systems: vec![ThermalSystem {
                id: "heat".into(),
                service: SystemService::Heating,
                source_reference: "schedule".into(),
                bacs: None,
                generators: vec![Generator {
                    id: "boiler".into(),
                    nominal_thermal_capacity_kw: Some(291.0),
                    source_reference: "nameplate".into(),
                }],
            }],
            bacs: Some(BacsEvidence {
                present: true,
                automatic_controls_class: Some(BacsClass::C),
                energy_management_class: Some(BacsClass::C),
                source_reference: "inspection".into(),
            }),
        });
        assert_eq!(assess_epus_draft(&sample).status, "input_valid");
        sample.bacs_factor = 1.0;
        let result = assess_epus_draft(&sample);
        assert_eq!(result.status, "invalid");
        assert!(result.final_energy.is_none());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "bacs_factor_evidence_mismatch"));
    }
}
