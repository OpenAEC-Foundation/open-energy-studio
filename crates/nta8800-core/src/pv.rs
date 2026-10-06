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
            } => module_type.peak_power_w_per_m2() * area(*panel_area_m2) / 1000.0,
            // NTA 8800:2020+A1 (16.4) (p. 651): the measured K_pk is rounded
            // down to a multiple of 5 W/m²; from 2022 (p. 656–657) K_pk and
            // the area carry two decimals.
            Self::DeclaredSpecific {
                peak_power_w_per_m2,
                panel_area_m2,
            } if crate::norm_versions::profile().pv_kpk_per_m2_floor => {
                (peak_power_w_per_m2 / PANEL_POWER_STEP_W + 1e-9).floor()
                    * PANEL_POWER_STEP_W
                    * panel_area_m2
                    / 1000.0
            }
            Self::DeclaredSpecific {
                peak_power_w_per_m2,
                panel_area_m2,
            } => round2(*peak_power_w_per_m2) * round2(*panel_area_m2) / 1000.0,
            Self::Panels {
                panel_peak_power_w,
                panel_count,
            } => {
                let rounded =
                    (panel_peak_power_w / PANEL_POWER_STEP_W + 1e-9).floor() * PANEL_POWER_STEP_W;
                rounded * f64::from(*panel_count) / 1000.0
            }
        }
    }
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// The panel area: two decimals from NTA 8800:2022 (p. 657), as given in
/// 2020+A1 (p. 651).
fn area(value: f64) -> f64 {
    if crate::norm_versions::profile().pv_kpk_per_m2_floor {
        value
    } else {
        round2(value)
    }
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
    /// `F_sh;obst;mi` per §17.3: one value for all months or twelve values;
    /// exclusive with `obstruction`.
    #[serde(default)]
    pub obstruction_factors: Vec<f64>,
    /// §17.3 situation for collectors and PV (tables 17.6/17.12/17.15),
    /// at the nearest of the eight orientations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obstruction: Option<crate::solar_shading::CollectorObstruction>,
    #[serde(default)]
    pub collective: Option<CollectivePv>,
    /// §16.3: a PVT system; its electricity is 16.10 = the PV yield of
    /// 16.2 × f_PVT;PV of table 16.4 (the heat follows 13.7.2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pvt: Option<PvtCover>,
    pub source_reference: String,
}

/// Table 16.4 covers of a PVT system.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PvtCover {
    /// Unglazed: f_PVT;PV = 1,00.
    Unglazed,
    /// Covered with single glass: by (A_sol;mod·N_col)/V_sto;tot.
    Glazed {
        #[serde(rename = "collectorAreaM2")]
        collector_area_m2: f64,
        #[serde(rename = "storageVolumeL")]
        storage_volume_l: f64,
    },
}

impl PvtCover {
    /// Table 16.4 f_PVT;PV.
    pub fn factor(self) -> Option<f64> {
        match self {
            Self::Unglazed => Some(1.0),
            Self::Glazed {
                collector_area_m2,
                storage_volume_l,
            } => {
                if !(collector_area_m2 > 0.0 && storage_volume_l > 0.0) {
                    return None;
                }
                let ratio = collector_area_m2 / storage_volume_l;
                Some(if ratio < 0.015 {
                    0.88
                } else if ratio <= 0.03 {
                    0.84
                } else {
                    0.80
                })
            }
        }
    }
}

/// Nearest table orientation of an azimuth (0 = north, clockwise).
fn nearest_orientation(azimuth_deg: f64) -> crate::climate::Orientation {
    use crate::climate::Orientation::*;
    let sector = ((azimuth_deg.rem_euclid(360.0) / 45.0).round() as usize) % 8;
    [
        North, NorthEast, East, SouthEast, South, SouthWest, West, NorthWest,
    ][sector]
}

/// 17.3.7 (p. 749): the obstruction table value of the nearest orientation;
/// exactly midway between two the higher neighbouring value may be used,
/// and the kernel takes it (as 17.2 does for I_sol, p. 694).
fn collector_obstruction(
    situation: &crate::solar_shading::CollectorObstruction,
    azimuth_deg: f64,
    tilt_deg: f64,
    month: u8,
) -> Option<f64> {
    let first = nearest_orientation(azimuth_deg);
    let value =
        crate::solar_shading::collector_obstruction_factor(situation, first, tilt_deg, month)?;
    let sector = azimuth_deg.rem_euclid(360.0) / 45.0;
    if (sector - sector.floor() - 0.5).abs() > 1e-9 {
        return Some(value);
    }
    // Exactly midway: `round` took the clockwise neighbour; compare with
    // the other one.
    let other = nearest_orientation(azimuth_deg - 22.5 - 1e-6);
    let alternative =
        crate::solar_shading::collector_obstruction_factor(situation, other, tilt_deg, month)?;
    Some(value.max(alternative))
}

fn unknown_mounting() -> PvMounting {
    PvMounting::Unknown
}

impl PvSystem {
    fn obstruction(&self, index: usize) -> f64 {
        if let Some(situation) = &self.obstruction {
            return collector_obstruction(
                situation,
                self.azimuth_deg,
                self.tilt_deg,
                index as u8 + 1,
            )
            .unwrap_or(1.0);
        }
        if self.obstruction_factors.len() == 12 {
            self.obstruction_factors[index]
        } else {
            self.obstruction_factors[0]
        }
    }
}

