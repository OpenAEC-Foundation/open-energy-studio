//! Space-heating distribution, NTA 8800:2025+C1:2026 §9.4 (pages 302–321),
//! the node loss of a collective buffer vessel (9.2.3, pages 290–292) and
//! the recoverable distribution losses of 9.2.5/9.4.3 (pages 293–294, 313).
//!
//! Pure formula helpers; the chain in `space_heating_chain` wires them per
//! zone and per month. All results are unverified.

use crate::climate::{MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use serde::{Deserialize, Serialize};

/// 9.31/9.32: design outdoor temperature of table 9.14.
pub const DESIGN_OUTDOOR_C: f64 = -10.0;
/// 9.26: ambient temperature of an unheated space without an entered AOR.
pub const DEFAULT_UNHEATED_AMBIENT_C: f64 = 13.0;
/// 9.26: unknown share of the pipe length in unheated spaces.
pub const UNKNOWN_UNHEATED_SHARE: f64 = 0.15;
/// 9.33–9.35: surface coefficient `h_a` unless another value is shown.
pub const DEFAULT_SURFACE_COEFFICIENT: f64 = 8.0;
/// 9.44: `f_comp` and table 9.20 `R_H;max` in kPa/m.
pub const COMPONENT_RESISTANCE_RATIO: f64 = 0.4;
pub const PRESSURE_LOSS_PER_M_KPA: f64 = 0.10;
/// Table 9.18.
pub const RECOVERABLE_AUX_FRACTION: f64 = 0.25;
/// 9.45: water properties.
const WATER_HEAT_CAPACITY_KJ_PER_KGK: f64 = 4.2;
const WATER_DENSITY_KG_PER_M3: f64 = 1000.0;
/// 9.2.3 with 13.58: label loss divided by 45 K.
const BUFFER_LABEL_TEMPERATURE_DIFFERENCE_K: f64 = 45.0;
/// Collective delivery-set systems: minimum mean medium temperature (9.30).
pub const DELIVERY_SET_MIN_MEAN_C: f64 = 65.0;

/// Table 9.14 design temperature classes (supply/return).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DesignTemperatureClass {
    #[serde(rename = "30_27")]
    C30,
    #[serde(rename = "35_30")]
    C35,
    #[serde(rename = "40_35")]
    C40,
    #[serde(rename = "45_40")]
    C45,
    #[serde(rename = "50_42")]
    C50,
    #[serde(rename = "55_47")]
    C55,
    #[serde(rename = "60_50")]
    C60,
    #[serde(rename = "65_55")]
    C65,
    #[serde(rename = "70_60")]
    C70,
    #[serde(rename = "75_65")]
    C75,
    #[serde(rename = "80_60")]
    C80,
    #[serde(rename = "90_70")]
    C90,
}

impl DesignTemperatureClass {
    /// Table 9.14 has 60/50 and 70/60 from NTA 8800:2023 (p. 294); 2022
    /// p. 290 lacks them.
    pub fn in_edition(self) -> bool {
        crate::norm_versions::profile().design_classes_60_and_70
            || !matches!(self, Self::C60 | Self::C70)
    }

    /// Table 9.14: (`ϑ_H,a;ontw`, `Δϑ_H,ontw`).
    pub fn design(self) -> (f64, f64) {
        match self {
            Self::C30 => (30.0, 3.0),
            Self::C35 => (35.0, 5.0),
            Self::C40 => (40.0, 5.0),
            Self::C45 => (45.0, 5.0),
            Self::C50 => (50.0, 8.0),
            Self::C55 => (55.0, 8.0),
            Self::C60 => (60.0, 10.0),
            Self::C65 => (65.0, 10.0),
            Self::C70 => (70.0, 10.0),
            Self::C75 => (75.0, 10.0),
            Self::C80 => (80.0, 20.0),
            Self::C90 => (90.0, 20.0),
        }
    }
}

