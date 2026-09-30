//! Reference climate De Bilt, NTA 8800:2025+C1:2026 chapter 17.
//!
//! Tables 17.1 (month length, outdoor temperature) and 17.2 (mean total
//! irradiance in W/m² for tilts 0°, 30°, 45°, 60°, 90°, 135° and 180° and
//! eight orientations, ground reflection 0,2). Selection rules from the norm
//! (p. 693): an intermediate orientation takes the nearest column, the
//! higher value on an exact tie; an intermediate tilt is interpolated
//! linearly.
//!
//! Transcription source: Open Heatloss Studio `nta8800-tables::climate::de_bilt`
//! and `nta8800-pv::tables::irradiation` (MIT, same organisation), which cite
//! PDF pages 689–694 with spot checks. A second independent review against
//! the licensed norm text is still required.

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

/// Tilts of table 17.2 in degrees (0 = facing up, 90 = vertical, 180 = facing down).
const TILTS_DEG: [f64; 7] = [0.0, 30.0, 45.0, 60.0, 90.0, 135.0, 180.0];

/// Column azimuths of table 17.2 (0/360 = north, clockwise): S, SW, W, NW, N, NE, E, SE.
const COLUMN_AZIMUTH_DEG: [f64; 8] = [180.0, 225.0, 270.0, 315.0, 360.0, 45.0, 90.0, 135.0];

/// Tilt 0°, orientation independent.
const HORIZONTAL: [f64; 12] = [
    28.0, 49.3, 96.6, 160.5, 197.0, 209.3, 191.0, 177.2, 123.9, 73.2, 34.3, 21.0,
];

const DOWN: [f64; 12] = [
    5.6, 9.8, 19.3, 32.1, 39.3, 41.8, 38.2, 35.3, 24.7, 14.6, 6.9, 4.2,
];

const ISOL_30: [[f64; 12]; 8] = [
    [
        50.5, 69.1, 122.5, 189.5, 211.1, 211.2, 196.1, 197.9, 154.0, 102.4, 54.8, 38.3,
    ],
    [
        44.4, 61.2, 109.3, 174.5, 201.5, 210.7, 193.2, 198.3, 146.2, 91.5, 47.7, 32.6,
    ],
    [
        29.0, 46.2, 87.7, 146.5, 179.9, 199.4, 180.2, 178.4, 121.1, 68.8, 32.9, 20.6,
    ],
    [
        16.2, 32.9, 66.7, 115.6, 155.8, 180.6, 162.1, 147.6, 91.6, 47.3, 20.5, 12.5,
    ],
    [
        14.9, 27.2, 56.4, 104.6, 148.5, 171.0, 153.0, 125.8, 73.7, 36.3, 18.6, 12.2,
    ],
    [
        15.8, 34.5, 72.8, 125.1, 160.6, 173.0, 156.9, 127.5, 86.5, 48.9, 20.9, 12.5,
    ],
    [
        26.9, 49.4, 97.6, 158.9, 186.3, 189.7, 175.0, 152.8, 113.7, 71.6, 33.8, 21.2,
    ],
    [
        42.2, 63.7, 117.7, 184.1, 206.3, 204.4, 190.0, 179.3, 140.1, 93.6, 48.6, 33.1,
    ],
];

const ISOL_45: [[f64; 12]; 8] = [
    [
        57.9, 74.1, 126.6, 189.7, 202.7, 197.3, 185.0, 193.5, 157.6, 109.4, 61.0, 44.1,
    ],
    [
        49.4, 63.2, 109.1, 171.0, 191.1, 199.3, 182.5, 194.9, 147.0, 94.2, 51.1, 36.1,
    ],
    [
        28.7, 44.0, 82.0, 136.7, 164.4, 186.2, 166.8, 169.8, 115.3, 64.8, 31.3, 19.9,
    ],
    [
        14.9, 29.2, 56.6, 96.5, 128.7, 156.3, 139.0, 127.2, 78.0, 40.2, 18.5, 11.7,
    ],
    [
        14.3, 25.9, 44.3, 70.0, 113.6, 139.6, 123.5, 91.5, 52.9, 33.5, 17.8, 11.7,
    ],
    [
        14.5, 30.4, 63.1, 107.1, 134.5, 145.9, 132.7, 102.9, 72.2, 41.4, 18.8, 11.7,
    ],
    [
        26.2, 47.9, 94.2, 152.2, 172.0, 173.3, 160.4, 137.9, 106.2, 68.4, 32.4, 20.5,
    ],
    [
        46.3, 66.5, 120.2, 183.5, 197.3, 190.7, 179.1, 171.0, 139.2, 97.2, 52.2, 36.7,
    ],
];

