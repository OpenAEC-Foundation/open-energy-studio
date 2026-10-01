//! External obstruction and movable solar shading of windows.
//!
//! - §17.3.2a minimal obstruction: `F_sh;obst` from table 17.4 for the
//!   heating balance and 1,00 for the cooling balance (table 17.5).
//! - §7.6.6.1.4 movable shading: `g_gl;mi = (1 − f_sh;with)·g + f_sh;with·F_c·g`
//!   (7.42/7.43) with `f_sh;with` from table 7.7 (manual, dwellings) or 7.9
//!   (automatic). For the heating need of dwellings `f_sh;with = 0`; this
//!   module applies movable shading to the cooling balance only.
//!
//! Tilts are bucketed as in the source: up to 22,5° horizontal, up to 67,5°
//! the 45° column, otherwise vertical; the norm's interpolation between tilts
//! and the downward-facing columns are not transcribed. Transcription source:
//! Open Heatloss Studio analysis F3d (2026-07-11), table 17.4 cross-checked
//! against a rendered page there. Review against the norm text is pending.

use crate::climate::Orientation;
use serde::{Deserialize, Serialize};

const TILT_HORIZONTAL_MAX_DEG: f64 = 22.5;
const TILT_SLOPED_MAX_DEG: f64 = 67.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Balance {
    Heating,
    Cooling,
}

/// `[month][N, NE, E, SE, S, SW, W, NW]`.
type Table = [[f64; 8]; 12];

struct TiltTable {
    vertical: Table,
    sloped_45: Table,
    horizontal: [f64; 12],
}

/// Table 17.4, minimal obstruction, heating.
const OBSTRUCTION_HEATING: TiltTable = TiltTable {
    vertical: [
        [1.00, 1.00, 0.92, 0.48, 0.23, 0.49, 0.85, 0.97],
        [1.00, 0.96, 0.79, 0.81, 0.91, 0.83, 0.85, 0.97],
        [1.00, 0.97, 0.82, 0.87, 1.00, 0.93, 0.89, 0.96],
        [0.99, 0.97, 0.91, 0.95, 1.00, 0.92, 0.82, 0.87],
        [0.97, 0.93, 0.95, 1.00, 1.00, 0.99, 0.88, 0.85],
        [0.97, 0.88, 0.90, 1.00, 1.00, 1.00, 0.93, 0.91],
        [0.97, 0.91, 0.93, 0.99, 1.00, 1.00, 0.92, 0.90],
        [0.98, 0.98, 0.94, 0.98, 1.00, 0.99, 0.89, 0.88],
        [1.00, 0.97, 0.87, 0.92, 1.00, 0.91, 0.85, 0.96],
        [1.00, 0.96, 0.84, 0.86, 0.97, 0.88, 0.83, 0.97],
        [1.00, 0.98, 0.92, 0.70, 0.61, 0.71, 0.90, 0.99],
        [1.00, 1.00, 0.86, 0.40, 0.19, 0.58, 0.87, 1.00],
    ],
    sloped_45: [
        [1.00, 0.99, 0.92, 0.56, 0.29, 0.57, 0.86, 0.97],
        [1.00, 0.92, 0.83, 0.86, 0.93, 0.87, 0.85, 0.93],
        [1.00, 0.88, 0.83, 0.89, 1.00, 0.94, 0.86, 0.89],
        [0.84, 0.80, 0.88, 0.93, 1.00, 0.93, 0.79, 0.79],
        [0.65, 0.74, 0.89, 0.96, 0.99, 0.95, 0.80, 0.70],
        [0.68, 0.78, 0.85, 0.96, 0.96, 0.94, 0.86, 0.75],
        [0.69, 0.81, 0.89, 0.96, 0.98, 0.93, 0.85, 0.77],
        [0.74, 0.80, 0.88, 0.94, 0.99, 0.96, 0.83, 0.76],
        [0.96, 0.85, 0.86, 0.92, 1.00, 0.91, 0.81, 0.87],
        [1.00, 0.90, 0.81, 0.89, 0.97, 0.90, 0.86, 0.92],
        [1.00, 0.95, 0.86, 0.76, 0.66, 0.77, 0.92, 0.97],
        [1.00, 0.99, 0.90, 0.48, 0.27, 0.65, 0.85, 1.00],
    ],
    horizontal: [1.00; 12],
};