/// Table 9.15, rows for heating limits 16 down to 8 °C, hours per month.
const HEATING_LIMIT_HOURS: [[f64; 12]; 9] = [
    [
        744., 672., 733., 666., 530., 414., 295., 230., 444., 670., 720., 744.,
    ],
    [
        744., 661., 725., 654., 491., 326., 219., 172., 351., 618., 720., 744.,
    ],
    [
        744., 652., 716., 637., 443., 245., 141., 119., 258., 588., 719., 744.,
    ],
    [
        744., 637., 709., 618., 377., 174., 78., 86., 185., 566., 701., 744.,
    ],
    [
        744., 629., 678., 573., 293., 107., 39., 64., 131., 524., 663., 743.,
    ],
    [
        743., 600., 639., 514., 225., 66., 18., 37., 84., 464., 613., 724.,
    ],
    [
        736., 557., 605., 448., 159., 42., 11., 20., 71., 402., 550., 711.,
    ],
    [
        716., 533., 570., 386., 76., 35., 6., 10., 54., 319., 461., 700.,
    ],
    [
        697., 518., 532., 316., 46., 23., 1., 5., 30., 254., 384., 662.,
    ],
];
pub const HEATING_LIMIT_MAX_C: i32 = 16;
pub const HEATING_LIMIT_MIN_C: i32 = 8;

/// Table 9.15: hours below the heating limit in month `index` (0-based).
pub fn heating_limit_hours(limit_c: i32, index: usize) -> f64 {
    let limit = limit_c.clamp(HEATING_LIMIT_MIN_C, HEATING_LIMIT_MAX_C);
    HEATING_LIMIT_HOURS[(HEATING_LIMIT_MAX_C - limit) as usize][index]
}

/// Use functions of table 7.15 for the heating reduction hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReductionFunction {
    Residential,
    Assembly,
    Cell,
    HealthcareWithBeds,
    HealthcareOther,
    Office,
    Lodging,
    Education,
    Sport,
    Retail,
}

impl ReductionFunction {
    /// Table 7.15 (page 220): (`t_H;red;day`, `t_H;red;wknd`) in h.
    pub fn reduction_hours(self) -> (f64, f64) {
        match self {
            Self::Assembly
            | Self::HealthcareOther
            | Self::Office
            | Self::Education
            | Self::Sport => (14.0, 48.0),
            Self::Cell | Self::HealthcareWithBeds => (8.0, 0.0),
            Self::Lodging | Self::Retail => (13.0, 24.0),
            Self::Residential => (10.0, 0.0),
        }
    }

    /// 9.32b with 7.62/7.63: `f_H;red = 1 − f_H;red;day − f_H;red;wknd`.
    pub fn heating_reduction_factor(self) -> f64 {
        let (day, weekend) = self.reduction_hours();
        let f_day = day * (7.0 - weekend / 24.0) / (24.0 * 7.0);
        let f_weekend = weekend / 168.0;
        1.0 - f_day - f_weekend
    }
}

/// 9.28 steps 2–5: least-squares heating limit from the monthly need for the
/// heating limit. Returns `None` when no decreasing line can be fitted.
pub fn heating_limit_c(need_kwh: &[f64; 12], setpoint_c: f64) -> Option<i32> {
    let max = need_kwh.iter().copied().fold(0.0_f64, f64::max);
    if max <= 0.0 {
        // No heating need: the lowest row of table 9.15 applies.
        return Some(HEATING_LIMIT_MIN_C);
    }
    let points: Vec<(f64, f64)> = need_kwh
        .iter()
        .zip(OUTDOOR_TEMPERATURE_C)
        .filter(|(need, _)| **need >= 0.1 * max)
        .map(|(need, outdoor)| (outdoor, *need))
        .collect();
    if points.len() < 2 {
        return None;
    }
    let n = points.len() as f64;
    let mean_x = points.iter().map(|point| point.0).sum::<f64>() / n;
    let mean_y = points.iter().map(|point| point.1).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|point| (point.0 - mean_x).powi(2)).sum();
    let sxy: f64 = points
        .iter()
        .map(|point| (point.0 - mean_x) * (point.1 - mean_y))
        .sum();
    if sxx <= 0.0 {
        return None;
    }
    let slope = sxy / sxx;
    if slope >= 0.0 {
        return None;
    }
    let intercept_x = mean_x - mean_y / slope;
    // Step 5: rounded, at most the setpoint, within the rows of table 9.15.
    let limit = intercept_x.round().min(setpoint_c.floor()) as i32;
    Some(limit.clamp(HEATING_LIMIT_MIN_C, HEATING_LIMIT_MAX_C))
}

