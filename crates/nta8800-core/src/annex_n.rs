//! Local heaters, air heaters, radiant heaters and stoves, NTA
//! 8800:2025+C1:2026 annex N (pp. 880–922).
//!
//! - N.3: on/off heaters, with the iteration on the load factor (N.36/N.37);
//! - N.4: high-low and modulating heaters: on/off at minimum load (N.60/N.61)
//!   or modulation (N.65–N.72);
//! - N.5: stoves and local heaters (envelope loss 0, chimney loss from the
//!   combustion efficiency, N.74–N.80);
//! - N.6: default values (tables N.20, N.22–N.30).
//!
//! Losses α are in % of the heat input at full load, as in the annex. The
//! performance values are multiplied by f_prac 0,95 (N.1.1/N.5); the kernel
//! applies that as `E_H;gen;in / 0,95`. The result is on the net
//! calorific value; the chain converts it with N.3.
//!
//! Interpretations (recorded in `INTERPRETATIONS`):
//! - N.69 prints only the chimney losses α_ch;ON;Pmin and α_ch;ON;Pn; the
//!   kernel interpolates the total losses α_ON;Pmin (N.50) and α_ON;Pn
//!   (N.49), because N.73 states that the envelope losses are part of the
//!   modulating calculation;
//! - in modulation (β_cmb;min > 1) the burner runs for the whole t_gen, so
//!   N.52 is evaluated with β_cmb;min = 1, as N.4.4.3.8 does at Pn;
//! - N.61 prints the denominator as "100 + (1 + Q_br/Q)"; the kernel uses
//!   100·(1 + Q_br/Q) as in N.37.

use serde::{Deserialize, Serialize};

pub const PRACTICE_FACTOR: f64 = 0.95;
/// Table N.7 c_p;air, Wh/(m³·K).
pub const AIR_HEAT_CAPACITY: f64 = 0.34;
/// Table N.30.
pub const DEFAULT_VENTILATION_M3_PER_H_KW: f64 = 10.0;
pub const DEFAULT_RADIANT_DIFFERENCE_K: f64 = 2.5;
pub const DEFAULT_GRADIENT_K_PER_M: f64 = 0.3;
/// Table N.24.
pub const PILOT_LOSS_PERCENT: f64 = 2.0;
const CONVERGENCE: f64 = 0.001;

pub const INTERPRETATIONS: &[&str] = &[
    "N.69 interpolates the total losses α_ON (N.49/N.50), not only the chimney losses (N.73)",
    "N.52 in modulation evaluated with β_cmb;min = 1 (burner on for the whole t_gen)",
    "N.61 denominator read as 100·(1 + Q_br/Q), as in N.37",
    "f_prac applied as E_H;gen;in / 0,95 (N.1.1/N.5)",
];

/// Table N.12 types (rows of tables N.20, N.22, N.26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalHeaterType {
    HighTemperatureRadiant,
    RadiantTubeWithoutFlue,
    RadiantTubeWithFlue,
    AirHeaterAtmospheric,
    AirHeaterFanBurner,
    AirHeaterModulatingCombustionAir,
    AirHeaterModulatingNoCombustionAir,
    AirHeaterModulatingEvaporative,
    CondensingAirHeater,
    /// N.5: stove or local heater (gas, oil or solid fuel).
    Stove,
}

impl LocalHeaterType {
    fn is_stove(self) -> bool {
        self == Self::Stove
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionPeriod {
    After2005,
    From1990To2005,
    Before1990,
}

/// Table N.15.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerControl {
    OnOff,
    HighLow,
    Modulating,
}

/// Table N.18.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VentilationNeed {
    /// Type A appliance, ventilation not interlocked with the burner.
    Required,
    /// Type A appliance, ventilation interlocked with the burner.
    Interlocked,
    /// Flue connection.
    None,
}

/// Table N.22 fan rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AirHeaterFan {
    Centrifugal,
    Axial,
}

/// Table N.23.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeInsulation {
    WellInsulatedNewHighEfficiency,
    WellInsulatedMaintained,
    OldAverage,
    OldPoor,
    None,
}

impl EnvelopeInsulation {
    fn coefficients(self) -> (f64, f64) {
        match self {
            Self::WellInsulatedNewHighEfficiency => (1.72, 0.44),
            Self::WellInsulatedMaintained => (3.45, 0.88),
            Self::OldAverage => (6.90, 1.76),
            Self::OldPoor => (8.36, 2.2),
            Self::None => (10.35, 2.64),
        }
    }
}

