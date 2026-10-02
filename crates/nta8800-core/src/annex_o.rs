//! Auxiliary electricity of individual central-heating appliances from
//! component measurements, NTA 8800:2025+C1:2026 annex O (pp. 923–931,
//! based on NEN 7120 annex C) with 9.85–9.90 (pp. 359–361).
//!
//! The kernel processes the measured data of O.2.2: straight-line fits of
//! the fan and pump power against modulation (O.2/O.3, least squares), the
//! mean on-time and mean load from the load curve after ignition (O.1.5,
//! O.2.3.4: product of mean load and on-time equal to 120 s at full load,
//! 1 800 s for bivalent appliances), and from those the constants A, B and C
//! (9.86–9.88) used monthly in 9.85.

use serde::{Deserialize, Serialize};

/// O.1.5: product of mean load and mean on-time, s.
pub const FULL_LOAD_ON_TIME_S: f64 = 120.0;
/// O.1.5: bivalent appliances with β < 1.
pub const BIVALENT_ON_TIME_S: f64 = 1800.0;
/// f_nuttig;p (9.87).
pub const USEFUL_PUMP_FRACTION: f64 = 0.5;
/// f_P;del;el of table 5.2.
pub const PRIMARY_FACTOR_ELECTRICITY: f64 = 1.45;

/// One measured point (modulation or load, electric power in W).
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PowerPoint {
    pub modulation: f64,
    pub power_w: f64,
}

