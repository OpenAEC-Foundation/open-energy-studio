//! Effective internal heat capacity, NTA 8800:2025+C1:2026 annex B
//! (p. 772): `C_m;i = Σ ρ·c·d·A` (B.1) over the building constructions in or
//! around the zone, with the effective depth `d` measured from the inner
//! surface while the thermal resistance from that surface stays below
//! 0,25 m²K/W, at most 100 mm and at most half the construction thickness.
//! Internal partitions count from both sides (note 2). A suspended ceiling
//! with at least 15 % net open area is skipped for the resistance; it is
//! not a building construction, so its mass is not counted either.

use serde::{Deserialize, Serialize};

/// Resistance limit from the inner surface, m²K/W.
pub const RESISTANCE_LIMIT: f64 = 0.25;
/// Maximum effective depth, m.
pub const DEPTH_LIMIT_M: f64 = 0.10;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MassLayer {
    pub thickness_m: f64,
    pub conductivity_w_per_mk: f64,
    pub density_kg_per_m3: f64,
    /// Per NEN-EN-ISO 10456, J/(kg·K).
    pub specific_heat_j_per_kgk: f64,
    /// Suspended ceiling with ≥ 15 % net open area.
    #[serde(default)]
    pub open_suspended_ceiling: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MassElement {
    pub id: String,
    pub area_m2: f64,
    /// Layers from the zone-side surface outwards.
    pub layers: Vec<MassLayer>,
    /// Internal partition inside the zone: both faces count.
    #[serde(default)]
    pub both_sides: bool,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MassIssue {
    pub code: &'static str,
    pub path: String,
}

/// Heat capacity per m² from one face, J/(m²·K).
fn face_capacity<'a>(layers: impl Iterator<Item = &'a MassLayer>, total_thickness: f64) -> f64 {
    let limit = DEPTH_LIMIT_M.min(total_thickness / 2.0);
    let mut depth = 0.0;
    let mut resistance = 0.0;
    let mut capacity = 0.0;
    for layer in layers {
        if layer.open_suspended_ceiling {
            continue;
        }
        if depth >= limit || resistance >= RESISTANCE_LIMIT {
            break;
        }
        let layer_resistance = layer.thickness_m / layer.conductivity_w_per_mk;
        let by_resistance = if resistance + layer_resistance <= RESISTANCE_LIMIT {
            layer.thickness_m
        } else {
            (RESISTANCE_LIMIT - resistance) * layer.conductivity_w_per_mk
        };
        let take = by_resistance.min(limit - depth);
        capacity += layer.density_kg_per_m3 * layer.specific_heat_j_per_kgk * take;
        depth += take;
        resistance += take / layer.conductivity_w_per_mk;
    }
    capacity
}

/// B.1 for one element, J/K.
pub fn element_capacity_j_per_k(element: &MassElement) -> f64 {
    let total: f64 = element
        .layers
        .iter()
        .filter(|layer| !layer.open_suspended_ceiling)
        .map(|layer| layer.thickness_m)
        .sum();
    let mut per_m2 = face_capacity(element.layers.iter(), total);
    if element.both_sides {
        per_m2 += face_capacity(element.layers.iter().rev(), total);
    }
    per_m2 * element.area_m2
}

/// `C_m;i` of the zone, J/K.
pub fn zone_capacity_j_per_k(elements: &[MassElement]) -> f64 {
    elements.iter().map(element_capacity_j_per_k).sum()
}

pub fn validate_mass_elements(elements: &[MassElement], path: &str) -> Vec<MassIssue> {
    let mut issues = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let item = format!("{path}[{index}]");
        if element.source_reference.trim().is_empty() {
            issues.push(MassIssue {
                code: "source_reference_required",
                path: format!("{item}.sourceReference"),
            });
        }
        if !(element.area_m2.is_finite() && element.area_m2 > 0.0) || element.layers.is_empty() {
            issues.push(MassIssue {
                code: "mass_element_invalid",
                path: item.clone(),
            });
        }
        for (l, layer) in element.layers.iter().enumerate() {
            let positive = [
                layer.thickness_m,
                layer.conductivity_w_per_mk,
                layer.density_kg_per_m3,
                layer.specific_heat_j_per_kgk,
            ]
            .iter()
            .all(|value| value.is_finite() && *value > 0.0);
            if !positive {
                issues.push(MassIssue {
                    code: "mass_layer_invalid",
                    path: format!("{item}.layers[{l}]"),
                });
            }
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(d: f64, lambda: f64, rho: f64, c: f64) -> MassLayer {
        MassLayer {
            thickness_m: d,
            conductivity_w_per_mk: lambda,
            density_kg_per_m3: rho,
            specific_heat_j_per_kgk: c,
            open_suspended_ceiling: false,
        }
    }

    #[test]
    fn depth_limits_follow_b_1() {
        // 200 mm concrete (λ 2,0): R at 100 mm is 0,05 < 0,25, so d = 100 mm.
        let floor = MassElement {
            id: "floor".into(),
            area_m2: 10.0,
            layers: vec![layer(0.2, 2.0, 2400.0, 1000.0)],
            both_sides: false,
            source_reference: "s".into(),
        };
        assert!((element_capacity_j_per_k(&floor) - 2400.0 * 1000.0 * 0.1 * 10.0).abs() < 1e-6);
        // 12,5 mm gypsum (λ 0,25, R 0,05) then insulation (λ 0,04): 0,2 m²K/W
        // left → 8 mm of insulation; total 162,5 mm so the half limit is 81 mm.
        let wall = MassElement {
            id: "wall".into(),
            area_m2: 1.0,
            layers: vec![
                layer(0.0125, 0.25, 900.0, 1000.0),
                layer(0.15, 0.04, 30.0, 1450.0),
            ],
            both_sides: false,
            source_reference: "s".into(),
        };
        let expected = 900.0 * 1000.0 * 0.0125 + 30.0 * 1450.0 * 0.008;
        assert!((element_capacity_j_per_k(&wall) - expected).abs() < 1e-6);
        // 100 mm brick partition from both sides: 50 mm each (half limit).
        let partition = MassElement {
            id: "p".into(),
            area_m2: 2.0,
            layers: vec![layer(0.1, 0.9, 1900.0, 840.0)],
            both_sides: true,
            source_reference: "s".into(),
        };
        let expected = 2.0 * 2.0 * 1900.0 * 840.0 * 0.05;
        assert!((element_capacity_j_per_k(&partition) - expected).abs() < 1e-6);
    }
}
