//! Source regeneration for heat pumps, NTA 8800:2025+C1:2026 annex V
//! (pp. 1114–1116): `c_source` for individual ground heat exchangers from
//! the degree of regeneration `R` (V.1–V.6, tables V.1/V.2) and for
//! collective groundwater sources (table V.3).
//!
//! Values and formulas are transcribed from the licensed norm; the norm text
//! itself is not part of this repository.

use serde::{Deserialize, Serialize};

use crate::climate::{irradiance_at, MONTH_HOURS};
use crate::significant_figures::round_down;

/// Table V.2: (heat demand / solar supply, regeneration efficiency).
const SOLAR_REGENERATION: [(f64, f64); 8] = [
    (0.0, 0.58),
    (0.25, 0.56),
    (0.60, 0.51),
    (0.80, 0.48),
    (1.00, 0.46),
    (1.60, 0.43),
    (2.00, 0.41),
    (5.00, 0.35),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectiveSourceType {
    Recirculation,
    Doublet,
}

/// Table V.3.
pub fn collective_source_correction(kind: CollectiveSourceType) -> f64 {
    match kind {
        CollectiveSourceType::Recirculation => 1.00,
        CollectiveSourceType::Doublet => 1.04,
    }
}

/// Table V.1.
pub fn regeneration_correction(degree: f64) -> f64 {
    if degree >= 0.75 {
        1.04
    } else if degree >= 0.5 {
        1.02
    } else {
        1.00
    }
}

/// A thermal solar collector field used for regeneration (V.4/V.5).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SolarRegeneration {
    /// `A_col;i` per NEN-EN 12975-2, m².
    pub collector_area_m2: f64,
    /// 0 = north, clockwise; table V.2 is valid from south-east to
    /// south-west.
    pub azimuth_deg: f64,
    pub tilt_deg: f64,
    /// `F_sh;obst` for May–September, one value or five.
    #[serde(default = "unobstructed")]
    pub obstruction_factors: Vec<f64>,
    /// A declared annual efficiency instead of table V.2 (rounded down per
    /// annex X).
    #[serde(default)]
    pub declared_efficiency: Option<f64>,
    pub source_reference: String,
}

