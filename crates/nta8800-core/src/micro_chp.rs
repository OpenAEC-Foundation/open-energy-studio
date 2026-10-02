//! Micro-CHP for space heating, NTA 8800:2025+C1:2026 §9.6.6.2 (method 1,
//! NEN-EN 15316-4-4 with NEN-EN 50465 test values, pages 343–354).
//!
//! Per month (time step `t = t_H;op;si;mi`, 9.32a; `t_sb = t_mi − t`):
//! - 9.66 `P_th;gen;out = min(P_th;chp_100+sup_100; Q_H;gen;out / t)`, with
//!   `Q_H;gen;out = Q_H;gen;j;out / f_gebouw;si;H` for a collective CHP;
//! - 9.67/9.68 electric output, 9.69/9.70 auxiliary power and 9.78/9.79
//!   losses, linearly interpolated between the stand-by point
//!   (`P_th;sb = 0`), CHP_100 %+Sup_0 % and CHP_100 %+Sup_100 %;
//! - 9.73–9.77 losses at the test points with `f_prac = 0,95`;
//! - 9.71/9.72 auxiliary energy (9.6.8 when no measured values exist);
//! - 9.80/9.81 recoverable loss `P_ls;sb · t` in a heated space, 0 otherwise
//!   and for a collective CHP;
//! - 9.82/9.83 input `E_gen;in = P_gen;in · t + P_gen;ls;sb · t_sb`;
//! - 16.15 electricity `P_el;chp;out;H · t_H;op`.
//!
//! Defaults of tables 9.37 (efficiencies) and 9.38 (stand-by loss 0,4 kW,
//! no pilot) apply per CGN_TYPE when no measured value is given; the ORC
//! type has no defaults.

use serde::{Deserialize, Serialize};

/// 9.74–9.77 practice factor.
pub const PRACTICE_FACTOR: f64 = 0.95;

pub const INTERPRETATIONS: &[&str] = &[
    "9.6.6.2: P_th;sb of figures 9.1–9.3 is taken as 0 kW (no heat output in the stand-by state)",
    "9.6.6.2: NEN-EN 50465 efficiencies refer to the net calorific value (table 9.37 totals above 1); the input is converted to the gross value with f_Hs/Hi of table M.3 (gas 1,11, oil 1,06)",
    "9.66: heat above P_th;chp_100+sup_100·t is booked at the CHP_100 %+Sup_100 % efficiency (η_th, η_el) instead of being rejected",
    "16.15: the electricity of a collective CHP is scaled with f_gebouw;si;H like the input energy",
];

/// Table 9.36 CGN_TYPE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MicroChpType {
    /// SE
    StirlingEngine,
    /// FCP (PEM fuel cell)
    PemFuelCell,
    /// FCS (solid oxide fuel cell)
    SolidOxideFuelCell,
    /// CEG (gas combustion engine)
    GasEngine,
    /// CED (diesel combustion engine)
    DieselEngine,
    /// MT
    MicroTurbine,
    /// ORC: no defaults in tables 9.37/9.38.
    OrganicRankineCycle,
}

/// Table 9.37 defaults: (η_th, η_el) at CHP_100+Sup_100 and at
/// CHP_100+Sup_0.
fn default_efficiencies(kind: MicroChpType) -> Option<((f64, f64), (f64, f64))> {
    use MicroChpType::*;
    Some(match kind {
        GasEngine => ((0.45, 0.21), (0.60, 0.30)),
        DieselEngine => ((0.50, 0.30), (0.60, 0.35)),
        MicroTurbine => ((0.52, 0.13), (0.65, 0.30)),
        StirlingEngine => ((0.92, 0.04), (0.78, 0.14)),
        PemFuelCell => ((0.98, 0.04), (0.53, 0.37)),
        SolidOxideFuelCell => ((0.98, 0.07), (0.55, 0.40)),
        OrganicRankineCycle => return None,
    })
}

/// Table 9.38: P_ls;sb 0,4 kW and P_pilot 0 for every listed type.
const DEFAULT_STANDBY_LOSS_KW: f64 = 0.4;

/// Table 9.36 CGN_FUEL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MicroChpFuel {
    NaturalGas,
    Oil,
}