/// Table N.29 (k_lrh;env) and table N.28 (k_lrh;aux;rh).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeaterLocation {
    HeatedSpaceFree,
    HeatedSpaceAgainstWallOrRoof,
    BoilerRoom,
    UnderRoofOutsideHeatedSpace,
    Outdoors,
}

impl HeaterLocation {
    fn envelope_factor(self) -> f64 {
        match self {
            Self::HeatedSpaceFree => 0.0,
            Self::HeatedSpaceAgainstWallOrRoof => 0.1,
            Self::BoilerRoom => 0.7,
            Self::UnderRoofOutsideHeatedSpace => 0.8,
            Self::Outdoors => 1.0,
        }
    }

    fn recovered_aux(self) -> f64 {
        match self {
            Self::HeatedSpaceFree | Self::HeatedSpaceAgainstWallOrRoof => 1.0,
            _ => 0.8,
        }
    }
}

/// Table N.27 stove kinds (default combustion efficiency).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoveKind {
    /// NEN-EN 13240: 50 %.
    SolidFuelRoomHeater,
    /// NEN-EN 13229 inset appliances and open fires: 30 %.
    InsetOrOpenFire,
    /// NEN-EN 14785 pellet appliances: 75 %.
    Pellet,
    /// NEN-EN 15250 slow heat release: 70 %.
    Accumulating,
    /// Gas or oil stove: efficiency from the product.
    GasOrOil,
}

impl StoveKind {
    fn default_efficiency(self) -> Option<f64> {
        match self {
            Self::SolidFuelRoomHeater => Some(50.0),
            Self::InsetOrOpenFire => Some(30.0),
            Self::Pellet => Some(75.0),
            Self::Accumulating => Some(70.0),
            Self::GasOrOil => None,
        }
    }
}

/// N.5.3: heat exchanger to a water-based heating system.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WaterConnection {
    pub output_to_air_kw: f64,
    pub output_to_water_kw: f64,
}

/// Product data of tables N.4 and N.8; `None` takes the N.6 default where
/// one exists.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalHeaterProduct {
    /// Φ_lrh;cmb;Pn, kW; or output with efficiency (N.4/N.74).
    #[serde(default)]
    pub input_full_kw: Option<f64>,
    #[serde(default)]
    pub output_full_kw: Option<f64>,
    /// Combustion efficiency on net calorific value at full load, %.
    #[serde(default)]
    pub combustion_efficiency_percent: Option<f64>,
    /// α_lrh;ch;on, %.
    #[serde(default)]
    pub chimney_loss_percent: Option<f64>,
    #[serde(default)]
    pub chimney_correction_factor: Option<f64>,
    #[serde(default)]
    pub test_air_temperature_c: Option<f64>,
    #[serde(default)]
    pub load_exponent: Option<f64>,
    /// Φ_lrh;aux;br;Pn, Φ_lrh;aux;blw;Pn, Φ_lrh;aux;sby in kW.
    #[serde(default)]
    pub aux_burner_kw: Option<f64>,
    #[serde(default)]
    pub aux_after_burner_kw: Option<f64>,
    #[serde(default)]
    pub aux_standby_kw: Option<f64>,
    /// α_lrh;env, %.
    #[serde(default)]
    pub envelope_loss_percent: Option<f64>,
    #[serde(default)]
    pub pilot_loss_percent: Option<f64>,
    /// N.8 minimum-load data.
    #[serde(default)]
    pub input_min_kw: Option<f64>,
    #[serde(default)]
    pub chimney_loss_min_percent: Option<f64>,
    #[serde(default)]
    pub combustion_efficiency_min_percent: Option<f64>,
    #[serde(default)]
    pub aux_burner_min_kw: Option<f64>,
    #[serde(default)]
    pub aux_after_burner_min_kw: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalHeater {
    pub heater_type: LocalHeaterType,
    pub control: PowerControl,
    pub production_period: ProductionPeriod,
    pub condensing: bool,
    pub pilot_flame: bool,
    pub ventilation: VentilationNeed,
    pub location: HeaterLocation,
    /// Fan type for the table N.22 defaults of air heaters.
    #[serde(default)]
    pub fan: Option<AirHeaterFan>,
    /// Envelope class for α_lrh;env when the product gives none (N.10).
    #[serde(default)]
    pub envelope_insulation: Option<EnvelopeInsulation>,
    /// Stoves only.
    #[serde(default)]
    pub stove_kind: Option<StoveKind>,
    #[serde(default)]
    pub water_connection: Option<WaterConnection>,
    /// H_lrh of the installation room, m (ventilation losses, N.18).
    #[serde(default)]
    pub room_height_m: Option<f64>,
    pub product: LocalHeaterProduct,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnexNIssue {
    pub code: &'static str,
    pub path: String,
}

/// Resolved product values in kW and %.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedHeater {
    input_full: f64,
    input_min: f64,
    k_mod: f64,
    chimney: f64,
    chimney_min: f64,
    correction: f64,
    test_air: f64,
    exponent: f64,
    aux_burner: f64,
    aux_after: f64,
    aux_standby: f64,
    aux_burner_min: f64,
    aux_after_min: f64,
    envelope: f64,
    pilot: f64,
    condensation: f64,
    condensation_min: f64,
    water_share: f64,
}

