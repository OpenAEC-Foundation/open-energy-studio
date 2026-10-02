//! Diagnostic direct-to-outdoor conductance from explicitly supplied elements.
//! This is only the elementary A·U + L·ψ + χ summation. It does not determine
//! the thermal envelope, ground/adjacent boundaries, NTA correction factors,
//! or the completeness and admissibility of the supplied properties.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

use crate::{KERNEL_VERSION, TARGET_NORM_VERSION};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DirectTransmissionInput {
    pub elements: Vec<DirectElement>,
    #[serde(default)]
    pub linear_bridges: Vec<LinearBridge>,
    #[serde(default)]
    pub point_bridges: Vec<PointBridge>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DirectElement {
    pub id: String,
    pub area_m2: f64,
    pub u_value_w_per_m2k: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LinearBridge {
    pub id: String,
    pub length_m: f64,
    pub psi_w_per_mk: f64,
    pub source_reference: String,
    /// Construction parts the bridge belongs to, for the TOjuli split of
    /// §5.7.2 step 2 (equal shares). Empty means "other" (pro rata).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orientations: Vec<EnvelopeSide>,
}

/// Orientation of a construction part for §5.7.2; `Horizontal` parts are
/// distributed pro rata over the orientations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeSide {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    Horizontal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PointBridge {
    pub id: String,
    pub chi_w_per_k: f64,
    pub source_reference: String,
    /// As for [`LinearBridge::orientations`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orientations: Vec<EnvelopeSide>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectTransmissionIssue {
    pub code: &'static str,
    pub path: String,
    pub message: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectTransmissionAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub element_conductance_w_per_k: Option<f64>,
    pub linear_bridge_conductance_w_per_k: Option<f64>,
    pub point_bridge_conductance_w_per_k: Option<f64>,
    pub total_direct_conductance_w_per_k: Option<f64>,
    pub issues: Vec<DirectTransmissionIssue>,
}

fn issue(code: &'static str, path: String, message: &'static str) -> DirectTransmissionIssue {
    DirectTransmissionIssue {
        code,
        path,
        message,
    }
}

/// Sum explicitly identified outdoor components. A successful diagnostic is
/// not an NTA result: the caller must establish boundary classification,
/// complete input, correction factors and independent reference evidence.
pub fn assess_direct_transmission(input: &DirectTransmissionInput) -> DirectTransmissionAssessment {
    let mut issues = Vec::new();
    let mut identifiers = HashSet::new();
    let mut element_sum = 0.0;
    let mut linear_sum = 0.0;
    let mut point_sum = 0.0;

    if input.elements.is_empty() {
        issues.push(issue(
            "direct_element_required",
            "elements".into(),
            "At least one directly exposed element is required",
        ));
    }
    for (index, element) in input.elements.iter().enumerate() {
        let path = format!("elements[{index}]");
        check_identity_and_source(
            &element.id,
            &element.source_reference,
            &path,
            &mut identifiers,
            &mut issues,
        );
        if !element.area_m2.is_finite() || element.area_m2 <= 0.0 {
            issues.push(issue(
                "direct_area_invalid",
                format!("{path}.areaM2"),
                "Area must be finite and positive",
            ));
        }
        if !element.u_value_w_per_m2k.is_finite() || element.u_value_w_per_m2k <= 0.0 {
            issues.push(issue(
                "direct_u_invalid",
                format!("{path}.uValueWPerM2k"),
                "U-value must be finite and positive",
            ));
        }
        element_sum += element.area_m2 * element.u_value_w_per_m2k;
    }
    for (index, bridge) in input.linear_bridges.iter().enumerate() {
        let path = format!("linearBridges[{index}]");
        check_identity_and_source(
            &bridge.id,
            &bridge.source_reference,
            &path,
            &mut identifiers,
            &mut issues,
        );
        if !bridge.length_m.is_finite() || bridge.length_m <= 0.0 {
            issues.push(issue(
                "direct_length_invalid",
                format!("{path}.lengthM"),
                "Length must be finite and positive",
            ));
        }
        if !bridge.psi_w_per_mk.is_finite() {
            issues.push(issue(
                "direct_psi_invalid",
                format!("{path}.psiWPerMk"),
                "Linear transmittance must be finite",
            ));
        }
        linear_sum += bridge.length_m * bridge.psi_w_per_mk;
    }
    for (index, bridge) in input.point_bridges.iter().enumerate() {
        let path = format!("pointBridges[{index}]");
        check_identity_and_source(
            &bridge.id,
            &bridge.source_reference,
            &path,
            &mut identifiers,
            &mut issues,
        );
        if !bridge.chi_w_per_k.is_finite() {
            issues.push(issue(
                "direct_chi_invalid",
                format!("{path}.chiWPerK"),
                "Point transmittance must be finite",
            ));
        }
        point_sum += bridge.chi_w_per_k;
    }
    let total = element_sum + linear_sum + point_sum;
    if !element_sum.is_finite()
        || !linear_sum.is_finite()
        || !point_sum.is_finite()
        || !total.is_finite()
        || total <= 0.0
    {
        issues.push(issue(
            "direct_total_invalid",
            "totalDirectConductanceWPerK".into(),
            "Summed direct conductance must be finite and positive",
        ));
    }
    let valid = issues.is_empty();
    let input_fingerprint = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(input).expect("typed input serializes"))
    );
    DirectTransmissionAssessment {
        status: if valid { "input_valid" } else { "invalid" },
        scope: "diagnostic_direct_outdoor_only",
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint,
        reference_verified: false,
        beng_calculation_available: false,
        element_conductance_w_per_k: valid.then_some(element_sum),
        linear_bridge_conductance_w_per_k: valid.then_some(linear_sum),
        point_bridge_conductance_w_per_k: valid.then_some(point_sum),
        total_direct_conductance_w_per_k: valid.then_some(total),
        issues,
    }
}

