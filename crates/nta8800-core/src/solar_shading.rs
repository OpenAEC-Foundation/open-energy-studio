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
//! Table values were generated from the licensed norm text and checked
//! against the rendered pages by the 2026-10-02 review.

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

/// Table 17.4 with the nearest-column rule; `None` outside 0–180°.
fn obstruction_lookup(orientation: Orientation, tilt_deg: f64, month: u8) -> Option<f64> {
    if !(0.0..=180.0).contains(&tilt_deg) {
        return None;
    }
    let row = &OBSTRUCTION_HEATING[column(orientation)][usize::from(month - 1)];
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
            Balance::Heating => obstruction_lookup(orientation, tilt_deg, month),
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
    if balance == Balance::Heating && control.off_for_heating() {
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
            (1.0 - fraction) + fraction * shading.reduction_factor
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
}
