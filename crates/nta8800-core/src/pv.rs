//! Photovoltaic electricity yield, NTA 8800 chapter 16, formulas 16.2 and
//! 16.3:
//!
//! `E_sol;mi = I_sol;mi · t_mi · F_sh;obst;mi / 1000` [kWh/m²]
//! `E_el;PV;out;mi = E_sol;mi · P_pk · f_perf · c_sh;PV · f_prac;PV / I_ref`
//!
//! with `f_prac;PV = 0,95` and `I_ref = 1 kW/m²`. Tilt and orientation enter
//! only through `I_sol;mi` of table 17.2. Transcription source: Open Heatloss
//! Studio analysis F3d-4 (2026-07-12). Tables 16.1–16.3 are not transcribed
//! as values; `P_pk`, `f_perf` and `c_sh;PV` are supplied with a source and
//! checked against the ranges named in that analysis.

use crate::climate::{irradiance_at, MONTH_HOURS};
use serde::{Deserialize, Serialize};

/// 16.2: practical correction.
pub const F_PRAC_PV: f64 = 0.95;
/// 16.2: reference irradiance in kW/m².
pub const I_REF: f64 = 1.0;
/// Table 16.2 values of `f_perf` named in the transcription analysis.
pub const PERFORMANCE_FACTORS: [f64; 3] = [0.76, 0.80, 0.82];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PvSystem {
    pub id: String,
    /// `P_pk` in kW.
    pub peak_power_kw: f64,
    /// Azimuth, 0 = north, clockwise.
    pub azimuth_deg: f64,
    /// 0 = horizontal, 90 = vertical.
    pub tilt_deg: f64,
    /// Table 16.2.
    pub performance_factor: f64,
    /// Table 16.3 `c_sh;PV`.
    pub shading_correction: f64,
    /// `F_sh;obst` in 16.3, constant over the year.
    pub obstruction_factor: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PvIssue {
    pub code: &'static str,
    pub path: String,
}

pub fn validate_pv(system: &PvSystem, path: &str) -> Vec<PvIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: &str| {
        issues.push(PvIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    if system.id.trim().is_empty() {
        push("id_invalid", "id");
    }
    if !system.peak_power_kw.is_finite() || system.peak_power_kw <= 0.0 {
        push("pv_peak_power_invalid", "peakPowerKw");
    }
    if !system.azimuth_deg.is_finite() {
        push("pv_azimuth_invalid", "azimuthDeg");
    }
    if !(0.0..=90.0).contains(&system.tilt_deg) {
        push("pv_tilt_invalid", "tiltDeg");
    }
    if !PERFORMANCE_FACTORS.contains(&system.performance_factor) {
        push("pv_performance_factor_invalid", "performanceFactor");
    }
    if !(0.75..=1.0).contains(&system.shading_correction) {
        push("pv_shading_correction_invalid", "shadingCorrection");
    }
    if !(0.0..=1.0).contains(&system.obstruction_factor) {
        push("pv_obstruction_factor_invalid", "obstructionFactor");
    }
    if system.source_reference.trim().is_empty() {
        push("source_reference_required", "sourceReference");
    }
    issues
}

/// Monthly yield in kWh; call only after [`validate_pv`] returned no issues.
pub fn monthly_yield_kwh(system: &PvSystem) -> [f64; 12] {
    let mut values = [0.0; 12];
    for (index, value) in values.iter_mut().enumerate() {
        let month = index as u8 + 1;
        let irradiance = irradiance_at(system.azimuth_deg, system.tilt_deg, month)
            .expect("validated tilt and azimuth");
        // 16.3
        let solar = irradiance * MONTH_HOURS[index] * system.obstruction_factor / 1000.0;
        // 16.2
        *value = solar
            * system.peak_power_kw
            * system.performance_factor
            * system.shading_correction
            * F_PRAC_PV
            / I_REF;
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system() -> PvSystem {
        PvSystem {
            id: "pv".into(),
            peak_power_kw: 4.0,
            azimuth_deg: 180.0,
            tilt_deg: 30.0,
            performance_factor: 0.80,
            shading_correction: 1.0,
            obstruction_factor: 1.0,
            source_reference: "datasheet".into(),
        }
    }

    #[test]
    fn june_south_30_degrees_matches_hand_calculation() {
        let yields = monthly_yield_kwh(&system());
        let expected = 211.2 * 720.0 / 1000.0 * 4.0 * 0.80 * 0.95;
        assert!((yields[5] - expected).abs() < 1e-9);
        let annual: f64 = yields.iter().sum();
        // About 900 kWh per kWp for an ideal south roof at 30°.
        assert!((850.0..1000.0).contains(&(annual / 4.0)));
    }

    #[test]
    fn west_and_north_roofs_yield_less_but_not_zero() {
        let south: f64 = monthly_yield_kwh(&system()).iter().sum();
        let mut west = system();
        west.azimuth_deg = 270.0;
        let mut north = system();
        north.azimuth_deg = 0.0;
        let west: f64 = monthly_yield_kwh(&west).iter().sum();
        let north: f64 = monthly_yield_kwh(&north).iter().sum();
        assert!(south > west && west > north && north > 0.0);
    }

    #[test]
    fn validation_rejects_values_outside_tables() {
        let mut bad = system();
        bad.performance_factor = 0.9;
        bad.shading_correction = 0.5;
        bad.tilt_deg = 120.0;
        bad.source_reference = " ".into();
        let codes: Vec<_> = validate_pv(&bad, "pv[0]")
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "pv_performance_factor_invalid",
            "pv_shading_correction_invalid",
            "pv_tilt_invalid",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
        assert!(validate_pv(&system(), "pv[0]").is_empty());
    }
}