/// Table N.20: (ϑ_test, f_corr, α_ch;on by period).
fn table_n20(heater: LocalHeaterType, period: ProductionPeriod) -> Option<(f64, f64, Option<f64>)> {
    use LocalHeaterType::*;
    let column = |values: [Option<f64>; 3]| match period {
        ProductionPeriod::After2005 => values[0],
        ProductionPeriod::From1990To2005 => values[1],
        ProductionPeriod::Before1990 => values[2],
    };
    Some(match heater {
        HighTemperatureRadiant | RadiantTubeWithoutFlue => (20.0, 0.0, Some(0.0)),
        RadiantTubeWithFlue => (20.0, 0.25, column([Some(10.0), Some(13.0), Some(16.0)])),
        AirHeaterAtmospheric => (20.0, 0.18, column([Some(13.0), Some(15.0), Some(18.0)])),
        AirHeaterFanBurner => (20.0, 0.18, column([Some(10.0), Some(13.0), Some(16.0)])),
        AirHeaterModulatingCombustionAir
        | AirHeaterModulatingNoCombustionAir
        | AirHeaterModulatingEvaporative => (20.0, 0.18, column([Some(8.0), Some(10.0), None])),
        CondensingAirHeater => (20.0, 0.18, column([Some(5.0), None, None])),
        Stove => return None,
    })
}

/// Table N.22: (n_ch;on, k_aux;blw %, k_aux;br %, k_aux;sby %).
fn table_n22(heater: &LocalHeater, input_full: f64) -> Option<(f64, f64, f64, f64)> {
    use LocalHeaterType::*;
    Some(match heater.heater_type {
        HighTemperatureRadiant => (0.0, 0.0, 0.18, 0.0),
        RadiantTubeWithoutFlue | RadiantTubeWithFlue => {
            if input_full <= 60.0 {
                (0.1, 0.0, 0.25, 0.0)
            } else {
                (0.15, 2.0, 0.3, 0.0)
            }
        }
        Stove => return None,
        _ => match heater.fan? {
            AirHeaterFan::Centrifugal => (0.1, 0.0, 1.7, 0.0),
            AirHeaterFan::Axial => (0.1, 0.0, 0.9, 0.0),
        },
    })
}

/// Table N.26: (β_Pmin, α_ch;on;Pmin by period).
fn table_n26(heater: LocalHeaterType, period: ProductionPeriod) -> Option<(f64, Option<f64>)> {
    use LocalHeaterType::*;
    let column = |values: [Option<f64>; 3]| match period {
        ProductionPeriod::After2005 => values[0],
        ProductionPeriod::From1990To2005 => values[1],
        ProductionPeriod::Before1990 => values[2],
    };
    Some(match heater {
        HighTemperatureRadiant => (0.5, Some(0.0)),
        RadiantTubeWithoutFlue => (0.7, Some(0.0)),
        RadiantTubeWithFlue => (0.7, column([Some(8.0), Some(10.0), Some(13.0)])),
        AirHeaterModulatingCombustionAir => (0.7, column([Some(6.0), Some(8.0), Some(10.0)])),
        AirHeaterModulatingNoCombustionAir => (0.7, column([Some(12.0), Some(14.0), None])),
        AirHeaterModulatingEvaporative => (0.3, column([Some(3.0), None, None])),
        _ => return None,
    })
}