const ISOL_60: [[f64; 12]; 8] = [
    [
        62.2, 75.4, 124.3, 180.2, 184.5, 175.1, 165.9, 179.7, 153.3, 110.7, 63.9, 47.4,
    ],
    [
        51.8, 62.1, 103.9, 160.4, 173.4, 180.9, 165.4, 182.9, 141.5, 92.6, 51.8, 37.6,
    ],
    [
        27.8, 41.1, 74.8, 125.1, 146.3, 169.1, 150.6, 156.9, 107.2, 59.9, 28.9, 19.0,
    ],
    [
        13.8, 26.4, 49.6, 83.1, 107.5, 134.1, 119.2, 110.2, 68.6, 35.9, 17.0, 10.9,
    ],
    [
        13.4, 24.1, 41.5, 57.8, 78.5, 102.9, 90.4, 68.0, 48.6, 31.5, 16.6, 10.9,
    ],
    [
        13.5, 27.3, 56.3, 93.9, 113.2, 123.3, 112.3, 85.8, 62.3, 36.6, 17.3, 10.9,
    ],
    [
        24.7, 45.4, 88.5, 142.0, 154.7, 154.5, 143.2, 122.0, 97.2, 63.5, 30.4, 19.6,
    ],
    [
        48.1, 66.3, 116.9, 174.2, 179.9, 170.7, 161.8, 156.4, 132.6, 96.0, 53.2, 38.4,
    ],
];

const ISOL_90: [[f64; 12]; 8] = [
    [
        60.1, 66.7, 101.8, 135.1, 124.9, 112.7, 109.7, 128.5, 122.3, 96.2, 59.5, 46.2,
    ],
    [
        48.1, 52.2, 82.1, 121.9, 122.1, 127.8, 117.1, 137.1, 112.2, 76.3, 45.6, 34.9,
    ],
    [
        23.4, 32.8, 57.3, 96.2, 107.3, 125.7, 112.7, 120.0, 83.9, 46.7, 22.7, 15.2,
    ],
    [
        11.4, 20.9, 38.5, 64.1, 78.9, 97.8, 88.5, 83.1, 53.6, 28.7, 13.8, 8.9,
    ],
    [
        11.1, 19.5, 34.8, 49.4, 61.9, 73.0, 66.7, 55.9, 41.4, 26.4, 13.6, 8.9,
    ],
    [
        11.1, 21.5, 44.2, 72.9, 82.9, 92.0, 81.2, 63.9, 47.9, 29.1, 14.0, 8.9,
    ],
    [
        20.2, 36.5, 70.7, 112.2, 114.6, 114.8, 104.9, 89.0, 73.7, 49.8, 23.9, 15.9,
    ],
    [
        43.9, 56.8, 95.4, 135.8, 128.4, 118.0, 113.2, 112.4, 103.6, 80.3, 47.1, 35.8,
    ],
];

