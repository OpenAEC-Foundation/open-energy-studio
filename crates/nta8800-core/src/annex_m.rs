//! Boilers with product values, NTA 8800:2025+C1:2026 annex M (based on
//! NEN-EN 15316-4-1, pp. 870–879): method 1 of 9.6.2.2 (gas or oil) and
//! 9.6.5.2 (biomass boilers).
//!
//! Per month: the part-load ratio (M.25/M.26), temperature-corrected
//! efficiencies at full and intermediate load (M.7/M.8/M.10, tables
//! M.2/M.4, or M.13/M.14 from additional tests), the losses at full,
//! intermediate and stand-by load (M.9/M.11/M.12) interpolated by load
//! (M.4/M.5) over the operating time of table 9.15 (M.6), the auxiliary
//! energy (M.21–M.23), the auxiliary energy recovered by the medium
//! (M.18, f_aux;rvd 0,75) and the input energy (M.1) with the control factor
//! of table M.7 (M.24). Recoverable losses to the space (M.16, M.19) are
//! fed back into the need through 9.7 by the heating chain (individual
//! dwelling installations up to 500 m², 9.2.5.1).

use serde::{Deserialize, Serialize};

/// f_prac (M.9, M.11, M.12).
pub const PRACTICE_FACTOR: f64 = 0.95;
/// f_aux;rvd (M.5.3).
pub const AUX_TO_MEDIUM_FRACTION: f64 = 0.75;
/// M.29: P_int = 0,3·P_n when nothing else is known.
pub const DEFAULT_INTERMEDIATE_RATIO: f64 = 0.3;

/// Tables M.2 and M.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoilerTechnology {
    SolidFuelStandard,
    GasOilStandard,
    LowTemperature,
    CondensingGas,
    CondensingOil,
}

impl BoilerTechnology {
    pub fn condensing(self) -> bool {
        matches!(self, Self::CondensingGas | Self::CondensingOil)
    }

    /// Table M.2: (ϑ_gen;test;Pn, f_corr;Pn in 1/K).
    fn full_load_correction(self) -> (f64, f64) {
        match self {
            Self::SolidFuelStandard | Self::GasOilStandard => (70.0, 0.0),
            Self::LowTemperature | Self::CondensingOil => (70.0, 0.0004),
            Self::CondensingGas => (70.0, 0.0020),
        }
    }

    /// Table M.4: (ϑ_gen;test;Pint, f_corr;Pint in 1/K).
    fn part_load_correction(self) -> (f64, f64) {
        match self {
            Self::SolidFuelStandard => (70.0, 0.0004),
            Self::GasOilStandard => (50.0, 0.0004),
            Self::LowTemperature => (40.0, 0.0004),
            Self::CondensingGas => (30.0, 0.0020),
            Self::CondensingOil => (30.0, 0.0010),
        }
    }
}

/// Table M.3 f_Hs/Hi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoilerFuel {
    NaturalGas,
    Oil,
    Wood,
}

impl BoilerFuel {
    pub fn gross_to_net(self) -> f64 {
        match self {
            Self::NaturalGas => 1.11,
            Self::Oil => 1.06,
            Self::Wood => 1.08,
        }
    }
}

/// Table M.6: (f_brm, default ϑ_brm; `None` means the outdoor temperature).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoilerPlacement {
    Outdoors,
    InstallationRoom,
    UnderRoof,
    HeatedSpace,
}

impl BoilerPlacement {
    fn reduction(self) -> f64 {
        match self {
            Self::Outdoors => 1.0,
            Self::InstallationRoom => 0.3,
            Self::UnderRoof => 0.2,
            Self::HeatedSpace => 0.0,
        }
    }

    fn ambient_c(self, outdoor_c: f64) -> f64 {
        match self {
            Self::Outdoors => outdoor_c,
            Self::InstallationRoom => 13.0,
            Self::UnderRoof => 5.0,
            Self::HeatedSpace => 20.0,
        }
    }
}

/// Table M.5 f_gen;env.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BurnerDraught {
    Atmospheric,
    FanAssisted,
}

impl BurnerDraught {
    fn envelope_share(self) -> f64 {
        match self {
            Self::Atmospheric => 0.50,
            Self::FanAssisted => 0.75,
        }
    }
}

/// Table M.7 f_ctr;ls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoilerControl {
    FloorStandingOutdoorCompensated,
    WallHungOutdoorCompensated,
    WallHungRoomTemperature,
}

