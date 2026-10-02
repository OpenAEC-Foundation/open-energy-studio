//! Space cooling, NTA 8800:2025+C1:2026 chapter 10 (pages 366–426), from
//! the chapter 7 cooling need of the zones served by one cooling system.
//!
//! The chain is additive (10.5/10.7–10.9): `Q_C;gen;in = Σ_zi (Q_C;nd +
//! Q_C;em;ls + Q_C;dis;ls + Q_C;dis;rvd) − Q_C;HP`, where
//! - the emission loss follows 10.10/10.11, 10.15/10.16 with tables 10.35,
//!   10.4, 10.5 and 10.5a (`MAX(…; 0,15)` as printed on p. 376);
//! - the operating hours `t_C;mi` follow the cooling limit of 10.19 steps 1–6
//!   and table 10.6;
//! - a water-based distribution loses 10.21/10.22 in unconditioned spaces
//!   (cooled spaces have `L = 0`), with `L_si = 0,64·A_g` (10.27), table 10.9
//!   and the mean water temperature of table 10.8; pump energy follows
//!   10.28–10.38 with tables 10.10–10.13 and recovered pump heat 10.45;
//!   direct expansion has no distribution loss;
//! - fan-coil fans use 10.17/10.18 with table 10.7;
//! - the cold is split over generators by the priority of table 10.15 and
//!   the β/f(β) rule of 10.49–10.52 with table 10.16;
//! - generation uses method 3 (§10.5.6): table 10.29 compression (EER 3,00,
//!   gas engine 3,00·η_ge with ε_chp;el of table 9.31), table 10.30
//!   absorption (gas 0,80; external heat 0,70·η_dh; CHP 1,00·ε_chp;th),
//!   external cold (10.78), free cooling with table 10.34 (10.86) and the
//!   aquifer/ground regeneration surcharge 10.84/10.85; declared values are
//!   rounded down to 0,05 (electric) or 0,025 (gas-fired), §10.1;
//! - generator auxiliaries follow 10.79: no condenser fans in method 3
//!   (10.5.7.1), condenser-water distribution 10.83 with table 10.33 for
//!   water-cooled machines, control 0,010 kW for every month (10.87); free
//!   cooling counts pump energy only (§10.5.7.2.1);
//! - ambient cold (5.34) is the cold of free cooling with `EER ≥ 8`.
//!
//! Not modelled (zero): AHU cooling `Q_C;ahu;in;req` (chapter 11), the
//! dehumidification need `Q_C;dhum` (chapter 12), methods 1 and 2 (EN 14825 /
//! EN 14511 data), recoverable distribution losses (zero because `L_C;zi = 0`
//! in cooled zones) and the supply-air term of 10.20 in the cooling limit.

use crate::climate::{MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use serde::{Deserialize, Serialize};

/// Table 10.29: forfait EER of an electric compression chiller.
pub const EER_COMPRESSION_FORFAIT: f64 = 3.0;
/// Table 10.30: forfait heat ratio of gas-fired absorption cooling.
pub const ZETA_GAS_ABSORPTION: f64 = 0.8;
/// Table 10.30: absorption chiller on external heat, times η_H;gen;equiv;dh.
pub const ZETA_EXTERNAL_HEAT_FACTOR: f64 = 0.7;
/// 5.34: minimum EER for ambient cold to count as renewable.
pub const RENCOLD_MIN_EER: f64 = 8.0;
/// 10.10: initial internal temperature for cooling, °C.
pub const COOLING_INITIAL_TEMPERATURE_C: f64 = 24.0;
/// Table 10.7: electric power per fan coil, W.
pub const FAN_COIL_POWER_W: f64 = 10.0;
/// 10.87: control power of a cooling generation system, kW.
pub const CONTROL_POWER_KW: f64 = 0.010;
/// 10.27: forfait pipe length per m² usable floor area.
pub const PIPE_LENGTH_PER_M2: f64 = 0.64;
/// 10.21: share of pipe length in unconditioned spaces when unknown.
pub const UNCONDITIONED_PIPE_SHARE: f64 = 0.15;
/// 10.23b: unconditioned-space temperature without an AOR, °C.
pub const UNCONDITIONED_AMBIENT_C: f64 = 19.0;
/// Table 10.14 / 10.45: recoverable share of pump energy.
pub const PUMP_RECOVERABLE_FACTOR: f64 = 0.10;
/// 10.84/10.85: regeneration threshold and efficiency number.
pub const REGENERATION_SHARE: f64 = 0.7;
pub const REGENERATION_EFFICIENCY: f64 = 10.0;
/// 10.32: water properties.
const WATER_CP_KJ_PER_KGK: f64 = 4.2;
const WATER_DENSITY_KG_PER_M3: f64 = 1000.0;

/// Table 10.6: operating hours per month for cooling limits 14 (and lower)
/// up to 25 (and higher) °C.
const OPERATING_HOURS: [[f64; 12]; 12] = [
    [
        0.0, 35.0, 35.0, 102.0, 367.0, 546.0, 666.0, 658.0, 535.0, 178.0, 19.0, 0.0,
    ],
    [
        0.0, 20.0, 28.0, 83.0, 301.0, 475.0, 603.0, 625.0, 462.0, 156.0, 1.0, 0.0,
    ],
    [
        0.0, 11.0, 19.0, 66.0, 253.0, 394.0, 525.0, 572.0, 369.0, 126.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 11.0, 54.0, 214.0, 306.0, 449.0, 514.0, 276.0, 74.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 4.0, 33.0, 183.0, 216.0, 376.0, 450.0, 198.0, 51.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 22.0, 155.0, 159.0, 295.0, 353.0, 134.0, 28.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 12.0, 140.0, 119.0, 229.0, 269.0, 97.0, 14.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 126.0, 86.0, 183.0, 199.0, 77.0, 9.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 116.0, 67.0, 143.0, 149.0, 63.0, 3.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 104.0, 50.0, 111.0, 109.0, 40.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 88.0, 40.0, 91.0, 87.0, 29.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 61.0, 34.0, 71.0, 74.0, 17.0, 0.0, 0.0, 0.0,
    ],
];

/// Table 10.6 hours for an (integer, clamped) cooling limit.
pub fn operating_hours(cooling_limit_c: f64, month_index: usize) -> f64 {
    let row = (cooling_limit_c.round().clamp(14.0, 25.0) - 14.0) as usize;
    OPERATING_HOURS[row][month_index]
}

/// 10.19 steps 2–6: least-squares line of the monthly cooling need against
/// the outdoor temperature over months with at least 10 % of the maximum;
/// its zero crossing, rounded and clamped to 14–25 °C. With fewer than two
/// usable months or a non-rising line the limit is 25 °C.
pub fn cooling_limit_c(monthly_need_kwh: &[f64; 12]) -> f64 {
    let max = monthly_need_kwh.iter().copied().fold(0.0, f64::max);
    if max <= 0.0 {
        return 25.0;
    }
    let points: Vec<(f64, f64)> = monthly_need_kwh
        .iter()
        .enumerate()
        .filter(|(_, need)| **need >= 0.1 * max)
        .map(|(index, need)| (OUTDOOR_TEMPERATURE_C[index], *need))
        .collect();
    if points.len() < 2 {
        return 25.0;
    }
    let n = points.len() as f64;
    let mean_x = points.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y = points.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|p| (p.0 - mean_x).powi(2)).sum();
    let sxy: f64 = points.iter().map(|p| (p.0 - mean_x) * (p.1 - mean_y)).sum();
    if sxx <= 0.0 || sxy <= 0.0 {
        return 25.0;
    }
    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    (-intercept / slope).round().clamp(14.0, 25.0)
}