/// Table 7.7, manual movable shading in dwellings.
const SHADING_MANUAL: TiltTable = TiltTable {
    vertical: [
        [0.00; 8],
        [0.00; 8],
        [0.00, 0.12, 0.47, 0.64, 0.68, 0.55, 0.31, 0.00],
        [0.00, 0.29, 0.59, 0.70, 0.71, 0.66, 0.48, 0.13],
        [0.00, 0.30, 0.56, 0.65, 0.67, 0.60, 0.50, 0.23],
        [0.00, 0.32, 0.51, 0.52, 0.56, 0.56, 0.57, 0.35],
        [0.00, 0.25, 0.49, 0.55, 0.59, 0.54, 0.51, 0.30],
        [0.00, 0.08, 0.44, 0.63, 0.68, 0.70, 0.61, 0.28],
        [0.00, 0.01, 0.43, 0.66, 0.70, 0.66, 0.48, 0.07],
        [0.00, 0.00, 0.39, 0.67, 0.69, 0.62, 0.33, 0.00],
        [0.00; 8],
        [0.00; 8],
    ],
    sloped_45: [
        [0.00; 8],
        [0.00; 8],
        [0.00, 0.20, 0.56, 0.70, 0.73, 0.68, 0.47, 0.10],
        [0.00, 0.49, 0.71, 0.78, 0.80, 0.76, 0.67, 0.43],
        [0.16, 0.59, 0.75, 0.79, 0.82, 0.79, 0.73, 0.58],
        [0.54, 0.58, 0.71, 0.74, 0.75, 0.77, 0.71, 0.64],
        [0.34, 0.55, 0.66, 0.74, 0.74, 0.75, 0.68, 0.57],
        [0.00, 0.44, 0.67, 0.79, 0.82, 0.82, 0.75, 0.57],
        [0.00, 0.19, 0.60, 0.74, 0.76, 0.72, 0.62, 0.25],
        [0.00, 0.01, 0.50, 0.69, 0.71, 0.67, 0.47, 0.00],
        [0.00; 8],
        [0.00; 8],
    ],
    horizontal: [
        0.00, 0.00, 0.56, 0.72, 0.79, 0.79, 0.74, 0.81, 0.65, 0.48, 0.00, 0.00,
    ],
};

/// Table 7.9, automatic movable shading.
const SHADING_AUTOMATIC: TiltTable = TiltTable {
    vertical: [
        [0.00, 0.00, 0.45, 0.78, 0.86, 0.80, 0.48, 0.00],
        [0.00, 0.04, 0.54, 0.74, 0.79, 0.73, 0.44, 0.03],
        [0.00, 0.23, 0.65, 0.81, 0.86, 0.78, 0.55, 0.10],
        [0.12, 0.51, 0.75, 0.83, 0.88, 0.81, 0.71, 0.46],
        [0.25, 0.59, 0.75, 0.81, 0.85, 0.81, 0.73, 0.58],
        [0.36, 0.63, 0.73, 0.75, 0.79, 0.79, 0.78, 0.68],
        [0.34, 0.59, 0.72, 0.75, 0.78, 0.76, 0.73, 0.63],
        [0.22, 0.46, 0.70, 0.81, 0.88, 0.86, 0.79, 0.62],
        [0.04, 0.21, 0.64, 0.84, 0.88, 0.84, 0.68, 0.30],
        [0.00, 0.05, 0.59, 0.83, 0.87, 0.82, 0.51, 0.03],
        [0.00, 0.00, 0.50, 0.79, 0.82, 0.78, 0.37, 0.00],
        [0.00, 0.00, 0.41, 0.79, 0.86, 0.79, 0.43, 0.00],
    ],
    sloped_45: [
        [0.00, 0.00, 0.51, 0.77, 0.81, 0.78, 0.54, 0.00],
        [0.01, 0.19, 0.61, 0.77, 0.79, 0.75, 0.58, 0.12],
        [0.29, 0.62, 0.80, 0.88, 0.89, 0.86, 0.76, 0.57],
        [0.56, 0.78, 0.87, 0.92, 0.92, 0.91, 0.86, 0.78],
        [0.86, 0.83, 0.88, 0.91, 0.92, 0.91, 0.89, 0.83],
        [0.90, 0.86, 0.88, 0.90, 0.91, 0.91, 0.89, 0.88],
        [0.84, 0.82, 0.86, 0.88, 0.89, 0.89, 0.86, 0.83],
        [0.84, 0.80, 0.87, 0.91, 0.94, 0.92, 0.91, 0.86],
        [0.36, 0.66, 0.81, 0.89, 0.91, 0.90, 0.84, 0.70],
        [0.05, 0.32, 0.74, 0.86, 0.88, 0.85, 0.70, 0.32],
        [0.00, 0.00, 0.56, 0.76, 0.81, 0.76, 0.55, 0.00],
        [0.00, 0.00, 0.51, 0.76, 0.82, 0.75, 0.48, 0.00],
    ],
    horizontal: [
        0.48, 0.67, 0.84, 0.91, 0.92, 0.92, 0.92, 0.93, 0.89, 0.79, 0.58, 0.42,
    ],
};

