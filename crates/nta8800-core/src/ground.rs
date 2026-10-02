//! Ground heat transfer of a slab on ground, NTA 8800 §8.3 (p. 250–257) and
//! annex D (p. 790–795).
//!
//! - Steady state: 8.30, 8.32, 8.35, 8.40/8.41 for `U_fl`, then 8.36
//!   `H_g = A·U_fl + Σ ℓ·ψ_gr` (detailed floor-edge bridges) or 8.37
//!   `H_g;for = A·U_fl + 0,5·P` (forfait).
//! - Monthly: D.4 heat flow `Φ_mi` with the periodic coefficients D.5 and
//!   D.6 (no edge insulation), D.7 (horizontal edge insulation) or D.8
//!   (vertical edge insulation; the smaller of D.7/D.8 when both exist),
//!   phase shifts from table D.1 and `δ = 3 m`; D.1 `H_g;an;mi`; D.2/D.3 the
//!   seasonal `H_H;g;adj` and `H_C;g;adj` for the time constant.
//!
//! Suspended floors, basements and crawlspaces are not covered.

use crate::climate::{ANNUAL_MEAN_OUTDOOR_TEMPERATURE_C, OUTDOOR_TEMPERATURE_C};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// 8.35: thermal conductivity of the ground in W/(mK).
pub const LAMBDA_GROUND: f64 = 2.0;
/// §8.3.2.3: full wall thickness used for the equivalent thickness, m.
pub const WALL_THICKNESS_M: f64 = 0.5;
/// §8.3.2.3 note 8: external surface resistance for ground contact, m²K/W.
pub const R_SE_GROUND: f64 = 0.04;
/// D.2.2.1: periodic penetration depth, m.
pub const PENETRATION_DEPTH_M: f64 = 3.0;
/// D.2.1: amplitude of the monthly mean indoor temperature, K.
pub const INDOOR_AMPLITUDE_K: f64 = 2.0;
/// D.2.1: amplitude of the monthly mean outdoor temperature (NEN 5060), K.
pub const OUTDOOR_AMPLITUDE_K: f64 = 7.9;
/// D.2.1: month with the lowest mean outdoor temperature.
pub const COLDEST_MONTH: f64 = 1.0;
/// Table D.1: horizontal edge insulation counts as edge-insulated from this
/// resistance on, m²K/W.
pub const EDGE_INSULATION_MIN_RESISTANCE: f64 = 2.0;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SlabOnGround {
    pub id: String,
    /// `A_fl` in m².
    pub area_m2: f64,
    /// Exposed perimeter `P` in m (edges bordering outdoor air or unheated
    /// space outside the thermal envelope).
    pub exposed_perimeter_m: f64,
    /// `R_si + R_c` of the floor construction in m²K/W (no `R_se`).
    pub construction_resistance_m2k_per_w: f64,
    /// Floor-edge linear thermal bridges of 8.36, or the 8.37 forfait.
    pub edge_thermal_bridges: EdgeThermalBridges,
    /// Edge insulation for D.7/D.8 and table D.1; empty when absent.
    #[serde(default)]
    pub edge_insulation: Vec<EdgeInsulation>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum EdgeThermalBridges {
    /// 8.36: `Σ ℓ_j·ψ_gr;j` along the floor perimeter (§8.3.6).
    Detailed { bridges: Vec<GroundEdgeBridge> },
    /// 8.37: `+ 0,5·P` W/K.
    Forfait,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroundEdgeBridge {
    pub length_m: f64,
    pub psi_w_per_mk: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeInsulationKind {
    /// Edge zone of the floor insulated thicker than the middle (D.7).
    Horizontal,
    /// Vertical insulation at the floor edge or foundation (D.8).
    Vertical,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EdgeInsulation {
    pub kind: EdgeInsulationKind,
    /// `R_n` of the edge insulation layer, m²K/W.
    pub resistance_m2k_per_w: f64,
    /// `d_n` of the edge insulation layer, m.
    pub thickness_m: f64,
    pub source_reference: String,
}

/// Steady and periodic coefficients of one slab.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlabCoefficients {
    /// `H_g` (8.36/8.37), W/K.
    pub steady_w_per_k: f64,
    /// `H_pi` (D.5), W/K.
    pub periodic_internal_w_per_k: f64,
    /// `H_pe` (D.6–D.8), W/K.
    pub periodic_external_w_per_k: f64,
    /// Table D.1 phase shifts in months.
    pub alpha: f64,
    pub beta: f64,
}

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// 8.32: equivalent floor thickness `d_f;equi`.
fn equivalent_thickness(resistance: f64) -> f64 {
    WALL_THICKNESS_M + LAMBDA_GROUND * (resistance + R_SE_GROUND)
}

/// `A·U_fl` (8.30, 8.32, 8.40/8.41) in W/K, or `None` for non-physical input.
pub fn slab_floor_conductance(area: f64, perimeter: f64, resistance: f64) -> Option<f64> {
    if !(positive(area) && positive(perimeter) && positive(resistance)) {
        return None;
    }
    // 8.30
    let b_prime = area / (0.5 * perimeter);
    let d_equivalent = equivalent_thickness(resistance);
    let u_floor = if d_equivalent < b_prime {
        // 8.40: uninsulated or moderately insulated.
        2.0 * LAMBDA_GROUND / (PI * b_prime + d_equivalent)
            * (PI * b_prime / d_equivalent + 1.0).ln()
    } else {
        // 8.41: well insulated.
        LAMBDA_GROUND / (0.457 * b_prime + d_equivalent)
    };
    Some(area * u_floor)
}

/// `H_g` of 8.36/8.37 in W/K, or `None` for non-physical input.
pub fn slab_on_ground_conductance(slab: &SlabOnGround) -> Option<f64> {
    let floor = slab_floor_conductance(
        slab.area_m2,
        slab.exposed_perimeter_m,
        slab.construction_resistance_m2k_per_w,
    )?;
    let edge = match &slab.edge_thermal_bridges {
        EdgeThermalBridges::Forfait => 0.5 * slab.exposed_perimeter_m,
        EdgeThermalBridges::Detailed { bridges } => {
            let mut sum = 0.0;
            for bridge in bridges {
                if !(positive(bridge.length_m) && bridge.psi_w_per_mk.is_finite()) {
                    return None;
                }
                sum += bridge.length_m * bridge.psi_w_per_mk;
            }
            sum
        }
    };
    Some(floor + edge)
}

/// D.7 (horizontal) and D.8 (vertical) with D.9 `d'`, as printed in the
/// norm (exponents in `d_f;equi/δ`).
fn edge_insulated_external(perimeter: f64, d_f: f64, insulation: &EdgeInsulation) -> f64 {
    let delta = PENETRATION_DEPTH_M;
    // D.9
    let d_prime =
        LAMBDA_GROUND * (insulation.resistance_m2k_per_w - insulation.thickness_m / LAMBDA_GROUND);
    let second_exponent = match insulation.kind {
        EdgeInsulationKind::Horizontal => 2.0 * d_f / delta,
        EdgeInsulationKind::Vertical => d_f / delta,
    };
    0.37 * perimeter
        * LAMBDA_GROUND
        * ((1.0 - (-(d_f / delta)).exp()) * (delta / (d_f + d_prime) + 1.0).ln()
            + (-second_exponent).exp() * (delta / d_f + 1.0).ln())
}

/// Steady and periodic coefficients, or `None` for non-physical input.
pub fn slab_coefficients(slab: &SlabOnGround) -> Option<SlabCoefficients> {
    let steady = slab_on_ground_conductance(slab)?;
    for insulation in &slab.edge_insulation {
        if !(positive(insulation.resistance_m2k_per_w) && positive(insulation.thickness_m)) {
            return None;
        }
    }
    let delta = PENETRATION_DEPTH_M;
    let d_f = equivalent_thickness(slab.construction_resistance_m2k_per_w);
    // D.5
    let internal =
        slab.area_m2 * LAMBDA_GROUND / d_f * (2.0 / ((1.0 + delta / d_f).powi(2) + 1.0)).sqrt();
    let external = if slab.edge_insulation.is_empty() {
        // D.6
        0.37 * slab.exposed_perimeter_m * LAMBDA_GROUND * (delta / d_f + 1.0).ln()
    } else {
        slab.edge_insulation
            .iter()
            .map(|insulation| edge_insulated_external(slab.exposed_perimeter_m, d_f, insulation))
            .fold(f64::INFINITY, f64::min)
    };
    // Table D.1: edge-insulated slab (0, 2), otherwise (0, 1).
    let edge_insulated = slab.edge_insulation.iter().any(|insulation| {
        insulation.kind == EdgeInsulationKind::Vertical
            || insulation.resistance_m2k_per_w >= EDGE_INSULATION_MIN_RESISTANCE
    });
    let values = [steady, internal, external];
    values
        .iter()
        .all(|value| value.is_finite())
        .then_some(SlabCoefficients {
            steady_w_per_k: steady,
            periodic_internal_w_per_k: internal,
            periodic_external_w_per_k: external,
            alpha: 0.0,
            beta: if edge_insulated { 2.0 } else { 1.0 },
        })
}

/// D.4: mean ground heat flow `Φ_mi` in W for month 1–12, with `θ̄_i` the
/// annual mean indoor temperature (the heating setpoint of the function).
pub fn monthly_heat_flow_w(coefficients: &SlabCoefficients, indoor_mean_c: f64, month: u8) -> f64 {
    let m = f64::from(month);
    coefficients.steady_w_per_k * (indoor_mean_c - ANNUAL_MEAN_OUTDOOR_TEMPERATURE_C)
        - coefficients.periodic_internal_w_per_k
            * INDOOR_AMPLITUDE_K
            * (2.0 * PI * (m - COLDEST_MONTH + coefficients.alpha) / 12.0).cos()
        + coefficients.periodic_external_w_per_k
            * OUTDOOR_AMPLITUDE_K
            * (2.0 * PI * (m - COLDEST_MONTH - coefficients.beta) / 12.0).cos()
}

/// Monthly and seasonal ground coefficients of a zone.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundMonthly {
    /// D.1 `H_g;an;mi`, W/K.
    pub monthly_w_per_k: [f64; 12],
    /// D.2 `H_H;g;adj`, W/K.
    pub heating_adjusted_w_per_k: f64,
    /// D.3 `H_C;g;adj`, W/K.
    pub cooling_adjusted_w_per_k: f64,
}

/// D.1–D.3 for the summed monthly heat flows of all slabs, with `θ_e;an`
/// the annual mean outdoor temperature used in 7.14.
pub fn ground_monthly(
    slabs: &[SlabCoefficients],
    heating_setpoint_c: f64,
    cooling_setpoint_c: f64,
    annual_outdoor_c: f64,
) -> GroundMonthly {
    let mut monthly = [0.0; 12];
    for (index, value) in monthly.iter_mut().enumerate() {
        let month = index as u8 + 1;
        let flow: f64 = slabs
            .iter()
            .map(|slab| monthly_heat_flow_w(slab, heating_setpoint_c, month))
            .sum();
        // D.1
        *value = flow / (heating_setpoint_c - annual_outdoor_c);
    }
    let seasonal = |months: &[usize], setpoint: f64| {
        let mean_h = months.iter().map(|&i| monthly[i]).sum::<f64>() / 6.0;
        let mean_dt = months
            .iter()
            .map(|&i| setpoint - OUTDOOR_TEMPERATURE_C[i])
            .sum::<f64>()
            / (6.0 * (setpoint - annual_outdoor_c));
        mean_h * mean_dt
    };
    GroundMonthly {
        monthly_w_per_k: monthly,
        // D.2: October–March.
        heating_adjusted_w_per_k: seasonal(&[9, 10, 11, 0, 1, 2], heating_setpoint_c),
        // D.3: April–September.
        cooling_adjusted_w_per_k: seasonal(&[3, 4, 5, 6, 7, 8], cooling_setpoint_c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slab(area: f64, perimeter: f64, resistance: f64) -> SlabOnGround {
        SlabOnGround {
            id: "floor".into(),
            area_m2: area,
            exposed_perimeter_m: perimeter,
            construction_resistance_m2k_per_w: resistance,
            edge_thermal_bridges: EdgeThermalBridges::Detailed { bridges: vec![] },
            edge_insulation: vec![],
            source_reference: "test".into(),
        }
    }

    #[test]
    fn well_insulated_branch_matches_hand_calculation() {
        // Aalten example from analysis C1: A 67 m², P 32,92 m, U 0,258398.
        let resistance = 1.0 / 0.258_398;
        let h = slab_on_ground_conductance(&slab(67.0, 32.92, resistance)).unwrap();
        let b = 67.0 / (0.5 * 32.92);
        let d = 0.5 + 2.0 * (resistance + 0.04);
        assert!(d >= b);
        assert!((h - 67.0 * 2.0 / (0.457 * b + d)).abs() < 1e-12);
        assert!((h - 13.163).abs() < 1e-3);
    }

    #[test]
    fn uninsulated_branch_uses_logarithm() {
        let h = slab_on_ground_conductance(&slab(200.0, 40.0, 0.2)).unwrap();
        let b = 200.0 / 20.0;
        let d = 0.5 + 2.0 * 0.24;
        assert!(d < b);
        let expected = 200.0 * 4.0 / (PI * b + d) * (PI * b / d + 1.0).ln();
        assert!((h - expected).abs() < 1e-12);
    }

    #[test]
    fn floor_edge_term_follows_8_36_or_8_37() {
        let base = slab_on_ground_conductance(&slab(67.0, 32.92, 3.5)).unwrap();
        let mut detailed = slab(67.0, 32.92, 3.5);
        detailed.edge_thermal_bridges = EdgeThermalBridges::Detailed {
            bridges: vec![GroundEdgeBridge {
                length_m: 32.92,
                psi_w_per_mk: 0.1,
                source_reference: "detail".into(),
            }],
        };
        let with_edge = slab_on_ground_conductance(&detailed).unwrap();
        assert!((with_edge - base - 3.292).abs() < 1e-12);
        let mut forfait = slab(67.0, 32.92, 3.5);
        forfait.edge_thermal_bridges = EdgeThermalBridges::Forfait;
        let with_forfait = slab_on_ground_conductance(&forfait).unwrap();
        assert!((with_forfait - base - 16.46).abs() < 1e-12);
    }

    #[test]
    fn periodic_coefficients_follow_d5_d6_and_table_d1() {
        let floor = slab(100.0, 40.0, 3.5);
        let c = slab_coefficients(&floor).unwrap();
        let d: f64 = 0.5 + 2.0 * 3.54;
        let h_pi = 100.0 * 2.0 / d * (2.0 / ((1.0 + 3.0 / d).powi(2) + 1.0)).sqrt();
        let h_pe = 0.37 * 40.0 * 2.0 * (3.0 / d + 1.0).ln();
        assert!((c.periodic_internal_w_per_k - h_pi).abs() < 1e-12);
        assert!((c.periodic_external_w_per_k - h_pe).abs() < 1e-12);
        assert_eq!((c.alpha, c.beta), (0.0, 1.0));
        // January (m = 1): cos(0) = 1 for the internal term, cos(-π/6) for the
        // external term with β = 1.
        let phi = monthly_heat_flow_w(&c, 20.0, 1);
        let expected =
            c.steady_w_per_k * (20.0 - 10.67) - h_pi * 2.0 + h_pe * 7.9 * (-PI / 6.0).cos();
        assert!((phi - expected).abs() < 1e-9);
    }

    #[test]
    fn edge_insulation_uses_d7_d8_minimum_and_shifts_phase() {
        let mut floor = slab(100.0, 40.0, 3.5);
        let vertical = EdgeInsulation {
            kind: EdgeInsulationKind::Vertical,
            resistance_m2k_per_w: 2.5,
            thickness_m: 0.1,
            source_reference: "detail".into(),
        };
        floor.edge_insulation = vec![vertical.clone()];
        let c = slab_coefficients(&floor).unwrap();
        let d: f64 = 0.5 + 2.0 * 3.54;
        let d_prime = 2.0 * (2.5 - 0.1 / 2.0);
        let e = (-(d / 3.0)).exp();
        let d8 = 0.37
            * 40.0
            * 2.0
            * ((1.0 - e) * (3.0 / (d + d_prime) + 1.0).ln() + e * (3.0 / d + 1.0).ln());
        assert!((c.periodic_external_w_per_k - d8).abs() < 1e-12);
        assert_eq!(c.beta, 2.0);
        let horizontal = EdgeInsulation {
            kind: EdgeInsulationKind::Horizontal,
            resistance_m2k_per_w: 1.0,
            ..vertical
        };
        floor.edge_insulation = vec![horizontal];
        let c = slab_coefficients(&floor).unwrap();
        let d_prime = 2.0 * (1.0 - 0.1 / 2.0);
        let d7 = 0.37
            * 40.0
            * 2.0
            * ((1.0 - e) * (3.0 / (d + d_prime) + 1.0).ln()
                + (-(2.0 * d / 3.0)).exp() * (3.0 / d + 1.0).ln());
        assert!((c.periodic_external_w_per_k - d7).abs() < 1e-12);
        // Horizontal edge insulation below R 2,0 stays in "all other cases".
        assert_eq!(c.beta, 1.0);
    }

    #[test]
    fn seasonal_coefficients_follow_d1_to_d3() {
        let c = slab_coefficients(&slab(100.0, 40.0, 3.5)).unwrap();
        let annual = 10.6725;
        let g = ground_monthly(&[c], 20.0, 24.0, annual);
        for month in 1..=12u8 {
            let phi = monthly_heat_flow_w(&c, 20.0, month);
            assert!(
                (g.monthly_w_per_k[usize::from(month - 1)] - phi / (20.0 - annual)).abs() < 1e-12
            );
        }
        let heating_months = [9, 10, 11, 0, 1, 2];
        let mean_h: f64 = heating_months
            .iter()
            .map(|&i| g.monthly_w_per_k[i])
            .sum::<f64>()
            / 6.0;
        let mean_dt: f64 = heating_months
            .iter()
            .map(|&i| 20.0 - OUTDOOR_TEMPERATURE_C[i])
            .sum::<f64>()
            / (6.0 * (20.0 - annual));
        assert!((g.heating_adjusted_w_per_k - mean_h * mean_dt).abs() < 1e-12);
        // Annual mean of D.1 equals the steady coefficient (cosines cancel).
        let mean: f64 = g.monthly_w_per_k.iter().sum::<f64>() / 12.0;
        assert!((mean - c.steady_w_per_k * (20.0 - 10.67) / (20.0 - annual)).abs() < 1e-9);
    }

    #[test]
    fn rejects_non_physical_input() {
        assert!(slab_on_ground_conductance(&slab(0.0, 10.0, 1.0)).is_none());
        assert!(slab_on_ground_conductance(&slab(10.0, 0.0, 1.0)).is_none());
        assert!(slab_on_ground_conductance(&slab(10.0, 10.0, f64::NAN)).is_none());
        let mut bad = slab(10.0, 10.0, 1.0);
        bad.edge_insulation = vec![EdgeInsulation {
            kind: EdgeInsulationKind::Vertical,
            resistance_m2k_per_w: 0.0,
            thickness_m: 0.1,
            source_reference: "x".into(),
        }];
        assert!(slab_coefficients(&bad).is_none());
    }
}
