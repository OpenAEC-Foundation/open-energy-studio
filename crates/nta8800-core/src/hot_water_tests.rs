//! Test-based efficiencies for chapter 13, NTA 8800:2025+C1:2026:
//!
//! - annex T (pp. 1096–1109): generation efficiency of gas-fired hot-water
//!   appliances from a Dutch tapping-pattern test, as used for the Gaskeur
//!   coupling: T.4 (tap-side efficiency on lower heating value), T.5/T.6 and
//!   table T.7 (conversion to higher heating value), and the annual
//!   efficiency of combi appliances from measured summer and winter values
//!   (T.12–T.14) or by the forfait conversion (T.15–T.17, K_f = 0,5);
//! - annex U (pp. 1110–1113): shower drain-water heat recovery efficiency
//!   (U.2–U.6), the mean of three runs rounded down to 0,025.
//!
//! Electric appliances (T.3), bivalent heat pumps (T.7/T.8) and micro-CHP
//! (T.9/T.10) are not offered here: chapter 13 takes electric and heat-pump
//! efficiencies on the delivered-energy basis, whereas annex T includes the
//! primary conversion of electricity.

use serde::{Deserialize, Serialize};

use crate::building_performance::F_P_ELECTRICITY;

/// T.4.1/T.4.2: days in the summer and winter period.
pub const SUMMER_DAYS: f64 = 153.0;
pub const WINTER_DAYS: f64 = 212.0;
/// T.4.2.2: forfait correction factor K_f.
pub const COMBI_FORFAIT_FACTOR: f64 = 0.5;
/// T.4.2.1: minimum tap-side efficiency on the higher heating value (CW).
pub const COMBI_FORFAIT_MIN_EFFICIENCY: f64 = 0.40;

/// η_el;ow = 1/f_P;del;el (T.3/T.4, 5.5.5).
fn electricity_supply_efficiency() -> f64 {
    1.0 / F_P_ELECTRICITY
}

/// Table T.7 fuels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestFuel {
    NaturalGas,
    Propane,
    Butane,
}

impl TestFuel {
    /// Table T.7: η_bw/η_ow.
    pub fn higher_heating_value_factor(self) -> f64 {
        match self {
            Self::NaturalGas => 0.902,
            Self::Propane => 0.921,
            Self::Butane => 0.924,
        }
    }
}

/// T.4: η_tg on the lower heating value, auxiliary electricity included.
pub fn tap_side_efficiency(useful_mj: f64, fuel_input_mj: f64, electricity_kwh: f64) -> f64 {
    useful_mj / (fuel_input_mj + 3.6 * electricity_kwh / electricity_supply_efficiency())
}

