//! Booster heat pumps for hot water, NTA 8800:2025+C1:2026 annex W
//! (pp. 1118–1128).
//!
//! A booster heat pump (BWP) heats tap water from a collective heating
//! system at more than 12 °C, optionally also from the cooling emission
//! system. Monthly: COP from the source temperature (W.11–W.13) corrected
//! for the tapped quantity with the standing loss (W.14), electricity
//! (W.1, annex X rounding up), evaporator heat including the standing-loss
//! compensation (W.8–W.10), heat taken from the cooling system (W.3) and
//! heat drawn from the collective heating system (W.2).
//!
//! Values and formulas are transcribed from the licensed norm; the norm text
//! itself is not part of this repository.

use serde::{Deserialize, Serialize};

use crate::climate::MONTH_HOURS;
use crate::significant_figures::round_up;

/// W.1: `f_prac;gi` for measured booster heat pumps.
pub const PRACTICE_FACTOR: f64 = 0.95;
/// W.3.1: extrapolation allowed 4 K beyond the measured source temperatures.
pub const EXTRAPOLATION_K: f64 = 4.0;

/// One W.4 test at a source temperature for the measured class.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoosterTest {
    /// Inlet source temperature, °C (24 or 16, and 40).
    pub source_temperature_c: f64,
    /// `COP_ki;θj` (W.16).
    pub cop: f64,
}

/// Annual hot-water quantity of the measured application class (table
/// 13.23), kWh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoosterClass {
    Class1,
    Class2,
    Class3,
    Class4,
}

impl BoosterClass {
    pub fn annual_kwh(self) -> f64 {
        match self {
            Self::Class1 => 1805.0,
            Self::Class2 => 2500.0,
            Self::Class3 => 3195.0,
            Self::Class4 => 3890.0,
        }
    }
}