/// Table N.25 condensing defaults: (η full, η min).
fn table_n25(control: PowerControl, heater: LocalHeaterType) -> (Option<f64>, Option<f64>) {
    match (control, heater) {
        (PowerControl::OnOff, _) => (Some(104.0), None),
        (_, LocalHeaterType::AirHeaterModulatingNoCombustionAir) => (Some(102.0), Some(90.0)),
        _ => (Some(94.0), Some(104.0)),
    }
}

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// Fills the N.6 defaults; `Err` names the values that must be supplied.
pub fn resolve_heater(
    heater: &LocalHeater,
    path: &str,
) -> Result<ResolvedHeater, Vec<AnnexNIssue>> {
    let mut issues = Vec::new();
    let mut missing = |field: &str| {
        issues.push(AnnexNIssue {
            code: "local_heater_value_required",
            path: format!("{path}.product.{field}"),
        })
    };
    let p = &heater.product;
    let stove = heater.heater_type.is_stove();
    if stove != heater.stove_kind.is_some() {
        missing("stoveKind");
    }
    let stove_efficiency = heater
        .stove_kind
        .and_then(StoveKind::default_efficiency)
        .filter(|_| stove);
    let efficiency = p.combustion_efficiency_percent.or(stove_efficiency);

    // N.4/N.74/N.77: heat input at full load.
    let water = heater.water_connection;
    let input_full = match (water, p.input_full_kw, p.output_full_kw, efficiency) {
        (Some(connection), _, _, Some(eta)) => {
            Some((connection.output_to_air_kw + connection.output_to_water_kw) / (eta / 100.0))
        }
        (None, Some(input), _, _) => Some(input),
        (None, None, Some(output), Some(eta)) => Some(output / (eta / 100.0)),
        _ => None,
    };
    let Some(input_full) = input_full.filter(|value| positive(*value)) else {
        missing("inputFullKw");
        return Err(issues);
    };

    // Chimney loss at full load (N.6, N.75, N.79).
    let defaults = table_n20(heater.heater_type, heater.production_period);
    let chimney = if let Some(connection) = water {
        // N.78/N.79: efficiency to air.
        Some((100.0 - connection.output_to_air_kw / input_full * 100.0).max(0.0))
    } else if let Some(loss) = p.chimney_loss_percent {
        Some(loss)
    } else if let Some(eta) = efficiency {
        Some((100.0 - eta).max(0.0))
    } else {
        defaults.and_then(|(_, _, loss)| loss)
    };
    let Some(chimney) = chimney else {
        missing("chimneyLossPercent");
        return Err(issues);
    };
    let (default_test, default_correction) = defaults.map_or((20.0, 0.0), |(t, f, _)| (t, f));
    let n22 = table_n22(heater, input_full);
    let exponent = p
        .load_exponent
        .or(n22.map(|values| values.0))
        .unwrap_or(0.0);
    let aux = |given: Option<f64>, percent: Option<f64>| {
        given.or(percent.map(|share| share / 100.0 * input_full))
    };
    let aux_burner = aux(p.aux_burner_kw, n22.map(|v| v.2));
    let aux_after = aux(p.aux_after_burner_kw, n22.map(|v| v.1));
    let aux_standby = aux(p.aux_standby_kw, n22.map(|v| v.3));
    let (aux_burner, aux_after, aux_standby) = if stove {
        // Table N.33: Ecodesign data, otherwise no auxiliary appliances.
        (
            aux_burner.unwrap_or(0.0),
            aux_after.unwrap_or(0.0),
            aux_standby.unwrap_or(0.0),
        )
    } else {
        match (aux_burner, aux_after, aux_standby) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => {
                missing("auxBurnerKw");
                return Err(issues);
            }
        }
    };

    // N.10, N.5.2.2.3, N.5.3.4.
    let envelope = if stove {
        0.0
    } else if let Some(loss) = p.envelope_loss_percent {
        loss
    } else if let Some(class) = heater.envelope_insulation {
        let (c1, c2) = class.coefficients();
        (c1 - c2 * input_full.log10()).max(0.0)
    } else {
        missing("envelopeLossPercent");
        return Err(issues);
    };
    let pilot = if heater.pilot_flame {
        p.pilot_loss_percent.unwrap_or(PILOT_LOSS_PERCENT)
    } else {
        0.0
    };

    // Condensation (N.26/N.53, table N.25); none for water-connected stoves.
    let (cond_full, cond_min) = if heater.condensing && water.is_none() {
        let (full, min) = table_n25(heater.control, heater.heater_type);
        (
            p.combustion_efficiency_percent.or(full),
            p.combustion_efficiency_min_percent.or(min),
        )
    } else {
        (None, None)
    };
    let condensation = cond_full.map_or(0.0, |eta| (eta - 100.0).max(0.0));
    let condensation_min = cond_min.map_or(0.0, |eta| (eta - 100.0).max(0.0));

    // Minimum load (N.41–N.44, table N.26).
    let n26 = table_n26(heater.heater_type, heater.production_period);
    let (input_min, chimney_min) = if heater.control == PowerControl::OnOff {
        (input_full, chimney)
    } else {
        let input_min = p.input_min_kw.or(n26.map(|(beta, _)| beta * input_full));
        let chimney_min = p
            .chimney_loss_min_percent
            .or(p
                .combustion_efficiency_min_percent
                .filter(|_| !heater.condensing)
                .map(|eta| (100.0 - eta).max(0.0)))
            .or(n26.and_then(|(_, loss)| loss));
        match (input_min, chimney_min) {
            (Some(min), Some(loss)) if positive(min) && min < input_full => (min, loss),
            _ => {
                missing("inputMinKw");
                return Err(issues);
            }
        }
    };
    Ok(ResolvedHeater {
        input_full,
        input_min,
        k_mod: input_min / input_full,
        chimney,
        chimney_min,
        correction: p.chimney_correction_factor.unwrap_or(default_correction),
        test_air: p.test_air_temperature_c.unwrap_or(default_test),
        exponent,
        aux_burner,
        aux_after,
        aux_standby,
        aux_burner_min: p.aux_burner_min_kw.unwrap_or(aux_burner),
        aux_after_min: p.aux_after_burner_min_kw.unwrap_or(aux_after),
        envelope,
        pilot,
        condensation,
        condensation_min,
        water_share: water.map_or(0.0, |c| c.output_to_water_kw / c.output_to_air_kw),
    })
}