impl MicroChpFuel {
    /// Table M.3 f_Hs/Hi.
    pub fn gross_to_net(self) -> f64 {
        match self {
            Self::NaturalGas => 1.11,
            Self::Oil => 1.06,
        }
    }
}

/// Table 9.39 CGN_LOC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MicroChpLocation {
    /// INT
    HeatedSpace,
    /// UNH
    UnheatedSpace,
    /// CGN
    InstallationRoom,
    /// EXT
    Outdoors,
}

/// Table 9.39 CGN_HCON (recorded; it does not enter the formulas).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MicroChpHydraulics {
    Direct,
    Decoupled,
    CondensationPump,
    HeatExchanger,
}

/// One NEN-EN 50465 test point of table 9.33.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChpTestPoint {
    /// P_th, kW.
    pub thermal_power_kw: f64,
    /// P_el;out, kW; derived from the efficiencies when omitted.
    #[serde(default)]
    pub electric_power_kw: Option<f64>,
    /// η_th; table 9.37 when omitted.
    #[serde(default)]
    pub thermal_efficiency: Option<f64>,
    /// η_el; table 9.37 when omitted.
    #[serde(default)]
    pub electric_efficiency: Option<f64>,
    /// P_aux, kW.
    #[serde(default)]
    pub auxiliary_power_kw: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MicroChp {
    pub kind: MicroChpType,
    pub fuel: MicroChpFuel,
    pub location: MicroChpLocation,
    #[serde(default)]
    pub hydraulics: Option<MicroChpHydraulics>,
    /// CHP_100 %+Sup_100 %.
    pub full_load: ChpTestPoint,
    /// CHP_100 %+Sup_0 %.
    pub chp_only: ChpTestPoint,
    /// P_ls;sb, kW; table 9.38 when omitted.
    #[serde(default)]
    pub standby_loss_kw: Option<f64>,
    /// P_pilot, kW; table 9.38 (0) when omitted.
    #[serde(default)]
    pub pilot_kw: Option<f64>,
    /// P_el;out;sb, kW; 0 when omitted (note to 9.67).
    #[serde(default)]
    pub standby_electric_kw: Option<f64>,
    /// P_aux;sb, kW.
    #[serde(default)]
    pub standby_auxiliary_kw: Option<f64>,
    /// Only the net power production was measured: 9.72 with P_aux;sb.
    #[serde(default)]
    pub net_production_measured: bool,
    pub test_report_reference: String,
}

/// Resolved test point.
#[derive(Debug, Clone, Copy)]
struct Point {
    thermal_kw: f64,
    electric_kw: f64,
    thermal_eff: f64,
    electric_eff: f64,
}

impl Point {
    /// 9.74–9.77: P_gen;ls = (1 − f_prac·η_th − f_prac·η_el)·P_th/(f_prac·η_th).
    fn loss_kw(self) -> f64 {
        let input = self.thermal_kw / (PRACTICE_FACTOR * self.thermal_eff);
        (1.0 - PRACTICE_FACTOR * self.thermal_eff - PRACTICE_FACTOR * self.electric_eff) * input
    }
}

