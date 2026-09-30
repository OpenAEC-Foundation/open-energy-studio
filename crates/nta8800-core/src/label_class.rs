//! Indicative energy label class from the primary fossil energy indicator.
//!
//! Omgevingsregeling articles 5.11/5.12, fourth paragraph: the primary
//! fossil energy use is converted into a letter with the table of annex IX
//! (dwellings, residential buildings, lodging outside a lodging building) or
//! annex X (utility buildings, per use function). Source: consolidated text
//! on wetten.overheid.nl, version valid from 2026-01-01 (BWBR0045528),
//! retrieved 2026-10-01. The official label is issued by the Minister only
//! after registration by a certified adviser with a BRL 9501-attested
//! program; this module therefore returns an indicative class only.

use serde::{Deserialize, Serialize};

pub const LABEL_SOURCE: &str =
    "Omgevingsregeling art. 5.11/5.12 lid 4, bijlagen IX en X (BWBR0045528, versie 2026-01-01)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelFunction {
    Residential,
    Office,
    AssemblyWithoutDayCare,
    AssemblyWithDayCare,
    Education,
    HealthcareWithoutBeds,
    HealthcareWithBeds,
    Retail,
    Sport,
    Lodging,
    Cell,
}

/// Annex IX, upper bounds (inclusive, kWh/m²·yr) for A++++ … F; above F is G.
const RESIDENTIAL: (&[&str], [f64; 10]) = (
    &[
        "A++++", "A+++", "A++", "A+", "A", "B", "C", "D", "E", "F", "G",
    ],
    [
        0.0, 50.0, 75.0, 105.0, 160.0, 190.0, 250.0, 290.0, 335.0, 380.0,
    ],
);

const UTILITY_CLASSES: &[&str] = &[
    "A+++++", "A++++", "A+++", "A++", "A+", "A", "B", "C", "D", "E", "F", "G",
];

/// Annex X tables 1 and 2, upper bounds for A+++++ … F per use function.
fn utility_bounds(function: LabelFunction) -> Option<[f64; 11]> {
    Some(match function {
        LabelFunction::Office => [
            0.0, 40.0, 80.0, 120.0, 160.0, 180.0, 200.0, 225.0, 250.0, 275.0, 300.0,
        ],
        LabelFunction::AssemblyWithoutDayCare => [
            0.0, 50.0, 100.0, 150.0, 200.0, 230.0, 255.0, 285.0, 320.0, 355.0, 385.0,
        ],
        LabelFunction::AssemblyWithDayCare => [
            0.0, 55.0, 110.0, 165.0, 220.0, 265.0, 290.0, 330.0, 365.0, 405.0, 445.0,
        ],
        LabelFunction::Education => [
            0.0, 50.0, 100.0, 150.0, 200.0, 235.0, 260.0, 295.0, 330.0, 360.0, 395.0,
        ],
        LabelFunction::HealthcareWithoutBeds => [
            0.0, 45.0, 90.0, 135.0, 180.0, 210.0, 230.0, 260.0, 295.0, 325.0, 355.0,
        ],
        LabelFunction::HealthcareWithBeds => [
            0.0, 90.0, 180.0, 270.0, 360.0, 430.0, 470.0, 530.0, 595.0, 655.0, 715.0,
        ],
        LabelFunction::Retail => [
            0.0, 60.0, 120.0, 180.0, 240.0, 285.0, 315.0, 355.0, 395.0, 435.0, 475.0,
        ],
        LabelFunction::Sport => [
            0.0, 35.0, 70.0, 105.0, 140.0, 155.0, 170.0, 195.0, 215.0, 240.0, 260.0,
        ],
        LabelFunction::Lodging => [
            0.0, 50.0, 100.0, 150.0, 200.0, 230.0, 255.0, 285.0, 320.0, 355.0, 385.0,
        ],
        LabelFunction::Cell => [
            0.0, 60.0, 120.0, 180.0, 240.0, 300.0, 330.0, 370.0, 415.0, 455.0, 500.0,
        ],
        LabelFunction::Residential => return None,
    })
}

fn classify(classes: &[&'static str], bounds: &[f64], value: f64) -> &'static str {
    bounds
        .iter()
        .position(|bound| value <= *bound)
        .map_or(classes[classes.len() - 1], |index| classes[index])
}

/// Indicative class for an EP2 value already rounded to 0,01 kWh/m²·yr.
/// Returns `None` for a non-finite value.
pub fn indicative_label_class(
    function: LabelFunction,
    ep2_kwh_per_m2: f64,
) -> Option<&'static str> {
    if !ep2_kwh_per_m2.is_finite() {
        return None;
    }
    Some(match utility_bounds(function) {
        None => classify(RESIDENTIAL.0, &RESIDENTIAL.1, ep2_kwh_per_m2),
        Some(bounds) => classify(UTILITY_CLASSES, &bounds, ep2_kwh_per_m2),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn residential_annex_ix_boundaries() {
        let class = |value| indicative_label_class(LabelFunction::Residential, value).unwrap();
        assert_eq!(class(-5.0), "A++++");
        assert_eq!(class(0.0), "A++++");
        assert_eq!(class(0.01), "A+++");
        assert_eq!(class(50.0), "A+++");
        assert_eq!(class(50.01), "A++");
        assert_eq!(class(105.0), "A+");
        assert_eq!(class(160.0), "A");
        assert_eq!(class(190.0), "B");
        assert_eq!(class(250.0), "C");
        assert_eq!(class(290.0), "D");
        assert_eq!(class(335.0), "E");
        assert_eq!(class(380.0), "F");
        assert_eq!(class(380.01), "G");
        assert_eq!(
            indicative_label_class(LabelFunction::Residential, f64::NAN),
            None
        );
    }

    #[test]
    fn utility_annex_x_columns() {
        assert_eq!(
            indicative_label_class(LabelFunction::Office, 0.0),
            Some("A+++++")
        );
        assert_eq!(
            indicative_label_class(LabelFunction::Office, 40.0),
            Some("A++++")
        );
        assert_eq!(
            indicative_label_class(LabelFunction::Office, 300.0),
            Some("F")
        );
        assert_eq!(
            indicative_label_class(LabelFunction::Office, 300.01),
            Some("G")
        );
        assert_eq!(
            indicative_label_class(LabelFunction::HealthcareWithBeds, 715.0),
            Some("F")
        );
        assert_eq!(
            indicative_label_class(LabelFunction::Sport, 155.0),
            Some("A")
        );
        assert_eq!(
            indicative_label_class(LabelFunction::Cell, 300.01),
            Some("B")
        );
    }
}
