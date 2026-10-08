//! Solar water heaters and solar combi systems, NTA 8800:2025+C1:2026
//! §13.7 (pages 572–604).
//!
//! - 13.7.2.2 (NEN-EN 15316-4-3 method 2) per service, hot water
//!   (13.77–13.109) or space heating (13.85–13.127): effective collector
//!   area, volume and loss per service share `f_use`, the X/Y correlation
//!   13.97–13.100 with table 13.15, storage losses 13.95/13.102, rounding
//!   of `Q_ren` per annex X, recoverable losses 13.107/13.125 and pump
//!   energy 13.108/13.126;
//! - 13.7.2.3 (method 1) systems tested as a whole per NEN-EN 12976/12977-2,
//!   hot water only: interpolation 13.130 and monthly distribution
//!   13.128–13.140;
//! - 13.7.2.4 PVT reduction (table 13.16).
//!
//! The vessel is taken to stand in a heated room (13.7.2.2.1): `ϑ_sto;amb`
//! is the levelled setpoint `ϑ_int;set;H;zi,mi` (13.69/13.137b), or
//! `ϑ_int;set;H;stc` with an exhaust-air heat pump for hot water
//! (13.69a/13.137a), and `f_rbl = 1` (13.107); the 500 m² rule of 13.13 is
//! applied by the hot-water chain. SOL_USE = SHS (space heating only) uses
//! `f_H;use = 1` (13.85) and 2 000 pump hours (13.127).

