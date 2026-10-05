//! Collective source heat transfer and forfait primary factors for a linked gas heat pump,
//! NTA 8800:2025+C1:2026 formula 5.20 (p. 89–90) and §9.6.8.1.1.2.3 (p. 362).
//! This is a separate district-heat (`dh`) contribution, not gas-generator input or a BENG result.

use crate::forfait_heat_pump_monthly_draft::SourceSystem;
use crate::gas_heat_pump_chain_draft::{
    assess_gas_heat_pump_chain_draft, GasHeatPumpChainDraftInput,
};
use crate::gas_heat_pump_forfait_draft::GasPumpSource;
use crate::{input_fingerprint, KERNEL_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;

const CHAPTER_5: &str = "NTA 8800:2025+C1:2026, formule 5.20 (p. 89–90)";
const CHAPTER_9: &str = "NTA 8800:2025+C1:2026, formule 9.62 (p. 331) en §9.6.8.1.1.2.3 (p. 362)";
const ELECTRICITY_PRIMARY_FACTOR: f64 = 1.45;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceTemperatureClass {
    Below20C,
    AtLeast20C,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasCollectiveSourceDraftInput {
    pub chain: GasHeatPumpChainDraftInput,
    pub source_temperature_class: SourceTemperatureClass,
    pub source_temperature_reference: String,
    pub no_quality_declaration_confirmed: bool,
    pub no_quality_declaration_reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasCollectiveSourceDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub chapter_5_draft_source: &'static str,
    pub chapter_9_draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub source_energy_carrier: &'static str,
    pub primary_fossil_factor: Option<f64>,
    pub primary_renewable_factor: Option<f64>,
    pub factor_basis: Option<&'static str>,
    pub monthly: Vec<CollectiveSourceMonth>,
    pub annual_delivered_source_heat_kwh: Option<f64>,
    pub annual_draft_primary_fossil_kwh: Option<f64>,
    pub annual_draft_primary_renewable_kwh: Option<f64>,
    pub gas_input_energy_available: bool,
    pub source_pump_or_fan_included: bool,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<CollectiveSourceIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectiveSourceMonth {
    pub month: u8,
    pub delivered_source_heat_dh_kwh: f64,
    pub draft_primary_fossil_kwh: f64,
    pub draft_primary_renewable_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectiveSourceIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> CollectiveSourceIssue {
    CollectiveSourceIssue {
        code,
        path: path.into(),
    }
}

pub fn assess_gas_collective_source_draft(
    input: &GasCollectiveSourceDraftInput,
) -> GasCollectiveSourceDraftAssessment {
    let mut issues = Vec::new();
    let chain = assess_gas_heat_pump_chain_draft(&input.chain);
    if chain.status == "invalid" {
        issues.push(issue("chain_invalid", "chain"));
    }
    if input.chain.monthly.source_system == SourceSystem::Individual {
        issues.push(issue(
            "collective_source_required",
            "chain.monthly.sourceSystem",
        ));
    }
    if input.source_temperature_reference.trim().is_empty() {
        issues.push(issue(
            "source_temperature_reference_required",
            "sourceTemperatureReference",
        ));
    }
    if !input.no_quality_declaration_confirmed {
        issues.push(issue(
            "appendix_p_valuation_not_supported",
            "noQualityDeclarationConfirmed",
        ));
    }
    if input.no_quality_declaration_reference.trim().is_empty() {
        issues.push(issue(
            "quality_declaration_evidence_required",
            "noQualityDeclarationReference",
        ));
    }
    let source = input.chain.monthly.forfait.source;
    match (source, input.source_temperature_class) {
        (GasPumpSource::Ground, SourceTemperatureClass::Below20C) => {}
        (GasPumpSource::Ground, _) => issues.push(issue(
            "ground_temperature_class_invalid",
            "sourceTemperatureClass",
        )),
        (GasPumpSource::GroundwaterAquifer, SourceTemperatureClass::Unknown) => issues.push(issue(
            "groundwater_temperature_evidence_required",
            "sourceTemperatureClass",
        )),
        (GasPumpSource::GroundwaterAquifer | GasPumpSource::SurfaceWater, _) => {}
        _ => issues.push(issue(
            "collective_source_type_invalid",
            "chain.monthly.forfait.source",
        )),
    }
    let low = source != GasPumpSource::SurfaceWater
        && input.source_temperature_class == SourceTemperatureClass::Below20C;
    let (fossil_factor, renewable_factor, basis) = if low {
        (
            ELECTRICITY_PRIMARY_FACTOR / 23.0,
            0.95,
            "chapter_9_draft_9_6_8_1_1_2_3_below20_forfait",
        )
    } else {
        (
            0.9,
            0.0,
            "chapter_5_draft_tables_5_2_and_5_4_without_declaration",
        )
    };
    let mut monthly = Vec::new();
    let mut annual_heat = 0.0;
    let mut annual_fossil = 0.0;
    let mut annual_renewable = 0.0;
    if issues.is_empty() {
        for item in &chain.monthly {
            let heat = item.collective_source_heat_kwh;
            let fossil = heat * fossil_factor;
            let renewable = heat * renewable_factor;
            annual_heat += heat;
            annual_fossil += fossil;
            annual_renewable += renewable;
            if !heat.is_finite()
                || !fossil.is_finite()
                || !renewable.is_finite()
                || !annual_heat.is_finite()
                || !annual_fossil.is_finite()
                || !annual_renewable.is_finite()
            {
                issues.push(issue(
                    "collective_source_overflow",
                    format!("chain.monthly[{}]", item.month - 1),
                ));
                break;
            }
            monthly.push(CollectiveSourceMonth {
                month: item.month,
                delivered_source_heat_dh_kwh: heat,
                draft_primary_fossil_kwh: fossil,
                draft_primary_renewable_kwh: renewable,
            });
        }
    }
    if !issues.is_empty() {
        monthly.clear();
    }
    GasCollectiveSourceDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "public_chapters_5_and_9_collective_gas_heat_pump_source_forfait_only",
        chapter_5_draft_source: CHAPTER_5,
        chapter_9_draft_source: CHAPTER_9,
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        source_energy_carrier: "dh",
        primary_fossil_factor: if issues.is_empty() {
            Some(fossil_factor)
        } else {
            None
        },
        primary_renewable_factor: if issues.is_empty() {
            Some(renewable_factor)
        } else {
            None
        },
        factor_basis: if issues.is_empty() { Some(basis) } else { None },
        monthly,
        annual_delivered_source_heat_kwh: if issues.is_empty() {
            Some(annual_heat)
        } else {
            None
        },
        annual_draft_primary_fossil_kwh: if issues.is_empty() {
            Some(annual_fossil)
        } else {
            None
        },
        annual_draft_primary_renewable_kwh: if issues.is_empty() {
            Some(annual_renewable)
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
    use serde_json::json;

    fn sample(source: &str, system: &str) -> GasCollectiveSourceDraftInput {
        serde_json::from_value(json!({
            "chain": {
                "monthly": {
                    "forfait": {"generatorId":"ga-1","drive":"absorption","application":"utility",
                        "applicationReference":"office","collectiveBuildingInstallation":false,
                        "externalHeatSupply":false,"thermalCapacityKw":20.0,"capacityReference":"plate",
                        "source":source,"sourceReference":"source plan","designSupplyTemperatureC":35.0,
                        "designSupplyReference":"design"},
                    "generatorOutputKwh":(1..=12).map(|month|json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
                    "generatorOutputReference":"heat meter","sourceSystem":system,"sourceSystemReference":"invoice"
                },
                "auxiliary":{"generatorId":"ga-1","drive":"absorption","nominalThermalCapacityKw":20.0,
                    "capacityReference":"plate","standbyElectronicsW":10.0,"burnerAuxiliaryWPerKw":1.0,
                    "solutionPumpWPerKw":0.0,"coefficientsReference":"draft","meanModulation":1.0,
                    "modulationReference":"draft","buildingShare":1.0,"buildingShareReference":"whole building",
                    "forfaitCopUsed":true,"monthHoursReference":"schedule","generatorOutputReference":"heat meter",
                    "months":(1..=12).map(|month|json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0})).collect::<Vec<_>>()}
            },
            "sourceTemperatureClass":"below20_c","sourceTemperatureReference":"source design",
            "noQualityDeclarationConfirmed":true,"noQualityDeclarationReference":"register check"
        })).unwrap()
    }

    #[test]
    fn collective_groundwater_below20_keeps_dh_separate_from_gas_and_equipment_electricity() {
        let input = sample(
            "groundwater_aquifer",
            "collective_groundwater_surface_or_at_least15_c",
        );
        let result = assess_gas_collective_source_draft(&input);
        assert_eq!(result.status, "diagnostic_valid");
        let source_heat = 1000.0 * (1.0 - 1.0 / 2.1);
        assert!((result.monthly[0].delivered_source_heat_dh_kwh - source_heat).abs() < 1e-10);
        assert!(
            (result.monthly[0].draft_primary_fossil_kwh - source_heat * 1.45 / 23.0).abs() < 1e-10
        );
        assert!((result.monthly[0].draft_primary_renewable_kwh - source_heat * 0.95).abs() < 1e-10);
        assert_eq!(result.source_energy_carrier, "dh");
        assert!(!result.gas_input_energy_available && !result.beng_calculation_available);
    }

    #[test]
    fn surface_water_uses_high_branch_without_quality_declaration() {
        let mut input = sample(
            "surface_water",
            "collective_groundwater_surface_or_at_least15_c",
        );
        input.source_temperature_class = SourceTemperatureClass::Below20C;
        let result = assess_gas_collective_source_draft(&input);
        assert_eq!(result.status, "diagnostic_valid");
        assert_eq!(result.primary_fossil_factor, Some(0.9));
        assert_eq!(result.primary_renewable_factor, Some(0.0));
        assert_eq!(result.monthly[0].draft_primary_renewable_kwh, 0.0);
    }

    #[test]
    fn missing_evidence_or_declaration_route_never_produces_partial_primary_values() {
        let mut input = sample("ground", "collective_ground");
        input.no_quality_declaration_confirmed = false;
        input.source_temperature_reference.clear();
        let result = assess_gas_collective_source_draft(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.monthly.is_empty() && result.annual_draft_primary_fossil_kwh.is_none());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "appendix_p_valuation_not_supported"));
        input.no_quality_declaration_confirmed = true;
        input.source_temperature_reference = "design".into();
        input.source_temperature_class = SourceTemperatureClass::AtLeast20C;
        let result = assess_gas_collective_source_draft(&input);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "ground_temperature_class_invalid"));
        assert!(result.monthly.is_empty());
    }

    #[test]
    fn individual_source_and_heat_mismatch_cannot_acquire_dh_values() {
        let mut input = sample("outdoor_air", "individual");
        let result = assess_gas_collective_source_draft(&input);
        assert!(result.monthly.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "collective_source_required"));
        input = sample(
            "groundwater_aquifer",
            "collective_groundwater_surface_or_at_least15_c",
        );
        input.chain.auxiliary.months[0].generator_output_kwh = 900.0;
        let result = assess_gas_collective_source_draft(&input);
        assert!(result.monthly.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "chain_invalid"));
    }
}