/// 9.31/9.32: (`ϑ_H,in`, `ϑ_H,out`) for one month.
pub fn supply_return_c(
    setpoint_c: f64,
    class: DesignTemperatureClass,
    heating_limit_c: i32,
    outdoor_c: f64,
    increment_k: f64,
) -> (f64, f64) {
    let (design_supply, design_spread) = class.design();
    let limit = f64::from(heating_limit_c);
    let span = DESIGN_OUTDOOR_C - limit;
    let supply = setpoint_c
        .max(setpoint_c + (design_supply - setpoint_c) / span * (outdoor_c - limit) + increment_k);
    let ret = setpoint_c.max(
        (setpoint_c
            + ((design_supply - design_spread) - setpoint_c) / span * (outdoor_c - limit)
            + increment_k)
            .min(supply),
    );
    (supply, ret)
}

/// 9.36: forfait pipe length.
pub fn forfait_pipe_length_m(area_m2: f64) -> f64 {
    0.64 * area_m2
}

/// 9.37: forfait maximum pipe length, `n ≥ 1`.
pub fn forfait_max_pipe_length_m(storeys: u32, area_m2: f64) -> f64 {
    let n = f64::from(storeys.max(1));
    35.0 + 6.0 * n + 0.13 * area_m2 / n
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InsulationPeriod {
    From1995,
    From1980To1995,
    Before1980OrUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum PipeInsulation {
    Insulated {
        period: InsulationPeriod,
    },
    Uninsulated,
    /// The highest value of table 9.16 applies.
    Unknown,
}

/// Linear thermal transmittance `Ψ` of the distribution pipes.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum PipeTransmittance {
    /// Table 9.16.
    Forfait { insulation: PipeInsulation },
    /// 9.33: insulated pipe in air.
    InsulatedInAir {
        #[serde(rename = "pipeOuterDiameterM")]
        pipe_outer_diameter_m: f64,
        #[serde(rename = "insulatedDiameterM")]
        insulated_diameter_m: f64,
        #[serde(rename = "insulationLambdaWPerMK")]
        insulation_lambda: f64,
        #[serde(default, rename = "surfaceCoefficientWPerM2K")]
        surface_coefficient: Option<f64>,
    },
    /// 9.34: insulated pipe embedded in the construction.
    InsulatedEmbedded {
        #[serde(rename = "pipeOuterDiameterM")]
        pipe_outer_diameter_m: f64,
        #[serde(rename = "insulatedDiameterM")]
        insulated_diameter_m: f64,
        #[serde(rename = "insulationLambdaWPerMK")]
        insulation_lambda: f64,
        #[serde(rename = "embeddingLambdaWPerMK")]
        embedding_lambda: f64,
        #[serde(rename = "depthM")]
        depth_m: f64,
    },
    /// 9.35: uninsulated pipe.
    Uninsulated {
        #[serde(rename = "innerDiameterM")]
        inner_diameter_m: f64,
        #[serde(rename = "outerDiameterM")]
        outer_diameter_m: f64,
        #[serde(rename = "pipeLambdaWPerMK")]
        pipe_lambda: f64,
        #[serde(default, rename = "surfaceCoefficientWPerM2K")]
        surface_coefficient: Option<f64>,
    },
}

impl PipeTransmittance {
    pub fn is_forfait(&self) -> bool {
        matches!(self, Self::Forfait { .. })
    }

    /// `Ψ` in W/(m·K); `connected_area_m2` is the usable area of the whole
    /// building on the system, `shared_with_hot_water` selects the collective
    /// heating-and-hot-water rows of table 9.16. `None` for invalid geometry.
    pub fn value(&self, connected_area_m2: f64, shared_with_hot_water: bool) -> Option<f64> {
        match *self {
            Self::Forfait { insulation } => {
                let uninsulated = if shared_with_hot_water {
                    // Table 9.16: 1,0 up to 500 m² in 2025+C1 (p. 310),
                    // 2,0 in 2024 (p. 292).
                    if connected_area_m2 <= 500.0 {
                        crate::norm_versions::profile().psi_collective_combined_small
                    } else {
                        2.0
                    }
                } else if connected_area_m2 <= 200.0 {
                    1.0
                } else if connected_area_m2 <= 500.0 {
                    2.0
                } else {
                    3.0
                };
                Some(match insulation {
                    PipeInsulation::Insulated { .. } if shared_with_hot_water => 0.4,
                    PipeInsulation::Insulated {
                        period: InsulationPeriod::From1995,
                    } => 0.3,
                    PipeInsulation::Insulated { .. } => 0.4,
                    PipeInsulation::Uninsulated | PipeInsulation::Unknown => uninsulated,
                })
            }
            Self::InsulatedInAir {
                pipe_outer_diameter_m,
                insulated_diameter_m,
                insulation_lambda,
                surface_coefficient,
            } => PipeGeometry::InsulatedInAir {
                pipe_outer_diameter_m,
                insulated_diameter_m,
                insulation_lambda,
                surface_coefficient,
            }
            .psi(),
            Self::InsulatedEmbedded {
                pipe_outer_diameter_m,
                insulated_diameter_m,
                insulation_lambda,
                embedding_lambda,
                depth_m,
            } => PipeGeometry::InsulatedEmbedded {
                pipe_outer_diameter_m,
                insulated_diameter_m,
                insulation_lambda,
                embedding_lambda,
                depth_m,
            }
            .psi(),
            Self::Uninsulated {
                inner_diameter_m,
                outer_diameter_m,
                pipe_lambda,
                surface_coefficient,
            } => PipeGeometry::Uninsulated {
                inner_diameter_m,
                outer_diameter_m,
                pipe_lambda,
                surface_coefficient,
            }
            .psi(),
        }
    }
}

/// Pipe geometry for a calculated `Ψ`: 9.33–9.35 (heating, 2025+C1
/// p. 311–312), 10.24–10.26 (cooling, p. 384–385) and 13.27–13.29 (hot
/// water, p. 551) are the same three formulas.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum PipeGeometry {
    /// Insulated pipe surrounded by air (9.33/10.24/13.27).
    InsulatedInAir {
        #[serde(rename = "pipeOuterDiameterM")]
        pipe_outer_diameter_m: f64,
        #[serde(rename = "insulatedDiameterM")]
        insulated_diameter_m: f64,
        #[serde(rename = "insulationLambdaWPerMK")]
        insulation_lambda: f64,
        #[serde(default, rename = "surfaceCoefficientWPerM2K")]
        surface_coefficient: Option<f64>,
    },
    /// Insulated pipe embedded in the construction (9.34/10.25/13.28).
    InsulatedEmbedded {
        #[serde(rename = "pipeOuterDiameterM")]
        pipe_outer_diameter_m: f64,
        #[serde(rename = "insulatedDiameterM")]
        insulated_diameter_m: f64,
        #[serde(rename = "insulationLambdaWPerMK")]
        insulation_lambda: f64,
        #[serde(rename = "embeddingLambdaWPerMK")]
        embedding_lambda: f64,
        #[serde(rename = "depthM")]
        depth_m: f64,
    },
    /// Uninsulated pipe (9.35/10.26/13.29).
    Uninsulated {
        #[serde(rename = "innerDiameterM")]
        inner_diameter_m: f64,
        #[serde(rename = "outerDiameterM")]
        outer_diameter_m: f64,
        #[serde(rename = "pipeLambdaWPerMK")]
        pipe_lambda: f64,
        #[serde(default, rename = "surfaceCoefficientWPerM2K")]
        surface_coefficient: Option<f64>,
    },
}