use crate::climate::{irradiance_w_per_m2, Orientation, MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use crate::domestic_hot_water::{StorageLabel, StorageLoss};
use crate::significant_figures::{round_down, round_up};
use crate::solar_shading::{collector_obstruction_factor, CollectorObstruction as Obstruction};
use serde::{Deserialize, Serialize};

/// 13.66: `f_prac;sol`.
pub const PRACTICE_FACTOR: f64 = 0.95;
/// 13.100/13.118: `f_app`.
pub const APPLICATION_FACTOR: f64 = 1.08;
/// Table 13.15 (NEN-EN 15316-4-3 table B.18), water storage: a..f.
pub const CORRELATION: [f64; 6] = [1.029, -0.065, -0.245, 0.0018, 0.0215, 0.0];
/// 13.73: `c_1`, W/(m²·K).
pub const HEAT_EXCHANGER_PER_M2: f64 = 100.0;
/// Table 13.10.
pub const LOOP_LOSS_STANDARD_W_PER_K: f64 = 5.0;
pub const LOOP_LOSS_PER_M2: f64 = 0.5;
/// Table 13.11.
pub const PUMP_STANDARD_W: f64 = 10.0;
pub const PUMP_PER_M2: f64 = 1.0;
/// 13.109/13.127: annual pump hours for hot water only and combi systems.
pub const PUMP_HOURS_WATER: f64 = 1500.0;
pub const PUMP_HOURS_COMBI: f64 = 2000.0;
/// 13.83: `ϑ_W;srv` and `ϑ_W;cw`.
pub const SERVICE_TEMPERATURE_C: f64 = 40.0;
pub const COLD_WATER_C: f64 = 10.0;
/// 13.92/13.110: reference specific volume, l/m².
pub const REFERENCE_VOLUME_L_PER_M2: f64 = 75.0;

pub const INTERPRETATIONS: &[&str] = &[
    "13.7.2.2.1: the vessel stands in a heated room, so ϑ_sto;amb follows 13.69/13.69a and f_rbl = 1 (13.107); the 500 m² rule of 13.13 applies in the hot-water chain",
    "13.69: the levelled setpoint comes from the space-heating chain of the building performance; the run that feeds the hot-water gains of 7.29 uses the heating setpoint",
    "13.129 (method 1) is applied per physical system and per building part as in 13.96: Q_W;sol;us/(f_gebouw;si;W·N_soli)",
    "13.130 is not extrapolated: an annual demand outside the tested range is rejected",
    "13.134: a negative monthly Q_W;ren of an integrated-backup system tested as a whole is set to 0",
    "13.134 (p. 602) is applied as printed: Q_W;ren = Q_W;use − f_dis·Q_W;bu;out, so Σ f_dis below 1 (a worse orientation or more obstruction than south 45° unobstructed) shrinks the backup share and raises the solar yield. This is a defect of the norm; the kernel keeps the formula and reports solar_tested_backup_distribution_below_one",
    "13.77/13.85: Q_H;sol;us is the space-heating node output plus node losses of the chain before solar gains; the hot-water recoverable losses feeding that chain run do not yet include the solar losses (no iteration, cf. the note at 13.77)",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SolarUse {
    /// SOL_USE = WHS.
    WaterHeating,
    /// SOL_USE = COMBI: hot water and space heating.
    Combi,
    /// SOL_USE = SHS: space heating only (`f_H;use = 1`, 13.85).
    SpaceHeating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SolarType {
    /// SOL_TYPE = SER: preheater, the vessel stores solar heat only.
    Preheater,
    /// SOL_TYPE = PAR: integrated backup heating ("hottop").
    IntegratedBackup,
}

/// Table 13.14 collector types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectorType {
    UnglazedOrUnknown,
    Glazed,
    EvacuatedTube,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum CollectorEfficiency {
    /// Table 13.14.
    Forfait { collector: CollectorType },
    /// Product data (NEN-EN ISO 9806).
    Declared {
        #[serde(rename = "eta0")]
        eta0: f64,
        #[serde(rename = "a1WPerM2K")]
        a1_w_per_m2k: f64,
        #[serde(rename = "a2WPerM2K2")]
        a2_w_per_m2k2: f64,
        /// `K_hem(50°)`.
        #[serde(rename = "incidenceAngleModifier")]
        incidence_angle_modifier: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

impl CollectorEfficiency {
    /// (η_o, a1, a2, K_hem(50°)).
    pub fn parameters(&self) -> (f64, f64, f64, f64) {
        match self {
            Self::Forfait { collector } => match collector {
                CollectorType::UnglazedOrUnknown => (0.8, 15.0, 0.0, 1.0),
                CollectorType::Glazed => (0.8, 3.5, 0.0, 0.94),
                CollectorType::EvacuatedTube => (0.8, 1.8, 0.0, 1.0),
            },
            Self::Declared {
                eta0,
                a1_w_per_m2k,
                a2_w_per_m2k2,
                incidence_angle_modifier,
                ..
            } => (
                *eta0,
                *a1_w_per_m2k,
                *a2_w_per_m2k2,
                *incidence_angle_modifier,
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoopPipe {
    pub length_m: f64,
    /// `Ḣ_pipe;i`, W/(m·K), e.g. table 13.4.
    pub psi_w_per_mk: f64,
}

/// `H_loop;p` of the collector circuit.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum LoopPipes {
    /// 13.75 with table 13.10.
    Forfait,
    /// 13.74.
    Pipes {
        pipes: Vec<LoopPipe>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Supplier value, W/K.
    Declared {
        #[serde(rename = "heatLossWPerK")]
        heat_loss_w_per_k: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectorField {
    /// `A_sol;mod`: reference area of one module, m² (table 13.14
    /// forfait: 60 % of the gross area for evacuated tubes, else gross).
    pub module_area_m2: f64,
    /// `N_col;soli`.
    pub module_count: u32,
    pub orientation: Orientation,
    pub tilt_deg: f64,
    /// §17.3 for `F_sh;obst`.
    pub obstruction: Obstruction,
    pub efficiency: CollectorEfficiency,
    /// `H_s;sto;hx`, W/K; omitted: 13.73.
    #[serde(default)]
    pub heat_exchanger_w_per_k: Option<f64>,
    pub loop_pipes: LoopPipes,
    /// `P_sol;pmp` from the supplier, W; omitted: 13.76.
    #[serde(default)]
    pub pump_power_w: Option<f64>,
}

impl CollectorField {
    pub fn area_m2(&self) -> f64 {
        self.module_area_m2 * f64::from(self.module_count)
    }

    /// 13.72 with 13.73.
    pub fn loop_efficiency(&self) -> f64 {
        let (eta0, a1, _, _) = self.efficiency.parameters();
        let area = self.area_m2();
        let exchanger = self
            .heat_exchanger_w_per_k
            .unwrap_or(HEAT_EXCHANGER_PER_M2 * area);
        if exchanger > 0.0 {
            1.0 - eta0 * area * a1 / exchanger
        } else {
            0.9
        }
    }

    /// `H_loop;p`, W/K (13.74/13.75).
    pub fn loop_loss_w_per_k(&self) -> f64 {
        match &self.loop_pipes {
            LoopPipes::Forfait => LOOP_LOSS_STANDARD_W_PER_K + LOOP_LOSS_PER_M2 * self.area_m2(),
            LoopPipes::Pipes { pipes, .. } => pipes
                .iter()
                .map(|pipe| pipe.length_m * pipe.psi_w_per_mk)
                .sum(),
            LoopPipes::Declared {
                heat_loss_w_per_k, ..
            } => *heat_loss_w_per_k,
        }
    }

    /// `P_sol;pmp` rounded up per annex X (13.76, 13.84).
    pub fn pump_power_w(&self) -> f64 {
        round_up(
            self.pump_power_w
                .unwrap_or(PUMP_STANDARD_W + PUMP_PER_M2 * self.area_m2()),
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SolarStorage {
    /// `V_sto;tot`, l.
    pub total_volume_l: f64,
    /// `V_sto;bu`, l; omitted with integrated backup: the forfait of 13.80.
    #[serde(default)]
    pub backup_volume_l: Option<f64>,
    /// `H_sto;ls;tot` (13.81/13.82): measured UA, label or production year.
    pub loss: StorageLoss,
    /// Note 1 at 13.95/13.113: the backup efficiency already includes the
    /// vessel loss, so `Q_bu;sto;ls = 0`.
    #[serde(default)]
    pub backup_loss_in_generator_efficiency: bool,
}

impl SolarStorage {
    /// 13.80/13.88 forfait for `V_sto;bu`.
    pub fn forfait_backup_volume_l(volume_l: f64) -> f64 {
        if volume_l < 80.0 {
            volume_l
        } else if volume_l <= 200.0 {
            80.0
        } else if volume_l >= 300.0 {
            120.0
        } else {
            80.0 + (volume_l - 200.0) / 100.0 * 40.0
        }
    }

    pub fn backup_volume_l(&self, solar_type: SolarType) -> f64 {
        match solar_type {
            SolarType::Preheater => 0.0,
            SolarType::IntegratedBackup => self
                .backup_volume_l
                .unwrap_or_else(|| Self::forfait_backup_volume_l(self.total_volume_l)),
        }
    }

    /// `H_sto;ls;tot`, W/K: UA rounded up per annex X, or 13.82.
    pub fn loss_w_per_k(&self) -> f64 {
        match &self.loss {
            StorageLoss::Measured { .. } | StorageLoss::MeasuredStandby { .. } => {
                self.loss.measured_transmission_w_per_k().unwrap_or(0.0)
            }
            StorageLoss::Label { label } => label.standing_loss_w(self.total_volume_l) / 45.0,
            StorageLoss::UnknownLabel { produced_from_2018 } => {
                let label = if *produced_from_2018 {
                    StorageLabel::C
                } else {
                    StorageLabel::G
                };
                label.standing_loss_w(self.total_volume_l) / 45.0
            }
        }
    }
}

/// One NEN-EN 12976 / 12977-2 result of a system tested as a whole.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SolarTestPoint {
    /// `Q_W;sol;us;an` (Q_D), kWh/year.
    pub annual_demand_kwh: f64,
    /// `Q_W;sol;out;an` (Q_L), kWh/year; required for a preheater.
    #[serde(default)]
    pub solar_output_kwh: Option<f64>,
    /// `Q_W;bu;out;an` (Q_aux;net), kWh/year; required with integrated
    /// backup.
    #[serde(default)]
    pub backup_output_kwh: Option<f64>,
    /// `W_W;sol;aux;an` (Q_par), kWh/year.
    pub auxiliary_kwh: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum SolarMethod {
    /// 13.7.2.2 (NEN-EN 15316-4-3 method 2).
    Calculated {
        #[serde(rename = "solarType")]
        solar_type: SolarType,
        collectors: CollectorField,
        storage: SolarStorage,
    },
    /// 13.7.2.3 (method 1), hot water only.
    Tested {
        #[serde(rename = "solarType")]
        solar_type: SolarType,
        orientation: Orientation,
        #[serde(rename = "tiltDeg")]
        tilt_deg: f64,
        obstruction: Obstruction,
        /// `V_sto;tot` for splitting the demand over different systems, l.
        #[serde(rename = "totalVolumeL")]
        total_volume_l: f64,
        #[serde(rename = "testPoints")]
        test_points: Vec<SolarTestPoint>,
        #[serde(default, rename = "backupLossInGeneratorEfficiency")]
        backup_loss_in_generator_efficiency: bool,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

impl SolarMethod {
    pub fn solar_type(&self) -> SolarType {
        match self {
            Self::Calculated { solar_type, .. } | Self::Tested { solar_type, .. } => *solar_type,
        }
    }

    pub fn total_volume_l(&self) -> f64 {
        match self {
            Self::Calculated { storage, .. } => storage.total_volume_l,
            Self::Tested { total_volume_l, .. } => *total_volume_l,
        }
    }
}

/// 13.7.2.4: PVT collectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PvtCover {
    Unglazed,
    SingleGlazed,
    /// Tested to NEN-EN-ISO 9806: `f_PVT;th = 1`.
    TestedIso9806,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SolarWaterHeater {
    pub id: String,
    pub solar_use: SolarUse,
    /// `N_soli`: identical physical systems modelled together.
    #[serde(default = "one")]
    pub count: u32,
    pub method: SolarMethod,
    #[serde(default)]
    pub pvt: Option<PvtCover>,
    pub source_reference: String,
}

fn one() -> u32 {
    1
}

impl SolarWaterHeater {
    /// `f_PVT;th` (13.66, table 13.16); `None` when single glazing lacks
    /// collector data.
    pub fn pvt_factor(&self) -> Option<f64> {
        match self.pvt {
            None | Some(PvtCover::TestedIso9806) => Some(1.0),
            Some(PvtCover::Unglazed) => Some(0.9),
            Some(PvtCover::SingleGlazed) => match &self.method {
                SolarMethod::Calculated {
                    collectors,
                    storage,
                    ..
                } => {
                    let ratio = collectors.area_m2() / storage.total_volume_l;
                    Some(if ratio < 0.015 {
                        0.76
                    } else if ratio <= 0.03 {
                        0.83
                    } else {
                        0.89
                    })
                }
                SolarMethod::Tested { .. } => None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SolarIssue {
    pub code: &'static str,
    pub path: String,
}

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn obstruction_issue(obstruction: &Obstruction) -> bool {
    match obstruction {
        Obstruction::Declared {
            factors,
            source_reference,
        } => {
            factors.len() != 12
                || factors.iter().any(|value| !(0.0..=1.0).contains(value))
                || source_reference.trim().is_empty()
        }
        // The geometric situations are checked by evaluating them.
        other => (1..=12).any(|month| {
            collector_obstruction_factor(other, Orientation::South, 45.0, month).is_none()
        }),
    }
}

pub fn validate_solar(heater: &SolarWaterHeater, path: &str) -> Vec<SolarIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: &str| {
        issues.push(SolarIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    if heater.id.trim().is_empty() {
        push("id_invalid", "id");
    }
    if heater.count == 0 {
        push("solar_count_invalid", "count");
    }
    if heater.source_reference.trim().is_empty() {
        push("source_reference_required", "sourceReference");
    }
    if heater.pvt_factor().is_none() {
        push("solar_pvt_requires_collector_data", "pvt");
    }
    match &heater.method {
        SolarMethod::Calculated {
            collectors,
            storage,
            solar_type,
        } => {
            if !positive(collectors.module_area_m2) {
                push("solar_area_invalid", "method.collectors.moduleAreaM2");
            }
            if collectors.module_count == 0 {
                push("solar_area_invalid", "method.collectors.moduleCount");
            }
            if !(0.0..=180.0).contains(&collectors.tilt_deg) {
                push("tilt_unsupported", "method.collectors.tiltDeg");
            }
            if obstruction_issue(&collectors.obstruction) {
                push("solar_obstruction_invalid", "method.collectors.obstruction");
            }
            if let CollectorEfficiency::Declared {
                eta0,
                a1_w_per_m2k,
                a2_w_per_m2k2,
                incidence_angle_modifier,
                source_reference,
            } = &collectors.efficiency
            {
                if !(positive(*eta0) && *eta0 <= 1.0)
                    || !a1_w_per_m2k.is_finite()
                    || *a1_w_per_m2k < 0.0
                    || !a2_w_per_m2k2.is_finite()
                    || *a2_w_per_m2k2 < 0.0
                    || !positive(*incidence_angle_modifier)
                {
                    push(
                        "solar_collector_efficiency_invalid",
                        "method.collectors.efficiency",
                    );
                }
                if source_reference.trim().is_empty() {
                    push(
                        "source_reference_required",
                        "method.collectors.efficiency.sourceReference",
                    );
                }
            }
            if collectors
                .heat_exchanger_w_per_k
                .is_some_and(|value| !positive(value))
            {
                push(
                    "solar_heat_exchanger_invalid",
                    "method.collectors.heatExchangerWPerK",
                );
            }
            if collectors.loop_efficiency() <= 0.0 {
                push(
                    "solar_heat_exchanger_invalid",
                    "method.collectors.heatExchangerWPerK",
                );
            }
            match &collectors.loop_pipes {
                LoopPipes::Forfait => {}
                LoopPipes::Pipes {
                    pipes,
                    source_reference,
                } => {
                    if pipes.is_empty()
                        || pipes.iter().any(|pipe| {
                            !positive(pipe.length_m)
                                || !pipe.psi_w_per_mk.is_finite()
                                || pipe.psi_w_per_mk < 0.0
                        })
                    {
                        push("solar_loop_pipes_invalid", "method.collectors.loopPipes");
                    }
                    if source_reference.trim().is_empty() {
                        push(
                            "source_reference_required",
                            "method.collectors.loopPipes.sourceReference",
                        );
                    }
                }
                LoopPipes::Declared {
                    heat_loss_w_per_k,
                    source_reference,
                } => {
                    if !heat_loss_w_per_k.is_finite() || *heat_loss_w_per_k < 0.0 {
                        push("solar_loop_pipes_invalid", "method.collectors.loopPipes");
                    }
                    if source_reference.trim().is_empty() {
                        push(
                            "source_reference_required",
                            "method.collectors.loopPipes.sourceReference",
                        );
                    }
                }
            }
            if collectors
                .pump_power_w
                .is_some_and(|value| !value.is_finite() || value < 0.0)
            {
                push("solar_pump_power_invalid", "method.collectors.pumpPowerW");
            }
            if !positive(storage.total_volume_l) {
                push(
                    "hot_water_storage_volume_invalid",
                    "method.storage.totalVolumeL",
                );
            }
            if let Some(backup) = storage.backup_volume_l {
                if *solar_type == SolarType::Preheater && backup != 0.0 {
                    push(
                        "solar_backup_volume_preheater",
                        "method.storage.backupVolumeL",
                    );
                }
                // 13.80 (2025+C1 p. 581, 2022 p. 562): the forfait itself
                // gives V_sto;bu = V_sto below 80 l, so the backup part may
                // fill the whole vessel; only more than the vessel is invalid.
                if !backup.is_finite() || backup < 0.0 || backup > storage.total_volume_l {
                    push(
                        "hot_water_storage_volume_invalid",
                        "method.storage.backupVolumeL",
                    );
                }
            }
            if let StorageLoss::Measured {
                transmission_w_per_k,
            } = storage.loss
            {
                if !positive(transmission_w_per_k) {
                    push(
                        "hot_water_storage_loss_invalid",
                        "method.storage.loss.transmissionWPerK",
                    );
                }
            }
            // 13.60 needs a positive standby loss and ϑ_sto;set;ref > ϑ_amb;ref.
            if let StorageLoss::MeasuredStandby {
                standby_kwh_per_day,
                reference_storage_c,
                reference_ambient_c,
            } = storage.loss
            {
                if !positive(standby_kwh_per_day)
                    || !reference_ambient_c.is_finite()
                    || !positive(reference_storage_c - reference_ambient_c)
                {
                    push("hot_water_storage_loss_invalid", "method.storage.loss");
                }
            }
        }
        SolarMethod::Tested {
            solar_type,
            tilt_deg,
            obstruction,
            total_volume_l,
            test_points,
            source_reference,
            ..
        } => {
            if heater.solar_use != SolarUse::WaterHeating {
                push("solar_tested_hot_water_only", "solarUse");
            }
            if !(0.0..=180.0).contains(tilt_deg) {
                push("tilt_unsupported", "method.tiltDeg");
            }
            if obstruction_issue(obstruction) {
                push("solar_obstruction_invalid", "method.obstruction");
            }
            if !positive(*total_volume_l) {
                push("hot_water_storage_volume_invalid", "method.totalVolumeL");
            }
            let mut demands: Vec<f64> = test_points
                .iter()
                .map(|point| point.annual_demand_kwh)
                .collect();
            demands.sort_by(f64::total_cmp);
            if test_points.len() < 2 || demands.windows(2).any(|pair| pair[0] >= pair[1]) {
                push("solar_test_points_invalid", "method.testPoints");
            }
            for (index, point) in test_points.iter().enumerate() {
                let value = match solar_type {
                    SolarType::Preheater => point.solar_output_kwh,
                    SolarType::IntegratedBackup => point.backup_output_kwh,
                };
                if !positive(point.annual_demand_kwh)
                    || !value.is_some_and(|value| value.is_finite() && value >= 0.0)
                    || !point.auxiliary_kwh.is_finite()
                    || point.auxiliary_kwh < 0.0
                {
                    push(
                        "solar_test_points_invalid",
                        &format!("method.testPoints[{index}]"),
                    );
                }
            }
            if source_reference.trim().is_empty() {
                push("source_reference_required", "method.sourceReference");
            }
        }
    }
    issues
}

/// Monthly incident irradiance on the collector plane and `F_sh;obst`.
fn plane(orientation: Orientation, tilt_deg: f64, obstruction: &Obstruction) -> [(f64, f64); 12] {
    std::array::from_fn(|index| {
        let month = index as u8 + 1;
        (
            irradiance_w_per_m2(orientation, tilt_deg, month).unwrap_or(0.0),
            // §17.3 collector tables 17.6/17.12/17.15.
            collector_obstruction_factor(obstruction, orientation, tilt_deg, month).unwrap_or(1.0),
        )
    })
}

/// Settings of one service for 13.7.2.2 (tables 13.12/13.13).
#[derive(Debug, Clone, Copy)]
pub struct ServiceSettings {
    /// `Q_X;sol;ls;us;mi` per physical system, kWh (13.96 or 13.114 without
    /// the backup loss, which is added here for space heating).
    pub use_kwh: [f64; 12],
    /// `f_X;use;mi`.
    pub share: [f64; 12],
    /// `ϑ_X;ref;mi`, °C.
    pub reference_c: [f64; 12],
    /// `ϑ_X;low;mi`, °C.
    pub low_c: [f64; 12],
    /// `ϑ_X;high`, °C.
    pub high_c: f64,
    /// `ϑ_X;bu;set`, °C.
    pub backup_set_c: f64,
    /// `ϑ_sto;amb;mi` (13.69/13.69a), °C.
    pub ambient_c: [f64; 12],
    /// `t_X;aux`, h/year.
    pub pump_hours: f64,
    /// Space heating: 13.114 adds `Q_H;bu;sto;ls` to the use.
    pub add_backup_loss_to_use: bool,
}

/// Monthly result of one service per physical system.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceMonth {
    pub x: f64,
    pub y: f64,
    /// `Q_X;sol;tmp;mi`.
    pub first_estimate_kwh: f64,
    /// `Q_X;sol;sto;ls;mi`.
    pub storage_loss_kwh: f64,
    /// `Q_X;bu;sto;ls;mi`.
    pub backup_storage_loss_kwh: f64,
    /// `Q_X;ren;si,mi` rounded down per annex X.
    pub renewable_kwh: f64,
    /// `Q_X;bu;out;mi`.
    pub backup_output_kwh: f64,
    /// `Q_X;sol;ls;rbl;mi`.
    pub recoverable_kwh: f64,
    /// `W_X;aux;sol;mi`.
    pub auxiliary_kwh: f64,
}

/// 13.7.2.2.4 / 13.7.2.2.5 for one service of one physical system.
pub fn calculated_service(
    solar_type: SolarType,
    collectors: &CollectorField,
    storage: &SolarStorage,
    settings: &ServiceSettings,
) -> [ServiceMonth; 12] {
    let (eta0, a1, a2, k_hem) = collectors.efficiency.parameters();
    let eta_loop = collectors.loop_efficiency();
    let loop_loss = collectors.loop_loss_w_per_k();
    let area_total = collectors.area_m2();
    let volume = storage.total_volume_l;
    let backup_volume = storage.backup_volume_l(solar_type);
    let loss_total = storage.loss_w_per_k();
    let pump = collectors.pump_power_w();
    let incident = plane(
        collectors.orientation,
        collectors.tilt_deg,
        &collectors.obstruction,
    );
    let irradiance_sum: f64 = incident.iter().map(|(irradiance, _)| irradiance).sum();
    std::array::from_fn(|index| {
        let share = settings.share[index];
        let base_use = settings.use_kwh[index];
        if base_use <= 0.0 || share <= 0.0 {
            return ServiceMonth::default();
        }
        let hours = MONTH_HOURS[index];
        // 13.78–13.81 / 13.86–13.89.
        let area = share * area_total;
        let total = share * volume;
        let backup = share * backup_volume;
        let loss = share * loss_total;
        // 13.93/13.94 with f_bu = 1.
        let auxiliary_share = if total > 0.0 { backup / total } else { 0.0 };
        let solar_volume = match solar_type {
            SolarType::Preheater => total,
            SolarType::IntegratedBackup => total * (1.0 - auxiliary_share),
        };
        // 13.95/13.113.
        let backup_loss = if storage.backup_loss_in_generator_efficiency || total <= 0.0 {
            0.0
        } else {
            loss * (total - solar_volume) / total
                * (settings.backup_set_c - settings.ambient_c[index])
                * hours
                / 1000.0
        };
        let use_kwh = if settings.add_backup_loss_to_use {
            base_use + backup_loss
        } else {
            base_use
        };
        // 13.92/13.110.
        let storage_factor = (REFERENCE_VOLUME_L_PER_M2 * area / solar_volume).powf(0.25);
        // 13.98/13.116.
        let loop_coefficient = a1 + a2 * 40.0 + loop_loss / area;
        let outdoor = OUTDOOR_TEMPERATURE_C[index];
        // 13.97/13.115.
        let x = (area
            * loop_coefficient
            * eta_loop
            * (settings.reference_c[index] - outdoor)
            * storage_factor
            * hours
            / (use_kwh * 1000.0))
            .clamp(0.0, 18.0);
        // 13.99/13.117.
        let (irradiance, obstruction) = incident[index];
        let y = (area * k_hem * eta0 * eta_loop * irradiance * obstruction * hours
            / (use_kwh * 1000.0))
            .max(0.0);
        // 13.100/13.118 with table 13.15.
        let [a, b, c, d, e, f] = CORRELATION;
        let first = (APPLICATION_FACTOR
            * (a * y + b * x + c * y * y + d * x * x + e * y.powi(3) + f * x.powi(3))
            * use_kwh)
            .max(0.0);
        // 13.101/13.119.
        let fraction = (first / use_kwh).min(1.0);
        // 13.102/13.120.
        let storage_loss = (loss * solar_volume / total
            * (settings.low_c[index] + (settings.high_c - settings.low_c[index]) * fraction
                - settings.ambient_c[index])
            * fraction
            * hours
            / 1000.0)
            .max(0.0);
        // 13.103/13.104 and 13.121/13.122, rounded down per annex X.
        let output = (first - storage_loss).max(0.0);
        let renewable = round_down(output);
        // 13.105/13.123 with the use per building part restored by the
        // caller; here per physical system.
        let backup_output = (base_use - output + backup_loss).max(0.0);
        // 13.106/13.124 with f_bu;ins = 0; 13.107/13.125 with f_rbl = 1.
        let recoverable = storage_loss + backup_loss;
        // 13.108/13.109 and 13.126/13.127.
        let pump_hours = if irradiance_sum > 0.0 {
            irradiance / irradiance_sum * settings.pump_hours
        } else {
            0.0
        };
        ServiceMonth {
            x,
            y,
            first_estimate_kwh: first,
            storage_loss_kwh: storage_loss,
            backup_storage_loss_kwh: backup_loss,
            renewable_kwh: renewable,
            backup_output_kwh: backup_output,
            recoverable_kwh: recoverable,
            auxiliary_kwh: share * pump * pump_hours / 1000.0,
        }
    })
}

/// 13.83.
pub fn water_reference_c(month_index: usize) -> f64 {
    11.6 + 1.18 * SERVICE_TEMPERATURE_C + 3.86 * COLD_WATER_C
        - 1.32 * OUTDOOR_TEMPERATURE_C[month_index]
}

/// 13.90 with `ϑ_H;dis;rtn = ϑ_H,a;ontw − Δϑ_H;ontw`.
pub fn heating_reference_c(return_c: f64) -> f64 {
    0.75 * return_c + 55.0
}

/// 13.130: linear interpolation over the tested annual demands; `None`
/// outside the tested range.
pub fn interpolate_test(points: &[(f64, f64)], demand_kwh: f64) -> Option<f64> {
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
    let first = sorted.first()?;
    let last = sorted.last()?;
    if demand_kwh < first.0 - 1e-9 || demand_kwh > last.0 + 1e-9 {
        return None;
    }
    for pair in sorted.windows(2) {
        let ((x1, y1), (x2, y2)) = (pair[0], pair[1]);
        if demand_kwh <= x2 + 1e-9 {
            return Some(y1 + (demand_kwh - x1) * (y2 - y1) / (x2 - x1));
        }
    }
    Some(last.1)
}

/// 13.128: the monthly distribution factors `f_dis;mi` of a tested system,
/// I_sol·F_sh·t over the annual south-45° irradiation.
pub fn tested_distribution(
    orientation: Orientation,
    tilt_deg: f64,
    obstruction: &Obstruction,
) -> [f64; 12] {
    let incident = plane(orientation, tilt_deg, obstruction);
    // 13.128 with I_sol;s45;an·t_an.
    let reference: f64 = (0..12)
        .map(|index| {
            irradiance_w_per_m2(Orientation::South, 45.0, index as u8 + 1).unwrap_or(0.0)
                * MONTH_HOURS[index]
        })
        .sum();
    std::array::from_fn(|index| {
        let (irradiance, obstruction) = incident[index];
        irradiance * obstruction * MONTH_HOURS[index] / reference
    })
}

/// Σ f_dis below which 13.134 (integrated backup, tested as a whole) gives
/// more renewable heat for a worse orientation or more obstruction.
pub const TESTED_DISTRIBUTION_WARNING: f64 = 1.0 - 1e-6;

/// Whether a solar water heater is an integrated-backup system tested as a
/// whole whose Σ f_dis is below 1 (`solar_tested_backup_distribution_below_one`).
pub fn tested_backup_distribution_below_one(heater: &SolarWaterHeater) -> bool {
    match &heater.method {
        SolarMethod::Tested {
            solar_type: SolarType::IntegratedBackup,
            orientation,
            tilt_deg,
            obstruction,
            ..
        } => {
            tested_distribution(*orientation, *tilt_deg, obstruction)
                .iter()
                .sum::<f64>()
                < TESTED_DISTRIBUTION_WARNING
        }
        _ => false,
    }
}

/// 13.7.2.3 hot water of one physical system tested as a whole; `None`
/// when the annual demand lies outside the tested range.
#[allow(clippy::too_many_arguments)]
pub fn tested_water(
    solar_type: SolarType,
    orientation: Orientation,
    tilt_deg: f64,
    obstruction: &Obstruction,
    test_points: &[SolarTestPoint],
    backup_loss_in_generator_efficiency: bool,
    use_kwh: &[f64; 12],
    hot_water_c: f64,
    ambient_c: [f64; 12],
) -> Option<[ServiceMonth; 12]> {
    let distribution = tested_distribution(orientation, tilt_deg, obstruction);
    let annual: f64 = use_kwh.iter().sum();
    let pick = |value: fn(&SolarTestPoint) -> Option<f64>| -> Option<f64> {
        let points: Option<Vec<(f64, f64)>> = test_points
            .iter()
            .map(|point| value(point).map(|y| (point.annual_demand_kwh, y)))
            .collect();
        interpolate_test(&points?, annual)
    };
    let auxiliary_annual = pick(|point| Some(point.auxiliary_kwh))?;
    let mut months = [ServiceMonth::default(); 12];
    match solar_type {
        SolarType::Preheater => {
            // 13.131/13.132.
            let output = pick(|point| point.solar_output_kwh)?;
            for (index, month) in months.iter_mut().enumerate() {
                month.renewable_kwh = round_down(distribution[index] * output);
                month.backup_output_kwh = use_kwh[index] - month.renewable_kwh;
            }
        }
        SolarType::IntegratedBackup => {
            // 13.133/13.134.
            let backup = pick(|point| point.backup_output_kwh)?;
            for (index, month) in months.iter_mut().enumerate() {
                month.renewable_kwh =
                    round_down((use_kwh[index] - distribution[index] * backup).max(0.0));
                month.backup_output_kwh = use_kwh[index] - month.renewable_kwh;
            }
        }
    }
    // 13.136.
    let backup_annual: f64 = months.iter().map(|month| month.backup_output_kwh).sum();
    let solar_fraction = if annual > 0.0 {
        (annual - backup_annual) / annual
    } else {
        0.0
    };
    for (index, month) in months.iter_mut().enumerate() {
        let hours = MONTH_HOURS[index];
        // 13.137/13.138.
        let standing = (hot_water_c - ambient_c[index])
            * (0.37 + 2.06 * (use_kwh[index] / hours).powf(0.4))
            * hours
            / 1000.0;
        month.storage_loss_kwh = (standing * solar_fraction).max(0.0);
        month.backup_storage_loss_kwh = match solar_type {
            SolarType::IntegratedBackup if !backup_loss_in_generator_efficiency => {
                standing.max(0.0)
            }
            _ => 0.0,
        };
        // 13.139/13.140 with f_bu;ins = 0 and f_rbl = 1.
        month.recoverable_kwh = month.storage_loss_kwh + month.backup_storage_loss_kwh;
        // 13.135.
        month.auxiliary_kwh = distribution[index] * auxiliary_annual;
    }
    Some(months)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> CollectorField {
        CollectorField {
            module_area_m2: 2.0,
            module_count: 2,
            orientation: Orientation::South,
            tilt_deg: 45.0,
            obstruction: Obstruction::Minimal,
            efficiency: CollectorEfficiency::Forfait {
                collector: CollectorType::Glazed,
            },
            heat_exchanger_w_per_k: None,
            loop_pipes: LoopPipes::Forfait,
            pump_power_w: None,
        }
    }

    #[test]
    fn collector_parameters_follow_13_72_to_13_76() {
        let field = field();
        // 13.73: H_hx = 100·4 = 400 → η_loop = 1 − 0,8·4·3,5/400 = 0,972.
        assert!((field.loop_efficiency() - 0.972).abs() < 1e-12);
        // 13.75: 5 + 0,5·4 = 7 W/K.
        assert!((field.loop_loss_w_per_k() - 7.0).abs() < 1e-12);
        // 13.76: 10 + 4 = 14 W, rounded up per annex X to 15 W.
        assert!((field.pump_power_w() - 15.0).abs() < 1e-12);
    }

    #[test]
    fn backup_volume_forfait_of_13_80() {
        assert_eq!(SolarStorage::forfait_backup_volume_l(60.0), 60.0);
        assert_eq!(SolarStorage::forfait_backup_volume_l(150.0), 80.0);
        assert_eq!(SolarStorage::forfait_backup_volume_l(250.0), 100.0);
        assert_eq!(SolarStorage::forfait_backup_volume_l(400.0), 120.0);
    }

    /// 13.80 (2025+C1 p. 581): the forfait gives V_sto;bu = V_sto below
    /// 80 l, so a declared backup part filling the whole vessel is valid;
    /// a backup part larger than the vessel is not.
    #[test]
    fn backup_volume_may_fill_the_whole_vessel() {
        let heater = |backup: f64| SolarWaterHeater {
            id: "zb".into(),
            solar_use: SolarUse::WaterHeating,
            count: 1,
            method: SolarMethod::Calculated {
                solar_type: SolarType::IntegratedBackup,
                collectors: field(),
                storage: SolarStorage {
                    total_volume_l: 150.0,
                    backup_volume_l: Some(backup),
                    loss: StorageLoss::Measured {
                        transmission_w_per_k: 0.5,
                    },
                    backup_loss_in_generator_efficiency: false,
                },
            },
            pvt: None,
            source_reference: "ISSO 54 v2.0 EP-W405d".into(),
        };
        let codes = |backup| -> Vec<&str> {
            validate_solar(&heater(backup), "solar[0]")
                .iter()
                .map(|issue| issue.code)
                .collect()
        };
        assert!(!codes(150.0).contains(&"hot_water_storage_volume_invalid"));
        assert!(codes(150.1).contains(&"hot_water_storage_volume_invalid"));
    }

    #[test]
    fn hot_water_service_reproduces_a_hand_calculation() {
        let collectors = field();
        let storage = SolarStorage {
            total_volume_l: 200.0,
            backup_volume_l: None,
            loss: StorageLoss::Measured {
                transmission_w_per_k: 1.5,
            },
            backup_loss_in_generator_efficiency: false,
        };
        let settings = ServiceSettings {
            use_kwh: [200.0; 12],
            share: [1.0; 12],
            reference_c: std::array::from_fn(water_reference_c),
            low_c: [COLD_WATER_C; 12],
            high_c: 60.0,
            backup_set_c: 60.0,
            ambient_c: [20.0; 12],
            pump_hours: PUMP_HOURS_WATER,
            add_backup_loss_to_use: false,
        };
        let months = calculated_service(SolarType::Preheater, &collectors, &storage, &settings);
        // July by hand.
        let index = 6;
        let hours = MONTH_HOURS[index];
        let area = 4.0;
        let eta_loop = 0.972;
        let f_sto = (75.0 * area / 200.0_f64).powf(0.25);
        let h_loop = 3.5 + 7.0 / area;
        let reference = 11.6 + 1.18 * 40.0 + 3.86 * 10.0 - 1.32 * OUTDOOR_TEMPERATURE_C[index];
        let x =
            (area * h_loop * eta_loop * (reference - OUTDOOR_TEMPERATURE_C[index]) * f_sto * hours
                / 200_000.0)
                .clamp(0.0, 18.0);
        let irradiance = irradiance_w_per_m2(Orientation::South, 45.0, 7).unwrap();
        let shading =
            collector_obstruction_factor(&Obstruction::Minimal, Orientation::South, 45.0, 7)
                .unwrap();
        let y = area * 0.94 * 0.8 * eta_loop * irradiance * shading * hours / 200_000.0;
        let first = 1.08
            * (1.029 * y - 0.065 * x - 0.245 * y * y + 0.0018 * x * x + 0.0215 * y.powi(3))
            * 200.0;
        let fraction = (first / 200.0).min(1.0);
        // H_sto;ls 1,5 → 1,6 (annex X, rounded up).
        let loss = 1.6 * (10.0 + 50.0 * fraction - 20.0) * fraction * hours / 1000.0;
        let july = months[index];
        assert!((july.x - x).abs() < 1e-9);
        assert!((july.y - y).abs() < 1e-9);
        assert!((july.first_estimate_kwh - first).abs() < 1e-9);
        assert!((july.storage_loss_kwh - loss.max(0.0)).abs() < 1e-9);
        assert_eq!(july.renewable_kwh, round_down(first - loss.max(0.0)));
        assert_eq!(july.backup_storage_loss_kwh, 0.0);
        // Pump: 15 W over 1500 h distributed by irradiance.
        let total: f64 = months.iter().map(|month| month.auxiliary_kwh).sum();
        assert!((total - 15.0 * 1500.0 / 1000.0).abs() < 1e-9);
    }

    #[test]
    fn measured_standby_loss_is_validated_for_solar_storage() {
        // 13.60 with ϑ_sto;set;ref ≤ ϑ_amb;ref has no finite positive H_sto;ls.
        let loss = StorageLoss::MeasuredStandby {
            standby_kwh_per_day: 1.0,
            reference_storage_c: 20.0,
            reference_ambient_c: 20.0,
        };
        assert_eq!(loss.measured_transmission_w_per_k(), None);
        let heater = SolarWaterHeater {
            id: "sol".into(),
            solar_use: SolarUse::WaterHeating,
            count: 1,
            method: SolarMethod::Calculated {
                collectors: field(),
                storage: SolarStorage {
                    total_volume_l: 200.0,
                    backup_volume_l: None,
                    loss,
                    backup_loss_in_generator_efficiency: false,
                },
                solar_type: SolarType::Preheater,
            },
            pvt: None,
            source_reference: "test".into(),
        };
        assert!(validate_solar(&heater, "solar")
            .iter()
            .any(|issue| issue.code == "hot_water_storage_loss_invalid"
                && issue.path == "solar.method.storage.loss"));
    }

    #[test]
    fn integrated_backup_loses_heat_from_the_backup_part() {
        let collectors = field();
        let storage = SolarStorage {
            total_volume_l: 250.0,
            backup_volume_l: None,
            loss: StorageLoss::Label {
                label: StorageLabel::B,
            },
            backup_loss_in_generator_efficiency: false,
        };
        let settings = ServiceSettings {
            use_kwh: [200.0; 12],
            share: [1.0; 12],
            reference_c: std::array::from_fn(water_reference_c),
            low_c: [COLD_WATER_C; 12],
            high_c: 60.0,
            backup_set_c: 60.0,
            ambient_c: [20.0; 12],
            pump_hours: PUMP_HOURS_WATER,
            add_backup_loss_to_use: false,
        };
        let months = calculated_service(
            SolarType::IntegratedBackup,
            &collectors,
            &storage,
            &settings,
        );
        // 13.82 label B: S = 10,25 + 5,09·250^0,4; V_bu = 100 l (13.80).
        let h = (10.25 + 5.09 * 250.0_f64.powf(0.4)) / 45.0;
        let expected = h * 100.0 / 250.0 * 40.0 * MONTH_HOURS[0] / 1000.0;
        assert!((months[0].backup_storage_loss_kwh - expected).abs() < 1e-9);
    }

    #[test]
    fn tested_backup_flags_a_distribution_below_one() {
        let heater = |solar_type, obstruction| SolarWaterHeater {
            id: "sol".into(),
            solar_use: SolarUse::WaterHeating,
            count: 1,
            method: SolarMethod::Tested {
                solar_type,
                orientation: Orientation::South,
                tilt_deg: 37.5,
                obstruction,
                total_volume_l: 200.0,
                test_points: Vec::new(),
                backup_loss_in_generator_efficiency: false,
                source_reference: "test".into(),
            },
            pvt: None,
            source_reference: "test".into(),
        };
        // South 45° unobstructed: Σ f_dis = 1, no flag.
        let open = SolarWaterHeater {
            method: SolarMethod::Tested {
                solar_type: SolarType::IntegratedBackup,
                orientation: Orientation::South,
                tilt_deg: 45.0,
                obstruction: Obstruction::Minimal,
                total_volume_l: 200.0,
                test_points: Vec::new(),
                backup_loss_in_generator_efficiency: false,
                source_reference: "test".into(),
            },
            ..heater(SolarType::IntegratedBackup, Obstruction::Minimal)
        };
        assert!(!tested_backup_distribution_below_one(&open));
        // Fully obstructed: Σ f_dis < 1 raises Q_W;ren by 13.134.
        let full = heater(SolarType::IntegratedBackup, Obstruction::Full);
        let distribution = tested_distribution(Orientation::South, 37.5, &Obstruction::Full);
        assert!(distribution.iter().sum::<f64>() < 1.0);
        assert!(tested_backup_distribution_below_one(&full));
        // A preheater (13.131) scales its output down instead: not flagged.
        assert!(!tested_backup_distribution_below_one(&heater(
            SolarType::Preheater,
            Obstruction::Full
        )));
    }

    #[test]
    fn tested_preheater_interpolates_and_distributes() {
        let points = vec![
            SolarTestPoint {
                annual_demand_kwh: 1000.0,
                solar_output_kwh: Some(500.0),
                backup_output_kwh: None,
                auxiliary_kwh: 30.0,
            },
            SolarTestPoint {
                annual_demand_kwh: 3000.0,
                solar_output_kwh: Some(900.0),
                backup_output_kwh: None,
                auxiliary_kwh: 40.0,
            },
        ];
        let use_kwh = [2000.0 / 12.0; 12];
        let open = Obstruction::Declared {
            factors: vec![1.0; 12],
            source_reference: "free horizon".into(),
        };
        let months = tested_water(
            SolarType::Preheater,
            Orientation::South,
            45.0,
            &open,
            &points,
            false,
            &use_kwh,
            60.0,
            [20.0; 12],
        )
        .unwrap();
        // 13.130: Q_L at 2000 kWh = 700; south 45° gives Σ f_dis = 1.
        let total: f64 = months.iter().map(|month| month.auxiliary_kwh).sum();
        assert!((total - 35.0).abs() < 1e-9);
        let reference: f64 = (0..12)
            .map(|index| {
                irradiance_w_per_m2(Orientation::South, 45.0, index as u8 + 1).unwrap()
                    * MONTH_HOURS[index]
            })
            .sum();
        let july =
            irradiance_w_per_m2(Orientation::South, 45.0, 7).unwrap() * MONTH_HOURS[6] / reference;
        assert_eq!(months[6].renewable_kwh, round_down(july * 700.0));
        assert!(tested_water(
            SolarType::Preheater,
            Orientation::South,
            45.0,
            &Obstruction::Minimal,
            &points,
            false,
            &[500.0; 12],
            60.0,
            [20.0; 12],
        )
        .is_none());
    }
}