fn unobstructed() -> Vec<f64> {
    vec![1.0]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegenerationInput {
    /// Free cooling from the same source (V.2); `Q_C;nd;fc` is then the
    /// annual cooling need.
    #[serde(default)]
    pub free_cooling_from_source: bool,
    #[serde(default)]
    pub solar: Vec<SolarRegeneration>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegenerationIssue {
    pub code: &'static str,
    pub path: String,
}

/// Annual quantities that depend on the rest of the calculation.
#[derive(Debug, Clone, Copy)]
pub struct RegenerationContext {
    /// Σ `Q_H;dis;nren` delivered by heat pumps on the same source, kWh.
    pub heating_kwh: f64,
    /// `η_H;gen` with `c_source = 1`.
    pub heating_efficiency: f64,
    /// Σ `Q_W;dis;nren` of hot water from heat pumps on the same source, kWh.
    pub hot_water_kwh: f64,
    pub hot_water_efficiency: f64,
    /// Σ `Q_C;nd` of the zones, kWh.
    pub annual_cooling_need_kwh: f64,
    /// `η_el = 1/f_P;del;el`.
    pub electricity_efficiency: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegenerationResult {
    /// Heat taken from the source: the denominator of V.1, kWh.
    pub source_extraction_kwh: f64,
    pub free_cooling_kwh: f64,
    pub solar_kwh: f64,
    /// `R` (V.1).
    pub degree: f64,
    /// Table V.1.
    pub correction: f64,
}

pub fn validate_regeneration(input: &RegenerationInput, path: &str) -> Vec<RegenerationIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: String| {
        issues.push(RegenerationIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    if input.source_reference.trim().is_empty() {
        push("source_reference_required", "sourceReference".into());
    }
    for (index, solar) in input.solar.iter().enumerate() {
        let base = format!("solar[{index}]");
        if !(solar.collector_area_m2.is_finite() && solar.collector_area_m2 > 0.0) {
            push(
                "regeneration_collector_area_invalid",
                format!("{base}.collectorAreaM2"),
            );
        }
        if !(0.0..=90.0).contains(&solar.tilt_deg) || !solar.azimuth_deg.is_finite() {
            push(
                "regeneration_collector_orientation_invalid",
                format!("{base}.tiltDeg"),
            );
        }
        // Table V.2 note a: south-east to south-west.
        let azimuth = solar.azimuth_deg.rem_euclid(360.0);
        if !(135.0..=225.0).contains(&azimuth) && solar.declared_efficiency.is_none() {
            push(
                "regeneration_collector_outside_table_v2",
                format!("{base}.azimuthDeg"),
            );
        }
        if !(solar.obstruction_factors.len() == 1 || solar.obstruction_factors.len() == 5)
            || solar
                .obstruction_factors
                .iter()
                .any(|value| !(0.0..=1.0).contains(value))
        {
            push(
                "regeneration_obstruction_invalid",
                format!("{base}.obstructionFactors"),
            );
        }
        if solar
            .declared_efficiency
            .is_some_and(|value| !(value > 0.0 && value <= 1.0))
        {
            push(
                "regeneration_efficiency_invalid",
                format!("{base}.declaredEfficiency"),
            );
        }
        if solar.source_reference.trim().is_empty() {
            push(
                "source_reference_required",
                format!("{base}.sourceReference"),
            );
        }
    }
    issues
}

fn interpolate(points: &[(f64, f64)], x: f64) -> f64 {
    if x <= points[0].0 {
        return points[0].1;
    }
    for pair in points.windows(2) {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        if x <= x1 {
            return y0 + (x - x0) / (x1 - x0) * (y1 - y0);
        }
    }
    points[points.len() - 1].1
}

/// V.1–V.6; call after [`validate_regeneration`].
pub fn calculate_regeneration(
    input: &RegenerationInput,
    context: RegenerationContext,
) -> RegenerationResult {
    let share = |kwh: f64, efficiency: f64| {
        if efficiency > 0.0 {
            kwh * (1.0 - context.electricity_efficiency / efficiency)
        } else {
            0.0
        }
    };
    let extraction = (share(context.heating_kwh, context.heating_efficiency)
        + share(context.hot_water_kwh, context.hot_water_efficiency))
    .max(0.0);
    let free_cooling = if input.free_cooling_from_source {
        context.annual_cooling_need_kwh
    } else {
        0.0
    };
    let mut solar = 0.0;
    for field in &input.solar {
        // V.5: May–September.
        let received: f64 = (4..9)
            .map(|month_index| {
                let obstruction = if field.obstruction_factors.len() == 5 {
                    field.obstruction_factors[month_index - 4]
                } else {
                    field.obstruction_factors[0]
                };
                let irradiance =
                    irradiance_at(field.azimuth_deg, field.tilt_deg, month_index as u8 + 1)
                        .unwrap_or(0.0);
                obstruction * field.collector_area_m2 * irradiance * MONTH_HOURS[month_index]
                    / 1000.0
            })
            .sum();
        // V.6 and table V.2.
        let efficiency = match field.declared_efficiency {
            Some(value) => round_down(value),
            None => {
                let ratio = if received > 0.0 {
                    extraction / received
                } else {
                    f64::INFINITY
                };
                interpolate(&SOLAR_REGENERATION, ratio)
            }
        };
        solar += efficiency * received;
    }
    let degree = if extraction > 0.0 {
        (free_cooling + solar) / extraction
    } else {
        0.0
    };
    RegenerationResult {
        source_extraction_kwh: extraction,
        free_cooling_kwh: free_cooling,
        solar_kwh: solar,
        degree,
        correction: regeneration_correction(degree),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> RegenerationContext {
        RegenerationContext {
            heating_kwh: 8000.0,
            heating_efficiency: 4.0,
            hot_water_kwh: 2000.0,
            hot_water_efficiency: 2.5,
            annual_cooling_need_kwh: 1000.0,
            electricity_efficiency: 1.0 / 1.45,
        }
    }

    #[test]
    fn table_v1_and_v3() {
        assert_eq!(regeneration_correction(0.49), 1.00);
        assert_eq!(regeneration_correction(0.5), 1.02);
        assert_eq!(regeneration_correction(0.75), 1.04);
        assert_eq!(
            collective_source_correction(CollectiveSourceType::Doublet),
            1.04
        );
    }

    #[test]
    fn degree_follows_v1_with_free_cooling() {
        let input = RegenerationInput {
            free_cooling_from_source: true,
            solar: Vec::new(),
            source_reference: "design".into(),
        };
        let result = calculate_regeneration(&input, context());
        let el = 1.0 / 1.45;
        let extraction = 8000.0 * (1.0 - el / 4.0) + 2000.0 * (1.0 - el / 2.5);
        assert!((result.source_extraction_kwh - extraction).abs() < 1e-9);
        assert!((result.degree - 1000.0 / extraction).abs() < 1e-12);
        assert_eq!(result.correction, 1.00);
    }

    #[test]
    fn solar_regeneration_follows_v4_to_v6() {
        let input = RegenerationInput {
            free_cooling_from_source: false,
            solar: vec![SolarRegeneration {
                collector_area_m2: 10.0,
                azimuth_deg: 180.0,
                tilt_deg: 45.0,
                obstruction_factors: vec![1.0],
                declared_efficiency: None,
                source_reference: "datasheet".into(),
            }],
            source_reference: "design".into(),
        };
        assert!(validate_regeneration(&input, "r").is_empty());
        let result = calculate_regeneration(&input, context());
        let received: f64 = (4..9)
            .map(|index| {
                10.0 * irradiance_at(180.0, 45.0, index as u8 + 1).unwrap() * MONTH_HOURS[index]
                    / 1000.0
            })
            .sum();
        let ratio = result.source_extraction_kwh / received;
        let efficiency = interpolate(&SOLAR_REGENERATION, ratio);
        assert!((result.solar_kwh - efficiency * received).abs() < 1e-9);
        assert_eq!(interpolate(&SOLAR_REGENERATION, 0.7), 0.495);
        assert_eq!(interpolate(&SOLAR_REGENERATION, 9.0), 0.35);
    }
}
