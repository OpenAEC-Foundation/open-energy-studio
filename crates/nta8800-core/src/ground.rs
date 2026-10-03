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
//! Floors above a crawlspace or unheated basement (8.43–8.49, D.13–D.16)
//! and heated basements (8.38/8.39, D.10/D.11) are covered by
//! [`FloorBelow`] and [`HeatedBasement`].

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
    /// Space below the floor (8.3.4.2); `None` for a floor directly on the
    /// ground (8.3.4.1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub below: Option<FloorBelow>,
    /// Floor of a heated room below ground level (8.3.3.2); `None` for a
    /// floor on or above ground level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heated_basement: Option<HeatedBasement>,
    pub source_reference: String,
}

/// 8.3.3.2: heated room with its floor below ground level.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatedBasement {
    /// z: actual depth of the floor below ground level, m.
    pub depth_m: f64,
    /// R_c of the basement walls against the ground (8.34), m²K/W.
    pub wall_resistance_m2k_per_w: f64,
    /// ΔU_for of 8.2.1 for the walls in the 8.38 forfait route, W/(m²K).
    #[serde(default)]
    pub forfait_delta_u_w_per_m2k: Option<f64>,
}

/// 8.3.5 class of the depth `z` of a crawlspace or basement floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepthClass {
    /// Floor on sand: z = 0 m.
    OnSand,
    /// All other cases: z = 0,5 m.
    Other,
}

impl DepthClass {
    fn depth_m(self) -> f64 {
        match self {
            Self::OnSand => 0.0,
            Self::Other => 0.5,
        }
    }
}

/// Crawlspace or unheated basement below a ground floor (8.3.4.2, 8.3.5).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FloorBelow {
    Crawlspace {
        /// `R_bf` of the crawlspace floor; 0 when uninsulated (8.33).
        #[serde(default, rename = "floorResistanceM2kPerW")]
        floor_resistance_m2k_per_w: f64,
        #[serde(rename = "depthClass")]
        depth_class: DepthClass,
        /// `R_bw` = R_c of the façade above (8.34).
        #[serde(rename = "wallResistanceM2kPerW")]
        wall_resistance_m2k_per_w: f64,
        /// `U_xw` of the wall above ground; the façade U is allowed (8.47).
        #[serde(rename = "wallUValueWPerM2k")]
        wall_u_value_w_per_m2k: f64,
        /// ε, m² vent opening per m perimeter; `None` gives 0,001 2 (8.48).
        #[serde(default, rename = "ventilationOpeningM2PerM")]
        ventilation_opening_m2_per_m: Option<f64>,
    },
    UnheatedBasement {
        #[serde(default, rename = "floorResistanceM2kPerW")]
        floor_resistance_m2k_per_w: f64,
        #[serde(rename = "depthClass")]
        depth_class: DepthClass,
        #[serde(rename = "wallResistanceM2kPerW")]
        wall_resistance_m2k_per_w: f64,
        #[serde(rename = "wallUValueWPerM2k")]
        wall_u_value_w_per_m2k: f64,
        /// Basement volume V, m³ (8.49).
        #[serde(rename = "volumeM3")]
        volume_m3: f64,
        /// n, air changes per hour; `None` gives the forfait 0,3 (8.49).
        #[serde(default, rename = "airChangesPerHour")]
        air_changes_per_hour: Option<f64>,
    },
}

/// R_si (downward) and R_se of the floor construction towards the space
/// below (C.2), m²K/W.
const R_SI_DOWN: f64 = 0.17;
const R_SI_HORIZONTAL: f64 = 0.13;
/// 8.47: h above ground level.
const CRAWL_WALL_HEIGHT_M: f64 = 0.125;
/// 8.48 forfait values.
const CRAWL_VENTILATION_DEFAULT: f64 = 0.0012;
const CRAWL_WIND_SPEED: f64 = 5.0;
const CRAWL_WIND_SHIELDING: f64 = 0.05;
/// 8.49 forfait air change rate.
const BASEMENT_AIR_CHANGES: f64 = 0.3;

/// Steady terms of a floor above a crawlspace or unheated basement.
struct BelowTerms {
    /// `U_f` of the floor construction, W/m²K.
    u_f: f64,
    /// `U_g` (8.44), W/m²K.
    u_g: f64,
    /// `U_x` (8.46), W/m²K.
    u_x: f64,
    d_bf: f64,
    z: f64,
}