impl PipeGeometry {
    /// `Ψ` in W/(m·K); `None` for invalid geometry (non-positive values,
    /// an outer diameter below the inner one, or for 9.34 a depth with
    /// `4·z ≤ d_a`, where the logarithm would not be positive).
    pub fn psi(&self) -> Option<f64> {
        let positive =
            |values: &[f64]| values.iter().all(|value| value.is_finite() && *value > 0.0);
        match *self {
            Self::InsulatedInAir {
                pipe_outer_diameter_m: di,
                insulated_diameter_m: da,
                insulation_lambda: lambda,
                surface_coefficient,
            } => {
                let h = surface_coefficient.unwrap_or(DEFAULT_SURFACE_COEFFICIENT);
                (positive(&[di, da, lambda, h]) && da >= di).then(|| {
                    std::f64::consts::PI / ((da / di).ln() / (2.0 * lambda) + 1.0 / (h * da))
                })
            }
            Self::InsulatedEmbedded {
                pipe_outer_diameter_m: di,
                insulated_diameter_m: da,
                insulation_lambda: lambda,
                embedding_lambda,
                depth_m,
            } => (positive(&[di, da, lambda, embedding_lambda, depth_m])
                && da >= di
                && 4.0 * depth_m > da)
                .then(|| {
                    std::f64::consts::PI
                        / (0.5
                            * ((da / di).ln() / lambda
                                + (4.0 * depth_m / da).ln() / embedding_lambda))
                }),
            Self::Uninsulated {
                inner_diameter_m: di,
                outer_diameter_m: da,
                pipe_lambda,
                surface_coefficient,
            } => {
                let h = surface_coefficient.unwrap_or(DEFAULT_SURFACE_COEFFICIENT);
                (positive(&[di, da, pipe_lambda, h]) && da >= di).then(|| {
                    std::f64::consts::PI / ((da / di).ln() / (2.0 * pipe_lambda) + 1.0 / (h * da))
                })
            }
        }
    }
}