pub fn validate_local_heater(heater: &LocalHeater, path: &str) -> Vec<AnnexNIssue> {
    let mut issues = match resolve_heater(heater, path) {
        Ok(_) => Vec::new(),
        Err(issues) => issues,
    };
    if heater.source_reference.trim().is_empty() {
        issues.push(AnnexNIssue {
            code: "source_reference_required",
            path: format!("{path}.sourceReference"),
        });
    }
    if heater.water_connection.is_some_and(|c| {
        !positive(c.output_to_air_kw)
            || !(c.output_to_water_kw.is_finite() && c.output_to_water_kw >= 0.0)
    }) {
        issues.push(AnnexNIssue {
            code: "water_connection_invalid",
            path: format!("{path}.waterConnection"),
        });
    }
    if heater.water_connection.is_some() && !heater.heater_type.is_stove() {
        issues.push(AnnexNIssue {
            code: "water_connection_stoves_only",
            path: format!("{path}.waterConnection"),
        });
    }
    if heater.ventilation != VentilationNeed::None && heater.room_height_m.is_none_or_invalid() {
        issues.push(AnnexNIssue {
            code: "local_heater_value_required",
            path: format!("{path}.roomHeightM"),
        });
    }
    issues
}

trait OptionalHeight {
    fn is_none_or_invalid(&self) -> bool;
}

impl OptionalHeight for Option<f64> {
    fn is_none_or_invalid(&self) -> bool {
        !self.is_some_and(|value| value.is_finite() && value >= 0.0)
    }
}

