//! Humidification and dehumidification, NTA 8800:2025+C1:2026 chapter 12
//! (pp. 520–524).
//!
//! Humidification (12.1–12.4, table 12.1): the latent heat follows from the
//! mechanical supply flow of the heating balance (11.3.1.1). Atomising
//! humidifiers take that heat from the space-heating system (chapter 9);
//! steam humidifiers have their own generator with η 0,8 (electric) or 0,6
//! (gas or oil), and part of their loss is recoverable in the zone.
//!
//! Dehumidification (12.5, table 12.2): a surcharge on the sensible cooling
//! need, by the design temperature of the cooling emission system. Every
//! central air-handling cooling, room air conditioner or fan-coil system
//! dehumidifies; floor, wall and ceiling cooling (17/21 °C) do not.

use serde::{Deserialize, Serialize};

/// h_we at 20 °C, kJ/kg.
pub const LATENT_HEAT_KJ_PER_KG: f64 = 2538.2;
/// ρ_a, kg/m³.
pub const AIR_DENSITY: f64 = 1.205;
/// η_HU;rvd for a rotary heat wheel; 0 for other heat recovery.
pub const WHEEL_LATENT_RECOVERY: f64 = 0.55;

/// Table 12.1 (Δx·t_mi), kg·h/kg.
const MOISTURE_GENERAL: [f64; 12] = [
    0.82, 0.47, 0.64, 0.04, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.59,
];
const MOISTURE_HEALTHCARE: [f64; 12] = [
    1.69, 1.26, 1.51, 0.88, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.37, 1.46,
];

/// Table 12.2 f_DHU;C by design temperature of the cooling emission.
const DEHUMIDIFICATION_6_12: [f64; 12] = [
    0.0, 0.0, 0.0, 0.0, 0.20, 1.15, 1.84, 2.15, 1.91, 0.0, 0.0, 0.0,
];
const DEHUMIDIFICATION_12_16: [f64; 12] =
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.09, 0.43, 0.0, 0.0, 0.0, 0.0];
const DEHUMIDIFICATION_12_18: [f64; 12] =
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.02, 0.0, 0.0, 0.0, 0.0];

/// Columns of table 12.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HumidityFunction {
    /// Assembly, cell, lodging, office, education and retail.
    General,
    Healthcare,
    Residential,
    /// Note 1: no humidification.
    Sport,
}