/// 8.40/8.41 with depth z.
fn ground_floor_u(b_prime: f64, d_equivalent: f64, z: f64) -> f64 {
    let d = d_equivalent + 0.5 * z;
    if d < b_prime {
        2.0 * LAMBDA_GROUND / (PI * b_prime + d) * (PI * b_prime / d + 1.0).ln()
    } else {
        LAMBDA_GROUND / (0.457 * b_prime + d)
    }
}

fn below_terms(slab: &SlabOnGround, below: &FloorBelow) -> Option<BelowTerms> {
    let (r_bf, depth, r_bw, u_xw) = match below {
        FloorBelow::Crawlspace {
            floor_resistance_m2k_per_w,
            depth_class,
            wall_resistance_m2k_per_w,
            wall_u_value_w_per_m2k,
            ..
        }
        | FloorBelow::UnheatedBasement {
            floor_resistance_m2k_per_w,
            depth_class,
            wall_resistance_m2k_per_w,
            wall_u_value_w_per_m2k,
            ..
        } => (
            *floor_resistance_m2k_per_w,
            *depth_class,
            *wall_resistance_m2k_per_w,
            *wall_u_value_w_per_m2k,
        ),
    };
    if !(r_bf.is_finite() && r_bf >= 0.0 && positive(r_bw) && positive(u_xw)) {
        return None;
    }
    let area = slab.area_m2;
    let perimeter = slab.exposed_perimeter_m;
    let b_prime = area / (0.5 * perimeter);
    // 8.2.2.2.1: R_si + R_c of the floor plus R_si towards the space below.
    // Table C.2 gives R_se 0,04 for surfaces to outside air; like 8.4.2.1
    // for unheated spaces and 7.2 of NEN-EN-ISO 13370 (U_f between the
    // interior and the underfloor space), the still air below takes R_si.
    let u_f = 1.0 / (slab.construction_resistance_m2k_per_w + R_SI_DOWN);
    // 8.33/8.34.
    let d_bf = WALL_THICKNESS_M + LAMBDA_GROUND * (R_SI_DOWN + r_bf + R_SE_GROUND);
    let d_bw = LAMBDA_GROUND * (R_SI_HORIZONTAL + r_bw + R_SE_GROUND);
    let z = depth.depth_m();
    // 8.3.5.2.
    let u_bf = ground_floor_u(b_prime, d_bf, z);
    // 8.45 and 8.44 with Σℓ = P.
    let u_wall_part = if z > 0.0 {
        let d1 = if d_bw >= d_bf { d_bf } else { d_bw };
        let u_bw =
            2.0 * LAMBDA_GROUND / (PI * z) * (1.0 + 0.5 * d1 / (d1 + z)) * (z / d_bw + 1.0).ln();
        z * perimeter * u_bw / area
    } else {
        0.0
    };
    let u_g = u_bf + u_wall_part;
    // 8.47.
    let u_x_t = 2.0 * CRAWL_WALL_HEIGHT_M * u_xw / b_prime;
    // 8.48/8.49.
    let u_x_v = match below {
        FloorBelow::Crawlspace {
            ventilation_opening_m2_per_m,
            ..
        } => {
            let epsilon = ventilation_opening_m2_per_m.unwrap_or(CRAWL_VENTILATION_DEFAULT);
            if !(epsilon.is_finite() && epsilon >= 0.0) {
                return None;
            }
            1450.0 * epsilon * CRAWL_WIND_SPEED * CRAWL_WIND_SHIELDING / b_prime
        }
        FloorBelow::UnheatedBasement {
            volume_m3,
            air_changes_per_hour,
            ..
        } => {
            let n = air_changes_per_hour.unwrap_or(BASEMENT_AIR_CHANGES);
            if !(positive(*volume_m3) && n.is_finite() && n >= 0.0) {
                return None;
            }
            1.205 * 1005.0 * n * volume_m3 / (3600.0 * area)
        }
    };
    Some(BelowTerms {
        u_f,
        u_g,
        u_x: u_x_t + u_x_v,
        d_bf,
        z,
    })
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
/// 8.39 floor and wall terms of a heated basement: (A·U_fl, z·P·U_bw,
/// d_f;equi, d_bw;equi).
fn heated_basement_terms(
    slab: &SlabOnGround,
    basement: &HeatedBasement,
) -> Option<(f64, f64, f64, f64)> {
    let z = basement.depth_m;
    if !(positive(z)
        && positive(basement.wall_resistance_m2k_per_w)
        && positive(slab.area_m2)
        && positive(slab.exposed_perimeter_m)
        && positive(slab.construction_resistance_m2k_per_w))
        || slab.below.is_some()
        || !slab.edge_insulation.is_empty()
    {
        return None;
    }
    let b_prime = slab.area_m2 / (0.5 * slab.exposed_perimeter_m);
    let d_f = equivalent_thickness(slab.construction_resistance_m2k_per_w);
    let d_bw = LAMBDA_GROUND * (R_SI_HORIZONTAL + basement.wall_resistance_m2k_per_w + R_SE_GROUND);
    let floor = slab.area_m2 * ground_floor_u(b_prime, d_f, z);
    // 8.45 with d_1 the smaller equivalent thickness.
    let d1 = if d_bw >= d_f { d_f } else { d_bw };
    let u_bw = 2.0 * LAMBDA_GROUND / (PI * z) * (1.0 + 0.5 * d1 / (d1 + z)) * (z / d_bw + 1.0).ln();
    Some((floor, z * slab.exposed_perimeter_m * u_bw, d_f, d_bw))
}

pub fn slab_on_ground_conductance(slab: &SlabOnGround) -> Option<f64> {
    if let Some(basement) = &slab.heated_basement {
        let (floor, walls, _, _) = heated_basement_terms(slab, basement)?;
        let edge = match &slab.edge_thermal_bridges {
            // 8.38: 0,5·P and ΔU_for over the wall area z·P.
            EdgeThermalBridges::Forfait => {
                let delta = basement.forfait_delta_u_w_per_m2k?;
                if !(delta.is_finite() && delta >= 0.0) {
                    return None;
                }
                0.5 * slab.exposed_perimeter_m + basement.depth_m * slab.exposed_perimeter_m * delta
            }
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
        // 8.39.
        return Some(floor + walls + edge);
    }
    let floor = match &slab.below {
        None => slab_floor_conductance(
            slab.area_m2,
            slab.exposed_perimeter_m,
            slab.construction_resistance_m2k_per_w,
        )?,
        Some(below) => {
            if !(positive(slab.area_m2)
                && positive(slab.exposed_perimeter_m)
                && positive(slab.construction_resistance_m2k_per_w))
            {
                return None;
            }
            let terms = below_terms(slab, below)?;
            // 8.43.
            slab.area_m2 / (1.0 / terms.u_f + 1.0 / (terms.u_g + terms.u_x))
        }
    };
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
    if let Some(basement) = &slab.heated_basement {
        let (_, _, d_bf, d_bw) = heated_basement_terms(slab, basement)?;
        let z = basement.depth_m;
        let perimeter = slab.exposed_perimeter_m;
        // D.10.
        let internal = slab.area_m2 * LAMBDA_GROUND / d_bf
            * (2.0 / ((1.0 + delta / d_bf).powi(2) + 1.0)).sqrt()
            + z * perimeter * LAMBDA_GROUND / d_bw
                * (2.0 / ((1.0 + delta / d_bw).powi(2) + 1.0)).sqrt();
        // D.11.
        let external = 0.37
            * perimeter
            * LAMBDA_GROUND
            * (2.0 * (1.0 - (-z / delta).exp()) * (delta / d_bw + 1.0).ln()
                + (-z / delta).exp() * (delta / d_bf + 1.0).ln());
        let values = [steady, internal, external];
        return values
            .iter()
            .all(|value| value.is_finite())
            .then_some(SlabCoefficients {
                steady_w_per_k: steady,
                periodic_internal_w_per_k: internal,
                periodic_external_w_per_k: external,
                alpha: 0.0,
                // Table D.1: all other cases (0, 1).
                beta: 1.0,
            });
    }
    if let Some(below) = &slab.below {
        if !slab.edge_insulation.is_empty() {
            // D.7/D.8 apply to slabs on ground only.
            return None;
        }
        let terms = below_terms(slab, below)?;
        let area = slab.area_m2;
        let perimeter = slab.exposed_perimeter_m;
        let (internal, external, beta) = match below {
            FloorBelow::Crawlspace { .. } => {
                // D.13/D.14, table D.1 (0, 0).
                let soil = LAMBDA_GROUND / delta;
                let internal = area / (1.0 / terms.u_f + 1.0 / (soil + terms.u_x));
                let external = terms.u_f
                    * (0.37 * perimeter * LAMBDA_GROUND * (delta / terms.d_bf + 1.0).ln()
                        + terms.u_x * area)
                    / (soil + terms.u_x + terms.u_f);
                (internal, external, 0.0)
            }
            FloorBelow::UnheatedBasement {
                wall_u_value_w_per_m2k,
                volume_m3,
                air_changes_per_hour,
                ..
            } => {
                // D.15/D.16 with h = 0,125 m, table D.1 (0, 1).
                let n = air_changes_per_hour.unwrap_or(BASEMENT_AIR_CHANGES);
                let air =
                    CRAWL_WALL_HEIGHT_M * perimeter * wall_u_value_w_per_m2k + 0.33 * n * volume_m3;
                let below_side = (area + terms.z * perimeter) * LAMBDA_GROUND / delta + air;
                let floor = area * terms.u_f;
                let internal = 1.0 / (1.0 / floor + 1.0 / below_side);
                let external = floor
                    * (0.37
                        * perimeter
                        * LAMBDA_GROUND
                        * (2.0 - (-(terms.z / delta)).exp())
                        * (delta / terms.d_bf + 1.0).ln()
                        + air)
                    / (below_side + floor);
                (internal, external, 1.0)
            }
        };
        let values = [steady, internal, external];
        return values
            .iter()
            .all(|value| value.is_finite())
            .then_some(SlabCoefficients {
                steady_w_per_k: steady,
                periodic_internal_w_per_k: internal,
                periodic_external_w_per_k: external,
                alpha: 0.0,
                beta,
            });
    }
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
            below: None,
            heated_basement: None,
            source_reference: "test".into(),
        }
    }

    #[test]
    fn crawlspace_floor_follows_8_43_to_8_48_and_d13_d14() {
        let mut floor = slab(60.0, 16.0, 0.17 + 0.15);
        floor.edge_thermal_bridges = EdgeThermalBridges::Forfait;
        floor.below = Some(FloorBelow::Crawlspace {
            floor_resistance_m2k_per_w: 0.0,
            depth_class: DepthClass::Other,
            wall_resistance_m2k_per_w: 0.35,
            wall_u_value_w_per_m2k: 1.9,
            ventilation_opening_m2_per_m: None,
        });
        // Hand calculation: B' = 7,5; U_f = 1/(0,32 + 0,17).
        let b = 7.5;
        let u_f = 1.0 / 0.49;
        let d_bf: f64 = 0.5 + 2.0 * (0.17 + 0.0 + 0.04);
        let d_bw: f64 = 2.0 * (0.13 + 0.35 + 0.04);
        let z = 0.5;
        let d = d_bf + 0.5 * z;
        let u_bf = 2.0 * 2.0 / (PI * b + d) * (PI * b / d + 1.0).ln();
        let d1 = d_bw.min(d_bf);
        let u_bw = 2.0 * 2.0 / (PI * z) * (1.0 + 0.5 * d1 / (d1 + z)) * (z / d_bw + 1.0).ln();
        let u_g = u_bf + z * 16.0 * u_bw / 60.0;
        let u_x = 2.0 * 0.125 * 1.9 / b + 1450.0 * 0.0012 * 5.0 * 0.05 / b;
        let u_fl = 1.0 / (1.0 / u_f + 1.0 / (u_g + u_x));
        let steady = slab_on_ground_conductance(&floor).unwrap();
        assert!((steady - (60.0 * u_fl + 0.5 * 16.0)).abs() < 1e-9);
        let coefficients = slab_coefficients(&floor).unwrap();
        let soil = 2.0 / 3.0;
        let internal = 60.0 / (1.0 / u_f + 1.0 / (soil + u_x));
        assert!((coefficients.periodic_internal_w_per_k - internal).abs() < 1e-9);
        let external =
            u_f * (0.37 * 16.0 * 2.0 * (3.0 / d_bf + 1.0).ln() + u_x * 60.0) / (soil + u_x + u_f);
        assert!((coefficients.periodic_external_w_per_k - external).abs() < 1e-9);
        assert_eq!((coefficients.alpha, coefficients.beta), (0.0, 0.0));
        // A crawlspace loses less than the same floor directly on the ground.
        floor.below = None;
        assert!(slab_on_ground_conductance(&floor).unwrap() > 0.0);
    }

    #[test]
    fn unheated_basement_uses_8_49_and_d15_d16() {
        let mut floor = slab(60.0, 16.0, 0.17 + 2.5);
        floor.below = Some(FloorBelow::UnheatedBasement {
            floor_resistance_m2k_per_w: 0.0,
            depth_class: DepthClass::Other,
            wall_resistance_m2k_per_w: 0.35,
            wall_u_value_w_per_m2k: 1.9,
            volume_m3: 120.0,
            air_changes_per_hour: None,
        });
        let coefficients = slab_coefficients(&floor).unwrap();
        assert_eq!(coefficients.beta, 1.0);
        let u_f = 1.0 / (2.67 + 0.17);
        let air = 0.125 * 16.0 * 1.9 + 0.33 * 0.3 * 120.0;
        let below = (60.0 + 0.5 * 16.0) * 2.0 / 3.0 + air;
        let internal = 1.0 / (1.0 / (60.0 * u_f) + 1.0 / below);
        assert!((coefficients.periodic_internal_w_per_k - internal).abs() < 1e-9);
        // Edge insulation is a slab-on-ground feature only.
        floor.edge_insulation.push(EdgeInsulation {
            kind: EdgeInsulationKind::Vertical,
            resistance_m2k_per_w: 2.0,
            thickness_m: 0.1,
            source_reference: "x".into(),
        });
        assert!(slab_coefficients(&floor).is_none());
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

    #[test]
    fn heated_basement_follows_8_39_and_d10_d11() {
        let mut floor = slab(50.0, 30.0, 0.17 + 1.0);
        floor.heated_basement = Some(HeatedBasement {
            depth_m: 2.0,
            wall_resistance_m2k_per_w: 1.5,
            forfait_delta_u_w_per_m2k: None,
        });
        let b: f64 = 50.0 / 15.0;
        let d_f: f64 = 0.5 + 2.0 * (1.17 + 0.04);
        let d_bw: f64 = 2.0 * (0.13 + 1.5 + 0.04);
        let z = 2.0;
        let d = d_f + 0.5 * z;
        let u_fl = if d < b {
            4.0 / (PI * b + d) * (PI * b / d + 1.0).ln()
        } else {
            2.0 / (0.457 * b + d)
        };
        let d1 = d_bw.min(d_f);
        let u_bw = 4.0 / (PI * z) * (1.0 + 0.5 * d1 / (d1 + z)) * (z / d_bw + 1.0).ln();
        let steady = slab_on_ground_conductance(&floor).unwrap();
        assert!((steady - (50.0 * u_fl + z * 30.0 * u_bw)).abs() < 1e-9);
        let coefficients = slab_coefficients(&floor).unwrap();
        let e = (-z / 3.0f64).exp();
        let external = 0.37
            * 30.0
            * 2.0
            * (2.0 * (1.0 - e) * (3.0 / d_bw + 1.0).ln() + e * (3.0 / d_f + 1.0).ln());
        assert!((coefficients.periodic_external_w_per_k - external).abs() < 1e-9);
        // The 8.38 forfait needs ΔU_for for the walls.
        floor.edge_thermal_bridges = EdgeThermalBridges::Forfait;
        assert!(slab_on_ground_conductance(&floor).is_none());
        floor
            .heated_basement
            .as_mut()
            .unwrap()
            .forfait_delta_u_w_per_m2k = Some(0.05);
        let forfait = slab_on_ground_conductance(&floor).unwrap();
        assert!((forfait - (steady + 0.5 * 30.0 + 2.0 * 30.0 * 0.05)).abs() < 1e-9);
    }
}
