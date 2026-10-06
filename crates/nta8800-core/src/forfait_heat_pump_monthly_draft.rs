//! Consultation-draft equations 9.62 and 9.6.8.1.1.2.3 for supplied monthly generator output.
//! Upstream generator dispatch (9.2.2.1.3), auxiliaries and annual/BENG accounting
//! are outside this diagnostic.

use crate::final_energy_draft::MonthlyEnergy;
use crate::forfait_heat_pump_draft::{
    assess_forfait_heat_pump_draft, ForfaitHeatPumpDraftInput, TableSource, DRAFT_SOURCE,
};
use crate::{input_fingerprint, KERNEL_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceSystem {
    Individual,
    CollectiveGround,
    CollectiveGroundwaterSurfaceOrAtLeast15C,
}

impl SourceSystem {
    /// (9.62) `f_cor.bron.col` (2025+C1 p. 331–332); NTA 8800:2024 (9.62)
    /// has no such term (p. 314–315).
    fn correction_factor(self) -> f64 {
        if !crate::norm_versions::profile().collective_source_correction {
            return 0.0;
        }
        match self {
            Self::Individual => 0.0,
            Self::CollectiveGround => 0.009,
            Self::CollectiveGroundwaterSurfaceOrAtLeast15C => 0.022,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForfaitHeatPumpMonthlyDraftInput {
    pub forfait: ForfaitHeatPumpDraftInput,
    pub generator_output_kwh: Vec<MonthlyEnergy>,
    pub generator_output_reference: String,
    pub source_system: SourceSystem,
    pub source_system_reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyGeneratorInput {
    pub month: u8,
    pub generator_output_kwh: f64,
    pub collective_source_heat_kwh: f64,
    pub uncorrected_input_electricity_kwh: f64,
    pub collective_source_correction_kwh: f64,
    pub generator_input_electricity_kwh: f64,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForfaitHeatPumpMonthlyDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub consultation_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub corrected_cop: Option<f64>,
    pub collective_source_correction_factor: Option<f64>,
    pub monthly: Vec<MonthlyGeneratorInput>,
    pub generator_output_derived: bool,
    pub collective_source_heat_derived: bool,
    pub auxiliaries_included: bool,
    pub annual_performance_available: bool,
    pub beng_calculation_available: bool,
    pub final_edition_verified: bool,
    pub issues: Vec<DraftIssue>,
}

fn monthly_values(
    values: &[MonthlyEnergy],
    path: &str,
    issues: &mut Vec<DraftIssue>,
) -> Option<[f64; 12]> {
    let mut months = [None; 12];
    for (index, entry) in values.iter().enumerate() {
        let field = format!("{path}[{index}]");
        if !(1..=12).contains(&entry.month) {
            issues.push(issue("month_out_of_range", format!("{field}.month")));
            continue;
        }
        let slot = &mut months[usize::from(entry.month - 1)];
        if slot.is_some() {
            issues.push(issue("month_duplicate", format!("{field}.month")));
        } else if !entry.energy_kwh.is_finite() || entry.energy_kwh < 0.0 {
            issues.push(issue("monthly_heat_invalid", format!("{field}.energyKwh")));
        } else {
            *slot = Some(entry.energy_kwh);
        }
    }
    if months.iter().any(Option::is_none) {
        issues.push(issue("months_incomplete", path));
        None
    } else {
        Some(months.map(|value| value.expect("all months checked")))
    }
}

pub fn assess_forfait_heat_pump_monthly_draft(
    input: &ForfaitHeatPumpMonthlyDraftInput,
) -> ForfaitHeatPumpMonthlyDraftAssessment {
    let mut issues = Vec::new();
    let lookup = assess_forfait_heat_pump_draft(&input.forfait);
    if lookup.status == "invalid" || lookup.corrected_cop.is_none() {
        issues.push(issue("forfait_lookup_invalid", "forfait"));
    }
    // An input the chosen edition lacks keeps its own code.
    for item in &lookup.issues {
        if item.code == "route_not_in_edition" {
            issues.push(issue(item.code, format!("forfait.{}", item.path)));
        }
    }
    if input.generator_output_reference.trim().is_empty() {
        issues.push(issue("source_required", "generatorOutputReference"));
    }
    if input.source_system_reference.trim().is_empty() {
        issues.push(issue("source_required", "sourceSystemReference"));
    }
    let output = monthly_values(
        &input.generator_output_kwh,
        "generatorOutputKwh",
        &mut issues,
    );
    let collective = input.source_system != SourceSystem::Individual;
    let source_compatible = match input.source_system {
        SourceSystem::Individual => !matches!(
            input.forfait.source,
            TableSource::Collective15To20C
                | TableSource::Collective20To40C
                | TableSource::CollectiveAtLeast40C
        ),
        SourceSystem::CollectiveGround => input.forfait.source == TableSource::Ground,
        SourceSystem::CollectiveGroundwaterSurfaceOrAtLeast15C => matches!(
            input.forfait.source,
            TableSource::GroundwaterBelow15C
                | TableSource::SurfaceWater
                | TableSource::Collective15To20C
                | TableSource::Collective20To40C
                | TableSource::CollectiveAtLeast40C
        ),
    };
    if !source_compatible {
        issues.push(issue("source_system_mismatch", "sourceSystem"));
    }
    let mut monthly = Vec::new();
    if issues.is_empty() {
        let output = output.expect("complete generator output checked");
        let cop = lookup.corrected_cop.expect("valid lookup checked");
        let factor = input.source_system.correction_factor();
        for (index, &thermal) in output.iter().enumerate() {
            let uncorrected = thermal / cop;
            let source_heat = if collective {
                thermal * (1.0 - 1.0 / cop)
            } else {
                0.0
            };
            let correction = source_heat * factor;
            let electricity = uncorrected - correction;
            if !uncorrected.is_finite()
                || !source_heat.is_finite()
                || source_heat < 0.0
                || source_heat > thermal
                || !correction.is_finite()
                || !electricity.is_finite()
                || electricity < 0.0
            {
                issues.push(issue(
                    "generator_input_invalid",
                    format!("generatorOutputKwh[{index}]"),
                ));
                break;
            }
            monthly.push(MonthlyGeneratorInput {
                month: (index + 1) as u8,
                generator_output_kwh: thermal,
                collective_source_heat_kwh: source_heat,
                uncorrected_input_electricity_kwh: uncorrected,
                collective_source_correction_kwh: correction,
                generator_input_electricity_kwh: electricity,
            });
        }
    }
    if !issues.is_empty() {
        monthly.clear();
    }
    ForfaitHeatPumpMonthlyDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_equation_9_62_supplied_monthly_flows_only",
        consultation_source: DRAFT_SOURCE,
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        corrected_cop: if issues.is_empty() {
            lookup.corrected_cop
        } else {
            None
        },
        collective_source_correction_factor: if issues.is_empty() {
            Some(input.source_system.correction_factor())
        } else {
            None
        },
        monthly,
        generator_output_derived: false,
        collective_source_heat_derived: collective && issues.is_empty(),
        auxiliaries_included: false,
        annual_performance_available: false,
        beng_calculation_available: false,
        final_edition_verified: false,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample(source: &str, source_system: SourceSystem) -> ForfaitHeatPumpMonthlyDraftInput {
        let mut forfait = json!({
            "generatorId":"hp-1", "classificationSourceReference":"source design",
            "scope":"residential_at_most25_kw", "source":source, "sink":"hydronic",
            "designSupplyTemperatureC":35.0, "sourceCorrectionFactor":null,
            "sourceCorrectionReference":null, "sourceTemperatureC":null,
            "sourceTemperatureEvidenceReference":null,
            "thermalCapacityKw":8.0, "capacitySourceReference":"manufacturer sheet",
            "collectiveBuildingInstallation":false
        });
        if source == "collective15_to20_c" {
            forfait["sourceTemperatureC"] = json!(15.0);
            forfait["sourceTemperatureEvidenceReference"] = json!("source meter");
        }
        serde_json::from_value(json!({
            "forfait":forfait,
            "generatorOutputKwh":(1..=12).map(|month| json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
            "generatorOutputReference":"monthly generator heat meter",
            "sourceSystem":source_system,
            "sourceSystemReference":"source design"
        })).unwrap()
    }

    #[test]
    fn equations_962_and_collective_source_heat_derive_monthly_input() {
        let mut input = sample(
            "collective15_to20_c",
            SourceSystem::CollectiveGroundwaterSurfaceOrAtLeast15C,
        );
        let result = assess_forfait_heat_pump_monthly_draft(&input);
        assert_eq!(result.status, "diagnostic_valid");
        assert_eq!(result.corrected_cop, Some(4.8));
        assert!((result.monthly[0].uncorrected_input_electricity_kwh - 1000.0 / 4.8).abs() < 1e-9);
        let source_heat = 1000.0 * (1.0 - 1.0 / 4.8);
        assert!((result.monthly[0].collective_source_heat_kwh - source_heat).abs() < 1e-9);
        assert!(
            (result.monthly[0].collective_source_correction_kwh - source_heat * 0.022).abs() < 1e-9
        );
        assert!(
            (result.monthly[0].generator_input_electricity_kwh
                - (1000.0 / 4.8 - source_heat * 0.022))
                .abs()
                < 1e-9
        );
        assert!(result.collective_source_heat_derived);
        assert!(!result.annual_performance_available);
        input.source_system = SourceSystem::Individual;
        let invalid = assess_forfait_heat_pump_monthly_draft(&input);
        assert_eq!(invalid.status, "invalid");
        assert!(invalid.monthly.is_empty());
    }

    #[test]
    fn individual_ground_and_collective_ground_have_distinct_correction() {
        let mut input = sample("ground", SourceSystem::Individual);
        input.forfait.source_correction_factor = Some(1.0);
        input.forfait.source_correction_reference = Some("no regeneration".into());
        let individual = assess_forfait_heat_pump_monthly_draft(&input);
        assert_eq!(
            individual.monthly[0].generator_input_electricity_kwh,
            1000.0 / 3.8
        );
        input.source_system = SourceSystem::CollectiveGround;
        let collective = assess_forfait_heat_pump_monthly_draft(&input);
        assert!(
            (collective.monthly[0].collective_source_correction_kwh
                - 1000.0 * (1.0 - 1.0 / 3.8) * 0.009)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn collective_source_correction_is_absent_in_2024() {
        let mut input = sample("ground", SourceSystem::Individual);
        input.forfait.source_correction_factor = Some(1.0);
        input.forfait.source_correction_reference = Some("no regeneration".into());
        input.source_system = SourceSystem::CollectiveGround;
        // (9.62) f_cor.bron.col 0,009 in 2025+C1 (p. 331–332).
        let current = assess_forfait_heat_pump_monthly_draft(&input);
        assert_eq!(current.collective_source_correction_factor, Some(0.009));
        // NTA 8800:2024 (9.62), p. 314–315: no correction term.
        let legacy =
            crate::norm_versions::with_version(crate::norm_versions::NormVersion::V2024, || {
                assess_forfait_heat_pump_monthly_draft(&input)
            });
        assert_eq!(legacy.collective_source_correction_factor, Some(0.0));
        assert_eq!(legacy.monthly[0].collective_source_correction_kwh, 0.0);
    }

    #[test]
    fn incomplete_or_impossible_months_emit_no_partial_energy() {
        let mut input = sample(
            "collective15_to20_c",
            SourceSystem::CollectiveGroundwaterSurfaceOrAtLeast15C,
        );
        input.generator_output_kwh.pop();
        assert!(assess_forfait_heat_pump_monthly_draft(&input)
            .monthly
            .is_empty());
        input.generator_output_kwh.push(MonthlyEnergy {
            month: 12,
            energy_kwh: 1000.0,
        });
        input.generator_output_kwh[0].energy_kwh = -1.0;
        let invalid = assess_forfait_heat_pump_monthly_draft(&input);
        assert_eq!(invalid.status, "invalid");
        assert!(invalid.monthly.is_empty());
    }

    #[test]
    fn generator_input_overflow_suppresses_all_monthly_values() {
        let mut input = sample("ground", SourceSystem::Individual);
        input.forfait.source_correction_factor = Some(1e-308);
        input.forfait.source_correction_reference = Some("supplied correction".into());
        let invalid = assess_forfait_heat_pump_monthly_draft(&input);
        assert_eq!(invalid.status, "invalid");
        assert!(invalid.monthly.is_empty());
        assert!(invalid.corrected_cop.is_none());
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "generator_input_invalid"));
    }
}
