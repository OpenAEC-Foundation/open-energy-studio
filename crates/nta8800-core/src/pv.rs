//! Photovoltaic electricity yield, NTA 8800:2025+C1:2026 §16.2
//! (pages 678–682):
//!
//! `E_sol;mi = I_sol;mi · t_mi · F_sh;obst;mi / 1000` [kWh/m²] (16.3)
//! `E_el;PV;out;mi = E_sol;mi · P_pk · f_perf · c_sh;PV;mi · f_prac;PV / I_ref` (16.2)
//!
//! with `f_prac;PV = 0,95` and `I_ref = 1 kW/m²`. `P_pk` follows 16.4a
//! (`K_pk · A_PV / 1000`, `K_pk` from table 16.1 or a declaration) or 16.4b
//! (panel peak power rounded down to a multiple of 5 W times the panel
//! count). `f_perf` follows table 16.2; an unknown mounting is "not
//! ventilated" (0,76). `c_sh;PV;mi` is derived from the monthly `F_sh;obst;mi`
//! with table 16.3 and linear interpolation. A collective system is split by
//! `A_g;tot / A_g;gebouw;PV` (p. 678). Tilt and orientation enter only
//! through `I_sol;mi` of table 17.2.

use crate::climate::{irradiance_at, MONTH_HOURS};
use serde::{Deserialize, Serialize};

/// 16.2.2.5: practical correction.
pub const F_PRAC_PV: f64 = 0.95;
/// 16.2: reference irradiance in kW/m².
pub const I_REF: f64 = 1.0;
/// 16.4b: panel peak power is rounded down to a multiple of this, W.
pub const PANEL_POWER_STEP_W: f64 = 5.0;

/// Table 16.2: integration and ventilation of the panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PvMounting {
    NotVentilated,
    ModeratelyVentilated,
    StronglyVentilated,
    /// p. 681: an unknown mounting counts as not ventilated.
    Unknown,
}

impl PvMounting {
    /// Table 16.2 `f_perf`.
    pub fn performance_factor(self) -> f64 {
        match self {
            Self::NotVentilated | Self::Unknown => 0.76,
            Self::ModeratelyVentilated => 0.80,
            Self::StronglyVentilated => 0.82,
        }
    }
}

/// Table 16.1 module types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PvModuleType {
    MonocrystallineBefore2001,
    Monocrystalline2001To2010,
    Monocrystalline2011To2014,
    Monocrystalline2015To2017,
    MonocrystallineFrom2018,
    MulticrystallineBefore2001,
    Multicrystalline2001To2010,
    Multicrystalline2011To2014,
    Multicrystalline2015To2017,
    MulticrystallineFrom2018,
    AmorphousSingleJunction,
    AmorphousMultiJunction,
    Cigs,
    CdTe,
}

impl PvModuleType {
    /// Table 16.1 `K_pk` in W/m².
    pub fn peak_power_w_per_m2(self) -> f64 {
        match self {
            Self::MonocrystallineBefore2001 => 125.0,
            Self::Monocrystalline2001To2010 => 135.0,
            Self::Monocrystalline2011To2014 => 150.0,
            Self::Monocrystalline2015To2017 => 165.0,
            Self::MonocrystallineFrom2018 => 175.0,
            Self::MulticrystallineBefore2001 => 115.0,
            Self::Multicrystalline2001To2010 => 125.0,
            Self::Multicrystalline2011To2014 => 140.0,
            Self::Multicrystalline2015To2017 => 155.0,
            Self::MulticrystallineFrom2018 => 165.0,
            Self::AmorphousSingleJunction => 65.0,
            Self::AmorphousMultiJunction => 55.0,
            Self::Cigs => 105.0,
            Self::CdTe => 95.0,
        }
    }
}

/// Route for `P_pk` (16.4a/16.4b).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum PeakPower {
    /// 16.4a with `K_pk` from table 16.1.
    Table16_1 {
        #[serde(rename = "moduleType")]
        module_type: PvModuleType,
        #[serde(rename = "panelAreaM2")]
        panel_area_m2: f64,
    },
    /// 16.4a with a declared `K_pk` (quality declaration, two decimals).
    DeclaredSpecific {
        #[serde(rename = "peakPowerWPerM2")]
        peak_power_w_per_m2: f64,
        #[serde(rename = "panelAreaM2")]
        panel_area_m2: f64,
    },
    /// 16.4b: tested panel peak power (rounded down to 5 W) times count.
    Panels {
        #[serde(rename = "panelPeakPowerW")]
        panel_peak_power_w: f64,
        #[serde(rename = "panelCount")]
        panel_count: u32,
    },
}