/// Readings of chapter 16 and 17.3.7 where the norm leaves a choice.
pub const INTERPRETATIONS: &[&str] = &[
    "Table 16.3 (p. 681) lists c_sh;PV from F_sh;obst 0,80 to 1,00 and gives no rule outside that range; below 0,80 the kernel holds the last value 0,75 (as other tables are held at their ends). The yield of 16.2/16.3 still falls with F_sh;obst itself, so only the extra mismatch correction stops at 0,75",
    "17.3.7 (p. 749): an azimuth exactly midway between two table orientations may take the higher neighbouring obstruction value; the kernel does so per month, consistent with I_sol in 17.2 (p. 694), where the higher value is required",
];

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
            // 16.4b is not in NTA 8800:2020+A1 (p. 651).
            if crate::norm_versions::profile().pv_kpk_per_m2_floor {
                push("route_not_in_edition", "peakPower.panelPeakPowerW");
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
    match &system.obstruction {
        Some(situation) => {
            if !factors.is_empty() {
                push("pv_obstruction_declared_twice", "obstruction");
            }
            if (1..=12).any(|month| {
                collector_obstruction(situation, system.azimuth_deg, system.tilt_deg, month)
                    .is_none()
            }) {
                push("pv_obstruction_invalid", "obstruction");
            }
        }
        None => {
            if !(factors.len() == 1 || factors.len() == 12)
                || factors
                    .iter()
                    .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            {
                push("pv_obstruction_factor_invalid", "obstructionFactors");
            }
        }
    }
    if system.pvt.is_some_and(|cover| cover.factor().is_none()) {
        push("pvt_cover_invalid", "pvt");
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
        *value = solar
            * peak
            * performance
            * shading_correction(obstruction)
            * F_PRAC_PV
            * system.pvt.and_then(PvtCover::factor).unwrap_or(1.0)
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
            peak_power: PeakPower::Panels {
                panel_peak_power_w: 400.0,
                panel_count: 10,
            },
            azimuth_deg: 180.0,
            tilt_deg: 30.0,
            mounting: PvMounting::ModeratelyVentilated,
            obstruction_factors: vec![1.0],
            obstruction: None,
            collective: None,
            pvt: None,
            source_reference: "datasheet".into(),
        }
    }

    #[test]
    fn pvt_electricity_follows_16_10_and_table_16_4() {
        let plain = monthly_yield_kwh(&system(), 100.0);
        let mut pvt = system();
        pvt.pvt = Some(PvtCover::Glazed {
            collector_area_m2: 4.0,
            storage_volume_l: 200.0,
        });
        // 4/200 = 0,02 m²/dm³: 0,84.
        let glazed = monthly_yield_kwh(&pvt, 100.0);
        assert!((glazed[5] - 0.84 * plain[5]).abs() < 1e-9);
        assert_eq!(
            PvtCover::Glazed {
                collector_area_m2: 2.0,
                storage_volume_l: 200.0
            }
            .factor(),
            Some(0.88)
        );
        assert_eq!(
            PvtCover::Glazed {
                collector_area_m2: 8.0,
                storage_volume_l: 200.0
            }
            .factor(),
            Some(0.80)
        );
        pvt.pvt = Some(PvtCover::Unglazed);
        assert!((monthly_yield_kwh(&pvt, 100.0)[5] - plain[5]).abs() < 1e-12);
    }

    #[test]
    fn collector_obstruction_situation_feeds_16_3() {
        use crate::solar_shading::{CollectorObstruction, ObstructionSide};
        let mut shaded = system();
        shaded.obstruction_factors.clear();
        shaded.obstruction = Some(CollectorObstruction::Full);
        assert!(
            validate_pv(&shaded, "pv").is_empty(),
            "{:?}",
            validate_pv(&shaded, "pv")
        );
        let open = monthly_yield_kwh(&system(), 100.0);
        let full = monthly_yield_kwh(&shaded, 100.0);
        assert!(full[0] < open[0]);
        // Minimal obstruction for collectors is 1,00 (table 17.6).
        shaded.obstruction = Some(CollectorObstruction::Minimal);
        let minimal = monthly_yield_kwh(&shaded, 100.0);
        assert!((minimal[5] - open[5]).abs() < 1e-9);
        // Both routes at once are rejected.
        shaded.obstruction_factors = vec![1.0];
        shaded.obstruction = Some(CollectorObstruction::SideObstruction {
            side: ObstructionSide::Both,
            relative_width: 0.5,
        });
        assert!(!validate_pv(&shaded, "pv").is_empty());
        assert_eq!(
            nearest_orientation(170.0),
            crate::climate::Orientation::South
        );
        assert_eq!(
            nearest_orientation(350.0),
            crate::climate::Orientation::North
        );
    }

    #[test]
    fn midway_orientation_takes_the_higher_obstruction_value() {
        use crate::climate::Orientation::{North, NorthEast, NorthWest, West};
        use crate::solar_shading::collector_obstruction_factor as table;
        let edge = crate::solar_shading::CollectorObstruction::RoofEdge {
            height_m: 2.0,
            distance_m: 1.0,
        };
        // 17.3.7 (p. 749): exactly midway the higher neighbouring value.
        for (azimuth, tilt, a, b) in [
            (22.5, 45.0, North, NorthEast),
            (292.5, 90.0, West, NorthWest),
        ] {
            for month in 1..=12 {
                let expected = table(&edge, a, tilt, month)
                    .unwrap()
                    .max(table(&edge, b, tilt, month).unwrap());
                assert_eq!(
                    collector_obstruction(&edge, azimuth, tilt, month),
                    Some(expected)
                );
            }
        }
        // Off the midpoint the nearest orientation alone.
        assert_eq!(
            collector_obstruction(&edge, 20.0, 45.0, 6),
            table(&edge, North, 45.0, 6)
        );
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
