//! Steady-state ground heat transfer of a slab on ground, NTA 8800 §8.3
//! (via NEN-EN-ISO 13370), formulas 8.30, 8.32, 8.35, 8.36, 8.40 and 8.41.
//!
//! Transcription source: Open Heatloss Studio norm analysis C1 (2026-07-13).
//! The monthly periodic term of annex D is not applied; `H_g` is used as the
//! annual coefficient `H_g;an` in 7.14 against the annual mean outdoor
//! temperature. Suspended floors, basements and edge insulation are not
//! covered and are rejected by the caller-facing input types.

use serde::{Deserialize, Serialize};

/// 8.35: thermal conductivity of the ground in W/(mK).
pub const LAMBDA_GROUND: f64 = 2.0;
/// §8.3.2.3: full wall thickness used for the equivalent thickness, m.
pub const WALL_THICKNESS_M: f64 = 0.5;
/// §8.3.2.3 note 8: external surface resistance for ground contact, m²K/W.
pub const R_SE_GROUND: f64 = 0.04;

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
    pub source_reference: String,
}

/// Returns `H_g` in W/K, or `None` for non-physical input.
pub fn slab_on_ground_conductance(slab: &SlabOnGround) -> Option<f64> {
    let (area, perimeter, resistance) = (
        slab.area_m2,
        slab.exposed_perimeter_m,
        slab.construction_resistance_m2k_per_w,
    );
    if !(area.is_finite() && perimeter.is_finite() && resistance.is_finite())
        || area <= 0.0
        || perimeter <= 0.0
        || resistance <= 0.0
    {
        return None;
    }
    // 8.30
    let b_prime = area / (0.5 * perimeter);
    // 8.32
    let d_equivalent = WALL_THICKNESS_M + LAMBDA_GROUND * (resistance + R_SE_GROUND);
    let pi = std::f64::consts::PI;
    let u_floor = if d_equivalent < b_prime {
        // 8.40: uninsulated or moderately insulated.
        2.0 * LAMBDA_GROUND / (pi * b_prime + d_equivalent)
            * (pi * b_prime / d_equivalent + 1.0).ln()
    } else {
        // 8.41: well insulated.
        LAMBDA_GROUND / (0.457 * b_prime + d_equivalent)
    };
    // 8.36
    Some(area * u_floor)
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
        let pi = std::f64::consts::PI;
        let expected = 200.0 * 4.0 / (pi * b + d) * (pi * b / d + 1.0).ln();
        assert!((h - expected).abs() < 1e-12);
    }

    #[test]
    fn rejects_non_physical_input() {
        assert!(slab_on_ground_conductance(&slab(0.0, 10.0, 1.0)).is_none());
        assert!(slab_on_ground_conductance(&slab(10.0, 0.0, 1.0)).is_none());
        assert!(slab_on_ground_conductance(&slab(10.0, 10.0, f64::NAN)).is_none());
    }
}
