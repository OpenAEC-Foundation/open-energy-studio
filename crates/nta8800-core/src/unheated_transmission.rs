//! Diagnostic conductance through named unheated spaces with caller-supplied
//! reduction factors. The factors, thermal boundaries and property evidence
//! are not derived or verified against NTA 8800.

use crate::direct_transmission::{assess_direct_transmission, DirectTransmissionInput};
use crate::{KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnheatedTransmissionInput {
    pub spaces: Vec<UnheatedSpaceInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnheatedSpaceInput {
    pub id: String,
    pub reduction_factor: f64,
    pub factor_source_reference: String,
    /// Explicit A·U, L·Ψ and χ terms at the zone-to-space boundary.
    pub boundary: DirectTransmissionInput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnheatedSpaceDiagnostic {
    pub id: String,
    pub unreduced_conductance_w_per_k: f64,
    pub reduction_factor: f64,
    pub reduced_conductance_w_per_k: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnheatedIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnheatedTransmissionAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub total_reduced_conductance_w_per_k: Option<f64>,
    pub spaces: Vec<UnheatedSpaceDiagnostic>,
    pub issues: Vec<UnheatedIssue>,
}

fn issue(code: &'static str, path: impl Into<String>) -> UnheatedIssue {
    UnheatedIssue {
        code,
        path: path.into(),
    }
}

pub fn assess_unheated_transmission(
    input: &UnheatedTransmissionInput,
) -> UnheatedTransmissionAssessment {
    let mut issues = Vec::new();
    if input.spaces.is_empty() {
        issues.push(issue("unheated_space_required", "spaces"));
    }
    let mut seen = HashSet::new();
    let mut spaces = Vec::with_capacity(input.spaces.len());
    let mut total = 0.0;
    for (index, space) in input.spaces.iter().enumerate() {
        let path = format!("spaces[{index}]");
        if space.id.trim().is_empty() || !seen.insert(space.id.as_str()) {
            issues.push(issue("unheated_space_id_invalid", format!("{path}.id")));
        }
        if !space.reduction_factor.is_finite() || !(0.0..=1.0).contains(&space.reduction_factor) {
            issues.push(issue(
                "unheated_reduction_factor_invalid",
                format!("{path}.reductionFactor"),
            ));
        }
        if space.factor_source_reference.trim().is_empty() {
            issues.push(issue(
                "unheated_factor_source_required",
                format!("{path}.factorSourceReference"),
            ));
        }
        let boundary = assess_direct_transmission(&space.boundary);
        issues.extend(
            boundary
                .issues
                .into_iter()
                .map(|item| issue(item.code, format!("{path}.boundary.{}", item.path))),
        );
        if let Some(base) = boundary.total_direct_conductance_w_per_k {
            let reduced = base * space.reduction_factor;
            if !reduced.is_finite() {
                issues.push(issue("unheated_reduced_overflow", path));
            } else {
                total += reduced;
                spaces.push(UnheatedSpaceDiagnostic {
                    id: space.id.clone(),
                    unreduced_conductance_w_per_k: base,
                    reduction_factor: space.reduction_factor,
                    reduced_conductance_w_per_k: reduced,
                });
            }
        }
    }
    if !total.is_finite() {
        issues.push(issue("unheated_total_overflow", "spaces"));
    }
    let valid = issues.is_empty();
    let input_fingerprint = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(input).expect("typed input serializes"))
    );
    UnheatedTransmissionAssessment {
        status: if valid { "input_valid" } else { "invalid" },
        scope: "diagnostic_unheated_space_with_supplied_factors_only",
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint,
        reference_verified: false,
        beng_calculation_available: false,
        total_reduced_conductance_w_per_k: valid.then_some(total),
        spaces: if valid { spaces } else { Vec::new() },
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direct_transmission::{DirectElement, LinearBridge};

    fn example() -> UnheatedTransmissionInput {
        UnheatedTransmissionInput {
            spaces: vec![UnheatedSpaceInput {
                id: "garage".into(),
                reduction_factor: 0.5,
                factor_source_reference: "supplied-assumption-1".into(),
                boundary: DirectTransmissionInput {
                    elements: vec![DirectElement {
                        id: "wall".into(),
                        area_m2: 10.0,
                        u_value_w_per_m2k: 0.4,
                        source_reference: "drawing-1".into(),
                    }],
                    linear_bridges: vec![LinearBridge {
                        id: "edge".into(),
                        length_m: 2.0,
                        psi_w_per_mk: 0.1,
                        source_reference: "detail-1".into(),
                    }],
                    point_bridges: Vec::new(),
                },
            }],
        }
    }

    #[test]
    fn hand_example_reduces_each_space_separately() {
        let mut input = example();
        input.spaces.push(UnheatedSpaceInput {
            id: "attic".into(),
            reduction_factor: 0.8,
            factor_source_reference: "supplied-assumption-2".into(),
            boundary: DirectTransmissionInput {
                elements: vec![DirectElement {
                    id: "ceiling".into(),
                    area_m2: 20.0,
                    u_value_w_per_m2k: 0.3,
                    source_reference: "drawing-2".into(),
                }],
                linear_bridges: Vec::new(),
                point_bridges: Vec::new(),
            },
        });
        let result = assess_unheated_transmission(&input);
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.spaces.len(), 2);
        assert!((result.spaces[0].unreduced_conductance_w_per_k - 4.2).abs() < 1e-12);
        assert!((result.spaces[0].reduced_conductance_w_per_k - 2.1).abs() < 1e-12);
        assert!((result.total_reduced_conductance_w_per_k.unwrap() - 6.9).abs() < 1e-12);
        assert!(!result.reference_verified);
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn invalid_factor_or_boundary_returns_no_partial_numbers() {
        let mut input = example();
        input.spaces[0].reduction_factor = 1.2;
        input.spaces[0].boundary.elements[0].area_m2 = 0.0;
        let result = assess_unheated_transmission(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.total_reduced_conductance_w_per_k.is_none());
        assert!(result.spaces.is_empty());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "unheated_reduction_factor_invalid"));
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "direct_area_invalid"
                && item.path == "spaces[0].boundary.elements[0].areaM2"));
    }

    #[test]
    fn rejects_duplicate_space_ids_and_misspelled_fields() {
        let mut input = example();
        input.spaces.push(input.spaces[0].clone());
        let result = assess_unheated_transmission(&input);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "unheated_space_id_invalid"));
        let mut value = serde_json::to_value(example()).unwrap();
        value["spaces"][0]["reductionFactors"] = value["spaces"][0]["reductionFactor"].take();
        assert!(serde_json::from_value::<UnheatedTransmissionInput>(value).is_err());
    }
}