/// Table 10.35 emitter, also deciding radiant (no dehumidification, 17/21
/// design temperatures, 4,5 kPa) versus fan-coil (2 kPa) resistance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingEmitter {
    FloorCooling,
    WallCooling,
    FanCoilOrRacOnOuterWall,
    CeilingCooling,
    FanCoilOrRacOnCeiling,
    OtherOrUnknown,
}

impl CoolingEmitter {
    /// Table 10.35: Δϑ_str + Δϑ_emb + Δϑ_rad + Δϑ_im, K.
    pub fn delta(self) -> f64 {
        match self {
            Self::FloorCooling => -1.7,
            Self::WallCooling | Self::FanCoilOrRacOnOuterWall => -1.4,
            Self::CeilingCooling | Self::FanCoilOrRacOnCeiling | Self::OtherOrUnknown => -0.5,
        }
    }

    pub fn radiant(self) -> bool {
        matches!(
            self,
            Self::FloorCooling | Self::WallCooling | Self::CeilingCooling
        )
    }

    fn fan_coil(self) -> bool {
        matches!(
            self,
            Self::FanCoilOrRacOnOuterWall | Self::FanCoilOrRacOnCeiling
        )
    }
}

/// Table 10.4; `NotApplicable` for room air conditioners and air systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingBalancing {
    NoneOrUnknown,
    Static,
    Dynamic,
    NotApplicable,
}

impl CoolingBalancing {
    pub fn delta(self) -> f64 {
        match self {
            Self::NoneOrUnknown => -0.6,
            Self::Static => -0.4,
            Self::Dynamic | Self::NotApplicable => 0.0,
        }
    }
}

/// Table 10.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingControl {
    UnknownOrOther,
    StandalonePerRoom,
    CentralWithRoomControl,
}

impl CoolingControl {
    pub fn delta(self) -> f64 {
        match self {
            Self::UnknownOrOther => -2.5,
            Self::StandalonePerRoom => -1.25,
            Self::CentralWithRoomControl => -0.75,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoolingEmission {
    pub emitter: CoolingEmitter,
    pub balancing: CoolingBalancing,
    pub control: CoolingControl,
    /// `n_fan` of 10.18; 0 without fan coils.
    #[serde(default)]
    pub fan_coil_count: u32,
    pub source_reference: String,
}

impl CoolingEmission {
    /// 10.11: Δϑ_int;inc.
    pub fn delta_internal(&self) -> f64 {
        self.emitter.delta() + self.balancing.delta() + self.control.delta()
    }
}

/// Table 10.8 design temperatures (in/out).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingDesignTemperature {
    /// 6/12 °C, also for an unknown design.
    T6To12OrUnknown,
    T12To16,
    T12To18,
    T17To21,
}

impl CoolingDesignTemperature {
    pub fn supply_return_c(self) -> (f64, f64) {
        match self {
            Self::T6To12OrUnknown => (6.0, 12.0),
            Self::T12To16 => (12.0, 16.0),
            Self::T12To18 => (12.0, 18.0),
            Self::T17To21 => (17.0, 21.0),
        }
    }
}

/// Table 10.9 (or a calculated Ψ per 10.24–10.26).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CoolingPipe {
    InsulatedFrom1995,
    Insulated1980To1995,
    InsulatedBefore1980OrUnknownAge,
    Uninsulated,
    Declared {
        #[serde(rename = "psiWPerMK")]
        psi_w_per_mk: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

impl CoolingPipe {
    /// Ψ in W/(m·K); uninsulated pipes by the building area of table 10.9.
    pub fn psi(&self, building_area_m2: f64) -> f64 {
        match self {
            Self::InsulatedFrom1995 => 0.3,
            Self::Insulated1980To1995 | Self::InsulatedBefore1980OrUnknownAge => 0.4,
            Self::Uninsulated if building_area_m2 <= 200.0 => 1.0,
            Self::Uninsulated if building_area_m2 <= 500.0 => 2.0,
            Self::Uninsulated => 3.0,
            Self::Declared { psi_w_per_mk, .. } => *psi_w_per_mk,
        }
    }
}

/// §10.4.2.4: a separate distribution pump (not part of the generator
/// auxiliary energy).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoolingPump {
    /// Table 10.11: a balancing declaration per NEN-EN 14336.
    pub hydraulically_balanced: bool,
    /// `n_si` of 10.27, at least 1.
    pub floor_count: u32,
    /// Table 10.12 heat meter in the circuit.
    pub heat_meter: bool,
    /// Table 10.10: dwelling with an individual installation (0,10).
    pub individual_dwelling_installation: bool,
    /// 10.33: power on the pump energy label, kW.
    #[serde(default)]
    pub label_power_kw: Option<f64>,
    /// EEI per EU 622/2012; default 0,23 / 0,25.
    #[serde(default)]
    pub energy_efficiency_index: Option<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoolingDistribution {
    pub design_temperature: CoolingDesignTemperature,
    pub pipe: CoolingPipe,
    pub fittings_insulated: bool,
    /// Actual total pipe length; omitted means `0,64·A_g` (10.27).
    #[serde(default)]
    pub pipe_length_m: Option<f64>,
    /// Pipe length in unconditioned spaces; omitted means 15 %.
    #[serde(default)]
    pub unconditioned_pipe_length_m: Option<f64>,
    /// `ϑ_ztu` of the unconditioned space; omitted means 19 °C (10.23b).
    #[serde(default)]
    pub unconditioned_ambient_c: Option<f64>,
    /// Omitted when the pump energy is part of the generator auxiliaries.
    #[serde(default)]
    pub pump: Option<CoolingPump>,
    pub source_reference: String,
}

/// Free-cooling sources of table 10.34.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FreeCoolingSource {
    /// Aquifer storage realised or permitted from 2013 (all buildings).
    AquiferFrom2013,
    AquiferUtilityBefore2013,
    AquiferDwellingsBefore2013,
    SurfaceWater,
    ClosedGroundLoop,
    /// Footnote b: only with the actually installed fan power of §11.4.
    DewPointCooling,
}

impl FreeCoolingSource {
    /// Table 10.34.
    pub fn eer(self) -> f64 {
        match self {
            Self::AquiferFrom2013 => 23.0,
            Self::AquiferUtilityBefore2013 => 16.0,
            Self::AquiferDwellingsBefore2013 => 14.0,
            Self::SurfaceWater | Self::ClosedGroundLoop => 10.0,
            Self::DewPointCooling => 8.0,
        }
    }

    /// Table 10.15 priority (1 = highest).
    fn priority(self) -> u8 {
        match self {
            Self::AquiferFrom2013
            | Self::AquiferUtilityBefore2013
            | Self::AquiferDwellingsBefore2013
            | Self::ClosedGroundLoop => 1,
            Self::SurfaceWater => 2,
            Self::DewPointCooling => 3,
        }
    }

    /// 10.84: ground storage that a heat pump may deplete.
    fn ground_storage(self) -> bool {
        matches!(
            self,
            Self::AquiferFrom2013
                | Self::AquiferUtilityBefore2013
                | Self::AquiferDwellingsBefore2013
                | Self::ClosedGroundLoop
        )
    }
}

/// Table 10.31/10.33 condenser heat rejection of a compression or
/// absorption machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatRejection {
    /// Direct condensation against outside air (RAC, air-cooled chiller).
    AirCooled,
    ClosedCoolingTower,
    OpenCoolingTower,
    DryCooler,
    GroundStorage,
    SurfaceWater,
}

impl HeatRejection {
    /// Table 10.33 `p_dis;el`, kW/kW.
    pub fn distribution_power(self) -> f64 {
        match self {
            Self::AirCooled => 0.0,
            Self::ClosedCoolingTower => 0.033,
            Self::OpenCoolingTower | Self::SurfaceWater => 0.018,
            Self::DryCooler | Self::GroundStorage => 0.045,
        }
    }
}

/// Table 9.31 `ε_chp;el` / `ε_chp;th` row selection.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChpClass {
    /// Electric (gas engine: mechanical) power, kW.
    pub power_kw: f64,
    pub built_after_2006: bool,
    /// ≤ 2 kW units with an HRe declaration.
    #[serde(default)]
    pub hre_declared: bool,
    /// Low-temperature heat delivery (table 9.26); default high temperature.
    #[serde(default)]
    pub low_temperature: bool,
}

