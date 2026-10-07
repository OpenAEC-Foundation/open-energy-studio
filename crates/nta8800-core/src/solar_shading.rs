//! External obstruction and movable solar shading of windows.
//!
//! - §17.3.3 minimal obstruction (p. 708–715): `F_sh;obst` from table 17.4
//!   for the heating balance and 1,00 for the cooling balance (table 17.5).
//!   Intermediate tilts take the nearest tilt column, the higher value on an
//!   exact tie.
//! - §7.6.6.1.4 movable shading (p. 197–202):
//!   `g_gl;mi = (1 − f_sh;with)·g + f_sh;with·F_c·g` (7.42/7.43) with
//!   `f_sh;with` from table 7.7 (manual in dwellings, or manual combined with
//!   glare protection in utility buildings), table 7.8 (manual in utility
//!   buildings without glare protection) or table 7.9 (automatic). Tilts are
//!   interpolated linearly between the 0°, 45°, 90° and 180° columns.
//!   `f_sh;with = 0` on the heating balance only for dwellings with manual
//!   shading or with automatic shading tuned per ISO 52016-3 table C.1
//!   (AC1/AC2); in every other case the tables apply to both balances.
//!
//! - §17.3.4–17.3.7 situations b–g (p. 698–763) with tables 17.7–17.15:
//!   one situation per window (17.3.2; the situations are alternatives, not
//!   multiplied). Relative heights and widths take the column of their range;
//!   tilts take the nearest column of table 17.4 (higher value on a tie).
//!   Surfaces with a tilt below 15° use the south field of view (17.3.1).
//!   A situation that table 17.3 marks "not available" for a balance falls
//!   back to situation g, which for cooling is table 17.5 (1,00) unless an
//!   overhang parallel to a vertical window is present (table 17.9).
//! - Collectors and PV (x = P): tables 17.6, 17.12 and 17.15 through
//!   [`collector_obstruction_factor`].
//! - The extended method of §17.3.8 needs hourly NEN 5060 data; its result
//!   enters as `Declared` factors.
//!
//! Table values were generated from the licensed norm text and checked
//! against the rendered pages by the 2026-10-02 review.

#[path = "solar_shading_tables.rs"]
mod tables;

use crate::climate::Orientation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Balance {
    Heating,
    Cooling,
}

/// Tilt columns of table 17.4 in degrees (0° facing up, 180° facing down).
pub const OBSTRUCTION_TILTS_DEG: [f64; 13] = [
    90.0, 75.0, 60.0, 45.0, 30.0, 15.0, 0.0, 105.0, 120.0, 135.0, 150.0, 165.0, 180.0,
];

/// `[month][N, NE, E, SE, S, SW, W, NW]`.
type Table = [[f64; 8]; 12];

struct ShadingTable {
    vertical: Table,
    sloped_45: Table,
    horizontal_up: [f64; 12],
    horizontal_down: [f64; 12],
}