impl HumidityFunction {
    fn moisture(self, month_index: usize) -> f64 {
        match self {
            Self::General => MOISTURE_GENERAL[month_index],
            Self::Healthcare => MOISTURE_HEALTHCARE[month_index],
            Self::Residential | Self::Sport => 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumidityFunctionArea {
    pub function: HumidityFunction,
    pub area_m2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SteamCarrier {
    Electricity,
    GasOrOil,
}

impl SteamCarrier {
    /// η_H;hum;si (12.3).
    fn efficiency(self) -> f64 {
        match self {
            Self::Electricity => 0.8,
            Self::GasOrOil => 0.6,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Humidifier {
    /// Water added to the air; the heating system supplies the latent heat.
    Atomising,
    /// Steam from a separate generator.
    Steam { carrier: SteamCarrier },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Humidification {
    pub humidifier: Humidifier,
    /// Rotary heat wheel (η_HU;rvd 0,55).
    pub rotary_wheel: bool,
    pub equipment_reference: String,
}

/// Columns of table 12.2; direct expansion and unknown designs count as
/// 6/12 °C.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingDesignTemperature {
    From6To12,
    From12To16,
    From12To18,
    /// Floor, wall or ceiling cooling: no dehumidification.
    From17To21,
    DirectExpansion,
    Unknown,
}

impl CoolingDesignTemperature {
    /// Table 12.2 f_DHU;C for month 0–11.
    pub fn fraction(self, month_index: usize) -> f64 {
        match self {
            Self::From6To12 | Self::DirectExpansion | Self::Unknown => {
                DEHUMIDIFICATION_6_12[month_index]
            }
            Self::From12To16 => DEHUMIDIFICATION_12_16[month_index],
            Self::From12To18 => DEHUMIDIFICATION_12_18[month_index],
            Self::From17To21 => 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HumidityInput {
    pub zone_id: String,
    /// A_g of the zone (12.2.1: no recoverable loss above 500 m²).
    pub usable_floor_area_m2: f64,
    pub functions: Vec<HumidityFunctionArea>,
    #[serde(default)]
    pub humidification: Option<Humidification>,
    /// Monthly q_V;SUP;dis;out of the heating balance, m³/h (11.3.1.1).
    #[serde(default)]
    pub supply_flow_m3_per_h: Vec<f64>,
    /// Emission design of a dehumidifying cooling system; `None` without
    /// cooling.
    #[serde(default)]
    pub cooling_design: Option<CoolingDesignTemperature>,
    /// Monthly sensible cooling need Q_C;nd, kWh.
    #[serde(default)]
    pub cooling_need_kwh: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HumidityIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HumidityMonth {
    pub month: u8,
    /// Q_H;hum (12.1), kWh.
    pub humidification_need_kwh: f64,
    /// Part of Q_H;hum delivered by the space-heating system, kWh.
    pub heating_system_load_kwh: f64,
    /// E_hum for steam generators (12.3), kWh.
    pub steam_electricity_kwh: f64,
    pub steam_fuel_kwh: f64,
    /// Q_H;hum;rbl (12.4), kWh.
    pub recoverable_kwh: f64,
    /// Q_C;dhum (12.5), kWh, delivered by the cooling system.
    pub dehumidification_need_kwh: f64,
}

pub fn validate_humidity(input: &HumidityInput) -> Vec<HumidityIssue> {
    let mut issues = Vec::new();
    let mut push = |code, path: &str| {
        issues.push(HumidityIssue {
            code,
            path: path.to_string(),
        })
    };
    if !(input.usable_floor_area_m2.is_finite() && input.usable_floor_area_m2 > 0.0) {
        push("usable_floor_area_invalid", "usableFloorAreaM2");
    }
    if input.functions.is_empty()
        || input
            .functions
            .iter()
            .any(|f| !(f.area_m2.is_finite() && f.area_m2 > 0.0))
    {
        push("function_area_invalid", "functions");
    }
    if let Some(humidification) = &input.humidification {
        if humidification.equipment_reference.trim().is_empty() {
            push(
                "source_reference_required",
                "humidification.equipmentReference",
            );
        }
        if input.supply_flow_m3_per_h.len() != 12
            || input
                .supply_flow_m3_per_h
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0)
        {
            push("supply_flow_twelve_months_required", "supplyFlowM3PerH");
        }
    }
    if input.cooling_design.is_some()
        && (input.cooling_need_kwh.len() != 12
            || input
                .cooling_need_kwh
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0))
    {
        push("cooling_need_twelve_months_required", "coolingNeedKwh");
    }
    issues
}

pub fn calculate_humidity(input: &HumidityInput) -> Result<Vec<HumidityMonth>, Vec<HumidityIssue>> {
    let issues = validate_humidity(input);
    if !issues.is_empty() {
        return Err(issues);
    }
    let total_area: f64 = input.functions.iter().map(|f| f.area_m2).sum();
    let months = (0..12)
        .map(|m| {
            let mut month = HumidityMonth {
                month: m as u8 + 1,
                humidification_need_kwh: 0.0,
                heating_system_load_kwh: 0.0,
                steam_electricity_kwh: 0.0,
                steam_fuel_kwh: 0.0,
                recoverable_kwh: 0.0,
                dehumidification_need_kwh: 0.0,
            };
            if let Some(humidification) = &input.humidification {
                let moisture = input
                    .functions
                    .iter()
                    .map(|f| f.function.moisture(m) * f.area_m2)
                    .sum::<f64>()
                    / total_area;
                let recovery = if humidification.rotary_wheel {
                    WHEEL_LATENT_RECOVERY
                } else {
                    0.0
                };
                // 12.1 with f_HU;mi = 1.
                let need = LATENT_HEAT_KJ_PER_KG
                    * (1.0 - recovery)
                    * AIR_DENSITY
                    * input.supply_flow_m3_per_h[m]
                    / 3600.0
                    * moisture;
                month.humidification_need_kwh = need;
                match humidification.humidifier {
                    Humidifier::Atomising => month.heating_system_load_kwh = need,
                    Humidifier::Steam { carrier } => {
                        let efficiency = carrier.efficiency();
                        let energy = need / efficiency;
                        match carrier {
                            SteamCarrier::Electricity => month.steam_electricity_kwh = energy,
                            SteamCarrier::GasOrOil => month.steam_fuel_kwh = energy,
                        }
                        if input.usable_floor_area_m2 <= 500.0 {
                            month.recoverable_kwh = (1.0 - efficiency) * need;
                        }
                    }
                }
            }
            if let Some(design) = input.cooling_design {
                month.dehumidification_need_kwh = design.fraction(m) * input.cooling_need_kwh[m];
            }
            month
        })
        .collect();
    Ok(months)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn office(humidifier: Humidifier) -> HumidityInput {
        HumidityInput {
            zone_id: "z".into(),
            usable_floor_area_m2: 400.0,
            functions: vec![HumidityFunctionArea {
                function: HumidityFunction::General,
                area_m2: 400.0,
            }],
            humidification: Some(Humidification {
                humidifier,
                rotary_wheel: false,
                equipment_reference: "ahu".into(),
            }),
            supply_flow_m3_per_h: vec![1000.0; 12],
            cooling_design: Some(CoolingDesignTemperature::From12To16),
            cooling_need_kwh: vec![100.0; 12],
        }
    }

    #[test]
    fn humidification_follows_12_1_to_12_4() {
        let months = calculate_humidity(&office(Humidifier::Steam {
            carrier: SteamCarrier::Electricity,
        }))
        .unwrap();
        let need = 2538.2 * 1.205 * 1000.0 / 3600.0 * 0.82;
        assert!((months[0].humidification_need_kwh - need).abs() < 1e-9);
        assert!((months[0].steam_electricity_kwh - need / 0.8).abs() < 1e-9);
        assert!((months[0].recoverable_kwh - 0.2 * need).abs() < 1e-9);
        assert_eq!(months[5].humidification_need_kwh, 0.0);
        let atomising = calculate_humidity(&office(Humidifier::Atomising)).unwrap();
        assert!((atomising[0].heating_system_load_kwh - need).abs() < 1e-9);
        assert_eq!(atomising[0].recoverable_kwh, 0.0);
    }

    #[test]
    fn dehumidification_follows_table_12_2() {
        let months = calculate_humidity(&office(Humidifier::Atomising)).unwrap();
        assert!((months[7].dehumidification_need_kwh - 43.0).abs() < 1e-9);
        assert_eq!(months[5].dehumidification_need_kwh, 0.0);
        let mut floor = office(Humidifier::Atomising);
        floor.cooling_design = Some(CoolingDesignTemperature::From17To21);
        assert_eq!(
            calculate_humidity(&floor).unwrap()[7].dehumidification_need_kwh,
            0.0
        );
    }

    #[test]
    fn missing_flows_are_reported() {
        let mut input = office(Humidifier::Atomising);
        input.supply_flow_m3_per_h.pop();
        let codes: Vec<_> = validate_humidity(&input).iter().map(|i| i.code).collect();
        assert!(codes.contains(&"supply_flow_twelve_months_required"));
    }
}
