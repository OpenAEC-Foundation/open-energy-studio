//! Space cooling, NTA 8800:2025+C1:2026 chapter 10 (pages 366–426), from
//! the chapter 7 cooling need of the zones served by one cooling system.
//!
//! The chain is additive (10.5/10.7–10.9): `Q_C;gen;in = Σ_zi (Q_C;nd +
//! Q_C;em;ls + Q_C;dis;ls + Q_C;dis;rvd) − Q_C;HP`, where
//! - the emission loss follows 10.10/10.11, 10.15/10.16 with tables 10.35,
//!   10.4, 10.5 and 10.5a (`MAX(…; 0,15)` as printed on p. 376);
//! - the operating hours `t_C;mi` follow the cooling limit of 10.19 steps 1–6
//!   and table 10.6; step 1 uses the zone need without recoverable losses
//!   and with the supply-air term of 10.20 (`limit_need_kwh`);
//! - a water-based distribution loses 10.21/10.22 in unconditioned spaces
//!   (cooled spaces have `L = 0`), with `L_si = 0,64·A_g` (10.27), table 10.9
//!   and the mean water temperature of table 10.8; pump energy follows
//!   10.28–10.38 with tables 10.10–10.13 and recovered pump heat 10.45;
//!   direct expansion has no distribution loss;
//! - fan-coil fans use 10.17/10.18 with table 10.7;
//! - the cold is split over generators by the priority of table 10.15 and
//!   the β/f(β) rule of 10.49–10.52 with table 10.16;
//! - electric compression generators with NEN-EN 14825 part-load results
//!   use method 1 (§10.5.4: 10.53–10.64 with tables 10.17/10.18), those with
//!   a NEN-EN 14511 rating method 2 (§10.5.5: 10.65–10.75 with tables
//!   10.19–10.27 and f_prpr 0,60/0,9);
//! - other generation uses method 3 (§10.5.6): table 10.29 compression (EER 3,00,
//!   gas engine 3,00·η_ge with ε_chp;el of table 9.31), table 10.30
//!   absorption (gas 0,80; external heat 0,70·η_dh; CHP 1,00·ε_chp;th),
//!   external cold (10.78), free cooling with table 10.34 (10.86) and the
//!   aquifer/ground regeneration surcharge 10.84/10.85; declared values are
//!   rounded down to 0,05 (electric) or 0,025 (gas-fired), §10.1;
//! - generator auxiliaries follow 10.79: no condenser fans in method 3
//!   (10.5.7.1) and table 10.31 fans (10.82) for water-cooled method 2
//!   machines, condenser-water distribution 10.83 with table 10.33 for
//!   water-cooled machines (10.80 with the rated EER for methods 1/2),
//!   control 0,010 kW for every month (10.87); free cooling counts pump
//!   energy only (§10.5.7.2.1);
//! - ambient cold (5.34) is the cold of free cooling with `EER ≥ 8`.
//!
//! The dehumidification need `Q_C;dhum` (12.5, table 12.2) is added to the
//! generator load: by the design temperature of the distribution (6/12 °C
//! for direct expansion or an unknown design), zero for radiant emitters.
//!
//! AHU cooling coils `Q_C;ahu;in;req` (11.116) are added to the generator
//! load without emission or distribution loss (`CoolingZoneNeed::ahu_load_kwh`).
//!
//! Absorption chillers with a NEN-EN 14511 rating use method 2 (10.66:
//! PLV 0,95 · ζ_n · f_prpr) and 10.81 with ζ_n·0,95 for the rejected heat.
//! Absorption on building CHP books the CHP fuel (table 10.30 with 9.65,
//! ε_chp;th) and its electricity (ε_chp;el, §9.6.6.1) for chapter 16.
//! Table 10.32 has only the "not controlled" row (1), so f_hr;PL;el = 1 is
//! the complete table.
//!
//! Not modelled: recoverable distribution losses (zero because `L_C;zi = 0`
//! in cooled zones). Interpretations are listed in
//! [`COOLING_INTERPRETATIONS`].

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

/// Method 2 (§10.5.5, 10.66) of an absorption chiller: the nominal heat
/// ratio `ζ_n` at the NEN-EN 14511 standard rating conditions; PLV = 0,95.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AbsorptionRating {
    pub nominal_heat_ratio: f64,
    pub source_reference: String,
}

/// 10.67: part-load value of absorption chillers in method 2.
const ABSORPTION_PLV: f64 = 0.95;

/// A quality-statement value replacing a table value (§10.1).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredEfficiency {
    pub value: f64,
    pub source_reference: String,
}

/// Measured or rated performance of an electric compression generator:
/// method 1 (§10.5.4, NEN-EN 14825 part-load results, direct condensation
/// against outdoor air only) or method 2 (§10.5.5, NEN-EN 14511 rating).
/// Without it method 3 (table 10.29) applies.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum CompressionPerformance {
    En14825(En14825Performance),
    En14511(En14511Performance),
}

/// One NEN-EN 14825 part-load test point (conditions A–D, or the fifth
/// point of 10.63).
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct En14825Point {
    /// `f_C;PL` in %.
    pub part_load_percent: f64,
    pub eer: f64,
    /// `ϑ_C;evap;out` (10.63): the temperature leaving the evaporator, °C.
    /// For air-to-air units this is the leaving (supply) air temperature of
    /// the indoor unit, not the indoor air temperature of the test
    /// condition; it must lie below `ϑ_C;cond;in` (10.56/10.64 divide by the
    /// difference).
    pub evaporator_outlet_c: f64,
    /// `ϑ_C;cond;in` (10.63): the temperature entering the condenser, for
    /// air-cooled units the outdoor air temperature of the test condition,
    /// °C.
    pub condenser_inlet_c: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct En14825Performance {
    /// `EER_n;gi` at the standard rating conditions.
    pub nominal_eer: f64,
    /// `Φ_C;gen;gi;n`, kW.
    pub nominal_capacity_kw: f64,
    /// `Φ_C;gen;gi;min` in continuous operation, kW.
    pub minimum_capacity_kw: f64,
    /// Conditions A, B, C and D.
    pub test_points: Vec<En14825Point>,
    /// Fifth point (C part load at the A condenser temperature); omitted
    /// means 10.64 with `Δϑ_corr = 0`.
    #[serde(default)]
    pub fifth_point: Option<En14825Point>,
    /// `ϑ_cond;in;lim` of 10.62, °C; omitted means the bin temperature.
    #[serde(default)]
    pub condenser_inlet_limit_c: Option<f64>,
    /// `ϑ_C;gen;req;out` (10.61), °C; default: the distribution supply
    /// temperature of table 10.8, or 24 °C for evaporation in the room.
    #[serde(default)]
    pub required_outlet_c: Option<f64>,
    pub source_reference: String,
}

/// Table 10.19 room air-conditioning system codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomUnitType {
    /// AIR_CLG_RAC_A: split system.
    Split,
    /// AIR_CLG_RAC_B: multi-split with cylinder switching.
    MultiSplitStaged,
    /// AIR_CLG_RAC_C: split system with frequency control.
    SplitInverter,
    /// AIR_CLG_RAC_D: multi-split with frequency control (VRF).
    MultiSplitInverter,
}