/// One annex T test report.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnnexTTest {
    /// Water heater without a heating function: T.4 and T.5.
    WaterHeater {
        /// Q_n, MJ per test day.
        #[serde(rename = "usefulMj")]
        useful_mj: f64,
        /// Q_prim;toe;ov on the lower heating value, MJ per test day.
        #[serde(rename = "fuelInputMj")]
        fuel_input_mj: f64,
        /// Q_toe;el including auxiliaries, kWh per test day.
        #[serde(rename = "electricityKwh")]
        electricity_kwh: f64,
        fuel: TestFuel,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Gas-fired combi appliance, forfait winter efficiency: T.15–T.17.
    CombiForfait {
        #[serde(rename = "usefulMj")]
        useful_mj: f64,
        #[serde(rename = "fuelInputMj")]
        fuel_input_mj: f64,
        #[serde(rename = "electricityKwh")]
        electricity_kwh: f64,
        /// η_cv-nom on the lower heating value.
        #[serde(rename = "fullLoadEfficiency")]
        full_load_efficiency: f64,
        fuel: TestFuel,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Gas-fired combi appliance, measured summer and winter: T.12–T.14.
    CombiMeasured {
        /// Q_tap;Z and Q_toe;Z (lower heating value), MJ/day.
        #[serde(rename = "summerUsefulMjPerDay")]
        summer_useful_mj_per_day: f64,
        #[serde(rename = "summerFuelMjPerDay")]
        summer_fuel_mj_per_day: f64,
        /// Q_tap;W, Q_toe;W and Q_CV, MJ/day.
        #[serde(rename = "winterUsefulMjPerDay")]
        winter_useful_mj_per_day: f64,
        #[serde(rename = "winterFuelMjPerDay")]
        winter_fuel_mj_per_day: f64,
        #[serde(rename = "heatingFuelMjPerDay")]
        heating_fuel_mj_per_day: f64,
        #[serde(rename = "fullLoadEfficiency")]
        full_load_efficiency: f64,
        /// Q_ZE and Q_EE, kWh/day.
        #[serde(rename = "summerElectricityKwhPerDay")]
        summer_electricity_kwh_per_day: f64,
        #[serde(rename = "electronicsKwhPerDay")]
        electronics_kwh_per_day: f64,
        fuel: TestFuel,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

impl AnnexTTest {
    pub fn is_combi(&self) -> bool {
        !matches!(self, Self::WaterHeater { .. })
    }

    pub fn source_reference(&self) -> &str {
        match self {
            Self::WaterHeater {
                source_reference, ..
            }
            | Self::CombiForfait {
                source_reference, ..
            }
            | Self::CombiMeasured {
                source_reference, ..
            } => source_reference,
        }
    }

    fn values(&self) -> Vec<f64> {
        match self {
            Self::WaterHeater {
                useful_mj,
                fuel_input_mj,
                electricity_kwh,
                ..
            } => vec![*useful_mj, *fuel_input_mj, *electricity_kwh],
            Self::CombiForfait {
                useful_mj,
                fuel_input_mj,
                electricity_kwh,
                full_load_efficiency,
                ..
            } => vec![
                *useful_mj,
                *fuel_input_mj,
                *electricity_kwh,
                *full_load_efficiency,
            ],
            Self::CombiMeasured {
                summer_useful_mj_per_day,
                summer_fuel_mj_per_day,
                winter_useful_mj_per_day,
                winter_fuel_mj_per_day,
                heating_fuel_mj_per_day,
                full_load_efficiency,
                summer_electricity_kwh_per_day,
                electronics_kwh_per_day,
                ..
            } => vec![
                *summer_useful_mj_per_day,
                *summer_fuel_mj_per_day,
                *winter_useful_mj_per_day,
                *winter_fuel_mj_per_day,
                *heating_fuel_mj_per_day,
                *full_load_efficiency,
                *summer_electricity_kwh_per_day,
                *electronics_kwh_per_day,
            ],
        }
    }

    /// Validation codes; empty means the report can be evaluated.
    pub fn issues(&self) -> Vec<&'static str> {
        let mut issues = Vec::new();
        if self.values().iter().any(|v| !v.is_finite() || *v < 0.0) {
            issues.push("annex_t_value_invalid");
        }
        if self.source_reference().trim().is_empty() {
            issues.push("source_reference_required");
        }
        if issues.is_empty() {
            match self.efficiency() {
                Ok(value) if value > 0.0 && value <= 1.2 => {}
                Ok(_) => issues.push("annex_t_value_invalid"),
                Err(code) => issues.push(code),
            }
        }
        issues
    }

    /// η_W;gen on the higher heating value (T.5/T.6), before rounding.
    pub fn efficiency(&self) -> Result<f64, &'static str> {
        let positive = |value: f64| value > 0.0;
        match self {
            Self::WaterHeater {
                useful_mj,
                fuel_input_mj,
                electricity_kwh,
                fuel,
                ..
            } => {
                if !positive(*fuel_input_mj + *electricity_kwh) {
                    return Err("annex_t_value_invalid");
                }
                Ok(
                    tap_side_efficiency(*useful_mj, *fuel_input_mj, *electricity_kwh)
                        * fuel.higher_heating_value_factor(),
                )
            }
            Self::CombiForfait {
                useful_mj,
                fuel_input_mj,
                electricity_kwh,
                full_load_efficiency,
                fuel,
                ..
            } => {
                if !positive(*fuel_input_mj + *electricity_kwh) {
                    return Err("annex_t_value_invalid");
                }
                let tap = tap_side_efficiency(*useful_mj, *fuel_input_mj, *electricity_kwh);
                // T.4.2.1: only for CW appliances.
                if tap * fuel.higher_heating_value_factor() < COMBI_FORFAIT_MIN_EFFICIENCY {
                    return Err("annex_t_combi_forfait_requires_cw");
                }
                // T.16/T.17 and T.15.
                let summer = tap;
                let winter = tap + COMBI_FORFAIT_FACTOR * (full_load_efficiency - tap);
                if !positive(summer) || !positive(winter) {
                    return Err("annex_t_value_invalid");
                }
                let annual =
                    (SUMMER_DAYS + WINTER_DAYS) / (SUMMER_DAYS / summer + WINTER_DAYS / winter);
                Ok(annual * fuel.higher_heating_value_factor())
            }
            Self::CombiMeasured {
                summer_useful_mj_per_day,
                summer_fuel_mj_per_day,
                winter_useful_mj_per_day,
                winter_fuel_mj_per_day,
                heating_fuel_mj_per_day,
                full_load_efficiency,
                summer_electricity_kwh_per_day,
                electronics_kwh_per_day,
                fuel,
                ..
            } => {
                if !positive(*summer_fuel_mj_per_day) || !positive(*full_load_efficiency) {
                    return Err("annex_t_value_invalid");
                }
                // T.13/T.14.
                let summer = summer_useful_mj_per_day / summer_fuel_mj_per_day;
                let winter_fuel =
                    winter_fuel_mj_per_day - heating_fuel_mj_per_day / full_load_efficiency;
                if !positive(winter_fuel) || !positive(summer) {
                    return Err("annex_t_value_invalid");
                }
                let winter = winter_useful_mj_per_day / winter_fuel;
                if !positive(winter) {
                    return Err("annex_t_value_invalid");
                }
                // T.12.
                let useful =
                    summer_useful_mj_per_day * SUMMER_DAYS + winter_useful_mj_per_day * WINTER_DAYS;
                let electricity = (SUMMER_DAYS + WINTER_DAYS)
                    * 3.6
                    * (summer_electricity_kwh_per_day - electronics_kwh_per_day)
                    / electricity_supply_efficiency();
                let input = summer_useful_mj_per_day * SUMMER_DAYS / summer
                    + winter_useful_mj_per_day * WINTER_DAYS / winter
                    + electricity;
                if !positive(input) {
                    return Err("annex_t_value_invalid");
                }
                Ok(useful / input * fuel.higher_heating_value_factor())
            }
        }
    }
}

/// U.5: density of water, kg/m³.
pub fn water_density(temperature_c: f64) -> f64 {
    let t = temperature_c;
    999.9649 + 0.0264672 * t - 0.0061549 * t * t + 1.775e-5 * t.powi(3)
}

/// U.6: specific enthalpy of water, kJ/kg.
pub fn water_enthalpy(temperature_c: f64) -> f64 {
    let t = temperature_c;
    0.167853 + 4.18587 * t - 0.000146789 * t * t + 9.38153e-7 * t.powi(3) + 8.36764e-9 * t.powi(4)
}

/// Shower classes of table U.1 (the heater's application class).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowerTestClass {
    Class2,
    Class3,
    /// CW classes 4, 5 and 6.
    Class4,
}

