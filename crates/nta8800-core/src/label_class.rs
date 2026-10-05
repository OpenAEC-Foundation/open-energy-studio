//! Indicative energy label class from the primary fossil energy indicator.
//!
//! Omgevingsregeling articles 5.11/5.12, fourth paragraph: the primary
//! fossil energy use is converted into a letter with the table of annex IX
//! (dwellings, residential buildings, lodging outside a lodging building) or
//! annex X (utility buildings, per use function). Source: consolidated text
//! on wetten.overheid.nl, version valid from 2026-01-01 (BWBR0045528),
//! retrieved 2026-10-01; every class bound was checked again on 2026-10-05
//! against the official XML of version 2026-10-01 (unchanged). The official label is issued by the Minister only
//! after registration by a certified adviser with a BRL 9501-attested
//! program; this module therefore returns an indicative class only.

use serde::{Deserialize, Serialize};

pub const LABEL_SOURCE: &str =
    "Omgevingsregeling art. 5.11/5.12 lid 4, bijlagen IX en X (BWBR0045528, versie 2026-10-01)";

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

/// Position of a class letter from best (A+++++ = 0) to worst (G); the
/// utility list contains every residential letter, so ranks compare across
/// both annexes. `None` for an unknown letter.
pub fn class_rank(class: &str) -> Option<usize> {
    UTILITY_CLASSES
        .iter()
        .position(|item| *item == class.trim())
}

/// Inclusive upper bound of `class` for one label function (annex IX or
/// X), `None` for G (no upper bound) or a letter the annex does not use.
pub fn class_upper_bound(function: LabelFunction, class: &str) -> Option<f64> {
    match utility_bounds(function) {
        None => {
            let index = RESIDENTIAL.0.iter().position(|item| *item == class)?;
            RESIDENTIAL.1.get(index).copied()
        }
        Some(bounds) => {
            let index = UTILITY_CLASSES.iter().position(|item| *item == class)?;
            bounds.get(index).copied()
        }
    }
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

/// One use function of a utility building with its usable floor area.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LabelFunctionArea {
    pub function: LabelFunction,
    pub area_m2: f64,
}

fn utility_parts(functions: &[LabelFunctionArea]) -> Option<f64> {
    let total: f64 = functions.iter().map(|part| part.area_m2).sum();
    let valid = !functions.is_empty()
        && total.is_finite()
        && total > 0.0
        && functions.iter().all(|part| {
            part.area_m2.is_finite()
                && part.area_m2 > 0.0
                && part.function != LabelFunction::Residential
        });
    valid.then_some(total)
}

/// NTA 8800 §5.3.1: an existing utility building with several use functions
/// gets label class bounds weighted by usable floor area. Dwellings are
/// never weighted together with utility functions, so a list with
/// `Residential` gives `None`.
pub fn indicative_label_class_mixed(
    functions: &[LabelFunctionArea],
    ep2_kwh_per_m2: f64,
) -> Option<&'static str> {
    if functions.len() == 1 {
        return indicative_label_class(functions[0].function, ep2_kwh_per_m2);
    }
    let total = utility_parts(functions)?;
    if !ep2_kwh_per_m2.is_finite() {
        return None;
    }
    let mut bounds = [0.0; 11];
    for part in functions {
        let own = utility_bounds(part.function)?;
        for (bound, value) in bounds.iter_mut().zip(own) {
            *bound += value * part.area_m2 / total;
        }
    }
    Some(classify(UTILITY_CLASSES, &bounds, ep2_kwh_per_m2))
}

/// NTA 8800 table 5.7 (§5.3.1.2): EwePTot;Renovatiestandaard per use
/// function, kWh/m² per year; `None` for dwellings.
pub fn renovation_standard_limit(function: LabelFunction) -> Option<f64> {
    Some(match function {
        LabelFunction::AssemblyWithDayCare => 110.0,
        LabelFunction::AssemblyWithoutDayCare => 100.0,
        LabelFunction::Cell => 180.0,
        LabelFunction::HealthcareWithBeds => 270.0,
        LabelFunction::HealthcareWithoutBeds => 90.0,
        LabelFunction::Office => 80.0,
        LabelFunction::Lodging => 150.0,
        LabelFunction::Education => 100.0,
        LabelFunction::Sport => 105.0,
        LabelFunction::Retail => 120.0,
        LabelFunction::Residential => return None,
    })
}

/// §5.3.1.2: the renovatiestandaard of a utility building, weighted by
/// usable floor area over table 5.7 and rounded to two decimals.
pub fn renovation_standard(functions: &[LabelFunctionArea]) -> Option<f64> {
    let total = utility_parts(functions)?;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal::RoundingStrategy;
    // Decimal arithmetic so that x,xx5 rounds up.
    let total = crate::final_energy_draft::decimal(total)?;
    let mut weighted = rust_decimal::Decimal::ZERO;
    for part in functions {
        weighted += crate::final_energy_draft::decimal(renovation_standard_limit(part.function)?)?
            * crate::final_energy_draft::decimal(part.area_m2)?
            / total;
    }
    weighted
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
        .to_f64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_utility_bounds_and_renovation_standard() {
        let parts = [
            LabelFunctionArea {
                function: LabelFunction::Office,
                area_m2: 1000.0,
            },
            LabelFunctionArea {
                function: LabelFunction::Retail,
                area_m2: 500.0,
            },
        ];
        // A+ bound: (1000·160 + 500·240)/1500 = 186,67.
        assert_eq!(indicative_label_class_mixed(&parts, 186.0), Some("A+"));
        assert_eq!(indicative_label_class_mixed(&parts, 186.7), Some("A"));
        // Table 5.7: (1000·80 + 500·120)/1500 = 93,33.
        assert_eq!(renovation_standard(&parts), Some(93.33));
        let with_dwelling = [
            parts[0],
            LabelFunctionArea {
                function: LabelFunction::Residential,
                area_m2: 100.0,
            },
        ];
        assert_eq!(indicative_label_class_mixed(&with_dwelling, 100.0), None);
        assert_eq!(renovation_standard(&with_dwelling), None);
        // One function falls back to its own column.
        assert_eq!(indicative_label_class_mixed(&parts[..1], 300.01), Some("G"));
    }

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