/// One point of the load curve after ignition (O.2.2).
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoadCurvePoint {
    pub time_s: f64,
    /// Running mean load since ignition (0–1, relative to B_nom).
    pub mean_load: f64,
    /// Running mean pump modulation (modulating pumps only).
    #[serde(default)]
    pub mean_pump_modulation: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum FanMeasurement {
    None,
    /// One speed: power during burner operation (9.6.8.1, a).
    SingleSpeed {
        #[serde(rename = "powerW")]
        power_w: f64,
    },
    /// Modulating fan: points for the straight line of O.2.
    Modulating {
        points: Vec<PowerPoint>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum PumpMeasurement {
    /// No pump in the appliance.
    None,
    /// Speed setting: highest measured power per setting (O.2.3.2).
    Staged {
        #[serde(rename = "operationW")]
        operation_w: f64,
        #[serde(rename = "prePostRunW")]
        pre_post_run_w: f64,
    },
    /// Modulating pump: points for O.3; the pre/post-run modulation.
    Modulating {
        points: Vec<PowerPoint>,
        #[serde(rename = "prePostRunModulation")]
        pre_post_run_modulation: f64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppliancePowerMeasurements {
    /// B_nom on gross calorific value, kW (O.2.2).
    pub nominal_load_kw: f64,
    pub standby_electronics_w: f64,
    #[serde(default)]
    pub gas_valve_w: f64,
    pub fan: FanMeasurement,
    pub pump: PumpMeasurement,
    #[serde(default)]
    pub pump_pre_run_s: f64,
    #[serde(default)]
    pub pump_post_run_s: f64,
    #[serde(default)]
    pub fan_pre_run_s: f64,
    #[serde(default)]
    pub fan_post_run_s: f64,
    /// Running means after ignition (O.2.3.4); ignored for bivalent units.
    #[serde(default)]
    pub load_curve: Vec<LoadCurvePoint>,
    /// Bivalent appliance with β < 1 (O.1.5): 1 800 s on-time.
    #[serde(default)]
    pub bivalent: bool,
    /// Mean burner modulation for bivalent appliances.
    #[serde(default)]
    pub bivalent_mean_load: Option<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnexOIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuxiliaryConstants {
    /// t_a;gem, s.
    pub mean_on_time_s: f64,
    /// m_b.
    pub mean_modulation: f64,
    /// A in kWh, B in kW, C (9.86–9.88).
    pub a_kwh: f64,
    pub b_kw: f64,
    pub c: f64,
    pub nominal_load_kw: f64,
}

/// O.2/O.3: least-squares straight line `a + b·x`.
pub fn fit_line(points: &[PowerPoint]) -> Option<(f64, f64)> {
    let n = points.len() as f64;
    if points.len() < 2 {
        return None;
    }
    let mean_x = points.iter().map(|p| p.modulation).sum::<f64>() / n;
    let mean_y = points.iter().map(|p| p.power_w).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|p| (p.modulation - mean_x).powi(2)).sum();
    if sxx <= 0.0 {
        return None;
    }
    let sxy: f64 = points
        .iter()
        .map(|p| (p.modulation - mean_x) * (p.power_w - mean_y))
        .sum();
    let b = sxy / sxx;
    Some((mean_y - b * mean_x, b))
}

/// O.1.5/O.2.3.4: the on-time where mean load × time equals 120 s, with
/// linear interpolation between the measured points; returns (t_a;gem, mean
/// load, mean pump modulation).
pub fn mean_on_time(curve: &[LoadCurvePoint]) -> Option<(f64, f64, Option<f64>)> {
    let product = |point: &LoadCurvePoint| point.mean_load * point.time_s;
    let first = curve.first()?;
    if product(first) >= FULL_LOAD_ON_TIME_S {
        let t = FULL_LOAD_ON_TIME_S / first.mean_load;
        return Some((t, first.mean_load, first.mean_pump_modulation));
    }
    for pair in curve.windows(2) {
        let (p0, p1) = (product(&pair[0]), product(&pair[1]));
        if p0 < FULL_LOAD_ON_TIME_S && p1 >= FULL_LOAD_ON_TIME_S {
            let f = (FULL_LOAD_ON_TIME_S - p0) / (p1 - p0);
            let lerp = |a: f64, b: f64| a + f * (b - a);
            let pump = match (pair[0].mean_pump_modulation, pair[1].mean_pump_modulation) {
                (Some(a), Some(b)) => Some(lerp(a, b)),
                _ => None,
            };
            return Some((
                lerp(pair[0].time_s, pair[1].time_s),
                lerp(pair[0].mean_load, pair[1].mean_load),
                pump,
            ));
        }
    }
    None
}

fn issue(code: &'static str, path: String) -> AnnexOIssue {
    AnnexOIssue { code, path }
}

/// 9.86–9.90 with O.2.3; `Err` lists what is missing or invalid.
pub fn auxiliary_constants(
    m: &AppliancePowerMeasurements,
    path: &str,
) -> Result<AuxiliaryConstants, Vec<AnnexOIssue>> {
    let mut issues = Vec::new();
    if m.source_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            format!("{path}.sourceReference"),
        ));
    }
    if !(m.nominal_load_kw.is_finite() && m.nominal_load_kw > 0.0) {
        issues.push(issue(
            "nominal_load_invalid",
            format!("{path}.nominalLoadKw"),
        ));
    }
    for (field, value) in [
        ("standbyElectronicsW", m.standby_electronics_w),
        ("gasValveW", m.gas_valve_w),
        ("pumpPreRunS", m.pump_pre_run_s),
        ("pumpPostRunS", m.pump_post_run_s),
        ("fanPreRunS", m.fan_pre_run_s),
        ("fanPostRunS", m.fan_post_run_s),
    ] {
        if !(value.is_finite() && value >= 0.0) {
            issues.push(issue("measurement_invalid", format!("{path}.{field}")));
        }
    }
    let timing = if m.bivalent {
        m.bivalent_mean_load
            .filter(|load| *load > 0.0 && *load <= 1.0)
            .map(|load| (BIVALENT_ON_TIME_S, load, None))
    } else {
        mean_on_time(&m.load_curve)
    };
    let Some((on_time, mean_load, curve_pump)) = timing else {
        issues.push(issue(
            "mean_on_time_undetermined",
            format!("{path}.loadCurve"),
        ));
        return Err(issues);
    };
    let fan_w = match &m.fan {
        FanMeasurement::None => 0.0,
        FanMeasurement::SingleSpeed { power_w } => *power_w,
        FanMeasurement::Modulating { points } => match fit_line(points) {
            Some((a, b)) => a + b * mean_load,
            None => {
                issues.push(issue("fan_fit_undetermined", format!("{path}.fan.points")));
                0.0
            }
        },
    };
    let (pump_operation, pump_pre_post) = match &m.pump {
        PumpMeasurement::None => (0.0, 0.0),
        PumpMeasurement::Staged {
            operation_w,
            pre_post_run_w,
        } => (*operation_w, *pre_post_run_w),
        PumpMeasurement::Modulating {
            points,
            pre_post_run_modulation,
        } => match (fit_line(points), curve_pump) {
            (Some((c, d)), Some(mp)) => (c + d * mp, c + d * pre_post_run_modulation),
            (None, _) => {
                issues.push(issue(
                    "pump_fit_undetermined",
                    format!("{path}.pump.points"),
                ));
                (0.0, 0.0)
            }
            (_, None) => {
                issues.push(issue(
                    "pump_modulation_undetermined",
                    format!("{path}.loadCurve"),
                ));
                (0.0, 0.0)
            }
        },
    };
    if !issues.is_empty() {
        return Err(issues);
    }
    // 9.86–9.88.
    let a = m.standby_electronics_w * 8760.0 / 1000.0;
    let b = (m.gas_valve_w
        + (1.0 - USEFUL_PUMP_FRACTION / PRIMARY_FACTOR_ELECTRICITY)
            * (pump_operation + pump_pre_post * (m.pump_pre_run_s + m.pump_post_run_s) / on_time)
        + fan_w * (1.0 + (m.fan_pre_run_s + m.fan_post_run_s) / on_time))
        / 1000.0;
    Ok(AuxiliaryConstants {
        mean_on_time_s: on_time,
        mean_modulation: mean_load,
        a_kwh: a,
        b_kw: b,
        c: mean_load,
        nominal_load_kw: m.nominal_load_kw,
    })
}

/// 9.85 for one month: `1,0·(A·N/12 + B·E_H;ci/(C·B_nom))`, N = 1.
pub fn monthly_auxiliary_kwh(constants: &AuxiliaryConstants, carrier_input_kwh: f64) -> f64 {
    constants.a_kwh / 12.0
        + constants.b_kw * carrier_input_kwh / (constants.c * constants.nominal_load_kw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_on_time_matches_the_table_o2_example() {
        let curve = [
            (60.0, 0.22, 0.71),
            (120.0, 0.25, 0.84),
            (180.0, 0.28, 0.90),
            (240.0, 0.35, 0.92),
            (300.0, 0.45, 0.94),
        ]
        .map(|(t, l, p)| LoadCurvePoint {
            time_s: t,
            mean_load: l,
            mean_pump_modulation: Some(p),
        });
        let (t, load, pump) = mean_on_time(&curve).unwrap();
        // 240·0,35 = 84 and 300·0,45 = 135; 120 lies at 36/51 of the step.
        let f = 36.0 / 51.0;
        assert!((t - (240.0 + 60.0 * f)).abs() < 1e-9);
        assert!((load - (0.35 + 0.10 * f)).abs() < 1e-12);
        assert!((pump.unwrap() - (0.92 + 0.02 * f)).abs() < 1e-12);
    }

    #[test]
    fn fan_line_fit_and_constants_follow_9_86_to_9_88() {
        let points = [(0.28, 8.8), (0.54, 20.0), (0.80, 31.2)].map(|(m, p)| PowerPoint {
            modulation: m,
            power_w: p,
        });
        let (a, b) = fit_line(&points).unwrap();
        assert!((b - 11.2 / 0.26).abs() < 1e-9);
        assert!((a - (8.8 - b * 0.28)).abs() < 1e-9);
        let m = AppliancePowerMeasurements {
            nominal_load_kw: 24.0,
            standby_electronics_w: 2.0,
            gas_valve_w: 5.0,
            fan: FanMeasurement::Modulating {
                points: points.to_vec(),
            },
            pump: PumpMeasurement::Staged {
                operation_w: 40.0,
                pre_post_run_w: 30.0,
            },
            pump_pre_run_s: 0.0,
            pump_post_run_s: 60.0,
            fan_pre_run_s: 5.0,
            fan_post_run_s: 10.0,
            load_curve: vec![
                LoadCurvePoint {
                    time_s: 200.0,
                    mean_load: 0.5,
                    mean_pump_modulation: None,
                },
                LoadCurvePoint {
                    time_s: 300.0,
                    mean_load: 0.5,
                    mean_pump_modulation: None,
                },
            ],
            bivalent: false,
            bivalent_mean_load: None,
            source_reference: "test report".into(),
        };
        let constants = auxiliary_constants(&m, "aux").unwrap();
        // 0,5·t = 120 → t = 240 s.
        assert!((constants.mean_on_time_s - 240.0).abs() < 1e-9);
        assert!((constants.a_kwh - 2.0 * 8.76).abs() < 1e-12);
        let fan = a + b * 0.5;
        let expected_b =
            (5.0 + (1.0 - 0.5 / 1.45) * (40.0 + 30.0 * 60.0 / 240.0) + fan * (1.0 + 15.0 / 240.0))
                / 1000.0;
        assert!((constants.b_kw - expected_b).abs() < 1e-12);
        assert_eq!(constants.c, 0.5);
        let monthly = monthly_auxiliary_kwh(&constants, 1200.0);
        assert!((monthly - (2.0 * 8.76 / 12.0 + expected_b * 1200.0 / (0.5 * 24.0))).abs() < 1e-12);
    }

    #[test]
    fn missing_timing_is_reported() {
        let m = AppliancePowerMeasurements {
            nominal_load_kw: 24.0,
            standby_electronics_w: 2.0,
            gas_valve_w: 0.0,
            fan: FanMeasurement::None,
            pump: PumpMeasurement::None,
            pump_pre_run_s: 0.0,
            pump_post_run_s: 0.0,
            fan_pre_run_s: 0.0,
            fan_post_run_s: 0.0,
            load_curve: Vec::new(),
            bivalent: false,
            bivalent_mean_load: None,
            source_reference: "x".into(),
        };
        let codes: Vec<_> = auxiliary_constants(&m, "aux")
            .unwrap_err()
            .into_iter()
            .map(|item| item.code)
            .collect();
        assert_eq!(codes, ["mean_on_time_undetermined"]);
    }
}
