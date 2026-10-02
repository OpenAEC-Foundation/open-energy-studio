//! Schematisation checks of NTA 8800:2025+C1:2026 §6.4 and §6.5.2
//! (pp. 128–131).
//!
//! §6.4: a conditioning zone has at most one heating, one cooling and one
//! humidification system, and at least 80 % of its floor area has one
//! ventilation system variant; a zone smaller than 10 % of an adjoining one
//! may be merged into it (10 % rule).
//!
//! §6.5.2: a calculation zone with a residential function holds no other
//! functions; within a zone (a) heating setpoints differ at most 4 K unless
//! the largest function covers at least 90 %, (b) with ventilation systems
//! A, B, C or E the specific capacity q_usi;spec differs at most a factor 4
//! unless the residence areas are openly connected or more than 80 % has the
//! same capacity, and (c) the specific internal heat capacity differs at
//! most a factor 3 unless more than 80 % has the same value.

use serde::{Deserialize, Serialize};

use crate::monthly_demand::UsageFunction;
use crate::ventilation::{VentSysOp, VentilationFunction};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZonePart {
    pub function: UsageFunction,
    pub area_m2: f64,
    /// D_m;int;eff of this part, kJ/(m²·K).
    pub heat_capacity_kj_per_m2k: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VentilationShare {
    pub op: VentSysOpInput,
    pub area_m2: f64,
}

/// Ventilation principle of §6.4 note 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VentSysOpInput {
    Natural,
    Supply,
    Extract,
    Balanced,
    Decentral,
}