impl PeakPower {
    pub fn kw(&self) -> f64 {
        match self {
            Self::Table16_1 {
                module_type,
                panel_area_m2,
            } => module_type.peak_power_w_per_m2() * round2(*panel_area_m2) / 1000.0,
            Self::DeclaredSpecific {
                peak_power_w_per_m2,
                panel_area_m2,
            } => round2(*peak_power_w_per_m2) * round2(*panel_area_m2) / 1000.0,
            Self::Panels {
                panel_peak_power_w,
                panel_count,
            } => {
                let rounded = (panel_peak_power_w / PANEL_POWER_STEP_W + 1e-9).floor()
                    * PANEL_POWER_STEP_W;
                rounded * f64::from(*panel_count) / 1000.0
            }
        }
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// p. 678: a collective system on a building of which only part is assessed.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectivePv {
    /// `A_g;gebouw;PV` (6.6.7), m².
    pub building_usable_floor_area_m2: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PvSystem {
    pub id: String,
    pub peak_power: PeakPower,
    /// Azimuth, 0 = north, clockwise.
    pub azimuth_deg: f64,
    /// 0 = horizontal, 90 = vertical.
    pub tilt_deg: f64,
    /// Table 16.2; omitted means unknown (0,76).
    #[serde(default = "unknown_mounting")]
    pub mounting: PvMounting,
    /// `F_sh;obst;mi` per §17.3: one value for all months or twelve values.
    pub obstruction_factors: Vec<f64>,
    #[serde(default)]
    pub collective: Option<CollectivePv>,
    pub source_reference: String,
}

fn unknown_mounting() -> PvMounting {
    PvMounting::Unknown
}

impl PvSystem {
    fn obstruction(&self, index: usize) -> f64 {
        if self.obstruction_factors.len() == 12 {
            self.obstruction_factors[index]
        } else {
            self.obstruction_factors[0]
        }
    }
}

/// Table 16.3 with linear interpolation; `F_sh;obst ≤ 0,80` gives 0,75.
pub fn shading_correction(obstruction_factor: f64) -> f64 {
    const POINTS: [(f64, f64); 5] = [
        (0.80, 0.75),
        (0.85, 0.82),
        (0.90, 0.89),
        (0.95, 0.95),
        (1.00, 1.00),
    ];
    if obstruction_factor <= POINTS[0].0 {
        return POINTS[0].1;
    }
    for pair in POINTS.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        if obstruction_factor <= x1 {
            return y0 + (obstruction_factor - x0) / (x1 - x0) * (y1 - y0);
        }
    }
    1.0
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
    let positive = |value: f64| value.is_finite() && value > 0.0;
    match &system.peak_power {
        PeakPower::Table16_1 { panel_area_m2, .. } => {
            if !positive(*panel_area_m2) {
                push("pv_panel_area_invalid", "peakPower.panelAreaM2");
            }
        }
        PeakPower::DeclaredSpecific {
            peak_power_w_per_m2,
            panel_area_m2,
        } => {
            if !positive(*panel_area_m2) {
                push("pv_panel_area_invalid", "peakPower.panelAreaM2");
            }
            if !positive(*peak_power_w_per_m2) || *peak_power_w_per_m2 > 300.0 {
                push("pv_peak_power_invalid", "peakPower.peakPowerWPerM2");
            }
        }
        PeakPower::Panels {
            panel_peak_power_w,
            panel_count,
        } => {
            if !positive(*panel_peak_power_w) || *panel_peak_power_w < PANEL_POWER_STEP_W {
                push("pv_peak_power_invalid", "peakPower.panelPeakPowerW");
            }
            if *panel_count == 0 {
                push("pv_panel_count_invalid", "peakPower.panelCount");
            }
        }
    }
    if !system.azimuth_deg.is_finite() {
        push("pv_azimuth_invalid", "azimuthDeg");
    }
    if irradiance_at(system.azimuth_deg.rem_euclid(360.0), system.tilt_deg, 1).is_none() {
        push("pv_tilt_invalid", "tiltDeg");
    }
    let factors = &system.obstruction_factors;
    if !(factors.len() == 1 || factors.len() == 12)
        || factors
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        push("pv_obstruction_factor_invalid", "obstructionFactors");
    }
    if let Some(collective) = &system.collective {
        if !positive(collective.building_usable_floor_area_m2) {
            push(
                "pv_collective_area_invalid",
                "collective.buildingUsableFloorAreaM2",
            );
        }
        if collective.source_reference.trim().is_empty() {
            push("source_reference_required", "collective.sourceReference");
        }
    }
    if system.source_reference.trim().is_empty() {
        push("source_reference_required", "sourceReference");
    }
    issues
}

/// Collective share `A_g;tot / A_g;gebouw;PV`, at most 1.
pub fn collective_share(system: &PvSystem, assessed_usable_floor_area_m2: f64) -> f64 {
    system.collective.as_ref().map_or(1.0, |collective| {
        (assessed_usable_floor_area_m2 / collective.building_usable_floor_area_m2).min(1.0)
    })
}