fn resolve(point: &ChpTestPoint, defaults: Option<(f64, f64)>) -> Option<Point> {
    let thermal_eff = point.thermal_efficiency.or(defaults.map(|d| d.0))?;
    let electric_eff = match (point.electric_efficiency, point.electric_power_kw) {
        (Some(value), _) => value,
        (None, Some(power)) => power * thermal_eff / point.thermal_power_kw,
        (None, None) => defaults?.1,
    };
    let electric_kw = point
        .electric_power_kw
        .unwrap_or(point.thermal_power_kw * electric_eff / thermal_eff);
    Some(Point {
        thermal_kw: point.thermal_power_kw,
        electric_kw,
        thermal_eff,
        electric_eff,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct MicroChpIssue {
    pub code: &'static str,
    pub path: String,
}

fn in_range(value: f64, low: f64, high: f64) -> bool {
    value.is_finite() && value >= low && value <= high
}

/// Table 9.33 validity intervals and the presence of defaults.
pub fn validate_micro_chp(chp: &MicroChp, path: &str) -> Vec<MicroChpIssue> {
    let mut issues = Vec::new();
    let mut push = |code: &'static str, field: &str| {
        issues.push(MicroChpIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    let defaults = default_efficiencies(chp.kind);
    for (name, point, default) in [
        ("fullLoad", &chp.full_load, defaults.map(|d| d.0)),
        ("chpOnly", &chp.chp_only, defaults.map(|d| d.1)),
    ] {
        if !in_range(point.thermal_power_kw, 0.0, 70.0) || point.thermal_power_kw <= 0.0 {
            push(
                "micro_chp_thermal_power_invalid",
                &format!("{name}.thermalPowerKw"),
            );
        }
        if point
            .electric_power_kw
            .is_some_and(|v| !in_range(v, 0.0, 50.0))
        {
            push(
                "micro_chp_electric_power_invalid",
                &format!("{name}.electricPowerKw"),
            );
        }
        if point
            .thermal_efficiency
            .is_some_and(|v| !in_range(v, 0.0, 1.0) || v <= 0.0)
        {
            push(
                "micro_chp_efficiency_invalid",
                &format!("{name}.thermalEfficiency"),
            );
        }
        if point
            .electric_efficiency
            .is_some_and(|v| !in_range(v, 0.0, 0.5))
        {
            push(
                "micro_chp_efficiency_invalid",
                &format!("{name}.electricEfficiency"),
            );
        }
        if point
            .auxiliary_power_kw
            .is_some_and(|v| !in_range(v, 0.0, 20.0))
        {
            push(
                "micro_chp_auxiliary_power_invalid",
                &format!("{name}.auxiliaryPowerKw"),
            );
        }
        if resolve(point, default).is_none() {
            // ORC (no table 9.37 column) needs measured efficiencies.
            push("micro_chp_efficiency_required", name);
        }
    }
    if chp.full_load.thermal_power_kw < chp.chp_only.thermal_power_kw {
        push(
            "micro_chp_test_points_inconsistent",
            "fullLoad.thermalPowerKw",
        );
    }
    for (field, value) in [
        ("standbyLossKw", chp.standby_loss_kw),
        ("pilotKw", chp.pilot_kw),
        ("standbyElectricKw", chp.standby_electric_kw),
        ("standbyAuxiliaryKw", chp.standby_auxiliary_kw),
    ] {
        if value.is_some_and(|v| !in_range(v, 0.0, 20.0)) {
            push("micro_chp_standby_value_invalid", field);
        }
    }
    if chp.kind == MicroChpType::OrganicRankineCycle && chp.standby_loss_kw.is_none() {
        push("micro_chp_standby_loss_required", "standbyLossKw");
    }
    if chp.net_production_measured && chp.standby_auxiliary_kw.is_none() {
        push("micro_chp_standby_auxiliary_required", "standbyAuxiliaryKw");
    }
    if chp.test_report_reference.trim().is_empty() {
        push("source_reference_required", "testReportReference");
    }
    issues
}

/// Monthly result for the assessed building part.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicroChpMonth {
    /// P_th;gen;out of the whole installation, kW.
    pub thermal_power_kw: f64,
    /// P_el;gen;out, kW.
    pub electric_power_kw: f64,
    /// E_gen;in on the gross calorific value, kWh.
    pub input_kwh: f64,
    /// 16.15, kWh.
    pub electricity_kwh: f64,
    /// 9.71/9.72; `None` when 9.6.8 applies.
    pub auxiliary_kwh: Option<f64>,
    /// 9.80/9.81, kWh.
    pub recoverable_kwh: f64,
}

/// Interpolation between stand-by (P_th = 0), CHP_100+Sup_0 and
/// CHP_100+Sup_100 (figures 9.1–9.3).
fn interpolate(p: f64, p0: f64, p100: f64, at_sb: f64, at0: f64, at100: f64) -> f64 {
    if p <= p0 {
        if p0 > 0.0 {
            at_sb + (at0 - at_sb) * p / p0
        } else {
            at0
        }
    } else if p100 > p0 {
        at0 + (at100 - at0) * (p - p0) / (p100 - p0)
    } else {
        at100
    }
}

/// One month. `output_kwh` is Q_H;gen;j;out of the building part,
/// `operating_hours` t_H;op;si;mi (9.32a), `building_fraction` f_gebouw;si;H
/// and `collective` whether the CHP serves the building as a whole.
/// `None` for an invalid (unvalidated) product.
pub fn micro_chp_month(
    chp: &MicroChp,
    output_kwh: f64,
    operating_hours: f64,
    month_hours: f64,
    building_fraction: f64,
    collective: bool,
) -> Option<MicroChpMonth> {
    let defaults = default_efficiencies(chp.kind);
    let full = resolve(&chp.full_load, defaults.map(|d| d.0))?;
    let only = resolve(&chp.chp_only, defaults.map(|d| d.1))?;
    let fraction = if building_fraction > 0.0 {
        building_fraction
    } else {
        1.0
    };
    // 9.66 with the whole installation's output.
    let whole = output_kwh.max(0.0) / fraction;
    let t = if whole > 0.0 && operating_hours <= 0.0 {
        month_hours
    } else {
        operating_hours.clamp(0.0, month_hours)
    };
    let t_sb = month_hours - t;
    let required = if t > 0.0 { whole / t } else { 0.0 };
    let p_th = required.min(full.thermal_kw);
    let excess_kwh = (required - p_th).max(0.0) * t;
    // 9.73.
    let standby_loss = chp.standby_loss_kw.unwrap_or(DEFAULT_STANDBY_LOSS_KW);
    let loss_sb = standby_loss + chp.pilot_kw.unwrap_or(0.0);
    let el_sb = chp.standby_electric_kw.unwrap_or(0.0);
    let (p0, p100) = (only.thermal_kw, full.thermal_kw);
    // 9.67/9.68.
    let p_el = interpolate(p_th, p0, p100, el_sb, only.electric_kw, full.electric_kw);
    // 9.78/9.79.
    let p_ls = interpolate(p_th, p0, p100, loss_sb, only.loss_kw(), full.loss_kw());
    // 9.82/9.83, net calorific value; the excess above full load at the
    // CHP_100+Sup_100 efficiency (interpretation).
    let mut input_net = (p_th + p_el + p_ls) * t + loss_sb * t_sb;
    let mut electricity = p_el * t;
    if excess_kwh > 0.0 {
        input_net += excess_kwh / (PRACTICE_FACTOR * full.thermal_eff);
        electricity += excess_kwh * full.electric_eff / full.thermal_eff;
    }
    // 9.69–9.72.
    let auxiliary = if chp.net_production_measured {
        chp.standby_auxiliary_kw.map(|aux| aux * t_sb * fraction)
    } else {
        match (
            chp.standby_auxiliary_kw,
            chp.chp_only.auxiliary_power_kw,
            chp.full_load.auxiliary_power_kw,
        ) {
            (Some(sb), Some(a0), Some(a100)) => {
                Some(interpolate(p_th, p0, p100, sb, a0, a100) * t * fraction)
            }
            _ => None,
        }
    };
    // 9.80/9.81.
    let recoverable = if !collective && chp.location == MicroChpLocation::HeatedSpace {
        standby_loss * t
    } else {
        0.0
    };
    Some(MicroChpMonth {
        thermal_power_kw: p_th,
        electric_power_kw: p_el,
        input_kwh: input_net * chp.fuel.gross_to_net() * fraction,
        electricity_kwh: electricity * fraction,
        auxiliary_kwh: auxiliary,
        recoverable_kwh: recoverable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stirling() -> MicroChp {
        MicroChp {
            kind: MicroChpType::StirlingEngine,
            fuel: MicroChpFuel::NaturalGas,
            location: MicroChpLocation::HeatedSpace,
            hydraulics: None,
            full_load: ChpTestPoint {
                thermal_power_kw: 20.0,
                electric_power_kw: None,
                thermal_efficiency: None,
                electric_efficiency: None,
                auxiliary_power_kw: Some(0.10),
            },
            chp_only: ChpTestPoint {
                thermal_power_kw: 6.0,
                electric_power_kw: Some(1.0),
                thermal_efficiency: Some(0.80),
                electric_efficiency: None,
                auxiliary_power_kw: Some(0.06),
            },
            standby_loss_kw: None,
            pilot_kw: None,
            standby_electric_kw: None,
            standby_auxiliary_kw: Some(0.01),
            net_production_measured: false,
            test_report_reference: "EN 50465 report".into(),
        }
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} vs {b}");
    }

    #[test]
    fn part_load_below_chp_only_point() {
        let chp = stirling();
        assert!(validate_micro_chp(&chp, "chp").is_empty());
        // 1 500 kWh in 500 h of 744: P_th = 3 kW, half of the 6 kW point.
        let month = micro_chp_month(&chp, 1500.0, 500.0, 744.0, 1.0, false).unwrap();
        close(month.thermal_power_kw, 3.0);
        // 9.67 with P_el;sb = 0: 0,5 · 1,0 kW.
        close(month.electric_power_kw, 0.5);
        // η_el at Sup_0 from 1,0 kW: 1,0·0,80/6 = 0,13333.
        let eta_el0: f64 = 1.0 * 0.80 / 6.0;
        let input0 = 6.0 / (0.95 * 0.80);
        let loss0 = (1.0 - 0.95 * 0.80 - 0.95 * eta_el0) * input0;
        // 9.78 between P_ls;sb (0,4 table 9.38) and loss0.
        let p_ls = 0.4 + (loss0 - 0.4) * 0.5;
        let input = ((3.0 + 0.5 + p_ls) * 500.0 + 0.4 * 244.0) * 1.11;
        close(month.input_kwh, input);
        close(month.electricity_kwh, 0.5 * 500.0);
        // 9.69/9.71: 0,01 + (0,06 − 0,01)·0,5 kW over 500 h.
        close(month.auxiliary_kwh.unwrap(), 0.035 * 500.0);
        // 9.80: P_ls;sb · t in a heated space.
        close(month.recoverable_kwh, 0.4 * 500.0);
    }

    #[test]
    fn upper_range_and_defaults_of_table_9_37() {
        let chp = stirling();
        // P_th = 13 kW, halfway between 6 and 20 kW.
        let month = micro_chp_month(&chp, 13.0 * 400.0, 400.0, 720.0, 1.0, false).unwrap();
        // Sup_100 defaults SE: η_th 0,92, η_el 0,04 → P_el = 20·0,04/0,92.
        let el100 = 20.0 * 0.04 / 0.92;
        close(month.electric_power_kw, 1.0 + (el100 - 1.0) * 0.5);
        // 9.70: 0,06 + (0,10 − 0,06)·0,5.
        close(month.auxiliary_kwh.unwrap(), 0.08 * 400.0);
    }

    #[test]
    fn collective_net_production_and_excess() {
        let mut chp = stirling();
        chp.net_production_measured = true;
        chp.location = MicroChpLocation::InstallationRoom;
        // Building part 25 %: whole output 4 · 100 kWh over 100 h → 4 kW.
        let month = micro_chp_month(&chp, 100.0, 100.0, 744.0, 0.25, true).unwrap();
        close(month.thermal_power_kw, 4.0);
        // 9.72: P_aux;sb · t_sb · f_gebouw.
        close(month.auxiliary_kwh.unwrap(), 0.01 * 644.0 * 0.25);
        assert_eq!(month.recoverable_kwh, 0.0);
        // Above the 20 kW full-load point the excess uses Sup_100 values.
        let high = micro_chp_month(&stirling(), 30.0 * 100.0, 100.0, 744.0, 1.0, false).unwrap();
        close(high.thermal_power_kw, 20.0);
        let el100 = 20.0 * 0.04 / 0.92;
        close(high.electricity_kwh, el100 * 100.0 + 1000.0 * 0.04 / 0.92);
    }

    #[test]
    fn orc_needs_measured_efficiencies() {
        let mut chp = stirling();
        chp.kind = MicroChpType::OrganicRankineCycle;
        let codes: Vec<_> = validate_micro_chp(&chp, "chp")
            .into_iter()
            .map(|issue| issue.code)
            .collect();
        assert!(codes.contains(&"micro_chp_efficiency_required"));
        assert!(codes.contains(&"micro_chp_standby_loss_required"));
    }
}
