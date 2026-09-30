//! Space-heating emission losses, NTA 8800 §9.3 (NEN-EN 15316-2).
//!
//! Implements 9.9–9.12 and 9.16 with the forfait temperature corrections of
//! tables 9.2 (system), 9.3 (hydronic balancing) and 9.4 (control), as given
//! in the public 2026 consultation draft of chapter 9 (pages 13–15). The
//! draft is not the verified target edition; results stay unverified.

use serde::{Deserialize, Serialize};

pub const DRAFT_SOURCE: &str = "https://www.internetconsultatie.nl/epg2026/document/14150";

/// 9.16: upper bound of the relative emission loss.
pub const MAX_RELATIVE_LOSS: f64 = 0.15;

/// Table 9.2 rows, emission system in the main room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionSystem {
    RadiatorsOrConvectors,
    FloorHeating,
    FanAssistedRadiatorsOrConvectors,
    AirHeating,
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
}

pub fn system_correction_k(system: EmissionSystem) -> f64 {
    match system {
        EmissionSystem::RadiatorsOrConvectors | EmissionSystem::OtherOrUnknown => 0.35,
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
    let air = input.system == EmissionSystem::AirHeating;
    air == (input.balancing == HydronicBalancing::NotApplicable)
}

/// 9.16 for one month: `Q_H;em;ls` in kWh for `Q_H;em;out = Q_H;nd`.
pub fn monthly_loss_kwh(
    heat_output_kwh: f64,
    setpoint_c: f64,
    outdoor_c: f64,
    increment_k: f64,
) -> f64 {
    // 9.16 condition: θ_H;int;ini − θ_e;comb > 0, with θ_e;comb = θ_e;avg;mi.
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