impl From<VentSysOp> for VentSysOpInput {
    fn from(op: VentSysOp) -> Self {
        match op {
            VentSysOp::Natural => Self::Natural,
            VentSysOp::Supply => Self::Supply,
            VentSysOp::Extract => Self::Extract,
            VentSysOp::Balanced => Self::Balanced,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalculationZoneLayout {
    pub id: String,
    pub parts: Vec<ZonePart>,
    pub heating_system_ids: Vec<String>,
    #[serde(default)]
    pub cooling_system_ids: Vec<String>,
    #[serde(default)]
    pub humidification_system_ids: Vec<String>,
    pub ventilation: Vec<VentilationShare>,
    /// Residence areas in open connection (§6.5.2 b exemption).
    #[serde(default)]
    pub residence_areas_open: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ZoningIssue {
    pub code: &'static str,
    pub zone_id: String,
}

fn largest_share<T: PartialEq + Copy>(values: impl Iterator<Item = (T, f64)>) -> f64 {
    let mut groups: Vec<(T, f64)> = Vec::new();
    let mut total = 0.0;
    for (key, area) in values {
        total += area;
        match groups.iter_mut().find(|(existing, _)| *existing == key) {
            Some(group) => group.1 += area,
            None => groups.push((key, area)),
        }
    }
    if total <= 0.0 {
        return 1.0;
    }
    groups.iter().map(|(_, area)| *area).fold(0.0, f64::max) / total
}

fn ventilation_function(function: UsageFunction) -> VentilationFunction {
    match function {
        UsageFunction::Residential => VentilationFunction::Residential,
        UsageFunction::AssemblyChildCare => VentilationFunction::AssemblyChildCare,
        UsageFunction::OtherAssembly => VentilationFunction::OtherAssembly,
        UsageFunction::Cell => VentilationFunction::Cell,
        UsageFunction::HealthcareWithBeds => VentilationFunction::HealthcareBedArea,
        UsageFunction::OtherHealthcare => VentilationFunction::OtherHealthcare,
        UsageFunction::Office => VentilationFunction::Office,
        UsageFunction::Lodging => VentilationFunction::LodgingBuilding,
        UsageFunction::Education => VentilationFunction::Education,
        UsageFunction::Sport => VentilationFunction::Sport,
        UsageFunction::Retail => VentilationFunction::Retail,
    }
}

/// §6.4 and §6.5.2 for one calculation zone (one zone is also one
/// conditioning zone in this kernel).
pub fn check_zone(zone: &CalculationZoneLayout) -> Vec<ZoningIssue> {
    let mut issues = Vec::new();
    let mut push = |code| {
        issues.push(ZoningIssue {
            code,
            zone_id: zone.id.clone(),
        })
    };
    // §6.4 a–c.
    if zone.heating_system_ids.len() > 1 {
        push("zone_multiple_heating_systems");
    }
    if zone.cooling_system_ids.len() > 1 {
        push("zone_multiple_cooling_systems");
    }
    if zone.humidification_system_ids.len() > 1 {
        push("zone_multiple_humidification_systems");
    }
    // §6.4 d: ≥ 80 % by one ventilation principle.
    if largest_share(
        zone.ventilation
            .iter()
            .map(|share| (share.op, share.area_m2)),
    ) < 0.8
    {
        push("zone_ventilation_not_80_percent_uniform");
    }
    // §6.5.2 opening sentence.
    let residential = zone
        .parts
        .iter()
        .any(|part| part.function == UsageFunction::Residential);
    if residential
        && zone
            .parts
            .iter()
            .any(|part| part.function != UsageFunction::Residential)
    {
        push("zone_residential_mixed_with_other_function");
    }
    // §6.5.2 a.
    let setpoints: Vec<f64> = zone
        .parts
        .iter()
        .map(|part| part.function.heating_setpoint_c())
        .collect();
    let spread = setpoints.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - setpoints.iter().cloned().fold(f64::INFINITY, f64::min);
    let largest_function =
        largest_share(zone.parts.iter().map(|part| (part.function, part.area_m2)));
    if spread > 4.0 && largest_function < 0.9 {
        push("zone_setpoint_spread_exceeds_4_k");
    }
    // §6.5.2 b, only for A, B, C or E.
    let dominant_balanced = zone
        .ventilation
        .iter()
        .all(|share| share.op == VentSysOpInput::Balanced);
    if !dominant_balanced && !zone.residence_areas_open {
        let capacities: Vec<(u64, f64)> = zone
            .parts
            .iter()
            .map(|part| {
                let capacity =
                    crate::ventilation::specific_capacity(ventilation_function(part.function));
                ((capacity * 1000.0).round() as u64, part.area_m2)
            })
            .collect();
        let max = capacities.iter().map(|(c, _)| *c).max().unwrap_or(0);
        let min = capacities.iter().map(|(c, _)| *c).min().unwrap_or(0);
        if min > 0 && max > 4 * min && largest_share(capacities.into_iter()) <= 0.8 {
            push("zone_ventilation_capacity_spread_exceeds_4");
        }
    }
    // §6.5.2 c.
    let capacities: Vec<(u64, f64)> = zone
        .parts
        .iter()
        .map(|part| {
            (
                (part.heat_capacity_kj_per_m2k * 1000.0).round() as u64,
                part.area_m2,
            )
        })
        .collect();
    let max = capacities.iter().map(|(c, _)| *c).max().unwrap_or(0);
    let min = capacities.iter().map(|(c, _)| *c).min().unwrap_or(0);
    if min > 0 && max > 3 * min && largest_share(capacities.into_iter()) <= 0.8 {
        push("zone_heat_capacity_spread_exceeds_3");
    }
    issues
}

/// §6.4: zones that may be merged by the 10 % rule, as (small, large) ids.
pub fn ten_percent_merges(zones: &[(String, f64)]) -> Vec<(String, String)> {
    let mut merges = Vec::new();
    for (id, area) in zones {
        if let Some((large, _)) = zones
            .iter()
            .filter(|(other, other_area)| other != id && *area < 0.1 * *other_area)
            .max_by(|a, b| a.1.total_cmp(&b.1))
        {
            merges.push((id.clone(), large.clone()));
        }
    }
    merges
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(function: UsageFunction, area: f64, capacity: f64) -> ZonePart {
        ZonePart {
            function,
            area_m2: area,
            heat_capacity_kj_per_m2k: capacity,
        }
    }

    fn zone(parts: Vec<ZonePart>, op: VentSysOpInput) -> CalculationZoneLayout {
        let area = parts.iter().map(|p| p.area_m2).sum();
        CalculationZoneLayout {
            id: "z".into(),
            parts,
            heating_system_ids: vec!["h1".into()],
            cooling_system_ids: Vec::new(),
            humidification_system_ids: Vec::new(),
            ventilation: vec![VentilationShare { op, area_m2: area }],
            residence_areas_open: false,
        }
    }

    fn codes(zone: &CalculationZoneLayout) -> Vec<&'static str> {
        check_zone(zone)
            .into_iter()
            .map(|issue| issue.code)
            .collect()
    }

    #[test]
    fn residential_zone_cannot_mix_functions() {
        let mixed = zone(
            vec![
                part(UsageFunction::Residential, 100.0, 180.0),
                part(UsageFunction::Office, 20.0, 180.0),
            ],
            VentSysOpInput::Extract,
        );
        assert!(codes(&mixed).contains(&"zone_residential_mixed_with_other_function"));
    }

    #[test]
    fn setpoint_capacity_and_mass_rules_follow_6_5_2() {
        // Sport 16 °C next to healthcare 22 °C: 6 K, largest 60 % → split.
        let spread = zone(
            vec![
                part(UsageFunction::Sport, 60.0, 180.0),
                part(UsageFunction::HealthcareWithBeds, 40.0, 180.0),
            ],
            VentSysOpInput::Extract,
        );
        assert!(codes(&spread).contains(&"zone_setpoint_spread_exceeds_4_k"));
        // Education 3,64 vs retail 0,28 dm³/(s·m²): factor 13 → split.
        let capacity = zone(
            vec![
                part(UsageFunction::Education, 50.0, 180.0),
                part(UsageFunction::Retail, 50.0, 180.0),
            ],
            VentSysOpInput::Extract,
        );
        assert!(codes(&capacity).contains(&"zone_ventilation_capacity_spread_exceeds_4"));
        // Not for balanced ventilation (note 3).
        let balanced = zone(
            vec![
                part(UsageFunction::Education, 50.0, 180.0),
                part(UsageFunction::Retail, 50.0, 180.0),
            ],
            VentSysOpInput::Balanced,
        );
        assert!(!codes(&balanced).contains(&"zone_ventilation_capacity_spread_exceeds_4"));
        // 80/360 kJ: factor 4,5 with 50/50 → split; 85/15 → allowed.
        let mass = zone(
            vec![
                part(UsageFunction::Office, 50.0, 80.0),
                part(UsageFunction::Office, 50.0, 360.0),
            ],
            VentSysOpInput::Balanced,
        );
        assert!(codes(&mass).contains(&"zone_heat_capacity_spread_exceeds_3"));
        let mostly = zone(
            vec![
                part(UsageFunction::Office, 85.0, 80.0),
                part(UsageFunction::Office, 15.0, 360.0),
            ],
            VentSysOpInput::Balanced,
        );
        assert!(codes(&mostly).is_empty());
    }

    #[test]
    fn conditioning_rules_and_ten_percent_merge() {
        let mut two = zone(
            vec![part(UsageFunction::Office, 100.0, 180.0)],
            VentSysOpInput::Balanced,
        );
        two.heating_system_ids.push("h2".into());
        two.ventilation = vec![
            VentilationShare {
                op: VentSysOpInput::Balanced,
                area_m2: 70.0,
            },
            VentilationShare {
                op: VentSysOpInput::Extract,
                area_m2: 30.0,
            },
        ];
        let found = codes(&two);
        assert!(found.contains(&"zone_multiple_heating_systems"));
        assert!(found.contains(&"zone_ventilation_not_80_percent_uniform"));
        let merges = ten_percent_merges(&[
            ("hall".into(), 8.0),
            ("offices".into(), 100.0),
            ("shop".into(), 50.0),
        ]);
        assert_eq!(merges, vec![("hall".to_string(), "offices".to_string())]);
    }
}