/// 9.27a/9.27b: equivalent length factor for valves and brackets.
pub fn equivalent_length_m(length_m: f64, psi: f64, valves_insulated: bool) -> f64 {
    let factor = if valves_insulated { 0.03 } else { 0.15 };
    factor / psi * length_m
}

/// Table 9.21 emitter resistance in kPa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmitterResistance {
    RadiatorConvectorOrDarkRadiator,
    SurfaceAirOrOther,
}

impl EmitterResistance {
    pub fn kpa(self) -> f64 {
        match self {
            Self::RadiatorConvectorOrDarkRadiator => 2.0,
            Self::SurfaceAirOrOther => 4.5,
        }
    }
}

/// Table 9.21 generator resistance in kPa for the emitter design spread
/// `Δϑ_h;a;ontw` and the generator full-load spread `Δϑ_h;g;ontw`.
///
/// The table has no row for a generator spread of at most 10 K with an
/// emitter spread above 10 K; the first row is used for it (documented
/// interpretation).
pub fn generator_resistance_kpa(emitter_spread_k: f64, generator_spread_k: f64) -> f64 {
    let inverse_square = (1.0 / emitter_spread_k).powi(2);
    if generator_spread_k <= 10.0 {
        1000.0 * inverse_square
    } else if emitter_spread_k <= 10.0 {
        emitter_spread_k / generator_spread_k * 4000.0 * inverse_square
    } else {
        4000.0 * inverse_square
    }
}

/// 9.44: design pressure difference in kPa.
pub fn design_pressure_kpa(max_length_m: f64, additional_kpa: f64) -> f64 {
    (1.0 + COMPONENT_RESISTANCE_RATIO) * PRESSURE_LOSS_PER_M_KPA * max_length_m + additional_kpa
}

/// 9.45: design volume flow in m³/h.
pub fn design_flow_m3_per_h(
    max_need_kwh: f64,
    operating_hours: f64,
    spread_k: f64,
    building_fraction: f64,
) -> f64 {
    max_need_kwh
        / (operating_hours
            * WATER_HEAT_CAPACITY_KJ_PER_KGK
            * spread_k
            * WATER_DENSITY_KG_PER_M3
            * building_fraction)
        * 3600.0
}

/// 9.43: hydraulic pump power in kW.
pub fn hydraulic_power_kw(pressure_kpa: f64, flow_m3_per_h: f64) -> f64 {
    (pressure_kpa * flow_m3_per_h / 3600.0).max(0.01)
}

/// 9.46–9.50 with table 9.22 and `β = 1`: pump energy factor `ε_H;dis`.
pub fn pump_energy_factor(
    hydraulic_kw: f64,
    eei: Option<f64>,
    electric_power_kw: Option<f64>,
) -> f64 {
    let eei = eei.unwrap_or(if hydraulic_kw < 2.5 { 0.23 } else { 0.25 });
    let efficiency_factor = match electric_power_kw {
        // 9.47
        Some(power) => power / hydraulic_kw,
        // 9.48 with 9.49
        None if hydraulic_kw < 2.5 => {
            // 10⁻³ applies to the exponential term only (rendered p. 320).
            let reference =
                1.7 * hydraulic_kw + 17.0 * (1.0 - (-0.3 * hydraulic_kw * 1000.0).exp()) * 1e-3;
            reference / hydraulic_kw
        }
        // 9.50 with b = 2
        None => (1.25 + (0.2 / hydraulic_kw).sqrt()) * 2.0,
    };
    efficiency_factor * (0.25 + 0.75) * eei / 0.25
}

/// Table 9.19: hydronic balancing correction.
pub fn balancing_factor(balanced: bool) -> f64 {
    if balanced {
        1.0
    } else {
        1.15
    }
}

