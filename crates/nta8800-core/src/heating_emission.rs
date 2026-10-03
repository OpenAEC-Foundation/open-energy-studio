//! Space-heating emission losses, NTA 8800 §9.3 (NEN-EN 15316-2).
//!
//! Implements 9.9–9.12 and 9.12a with the forfait temperature corrections of
//! tables 9.2 (system), 9.3 (hydronic balancing) and 9.4 (control). Checked
//! against NTA 8800:2025+C1:2026 pages 296–297: identical to the 2026
//! consultation draft except that the draft's 9.16 is numbered 9.12a.

use serde::{Deserialize, Serialize};

pub const DRAFT_SOURCE: &str =
    "NTA 8800:2025+C1:2026, §9.3, formules 9.9–9.12a en tabellen 9.2–9.4 (p. 294–297)";

/// 9.12a: upper bound of the relative emission loss.
pub const MAX_RELATIVE_LOSS: f64 = 0.15;

/// Table 9.2 rows, emission system in the main room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionSystem {
    RadiatorsOrConvectors,
    FloorHeating,
    FanAssistedRadiatorsOrConvectors,
    AirHeating,
    /// Local heaters (stove, local electric heater): table 9.2 "overige", Δθ_hydr = 0.
    LocalHeater,
    OtherOrUnknown,
}

/// Table 9.3 rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HydronicBalancing {
    NoneOrUnknown,
    Static,
    Dynamic,
    /// Local heaters and air heating via ducts: `Δθ_hydr = 0` (footnote a).
    NotApplicable,
}

/// Table 9.4 rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionControl {
    MainRoomThermostat,
    CentralWithRoomValves,
    IndividualRoomThermostats,
    OtherOrUnknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmissionInput {
    pub system: EmissionSystem,
    pub balancing: HydronicBalancing,
    pub control: EmissionControl,
    pub source_reference: String,
    /// Fans for air circulation in the room (9.21/9.22, table 9.11);
    /// required for fan-assisted radiators or convectors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fans: Option<EmissionFans>,
    /// 9.23 with tables 9.12/9.13: fans and controls of direct-fired or
    /// indirect air heaters in the zone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub air_heaters: Option<AirHeaterAuxiliary>,
}

/// Table 9.12 (direct-fired) or 9.13 (indirect) air heater type; unknown
/// properties take the highest value of the category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AirHeaterKind {
    /// Table 9.12: direct-fired air heater in the work space.
    Direct {
        #[serde(default, rename = "radialFan")]
        radial_fan: Option<bool>,
    },
    /// Table 9.13: indirect air heater on a central generator.
    Indirect {
        #[serde(default, rename = "roomHeightAbove8M")]
        room_height_above_8_m: Option<bool>,
        #[serde(default, rename = "warmAirReturn")]
        warm_air_return: Option<bool>,
        #[serde(default, rename = "ecMotor")]
        ec_motor: Option<bool>,
    },
}

