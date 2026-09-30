//! Reference climate De Bilt, NTA 8800:2025+C1:2026 chapter 17.
//!
//! Values are transcribed from tables 17.1 (month length, outdoor
//! temperature) and 17.2 (mean solar irradiance in W/m² for tilt 0° and 90°).
//! Transcription source: Open Heatloss Studio `nta8800-tables::climate::de_bilt`
//! (MIT, same organisation), which cites PDF pages 689–694. A second,
//! independent review against the licensed norm text is still required.

use serde::{Deserialize, Serialize};

pub const CLIMATE_SOURCE: &str =
    "NTA 8800:2025+C1:2026 tables 17.1/17.2 (transcribed via Open Heatloss Studio; review pending)";

/// Table 17.1, `t_mi` in hours; sums to 8760 h.
pub const MONTH_HOURS: [f64; 12] = [
    744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0,
];

/// Table 17.1, `θ_e;avg;mi` in °C.
pub const OUTDOOR_TEMPERATURE_C: [f64; 12] = [
    2.61, 4.82, 5.91, 9.32, 14.73, 16.12, 18.05, 18.48, 15.63, 10.40, 7.99, 4.00,
];

/// Table 17.2, tilt 0° (horizontal), mean irradiance in W/m².
const HORIZONTAL_W_PER_M2: [f64; 12] = [
    28.0, 49.3, 96.6, 160.5, 197.0, 209.3, 191.0, 177.2, 123.9, 73.2, 34.3, 21.0,
];

/// Table 17.2, tilt 90° (vertical), mean irradiance in W/m² per orientation.
const VERTICAL_W_PER_M2: [(Orientation, [f64; 12]); 8] = [
    (
        Orientation::North,
        [
            11.1, 19.5, 34.8, 49.4, 61.9, 73.0, 66.7, 55.9, 41.4, 26.4, 13.6, 8.9,
        ],
    ),
    (
        Orientation::NorthEast,
        [
            11.1, 21.5, 44.2, 72.9, 82.9, 92.0, 81.2, 63.9, 47.9, 29.1, 14.0, 8.9,
        ],
    ),
    (
        Orientation::East,
        [
            20.2, 36.5, 70.7, 112.2, 114.6, 114.8, 104.9, 89.0, 73.7, 49.8, 23.9, 15.9,
        ],
    ),
    (
        Orientation::SouthEast,
        [
            43.9, 56.8, 95.4, 135.8, 128.4, 118.0, 113.2, 112.4, 103.6, 80.3, 47.1, 35.8,
        ],
    ),
    (
        Orientation::South,
        [
            60.1, 66.7, 101.8, 135.1, 124.9, 112.7, 109.7, 128.5, 122.3, 96.2, 59.5, 46.2,
        ],
    ),
    (
        Orientation::SouthWest,
        [
            48.1, 52.2, 82.1, 121.9, 122.1, 127.8, 117.1, 137.1, 112.2, 76.3, 45.6, 34.9,
        ],
    ),
    (
        Orientation::West,
        [
            23.4, 32.8, 57.3, 96.2, 107.3, 125.7, 112.7, 120.0, 83.9, 46.7, 22.7, 15.2,
        ],
    ),
    (
        Orientation::NorthWest,
        [
            11.4, 20.9, 38.5, 64.1, 78.9, 97.8, 88.5, 83.1, 53.6, 28.7, 13.8, 8.9,
        ],
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

/// Mean irradiance `I_sol;mi` in W/m² for a surface with the given tilt.
///
/// Only the 0° and 90° columns of table 17.2 are transcribed. Other tilts
/// return `None` rather than an interpolated value; orientation is ignored
/// for a horizontal surface.
pub fn irradiance_w_per_m2(orientation: Orientation, tilt_deg: f64, month: u8) -> Option<f64> {
    let index = usize::from(month.checked_sub(1)?);
    if index >= 12 {
        return None;
    }
    if tilt_deg == 0.0 {
        return Some(HORIZONTAL_W_PER_M2[index]);
    }
    if tilt_deg == 90.0 {
        return VERTICAL_W_PER_M2
            .iter()
            .find(|(item, _)| *item == orientation)
            .map(|(_, values)| values[index]);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_hours_sum_to_a_year() {
        assert_eq!(MONTH_HOURS.iter().sum::<f64>(), 8760.0);
    }

    #[test]
    fn looks_up_only_transcribed_tilts() {
        assert_eq!(
            irradiance_w_per_m2(Orientation::South, 90.0, 7),
            Some(109.7)
        );
        assert_eq!(irradiance_w_per_m2(Orientation::North, 0.0, 1), Some(28.0));
        assert_eq!(irradiance_w_per_m2(Orientation::South, 45.0, 7), None);
        assert_eq!(irradiance_w_per_m2(Orientation::South, 90.0, 13), None);
        assert_eq!(irradiance_w_per_m2(Orientation::South, 90.0, 0), None);
    }
}