fn column(orientation: Orientation) -> usize {
    match orientation {
        Orientation::North => 0,
        Orientation::NorthEast => 1,
        Orientation::East => 2,
        Orientation::SouthEast => 3,
        Orientation::South => 4,
        Orientation::SouthWest => 5,
        Orientation::West => 6,
        Orientation::NorthWest => 7,
    }
}

fn lookup(table: &TiltTable, orientation: Orientation, tilt_deg: f64, month: u8) -> f64 {
    let index = usize::from(month - 1);
    if tilt_deg <= TILT_HORIZONTAL_MAX_DEG {
        table.horizontal[index]
    } else if tilt_deg <= TILT_SLOPED_MAX_DEG {
        table.sloped_45[index][column(orientation)]
    } else {
        table.vertical[index][column(orientation)]
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Obstruction {
    /// §17.3.2a: no obstructions above 20° and no overhangs below 45°.
    Minimal,
    /// Other §17.3 situations determined elsewhere, twelve monthly factors.
    Declared {
        heating: Vec<f64>,
        cooling: Vec<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadingControl {
    /// Table 7.7, dwellings with manual operation.
    ManualResidential,
    /// Table 7.9, automatic control.
    Automatic,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MovableShading {
    /// `F_c` from table 7.5/7.6 (rounded up to two decimals by the caller).
    pub reduction_factor: f64,
    pub control: ShadingControl,
    pub source_reference: String,
}

/// `F_sh;obst` for one month and balance; `None` for unsupported input.
pub fn obstruction_factor(
    obstruction: &Obstruction,
    orientation: Orientation,
    tilt_deg: f64,
    month: u8,
    balance: Balance,
) -> Option<f64> {
    match obstruction {
        Obstruction::Minimal => match balance {
            Balance::Cooling => Some(1.0),
            Balance::Heating => (tilt_deg <= 90.0)
                .then(|| lookup(&OBSTRUCTION_HEATING, orientation, tilt_deg, month)),
        },
        Obstruction::Declared {
            heating, cooling, ..
        } => {
            let values = match balance {
                Balance::Heating => heating,
                Balance::Cooling => cooling,
            };
            values.get(usize::from(month - 1)).copied()
        }
    }
}

/// Effective g reduction `(1 − f_sh;with) + f_sh;with·F_c` (7.42/7.43); 1,0 on
/// the heating balance and without movable shading.
pub fn movable_shading_factor(
    shading: Option<&MovableShading>,
    orientation: Orientation,
    tilt_deg: f64,
    month: u8,
    balance: Balance,
) -> f64 {
    match (shading, balance) {
        (Some(shading), Balance::Cooling) => {
            let table = match shading.control {
                ShadingControl::ManualResidential => &SHADING_MANUAL,
                ShadingControl::Automatic => &SHADING_AUTOMATIC,
            };
            let fraction = lookup(table, orientation, tilt_deg, month);
            (1.0 - fraction) + fraction * shading.reduction_factor
        }
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_obstruction_reduces_winter_heating_gains_only() {
        let minimal = Obstruction::Minimal;
        let south =
            |month, balance| obstruction_factor(&minimal, Orientation::South, 90.0, month, balance);
        assert_eq!(south(1, Balance::Heating), Some(0.23));
        assert_eq!(south(12, Balance::Heating), Some(0.19));
        assert_eq!(south(7, Balance::Heating), Some(1.0));
        assert_eq!(south(1, Balance::Cooling), Some(1.0));
        assert_eq!(
            obstruction_factor(&minimal, Orientation::South, 0.0, 1, Balance::Heating),
            Some(1.0)
        );
        assert_eq!(
            obstruction_factor(&minimal, Orientation::South, 45.0, 1, Balance::Heating),
            Some(0.29)
        );
        assert_eq!(
            obstruction_factor(&minimal, Orientation::South, 120.0, 1, Balance::Heating),
            None
        );
    }

    #[test]
    fn movable_shading_follows_7_42_on_cooling_balance() {
        let screen = MovableShading {
            reduction_factor: 0.2,
            control: ShadingControl::ManualResidential,
            source_reference: "table 7.5".into(),
        };
        // July, south vertical, manual: f = 0,59 → 0,41 + 0,59·0,20 = 0,528.
        let july =
            movable_shading_factor(Some(&screen), Orientation::South, 90.0, 7, Balance::Cooling);
        assert!((july - 0.528).abs() < 1e-12);
        assert_eq!(
            movable_shading_factor(Some(&screen), Orientation::South, 90.0, 1, Balance::Cooling),
            1.0
        );
        assert_eq!(
            movable_shading_factor(Some(&screen), Orientation::South, 90.0, 7, Balance::Heating),
            1.0
        );
        assert_eq!(
            movable_shading_factor(None, Orientation::South, 90.0, 7, Balance::Cooling),
            1.0
        );
    }
}