fn check_identity_and_source(
    id: &str,
    source_reference: &str,
    path: &str,
    identifiers: &mut HashSet<String>,
    issues: &mut Vec<DirectTransmissionIssue>,
) {
    if id.trim().is_empty() || !identifiers.insert(id.to_string()) {
        issues.push(issue(
            "direct_id_invalid",
            format!("{path}.id"),
            "Component IDs must be nonempty and unique across the diagnostic",
        ));
    }
    if source_reference.trim().is_empty() {
        issues.push(issue(
            "direct_source_required",
            format!("{path}.sourceReference"),
            "Each input property needs a source reference",
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> DirectTransmissionInput {
        DirectTransmissionInput {
            elements: vec![
                DirectElement {
                    id: "wall".into(),
                    area_m2: 10.0,
                    u_value_w_per_m2k: 0.2,
                    source_reference: "drawing-1".into(),
                },
                DirectElement {
                    id: "window".into(),
                    area_m2: 2.0,
                    u_value_w_per_m2k: 1.1,
                    source_reference: "declaration-1".into(),
                },
            ],
            linear_bridges: vec![LinearBridge {
                id: "edge".into(),
                length_m: 3.0,
                psi_w_per_mk: 0.05,
                source_reference: "detail-1".into(),
                orientations: Vec::new(),
            }],
            point_bridges: vec![PointBridge {
                id: "junction".into(),
                chi_w_per_k: 0.02,
                source_reference: "detail-2".into(),
                orientations: Vec::new(),
            }],
        }
    }

    #[test]
    fn hand_calculation_keeps_element_and_bridge_terms_separate() {
        let result = assess_direct_transmission(&example());
        assert_eq!(result.status, "input_valid");
        assert!((result.element_conductance_w_per_k.unwrap() - 4.2).abs() < 1e-12);
        assert!((result.linear_bridge_conductance_w_per_k.unwrap() - 0.15).abs() < 1e-12);
        assert!((result.point_bridge_conductance_w_per_k.unwrap() - 0.02).abs() < 1e-12);
        assert!((result.total_direct_conductance_w_per_k.unwrap() - 4.37).abs() < 1e-12);
        assert!(!result.reference_verified);
        assert!(!result.beng_calculation_available);
        assert!(result.input_fingerprint.starts_with("sha256:"));
    }

    #[test]
    fn invalid_or_untraceable_components_do_not_emit_a_number() {
        let mut input = example();
        input.elements[0].u_value_w_per_m2k = 0.0;
        input.linear_bridges[0].source_reference.clear();
        input.point_bridges[0].id = "window".into();
        let result = assess_direct_transmission(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.total_direct_conductance_w_per_k.is_none());
        for code in [
            "direct_u_invalid",
            "direct_source_required",
            "direct_id_invalid",
        ] {
            assert!(result.issues.iter().any(|issue| issue.code == code));
        }
    }

    #[test]
    fn malformed_fields_are_rejected_instead_of_ignored() {
        let mut value = serde_json::to_value(serde_json::json!({
            "elements": [{"id":"wall", "areaM2": 10, "uValueWPerM2k": 0.2,
                "sourceReference":"drawing", "uValue": 0.3}]
        }))
        .unwrap();
        assert!(serde_json::from_value::<DirectTransmissionInput>(value.clone()).is_err());
        value["elements"][0]
            .as_object_mut()
            .unwrap()
            .remove("uValue");
        assert!(serde_json::from_value::<DirectTransmissionInput>(value).is_ok());
    }

    #[test]
    fn source_change_changes_diagnostic_fingerprint_even_if_number_does_not() {
        let original = example();
        let mut changed = original.clone();
        changed.elements[0].source_reference = "drawing-2".into();
        let before = assess_direct_transmission(&original);
        let after = assess_direct_transmission(&changed);
        assert_eq!(
            before.total_direct_conductance_w_per_k,
            after.total_direct_conductance_w_per_k
        );
        assert_ne!(before.input_fingerprint, after.input_fingerprint);
    }
}
