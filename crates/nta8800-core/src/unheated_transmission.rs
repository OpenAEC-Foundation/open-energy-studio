//! Conductance through unheated spaces, NTA 8800:2025+C1:2026 §8.4
//! (pp. 262–265): `H_U = H_zi,j;ztu · b_U` (8.52).
//!
//! `b_U` is either derived from the space's own losses to outside,
//! `b_U = H_ue/(H_ue + Σ_j H_zi,j;ztu)` (8.53–8.57, with `H_V;iu = 0`), using
//! a ventilation flow `V_ue` or the approximation `H_V;ue ≈ 0,5·H_D;ue`
//! (8.58/8.59), or declared (the basic survey of ISSO 82.1/75.1 with `H_ue`
//! from annex I.2.4). Unheated basements belong to `H_g` (8.3), not here.

use crate::direct_transmission::{assess_direct_transmission, DirectTransmissionInput};
use crate::KERNEL_VERSION;
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
    /// Declared `b_U`; exclusive with `outside`.
    #[serde(default)]
    pub reduction_factor: Option<f64>,
    #[serde(default)]
    pub factor_source_reference: String,
    /// Explicit A·U, L·Ψ and χ terms at the zone-to-space boundary, with
    /// the space-side R_si (8.4.2.1).
    pub boundary: DirectTransmissionInput,
    /// Losses of the space to outside for deriving `b_U` (8.53–8.59).
    #[serde(default)]
    pub outside: Option<UnheatedOutside>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnheatedOutside {
    /// `H_D;ue` (8.4.2.2): space to outside, excluding the partitions to
    /// heated zones and the ground.
    pub transmission: DirectTransmissionInput,
    pub ventilation: UnheatedVentilation,
    /// `H_zi,j;ztu` of other heated zones bordering the same space (Σ_j in
    /// 8.53/8.59), W/K.
    #[serde(default)]
    pub other_zones_conductance_w_per_k: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum UnheatedVentilation {
    /// 8.57: `H_V;ue = ρ_a·c_a·V_ue/3600`.
    Flow {
        #[serde(rename = "airflowM3PerH")]
        airflow_m3_per_h: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 8.58: `H_V;ue ≈ 0,5·H_D;ue`.
    HalfOfTransmission,
}

/// ρ_a·c_a = 1 211 J/(m³·K) (8.57).
const VOLUMETRIC_HEAT_CAPACITY: f64 = 1.205 * 1005.0;

/// `b_U` per 8.53–8.59.
pub fn derived_reduction_factor(
    zone_conductance_w_per_k: f64,
    outside_transmission_w_per_k: f64,
    ventilation: &UnheatedVentilation,
    other_zones_conductance_w_per_k: f64,
) -> f64 {
    let ventilation_w_per_k = match ventilation {
        UnheatedVentilation::Flow {
            airflow_m3_per_h, ..
        } => VOLUMETRIC_HEAT_CAPACITY * airflow_m3_per_h / 3600.0,
        UnheatedVentilation::HalfOfTransmission => 0.5 * outside_transmission_w_per_k,
    };
    let h_ue = outside_transmission_w_per_k + ventilation_w_per_k;
    let denominator = h_ue + zone_conductance_w_per_k + other_zones_conductance_w_per_k;
    if denominator > 0.0 {
        h_ue / denominator
    } else {
        1.0
    }
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
        match (&space.reduction_factor, &space.outside) {
            (Some(factor), None) => {
                if !factor.is_finite() || !(0.0..=1.0).contains(factor) {
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
            }
            (None, Some(outside)) => {
                if !outside.other_zones_conductance_w_per_k.is_finite()
                    || outside.other_zones_conductance_w_per_k < 0.0
                {
                    issues.push(issue(
                        "unheated_other_zones_conductance_invalid",
                        format!("{path}.outside.otherZonesConductanceWPerK"),
                    ));
                }
                if let UnheatedVentilation::Flow {
                    airflow_m3_per_h,
                    source_reference,
                } = &outside.ventilation
                {
                    if !airflow_m3_per_h.is_finite() || *airflow_m3_per_h < 0.0 {
                        issues.push(issue(
                            "unheated_airflow_invalid",
                            format!("{path}.outside.ventilation.airflowM3PerH"),
                        ));
                    }
                    if source_reference.trim().is_empty() {
                        issues.push(issue(
                            "unheated_factor_source_required",
                            format!("{path}.outside.ventilation.sourceReference"),
                        ));
                    }
                }
            }
            (Some(_), Some(_)) => issues.push(issue(
                "unheated_factor_declared_and_derived",
                format!("{path}.reductionFactor"),
            )),
            (None, None) => issues.push(issue(
                "unheated_reduction_factor_required",
                format!("{path}.reductionFactor"),
            )),
        }
        let boundary = assess_direct_transmission(&space.boundary);
        issues.extend(
            boundary
                .issues
                .into_iter()
                .map(|item| issue(item.code, format!("{path}.boundary.{}", item.path))),
        );
        let outside_conductance = space.outside.as_ref().map(|outside| {
            let assessed = assess_direct_transmission(&outside.transmission);
            issues.extend(assessed.issues.into_iter().map(|item| {
                issue(
                    item.code,
                    format!("{path}.outside.transmission.{}", item.path),
                )
            }));
            assessed.total_direct_conductance_w_per_k
        });
        if let Some(base) = boundary.total_direct_conductance_w_per_k {
            let factor = match (&space.reduction_factor, &space.outside, outside_conductance) {
                (Some(factor), _, _) => *factor,
                (None, Some(outside), Some(Some(h_d_ue))) => derived_reduction_factor(
                    base,
                    h_d_ue,
                    &outside.ventilation,
                    outside.other_zones_conductance_w_per_k,
                ),
                _ => continue,
            };
            let reduced = base * factor;
            if !reduced.is_finite() {
                issues.push(issue("unheated_reduced_overflow", path));
            } else {
                total += reduced;
                spaces.push(UnheatedSpaceDiagnostic {
                    id: space.id.clone(),
                    unreduced_conductance_w_per_k: base,
                    reduction_factor: factor,
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
        scope: "unverified_unheated_space_conductance_8_4",
        target_norm_version: crate::norm_versions::current_label(),
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
                reduction_factor: Some(0.5),
                factor_source_reference: "supplied-assumption-1".into(),
                outside: None,
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
                        orientations: Vec::new(),
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
            reduction_factor: Some(0.8),
            factor_source_reference: "supplied-assumption-2".into(),
            outside: None,
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
        input.spaces[0].reduction_factor = Some(1.2);
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
    fn derived_factor_follows_8_53_and_8_59() {
        let mut input = example();
        input.spaces[0].reduction_factor = None;
        input.spaces[0].outside = Some(UnheatedOutside {
            transmission: DirectTransmissionInput {
                elements: vec![DirectElement {
                    id: "garage-wall".into(),
                    area_m2: 30.0,
                    u_value_w_per_m2k: 2.0,
                    source_reference: "drawing-3".into(),
                }],
                linear_bridges: Vec::new(),
                point_bridges: Vec::new(),
            },
            ventilation: UnheatedVentilation::HalfOfTransmission,
            other_zones_conductance_w_per_k: 0.0,
        });
        let result = assess_unheated_transmission(&input);
        assert_eq!(result.status, "input_valid");
        // 8.59: b = 1,5·60/(1,5·60 + 4,2).
        let b = 90.0 / (90.0 + 4.2);
        assert!((result.spaces[0].reduction_factor - b).abs() < 1e-12);
        assert!((result.spaces[0].reduced_conductance_w_per_k - 4.2 * b).abs() < 1e-12);
        let flow = derived_reduction_factor(
            4.2,
            60.0,
            &UnheatedVentilation::Flow {
                airflow_m3_per_h: 100.0,
                source_reference: "x".into(),
            },
            1.0,
        );
        let h_ue = 60.0 + 1211.025 * 100.0 / 3600.0;
        assert!((flow - h_ue / (h_ue + 5.2)).abs() < 1e-12);
        input.spaces[0].reduction_factor = Some(0.5);
        assert!(assess_unheated_transmission(&input)
            .issues
            .iter()
            .any(|item| item.code == "unheated_factor_declared_and_derived"));
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
