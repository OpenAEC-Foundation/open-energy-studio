//! f_BACS of NTA 8800:2025+C1:2026 §5.5.8 (p. 99–101): for utility buildings
//! with a heating or cooling system above 290 kW, or one whose power cannot
//! be determined (sum of the nominal generator powers per system, 9.6.1 and
//! 10.5), f_BACS = 1,05 without a BACS, with class D automatic controls or
//! with class C or D energy management; 1,0 otherwise. One failing system
//! sets the factor for all heating and cooling. The module name keeps its
//! history; the rules were checked against the target edition.

use crate::final_energy_draft::{as_f64, decimal};

/// Final-edition citation of §5.5.8.
const DRAFT_SOURCE: &str = "NTA 8800:2025+C1:2026, §5.5.8 (p. 99–101)";
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingUse {
    Residential,
    Utility,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemService {
    Heating,
    Cooling,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum BacsClass {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacsDraftInput {
    pub building_use: BuildingUse,
    pub system_inventory_complete: bool,
    pub systems: Vec<ThermalSystem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bacs: Option<BacsEvidence>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThermalSystem {
    pub id: String,
    pub service: SystemService,
    pub source_reference: String,
    pub generators: Vec<Generator>,
    /// System-specific inspection overrides the shared building BACS evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bacs: Option<BacsEvidence>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Generator {
    pub id: String,
    /// `null` means genuinely undeterminable, not zero or an omitted system.
    #[serde(deserialize_with = "deserialize_explicit_capacity")]
    pub nominal_thermal_capacity_kw: Option<f64>,
    pub source_reference: String,
}

fn deserialize_explicit_capacity<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<f64>::deserialize(deserializer)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BacsEvidence {
    pub present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automatic_controls_class: Option<BacsClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_management_class: Option<BacsClass>,
    pub source_reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacsDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub draft_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub factor: Option<f64>,
    pub applicability: &'static str,
    pub triggering_system_ids: Vec<String>,
    pub systems: Vec<SystemAssessment>,
    pub issues: Vec<BacsIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemAssessment {
    pub id: String,
    pub nominal_thermal_capacity_kw: Option<f64>,
    pub capacity_undeterminable: bool,
    pub above_290_kw: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BacsIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> BacsIssue {
    BacsIssue {
        code,
        path: path.into(),
    }
}

/// Some(true) means that this triggered system requires the 1.05 correction.
fn evidence_fails(bacs: Option<&BacsEvidence>) -> Option<bool> {
    let bacs = bacs?;
    if !bacs.present
        || matches!(bacs.automatic_controls_class, Some(BacsClass::D))
        || matches!(
            bacs.energy_management_class,
            Some(BacsClass::C | BacsClass::D)
        )
    {
        Some(true)
    } else if bacs.automatic_controls_class.is_some() && bacs.energy_management_class.is_some() {
        Some(false)
    } else {
        None
    }
}

fn validate_bacs(bacs: &BacsEvidence, path: &str, issues: &mut Vec<BacsIssue>) {
    if bacs.source_reference.trim().is_empty() {
        issues.push(issue("source_required", format!("{path}.sourceReference")));
    }
    if !bacs.present
        && (bacs.automatic_controls_class.is_some() || bacs.energy_management_class.is_some())
    {
        issues.push(issue("absent_bacs_has_classes", path));
    }
}

pub fn assess_bacs_draft(input: &BacsDraftInput) -> BacsDraftAssessment {
    let mut issues = Vec::new();
    let mut systems = Vec::new();
    let mut triggering_system_ids = Vec::new();
    let mut seen_systems = HashSet::new();
    for (index, system) in input.systems.iter().enumerate() {
        let path = format!("systems[{index}]");
        if system.id.trim().is_empty() || !seen_systems.insert(system.id.as_str()) {
            issues.push(issue(
                "system_id_required_or_duplicate",
                format!("{path}.id"),
            ));
        }
        if system.source_reference.trim().is_empty() {
            issues.push(issue("source_required", format!("{path}.sourceReference")));
        }
        if let Some(bacs) = &system.bacs {
            validate_bacs(bacs, &format!("{path}.bacs"), &mut issues);
        }
        if system.generators.is_empty() {
            issues.push(issue("generator_required", format!("{path}.generators")));
        }
        let mut seen_generators = HashSet::new();
        let mut capacity = Decimal::ZERO;
        let mut unknown = false;
        for (generator_index, generator) in system.generators.iter().enumerate() {
            let gp = format!("{path}.generators[{generator_index}]");
            if generator.id.trim().is_empty() || !seen_generators.insert(generator.id.as_str()) {
                issues.push(issue(
                    "generator_id_required_or_duplicate",
                    format!("{gp}.id"),
                ));
            }
            if generator.source_reference.trim().is_empty() {
                issues.push(issue("source_required", format!("{gp}.sourceReference")));
            }
            match generator.nominal_thermal_capacity_kw {
                None => unknown = true,
                Some(value) if !value.is_finite() || value <= 0.0 => {
                    issues.push(issue(
                        "capacity_invalid",
                        format!("{gp}.nominalThermalCapacityKw"),
                    ));
                }
                Some(value) => match decimal(value).and_then(|v| capacity.checked_add(v)) {
                    Some(sum) => capacity = sum,
                    None => issues.push(issue(
                        "capacity_decimal_range",
                        format!("{gp}.nominalThermalCapacityKw"),
                    )),
                },
            }
        }
        let above = capacity > Decimal::from(290);
        if unknown || above {
            triggering_system_ids.push(system.id.clone());
        }
        systems.push(SystemAssessment {
            id: system.id.clone(),
            nominal_thermal_capacity_kw: if unknown { None } else { as_f64(capacity) },
            capacity_undeterminable: unknown,
            above_290_kw: above,
        });
    }
    if let Some(bacs) = &input.bacs {
        validate_bacs(bacs, "bacs", &mut issues);
    }
    let mut any_failed = false;
    let mut all_assessed = true;
    for (system, assessment) in input.systems.iter().zip(&systems) {
        if assessment.capacity_undeterminable || assessment.above_290_kw {
            match evidence_fails(system.bacs.as_ref().or(input.bacs.as_ref())) {
                Some(true) => any_failed = true,
                Some(false) => {}
                None => all_assessed = false,
            }
        }
    }
    let status = if !issues.is_empty() {
        "invalid"
    } else if !input.system_inventory_complete {
        "incomplete"
    } else if matches!(input.building_use, BuildingUse::Residential)
        || triggering_system_ids.is_empty()
        || any_failed
        || all_assessed
    {
        "input_valid"
    } else {
        "incomplete"
    };
    if status == "incomplete" {
        if !input.system_inventory_complete {
            issues.push(issue(
                "system_inventory_incomplete",
                "systemInventoryComplete",
            ));
        } else {
            for (index, (system, assessment)) in input.systems.iter().zip(&systems).enumerate() {
                if (assessment.capacity_undeterminable || assessment.above_290_kw)
                    && evidence_fails(system.bacs.as_ref().or(input.bacs.as_ref())).is_none()
                {
                    issues.push(issue(
                        "bacs_evidence_incomplete",
                        format!("systems[{index}].bacs"),
                    ));
                }
            }
        }
    }
    let factor = if status != "input_valid" {
        None
    } else if matches!(input.building_use, BuildingUse::Residential)
        || triggering_system_ids.is_empty()
    {
        Some(1.0)
    } else if any_failed {
        Some(1.05)
    } else {
        Some(1.0)
    };
    let applicability = if status != "input_valid" {
        "undetermined"
    } else if matches!(input.building_use, BuildingUse::Residential) {
        "residential_exempt"
    } else if triggering_system_ids.is_empty() {
        "below_or_equal_threshold"
    } else {
        "utility_triggered"
    };
    BacsDraftAssessment {
        status,
        scope: "nta8800_5_5_8_bacs_factor",
        draft_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        final_edition_verified: true,
        reference_verified: false,
        beng_calculation_available: false,
        factor,
        applicability,
        triggering_system_ids,
        systems,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(capacity: Option<f64>) -> BacsDraftInput {
        BacsDraftInput {
            building_use: BuildingUse::Utility,
            system_inventory_complete: true,
            systems: vec![ThermalSystem {
                id: "h1".into(),
                service: SystemService::Heating,
                source_reference: "schedule".into(),
                bacs: None,
                generators: vec![Generator {
                    id: "g1".into(),
                    nominal_thermal_capacity_kw: capacity,
                    source_reference: "nameplate".into(),
                }],
            }],
            bacs: Some(BacsEvidence {
                present: true,
                automatic_controls_class: Some(BacsClass::C),
                energy_management_class: Some(BacsClass::B),
                source_reference: "BACS audit".into(),
            }),
        }
    }
    #[test]
    fn strict_per_system_threshold() {
        let mut input = sample(Some(290.0));
        assert_eq!(assess_bacs_draft(&input).factor, Some(1.0));
        input.systems.push(ThermalSystem {
            id: "h2".into(),
            service: SystemService::Cooling,
            source_reference: "schedule".into(),
            bacs: None,
            generators: vec![Generator {
                id: "g2".into(),
                nominal_thermal_capacity_kw: Some(200.0),
                source_reference: "nameplate".into(),
            }],
        });
        assert_eq!(assess_bacs_draft(&input).factor, Some(1.0));
        input.systems[0].generators.push(Generator {
            id: "g3".into(),
            nominal_thermal_capacity_kw: Some(0.1),
            source_reference: "nameplate".into(),
        });
        assert_eq!(assess_bacs_draft(&input).factor, Some(1.0));
        input.bacs.as_mut().unwrap().energy_management_class = Some(BacsClass::C);
        assert_eq!(assess_bacs_draft(&input).factor, Some(1.05));
    }
    #[test]
    fn unknown_capacity_triggers_and_missing_class_never_defaults_to_compliant() {
        let mut input = sample(None);
        input.bacs.as_mut().unwrap().energy_management_class = None;
        let assessment = assess_bacs_draft(&input);
        assert_eq!(assessment.status, "incomplete");
        assert_eq!(assessment.factor, None);
        input.bacs.as_mut().unwrap().present = false;
        input.bacs.as_mut().unwrap().automatic_controls_class = None;
        assert_eq!(assess_bacs_draft(&input).factor, Some(1.05));
    }
    #[test]
    fn incomplete_inventory_and_invalid_capacity_have_no_factor() {
        let mut input = sample(Some(200.0));
        input.system_inventory_complete = false;
        assert_eq!(assess_bacs_draft(&input).factor, None);
        input.system_inventory_complete = true;
        input.systems[0].generators[0].nominal_thermal_capacity_kw = Some(0.0);
        let assessment = assess_bacs_draft(&input);
        assert_eq!(assessment.status, "invalid");
        assert_eq!(assessment.factor, None);
    }
    #[test]
    fn one_noncompliant_system_corrects_all_even_when_another_has_missing_evidence() {
        let mut input = sample(Some(291.0));
        input.bacs = None;
        input.systems.push(ThermalSystem {
            id: "cooling".into(),
            service: SystemService::Cooling,
            source_reference: "schedule".into(),
            generators: vec![Generator {
                id: "chiller".into(),
                nominal_thermal_capacity_kw: Some(300.0),
                source_reference: "nameplate".into(),
            }],
            bacs: Some(BacsEvidence {
                present: true,
                automatic_controls_class: Some(BacsClass::D),
                energy_management_class: None,
                source_reference: "system inspection".into(),
            }),
        });
        let assessment = assess_bacs_draft(&input);
        assert_eq!(assessment.status, "input_valid");
        assert_eq!(assessment.factor, Some(1.05));
        input.systems[1]
            .bacs
            .as_mut()
            .unwrap()
            .automatic_controls_class = Some(BacsClass::C);
        let assessment = assess_bacs_draft(&input);
        assert_eq!(assessment.status, "incomplete");
        assert_eq!(assessment.factor, None);
    }
    #[test]
    fn unknown_power_must_be_explicit_null_not_an_omitted_field() {
        let mut value = serde_json::to_value(sample(None)).unwrap();
        assert!(serde_json::from_value::<BacsDraftInput>(value.clone()).is_ok());
        value["systems"][0]["generators"][0]
            .as_object_mut()
            .unwrap()
            .remove("nominalThermalCapacityKw");
        assert!(serde_json::from_value::<BacsDraftInput>(value).is_err());
    }
}