impl ShowerTestClass {
    /// Table U.1: (flow l/min, volume l).
    pub fn conditions(self) -> (f64, f64) {
        match self {
            Self::Class2 => (5.8, 47.0),
            Self::Class3 => (9.2, 73.0),
            Self::Class4 => (12.5, 100.0),
        }
    }
}

/// One sample of an annex U run.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShowerSample {
    pub cold_flow_m3_per_s: f64,
    pub cold_in_c: f64,
    pub cold_out_c: f64,
    pub shower_flow_m3_per_s: f64,
    pub shower_c: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShowerRun {
    /// Q_DWTW and Q_douche from the report, kJ.
    Energies {
        #[serde(rename = "recoveredKj")]
        recovered_kj: f64,
        #[serde(rename = "showerKj")]
        shower_kj: f64,
    },
    /// U.3/U.4 from the logged samples (sample time at most 1 s).
    Samples {
        #[serde(rename = "sampleTimeS")]
        sample_time_s: f64,
        samples: Vec<ShowerSample>,
    },
}

impl ShowerRun {
    /// U.2: η_DWTW of one run.
    pub fn efficiency(&self) -> Option<f64> {
        let (recovered, shower) = match self {
            Self::Energies {
                recovered_kj,
                shower_kj,
            } => (*recovered_kj, *shower_kj),
            Self::Samples {
                sample_time_s,
                samples,
            } => samples.iter().fold((0.0, 0.0), |(r, s), item| {
                (
                    r + item.cold_flow_m3_per_s
                        * water_density(item.cold_in_c)
                        * (water_enthalpy(item.cold_out_c) - water_enthalpy(item.cold_in_c))
                        * sample_time_s,
                    s + item.shower_flow_m3_per_s
                        * water_density(item.shower_c)
                        * (water_enthalpy(item.shower_c) - water_enthalpy(item.cold_in_c))
                        * sample_time_s,
                )
            }),
        };
        (shower > 0.0 && recovered.is_finite() && recovered >= 0.0).then(|| recovered / shower)
    }