/// Energy label classes of table 13.9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum StorageLabel {
    #[serde(rename = "a_plus")]
    APlus,
    #[serde(rename = "a")]
    A,
    #[serde(rename = "b")]
    B,
    #[serde(rename = "c")]
    C,
    #[serde(rename = "d")]
    D,
    #[serde(rename = "e")]
    E,
    #[serde(rename = "f")]
    F,
    #[serde(rename = "g")]
    G,
}

/// Table 13.9: maximum standing loss `S_sto;ls` in W for volume `V` in l.
pub fn label_standing_loss_w(label: StorageLabel, volume_l: f64) -> f64 {
    let (fixed, slope) = match label {
        StorageLabel::APlus => (5.5, 3.16),
        StorageLabel::A => (7.0, 3.7),
        StorageLabel::B => (10.25, 5.09),
        StorageLabel::C => (14.33, 7.13),
        StorageLabel::D => (18.83, 9.33),
        StorageLabel::E => (23.5, 11.99),
        StorageLabel::F => (28.5, 15.16),
        StorageLabel::G => (31.0, 16.66),
    };
    fixed + slope * volume_l.powf(0.4)
}

/// 9.2.3 with 13.58 (`f_sto;bac;acc = 1`, `f_sto;dis;ls = 1`): buffer loss
/// in kWh for one month.
pub fn buffer_loss_kwh(
    standing_loss_w: f64,
    set_c: f64,
    ambient_c: f64,
    operating_hours: f64,
    building_fraction: f64,
) -> f64 {
    let transmission = standing_loss_w / BUFFER_LABEL_TEMPERATURE_DIFFERENCE_K;
    building_fraction * operating_hours / 1000.0 * transmission * (set_c - ambient_c)
}

