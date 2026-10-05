//! Dynamic transparent building elements, NTA 8800:2025+C1:2026 annex A
//! (pp. 766–771).
//!
//! Method A (A.3, step 1): monthly mean properties weighted over the
//! states of the element, `U_mi;mn = Σ U_dyn;i·Δθ / ΣΔθ` (A.1) and
//! `g_mi;mn = Σ g_dyn;i·I_sol / ΣI_sol` (A.2). The kernel has no hourly
//! climate, so the caller supplies per month the share of `Σ Δθ·Δt` and of
//! `Σ I_sol·Δt` spent in each state. Method B: one chosen state.
//!
//! Step 2 (p. 770): correction factors for the dynamic effects "kunnen
//! worden afgeleid" from hourly calculations; the norm gives no values and
//! no way to derive the state shares from monthly data. Declared factors
//! per month (with their source) may be supplied in `correction`; without
//! them the factors are 1. τ_sol and τ_vis (A.3/A.4) are weighted like g
//! with the solar weights when the states give them. Chapter 14 has no
//! input for them: 14.38 (vertical windows) has no transmittance term and
//! 14.41 fixes τ_D65 = 0,6 for rooflights (p. 673), so τ_vis is kept for
//! the record only (see the interpretation list).
//!
//! The numeric inputs are optional in the serde shape so that a half-filled
//! form entry gives one issue at its own path (`dynamic_value_missing`)
//! instead of making the whole project block unreadable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicState {
    pub id: String,
    /// `g_dyn;i` at normal incidence (as `gPerpendicular`).
    #[serde(default)]
    pub g_perpendicular: Option<f64>,
    /// `U_dyn;i`, W/(m²·K).
    #[serde(default)]
    pub u_value_w_per_m2k: Option<f64>,
    /// `τ_sol;dyn;i` (A.3), optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tau_solar: Option<f64>,
    /// `τ_vis;dyn;i` (A.4), optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tau_visual: Option<f64>,
}