/// Monthly yield in kWh; call only after [`validate_pv`] returned no issues.
/// `assessed_usable_floor_area_m2` is `A_g;tot` for a collective split.
pub fn monthly_yield_kwh(system: &PvSystem, assessed_usable_floor_area_m2: f64) -> [f64; 12] {
    let peak = system.peak_power.kw() * collective_share(system, assessed_usable_floor_area_m2);
    let performance = system.mounting.performance_factor();
    let azimuth = system.azimuth_deg.rem_euclid(360.0);
    let mut values = [0.0; 12];
    for (index, value) in values.iter_mut().enumerate() {
        let month = index as u8 + 1;
        let irradiance =
            irradiance_at(azimuth, system.tilt_deg, month).expect("validated tilt and azimuth");
        let obstruction = system.obstruction(index);
        // 16.3
        let solar = irradiance * MONTH_HOURS[index] * obstruction / 1000.0;
        // 16.2 with table 16.3
        *value = solar * peak * performance * shading_correction(obstruction) * F_PRAC_PV / I_REF;
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system() -> PvSystem {
        PvSystem {
            id: "pv".into(),
            peak_power: PeakPower::Panels {
                panel_peak_power_w: 400.0,
                panel_count: 10,
            },
            azimuth_deg: 180.0,
            tilt_deg: 30.0,
            mounting: PvMounting::ModeratelyVentilated,
            obstruction_factors: vec![1.0],
            collective: None,
            source_reference: "datasheet".into(),
        }
    }

    #[test]
    fn june_south_30_degrees_matches_hand_calculation() {
        let yields = monthly_yield_kwh(&system(), 100.0);
        let expected = 211.2 * 720.0 / 1000.0 * 4.0 * 0.80 * 0.95;
        assert!((yields[5] - expected).abs() < 1e-9);
        let annual: f64 = yields.iter().sum();
        // About 900 kWh per kWp for an ideal south roof at 30°.
        assert!((850.0..1000.0).contains(&(annual / 4.0)));
    }

    #[test]
    fn peak_power_routes_follow_16_4a_and_16_4b() {
        let table = PeakPower::Table16_1 {
            module_type: PvModuleType::MonocrystallineFrom2018,
            panel_area_m2: 20.004,
        };
        assert!((table.kw() - 175.0 * 20.0 / 1000.0).abs() < 1e-12);
        let panels = PeakPower::Panels {
            panel_peak_power_w: 409.0,
            panel_count: 10,
        };
        assert!((panels.kw() - 4.05).abs() < 1e-12);
        assert_eq!(PvModuleType::CdTe.peak_power_w_per_m2(), 95.0);
        assert_eq!(PvMounting::Unknown.performance_factor(), 0.76);
    }

    #[test]
    fn table_16_3_interpolates_and_floors_at_0_80() {
        assert!((shading_correction(1.0) - 1.0).abs() < 1e-12);
        assert!((shading_correction(0.925) - 0.92).abs() < 1e-12);
        assert!((shading_correction(0.87) - 0.848).abs() < 1e-12);
        assert!((shading_correction(0.80) - 0.75).abs() < 1e-12);
        assert!((shading_correction(0.5) - 0.75).abs() < 1e-12);
        let mut shaded = system();
        shaded.obstruction_factors = vec![0.9];
        let base = monthly_yield_kwh(&system(), 100.0);
        let yields = monthly_yield_kwh(&shaded, 100.0);
        assert!((yields[5] - base[5] * 0.9 * 0.89).abs() < 1e-9);
    }

    #[test]
    fn collective_system_is_split_by_floor_area() {
        let mut shared = system();
        shared.collective = Some(CollectivePv {
            building_usable_floor_area_m2: 400.0,
            source_reference: "building plan".into(),
        });
        let own = monthly_yield_kwh(&system(), 100.0);
        let part = monthly_yield_kwh(&shared, 100.0);
        assert!((part[5] - own[5] / 4.0).abs() < 1e-9);
    }

    #[test]
    fn west_and_north_roofs_yield_less_but_not_zero() {
        let south: f64 = monthly_yield_kwh(&system(), 100.0).iter().sum();
        let mut west = system();
        west.azimuth_deg = 270.0;
        let mut north = system();
        north.azimuth_deg = 0.0;
        let west: f64 = monthly_yield_kwh(&west, 100.0).iter().sum();
        let north: f64 = monthly_yield_kwh(&north, 100.0).iter().sum();
        assert!(south > west && west > north && north > 0.0);
    }

    #[test]
    fn validation_rejects_values_outside_tables() {
        let mut bad = system();
        bad.peak_power = PeakPower::Panels {
            panel_peak_power_w: 0.0,
            panel_count: 0,
        };
        bad.obstruction_factors = vec![1.0; 5];
        bad.tilt_deg = 190.0;
        bad.source_reference = " ".into();
        let codes: Vec<_> = validate_pv(&bad, "pv[0]")
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "pv_peak_power_invalid",
            "pv_panel_count_invalid",
            "pv_obstruction_factor_invalid",
            "pv_tilt_invalid",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
        assert!(validate_pv(&system(), "pv[0]").is_empty());
    }
}