impl ChpClass {
    /// Table 9.31: (ε_chp;th, ε_chp;el); `None` where the table has no value.
    pub fn factors(&self) -> Option<(f64, f64)> {
        let p = self.power_kw;
        if !p.is_finite() || p <= 0.0 || p > 25_000.0 {
            return None;
        }
        if p <= 2.0 {
            if !self.built_after_2006 {
                return None;
            }
            return Some(if self.hre_declared {
                (0.83, 0.10)
            } else {
                (0.86, 0.05)
            });
        }
        // (old th, old el, new th LT, new th HT, new el)
        let row = if p <= 20.0 {
            (0.57, 0.26, 0.57, 0.55, 0.28)
        } else if p <= 200.0 {
            (0.54, 0.27, 0.51, 0.49, 0.30)
        } else if p <= 500.0 {
            (0.50, 0.32, 0.52, 0.50, 0.32)
        } else if p <= 1000.0 {
            (0.44, 0.35, 0.46, 0.44, 0.35)
        } else {
            (0.40, 0.36, 0.41, 0.39, 0.37)
        };
        Some(if !self.built_after_2006 {
            (row.0, row.1)
        } else if self.low_temperature {
            (row.2, row.4)
        } else {
            (row.3, row.4)
        })
    }
}

/// A quality-statement value replacing a table value (§10.1).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredEfficiency {
    pub value: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CoolingGeneratorKind {
    /// Table 10.29 electric compression chiller (central).
    Compression {
        #[serde(default, rename = "heatRejection")]
        heat_rejection: Option<HeatRejection>,
        #[serde(default)]
        declared: Option<DeclaredEfficiency>,
    },
    /// Table 10.29 room air conditioner (local, lowest priority).
    RoomAirConditioner {
        #[serde(default)]
        declared: Option<DeclaredEfficiency>,
    },
    /// Table 10.29: unknown generator in a collective installation; no
    /// generator auxiliaries (§10.5.7).
    UnknownCollective,
    /// Table 10.29: gas-engine compression, EER 3,00·η_ge.
    GasEngineCompression {
        #[serde(rename = "gasEngine")]
        gas_engine: ChpClass,
        #[serde(default, rename = "heatRejection")]
        heat_rejection: Option<HeatRejection>,
    },
    /// Table 10.30 gas-fired absorption, ζ 0,80.
    GasAbsorption {
        #[serde(default, rename = "heatRejection")]
        heat_rejection: Option<HeatRejection>,
        #[serde(default)]
        declared: Option<DeclaredEfficiency>,
    },
    /// Table 10.30 absorption on forfait external heat (η_dh = 1).
    AbsorptionExternalHeat {
        #[serde(default, rename = "heatRejection")]
        heat_rejection: Option<HeatRejection>,
    },
    /// Table 10.30 absorption on building CHP heat, ζ = ε_chp;th.
    AbsorptionChp {
        chp: ChpClass,
        #[serde(default, rename = "heatRejection")]
        heat_rejection: Option<HeatRejection>,
    },
    /// 10.78 external cold without an annex P declaration.
    ExternalCold,
    /// Table 10.34.
    FreeCooling {
        source: FreeCoolingSource,
        /// 10.84: a heat pump uses this ground storage as its source.
        #[serde(default, rename = "heatPumpSource")]
        heat_pump_source: bool,
        /// p. 423: the ground stays above 0 °C with free-cooling heat only.
        #[serde(default, rename = "groundAboveZeroDemonstrated")]
        ground_above_zero_demonstrated: bool,
    },
}