impl AirHeaterKind {
    /// P_H,aux = factor · Q_h;b / n_H,aux (tables 9.12/9.13).
    pub fn factor(self) -> f64 {
        match self {
            Self::Direct { radial_fan } => match radial_fan {
                Some(false) => 0.014,
                _ => 0.022,
            },
            Self::Indirect {
                room_height_above_8_m,
                warm_air_return,
                ec_motor,
            } => {
                let cell = |high: bool, ret: bool, ec: bool| match (high, ret, ec) {
                    (false, true, false) => 0.008,
                    (false, true, true) => 0.004,
                    (false, false, false) => 0.009,
                    (false, false, true) => 0.005,
                    (true, true, false) => 0.012,
                    (true, true, true) => 0.006,
                    (true, false, false) => 0.013,
                    (true, false, true) => 0.007,
                };
                let options = |value: Option<bool>| match value {
                    Some(value) => vec![value],
                    None => vec![false, true],
                };
                let mut highest: f64 = 0.0;
                for high in options(room_height_above_8_m) {
                    for ret in options(warm_air_return) {
                        for ec in options(ec_motor) {
                            highest = highest.max(cell(high, ret, ec));
                        }
                    }
                }
                highest
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AirHeaterAuxiliary {
    pub kind: AirHeaterKind,
    /// Q_h;b per EN 12831-1, W; absent means the table 9.12 estimate.
    #[serde(default)]
    pub design_heat_load_w: Option<f64>,
    pub source_reference: String,
}

/// Table 9.12 estimate of Q_h;b: Q_H;ht of January over t_jan scaled from
/// 21 °C to −10 °C, plus (√A_g·4·3 + A_g)·5 W for heating up.
pub fn estimated_design_heat_load_w(january_heat_transfer_kwh: f64, area_m2: f64) -> f64 {
    let january_outdoor = crate::climate::OUTDOOR_TEMPERATURE_C[0];
    let hours = crate::climate::MONTH_HOURS[0];
    let transmission =
        january_heat_transfer_kwh / (0.001 * hours) * (21.0 - (-10.0)) / (21.0 - january_outdoor);
    let heating_up = (area_m2.max(0.0).sqrt() * 4.0 * 3.0 + area_m2) * 5.0;
    transmission + heating_up
}

/// Table 9.11 decisive property of the room fans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionFanKind {
    /// Fan convector (fancoil), also the indoor unit of a (multi-)split.
    FanConvector,
    ElectricHeating,
    /// Local dynamic heat storage (soapstone, PCM).
    DynamicStorage,
    /// Not known: the highest value of table 9.11.
    Unknown,
}

impl EmissionFanKind {
    /// Table 9.11, W per fan.
    pub fn power_w(self) -> f64 {
        match self {
            Self::FanConvector | Self::ElectricHeating => 10.0,
            Self::DynamicStorage | Self::Unknown => 12.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmissionFans {
    pub kind: EmissionFanKind,
    /// n_fan;zi of 9.22.
    pub count: u32,
    /// Fan power of an assembly tested to NEN-EN 16430, W per fan; replaces
    /// table 9.11.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tested_power_w: Option<f64>,
    pub source_reference: String,
}

impl EmissionFans {
    pub fn power_w(&self) -> f64 {
        self.tested_power_w.unwrap_or_else(|| self.kind.power_w())
    }
}

pub fn system_correction_k(system: EmissionSystem) -> f64 {
    match system {
        EmissionSystem::RadiatorsOrConvectors
        | EmissionSystem::OtherOrUnknown
        | EmissionSystem::LocalHeater => 0.35,
        EmissionSystem::FloorHeating => 0.3,
        EmissionSystem::FanAssistedRadiatorsOrConvectors => -0.15,
        EmissionSystem::AirHeating => 0.0,
    }
}

pub fn balancing_correction_k(balancing: HydronicBalancing) -> f64 {
    match balancing {
        HydronicBalancing::NoneOrUnknown => 0.7,
        HydronicBalancing::Static => 0.4,
        HydronicBalancing::Dynamic => 0.2,
        HydronicBalancing::NotApplicable => 0.0,
    }
}

pub fn control_correction_k(control: EmissionControl) -> f64 {
    match control {
        EmissionControl::MainRoomThermostat | EmissionControl::OtherOrUnknown => 2.5,
        EmissionControl::CentralWithRoomValves => 2.0,
        EmissionControl::IndividualRoomThermostats => 1.5,
    }
}

/// 9.12: `Δθ_int;inc` in K.
pub fn temperature_increment_k(input: &EmissionInput) -> f64 {
    system_correction_k(input.system)
        + balancing_correction_k(input.balancing)
        + control_correction_k(input.control)
}

/// Returns `true` when the balancing choice contradicts footnote a of
/// table 9.3 (air heating needs `NotApplicable`; wet systems must state it).
pub fn balancing_consistent(input: &EmissionInput) -> bool {
    // Table 9.3 footnote a: local heaters and air heating have Δθ_hydr = 0.
    let non_hydronic = matches!(
        input.system,
        EmissionSystem::AirHeating | EmissionSystem::LocalHeater
    );
    non_hydronic == (input.balancing == HydronicBalancing::NotApplicable)
}

/// 9.12a for one month: `Q_H;em;ls` in kWh for `Q_H;em;out = Q_H;nd`.
pub fn monthly_loss_kwh(
    heat_output_kwh: f64,
    setpoint_c: f64,
    outdoor_c: f64,
    increment_k: f64,
) -> f64 {
    // 9.12a condition: θ_H;int;ini − θ_e;comb > 0, with θ_e;comb = θ_e;avg;mi.
    if setpoint_c - outdoor_c <= 0.0 {
        return 0.0;
    }
    // 9.11
    let inclusive = setpoint_c + increment_k;
    let ratio = increment_k / (inclusive - outdoor_c);
    heat_output_kwh * ratio.min(MAX_RELATIVE_LOSS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        system: EmissionSystem,
        balancing: HydronicBalancing,
        control: EmissionControl,
    ) -> EmissionInput {
        EmissionInput {
            system,
            balancing,
            control,
            source_reference: "test".into(),
            fans: None,
            air_heaters: None,
        }
    }

    #[test]
    fn increments_follow_tables_9_2_to_9_4() {
        let radiators = input(
            EmissionSystem::RadiatorsOrConvectors,
            HydronicBalancing::NoneOrUnknown,
            EmissionControl::MainRoomThermostat,
        );
        assert!((temperature_increment_k(&radiators) - 3.55).abs() < 1e-12);
        let floor = input(
            EmissionSystem::FloorHeating,
            HydronicBalancing::Dynamic,
            EmissionControl::IndividualRoomThermostats,
        );
        assert!((temperature_increment_k(&floor) - 2.0).abs() < 1e-12);
        let air = input(
            EmissionSystem::AirHeating,
            HydronicBalancing::NotApplicable,
            EmissionControl::CentralWithRoomValves,
        );
        assert!((temperature_increment_k(&air) - 2.0).abs() < 1e-12);
        assert!(balancing_consistent(&air));
        assert!(balancing_consistent(&floor));
        let wrong = input(
            EmissionSystem::AirHeating,
            HydronicBalancing::Static,
            EmissionControl::OtherOrUnknown,
        );
        assert!(!balancing_consistent(&wrong));
        let local = input(
            EmissionSystem::LocalHeater,
            HydronicBalancing::NotApplicable,
            EmissionControl::OtherOrUnknown,
        );
        assert!(balancing_consistent(&local));
        assert!((temperature_increment_k(&local) - 2.85).abs() < 1e-12);
    }

    #[test]
    fn loss_uses_ratio_and_cap() {
        // January De Bilt, floor heating with dynamic balancing and room
        // thermostats: 2,0/(22,0 − 2,61) stays below the cap.
        let loss = monthly_loss_kwh(1000.0, 20.0, 2.61, 2.0);
        assert!((loss - 1000.0 * 2.0 / (22.0 - 2.61)).abs() < 1e-9);
        // Radiators without balancing: 3,55/(23,55 − 2,61) = 0,17 → capped.
        assert!((monthly_loss_kwh(1000.0, 20.0, 2.61, 3.55) - 150.0).abs() < 1e-9);
        // Mild month hits the 0,15 cap.
        assert!((monthly_loss_kwh(100.0, 20.0, 18.0, 3.55) - 15.0).abs() < 1e-12);
        // Outdoor at or above setpoint: no loss.
        assert_eq!(monthly_loss_kwh(100.0, 20.0, 20.0, 3.55), 0.0);
    }
}