    fn valid(&self) -> bool {
        match self {
            Self::Energies {
                recovered_kj,
                shower_kj,
            } => recovered_kj.is_finite() && *recovered_kj >= 0.0 && *shower_kj > 0.0,
            Self::Samples {
                sample_time_s,
                samples,
            } => {
                *sample_time_s > 0.0
                    && *sample_time_s <= 1.0
                    && !samples.is_empty()
                    && samples.iter().all(|item| {
                        [
                            item.cold_flow_m3_per_s,
                            item.cold_in_c,
                            item.cold_out_c,
                            item.shower_flow_m3_per_s,
                            item.shower_c,
                        ]
                        .iter()
                        .all(|value| value.is_finite())
                            && item.cold_flow_m3_per_s >= 0.0
                            && item.shower_flow_m3_per_s >= 0.0
                    })
            }
        }
    }
}

/// An annex U test at one shower class.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnnexUTest {
    pub class: ShowerTestClass,
    /// Three runs per class (U.3.3).
    pub runs: Vec<ShowerRun>,
    pub source_reference: String,
}

impl AnnexUTest {
    pub fn issues(&self) -> Vec<&'static str> {
        let mut issues = Vec::new();
        if self.runs.len() != 3 {
            issues.push("annex_u_three_runs_required");
        }
        if self.runs.iter().any(|run| !run.valid()) {
            issues.push("annex_u_run_invalid");
        } else if let Some(value) = self.efficiency() {
            if value > 1.0 {
                issues.push("annex_u_run_invalid");
            }
        }
        if self.source_reference.trim().is_empty() {
            issues.push("source_reference_required");
        }
        issues
    }

    /// Mean of the runs, rounded down to a multiple of 0,025.
    pub fn efficiency(&self) -> Option<f64> {
        let values: Option<Vec<f64>> = self.runs.iter().map(ShowerRun::efficiency).collect();
        let values = values?;
        if values.is_empty() {
            return None;
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        Some((mean / 0.025 + 1e-9).floor() * 0.025)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} vs {b}");
    }

    #[test]
    fn water_heater_follows_t_4_t_5_and_table_t_7() {
        let test = AnnexTTest::WaterHeater {
            useful_mj: 20.0,
            fuel_input_mj: 25.0,
            electricity_kwh: 0.1,
            fuel: TestFuel::NaturalGas,
            source_reference: "Gaskeur report".into(),
        };
        // η_el;ow = 1/1,45: 3,6·0,1·1,45 = 0,522 MJ.
        let tap = 20.0 / (25.0 + 0.522);
        close(test.efficiency().unwrap(), tap * 0.902);
        assert!(test.issues().is_empty());
    }

    #[test]
    fn combi_forfait_follows_t_15_to_t_17() {
        let test = AnnexTTest::CombiForfait {
            useful_mj: 20.0,
            fuel_input_mj: 25.0,
            electricity_kwh: 0.0,
            full_load_efficiency: 0.96,
            fuel: TestFuel::NaturalGas,
            source_reference: "Gaskeur report".into(),
        };
        let summer = 0.8;
        let winter = 0.8 + 0.5 * (0.96 - 0.8);
        let annual = 365.0 / (153.0 / summer + 212.0 / winter);
        close(test.efficiency().unwrap(), annual * 0.902);
        let poor = AnnexTTest::CombiForfait {
            useful_mj: 10.0,
            fuel_input_mj: 30.0,
            electricity_kwh: 0.0,
            full_load_efficiency: 0.96,
            fuel: TestFuel::NaturalGas,
            source_reference: "x".into(),
        };
        assert_eq!(poor.issues(), vec!["annex_t_combi_forfait_requires_cw"]);
    }

    #[test]
    fn combi_measured_follows_t_12_to_t_14() {
        let test = AnnexTTest::CombiMeasured {
            summer_useful_mj_per_day: 20.0,
            summer_fuel_mj_per_day: 25.0,
            winter_useful_mj_per_day: 22.0,
            winter_fuel_mj_per_day: 30.0,
            heating_fuel_mj_per_day: 4.8,
            full_load_efficiency: 0.96,
            summer_electricity_kwh_per_day: 0.2,
            electronics_kwh_per_day: 0.1,
            fuel: TestFuel::Propane,
            source_reference: "report".into(),
        };
        let z = 20.0 / 25.0;
        let w = 22.0 / (30.0 - 4.8 / 0.96);
        let useful = 20.0 * 153.0 + 22.0 * 212.0;
        let input = 20.0 * 153.0 / z + 22.0 * 212.0 / w + 365.0 * 3.6 * 0.1 * 1.45;
        close(test.efficiency().unwrap(), useful / input * 0.921);
    }

    #[test]
    fn water_properties_follow_u_5_and_u_6() {
        close(water_density(0.0), 999.9649);
        close(
            water_density(40.0),
            999.9649 + 0.0264672 * 40.0 - 0.0061549 * 1600.0 + 1.775e-5 * 64000.0,
        );
        close(
            water_enthalpy(10.0),
            0.167853 + 41.8587 - 0.0146789 + 9.38153e-4 + 8.36764e-5,
        );
    }

    #[test]
    fn shower_test_rounds_the_mean_down() {
        let run = |r: f64| ShowerRun::Energies {
            recovered_kj: r,
            shower_kj: 100.0,
        };
        let test = AnnexUTest {
            class: ShowerTestClass::Class3,
            runs: vec![run(47.0), run(48.0), run(49.9)],
            source_reference: "lab report".into(),
        };
        // Mean 0,483 → 0,475.
        close(test.efficiency().unwrap(), 0.475);
        assert!(test.issues().is_empty());
        let short = AnnexUTest {
            runs: vec![run(47.0)],
            ..test
        };
        assert!(short.issues().contains(&"annex_u_three_runs_required"));
    }

    #[test]
    fn shower_samples_follow_u_3_and_u_4() {
        let sample = ShowerSample {
            cold_flow_m3_per_s: 1e-4,
            cold_in_c: 10.0,
            cold_out_c: 22.0,
            shower_flow_m3_per_s: 1e-4,
            shower_c: 40.0,
        };
        let run = ShowerRun::Samples {
            sample_time_s: 1.0,
            samples: vec![sample; 10],
        };
        let expected = water_density(10.0) * (water_enthalpy(22.0) - water_enthalpy(10.0))
            / (water_density(40.0) * (water_enthalpy(40.0) - water_enthalpy(10.0)));
        close(run.efficiency().unwrap(), expected);
        assert_eq!(ShowerTestClass::Class4.conditions(), (12.5, 100.0));
    }
}