/// Step 2 correction factors, twelve per property, from a documented
/// comparison with hourly calculations (p. 770).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StepTwoCorrection {
    #[serde(default)]
    pub u_factors: Vec<Option<f64>>,
    #[serde(default)]
    pub g_factors: Vec<Option<f64>>,
    #[serde(default)]
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum DynamicTransparent {
    /// Method A: twelve rows of weights per state, each row summing to 1.
    WeightedStates {
        states: Vec<DynamicState>,
        /// Share of `Σ I_sol·Δt` per state and month (A.2).
        #[serde(rename = "solarWeights", default)]
        solar_weights: Vec<Vec<Option<f64>>>,
        /// Share of `Σ Δθ_int-e·Δt` per state and month (A.1).
        #[serde(rename = "temperatureWeights", default)]
        temperature_weights: Vec<Vec<Option<f64>>>,
        #[serde(rename = "sourceReference", default)]
        source_reference: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        correction: Option<StepTwoCorrection>,
    },
    /// Method B: one state for every month.
    SingleState {
        state: DynamicState,
        #[serde(rename = "sourceReference", default)]
        source_reference: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        correction: Option<StepTwoCorrection>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct DynamicIssue {
    pub code: &'static str,
    pub path: String,
}

/// Weighted mean over the states. Only called on validated input, where
/// every value is present; a missing one counts as 0.
fn weighted(
    states: &[DynamicState],
    weights: &[Option<f64>],
    value: fn(&DynamicState) -> Option<f64>,
) -> f64 {
    states
        .iter()
        .zip(weights)
        .map(|(state, weight)| value(state).unwrap_or(0.0) * weight.unwrap_or(0.0))
        .sum()
}

fn factor(values: &[Option<f64>], month_index: usize) -> f64 {
    values.get(month_index).copied().flatten().unwrap_or(1.0)
}

fn weight_rows(rows: &[Vec<Option<f64>>], month_index: usize) -> &[Option<f64>] {
    rows.get(month_index).map_or(&[], Vec::as_slice)
}

impl DynamicTransparent {
    fn correction(&self) -> Option<&StepTwoCorrection> {
        match self {
            Self::WeightedStates { correction, .. } | Self::SingleState { correction, .. } => {
                correction.as_ref()
            }
        }
    }

    /// `g_mi;mn` for month index 0–11 (A.2), times the step-2 factor.
    pub fn g_for_month(&self, month_index: usize) -> f64 {
        let base = match self {
            Self::WeightedStates {
                states,
                solar_weights,
                ..
            } => weighted(states, weight_rows(solar_weights, month_index), |s| {
                s.g_perpendicular
            }),
            Self::SingleState { state, .. } => state.g_perpendicular.unwrap_or(0.0),
        };
        base * self
            .correction()
            .map_or(1.0, |item| factor(&item.g_factors, month_index))
    }

    /// `U_mi;mn` for month index 0–11 (A.1), times the step-2 factor.
    pub fn u_for_month(&self, month_index: usize) -> f64 {
        let base = match self {
            Self::WeightedStates {
                states,
                temperature_weights,
                ..
            } => weighted(states, weight_rows(temperature_weights, month_index), |s| {
                s.u_value_w_per_m2k
            }),
            Self::SingleState { state, .. } => state.u_value_w_per_m2k.unwrap_or(0.0),
        };
        base * self
            .correction()
            .map_or(1.0, |item| factor(&item.u_factors, month_index))
    }

    /// τ_sol;mi;mn (A.3) or τ_vis;mi;mn (A.4) with the solar weights; `None`
    /// when a state lacks the property.
    fn tau_for_month(
        &self,
        month_index: usize,
        value: fn(&DynamicState) -> Option<f64>,
    ) -> Option<f64> {
        match self {
            Self::WeightedStates {
                states,
                solar_weights,
                ..
            } => states
                .iter()
                .zip(weight_rows(solar_weights, month_index))
                .map(|(state, weight)| value(state).map(|tau| tau * weight.unwrap_or(0.0)))
                .sum(),
            Self::SingleState { state, .. } => value(state),
        }
    }

    pub fn tau_solar_for_month(&self, month_index: usize) -> Option<f64> {
        self.tau_for_month(month_index, |state| state.tau_solar)
    }

    pub fn tau_visual_for_month(&self, month_index: usize) -> Option<f64> {
        self.tau_for_month(month_index, |state| state.tau_visual)
    }

    pub fn validate(&self, path: &str) -> Vec<DynamicIssue> {
        let mut issues = Vec::new();
        let mut push = |code, field: String| {
            issues.push(DynamicIssue {
                code,
                path: format!("{path}.{field}"),
            })
        };
        let (states, prefix, source) = match self {
            Self::WeightedStates {
                states,
                source_reference,
                ..
            } => (states.as_slice(), "states", source_reference),
            Self::SingleState {
                state,
                source_reference,
                ..
            } => (std::slice::from_ref(state), "state", source_reference),
        };
        if let Some(correction) = self.correction() {
            for (field, values) in [
                ("uFactors", &correction.u_factors),
                ("gFactors", &correction.g_factors),
            ] {
                if values.len() != 12 {
                    push("dynamic_correction_invalid", "correction".into());
                } else if let Some(index) = values.iter().position(Option::is_none) {
                    push(
                        "dynamic_value_missing",
                        format!("correction.{field}[{index}]"),
                    );
                } else if values
                    .iter()
                    .flatten()
                    .any(|v| !(v.is_finite() && *v > 0.0))
                {
                    push("dynamic_correction_invalid", "correction".into());
                }
            }
            if correction.source_reference.trim().is_empty() {
                push(
                    "source_reference_required",
                    "correction.sourceReference".into(),
                );
            }
        }
        if source.trim().is_empty() {
            push("source_reference_required", "sourceReference".into());
        }
        if states.is_empty() {
            push("dynamic_state_required", "states".into());
        }
        for (index, state) in states.iter().enumerate() {
            let state_path = match self {
                Self::WeightedStates { .. } => format!("{prefix}[{index}]"),
                Self::SingleState { .. } => prefix.to_owned(),
            };
            let (Some(g), Some(u)) = (state.g_perpendicular, state.u_value_w_per_m2k) else {
                if state.g_perpendicular.is_none() {
                    push(
                        "dynamic_value_missing",
                        format!("{state_path}.gPerpendicular"),
                    );
                }
                if state.u_value_w_per_m2k.is_none() {
                    push(
                        "dynamic_value_missing",
                        format!("{state_path}.uValueWPerM2k"),
                    );
                }
                continue;
            };
            if !(0.0..=1.0).contains(&g)
                || !(u.is_finite() && u > 0.0)
                || [state.tau_solar, state.tau_visual]
                    .iter()
                    .flatten()
                    .any(|tau| !(0.0..=1.0).contains(tau))
            {
                push("dynamic_state_invalid", state_path);
            }
        }
        if let Self::WeightedStates {
            states,
            solar_weights,
            temperature_weights,
            ..
        } = self
        {
            for (field, rows) in [
                ("solarWeights", solar_weights),
                ("temperatureWeights", temperature_weights),
            ] {
                let missing = rows.iter().enumerate().find_map(|(month, row)| {
                    row.iter()
                        .position(Option::is_none)
                        .map(|column| (month, column))
                });
                if let Some((month, column)) = missing {
                    push(
                        "dynamic_value_missing",
                        format!("{field}[{month}][{column}]"),
                    );
                    continue;
                }
                let valid = rows.len() == 12
                    && rows.iter().all(|row| {
                        row.len() == states.len()
                            && row.iter().flatten().all(|w| (0.0..=1.0).contains(w))
                            && (row.iter().flatten().sum::<f64>() - 1.0).abs() < 1e-6
                    });
                if !valid {
                    push("dynamic_weights_invalid", field.into());
                }
            }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_a_weights_follow_a_1_and_a_2() {
        let states = vec![
            DynamicState {
                id: "clear".into(),
                g_perpendicular: Some(0.6),
                u_value_w_per_m2k: Some(1.0),
                tau_solar: Some(0.5),
                tau_visual: Some(0.7),
            },
            DynamicState {
                id: "tinted".into(),
                g_perpendicular: Some(0.2),
                u_value_w_per_m2k: Some(1.4),
                tau_solar: Some(0.1),
                tau_visual: Some(0.2),
            },
        ];
        let element = DynamicTransparent::WeightedStates {
            states,
            solar_weights: vec![vec![Some(0.75), Some(0.25)]; 12],
            temperature_weights: vec![vec![Some(0.5), Some(0.5)]; 12],
            source_reference: "control strategy".into(),
            correction: None,
        };
        assert!(element.validate("w").is_empty());
        assert!((element.g_for_month(6) - (0.75 * 0.6 + 0.25 * 0.2)).abs() < 1e-12);
        assert!((element.u_for_month(0) - 1.2).abs() < 1e-12);
        // A.3/A.4 with the solar weights.
        assert!((element.tau_solar_for_month(6).unwrap() - 0.4).abs() < 1e-12);
        assert!((element.tau_visual_for_month(6).unwrap() - 0.575).abs() < 1e-12);
        // Step 2: declared correction factors scale the monthly means.
        let mut corrected = element.clone();
        if let DynamicTransparent::WeightedStates { correction, .. } = &mut corrected {
            *correction = Some(StepTwoCorrection {
                u_factors: vec![Some(1.1); 12],
                g_factors: vec![Some(0.9); 12],
                source_reference: "hourly comparison".into(),
            });
        }
        assert!(corrected.validate("w").is_empty());
        assert!((corrected.g_for_month(6) - 0.9 * 0.5).abs() < 1e-12);
        assert!((corrected.u_for_month(0) - 1.1 * 1.2).abs() < 1e-12);
        let DynamicTransparent::WeightedStates { states, .. } = element else {
            unreachable!()
        };
        let broken = DynamicTransparent::WeightedStates {
            states,
            solar_weights: vec![vec![Some(0.5), Some(0.4)]; 12],
            temperature_weights: vec![vec![Some(0.5), Some(0.5)]; 11],
            source_reference: "x".into(),
            correction: None,
        };
        assert_eq!(broken.validate("w").len(), 2);
    }

    /// A half-filled form entry: each blank value is one issue at its own
    /// path, and the entry still deserializes.
    #[test]
    fn blank_values_are_reported_at_their_path() {
        let element: DynamicTransparent = serde_json::from_value(serde_json::json!({
            "method": "weighted_states",
            "states": [
                {"id": "open", "gPerpendicular": null, "uValueWPerM2k": 1.1},
                {"id": "closed", "gPerpendicular": 0.1, "uValueWPerM2k": null}
            ],
            "solarWeights": vec![vec![serde_json::json!(0.5), serde_json::Value::Null]; 12],
            "temperatureWeights": vec![vec![0.5, 0.5]; 12],
            "sourceReference": "simulation",
            "correction": {
                "uFactors": vec![serde_json::Value::Null; 12],
                "gFactors": vec![1.0; 12],
                "sourceReference": "comparison"
            }
        }))
        .unwrap();
        let paths: Vec<(&str, String)> = element
            .validate("w")
            .into_iter()
            .map(|issue| (issue.code, issue.path))
            .collect();
        assert_eq!(
            paths,
            vec![
                (
                    "dynamic_value_missing",
                    "w.correction.uFactors[0]".to_owned()
                ),
                (
                    "dynamic_value_missing",
                    "w.states[0].gPerpendicular".to_owned()
                ),
                (
                    "dynamic_value_missing",
                    "w.states[1].uValueWPerM2k".to_owned()
                ),
                ("dynamic_value_missing", "w.solarWeights[0][1]".to_owned()),
            ]
        );
        let single: DynamicTransparent = serde_json::from_value(serde_json::json!({
            "method": "single_state",
            "state": {"id": "closed"},
            "sourceReference": "sheet"
        }))
        .unwrap();
        assert_eq!(single.validate("w").len(), 2);
        assert_eq!(single.validate("w")[0].path, "w.state.gPerpendicular");
    }
}