/// Hours of month `index`.
pub fn month_hours(index: usize) -> f64 {
    MONTH_HOURS[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_9_15_and_reduction_factor() {
        assert_eq!(heating_limit_hours(16, 6), 295.0);
        assert_eq!(heating_limit_hours(8, 0), 697.0);
        assert_eq!(heating_limit_hours(20, 6), 295.0);
        assert_eq!(heating_limit_hours(3, 0), 697.0);
        // Residential: 1 − 10·7/168.
        let residential = ReductionFunction::Residential.heating_reduction_factor();
        assert!((residential - (1.0 - 70.0 / 168.0)).abs() < 1e-12);
        // Office: 1 − 14·5/168 − 48/168.
        let office = ReductionFunction::Office.heating_reduction_factor();
        assert!((office - (1.0 - 70.0 / 168.0 - 48.0 / 168.0)).abs() < 1e-12);
    }

    #[test]
    fn heating_limit_fits_a_line_and_clamps() {
        // Need proportional to (14 − θe): intercept at 14 °C.
        let need: [f64; 12] =
            std::array::from_fn(|index| (100.0 * (14.0 - OUTDOOR_TEMPERATURE_C[index])).max(0.0));
        assert_eq!(heating_limit_c(&need, 20.0), Some(14));
        // Intercept above the table is clamped to 16; capped by the setpoint.
        let warm: [f64; 12] =
            std::array::from_fn(|index| 100.0 * (30.0 - OUTDOOR_TEMPERATURE_C[index]));
        assert_eq!(heating_limit_c(&warm, 20.0), Some(16));
        assert_eq!(heating_limit_c(&warm, 12.0), Some(12));
        assert_eq!(heating_limit_c(&[0.0; 12], 20.0), Some(8));
        assert_eq!(heating_limit_c(&[1.0; 12], 20.0), None);
    }

    #[test]
    fn supply_and_return_follow_9_31_and_9_32() {
        // 90/70, limit 15, θe = 2,61, Δθinc = 3,55, setpoint 20.
        let (supply, ret) = supply_return_c(20.0, DesignTemperatureClass::C90, 15, 2.61, 3.55);
        let expected_supply = 20.0 + 70.0 / -25.0 * (2.61 - 15.0) + 3.55;
        let expected_return = 20.0 + 50.0 / -25.0 * (2.61 - 15.0) + 3.55;
        assert!((supply - expected_supply).abs() < 1e-12);
        assert!((ret - expected_return).abs() < 1e-12);
        // Above the heating limit both fall back to the setpoint.
        let (supply, ret) = supply_return_c(20.0, DesignTemperatureClass::C35, 12, 18.0, 0.0);
        assert_eq!((supply, ret), (20.0, 20.0));
    }

    #[test]
    fn pipe_transmittance_table_and_formulas() {
        let forfait = |insulation| PipeTransmittance::Forfait { insulation };
        let insulated = forfait(PipeInsulation::Insulated {
            period: InsulationPeriod::From1995,
        });
        assert_eq!(insulated.value(100.0, false), Some(0.3));
        assert_eq!(insulated.value(100.0, true), Some(0.4));
        assert_eq!(
            forfait(PipeInsulation::Unknown).value(300.0, false),
            Some(2.0)
        );
        assert_eq!(
            forfait(PipeInsulation::Uninsulated).value(600.0, false),
            Some(3.0)
        );
        assert_eq!(
            forfait(PipeInsulation::Uninsulated).value(600.0, true),
            Some(2.0)
        );
        // 9.33: 22 mm pipe, 62 mm insulated, λ 0,035, h 8.
        let air = PipeTransmittance::InsulatedInAir {
            pipe_outer_diameter_m: 0.022,
            insulated_diameter_m: 0.062,
            insulation_lambda: 0.035,
            surface_coefficient: None,
        };
        let expected =
            std::f64::consts::PI / ((0.062_f64 / 0.022).ln() / 0.07 + 1.0 / (8.0 * 0.062));
        assert!((air.value(0.0, false).unwrap() - expected).abs() < 1e-12);
        // 9.34
        let embedded = PipeTransmittance::InsulatedEmbedded {
            pipe_outer_diameter_m: 0.016,
            insulated_diameter_m: 0.026,
            insulation_lambda: 0.04,
            embedding_lambda: 1.2,
            depth_m: 0.05,
        };
        let expected = std::f64::consts::PI
            / (0.5 * ((0.026_f64 / 0.016).ln() / 0.04 + (0.2_f64 / 0.026).ln() / 1.2));
        assert!((embedded.value(0.0, false).unwrap() - expected).abs() < 1e-12);
        assert_eq!(
            PipeTransmittance::Uninsulated {
                inner_diameter_m: 0.02,
                outer_diameter_m: 0.01,
                pipe_lambda: 380.0,
                surface_coefficient: None,
            }
            .value(0.0, false),
            None
        );
    }

    #[test]
    fn pump_formulas_9_37_to_9_50() {
        assert_eq!(forfait_max_pipe_length_m(0, 100.0), 35.0 + 6.0 + 13.0);
        assert!((design_pressure_kpa(100.0, 2.0) - 16.0).abs() < 1e-12);
        assert!((generator_resistance_kpa(20.0, 20.0) - 10.0).abs() < 1e-12);
        assert!((generator_resistance_kpa(5.0, 10.0) - 40.0).abs() < 1e-12);
        assert!((generator_resistance_kpa(5.0, 20.0) - 40.0).abs() < 1e-12);
        // 20 kW at 20 K: 0,857 m³/h.
        let flow = design_flow_m3_per_h(20.0 * 744.0, 744.0, 20.0, 1.0);
        assert!((flow - 20.0 / 84_000.0 * 3600.0).abs() < 1e-12);
        assert_eq!(hydraulic_power_kw(30.0, 0.5), 0.01);
        let power = 0.05;
        let reference = 1.7 * power + 17.0 * (1.0 - (-15.0_f64).exp()) * 1e-3;
        // f_e = P_ref/P_hydr ≈ 2,04 at 0,05 kW.
        assert!((reference / power - 2.04).abs() < 0.01);
        assert!((pump_energy_factor(power, None, None) - reference / power * 0.92).abs() < 1e-12);
        assert!(
            (pump_energy_factor(3.0, None, None) - (1.25 + (0.2_f64 / 3.0).sqrt()) * 2.0).abs()
                < 1e-12
        );
        assert!((pump_energy_factor(0.05, Some(0.2), Some(0.1)) - 2.0 * 0.8).abs() < 1e-12);
    }

    #[test]
    fn buffer_and_label_loss() {
        let loss = label_standing_loss_w(StorageLabel::C, 200.0);
        assert!((loss - (14.33 + 7.13 * 200.0_f64.powf(0.4))).abs() < 1e-12);
        let monthly = buffer_loss_kwh(90.0, 60.0, 13.0, 744.0, 0.5);
        assert!((monthly - 0.5 * 0.744 * 2.0 * 47.0).abs() < 1e-12);
    }
}
