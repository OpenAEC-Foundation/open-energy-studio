//! Dynamic transparent building elements, NTA 8800:2025+C1:2026 annex A
//! (pp. 766–771).
//!
//! Method A (A.3, step 1): monthly mean properties weighted over the
//! states of the element, `U_mi;mn = Σ U_dyn;i·Δθ / ΣΔθ` (A.1) and
//! `g_mi;mn = Σ g_dyn;i·I_sol / ΣI_sol` (A.2). The kernel has no hourly
//! climate, so the caller supplies per month the share of `Σ Δθ·Δt` and of
//! `Σ I_sol·Δt` spent in each state. Method B: one chosen state. The
//! step-2 correction factors have no forfait value and are taken as 1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicState {
    pub id: String,
    /// `g_dyn;i` at normal incidence (as `gPerpendicular`).
    pub g_perpendicular: f64,
    /// `U_dyn;i`, W/(m²·K).
    pub u_value_w_per_m2k: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum DynamicTransparent {
    /// Method A: twelve rows of weights per state, each row summing to 1.
    WeightedStates {
        states: Vec<DynamicState>,
        /// Share of `Σ I_sol·Δt` per state and month (A.2).
        #[serde(rename = "solarWeights")]
        solar_weights: Vec<Vec<f64>>,
        /// Share of `Σ Δθ_int-e·Δt` per state and month (A.1).
        #[serde(rename = "temperatureWeights")]
        temperature_weights: Vec<Vec<f64>>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Method B: one state for every month.
    SingleState {
        state: DynamicState,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct DynamicIssue {
    pub code: &'static str,
    pub path: String,
}

fn weighted(states: &[DynamicState], weights: &[f64], value: fn(&DynamicState) -> f64) -> f64 {
    states
        .iter()
        .zip(weights)
        .map(|(state, weight)| value(state) * weight)
        .sum()
}

impl DynamicTransparent {
    /// `g_mi;mn` for month index 0–11 (A.2).
    pub fn g_for_month(&self, month_index: usize) -> f64 {
        match self {
            Self::WeightedStates {
                states,
                solar_weights,
                ..
            } => weighted(states, &solar_weights[month_index], |s| s.g_perpendicular),
            Self::SingleState { state, .. } => state.g_perpendicular,
        }
    }

    /// `U_mi;mn` for month index 0–11 (A.1).
    pub fn u_for_month(&self, month_index: usize) -> f64 {
        match self {
            Self::WeightedStates {
                states,
                temperature_weights,
                ..
            } => weighted(states, &temperature_weights[month_index], |s| {
                s.u_value_w_per_m2k
            }),
            Self::SingleState { state, .. } => state.u_value_w_per_m2k,
        }
    }

    pub fn validate(&self, path: &str) -> Vec<DynamicIssue> {
        let mut issues = Vec::new();
        let mut push = |code, field: &str| {
            issues.push(DynamicIssue {
                code,
                path: format!("{path}.{field}"),
            })
        };
        let (states, source) = match self {
            Self::WeightedStates {
                states,
                source_reference,
                ..
            } => (states.as_slice(), source_reference),
            Self::SingleState {
                state,
                source_reference,
            } => (std::slice::from_ref(state), source_reference),
        };
        if source.trim().is_empty() {
            push("source_reference_required", "sourceReference");
        }
        if states.is_empty() {
            push("dynamic_state_required", "states");
        }
        for state in states {
            if !(0.0..=1.0).contains(&state.g_perpendicular)
                || !(state.u_value_w_per_m2k.is_finite() && state.u_value_w_per_m2k > 0.0)
            {
                push("dynamic_state_invalid", "states");
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
                let valid = rows.len() == 12
                    && rows.iter().all(|row| {
                        row.len() == states.len()
                            && row.iter().all(|w| (0.0..=1.0).contains(w))
                            && (row.iter().sum::<f64>() - 1.0).abs() < 1e-6
                    });
                if !valid {
                    push("dynamic_weights_invalid", field);
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
                g_perpendicular: 0.6,
                u_value_w_per_m2k: 1.0,
            },
            DynamicState {
                id: "tinted".into(),
                g_perpendicular: 0.2,
                u_value_w_per_m2k: 1.4,
            },
        ];
        let element = DynamicTransparent::WeightedStates {
            states,
            solar_weights: vec![vec![0.75, 0.25]; 12],
            temperature_weights: vec![vec![0.5, 0.5]; 12],
            source_reference: "control strategy".into(),
        };
        assert!(element.validate("w").is_empty());
        assert!((element.g_for_month(6) - (0.75 * 0.6 + 0.25 * 0.2)).abs() < 1e-12);
        assert!((element.u_for_month(0) - 1.2).abs() < 1e-12);
        let DynamicTransparent::WeightedStates { states, .. } = element else {
            unreachable!()
        };
        let broken = DynamicTransparent::WeightedStates {
            states,
            solar_weights: vec![vec![0.5, 0.4]; 12],
            temperature_weights: vec![vec![0.5, 0.5]; 11],
            source_reference: "x".into(),
        };
        assert_eq!(broken.validate("w").len(), 2);
    }
}