/// Where the source heat comes from (figure W.1).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BoosterHeatSource {
    /// External heat supply (9.6.7, η 1,0, carrier `dh`).
    ExternalHeat,
    /// A collective generator in the building with its heating generation
    /// efficiency (chapter 9) and carrier.
    CollectiveGenerator {
        #[serde(rename = "generationEfficiency")]
        generation_efficiency: f64,
        carrier: BoosterSourceCarrier,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoosterSourceCarrier {
    Gas,
    Oil,
    Electricity,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoosterHeatPump {
    /// Test at 24 °C (or 16 °C).
    pub low_test: BoosterTest,
    /// Test at 40 °C.
    pub high_test: BoosterTest,
    pub measured_class: BoosterClass,
    /// `P_ls` (W.17), kW.
    pub standing_loss_kw: f64,
    /// Monthly mean source inlet temperature `θ_evap;mi` (one or twelve
    /// values); in months using the cooling system as source the mean of
    /// the heating-system temperature and 20 °C.
    pub source_temperatures_c: Vec<f64>,
    /// `Q_C;HP;si;mi` per month (10.6/10.9), kWh, when the BWP also takes
    /// heat from the cooling emission system (figure W.2, `f_C = 1`).
    #[serde(default)]
    pub cooling_extraction_kwh: Option<Vec<f64>>,
    pub heat_source: BoosterHeatSource,
    pub test_report_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoosterIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoosterMonth {
    pub cop: f64,
    /// `W_W;aux;gen;bwp;mi` (W.1), kWh.
    pub electricity_kwh: f64,
    /// `Q_W;hp;mi` (W.8), kWh.
    pub evaporator_heat_kwh: f64,
    /// `Q_W;hp;ls;mi` (W.9), kWh.
    pub standing_loss_heat_kwh: f64,
    /// `Q_C;gen,BWP;in;mi` (W.3), kWh.
    pub cooling_heat_kwh: f64,
    /// `E_W;gen,in;hj;mi` (W.2), kWh.
    pub heating_system_heat_kwh: f64,
}

fn push(issues: &mut Vec<BoosterIssue>, code: &'static str, path: String) {
    issues.push(BoosterIssue { code, path });
}

pub fn validate_booster(pump: &BoosterHeatPump, path: &str) -> Vec<BoosterIssue> {
    let mut issues = Vec::new();
    for (field, test) in [("lowTest", &pump.low_test), ("highTest", &pump.high_test)] {
        if !(test.cop.is_finite() && test.cop > 1.0 && test.source_temperature_c.is_finite()) {
            push(
                &mut issues,
                "booster_test_invalid",
                format!("{path}.{field}"),
            );
        }
    }
    let (low, high) = (
        pump.low_test.source_temperature_c,
        pump.high_test.source_temperature_c,
    );
    if high - low < 1.0 {
        push(
            &mut issues,
            "booster_test_temperatures_invalid",
            format!("{path}.highTest.sourceTemperatureC"),
        );
    }
    if !(pump.standing_loss_kw.is_finite() && pump.standing_loss_kw >= 0.0) {
        push(
            &mut issues,
            "booster_standing_loss_invalid",
            format!("{path}.standingLossKw"),
        );
    }
    let temperatures = &pump.source_temperatures_c;
    if !(temperatures.len() == 1 || temperatures.len() == 12) {
        push(
            &mut issues,
            "booster_source_temperature_invalid",
            format!("{path}.sourceTemperaturesC"),
        );
    } else if temperatures.iter().any(|value| {
        // W.3.1: at most 4 K beyond the measured range.
        !value.is_finite() || *value < low - EXTRAPOLATION_K || *value > high + EXTRAPOLATION_K
    }) {
        push(
            &mut issues,
            "booster_source_temperature_outside_range",
            format!("{path}.sourceTemperaturesC"),
        );
    }
    if let Some(extraction) = &pump.cooling_extraction_kwh {
        if extraction.len() != 12
            || extraction
                .iter()
                .any(|value| !(value.is_finite() && *value >= 0.0))
        {
            push(
                &mut issues,
                "booster_cooling_extraction_invalid",
                format!("{path}.coolingExtractionKwh"),
            );
        }
    }
    if let BoosterHeatSource::CollectiveGenerator {
        generation_efficiency,
        source_reference,
        ..
    } = &pump.heat_source
    {
        if !(generation_efficiency.is_finite() && *generation_efficiency > 0.0) {
            push(
                &mut issues,
                "booster_source_efficiency_invalid",
                format!("{path}.heatSource.generationEfficiency"),
            );
        }
        if source_reference.trim().is_empty() {
            push(
                &mut issues,
                "source_reference_required",
                format!("{path}.heatSource.sourceReference"),
            );
        }
    }
    if pump.test_report_reference.trim().is_empty() {
        push(
            &mut issues,
            "source_reference_required",
            format!("{path}.testReportReference"),
        );
    }
    issues
}

/// W.14: COP at a test temperature for the actual annual quantity.
fn tapped_cop(class_cop: f64, class_kwh: f64, annual_kwh: f64, standing_kwh: f64) -> f64 {
    let quantity = annual_kwh.min(class_kwh);
    quantity / ((class_kwh / class_cop - standing_kwh) * quantity / class_kwh + standing_kwh)
}

/// Annex W per month for the generator output `Q_W;gen;gi,out;mi` (kWh);
/// call after [`validate_booster`].
pub fn calculate_booster(pump: &BoosterHeatPump, output_kwh: &[f64; 12]) -> [BoosterMonth; 12] {
    let annual: f64 = output_kwh.iter().sum();
    let year_hours: f64 = MONTH_HOURS.iter().sum();
    let standing_year = pump.standing_loss_kw * year_hours;
    let class = pump.measured_class.annual_kwh();
    let low = tapped_cop(pump.low_test.cop, class, annual, standing_year);
    let high = tapped_cop(pump.high_test.cop, class, annual, standing_year);
    // W.12/W.13.
    let c1 =
        (low - high) / (pump.low_test.source_temperature_c - pump.high_test.source_temperature_c);
    let c2 = high - c1 * pump.high_test.source_temperature_c;
    std::array::from_fn(|index| {
        let output = output_kwh[index];
        let source = if pump.source_temperatures_c.len() == 12 {
            pump.source_temperatures_c[index]
        } else {
            pump.source_temperatures_c[0]
        };
        // W.11.
        let cop = c1 * source + c2;
        if output <= 0.0 || cop <= 0.0 {
            return BoosterMonth {
                cop,
                ..BoosterMonth::default()
            };
        }
        // W.1 rounded up per annex X.
        let electricity = round_up(output / (cop * PRACTICE_FACTOR));
        // W.8–W.10 (in kWh instead of MJ).
        let evaporator = output * (cop - 1.0) / cop;
        let standing = pump.standing_loss_kw * MONTH_HOURS[index];
        let denominator = (output / cop - standing).max(standing);
        let standing_heat = if denominator > 0.0 {
            (output / denominator - 1.0) * standing
        } else {
            0.0
        };
        // W.3.
        let cooling = pump
            .cooling_extraction_kwh
            .as_ref()
            .map_or(0.0, |values| values[index].min(evaporator + standing_heat));
        // W.2 rounded up per annex X.
        let heating = round_up((evaporator + standing_heat - cooling).max(0.0));
        BoosterMonth {
            cop,
            electricity_kwh: electricity,
            evaporator_heat_kwh: evaporator,
            standing_loss_heat_kwh: standing_heat,
            cooling_heat_kwh: cooling,
            heating_system_heat_kwh: heating,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pump() -> BoosterHeatPump {
        BoosterHeatPump {
            low_test: BoosterTest {
                source_temperature_c: 24.0,
                cop: 3.0,
            },
            high_test: BoosterTest {
                source_temperature_c: 40.0,
                cop: 4.2,
            },
            measured_class: BoosterClass::Class3,
            standing_loss_kw: 0.02,
            source_temperatures_c: vec![32.0],
            cooling_extraction_kwh: None,
            heat_source: BoosterHeatSource::ExternalHeat,
            test_report_reference: "synthetic Kiwa report".into(),
        }
    }

    #[test]
    fn w14_keeps_the_class_cop_above_the_class_quantity() {
        assert!((tapped_cop(3.0, 3195.0, 4000.0, 175.2) - 3.0).abs() < 1e-12);
        // Below the class quantity the standing loss weighs more.
        let reduced = tapped_cop(3.0, 3195.0, 1600.0, 175.2);
        let expected = 1600.0 / ((3195.0 / 3.0 - 175.2) * 1600.0 / 3195.0 + 175.2);
        assert!((reduced - expected).abs() < 1e-12);
        assert!(reduced < 3.0);
    }

    #[test]
    fn monthly_values_follow_w1_to_w11() {
        let input = pump();
        assert!(validate_booster(&input, "b").is_empty());
        let output = [300.0; 12];
        let months = calculate_booster(&input, &output);
        // Annual 3 600 kWh > class 3: COP_θj equal to the test values.
        let cop = 3.0 + (4.2 - 3.0) / 16.0 * 8.0;
        let jan = &months[0];
        assert!((jan.cop - cop).abs() < 1e-12);
        assert_eq!(jan.electricity_kwh, round_up(300.0 / (cop * 0.95)));
        let evaporator = 300.0 * (cop - 1.0) / cop;
        assert!((jan.evaporator_heat_kwh - evaporator).abs() < 1e-12);
        let standing = 0.02 * 744.0;
        let standing_heat = (300.0 / (300.0 / cop - standing) - 1.0) * standing;
        assert!((jan.standing_loss_heat_kwh - standing_heat).abs() < 1e-9);
        assert_eq!(
            jan.heating_system_heat_kwh,
            round_up(evaporator + standing_heat)
        );
    }

    #[test]
    fn cooling_source_reduces_the_heating_system_heat() {
        let mut input = pump();
        input.cooling_extraction_kwh = Some(vec![50.0; 12]);
        let months = calculate_booster(&input, &[300.0; 12]);
        let jan = &months[0];
        assert_eq!(jan.cooling_heat_kwh, 50.0);
        assert_eq!(
            jan.heating_system_heat_kwh,
            round_up(jan.evaporator_heat_kwh + jan.standing_loss_heat_kwh - 50.0)
        );
    }

    #[test]
    fn source_temperature_outside_the_extrapolation_range_is_rejected() {
        let mut input = pump();
        input.source_temperatures_c = vec![18.0];
        let codes: Vec<_> = validate_booster(&input, "b")
            .iter()
            .map(|item| item.code)
            .collect();
        assert!(codes.contains(&"booster_source_temperature_outside_range"));
    }
}