impl CoolingGeneratorKind {
    /// Table 10.15 priority (1 = highest).
    fn priority(&self) -> u8 {
        match self {
            Self::FreeCooling { source, .. } => source.priority(),
            Self::GasAbsorption { .. }
            | Self::AbsorptionExternalHeat { .. }
            | Self::AbsorptionChp { .. } => 4,
            // External cold is not in table 10.15; it is ranked with the
            // central generators (interpretation, see the verification file).
            Self::Compression { .. }
            | Self::UnknownCollective
            | Self::GasEngineCompression { .. }
            | Self::ExternalCold => 5,
            Self::RoomAirConditioner { .. } => 6,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoolingGenerator {
    pub id: String,
    pub generator: CoolingGeneratorKind,
    /// Nominal cooling capacity, kW; required with more than one priority.
    #[serde(default)]
    pub capacity_kw: Option<f64>,
    pub equipment_reference: String,
}

/// Collective installation for part of a building (f_gebouw;C, p. 368).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectiveCooling {
    pub building_usable_floor_area_m2: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoolingSystem {
    pub emission: CoolingEmission,
    /// Water-based distribution; omitted for direct expansion.
    #[serde(default)]
    pub distribution: Option<CoolingDistribution>,
    pub generators: Vec<CoolingGenerator>,
    /// `Q_C;HP;si` of 10.6 (annex W) per month, kWh; empty means none.
    #[serde(default)]
    pub booster_heat_pump_extraction_kwh: Vec<f64>,
    #[serde(default)]
    pub collective: Option<CollectiveCooling>,
}

/// One cooled zone: usable floor area and monthly need `Q_C;nd`.
#[derive(Debug, Clone, Copy)]
pub struct CoolingZoneNeed {
    pub usable_floor_area_m2: f64,
    pub need_kwh: [f64; 12],
}

/// Context from outside chapter 10.
#[derive(Debug, Clone, Copy)]
pub struct CoolingContext<'a> {
    pub zones: &'a [CoolingZoneNeed],
    pub residential: bool,
    /// 10.84: Σ(Q_H;gen;out − E_H;gen;in) of heat pumps on the ground
    /// storage per month, kWh.
    pub heat_pump_source_extraction_kwh: [f64; 12],
}

#[derive(Debug, Clone, Serialize)]
pub struct CoolingIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoolingMonth {
    pub month: u8,
    pub operating_hours: f64,
    pub need_kwh: f64,
    pub emission_loss_kwh: f64,
    pub distribution_loss_kwh: f64,
    pub pump_recovered_kwh: f64,
    pub booster_extraction_kwh: f64,
    /// `Q_C;gen;in`.
    pub generator_cold_kwh: f64,
    /// Drive energy of compression chillers and free-cooling pumps.
    pub electricity_kwh: f64,
    pub natural_gas_kwh: f64,
    /// Heat from external delivery for absorption chillers.
    pub district_heat_kwh: f64,
    /// Heat from building CHP for absorption chillers.
    pub chp_heat_kwh: f64,
    /// External cold delivery (carrier dc).
    pub district_cold_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    pub ambient_cold_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorShare {
    pub id: String,
    pub priority: u8,
    /// `β_C;gen;si;pref` rounded up to 0,1.
    pub beta: f64,
    pub share_july_to_september: f64,
    pub share_other_months: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoolingAssessment {
    pub cooling_limit_c: f64,
    pub internal_temperature_shift_k: f64,
    pub generator_shares: Vec<GeneratorShare>,
    pub months: Vec<CoolingMonth>,
    /// `Q_C;HP;zi` per zone and month (10.6), for TOjuli (5.41c).
    pub zone_booster_extraction_kwh: Vec<[f64; 12]>,
}

fn round_down(value: f64, step: f64) -> f64 {
    (value / step + 1e-9).floor() * step
}

fn source_missing(value: &str) -> bool {
    value.trim().is_empty()
}

pub fn validate_cooling(system: &CoolingSystem, path: &str) -> Vec<CoolingIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: String| {
        issues.push(CoolingIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    if source_missing(&system.emission.source_reference) {
        push(
            "source_reference_required",
            "emission.sourceReference".into(),
        );
    }
    if system.emission.fan_coil_count > 0 && !system.emission.emitter.fan_coil() {
        push(
            "cooling_fan_coil_count_inconsistent",
            "emission.fanCoilCount".into(),
        );
    }
    if let Some(distribution) = &system.distribution {
        if source_missing(&distribution.source_reference) {
            push(
                "source_reference_required",
                "distribution.sourceReference".into(),
            );
        }
        if system.emission.emitter.radiant()
            && distribution.design_temperature != CoolingDesignTemperature::T17To21
        {
            // Table 10.8: floor/wall/ceiling cooling is designed at 17/21.
            push(
                "cooling_design_temperature_inconsistent",
                "distribution.designTemperature".into(),
            );
        }
        if let CoolingPipe::Declared {
            psi_w_per_mk,
            source_reference,
        } = &distribution.pipe
        {
            if !psi_w_per_mk.is_finite() || *psi_w_per_mk <= 0.0 {
                push(
                    "cooling_pipe_psi_invalid",
                    "distribution.pipe.psiWPerMK".into(),
                );
            }
            if source_missing(source_reference) {
                push(
                    "source_reference_required",
                    "distribution.pipe.sourceReference".into(),
                );
            }
        }
        for (field, value) in [
            ("pipeLengthM", distribution.pipe_length_m),
            (
                "unconditionedPipeLengthM",
                distribution.unconditioned_pipe_length_m,
            ),
        ] {
            if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
                push(
                    "cooling_pipe_length_invalid",
                    format!("distribution.{field}"),
                );
            }
        }
        if let (Some(total), Some(part)) = (
            distribution.pipe_length_m,
            distribution.unconditioned_pipe_length_m,
        ) {
            if part > total {
                push(
                    "cooling_pipe_length_invalid",
                    "distribution.unconditionedPipeLengthM".into(),
                );
            }
        }
        if let Some(pump) = &distribution.pump {
            if pump.floor_count == 0 {
                push(
                    "cooling_floor_count_invalid",
                    "distribution.pump.floorCount".into(),
                );
            }
            if pump
                .label_power_kw
                .is_some_and(|value| !value.is_finite() || value <= 0.0)
            {
                push(
                    "cooling_pump_power_invalid",
                    "distribution.pump.labelPowerKw".into(),
                );
            }
            if pump
                .energy_efficiency_index
                .is_some_and(|value| !value.is_finite() || value <= 0.0 || value > 1.0)
            {
                push(
                    "cooling_pump_eei_invalid",
                    "distribution.pump.energyEfficiencyIndex".into(),
                );
            }
            if source_missing(&pump.source_reference) {
                push(
                    "source_reference_required",
                    "distribution.pump.sourceReference".into(),
                );
            }
        }
    }
    if system.generators.is_empty() {
        push("cooling_generator_required", "generators".into());
    }
    let mut priorities: Vec<u8> = system
        .generators
        .iter()
        .map(|item| item.generator.priority())
        .collect();
    priorities.sort_unstable();
    priorities.dedup();
    let mut ids = std::collections::HashSet::new();
    for (index, generator) in system.generators.iter().enumerate() {
        let base = format!("generators[{index}]");
        if generator.id.trim().is_empty() || !ids.insert(generator.id.as_str()) {
            push("id_invalid", format!("{base}.id"));
        }
        if source_missing(&generator.equipment_reference) {
            push(
                "source_reference_required",
                format!("{base}.equipmentReference"),
            );
        }
        match generator.capacity_kw {
            Some(value) if !value.is_finite() || value <= 0.0 => {
                push("cooling_capacity_invalid", format!("{base}.capacityKw"));
            }
            None if priorities.len() > 1 => {
                // 10.49 needs the nominal capacities.
                push("cooling_capacity_required", format!("{base}.capacityKw"));
            }
            _ => {}
        }
        let declared = match &generator.generator {
            CoolingGeneratorKind::Compression { declared, .. }
            | CoolingGeneratorKind::RoomAirConditioner { declared }
            | CoolingGeneratorKind::GasAbsorption { declared, .. } => declared.as_ref(),
            _ => None,
        };
        if let Some(declared) = declared {
            if !declared.value.is_finite() || declared.value <= 0.0 {
                push(
                    "cooling_efficiency_invalid",
                    format!("{base}.generator.declared.value"),
                );
            }
            if source_missing(&declared.source_reference) {
                push(
                    "source_reference_required",
                    format!("{base}.generator.declared.sourceReference"),
                );
            }
        }
        match &generator.generator {
            CoolingGeneratorKind::GasEngineCompression {
                gas_engine: chp, ..
            }
            | CoolingGeneratorKind::AbsorptionChp { chp, .. }
                if chp.factors().is_none() =>
            {
                push("cooling_chp_class_invalid", format!("{base}.generator"));
            }
            _ => {}
        }
    }
    let months = &system.booster_heat_pump_extraction_kwh;
    if !months.is_empty()
        && (months.len() != 12
            || months
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0))
    {
        push(
            "monthly_values_invalid",
            "boosterHeatPumpExtractionKwh".into(),
        );
    }
    if let Some(collective) = &system.collective {
        if !collective.building_usable_floor_area_m2.is_finite()
            || collective.building_usable_floor_area_m2 <= 0.0
        {
            push(
                "cooling_collective_area_invalid",
                "collective.buildingUsableFloorAreaM2".into(),
            );
        }
        if source_missing(&collective.source_reference) {
            push(
                "source_reference_required",
                "collective.sourceReference".into(),
            );
        }
    }
    issues
}

/// Table 10.16: energy fraction for a capacity ratio β (0,1 steps).
pub fn energy_fraction(beta: f64, july_to_september: bool) -> f64 {
    const PEAK: [f64; 10] = [0.34, 0.54, 0.68, 0.77, 0.84, 0.90, 0.93, 0.96, 0.98, 1.00];
    const OTHER: [f64; 10] = [0.49, 0.78, 0.93, 0.98, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    if beta <= 0.0 {
        return 0.0;
    }
    let index = ((beta * 10.0).round() as usize).clamp(1, 10) - 1;
    if july_to_september {
        PEAK[index]
    } else {
        OTHER[index]
    }
}

/// Efficiency and drive of one generator under method 3.
enum Drive {
    Electric(f64),
    Gas(f64),
    DistrictHeat(f64),
    ChpHeat(f64),
    DistrictCold,
    FreeCooling(f64),
}

fn drive(kind: &CoolingGeneratorKind) -> Drive {
    // §10.1: declared values rounded down; f_prpr 0,9 unless table values.
    let declared = |declared: &Option<DeclaredEfficiency>, step: f64, table: f64| {
        declared
            .as_ref()
            .map_or(table, |item| round_down(item.value, step) * 0.9)
    };
    match kind {
        CoolingGeneratorKind::Compression { declared: d, .. }
        | CoolingGeneratorKind::RoomAirConditioner { declared: d } => {
            Drive::Electric(declared(d, 0.05, EER_COMPRESSION_FORFAIT))
        }
        CoolingGeneratorKind::UnknownCollective => Drive::Electric(EER_COMPRESSION_FORFAIT),
        CoolingGeneratorKind::GasEngineCompression { gas_engine, .. } => Drive::Gas(
            EER_COMPRESSION_FORFAIT * gas_engine.factors().map_or(0.0, |factors| factors.1),
        ),
        CoolingGeneratorKind::GasAbsorption { declared: d, .. } => {
            Drive::Gas(declared(d, 0.025, ZETA_GAS_ABSORPTION))
        }
        CoolingGeneratorKind::AbsorptionExternalHeat { .. } => {
            Drive::DistrictHeat(ZETA_EXTERNAL_HEAT_FACTOR)
        }
        CoolingGeneratorKind::AbsorptionChp { chp, .. } => {
            Drive::ChpHeat(chp.factors().map_or(0.0, |factors| factors.0))
        }
        CoolingGeneratorKind::ExternalCold => Drive::DistrictCold,
        CoolingGeneratorKind::FreeCooling { source, .. } => Drive::FreeCooling(source.eer()),
    }
}

fn heat_rejection(kind: &CoolingGeneratorKind) -> Option<HeatRejection> {
    match kind {
        CoolingGeneratorKind::Compression { heat_rejection, .. }
        | CoolingGeneratorKind::GasEngineCompression { heat_rejection, .. }
        | CoolingGeneratorKind::GasAbsorption { heat_rejection, .. }
        | CoolingGeneratorKind::AbsorptionExternalHeat { heat_rejection }
        | CoolingGeneratorKind::AbsorptionChp { heat_rejection, .. } => *heat_rejection,
        _ => None,
    }
}

/// Monthly cooling chain; call only after [`validate_cooling`] returned no
/// issues.
pub fn assess_cooling(system: &CoolingSystem, context: CoolingContext<'_>) -> CoolingAssessment {
    let zone_area: f64 = context
        .zones
        .iter()
        .map(|zone| zone.usable_floor_area_m2)
        .sum();
    let building_area = system
        .collective
        .as_ref()
        .map_or(zone_area, |item| item.building_usable_floor_area_m2);
    let f_building = if system.collective.is_some() && building_area > 0.0 {
        (zone_area / building_area).min(1.0)
    } else {
        1.0
    };
    let mut total_need = [0.0; 12];
    for zone in context.zones {
        for (sum, need) in total_need.iter_mut().zip(zone.need_kwh) {
            *sum += need;
        }
    }
    let limit = cooling_limit_c(&total_need);
    let hours: [f64; 12] = std::array::from_fn(|index| operating_hours(limit, index));

    // Emission (10.10–10.16).
    let delta = system.emission.delta_internal();
    let internal = COOLING_INITIAL_TEMPERATURE_C + delta;
    let solar_correction = if context.residential { 8.0 } else { 12.0 };

    // Distribution (10.21–10.27) and the pump (10.28–10.38).
    let mut distribution_unit = [0.0; 12]; // Q_C;dis;ls for the whole system per month
    let mut pump_energy = [0.0; 12];
    if let Some(distribution) = &system.distribution {
        let (supply, ret) = distribution.design_temperature.supply_return_c();
        let mean = (supply + ret) / 2.0 - delta;
        let psi = distribution.pipe.psi(building_area);
        let length = distribution
            .pipe_length_m
            .unwrap_or(PIPE_LENGTH_PER_M2 * building_area);
        let unconditioned = distribution
            .unconditioned_pipe_length_m
            .unwrap_or(UNCONDITIONED_PIPE_SHARE * length);
        let fittings = if distribution.fittings_insulated {
            0.03
        } else {
            0.15
        };
        let equivalent = unconditioned + fittings / psi * unconditioned;
        let ambient = distribution
            .unconditioned_ambient_c
            .unwrap_or(UNCONDITIONED_AMBIENT_C);
        for index in 0..12 {
            distribution_unit[index] =
                -psi * (mean - ambient) * equivalent * hours[index] / 1000.0 * f_building;
        }
        if let Some(pump) = &distribution.pump {
            let floors = f64::from(pump.floor_count.max(1));
            let max_length = 35.0 + 6.0 * floors + 0.13 * building_area / floors;
            let mut additional = 360.0 / (ret - supply).powi(2);
            if pump.heat_meter {
                additional += 10.0;
            }
            if system.emission.emitter.radiant() {
                additional += 4.5;
            } else if system.emission.emitter.fan_coil() {
                additional += 2.0;
            }
            let pressure = (1.0 + 0.4) * 0.1 * max_length + additional;
            // 10.32 with the month of maximum need (or the nearest with hours).
            let (peak_index, peak_need) =
                total_need
                    .iter()
                    .copied()
                    .enumerate()
                    .fold(
                        (0, 0.0),
                        |best, item| if item.1 > best.1 { item } else { best },
                    );
            let peak_hours = (0..12)
                .map(|offset| hours[(peak_index + offset) % 12])
                .chain((0..12).map(|offset| hours[(peak_index + 12 - offset) % 12]))
                .find(|value| *value > 0.0)
                .unwrap_or(0.0);
            let flow = if peak_hours > 0.0 && peak_need > 0.0 {
                peak_need
                    / (peak_hours
                        * WATER_CP_KJ_PER_KGK
                        * (ret - supply)
                        * WATER_DENSITY_KG_PER_M3
                        * f_building)
                    * 3600.0
            } else {
                0.0
            };
            let hydraulic = (pressure * flow / 3600.0).max(0.01);
            let efficiency_factor = match pump.label_power_kw {
                Some(label) => label / hydraulic,
                None if hydraulic < 2.5 => {
                    let reference =
                        1.7 * hydraulic + 17.0 * (1.0 - (-0.3 * hydraulic * 1000.0).exp()) / 1000.0;
                    reference / hydraulic
                }
                None => (1.25 + (0.2 / hydraulic).powf(0.5)) * 2.0,
            };
            let eei =
                pump.energy_efficiency_index
                    .unwrap_or(if hydraulic < 2.5 { 0.23 } else { 0.25 });
            let epsilon = efficiency_factor * (0.25 + 0.75) * eei / 0.25;
            let balancing = if pump.hydraulically_balanced {
                1.0
            } else {
                1.15
            };
            let operation = if context.residential && pump.individual_dwelling_installation {
                0.10
            } else {
                1.0
            };
            for index in 0..12 {
                pump_energy[index] =
                    hydraulic * hours[index] * balancing * operation * epsilon * f_building;
            }
        }
    }

    // Generator shares (10.49–10.52).
    let mut by_priority: Vec<(u8, f64)> = Vec::new();
    for generator in &system.generators {
        let priority = generator.generator.priority();
        let capacity = generator.capacity_kw.unwrap_or(1.0);
        match by_priority.iter_mut().find(|item| item.0 == priority) {
            Some(item) => item.1 += capacity,
            None => by_priority.push((priority, capacity)),
        }
    }
    by_priority.sort_by_key(|item| item.0);
    let total_capacity: f64 = by_priority.iter().map(|item| item.1).sum();
    let mut betas = Vec::with_capacity(by_priority.len());
    let mut cumulative = 0.0;
    for (priority, capacity) in &by_priority {
        cumulative += capacity;
        // 10.49: rounded up to one decimal.
        let beta = ((cumulative / total_capacity) * 10.0 - 1e-9).ceil() / 10.0;
        betas.push((*priority, beta.min(1.0)));
    }
    let priority_share = |priority: u8, peak: bool| -> f64 {
        if betas.len() == 1 {
            return 1.0;
        }
        let position = betas
            .iter()
            .position(|item| item.0 == priority)
            .expect("known priority");
        let previous = if position == 0 {
            0.0
        } else {
            energy_fraction(betas[position - 1].1, peak)
        };
        if position + 1 == betas.len() {
            1.0 - previous
        } else {
            energy_fraction(betas[position].1, peak) - previous
        }
    };
    let generator_share = |generator: &CoolingGenerator, peak: bool| -> f64 {
        let priority = generator.generator.priority();
        let group = by_priority
            .iter()
            .find(|item| item.0 == priority)
            .expect("known priority")
            .1;
        priority_share(priority, peak) * generator.capacity_kw.unwrap_or(1.0) / group
    };
    let shares: Vec<GeneratorShare> = system
        .generators
        .iter()
        .map(|generator| GeneratorShare {
            id: generator.id.clone(),
            priority: generator.generator.priority(),
            beta: betas
                .iter()
                .find(|item| item.0 == generator.generator.priority())
                .map_or(1.0, |item| item.1),
            share_july_to_september: generator_share(generator, true),
            share_other_months: generator_share(generator, false),
        })
        .collect();

    // Month loop.
    let booster: [f64; 12] = std::array::from_fn(|index| {
        system
            .booster_heat_pump_extraction_kwh
            .get(index)
            .copied()
            .unwrap_or(0.0)
    });
    let mut months = Vec::with_capacity(12);
    let mut free_cold_by_generator = vec![[0.0; 12]; system.generators.len()];
    for index in 0..12 {
        let outdoor = OUTDOOR_TEMPERATURE_C[index];
        let combined = outdoor + solar_correction;
        let mut need = 0.0;
        let mut emission = 0.0;
        let mut distribution = 0.0;
        for zone in context.zones {
            let zone_need = zone.need_kwh[index];
            need += zone_need;
            // 10.15/10.16
            if internal - combined < 0.0 && zone_need > 0.0 {
                emission += zone_need * (delta / (internal - combined)).max(0.15);
            }
            // 10.21 only for zones with a cooling need.
            if zone_need > 0.0 && zone_area > 0.0 {
                distribution += distribution_unit[index] * zone.usable_floor_area_m2 / zone_area;
            }
        }
        let has_load = need + emission + distribution > 0.0;
        let pump = if has_load { pump_energy[index] } else { 0.0 };
        // 10.45
        let recovered = (1.0 - PUMP_RECOVERABLE_FACTOR) * pump;
        let load = need + emission + distribution + recovered;
        // 10.7–10.9: the booster heat pump is limited to the load.
        let extraction = booster[index].min(load);
        let generator_cold = load - extraction;
        let peak = (6..=8).contains(&index);
        let mut row = CoolingMonth {
            month: index as u8 + 1,
            operating_hours: hours[index],
            need_kwh: need,
            emission_loss_kwh: emission,
            distribution_loss_kwh: distribution,
            pump_recovered_kwh: recovered,
            booster_extraction_kwh: extraction,
            generator_cold_kwh: generator_cold,
            ..CoolingMonth::default()
        };
        // 10.17/10.18
        let mut auxiliary =
            FAN_COIL_POWER_W * f64::from(system.emission.fan_coil_count) * hours[index] / 1000.0
                + pump;
        let mut machine_present = false;
        for (generator_index, generator) in system.generators.iter().enumerate() {
            let cold = generator_share(generator, peak) * generator_cold;
            let rejection =
                heat_rejection(&generator.generator).unwrap_or(HeatRejection::AirCooled);
            if !matches!(
                generator.generator,
                CoolingGeneratorKind::FreeCooling { .. }
                    | CoolingGeneratorKind::UnknownCollective
                    | CoolingGeneratorKind::ExternalCold
            ) {
                machine_present = true;
            }
            match drive(&generator.generator) {
                Drive::Electric(eer) => {
                    row.electricity_kwh += cold / eer;
                    if !matches!(generator.generator, CoolingGeneratorKind::UnknownCollective) {
                        // 10.80 with the table EER; 10.83.
                        auxiliary += cold * (1.0 + 1.0 / eer) * rejection.distribution_power();
                    }
                }
                Drive::Gas(efficiency) => {
                    row.natural_gas_kwh += cold / efficiency;
                    let ratio = if matches!(
                        generator.generator,
                        CoolingGeneratorKind::GasEngineCompression { .. }
                    ) {
                        EER_COMPRESSION_FORFAIT
                    } else {
                        efficiency
                    };
                    // 10.80b/10.81 and 10.83.
                    auxiliary += cold * (1.0 + 1.0 / ratio) * rejection.distribution_power();
                }
                Drive::DistrictHeat(zeta) => {
                    row.district_heat_kwh += cold / zeta;
                    auxiliary += cold * (1.0 + 1.0 / zeta) * rejection.distribution_power();
                }
                Drive::ChpHeat(zeta) => {
                    row.chp_heat_kwh += cold / zeta;
                    auxiliary += cold * (1.0 + 1.0 / zeta) * rejection.distribution_power();
                }
                Drive::DistrictCold => {
                    // 10.78 with η = 1 and f_prpr = 1 (forfait f_P;del;dc).
                    row.district_cold_kwh += cold;
                }
                Drive::FreeCooling(eer) => {
                    // 10.86: pump energy only.
                    row.electricity_kwh += cold / eer;
                    free_cold_by_generator[generator_index][index] = cold;
                    if eer >= RENCOLD_MIN_EER {
                        row.ambient_cold_kwh += cold;
                    }
                }
            }
        }
        if machine_present {
            // 10.87: control is always on.
            auxiliary += CONTROL_POWER_KW * MONTH_HOURS[index];
        }
        row.auxiliary_electricity_kwh = auxiliary;
        months.push(row);
    }
    // 10.84/10.85: regeneration surcharge for ground storage that feeds a
    // heat pump and receives less than 70 % of the extracted heat back.
    let extracted: f64 = context.heat_pump_source_extraction_kwh.iter().sum();
    let regeneration_months = 6.0;
    for (generator_index, generator) in system.generators.iter().enumerate() {
        if let CoolingGeneratorKind::FreeCooling {
            source,
            heat_pump_source: true,
            ground_above_zero_demonstrated: false,
        } = generator.generator
        {
            if !source.ground_storage() {
                continue;
            }
            let returned: f64 = free_cold_by_generator[generator_index].iter().sum();
            if extracted > 0.0 && returned < REGENERATION_SHARE * extracted {
                let surcharge = (REGENERATION_SHARE * extracted - returned)
                    / (REGENERATION_EFFICIENCY * regeneration_months);
                for row in months.iter_mut().skip(3).take(6) {
                    row.electricity_kwh += surcharge;
                }
            }
        }
    }
    let zone_booster = context
        .zones
        .iter()
        .map(|zone| {
            std::array::from_fn(|index| {
                if zone_area > 0.0 {
                    months[index].booster_extraction_kwh * zone.usable_floor_area_m2 / zone_area
                } else {
                    0.0
                }
            })
        })
        .collect();
    CoolingAssessment {
        cooling_limit_c: limit,
        internal_temperature_shift_k: delta,
        generator_shares: shares,
        months,
        zone_booster_extraction_kwh: zone_booster,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system(generators: Vec<CoolingGenerator>) -> CoolingSystem {
        CoolingSystem {
            emission: CoolingEmission {
                emitter: CoolingEmitter::OtherOrUnknown,
                balancing: CoolingBalancing::NotApplicable,
                control: CoolingControl::StandalonePerRoom,
                fan_coil_count: 0,
                source_reference: "design".into(),
            },
            distribution: None,
            generators,
            booster_heat_pump_extraction_kwh: Vec::new(),
            collective: None,
        }
    }

    fn generator(kind: CoolingGeneratorKind, capacity: Option<f64>) -> CoolingGenerator {
        CoolingGenerator {
            id: format!("g{}", kind_name(&kind)),
            generator: kind,
            capacity_kw: capacity,
            equipment_reference: "plate".into(),
        }
    }

    fn kind_name(kind: &CoolingGeneratorKind) -> &'static str {
        match kind {
            CoolingGeneratorKind::FreeCooling { .. } => "free",
            CoolingGeneratorKind::Compression { .. } => "comp",
            _ => "other",
        }
    }

    fn compression() -> CoolingGeneratorKind {
        CoolingGeneratorKind::Compression {
            heat_rejection: None,
            declared: None,
        }
    }

    fn summer_need() -> [f64; 12] {
        [
            0.0, 0.0, 0.0, 10.0, 60.0, 120.0, 180.0, 170.0, 80.0, 10.0, 0.0, 0.0,
        ]
    }

    fn context(zones: &[CoolingZoneNeed]) -> CoolingContext<'_> {
        CoolingContext {
            zones,
            residential: true,
            heat_pump_source_extraction_kwh: [0.0; 12],
        }
    }

    #[test]
    fn cooling_limit_follows_least_squares_steps() {
        // Need linear in θe: 20·(θe − 15) above 15 °C; May (14,73) is below
        // 10 % and dropped, the line crosses at 15 °C.
        let need: [f64; 12] =
            std::array::from_fn(|index| (20.0 * (OUTDOOR_TEMPERATURE_C[index] - 15.0)).max(0.0));
        assert_eq!(cooling_limit_c(&need), 15.0);
        assert_eq!(cooling_limit_c(&[0.0; 12]), 25.0);
        assert_eq!(operating_hours(15.0, 6), 603.0);
        assert_eq!(operating_hours(10.0, 4), 367.0);
        assert_eq!(operating_hours(30.0, 7), 74.0);
    }

    #[test]
    fn emission_loss_follows_10_15_with_floor_of_0_15() {
        let input = system(vec![generator(compression(), None)]);
        let zones = [CoolingZoneNeed {
            usable_floor_area_m2: 100.0,
            need_kwh: summer_need(),
        }];
        let result = assess_cooling(&input, context(&zones));
        // Δϑ = −0,5 + 0 − 1,25 = −1,75; ϑ_int,inc = 22,25.
        let july = &result.months[6];
        let combined = 18.05 + 8.0;
        let ratio: f64 = (-1.75_f64 / (22.25 - combined)).max(0.15);
        assert!((july.emission_loss_kwh - 180.0 * ratio).abs() < 1e-9);
        // April: 22,25 − 17,32 > 0, no loss.
        assert_eq!(result.months[3].emission_loss_kwh, 0.0);
        let cold = july.need_kwh + july.emission_loss_kwh;
        assert!((july.electricity_kwh - cold / 3.0).abs() < 1e-9);
        // Control 0,010 kW for 744 h.
        assert!((july.auxiliary_electricity_kwh - 7.44).abs() < 1e-9);
    }

    #[test]
    fn distribution_and_pump_follow_10_21_and_10_28_to_10_38() {
        let mut input = system(vec![generator(compression(), None)]);
        input.emission.emitter = CoolingEmitter::FanCoilOrRacOnCeiling;
        input.emission.balancing = CoolingBalancing::Dynamic;
        input.emission.fan_coil_count = 4;
        input.distribution = Some(CoolingDistribution {
            design_temperature: CoolingDesignTemperature::T6To12OrUnknown,
            pipe: CoolingPipe::InsulatedFrom1995,
            fittings_insulated: true,
            pipe_length_m: None,
            unconditioned_pipe_length_m: None,
            unconditioned_ambient_c: None,
            pump: Some(CoolingPump {
                hydraulically_balanced: false,
                floor_count: 2,
                heat_meter: false,
                individual_dwelling_installation: false,
                label_power_kw: None,
                energy_efficiency_index: None,
                source_reference: "design".into(),
            }),
            source_reference: "design".into(),
        });
        let zones = [CoolingZoneNeed {
            usable_floor_area_m2: 100.0,
            need_kwh: summer_need(),
        }];
        assert!(validate_cooling(&input, "cooling").is_empty());
        let result = assess_cooling(&input, context(&zones));
        let limit = result.cooling_limit_c;
        let hours = operating_hours(limit, 6);
        // Δϑ = −0,5 + 0 − 1,25 = −1,75: mean (6+12)/2 + 1,75 = 10,75 °C.
        let length = 0.15 * 64.0;
        let equivalent = length + 0.03 / 0.3 * length;
        let expected = -0.3 * (10.75 - 19.0) * equivalent * hours / 1000.0;
        let july = &result.months[6];
        assert!((july.distribution_loss_kwh - expected).abs() < 1e-9);
        // Pump: L_max = 35 + 12 + 0,13·100/2; Δp = 1,4·0,1·L_max + 10 + 2.
        let max_length = 35.0 + 12.0 + 6.5;
        let pressure = 1.4 * 0.1 * max_length + 10.0 + 2.0;
        let peak_hours = operating_hours(limit, 6);
        let flow = 180.0 / (peak_hours * 4.2 * 6.0 * 1000.0) * 3600.0;
        let hydraulic = (pressure * flow / 3600.0).max(0.01);
        let reference =
            1.7 * hydraulic + 17.0 * (1.0 - (-0.3 * hydraulic * 1000.0_f64).exp()) / 1000.0;
        let epsilon = reference / hydraulic * 0.23 / 0.25;
        let pump = hydraulic * hours * 1.15 * epsilon;
        let fans = 10.0 * 4.0 * hours / 1000.0;
        assert!((july.auxiliary_electricity_kwh - (pump + fans + 7.44)).abs() < 1e-9);
        assert!((july.pump_recovered_kwh - 0.9 * pump).abs() < 1e-12);
        // Months without load have no pump energy.
        assert_eq!(result.months[0].pump_recovered_kwh, 0.0);
    }

    #[test]
    fn priorities_split_cold_by_table_10_16() {
        let free = CoolingGeneratorKind::FreeCooling {
            source: FreeCoolingSource::ClosedGroundLoop,
            heat_pump_source: false,
            ground_above_zero_demonstrated: false,
        };
        let input = system(vec![
            generator(free, Some(3.0)),
            generator(compression(), Some(7.0)),
        ]);
        assert!(validate_cooling(&input, "cooling").is_empty());
        let zones = [CoolingZoneNeed {
            usable_floor_area_m2: 100.0,
            need_kwh: summer_need(),
        }];
        let result = assess_cooling(&input, context(&zones));
        let shares = &result.generator_shares;
        assert!((shares[0].beta - 0.3).abs() < 1e-12);
        assert!((shares[0].share_july_to_september - 0.68).abs() < 1e-12);
        assert!((shares[0].share_other_months - 0.93).abs() < 1e-12);
        assert!((shares[1].share_july_to_september - 0.32).abs() < 1e-12);
        let july = &result.months[6];
        let cold = july.generator_cold_kwh;
        let expected = 0.68 * cold / 10.0 + 0.32 * cold / 3.0;
        assert!((july.electricity_kwh - expected).abs() < 1e-9);
        assert!((july.ambient_cold_kwh - 0.68 * cold).abs() < 1e-9);
        // Missing capacities with two priorities are rejected.
        let missing = system(vec![
            generator(
                CoolingGeneratorKind::FreeCooling {
                    source: FreeCoolingSource::SurfaceWater,
                    heat_pump_source: false,
                    ground_above_zero_demonstrated: false,
                },
                None,
            ),
            generator(compression(), Some(7.0)),
        ]);
        assert!(validate_cooling(&missing, "c")
            .iter()
            .any(|item| item.code == "cooling_capacity_required"));
    }

    #[test]
    fn regeneration_surcharge_follows_10_85() {
        let free = CoolingGeneratorKind::FreeCooling {
            source: FreeCoolingSource::AquiferFrom2013,
            heat_pump_source: true,
            ground_above_zero_demonstrated: false,
        };
        let input = system(vec![generator(free, None)]);
        let zones = [CoolingZoneNeed {
            usable_floor_area_m2: 100.0,
            need_kwh: summer_need(),
        }];
        let mut ctx = context(&zones);
        ctx.heat_pump_source_extraction_kwh = [500.0; 12];
        let result = assess_cooling(&input, ctx);
        let returned: f64 = result.months.iter().map(|row| row.generator_cold_kwh).sum();
        let surcharge = (0.7 * 6000.0 - returned) / 60.0;
        let july = &result.months[6];
        assert!((july.electricity_kwh - (july.generator_cold_kwh / 23.0 + surcharge)).abs() < 1e-9);
        let january = &result.months[0];
        assert_eq!(january.electricity_kwh, 0.0);
        // No control energy for pure free cooling.
        assert_eq!(july.auxiliary_electricity_kwh, 0.0);
    }

    #[test]
    fn generation_tables_and_declared_rounding() {
        let declared = CoolingGeneratorKind::Compression {
            heat_rejection: Some(HeatRejection::ClosedCoolingTower),
            declared: Some(DeclaredEfficiency {
                value: 4.37,
                source_reference: "quality statement".into(),
            }),
        };
        let Drive::Electric(eer) = drive(&declared) else {
            panic!("electric expected");
        };
        assert!((eer - 4.35 * 0.9).abs() < 1e-12);
        let engine = ChpClass {
            power_kw: 50.0,
            built_after_2006: true,
            hre_declared: false,
            low_temperature: false,
        };
        assert_eq!(engine.factors(), Some((0.49, 0.30)));
        assert_eq!(
            ChpClass {
                power_kw: 1.5,
                built_after_2006: false,
                hre_declared: false,
                low_temperature: false,
            }
            .factors(),
            None
        );
        let Drive::Gas(efficiency) = drive(&CoolingGeneratorKind::GasEngineCompression {
            gas_engine: engine,
            heat_rejection: None,
        }) else {
            panic!("gas expected");
        };
        assert!((efficiency - 0.9).abs() < 1e-12);
        assert_eq!(FreeCoolingSource::AquiferFrom2013.eer(), 23.0);
        assert_eq!(FreeCoolingSource::DewPointCooling.eer(), 8.0);
        assert!((energy_fraction(0.1, true) - 0.34).abs() < 1e-12);
        assert!((energy_fraction(0.5, false) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn booster_heat_pump_reduces_generator_cold_and_is_split_by_area() {
        let mut input = system(vec![generator(compression(), None)]);
        input.booster_heat_pump_extraction_kwh = vec![30.0; 12];
        let zones = [
            CoolingZoneNeed {
                usable_floor_area_m2: 60.0,
                need_kwh: summer_need(),
            },
            CoolingZoneNeed {
                usable_floor_area_m2: 40.0,
                need_kwh: [0.0; 12],
            },
        ];
        let result = assess_cooling(&input, context(&zones));
        let july = &result.months[6];
        assert!((july.booster_extraction_kwh - 30.0).abs() < 1e-12);
        let load = july.need_kwh + july.emission_loss_kwh;
        assert!((july.generator_cold_kwh - (load - 30.0)).abs() < 1e-9);
        assert!((result.zone_booster_extraction_kwh[1][6] - 12.0).abs() < 1e-12);
        // January has no load: extraction limited to 0.
        assert_eq!(result.months[0].booster_extraction_kwh, 0.0);
    }

    #[test]
    fn validation_rejects_inconsistent_input() {
        let mut bad = system(Vec::new());
        bad.emission.emitter = CoolingEmitter::FloorCooling;
        bad.emission.fan_coil_count = 2;
        bad.distribution = Some(CoolingDistribution {
            design_temperature: CoolingDesignTemperature::T6To12OrUnknown,
            pipe: CoolingPipe::Uninsulated,
            fittings_insulated: false,
            pipe_length_m: Some(10.0),
            unconditioned_pipe_length_m: Some(20.0),
            unconditioned_ambient_c: None,
            pump: None,
            source_reference: String::new(),
        });
        let codes: Vec<_> = validate_cooling(&bad, "cooling")
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "cooling_generator_required",
            "cooling_fan_coil_count_inconsistent",
            "cooling_design_temperature_inconsistent",
            "cooling_pipe_length_invalid",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }
}