/// Monthly conditions from chapter 9.
#[derive(Debug, Clone, Copy)]
pub struct HeaterMonth {
    /// Q_H;gen;out;req, kWh.
    pub heat_required_kwh: f64,
    /// t_lrh;gen, h.
    pub operating_hours: f64,
    /// ϑ_int (setpoint of the room), °C.
    pub indoor_c: f64,
    pub outdoor_c: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaterResult {
    /// Q_H;gen;out, kWh (limited to the maximum, N.34/N.58).
    pub heat_output_kwh: f64,
    pub shortfall_kwh: f64,
    /// E_H;gen;in on net calorific value, divided by f_prac, kWh.
    pub input_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    /// N.80: heat to the water-based system, kWh (not credited).
    pub water_side_heat_kwh: f64,
    pub load_factor: f64,
    pub modulating: bool,
}

struct Losses {
    vent_on: f64,
    vent_off: f64,
    envelope: f64,
}

/// N.16–N.25.
fn losses(heater: &LocalHeater, r: &ResolvedHeater, month: &HeaterMonth) -> Losses {
    let vent = match heater.ventilation {
        VentilationNeed::None => 0.0,
        _ => {
            let height = heater.room_height_m.unwrap_or(0.0);
            let exhaust =
                month.indoor_c - DEFAULT_RADIANT_DIFFERENCE_K + height * DEFAULT_GRADIENT_K_PER_M;
            // N.16/N.17 in % of the nominal input.
            DEFAULT_VENTILATION_M3_PER_H_KW * AIR_HEAT_CAPACITY * (exhaust - month.outdoor_c)
                / 1000.0
                * 100.0
        }
    };
    let vent_off = match heater.ventilation {
        VentilationNeed::Required => vent,
        _ => 0.0,
    };
    Losses {
        vent_on: vent,
        vent_off,
        envelope: heater.location.envelope_factor() * r.envelope,
    }
}

/// N.15/N.52.
fn chimney_corrected(r: &ResolvedHeater, base: f64, air_c: f64, beta: f64) -> f64 {
    (base + (air_c - r.test_air) * r.correction) * beta.max(0.0).powf(r.exponent)
}

/// One month of annex N; call after [`validate_local_heater`].
pub fn heater_month(heater: &LocalHeater, r: &ResolvedHeater, month: HeaterMonth) -> HeaterResult {
    let t_gen = month.operating_hours;
    let k_rh = heater.location.recovered_aux();
    let l = losses(heater, r, &month);
    let air = month.indoor_c;
    if month.heat_required_kwh <= 0.0 || t_gen <= 0.0 {
        // Burner off for the whole t_gen: W_blw (N.29) and W_sby (N.31)
        // with t_ON = 0.
        let standby = (r.aux_standby + r.aux_after) * t_gen.max(0.0);
        return HeaterResult {
            heat_output_kwh: 0.0,
            shortfall_kwh: month.heat_required_kwh.max(0.0),
            input_kwh: 0.0,
            auxiliary_electricity_kwh: standby,
            ..HeaterResult::default()
        };
    }
    // N.33: maximum output at full load.
    let alpha_on_full =
        chimney_corrected(r, r.chimney, air, 1.0) + l.vent_on + l.envelope - r.condensation;
    let max = r.input_full * (1.0 - alpha_on_full / 100.0) * t_gen
        + (r.aux_burner + r.aux_after) * t_gen * k_rh;
    let q = month.heat_required_kwh.min(max);
    let shortfall = month.heat_required_kwh - q;

    let on_off = |input: f64,
                  chimney: f64,
                  vent_env_scale: f64,
                  off_scale: f64,
                  condensation: f64,
                  aux_br: f64,
                  aux_blw: f64| {
        // N.36/N.37 (N.60/N.61 at minimum load).
        let alpha_off = (r.pilot + l.vent_off) / off_scale;
        let mut beta = q / (input * t_gen);
        for _ in 0..200 {
            let alpha_on = chimney_corrected(r, chimney, air, beta)
                + (l.vent_on + l.envelope) / vent_env_scale
                - condensation;
            let t_on = beta * t_gen;
            let q_br = aux_br * t_on * k_rh;
            let q_blw = aux_blw * t_gen * k_rh;
            let q_sby = r.aux_standby * (t_gen - t_on).max(0.0) * k_rh;
            let next = (100.0 * (q - q_blw - q_sby) / (input * t_gen) + alpha_off)
                / (100.0 * (1.0 + q_br / q) - alpha_on + alpha_off);
            let done = (next - beta).abs() < CONVERGENCE;
            beta = next;
            if done {
                break;
            }
        }
        beta
    };

    let water = |output: f64| output * r.water_share;
    if heater.control == PowerControl::OnOff {
        let beta = on_off(
            r.input_full,
            r.chimney,
            1.0,
            1.0,
            r.condensation,
            r.aux_burner,
            r.aux_after,
        )
        .clamp(0.0, 1.0);
        let t_on = beta * t_gen;
        // N.38/N.39.
        return HeaterResult {
            heat_output_kwh: q,
            shortfall_kwh: shortfall,
            input_kwh: t_on * r.input_full / PRACTICE_FACTOR,
            auxiliary_electricity_kwh: r.aux_burner * t_on
                + r.aux_after * t_gen
                + r.aux_standby * (t_gen - t_on),
            water_side_heat_kwh: water(q),
            load_factor: beta,
            modulating: false,
        };
    }
    // N.4.4.5.3: on/off at minimum load.
    let beta_min = on_off(
        r.input_min,
        r.chimney_min,
        r.k_mod,
        r.k_mod,
        r.condensation_min,
        r.aux_burner_min,
        r.aux_after_min,
    );
    if beta_min <= 1.0 {
        let beta_min = beta_min.max(0.0);
        let t_on = beta_min * t_gen;
        // N.62/N.63.
        return HeaterResult {
            heat_output_kwh: q,
            shortfall_kwh: shortfall,
            input_kwh: t_on * r.input_min / PRACTICE_FACTOR,
            auxiliary_electricity_kwh: r.aux_burner_min * t_on
                + r.aux_after_min * t_gen
                + r.aux_standby * (t_gen - t_on),
            water_side_heat_kwh: water(q),
            load_factor: beta_min,
            modulating: false,
        };
    }
    // N.4.4.5.4: modulation with t_ON = t_gen.
    let w_br = r.aux_burner * t_gen;
    let w_blw = r.aux_after * t_gen;
    let w_br_min = r.aux_burner_min * t_gen;
    let w_blw_min = r.aux_after_min * t_gen;
    let alpha_full =
        chimney_corrected(r, r.chimney, air, 1.0) + l.vent_on + l.envelope - r.condensation;
    // N.50/N.52 with β_cmb;min = 1 (see the module notes).
    let alpha_min = chimney_corrected(r, r.chimney_min, air, 1.0)
        + (l.vent_on + l.envelope) / r.k_mod
        - r.condensation_min;
    let mut average = q / t_gen;
    let mut beta_mod = 0.0;
    for _ in 0..200 {
        let next = ((average - r.input_min) / (r.input_full - r.input_min)).clamp(0.0, 1.0);
        let q_br = (w_br_min + (w_br - w_br_min) * next) * k_rh;
        let q_blw = (w_blw_min + (w_blw - w_blw_min) * next) * k_rh;
        let alpha = alpha_min + (alpha_full - alpha_min) * next;
        average = (q - q_blw - q_br) / (t_gen * (1.0 - alpha / 100.0));
        let done = (next - beta_mod).abs() < CONVERGENCE;
        beta_mod = next;
        if done {
            break;
        }
    }
    HeaterResult {
        heat_output_kwh: q,
        shortfall_kwh: shortfall,
        input_kwh: average * t_gen / PRACTICE_FACTOR,
        auxiliary_electricity_kwh: w_br_min
            + w_blw_min
            + (w_br + w_blw - w_br_min - w_blw_min) * beta_mod,
        water_side_heat_kwh: water(q),
        load_factor: beta_mod,
        modulating: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air_heater() -> LocalHeater {
        LocalHeater {
            heater_type: LocalHeaterType::AirHeaterFanBurner,
            control: PowerControl::OnOff,
            production_period: ProductionPeriod::After2005,
            condensing: false,
            pilot_flame: false,
            ventilation: VentilationNeed::None,
            location: HeaterLocation::HeatedSpaceFree,
            fan: Some(AirHeaterFan::Axial),
            envelope_insulation: Some(EnvelopeInsulation::WellInsulatedMaintained),
            stove_kind: None,
            water_connection: None,
            room_height_m: None,
            product: LocalHeaterProduct {
                input_full_kw: Some(40.0),
                ..LocalHeaterProduct::default()
            },
            source_reference: "type plate".into(),
        }
    }

    #[test]
    fn defaults_follow_tables_n20_n22_n23() {
        let heater = air_heater();
        let r = resolve_heater(&heater, "h").unwrap();
        assert_eq!(r.chimney, 10.0);
        assert_eq!(r.correction, 0.18);
        assert_eq!(r.exponent, 0.1);
        assert!((r.aux_burner - 0.009 * 40.0).abs() < 1e-12);
        // N.10 with log10(40 kW·1000/1000 W)... the argument is Φ in W/1000 W.
        assert!((r.envelope - (3.45 - 0.88 * 40f64.log10())).abs() < 1e-12);
    }

    #[test]
    fn on_off_iteration_balances_n37() {
        let heater = air_heater();
        let r = resolve_heater(&heater, "h").unwrap();
        let month = HeaterMonth {
            heat_required_kwh: 5000.0,
            operating_hours: 600.0,
            indoor_c: 18.0,
            outdoor_c: 2.61,
        };
        let result = heater_month(&heater, &r, month);
        let beta = result.load_factor;
        // Check the converged N.37 balance directly.
        let t_on = beta * 600.0;
        let alpha_on = (10.0 + (18.0 - 20.0) * 0.18) * beta.powf(0.1);
        let q_br = r.aux_burner * t_on;
        let q_sby = 0.0;
        let rhs = (100.0 * (5000.0 - q_sby) / (40.0 * 600.0))
            / (100.0 * (1.0 + q_br / 5000.0) - alpha_on);
        assert!((beta - rhs).abs() < 0.002, "{beta} vs {rhs}");
        assert!((result.input_kwh - t_on * 40.0 / 0.95).abs() < 1e-9);
        assert!((result.auxiliary_electricity_kwh - r.aux_burner * t_on).abs() < 1e-9);
    }

    #[test]
    fn modulating_heater_switches_to_modulation_above_minimum_load() {
        let mut heater = air_heater();
        heater.heater_type = LocalHeaterType::AirHeaterModulatingCombustionAir;
        heater.control = PowerControl::Modulating;
        let r = resolve_heater(&heater, "h").unwrap();
        // Table N.26: β_Pmin 0,7 and α 6 % after 2005.
        assert!((r.input_min - 28.0).abs() < 1e-12);
        assert_eq!(r.chimney_min, 6.0);
        let low = heater_month(
            &heater,
            &r,
            HeaterMonth {
                heat_required_kwh: 2000.0,
                operating_hours: 600.0,
                indoor_c: 18.0,
                outdoor_c: 5.0,
            },
        );
        assert!(!low.modulating);
        let high = heater_month(
            &heater,
            &r,
            HeaterMonth {
                heat_required_kwh: 20000.0,
                operating_hours: 600.0,
                indoor_c: 18.0,
                outdoor_c: 5.0,
            },
        );
        assert!(high.modulating);
        assert!(high.input_kwh > 20000.0 / 0.95);
    }

    #[test]
    fn zero_demand_month_keeps_after_burner_and_standby_auxiliary() {
        let heater = air_heater();
        let r = resolve_heater(&heater, "h").unwrap();
        let result = heater_month(
            &heater,
            &r,
            HeaterMonth {
                heat_required_kwh: 0.0,
                operating_hours: 100.0,
                indoor_c: 20.0,
                outdoor_c: 15.0,
            },
        );
        assert_eq!(result.input_kwh, 0.0);
        // N.29 and N.31 with t_ON = 0.
        let expected = (r.aux_after + r.aux_standby) * 100.0;
        assert!((result.auxiliary_electricity_kwh - expected).abs() < 1e-12);
    }

    #[test]
    fn pellet_stove_uses_table_n27_and_no_envelope_loss() {
        let heater = LocalHeater {
            heater_type: LocalHeaterType::Stove,
            control: PowerControl::OnOff,
            production_period: ProductionPeriod::After2005,
            condensing: false,
            pilot_flame: false,
            ventilation: VentilationNeed::None,
            location: HeaterLocation::HeatedSpaceFree,
            fan: None,
            envelope_insulation: None,
            stove_kind: Some(StoveKind::Pellet),
            water_connection: None,
            room_height_m: None,
            product: LocalHeaterProduct {
                output_full_kw: Some(7.5),
                ..LocalHeaterProduct::default()
            },
            source_reference: "Ecodesign sheet".into(),
        };
        let r = resolve_heater(&heater, "h").unwrap();
        assert!((r.input_full - 10.0).abs() < 1e-12);
        assert_eq!(r.chimney, 25.0);
        assert_eq!(r.envelope, 0.0);
        let result = heater_month(
            &heater,
            &r,
            HeaterMonth {
                heat_required_kwh: 1500.0,
                operating_hours: 700.0,
                indoor_c: 20.0,
                outdoor_c: 2.61,
            },
        );
        // No auxiliaries and n = 0: β = Q/(Φ·t·0,75), E = Q/0,75/0,95.
        assert!((result.input_kwh - 1500.0 / 0.75 / 0.95).abs() < 1e-6);
    }
}