const ISOL_135: [[f64; 12]; 8] = [
    [
        33.4, 31.5, 37.3, 39.0, 45.5, 48.3, 44.9, 41.6, 40.2, 41.6, 30.9, 26.3,
    ],
    [
        25.1, 24.2, 35.1, 50.7, 50.4, 52.3, 49.7, 54.3, 47.5, 33.2, 21.7, 18.3,
    ],
    [
        12.7, 17.3, 29.9, 49.9, 55.2, 62.4, 57.7, 59.6, 43.2, 24.7, 12.0, 8.1,
    ],
    [
        7.6, 13.2, 25.2, 41.8, 50.7, 57.8, 53.5, 50.2, 33.8, 19.3, 9.2, 5.8,
    ],
    [
        7.5, 12.9, 24.5, 38.3, 46.7, 50.6, 46.5, 42.1, 30.4, 18.6, 9.1, 5.8,
    ],
    [
        7.5, 13.5, 27.6, 45.5, 51.9, 55.4, 48.9, 42.9, 31.4, 19.4, 9.3, 5.8,
    ],
    [
        10.6, 18.6, 36.7, 57.1, 57.8, 59.9, 53.0, 47.7, 37.9, 25.4, 12.7, 8.4,
    ],
    [
        22.2, 26.7, 42.0, 56.3, 51.9, 51.7, 48.0, 47.5, 43.2, 35.2, 22.7, 19.0,
    ],
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

impl Orientation {
    pub fn azimuth_deg(self) -> f64 {
        match self {
            Self::North => 0.0,
            Self::NorthEast => 45.0,
            Self::East => 90.0,
            Self::SouthEast => 135.0,
            Self::South => 180.0,
            Self::SouthWest => 225.0,
            Self::West => 270.0,
            Self::NorthWest => 315.0,
        }
    }
}

fn angular_distance(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

fn level(tilt_index: usize, azimuth_deg: f64, month_index: usize) -> f64 {
    let matrix = match tilt_index {
        0 => return HORIZONTAL[month_index],
        6 => return DOWN[month_index],
        1 => &ISOL_30,
        2 => &ISOL_45,
        3 => &ISOL_60,
        4 => &ISOL_90,
        _ => &ISOL_135,
    };
    let nearest = COLUMN_AZIMUTH_DEG
        .iter()
        .map(|column| angular_distance(azimuth_deg, *column))
        .fold(f64::INFINITY, f64::min);
    COLUMN_AZIMUTH_DEG
        .iter()
        .enumerate()
        .filter(|(_, column)| (angular_distance(azimuth_deg, **column) - nearest).abs() < 1e-9)
        .map(|(index, _)| matrix[index][month_index])
        .fold(f64::NEG_INFINITY, f64::max)
}

/// Mean irradiance `I_sol;mi` in W/m² for azimuth (0 = north, clockwise)
/// and tilt in `[0°, 180°]`; `None` outside that range or for an invalid month.
pub fn irradiance_at(azimuth_deg: f64, tilt_deg: f64, month: u8) -> Option<f64> {
    let month_index = usize::from(month.checked_sub(1)?);
    if month_index >= 12
        || !azimuth_deg.is_finite()
        || !tilt_deg.is_finite()
        || !(0.0..=180.0).contains(&tilt_deg)
    {
        return None;
    }
    let upper = TILTS_DEG
        .iter()
        .position(|tilt| *tilt >= tilt_deg)
        .expect("tilt within table range");
    if upper == 0 {
        return Some(level(0, azimuth_deg, month_index));
    }
    let lower = upper - 1;
    let fraction = (tilt_deg - TILTS_DEG[lower]) / (TILTS_DEG[upper] - TILTS_DEG[lower]);
    let low = level(lower, azimuth_deg, month_index);
    let high = level(upper, azimuth_deg, month_index);
    Some(low + fraction * (high - low))
}

/// Convenience wrapper for the eight table orientations.
pub fn irradiance_w_per_m2(orientation: Orientation, tilt_deg: f64, month: u8) -> Option<f64> {
    irradiance_at(orientation.azimuth_deg(), tilt_deg, month)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_hours_sum_to_a_year() {
        assert_eq!(MONTH_HOURS.iter().sum::<f64>(), 8760.0);
    }

    #[test]
    fn spot_checks_from_the_transcription_analysis() {
        // Values listed in the Open Heatloss Studio F3d-4 spot check.
        assert_eq!(irradiance_w_per_m2(Orientation::North, 0.0, 1), Some(28.0));
        assert_eq!(
            irradiance_w_per_m2(Orientation::South, 30.0, 6),
            Some(211.2)
        );
        assert_eq!(irradiance_w_per_m2(Orientation::West, 30.0, 7), Some(180.2));
        assert_eq!(
            irradiance_w_per_m2(Orientation::South, 45.0, 4),
            Some(189.7)
        );
        assert_eq!(irradiance_w_per_m2(Orientation::South, 90.0, 1), Some(60.1));
        assert_eq!(irradiance_w_per_m2(Orientation::North, 90.0, 6), Some(73.0));
        assert_eq!(
            irradiance_w_per_m2(Orientation::SouthEast, 135.0, 12),
            Some(19.0)
        );
        assert_eq!(
            irradiance_w_per_m2(Orientation::North, 180.0, 12),
            Some(4.2)
        );
    }

    #[test]
    fn selection_and_interpolation_rules() {
        // Tilt 15° lies halfway between 0° and 30°.
        let expected = (28.0 + 50.5) / 2.0;
        assert!(
            (irradiance_w_per_m2(Orientation::South, 15.0, 1).unwrap() - expected).abs() < 1e-12
        );
        // Azimuth 200° is nearest to south (180°).
        assert_eq!(irradiance_at(200.0, 90.0, 1), Some(60.1));
        // Azimuth 202,5° is exactly between S (60,1) and SW (48,1): higher value.
        assert_eq!(irradiance_at(202.5, 90.0, 1), Some(60.1));
        // Exactly between N (11,1) and NE (11,1) in January, and NE/E in June.
        assert_eq!(irradiance_at(67.5, 90.0, 6), Some(114.8));
        assert_eq!(irradiance_at(0.0, 181.0, 1), None);
        assert_eq!(irradiance_at(0.0, -1.0, 1), None);
        assert_eq!(irradiance_at(0.0, 90.0, 13), None);
    }
}