impl BoilerControl {
    pub fn factor(self) -> f64 {
        match self {
            Self::FloorStandingOutdoorCompensated => 1.00,
            Self::WallHungOutdoorCompensated => 1.03,
            Self::WallHungRoomTemperature => 1.06,
        }
    }
}

/// An additional test at another mean water temperature (M.13/M.14).
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdditionalTest {
    pub efficiency: f64,
    pub test_temperature_c: f64,
}

/// Full-load efficiency on net calorific value (fractions, e.g. 0,92).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum FullLoadEfficiency {
    /// Non-condensing boilers: η_gen;Pn at ϑ_gen;test;Pn (M.7).
    Single {
        efficiency: f64,
        #[serde(default, rename = "testTemperatureC")]
        test_temperature_c: Option<f64>,
        #[serde(default, rename = "additionalTest")]
        additional_test: Option<AdditionalTest>,
    },
    /// Condensing boilers: η_gen;Pn;60 and η_gen;Pn;30 (M.8).
    Condensing {
        #[serde(rename = "efficiencyAt60")]
        efficiency_at_60: f64,
        #[serde(rename = "efficiencyAt30")]
        efficiency_at_30: f64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoilerProductValues {
    pub nominal_power_kw: f64,
    /// P_int; `None` gives 0,3·P_n (M.29).
    #[serde(default)]
    pub intermediate_power_kw: Option<f64>,
    pub full_load: FullLoadEfficiency,
    pub part_load_efficiency: f64,
    #[serde(default)]
    pub part_load_test_temperature_c: Option<f64>,
    #[serde(default)]
    pub part_load_additional_test: Option<AdditionalTest>,
    /// f_gen;ls;P0.
    pub standby_loss_factor: f64,
    /// ϑ_gen;test;P0.
    pub standby_test_temperature_c: f64,
    /// P_aux;P0, P_aux;Pint and P_aux;Pn in W.
    pub auxiliary_standby_w: f64,
    pub auxiliary_intermediate_w: f64,
    pub auxiliary_full_w: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductBoiler {
    pub technology: BoilerTechnology,
    pub fuel: BoilerFuel,
    pub placement: BoilerPlacement,
    pub draught: BurnerDraught,
    pub control: BoilerControl,
    pub product: BoilerProductValues,
    pub equipment_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnexMIssue {
    pub code: &'static str,
    pub path: String,
}

/// Monthly operating conditions from chapter 9.
#[derive(Debug, Clone, Copy)]
pub struct BoilerMonth {
    /// Q_H;gen;j;out, kWh.
    pub heat_output_kwh: f64,
    /// t_H;op;si;mi of table 9.15, h.
    pub operating_hours: f64,
    /// t_mi, h.
    pub month_hours: f64,
    /// ϑ_Hc;mn = ϑ_H,out (9.32), °C.
    pub return_temperature_c: f64,
    pub outdoor_temperature_c: f64,
    /// ϑ_brm (M.12) as ϑ_H,amb of 9.4.2, °C; absent means table M.6.
    pub ambient_temperature_c: Option<f64>,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoilerResult {
    pub load_ratio: f64,
    pub full_load_efficiency: f64,
    pub part_load_efficiency: f64,
    pub generation_loss_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    pub auxiliary_to_medium_kwh: f64,
    /// E_H;gen;in (M.1) in kWh on the gross calorific value.
    pub input_kwh: f64,
    /// M.16 + M.19, kWh.
    pub recoverable_to_space_kwh: f64,
}

fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn efficiency_ok(value: f64) -> bool {
    value.is_finite() && value > 0.3 && value < 1.2
}

pub fn validate_product_boiler(boiler: &ProductBoiler, path: &str) -> Vec<AnnexMIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: &str| {
        issues.push(AnnexMIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    let product = &boiler.product;
    if boiler.equipment_reference.trim().is_empty() {
        push("source_reference_required", "equipmentReference");
    }
    if product.source_reference.trim().is_empty() {
        push("source_reference_required", "product.sourceReference");
    }
    if !finite_positive(product.nominal_power_kw) {
        push("boiler_power_invalid", "product.nominalPowerKw");
    }
    if let Some(intermediate) = product.intermediate_power_kw {
        if !finite_positive(intermediate) || intermediate >= product.nominal_power_kw {
            push("boiler_power_invalid", "product.intermediatePowerKw");
        }
    }
    match (&product.full_load, boiler.technology.condensing()) {
        (
            FullLoadEfficiency::Single {
                efficiency,
                additional_test,
                ..
            },
            false,
        ) => {
            if !efficiency_ok(*efficiency) {
                push("boiler_efficiency_invalid", "product.fullLoad.efficiency");
            }
            if additional_test.is_some_and(|test| !efficiency_ok(test.efficiency)) {
                push(
                    "boiler_efficiency_invalid",
                    "product.fullLoad.additionalTest",
                );
            }
        }
        (
            FullLoadEfficiency::Condensing {
                efficiency_at_60,
                efficiency_at_30,
            },
            true,
        ) => {
            if !efficiency_ok(*efficiency_at_60) || !efficiency_ok(*efficiency_at_30) {
                push("boiler_efficiency_invalid", "product.fullLoad");
            }
        }
        _ => push(
            "boiler_full_load_method_mismatch",
            "product.fullLoad.method",
        ),
    }
    if !efficiency_ok(product.part_load_efficiency) {
        push("boiler_efficiency_invalid", "product.partLoadEfficiency");
    }
    if product
        .part_load_additional_test
        .is_some_and(|test| !efficiency_ok(test.efficiency))
    {
        push(
            "boiler_efficiency_invalid",
            "product.partLoadAdditionalTest",
        );
    }
    if !(product.standby_loss_factor.is_finite() && product.standby_loss_factor >= 0.0) {
        push("boiler_standby_loss_invalid", "product.standbyLossFactor");
    }
    if !(product.standby_test_temperature_c.is_finite()
        && product.standby_test_temperature_c > 20.0)
    {
        push(
            "boiler_standby_loss_invalid",
            "product.standbyTestTemperatureC",
        );
    }
    for (field, value) in [
        ("product.auxiliaryStandbyW", product.auxiliary_standby_w),
        (
            "product.auxiliaryIntermediateW",
            product.auxiliary_intermediate_w,
        ),
        ("product.auxiliaryFullW", product.auxiliary_full_w),
    ] {
        if !(value.is_finite() && value >= 0.0) {
            push("boiler_auxiliary_invalid", field);
        }
    }
    if boiler.fuel == BoilerFuel::Wood && boiler.technology != BoilerTechnology::SolidFuelStandard {
        push("boiler_technology_fuel_mismatch", "technology");
    }
    if boiler.fuel != BoilerFuel::Wood && boiler.technology == BoilerTechnology::SolidFuelStandard {
        push("boiler_technology_fuel_mismatch", "technology");
    }
    if boiler.fuel == BoilerFuel::Oil && boiler.technology == BoilerTechnology::CondensingGas
        || boiler.fuel == BoilerFuel::NaturalGas
            && boiler.technology == BoilerTechnology::CondensingOil
    {
        push("boiler_technology_fuel_mismatch", "technology");
    }
    issues
}

/// M.13/M.14: correction factor from an additional test, in 1/K.
fn test_correction(base: f64, base_temperature: f64, test: &AdditionalTest) -> f64 {
    (base - test.efficiency) / (test.test_temperature_c - base_temperature)
}

/// Loss at a load point (M.9/M.11), kW.
fn load_loss_kw(power_kw: f64, efficiency: f64, gross_to_net: f64) -> f64 {
    power_kw * (gross_to_net - PRACTICE_FACTOR * efficiency) / (PRACTICE_FACTOR * efficiency)
}

/// Linear interpolation by load ratio (M.4/M.5, M.21/M.22).
fn by_load(beta: f64, beta_int: f64, at_zero: f64, at_int: f64, at_full: f64) -> f64 {
    if beta <= beta_int {
        beta / beta_int * (at_int - at_zero) + at_zero
    } else {
        (beta - beta_int) / (1.0 - beta_int) * (at_full - at_int) + at_int
    }
}

/// Annex M for one month; call after [`validate_product_boiler`].
pub fn boiler_month(boiler: &ProductBoiler, month: BoilerMonth) -> BoilerResult {
    let product = &boiler.product;
    let pn = product.nominal_power_kw;
    let pint = product
        .intermediate_power_kw
        .unwrap_or(DEFAULT_INTERMEDIATE_RATIO * pn);
    let beta_int = pint / pn;
    let f_hs = boiler.fuel.gross_to_net();
    let mean = month.return_temperature_c;

    // M.24/M.25.
    let gen_out = boiler.control.factor() * month.heat_output_kwh;
    let beta = if month.operating_hours > 0.0 {
        (gen_out / (pn * month.operating_hours)).min(1.0)
    } else {
        0.0
    };

    // M.7/M.8 (with M.13).
    let (full, rated_full) = match &product.full_load {
        FullLoadEfficiency::Single {
            efficiency,
            test_temperature_c,
            additional_test,
        } => {
            let (default_test, default_factor) = boiler.technology.full_load_correction();
            let test = test_temperature_c.unwrap_or(default_test);
            let factor = additional_test.as_ref().map_or(default_factor, |extra| {
                test_correction(*efficiency, test, extra)
            });
            (efficiency + factor * (test - mean), *efficiency)
        }
        FullLoadEfficiency::Condensing {
            efficiency_at_60,
            efficiency_at_30,
        } => (
            efficiency_at_60
                - (efficiency_at_60 - efficiency_at_30) / (60.0 - 30.0) * (60.0 - mean),
            *efficiency_at_60,
        ),
    };
    // M.10 (with M.14).
    let (default_test, default_factor) = boiler.technology.part_load_correction();
    let part_test = product.part_load_test_temperature_c.unwrap_or(default_test);
    let part_factor = product
        .part_load_additional_test
        .as_ref()
        .map_or(default_factor, |extra| {
            test_correction(product.part_load_efficiency, part_test, extra)
        });
    let part = product.part_load_efficiency + part_factor * (part_test - mean);

    // M.9, M.11, M.12.
    let loss_full = load_loss_kw(pn, full, f_hs);
    let loss_int = load_loss_kw(pint, part, f_hs);
    let ambient = month
        .ambient_temperature_c
        .unwrap_or_else(|| boiler.placement.ambient_c(month.outdoor_temperature_c));
    let ratio = ((mean - ambient) / (product.standby_test_temperature_c - 20.0)).max(0.0);
    let loss_standby =
        pn / (PRACTICE_FACTOR * rated_full) * product.standby_loss_factor * f_hs * ratio.powf(1.25);

    // M.4–M.6.
    let loss_at_load = by_load(beta, beta_int, loss_standby, loss_int, loss_full);
    let standby_hours = (month.month_hours - month.operating_hours).max(0.0);
    let generation_loss = loss_at_load * month.operating_hours + loss_standby * standby_hours;

    // M.21–M.23 (W → kW).
    let aux_at_load = by_load(
        beta,
        beta_int,
        product.auxiliary_standby_w,
        product.auxiliary_intermediate_w,
        product.auxiliary_full_w,
    ) / 1000.0;
    let auxiliary =
        aux_at_load * month.operating_hours + product.auxiliary_standby_w / 1000.0 * standby_hours;
    let to_medium = auxiliary * AUX_TO_MEDIUM_FRACTION;

    // M.16, M.17, M.19.
    let f_brm = boiler.placement.reduction();
    let envelope =
        loss_standby * (1.0 - f_brm) * boiler.draught.envelope_share() * month.operating_hours;
    let aux_to_space = auxiliary * (1.0 - f_brm) * (1.0 - AUX_TO_MEDIUM_FRACTION);

    BoilerResult {
        load_ratio: beta,
        full_load_efficiency: full,
        part_load_efficiency: part,
        generation_loss_kwh: generation_loss,
        auxiliary_electricity_kwh: auxiliary,
        auxiliary_to_medium_kwh: to_medium,
        // M.1 with Q_gen;ren = 0 for boilers.
        input_kwh: (gen_out - to_medium + generation_loss).max(0.0),
        recoverable_to_space_kwh: envelope + aux_to_space,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn condensing() -> ProductBoiler {
        ProductBoiler {
            technology: BoilerTechnology::CondensingGas,
            fuel: BoilerFuel::NaturalGas,
            placement: BoilerPlacement::HeatedSpace,
            draught: BurnerDraught::FanAssisted,
            control: BoilerControl::WallHungOutdoorCompensated,
            product: BoilerProductValues {
                nominal_power_kw: 24.0,
                intermediate_power_kw: None,
                full_load: FullLoadEfficiency::Condensing {
                    efficiency_at_60: 0.97,
                    efficiency_at_30: 1.06,
                },
                part_load_efficiency: 1.08,
                part_load_test_temperature_c: None,
                part_load_additional_test: None,
                standby_loss_factor: 0.006,
                standby_test_temperature_c: 50.0,
                auxiliary_standby_w: 3.0,
                auxiliary_intermediate_w: 15.0,
                auxiliary_full_w: 40.0,
                source_reference: "product sheet".into(),
            },
            equipment_reference: "boiler".into(),
        }
    }

    #[test]
    fn condensing_boiler_follows_m1_to_m25() {
        let boiler = condensing();
        assert!(validate_product_boiler(&boiler, "b").is_empty());
        let month = BoilerMonth {
            heat_output_kwh: 1000.0,
            operating_hours: 600.0,
            month_hours: 744.0,
            return_temperature_c: 35.0,
            outdoor_temperature_c: 2.61,
            ambient_temperature_c: None,
        };
        let result = boiler_month(&boiler, month);
        let out = 1.03 * 1000.0;
        let beta = out / (24.0 * 600.0);
        assert!((result.load_ratio - beta).abs() < 1e-12);
        let full = 0.97 - (0.97 - 1.06) / 30.0 * (60.0 - 35.0);
        let part = 1.08 + 0.002 * (30.0 - 35.0);
        assert!((result.full_load_efficiency - full).abs() < 1e-12);
        assert!((result.part_load_efficiency - part).abs() < 1e-12);
        let pint = 7.2;
        let loss_int = pint * (1.11 - 0.95 * part) / (0.95 * part);
        let loss_p0 = 24.0 / (0.95 * 0.97) * 0.006 * 1.11 * ((35.0 - 20.0) / 30.0_f64).powf(1.25);
        // β < 0,3: interpolation between stand-by and P_int (M.4).
        let loss_x = beta / 0.3 * (loss_int - loss_p0) + loss_p0;
        let loss = loss_x * 600.0 + loss_p0 * 144.0;
        assert!((result.generation_loss_kwh - loss).abs() < 1e-9);
        let aux_x = (beta / 0.3 * (15.0 - 3.0) + 3.0) / 1000.0;
        let aux = aux_x * 600.0 + 0.003 * 144.0;
        assert!((result.auxiliary_electricity_kwh - aux).abs() < 1e-12);
        assert!((result.input_kwh - (out - 0.75 * aux + loss)).abs() < 1e-9);
        // Heated space: f_brm = 0.
        let envelope = loss_p0 * 0.75 * 600.0;
        assert!((result.recoverable_to_space_kwh - (envelope + 0.25 * aux)).abs() < 1e-9);
    }

    #[test]
    fn single_efficiency_boiler_uses_table_m2_or_an_additional_test() {
        let mut boiler = condensing();
        boiler.technology = BoilerTechnology::LowTemperature;
        boiler.product.full_load = FullLoadEfficiency::Single {
            efficiency: 0.91,
            test_temperature_c: None,
            additional_test: None,
        };
        boiler.product.part_load_efficiency = 0.93;
        assert!(validate_product_boiler(&boiler, "b").is_empty());
        let month = BoilerMonth {
            heat_output_kwh: 2000.0,
            operating_hours: 500.0,
            month_hours: 720.0,
            return_temperature_c: 45.0,
            outdoor_temperature_c: 9.32,
            ambient_temperature_c: None,
        };
        let result = boiler_month(&boiler, month);
        assert!((result.full_load_efficiency - (0.91 + 0.0004 * 25.0)).abs() < 1e-12);
        assert!((result.part_load_efficiency - (0.93 + 0.0004 * -5.0)).abs() < 1e-12);
        boiler.product.full_load = FullLoadEfficiency::Single {
            efficiency: 0.91,
            test_temperature_c: Some(70.0),
            additional_test: Some(AdditionalTest {
                efficiency: 0.93,
                test_temperature_c: 50.0,
            }),
        };
        let tested = boiler_month(&boiler, month);
        // M.13: (0,91 − 0,93)/(50 − 70) = 0,001 per K.
        assert!((tested.full_load_efficiency - (0.91 + 0.001 * 25.0)).abs() < 1e-12);
    }

    #[test]
    fn validation_rejects_mismatched_technology_and_method() {
        let mut boiler = condensing();
        boiler.fuel = BoilerFuel::Oil;
        boiler.product.full_load = FullLoadEfficiency::Single {
            efficiency: 0.9,
            test_temperature_c: None,
            additional_test: None,
        };
        let codes: Vec<_> = validate_product_boiler(&boiler, "b")
            .into_iter()
            .map(|item| item.code)
            .collect();
        assert!(codes.contains(&"boiler_full_load_method_mismatch"));
        assert!(codes.contains(&"boiler_technology_fuel_mismatch"));
    }
}
