//! Domestic hot water, NTA 8800 chapter 13 (partial).
//!
//! The net need for the residential category follows 13.15 with the
//! specific need of 856 kWh per occupant per year (§13.2.3.1) and the
//! occupant bands of 13.16–13.18, spread over the months with `t_mi / 8760`.
//! Utility buildings supply their table 13.1 value with a source. Emission,
//! distribution and generation efficiencies, and any shower heat recovery,
//! are supplied with a source because their tables are not transcribed yet.
//!
//! Transcription source: Open Heatloss Studio `nta8800-dhw` references
//! (§13.2.2.1, §13.2.3.1); a review against the norm text is pending.

use crate::building_performance::Carrier;
use crate::climate::MONTH_HOURS;
use crate::monthly_demand::occupants_per_dwelling;
use serde::{Deserialize, Serialize};

/// §13.2.3.1: residential need per occupant, kWh per year.
pub const RESIDENTIAL_NEED_PER_OCCUPANT_KWH: f64 = 856.0;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum HotWaterNeed {
    Residential {
        #[serde(rename = "dwellingCount")]
        dwelling_count: u32,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Table 13.1 value for the utility function, kWh/m² per year.
    Declared {
        #[serde(rename = "specificNeedKwhPerM2Year")]
        specific_need_kwh_per_m2_year: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HotWaterSystem {
    pub need: HotWaterNeed,
    /// `η_W;em`, 0 < η ≤ 1.
    pub emission_efficiency: f64,
    /// `η_W;dis`, 0 < η ≤ 1.
    pub distribution_efficiency: f64,
    /// `η_W;gen;prac` or seasonal COP of a hot-water heat pump (may exceed 1).
    pub generation_efficiency: f64,
    pub carrier: Carrier,
    /// Monthly heat recovered from shower water (13.51), kWh; empty = none.
    #[serde(default)]
    pub shower_heat_recovery_kwh: Vec<f64>,
    /// Auxiliary electricity per month, kWh; empty = none declared.
    #[serde(default)]
    pub auxiliary_electricity_kwh: Vec<f64>,
    /// Hot-water heat pump with a source below 20 °C that is not exhaust air
    /// (5.35/5.36): ambient heat counts as renewable.
    pub renewable_heat_pump: bool,
    pub efficiency_source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotWaterMonth {
    pub month: u8,
    pub net_need_kwh: f64,
    pub recovered_kwh: f64,
    pub generator_output_kwh: f64,
    pub carrier_input_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    pub ambient_heat_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HotWaterIssue {
    pub code: &'static str,
    pub path: String,
}

pub fn validate_hot_water(system: &HotWaterSystem, path: &str) -> Vec<HotWaterIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: &str| {
        issues.push(HotWaterIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    match &system.need {
        HotWaterNeed::Residential {
            dwelling_count,
            source_reference,
        } => {
            if *dwelling_count == 0 {
                push("dwelling_count_invalid", "need.dwellingCount");
            }
            if source_reference.trim().is_empty() {
                push("source_reference_required", "need.sourceReference");
            }
        }
        HotWaterNeed::Declared {
            specific_need_kwh_per_m2_year,
            source_reference,
        } => {
            if !specific_need_kwh_per_m2_year.is_finite() || *specific_need_kwh_per_m2_year < 0.0 {
                push("hot_water_need_invalid", "need.specificNeedKwhPerM2Year");
            }
            if source_reference.trim().is_empty() {
                push("source_reference_required", "need.sourceReference");
            }
        }
    }
    for (field, value) in [
        ("emissionEfficiency", system.emission_efficiency),
        ("distributionEfficiency", system.distribution_efficiency),
    ] {
        if !value.is_finite() || value <= 0.0 || value > 1.0 {
            push("hot_water_efficiency_invalid", field);
        }
    }
    if !system.generation_efficiency.is_finite() || system.generation_efficiency <= 0.0 {
        push("hot_water_efficiency_invalid", "generationEfficiency");
    }
    if system.renewable_heat_pump
        && (system.carrier != Carrier::El || system.generation_efficiency < 1.0)
    {
        push(
            "hot_water_renewable_heat_pump_inconsistent",
            "renewableHeatPump",
        );
    }
    for (field, values) in [
        ("showerHeatRecoveryKwh", &system.shower_heat_recovery_kwh),
        ("auxiliaryElectricityKwh", &system.auxiliary_electricity_kwh),
    ] {
        if !values.is_empty()
            && (values.len() != 12
                || values
                    .iter()
                    .any(|value| !value.is_finite() || *value < 0.0))
        {
            push("monthly_values_invalid", field);
        }
    }
    if system.efficiency_source_reference.trim().is_empty() {
        push("source_reference_required", "efficiencySourceReference");
    }
    issues
}

/// Monthly results; call only after [`validate_hot_water`] returned no issues.
pub fn monthly_hot_water(system: &HotWaterSystem, usable_floor_area_m2: f64) -> Vec<HotWaterMonth> {
    let annual_need = match &system.need {
        HotWaterNeed::Residential { dwelling_count, .. } => {
            let dwellings = f64::from(*dwelling_count);
            // 13.15 with 13.16–13.18.
            RESIDENTIAL_NEED_PER_OCCUPANT_KWH
                * dwellings
                * occupants_per_dwelling(usable_floor_area_m2 / dwellings)
        }
        HotWaterNeed::Declared {
            specific_need_kwh_per_m2_year,
            ..
        } => specific_need_kwh_per_m2_year * usable_floor_area_m2,
    };
    let total_hours: f64 = MONTH_HOURS.iter().sum();
    (0..12)
        .map(|index| {
            let need = annual_need * MONTH_HOURS[index] / total_hours;
            let recovered = system
                .shower_heat_recovery_kwh
                .get(index)
                .copied()
                .unwrap_or(0.0)
                .min(need);
            let output =
                (need - recovered) / (system.emission_efficiency * system.distribution_efficiency);
            let input = output / system.generation_efficiency;
            let ambient = if system.renewable_heat_pump {
                output * (1.0 - 1.0 / system.generation_efficiency)
            } else {
                0.0
            };
            HotWaterMonth {
                month: index as u8 + 1,
                net_need_kwh: need,
                recovered_kwh: recovered,
                generator_output_kwh: output,
                carrier_input_kwh: input,
                auxiliary_electricity_kwh: system
                    .auxiliary_electricity_kwh
                    .get(index)
                    .copied()
                    .unwrap_or(0.0),
                ambient_heat_kwh: ambient,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system() -> HotWaterSystem {
        HotWaterSystem {
            need: HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "one dwelling".into(),
            },
            emission_efficiency: 0.9,
            distribution_efficiency: 1.0,
            generation_efficiency: 0.8,
            carrier: Carrier::Gas,
            shower_heat_recovery_kwh: Vec::new(),
            auxiliary_electricity_kwh: Vec::new(),
            renewable_heat_pump: false,
            efficiency_source_reference: "synthetic".into(),
        }
    }

    #[test]
    fn residential_need_follows_13_15_and_month_lengths() {
        let months = monthly_hot_water(&system(), 100.0);
        let annual: f64 = months.iter().map(|row| row.net_need_kwh).sum();
        assert!((annual - 856.0 * 2.28).abs() < 1e-9);
        assert!((months[1].net_need_kwh - 856.0 * 2.28 * 672.0 / 8760.0).abs() < 1e-9);
        let jan = &months[0];
        assert!((jan.carrier_input_kwh - jan.net_need_kwh / 0.9 / 0.8).abs() < 1e-9);
    }

    #[test]
    fn heat_pump_ambient_heat_and_recovery() {
        let mut pump = system();
        pump.carrier = Carrier::El;
        pump.generation_efficiency = 2.5;
        pump.renewable_heat_pump = true;
        pump.shower_heat_recovery_kwh = vec![20.0; 12];
        let jan = &monthly_hot_water(&pump, 100.0)[0];
        let output = (jan.net_need_kwh - 20.0) / 0.9;
        assert!((jan.generator_output_kwh - output).abs() < 1e-9);
        assert!((jan.ambient_heat_kwh - output * (1.0 - 1.0 / 2.5)).abs() < 1e-9);
        assert!(validate_hot_water(&pump, "dhw").is_empty());
    }

    #[test]
    fn validation_rejects_inconsistent_input() {
        let mut bad = system();
        bad.emission_efficiency = 1.2;
        bad.renewable_heat_pump = true;
        bad.shower_heat_recovery_kwh = vec![1.0; 11];
        bad.efficiency_source_reference = String::new();
        let codes: Vec<_> = validate_hot_water(&bad, "dhw")
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "hot_water_efficiency_invalid",
            "hot_water_renewable_heat_pump_inconsistent",
            "monthly_values_invalid",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }
}