impl RoomUnitType {
    /// Table 10.19 row.
    fn part_load_factors(self) -> [f64; 10] {
        match self {
            Self::Split => [1.34, 1.34, 1.34, 1.34, 1.27, 1.23, 1.16, 1.09, 1.02, 0.95],
            Self::MultiSplitStaged => [0.68, 0.73, 0.77, 0.80, 0.86, 0.93, 0.95, 0.97, 0.94, 0.90],
            Self::SplitInverter => [1.52, 1.54, 1.57, 1.69, 1.45, 1.31, 1.21, 1.09, 1.03, 0.95],
            Self::MultiSplitInverter => {
                [0.77, 1.18, 1.42, 1.55, 1.54, 1.46, 1.35, 1.19, 1.06, 0.92]
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct En14511Performance {
    /// `EER_n;gi` at the standard rating conditions.
    pub nominal_eer: f64,
    /// `Φ_C;gen;gi;n`, kW.
    pub nominal_capacity_kw: f64,
    /// `ϑ_C;gen;req;out;gi;n`: evaporator outlet at the rating, °C.
    pub nominal_evaporator_outlet_c: f64,
    /// `ϑ_C;gen;hr;req;in;gi;n`: condenser inlet at the rating, °C.
    pub nominal_condenser_inlet_c: f64,
    /// Table 10.19 code; required for a room air conditioner.
    #[serde(default)]
    pub room_unit_type: Option<RoomUnitType>,
    /// AIR_CLG_HEAT_REJ = INTERNAL (table 10.22, Δϑ_cond 20 K): an
    /// air-cooled chiller rejecting heat to exhaust air.
    #[serde(default)]
    pub heat_rejection_to_exhaust_air: bool,
    /// Table 10.31: axial fans without silencer; default with silencer or
    /// unknown.
    #[serde(default)]
    pub axial_fans_without_silencer: bool,
    /// `ϑ_C;gen;req;out;si;mi` of 10.73, °C; default: the distribution supply
    /// temperature of table 10.8, or 24 °C for evaporation in the room.
    #[serde(default)]
    pub required_outlet_c: Option<f64>,
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
        /// Method 1 or 2 (§10.5.4/10.5.5) instead of table 10.29.
        #[serde(default)]
        performance: Option<CompressionPerformance>,
    },
    /// Table 10.29 room air conditioner (local, lowest priority).
    RoomAirConditioner {
        #[serde(default)]
        declared: Option<DeclaredEfficiency>,
        /// Method 1 or 2 (§10.5.4/10.5.5) instead of table 10.29.
        #[serde(default)]
        performance: Option<CompressionPerformance>,
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
        /// Method 2 (10.66) instead of table 10.30.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rating: Option<AbsorptionRating>,
    },
    /// Table 10.30 absorption on forfait external heat (η_dh = 1).
    AbsorptionExternalHeat {
        #[serde(default, rename = "heatRejection")]
        heat_rejection: Option<HeatRejection>,
        /// Method 2 (10.66) instead of table 10.30.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rating: Option<AbsorptionRating>,
    },
    /// Table 10.30 absorption on building CHP heat, ζ = 1,00·ε_chp;th: the
    /// CHP fuel follows 9.65 (ε_chp;th of table 9.31) and its electricity
    /// (ε_chp;el) is credited in chapter 16 (§9.6.6.1).
    AbsorptionChp {
        chp: ChpClass,
        #[serde(default, rename = "heatRejection")]
        heat_rejection: Option<HeatRejection>,
        /// Method 2 (10.66) instead of table 10.30.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rating: Option<AbsorptionRating>,
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
    /// 11.116 Q_C;ahu;in;req of air handling unit cooling coils, kWh,
    /// supplied by the cooling generator without emission or distribution.
    pub ahu_load_kwh: [f64; 12],
    /// 10.19: Q_C;nd;ϑkoelgrens without recoverable losses and with the
    /// supply-air term of 10.20, kWh; `None` uses `need_kwh`.
    pub limit_need_kwh: Option<[f64; 12]>,
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
    /// `Q_C;dhum` (12.5), kWh.
    pub dehumidification_kwh: f64,
    /// 11.116 load of air handling unit cooling coils, kWh.
    pub ahu_cooling_kwh: f64,
    /// `Q_C;gen;in`.
    pub generator_cold_kwh: f64,
    /// Drive energy of compression chillers and free-cooling pumps.
    pub electricity_kwh: f64,
    pub natural_gas_kwh: f64,
    /// Heat from external delivery for absorption chillers.
    pub district_heat_kwh: f64,
    /// `Q_C;gen;out` of absorption chillers driven by external heat, for
    /// the renewable share (5.39g), kWh.
    pub district_heat_cold_kwh: f64,
    /// Gas input of the building CHP driving absorption chillers (table
    /// 10.30 with 9.65), kWh.
    pub chp_heat_kwh: f64,
    /// Electricity of that CHP (ε_chp;el · fuel, §9.6.6.1), credited in
    /// chapter 16, kWh.
    pub chp_electricity_kwh: f64,
    /// External cold delivery (carrier dc).
    pub district_cold_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    pub ambient_cold_kwh: f64,
    /// 10.72 `f_C;PL;cvd` of method 2 generators.
    pub part_load_coverage: Option<f64>,
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
    /// Generation method 1, 2 or 3 (§10.5.4–10.5.6).
    pub method: u8,
    /// Methods 1/2: EER_si;mi of 10.54 or PLV·EER_n·f_EER;corr·f_prpr of
    /// 10.65 per month.
    pub monthly_eer: Vec<f64>,
    /// Method 1: `[C1, C2, C3, C4, Δϑ_corr]` of 10.63.
    pub en14825_coefficients: Option<Vec<f64>>,
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
    pub interpretations: Vec<&'static str>,
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
            | CoolingGeneratorKind::RoomAirConditioner { declared, .. }
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
        let absorption_rating = match &generator.generator {
            CoolingGeneratorKind::GasAbsorption { rating, .. }
            | CoolingGeneratorKind::AbsorptionExternalHeat { rating, .. }
            | CoolingGeneratorKind::AbsorptionChp { rating, .. } => rating.as_ref(),
            _ => None,
        };
        if let Some(rating) = absorption_rating {
            let field = format!("{base}.generator.rating");
            if declared.is_some() {
                push("cooling_declared_and_performance", field.clone());
            }
            if !rating.nominal_heat_ratio.is_finite() || rating.nominal_heat_ratio <= 0.0 {
                push("cooling_performance_invalid", field.clone());
            }
            if source_missing(&rating.source_reference) {
                push(
                    "source_reference_required",
                    format!("{field}.sourceReference"),
                );
            }
        }
        if let Some(rated) = rated(&generator.generator) {
            let field = format!("{base}.generator.performance");
            if declared.is_some() {
                push("cooling_declared_and_performance", field.clone());
            }
            let positive = |value: f64| value.is_finite() && value > 0.0;
            let finite = |value: Option<f64>| value.map_or(true, f64::is_finite);
            match rated.performance {
                CompressionPerformance::En14825(performance) => {
                    if !positive(performance.nominal_eer)
                        || !positive(performance.nominal_capacity_kw)
                        || !positive(performance.minimum_capacity_kw)
                        || performance.minimum_capacity_kw > performance.nominal_capacity_kw
                        || !finite(performance.condenser_inlet_limit_c)
                        || !finite(performance.required_outlet_c)
                    {
                        push("cooling_performance_invalid", field.clone());
                    }
                    if !rated.room_unit && rated.rejection != HeatRejection::AirCooled {
                        // §10.5.4: direct condensation against outdoor air.
                        push("cooling_en14825_direct_condensation_only", field.clone());
                    }
                    if performance.test_points.len() != 4 {
                        push(
                            "cooling_en14825_points_required",
                            format!("{field}.testPoints"),
                        );
                    } else if performance
                        .test_points
                        .iter()
                        .chain(&performance.fifth_point)
                        .any(|point| {
                            !positive(point.part_load_percent)
                                || !positive(point.eer)
                                || !point.evaporator_outlet_c.is_finite()
                                || !point.condenser_inlet_c.is_finite()
                                || point.condenser_inlet_c <= point.evaporator_outlet_c
                        })
                    {
                        push(
                            "cooling_en14825_point_invalid",
                            format!("{field}.testPoints"),
                        );
                    } else if positive(performance.nominal_eer)
                        && en14825_coefficients(performance).is_none()
                    {
                        push(
                            "cooling_en14825_points_singular",
                            format!("{field}.testPoints"),
                        );
                    }
                    if source_missing(&performance.source_reference) {
                        push(
                            "source_reference_required",
                            format!("{field}.sourceReference"),
                        );
                    }
                }
                CompressionPerformance::En14511(performance) => {
                    let lift = performance.nominal_condenser_inlet_c
                        - performance.nominal_evaporator_outlet_c;
                    if !positive(performance.nominal_eer)
                        || !positive(performance.nominal_capacity_kw)
                        || !performance.nominal_evaporator_outlet_c.is_finite()
                        || !performance.nominal_condenser_inlet_c.is_finite()
                        || lift + 10.0 <= 0.0
                        || !finite(performance.required_outlet_c)
                    {
                        push("cooling_performance_invalid", field.clone());
                    }
                    match (rated.room_unit, performance.room_unit_type) {
                        (true, None) => push(
                            "cooling_room_unit_type_required",
                            format!("{field}.roomUnitType"),
                        ),
                        (false, Some(_)) => push(
                            "cooling_room_unit_type_inconsistent",
                            format!("{field}.roomUnitType"),
                        ),
                        _ => {}
                    }
                    if performance.heat_rejection_to_exhaust_air
                        && (rated.room_unit || rated.rejection != HeatRejection::AirCooled)
                    {
                        push(
                            "cooling_heat_rejection_inconsistent",
                            format!("{field}.heatRejectionToExhaustAir"),
                        );
                    }
                    if source_missing(&performance.source_reference) {
                        push(
                            "source_reference_required",
                            format!("{field}.sourceReference"),
                        );
                    }
                }
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
    /// Fuel ratio ζ = heat ratio · ε_chp;th and ε_chp;el.
    ChpHeat {
        zeta: f64,
        electric: f64,
    },
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
        | CoolingGeneratorKind::RoomAirConditioner { declared: d, .. } => {
            Drive::Electric(declared(d, 0.05, EER_COMPRESSION_FORFAIT))
        }
        CoolingGeneratorKind::UnknownCollective => Drive::Electric(EER_COMPRESSION_FORFAIT),
        CoolingGeneratorKind::GasEngineCompression { gas_engine, .. } => Drive::Gas(
            EER_COMPRESSION_FORFAIT * gas_engine.factors().map_or(0.0, |factors| factors.1),
        ),
        CoolingGeneratorKind::GasAbsorption {
            declared: d,
            rating,
            heat_rejection,
        } => Drive::Gas(
            absorption_method_2(rating.as_ref(), *heat_rejection)
                .unwrap_or_else(|| declared(d, 0.025, ZETA_GAS_ABSORPTION)),
        ),
        CoolingGeneratorKind::AbsorptionExternalHeat {
            rating,
            heat_rejection,
        } => Drive::DistrictHeat(
            absorption_method_2(rating.as_ref(), *heat_rejection)
                .unwrap_or(ZETA_EXTERNAL_HEAT_FACTOR),
        ),
        CoolingGeneratorKind::AbsorptionChp {
            chp,
            rating,
            heat_rejection,
        } => {
            let (thermal, electric) = chp.factors().unwrap_or((0.0, 0.0));
            // Table 10.30: ζ = 1,00·ε_chp;th, so cold/ζ is the CHP fuel;
            // method 2 replaces the 1,00 by PLV·ζ_n·f_prpr (10.66).
            let heat_ratio = absorption_method_2(rating.as_ref(), *heat_rejection).unwrap_or(1.0);
            Drive::ChpHeat {
                zeta: heat_ratio * thermal,
                electric,
            }
        }
        CoolingGeneratorKind::ExternalCold => Drive::DistrictCold,
        CoolingGeneratorKind::FreeCooling { source, .. } => Drive::FreeCooling(source.eer()),
    }
}

/// 10.66: `PLV·ζ_n·f_prpr` with PLV 0,95 (10.67) and f_prpr 0,60 for direct
/// condensation against outdoor air, 0,9 otherwise (§10.5.5.1).
fn absorption_method_2(
    rating: Option<&AbsorptionRating>,
    heat_rejection: Option<HeatRejection>,
) -> Option<f64> {
    let rating = rating?;
    let practice = if heat_rejection == Some(HeatRejection::AirCooled) {
        PRACTICE_FACTOR_DIRECT_CONDENSATION
    } else {
        PRACTICE_FACTOR_OTHER
    };
    Some(ABSORPTION_PLV * rating.nominal_heat_ratio * practice)
}

/// 10.81 for method 2 absorption: `ζ_n·f_C;PL` with f_C;PL = 0,95.
fn rated_heat_ratio(kind: &CoolingGeneratorKind) -> Option<f64> {
    match kind {
        CoolingGeneratorKind::GasAbsorption { rating, .. }
        | CoolingGeneratorKind::AbsorptionExternalHeat { rating, .. }
        | CoolingGeneratorKind::AbsorptionChp { rating, .. } => rating
            .as_ref()
            .map(|rating| ABSORPTION_PLV * rating.nominal_heat_ratio),
        _ => None,
    }
}

fn heat_rejection(kind: &CoolingGeneratorKind) -> Option<HeatRejection> {
    match kind {
        CoolingGeneratorKind::Compression { heat_rejection, .. }
        | CoolingGeneratorKind::GasEngineCompression { heat_rejection, .. }
        | CoolingGeneratorKind::GasAbsorption { heat_rejection, .. }
        | CoolingGeneratorKind::AbsorptionExternalHeat { heat_rejection, .. }
        | CoolingGeneratorKind::AbsorptionChp { heat_rejection, .. } => *heat_rejection,
        _ => None,
    }
}

/// T_0;abs of 10.56, 10.63 and 10.73, K.
const T0_ABS: f64 = 273.16;
/// 10.54: f_prpr;si of method 1.
const PRACTICE_FACTOR_METHOD_1: f64 = 0.9;
/// §10.5.5.1: f_prpr;si of method 2 with direct condensation against
/// outdoor air (principle 2), and for other systems.
const PRACTICE_FACTOR_DIRECT_CONDENSATION: f64 = 0.60;
const PRACTICE_FACTOR_OTHER: f64 = 0.9;

/// Interpretations of the methods 1/2 and 10.20 routes.
pub const COOLING_INTERPRETATIONS: &[&str] = &[
    "10.66 with an absorption chiller on building CHP: the table 10.30 factor 1,00 is replaced by PLV·ζ_n·f_prpr, so the CHP fuel is Q_C/(PLV·ζ_n·f_prpr·ε_chp;th)",
    "absorption method 2: f_prpr 0,60 only for an air-cooled absorber (direct condensation, principle 2), 0,9 otherwise or when the heat rejection is not given",
    "10.61/10.73: ϑ_C;gen;req;out is the distribution supply temperature of table 10.8 (6 °C without distribution) for chillers and 24 °C (10.10) for evaporation in the room, unless declared",
    "10.55: months without bins in table 10.18 (January, December) use the 14 °C bin",
    "10.56/10.57: a bin with a non-positive temperature lift uses f_EER;gi;bn = 1",
    "10.68: Q_C;gen;in;req;si;mi of a generator is its share of the generator cold (10.52); method 2 counts all of it in 10.65, 10.70–10.72 only report the coverage",
    "tables 10.24/10.27 have no column for a cooling limit of 14 °C; the ≥15 column is used",
    "10.20 as printed: θ_SUP;dis;out − Δθ_hr − Δθ_rca + Δθ_fan, with the signed temperature changes of chapter 11",
];

/// Table 10.17 `f_Q;C;bn`, rows ϑ_e;bn 14–32 °C, columns cooling limit
/// ≥14 … ≥25 °C, 1/h.
const FQC_BIN: [[f64; 12]; 19] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [
        0.0001, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0002, 0.0001, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0002, 0.0002, 0.0001, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0003, 0.0003, 0.0002, 0.0002, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0004, 0.0004, 0.0004, 0.0003, 0.0002, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0005, 0.0005, 0.0005, 0.0005, 0.0004, 0.0003, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0005, 0.0006, 0.0006, 0.0006, 0.0006, 0.0006, 0.0004, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0006, 0.0007, 0.0007, 0.0008, 0.0008, 0.0008, 0.0007, 0.0005, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0007, 0.0008, 0.0009, 0.001, 0.0011, 0.0011, 0.0011, 0.001, 0.0007, 0.0, 0.0, 0.0,
    ],
    [
        0.0008, 0.0009, 0.001, 0.0011, 0.0013, 0.0014, 0.0015, 0.0015, 0.0013, 0.0009, 0.0, 0.0,
    ],
    [
        0.0008, 0.001, 0.0011, 0.0013, 0.0015, 0.0017, 0.0018, 0.002, 0.002, 0.0019, 0.0013, 0.0,
    ],
    [
        0.0009, 0.001, 0.0012, 0.0014, 0.0017, 0.0019, 0.0022, 0.0025, 0.0027, 0.0028, 0.0027,
        0.0021,
    ],
    [
        0.001, 0.0011, 0.0014, 0.0016, 0.0019, 0.0022, 0.0026, 0.003, 0.0034, 0.0037, 0.004, 0.0041,
    ],
    [
        0.0011, 0.0012, 0.0015, 0.0018, 0.0021, 0.0025, 0.0029, 0.0034, 0.004, 0.0046, 0.0054,
        0.0062,
    ],
    [
        0.0011, 0.0013, 0.0016, 0.0019, 0.0023, 0.0028, 0.0033, 0.0039, 0.0047, 0.0056, 0.0067,
        0.0082,
    ],
    [
        0.0012, 0.0014, 0.0017, 0.0021, 0.0025, 0.0031, 0.0037, 0.0044, 0.0054, 0.0065, 0.0081,
        0.0103,
    ],
    [
        0.0013, 0.0015, 0.0018, 0.0022, 0.0027, 0.0033, 0.0041, 0.0049, 0.006, 0.0074, 0.0094,
        0.0124,
    ],
    [
        0.0014, 0.0016, 0.002, 0.0024, 0.003, 0.0036, 0.0044, 0.0054, 0.0067, 0.0084, 0.0108,
        0.0144,
    ],
];

/// Table 10.18 `f_t;bn;mi`, rows ϑ_e;bn 14–32 °C, columns January–December.
const FT_BIN: [[f64; 12]; 19] = [
    [
        0.0, 0.025, 0.0296, 0.1014, 0.0444, 0.0847, 0.0986, 0.0887, 0.0264, 0.0094, 0.0223, 0.0,
    ],
    [
        0.0, 0.0014, 0.0403, 0.1292, 0.0712, 0.1048, 0.1125, 0.0645, 0.0236, 0.0121, 0.0134, 0.0,
    ],
    [
        0.0, 0.0, 0.0699, 0.1292, 0.078, 0.1022, 0.1222, 0.0524, 0.0167, 0.0108, 0.0164, 0.0,
    ],
    [
        0.0, 0.0, 0.0309, 0.1083, 0.086, 0.0981, 0.125, 0.0417, 0.0292, 0.0094, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0309, 0.0889, 0.1304, 0.1089, 0.0792, 0.0376, 0.0153, 0.0054, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0188, 0.0514, 0.1129, 0.0887, 0.0556, 0.0202, 0.0139, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0067, 0.0278, 0.0941, 0.0618, 0.0458, 0.0188, 0.0167, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0081, 0.0194, 0.0672, 0.0538, 0.0264, 0.0134, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.004, 0.0319, 0.0538, 0.043, 0.0236, 0.0161, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0153, 0.0296, 0.0269, 0.0139, 0.0215, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0167, 0.0175, 0.0269, 0.0083, 0.0363, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0153, 0.0188, 0.0161, 0.0097, 0.0349, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0028, 0.0161, 0.0282, 0.0069, 0.0215, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0042, 0.0188, 0.0255, 0.0097, 0.0121, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0014, 0.0121, 0.0175, 0.0042, 0.0094, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 0.0134, 0.004, 0.0042, 0.004, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 0.0108, 0.0027, 0.0069, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 0.0094, 0.0013, 0.0014, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0042, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
];

/// Table 10.18 `f_t;tot;mi` as printed.
const FT_TOTAL: [f64; 12] = [
    0.0, 0.0264, 0.2392, 0.7431, 0.8844, 0.8952, 0.7583, 0.4933, 0.1417, 0.0470, 0.0521, 0.0,
];

/// Table 10.24 `ϑ_e;kg;mi`, columns cooling limit ≥15 … ≥25 °C.
const THETA_E_KG: [[f64; 11]; 12] = [
    [
        11.8, 11.8, 11.8, 11.8, 11.8, 11.8, 11.8, 11.8, 11.8, 11.8, 11.8,
    ],
    [
        15.6, 16.1, 16.4, 16.4, 16.4, 16.4, 16.4, 16.4, 16.4, 16.4, 16.4,
    ],
    [
        16.5, 17.1, 17.6, 18.1, 18.2, 18.2, 18.2, 18.2, 18.2, 18.2, 18.2,
    ],
    [
        17.6, 18.0, 18.6, 19.2, 19.7, 20.2, 20.4, 20.4, 20.4, 20.4, 20.4,
    ],
    [
        20.4, 21.3, 22.3, 23.1, 23.9, 24.2, 24.6, 24.9, 25.2, 25.6, 26.5,
    ],
    [
        18.6, 19.3, 20.3, 21.5, 22.6, 23.5, 24.6, 25.6, 26.8, 27.5, 28.2,
    ],
    [
        19.6, 20.2, 21.0, 21.8, 22.7, 23.5, 24.3, 25.1, 25.7, 26.3, 26.9,
    ],
    [
        19.9, 20.4, 20.8, 21.5, 22.4, 23.4, 24.6, 25.7, 26.7, 27.3, 27.9,
    ],
    [
        18.1, 18.9, 19.8, 20.9, 21.9, 22.6, 23.4, 23.8, 24.6, 25.3, 26.6,
    ],
    [
        17.1, 17.7, 18.7, 19.5, 20.4, 21.0, 21.5, 22.0, 22.0, 22.0, 22.0,
    ],
    [
        14.6, 14.6, 14.6, 14.6, 14.6, 14.6, 14.6, 14.6, 14.6, 14.6, 14.6,
    ],
    [
        12.6, 12.6, 12.6, 12.6, 12.6, 12.6, 12.6, 12.6, 12.6, 12.6, 12.6,
    ],
];

/// Table 10.27 `ϑ_C;hr;wb;mi`, columns cooling limit ≥15 … ≥25 °C.
const THETA_WET_BULB: [[f64; 11]; 12] = [
    [
        10.1, 10.1, 10.1, 10.1, 10.1, 10.1, 10.1, 10.1, 10.1, 10.1, 10.1,
    ],
    [
        11.7, 11.7, 12.4, 12.4, 12.4, 12.4, 12.4, 12.4, 12.4, 12.4, 12.4,
    ],
    [
        10.7, 11.0, 11.7, 12.7, 13.2, 13.2, 13.2, 13.2, 13.2, 13.2, 13.2,
    ],
    [
        11.9, 12.1, 12.2, 12.6, 13.0, 12.6, 15.9, 15.9, 15.9, 15.9, 15.9,
    ],
    [
        14.8, 15.2, 15.6, 16.0, 16.2, 16.4, 16.4, 16.5, 16.6, 16.7, 17.1,
    ],
    [
        14.8, 15.2, 15.7, 16.8, 17.7, 18.4, 19.0, 19.6, 20.2, 20.6, 21.0,
    ],
    [
        16.5, 16.9, 17.4, 17.8, 18.3, 18.8, 19.1, 19.6, 19.8, 20.2, 20.6,
    ],
    [
        17.2, 17.4, 17.7, 18.1, 18.5, 19.1, 19.8, 20.4, 20.9, 21.1, 21.3,
    ],
    [
        15.5, 16.0, 16.7, 17.5, 18.1, 18.4, 19.0, 19.3, 19.8, 20.4, 20.7,
    ],
    [
        15.0, 15.4, 15.8, 16.4, 16.9, 17.5, 17.7, 18.4, 18.4, 18.4, 18.4,
    ],
    [
        12.5, 12.5, 12.5, 12.5, 12.5, 12.5, 12.5, 12.5, 12.5, 12.5, 12.5,
    ],
    [
        11.7, 11.7, 11.7, 11.7, 11.7, 11.7, 11.7, 11.7, 11.7, 11.7, 11.7,
    ],
];

/// Table 10.21: chillers with condenser heat to outdoor air / to water.
const CHILLER_PART_LOAD_AIR: [f64; 10] =
    [0.83, 0.87, 0.92, 0.95, 0.98, 1.00, 1.01, 1.02, 1.01, 1.00];
const CHILLER_PART_LOAD_WATER: [f64; 10] =
    [0.96, 0.94, 0.92, 0.90, 0.90, 0.90, 0.92, 0.94, 0.96, 1.00];

/// Column of tables 10.24/10.27 (≥15 … ≥25) for a cooling limit.
fn limit_column_from_15(limit_c: f64) -> usize {
    (limit_c.round().clamp(15.0, 25.0) - 15.0) as usize
}

/// Table 10.31 `p_hr;el;si` (10.82), kW/kW; zero for direct condensation.
fn rejection_fan_power(rejection: HeatRejection, without_silencer: bool) -> f64 {
    match (rejection, without_silencer) {
        (HeatRejection::OpenCoolingTower, true) => 0.033,
        (HeatRejection::OpenCoolingTower, false) => 0.040,
        (HeatRejection::ClosedCoolingTower, true) => 0.018,
        (HeatRejection::ClosedCoolingTower, false) => 0.021,
        (HeatRejection::DryCooler, true) => 0.045,
        (HeatRejection::DryCooler, false) => 0.054,
        _ => 0.0,
    }
}

/// Gaussian elimination with partial pivoting; `None` when singular.
fn solve_linear(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let pivot = (col..n).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..n {
            let factor = a[row][col] / a[col][col];
            let pivot_row = a[col].clone();
            for (value, pivot_value) in a[row].iter_mut().zip(&pivot_row).skip(col) {
                *value -= factor * pivot_value;
            }
            b[row] -= factor * b[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let sum: f64 = (row + 1..n).map(|k| a[row][k] * x[k]).sum();
        x[row] = (b[row] - sum) / a[row][row];
    }
    x.iter().all(|value| value.is_finite()).then_some(x)
}

/// 10.63: `[C1, C2, C3, C4, Δϑ_corr]` from the NEN-EN 14825 points; without
/// a fifth point 10.64 applies with `Δϑ_corr = 0` (its equation repeats C).
pub fn en14825_coefficients(performance: &En14825Performance) -> Option<[f64; 5]> {
    if performance.test_points.len() != 4 || performance.nominal_eer <= 0.0 {
        return None;
    }
    let with_fifth = performance.fifth_point.is_some();
    let points: Vec<En14825Point> = performance
        .test_points
        .iter()
        .copied()
        .chain(performance.fifth_point)
        .collect();
    let mut a = Vec::new();
    let mut b = Vec::new();
    for point in &points {
        let x = point.part_load_percent / 100.0;
        let ratio = point.eer / performance.nominal_eer;
        let absolute = T0_ABS + point.evaporator_outlet_c;
        let mut row = vec![x.powi(3), x.powi(2), x, 1.0];
        if with_fifth {
            row.push(-ratio * x / absolute);
        }
        a.push(row);
        b.push(ratio * (point.condenser_inlet_c - point.evaporator_outlet_c) / absolute);
    }
    let x = solve_linear(a, b)?;
    Some([x[0], x[1], x[2], x[3], if with_fifth { x[4] } else { 0.0 }])
}

/// 10.56/10.57: `f_EER;gi;bn`; part loads in %.
fn en14825_bin_factor(
    coefficients: &[f64; 5],
    part_load_percent: f64,
    minimum_percent: f64,
    evaporator_c: f64,
    condenser_c: f64,
) -> f64 {
    let x = part_load_percent.max(minimum_percent) / 100.0;
    let lift = condenser_c - evaporator_c + x * coefficients[4];
    if lift <= 0.0 {
        return 1.0;
    }
    let [c1, c2, c3, c4, _] = *coefficients;
    (T0_ABS + evaporator_c) / lift * (c1 * x.powi(3) + c2 * x.powi(2) + c3 * x + c4)
}

/// 10.55 with 10.58–10.62: `f_EER;gi;mi` of method 1 for each month, from
/// the generator's annual cold `Σ Q_C;gen;in;mi` (kWh).
pub fn en14825_monthly_factor(
    performance: &En14825Performance,
    coefficients: &[f64; 5],
    evaporator_c: f64,
    cooling_limit_c: f64,
    annual_cold_kwh: f64,
    building_fraction: f64,
) -> [f64; 12] {
    let column = (cooling_limit_c.round().clamp(14.0, 25.0) - 14.0) as usize;
    let minimum = 100.0 * performance.minimum_capacity_kw / performance.nominal_capacity_kw;
    let bin_factor = |bin: usize| {
        let outdoor = 14.0 + bin as f64;
        // 10.60 and 10.58.
        let net_kw = annual_cold_kwh * FQC_BIN[bin][column] / building_fraction;
        let part_load = 100.0 * net_kw / performance.nominal_capacity_kw;
        // 10.62.
        let condenser = performance
            .condenser_inlet_limit_c
            .map_or(outdoor, |limit| limit.max(outdoor));
        en14825_bin_factor(coefficients, part_load, minimum, evaporator_c, condenser)
    };
    std::array::from_fn(|month| {
        if FT_TOTAL[month] <= 0.0 {
            return bin_factor(0);
        }
        (0..FT_BIN.len())
            .filter(|&bin| FT_BIN[bin][month] > 0.0)
            .map(|bin| FT_BIN[bin][month] / FT_TOTAL[month] * bin_factor(bin))
            .sum()
    })
}

/// 10.69 with table 10.19/10.21: `f_C;PL;k`; 1 below 5 %.
fn part_load_step_factor(table: &[f64; 10], part_load: f64) -> f64 {
    if part_load.is_nan() || part_load < 0.05 {
        return 1.0;
    }
    let step = ((part_load * 10.0 + 0.5 + 1e-9).floor() as usize).clamp(1, 10);
    table[step - 1]
}

/// 10.73: `f_EER;corr`.
pub fn eer_temperature_correction(
    required_outlet_c: f64,
    reference_inlet_c: f64,
    nominal_outlet_c: f64,
    nominal_inlet_c: f64,
    delta_evaporator_k: f64,
    delta_condenser_k: f64,
) -> f64 {
    let carnot = |outlet: f64, inlet: f64| {
        let cold = T0_ABS + outlet - delta_evaporator_k;
        cold / ((T0_ABS + inlet + delta_condenser_k) - cold)
    };
    carnot(required_outlet_c, reference_inlet_c) / carnot(nominal_outlet_c, nominal_inlet_c)
}

/// A compression generator with method 1 or 2 data.
struct Rated<'a> {
    performance: &'a CompressionPerformance,
    room_unit: bool,
    rejection: HeatRejection,
}

fn rated(kind: &CoolingGeneratorKind) -> Option<Rated<'_>> {
    match kind {
        CoolingGeneratorKind::Compression {
            performance: Some(performance),
            heat_rejection,
            ..
        } => Some(Rated {
            performance,
            room_unit: false,
            rejection: heat_rejection.unwrap_or(HeatRejection::AirCooled),
        }),
        CoolingGeneratorKind::RoomAirConditioner {
            performance: Some(performance),
            ..
        } => Some(Rated {
            performance,
            room_unit: true,
            rejection: HeatRejection::AirCooled,
        }),
        _ => None,
    }
}

/// Monthly efficiencies of a method 1 or 2 generator.
struct RatedMonths {
    method: u8,
    coefficients: Option<[f64; 5]>,
    /// Effective EER for the drive energy (10.53/10.65).
    eer: [f64; 12],
    /// EER in 10.80 (without f_prpr).
    rejection_eer: [f64; 12],
    /// 10.70/10.71 cold within the nominal capacity.
    delivered: [f64; 12],
    /// Table 10.31 `p_hr;el` (10.82).
    fan_power: f64,
}

fn rated_months(
    rated: &Rated<'_>,
    cold: &[f64; 12],
    hours: &[f64; 12],
    cooling_limit_c: f64,
    building_fraction: f64,
    supply_c: f64,
) -> RatedMonths {
    let air_cooled = rated.room_unit || rated.rejection == HeatRejection::AirCooled;
    // Table 10.22.
    let delta_evaporator = if rated.room_unit { 20.0 } else { 6.0 };
    let default_outlet = if rated.room_unit {
        COOLING_INITIAL_TEMPERATURE_C
    } else {
        supply_c
    };
    match rated.performance {
        CompressionPerformance::En14825(performance) => {
            let coefficients = en14825_coefficients(performance).unwrap_or([0.0; 5]);
            // 10.61.
            let evaporator =
                performance.required_outlet_c.unwrap_or(default_outlet) - delta_evaporator;
            let annual: f64 = cold.iter().sum();
            let factors = en14825_monthly_factor(
                performance,
                &coefficients,
                evaporator,
                cooling_limit_c,
                annual,
                building_fraction,
            );
            let eer = std::array::from_fn(|index| {
                performance.nominal_eer * factors[index] * PRACTICE_FACTOR_METHOD_1
            });
            let rejection_eer =
                std::array::from_fn(|index| performance.nominal_eer * factors[index]);
            RatedMonths {
                method: 1,
                coefficients: Some(coefficients),
                eer,
                rejection_eer,
                delivered: *cold,
                // Method 1 applies to direct condensation only (10.82 = 0).
                fan_power: 0.0,
            }
        }
        CompressionPerformance::En14511(performance) => {
            let delta_condenser = if !air_cooled {
                4.0
            } else if performance.heat_rejection_to_exhaust_air {
                20.0
            } else {
                10.0
            };
            let column = limit_column_from_15(cooling_limit_c);
            let outlet = performance.required_outlet_c.unwrap_or(default_outlet);
            let table: [f64; 10] = match performance.room_unit_type {
                Some(kind) if rated.room_unit => kind.part_load_factors(),
                _ if air_cooled => CHILLER_PART_LOAD_AIR,
                _ => CHILLER_PART_LOAD_WATER,
            };
            let practice = if air_cooled {
                PRACTICE_FACTOR_DIRECT_CONDENSATION
            } else {
                PRACTICE_FACTOR_OTHER
            };
            let mut eer = [0.0; 12];
            let mut rejection_eer = [0.0; 12];
            let mut delivered = [0.0; 12];
            for index in 0..12 {
                // 10.74/10.75 with table 10.26.
                let reference = if air_cooled {
                    THETA_E_KG[index][column]
                } else {
                    match rated.rejection {
                        HeatRejection::OpenCoolingTower | HeatRejection::ClosedCoolingTower => {
                            THETA_WET_BULB[index][column] + 6.0
                        }
                        HeatRejection::DryCooler => THETA_E_KG[index][column] + 15.0,
                        _ => 35.0,
                    }
                };
                let correction = eer_temperature_correction(
                    outlet,
                    reference,
                    performance.nominal_evaporator_outlet_c,
                    performance.nominal_condenser_inlet_c,
                    delta_evaporator,
                    delta_condenser,
                );
                // 10.68 and 10.69.
                let capacity_kwh =
                    hours[index] * performance.nominal_capacity_kw * building_fraction;
                let part_load = if capacity_kwh > 0.0 {
                    cold[index] / capacity_kwh
                } else if cold[index] > 0.0 {
                    f64::INFINITY
                } else {
                    0.0
                };
                let plv = part_load_step_factor(&table, part_load);
                rejection_eer[index] = performance.nominal_eer * plv * correction;
                eer[index] = rejection_eer[index] * practice;
                // 10.70/10.71.
                delivered[index] = if part_load <= 1.0 {
                    cold[index]
                } else {
                    capacity_kwh
                };
            }
            RatedMonths {
                method: 2,
                coefficients: None,
                eer,
                rejection_eer,
                delivered,
                fan_power: if air_cooled {
                    0.0
                } else {
                    rejection_fan_power(rated.rejection, performance.axial_fans_without_silencer)
                },
            }
        }
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
    let mut limit_need = [0.0; 12];
    for zone in context.zones {
        for (sum, need) in total_need.iter_mut().zip(zone.need_kwh) {
            *sum += need;
        }
        for (sum, need) in limit_need
            .iter_mut()
            .zip(zone.limit_need_kwh.unwrap_or(zone.need_kwh))
        {
            *sum += need;
        }
    }
    // 10.19 steps 1–6.
    let limit = cooling_limit_c(&limit_need);
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
    let mut shares: Vec<GeneratorShare> = system
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
            method: 3,
            monthly_eer: Vec::new(),
            en14825_coefficients: None,
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
    let mut base_auxiliary = [0.0; 12];
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
        // 12.5 with table 12.2.
        let dehumidification = if system.emission.emitter.radiant() {
            0.0
        } else {
            let design = match system
                .distribution
                .as_ref()
                .map(|item| item.design_temperature)
            {
                Some(CoolingDesignTemperature::T12To16) => {
                    crate::humidification::CoolingDesignTemperature::From12To16
                }
                Some(CoolingDesignTemperature::T12To18) => {
                    crate::humidification::CoolingDesignTemperature::From12To18
                }
                Some(CoolingDesignTemperature::T17To21) => {
                    crate::humidification::CoolingDesignTemperature::From17To21
                }
                Some(CoolingDesignTemperature::T6To12OrUnknown) | None => {
                    crate::humidification::CoolingDesignTemperature::From6To12
                }
            };
            design.fraction(index) * need
        };
        let ahu: f64 = context
            .zones
            .iter()
            .map(|zone| zone.ahu_load_kwh[index])
            .sum();
        let has_load = need + emission + distribution + dehumidification + ahu > 0.0;
        let pump = if has_load { pump_energy[index] } else { 0.0 };
        // 10.45
        let recovered = (1.0 - PUMP_RECOVERABLE_FACTOR) * pump;
        let load = need + emission + distribution + recovered + dehumidification + ahu;
        // 10.7–10.9: the booster heat pump is limited to the load.
        let extraction = booster[index].min(load);
        let generator_cold = load - extraction;
        let row = CoolingMonth {
            month: index as u8 + 1,
            operating_hours: hours[index],
            need_kwh: need,
            emission_loss_kwh: emission,
            distribution_loss_kwh: distribution,
            pump_recovered_kwh: recovered,
            booster_extraction_kwh: extraction,
            dehumidification_kwh: dehumidification,
            ahu_cooling_kwh: ahu,
            generator_cold_kwh: generator_cold,
            ..CoolingMonth::default()
        };
        // 10.17/10.18
        base_auxiliary[index] =
            FAN_COIL_POWER_W * f64::from(system.emission.fan_coil_count) * hours[index] / 1000.0
                + pump;
        months.push(row);
    }
    // 10.52: cold per generator and month.
    let generator_cold: Vec<[f64; 12]> = system
        .generators
        .iter()
        .map(|generator| {
            std::array::from_fn(|index| {
                generator_share(generator, (6..=8).contains(&index))
                    * months[index].generator_cold_kwh
            })
        })
        .collect();
    // Methods 1 and 2 (§10.5.4/10.5.5).
    let supply_c = system
        .distribution
        .as_ref()
        .map_or(6.0, |item| item.design_temperature.supply_return_c().0);
    let rated_months: Vec<Option<RatedMonths>> = system
        .generators
        .iter()
        .zip(&generator_cold)
        .map(|(generator, cold)| {
            rated(&generator.generator)
                .map(|rated| rated_months(&rated, cold, &hours, limit, f_building, supply_c))
        })
        .collect();
    for (share, rated) in shares.iter_mut().zip(&rated_months) {
        if let Some(rated) = rated {
            share.method = rated.method;
            share.monthly_eer = rated.eer.to_vec();
            share.en14825_coefficients = rated.coefficients.map(|values| values.to_vec());
        }
    }
    let mut free_cold_by_generator = vec![[0.0; 12]; system.generators.len()];
    for (index, row) in months.iter_mut().enumerate() {
        let mut auxiliary = base_auxiliary[index];
        let mut machine_present = false;
        let mut coverage: Option<(f64, f64)> = None;
        for (generator_index, generator) in system.generators.iter().enumerate() {
            let cold = generator_cold[generator_index][index];
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
            if let Some(rated) = &rated_months[generator_index] {
                // 10.53/10.65.
                if rated.eer[index] > 0.0 {
                    row.electricity_kwh += cold / rated.eer[index];
                }
                // 10.80 with 10.82/10.83.
                let rejected = cold * (1.0 + 1.0 / rated.rejection_eer[index]);
                auxiliary += rejected * (rejection.distribution_power() + rated.fan_power);
                if rated.method == 2 {
                    let (delivered, required) = coverage.get_or_insert((0.0, 0.0));
                    *delivered += rated.delivered[index];
                    *required += cold;
                }
                continue;
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
                        rated_heat_ratio(&generator.generator).unwrap_or(efficiency)
                    };
                    // 10.80b/10.81 and 10.83.
                    auxiliary += cold * (1.0 + 1.0 / ratio) * rejection.distribution_power();
                }
                Drive::DistrictHeat(zeta) => {
                    row.district_heat_kwh += cold / zeta;
                    row.district_heat_cold_kwh += cold;
                    let ratio = rated_heat_ratio(&generator.generator).unwrap_or(zeta);
                    auxiliary += cold * (1.0 + 1.0 / ratio) * rejection.distribution_power();
                }
                Drive::ChpHeat { zeta, electric } => {
                    let fuel = cold / zeta;
                    row.chp_heat_kwh += fuel;
                    row.chp_electricity_kwh += fuel * electric;
                    // 10.81 with the absorber's own heat ratio: the 1,00 of
                    // table 10.30 (ε_chp;th only converts to CHP fuel), or
                    // ζ_n·PLV for method 2.
                    let ratio = rated_heat_ratio(&generator.generator).unwrap_or(1.0);
                    auxiliary += cold * (1.0 + 1.0 / ratio) * rejection.distribution_power();
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
        // 10.72.
        row.part_load_coverage = coverage.map(|(delivered, required)| {
            if required > 0.0 {
                (delivered / required).min(1.0)
            } else {
                1.0
            }
        });
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
        interpretations: COOLING_INTERPRETATIONS.to_vec(),
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
            performance: None,
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
    fn absorption_on_external_heat_reports_its_cold_output() {
        let input = system(vec![generator(
            CoolingGeneratorKind::AbsorptionExternalHeat {
                heat_rejection: None,
                rating: None,
            },
            None,
        )]);
        let zones = [CoolingZoneNeed {
            usable_floor_area_m2: 100.0,
            need_kwh: summer_need(),
            ahu_load_kwh: [0.0; 12],
            limit_need_kwh: None,
        }];
        let result = assess_cooling(&input, context(&zones));
        let july = &result.months[6];
        // 5.39g counts Q_C;gen;out; the heat input is Q_C;gen;out/ζ.
        assert!(july.district_heat_cold_kwh > 0.0);
        assert!((july.district_heat_cold_kwh - july.generator_cold_kwh).abs() < 1e-9);
        assert!(july.district_heat_kwh > july.district_heat_cold_kwh);
    }

    #[test]
    fn absorption_method_2_follows_10_66() {
        let rated = |rejection: HeatRejection| {
            system(vec![generator(
                CoolingGeneratorKind::GasAbsorption {
                    heat_rejection: Some(rejection),
                    declared: None,
                    rating: Some(AbsorptionRating {
                        nominal_heat_ratio: 1.2,
                        source_reference: "EN 14511 report".into(),
                    }),
                },
                None,
            )])
        };
        let zones = [CoolingZoneNeed {
            usable_floor_area_m2: 100.0,
            need_kwh: summer_need(),
            ahu_load_kwh: [0.0; 12],
            limit_need_kwh: None,
        }];
        let tower = rated(HeatRejection::OpenCoolingTower);
        assert!(validate_cooling(&tower, "cooling").is_empty());
        let result = assess_cooling(&tower, context(&zones));
        let july = &result.months[6];
        // 10.66: Q_H = Q_C / (PLV 0,95 · ζ_n 1,2 · f_prpr 0,9).
        assert!((july.natural_gas_kwh - july.generator_cold_kwh / (0.95 * 1.2 * 0.9)).abs() < 1e-9);
        // Direct condensation against outdoor air: f_prpr 0,60.
        let air = assess_cooling(&rated(HeatRejection::AirCooled), context(&zones));
        let july_air = &air.months[6];
        assert!(
            (july_air.natural_gas_kwh - july_air.generator_cold_kwh / (0.95 * 1.2 * 0.6)).abs()
                < 1e-9
        );
        // A declared value and a rating together are rejected.
        let mut both = rated(HeatRejection::OpenCoolingTower);
        if let CoolingGeneratorKind::GasAbsorption { declared, .. } =
            &mut both.generators[0].generator
        {
            *declared = Some(DeclaredEfficiency {
                value: 0.9,
                source_reference: "statement".into(),
            });
        }
        assert!(validate_cooling(&both, "cooling")
            .iter()
            .any(|item| item.code == "cooling_declared_and_performance"));
    }

    #[test]
    fn emission_loss_follows_10_15_with_floor_of_0_15() {
        let input = system(vec![generator(compression(), None)]);
        let zones = [CoolingZoneNeed {
            usable_floor_area_m2: 100.0,
            need_kwh: summer_need(),
            ahu_load_kwh: [0.0; 12],
            limit_need_kwh: None,
        }];
        let result = assess_cooling(&input, context(&zones));
        // Δϑ = −0,5 + 0 − 1,25 = −1,75; ϑ_int,inc = 22,25.
        let july = &result.months[6];
        let combined = 18.05 + 8.0;
        let ratio: f64 = (-1.75_f64 / (22.25 - combined)).max(0.15);
        assert!((july.emission_loss_kwh - 180.0 * ratio).abs() < 1e-9);
        // April: 22,25 − 17,32 > 0, no loss.
        assert_eq!(result.months[3].emission_loss_kwh, 0.0);
        // 12.5: no distribution (direct expansion) counts as 6/12 °C, July 1,84.
        assert!((july.dehumidification_kwh - 1.84 * 180.0).abs() < 1e-9);
        let cold = july.need_kwh + july.emission_loss_kwh + july.dehumidification_kwh;
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
            ahu_load_kwh: [0.0; 12],
            limit_need_kwh: None,
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
            ahu_load_kwh: [0.0; 12],
            limit_need_kwh: None,
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
            ahu_load_kwh: [0.0; 12],
            limit_need_kwh: None,
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
            performance: None,
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
                ahu_load_kwh: [0.0; 12],
                limit_need_kwh: None,
            },
            CoolingZoneNeed {
                usable_floor_area_m2: 40.0,
                need_kwh: [0.0; 12],
                ahu_load_kwh: [0.0; 12],
                limit_need_kwh: None,
            },
        ];
        let result = assess_cooling(&input, context(&zones));
        let july = &result.months[6];
        assert!((july.booster_extraction_kwh - 30.0).abs() < 1e-12);
        let load = july.need_kwh + july.emission_loss_kwh + july.dehumidification_kwh;
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

    fn rac(performance: CompressionPerformance) -> CoolingGeneratorKind {
        CoolingGeneratorKind::RoomAirConditioner {
            declared: None,
            performance: Some(performance),
        }
    }

    fn rated_zones() -> [CoolingZoneNeed; 1] {
        [CoolingZoneNeed {
            usable_floor_area_m2: 100.0,
            need_kwh: summer_need(),
            ahu_load_kwh: [0.0; 12],
            limit_need_kwh: None,
        }]
    }

    #[test]
    fn bin_tables_match_their_printed_totals() {
        for month in 0..12 {
            let sum: f64 = FT_BIN.iter().map(|row| row[month]).sum();
            assert!((sum - FT_TOTAL[month]).abs() < 2.5e-4, "month {month}");
        }
        // Table 10.17: limit ≥ 25 has no demand below 26 °C.
        assert_eq!(FQC_BIN[11][11], 0.0);
        assert_eq!(FQC_BIN[18][11], 0.0144);
    }

    fn point(part_load: f64, condenser: f64, coefficients: [f64; 5], nominal: f64) -> En14825Point {
        // 10.56 evaluated at the point gives EER/EER_n.
        let x = part_load / 100.0;
        let [c1, c2, c3, c4, delta] = coefficients;
        let ratio = (T0_ABS + 7.0) / (condenser - 7.0 + x * delta)
            * (c1 * x.powi(3) + c2 * x.powi(2) + c3 * x + c4);
        En14825Point {
            part_load_percent: part_load,
            eer: ratio * nominal,
            evaporator_outlet_c: 7.0,
            condenser_inlet_c: condenser,
        }
    }

    fn en14825(coefficients: [f64; 5], fifth: bool) -> En14825Performance {
        En14825Performance {
            nominal_eer: 4.0,
            nominal_capacity_kw: 10.0,
            minimum_capacity_kw: 2.0,
            test_points: vec![
                point(100.0, 35.0, coefficients, 4.0),
                point(74.0, 30.0, coefficients, 4.0),
                point(47.0, 25.0, coefficients, 4.0),
                point(21.0, 20.0, coefficients, 4.0),
            ],
            fifth_point: fifth.then(|| point(47.0, 35.0, coefficients, 4.0)),
            condenser_inlet_limit_c: None,
            required_outlet_c: None,
            source_reference: "EN 14825 report".into(),
        }
    }

    #[test]
    fn en14825_coefficients_solve_10_63() {
        let truth = [0.2, -0.5, 0.3, 0.9, 3.0];
        let solved = en14825_coefficients(&en14825(truth, true)).unwrap();
        for (a, b) in solved.iter().zip(truth) {
            assert!((a - b).abs() < 1e-8, "{solved:?}");
        }
        // Without the fifth point: 10.64 with Δϑ_corr = 0.
        let truth = [0.1, -0.2, 0.15, 0.95, 0.0];
        let solved = en14825_coefficients(&en14825(truth, false)).unwrap();
        for (a, b) in solved.iter().zip(truth) {
            assert!((a - b).abs() < 1e-8, "{solved:?}");
        }
        // Four identical points are singular.
        let mut singular = en14825(truth, false);
        let first = singular.test_points[0];
        singular.test_points = vec![first; 4];
        assert!(en14825_coefficients(&singular).is_none());
    }

    #[test]
    fn en14825_monthly_factor_follows_10_55_to_10_62() {
        let mut performance = en14825([0.0; 5], false);
        performance.condenser_inlet_limit_c = Some(40.0);
        // C4 only: f_EER;bn = (273,16 + 1)/(40 − 1)·0,05 in every bin.
        let constant = 274.16 / 39.0 * 0.05;
        let factors = en14825_monthly_factor(
            &performance,
            &[0.0, 0.0, 0.0, 0.05, 0.0],
            1.0,
            18.0,
            0.0,
            1.0,
        );
        // April: Σ f_t;bn = 0,7432 against the printed total 0,7431.
        assert!((factors[3] - 0.7432 / 0.7431 * constant).abs() < 1e-12);
        // January has no bins: the 14 °C bin.
        assert!((factors[0] - constant).abs() < 1e-12);
        // C3 only: without demand the minimum part load (20 %) applies
        // (10.57); in July Σ f_t;bn equals the printed 0,7583.
        let factors = en14825_monthly_factor(
            &performance,
            &[0.0, 0.0, 1.0, 0.0, 0.0],
            1.0,
            18.0,
            0.0,
            1.0,
        );
        assert!((factors[6] - 274.16 / 39.0 * 0.2).abs() < 1e-12);
        // 10.60/10.58: 10 000 kWh, limit 18 °C, bin 25 °C: 15 kW → 150 %.
        let bin = en14825_bin_factor(&[0.0, 0.0, 1.0, 0.0, 0.0], 150.0, 20.0, 1.0, 40.0);
        assert!((bin - 274.16 / 39.0 * 1.5).abs() < 1e-12);
        let net = 10_000.0 * FQC_BIN[11][4];
        assert!((net - 15.0).abs() < 1e-9);
    }

    #[test]
    fn method_1_room_unit_uses_en14825_eer() {
        let truth = [0.1, -0.2, 0.15, 0.95, 0.0];
        let performance = en14825(truth, false);
        let input = system(vec![generator(
            rac(CompressionPerformance::En14825(performance.clone())),
            None,
        )]);
        assert!(validate_cooling(&input, "cooling").is_empty());
        let zones = rated_zones();
        let result = assess_cooling(&input, context(&zones));
        assert_eq!(result.generator_shares[0].method, 1);
        let coefficients = en14825_coefficients(&performance).unwrap();
        let annual: f64 = result.months.iter().map(|row| row.generator_cold_kwh).sum();
        // 10.61: 24 °C in the room minus Δϑ_evap 20 K.
        let factors = en14825_monthly_factor(
            &performance,
            &coefficients,
            4.0,
            result.cooling_limit_c,
            annual,
            1.0,
        );
        let july = &result.months[6];
        let eer = 4.0 * factors[6] * 0.9;
        assert!((july.electricity_kwh - july.generator_cold_kwh / eer).abs() < 1e-9);
        assert!((result.generator_shares[0].monthly_eer[6] - eer).abs() < 1e-12);
        // Direct condensation: no condenser auxiliaries, control only.
        assert!((july.auxiliary_electricity_kwh - 7.44).abs() < 1e-9);
    }

    #[test]
    fn method_2_room_unit_follows_10_65_to_10_74() {
        let input = system(vec![generator(
            rac(CompressionPerformance::En14511(En14511Performance {
                nominal_eer: 4.0,
                nominal_capacity_kw: 2.0,
                nominal_evaporator_outlet_c: 24.0,
                nominal_condenser_inlet_c: 35.0,
                room_unit_type: Some(RoomUnitType::SplitInverter),
                heat_rejection_to_exhaust_air: false,
                axial_fans_without_silencer: false,
                required_outlet_c: None,
                source_reference: "EN 14511 datasheet".into(),
            })),
            None,
        )]);
        assert!(validate_cooling(&input, "cooling").is_empty());
        let zones = rated_zones();
        let result = assess_cooling(&input, context(&zones));
        let july = &result.months[6];
        let column = limit_column_from_15(result.cooling_limit_c);
        let reference = THETA_E_KG[6][column];
        // 10.73 with ϑ_req;out = 24 °C, Δϑ_evap 20 K, Δϑ_cond 10 K:
        // both cold sides are 277,16 K, so f = (35 + 6)/(ϑ_e;kg + 6).
        let correction = 41.0 / (reference + 6.0);
        // 10.68/10.69 with table 10.19 row C.
        let part_load = july.generator_cold_kwh / (july.operating_hours * 2.0);
        let row = [1.52, 1.54, 1.57, 1.69, 1.45, 1.31, 1.21, 1.09, 1.03, 0.95];
        let plv = if part_load < 0.05 {
            1.0
        } else {
            row[((part_load * 10.0 + 0.5).floor() as usize).clamp(1, 10) - 1]
        };
        // §10.5.5.1: f_prpr 0,60 for direct condensation.
        let eer = plv * 4.0 * correction * 0.60;
        assert!((july.electricity_kwh - july.generator_cold_kwh / eer).abs() < 1e-9);
        let coverage = (july.operating_hours * 2.0 / july.generator_cold_kwh).min(1.0);
        assert!((july.part_load_coverage.unwrap() - coverage).abs() < 1e-12);
        assert_eq!(result.generator_shares[0].method, 2);
    }

    #[test]
    fn method_2_water_cooled_chiller_adds_condenser_auxiliaries() {
        let input = system(vec![generator(
            CoolingGeneratorKind::Compression {
                heat_rejection: Some(HeatRejection::OpenCoolingTower),
                declared: None,
                performance: Some(CompressionPerformance::En14511(En14511Performance {
                    nominal_eer: 5.0,
                    nominal_capacity_kw: 50.0,
                    nominal_evaporator_outlet_c: 7.0,
                    nominal_condenser_inlet_c: 30.0,
                    room_unit_type: None,
                    heat_rejection_to_exhaust_air: false,
                    axial_fans_without_silencer: false,
                    required_outlet_c: None,
                    source_reference: "EN 14511 datasheet".into(),
                })),
            },
            None,
        )]);
        assert!(validate_cooling(&input, "cooling").is_empty());
        let zones = rated_zones();
        let result = assess_cooling(&input, context(&zones));
        let july = &result.months[6];
        let column = limit_column_from_15(result.cooling_limit_c);
        // Table 10.26: wet cooling tower ϑ_wb + 6; ϑ_req;out 6 °C without
        // distribution; Δϑ_evap 6 K, Δϑ_cond 4 K.
        let reference = THETA_WET_BULB[6][column] + 6.0;
        let carnot = |outlet: f64, inlet: f64| (T0_ABS + outlet - 6.0) / (inlet + 10.0 - outlet);
        let correction = carnot(6.0, reference) / carnot(7.0, 30.0);
        let part_load = july.generator_cold_kwh / (july.operating_hours * 50.0);
        let row = [0.96, 0.94, 0.92, 0.90, 0.90, 0.90, 0.92, 0.94, 0.96, 1.00];
        let plv = if part_load < 0.05 {
            1.0
        } else {
            row[((part_load * 10.0 + 0.5).floor() as usize).clamp(1, 10) - 1]
        };
        let cold = july.generator_cold_kwh;
        let eer = plv * 5.0 * correction * 0.9;
        assert!((july.electricity_kwh - cold / eer).abs() < 1e-9);
        // 10.80 without f_prpr; 10.83 open system 0,018; 10.82 table 10.31
        // open circuit with silencer 0,040; control 7,44 kWh.
        let rejected = cold * (1.0 + 1.0 / (plv * 5.0 * correction));
        let auxiliary = 7.44 + rejected * (0.018 + 0.040);
        assert!((july.auxiliary_electricity_kwh - auxiliary).abs() < 1e-9);
    }

    #[test]
    fn rated_generators_are_validated() {
        let rated = CompressionPerformance::En14511(En14511Performance {
            nominal_eer: 4.0,
            nominal_capacity_kw: 2.0,
            nominal_evaporator_outlet_c: 24.0,
            nominal_condenser_inlet_c: 35.0,
            room_unit_type: None,
            heat_rejection_to_exhaust_air: true,
            axial_fans_without_silencer: false,
            required_outlet_c: None,
            source_reference: String::new(),
        });
        let input = system(vec![
            generator(rac(rated), None),
            generator(
                CoolingGeneratorKind::Compression {
                    heat_rejection: Some(HeatRejection::DryCooler),
                    declared: Some(DeclaredEfficiency {
                        value: 4.0,
                        source_reference: "statement".into(),
                    }),
                    performance: Some(CompressionPerformance::En14825(en14825(
                        [0.1, -0.2, 0.15, 0.95, 0.0],
                        false,
                    ))),
                },
                None,
            ),
        ]);
        let codes: Vec<_> = validate_cooling(&input, "cooling")
            .into_iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "cooling_room_unit_type_required",
            "cooling_heat_rejection_inconsistent",
            "source_reference_required",
            "cooling_declared_and_performance",
            "cooling_en14825_direct_condensation_only",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    #[test]
    fn cooling_limit_uses_the_10_19_need() {
        let input = system(vec![generator(compression(), None)]);
        let plain = assess_cooling(&input, context(&rated_zones()));
        let mut zones = rated_zones();
        // A need that only starts above 20 °C moves the limit up.
        zones[0].limit_need_kwh = Some(std::array::from_fn(|index| {
            (40.0 * (OUTDOOR_TEMPERATURE_C[index] - 15.5)).max(0.0)
        }));
        let shifted = assess_cooling(&input, context(&zones));
        assert_ne!(plain.cooling_limit_c, shifted.cooling_limit_c);
        assert_eq!(
            shifted.cooling_limit_c,
            cooling_limit_c(&zones[0].limit_need_kwh.unwrap())
        );
    }
}
