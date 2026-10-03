//! NTA 8800:2025+C1:2026 chapter 9: table 9.1, equations 9.2/9.3 and
//! 9.56/9.60, and table 9.23 (p. 289–290, 323–324), first taken from the
//! consultation draft and checked against the final edition. The upstream
//! node input and installed powers are supplied.

use crate::final_energy_draft::MonthlyEnergy;
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

const DRAFT_SOURCE: &str = "NTA 8800:2025+C1:2026, tabel 9.1 en formules 9.2/9.3 (p. 289–290), formules 9.56/9.60 en tabel 9.23 (p. 323–324)";
const WINTER: [f64; 11] = [
    0.0, 0.20, 0.40, 0.59, 0.75, 0.87, 0.95, 0.98, 0.99, 1.0, 1.0,
];
const SUMMER: [f64; 11] = [0.0, 0.57, 0.87, 0.95, 0.98, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorClass {
    ExhaustAirHeatPumpWithoutOverventilation,
    HeatPump,
    BiomassBoiler,
    CombinedHeatPower,
    OtherBoiler,
}

impl GeneratorClass {
    fn priority(self) -> u8 {
        match self {
            Self::ExhaustAirHeatPumpWithoutOverventilation => 0,
            Self::HeatPump | Self::BiomassBoiler | Self::CombinedHeatPower => 1,
            Self::OtherBoiler => 2,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Generator {
    pub id: String,
    pub class: GeneratorClass,
    pub classification_reference: String,
    pub nominal_thermal_power_kw: f64,
    pub power_reference: String,
    /// Only orders different generators within one table-9.1 priority class.
    pub priority_efficiency: f64,
    pub efficiency_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratorDispatchDraftInput {
    pub node_input_kwh: Vec<MonthlyEnergy>,
    pub node_input_reference: String,
    /// `new_build` or `existing`: β by installed power (9.56/9.57);
    /// `existing_added_preferred`: renovation or a changed installation with
    /// an added preferred generator, β = Σ Φ·f_gebouw;si;H / Φ_H;tot with
    /// Φ_H;tot = Σ Q_H;node;in / 1139 (9.58/9.59).
    pub design_context: String,
    pub generators: Vec<Generator>,
    /// f_gebouw;si;H for 9.58; 1 when absent.
    #[serde(default)]
    pub building_fraction: Option<f64>,
}

/// 9.59: 0,13 × the year length, rounded.
const FULL_LOAD_HOURS_9_59: f64 = 1139.0;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorOutput {
    pub generator_id: String,
    pub beta_cumulative: f64,
    pub energy_fraction: f64,
    pub required_heat_kwh: f64,
    pub maximum_heat_kwh: f64,
    pub delivered_heat_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyDispatch {
    pub month: u8,
    pub node_input_kwh: f64,
    pub generators: Vec<GeneratorOutput>,
    pub unallocated_heat_kwh: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorDispatchDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub consultation_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub monthly: Vec<MonthlyDispatch>,
    pub node_input_derived: bool,
    pub final_edition_verified: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<DispatchIssue>,
}

fn issue(code: &'static str, path: impl Into<String>) -> DispatchIssue {
    DispatchIssue {
        code,
        path: path.into(),
    }
}

fn fraction(beta: f64, month: u8) -> f64 {
    let row = if (5..=9).contains(&month) {
        &SUMMER
    } else {
        &WINTER
    };
    if beta >= 1.0 {
        return 1.0;
    }
    let scaled = beta * 10.0;
    let lower = scaled.floor() as usize;
    row[lower] + (row[lower + 1] - row[lower]) * (scaled - lower as f64)
}

pub fn assess_generator_dispatch_draft(
    input: &GeneratorDispatchDraftInput,
) -> GeneratorDispatchDraftAssessment {
    let mut issues = Vec::new();
    if input.node_input_reference.trim().is_empty() {
        issues.push(issue("source_required", "nodeInputReference"));
    }
    let added_preferred = match input.design_context.as_str() {
        "new_build" | "existing" => false,
        "existing_added_preferred" => true,
        _ => {
            issues.push(issue("design_context_unsupported", "designContext"));
            false
        }
    };
    let building_fraction = input.building_fraction.unwrap_or(1.0);
    if !(building_fraction.is_finite() && building_fraction > 0.0 && building_fraction <= 1.0) {
        issues.push(issue("building_fraction_invalid", "buildingFraction"));
    }
    let mut months = [None; 12];
    for (index, value) in input.node_input_kwh.iter().enumerate() {
        let path = format!("nodeInputKwh[{index}]");
        if !(1..=12).contains(&value.month) {
            issues.push(issue("month_out_of_range", format!("{path}.month")));
        } else if !value.energy_kwh.is_finite() || value.energy_kwh < 0.0 {
            issues.push(issue("node_input_invalid", format!("{path}.energyKwh")));
        } else if months[usize::from(value.month - 1)]
            .replace(value.energy_kwh)
            .is_some()
        {
            issues.push(issue("month_duplicate", format!("{path}.month")));
        }
    }
    if months.iter().any(Option::is_none) {
        issues.push(issue("months_incomplete", "nodeInputKwh"));
    }
    if input.generators.is_empty() {
        issues.push(issue("generators_missing", "generators"));
    }
    let mut ids = HashSet::new();
    let mut total_power = 0.0;
    for (index, generator) in input.generators.iter().enumerate() {
        let path = format!("generators[{index}]");
        if generator.id.trim().is_empty() || !ids.insert(generator.id.trim()) {
            issues.push(issue("generator_id_invalid", format!("{path}.id")));
        }
        if generator.classification_reference.trim().is_empty() {
            issues.push(issue(
                "source_required",
                format!("{path}.classificationReference"),
            ));
        }
        if !generator.nominal_thermal_power_kw.is_finite()
            || generator.nominal_thermal_power_kw <= 0.0
        {
            issues.push(issue(
                "power_invalid",
                format!("{path}.nominalThermalPowerKw"),
            ));
        } else {
            total_power += generator.nominal_thermal_power_kw;
        }
        if generator.power_reference.trim().is_empty() {
            issues.push(issue("source_required", format!("{path}.powerReference")));
        }
        if !generator.priority_efficiency.is_finite() || generator.priority_efficiency <= 0.0 {
            issues.push(issue(
                "efficiency_invalid",
                format!("{path}.priorityEfficiency"),
            ));
        }
        if generator.efficiency_reference.trim().is_empty() {
            issues.push(issue(
                "source_required",
                format!("{path}.efficiencyReference"),
            ));
        }
    }
    if !total_power.is_finite() || total_power <= 0.0 {
        issues.push(issue("total_power_invalid", "generators"));
    }
    let mut sorted: Vec<_> = input.generators.iter().collect();
    sorted.sort_by(|left, right| {
        left.class
            .priority()
            .cmp(&right.class.priority())
            .then_with(|| {
                right
                    .priority_efficiency
                    .total_cmp(&left.priority_efficiency)
            })
    });
    for pair in sorted.windows(2) {
        if pair[0].class.priority() == pair[1].class.priority()
            && pair[0].priority_efficiency == pair[1].priority_efficiency
        {
            issues.push(issue(
                "priority_ambiguous_combine_identical_generators",
                "generators",
            ));
            break;
        }
    }
    // 9.57 or 9.59 as the β reference.
    let reference = if added_preferred {
        months.iter().map(|value| value.unwrap_or(0.0)).sum::<f64>() / FULL_LOAD_HOURS_9_59
    } else {
        total_power
    };
    let scale = if added_preferred {
        building_fraction
    } else {
        1.0
    };
    let mut monthly = Vec::new();
    if issues.is_empty() {
        for (index, node) in months.into_iter().enumerate() {
            let month = (index + 1) as u8;
            let node = node.expect("all months checked");
            let mut cumulative_power = 0.0;
            let mut previous_fraction = 0.0;
            let mut delivered_total = 0.0;
            let mut outputs = Vec::new();
            for (position, generator) in sorted.iter().enumerate() {
                cumulative_power += generator.nominal_thermal_power_kw;
                // The lowest preference takes the remainder (9.6.1 note 4:
                // a fictitious identical generator covers missing power).
                let beta = if position + 1 == sorted.len() || reference <= 0.0 {
                    1.0
                } else {
                    (cumulative_power * scale / reference).min(1.0)
                };
                let current_fraction = fraction(beta, month);
                let energy_fraction = current_fraction - previous_fraction;
                let required = (node - delivered_total).max(0.0);
                let maximum = node * energy_fraction;
                let delivered = required.min(maximum);
                if !beta.is_finite()
                    || !energy_fraction.is_finite()
                    || energy_fraction < 0.0
                    || !maximum.is_finite()
                    || !delivered.is_finite()
                {
                    issues.push(issue("dispatch_overflow", format!("nodeInputKwh[{index}]")));
                    break;
                }
                outputs.push(GeneratorOutput {
                    generator_id: generator.id.clone(),
                    beta_cumulative: beta,
                    energy_fraction,
                    required_heat_kwh: required,
                    maximum_heat_kwh: maximum,
                    delivered_heat_kwh: delivered,
                });
                previous_fraction = current_fraction;
                delivered_total += delivered;
            }
            if !issues.is_empty() {
                break;
            }
            let remainder = (node - delivered_total).max(0.0);
            if !remainder.is_finite() || remainder > 1e-9 * node.max(1.0) {
                issues.push(issue(
                    "dispatch_balance_invalid",
                    format!("nodeInputKwh[{index}]"),
                ));
                break;
            }
            monthly.push(MonthlyDispatch {
                month,
                node_input_kwh: node,
                generators: outputs,
                unallocated_heat_kwh: remainder,
            });
        }
    }
    if !issues.is_empty() {
        monthly.clear();
    }
    GeneratorDispatchDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "nta8800_9_6_1_installed_power_dispatch",
        consultation_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        monthly,
        node_input_derived: false,
        final_edition_verified: false,
        beng_calculation_available: false,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case() -> GeneratorDispatchDraftInput {
        GeneratorDispatchDraftInput {
            node_input_kwh: (1..=12)
                .map(|month| MonthlyEnergy {
                    month,
                    energy_kwh: 1000.0,
                })
                .collect(),
            node_input_reference: "9.2.3.5 hand case".into(),
            design_context: "new_build".into(),
            building_fraction: None,
            generators: vec![
                Generator {
                    id: "hp".into(),
                    class: GeneratorClass::HeatPump,
                    classification_reference: "design schedule".into(),
                    nominal_thermal_power_kw: 4.5,
                    power_reference: "type plate".into(),
                    priority_efficiency: 4.0,
                    efficiency_reference: "product sheet".into(),
                },
                Generator {
                    id: "boiler".into(),
                    class: GeneratorClass::OtherBoiler,
                    classification_reference: "design schedule".into(),
                    nominal_thermal_power_kw: 5.5,
                    power_reference: "type plate".into(),
                    priority_efficiency: 0.9,
                    efficiency_reference: "product sheet".into(),
                },
            ],
        }
    }

    #[test]
    fn table_923_interpolation_and_hybrid_balance_both_seasons() {
        let result = assess_generator_dispatch_draft(&case());
        assert_eq!(result.status, "diagnostic_valid");
        assert!((result.monthly[0].generators[0].delivered_heat_kwh - 810.0).abs() < 1e-9);
        assert!((result.monthly[4].generators[0].delivered_heat_kwh - 990.0).abs() < 1e-9);
        for month in &result.monthly {
            assert!(
                (month
                    .generators
                    .iter()
                    .map(|g| g.delivered_heat_kwh)
                    .sum::<f64>()
                    - 1000.0)
                    .abs()
                    < 1e-9
            );
        }
    }

    #[test]
    fn existing_building_routes_follow_9_56_and_9_58() {
        // Existing as installed: the same 9.56 β as new build.
        let mut existing = case();
        existing.design_context = "existing".into();
        let result = assess_generator_dispatch_draft(&existing);
        assert_eq!(result.status, "diagnostic_valid");
        assert!((result.monthly[0].generators[0].delivered_heat_kwh - 810.0).abs() < 1e-9);
        // 9.58/9.59: Φ_H;tot = 12 000 / 1139 = 10,5356 kW; β_hp = 4,5/10,5356
        // = 0,42713 → f = 0,75 + 0,2713·(0,87 − 0,75) = 0,78256 in January.
        let mut added = case();
        added.design_context = "existing_added_preferred".into();
        let result = assess_generator_dispatch_draft(&added);
        assert_eq!(result.status, "diagnostic_valid");
        let beta = 4.5 / (12_000.0 / 1139.0);
        let expected = 0.75 + (beta * 10.0 - 4.0) * (0.87 - 0.75);
        assert!(
            (result.monthly[0].generators[0].delivered_heat_kwh - 1000.0 * expected).abs() < 1e-6
        );
        // The boiler takes the remainder.
        let total: f64 = result.monthly[0]
            .generators
            .iter()
            .map(|item| item.delivered_heat_kwh)
            .sum();
        assert!((total - 1000.0).abs() < 1e-9);
        added.design_context = "renovation".into();
        assert_eq!(assess_generator_dispatch_draft(&added).status, "invalid");
    }

    #[test]
    fn priority_and_invalid_input_do_not_silently_produce_heat() {
        let mut input = case();
        input.generators[0].class = GeneratorClass::ExhaustAirHeatPumpWithoutOverventilation;
        let result = assess_generator_dispatch_draft(&input);
        assert_eq!(result.monthly[0].generators[0].generator_id, "hp");
        input.generators[0].nominal_thermal_power_kw = f64::INFINITY;
        let result = assess_generator_dispatch_draft(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.monthly.is_empty());
    }
}
