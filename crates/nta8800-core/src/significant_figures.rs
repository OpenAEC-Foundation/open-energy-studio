//! Class rounding to two significant figures, NTA 8800:2025+C1:2026
//! annex X (p. 1129).
//!
//! Rounding up takes the next higher value of table X.1 for the first two
//! significant digits, also when the digits already equal a table value
//! (33 → 34, 3 400 → 3 600). Rounding down takes the table value itself or
//! the next lower one. Zero is never rounded.

/// Table X.1.
const CLASSES: [u32; 33] = [
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 44, 48, 52,
    56, 60, 65, 70, 75, 80, 85, 90, 95,
];

/// Split a positive value into its first two significant digits (10–99)
/// and the power of ten of the second digit.
fn leading(value: f64) -> (u32, i32) {
    let mut exponent = value.log10().floor() as i32 - 1;
    let mut digits = (value / 10f64.powi(exponent) + 1e-9).floor();
    // Guard against floating-point edges such as 99.99999 → 100.
    if digits >= 100.0 {
        exponent += 1;
        digits = (value / 10f64.powi(exponent) + 1e-9).floor();
    } else if digits < 10.0 {
        exponent -= 1;
        digits = (value / 10f64.powi(exponent) + 1e-9).floor();
    }
    (digits as u32, exponent)
}

fn compose(digits: u32, exponent: i32) -> f64 {
    // Round to the decimal grid to avoid 0.1 + 0.2 artefacts.
    let raw = f64::from(digits) * 10f64.powi(exponent);
    if exponent < 0 {
        let scale = 10f64.powi(-exponent);
        (raw * scale).round() / scale
    } else {
        raw
    }
}

/// Annex X: round up to two significant figures.
pub fn round_up(value: f64) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    if value < 0.0 {
        return -round_down(-value);
    }
    let (digits, exponent) = leading(value);
    match CLASSES.iter().find(|class| **class > digits) {
        Some(class) => compose(*class, exponent),
        None => compose(10, exponent + 1),
    }
}

/// Annex X: round down to two significant figures.
pub fn round_down(value: f64) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    if value < 0.0 {
        return -round_up(-value);
    }
    let (digits, exponent) = leading(value);
    let class = CLASSES
        .iter()
        .rev()
        .find(|class| **class <= digits)
        .copied()
        .unwrap_or(10);
    compose(class, exponent)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() <= 1e-9 * b.abs().max(1.0), "{a} vs {b}");
    }

    #[test]
    fn rounding_up_matches_the_annex_x_examples() {
        for (value, expected) in [
            (0.0, 0.0),
            (0.09, 0.095),
            (0.14, 0.15),
            (0.33, 0.34),
            (3.3, 3.4),
            (33.0, 34.0),
            (332.0, 340.0),
            (3327.0, 3400.0),
            (3400.0, 3600.0),
            (3450.0, 3600.0),
            (0.82, 0.85),
            (8.2, 8.5),
            (82.0, 85.0),
            (822.0, 850.0),
            (8227.0, 8500.0),
            (97.0, 100.0),
        ] {
            close(round_up(value), expected);
        }
    }

    #[test]
    fn rounding_down_matches_the_annex_x_examples() {
        for (value, expected) in [
            (0.0, 0.0),
            (0.09, 0.09),
            (0.14, 0.14),
            (0.33, 0.32),
            (3.3, 3.2),
            (33.0, 32.0),
            (332.0, 320.0),
            (3327.0, 3200.0),
            (3400.0, 3400.0),
            (3450.0, 3400.0),
            (0.47, 0.44),
        ] {
            close(round_down(value), expected);
        }
    }
}