/// Table 17.4 `[orientation N, NE, E, SE, S, SW, W, NW][month][tilt]`, tilts in
/// the order of [`OBSTRUCTION_TILTS_DEG`].
const OBSTRUCTION_HEATING: [[[f64; 13]; 12]; 8] = [
    [
        [
            1.00, 1.00, 1.00, 1.00, 1.00, 0.95, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 1.00, 0.78, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 0.82, 0.61, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            0.99, 0.98, 0.96, 0.84, 0.57, 0.44, 1.00, 0.99, 0.99, 0.99, 1.00, 1.00, 1.00,
        ],
        [
            0.97, 0.96, 0.89, 0.65, 0.51, 0.43, 1.00, 0.97, 0.97, 0.98, 0.99, 1.00, 1.00,
        ],
        [
            0.97, 0.96, 0.86, 0.68, 0.58, 0.51, 1.00, 0.97, 0.97, 0.98, 0.98, 0.99, 1.00,
        ],
        [
            0.97, 0.96, 0.87, 0.69, 0.58, 0.51, 1.00, 0.97, 0.97, 0.98, 0.98, 1.00, 1.00,
        ],
        [
            0.98, 0.97, 0.94, 0.74, 0.55, 0.45, 1.00, 0.98, 0.98, 0.99, 0.99, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 0.99, 0.96, 0.72, 0.53, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 0.96, 0.66, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 1.00, 0.86, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
    ],
    [
        [
            1.00, 0.99, 0.99, 0.99, 0.95, 0.79, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            0.96, 0.96, 0.95, 0.92, 0.85, 0.73, 1.00, 0.96, 0.96, 0.97, 0.98, 1.00, 1.00,
        ],
        [
            0.97, 0.96, 0.93, 0.88, 0.78, 0.66, 1.00, 0.97, 0.97, 0.97, 0.98, 0.99, 1.00,
        ],
        [
            0.97, 0.94, 0.89, 0.80, 0.68, 0.56, 1.00, 0.97, 0.97, 0.97, 0.98, 0.99, 1.00,
        ],
        [
            0.93, 0.90, 0.84, 0.74, 0.63, 0.53, 1.00, 0.94, 0.94, 0.95, 0.97, 1.00, 1.00,
        ],
        [
            0.88, 0.88, 0.85, 0.78, 0.68, 0.60, 1.00, 0.87, 0.88, 0.90, 0.94, 0.99, 1.00,
        ],
        [
            0.91, 0.91, 0.87, 0.81, 0.72, 0.64, 1.00, 0.91, 0.91, 0.92, 0.95, 0.99, 1.00,
        ],
        [
            0.98, 0.95, 0.89, 0.80, 0.67, 0.55, 1.00, 0.98, 0.98, 0.98, 0.99, 1.00, 1.00,
        ],
        [
            0.97, 0.96, 0.93, 0.85, 0.75, 0.62, 1.00, 0.97, 0.97, 0.97, 0.98, 0.99, 1.00,
        ],
        [
            0.96, 0.96, 0.95, 0.90, 0.80, 0.66, 1.00, 0.96, 0.96, 0.96, 0.98, 0.99, 1.00,
        ],
        [
            0.98, 0.97, 0.96, 0.95, 0.90, 0.73, 1.00, 0.98, 0.98, 0.99, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 0.99, 0.98, 0.84, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
    ],
    [
        [
            0.92, 0.93, 0.93, 0.92, 0.90, 0.86, 1.00, 0.91, 0.90, 0.90, 0.92, 0.96, 1.00,
        ],
        [
            0.79, 0.81, 0.82, 0.83, 0.83, 0.81, 1.00, 0.77, 0.76, 0.77, 0.83, 0.96, 1.00,
        ],
        [
            0.82, 0.84, 0.84, 0.83, 0.81, 0.78, 1.00, 0.81, 0.80, 0.81, 0.87, 0.96, 1.00,
        ],
        [
            0.91, 0.92, 0.91, 0.88, 0.83, 0.77, 1.00, 0.90, 0.90, 0.90, 0.93, 0.98, 1.00,
        ],
        [
            0.95, 0.96, 0.93, 0.89, 0.83, 0.76, 1.00, 0.94, 0.94, 0.95, 0.96, 0.99, 1.00,
        ],
        [
            0.90, 0.91, 0.89, 0.85, 0.80, 0.73, 1.00, 0.89, 0.89, 0.90, 0.93, 0.99, 1.00,
        ],
        [
            0.93, 0.94, 0.92, 0.89, 0.84, 0.79, 1.00, 0.92, 0.92, 0.92, 0.95, 0.99, 1.00,
        ],
        [
            0.94, 0.95, 0.93, 0.88, 0.82, 0.75, 1.00, 0.94, 0.94, 0.94, 0.96, 0.99, 1.00,
        ],
        [
            0.87, 0.88, 0.88, 0.86, 0.84, 0.79, 1.00, 0.86, 0.85, 0.86, 0.90, 0.98, 1.00,
        ],
        [
            0.84, 0.83, 0.83, 0.81, 0.78, 0.74, 1.00, 0.83, 0.82, 0.83, 0.87, 0.96, 1.00,
        ],
        [
            0.92, 0.90, 0.88, 0.86, 0.83, 0.79, 1.00, 0.91, 0.90, 0.90, 0.91, 0.96, 1.00,
        ],
        [
            0.86, 0.87, 0.89, 0.90, 0.90, 0.88, 1.00, 0.84, 0.83, 0.83, 0.86, 0.94, 1.00,
        ],
    ],
    [
        [
            0.48, 0.50, 0.53, 0.56, 0.60, 0.66, 1.00, 0.47, 0.46, 0.47, 0.54, 0.87, 1.00,
        ],
        [
            0.81, 0.83, 0.85, 0.86, 0.88, 0.90, 1.00, 0.80, 0.79, 0.78, 0.80, 0.94, 1.00,
        ],
        [
            0.87, 0.88, 0.89, 0.89, 0.90, 0.90, 1.00, 0.86, 0.85, 0.85, 0.88, 0.97, 1.00,
        ],
        [
            0.95, 0.95, 0.94, 0.93, 0.91, 0.88, 1.00, 0.95, 0.94, 0.94, 0.96, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 0.98, 0.96, 0.93, 0.89, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 0.98, 0.96, 0.92, 0.88, 1.00, 0.99, 0.99, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            0.99, 0.99, 0.98, 0.96, 0.93, 0.90, 1.00, 0.99, 0.99, 0.99, 1.00, 1.00, 1.00,
        ],
        [
            0.98, 0.98, 0.96, 0.94, 0.92, 0.88, 1.00, 0.97, 0.97, 0.97, 0.99, 1.00, 1.00,
        ],
        [
            0.92, 0.92, 0.92, 0.92, 0.91, 0.90, 1.00, 0.91, 0.90, 0.89, 0.92, 0.99, 1.00,
        ],
        [
            0.86, 0.87, 0.88, 0.89, 0.90, 0.91, 1.00, 0.85, 0.84, 0.83, 0.84, 0.96, 1.00,
        ],
        [
            0.70, 0.72, 0.74, 0.76, 0.79, 0.83, 1.00, 0.69, 0.67, 0.67, 0.69, 0.91, 1.00,
        ],
        [
            0.40, 0.42, 0.44, 0.48, 0.53, 0.60, 1.00, 0.39, 0.39, 0.40, 0.47, 0.81, 1.00,
        ],
    ],
    [
        [
            0.23, 0.24, 0.26, 0.29, 0.34, 0.42, 1.00, 0.23, 0.23, 0.26, 0.35, 0.82, 1.00,
        ],
        [
            0.91, 0.92, 0.92, 0.93, 0.94, 0.95, 1.00, 0.91, 0.91, 0.91, 0.92, 0.99, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 0.99, 0.99, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 0.99, 0.99, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 0.99, 0.99, 0.98, 0.97, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 0.99, 0.97, 0.96, 0.94, 0.92, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 0.99, 0.98, 0.97, 0.96, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 0.99, 0.99, 0.99, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            0.97, 0.97, 0.97, 0.97, 0.98, 0.98, 1.00, 0.96, 0.96, 0.96, 0.97, 0.99, 1.00,
        ],
        [
            0.61, 0.62, 0.64, 0.66, 0.69, 0.73, 1.00, 0.61, 0.61, 0.62, 0.67, 0.95, 1.00,
        ],
        [
            0.19, 0.21, 0.23, 0.27, 0.32, 0.42, 1.00, 0.19, 0.20, 0.22, 0.31, 0.75, 1.00,
        ],
    ],
    [
        [
            0.49, 0.51, 0.53, 0.57, 0.61, 0.68, 1.00, 0.47, 0.46, 0.46, 0.50, 0.80, 1.00,
        ],
        [
            0.83, 0.85, 0.86, 0.87, 0.89, 0.90, 1.00, 0.82, 0.80, 0.80, 0.83, 0.95, 1.00,
        ],
        [
            0.93, 0.94, 0.94, 0.94, 0.94, 0.93, 1.00, 0.92, 0.91, 0.91, 0.93, 0.98, 1.00,
        ],
        [
            0.92, 0.93, 0.93, 0.93, 0.91, 0.90, 1.00, 0.91, 0.90, 0.90, 0.93, 0.99, 1.00,
        ],
        [
            0.99, 0.98, 0.97, 0.95, 0.93, 0.90, 1.00, 0.99, 0.98, 0.98, 0.99, 1.00, 1.00,
        ],
        [
            1.00, 0.99, 0.97, 0.94, 0.91, 0.87, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            1.00, 0.99, 0.96, 0.93, 0.90, 0.86, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
        [
            0.99, 0.98, 0.98, 0.96, 0.95, 0.92, 1.00, 0.98, 0.98, 0.98, 0.99, 1.00, 1.00,
        ],
        [
            0.91, 0.91, 0.91, 0.91, 0.90, 0.90, 1.00, 0.90, 0.89, 0.89, 0.91, 0.99, 1.00,
        ],
        [
            0.88, 0.89, 0.89, 0.90, 0.91, 0.91, 1.00, 0.87, 0.85, 0.85, 0.87, 0.96, 1.00,
        ],
        [
            0.71, 0.72, 0.74, 0.77, 0.80, 0.84, 1.00, 0.69, 0.68, 0.68, 0.72, 0.93, 1.00,
        ],
        [
            0.58, 0.60, 0.62, 0.65, 0.69, 0.74, 1.00, 0.57, 0.56, 0.57, 0.59, 0.85, 1.00,
        ],
    ],
    [
        [
            0.85, 0.86, 0.86, 0.86, 0.84, 0.81, 1.00, 0.83, 0.82, 0.82, 0.84, 0.91, 1.00,
        ],
        [
            0.85, 0.86, 0.86, 0.85, 0.83, 0.79, 1.00, 0.83, 0.83, 0.84, 0.88, 0.97, 1.00,
        ],
        [
            0.89, 0.89, 0.88, 0.86, 0.82, 0.77, 1.00, 0.87, 0.87, 0.88, 0.91, 0.97, 1.00,
        ],
        [
            0.82, 0.82, 0.81, 0.79, 0.75, 0.70, 1.00, 0.80, 0.79, 0.80, 0.85, 0.96, 1.00,
        ],
        [
            0.88, 0.87, 0.84, 0.80, 0.75, 0.69, 1.00, 0.87, 0.86, 0.87, 0.91, 0.98, 1.00,
        ],
        [
            0.93, 0.92, 0.90, 0.86, 0.81, 0.75, 1.00, 0.92, 0.92, 0.92, 0.95, 0.99, 1.00,
        ],
        [
            0.92, 0.91, 0.89, 0.85, 0.79, 0.73, 1.00, 0.91, 0.91, 0.91, 0.94, 0.99, 1.00,
        ],
        [
            0.89, 0.88, 0.86, 0.83, 0.79, 0.74, 1.00, 0.88, 0.87, 0.88, 0.91, 0.98, 1.00,
        ],
        [
            0.85, 0.85, 0.83, 0.81, 0.77, 0.72, 1.00, 0.84, 0.84, 0.85, 0.89, 0.97, 1.00,
        ],
        [
            0.83, 0.85, 0.86, 0.86, 0.84, 0.79, 1.00, 0.82, 0.82, 0.83, 0.88, 0.97, 1.00,
        ],
        [
            0.90, 0.92, 0.93, 0.92, 0.90, 0.86, 1.00, 0.89, 0.89, 0.89, 0.92, 0.98, 1.00,
        ],
        [
            0.87, 0.86, 0.85, 0.85, 0.82, 0.79, 1.00, 0.86, 0.85, 0.85, 0.88, 0.95, 1.00,
        ],
    ],
    [
        [
            0.97, 0.97, 0.98, 0.97, 0.94, 0.77, 1.00, 0.98, 0.98, 0.99, 0.99, 1.00, 1.00,
        ],
        [
            0.97, 0.97, 0.96, 0.93, 0.87, 0.74, 1.00, 0.97, 0.97, 0.97, 0.98, 0.99, 1.00,
        ],
        [
            0.96, 0.96, 0.94, 0.89, 0.79, 0.67, 1.00, 0.96, 0.96, 0.96, 0.97, 0.99, 1.00,
        ],
        [
            0.87, 0.87, 0.85, 0.79, 0.69, 0.58, 1.00, 0.87, 0.87, 0.88, 0.92, 0.98, 1.00,
        ],
        [
            0.85, 0.83, 0.78, 0.70, 0.61, 0.53, 1.00, 0.84, 0.85, 0.87, 0.92, 0.98, 1.00,
        ],
        [
            0.91, 0.88, 0.83, 0.75, 0.66, 0.59, 1.00, 0.90, 0.90, 0.92, 0.94, 0.98, 1.00,
        ],
        [
            0.90, 0.88, 0.84, 0.77, 0.67, 0.59, 1.00, 0.90, 0.90, 0.91, 0.94, 0.99, 1.00,
        ],
        [
            0.88, 0.86, 0.83, 0.76, 0.66, 0.58, 1.00, 0.87, 0.88, 0.89, 0.93, 0.99, 1.00,
        ],
        [
            0.96, 0.96, 0.93, 0.87, 0.76, 0.63, 1.00, 0.96, 0.96, 0.96, 0.97, 0.99, 1.00,
        ],
        [
            0.97, 0.98, 0.96, 0.92, 0.82, 0.66, 1.00, 0.97, 0.98, 0.98, 0.99, 1.00, 1.00,
        ],
        [
            0.99, 0.99, 0.98, 0.97, 0.93, 0.75, 1.00, 0.99, 0.99, 0.99, 0.99, 1.00, 1.00,
        ],
        [
            1.00, 1.00, 1.00, 1.00, 0.99, 0.86, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
        ],
    ],
];
/// Table 7.7.
const SHADING_TABLE_7_7: ShadingTable = ShadingTable {
    vertical: [
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
        [0.00, 0.12, 0.47, 0.64, 0.68, 0.55, 0.31, 0.00],
        [0.00, 0.29, 0.59, 0.70, 0.71, 0.66, 0.48, 0.13],
        [0.00, 0.30, 0.56, 0.65, 0.67, 0.60, 0.50, 0.23],
        [0.00, 0.32, 0.51, 0.52, 0.56, 0.56, 0.57, 0.35],
        [0.00, 0.25, 0.49, 0.55, 0.59, 0.54, 0.51, 0.30],
        [0.00, 0.08, 0.44, 0.63, 0.68, 0.70, 0.61, 0.28],
        [0.00, 0.01, 0.43, 0.66, 0.70, 0.66, 0.48, 0.07],
        [0.00, 0.00, 0.39, 0.67, 0.69, 0.62, 0.33, 0.00],
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
    ],
    sloped_45: [
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
        [0.00, 0.20, 0.56, 0.70, 0.73, 0.68, 0.47, 0.10],
        [0.00, 0.49, 0.71, 0.78, 0.80, 0.76, 0.67, 0.43],
        [0.16, 0.59, 0.75, 0.79, 0.82, 0.79, 0.73, 0.58],
        [0.54, 0.58, 0.71, 0.74, 0.75, 0.77, 0.71, 0.64],
        [0.34, 0.55, 0.66, 0.74, 0.74, 0.75, 0.68, 0.57],
        [0.00, 0.44, 0.67, 0.79, 0.82, 0.82, 0.75, 0.57],
        [0.00, 0.19, 0.60, 0.74, 0.76, 0.72, 0.62, 0.25],
        [0.00, 0.01, 0.50, 0.69, 0.71, 0.67, 0.47, 0.00],
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
        [0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00],
    ],
    horizontal_up: [
        0.00, 0.00, 0.56, 0.72, 0.79, 0.79, 0.74, 0.81, 0.65, 0.48, 0.00, 0.00,
    ],
    horizontal_down: [
        0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00,
    ],
};
/// Table 7.8.
const SHADING_TABLE_7_8: ShadingTable = ShadingTable {
    vertical: [
        [0.00, 0.00, 0.19, 0.64, 0.70, 0.60, 0.26, 0.00],
        [0.00, 0.02, 0.34, 0.59, 0.62, 0.50, 0.27, 0.00],
        [0.00, 0.12, 0.47, 0.64, 0.68, 0.55, 0.31, 0.00],
        [0.00, 0.29, 0.59, 0.70, 0.71, 0.66, 0.48, 0.13],
        [0.00, 0.30, 0.56, 0.65, 0.67, 0.60, 0.50, 0.23],
        [0.00, 0.32, 0.51, 0.52, 0.56, 0.56, 0.57, 0.35],
        [0.00, 0.25, 0.49, 0.55, 0.59, 0.54, 0.51, 0.30],
        [0.00, 0.08, 0.44, 0.63, 0.68, 0.70, 0.61, 0.28],
        [0.00, 0.01, 0.43, 0.66, 0.70, 0.66, 0.48, 0.07],
        [0.00, 0.00, 0.39, 0.67, 0.69, 0.62, 0.33, 0.00],
        [0.00, 0.00, 0.18, 0.65, 0.72, 0.60, 0.18, 0.00],
        [0.00, 0.00, 0.06, 0.61, 0.66, 0.59, 0.06, 0.00],
    ],
    sloped_45: [
        [0.00, 0.00, 0.23, 0.61, 0.63, 0.58, 0.27, 0.00],
        [0.00, 0.02, 0.36, 0.54, 0.61, 0.56, 0.27, 0.00],
        [0.00, 0.20, 0.56, 0.70, 0.73, 0.68, 0.47, 0.10],
        [0.00, 0.49, 0.71, 0.78, 0.80, 0.76, 0.67, 0.43],
        [0.16, 0.59, 0.75, 0.79, 0.82, 0.79, 0.73, 0.58],
        [0.54, 0.58, 0.71, 0.74, 0.75, 0.77, 0.71, 0.64],
        [0.34, 0.55, 0.66, 0.74, 0.74, 0.75, 0.68, 0.57],
        [0.00, 0.44, 0.67, 0.79, 0.82, 0.82, 0.75, 0.57],
        [0.00, 0.19, 0.60, 0.74, 0.76, 0.72, 0.62, 0.25],
        [0.00, 0.01, 0.50, 0.69, 0.71, 0.67, 0.47, 0.00],
        [0.00, 0.00, 0.26, 0.61, 0.67, 0.59, 0.22, 0.00],
        [0.00, 0.00, 0.06, 0.57, 0.62, 0.52, 0.06, 0.00],
    ],
    horizontal_up: [
        0.04, 0.32, 0.56, 0.72, 0.79, 0.79, 0.74, 0.81, 0.65, 0.48, 0.21, 0.00,
    ],
    horizontal_down: [
        0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00, 0.00,
    ],
};
/// Table 7.9.
const SHADING_TABLE_7_9: ShadingTable = ShadingTable {
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
    horizontal_up: [
        0.48, 0.67, 0.84, 0.91, 0.92, 0.92, 0.92, 0.93, 0.89, 0.79, 0.58, 0.42,
    ],
    horizontal_down: [
        0.00, 0.00, 0.00, 0.01, 0.17, 0.19, 0.11, 0.02, 0.00, 0.00, 0.00, 0.00,
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

/// 17.3.1: below 15° the field of view is taken towards the south.
fn view_orientation(orientation: Orientation, tilt_deg: f64) -> Orientation {
    if tilt_deg < 15.0 {
        Orientation::South
    } else {
        orientation
    }
}

/// Nearest tilt column of a 13-column table (higher value on a tie);
/// `None` outside 0–180°.
fn tilt_lookup(
    table: &[[[f64; 13]; 12]; 8],
    orientation: Orientation,
    tilt_deg: f64,
    month: u8,
) -> Option<f64> {
    if !(0.0..=180.0).contains(&tilt_deg) {
        return None;
    }
    let orientation = view_orientation(orientation, tilt_deg);
    let row = &table[column(orientation)][usize::from(month - 1)];
    let mut best: Option<(f64, f64)> = None;
    for (index, tilt) in OBSTRUCTION_TILTS_DEG.iter().enumerate() {
        let distance = (tilt - tilt_deg).abs();
        let value = row[index];
        best = match best {
            Some((best_distance, best_value))
                if best_distance < distance - 1e-9
                    || ((best_distance - distance).abs() <= 1e-9 && best_value >= value) =>
            {
                Some((best_distance, best_value))
            }
            _ => Some((distance, value)),
        };
    }
    best.map(|(_, value)| value)
}

/// Table 17.4 with the nearest-column rule; `None` outside 0–180°.
fn obstruction_lookup(orientation: Orientation, tilt_deg: f64, month: u8) -> Option<f64> {
    tilt_lookup(&OBSTRUCTION_HEATING, orientation, tilt_deg, month)
}

/// Whether the nearest tilt column of table 17.4 is the vertical one;
/// situations b–d apply to vertical surfaces only.
pub fn is_vertical(tilt_deg: f64) -> bool {
    (82.5..97.5).contains(&tilt_deg)
}

/// Column of tables 17.7–17.9: relative height < 0,5, 0,5–1,0, ≥ 1,0.
fn height_column(relative_height: f64) -> usize {
    if relative_height < 0.5 {
        0
    } else if relative_height < 1.0 {
        1
    } else {
        2
    }
}

/// Column of tables 17.10–17.12 for a side and relative width.
fn side_column(side: ObstructionSide, relative_width: f64) -> usize {
    let base = match side {
        ObstructionSide::Left => 0,
        ObstructionSide::Right => 2,
        ObstructionSide::Both => 4,
    };
    base + usize::from(relative_width >= 1.0)
}

fn lerp(a: f64, b: f64, fraction: f64) -> f64 {
    a + (b - a) * fraction
}

/// Tables 7.7–7.9 with linear interpolation between 0°, 45°, 90° and 180°.
fn shading_lookup(table: &ShadingTable, orientation: Orientation, tilt_deg: f64, month: u8) -> f64 {
    let index = usize::from(month - 1);
    let col = column(orientation);
    let up = table.horizontal_up[index];
    let sloped = table.sloped_45[index][col];
    let vertical = table.vertical[index][col];
    let down = table.horizontal_down[index];
    let tilt = tilt_deg.clamp(0.0, 180.0);
    if tilt <= 45.0 {
        lerp(up, sloped, tilt / 45.0)
    } else if tilt <= 90.0 {
        lerp(sloped, vertical, (tilt - 45.0) / 45.0)
    } else {
        lerp(vertical, down, (tilt - 90.0) / 90.0)
    }
}

/// Side of the field of view with a side obstruction (situation d).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObstructionSide {
    Left,
    Right,
    /// Both sides: `b_b` of the larger obstruction (smallest `b_b`).
    Both,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Obstruction {
    /// §17.3.2a: no obstructions above 20° and no overhangs below 45°.
    Minimal,
    /// §17.3.2b: obstruction of constant height parallel to a vertical
    /// window; `relativeHeight` is h_b;⊥ (table 17.7). Cooling: situation g.
    #[serde(rename_all = "camelCase")]
    ParallelObstruction { relative_height: f64 },
    /// §17.3.2c: overhang of constant height parallel to a vertical window
    /// (balcony, gallery); `relativeHeight` is h_o;⊥ (tables 17.8/17.9).
    #[serde(rename_all = "camelCase")]
    Overhang { relative_height: f64 },
    /// §17.3.2d: side obstruction(s) at a vertical window (tables
    /// 17.10/17.11). Cooling needs `coolingHeightCondition` (lowest point at
    /// least 2,5 m above the window top); otherwise situation g (table 17.5).
    #[serde(rename_all = "camelCase")]
    SideObstruction {
        side: ObstructionSide,
        relative_width: f64,
        #[serde(default)]
        cooling_height_condition: bool,
    },
    /// §17.3.2e: full obstruction (tables 17.13/17.14), conservative for
    /// heating. Cooling uses table 17.14 only with `coolingConditionsMet`
    /// (a broad obstruction above h_b 0,36 and a broad overhang below
    /// h_o 1,00), otherwise table 17.5.
    #[serde(rename_all = "camelCase")]
    Full {
        #[serde(default)]
        cooling_conditions_met: bool,
    },
    /// §17.3.2g: other obstruction; heating table 17.13, cooling table 17.9
    /// when one of the obstructions is a parallel overhang of a vertical
    /// window (`overhangRelativeHeight`, h_o;⊥), otherwise table 17.5.
    #[serde(rename_all = "camelCase")]
    Other {
        #[serde(default)]
        overhang_relative_height: Option<f64>,
    },
    /// Other §17.3 situations determined elsewhere (e.g. the extended method
    /// of §17.3.8), twelve monthly factors.
    Declared {
        heating: Vec<f64>,
        cooling: Vec<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

/// Validation codes for an obstruction at a window of the given tilt, as
/// `(code, field suffix)`; the suffix is appended to the obstruction path.
pub fn validate_obstruction(
    obstruction: &Obstruction,
    tilt_deg: f64,
) -> Vec<(&'static str, &'static str)> {
    let mut issues = Vec::new();
    let geometry = |value: f64| value.is_finite() && value >= 0.0;
    match obstruction {
        Obstruction::Minimal | Obstruction::Full { .. } => {}
        Obstruction::ParallelObstruction { relative_height }
        | Obstruction::Overhang { relative_height } => {
            if !geometry(*relative_height) {
                issues.push(("obstruction_geometry_invalid", ".relativeHeight"));
            }
            if !is_vertical(tilt_deg) {
                issues.push(("obstruction_situation_requires_vertical", ".method"));
            }
        }
        Obstruction::SideObstruction { relative_width, .. } => {
            if !geometry(*relative_width) {
                issues.push(("obstruction_geometry_invalid", ".relativeWidth"));
            }
            if !is_vertical(tilt_deg) {
                issues.push(("obstruction_situation_requires_vertical", ".method"));
            }
        }
        Obstruction::Other {
            overhang_relative_height,
        } => {
            if overhang_relative_height.is_some_and(|value| !geometry(value)) {
                issues.push(("obstruction_geometry_invalid", ".overhangRelativeHeight"));
            }
        }
        Obstruction::Declared {
            heating, cooling, ..
        } => {
            if [heating, cooling].iter().any(|values| {
                values.len() != 12 || values.iter().any(|value| !(0.0..=1.0).contains(value))
            }) {
                issues.push(("window_obstruction_factor_invalid", ""));
            }
        }
    }
    issues
}

/// Obstruction of collectors for hot water and PV panels (x = P).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum CollectorObstruction {
    /// Situation a: table 17.6 (1,00).
    Minimal,
    /// Situation d: table 17.12; both sides with b_b < 1 on a non-vertical
    /// panel takes table 17.15 (footnote c).
    #[serde(rename_all = "camelCase")]
    SideObstruction {
        side: ObstructionSide,
        relative_width: f64,
    },
    /// Situation e: table 17.15.
    Full,
    /// Situation f: roof edge of height h_dakrand (top of the edge above the
    /// bottom of the panel) at the shortest horizontal distance l_dakrand;
    /// obstructing when h > 0,5 m and l < h (table 17.15), otherwise
    /// situation a.
    #[serde(rename_all = "camelCase")]
    RoofEdge { height_m: f64, distance_m: f64 },
    /// Situation g: table 17.15.
    Other,
    /// Determined elsewhere (extended method), twelve monthly factors.
    Declared {
        factors: Vec<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

/// `F_sh;obst;mi` of a collector or PV panel; `None` for invalid input.
pub fn collector_obstruction_factor(
    obstruction: &CollectorObstruction,
    orientation: Orientation,
    tilt_deg: f64,
    month: u8,
) -> Option<f64> {
    if !(1..=12).contains(&month) || !(0.0..=180.0).contains(&tilt_deg) {
        return None;
    }
    let index = usize::from(month - 1);
    match obstruction {
        CollectorObstruction::Minimal => Some(1.0),
        CollectorObstruction::SideObstruction {
            side,
            relative_width,
        } => {
            if !(relative_width.is_finite() && *relative_width >= 0.0) {
                return None;
            }
            if *side == ObstructionSide::Both && *relative_width < 1.0 && !is_vertical(tilt_deg) {
                return tilt_lookup(&tables::TABLE_17_15, orientation, tilt_deg, month);
            }
            let view = view_orientation(orientation, tilt_deg);
            Some(tables::TABLE_17_12[column(view)][index][side_column(*side, *relative_width)])
        }
        CollectorObstruction::Full | CollectorObstruction::Other => {
            tilt_lookup(&tables::TABLE_17_15, orientation, tilt_deg, month)
        }
        CollectorObstruction::RoofEdge {
            height_m,
            distance_m,
        } => {
            // Situation f is new in 2025+C1 (17.3.2 f, p. 702); NTA 8800:2024
            // has no roof-edge situation (p. 678–687).
            if !crate::norm_versions::profile().roof_edge_obstruction {
                return None;
            }
            let valid = |value: f64| value.is_finite() && value >= 0.0;
            if !(valid(*height_m) && valid(*distance_m)) {
                return None;
            }
            if *height_m > 0.5 && distance_m < height_m {
                tilt_lookup(&tables::TABLE_17_15, orientation, tilt_deg, month)
            } else {
                Some(1.0)
            }
        }
        CollectorObstruction::Declared { factors, .. } => factors
            .get(index)
            .copied()
            .filter(|value| (0.0..=1.0).contains(value)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadingControl {
    /// Dwellings, manual operation: table 7.7, not used on the heating balance.
    ManualResidential,
    /// Dwellings, automatic control with switching points per ISO 52016-3
    /// table C.1 (AC1/AC2 modes): table 7.9, not used on the heating balance.
    AutomaticResidentialIso52016,
    /// Automatic control in all other cases: table 7.9 on both balances.
    Automatic,
    /// Utility buildings, manual combined with glare protection: table 7.7 on
    /// both balances.
    ManualUtilityWithGlareProtection,
    /// Utility buildings, manual without glare protection: table 7.8 on both
    /// balances.
    ManualUtilityWithoutGlareProtection,
}

impl ShadingControl {
    fn table(self) -> &'static ShadingTable {
        match self {
            Self::ManualResidential | Self::ManualUtilityWithGlareProtection => &SHADING_TABLE_7_7,
            Self::ManualUtilityWithoutGlareProtection => &SHADING_TABLE_7_8,
            // NTA 8800:2023 p. 181: automatic shading of dwellings takes
            // table 7.7 (table 7.9 is for utility buildings only, p. 183).
            Self::AutomaticResidentialIso52016
                if !crate::norm_versions::profile().dwelling_shading_heating_off =>
            {
                &SHADING_TABLE_7_7
            }
            Self::Automatic | Self::AutomaticResidentialIso52016 => &SHADING_TABLE_7_9,
        }
    }

    /// p. 197 case 1: `f_sh;with = 0` on the heating balance.
    pub fn off_for_heating(self) -> bool {
        matches!(
            self,
            Self::ManualResidential | Self::AutomaticResidentialIso52016
        )
    }

    /// NTA 8800:2023 p. 181: automatic shading of a dwelling is a table 7.7
    /// case; the generic `Automatic` (table 7.9) belongs to utility
    /// buildings only in that edition.
    pub fn in_edition(self, residential: bool) -> bool {
        crate::norm_versions::profile().dwelling_shading_heating_off
            || !(residential && self == Self::Automatic)
    }

    /// Whether the control variant fits a residential (`true`) or utility
    /// (`false`) function; `Automatic` fits both.
    pub fn fits_function(self, residential: bool) -> bool {
        match self {
            Self::ManualResidential | Self::AutomaticResidentialIso52016 => residential,
            Self::ManualUtilityWithGlareProtection | Self::ManualUtilityWithoutGlareProtection => {
                !residential
            }
            Self::Automatic => true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MovableShading {
    /// `F_c` (7.43), rounded up to two decimals; omitted when `device`
    /// gives the table 7.5/7.6 value.
    #[serde(default = "nan", skip_serializing_if = "is_nan")]
    pub reduction_factor: f64,
    /// Table 7.5/7.6 device; replaces `reductionFactor`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<ShadingDevice>,
    pub control: ShadingControl,
    pub source_reference: String,
}

fn nan() -> f64 {
    f64::NAN
}

/// A table 7.5/7.6 device leaves `reductionFactor` unset (NaN); writing it
/// out would trip the finite-number guard on the derived input.
fn is_nan(value: &f64) -> bool {
    value.is_nan()
}

impl MovableShading {
    /// `F_c` of 7.43: table 7.5/7.6 for a `device`, otherwise the given
    /// value rounded up to two decimals.
    pub fn reduction_factor_for(&self, orientation: Orientation) -> f64 {
        match &self.device {
            Some(device) => device.reduction_factor(orientation),
            None => (self.reduction_factor * 100.0 - 1e-9).ceil() / 100.0,
        }
    }
}

/// Table 7.5 colour classes (by T_s or R_s, footnote a).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadeColour {
    /// Black, anthracite, dark brown.
    Dark,
    Other,
    White,
    Unknown,
}

/// Tables 7.5 and 7.6: common movable shading devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShadingDevice {
    ExternalScreen {
        colour: ShadeColour,
    },
    ExternalVenetianBlind {
        colour: ShadeColour,
    },
    /// Table 7.5 has no dark row for roller shutters: dark counts as other.
    ExternalRollerShutter {
        colour: ShadeColour,
    },
    /// Metallised fabric inside, R_s > 0,72 of the metal layer.
    InternalMetallisedFabric,
    /// Uitvalscherm (table 7.6).
    DropArmAwning,
    /// Knikarmscherm (table 7.6).
    FoldingArmAwning,
}

impl ShadingDevice {
    /// Table 7.5 has the "onbekende kleur" rows from NTA 8800:2023 (p. 180);
    /// 2022 p. 176 lacks them.
    pub fn in_edition(self) -> bool {
        let unknown = matches!(
            self,
            Self::ExternalScreen {
                colour: ShadeColour::Unknown
            } | Self::ExternalVenetianBlind {
                colour: ShadeColour::Unknown
            } | Self::ExternalRollerShutter {
                colour: ShadeColour::Unknown
            }
        );
        !unknown || crate::norm_versions::profile().unknown_shade_colour_rows
    }

    /// `F_c` of tables 7.5/7.6; table 7.6 by orientation.
    pub fn reduction_factor(self, orientation: Orientation) -> f64 {
        use Orientation::*;
        use ShadeColour::*;
        match self {
            Self::ExternalScreen { colour } => match colour {
                Dark => 0.12,
                Other | Unknown => 0.20,
                White => 0.25,
            },
            Self::ExternalVenetianBlind { colour } => match colour {
                Dark => 0.05,
                Other | Unknown => 0.10,
                White => 0.20,
            },
            Self::ExternalRollerShutter { colour } => match colour {
                Dark | Other | Unknown => 0.11,
                White => 0.04,
            },
            Self::InternalMetallisedFabric => 0.45,
            Self::DropArmAwning => match orientation {
                North => 0.50,
                NorthEast | NorthWest => 0.45,
                _ => 0.35,
            },
            Self::FoldingArmAwning => match orientation {
                North => 0.90,
                NorthEast | NorthWest => 0.80,
                East | West => 0.65,
                SouthEast | SouthWest => 0.55,
                South => 0.50,
            },
        }
    }
}

/// Table 7.4: `g_gl;n` of common glazing types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GlazingType {
    Single,
    Double,
    /// HR++: double with a spectrally selective low-e coating.
    DoubleLowE,
    /// Triple without or with one low-e coating.
    TripleOneCoating,
    TripleTwoCoatings,
    /// Single with an uncoated secondary pane (voor-/achterzetraam).
    SingleWithSecondaryPane,
    /// Solar-control film or glass (footnote b: utility buildings only,
    /// when no g of the film or glass is known).
    SolarControl,
}

impl GlazingType {
    pub fn g_perpendicular(self) -> f64 {
        match self {
            Self::Single => 0.85,
            Self::Double | Self::SingleWithSecondaryPane => 0.75,
            Self::DoubleLowE => 0.60,
            Self::TripleOneCoating => 0.50,
            Self::TripleTwoCoatings | Self::SolarControl => 0.40,
        }
    }
}

/// Fixed external horizontal louvres (7.41a/7.41b, tables 7.4a/7.4b).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FixedLouvres {
    /// Lamellae perpendicular to the window (90°): F_c;lam 0,27.
    Horizontal90,
    /// Downward-angled lamellae: F_c;lam 0,15.
    HorizontalAngled,
    /// Rotatable lamellae (7.41b): open 0,27, closed 0,06, closed for
    /// `f_sh;with` of 7.6.6.1.4.
    Rotatable { control: ShadingControl },
}

/// 7.41: diffusing glazing or fixed shading with ISO 15099 values.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiffusingGlazing {
    /// `g_gl,alt` at a solar altitude of 45°.
    pub g_altitude45: f64,
    /// `g_gl,dif` for isotropic diffuse radiation.
    pub g_diffuse: f64,
    pub source_reference: String,
}

/// §7.6.6.1.2/7.6.6.1.3 glazing details of a window.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GlazingSolar {
    /// Table 7.4 type; replaces `gPerpendicular`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glazing_type: Option<GlazingType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_louvres: Option<FixedLouvres>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diffusing: Option<DiffusingGlazing>,
}

/// `F_sh;obst` for one month and balance; `None` for unsupported input.
pub fn obstruction_factor(
    obstruction: &Obstruction,
    orientation: Orientation,
    tilt_deg: f64,
    month: u8,
    balance: Balance,
) -> Option<f64> {
    if !(1..=12).contains(&month) {
        return None;
    }
    let index = usize::from(month - 1);
    let col = column(orientation);
    // Table 17.5: 1,00 for every tilt and month.
    let minimal_cooling = (0.0..=180.0).contains(&tilt_deg).then_some(1.0);
    let vertical = |value: f64| is_vertical(tilt_deg).then_some(value);
    match obstruction {
        Obstruction::Minimal => match balance {
            Balance::Cooling => minimal_cooling,
            Balance::Heating => obstruction_lookup(orientation, tilt_deg, month),
        },
        Obstruction::ParallelObstruction { relative_height } => match balance {
            Balance::Heating => {
                vertical(tables::TABLE_17_7[col][index][height_column(*relative_height)])
            }
            // Table 17.3: not available for cooling; situation g without an
            // overhang is table 17.5.
            Balance::Cooling => vertical(1.0),
        },
        Obstruction::Overhang { relative_height } => {
            let table = match balance {
                Balance::Heating => &tables::TABLE_17_8,
                Balance::Cooling => &tables::TABLE_17_9,
            };
            vertical(table[col][index][height_column(*relative_height)])
        }
        Obstruction::SideObstruction {
            side,
            relative_width,
            cooling_height_condition,
        } => match balance {
            Balance::Heating => {
                vertical(tables::TABLE_17_10[col][index][side_column(*side, *relative_width)])
            }
            Balance::Cooling if *cooling_height_condition => {
                vertical(tables::TABLE_17_11[col][index][side_column(*side, *relative_width)])
            }
            Balance::Cooling => vertical(1.0),
        },
        Obstruction::Full {
            cooling_conditions_met,
        } => match balance {
            Balance::Heating => tilt_lookup(&tables::TABLE_17_13, orientation, tilt_deg, month),
            Balance::Cooling if *cooling_conditions_met => {
                tilt_lookup(&tables::TABLE_17_14, orientation, tilt_deg, month)
            }
            Balance::Cooling => minimal_cooling,
        },
        Obstruction::Other {
            overhang_relative_height,
        } => match balance {
            Balance::Heating => tilt_lookup(&tables::TABLE_17_13, orientation, tilt_deg, month),
            Balance::Cooling => match overhang_relative_height {
                Some(height) if is_vertical(tilt_deg) => {
                    Some(tables::TABLE_17_9[col][index][height_column(*height)])
                }
                _ => minimal_cooling,
            },
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

/// `f_sh;with` for one month and balance.
pub fn shading_fraction(
    control: ShadingControl,
    orientation: Orientation,
    tilt_deg: f64,
    month: u8,
    balance: Balance,
) -> f64 {
    // 2024+ p. 181 case 1; NTA 8800:2023 (p. 179–181) has no such case.
    if balance == Balance::Heating
        && control.off_for_heating()
        && crate::norm_versions::profile().dwelling_shading_heating_off
    {
        0.0
    } else {
        shading_lookup(control.table(), orientation, tilt_deg, month)
    }
}

/// Effective g reduction `(1 − f_sh;with) + f_sh;with·F_c` (7.42/7.43); 1,0
/// without movable shading.
pub fn movable_shading_factor(
    shading: Option<&MovableShading>,
    orientation: Orientation,
    tilt_deg: f64,
    month: u8,
    balance: Balance,
) -> f64 {
    match shading {
        Some(shading) => {
            let fraction = shading_fraction(shading.control, orientation, tilt_deg, month, balance);
            (1.0 - fraction) + fraction * shading.reduction_factor_for(orientation)
        }
        None => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heating(orientation: Orientation, tilt: f64, month: u8) -> Option<f64> {
        obstruction_factor(
            &Obstruction::Minimal,
            orientation,
            tilt,
            month,
            Balance::Heating,
        )
    }

    #[test]
    fn minimal_obstruction_reduces_winter_heating_gains_only() {
        assert_eq!(heating(Orientation::South, 90.0, 1), Some(0.23));
        assert_eq!(heating(Orientation::South, 90.0, 12), Some(0.19));
        assert_eq!(heating(Orientation::South, 90.0, 7), Some(1.0));
        assert_eq!(heating(Orientation::South, 0.0, 1), Some(1.0));
        assert_eq!(heating(Orientation::South, 45.0, 1), Some(0.29));
        assert_eq!(
            obstruction_factor(
                &Obstruction::Minimal,
                Orientation::South,
                90.0,
                1,
                Balance::Cooling
            ),
            Some(1.0)
        );
    }

    #[test]
    fn table_17_4_takes_the_nearest_tilt_column() {
        // Review examples: 15° south January 0,42; 30° south January 0,34.
        assert_eq!(heating(Orientation::South, 15.0, 1), Some(0.42));
        assert_eq!(heating(Orientation::South, 30.0, 1), Some(0.34));
        assert_eq!(heating(Orientation::NorthWest, 60.0, 5), Some(0.78));
        // 20° is nearest to 15°; 22,5° is a tie between 15° (0,42) and 30° (0,34).
        assert_eq!(heating(Orientation::South, 20.0, 1), Some(0.42));
        assert_eq!(heating(Orientation::South, 22.5, 1), Some(0.42));
        // Downward-facing columns: 120° south January 0,23, 165° 0,82.
        assert_eq!(heating(Orientation::South, 120.0, 1), Some(0.23));
        assert_eq!(heating(Orientation::South, 165.0, 1), Some(0.82));
        assert_eq!(heating(Orientation::South, 181.0, 1), None);
    }

    #[test]
    fn shading_tables_interpolate_between_tilt_columns() {
        // Table 7.9 April: south 90° 0,88, 45° 0,92, 0° 0,91, 180° 0,01.
        let auto = ShadingControl::Automatic;
        let at = |tilt| shading_fraction(auto, Orientation::South, tilt, 4, Balance::Cooling);
        assert!((at(90.0) - 0.88).abs() < 1e-12);
        assert!((at(67.5) - 0.90).abs() < 1e-12);
        assert!((at(22.5) - 0.915).abs() < 1e-12);
        assert!((at(135.0) - (0.88 + 0.01) / 2.0).abs() < 1e-12);
        assert!((at(180.0) - 0.01).abs() < 1e-12);
        // Table 7.8 January south vertical 0,70 (winter use in utility buildings).
        let utility = ShadingControl::ManualUtilityWithoutGlareProtection;
        assert!(
            (shading_fraction(utility, Orientation::South, 90.0, 1, Balance::Heating) - 0.70).abs()
                < 1e-12
        );
    }

    #[test]
    fn movable_shading_follows_7_42_per_balance() {
        let screen = MovableShading {
            reduction_factor: 0.2,
            device: None,
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
        // Dwellings with manual shading: f_sh;with = 0 on heating.
        assert_eq!(
            movable_shading_factor(Some(&screen), Orientation::South, 90.0, 7, Balance::Heating),
            1.0
        );
        // Automatic shading not tuned per ISO 52016-3 also acts on heating.
        let automatic = MovableShading {
            control: ShadingControl::Automatic,
            ..screen.clone()
        };
        let january = movable_shading_factor(
            Some(&automatic),
            Orientation::South,
            90.0,
            1,
            Balance::Heating,
        );
        assert!((january - (0.14 + 0.86 * 0.2)).abs() < 1e-12);
        let tuned = MovableShading {
            control: ShadingControl::AutomaticResidentialIso52016,
            ..screen
        };
        assert_eq!(
            movable_shading_factor(Some(&tuned), Orientation::South, 90.0, 1, Balance::Heating),
            1.0
        );
        assert_eq!(
            movable_shading_factor(None, Orientation::South, 90.0, 7, Balance::Cooling),
            1.0
        );
    }

    #[test]
    fn control_variants_fit_their_function() {
        assert!(ShadingControl::ManualResidential.fits_function(true));
        assert!(!ShadingControl::ManualResidential.fits_function(false));
        assert!(ShadingControl::ManualUtilityWithoutGlareProtection.fits_function(false));
        assert!(ShadingControl::Automatic.fits_function(true));
        assert!(ShadingControl::Automatic.fits_function(false));
    }

    fn factor(
        obstruction: &Obstruction,
        orientation: Orientation,
        tilt: f64,
        month: u8,
        balance: Balance,
    ) -> f64 {
        obstruction_factor(obstruction, orientation, tilt, month, balance).unwrap()
    }

    #[test]
    fn parallel_obstruction_follows_table_17_7() {
        let low = Obstruction::ParallelObstruction {
            relative_height: 0.3,
        };
        let mid = Obstruction::ParallelObstruction {
            relative_height: 0.5,
        };
        let high = Obstruction::ParallelObstruction {
            relative_height: 1.0,
        };
        // South, February: 0,60 / 0,30 / 0,30; June: 1,00 / 1,00 / 0,56.
        assert_eq!(
            factor(&low, Orientation::South, 90.0, 2, Balance::Heating),
            0.60
        );
        assert_eq!(
            factor(&mid, Orientation::South, 90.0, 2, Balance::Heating),
            0.30
        );
        assert_eq!(
            factor(&high, Orientation::South, 90.0, 6, Balance::Heating),
            0.56
        );
        // Table 17.3: not available for cooling, so situation g without an
        // overhang (table 17.5).
        assert_eq!(
            factor(&high, Orientation::South, 90.0, 6, Balance::Cooling),
            1.0
        );
        // Vertical surfaces only.
        assert!(obstruction_factor(&high, Orientation::South, 45.0, 6, Balance::Heating).is_none());
        assert_eq!(
            validate_obstruction(&high, 45.0),
            vec![("obstruction_situation_requires_vertical", ".method")]
        );
    }

    #[test]
    fn overhang_follows_tables_17_8_and_17_9() {
        let balcony = |relative_height| Obstruction::Overhang { relative_height };
        // Table 17.8 south, January: 0,19 / 0,21 / 0,23; July 0,56 / 0,56 / 0,57.
        assert_eq!(
            factor(&balcony(0.2), Orientation::South, 90.0, 1, Balance::Heating),
            0.19
        );
        assert_eq!(
            factor(&balcony(0.7), Orientation::South, 90.0, 1, Balance::Heating),
            0.21
        );
        assert_eq!(
            factor(&balcony(1.2), Orientation::South, 90.0, 7, Balance::Heating),
            0.57
        );
        // Table 17.9 south: January 0,88 / 1,00; April 0,36 / 0,61 / 1,00;
        // south-west July 0,56 / 0,70.
        assert_eq!(
            factor(&balcony(0.2), Orientation::South, 90.0, 1, Balance::Cooling),
            0.88
        );
        assert_eq!(
            factor(&balcony(0.7), Orientation::South, 90.0, 4, Balance::Cooling),
            0.61
        );
        assert_eq!(
            factor(&balcony(1.0), Orientation::South, 90.0, 4, Balance::Cooling),
            1.0
        );
        assert_eq!(
            factor(
                &balcony(0.7),
                Orientation::SouthWest,
                90.0,
                7,
                Balance::Cooling
            ),
            0.70
        );
    }

    #[test]
    fn side_obstructions_follow_tables_17_10_to_17_12() {
        let fin = |side, relative_width, cooling_height_condition| Obstruction::SideObstruction {
            side,
            relative_width,
            cooling_height_condition,
        };
        // Table 17.10 south-west, January: 0,24 0,25 | 0,49 0,49 | 0,24 0,27.
        let january = |side, width| {
            factor(
                &fin(side, width, false),
                Orientation::SouthWest,
                90.0,
                1,
                Balance::Heating,
            )
        };
        assert_eq!(january(ObstructionSide::Left, 0.5), 0.24);
        assert_eq!(january(ObstructionSide::Left, 1.0), 0.25);
        assert_eq!(january(ObstructionSide::Right, 2.0), 0.49);
        assert_eq!(january(ObstructionSide::Both, 0.9), 0.24);
        assert_eq!(january(ObstructionSide::Both, 1.5), 0.27);
        // Table 17.11 east, July: right b_b < 1 → 0,95; only with the 2,5 m
        // condition, otherwise table 17.5.
        let east = |condition| {
            factor(
                &fin(ObstructionSide::Right, 0.5, condition),
                Orientation::East,
                90.0,
                7,
                Balance::Cooling,
            )
        };
        assert_eq!(east(true), 0.95);
        assert_eq!(east(false), 1.0);
        // Table 17.12 south, June: 0,78 0,91 | 0,78 0,93 | 0,56 0,84; a flat
        // panel uses the south field of view.
        let panel = |side, relative_width, orientation, tilt| {
            collector_obstruction_factor(
                &CollectorObstruction::SideObstruction {
                    side,
                    relative_width,
                },
                orientation,
                tilt,
                6,
            )
            .unwrap()
        };
        assert_eq!(
            panel(ObstructionSide::Right, 1.2, Orientation::South, 90.0),
            0.93
        );
        assert_eq!(
            panel(ObstructionSide::Both, 1.2, Orientation::North, 10.0),
            0.84
        );
        // Footnote c: both sides, b_b < 1 on a tilted panel → table 17.15.
        assert_eq!(
            panel(ObstructionSide::Both, 0.5, Orientation::South, 30.0),
            collector_obstruction_factor(&CollectorObstruction::Full, Orientation::South, 30.0, 6)
                .unwrap()
        );
    }

    #[test]
    fn full_and_other_obstruction_follow_tables_17_13_to_17_15() {
        let full = Obstruction::Full {
            cooling_conditions_met: true,
        };
        // Table 17.13 south, January: 90° 0,19, 30° 0,30, 0° 0,55, 165° 0,82.
        assert_eq!(
            factor(&full, Orientation::South, 90.0, 1, Balance::Heating),
            0.19
        );
        assert_eq!(
            factor(&full, Orientation::South, 32.0, 1, Balance::Heating),
            0.30
        );
        assert_eq!(
            factor(&full, Orientation::South, 165.0, 1, Balance::Heating),
            0.82
        );
        // Below 15° the south field of view applies (17.3.1): north at 10°
        // takes the south 15° column (0,39), not the north one (0,95).
        assert_eq!(
            factor(&full, Orientation::North, 10.0, 1, Balance::Heating),
            0.39
        );
        // Table 17.14 west, July: 90° 0,55, 105° 0,60.
        assert_eq!(
            factor(&full, Orientation::West, 90.0, 7, Balance::Cooling),
            0.55
        );
        assert_eq!(
            factor(&full, Orientation::West, 105.0, 7, Balance::Cooling),
            0.60
        );
        // Without the conditions of e), cooling falls back to table 17.5.
        let conservative = Obstruction::Full {
            cooling_conditions_met: false,
        };
        assert_eq!(
            factor(&conservative, Orientation::West, 90.0, 7, Balance::Cooling),
            1.0
        );
        // Situation g: heating table 17.13; cooling table 17.9 with an
        // overhang, otherwise table 17.5.
        let other = Obstruction::Other {
            overhang_relative_height: Some(0.2),
        };
        assert_eq!(
            factor(&other, Orientation::South, 90.0, 1, Balance::Heating),
            0.19
        );
        assert_eq!(
            factor(&other, Orientation::South, 90.0, 1, Balance::Cooling),
            0.88
        );
        let plain = Obstruction::Other {
            overhang_relative_height: None,
        };
        assert_eq!(
            factor(&plain, Orientation::South, 90.0, 1, Balance::Cooling),
            1.0
        );
        // Table 17.15 south, January, 90°: 0,19.
        assert_eq!(
            collector_obstruction_factor(&CollectorObstruction::Other, Orientation::South, 90.0, 1),
            Some(0.19)
        );
    }

    #[test]
    fn roof_edges_obstruct_only_when_high_and_close() {
        let edge = |height_m, distance_m| {
            collector_obstruction_factor(
                &CollectorObstruction::RoofEdge {
                    height_m,
                    distance_m,
                },
                Orientation::South,
                30.0,
                1,
            )
            .unwrap()
        };
        let table_17_15 =
            collector_obstruction_factor(&CollectorObstruction::Full, Orientation::South, 30.0, 1)
                .unwrap();
        assert_eq!(table_17_15, 0.30);
        assert_eq!(edge(0.5, 0.1), 1.0);
        assert_eq!(edge(0.8, 0.8), 1.0);
        assert_eq!(edge(0.8, 0.5), table_17_15);
        assert_eq!(
            collector_obstruction_factor(
                &CollectorObstruction::Minimal,
                Orientation::North,
                0.0,
                3
            ),
            Some(1.0)
        );
    }
}
