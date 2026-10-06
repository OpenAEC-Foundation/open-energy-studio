//! NTA 8800:2025+C1:2026 table 9.25 (p. 327–328) and equation 9.61 (p. 326)
//! for gas-fired water boilers, first taken from the consultation draft and
//! checked against the final edition.
//! Generator heat remains supplied by the caller or a separate dispatch step.

use crate::final_energy_draft::MonthlyEnergy;
use crate::{input_fingerprint, KERNEL_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;

const DRAFT_SOURCE: &str =
    "NTA 8800:2025+C1:2026, tabel 9.25 (p. 327–328) en formule 9.61 (p. 326)";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BoilerRole {
    IndividualMain,
    IndividualSupplementary,
    Collective,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BoilerLocation {
    InsideThermalBoundary,
    OutsideThermalBoundary,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BoilerKind {
    /// A gas boiler without further designation, or any oil boiler.
    Conventional,
    Vr,
    Hr100,
    Hr104,
    Hr107,
    /// Table 9.25 a): unknown generator of a collective installation
    /// (0,70; W_H;aux;gen = 0 per 9.6.8.2.2, note 1 of table 9.25).
    Unknown,
}

impl BoilerKind {
    fn index(self) -> usize {
        match self {
            Self::Conventional | Self::Unknown => 0,
            Self::Vr => 1,
            Self::Hr100 => 2,
            Self::Hr104 => 3,
            Self::Hr107 => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmissionCircuit {
    Direct,
    MixingWithReturnLimit,
    MixingWithoutReturnLimit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoilerForfaitDraftInput {
    pub generator_id: String,
    pub role: BoilerRole,
    pub location: BoilerLocation,
    pub kind: BoilerKind,
    pub fuel: String,
    pub average_design_emission_temperature_c: f64,
    pub emission_circuit: EmissionCircuit,
    pub equipment_reference: String,
    pub location_reference: String,
    pub temperature_and_circuit_reference: String,
    pub pilot_flame_present: bool,
    #[serde(default)]
    pub installation_year: Option<u16>,
    #[serde(default)]
    pub installation_year_reference: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoilerIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> BoilerIssue {
    BoilerIssue {
        code,
        path: path.into(),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoilerForfaitDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub consultation_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub temperature_class: Option<&'static str>,
    pub generation_efficiency: Option<f64>,
    pub auxiliary_year_class: Option<&'static str>,
    pub pilot_flame_included: bool,
    pub auxiliaries_included: bool,
    pub final_edition_verified: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<BoilerIssue>,
}

fn table_efficiency(input: &BoilerForfaitDraftInput, low_temperature: bool) -> f64 {
    let row = match (input.role, input.location) {
        (BoilerRole::IndividualMain, BoilerLocation::InsideThermalBoundary) => [
            (0.75, 0.75),
            (0.80, 0.80),
            (0.925, 0.90),
            (0.95, 0.925),
            (0.975, 0.95),
        ],
        (BoilerRole::IndividualMain, BoilerLocation::OutsideThermalBoundary) => [
            (0.70, 0.70),
            (0.75, 0.75),
            (0.875, 0.85),
            (0.90, 0.875),
            (0.925, 0.90),
        ],
        (BoilerRole::IndividualSupplementary, BoilerLocation::InsideThermalBoundary) => [
            (0.75, 0.75),
            (0.80, 0.80),
            (0.90, 0.90),
            (0.925, 0.90),
            (0.95, 0.90),
        ],
        (BoilerRole::IndividualSupplementary, BoilerLocation::OutsideThermalBoundary) => [
            (0.70, 0.70),
            (0.75, 0.75),
            (0.85, 0.85),
            (0.875, 0.85),
            (0.90, 0.85),
        ],
        (BoilerRole::Collective, _) => [
            (0.70, 0.70),
            (0.75, 0.75),
            (0.875, 0.85),
            (0.90, 0.875),
            (0.925, 0.90),
        ],
    };
    let (lt, ht) = row[input.kind.index()];
    if low_temperature {
        lt
    } else {
        ht
    }
}

pub fn assess_boiler_forfait_draft(
    input: &BoilerForfaitDraftInput,
) -> BoilerForfaitDraftAssessment {
    let mut issues = Vec::new();
    if input.generator_id.trim().is_empty() {
        issues.push(issue("generator_id_required", "generatorId"));
    }
    // Table 9.25: oil boilers count as conventional (definition below the
    // table); VR and HR classes are gas appliances.
    match input.fuel.as_str() {
        "natural_gas" => {}
        "oil" => {
            if input.kind != BoilerKind::Conventional {
                issues.push(issue("oil_boiler_must_be_conventional", "kind"));
            }
            if input.pilot_flame_present {
                issues.push(issue("pilot_flame_gas_only", "pilotFlamePresent"));
            }
        }
        _ => issues.push(issue("fuel_unsupported", "fuel")),
    }
    if input.kind == BoilerKind::Unknown && input.role != BoilerRole::Collective {
        issues.push(issue("unknown_boiler_collective_only", "kind"));
    }
    if !input.average_design_emission_temperature_c.is_finite()
        || !(-30.0..=120.0).contains(&input.average_design_emission_temperature_c)
    {
        issues.push(issue(
            "average_emission_temperature_invalid",
            "averageDesignEmissionTemperatureC",
        ));
    }
    if input.equipment_reference.trim().is_empty() {
        issues.push(issue("source_required", "equipmentReference"));
    }
    if input.location_reference.trim().is_empty() {
        issues.push(issue("source_required", "locationReference"));
    }
    if input.temperature_and_circuit_reference.trim().is_empty() {
        issues.push(issue("source_required", "temperatureAndCircuitReference"));
    }
    if input
        .installation_year
        .is_some_and(|year| !(1900..=2026).contains(&year))
    {
        issues.push(issue("installation_year_invalid", "installationYear"));
    }
    if input.installation_year.is_some()
        && input
            .installation_year_reference
            .as_deref()
            .map_or(true, |reference| reference.trim().is_empty())
    {
        issues.push(issue(
            "installation_year_reference_required",
            "installationYearReference",
        ));
    }
    if input.installation_year.is_none() && input.installation_year_reference.is_some() {
        issues.push(issue(
            "installation_year_reference_without_year",
            "installationYearReference",
        ));
    }
    let lt = input.average_design_emission_temperature_c <= 50.0
        && input.emission_circuit != EmissionCircuit::MixingWithoutReturnLimit;
    BoilerForfaitDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_gas_water_boiler_table_9_25_only",
        consultation_source: DRAFT_SOURCE,
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        temperature_class: if issues.is_empty() {
            Some(if lt { "lt" } else { "ht" })
        } else {
            None
        },
        generation_efficiency: if issues.is_empty() {
            Some(table_efficiency(input, lt))
        } else {
            None
        },
        auxiliary_year_class: if issues.is_empty() && input.role != BoilerRole::Collective {
            Some(
                if input.installation_year.is_some_and(|year| year >= 2015) {
                    "from_2015"
                } else {
                    "before_2015_or_unknown"
                },
            )
        } else {
            None
        },
        pilot_flame_included: false,
        auxiliaries_included: false,
        final_edition_verified: false,
        beng_calculation_available: false,
        issues,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoilerForfaitMonthlyDraftInput {
    pub boiler: BoilerForfaitDraftInput,
    pub generator_output_kwh: Vec<MonthlyEnergy>,
    pub generator_output_reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoilerMonth {
    pub month: u8,
    pub generator_output_kwh: f64,
    pub input_natural_gas_kwh: f64,
    /// Oil input of a conventional oil boiler (gross value), kWh.
    #[serde(default)]
    pub input_oil_kwh: f64,
    pub auxiliary_electricity_kwh: Option<f64>,
    /// §9.6.2.1: 695 kWh gas per year for a pilot flame, by month length,
    /// for the whole device (not in `input_natural_gas_kwh`; a collective
    /// installation scales it with f_gebouw;H).
    pub pilot_flame_natural_gas_kwh: f64,
}

/// §9.6.2.1 annual gas use of a pilot flame, kWh.
pub const PILOT_FLAME_ANNUAL_KWH: f64 = 695.0;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoilerForfaitMonthlyDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub consultation_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub generation_efficiency: Option<f64>,
    pub auxiliary_year_class: Option<&'static str>,
    pub monthly: Vec<BoilerMonth>,
    pub annual_auxiliary_electricity_kwh: Option<f64>,
    pub generator_output_derived: bool,
    pub pilot_flame_included: bool,
    pub auxiliaries_included: bool,
    pub final_edition_verified: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<BoilerIssue>,
}

pub fn assess_boiler_forfait_monthly_draft(
    input: &BoilerForfaitMonthlyDraftInput,
) -> BoilerForfaitMonthlyDraftAssessment {
    let lookup = assess_boiler_forfait_draft(&input.boiler);
    let mut issues = lookup.issues;
    if input.generator_output_reference.trim().is_empty() {
        issues.push(issue("source_required", "generatorOutputReference"));
    }
    let mut months = [None; 12];
    for (index, value) in input.generator_output_kwh.iter().enumerate() {
        let path = format!("generatorOutputKwh[{index}]");
        if !(1..=12).contains(&value.month) {
            issues.push(issue("month_out_of_range", format!("{path}.month")));
        } else if !value.energy_kwh.is_finite() || value.energy_kwh < 0.0 {
            issues.push(issue(
                "generator_output_invalid",
                format!("{path}.energyKwh"),
            ));
        } else if months[usize::from(value.month - 1)]
            .replace(value.energy_kwh)
            .is_some()
        {
            issues.push(issue("month_duplicate", format!("{path}.month")));
        }
    }
    if months.iter().any(Option::is_none) {
        issues.push(issue("months_incomplete", "generatorOutputKwh"));
    }
    let mut monthly = Vec::new();
    let individual = input.boiler.role != BoilerRole::Collective;
    let annual_fixed = if input
        .boiler
        .installation_year
        .is_some_and(|year| year >= 2015)
    {
        // 13,0 kWh in NTA 8800:2020+A1 (p. 336), 43,8 from 2022 (p. 338).
        crate::norm_versions::profile().device_aux_a_from_2015_kwh
    } else {
        87.6
    };
    let mut annual_auxiliary = if individual { Some(0.0_f64) } else { None };
    if issues.is_empty() {
        let efficiency = lookup
            .generation_efficiency
            .expect("valid boiler table checked");
        for (index, thermal) in months.into_iter().enumerate() {
            let thermal = thermal.expect("all months checked");
            let fuel = thermal / efficiency;
            // Draft 9.85: A/12 + B*E_gas/(C*Bnom); individual gas-device
            // forfait A=87.6 or 43.8, B=.132, C=.4 and Bnom=24 kW.
            let auxiliary = if individual {
                Some(annual_fixed / 12.0 + 0.132 * fuel / (0.4 * 24.0))
            } else {
                None
            };
            if !fuel.is_finite()
                || fuel < 0.0
                || auxiliary.is_some_and(|value| !value.is_finite() || value < 0.0)
            {
                issues.push(issue(
                    "boiler_input_invalid",
                    format!("generatorOutputKwh[{index}]"),
                ));
                break;
            }
            if let (Some(total), Some(value)) = (annual_auxiliary, auxiliary) {
                annual_auxiliary = Some(total + value);
                if annual_auxiliary.is_some_and(|sum| !sum.is_finite()) {
                    issues.push(issue("auxiliary_sum_invalid", "generatorOutputKwh"));
                    break;
                }
            }
            let oil = input.boiler.fuel == "oil";
            monthly.push(BoilerMonth {
                month: (index + 1) as u8,
                generator_output_kwh: thermal,
                input_natural_gas_kwh: if oil { 0.0 } else { fuel },
                input_oil_kwh: if oil { fuel } else { 0.0 },
                auxiliary_electricity_kwh: auxiliary,
                pilot_flame_natural_gas_kwh: if input.boiler.pilot_flame_present {
                    PILOT_FLAME_ANNUAL_KWH * crate::climate::MONTH_HOURS[index]
                        / crate::climate::YEAR_HOURS
                } else {
                    0.0
                },
            });
        }
    }
    if !issues.is_empty() {
        monthly.clear();
        annual_auxiliary = None;
    }
    BoilerForfaitMonthlyDraftAssessment {
        status: if issues.is_empty() {
            "diagnostic_valid"
        } else {
            "invalid"
        },
        scope: "public_chapter_9_draft_gas_water_boiler_9_61_and_individual_forfait_aux_9_85",
        consultation_source: DRAFT_SOURCE,
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        generation_efficiency: if issues.is_empty() {
            lookup.generation_efficiency
        } else {
            None
        },
        auxiliary_year_class: if issues.is_empty() {
            lookup.auxiliary_year_class
        } else {
            None
        },
        monthly,
        annual_auxiliary_electricity_kwh: annual_auxiliary,
        generator_output_derived: false,
        pilot_flame_included: issues.is_empty() && input.boiler.pilot_flame_present,
        auxiliaries_included: issues.is_empty() && individual,
        final_edition_verified: false,
        beng_calculation_available: false,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> BoilerForfaitDraftInput {
        serde_json::from_value(json!({
            "generatorId":"boiler", "role":"individual_supplementary",
            "location":"inside_thermal_boundary", "kind":"hr107", "fuel":"natural_gas",
            "averageDesignEmissionTemperatureC":45.0, "emissionCircuit":"direct",
            "equipmentReference":"type plate", "locationReference":"building plan",
            "temperatureAndCircuitReference":"system design", "pilotFlamePresent":false
        }))
        .unwrap()
    }

    #[test]
    fn oil_boilers_are_conventional_and_unknown_is_collective() {
        // Table 9.25: an oil boiler is conventional (0,75 inside, main).
        let mut input = sample();
        input.role = BoilerRole::IndividualMain;
        input.fuel = "oil".into();
        assert!(assess_boiler_forfait_draft(&input)
            .issues
            .iter()
            .any(|item| item.code == "oil_boiler_must_be_conventional"));
        input.kind = BoilerKind::Conventional;
        assert_eq!(
            assess_boiler_forfait_draft(&input).generation_efficiency,
            Some(0.75)
        );
        let monthly = assess_boiler_forfait_monthly_draft(&BoilerForfaitMonthlyDraftInput {
            boiler: input.clone(),
            generator_output_kwh: (1..=12)
                .map(|month| MonthlyEnergy {
                    month,
                    energy_kwh: 300.0,
                })
                .collect(),
            generator_output_reference: "dispatch heat".into(),
        });
        assert_eq!(monthly.monthly[0].input_natural_gas_kwh, 0.0);
        assert!((monthly.monthly[0].input_oil_kwh - 400.0).abs() < 1e-9);
        // Table 9.25 a): unknown collective generator 0,70.
        let mut unknown = sample();
        unknown.kind = BoilerKind::Unknown;
        assert!(assess_boiler_forfait_draft(&unknown)
            .issues
            .iter()
            .any(|item| item.code == "unknown_boiler_collective_only"));
        unknown.role = BoilerRole::Collective;
        assert_eq!(
            assess_boiler_forfait_draft(&unknown).generation_efficiency,
            Some(0.70)
        );
    }

    #[test]
    fn table_925_separates_supplementary_from_main_and_lt_from_ht() {
        let mut input = sample();
        let result = assess_boiler_forfait_draft(&input);
        assert_eq!(result.temperature_class, Some("lt"));
        assert_eq!(result.generation_efficiency, Some(0.95));
        input.emission_circuit = EmissionCircuit::MixingWithoutReturnLimit;
        assert_eq!(
            assess_boiler_forfait_draft(&input).generation_efficiency,
            Some(0.90)
        );
        input.role = BoilerRole::IndividualMain;
        input.emission_circuit = EmissionCircuit::Direct;
        assert_eq!(
            assess_boiler_forfait_draft(&input).generation_efficiency,
            Some(0.975)
        );
        input.role = BoilerRole::IndividualSupplementary;
        input.location = BoilerLocation::OutsideThermalBoundary;
        assert_eq!(
            assess_boiler_forfait_draft(&input).generation_efficiency,
            Some(0.90)
        );
    }

    #[test]
    fn equation_961_auxiliaries_and_pilot_flame() {
        let mut input = BoilerForfaitMonthlyDraftInput {
            boiler: sample(),
            generator_output_kwh: (1..=12)
                .map(|month| MonthlyEnergy {
                    month,
                    energy_kwh: 250.0,
                })
                .collect(),
            generator_output_reference: "dispatch heat".into(),
        };
        let result = assess_boiler_forfait_monthly_draft(&input);
        assert_eq!(result.status, "diagnostic_valid");
        assert!((result.monthly[0].input_natural_gas_kwh - 250.0 / 0.95).abs() < 1e-9);
        let variable = 0.132 * (250.0 / 0.95) / (0.4 * 24.0);
        assert!(
            (result.monthly[0].auxiliary_electricity_kwh.unwrap() - (87.6 / 12.0 + variable)).abs()
                < 1e-9
        );
        assert_eq!(result.auxiliary_year_class, Some("before_2015_or_unknown"));
        assert!(result.auxiliaries_included);
        input.boiler.installation_year = Some(2015);
        input.boiler.installation_year_reference = Some("commissioning certificate".into());
        let recent = assess_boiler_forfait_monthly_draft(&input);
        assert_eq!(recent.auxiliary_year_class, Some("from_2015"));
        assert!(
            (recent.monthly[0].auxiliary_electricity_kwh.unwrap() - (43.8 / 12.0 + variable)).abs()
                < 1e-9
        );
        input.boiler.installation_year_reference = None;
        assert_eq!(
            assess_boiler_forfait_monthly_draft(&input).status,
            "invalid"
        );
        input.boiler.installation_year = None;
        input.boiler.pilot_flame_present = true;
        // §9.6.2.1: 695 kWh per year, by month length.
        let pilot = assess_boiler_forfait_monthly_draft(&input);
        assert_eq!(pilot.status, "diagnostic_valid");
        assert!(pilot.pilot_flame_included);
        assert!(
            (pilot.monthly[0].pilot_flame_natural_gas_kwh - 695.0 * 744.0 / 8760.0).abs() < 1e-9
        );
        assert!((pilot.monthly[0].input_natural_gas_kwh - 250.0 / 0.95).abs() < 1e-9);
    }
}
