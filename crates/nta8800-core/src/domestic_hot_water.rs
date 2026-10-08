//! Domestic hot water, NTA 8800:2025+C1:2026 chapter 13 (pages 525–655),
//! for one hot-water system serving all zones.
//!
//! Chain per month (13.3–13.9):
//! - net need 13.15–13.18 (856 kWh per occupant) or table 13.1 (13.19);
//! - shower heat recovery 13.51/13.52 with tables 13.7/13.8 and 13.53;
//! - `Q_W;em = Q_W;nd / η_W;em − Q_W;rcd` (13.9) with tables 13.2/13.3 and
//!   13.21–13.23;
//! - circulation loss 13.26 with table 13.4/13.29, `L = 0,3·A_red + 10`
//!   (13.31), pump energy 13.34–13.44 with table 13.6 and recovered pump heat
//!   13.50, distribution efficiency 13.25;
//! - storage loss 13.58/13.59 with table 13.9 (§13.6.3 defaults);
//! - delivery-set conversion loss 13.24/13.24a and its electronics 13.46;
//! - `Q_W;dis = Q_W;em/η_W;dis + Q_W;sto;ls;tot + Q_W;dis;conv;ls` (13.7)
//!   with the backup losses of solar vessels (13.8);
//! - solar water heaters and combi systems first (§13.7, 13.4/13.4a,
//!   13.66–13.68; see [`crate::solar_thermal`]);
//! - several generators (13.8.2): order 13.8.2.1, maximum output 13.141–
//!   13.142 with the series cases 13.141a–d, the exhaust-air limit 13.144a,
//!   the cascade 13.143a/13.145 and an extra electric instantaneous heater
//!   for an uncovered rest, shares 13.150;
//! - `E_W = Q_W;dis;nren · F_W;gen / (f_prac · η_W;gen)` (13.3, 13.152) per
//!   generator and carrier, with tables 13.25–13.28 and 13.18, the gas
//!   storage heater 13.165–13.175, booster heat pumps (13.8.4.4, annex W)
//!   and generator auxiliaries 13.181;
//! - ambient heat of heat pumps 5.36/5.37;
//! - quality-statement contributions `F_W;gen;gi` (13.146), delivered first;
//! - appliances tested at two tapping profiles (13.8.4.2: 13.153a–13.160,
//!   recoverable losses 13.160a), with the monthly mixed-air correction
//!   13.153b/d–i and the PFHRD contribution 13.156a/b from
//!   [`HotWaterExtras`]; below the lower limit of 13.154 heat pumps fall
//!   back to 13.160b with table 13.18;
//! - hot water from the space-heating system (13.8.4.9.3, 13.185): no
//!   carrier here, the output loads the space-heating node;
//! - 13.149 time fraction of an exhaust-air heat pump for 13.148/11.2.2.1.2;
//! - circulation (13.26) and vessel losses (13.58) with the levelled
//!   `ϑ_int;set;H;zi,mi` of 7.9.4 when known;
//! - standalone space-heating solar systems (SHS) via
//!   [`assess_standalone_solar`].
//!
//! Not modelled: series of more than two generators (not allowed by
//! 13.141b). The winter test method 13.153 is offered as
//! [`winter_gas_consumption_kwh`]; its combination into `Q_gas;p(i)` follows
//! NEN-EN 13203-2. The recoverable losses of 13.13 (13.47, 13.49, 13.63,
//! 13.68, 13.160a, 13.164, 13.179) are reported per month. Annex T and U
//! test reports are evaluated in [`crate::hot_water_tests`].

use crate::annex_w::{
    calculate_booster, calculate_booster_forfait, validate_booster, validate_booster_forfait,
    BoosterForfait, BoosterHeatPump, BoosterHeatSource, BoosterMonth, BoosterSourceCarrier,
};
use crate::building_performance::Carrier;
use crate::climate::{MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use crate::hot_water_tests::{AnnexTTest, AnnexUTest};
use crate::label_class::LabelFunction;
use crate::monthly_demand::occupants_per_dwelling;
use crate::solar_thermal::{
    calculated_service, heating_reference_c, tested_water, validate_solar, water_reference_c,
    ServiceMonth, ServiceSettings, SolarMethod, SolarUse, SolarWaterHeater, COLD_WATER_C,
    PRACTICE_FACTOR as SOLAR_PRACTICE_FACTOR, PUMP_HOURS_COMBI, PUMP_HOURS_WATER,
};
use serde::{Deserialize, Serialize};

/// §13.2.3.1: residential need per occupant, kWh per year.
pub const RESIDENTIAL_NEED_PER_OCCUPANT_KWH: f64 = 856.0;
/// §13.2.3.1: bathroom and kitchen shares.
pub const BATHROOM_SHARE: f64 = 0.8;
pub const KITCHEN_SHARE: f64 = 0.2;
/// 13.51: practical factor and temperature correction.
pub const SHOWER_PRACTICAL_FACTOR: f64 = 0.95;
pub const SHOWER_TEMPERATURE_FACTOR: f64 = 0.83;
/// 13.26: mean water temperature of a hot-water circulation, °C.
pub const CIRCULATION_MEAN_C: f64 = 62.5;
/// 13.26 / 13.58: unheated-space temperature without an AOR, °C.
pub const UNHEATED_AMBIENT_C: f64 = 13.0;
/// 13.24: standby loss of an individual delivery set, W.
pub const DELIVERY_SET_STANDBY_W: f64 = 30.0;
/// 13.24a: conversion efficiency for utility buildings.
pub const DELIVERY_SET_EFFICIENCY: f64 = 0.75;
/// 13.46 / 13.181: standby electronics, W.
pub const STANDBY_ELECTRONICS_W: f64 = 10.0;
/// 13.49/13.50: recoverable share of circulation pump energy.
pub const PUMP_RECOVERABLE_FACTOR: f64 = 0.20;

/// Table 13.1 `Q_W;nd;spec`, kWh/m² per year.
pub fn utility_specific_need(function: LabelFunction) -> Option<f64> {
    Some(match function {
        LabelFunction::Residential => return None,
        LabelFunction::AssemblyWithDayCare
        | LabelFunction::AssemblyWithoutDayCare
        | LabelFunction::HealthcareWithoutBeds => 2.8,
        LabelFunction::Cell => 4.2,
        LabelFunction::HealthcareWithBeds => 15.3,
        LabelFunction::Office | LabelFunction::Education | LabelFunction::Retail => 1.4,
        LabelFunction::Lodging | LabelFunction::Sport => 12.5,
    })
}

/// Table 13.7 `C_W;nd;sh`.
pub fn shower_share(function: LabelFunction) -> f64 {
    match function {
        LabelFunction::Residential | LabelFunction::Sport => 0.8,
        LabelFunction::Cell
        | LabelFunction::HealthcareWithBeds
        | LabelFunction::HealthcareWithoutBeds => 0.4,
        LabelFunction::Lodging => 0.6,
        LabelFunction::AssemblyWithDayCare
        | LabelFunction::AssemblyWithoutDayCare
        | LabelFunction::Office
        | LabelFunction::Education
        | LabelFunction::Retail => 0.0,
    }
}

/// Table 13.2 band for a draw-off length.
fn length_band(length_m: f64) -> usize {
    ((length_m / 2.0).floor() as usize).min(7)
}

/// Table 13.2 `η_W;em;k`.
pub fn kitchen_emission(length_m: f64) -> f64 {
    [1.00, 0.69, 0.53, 0.43, 0.36, 0.31, 0.27, 0.24][length_band(length_m)]
}

/// Table 13.2 of NTA 8800:2023 (p. 533): kitchen row by the inner diameter
/// over at least two thirds of the pipe length. From 2024 (p. 527) only the
/// "overig" row exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KitchenPipeDiameter {
    #[serde(rename = "up_to_8_mm")]
    UpTo8Mm,
    #[serde(rename = "up_to_10_mm")]
    UpTo10Mm,
    /// "Overig", or unknown.
    Other,
}

/// Table 13.2 `η_W;em;k` with the 2023 diameter rows where the active
/// edition has them.
pub fn kitchen_emission_for(length_m: f64, diameter: Option<KitchenPipeDiameter>) -> f64 {
    let band = length_band(length_m);
    match diameter {
        Some(KitchenPipeDiameter::UpTo8Mm)
            if crate::norm_versions::profile().kitchen_diameter_rows =>
        {
            [1.00, 0.86, 0.75, 0.67, 0.60, 0.55, 0.50, 0.46][band]
        }
        Some(KitchenPipeDiameter::UpTo10Mm)
            if crate::norm_versions::profile().kitchen_diameter_rows =>
        {
            [1.00, 0.79, 0.65, 0.55, 0.48, 0.43, 0.38, 0.35][band]
        }
        _ => kitchen_emission(length_m),
    }
}

/// Table 13.2 `η_W;em;b`.
pub fn bathroom_emission(length_m: f64) -> f64 {
    [1.00, 0.95, 0.90, 0.86, 0.82, 0.78, 0.75, 0.72][length_band(length_m)]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UtilityArea {
    pub function: LabelFunction,
    pub area_m2: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum HotWaterNeed {
    /// 13.15–13.18.
    Residential {
        #[serde(rename = "dwellingCount")]
        dwelling_count: u32,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 13.19 with table 13.1 per use function (areas sum to `A_g`).
    Utility {
        areas: Vec<UtilityArea>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

/// Which taps the system serves (13.21–13.23).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServedTaps {
    KitchenAndBathroom,
    BathroomOnly,
    KitchenOnly,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum HotWaterEmission {
    /// Table 13.2: mean shortest horizontal plus vertical distance.
    Residential {
        served: ServedTaps,
        #[serde(default, rename = "kitchenLengthM")]
        kitchen_length_m: Option<f64>,
        /// Table 13.2 of NTA 8800:2023 only: inner diameter of the kitchen
        /// draw-off pipe over at least two thirds of its length.
        #[serde(
            default,
            rename = "kitchenPipeDiameter",
            skip_serializing_if = "Option::is_none"
        )]
        kitchen_pipe_diameter: Option<KitchenPipeDiameter>,
        #[serde(default, rename = "bathroomLengthM")]
        bathroom_length_m: Option<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Table 13.3: mean draw-off length.
    Utility {
        #[serde(rename = "meanLengthM")]
        mean_length_m: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

/// Table 13.4 insulation thickness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipeInsulation {
    None,
    Unknown,
    Mm10,
    Mm15,
    Mm20,
    Mm25,
}

/// Table 13.4 Ψ for copper pipes; the nearest diameter, the smaller
/// insulation and the larger diameter on a tie (p. 552).
pub fn table_13_4_psi(outer_diameter_mm: f64, insulation: PipeInsulation) -> f64 {
    const DIAMETERS: [f64; 10] = [10.0, 12.0, 15.0, 22.0, 28.0, 35.0, 42.0, 54.0, 67.0, 80.0];
    const ROWS: [[f64; 6]; 10] = [
        [0.407, 0.165, 0.165, 0.136, 0.114, 0.106],
        [0.453, 0.184, 0.184, 0.154, 0.136, 0.124],
        [0.539, 0.211, 0.211, 0.174, 0.154, 0.138],
        [0.728, 0.271, 0.271, 0.219, 0.189, 0.169],
        [0.880, 0.321, 0.321, 0.256, 0.219, 0.194],
        [1.049, 0.378, 0.378, 0.299, 0.253, 0.223],
        [1.211, 0.435, 0.435, 0.341, 0.287, 0.251],
        [1.477, 0.531, 0.531, 0.412, 0.343, 0.299],
        [1.753, 0.635, 0.635, 0.488, 0.404, 0.349],
        [2.0, 0.74, 0.74, 0.56, 0.46, 0.4],
    ];
    let mut best = 0;
    for (index, diameter) in DIAMETERS.iter().enumerate() {
        let distance = (diameter - outer_diameter_mm).abs();
        let current = (DIAMETERS[best] - outer_diameter_mm).abs();
        if distance <= current {
            best = index;
        }
    }
    let column = match insulation {
        PipeInsulation::None => 0,
        PipeInsulation::Unknown => 1,
        PipeInsulation::Mm10 => 2,
        PipeInsulation::Mm15 => 3,
        PipeInsulation::Mm20 => 4,
        PipeInsulation::Mm25 => 5,
    };
    ROWS[best][column]
}

/// NTA 8800:2023 table 13.4 (p. 542) rows for an unknown diameter by type
/// of system, "klein" or "overig".
pub fn table_13_4_system_psi(small: bool, insulation: PipeInsulation) -> f64 {
    const SMALL: [f64; 6] = [1.0, 0.4, 0.4, 0.3, 0.25, 0.2];
    const OTHER: [f64; 6] = [2.0, 0.74, 0.74, 0.56, 0.46, 0.4];
    let column = match insulation {
        PipeInsulation::None => 0,
        PipeInsulation::Unknown => 1,
        PipeInsulation::Mm10 => 2,
        PipeInsulation::Mm15 => 3,
        PipeInsulation::Mm20 => 4,
        PipeInsulation::Mm25 => 5,
    };
    if small {
        SMALL[column]
    } else {
        OTHER[column]
    }
}

/// Table 13.6 pump control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PumpControl {
    UncontrolledOrUnknown,
    ConstantPressure,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CirculationPump {
    pub control: PumpControl,
    /// 13.39: power on the energy label, kW.
    #[serde(default)]
    pub label_power_kw: Option<f64>,
    /// EEI per EU 622/2012; default 0,23 (0,25 from 2,5 kW).
    #[serde(default)]
    pub energy_efficiency_index: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Circulation {
    /// Outer diameter; omitted means table 13.29 (dwellings) or 35/80 mm.
    #[serde(default)]
    pub outer_diameter_mm: Option<f64>,
    pub insulation: PipeInsulation,
    /// Declared Ψ (13.27–13.29) instead of table 13.4.
    #[serde(default)]
    pub declared_psi_w_per_mk: Option<f64>,
    /// Ψ calculated from the pipe geometry with 13.27–13.29 (2025+C1
    /// p. 551), e.g. a pipe embedded in the construction (13.28); exclusive
    /// with `declaredPsiWPerMK`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calculated_psi: Option<crate::heating_distribution::PipeGeometry>,
    pub fittings_insulated: bool,
    /// Actual length; omitted means 13.31.
    #[serde(default)]
    pub length_m: Option<f64>,
    /// Length in unheated spaces; omitted means 15 %.
    #[serde(default)]
    pub unheated_length_m: Option<f64>,
    /// `ϑ_ztu` of the unheated space; omitted means 13 °C.
    #[serde(default)]
    pub unheated_ambient_c: Option<f64>,
    /// `n_si` of 13.32, at least 1.
    pub floor_count: u32,
    /// Sport and swimming halls `A_g;si;sport` (13.32a), m².
    #[serde(default)]
    pub sport_hall_area_m2: f64,
    /// Dwellings on the system for table 13.29.
    #[serde(default)]
    pub connected_dwellings: Option<u32>,
    pub pump: CirculationPump,
    pub source_reference: String,
}

/// Table 13.9 label classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageLabel {
    APlus,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl StorageLabel {
    /// Table 13.9 `S_sto;ls` in W for volume V in litres.
    pub fn standing_loss_w(self, volume_l: f64) -> f64 {
        let (a, b) = match self {
            Self::APlus => (5.5, 3.16),
            Self::A => (7.0, 3.7),
            Self::B => (10.25, 5.09),
            Self::C => (14.33, 7.13),
            Self::D => (18.83, 9.33),
            Self::E => (23.5, 11.99),
            Self::F => (28.5, 15.16),
            Self::G => (31.0, 16.66),
        };
        a + b * volume_l.powf(0.4)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum StorageLoss {
    /// 13.59 with a CDR 812/2013 label.
    Label { label: StorageLabel },
    /// 13.59 without a known label: C from 2018, else G (p. 569).
    UnknownLabel {
        #[serde(rename = "producedFrom2018")]
        produced_from_2018: bool,
    },
    /// 13.58 with a measured `H_sto;ls` (13.60), W/K; `f_sto;dis;ls = 1`;
    /// rounded up per annex X (p. 568).
    Measured {
        #[serde(rename = "transmissionWPerK")]
        transmission_w_per_k: f64,
    },
    /// 13.58 with `H_sto;ls` derived from the standby test by 13.60 and
    /// rounded up per annex X; `f_sto;dis;ls = 1`.
    MeasuredStandby {
        /// `Q_stb;ls;ref`, kWh per 24 h.
        #[serde(rename = "standbyKwhPerDay")]
        standby_kwh_per_day: f64,
        /// `ϑ_sto;set;ref`, °C.
        #[serde(rename = "referenceStorageC")]
        reference_storage_c: f64,
        /// `ϑ_amb;ref`, °C.
        #[serde(rename = "referenceAmbientC")]
        reference_ambient_c: f64,
    },
}

impl StorageLoss {
    /// `H_sto;ls` of a measured vessel after annex X, W/K; `None` for a
    /// label route.
    pub fn measured_transmission_w_per_k(&self) -> Option<f64> {
        let raw = match *self {
            Self::Measured {
                transmission_w_per_k,
            } => transmission_w_per_k,
            // 13.60.
            Self::MeasuredStandby {
                standby_kwh_per_day,
                reference_storage_c,
                reference_ambient_c,
            } => {
                1000.0 * standby_kwh_per_day / (24.0 * (reference_storage_c - reference_ambient_c))
            }
            _ => return None,
        };
        if !(raw.is_finite() && raw > 0.0) {
            // Rejected by validation; never a silent infinite or negative loss.
            return None;
        }
        // Annex X (X.2): the next higher table X.1 value.
        Some(crate::significant_figures::round_up(raw))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageVessel {
    pub id: String,
    pub volume_l: f64,
    pub loss: StorageLoss,
    /// `f_sto;dis;ls` 1–5 (§13.6.3); ignored for a measured `H_sto;ls`.
    pub connection_factor: u8,
    /// NTA 8800:2022 §13.6.3 (p. 550): an electric boiler with insulated
    /// hot-water pipes, `f_sto;dis;ls` = 1,5 instead of `connectionFactor`.
    /// Not in 2023 (p. 557–558) and later.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub electric_boiler_insulated_pipe: bool,
    /// Placed in a heated zone; otherwise 13 °C or the given ambient.
    pub in_heated_zone: bool,
    #[serde(default)]
    pub unheated_ambient_c: Option<f64>,
    /// Note 1 of §13.6.2 (p. 566): the appliance of 13.8.4.2/13.8.4.3 was
    /// tested without this vessel, so its loss is calculated.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub not_in_appliance_test: bool,
    pub source_reference: String,
}

/// Table 13.8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowerConnection {
    MixerAndHeater,
    MixerOnly,
    HeaterOnly,
    SharedUnits,
    Unknown,
}

impl ShowerConnection {
    pub fn factor(self) -> f64 {
        match self {
            Self::MixerAndHeater => 1.0,
            Self::MixerOnly => 0.85,
            Self::HeaterOnly | Self::SharedUnits | Self::Unknown => 0.75,
        }
    }
}

/// Per shower: its recovery unit (§13.5.3).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "unit", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShowerUnit {
    None,
    Vertical,
    Horizontal,
    Unknown,
    /// Annex U at the application class of the heater (class 4 utility).
    Declared {
        efficiency: f64,
        /// Table U.1 class of the declared value; checked when given.
        #[serde(default, rename = "testClass", skip_serializing_if = "Option::is_none")]
        test_class: Option<crate::hot_water_tests::ShowerTestClass>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Annex U test report: mean of three runs, rounded down to 0,025.
    AnnexU {
        test: AnnexUTest,
    },
}

impl ShowerUnit {
    pub fn efficiency(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Vertical => 0.40,
            Self::Horizontal | Self::Unknown => 0.20,
            // Annex U: rounded down to a multiple of 0,025.
            Self::Declared { efficiency, .. } => round_down(*efficiency, 0.025),
            Self::AnnexU { test } => test.efficiency().unwrap_or(0.0),
        }
    }

    fn test_class(&self) -> Option<crate::hot_water_tests::ShowerTestClass> {
        match self {
            Self::Declared { test_class, .. } => *test_class,
            Self::AnnexU { test } => Some(test.class),
            _ => None,
        }
    }
}

/// §13.5.3: the annex U class to use. Dwellings use the application class
/// of the hot-water appliance (class 4 when unknown), utility buildings
/// class 4. Table U.1 has no class 1 row; a class 1 appliance uses the
/// class 2 measurement (interpretation, the nearest lower demand).
fn required_shower_class(
    system: &HotWaterSystem,
    residential: bool,
) -> crate::hot_water_tests::ShowerTestClass {
    use crate::hot_water_tests::ShowerTestClass;
    if !residential {
        return ShowerTestClass::Class4;
    }
    let class = match &system.generator {
        HotWaterGenerator::GasAppliance { measured_class, .. }
        | HotWaterGenerator::HeatPump { measured_class, .. } => *measured_class,
        _ => None,
    };
    match class.unwrap_or(ApplicationClass::Class4) {
        ApplicationClass::Class1 | ApplicationClass::Class2 => ShowerTestClass::Class2,
        ApplicationClass::Class3 => ShowerTestClass::Class3,
        ApplicationClass::Class4 => ShowerTestClass::Class4,
    }
}

/// §13.8.4.3 conditions for the annex T (NEN 7120 annex A) method.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnnexTConditions {
    /// The appliance type was already supplied before 2021.
    pub type_supplied_before_2021: bool,
    /// The appliance stands indoors.
    pub appliance_indoors: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShowerHeatRecovery {
    /// One entry per shower of the system (13.53); `none` for a shower
    /// without a unit.
    pub showers: Vec<ShowerUnit>,
    /// Utility buildings (p. 564): it is not known which shower is on which
    /// unit; with more than 80 % connected the lowest efficiency applies.
    #[serde(default)]
    pub assignment_unknown: bool,
    pub connection: ShowerConnection,
    pub source_reference: String,
}

/// Table 13.23 application class at which a heater was measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationClass {
    Class1,
    Class2,
    Class3,
    Class4,
}

/// Table 13.18 European tapping profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TappingProfile {
    S,
    M,
    L,
    Xl,
}

/// Table 13.25 gas appliances up to 70 kW and 300 l.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GasAppliance {
    WithoutGaskeur,
    WaterHeaterGaskeur,
    WaterHeaterGaskeurCw,
    KitchenGeyser,
    CombiGaskeur,
    CombiGaskeurHrCw,
    Unknown,
}

impl GasAppliance {
    /// Table 13.25 value and whether `c_W;gen` applies.
    fn table(self) -> (f64, bool) {
        match self {
            Self::WithoutGaskeur | Self::Unknown => (0.30, false),
            Self::WaterHeaterGaskeur => (0.40, true),
            Self::WaterHeaterGaskeurCw => (0.625, true),
            Self::KitchenGeyser | Self::CombiGaskeur => (0.50, true),
            Self::CombiGaskeurHrCw => (0.675, true),
        }
    }
}

/// Table 13.28 boiler class for indirectly heated storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndirectBoiler {
    ConventionalOrUnknown,
    Vr,
    Hr100Or104,
    Hr107,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredEfficiency {
    pub value: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HotWaterGenerator {
    /// Table 13.25 gas appliance with `c_W;gen` of table 13.26.
    GasAppliance {
        appliance: GasAppliance,
        /// Measured class; omitted means class 4.
        #[serde(default, rename = "measuredClass")]
        measured_class: Option<ApplicationClass>,
        /// Used for the kitchen only (`c_W;gen = 1`).
        #[serde(default, rename = "kitchenOnly")]
        kitchen_only: bool,
        /// A quality-statement value replacing the table value.
        #[serde(default)]
        declared: Option<DeclaredEfficiency>,
        /// Annex T test report replacing the table value (Gaskeur coupling).
        #[serde(default, rename = "annexT", skip_serializing_if = "Option::is_none")]
        annex_t: Option<AnnexTTest>,
        /// §13.8.4.3 conditions; required with `annexT`.
        #[serde(
            default,
            rename = "annexTConditions",
            skip_serializing_if = "Option::is_none"
        )]
        annex_t_conditions: Option<AnnexTConditions>,
    },
    /// Table 13.25 individual heat pump, 1,4·c_source with table 13.27.
    HeatPump {
        #[serde(rename = "exhaustAirSource")]
        exhaust_air_source: bool,
        /// Annex V `c_source`; omitted means 1,0.
        #[serde(default, rename = "sourceCorrection")]
        source_correction: Option<f64>,
        #[serde(default, rename = "measuredClass")]
        measured_class: Option<ApplicationClass>,
        /// 5.37: outdoor-air share of a partly exhaust-air source.
        #[serde(default, rename = "outdoorAirFraction")]
        outdoor_air_fraction: Option<f64>,
        /// Annex V (V.1, table V.1): on the same regenerated ground source
        /// as the space-heating heat pump. Its hot water joins the V.1
        /// denominator and it takes that c_source.
        #[serde(default, rename = "sameGroundSource", skip_serializing_if = "is_false")]
        same_ground_source: bool,
        /// A quality declaration replacing `1,4·c_source` of table 13.25
        /// (§13.8.4.7.2, p. 640): rounded down to a multiple of 0,05; the
        /// `c_W;gen` class correction applies for its measured class.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        declared: Option<DeclaredEfficiency>,
    },
    /// 13.160b: EN 16147 test at one European tapping profile.
    HeatPumpEn16147 {
        profile: TappingProfile,
        #[serde(rename = "deliveredKwhPerDay")]
        delivered_kwh_per_day: f64,
        #[serde(rename = "inputKwhPerDay")]
        input_kwh_per_day: f64,
        #[serde(rename = "exhaustAirSource")]
        exhaust_air_source: bool,
        /// Storage appliance tested without weekly legionella prevention
        /// (`f_prac = 0,9`).
        #[serde(rename = "storageWithoutLegionellaCycle")]
        storage_without_legionella_cycle: bool,
        #[serde(default, rename = "outdoorAirFraction")]
        outdoor_air_fraction: Option<f64>,
        /// NEN-EN 16147 `SCF`; `smart = 1` from 0,07 (13.153b).
        #[serde(
            default,
            rename = "smartControlFactor",
            skip_serializing_if = "Option::is_none"
        )]
        smart_control_factor: Option<f64>,
        /// `T_max;test;i` (13.153c), °C; omitted means 55 °C (no
        /// correction).
        #[serde(
            default,
            rename = "maxTestTemperatureC",
            skip_serializing_if = "Option::is_none"
        )]
        max_test_temperature_c: Option<f64>,
        /// `T_set;design` (13.153c), default 55 °C; a lower value needs
        /// the appliance and installation design as evidence.
        #[serde(
            default,
            rename = "designSetTemperatureC",
            skip_serializing_if = "Option::is_none"
        )]
        design_set_temperature_c: Option<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Table 13.25 electric instantaneous heater, 0,95.
    ElectricInstantaneous,
    /// §13.8.4.5.3 electric storage heater, 1,0; storage loss per §13.6.
    ElectricBoiler,
    /// 13.165–13.175 gas storage heater up to 150 kW.
    GasStorageHeater {
        #[serde(rename = "volumeL")]
        volume_l: f64,
        /// `q_B;S` measured per NEN-EN 89, kWh/day.
        #[serde(default, rename = "measuredStandbyKwhPerDay")]
        measured_standby_kwh_per_day: Option<f64>,
        #[serde(rename = "before1985")]
        before_1985: bool,
        #[serde(rename = "inHeatedZone")]
        in_heated_zone: bool,
    },
    /// §13.8.4.5.4 other directly heated storage (gas > 150 kW), 0,50.
    LargeDirectStorage {
        #[serde(rename = "gasFired")]
        gas_fired: bool,
    },
    /// Table 13.28 boiler with an indirectly heated vessel.
    IndirectBoiler {
        boiler: IndirectBoiler,
        oil: bool,
        #[serde(rename = "insideBoundary")]
        inside_boundary: bool,
        #[serde(rename = "alsoSpaceHeating")]
        also_space_heating: bool,
        #[serde(default)]
        declared: Option<DeclaredEfficiency>,
        /// §13.8.4.7.4: a gas pilot flame (695 kWh/year, 9.6.2.1), counted
        /// only when the boiler does not also heat the building.
        #[serde(default, rename = "pilotFlame")]
        pilot_flame: bool,
    },
    /// §13.8.4.6 table 13.22: solid-biomass combi appliance with a vessel
    /// meeting annex R; the efficiency includes the vessel losses, W_aux 0
    /// and no recoverable loss.
    BiomassCombi {
        insulation: BiomassStoreInsulation,
        #[serde(rename = "insideBoundary")]
        inside_boundary: bool,
    },
    /// §13.8.4.10: two or more electric heat pumps in series as one
    /// notional device over the whole rise (f_pref;serie = 1), with table
    /// 9.29 at 65 °C < θ_sup ≤ 70 °C for the source of the last heat pump.
    HeatPumpSeries {
        #[serde(rename = "lastSource")]
        last_source: crate::forfait_heat_pump_draft::TableSource,
    },
    /// §13.8.4.7.4 heat pump with an indirectly heated vessel, 1,4.
    IndirectHeatPump {
        #[serde(rename = "alsoSpaceHeating")]
        also_space_heating: bool,
    },
    /// §13.8.4.9.2 external heat delivery, η 1,0 (forfait f_P).
    ExternalHeat,
    /// Annex W booster heat pump on a collective heating system: source
    /// heat from that system plus electricity.
    BoosterHeatPump(Box<BoosterHeatPump>),
    /// §13.8.4.4 booster heat pump with the forfait values of 13.162/13.163
    /// instead of annex W measurements.
    BoosterHeatPumpForfait(Box<BoosterForfait>),
    /// §13.8.4.2: tested with 24-hour measurements at two tapping
    /// profiles (NEN-EN 13203-2 or NEN-EN 16147), 13.153a–13.160a.
    MeasuredTwoProfiles(Box<TwoProfileTest>),
    /// §13.8.4.7.4/§13.8.4.8 building CHP with an indirectly heated vessel:
    /// method 2 (table 9.31, HT column) or method 1 (micro-CHP per
    /// NEN-EN 15316-4-4 with NEN-EN 50465 test values).
    Chp(Box<HotWaterChp>),
    /// §13.8.4.9.3: hot water from the (collective) building system for
    /// space heating; `E_W;gen;in;conv;hj = Q_W;gen;out` (13.185) loads the
    /// space-heating node, with no carrier, auxiliary or recoverable loss for
    /// hot water.
    HeatingSystem,
}

/// Building CHP for hot water.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HotWaterChp {
    /// Method 2: η_W;gen = ε_chp;th of table 9.31 for an HT system
    /// (§13.8.4.7.4), electricity per 16.13.
    #[serde(default)]
    pub chp: Option<crate::space_cooling::ChpClass>,
    /// Method 1 (§13.8.4.8): 13.182/13.183 with 9.6.6.2, electricity per
    /// 16.16.
    #[serde(default)]
    pub method1: Option<crate::micro_chp::MicroChp>,
    /// The CHP also heats the building (13.181: no stand-by electronics).
    #[serde(default)]
    pub also_space_heating: bool,
    /// Method 1 without NEN-EN 50465 auxiliary powers: the 9.6.8 route
    /// (9.91/9.92) per 9.6.6.2.2.3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary: Option<crate::space_heating_chain::OtherGeneratorAuxiliary>,
    pub equipment_reference: String,
}

impl HotWaterChp {
    /// Table 9.31 HT factors (ε_chp;th, ε_chp;el) for method 2.
    fn forfait_factors(&self) -> Option<(f64, f64)> {
        let mut class = self.chp.clone()?;
        class.low_temperature = false;
        class.factors()
    }
}

/// Table 13.17 / note 5: tapping profiles of NEN-EN 13203-2 and
/// NEN-EN 16147 for 13.154.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestProfile {
    S,
    M,
    L,
    Xl,
    Xxl,
    #[serde(rename = "3xl")]
    ThreeXl,
    #[serde(rename = "4xl")]
    FourXl,
}

impl TestProfile {
    /// Note 5 of §13.8.4.2: `Q_ref` per day, kWh.
    pub fn reference_kwh_per_day(self) -> f64 {
        match self {
            Self::S => 2.1,
            Self::M => 5.845,
            Self::L => 11.655,
            Self::Xl => 19.07,
            Self::Xxl => 24.53,
            Self::ThreeXl => 46.76,
            Self::FourXl => 93.52,
        }
    }
}

/// Test standard of a §13.8.4.2 appliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TwoProfileStandard {
    /// Gas appliance per NEN-EN 13203-2 (fuel on net calorific value).
    En13203Gas,
    /// Electric heat pump per NEN-EN 16147.
    En16147HeatPump,
}

/// One tapping-profile test of table 13.17.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileTest {
    pub profile: TestProfile,
    /// `Q_W;test;i` (Q_ref), kWh/day.
    pub delivered_kwh_per_day: f64,
    /// NEN-EN 13203-2: `Q_gas;p(i)` corrected for summer and winter, net
    /// calorific value; NEN-EN 16147: `Q_elec`, kWh/day.
    pub input_kwh_per_day: f64,
    /// NEN-EN 13203-2 `E_elecco(i)` incl. stand-by, kWh/day (13.159).
    #[serde(default)]
    pub auxiliary_kwh_per_day: Option<f64>,
    /// NEN-EN 16147 `T_max;test;i` (13.153c), °C.
    #[serde(default)]
    pub max_test_temperature_c: Option<f64>,
}

/// §13.8.4.2 appliance tested at two tapping profiles `i1 < i2`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TwoProfileTest {
    pub standard: TwoProfileStandard,
    /// Storage appliance (voorraadtoestel); otherwise instantaneous.
    pub storage_appliance: bool,
    pub low: ProfileTest,
    pub high: ProfileTest,
    /// Combi appliance: its auxiliary energy is booked with space heating
    /// (`W_W;aux;gen = 0`).
    #[serde(default)]
    pub combi: bool,
    /// Combi appliance with an integrated vessel (13.160a).
    #[serde(default)]
    pub integrated_vessel: bool,
    /// NEN-EN 16147: heat pump on exhaust (return) air.
    #[serde(default)]
    pub exhaust_air_source: bool,
    /// 5.37: outdoor-air share of a partly exhaust-air source.
    #[serde(default)]
    pub outdoor_air_fraction: Option<f64>,
    /// NEN-EN 16147 `SCF` with `smart = 1` when it is at least 0,07
    /// (13.153b).
    #[serde(default)]
    pub smart_control_factor: Option<f64>,
    /// `T_set;design` (13.153c), default 55 °C; a lower value needs the
    /// appliance and installation design as evidence.
    #[serde(default)]
    pub design_set_temperature_c: Option<f64>,
    /// Weekly legionella prevention included in the NEN-EN 16147 test
    /// (`f_prac` 0,95 instead of 0,9 for storage appliances, 13.152).
    #[serde(default)]
    pub legionella_cycle_tested: bool,
    /// 13.153b/13.153d–i: combi heat pump on a mix of outdoor and
    /// ventilation return air.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mixed_air: Option<MixedAirCorrection>,
    /// 13.156a/b: passive flue heat recovery device of a gas combi
    /// (prEN 13203-7, 24-hour method).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pfhrd: Option<PfhrdTest>,
    pub source_reference: String,
}

/// `C_W;mixed air;mi` of 13.153b.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum MixedAirCorrection {
    /// Twelve monthly factors from a quality declaration.
    Declared {
        #[serde(rename = "monthlyFactors")]
        monthly_factors: Vec<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 13.153d–i from the NEN-EN 14511 result at condition 2 (A7/W55,
    /// table Q.11) and the minimum evaporator air flow `q_V;hp;W`.
    En14511 {
        /// `COP_H;hp;2` at maximum power.
        #[serde(rename = "copCondition2")]
        cop_condition_2: f64,
        /// `ϑ_cond;out;2`, °C.
        #[serde(rename = "condenserOutC")]
        condenser_out_c: f64,
        /// `ϑ_evap;in;2`, °C.
        #[serde(rename = "evaporatorInC")]
        evaporator_in_c: f64,
        /// `ϑ_evap;out;2`, °C.
        #[serde(rename = "evaporatorOutC")]
        evaporator_out_c: f64,
        /// `q_V;hp;W`, m³/h.
        #[serde(rename = "minimumAirFlowM3PerH")]
        minimum_air_flow_m3_per_h: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

/// 13.156b inputs, both on the net calorific value, kWh/day.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PfhrdTest {
    /// `Q_gas;indirect`.
    pub indirect_gas_kwh_per_day: f64,
    /// `Q_gas;CH;test`.
    pub heating_gas_kwh_per_day: f64,
    pub source_reference: String,
}

/// Data from outside chapter 13 for the final hot-water run.
#[derive(Debug, Clone, Default)]
pub struct HotWaterExtras {
    /// §13.8.4.8 (p. 650): the space-heating side of a combi micro-CHP.
    pub combi_chp: Option<CombiChpHeating>,
    /// 13.156a: `Σ E_H;gen;gi;in` of the combi appliance for space heating,
    /// gross calorific value, kWh per year.
    pub combi_space_heating_gas_kwh: Option<f64>,
    /// 13.153h/i per month.
    pub mixed_air: Option<MixedAirVentilation>,
}

/// The heating side of a combi micro-CHP (method 1) from the chain: its
/// product, its heat output (incl. storage share) and `t_H;op` per month.
#[derive(Debug, Clone)]
pub struct CombiChpHeating {
    pub product: crate::micro_chp::MicroChp,
    pub thermal_output_kwh: [f64; 12],
    pub operating_hours: [f64; 12],
}

/// The space-heating share of a combi micro-CHP month after the joint
/// 9.6.6.2 evaluation, kWh.
#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CombiChpHeatingMonth {
    pub month: u8,
    pub input_kwh: f64,
    pub electricity_kwh: f64,
    pub auxiliary_kwh: Option<f64>,
}

/// 13.153h/i inputs: `Σ q_V;ODA;req;zi,mi` of the zones whose return air
/// feeds the heat pump and `ϑ_ETA;dis;out;zi,mi` (11.130), flow-weighted.
#[derive(Debug, Clone, Copy)]
pub struct MixedAirVentilation {
    pub required_outdoor_air_m3_per_h: [f64; 12],
    pub extract_air_c: [f64; 12],
}

/// 13.153: alternative winter consumption of gas combis
/// (`Q_gas;w = Q_gas;s − 13,5 × 1,21·P_s / η_100`, net calorific value),
/// replacing formula (5) of NEN-EN 13203-2; kWh/day. The combination into
/// `Q_gas;p(i)` follows NEN-EN 13203-2.
pub fn winter_gas_consumption_kwh(
    summer_gas_kwh_per_day: f64,
    standby_loss_kw: f64,
    full_load_efficiency: f64,
) -> Option<f64> {
    (summer_gas_kwh_per_day.is_finite()
        && standby_loss_kw.is_finite()
        && standby_loss_kw >= 0.0
        && full_load_efficiency > 0.0)
        .then(|| summer_gas_kwh_per_day - 13.5 * 1.21 * standby_loss_kw / full_load_efficiency)
}

impl MixedAirCorrection {
    /// `C_W;mixed air;mi`; 1 when the ventilation data are missing.
    pub fn factor(&self, month_index: usize, ventilation: Option<&MixedAirVentilation>) -> f64 {
        match self {
            Self::Declared {
                monthly_factors, ..
            } => monthly_factors.get(month_index).copied().unwrap_or(1.0),
            Self::En14511 {
                cop_condition_2,
                condenser_out_c,
                evaporator_in_c,
                evaporator_out_c,
                minimum_air_flow_m3_per_h,
                ..
            } => {
                let Some(ventilation) = ventilation else {
                    return 1.0;
                };
                // 13.153f/g.
                let theoretical =
                    (condenser_out_c + 273.15 + 2.0) / (condenser_out_c - evaporator_out_c + 7.0);
                let carnot = cop_condition_2 / theoretical;
                // 13.153i/h.
                let outdoor_share = (1.0
                    - ventilation.required_outdoor_air_m3_per_h[month_index]
                        / minimum_air_flow_m3_per_h)
                    .max(0.0);
                let mixed = (OUTDOOR_TEMPERATURE_C[month_index] - 3.7) * outdoor_share
                    + ventilation.extract_air_c[month_index] * (1.0 - outdoor_share);
                // 13.153e (the two minus signs cancel).
                let cop = carnot * (condenser_out_c + 273.15 + 2.0)
                    / (condenser_out_c - (mixed - (evaporator_in_c - evaporator_out_c)) + 7.0);
                // 13.153d.
                cop / cop_condition_2
            }
        }
    }

    fn issues(&self, prefix: &str) -> Vec<(&'static str, String)> {
        let mut issues = Vec::new();
        match self {
            Self::Declared {
                monthly_factors,
                source_reference,
            } => {
                if monthly_factors.len() != 12
                    || monthly_factors.iter().any(|value| !positive(*value))
                {
                    issues.push((
                        "hot_water_mixed_air_invalid",
                        format!("{prefix}.monthlyFactors"),
                    ));
                }
                if source_reference.trim().is_empty() {
                    issues.push((
                        "source_reference_required",
                        format!("{prefix}.sourceReference"),
                    ));
                }
            }
            Self::En14511 {
                cop_condition_2,
                condenser_out_c,
                evaporator_in_c,
                evaporator_out_c,
                minimum_air_flow_m3_per_h,
                source_reference,
            } => {
                if !positive(*cop_condition_2)
                    || !positive(*minimum_air_flow_m3_per_h)
                    || ![condenser_out_c, evaporator_in_c, evaporator_out_c]
                        .iter()
                        .all(|value| value.is_finite())
                    || condenser_out_c <= evaporator_out_c
                {
                    issues.push(("hot_water_mixed_air_invalid", prefix.to_string()));
                }
                if source_reference.trim().is_empty() {
                    issues.push((
                        "source_reference_required",
                        format!("{prefix}.sourceReference"),
                    ));
                }
            }
        }
        issues
    }
}

/// 13.146: the energetic contribution `F_W;gen;gi` from a quality
/// declaration, interpolated over `Q_W;dis;nren;an` when given for several
/// tapping classes.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredGeneratorShare {
    pub points: Vec<SharePoint>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SharePoint {
    /// `Q_W;dis;nren;an` of the class, kWh.
    pub annual_kwh: f64,
    /// `F_W;gen;gi`.
    pub share: f64,
}

impl TwoProfileTest {
    fn electric(&self) -> bool {
        self.standard == TwoProfileStandard::En16147HeatPump
    }

    /// `E_W;gen;in;test;i`: 13.153a (gas to the gross calorific value) or
    /// 13.153b/13.153c (SCF and set temperature) with `C_W;mixed air` = 1.
    #[cfg(test)]
    fn corrected_input(&self, test: &ProfileTest) -> f64 {
        self.corrected_input_with(test, 1.0)
    }

    /// As [`Self::corrected_input`] with `C_W;mixed air;mi` (13.153b).
    fn corrected_input_with(&self, test: &ProfileTest, mixed_air: f64) -> f64 {
        match self.standard {
            TwoProfileStandard::En13203Gas => GAS_GROSS_PER_NET * test.input_kwh_per_day,
            TwoProfileStandard::En16147HeatPump => {
                let smart = self
                    .smart_control_factor
                    .filter(|value| *value >= 0.07)
                    .unwrap_or(0.0);
                let tested = test.max_test_temperature_c.unwrap_or(55.0);
                let design = self.design_set_temperature_c.unwrap_or(55.0);
                test.input_kwh_per_day * (1.0 - smart) / mixed_air * (1.0 + (55.0 - tested) * 0.02)
                    / (1.0 + (55.0 - design) * 0.02)
            }
        }
    }

    /// 13.156a/b: `E_W;gen;in;PFHRD`, kWh/day.
    fn pfhrd_daily(&self, extras: &HotWaterExtras) -> f64 {
        match (&self.pfhrd, extras.combi_space_heating_gas_kwh) {
            (Some(test), Some(annual)) if test.heating_gas_kwh_per_day > 0.0 => {
                annual / 365.0 * test.indirect_gas_kwh_per_day / test.heating_gas_kwh_per_day
            }
            _ => 0.0,
        }
    }

    /// §13.8.4.2 lower limit of `Q_W;b;d` (only for i2 = XXL, 3XL, 4XL).
    fn lower_bound_kwh_per_day(&self) -> f64 {
        use TestProfile::*;
        let q = |profile: TestProfile| profile.reference_kwh_per_day();
        match (self.low.profile, self.high.profile) {
            (_, FourXl) => (q(Xxl) + q(ThreeXl)) / 2.0,
            (_, ThreeXl) => q(Xl),
            (Xl, Xxl) => q(L),
            (L, Xxl) => q(M),
            _ => 0.0,
        }
    }

    /// Range conditions of §13.8.4.2 for `Q_W;b;d`; `None` when allowed.
    fn range_issue(&self, daily_kwh: f64) -> Option<&'static str> {
        use TestProfile::*;
        let q = |profile: TestProfile| profile.reference_kwh_per_day();
        let lower = self.lower_bound_kwh_per_day();
        let upper = if self.electric() {
            match self.high.profile {
                L => (q(L) + q(Xl)) / 2.0,
                other => q(other),
            }
        } else {
            f64::INFINITY
        };
        (daily_kwh + 1e-9 < lower || daily_kwh > upper + 1e-9)
            .then_some("hot_water_two_profile_out_of_range")
    }

    /// 13.154/13.154a, kWh/day, with `C_W;mixed air;mi` and
    /// `E_W;gen;in;PFHRD`.
    fn daily_input(&self, daily_kwh: f64, mixed_air: f64, pfhrd: f64) -> f64 {
        let (q1, q2) = (
            self.low.delivered_kwh_per_day,
            self.high.delivered_kwh_per_day,
        );
        let (e1, e2) = (
            self.corrected_input_with(&self.low, mixed_air),
            self.corrected_input_with(&self.high, mixed_air),
        );
        let linear = e1 + (e2 - e1) * (daily_kwh - q1) / (q2 - q1);
        if daily_kwh < q1 && (linear <= 0.0 || daily_kwh / linear > q1 / e1) {
            e1 / q1 * daily_kwh - pfhrd
        } else {
            linear - pfhrd
        }
    }

    /// 13.158 rounded down to 0,025 (gas) or 0,05 (electric).
    fn efficiency(&self, daily_kwh: f64) -> Result<f64, &'static str> {
        self.efficiency_with(daily_kwh, 1.0, 0.0)
    }

    /// 13.158 with `C_W;mixed air;mi` and `E_W;gen;in;PFHRD`.
    fn efficiency_with(
        &self,
        daily_kwh: f64,
        mixed_air: f64,
        pfhrd: f64,
    ) -> Result<f64, &'static str> {
        if daily_kwh + 1e-9 < self.lower_bound_kwh_per_day() {
            return self.below_range_efficiency(daily_kwh, mixed_air);
        }
        if let Some(code) = self.range_issue(daily_kwh) {
            return Err(code);
        }
        let input = self.daily_input(daily_kwh, mixed_air, pfhrd);
        if input <= 0.0 || daily_kwh <= 0.0 {
            return Err("hot_water_two_profile_out_of_range");
        }
        let step = if self.electric() { 0.05 } else { 0.025 };
        Ok(round_down(daily_kwh / input, step))
    }

    /// Below the §13.8.4.2 lower limit: for heat pumps 13.160b with the
    /// i1 test and `c_W,EU;gen` of table 13.18 over `Q_W;dis;nren;an`
    /// (taken as 365·`Q_W;b;d`); table 13.18 covers individual heat pumps
    /// only, so other appliances are rejected.
    fn below_range_efficiency(&self, daily_kwh: f64, mixed_air: f64) -> Result<f64, &'static str> {
        let profile = match self.low.profile {
            TestProfile::S => TappingProfile::S,
            TestProfile::M => TappingProfile::M,
            TestProfile::L => TappingProfile::L,
            TestProfile::Xl => TappingProfile::Xl,
            _ => return Err("hot_water_two_profile_below_range"),
        };
        if !self.electric() {
            return Err("hot_water_two_profile_below_range");
        }
        let correction = european_profile_correction(profile, 365.0 * daily_kwh)
            .ok_or("hot_water_two_profile_below_range")?;
        let input = self.corrected_input_with(&self.low, mixed_air);
        if input <= 0.0 {
            return Err("hot_water_two_profile_below_range");
        }
        Ok(round_down(
            self.low.delivered_kwh_per_day * correction / input,
            0.05,
        ))
    }

    /// 13.152: NEN-EN 16147 storage appliances 0,9 unless the test
    /// included weekly legionella prevention; otherwise 0,95.
    fn practical_factor(&self) -> f64 {
        if self.electric() && self.storage_appliance && !self.legionella_cycle_tested {
            0.9
        } else {
            0.95
        }
    }

    /// 13.159 for NEN-EN 13203-2 appliances other than combis, kWh/day.
    fn daily_auxiliary(&self, daily_kwh: f64) -> f64 {
        if self.electric() || self.combi {
            return 0.0;
        }
        let (q1, q2) = (
            self.low.delivered_kwh_per_day,
            self.high.delivered_kwh_per_day,
        );
        let w1 = self.low.auxiliary_kwh_per_day.unwrap_or(0.0);
        let w2 = self.high.auxiliary_kwh_per_day.unwrap_or(0.0);
        (w1 + (w2 - w1) * (daily_kwh - q1) / (q2 - q1)).max(0.0)
    }

    /// 13.160a per day: exhaust-air heat pumps and combis (gas or electric)
    /// with an integrated vessel; 0 for other appliances. `E_W;gen;in;test`
    /// is only corrected for summer and winter (gross value for gas), not
    /// with 13.153b/c.
    fn daily_recoverable(&self) -> f64 {
        let applies =
            (self.electric() && self.exhaust_air_source) || (self.combi && self.integrated_vessel);
        if !applies {
            return 0.0;
        }
        let (q1, q2) = (
            self.low.delivered_kwh_per_day,
            self.high.delivered_kwh_per_day,
        );
        let summer_winter = |test: &ProfileTest| match self.standard {
            TwoProfileStandard::En13203Gas => GAS_GROSS_PER_NET * test.input_kwh_per_day,
            TwoProfileStandard::En16147HeatPump => test.input_kwh_per_day,
        };
        let (e1, e2) = (summer_winter(&self.low), summer_winter(&self.high));
        (q1 / e1 * (e2 - (e2 - e1) / (q2 - q1) * q2)).max(0.0)
    }

    fn issues(&self, prefix: &str) -> Vec<(&'static str, String)> {
        use TestProfile::*;
        let mut issues = Vec::new();
        let allowed: &[TestProfile] = match (self.high.profile, self.storage_appliance) {
            (L, true) => &[S, M],
            (L, false) => &[M],
            (Xl, true) => &[S, M, L],
            (Xl, false) => &[M, L],
            (Xxl, _) => &[M, L, Xl],
            (ThreeXl, _) => &[Xxl],
            (FourXl, _) => &[ThreeXl],
            _ => &[],
        };
        if !allowed.contains(&self.low.profile) {
            issues.push((
                "hot_water_two_profile_pair_invalid",
                format!("{prefix}.low.profile"),
            ));
        }
        for (name, test) in [("low", &self.low), ("high", &self.high)] {
            if !positive(test.delivered_kwh_per_day) || !positive(test.input_kwh_per_day) {
                issues.push(("hot_water_test_values_invalid", format!("{prefix}.{name}")));
            }
            if !self.electric() && !self.combi && test.auxiliary_kwh_per_day.is_none() {
                issues.push((
                    "hot_water_two_profile_auxiliary_required",
                    format!("{prefix}.{name}.auxiliaryKwhPerDay"),
                ));
            }
            if test
                .auxiliary_kwh_per_day
                .is_some_and(|value| !value.is_finite() || value < 0.0)
            {
                issues.push((
                    "hot_water_test_values_invalid",
                    format!("{prefix}.{name}.auxiliaryKwhPerDay"),
                ));
            }
        }
        if self.low.delivered_kwh_per_day >= self.high.delivered_kwh_per_day {
            issues.push(("hot_water_test_values_invalid", format!("{prefix}.high")));
        }
        if !self.electric()
            && (self.exhaust_air_source
                || self.outdoor_air_fraction.is_some()
                || self.smart_control_factor.is_some()
                || self.design_set_temperature_c.is_some())
        {
            issues.push(("hot_water_two_profile_heat_pump_field", prefix.to_string()));
        }
        if self
            .smart_control_factor
            .is_some_and(|value| !(0.0..=1.0).contains(&value))
        {
            issues.push((
                "hot_water_test_values_invalid",
                format!("{prefix}.smartControlFactor"),
            ));
        }
        if self
            .outdoor_air_fraction
            .is_some_and(|value| !(0.0..=1.0).contains(&value))
        {
            issues.push((
                "hot_water_outdoor_fraction_invalid",
                format!("{prefix}.outdoorAirFraction"),
            ));
        }
        if let Some(mixed) = &self.mixed_air {
            // 13.153b is new in NTA 8800:2023 (p. 609–611).
            if !crate::norm_versions::profile().mixed_air_route {
                issues.push(("route_not_in_edition", format!("{prefix}.mixedAir")));
            }
            // 13.153b: combi heat pumps on outdoor and return air only.
            if !(self.electric() && self.combi && self.exhaust_air_source) {
                issues.push((
                    "hot_water_mixed_air_not_applicable",
                    format!("{prefix}.mixedAir"),
                ));
            }
            issues.extend(mixed.issues(&format!("{prefix}.mixedAir")));
        }
        if let Some(pfhrd) = &self.pfhrd {
            // 13.156a: gas combis only.
            if self.electric() || !self.combi {
                issues.push(("hot_water_pfhrd_not_applicable", format!("{prefix}.pfhrd")));
            }
            if !positive(pfhrd.heating_gas_kwh_per_day)
                || !pfhrd.indirect_gas_kwh_per_day.is_finite()
                || pfhrd.indirect_gas_kwh_per_day < 0.0
            {
                issues.push(("hot_water_pfhrd_invalid", format!("{prefix}.pfhrd")));
            }
            if pfhrd.source_reference.trim().is_empty() {
                issues.push((
                    "source_reference_required",
                    format!("{prefix}.pfhrd.sourceReference"),
                ));
            }
        }
        if self.source_reference.trim().is_empty() {
            issues.push((
                "source_reference_required",
                format!("{prefix}.sourceReference"),
            ));
        }
        issues
    }
}

/// 13.153a: `f_Hs/Hi` for natural gas.
const GAS_GROSS_PER_NET: f64 = 1.11;

impl DeclaredGeneratorShare {
    fn share(&self, annual_kwh: f64) -> f64 {
        let mut points: Vec<(f64, f64)> = self
            .points
            .iter()
            .map(|point| (point.annual_kwh, point.share))
            .collect();
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        interpolate(&points, annual_kwh)
    }
}

/// Individual delivery sets (afleversets) on external heat (§13.4.2).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeliverySets {
    pub count: u32,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectiveHotWater {
    /// `A_g;gebouw;W` (6.6.7), m².
    pub building_usable_floor_area_m2: f64,
    pub source_reference: String,
}

/// 13.144a: an exhaust-air heat pump in a system with several generators.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExhaustAirUse {
    /// Ventilation system C (EXTRACT_OP) or D without heat recovery; for
    /// other systems `Q_W;gen;gi;out;max = 0`.
    pub ventilation_suitable: bool,
    /// `f_combi;mi` (13.144a): 0 for hot water only, 1 in October–March with
    /// the forfait space-heating efficiency (9.6.3.1), or Q.5.2.3 fractions;
    /// empty means 0.
    #[serde(default)]
    pub heating_time_fraction: Vec<f64>,
    /// `q_ve;hp;W` from a quality declaration (13.148a), m³/h; without it
    /// the forfait of 13.148 applies to a hot-water-only heat pump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_flow_m3_per_h: Option<f64>,
}

/// A further generator of the hot-water system (13.8.2).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdditionalHotWaterGenerator {
    pub generator: HotWaterGenerator,
    /// `P_nom` (13.141), kW.
    #[serde(default)]
    pub nominal_power_kw: Option<f64>,
    #[serde(default)]
    pub exhaust_air: Option<ExhaustAirUse>,
    /// 13.146.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_share: Option<DeclaredGeneratorShare>,
    pub equipment_reference: String,
}

/// Two generators in series (13.8.2.1/13.8.2.2): the main generator first,
/// the single additional generator second.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SeriesArrangement {
    /// 13.141a: an individual system with an electric boiler after another
    /// appliance (hotfill); the first appliance supplies at most 80 %.
    HotfillElectricBoiler,
    /// 13.141b–d: a collective system whose first appliance also heats
    /// the building; `ϑ_H,max;si,mi` per month, °C.
    CollectiveFirstAlsoHeating {
        #[serde(rename = "maximumSupplyC")]
        maximum_supply_c: Vec<f64>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HotWaterSystem {
    pub need: HotWaterNeed,
    pub emission: HotWaterEmission,
    #[serde(default)]
    pub shower_heat_recovery: Option<ShowerHeatRecovery>,
    #[serde(default)]
    pub circulation: Option<Circulation>,
    #[serde(default)]
    pub storage: Vec<StorageVessel>,
    #[serde(default)]
    pub delivery_sets: Option<DeliverySets>,
    /// Boiling-water tap (`ϑ_sto;set = 90 °C`).
    #[serde(default)]
    pub boiling_water_tap: bool,
    pub generator: HotWaterGenerator,
    /// `P_nom` of the main generator (13.141), kW.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nominal_power_kw: Option<f64>,
    /// 13.144a limits when the main generator is an exhaust-air heat pump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exhaust_air: Option<ExhaustAirUse>,
    /// 13.146 for the main generator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_share: Option<DeclaredGeneratorShare>,
    /// Further generators (13.8.2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_generators: Vec<AdditionalHotWaterGenerator>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<SeriesArrangement>,
    /// §13.7 solar water heaters and solar combi systems.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solar: Vec<SolarWaterHeater>,
    #[serde(default)]
    pub collective: Option<CollectiveHotWater>,
    /// `b_U` of the unheated space with circulation pipes or vessels (8.53–
    /// 8.59): ϑ_ztu = ϑ_int;set;H − b_U·(ϑ_int;set;H − ϑ_e;avg) (7.82).
    /// Explicit `unheatedAmbientC` values take precedence; without both,
    /// 13 °C applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unheated_reduction_factor: Option<f64>,
    /// §13.2.4.1 (13.19a): bathrooms and kitchens connected to this system
    /// when a dwelling has several hot-water systems.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connected_taps: Option<ConnectedTaps>,
    pub equipment_reference: String,
}

/// 13.19a `n_b;si` and `n_k;si`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectedTaps {
    pub bathrooms: u32,
    pub kitchens: u32,
}

/// 13.19a: `F_W;si = n_b;si·C_W;nd;b/Σn_b + n_k;si·C_W;nd;k/Σn_k` for the
/// hot-water systems of a dwelling (category woningbouw); `None` when a
/// system lacks its connected taps or no tap is connected at all.
pub fn residential_need_fractions(taps: &[Option<ConnectedTaps>]) -> Option<Vec<f64>> {
    if taps.len() == 1 {
        return Some(vec![1.0]);
    }
    let taps: Vec<ConnectedTaps> = taps.iter().copied().collect::<Option<_>>()?;
    let bathrooms: u32 = taps.iter().map(|item| item.bathrooms).sum();
    let kitchens: u32 = taps.iter().map(|item| item.kitchens).sum();
    if bathrooms + kitchens == 0 {
        return None;
    }
    Some(
        taps.iter()
            .map(|item| {
                let bathroom = if bathrooms > 0 {
                    f64::from(item.bathrooms) * BATHROOM_SHARE / f64::from(bathrooms)
                } else {
                    0.0
                };
                let kitchen = if kitchens > 0 {
                    f64::from(item.kitchens) * KITCHEN_SHARE / f64::from(kitchens)
                } else {
                    0.0
                };
                bathroom + kitchen
            })
            .collect(),
    )
}

/// Sums the results of several hot-water systems of one building (§13.2.4)
/// for the energy performance; ratios are recombined from their sums.
pub fn merge_hot_water(results: Vec<HotWaterAssessment>) -> Option<HotWaterAssessment> {
    let mut iter = results.into_iter();
    let mut merged = iter.next()?;
    for result in iter {
        let need_ratio = |a: f64, eta_a: f64, b: f64, eta_b: f64| {
            let denominator = if eta_a > 0.0 { a / eta_a } else { 0.0 }
                + if eta_b > 0.0 { b / eta_b } else { 0.0 };
            if denominator > 0.0 {
                (a + b) / denominator
            } else {
                eta_a
            }
        };
        merged.emission_efficiency = need_ratio(
            merged.annual_net_need_kwh,
            merged.emission_efficiency,
            result.annual_net_need_kwh,
            result.emission_efficiency,
        );
        for (month, other) in merged.months.iter_mut().zip(&result.months) {
            month.distribution_efficiency = need_ratio(
                month.emission_input_kwh,
                month.distribution_efficiency,
                other.emission_input_kwh,
                other.distribution_efficiency,
            );
            month.generation_efficiency = need_ratio(
                month.generator_output_kwh,
                month.generation_efficiency,
                other.generator_output_kwh,
                other.generation_efficiency,
            );
            month.net_need_kwh += other.net_need_kwh;
            month.recovered_kwh += other.recovered_kwh;
            month.emission_input_kwh += other.emission_input_kwh;
            month.circulation_loss_kwh += other.circulation_loss_kwh;
            month.storage_loss_kwh += other.storage_loss_kwh;
            month.conversion_loss_kwh += other.conversion_loss_kwh;
            month.generator_output_kwh += other.generator_output_kwh;
            month.carrier_input_kwh += other.carrier_input_kwh;
            month.auxiliary_electricity_kwh += other.auxiliary_electricity_kwh;
            month.ambient_heat_kwh += other.ambient_heat_kwh;
            month.recoverable_loss_kwh += other.recoverable_loss_kwh;
            month.electricity_kwh += other.electricity_kwh;
            month.natural_gas_kwh += other.natural_gas_kwh;
            month.oil_kwh += other.oil_kwh;
            month.district_heat_kwh += other.district_heat_kwh;
            month.biomass_kwh += other.biomass_kwh;
            month.biomass_output_kwh += other.biomass_output_kwh;
            month.solar_renewable_kwh += other.solar_renewable_kwh;
            month.solar_space_heating_kwh += other.solar_space_heating_kwh;
            month.solar_auxiliary_kwh += other.solar_auxiliary_kwh;
            month.solar_backup_storage_loss_kwh += other.solar_backup_storage_loss_kwh;
            month.solar_recoverable_kwh += other.solar_recoverable_kwh;
            month.extra_electric_output_kwh += other.extra_electric_output_kwh;
            month.heating_system_load_kwh += other.heating_system_load_kwh;
            month.chp_electricity_kwh += other.chp_electricity_kwh;
            month.chp_excess_kwh += other.chp_excess_kwh;
        }
        merged.annual_net_need_kwh += result.annual_net_need_kwh;
        merged.annual_generator_output_kwh += result.annual_generator_output_kwh;
        merged.annual_solar_renewable_kwh += result.annual_solar_renewable_kwh;
        merged.annual_solar_space_heating_kwh += result.annual_solar_space_heating_kwh;
        merged.generators.extend(result.generators);
        if merged.exhaust_air.is_none() {
            merged.exhaust_air = result.exhaust_air;
        }
        if merged.combi_chp_heating.is_none() {
            merged.combi_chp_heating = result.combi_chp_heating;
        }
    }
    Some(merged)
}

impl HotWaterSystem {
    /// Carrier of the main generator.
    pub fn carrier(&self) -> HotWaterCarrier {
        self.generator.carrier()
    }
}

impl HotWaterGenerator {
    pub fn carrier(&self) -> HotWaterCarrier {
        match self {
            HotWaterGenerator::GasAppliance { .. } | HotWaterGenerator::GasStorageHeater { .. } => {
                HotWaterCarrier::Fuel(Carrier::Gas)
            }
            HotWaterGenerator::LargeDirectStorage { gas_fired: true } => {
                HotWaterCarrier::Fuel(Carrier::Gas)
            }
            HotWaterGenerator::BiomassCombi { .. } => HotWaterCarrier::Biomass,
            HotWaterGenerator::IndirectBoiler { oil, .. } => {
                HotWaterCarrier::Fuel(if *oil { Carrier::Oil } else { Carrier::Gas })
            }
            HotWaterGenerator::ExternalHeat => HotWaterCarrier::DistrictHeat,
            HotWaterGenerator::BoosterHeatPump(_)
            | HotWaterGenerator::BoosterHeatPumpForfait(_) => {
                match booster_heat_source(self).unwrap() {
                    BoosterHeatSource::ExternalHeat => HotWaterCarrier::DistrictHeat,
                    BoosterHeatSource::HeatingSystem => HotWaterGenerator::HeatingSystem.carrier(),
                    BoosterHeatSource::CollectiveGenerator { carrier, .. } => {
                        HotWaterCarrier::Fuel(match carrier {
                            BoosterSourceCarrier::Gas => Carrier::Gas,
                            BoosterSourceCarrier::Oil => Carrier::Oil,
                            BoosterSourceCarrier::Electricity => Carrier::El,
                        })
                    }
                }
            }
            HotWaterGenerator::MeasuredTwoProfiles(test) if !test.electric() => {
                HotWaterCarrier::Fuel(Carrier::Gas)
            }
            HotWaterGenerator::Chp(chp) => {
                HotWaterCarrier::Fuel(match chp.method1.as_ref().map(|product| product.fuel) {
                    Some(crate::micro_chp::MicroChpFuel::Oil) => Carrier::Oil,
                    _ => Carrier::Gas,
                })
            }
            _ => HotWaterCarrier::Fuel(Carrier::El),
        }
    }
}

/// Table 13.22 insulation of the store and pipework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomassStoreInsulation {
    AtLeast20Mm,
    AtLeast10Mm,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotWaterCarrier {
    Fuel(Carrier),
    DistrictHeat,
    /// Solid biomass (bmB: table 13.22 appliances meet annex R).
    Biomass,
}

/// Space-heating data for the heating part of solar combi systems
/// (13.7.2.2.3).
#[derive(Debug, Clone, Copy)]
pub struct SolarSpaceHeating {
    /// `Q_H;sol;us;mi = Q_H;nod;out + Q_H;nod;ls` before solar gains, kWh.
    pub node_kwh: [f64; 12],
    /// `f_gebouw;si;H` (9.1).
    pub building_fraction: f64,
    /// `ϑ_H,a;ontw` (table 9.14), °C.
    pub design_supply_c: f64,
    /// `ϑ_H,a;ontw − Δϑ_H;ontw` (13.90), °C.
    pub design_return_c: f64,
}

/// Context from outside chapter 13.
#[derive(Debug, Clone, Copy)]
pub struct HotWaterContext {
    pub residential: bool,
    /// `A_g;tot` of the assessed building (part), m².
    pub usable_floor_area_m2: f64,
    /// Heating setpoint of the heated zones (`ϑ_int;set;H`), °C.
    pub heated_ambient_c: f64,
    /// Space heating for solar combi systems; without it a combi system
    /// supplies hot water only (`f_W;use = 1`).
    pub space_heating: Option<SolarSpaceHeating>,
    /// `ϑ_int;set;H;stc` of table 7.13 (13.69a/13.137a), °C.
    pub standard_setpoint_c: Option<f64>,
    /// `ϑ_int;set;H;zi,mi` after levelling (7.76, 13.69/13.137b), °C.
    pub levelled_setpoint_c: Option<[f64; 12]>,
    /// §13.2.4 `F_W;si`: the share of the building's net need delivered by
    /// this system; `None` means 1.
    pub need_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HotWaterIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotWaterMonth {
    pub month: u8,
    pub net_need_kwh: f64,
    pub recovered_kwh: f64,
    pub emission_input_kwh: f64,
    pub circulation_loss_kwh: f64,
    pub storage_loss_kwh: f64,
    pub conversion_loss_kwh: f64,
    pub distribution_efficiency: f64,
    /// `Q_W;dis;nren`, delivered by the generator.
    pub generator_output_kwh: f64,
    pub generation_efficiency: f64,
    pub carrier_input_kwh: f64,
    pub auxiliary_electricity_kwh: f64,
    pub ambient_heat_kwh: f64,
    /// Q_W;ls;rbl (13.13) for the zones served, kWh, including 13.68.
    pub recoverable_loss_kwh: f64,
    /// Carrier inputs of all generators (13.1/13.3), kWh.
    pub electricity_kwh: f64,
    pub natural_gas_kwh: f64,
    pub oil_kwh: f64,
    pub district_heat_kwh: f64,
    /// Solid biomass of a table 13.22 appliance (bmB), kWh.
    pub biomass_kwh: f64,
    /// Q_W;gen;out of that appliance, for 5.39 (f_Pren;bmB), kWh.
    pub biomass_output_kwh: f64,
    /// 13.4a `Q_W;ren;sol,prac`, kWh.
    pub solar_renewable_kwh: f64,
    /// 13.66a `Q_H;ren;prac` of solar combi systems for the space-heating
    /// node (9.2.3.4), kWh.
    pub solar_space_heating_kwh: f64,
    /// 13.67, included in `auxiliaryElectricityKwh`, kWh.
    pub solar_auxiliary_kwh: f64,
    /// 13.8 backup-part loss of solar vessels, included in the output, kWh.
    pub solar_backup_storage_loss_kwh: f64,
    /// 13.68, included in `recoverableLossKwh`, kWh.
    pub solar_recoverable_kwh: f64,
    /// 13.8.2.3: the extra electric instantaneous heater for the rest, kWh.
    pub extra_electric_output_kwh: f64,
    /// 13.185 `E_W;gen;in;conv;hj`: output of a §13.8.4.9.3 generator,
    /// supplied by the space-heating system, kWh.
    pub heating_system_load_kwh: f64,
    /// 16.13/16.16 `E_el;chp;out;W`: electricity of a hot-water CHP, kWh.
    pub chp_electricity_kwh: f64,
    /// 9.66 with 13.183: hot-water heat above P_th;chp_100+sup_100·t_W;op,
    /// booked without electricity (`micro_chp_capacity_exceeded`), kWh.
    pub chp_excess_kwh: f64,
}

fn round_down(value: f64, step: f64) -> f64 {
    (value / step + 1e-9).floor() * step
}

fn interpolate(points: &[(f64, f64)], x: f64) -> f64 {
    if x <= points[0].0 {
        return points[0].1;
    }
    for pair in points.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        if x <= x1 {
            return y0 + (x - x0) / (x1 - x0) * (y1 - y0);
        }
    }
    points[points.len() - 1].1
}

/// Table 13.26 `c_W;gen` for gas appliances, electric boilers and CHP.
pub fn gas_class_correction(class: ApplicationClass, annual_kwh: f64) -> f64 {
    let x = [555.0, 1805.0, 2500.0, 3195.0, 3890.0];
    let y: [f64; 5] = match class {
        ApplicationClass::Class1 => [1.0, 1.0, 1.0, 1.0, 1.0],
        ApplicationClass::Class2 => [0.72, 0.90, 1.0, 1.0, 1.0],
        ApplicationClass::Class3 => [0.72, 0.85, 0.925, 1.0, 1.0],
        ApplicationClass::Class4 => [0.68, 0.80, 0.867, 0.933, 1.0],
    };
    let points: Vec<(f64, f64)> = x.into_iter().zip(y).collect();
    interpolate(&points, annual_kwh)
}

/// Table 13.27 `c_W;gen` for heat pumps; `None` above the measured class.
pub fn heat_pump_class_correction(class: ApplicationClass, annual_kwh: f64) -> Option<f64> {
    let points: &[(f64, f64)] = match class {
        ApplicationClass::Class1 => &[(1805.0, 1.0)],
        ApplicationClass::Class2 => &[(1805.0, 0.60), (2500.0, 1.0)],
        ApplicationClass::Class3 => &[(1805.0, 0.49), (2500.0, 0.81), (3195.0, 1.0)],
        ApplicationClass::Class4 => &[
            (1805.0, 0.45),
            (2500.0, 0.75),
            (3195.0, 0.92),
            (3890.0, 1.0),
        ],
    };
    let last = points[points.len() - 1].0;
    // No use in a higher class than measured (p. 644); class 4 is open-ended.
    if class != ApplicationClass::Class4 && annual_kwh > last + 1e-9 {
        return None;
    }
    Some(interpolate(points, annual_kwh))
}

/// 13.153b with `C_W;mixed air` = 1 and 13.153c: `E_W;gen;in;test;i` of a
/// NEN-EN 16147 test, kWh/day. Without SCF and temperatures (55 °C) the
/// measured `Q_elec;i` is returned unchanged.
pub fn en16147_corrected_input(
    input_kwh_per_day: f64,
    smart_control_factor: Option<f64>,
    max_test_temperature_c: Option<f64>,
    design_set_temperature_c: Option<f64>,
) -> f64 {
    let smart = smart_control_factor
        .filter(|value| *value >= 0.07)
        .unwrap_or(0.0);
    let tested = max_test_temperature_c.unwrap_or(55.0);
    let design = design_set_temperature_c.unwrap_or(55.0);
    input_kwh_per_day * (1.0 - smart) * (1.0 + (55.0 - tested) * 0.02)
        / (1.0 + (55.0 - design) * 0.02)
}

/// Table 13.18 `c_W,EU;gen`; `None` above the measured profile.
///
/// Edges: below 765 kWh the first column applies (the class is not
/// exceeded and the table has no lower column); above the last column of
/// the measured profile the heat pump would be used in a higher class than
/// tested, which p. 630 forbids (`hot_water_heat_pump_class_exceeded`).
pub fn european_profile_correction(profile: TappingProfile, annual_kwh: f64) -> Option<f64> {
    let points: &[(f64, f64)] = match profile {
        TappingProfile::S => &[(765.0, 1.0)],
        TappingProfile::M => &[(765.0, 0.56), (2130.0, 1.0)],
        TappingProfile::L => &[(765.0, 0.43), (2130.0, 0.74), (4250.0, 1.0)],
        TappingProfile::Xl => &[(765.0, 0.35), (2130.0, 0.61), (4250.0, 0.79), (6960.0, 1.0)],
    };
    let last = points[points.len() - 1].0;
    if annual_kwh > last + 1e-9 {
        return None;
    }
    match crate::norm_versions::profile().table_13_18 {
        crate::norm_versions::Table1318Lookup::Interpolate => Some(interpolate(points, annual_kwh)),
        // NTA 8800:2024 table 13.18 (p. 614) gives no interpolation rule:
        // the column of the largest quantity not above the demand.
        crate::norm_versions::Table1318Lookup::LowerColumn => points
            .iter()
            .rev()
            .find(|(quantity, _)| *quantity <= annual_kwh + 1e-9)
            .or(points.first())
            .map(|(_, value)| *value),
    }
}

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

pub fn validate_hot_water(
    system: &HotWaterSystem,
    context: HotWaterContext,
    path: &str,
) -> Vec<HotWaterIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: &str| {
        issues.push(HotWaterIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    if system
        .unheated_reduction_factor
        .is_some_and(|b| !(0.0..=1.0).contains(&b))
    {
        push(
            "hot_water_unheated_reduction_factor_invalid",
            "unheatedReductionFactor",
        );
    }
    match &system.need {
        HotWaterNeed::Residential {
            dwelling_count,
            source_reference,
        } => {
            if !context.residential {
                push("hot_water_need_scope_mismatch", "need.method");
            }
            if *dwelling_count == 0 {
                push("dwelling_count_invalid", "need.dwellingCount");
            }
            if source_reference.trim().is_empty() {
                push("source_reference_required", "need.sourceReference");
            }
        }
        HotWaterNeed::Utility {
            areas,
            source_reference,
        } => {
            if context.residential {
                push("hot_water_need_scope_mismatch", "need.method");
            }
            let total: f64 = areas.iter().map(|item| item.area_m2).sum();
            if areas.is_empty()
                || areas.iter().any(|item| {
                    !positive(item.area_m2) || item.function == LabelFunction::Residential
                })
                // 13.20/13.20a: a system serves part of the building when
                // there are several; it never serves more than the building.
                || total > context.usable_floor_area_m2 * (1.0 + 1e-6) + 1e-9
            {
                push("hot_water_function_areas_invalid", "need.areas");
            }
            if source_reference.trim().is_empty() {
                push("source_reference_required", "need.sourceReference");
            }
        }
    }
    match &system.emission {
        HotWaterEmission::Residential {
            served,
            kitchen_length_m,
            kitchen_pipe_diameter,
            bathroom_length_m,
            source_reference,
        } => {
            if !context.residential {
                push("hot_water_emission_scope_mismatch", "emission.method");
            }
            let needs_kitchen = *served != ServedTaps::BathroomOnly;
            let needs_bathroom = *served != ServedTaps::KitchenOnly;
            if needs_kitchen && !kitchen_length_m.is_some_and(|v| v.is_finite() && v >= 0.0) {
                push("hot_water_length_invalid", "emission.kitchenLengthM");
            }
            // Table 13.2 diameter rows exist in NTA 8800:2023 only.
            if matches!(
                kitchen_pipe_diameter,
                Some(KitchenPipeDiameter::UpTo8Mm | KitchenPipeDiameter::UpTo10Mm)
            ) && !crate::norm_versions::profile().kitchen_diameter_rows
            {
                push("route_not_in_edition", "emission.kitchenPipeDiameter");
            }
            if needs_bathroom && !bathroom_length_m.is_some_and(|v| v.is_finite() && v >= 0.0) {
                push("hot_water_length_invalid", "emission.bathroomLengthM");
            }
            if source_reference.trim().is_empty() {
                push("source_reference_required", "emission.sourceReference");
            }
        }
        HotWaterEmission::Utility {
            mean_length_m,
            source_reference,
        } => {
            if context.residential {
                push("hot_water_emission_scope_mismatch", "emission.method");
            }
            if !mean_length_m.is_finite() || *mean_length_m < 0.0 {
                push("hot_water_length_invalid", "emission.meanLengthM");
            }
            if source_reference.trim().is_empty() {
                push("source_reference_required", "emission.sourceReference");
            }
        }
    }
    if let Some(shower) = &system.shower_heat_recovery {
        if shower.showers.is_empty() {
            push("hot_water_showers_required", "showerHeatRecovery.showers");
        }
        let required = required_shower_class(system, context.residential);
        for (index, unit) in shower.showers.iter().enumerate() {
            if unit.test_class().is_some_and(|class| class != required) {
                push(
                    "annex_u_test_class_mismatch",
                    &format!("showerHeatRecovery.showers[{index}]"),
                );
            }
            if let ShowerUnit::AnnexU { test } = unit {
                for code in test.issues() {
                    push(code, &format!("showerHeatRecovery.showers[{index}].test"));
                }
            }
            if let ShowerUnit::Declared {
                efficiency,
                source_reference,
                ..
            } = unit
            {
                if !(0.0..=1.0).contains(efficiency) {
                    push(
                        "hot_water_shower_efficiency_invalid",
                        &format!("showerHeatRecovery.showers[{index}].efficiency"),
                    );
                }
                if source_reference.trim().is_empty() {
                    push(
                        "source_reference_required",
                        &format!("showerHeatRecovery.showers[{index}].sourceReference"),
                    );
                }
            }
        }
        if shower.source_reference.trim().is_empty() {
            push(
                "source_reference_required",
                "showerHeatRecovery.sourceReference",
            );
        }
    }
    if let Some(circulation) = &system.circulation {
        if circulation.floor_count == 0
            || circulation.floor_count > crate::monthly_demand::MAX_STOREYS
        {
            push("hot_water_floor_count_invalid", "circulation.floorCount");
        }
        for (field, value) in [
            ("outerDiameterMm", circulation.outer_diameter_mm),
            ("declaredPsiWPerMK", circulation.declared_psi_w_per_mk),
            ("lengthM", circulation.length_m),
            ("pump.labelPowerKw", circulation.pump.label_power_kw),
        ] {
            if value.is_some_and(|value| !positive(value)) {
                push(
                    "hot_water_circulation_value_invalid",
                    &format!("circulation.{field}"),
                );
            }
        }
        if let Some(geometry) = &circulation.calculated_psi {
            if geometry.psi().is_none() {
                push(
                    "hot_water_pipe_geometry_invalid",
                    "circulation.calculatedPsi",
                );
            }
            if circulation.declared_psi_w_per_mk.is_some() {
                push(
                    "hot_water_psi_declared_and_calculated",
                    "circulation.calculatedPsi",
                );
            }
        }
        if circulation
            .unheated_length_m
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            push(
                "hot_water_circulation_value_invalid",
                "circulation.unheatedLengthM",
            );
        }
        if circulation
            .pump
            .energy_efficiency_index
            .is_some_and(|value| !positive(value) || value > 1.0)
        {
            push(
                "hot_water_circulation_value_invalid",
                "circulation.pump.energyEfficiencyIndex",
            );
        }
        if circulation.source_reference.trim().is_empty() {
            push("source_reference_required", "circulation.sourceReference");
        }
    }
    let mut ids = std::collections::HashSet::new();
    for (index, vessel) in system.storage.iter().enumerate() {
        let base = format!("storage[{index}]");
        if vessel.id.trim().is_empty() || !ids.insert(vessel.id.as_str()) {
            push("id_invalid", &format!("{base}.id"));
        }
        if !positive(vessel.volume_l) {
            push(
                "hot_water_storage_volume_invalid",
                &format!("{base}.volumeL"),
            );
        }
        if !(1..=5).contains(&vessel.connection_factor) {
            push(
                "hot_water_storage_connection_invalid",
                &format!("{base}.connectionFactor"),
            );
        }
        if vessel.electric_boiler_insulated_pipe
            && !crate::norm_versions::profile().electric_boiler_insulated_pipe_factor
        {
            push(
                "route_not_in_edition",
                &format!("{base}.electricBoilerInsulatedPipe"),
            );
        }
        if let StorageLoss::Measured {
            transmission_w_per_k,
        } = vessel.loss
        {
            if !positive(transmission_w_per_k) {
                push(
                    "hot_water_storage_loss_invalid",
                    &format!("{base}.loss.transmissionWPerK"),
                );
            }
        }
        if let StorageLoss::MeasuredStandby {
            standby_kwh_per_day,
            reference_storage_c,
            reference_ambient_c,
        } = vessel.loss
        {
            if !positive(standby_kwh_per_day)
                || !reference_ambient_c.is_finite()
                || !positive(reference_storage_c - reference_ambient_c)
            {
                push("hot_water_storage_loss_invalid", &format!("{base}.loss"));
            }
        }
        if vessel.source_reference.trim().is_empty() {
            push(
                "source_reference_required",
                &format!("{base}.sourceReference"),
            );
        }
    }
    // §13.6.2: the storage loss is part of the efficiency of complete
    // appliances; separate vessels belong to boilers and indirect systems.
    let generators: Vec<&HotWaterGenerator> = std::iter::once(&system.generator)
        .chain(
            system
                .additional_generators
                .iter()
                .map(|unit| &unit.generator),
        )
        .collect();
    // §13.6.2 (2025+C1 p. 566, 2022 p. 547): for a solar system the loss of
    // the vessel and of its backup part follows 13.7 (13.95/13.113), so a
    // boiler that heats the backup part of an integrated-backup solar vessel
    // needs no separate vessel of its own.
    let solar_vessel_with_backup = system.solar.iter().any(|heater| {
        heater.solar_use != crate::solar_thermal::SolarUse::SpaceHeating
            && heater.method.solar_type() == crate::solar_thermal::SolarType::IntegratedBackup
    });
    let needs_storage = !solar_vessel_with_backup
        && generators.iter().any(|generator| {
            matches!(
                generator,
                HotWaterGenerator::ElectricBoiler
                    | HotWaterGenerator::IndirectBoiler { .. }
                    | HotWaterGenerator::IndirectHeatPump { .. }
            )
        });
    // Note 1 of §13.6.2: a vessel outside the 24-hour test of a
    // 13.8.4.2/13.8.4.3 appliance is calculated like any other.
    let tested = generators.iter().any(|generator| {
        matches!(
            generator,
            HotWaterGenerator::MeasuredTwoProfiles(_)
                | HotWaterGenerator::HeatPumpEn16147 { .. }
                | HotWaterGenerator::GasAppliance {
                    annex_t: Some(_),
                    ..
                }
        )
    });
    let separately_tested = tested
        && system
            .storage
            .iter()
            .all(|vessel| vessel.not_in_appliance_test);
    let allows_storage = needs_storage
        || separately_tested
        || generators
            .iter()
            .any(|generator| matches!(generator, HotWaterGenerator::ExternalHeat));
    if needs_storage && system.storage.is_empty() {
        push("hot_water_storage_required", "storage");
    }
    if !allows_storage && !system.storage.is_empty() {
        push("hot_water_storage_in_generator_efficiency", "storage");
    }
    for (code, field) in generator_issues(&system.generator, "generator") {
        push(code, &field);
    }
    let several = !system.additional_generators.is_empty();
    let collective = system.collective.is_some();
    let units = std::iter::once((
        &system.generator,
        system.nominal_power_kw,
        system.exhaust_air.as_ref(),
        String::new(),
    ))
    .chain(
        system
            .additional_generators
            .iter()
            .enumerate()
            .map(|(index, unit)| {
                (
                    &unit.generator,
                    unit.nominal_power_kw,
                    unit.exhaust_air.as_ref(),
                    format!("additionalGenerators[{index}]."),
                )
            }),
    );
    for (generator, power, exhaust, prefix) in units {
        let power_field = format!("{prefix}nominalPowerKw");
        if power.is_some_and(|value| !value.is_finite() || value <= 0.0) {
            push("hot_water_nominal_power_invalid", &power_field);
        }
        // 13.141: every parallel generator needs P_nom; 13.142 for external
        // heat; an individual exhaust-air heat pump may use 1,0 kW.
        if several
            && system.series.is_none()
            && power.is_none()
            && !matches!(generator, HotWaterGenerator::ExternalHeat)
            && !(exhaust_air_heat_pump(generator) && !collective)
        {
            push("hot_water_nominal_power_required", &power_field);
        }
        let exhaust_field = format!("{prefix}exhaustAir");
        if let Some(exhaust) = exhaust {
            if !exhaust_air_heat_pump(generator) {
                push("hot_water_exhaust_air_not_applicable", &exhaust_field);
            }
            if exhaust
                .declared_flow_m3_per_h
                .is_some_and(|value| !positive(value))
            {
                push(
                    "hot_water_exhaust_air_flow_invalid",
                    &format!("{exhaust_field}.declaredFlowM3PerH"),
                );
            }
            // (13.148a) is new in NTA 8800:2022 (p. 594; 2020+A1 p. 590).
            let declared_route = crate::norm_versions::profile().declared_exhaust_air_flow_route;
            if !declared_route && exhaust.declared_flow_m3_per_h.is_some() {
                push(
                    "route_not_in_edition",
                    &format!("{exhaust_field}.declaredFlowM3PerH"),
                );
            }
            // 13.148 note 3: with measured efficiencies the flow comes from
            // the declaration with the measurements.
            if declared_route
                && exhaust.declared_flow_m3_per_h.is_none()
                && matches!(
                    generator,
                    HotWaterGenerator::HeatPumpEn16147 { .. }
                        | HotWaterGenerator::MeasuredTwoProfiles(_)
                )
            {
                push(
                    "hot_water_exhaust_air_declared_flow_required",
                    &format!("{exhaust_field}.declaredFlowM3PerH"),
                );
            }
            let fractions = &exhaust.heating_time_fraction;
            if !fractions.is_empty()
                && (fractions.len() != 12 || fractions.iter().any(|v| !(0.0..=1.0).contains(v)))
            {
                push(
                    "hot_water_exhaust_air_fraction_invalid",
                    &format!("{exhaust_field}.heatingTimeFraction"),
                );
            }
        } else if exhaust_air_heat_pump(generator) {
            // 13.144a needs the ventilation system, also for one generator.
            push("hot_water_exhaust_air_use_required", &exhaust_field);
        }
    }
    // 13.146: declared contributions.
    let declared = std::iter::once((system.declared_share.as_ref(), String::new())).chain(
        system
            .additional_generators
            .iter()
            .enumerate()
            .map(|(index, unit)| {
                (
                    unit.declared_share.as_ref(),
                    format!("additionalGenerators[{index}]."),
                )
            }),
    );
    let mut declared_total = 0.0;
    for (share, prefix) in declared {
        let Some(share) = share else { continue };
        let field = format!("{prefix}declaredShare");
        if share.points.is_empty()
            || share.points.iter().any(|point| {
                !point.annual_kwh.is_finite()
                    || point.annual_kwh < 0.0
                    || !(0.0..=1.0).contains(&point.share)
            })
        {
            push("hot_water_declared_share_invalid", &field);
        }
        if share.source_reference.trim().is_empty() {
            push(
                "source_reference_required",
                &format!("{field}.sourceReference"),
            );
        }
        declared_total += share
            .points
            .iter()
            .map(|point| point.share)
            .fold(0.0, f64::max);
    }
    if declared_total > 1.0 + 1e-9 {
        push("hot_water_declared_share_sum_exceeds_one", "declaredShare");
    }
    for (index, unit) in system.additional_generators.iter().enumerate() {
        for (code, field) in generator_issues(
            &unit.generator,
            &format!("additionalGenerators[{index}].generator"),
        ) {
            push(code, &field);
        }
        if unit.equipment_reference.trim().is_empty() {
            push(
                "source_reference_required",
                &format!("additionalGenerators[{index}].equipmentReference"),
            );
        }
    }
    if let Some(series) = &system.series {
        // 13.141a–d exist from 2024 (p. 595); NTA 8800:2023 has no series
        // arrangements.
        if !crate::norm_versions::profile().hot_water_series_routes {
            push("route_not_in_edition", "series");
        }
        if system.additional_generators.len() != 1 {
            push("hot_water_series_requires_two_generators", "series");
        }
        match series {
            SeriesArrangement::HotfillElectricBoiler => {
                if collective {
                    push("hot_water_series_hotfill_individual_only", "series");
                }
                if !system
                    .additional_generators
                    .first()
                    .is_some_and(|unit| matches!(unit.generator, HotWaterGenerator::ElectricBoiler))
                {
                    push(
                        "hot_water_series_hotfill_requires_electric_boiler",
                        "series",
                    );
                }
            }
            SeriesArrangement::CollectiveFirstAlsoHeating { maximum_supply_c } => {
                if !collective {
                    push("hot_water_series_collective_only", "series");
                }
                if maximum_supply_c.len() != 12
                    || maximum_supply_c.iter().any(|value| !value.is_finite())
                {
                    push("hot_water_series_supply_invalid", "series.maximumSupplyC");
                }
            }
        }
    }
    let mut solar_ids = std::collections::HashSet::new();
    for (index, heater) in system.solar.iter().enumerate() {
        let base = format!("solar[{index}]");
        if !solar_ids.insert(heater.id.as_str()) {
            push("id_invalid", &format!("{base}.id"));
        }
        for found in validate_solar(heater, "") {
            push(found.code, &format!("{base}{}", found.path));
        }
    }
    if let Some(sets) = &system.delivery_sets {
        if sets.count == 0 {
            push("hot_water_delivery_set_count_invalid", "deliverySets.count");
        }
        if sets.source_reference.trim().is_empty() {
            push("source_reference_required", "deliverySets.sourceReference");
        }
    }
    if let Some(collective) = &system.collective {
        if !positive(collective.building_usable_floor_area_m2) {
            push(
                "hot_water_collective_area_invalid",
                "collective.buildingUsableFloorAreaM2",
            );
        }
        if collective.source_reference.trim().is_empty() {
            push("source_reference_required", "collective.sourceReference");
        }
    }
    if system.equipment_reference.trim().is_empty() {
        push("source_reference_required", "equipmentReference");
    }
    issues
}

/// Issues of one generator (main or additional), as (code, path).
fn generator_issues(generator: &HotWaterGenerator, prefix: &str) -> Vec<(&'static str, String)> {
    let mut issues = Vec::new();
    // §13.8.4.10 exists from 2024 (p. 638); NTA 8800:2023 has no stepped
    // temperature rise.
    if matches!(generator, HotWaterGenerator::HeatPumpSeries { .. })
        && !crate::norm_versions::profile().hot_water_series_routes
    {
        issues.push(("route_not_in_edition", format!("{prefix}.kind")));
    }
    if let HotWaterGenerator::Chp(chp) = generator {
        if chp.equipment_reference.trim().is_empty() {
            issues.push((
                "source_reference_required",
                format!("{prefix}.equipmentReference"),
            ));
        }
        match (&chp.chp, &chp.method1) {
            (Some(_), None) => {
                if chp.forfait_factors().is_none() {
                    issues.push(("chp_class_invalid", format!("{prefix}.chp")));
                }
            }
            (None, Some(product)) => {
                for item in
                    crate::micro_chp::validate_micro_chp(product, &format!("{prefix}.method1"))
                {
                    issues.push((item.code, item.path));
                }
                // 13.8.4.8.3 takes W_H;gen;aux of 9.6.6.2: measured values.
                let measured_aux = product.net_production_measured
                    || (product.standby_auxiliary_kw.is_some()
                        && product.chp_only.auxiliary_power_kw.is_some()
                        && product.full_load.auxiliary_power_kw.is_some());
                // 9.6.6.2.2.3: without measured auxiliary powers 9.6.8.
                match (&chp.auxiliary, measured_aux) {
                    (None, false) => issues.push((
                        "hot_water_chp_auxiliary_required",
                        format!("{prefix}.auxiliary"),
                    )),
                    (Some(auxiliary), _) => {
                        if auxiliary.source_reference.trim().is_empty() {
                            issues.push((
                                "source_reference_required",
                                format!("{prefix}.auxiliary.sourceReference"),
                            ));
                        }
                        if !auxiliary
                            .nominal_power_kw
                            .is_some_and(|power| power.is_finite() && power > 0.0)
                        {
                            issues.push((
                                "generator_nominal_power_required",
                                format!("{prefix}.auxiliary.nominalPowerKw"),
                            ));
                        }
                    }
                    (None, true) => {}
                }
                if product.storage.is_some() {
                    // §13.8.4.8.1: hot-water vessels follow 13.6.
                    issues.push((
                        "hot_water_chp_storage_via_13_6",
                        format!("{prefix}.method1.storage"),
                    ));
                }
            }
            _ => issues.push(("hot_water_chp_method_required", prefix.to_string())),
        }
    }
    if let HotWaterGenerator::GasAppliance {
        appliance,
        measured_class,
        declared,
        annex_t: Some(test),
        annex_t_conditions,
        ..
    } = generator
    {
        for code in test.issues() {
            issues.push((code, format!("{prefix}.annexT")));
        }
        match annex_t_conditions {
            None => issues.push((
                "annex_t_conditions_required",
                format!("{prefix}.annexTConditions"),
            )),
            Some(conditions) => {
                if !conditions.type_supplied_before_2021 || !conditions.appliance_indoors {
                    issues.push((
                        "annex_t_not_applicable",
                        format!("{prefix}.annexTConditions"),
                    ));
                }
            }
        }
        if declared.is_some() {
            issues.push((
                "hot_water_efficiency_declared_twice",
                format!("{prefix}.annexT"),
            ));
        }
        if measured_class.is_none() {
            issues.push((
                "annex_t_measured_class_required",
                format!("{prefix}.measuredClass"),
            ));
        }
        let combi = matches!(
            appliance,
            GasAppliance::CombiGaskeur | GasAppliance::CombiGaskeurHrCw
        );
        if combi != test.is_combi() {
            issues.push((
                "annex_t_appliance_mismatch",
                format!("{prefix}.annexT.method"),
            ));
        }
    }
    match generator {
        HotWaterGenerator::GasAppliance { declared, .. }
        | HotWaterGenerator::IndirectBoiler { declared, .. } => {
            if let Some(item) = declared {
                // Rounded down to a multiple of 0,025 (§13.8.4.7.2): below one
                // step the value would become 0.
                if !positive(item.value) || item.value < 0.025 || item.value > 1.2 {
                    issues.push((
                        "hot_water_efficiency_invalid",
                        format!("{prefix}.declared.value"),
                    ));
                }
                if item.source_reference.trim().is_empty() {
                    issues.push((
                        "source_reference_required",
                        format!("{prefix}.declared.sourceReference"),
                    ));
                }
            }
        }
        HotWaterGenerator::HeatPump {
            source_correction,
            outdoor_air_fraction,
            exhaust_air_source,
            same_ground_source,
            declared,
            ..
        } => {
            if let Some(item) = declared {
                // Rounded down to a multiple of 0,05 (§13.8.4.7.2): below one
                // step the value would become 0.
                if !positive(item.value) || item.value < 0.05 || item.value > 10.0 {
                    issues.push((
                        "hot_water_efficiency_invalid",
                        format!("{prefix}.declared.value"),
                    ));
                }
                if item.source_reference.trim().is_empty() {
                    issues.push((
                        "source_required",
                        format!("{prefix}.declared.sourceReference"),
                    ));
                }
                if source_correction.is_some() {
                    issues.push((
                        "hot_water_efficiency_declared_twice",
                        format!("{prefix}.sourceCorrection"),
                    ));
                }
            }
            // Tables V.1/V.3: c_source is 1,00, 1,02 or 1,04.
            if source_correction.is_some_and(|value| {
                ![1.00, 1.02, 1.04]
                    .iter()
                    .any(|allowed| (value - allowed).abs() < 1e-9)
            }) {
                issues.push((
                    "hot_water_source_correction_invalid",
                    format!("{prefix}.sourceCorrection"),
                ));
            }
            if *same_ground_source && *exhaust_air_source {
                issues.push((
                    "hot_water_same_ground_source_exhaust_air",
                    format!("{prefix}.sameGroundSource"),
                ));
            }
            if outdoor_air_fraction.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
                issues.push((
                    "hot_water_outdoor_fraction_invalid",
                    format!("{prefix}.outdoorAirFraction"),
                ));
            }
        }
        HotWaterGenerator::HeatPumpEn16147 {
            delivered_kwh_per_day,
            input_kwh_per_day,
            outdoor_air_fraction,
            smart_control_factor,
            max_test_temperature_c,
            design_set_temperature_c,
            source_reference,
            ..
        } => {
            if !positive(*delivered_kwh_per_day) || !positive(*input_kwh_per_day) {
                issues.push(("hot_water_test_values_invalid", prefix.to_string()));
            }
            if smart_control_factor.is_some_and(|value| !(0.0..1.0).contains(&value)) {
                issues.push((
                    "hot_water_test_values_invalid",
                    format!("{prefix}.smartControlFactor"),
                ));
            }
            for (value, field) in [
                (max_test_temperature_c, "maxTestTemperatureC"),
                (design_set_temperature_c, "designSetTemperatureC"),
            ] {
                if value.is_some_and(|value| !(30.0..=95.0).contains(&value)) {
                    issues.push(("hot_water_test_values_invalid", format!("{prefix}.{field}")));
                }
            }
            if outdoor_air_fraction.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
                issues.push((
                    "hot_water_outdoor_fraction_invalid",
                    format!("{prefix}.outdoorAirFraction"),
                ));
            }
            if source_reference.trim().is_empty() {
                issues.push((
                    "source_reference_required",
                    format!("{prefix}.sourceReference"),
                ));
            }
        }
        HotWaterGenerator::BoosterHeatPump(pump) => {
            for found in validate_booster(pump, "generator") {
                issues.push((found.code, found.path.replacen("generator", prefix, 1)));
            }
        }
        HotWaterGenerator::BoosterHeatPumpForfait(pump) => {
            for found in validate_booster_forfait(pump, "generator") {
                issues.push((found.code, found.path.replacen("generator", prefix, 1)));
            }
        }
        HotWaterGenerator::MeasuredTwoProfiles(test) => {
            issues.extend(test.issues(prefix));
        }
        HotWaterGenerator::GasStorageHeater {
            volume_l,
            measured_standby_kwh_per_day,
            ..
        } => {
            if !positive(*volume_l) {
                issues.push((
                    "hot_water_storage_volume_invalid",
                    format!("{prefix}.volumeL"),
                ));
            }
            if measured_standby_kwh_per_day.is_some_and(|value| !positive(value)) {
                issues.push((
                    "hot_water_storage_loss_invalid",
                    format!("{prefix}.measuredStandbyKwhPerDay"),
                ));
            }
        }
        _ => {}
    }
    issues
}

/// 13.8.2.1: a heat pump on exhaust (return) air.
fn exhaust_air_heat_pump(generator: &HotWaterGenerator) -> bool {
    match generator {
        HotWaterGenerator::HeatPump {
            exhaust_air_source, ..
        }
        | HotWaterGenerator::HeatPumpEn16147 {
            exhaust_air_source, ..
        } => *exhaust_air_source,
        HotWaterGenerator::MeasuredTwoProfiles(test) => test.electric() && test.exhaust_air_source,
        _ => false,
    }
}

/// ϑ_ztu of the unheated space of month `index` for pipes and vessels:
/// the entered value, else 7.82 with `b_U`, else 13 °C.
fn unheated_ambient(
    system: &HotWaterSystem,
    explicit: Option<f64>,
    setpoint_c: f64,
    index: usize,
) -> f64 {
    explicit.unwrap_or_else(|| match system.unheated_reduction_factor {
        Some(b) => setpoint_c - b * (setpoint_c - crate::climate::OUTDOOR_TEMPERATURE_C[index]),
        None => UNHEATED_AMBIENT_C,
    })
}

/// 13.53: the mean efficiency over all showers of the system (0 for a
/// shower without a unit). Utility buildings where the assignment of
/// showers to units is unknown and more than 80 % of the showers are
/// connected take the lowest unit efficiency (p. 564).
pub fn shower_recovery_efficiency(recovery: &ShowerHeatRecovery, residential: bool) -> f64 {
    let count = recovery.showers.len();
    if count == 0 {
        return 0.0;
    }
    let efficiencies: Vec<f64> = recovery
        .showers
        .iter()
        .map(ShowerUnit::efficiency)
        .collect();
    let connected: Vec<f64> = efficiencies
        .iter()
        .copied()
        .filter(|eta| *eta > 0.0)
        .collect();
    if !residential && recovery.assignment_unknown && connected.len() as f64 > 0.8 * count as f64 {
        return connected.iter().copied().fold(f64::INFINITY, f64::min);
    }
    efficiencies.iter().sum::<f64>() / count as f64
}

/// Annual net need `Q_W;nd` in kWh (13.15/13.19) and the shower share.
fn annual_need(system: &HotWaterSystem, area: f64) -> (f64, f64) {
    match &system.need {
        HotWaterNeed::Residential { dwelling_count, .. } => {
            let dwellings = f64::from(*dwelling_count);
            let need = RESIDENTIAL_NEED_PER_OCCUPANT_KWH
                * dwellings
                * occupants_per_dwelling(area / dwellings);
            (need, shower_share(LabelFunction::Residential))
        }
        HotWaterNeed::Utility { areas, .. } => {
            let mut need = 0.0;
            let mut shower = 0.0;
            for item in areas {
                let part = utility_specific_need(item.function).unwrap_or(0.0) * item.area_m2;
                need += part;
                shower += part * shower_share(item.function);
            }
            (need, if need > 0.0 { shower / need } else { 0.0 })
        }
    }
}

fn emission_efficiency(system: &HotWaterSystem) -> f64 {
    match &system.emission {
        HotWaterEmission::Residential {
            served,
            kitchen_length_m,
            kitchen_pipe_diameter,
            bathroom_length_m,
            ..
        } => {
            let kitchen =
                kitchen_emission_for(kitchen_length_m.unwrap_or(0.0), *kitchen_pipe_diameter);
            let bathroom = bathroom_emission(bathroom_length_m.unwrap_or(0.0));
            match served {
                ServedTaps::BathroomOnly => bathroom,
                ServedTaps::KitchenOnly => kitchen,
                // 13.23
                ServedTaps::KitchenAndBathroom => {
                    1.0 / (BATHROOM_SHARE / bathroom + KITCHEN_SHARE / kitchen)
                }
            }
        }
        HotWaterEmission::Utility { mean_length_m, .. } => {
            if *mean_length_m <= 3.0 {
                1.0
            } else {
                0.8
            }
        }
    }
}

/// Generation efficiency (before `f_prac`), `f_prac`, and an issue code when
/// the generator cannot serve the annual demand.
///
/// The measured and declared routes round their value down (to 0,025 for gas,
/// 0,05 for electric, §13.8.4.7.2); a value below one step rounds to 0, which
/// no generator can have and which would divide the carrier input by zero.
/// Such an efficiency is refused with `hot_water_efficiency_invalid`.
fn generation(
    generator: &HotWaterGenerator,
    annual_output_kwh: f64,
) -> Result<(f64, f64), &'static str> {
    let (efficiency, practical) = generation_values(generator, annual_output_kwh)?;
    if efficiency > 0.0 && efficiency.is_finite() && practical > 0.0 {
        Ok((efficiency, practical))
    } else {
        Err("hot_water_efficiency_invalid")
    }
}

fn generation_values(
    generator: &HotWaterGenerator,
    annual_output_kwh: f64,
) -> Result<(f64, f64), &'static str> {
    match generator {
        // Table 13.22, including the vessel losses.
        HotWaterGenerator::BiomassCombi {
            insulation,
            inside_boundary,
        } => Ok((
            match (insulation, inside_boundary) {
                (BiomassStoreInsulation::AtLeast20Mm, true) => 0.65,
                (BiomassStoreInsulation::AtLeast20Mm, false) => 0.60,
                (BiomassStoreInsulation::AtLeast10Mm, true) => 0.575,
                (BiomassStoreInsulation::AtLeast10Mm, false) => 0.525,
                (BiomassStoreInsulation::None, true) => 0.35,
                (BiomassStoreInsulation::None, false) => 0.325,
            },
            1.0,
        )),
        HotWaterGenerator::GasAppliance {
            appliance,
            measured_class,
            kitchen_only,
            declared,
            annex_t,
            ..
        } => {
            let (table, corrected) = appliance.table();
            let base = match (annex_t, declared) {
                (Some(test), _) => round_down(
                    test.efficiency().map_err(|_| "annex_t_value_invalid")?,
                    0.025,
                ),
                (None, Some(item)) => round_down(item.value, 0.025),
                (None, None) => table,
            };
            // Table 13.25 note 3 d: every measured value holds for its class.
            let measured = annex_t.is_some() || declared.is_some();
            let correction = if *kitchen_only || !(corrected || measured) {
                1.0
            } else {
                gas_class_correction(
                    measured_class.unwrap_or(ApplicationClass::Class4),
                    annual_output_kwh,
                )
            };
            Ok((base * correction, 1.0))
        }
        HotWaterGenerator::HeatPump {
            source_correction,
            measured_class,
            declared,
            ..
        } => {
            // §13.8.4.7.2 (p. 640): a declared value replaces the table
            // value, rounded down to a multiple of 0,05 (electric). A value
            // declared for one tapping class takes c_W;gen for that class;
            // without a class it is the declaration's value interpolated
            // for this system's demand (allowed before rounding), so no
            // class correction applies.
            let correction = if declared.is_some() && measured_class.is_none() {
                1.0
            } else {
                heat_pump_class_correction(
                    measured_class.unwrap_or(ApplicationClass::Class4),
                    annual_output_kwh,
                )
                .ok_or("hot_water_heat_pump_class_exceeded")?
            };
            // 13.152 (p. 616–617): f_prac;gi = 1,0 only for the forfait
            // values of 13.8.4.5–13.8.4.7; a declared value is not a forfait
            // value, so "alle overige gevallen" (0,95) applies.
            let (base, practical) = match declared {
                Some(item) => (round_down(item.value, 0.05), 0.95),
                None => (1.4 * source_correction.unwrap_or(1.0), 1.0),
            };
            Ok((base * correction, practical))
        }
        HotWaterGenerator::HeatPumpEn16147 {
            profile,
            delivered_kwh_per_day,
            input_kwh_per_day,
            storage_without_legionella_cycle,
            smart_control_factor,
            max_test_temperature_c,
            design_set_temperature_c,
            ..
        } => {
            let correction = european_profile_correction(*profile, annual_output_kwh)
                .ok_or("hot_water_heat_pump_class_exceeded")?;
            // 13.153b (C_W;mixed air = 1) and 13.153c.
            let input = en16147_corrected_input(
                *input_kwh_per_day,
                *smart_control_factor,
                *max_test_temperature_c,
                *design_set_temperature_c,
            );
            let efficiency = round_down(delivered_kwh_per_day * correction / input, 0.05);
            let practical = if *storage_without_legionella_cycle {
                0.9
            } else {
                0.95
            };
            Ok((efficiency, practical))
        }
        HotWaterGenerator::ElectricInstantaneous => Ok((0.95, 1.0)),
        HotWaterGenerator::ElectricBoiler => Ok((1.0, 1.0)),
        // Monthly in 13.165; see the month loop.
        HotWaterGenerator::GasStorageHeater {
            measured_standby_kwh_per_day,
            ..
        } => Ok((
            1.0,
            if measured_standby_kwh_per_day.is_some() {
                0.95
            } else {
                1.0
            },
        )),
        HotWaterGenerator::LargeDirectStorage { .. } => Ok((0.50, 1.0)),
        HotWaterGenerator::IndirectBoiler {
            boiler,
            inside_boundary,
            declared,
            ..
        } => {
            let table = match (boiler, inside_boundary) {
                (IndirectBoiler::ConventionalOrUnknown, true) => 0.75,
                (IndirectBoiler::Vr, true) => 0.80,
                (IndirectBoiler::Hr100Or104, true) => 0.85,
                (IndirectBoiler::Hr107, true) => 0.90,
                (IndirectBoiler::ConventionalOrUnknown, false) => 0.70,
                (IndirectBoiler::Vr, false) => 0.75,
                (IndirectBoiler::Hr100Or104, false) => 0.80,
                (IndirectBoiler::Hr107, false) => 0.85,
            };
            Ok((
                declared
                    .as_ref()
                    .map_or(table, |item| round_down(item.value, 0.025)),
                1.0,
            ))
        }
        HotWaterGenerator::IndirectHeatPump { .. } => Ok((1.4, 1.0)),
        HotWaterGenerator::HeatPumpSeries { last_source } => {
            crate::forfait_heat_pump_draft::utility_cop_65_to_70(*last_source)
                .map(|cop| (cop, 1.0))
                .ok_or("hot_water_heat_pump_series_source_invalid")
        }
        HotWaterGenerator::ExternalHeat => Ok((1.0, 1.0)),
        // Annex W per month; see the month loop.
        HotWaterGenerator::BoosterHeatPump(_) | HotWaterGenerator::BoosterHeatPumpForfait(_) => {
            Ok((1.0, 1.0))
        }
        // 13.157 needs the appliance's own output (see the booking); here
        // the system total serves the ordering and single-unit reporting.
        HotWaterGenerator::MeasuredTwoProfiles(test) => Ok((
            test.efficiency(annual_output_kwh / 365.0)?,
            test.practical_factor(),
        )),
        // 13.8.4.9.3: no hot-water efficiency.
        HotWaterGenerator::HeatingSystem => Ok((1.0, 1.0)),
        // §13.8.4.7.4: ε_chp;th (HT); method 1 per month in the booking,
        // here its full-load value serves the ordering.
        HotWaterGenerator::Chp(chp) => match (&chp.chp, &chp.method1) {
            (Some(_), None) => chp
                .forfait_factors()
                .map(|(thermal, _)| (thermal, 1.0))
                .ok_or("chp_class_invalid"),
            (None, Some(product)) => crate::micro_chp::nominal_gross_efficiency(product)
                .map(|value| (value, 1.0))
                .ok_or("micro_chp_efficiency_required"),
            _ => Err("hot_water_chp_method_required"),
        },
    }
}

/// Output of one generator after the dispatch of 13.8.2.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotWaterGeneratorResult {
    /// 0 for the main generator, 1.. for the additional generators.
    pub index: usize,
    /// Dispatch position (13.8.2.1), 0 first.
    pub order: usize,
    /// `Q_W;gen;gi;out;mi`, kWh.
    pub monthly_output_kwh: Vec<f64>,
    /// `F_W;gen;si,gi,mi` (13.150).
    pub monthly_share: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotWaterAssessment {
    pub annual_net_need_kwh: f64,
    pub emission_efficiency: f64,
    pub annual_generator_output_kwh: f64,
    pub months: Vec<HotWaterMonth>,
    pub generators: Vec<HotWaterGeneratorResult>,
    pub annual_solar_renewable_kwh: f64,
    pub annual_solar_space_heating_kwh: f64,
    /// 13.148/13.149 data of an exhaust-air heat pump for chapter 11.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exhaust_air: Option<ExhaustAirHotWater>,
    /// §13.8.4.8 (p. 650): the space-heating share of a combi micro-CHP;
    /// it replaces the chain's own CHP booking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combi_chp_heating: Option<Vec<CombiChpHeatingMonth>>,
}

/// 13.149 `f_W;t;hp-on;mi` and the 13.148/13.148a flow data of the
/// exhaust-air heat pump of the hot-water system.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExhaustAirHotWater {
    /// `Q_W;gen;gi;out;mi / Q_W;gen;gi;Pout;max;mi` (13.149).
    pub time_fraction: Vec<f64>,
    /// `f_combi;mi = 0` in every month: the forfait flow of 13.148 applies.
    pub hot_water_only: bool,
    /// `q_ve;hp;W` of a quality declaration (13.148a), m³/h.
    pub declared_flow_m3_per_h: Option<f64>,
}

/// One generator of the system with its 13.8.2 data.
struct Unit<'a> {
    index: usize,
    generator: &'a HotWaterGenerator,
    nominal_power_kw: Option<f64>,
    exhaust_air: Option<&'a ExhaustAirUse>,
    declared_share: Option<&'a DeclaredGeneratorShare>,
}

fn units(system: &HotWaterSystem) -> Vec<Unit<'_>> {
    std::iter::once(Unit {
        index: 0,
        generator: &system.generator,
        nominal_power_kw: system.nominal_power_kw,
        exhaust_air: system.exhaust_air.as_ref(),
        declared_share: system.declared_share.as_ref(),
    })
    .chain(
        system
            .additional_generators
            .iter()
            .enumerate()
            .map(|(index, unit)| Unit {
                index: index + 1,
                generator: &unit.generator,
                nominal_power_kw: unit.nominal_power_kw,
                exhaust_air: unit.exhaust_air.as_ref(),
                declared_share: unit.declared_share.as_ref(),
            }),
    )
    .collect()
}

/// 13.8.2.1 categories: (a) exhaust-air heat pump without outdoor air,
/// (b) heat pumps and CHP (biomass boilers do not occur here), (c) others.
fn category(generator: &HotWaterGenerator) -> u8 {
    match generator {
        HotWaterGenerator::HeatPump {
            exhaust_air_source: true,
            outdoor_air_fraction,
            ..
        }
        | HotWaterGenerator::HeatPumpEn16147 {
            exhaust_air_source: true,
            outdoor_air_fraction,
            ..
        } if outdoor_air_fraction.unwrap_or(0.0) <= 0.0 => 0,
        HotWaterGenerator::MeasuredTwoProfiles(test) if test.electric() => {
            if test.exhaust_air_source && test.outdoor_air_fraction.unwrap_or(0.0) <= 0.0 {
                0
            } else {
                1
            }
        }
        HotWaterGenerator::HeatPump { .. }
        | HotWaterGenerator::HeatPumpEn16147 { .. }
        | HotWaterGenerator::IndirectHeatPump { .. }
        | HotWaterGenerator::HeatPumpSeries { .. }
        | HotWaterGenerator::BoosterHeatPump(_)
        | HotWaterGenerator::BoosterHeatPumpForfait(_)
        | HotWaterGenerator::Chp(_) => 1,
        _ => 2,
    }
}

/// Monthly booking of one generator (13.3, 13.181, 5.36/5.37, 13.179,
/// annex W).
#[derive(Default)]
struct Booking {
    input: [f64; 12],
    auxiliary: [f64; 12],
    ambient: [f64; 12],
    recoverable: [f64; 12],
    /// Denominator of the reported efficiency (annex W: heat plus
    /// electricity).
    efficiency_input: [f64; 12],
    /// 13.185: output supplied by the space-heating system.
    heating_system: [f64; 12],
    /// 16.13/16.16: CHP electricity.
    chp_electricity: [f64; 12],
    /// 9.66: CHP heat above full load (no electricity).
    chp_excess: [f64; 12],
    /// Q_W;gen;out of a table 13.22 biomass appliance.
    biomass_output: [f64; 12],
    /// §13.8.4.8: the heating share of a combi micro-CHP.
    combi_heating: Option<[CombiChpHeatingMonth; 12]>,
}

/// The heat source of an annex W or forfait booster heat pump.
fn booster_heat_source(generator: &HotWaterGenerator) -> Option<&BoosterHeatSource> {
    match generator {
        HotWaterGenerator::BoosterHeatPump(pump) => Some(&pump.heat_source),
        HotWaterGenerator::BoosterHeatPumpForfait(pump) => Some(&pump.heat_source),
        _ => None,
    }
}

/// Booster heat pump per month: annex W, or the forfait of 13.162/13.163
/// with `c_W;gen` of class 4 (table 13.27) on the annual gross demand.
fn booster_months(
    generator: &HotWaterGenerator,
    outputs: &[f64; 12],
    annual_total: f64,
) -> Option<[BoosterMonth; 12]> {
    match generator {
        HotWaterGenerator::BoosterHeatPump(pump) => Some(calculate_booster(pump, outputs)),
        HotWaterGenerator::BoosterHeatPumpForfait(pump) => {
            let class = heat_pump_class_correction(ApplicationClass::Class4, annual_total)?;
            Some(calculate_booster_forfait(pump, outputs, class))
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn book_generator(
    generator: &HotWaterGenerator,
    system: &HotWaterSystem,
    context: HotWaterContext,
    extras: &HotWaterExtras,
    f_building: f64,
    building_area: f64,
    recoverable_counts: bool,
    outputs: &[f64; 12],
    annual_total: f64,
) -> Result<Booking, &'static str> {
    let mut booking = Booking::default();
    // (13.157)/(13.160): f_gebouw;si;W in the daily heat and the auxiliary
    // energy from 2025+C1 (p. 627–628); NTA 8800:2024 divides by 365 and
    // leaves the auxiliary energy unweighted (p. 611–612).
    let f_daily = if crate::norm_versions::profile().hot_water_building_share {
        f_building
    } else {
        1.0
    };
    // 13.8.4.9.3: the space-heating system supplies the output (13.185);
    // hot water keeps no carrier, auxiliary energy or recoverable loss.
    if matches!(generator, HotWaterGenerator::HeatingSystem) {
        booking.heating_system = *outputs;
        return Ok(booking);
    }
    // 13.157 with the appliance's own output; per month with
    // `C_W;mixed air;mi` (13.153b) and `E_W;gen;in;PFHRD` (13.156a).
    let mut monthly_two_profile: Option<[f64; 12]> = None;
    let (base, practical) = match generator {
        HotWaterGenerator::MeasuredTwoProfiles(test) => {
            let daily = outputs.iter().sum::<f64>() / (365.0 * f_daily);
            if daily > 0.0 {
                let pfhrd = test.pfhrd_daily(extras);
                let mut monthly = [0.0; 12];
                for (index, value) in monthly.iter_mut().enumerate() {
                    let mixed = test
                        .mixed_air
                        .as_ref()
                        .map_or(1.0, |item| item.factor(index, extras.mixed_air.as_ref()));
                    *value = test.efficiency_with(daily, mixed, pfhrd)?;
                }
                monthly_two_profile = Some(monthly);
                (monthly[0], test.practical_factor())
            } else {
                (1.0, 1.0)
            }
        }
        _ => generation(generator, annual_total)?,
    };
    for index in 0..12 {
        let output = outputs[index];
        let mut efficiency = monthly_two_profile.map_or(base, |values| values[index]);
        if let HotWaterGenerator::GasStorageHeater {
            volume_l,
            measured_standby_kwh_per_day,
            before_1985,
            in_heated_zone,
        } = generator
        {
            // 13.165–13.175 with η_100 % = 0,84 and Hs/Hi = 1,11.
            let standby = measured_standby_kwh_per_day.unwrap_or_else(|| {
                let base = 2.0 + 0.033 * volume_l.powf(1.1);
                if *before_1985 {
                    1.4 * base
                } else {
                    base
                }
            });
            let mean = if system.circulation.is_some() {
                60.0
            } else {
                55.0
            };
            // 13.168: ϑ_int;set;H;zi,mi after levelling (7.9.4) in a heated
            // zone, 13 °C otherwise.
            let ambient = if *in_heated_zone {
                context
                    .levelled_setpoint_c
                    .map_or(context.heated_ambient_c, |values| values[index])
            } else {
                unheated_ambient(system, None, context.heated_ambient_c, index)
            };
            let standing =
                f_building * (mean - ambient) / 45.0 * MONTH_HOURS[index] / 24.0 * standby;
            let generation_loss = (1.11 - 0.84) / 0.84 * output;
            efficiency = if output > 0.0 {
                round_down(output / (output + generation_loss + standing), 0.025)
            } else {
                1.0
            };
        }
        let practical_efficiency = practical * efficiency;
        booking.input[index] = output / practical_efficiency;
        booking.efficiency_input[index] = booking.input[index];
        if matches!(generator, HotWaterGenerator::BiomassCombi { .. }) {
            booking.biomass_output[index] = output;
        }
        // §13.8.4.7.4 with 9.6.2.1: pilot gas of a hot-water-only gas boiler.
        if let HotWaterGenerator::IndirectBoiler {
            pilot_flame: true,
            oil: false,
            also_space_heating: false,
            ..
        } = generator
        {
            booking.input[index] += crate::boiler_forfait_draft::PILOT_FLAME_ANNUAL_KWH
                * MONTH_HOURS[index]
                / crate::climate::YEAR_HOURS
                * f_building;
        }
        // 13.181 for generators whose auxiliaries are not in the efficiency.
        let (electronics, burner) = match generator {
            HotWaterGenerator::ElectricInstantaneous => (STANDBY_ELECTRONICS_W, 0.0),
            HotWaterGenerator::GasStorageHeater { .. } => (STANDBY_ELECTRONICS_W, 1.0),
            HotWaterGenerator::LargeDirectStorage { gas_fired } => {
                (STANDBY_ELECTRONICS_W, if *gas_fired { 1.0 } else { 0.0 })
            }
            HotWaterGenerator::IndirectBoiler {
                also_space_heating, ..
            } => (
                if *also_space_heating {
                    0.0
                } else {
                    STANDBY_ELECTRONICS_W
                },
                1.0,
            ),
            HotWaterGenerator::Chp(chp) if chp.method1.is_none() => (
                if chp.also_space_heating {
                    0.0
                } else {
                    STANDBY_ELECTRONICS_W
                },
                1.0,
            ),
            HotWaterGenerator::IndirectHeatPump { also_space_heating } => (
                if *also_space_heating {
                    0.0
                } else {
                    STANDBY_ELECTRONICS_W
                },
                0.0,
            ),
            _ => (0.0, 0.0),
        };
        booking.auxiliary[index] +=
            (electronics * MONTH_HOURS[index] * f_building + burner * output * 1.1) / 1000.0;
        // 5.36/5.37
        let renewable_share = match generator {
            HotWaterGenerator::HeatPump {
                exhaust_air_source,
                outdoor_air_fraction,
                ..
            }
            | HotWaterGenerator::HeatPumpEn16147 {
                exhaust_air_source,
                outdoor_air_fraction,
                ..
            } => {
                if *exhaust_air_source {
                    outdoor_air_fraction.unwrap_or(0.0)
                } else {
                    1.0
                }
            }
            HotWaterGenerator::IndirectHeatPump { .. } => 1.0,
            HotWaterGenerator::HeatPumpSeries { last_source } => {
                if *last_source == crate::forfait_heat_pump_draft::TableSource::ExhaustAir {
                    0.0
                } else {
                    1.0
                }
            }
            HotWaterGenerator::MeasuredTwoProfiles(test) if test.electric() => {
                if test.exhaust_air_source {
                    test.outdoor_air_fraction.unwrap_or(0.0)
                } else {
                    1.0
                }
            }
            _ => 0.0,
        };
        if renewable_share > 0.0 && practical_efficiency >= 1.0 {
            booking.ambient[index] = output * (1.0 - 1.0 / practical_efficiency) * renewable_share;
        }
        // 13.179: electric instantaneous heaters (individual appliances).
        if matches!(generator, HotWaterGenerator::ElectricInstantaneous) && recoverable_counts {
            booking.recoverable[index] += output / practical_efficiency - output;
        }
    }
    // Annex W: the carrier input is the heat drawn from the collective
    // heating system (W.2) divided by its generation efficiency; the BWP
    // electricity (W.1) is auxiliary energy.
    if let (Some(booster), Some(heat_source)) = (
        booster_months(generator, outputs, annual_total),
        booster_heat_source(generator),
    ) {
        let from_heating_system = matches!(heat_source, BoosterHeatSource::HeatingSystem);
        let source_efficiency = match heat_source {
            BoosterHeatSource::ExternalHeat | BoosterHeatSource::HeatingSystem => 1.0,
            BoosterHeatSource::CollectiveGenerator {
                generation_efficiency,
                ..
            } => *generation_efficiency,
        };
        for (index, month) in booster.iter().enumerate() {
            // 13.6.5 with 13.164: the booster's standing loss Q_W;hp;ls
            // (annex W, already in kWh) replaces Q_sto;ls, rounded down per
            // annex X, subject to the 500 m² rule of 13.13; the booster is
            // taken to stand in the heated zone.
            if recoverable_counts {
                booking.recoverable[index] +=
                    crate::significant_figures::round_down(month.standing_loss_heat_kwh);
            }
            if from_heating_system {
                // 9.4: Q_W;BWP;si;in = E_W;gen;in;prac;BWP loads the
                // space-heating node; the heating generator supplies it.
                booking.input[index] = 0.0;
                booking.heating_system[index] += month.heating_system_heat_kwh;
            } else {
                booking.input[index] = month.heating_system_heat_kwh / source_efficiency;
            }
            booking.efficiency_input[index] = month.heating_system_heat_kwh + month.electricity_kwh;
            booking.auxiliary[index] += month.electricity_kwh;
            booking.ambient[index] = 0.0;
        }
    }
    // §13.8.4.7.4 and §13.8.4.8: building CHP.
    if let HotWaterGenerator::Chp(chp) = generator {
        match (&chp.chp, &chp.method1) {
            (Some(_), None) => {
                // 16.13: E_el;chp;out;W = Q_W;gen;out·ε_chp;el/ε_chp;th.
                let (thermal, electric) = chp.forfait_factors().ok_or("chp_class_invalid")?;
                for (index, output) in outputs.iter().enumerate() {
                    booking.chp_electricity[index] = output * electric / thermal;
                }
            }
            (None, Some(product)) => {
                let collective = system.collective.is_some();
                let functioning = if building_area > 500.0 { 0.6 } else { 1.0 };
                for (index, output) in outputs.iter().enumerate() {
                    if *output <= 0.0 {
                        booking.input[index] = 0.0;
                        booking.efficiency_input[index] = 0.0;
                        continue;
                    }
                    // 13.183 and 9.6.6.2 with Q_W;gen;out and t_W;op.
                    let hours = crate::micro_chp::hot_water_operating_hours(
                        product,
                        *output,
                        f_building,
                        functioning,
                        MONTH_HOURS[index],
                    );
                    // p. 650: a combi appliance has one E_H;gen;in, split
                    // over heating and hot water by output; one 9.6.6.2
                    // month on Q_H + Q_W with t_H;op + t_W;op.
                    let combi = extras.combi_chp.as_ref().filter(|_| chp.also_space_heating);
                    let (month, water_share) = match combi {
                        Some(heating) => {
                            let q_h = heating.thermal_output_kwh[index].max(0.0);
                            let total = q_h + output;
                            let joint = crate::micro_chp::micro_chp_month(
                                &heating.product,
                                total,
                                (heating.operating_hours[index] + hours).min(MONTH_HOURS[index]),
                                MONTH_HOURS[index],
                                f_building,
                                collective,
                            )
                            .ok_or("micro_chp_efficiency_required")?;
                            let share = output / total;
                            let combi_heating = booking.combi_heating.get_or_insert_with(|| {
                                std::array::from_fn(|month| CombiChpHeatingMonth {
                                    month: month as u8 + 1,
                                    ..CombiChpHeatingMonth::default()
                                })
                            });
                            combi_heating[index] = CombiChpHeatingMonth {
                                month: index as u8 + 1,
                                input_kwh: joint.input_kwh * (1.0 - share),
                                electricity_kwh: joint.electricity_kwh * (1.0 - share),
                                auxiliary_kwh: joint
                                    .auxiliary_kwh
                                    .map(|value| value * (1.0 - share)),
                            };
                            (joint, share)
                        }
                        None => (
                            crate::micro_chp::micro_chp_month(
                                product,
                                *output,
                                hours,
                                MONTH_HOURS[index],
                                f_building,
                                collective,
                            )
                            .ok_or("micro_chp_efficiency_required")?,
                            1.0,
                        ),
                    };
                    let month = crate::micro_chp::MicroChpMonth {
                        input_kwh: month.input_kwh * water_share,
                        electricity_kwh: month.electricity_kwh * water_share,
                        auxiliary_kwh: month.auxiliary_kwh.map(|value| value * water_share),
                        recoverable_kwh: month.recoverable_kwh * water_share,
                        excess_kwh: month.excess_kwh * water_share,
                        ..month
                    };
                    booking.chp_excess[index] = month.excess_kwh;
                    // 13.182 rounded down to 0,025.
                    let efficiency = round_down(output / month.input_kwh, 0.025);
                    booking.input[index] = output / efficiency;
                    booking.efficiency_input[index] = booking.input[index];
                    // 13.8.4.8.3: W_H;gen;aux × f_gebouw (in the month result).
                    booking.auxiliary[index] += match (month.auxiliary_kwh, &chp.auxiliary) {
                        (Some(value), _) => value,
                        // 9.6.6.2.2.3 → 9.6.8 (9.91/9.92).
                        (None, Some(auxiliary)) => {
                            crate::space_heating_chain::other_generator_auxiliary_kwh(
                                auxiliary,
                                crate::space_heating_chain::OTHER_AUX_GAS_OIL_W_PER_KW,
                                *output,
                                MONTH_HOURS[index],
                                f_building,
                            )
                        }
                        (None, None) => return Err("hot_water_chp_auxiliary_required"),
                    };
                    // 13.8.4.8.4, subject to the 500 m² rule of 13.13.
                    if recoverable_counts {
                        booking.recoverable[index] += month.recoverable_kwh * f_building;
                    }
                    // 16.16: P_el;chp;out;W · t_W;op.
                    booking.chp_electricity[index] = month.electricity_kwh;
                }
            }
            _ => return Err("hot_water_chp_method_required"),
        }
    }
    // 13.159/13.160 and 13.160a.
    if let HotWaterGenerator::MeasuredTwoProfiles(test) = generator {
        let daily = outputs.iter().sum::<f64>() / (365.0 * f_daily);
        let year: f64 = MONTH_HOURS.iter().sum();
        let auxiliary = test.daily_auxiliary(daily);
        let recoverable = test.daily_recoverable();
        for (index, hours) in MONTH_HOURS.iter().enumerate() {
            let days = 365.0 * hours / year;
            booking.auxiliary[index] += auxiliary * days * f_daily;
            // 13.160a: × A_g;zi,si / A_g;si;W summed over the zones.
            if recoverable_counts {
                booking.recoverable[index] += recoverable * days * f_building;
            }
        }
    }
    Ok(booking)
}

/// Monthly totals of all solar systems on the hot-water system.
#[derive(Default)]
struct SolarTotals {
    /// Σ 13.66 `Q_W;ren;prac`.
    water: [f64; 12],
    /// Σ 13.66a `Q_H;ren;prac`.
    space_heating: [f64; 12],
    /// 13.8: Σ `Q_W;bu;sto;ls × f_gebouw;si;W`.
    backup_storage_loss: [f64; 12],
    /// 13.67.
    auxiliary: [f64; 12],
    /// 13.68.
    recoverable: [f64; 12],
}

/// 13.69/13.69a and 13.137a/b: `ϑ_sto;amb` of a solar vessel in a heated
/// room is `ϑ_int;set;H;stc` with an exhaust-air heat pump for hot water,
/// otherwise the levelled `ϑ_int;set;H;zi,mi` of 7.9.4.
fn solar_storage_ambient(system: &HotWaterSystem, context: HotWaterContext) -> [f64; 12] {
    // (13.69a)/(13.137a) are new in NTA 8800:2023 (p. 565, 592).
    let exhaust_air = crate::norm_versions::profile().storage_ambient_exhaust_air
        && units(system)
            .iter()
            .any(|unit| exhaust_air_heat_pump(unit.generator));
    solar_ambient(exhaust_air, context)
}

fn solar_ambient(exhaust_air: bool, context: HotWaterContext) -> [f64; 12] {
    std::array::from_fn(|index| {
        if exhaust_air {
            context
                .standard_setpoint_c
                .unwrap_or(context.heated_ambient_c)
        } else {
            context
                .levelled_setpoint_c
                .map_or(context.heated_ambient_c, |values| values[index])
        }
    })
}

/// §13.7 for all solar systems; `before` is `Q_W;dis` without the backup
/// losses of solar vessels (`Q_W;sol;us`, 13.77).
#[allow(clippy::too_many_arguments)]
fn solar_contribution(
    heaters: &[SolarWaterHeater],
    storage_ambient: [f64; 12],
    context: HotWaterContext,
    f_building: f64,
    hot_water_c: f64,
    before: &[f64; 12],
    path: &str,
) -> Result<SolarTotals, HotWaterIssue> {
    let mut totals = SolarTotals::default();
    if heaters.is_empty() {
        return Ok(totals);
    }
    // 13.77/13.96: different systems share a demand by V_sto;tot, identical
    // ones per physical system; a space-heating-only system (SHS) takes no
    // hot water and a water-only system no space heating.
    let weight = |water: bool| -> f64 {
        heaters
            .iter()
            .filter(|heater| match heater.solar_use {
                SolarUse::WaterHeating => water,
                SolarUse::Combi => true,
                SolarUse::SpaceHeating => !water,
            })
            .map(|heater| heater.method.total_volume_l() * f64::from(heater.count))
            .sum()
    };
    let (water_weight, heating_weight) = (weight(true), weight(false));
    for (position, heater) in heaters.iter().enumerate() {
        let count = f64::from(heater.count);
        let volume = heater.method.total_volume_l() * count;
        let space_only = heater.solar_use == SolarUse::SpaceHeating;
        let share = if space_only || water_weight <= 0.0 {
            0.0
        } else {
            volume / water_weight
        };
        let heating_share = if heater.solar_use == SolarUse::WaterHeating || heating_weight <= 0.0 {
            0.0
        } else {
            volume / heating_weight
        };
        let heating = context
            .space_heating
            .filter(|_| heater.solar_use != SolarUse::WaterHeating);
        // 13.77/13.85: f_W;use per month (SOL_SYS = SH: f_H;use = 1).
        let water_share: [f64; 12] = std::array::from_fn(|index| match &heating {
            _ if space_only => 0.0,
            Some(heating) => {
                let total = before[index] + heating.node_kwh[index];
                if total > 0.0 {
                    before[index] / total
                } else {
                    0.0
                }
            }
            None => 1.0,
        });
        let water_use: [f64; 12] =
            std::array::from_fn(|index| before[index] * share / (f_building * count));
        let pump_hours = match heater.solar_use {
            SolarUse::WaterHeating => PUMP_HOURS_WATER,
            SolarUse::Combi | SolarUse::SpaceHeating => PUMP_HOURS_COMBI,
        };
        let (water, space): ([ServiceMonth; 12], [ServiceMonth; 12]) = match &heater.method {
            SolarMethod::Calculated {
                solar_type,
                collectors,
                storage,
            } => {
                let water = calculated_service(
                    *solar_type,
                    collectors,
                    storage,
                    &ServiceSettings {
                        use_kwh: water_use,
                        share: water_share,
                        reference_c: std::array::from_fn(water_reference_c),
                        low_c: [COLD_WATER_C; 12],
                        high_c: hot_water_c,
                        backup_set_c: hot_water_c,
                        ambient_c: storage_ambient,
                        pump_hours,
                        add_backup_loss_to_use: false,
                    },
                );
                let space = match &heating {
                    Some(heating) => calculated_service(
                        *solar_type,
                        collectors,
                        storage,
                        &ServiceSettings {
                            use_kwh: std::array::from_fn(|index| {
                                heating.node_kwh[index] * heating_share
                                    / (heating.building_fraction * count)
                            }),
                            share: std::array::from_fn(|index| {
                                if space_only {
                                    1.0
                                } else if before[index] + heating.node_kwh[index] > 0.0 {
                                    1.0 - water_share[index]
                                } else {
                                    0.0
                                }
                            }),
                            reference_c: [heating_reference_c(heating.design_return_c); 12],
                            low_c: storage_ambient,
                            high_c: heating.design_return_c,
                            backup_set_c: heating.design_supply_c,
                            ambient_c: storage_ambient,
                            pump_hours: PUMP_HOURS_COMBI,
                            add_backup_loss_to_use: true,
                        },
                    ),
                    None => [ServiceMonth::default(); 12],
                };
                (water, space)
            }
            SolarMethod::Tested {
                solar_type,
                orientation,
                tilt_deg,
                obstruction,
                test_points,
                backup_loss_in_generator_efficiency,
                ..
            } => {
                let water = tested_water(
                    *solar_type,
                    *orientation,
                    *tilt_deg,
                    obstruction,
                    test_points,
                    *backup_loss_in_generator_efficiency,
                    &water_use,
                    hot_water_c,
                    storage_ambient,
                )
                .ok_or_else(|| HotWaterIssue {
                    code: "solar_test_out_of_range",
                    path: format!("{path}[{position}].method.testPoints"),
                })?;
                (water, [ServiceMonth::default(); 12])
            }
        };
        let pvt = heater.pvt_factor().unwrap_or(1.0);
        let heating_fraction = heating.map_or(1.0, |item| item.building_fraction);
        for index in 0..12 {
            // 13.66/13.66a with the physical systems summed.
            totals.water[index] +=
                f_building * SOLAR_PRACTICE_FACTOR * water[index].renewable_kwh * pvt * count;
            totals.space_heating[index] +=
                heating_fraction * SOLAR_PRACTICE_FACTOR * space[index].renewable_kwh * pvt * count;
            totals.backup_storage_loss[index] +=
                f_building * water[index].backup_storage_loss_kwh * count;
            totals.auxiliary[index] +=
                f_building * (water[index].auxiliary_kwh + space[index].auxiliary_kwh) * count;
            totals.recoverable[index] +=
                f_building * (water[index].recoverable_kwh + space[index].recoverable_kwh) * count;
        }
    }
    Ok(totals)
}

/// Monthly result of solar systems for space heating only that are not
/// part of a hot-water system (SHS, 13.85 with `f_H;use = 1`).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandaloneSolarHeating {
    /// 13.66a `Q_H;ren;prac` for the space-heating node (9.2.3.4), kWh.
    pub space_heating_kwh: Vec<f64>,
    /// 13.67 pump energy, kWh.
    pub auxiliary_kwh: Vec<f64>,
    /// 13.68 recoverable losses (reported), kWh.
    pub recoverable_kwh: Vec<f64>,
}

/// Validation of standalone space-heating solar systems.
pub fn validate_standalone_solar(heaters: &[SolarWaterHeater], path: &str) -> Vec<HotWaterIssue> {
    let mut issues = Vec::new();
    for (index, heater) in heaters.iter().enumerate() {
        let prefix = format!("{path}[{index}]");
        if heater.solar_use != SolarUse::SpaceHeating {
            issues.push(HotWaterIssue {
                code: "solar_standalone_space_heating_only",
                path: format!("{prefix}.solarUse"),
            });
        }
        if !matches!(heater.method, SolarMethod::Calculated { .. }) {
            // 13.7.2.3 tested systems cover hot water only.
            issues.push(HotWaterIssue {
                code: "solar_tested_water_only",
                path: format!("{prefix}.method"),
            });
        }
        issues.extend(
            crate::solar_thermal::validate_solar(heater, &prefix)
                .into_iter()
                .map(|item| HotWaterIssue {
                    code: item.code,
                    path: item.path,
                }),
        );
    }
    issues
}

/// §13.7 for space-heating-only solar systems without a hot-water system;
/// `context.space_heating` carries the node of a chain run without solar
/// gains. Call after [`validate_standalone_solar`] returned no issues.
pub fn assess_standalone_solar(
    heaters: &[SolarWaterHeater],
    context: HotWaterContext,
    path: &str,
) -> Result<StandaloneSolarHeating, HotWaterIssue> {
    let fraction = context
        .space_heating
        .map_or(1.0, |heating| heating.building_fraction);
    let totals = solar_contribution(
        heaters,
        solar_ambient(false, context),
        context,
        fraction,
        60.0,
        &[0.0; 12],
        path,
    )?;
    Ok(StandaloneSolarHeating {
        space_heating_kwh: totals.space_heating.to_vec(),
        auxiliary_kwh: totals.auxiliary.to_vec(),
        recoverable_kwh: totals.recoverable.to_vec(),
    })
}

/// Result of the 13.8.2 dispatch.
struct Dispatch {
    order: Vec<usize>,
    /// `Q_W;gen;gi;out;mi` per unit.
    shares: Vec<[f64; 12]>,
    /// Rest for an extra electric instantaneous heater.
    extra: [f64; 12],
    /// `Q_W;gen;gi;Pout;max;mi` (13.141) per unit, when power-limited.
    power_maximum: Vec<Option<[f64; 12]>>,
}

/// 13.8.2.1: the efficiency that orders generators within a category.
fn ordering_efficiency(unit: &Unit<'_>, outputs: &[f64; 12], annual_total: f64) -> f64 {
    match unit.generator {
        // Annex W: COP_W;BWP of the booster on the system output.
        HotWaterGenerator::BoosterHeatPump(_) | HotWaterGenerator::BoosterHeatPumpForfait(_) => {
            let months = booster_months(unit.generator, outputs, annual_total).unwrap_or_default();
            let weight: f64 = outputs.iter().sum();
            if weight > 0.0 {
                months
                    .iter()
                    .zip(outputs)
                    .map(|(month, output)| month.cop * output)
                    .sum::<f64>()
                    / weight
            } else {
                0.0
            }
        }
        generator => {
            generation(generator, annual_total).map_or(0.0, |(base, practical)| base * practical)
        }
    }
}

/// 13.8.2.2/13.8.2.3: monthly output per generator in dispatch order and
/// the rest for an extra electric instantaneous heater; generators with a
/// declared contribution (13.146) deliver `F_W;gen;gi × Q_W;dis;nren`
/// first, the others share the rest.
fn dispatch(
    system: &HotWaterSystem,
    units: &[Unit<'_>],
    outputs: &[f64; 12],
    annual_total: f64,
    f_building: f64,
    building_area: f64,
) -> Dispatch {
    let mut order: Vec<usize> = (0..units.len()).collect();
    if system.series.is_none() {
        // 13.8.2.1: by category, then the highest efficiency first.
        order.sort_by(|a, b| {
            let (a, b) = (&units[*a], &units[*b]);
            category(a.generator).cmp(&category(b.generator)).then(
                ordering_efficiency(b, outputs, annual_total).total_cmp(&ordering_efficiency(
                    a,
                    outputs,
                    annual_total,
                )),
            )
        });
    }
    let mut result = Dispatch {
        order,
        shares: vec![[0.0; 12]; units.len()],
        extra: [0.0; 12],
        power_maximum: vec![None; units.len()],
    };
    // 13.141 f_func.
    let functioning = if building_area > 500.0 { 0.6 } else { 1.0 };
    // 13.141, with 1,0 kW for an exhaust-air heat pump without P_nom.
    for (unit, maximum) in units.iter().zip(result.power_maximum.iter_mut()) {
        let power = unit
            .nominal_power_kw
            .or_else(|| exhaust_air_heat_pump(unit.generator).then_some(1.0));
        if let (Some(power), false) = (
            power,
            matches!(unit.generator, HotWaterGenerator::ExternalHeat),
        ) {
            *maximum = Some(std::array::from_fn(|index| {
                f_building * functioning * power * MONTH_HOURS[index]
            }));
        }
    }
    // A single generator delivers everything, except an exhaust-air heat
    // pump, which keeps the 1,0 kW default of 13.141 and 13.144a, and a
    // declared contribution below 1.
    if units.len() == 1
        && units[0].nominal_power_kw.is_none()
        && !exhaust_air_heat_pump(units[0].generator)
        && units[0].declared_share.is_none()
    {
        result.shares[0] = *outputs;
        return result;
    }
    // 13.141b/c.
    let preference = match &system.series {
        Some(SeriesArrangement::CollectiveFirstAlsoHeating { maximum_supply_c }) => {
            let year: f64 = MONTH_HOURS.iter().sum();
            let first: f64 = MONTH_HOURS
                .iter()
                .zip(maximum_supply_c)
                .map(|(hours, supply)| hours * (supply - 10.0) / (70.0 - 10.0))
                .sum::<f64>()
                / year;
            Some(first.clamp(0.0, 1.0))
        }
        _ => None,
    };
    for index in 0..12 {
        let need = outputs[index];
        // 13.146 first.
        let mut rest = need;
        for (unit_index, unit) in units.iter().enumerate() {
            if let Some(declared) = unit.declared_share {
                let output = (declared.share(annual_total) * need).min(rest).max(0.0);
                result.shares[unit_index][index] = output;
                rest -= output;
            }
        }
        for (position, unit_index) in result.order.iter().enumerate() {
            let unit = &units[*unit_index];
            if unit.declared_share.is_some() {
                continue;
            }
            // 13.141, 13.141a, 13.141d and 13.142.
            let mut maximum = match (&system.series, preference) {
                (Some(_), Some(first)) => {
                    Some(need * if position == 0 { first } else { 1.0 - first })
                }
                (Some(SeriesArrangement::HotfillElectricBoiler), _) if position == 0 => {
                    Some(need * 0.8)
                }
                _ => result.power_maximum[*unit_index].map(|values| values[index]),
            };
            // 13.144a.
            if let (true, Some(value)) = (exhaust_air_heat_pump(unit.generator), maximum) {
                maximum = Some(match unit.exhaust_air {
                    Some(exhaust) if exhaust.ventilation_suitable => {
                        value
                            * (1.0
                                - exhaust
                                    .heating_time_fraction
                                    .get(index)
                                    .copied()
                                    .unwrap_or(0.0))
                    }
                    Some(_) => 0.0,
                    None => value,
                });
            }
            // 13.143a/13.145.
            let output = maximum.map_or(rest, |value| rest.min(value.max(0.0)));
            result.shares[*unit_index][index] = output;
            rest -= output;
        }
        result.extra[index] = rest.max(0.0);
    }
    result
}

/// 13.13 (p. 535): above 500 m² A_g;gebouw the recoverable losses of vessels,
/// generators and solar systems are 0, except for systems of individual
/// appliances (kitchen boilers, instantaneous heaters, booster heat pumps,
/// individual solar heaters) that each serve less than 500 m². A system
/// without `collective` is taken as individual appliances per dwelling
/// (residential) or one appliance for the assessed area (utility).
fn storage_and_generator_losses_recoverable(
    system: &HotWaterSystem,
    area: f64,
    building_area: f64,
) -> bool {
    if building_area <= 500.0 {
        return true;
    }
    if system.collective.is_some() {
        return false;
    }
    let served = match &system.need {
        HotWaterNeed::Residential { dwelling_count, .. } if *dwelling_count > 0 => {
            area / f64::from(*dwelling_count)
        }
        _ => area,
    };
    served < 500.0
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// Monthly chain; call only after [`validate_hot_water`] returned no issues.
pub fn assess_hot_water(
    system: &HotWaterSystem,
    context: HotWaterContext,
) -> Result<HotWaterAssessment, HotWaterIssue> {
    assess_hot_water_with(system, context, &HotWaterExtras::default())
}

/// [`assess_hot_water`] with data from the space-heating and ventilation
/// results (PFHRD 13.156a, mixed air 13.153h/i).
pub fn assess_hot_water_with(
    system: &HotWaterSystem,
    context: HotWaterContext,
    extras: &HotWaterExtras,
) -> Result<HotWaterAssessment, HotWaterIssue> {
    let area = context.usable_floor_area_m2;
    // 13.26 (ϑ_W;amb) and 13.58 (ϑ_sto;amb): ϑ_int;set;H;zi,mi of 7.9.4,
    // after levelling when known.
    let heated_ambient = |index: usize| {
        context
            .levelled_setpoint_c
            .map_or(context.heated_ambient_c, |values| values[index])
    };
    let building_area = system
        .collective
        .as_ref()
        .map_or(area, |item| item.building_usable_floor_area_m2);
    let f_building = if system.collective.is_some() && building_area > 0.0 {
        (area / building_area).min(1.0)
    } else {
        1.0
    };
    let (annual, shower) = annual_need(system, area);
    // §13.2.4: the share delivered by this system.
    let fraction = context.need_fraction.unwrap_or(1.0);
    let annual = annual * fraction;
    // 13.51: Q_W;nd;d is the dwelling need of 13.1, not the 13.19a share;
    // the system with the shower units books the whole recovery.
    let unsplit = if fraction > 0.0 { 1.0 / fraction } else { 0.0 };
    let year_hours: f64 = MONTH_HOURS.iter().sum();
    let eta_em = emission_efficiency(system);
    // 13.51/13.52 with 13.53.
    let recovery_factor = system.shower_heat_recovery.as_ref().map_or(0.0, |item| {
        let mean = shower_recovery_efficiency(item, context.residential);
        shower
            * mean
            * SHOWER_PRACTICAL_FACTOR
            * SHOWER_TEMPERATURE_FACTOR
            * item.connection.factor()
    });
    let need: [f64; 12] = std::array::from_fn(|index| annual * MONTH_HOURS[index] / year_hours);
    let emission_input: [f64; 12] = std::array::from_fn(|index| {
        need[index] / eta_em - (recovery_factor * need[index] * unsplit).min(need[index] / eta_em)
    });

    // Circulation (13.26–13.44).
    let mut circulation_loss = [0.0; 12];
    let mut pump = [0.0; 12];
    if let Some(circulation) = &system.circulation {
        // 13.20a: A_g;si;W of a utility system is the area it serves.
        let building_area = match (&system.collective, &system.need) {
            (None, HotWaterNeed::Utility { areas, .. }) => {
                let served: f64 = areas.iter().map(|item| item.area_m2).sum();
                if served > 0.0 {
                    served.min(building_area)
                } else {
                    building_area
                }
            }
            _ => building_area,
        };
        let reduced = (building_area
            - if building_area < 1000.0 {
                0.0
            } else {
                circulation.sport_hall_area_m2
            })
        .max(0.0);
        let diameter = circulation.outer_diameter_mm.unwrap_or_else(|| {
            if context.residential {
                let dwellings = circulation.connected_dwellings.unwrap_or(1);
                if dwellings <= 10 {
                    28.0
                } else if dwellings <= 20 {
                    35.0
                } else {
                    42.0
                }
            } else if building_area <= 500.0 {
                35.0
            } else {
                80.0
            }
        });
        let calculated = circulation
            .calculated_psi
            .as_ref()
            .and_then(crate::heating_distribution::PipeGeometry::psi);
        let psi = circulation
            .declared_psi_w_per_mk
            .or(calculated)
            .unwrap_or_else(|| {
                match circulation.outer_diameter_mm {
                    // NTA 8800:2023 (p. 542) has no table 13.29 nor the 35/80 mm
                    // rule (2024 p. 537–538) but rows "klein"/"overig" for an
                    // unknown diameter; "klein" is read as at most 500 m²
                    // connected (the 2024 utility split), see
                    // docs/nta8800-normversies.md.
                    None if crate::norm_versions::profile().table_13_4_system_rows => {
                        table_13_4_system_psi(building_area <= 500.0, circulation.insulation)
                    }
                    _ => table_13_4_psi(diameter, circulation.insulation),
                }
            });
        let length = circulation.length_m.unwrap_or(0.3 * reduced + 10.0);
        let unheated = circulation.unheated_length_m.unwrap_or(0.15 * length);
        let heated = length - unheated;
        let fitting =
            if circulation.fittings_insulated && circulation.insulation != PipeInsulation::None {
                0.03
            } else {
                0.15
            };
        let equivalent = |pipe: f64| pipe + fitting / psi * pipe;
        for (index, loss) in circulation_loss.iter_mut().enumerate() {
            let unheated_c = unheated_ambient(
                system,
                circulation.unheated_ambient_c,
                context.heated_ambient_c,
                index,
            );
            *loss = f_building * MONTH_HOURS[index] / 1000.0
                * psi
                * ((CIRCULATION_MEAN_C - heated_ambient(index)) * equivalent(heated)
                    + (CIRCULATION_MEAN_C - unheated_c) * equivalent(unheated));
        }
        let floors = f64::from(circulation.floor_count.max(1));
        let max_length = 12.0 + 3.0 * floors + 0.089 * reduced / floors;
        let pressure = (1.0 + 0.4) * 0.10 * max_length + 80.0;
        let peak_emission = emission_input.iter().copied().fold(0.0, f64::max);
        let peak_loss = circulation_loss.iter().copied().fold(0.0, f64::max);
        // 13.38 with ΔT = 50 K.
        let flow = (peak_emission + peak_loss) / (30.0 * 4.2 * 50.0 * 1000.0) * 3600.0 / f_building;
        let hydraulic = (pressure * flow / 3600.0).max(0.010);
        let efficiency_factor = match circulation.pump.label_power_kw {
            Some(label) => label / hydraulic,
            None if hydraulic < 2.5 => {
                (1.7 * hydraulic + 17.0 * (1.0 - (-0.3 * hydraulic * 1000.0).exp()) * 1e-3)
                    / hydraulic
            }
            None => (1.25 + (0.2 / hydraulic).powf(0.5)) * 2.0,
        };
        let (cp1, cp2) = match circulation.pump.control {
            PumpControl::UncontrolledOrUnknown => (0.25, 0.94),
            PumpControl::ConstantPressure => (0.50, 0.63),
        };
        let eei = circulation
            .pump
            .energy_efficiency_index
            .unwrap_or(if hydraulic >= 2.5 { 0.25 } else { 0.23 });
        let epsilon = efficiency_factor * (cp1 + cp2) * eei / 0.25;
        for (index, energy) in pump.iter_mut().enumerate() {
            // 13.34 with f_HB = 1,15 and 13.43.
            *energy = hydraulic * MONTH_HOURS[index] * 1.15 * epsilon * f_building;
        }
    }

    // Storage (13.58/13.59).
    let set_temperature = if system.boiling_water_tap {
        90.0
    } else if system.circulation.is_some() {
        65.0
    } else {
        60.0
    };
    let mut storage_loss = [0.0; 12];
    // 13.63: losses of vessels in a heated zone are recoverable.
    let mut heated_storage_loss = [0.0; 12];
    for vessel in &system.storage {
        let ambient = |index: usize| {
            if vessel.in_heated_zone {
                heated_ambient(index)
            } else {
                unheated_ambient(
                    system,
                    vessel.unheated_ambient_c,
                    context.heated_ambient_c,
                    index,
                )
            }
        };
        let factor = if vessel.electric_boiler_insulated_pipe
            && crate::norm_versions::profile().electric_boiler_insulated_pipe_factor
        {
            1.5
        } else {
            f64::from(vessel.connection_factor)
        };
        let label_c = StorageLabel::C.standing_loss_w(vessel.volume_l);
        let measured = vessel.loss.measured_transmission_w_per_k();
        let watts = |index: usize| match vessel.loss {
            StorageLoss::Measured { .. } | StorageLoss::MeasuredStandby { .. } => {
                measured.unwrap_or(0.0) * (set_temperature - ambient(index))
            }
            StorageLoss::Label { label } => {
                let standing = label.standing_loss_w(vessel.volume_l);
                let connection = match label {
                    StorageLabel::APlus | StorageLabel::A | StorageLabel::B => standing,
                    _ => label_c,
                };
                standing + (factor - 1.0) * connection
            }
            StorageLoss::UnknownLabel { produced_from_2018 } => {
                let label = if produced_from_2018 {
                    StorageLabel::C
                } else {
                    StorageLabel::G
                };
                label.standing_loss_w(vessel.volume_l) + (factor - 1.0) * label_c
            }
        };
        for (index, loss) in storage_loss.iter_mut().enumerate() {
            let value = f_building * MONTH_HOURS[index] / 1000.0 * watts(index);
            *loss += value;
            if vessel.in_heated_zone {
                heated_storage_loss[index] += value;
            }
        }
    }

    // Delivery sets (13.24/13.24a, 13.46).
    let sets = system.delivery_sets.as_ref().map_or(0, |item| item.count);
    let recoverable_counts = storage_and_generator_losses_recoverable(system, area, building_area);

    let mut months = Vec::with_capacity(12);
    let mut before_solar = [0.0; 12];
    for index in 0..12 {
        let recovered_pump = (1.0 - PUMP_RECOVERABLE_FACTOR) * pump[index];
        let emission = emission_input[index];
        // 13.25
        let eta_dis = if system.circulation.is_some() && emission > 0.0 {
            emission / (emission + circulation_loss[index] - recovered_pump)
        } else {
            1.0
        };
        let conversion = if sets == 0 {
            0.0
        } else if context.residential {
            f64::from(sets) * DELIVERY_SET_STANDBY_W * MONTH_HOURS[index] / 1000.0
        } else {
            (1.0 - DELIVERY_SET_EFFICIENCY) * (emission / eta_dis + storage_loss[index])
        };
        // 13.7 without the backup losses of solar vessels (13.8).
        let output = emission / eta_dis + storage_loss[index] + conversion;
        before_solar[index] = output;
        months.push(HotWaterMonth {
            month: index as u8 + 1,
            net_need_kwh: need[index],
            recovered_kwh: (recovery_factor * need[index] * unsplit).min(need[index] / eta_em),
            emission_input_kwh: emission,
            circulation_loss_kwh: circulation_loss[index],
            storage_loss_kwh: storage_loss[index],
            conversion_loss_kwh: conversion,
            distribution_efficiency: eta_dis,
            auxiliary_electricity_kwh: pump[index]
                + f64::from(sets) * STANDBY_ELECTRONICS_W * MONTH_HOURS[index] / 1000.0,
            ..HotWaterMonth::default()
        });
    }
    // §13.7 (13.4/13.4a, 13.8, 13.66–13.68): solar heat is used first.
    let hot_water_c = if system.circulation.is_some() {
        65.0
    } else {
        60.0
    };
    let solar = solar_contribution(
        &system.solar,
        solar_storage_ambient(system, context),
        context,
        f_building,
        hot_water_c,
        &before_solar,
        "hotWater.solar",
    )?;
    let mut outputs = [0.0; 12];
    for (index, row) in months.iter_mut().enumerate() {
        let distributed = before_solar[index] + solar.backup_storage_loss[index];
        let renewable = solar.water[index].min(distributed);
        row.solar_backup_storage_loss_kwh = solar.backup_storage_loss[index];
        row.solar_renewable_kwh = renewable;
        row.solar_space_heating_kwh = solar.space_heating[index];
        row.solar_auxiliary_kwh = solar.auxiliary[index];
        row.auxiliary_electricity_kwh += solar.auxiliary[index];
        // 13.4: Q_W;dis;nren.
        outputs[index] = distributed - renewable;
        row.generator_output_kwh = outputs[index];
    }
    let annual_output: f64 = outputs.iter().sum();
    // 13.8.2: dispatch over the generators.
    let units = units(system);
    let Dispatch {
        order,
        shares,
        extra,
        power_maximum,
    } = dispatch(
        system,
        &units,
        &outputs,
        annual_output,
        f_building,
        building_area,
    );
    let mut generator_recoverable = [0.0; 12];
    let mut weighted_input = [0.0; 12];
    let mut combi_chp_heating: Option<Vec<CombiChpHeatingMonth>> = None;
    for (unit, unit_outputs) in units.iter().zip(&shares) {
        let path = if unit.index == 0 {
            "hotWater.generator".to_string()
        } else {
            format!(
                "hotWater.additionalGenerators[{}].generator",
                unit.index - 1
            )
        };
        let booking = book_generator(
            unit.generator,
            system,
            context,
            extras,
            f_building,
            building_area,
            recoverable_counts,
            unit_outputs,
            annual_output,
        )
        .map_err(|code| HotWaterIssue {
            code,
            path: path.clone(),
        })?;
        if let Some(heating) = booking.combi_heating {
            combi_chp_heating = Some(heating.to_vec());
        }
        let carrier = unit.generator.carrier();
        for (index, row) in months.iter_mut().enumerate() {
            let input = booking.input[index];
            row.carrier_input_kwh += input;
            match carrier {
                HotWaterCarrier::Fuel(Carrier::El) => row.electricity_kwh += input,
                HotWaterCarrier::Fuel(Carrier::Gas) => row.natural_gas_kwh += input,
                HotWaterCarrier::Fuel(Carrier::Oil) => row.oil_kwh += input,
                HotWaterCarrier::DistrictHeat => row.district_heat_kwh += input,
                HotWaterCarrier::Biomass => {
                    row.biomass_kwh += input;
                    row.biomass_output_kwh += booking.biomass_output[index];
                }
            }
            row.auxiliary_electricity_kwh += booking.auxiliary[index];
            row.ambient_heat_kwh += booking.ambient[index];
            row.heating_system_load_kwh += booking.heating_system[index];
            row.chp_electricity_kwh += booking.chp_electricity[index];
            row.chp_excess_kwh += booking.chp_excess[index];
            generator_recoverable[index] += booking.recoverable[index];
            weighted_input[index] += booking.efficiency_input[index];
        }
    }
    // 13.149 for the (first) exhaust-air heat pump.
    let exhaust_air = units
        .iter()
        .zip(&shares)
        .zip(&power_maximum)
        .find(|((unit, _), _)| exhaust_air_heat_pump(unit.generator))
        .map(|((unit, unit_outputs), maximum)| ExhaustAirHotWater {
            time_fraction: (0..12)
                .map(|index| match maximum {
                    Some(values) if values[index] > 0.0 => {
                        (unit_outputs[index] / values[index]).min(1.0)
                    }
                    _ => 0.0,
                })
                .collect(),
            hot_water_only: unit.exhaust_air.map_or(true, |exhaust| {
                exhaust
                    .heating_time_fraction
                    .iter()
                    .all(|value| *value <= 0.0)
            }),
            declared_flow_m3_per_h: unit
                .exhaust_air
                .and_then(|exhaust| exhaust.declared_flow_m3_per_h),
        });
    // 13.8.2.3: an extra electric instantaneous heater (η 0,95) for the rest.
    for (index, row) in months.iter_mut().enumerate() {
        let rest = extra[index];
        if rest > 0.0 {
            let input = rest / 0.95;
            row.extra_electric_output_kwh = rest;
            row.carrier_input_kwh += input;
            row.electricity_kwh += input;
            weighted_input[index] += input;
            // 13.179 and 13.181 for the extra heater.
            if recoverable_counts {
                generator_recoverable[index] += input - rest;
            }
            row.auxiliary_electricity_kwh +=
                STANDBY_ELECTRONICS_W * MONTH_HOURS[index] * f_building / 1000.0;
        }
        row.generation_efficiency = if weighted_input[index] > 0.0 {
            row.generator_output_kwh / weighted_input[index]
        } else if units.len() == 1 {
            generation(&system.generator, annual_output)
                .map_or(0.0, |(base, practical)| base * practical)
        } else {
            0.0
        };
        // 13.13: recoverable losses for the space-heating balance.
        // 13.47/13.49: circulation (f_W;dis;rbl), delivery sets and pump.
        let all_pipes_heated = system
            .circulation
            .as_ref()
            .is_some_and(|item| item.unheated_length_m == Some(0.0));
        let f_dis = if all_pipes_heated { 1.0 } else { 0.85 };
        let distribution = f_dis * row.circulation_loss_kwh
            + row.conversion_loss_kwh
            + PUMP_RECOVERABLE_FACTOR * pump[index];
        // 13.13: above 500 m² the generators, vessels and solar systems sit
        // in a separate zone, except individual appliances serving less.
        let (storage, solar_recoverable) = if recoverable_counts {
            (heated_storage_loss[index], solar.recoverable[index])
        } else {
            (0.0, 0.0)
        };
        row.solar_recoverable_kwh = solar_recoverable;
        row.recoverable_loss_kwh =
            distribution + storage + generator_recoverable[index] + solar_recoverable;
    }
    let generators = units
        .iter()
        .zip(&shares)
        .map(|(unit, unit_outputs)| HotWaterGeneratorResult {
            index: unit.index,
            order: order
                .iter()
                .position(|item| *item == unit.index)
                .unwrap_or(unit.index),
            monthly_output_kwh: unit_outputs.to_vec(),
            monthly_share: (0..12)
                .map(|index| {
                    if outputs[index] > 0.0 {
                        unit_outputs[index] / outputs[index]
                    } else if unit.index == order[0] {
                        1.0
                    } else {
                        0.0
                    }
                })
                .collect(),
        })
        .collect();
    Ok(HotWaterAssessment {
        annual_net_need_kwh: annual,
        emission_efficiency: eta_em,
        annual_generator_output_kwh: annual_output,
        annual_solar_renewable_kwh: months.iter().map(|row| row.solar_renewable_kwh).sum(),
        annual_solar_space_heating_kwh: months.iter().map(|row| row.solar_space_heating_kwh).sum(),
        months,
        generators,
        exhaust_air,
        combi_chp_heating,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> HotWaterContext {
        HotWaterContext {
            levelled_setpoint_c: None,
            need_fraction: None,
            standard_setpoint_c: None,
            residential: true,
            usable_floor_area_m2: 100.0,
            heated_ambient_c: 20.0,
            space_heating: None,
        }
    }

    fn system(generator: HotWaterGenerator) -> HotWaterSystem {
        HotWaterSystem {
            declared_share: None,
            need: HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "one dwelling".into(),
            },
            emission: HotWaterEmission::Residential {
                served: ServedTaps::KitchenAndBathroom,
                kitchen_length_m: Some(3.0),
                kitchen_pipe_diameter: None,
                bathroom_length_m: Some(5.0),
                source_reference: "drawing".into(),
            },
            shower_heat_recovery: None,
            circulation: None,
            storage: Vec::new(),
            delivery_sets: None,
            boiling_water_tap: false,
            generator,
            nominal_power_kw: None,
            exhaust_air: None,
            additional_generators: Vec::new(),
            series: None,
            solar: Vec::new(),
            collective: None,
            equipment_reference: "plate".into(),
            connected_taps: None,
            unheated_reduction_factor: None,
        }
    }

    fn dhw_chp_class() -> crate::space_cooling::ChpClass {
        crate::space_cooling::ChpClass {
            power_kw: 50.0,
            built_after_2006: true,
            hre_declared: false,
            // §13.8.4.7.4 uses the HT column whatever the heating system.
            low_temperature: true,
        }
    }

    #[test]
    fn hot_water_chp_method_2_uses_the_ht_conversion_factors() {
        let input = system(HotWaterGenerator::Chp(Box::new(HotWaterChp {
            chp: Some(dhw_chp_class()),
            method1: None,
            also_space_heating: true,
            auxiliary: None,
            equipment_reference: "CHP plate".into(),
        })));
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        assert_eq!(input.carrier(), HotWaterCarrier::Fuel(Carrier::Gas));
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        // Table 9.31, 20–200 kW after 2006, HT: ε_th 0,49, ε_el 0,30.
        assert!((jan.natural_gas_kwh - jan.generator_output_kwh / 0.49).abs() < 1e-9);
        // 16.13.
        assert!((jan.chp_electricity_kwh - jan.generator_output_kwh * 0.30 / 0.49).abs() < 1e-9);
        // No method or both methods are rejected.
        let neither = system(HotWaterGenerator::Chp(Box::new(HotWaterChp {
            chp: None,
            method1: None,
            also_space_heating: false,
            auxiliary: None,
            equipment_reference: "CHP plate".into(),
        })));
        assert!(validate_hot_water(&neither, context(), "hotWater")
            .iter()
            .any(|item| item.code == "hot_water_chp_method_required"));
    }

    #[test]
    fn hot_water_micro_chp_follows_13_182_and_13_183() {
        use crate::micro_chp::{
            hot_water_operating_hours, micro_chp_month, ChpTestPoint, MicroChp, MicroChpFuel,
            MicroChpLocation, MicroChpType,
        };
        let product = MicroChp {
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
            storage: None,
            test_report_reference: "EN 50465 report".into(),
        };
        let input = system(HotWaterGenerator::Chp(Box::new(HotWaterChp {
            chp: None,
            method1: Some(product.clone()),
            also_space_heating: false,
            auxiliary: None,
            equipment_reference: "micro-CHP".into(),
        })));
        assert!(
            validate_hot_water(&input, context(), "hotWater").is_empty(),
            "{:?}",
            validate_hot_water(&input, context(), "hotWater")
        );
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        let output = jan.generator_output_kwh;
        // 13.183 with f_gebouw 1 and f_func 1: t = MIN(Q/20 kW; 744 h).
        let hours = hot_water_operating_hours(&product, output, 1.0, 1.0, 744.0);
        assert!((hours - (output / 20.0).min(744.0)).abs() < 1e-9);
        let month = micro_chp_month(&product, output, hours, 744.0, 1.0, false).unwrap();
        // 13.182 rounded down to 0,025.
        let efficiency = ((output / month.input_kwh) / 0.025 + 1e-9).floor() * 0.025;
        assert!((jan.natural_gas_kwh - output / efficiency).abs() < 1e-9);
        // 16.16.
        assert!((jan.chp_electricity_kwh - month.electricity_kwh).abs() < 1e-9);
        assert!(jan.chp_electricity_kwh > 0.0);
        // 13.8.4.8.3: the measured auxiliary energy.
        assert!(jan.auxiliary_electricity_kwh >= month.auxiliary_kwh.unwrap() - 1e-9);
        // Without measured auxiliary power and without 9.6.8 input the
        // method is rejected; with 9.91/9.92 input it falls back (9.6.6.2.2.3).
        let mut unmeasured = product;
        unmeasured.full_load.auxiliary_power_kw = None;
        let mut fallback = HotWaterChp {
            chp: None,
            method1: Some(unmeasured),
            also_space_heating: false,
            auxiliary: None,
            equipment_reference: "micro-CHP".into(),
        };
        let input = system(HotWaterGenerator::Chp(Box::new(fallback.clone())));
        assert!(validate_hot_water(&input, context(), "hotWater")
            .iter()
            .any(|item| item.code == "hot_water_chp_auxiliary_required"));
        fallback.auxiliary = Some(crate::space_heating_chain::OtherGeneratorAuxiliary {
            electrically_connected_devices: 1,
            nominal_power_kw: Some(25.0),
            source_reference: "plate".into(),
        });
        let input = system(HotWaterGenerator::Chp(Box::new(fallback)));
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        // 9.91: 10 W stand-by × 744 h plus 1 W/kW × 25 kW over the burner time.
        let jan = &result.months[0];
        let on = (jan.generator_output_kwh * 1.1 / 25.0).min(744.0);
        let expected = (10.0 * 744.0 + 25.0 * on) / 1000.0;
        assert!(jan.auxiliary_electricity_kwh >= expected - 1e-6);
    }

    #[test]
    fn booster_heat_pump_books_source_heat_and_electricity() {
        use crate::annex_w::{BoosterClass, BoosterTest};
        let pump = BoosterHeatPump {
            low_test: BoosterTest {
                source_temperature_c: 24.0,
                cop: 3.0,
            },
            high_test: BoosterTest {
                source_temperature_c: 40.0,
                cop: 4.2,
            },
            measured_class: BoosterClass::Class2,
            standing_loss_kw: 0.02,
            source_temperatures_c: vec![30.0],
            cooling_extraction_kwh: None,
            heat_source: BoosterHeatSource::CollectiveGenerator {
                generation_efficiency: 0.9,
                carrier: BoosterSourceCarrier::Gas,
                source_reference: "collective boiler".into(),
            },
            test_report_reference: "synthetic".into(),
        };
        let input = system(HotWaterGenerator::BoosterHeatPump(Box::new(pump.clone())));
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        assert_eq!(input.carrier(), HotWaterCarrier::Fuel(Carrier::Gas));
        let result = assess_hot_water(&input, context()).unwrap();
        let outputs: [f64; 12] =
            std::array::from_fn(|index| result.months[index].generator_output_kwh);
        let booster = calculate_booster(&pump, &outputs);
        let jan = &result.months[0];
        assert!((jan.carrier_input_kwh - booster[0].heating_system_heat_kwh / 0.9).abs() < 1e-9);
        assert!(jan.auxiliary_electricity_kwh >= booster[0].electricity_kwh);
        assert_eq!(jan.ambient_heat_kwh, 0.0);
        // 13.164: the standing loss joins the recoverable losses.
        let loss = crate::significant_figures::round_down(booster[0].standing_loss_heat_kwh);
        assert!(loss > 0.0);
        assert!(jan.recoverable_loss_kwh >= loss - 1e-9);
        // 9.4: from the building's heating system the W.2 heat loads the
        // space-heating node instead of a carrier.
        let mut node = pump.clone();
        node.heat_source = BoosterHeatSource::HeatingSystem;
        let input = system(HotWaterGenerator::BoosterHeatPump(Box::new(node)));
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        assert_eq!(jan.carrier_input_kwh, 0.0);
        assert!((jan.heating_system_load_kwh - booster[0].heating_system_heat_kwh).abs() < 1e-9);
        assert!(jan.auxiliary_electricity_kwh >= booster[0].electricity_kwh);
    }

    #[test]
    fn forfait_booster_heat_pump_books_13_162_and_13_163() {
        // §13.8.4.4 without annex W data (2025+C1 p. 631–633, 2022
        // p. 607–609): heat from the heating system Q/(η_gen;hj·c_W;gen) and
        // electricity Q/(COP·c_W;gen) with class 4 of table 13.27; unknown
        // supply temperature: η 1,15 and COP 3.
        let pump = BoosterForfait {
            design_supply_temperature_c: None,
            heat_source: BoosterHeatSource::HeatingSystem,
            source_reference: "collectieve ketel".into(),
        };
        let input = system(HotWaterGenerator::BoosterHeatPumpForfait(Box::new(
            pump.clone(),
        )));
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        assert_eq!(jan.carrier_input_kwh, 0.0);
        assert!(jan.heating_system_load_kwh > 0.0);
        let class = jan.generator_output_kwh / (1.15 * jan.heating_system_load_kwh);
        assert!((0.45..=1.0).contains(&class), "{class}");
        assert!(jan.auxiliary_electricity_kwh >= jan.generator_output_kwh / (3.0 * class) - 1e-9);
        // From a collective gas boiler the heat becomes gas input.
        let mut gas = pump;
        gas.heat_source = BoosterHeatSource::CollectiveGenerator {
            generation_efficiency: 0.9,
            carrier: BoosterSourceCarrier::Gas,
            source_reference: "collectieve HR107".into(),
        };
        let input = system(HotWaterGenerator::BoosterHeatPumpForfait(Box::new(gas)));
        assert_eq!(input.carrier(), HotWaterCarrier::Fuel(Carrier::Gas));
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        assert!(
            (jan.carrier_input_kwh * 0.9 * 1.15 * class - jan.generator_output_kwh).abs() < 1e-6
        );
    }

    #[test]
    fn single_exhaust_air_heat_pump_follows_13_144a() {
        let mut input = system(HotWaterGenerator::HeatPump {
            declared: None,
            same_ground_source: false,
            exhaust_air_source: true,
            source_correction: None,
            measured_class: None,
            outdoor_air_fraction: None,
        });
        // 13.144a needs the ventilation system, also for one generator.
        assert!(validate_hot_water(&input, context(), "hotWater")
            .iter()
            .any(|item| item.code == "hot_water_exhaust_air_use_required"));
        input.exhaust_air = Some(ExhaustAirUse {
            declared_flow_m3_per_h: None,
            ventilation_suitable: false,
            heating_time_fraction: Vec::new(),
        });
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        let unsuitable = assess_hot_water(&input, context()).unwrap();
        // Unsuitable ventilation: Q_W;gen;out;max = 0, the extra electric
        // heater delivers everything.
        let jan = &unsuitable.months[0];
        assert!((jan.extra_electric_output_kwh - jan.generator_output_kwh).abs() < 1e-9);
        input.exhaust_air = Some(ExhaustAirUse {
            declared_flow_m3_per_h: None,
            ventilation_suitable: true,
            heating_time_fraction: Vec::new(),
        });
        let suitable = assess_hot_water(&input, context()).unwrap();
        // 1,0 kW × 744 h is far above the monthly need: no extra heater.
        assert_eq!(suitable.months[0].extra_electric_output_kwh, 0.0);
    }

    fn two_profile(standard: TwoProfileStandard) -> TwoProfileTest {
        let gas = standard == TwoProfileStandard::En13203Gas;
        TwoProfileTest {
            standard,
            storage_appliance: !gas,
            low: ProfileTest {
                profile: TestProfile::M,
                delivered_kwh_per_day: 5.845,
                input_kwh_per_day: if gas { 7.2 } else { 2.6 },
                auxiliary_kwh_per_day: gas.then_some(0.05),
                max_test_temperature_c: None,
            },
            high: ProfileTest {
                profile: TestProfile::L,
                delivered_kwh_per_day: 11.655,
                input_kwh_per_day: if gas { 13.5 } else { 4.4 },
                auxiliary_kwh_per_day: gas.then_some(0.06),
                max_test_temperature_c: None,
            },
            combi: false,
            integrated_vessel: false,
            exhaust_air_source: false,
            outdoor_air_fraction: None,
            smart_control_factor: None,
            design_set_temperature_c: None,
            legionella_cycle_tested: false,
            mixed_air: None,
            pfhrd: None,
            source_reference: "test report".into(),
        }
    }

    #[test]
    fn building_share_in_13_157_and_13_160_is_new_in_2025() {
        let test = two_profile(TwoProfileStandard::En13203Gas);
        let single = system(HotWaterGenerator::MeasuredTwoProfiles(Box::new(test)));
        let mut shared = single.clone();
        shared.collective = Some(CollectiveHotWater {
            building_usable_floor_area_m2: 4.0 * context().usable_floor_area_m2,
            source_reference: "drawings".into(),
        });
        let current = assess_hot_water(&shared, context()).unwrap();
        let legacy =
            crate::norm_versions::with_version(crate::norm_versions::NormVersion::V2024, || {
                assess_hot_water(&shared, context()).unwrap()
            });
        let reference = assess_hot_water(&single, context()).unwrap();
        // 2024 (p. 611–612): Q_W;b;d = Q/365 and W_W;aux unweighted, as for
        // one dwelling. 2025+C1 (p. 627–628) divides by 365 · f_gebouw;si;W
        // and weights the auxiliary energy with it.
        let jan = |result: &HotWaterAssessment| {
            (
                result.months[0].natural_gas_kwh,
                result.months[0].auxiliary_electricity_kwh,
            )
        };
        let (gas_old, aux_old) = jan(&legacy);
        let (gas_ref, aux_ref) = jan(&reference);
        assert!((gas_old - gas_ref).abs() < 1e-9);
        assert!((aux_old - aux_ref).abs() < 1e-9);
        assert!((jan(&current).1 - aux_ref).abs() > 1e-6);
    }

    #[test]
    fn two_profile_gas_appliance_follows_13_154_to_13_160() {
        let test = two_profile(TwoProfileStandard::En13203Gas);
        let input = system(HotWaterGenerator::MeasuredTwoProfiles(Box::new(
            test.clone(),
        )));
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        // 13.157 with f_gebouw = 1.
        let daily = result.annual_generator_output_kwh / 365.0;
        // 13.153a: gross = 1,11 × net.
        let (e1, e2) = (1.11 * 7.2, 1.11 * 13.5);
        let linear = e1 + (e2 - e1) * (daily - 5.845) / (11.655 - 5.845);
        // 13.154a below i1 when the linear ratio exceeds Q_i1/E_i1.
        let input_d = if daily < 5.845 && daily / linear > 5.845 / e1 {
            e1 / 5.845 * daily
        } else {
            linear
        };
        let eta = ((daily / input_d) / 0.025 + 1e-9).floor() * 0.025;
        let jan = &result.months[0];
        assert!((jan.natural_gas_kwh - jan.generator_output_kwh / (0.95 * eta)).abs() < 1e-9);
        // 13.159/13.160 relative to the same appliance without auxiliaries.
        let w_d = 0.05 + 0.01 * (daily - 5.845) / (11.655 - 5.845);
        let mut silent = test.clone();
        silent.low.auxiliary_kwh_per_day = Some(0.0);
        silent.high.auxiliary_kwh_per_day = Some(0.0);
        let without = assess_hot_water(
            &system(HotWaterGenerator::MeasuredTwoProfiles(Box::new(silent))),
            context(),
        )
        .unwrap();
        let days = 365.0 * 744.0 / 8760.0;
        assert!(
            (jan.auxiliary_electricity_kwh
                - without.months[0].auxiliary_electricity_kwh
                - w_d.max(0.0) * days)
                .abs()
                < 1e-9
        );
        // No 13.160a for a gas appliance that is not a combi with vessel.
        assert_eq!(
            jan.recoverable_loss_kwh,
            without.months[0].recoverable_loss_kwh
        );
    }

    #[test]
    fn two_profile_exhaust_heat_pump_recovers_13_160a_and_reports_13_149() {
        let mut test = two_profile(TwoProfileStandard::En16147HeatPump);
        test.exhaust_air_source = true;
        let mut input = system(HotWaterGenerator::MeasuredTwoProfiles(Box::new(
            test.clone(),
        )));
        input.exhaust_air = Some(ExhaustAirUse {
            ventilation_suitable: true,
            heating_time_fraction: Vec::new(),
            declared_flow_m3_per_h: None,
        });
        // 13.148 note 3: measured efficiency → the declared flow is needed.
        assert!(validate_hot_water(&input, context(), "hotWater")
            .iter()
            .any(|item| item.code == "hot_water_exhaust_air_declared_flow_required"));
        input.exhaust_air = Some(ExhaustAirUse {
            ventilation_suitable: true,
            heating_time_fraction: Vec::new(),
            declared_flow_m3_per_h: Some(150.0),
        });
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        // 13.158 rounded down to 0,05, f_prac 0,9 (storage, no legionella
        // cycle in the test), 13.153c with T_max = T_set = 55 °C.
        let daily = result.annual_generator_output_kwh / 365.0;
        let linear = 2.6 + 1.8 * (daily - 5.845) / (11.655 - 5.845);
        let input_d = if daily < 5.845 && daily / linear > 5.845 / 2.6 {
            2.6 / 5.845 * daily
        } else {
            linear
        };
        let eta = ((daily / input_d) / 0.05 + 1e-9).floor() * 0.05;
        assert!((jan.electricity_kwh - jan.generator_output_kwh / (0.9 * eta)).abs() < 1e-9);
        // 13.160a: Q_i1/E_i1 × (E_i2 − ΔE/ΔQ·Q_i2) × 365 × t/t_an.
        let standing = 5.845 / 2.6 * (4.4 - 1.8 / (11.655 - 5.845) * 11.655);
        let mut plain = test.clone();
        plain.exhaust_air_source = false;
        let mut plain_input = input.clone();
        plain_input.generator = HotWaterGenerator::MeasuredTwoProfiles(Box::new(plain));
        plain_input.exhaust_air = None;
        let reference = assess_hot_water(&plain_input, context()).unwrap();
        assert!(
            (jan.recoverable_loss_kwh
                - reference.months[0].recoverable_loss_kwh
                - standing * 365.0 * 744.0 / 8760.0)
                .abs()
                < 1e-9
        );
        // 13.149: Q_out / (1,0 kW × 744 h); hot water only (f_combi = 0).
        let exhaust = result.exhaust_air.as_ref().unwrap();
        assert!(exhaust.hot_water_only);
        assert!((exhaust.time_fraction[0] - jan.generator_output_kwh / 744.0).abs() < 1e-12);
        // 13.153b: SCF ≥ 0,07 lowers the input; below 0,07 it is ignored.
        test.smart_control_factor = Some(0.05);
        assert_eq!(test.corrected_input(&test.low), 2.6);
        test.smart_control_factor = Some(0.1);
        assert!((test.corrected_input(&test.low) - 2.6 * 0.9).abs() < 1e-12);
        // 13.153c: tested at 52 °C, designed for 50 °C.
        test.smart_control_factor = None;
        test.low.max_test_temperature_c = Some(52.0);
        test.design_set_temperature_c = Some(50.0);
        assert!((test.corrected_input(&test.low) - 2.6 * 1.06 / 1.1).abs() < 1e-12);
        // 13.160a uses E_test without the 13.153b/c corrections.
        test.smart_control_factor = Some(0.2);
        assert!((test.daily_recoverable() - standing).abs() < 1e-12);
    }

    #[test]
    fn two_profile_13_160a_covers_electric_combis_with_a_vessel() {
        let mut test = two_profile(TwoProfileStandard::En16147HeatPump);
        test.combi = true;
        test.integrated_vessel = true;
        let standing = 5.845 / 2.6 * (4.4 - 1.8 / (11.655 - 5.845) * 11.655);
        assert!((test.daily_recoverable() - standing).abs() < 1e-12);
        test.integrated_vessel = false;
        assert_eq!(test.daily_recoverable(), 0.0);
    }

    #[test]
    fn two_profile_mixed_air_and_pfhrd_lower_the_input() {
        // 13.153d–i by hand: condition 2 A7/W55 with ϑ_evap 7 → 3 °C,
        // COP 3,0; q_V;hp;W 300 m³/h against 150 m³/h ventilation.
        let mixed = MixedAirCorrection::En14511 {
            cop_condition_2: 3.0,
            condenser_out_c: 55.0,
            evaporator_in_c: 7.0,
            evaporator_out_c: 3.0,
            minimum_air_flow_m3_per_h: 300.0,
            source_reference: "EN 14511 report".into(),
        };
        let ventilation = MixedAirVentilation {
            required_outdoor_air_m3_per_h: [150.0; 12],
            extract_air_c: [20.0; 12],
        };
        let carnot = 3.0 / ((55.0 + 275.15) / (55.0 - 3.0 + 7.0));
        let theta = (OUTDOOR_TEMPERATURE_C[0] - 3.7) * 0.5 + 20.0 * 0.5;
        let cop = carnot * (55.0 + 275.15) / (55.0 - (theta - 4.0) + 7.0);
        let expected = cop / 3.0;
        assert!((mixed.factor(0, Some(&ventilation)) - expected).abs() < 1e-12);
        assert_eq!(mixed.factor(0, None), 1.0);
        let mut test = two_profile(TwoProfileStandard::En16147HeatPump);
        assert!((test.corrected_input_with(&test.low, expected) - 2.6 / expected).abs() < 1e-12);
        // Mixed air is for combi heat pumps on return air only.
        test.mixed_air = Some(mixed.clone());
        assert!(test
            .issues("g")
            .iter()
            .any(|(code, _)| *code == "hot_water_mixed_air_not_applicable"));
        test.combi = true;
        test.exhaust_air_source = true;
        assert!(test.issues("g").is_empty(), "{:?}", test.issues("g"));
        // 13.156a/b: 3650 kWh gas for heating, f_PFHRD = 0,5 / 10.
        let mut gas = two_profile(TwoProfileStandard::En13203Gas);
        gas.combi = true;
        gas.pfhrd = Some(PfhrdTest {
            indirect_gas_kwh_per_day: 0.5,
            heating_gas_kwh_per_day: 10.0,
            source_reference: "prEN 13203-7".into(),
        });
        assert!(gas.issues("g").is_empty(), "{:?}", gas.issues("g"));
        let extras = HotWaterExtras {
            combi_chp: None,
            combi_space_heating_gas_kwh: Some(3650.0),
            mixed_air: None,
        };
        assert!((gas.pfhrd_daily(&extras) - 0.5).abs() < 1e-12);
        let daily = 8.0;
        let plain = gas.daily_input(daily, 1.0, 0.0);
        assert!((gas.daily_input(daily, 1.0, 0.5) - (plain - 0.5)).abs() < 1e-12);
        assert!(gas.efficiency_with(daily, 1.0, 0.5).unwrap() > gas.efficiency(daily).unwrap());
        // 13.153: Q_gas;w = Q_gas;s − 13,5 × 1,21 × P_s / η_100.
        let winter = winter_gas_consumption_kwh(12.0, 0.1, 0.9).unwrap();
        assert!((winter - (12.0 - 13.5 * 1.21 * 0.1 / 0.9)).abs() < 1e-12);
    }

    #[test]
    fn two_profile_pairs_and_ranges_follow_13_8_4_2() {
        let mut test = two_profile(TwoProfileStandard::En16147HeatPump);
        // Instantaneous with i2 = L only allows M; S needs a storage appliance.
        test.low.profile = TestProfile::S;
        test.storage_appliance = false;
        assert!(test
            .issues("g")
            .iter()
            .any(|(code, _)| *code == "hot_water_two_profile_pair_invalid"));
        test.storage_appliance = true;
        assert!(test.issues("g").is_empty());
        // Heat pumps with i2 = L: Q_W;b;d ≤ (L + XL)/2 = 15,3625.
        let test = two_profile(TwoProfileStandard::En16147HeatPump);
        assert!(test.range_issue(15.36).is_none());
        assert!(test.range_issue(15.37).is_some());
        // The lower limits only apply with i2 = XXL (and 3XL/4XL): L/XL has
        // none, so small dwellings stay within range.
        let mut gas = two_profile(TwoProfileStandard::En13203Gas);
        gas.low.profile = TestProfile::L;
        gas.low.delivered_kwh_per_day = 11.655;
        gas.high.profile = TestProfile::Xl;
        gas.high.delivered_kwh_per_day = 19.07;
        assert!(gas.range_issue(5.0).is_none());
        assert!(gas.range_issue(100.0).is_none());
        // i1 = L with i2 = XXL requires Q_W;b;d ≥ M; gas below it is
        // rejected (table 13.18 covers heat pumps only).
        gas.high.profile = TestProfile::Xxl;
        gas.high.delivered_kwh_per_day = 24.53;
        assert_eq!(gas.lower_bound_kwh_per_day(), 5.845);
        assert_eq!(
            gas.efficiency(5.0),
            Err("hot_water_two_profile_below_range")
        );
        // Heat pumps below the limit: 13.160b with table 13.18 for i1 = L.
        let mut pump = gas.clone();
        pump.standard = TwoProfileStandard::En16147HeatPump;
        pump.low.input_kwh_per_day = 4.4;
        pump.high.input_kwh_per_day = 8.0;
        let c = european_profile_correction(TappingProfile::L, 365.0 * 5.0).unwrap();
        let eta = ((11.655 * c / 4.4) / 0.05 + 1e-9).floor() * 0.05;
        assert!((pump.efficiency(5.0).unwrap() - eta).abs() < 1e-12);
    }

    #[test]
    fn declared_share_takes_13_146_first() {
        let mut input = system(combi());
        input.declared_share = Some(DeclaredGeneratorShare {
            points: vec![
                SharePoint {
                    annual_kwh: 1000.0,
                    share: 0.6,
                },
                SharePoint {
                    annual_kwh: 4000.0,
                    share: 0.9,
                },
            ],
            source_reference: "quality declaration".into(),
        });
        input.nominal_power_kw = Some(20.0);
        input.additional_generators = vec![AdditionalHotWaterGenerator {
            generator: HotWaterGenerator::ElectricInstantaneous,
            nominal_power_kw: Some(20.0),
            exhaust_air: None,
            declared_share: None,
            equipment_reference: "boiler".into(),
        }];
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        let annual = result.annual_generator_output_kwh;
        let share = 0.6 + 0.3 * (annual - 1000.0) / 3000.0;
        let jan = &result.months[0];
        let main = &result.generators[0];
        assert!((main.monthly_output_kwh[0] - share * jan.generator_output_kwh).abs() < 1e-9);
        let boiler = &result.generators[1];
        assert!(
            (boiler.monthly_output_kwh[0] - (1.0 - share) * jan.generator_output_kwh).abs() < 1e-9
        );
        // Declared shares may not exceed 1 together.
        input.additional_generators[0].declared_share = Some(DeclaredGeneratorShare {
            points: vec![SharePoint {
                annual_kwh: 0.0,
                share: 0.5,
            }],
            source_reference: "declaration".into(),
        });
        assert!(validate_hot_water(&input, context(), "hotWater")
            .iter()
            .any(|item| item.code == "hot_water_declared_share_sum_exceeds_one"));
    }

    #[test]
    fn heating_system_generator_moves_hot_water_to_the_space_heating_node() {
        let input = system(HotWaterGenerator::HeatingSystem);
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        for month in &result.months {
            assert_eq!(month.carrier_input_kwh, 0.0);
            assert_eq!(month.extra_electric_output_kwh, 0.0);
            assert!((month.heating_system_load_kwh - month.generator_output_kwh).abs() < 1e-12);
        }
        assert!(result.months[0].heating_system_load_kwh > 0.0);
    }

    #[test]
    fn booster_orders_by_its_cop_within_category_b() {
        use crate::annex_w::{BoosterClass, BoosterTest};
        let pump = BoosterHeatPump {
            low_test: BoosterTest {
                source_temperature_c: 24.0,
                cop: 3.0,
            },
            high_test: BoosterTest {
                source_temperature_c: 40.0,
                cop: 4.2,
            },
            measured_class: BoosterClass::Class2,
            standing_loss_kw: 0.02,
            source_temperatures_c: vec![30.0],
            cooling_extraction_kwh: None,
            heat_source: BoosterHeatSource::ExternalHeat,
            test_report_reference: "synthetic".into(),
        };
        let booster = HotWaterGenerator::BoosterHeatPump(Box::new(pump.clone()));
        let outputs = [150.0; 12];
        let unit = Unit {
            index: 0,
            generator: &booster,
            nominal_power_kw: None,
            exhaust_air: None,
            declared_share: None,
        };
        let months = calculate_booster(&pump, &outputs);
        let expected = months.iter().map(|month| month.cop).sum::<f64>() / 12.0;
        assert!((ordering_efficiency(&unit, &outputs, 1800.0) - expected).abs() < 1e-12);
        assert!(expected > 1.4);
    }

    #[test]
    fn solar_storage_ambient_follows_13_69_and_13_69a() {
        let mut ctx = context();
        ctx.standard_setpoint_c = Some(20.0);
        ctx.levelled_setpoint_c = Some([17.0; 12]);
        let base = system(HotWaterGenerator::ElectricBoiler);
        let mut exhaust = system(HotWaterGenerator::HeatPump {
            declared: None,
            same_ground_source: false,
            exhaust_air_source: true,
            source_correction: None,
            measured_class: None,
            outdoor_air_fraction: None,
        });
        exhaust.exhaust_air = Some(ExhaustAirUse {
            ventilation_suitable: true,
            heating_time_fraction: Vec::new(),
            declared_flow_m3_per_h: None,
        });
        // 13.69: levelled setpoint; 13.69a: table 7.13 with an exhaust-air
        // heat pump.
        assert_eq!(solar_storage_ambient(&base, ctx), [17.0; 12]);
        assert_eq!(solar_storage_ambient(&exhaust, ctx), [20.0; 12]);
        // Without the chain's values the heating setpoint applies.
        assert_eq!(solar_storage_ambient(&base, context()), [20.0; 12]);
    }

    #[test]
    fn biomass_combi_and_indirect_pilot_flame() {
        // §13.8.4.6 table 13.22: 20 mm inside 0,65, no aux.
        let input = system(HotWaterGenerator::BiomassCombi {
            insulation: BiomassStoreInsulation::AtLeast20Mm,
            inside_boundary: true,
        });
        assert!(validate_hot_water(&input, context(), "hotWater").is_empty());
        assert_eq!(input.carrier(), HotWaterCarrier::Biomass);
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        assert!((jan.biomass_kwh - jan.generator_output_kwh / 0.65).abs() < 1e-9);
        assert!((jan.biomass_output_kwh - jan.generator_output_kwh).abs() < 1e-9);
        assert_eq!(jan.auxiliary_electricity_kwh, 0.0);
        // §13.8.4.7.4: pilot gas only for a hot-water-only gas boiler.
        let pilot = |also_space_heating: bool, pilot_flame: bool| {
            let input = system(HotWaterGenerator::IndirectBoiler {
                boiler: IndirectBoiler::Hr107,
                oil: false,
                inside_boundary: true,
                also_space_heating,
                declared: None,
                pilot_flame,
            });
            assess_hot_water(&input, context()).unwrap().months[0].natural_gas_kwh
        };
        let extra = 695.0 * 744.0 / crate::climate::YEAR_HOURS;
        assert!((pilot(false, true) - pilot(false, false) - extra).abs() < 1e-9);
        assert!((pilot(true, true) - pilot(true, false)).abs() < 1e-12);
    }

    #[test]
    fn heat_pump_series_uses_table_9_29_above_65() {
        use crate::forfait_heat_pump_draft::TableSource;
        // §13.8.4.10: outdoor air, 65–70 °C column of table 9.29: 2,0.
        let input = system(HotWaterGenerator::HeatPumpSeries {
            last_source: TableSource::OutdoorAir,
        });
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        assert!((jan.electricity_kwh - jan.generator_output_kwh / 2.0).abs() < 1e-9);
        assert!((jan.ambient_heat_kwh - jan.generator_output_kwh * (1.0 - 1.0 / 2.0)).abs() < 1e-9);
    }

    fn combi() -> HotWaterGenerator {
        HotWaterGenerator::GasAppliance {
            appliance: GasAppliance::CombiGaskeurHrCw,
            measured_class: Some(ApplicationClass::Class4),
            kitchen_only: false,
            declared: None,
            annex_t: None,
            annex_t_conditions: None,
        }
    }

    #[test]
    fn emission_follows_13_23_and_table_13_2() {
        let input = system(combi());
        assert!(validate_hot_water(&input, context(), "dhw").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        // l_k 3 m → 0,69; l_b 5 m → 0,90.
        let eta = 1.0 / (0.8 / 0.90 + 0.2 / 0.69);
        assert!((result.emission_efficiency - eta).abs() < 1e-12);
        assert!((result.annual_net_need_kwh - 856.0 * 2.28).abs() < 1e-9);
        let jan = &result.months[0];
        assert!((jan.emission_input_kwh - jan.net_need_kwh / eta).abs() < 1e-9);
        assert_eq!(kitchen_emission(2.0), 0.69);
        assert_eq!(bathroom_emission(14.0), 0.72);
    }

    #[test]
    fn gas_combi_uses_table_13_25_with_class_correction() {
        let input = system(combi());
        let result = assess_hot_water(&input, context()).unwrap();
        let annual = result.annual_generator_output_kwh;
        let correction = gas_class_correction(ApplicationClass::Class4, annual);
        let jan = &result.months[0];
        assert!((jan.generation_efficiency - 0.675 * correction).abs() < 1e-12);
        assert!(
            (jan.carrier_input_kwh - jan.generator_output_kwh / (0.675 * correction)).abs() < 1e-9
        );
        // Worked example of p. 642 (2 150 kWh): about 0,95 and 0,834.
        let share = 345.0 / 695.0;
        assert!(
            (gas_class_correction(ApplicationClass::Class2, 2150.0) - (0.90 + 0.1 * share)).abs()
                < 1e-12
        );
        assert!(
            (gas_class_correction(ApplicationClass::Class4, 2150.0) - (0.80 + 0.067 * share)).abs()
                < 1e-12
        );
    }

    #[test]
    fn shower_recovery_follows_13_53_and_the_utility_minimum() {
        let recovery = |showers: Vec<ShowerUnit>, unknown: bool| ShowerHeatRecovery {
            assignment_unknown: unknown,
            showers,
            connection: ShowerConnection::MixerAndHeater,
            source_reference: "plan".into(),
        };
        // 13.57: two showers, one with a unit: (0,40 + 0)/2.
        let one = recovery(vec![ShowerUnit::Vertical, ShowerUnit::None], false);
        assert!((shower_recovery_efficiency(&one, true) - 0.20).abs() < 1e-12);
        // 13.56: two different units: (0,40 + 0,20)/2.
        let two = recovery(vec![ShowerUnit::Vertical, ShowerUnit::Horizontal], false);
        assert!((shower_recovery_efficiency(&two, true) - 0.30).abs() < 1e-12);
        // Utility, assignment unknown, 9 of 10 connected (> 80 %): the minimum.
        let mut showers = vec![ShowerUnit::Vertical; 8];
        showers.push(ShowerUnit::Horizontal);
        showers.push(ShowerUnit::None);
        let utility = recovery(showers.clone(), true);
        assert!((shower_recovery_efficiency(&utility, false) - 0.20).abs() < 1e-12);
        // Dwellings keep the mean of 13.53.
        let mean = (8.0 * 0.40 + 0.20) / 10.0;
        assert!((shower_recovery_efficiency(&utility, true) - mean).abs() < 1e-12);
        // At 80 % connected the minimum rule does not apply.
        showers[8] = ShowerUnit::None;
        let at_80 = recovery(showers, true);
        assert!((shower_recovery_efficiency(&at_80, false) - 0.32).abs() < 1e-12);
    }

    /// 13.28 (2025+C1 p. 551) for the ISSO 54 EP-W402f pipe embedded in
    /// the construction: d_i 0,02, d_a 0,04, z 0,03, λ_D 0,04, λ_em 2.
    #[test]
    fn calculated_psi_of_an_embedded_circulation_pipe() {
        let geometry = crate::heating_distribution::PipeGeometry::InsulatedEmbedded {
            pipe_outer_diameter_m: 0.02,
            insulated_diameter_m: 0.04,
            insulation_lambda: 0.04,
            embedding_lambda: 2.0,
            depth_m: 0.03,
        };
        let psi = geometry.psi().unwrap();
        let expected =
            std::f64::consts::PI / (0.5 * ((2.0_f64).ln() / 0.04 + (3.0_f64).ln() / 2.0));
        assert!((psi - expected).abs() < 1e-12);
        let mut input = system(HotWaterGenerator::IndirectBoiler {
            boiler: IndirectBoiler::Hr107,
            oil: false,
            inside_boundary: true,
            also_space_heating: true,
            declared: None,
            pilot_flame: false,
        });
        let circulation = |declared: Option<f64>, calculated| Circulation {
            outer_diameter_mm: Some(15.0),
            insulation: PipeInsulation::Mm15,
            declared_psi_w_per_mk: declared,
            calculated_psi: calculated,
            fittings_insulated: true,
            length_m: None,
            unheated_length_m: Some(0.0),
            unheated_ambient_c: None,
            floor_count: 2,
            sport_hall_area_m2: 0.0,
            connected_dwellings: None,
            pump: CirculationPump {
                control: PumpControl::UncontrolledOrUnknown,
                label_power_kw: None,
                energy_efficiency_index: None,
            },
            source_reference: "design".into(),
        };
        input.circulation = Some(circulation(None, Some(geometry)));
        let calculated = assess_hot_water(&input, context()).unwrap();
        input.circulation = Some(circulation(Some(psi), None));
        let declared = assess_hot_water(&input, context()).unwrap();
        for (a, b) in calculated.months.iter().zip(&declared.months) {
            assert!((a.circulation_loss_kwh - b.circulation_loss_kwh).abs() < 1e-12);
        }
        // Not both, and no Ψ for impossible geometry (4·z ≤ d_a).
        input.circulation = Some(circulation(Some(psi), Some(geometry)));
        let codes: Vec<&str> = validate_hot_water(&input, context(), "w")
            .iter()
            .map(|issue| issue.code)
            .collect();
        assert!(
            codes.contains(&"hot_water_psi_declared_and_calculated"),
            "{codes:?}"
        );
        let shallow = crate::heating_distribution::PipeGeometry::InsulatedEmbedded {
            pipe_outer_diameter_m: 0.02,
            insulated_diameter_m: 0.04,
            insulation_lambda: 0.04,
            embedding_lambda: 2.0,
            depth_m: 0.005,
        };
        input.circulation = Some(circulation(None, Some(shallow)));
        let codes: Vec<&str> = validate_hot_water(&input, context(), "w")
            .iter()
            .map(|issue| issue.code)
            .collect();
        assert!(
            codes.contains(&"hot_water_pipe_geometry_invalid"),
            "{codes:?}"
        );
    }

    #[test]
    fn unheated_pipes_use_theta_ztu_from_b_u() {
        let mut input = system(HotWaterGenerator::IndirectBoiler {
            boiler: IndirectBoiler::Hr107,
            oil: false,
            inside_boundary: true,
            also_space_heating: true,
            declared: None,
            pilot_flame: false,
        });
        input.circulation = Some(Circulation {
            outer_diameter_mm: Some(15.0),
            insulation: PipeInsulation::Mm15,
            declared_psi_w_per_mk: None,
            calculated_psi: None,
            fittings_insulated: true,
            length_m: None,
            unheated_length_m: None,
            unheated_ambient_c: None,
            floor_count: 2,
            sport_hall_area_m2: 0.0,
            connected_dwellings: None,
            pump: CirculationPump {
                control: PumpControl::UncontrolledOrUnknown,
                label_power_kw: None,
                energy_efficiency_index: None,
            },
            source_reference: "design".into(),
        });
        let base = assess_hot_water(&input, context()).unwrap();
        input.unheated_reduction_factor = Some(0.5);
        let derived = assess_hot_water(&input, context()).unwrap();
        // 7.82 in January: 20 − 0,5·(20 − 2,61) = 11,305 °C instead of 13.
        let psi = 0.174;
        let equivalent = |pipe: f64| pipe + 0.03 / psi * pipe;
        let ztu = 20.0 - 0.5 * (20.0 - crate::climate::OUTDOOR_TEMPERATURE_C[0]);
        let delta = 744.0 / 1000.0 * psi * (13.0 - ztu) * equivalent(6.0);
        assert!(
            (derived.months[0].circulation_loss_kwh - base.months[0].circulation_loss_kwh - delta)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn shower_recovery_is_subtracted_after_emission_efficiency() {
        let mut input = system(combi());
        input.shower_heat_recovery = Some(ShowerHeatRecovery {
            assignment_unknown: false,
            showers: vec![ShowerUnit::Vertical, ShowerUnit::None],
            connection: ShowerConnection::MixerAndHeater,
            source_reference: "plan".into(),
        });
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        // 13.51: 0,8 · Q_nd · 0,20 · 0,95 · 0,83 · 1,0
        let recovered = 0.8 * jan.net_need_kwh * 0.2 * 0.95 * 0.83;
        assert!((jan.recovered_kwh - recovered).abs() < 1e-9);
        let expected = jan.net_need_kwh / result.emission_efficiency - recovered;
        assert!((jan.emission_input_kwh - expected).abs() < 1e-9);
    }

    #[test]
    fn shower_recovery_uses_the_dwelling_need_under_a_13_19a_split() {
        let mut input = system(combi());
        input.shower_heat_recovery = Some(ShowerHeatRecovery {
            assignment_unknown: false,
            showers: vec![ShowerUnit::Vertical],
            connection: ShowerConnection::MixerAndHeater,
            source_reference: "plan".into(),
        });
        let whole = assess_hot_water(&input, context()).unwrap();
        let mut split = context();
        split.need_fraction = Some(0.8);
        let bathroom = assess_hot_water(&input, split).unwrap();
        // 13.51: the recovery follows Q_W;nd;d of the dwelling (13.1), not
        // the bathroom system's 13.19a share.
        assert!((bathroom.months[0].recovered_kwh - whole.months[0].recovered_kwh).abs() < 1e-9);
        assert!(
            (bathroom.months[0].net_need_kwh - 0.8 * whole.months[0].net_need_kwh).abs() < 1e-9
        );
    }

    #[test]
    fn utility_circulation_uses_the_served_area_of_13_20a() {
        let mut input = system(HotWaterGenerator::ExternalHeat);
        input.need = HotWaterNeed::Utility {
            areas: vec![UtilityArea {
                function: LabelFunction::Office,
                area_m2: 1000.0,
            }],
            source_reference: "plan".into(),
        };
        input.emission = HotWaterEmission::Utility {
            mean_length_m: 5.0,
            source_reference: "plan".into(),
        };
        input.circulation = Some(Circulation {
            outer_diameter_mm: Some(28.0),
            insulation: PipeInsulation::Mm15,
            declared_psi_w_per_mk: None,
            calculated_psi: None,
            fittings_insulated: true,
            length_m: None,
            unheated_length_m: Some(0.0),
            unheated_ambient_c: None,
            floor_count: 1,
            sport_hall_area_m2: 0.0,
            connected_dwellings: None,
            pump: CirculationPump {
                control: PumpControl::UncontrolledOrUnknown,
                label_power_kw: None,
                energy_efficiency_index: None,
            },
            source_reference: "design".into(),
        });
        let mut own = context();
        own.residential = false;
        own.usable_floor_area_m2 = 1000.0;
        let mut building = own;
        building.usable_floor_area_m2 = 2000.0;
        let a = assess_hot_water(&input, own).unwrap();
        let b = assess_hot_water(&input, building).unwrap();
        // 13.31 with A_g;si;W = Σ A_g;zi,si (13.20a): the same pipe length.
        assert!(a.months[0].circulation_loss_kwh > 0.0);
        assert!((a.months[0].circulation_loss_kwh - b.months[0].circulation_loss_kwh).abs() < 1e-9);
    }

    #[test]
    fn source_correction_takes_table_v1_values() {
        for (value, valid) in [
            (1.0, true),
            (1.02, true),
            (1.04, true),
            (1.1, false),
            (0.9, false),
        ] {
            let input = system(HotWaterGenerator::HeatPump {
                declared: None,
                exhaust_air_source: false,
                source_correction: Some(value),
                measured_class: None,
                outdoor_air_fraction: None,
                same_ground_source: false,
            });
            let found = validate_hot_water(&input, context(), "hotWater")
                .iter()
                .any(|item| item.code == "hot_water_source_correction_invalid");
            assert_eq!(found, !valid, "{value}");
        }
    }

    #[test]
    fn circulation_storage_and_pump_follow_13_25_to_13_59() {
        let mut input = system(HotWaterGenerator::IndirectBoiler {
            boiler: IndirectBoiler::Hr107,
            oil: false,
            inside_boundary: true,
            also_space_heating: true,
            declared: None,
            pilot_flame: false,
        });
        input.circulation = Some(Circulation {
            outer_diameter_mm: Some(15.0),
            insulation: PipeInsulation::Mm15,
            declared_psi_w_per_mk: None,
            calculated_psi: None,
            fittings_insulated: true,
            length_m: None,
            unheated_length_m: None,
            unheated_ambient_c: None,
            floor_count: 2,
            sport_hall_area_m2: 0.0,
            connected_dwellings: None,
            pump: CirculationPump {
                control: PumpControl::UncontrolledOrUnknown,
                label_power_kw: None,
                energy_efficiency_index: None,
            },
            source_reference: "design".into(),
        });
        input.storage.push(StorageVessel {
            id: "vat".into(),
            volume_l: 120.0,
            loss: StorageLoss::UnknownLabel {
                produced_from_2018: true,
            },
            connection_factor: 3,
            electric_boiler_insulated_pipe: false,
            in_heated_zone: true,
            unheated_ambient_c: None,
            not_in_appliance_test: false,
            source_reference: "plate".into(),
        });
        assert!(validate_hot_water(&input, context(), "dhw").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        // L = 0,3·100 + 10 = 40 m; 15 % unheated at 13 °C; Ψ = 0,174.
        let psi = 0.174;
        let equivalent = |pipe: f64| pipe + 0.03 / psi * pipe;
        let loss = 744.0 / 1000.0
            * psi
            * ((62.5 - 20.0) * equivalent(34.0) + (62.5 - 13.0) * equivalent(6.0));
        assert!((jan.circulation_loss_kwh - loss).abs() < 1e-9);
        // Label C (65 °C set point irrelevant for the label route): f = 3.
        let label_c = 14.33 + 7.13 * 120.0_f64.powf(0.4);
        let storage = 744.0 / 1000.0 * (label_c + 2.0 * label_c);
        assert!((jan.storage_loss_kwh - storage).abs() < 1e-9);
        // Auxiliaries: pump plus the burner term of 13.181 (1 W/kW · Q · 1,1).
        let pump = jan.auxiliary_electricity_kwh - 1.1 * jan.generator_output_kwh / 1000.0;
        assert!(pump > 0.0);
        let eta_dis = jan.emission_input_kwh
            / (jan.emission_input_kwh + jan.circulation_loss_kwh - 0.8 * pump);
        assert!((jan.distribution_efficiency - eta_dis).abs() < 1e-9);
        assert!(
            (jan.generator_output_kwh - (jan.emission_input_kwh / eta_dis + storage)).abs() < 1e-9
        );
        assert!((jan.generation_efficiency - 0.90).abs() < 1e-12);
        let carrier = jan.generator_output_kwh / 0.9;
        assert!((jan.carrier_input_kwh - carrier).abs() < 1e-9);
    }

    #[test]
    fn heat_pump_classes_and_ambient_heat() {
        let mut input = system(HotWaterGenerator::HeatPump {
            declared: None,
            same_ground_source: false,
            exhaust_air_source: false,
            source_correction: None,
            measured_class: Some(ApplicationClass::Class3),
            outdoor_air_fraction: None,
        });
        let result = assess_hot_water(&input, context()).unwrap();
        let annual = result.annual_generator_output_kwh;
        let correction = heat_pump_class_correction(ApplicationClass::Class3, annual).unwrap();
        let jan = &result.months[0];
        let eta = 1.4 * correction;
        assert!((jan.generation_efficiency - eta).abs() < 1e-12);
        if eta >= 1.0 {
            let ambient = jan.generator_output_kwh * (1.0 - 1.0 / eta);
            assert!((jan.ambient_heat_kwh - ambient).abs() < 1e-9);
        }
        // A class-1 measurement cannot serve a larger demand.
        input.generator = HotWaterGenerator::HeatPump {
            declared: None,
            same_ground_source: false,
            exhaust_air_source: false,
            source_correction: None,
            measured_class: Some(ApplicationClass::Class1),
            outdoor_air_fraction: None,
        };
        assert_eq!(
            assess_hot_water(&input, context()).unwrap_err().code,
            "hot_water_heat_pump_class_exceeded"
        );
        let class2 = heat_pump_class_correction(ApplicationClass::Class2, 2000.0).unwrap();
        assert!((class2 - (0.6 + 0.4 * 195.0 / 695.0)).abs() < 1e-12);
        assert_eq!(european_profile_correction(TappingProfile::M, 3000.0), None);
        // Exhaust-air heat pump: no ambient heat without an outdoor share.
        input.generator = HotWaterGenerator::HeatPumpEn16147 {
            profile: TappingProfile::Xl,
            delivered_kwh_per_day: 11.0,
            input_kwh_per_day: 4.0,
            exhaust_air_source: true,
            storage_without_legionella_cycle: true,
            outdoor_air_fraction: None,
            smart_control_factor: None,
            max_test_temperature_c: None,
            design_set_temperature_c: None,
            source_reference: "EN 16147 report".into(),
        };
        let result = assess_hot_water(&input, context()).unwrap();
        assert_eq!(result.months[0].ambient_heat_kwh, 0.0);
        let correction =
            european_profile_correction(TappingProfile::Xl, result.annual_generator_output_kwh)
                .unwrap();
        let eta = round_down(11.0 * correction / 4.0, 0.05) * 0.9;
        assert!((result.months[0].generation_efficiency - eta).abs() < 1e-12);
        // 13.153b/13.153c: SCF 0,10 (smart = 1), T_max;test 52 °C and
        // T_set;design 50 °C: 4,0·0,9·1,06/1,10.
        if let HotWaterGenerator::HeatPumpEn16147 {
            smart_control_factor,
            max_test_temperature_c,
            design_set_temperature_c,
            ..
        } = &mut input.generator
        {
            *smart_control_factor = Some(0.10);
            *max_test_temperature_c = Some(52.0);
            *design_set_temperature_c = Some(50.0);
        }
        let corrected = 4.0 * 0.9 * 1.06 / 1.10;
        assert!(
            (en16147_corrected_input(4.0, Some(0.10), Some(52.0), Some(50.0)) - corrected).abs()
                < 1e-12
        );
        // SCF below 0,07: smart = 0.
        assert_eq!(en16147_corrected_input(4.0, Some(0.05), None, None), 4.0);
        let result = assess_hot_water(&input, context()).unwrap();
        let correction =
            european_profile_correction(TappingProfile::Xl, result.annual_generator_output_kwh)
                .unwrap();
        let eta = round_down(11.0 * correction / corrected, 0.05) * 0.9;
        assert!((result.months[0].generation_efficiency - eta).abs() < 1e-12);
    }

    #[test]
    fn gas_storage_heater_follows_13_165_to_13_175() {
        let input = system(HotWaterGenerator::GasStorageHeater {
            volume_l: 100.0,
            measured_standby_kwh_per_day: None,
            before_1985: false,
            in_heated_zone: true,
        });
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        let standby = 2.0 + 0.033 * 100.0_f64.powf(1.1);
        let standing = (55.0 - 20.0) / 45.0 * 744.0 / 24.0 * standby;
        let output = jan.generator_output_kwh;
        let loss = 0.27 / 0.84 * output;
        let eta = round_down(output / (output + loss + standing), 0.025);
        assert!((jan.generation_efficiency - eta).abs() < 1e-12);
        let aux = (10.0 * 744.0 + output * 1.1) / 1000.0;
        assert!((jan.auxiliary_electricity_kwh - aux).abs() < 1e-9);
        // 13.168: ϑ_W;amb is the levelled ϑ_int;set;H;zi,mi (7.9.4).
        let mut levelled = context();
        levelled.levelled_setpoint_c = Some([17.0; 12]);
        let result = assess_hot_water(&input, levelled).unwrap();
        let jan = &result.months[0];
        let output = jan.generator_output_kwh;
        let standing = (55.0 - 17.0) / 45.0 * 744.0 / 24.0 * standby;
        let loss = 0.27 / 0.84 * output;
        let eta = round_down(output / (output + loss + standing), 0.025);
        assert!((jan.generation_efficiency - eta).abs() < 1e-12);
    }

    #[test]
    fn utility_need_storage_label_and_delivery_sets() {
        let ctx = HotWaterContext {
            levelled_setpoint_c: None,
            need_fraction: None,
            standard_setpoint_c: None,
            residential: false,
            usable_floor_area_m2: 1000.0,
            heated_ambient_c: 21.0,
            space_heating: None,
        };
        let mut input = system(HotWaterGenerator::ExternalHeat);
        input.need = HotWaterNeed::Utility {
            areas: vec![
                UtilityArea {
                    function: LabelFunction::Office,
                    area_m2: 800.0,
                },
                UtilityArea {
                    function: LabelFunction::Sport,
                    area_m2: 200.0,
                },
            ],
            source_reference: "plan".into(),
        };
        input.emission = HotWaterEmission::Utility {
            mean_length_m: 5.0,
            source_reference: "plan".into(),
        };
        input.delivery_sets = Some(DeliverySets {
            count: 2,
            source_reference: "plan".into(),
        });
        assert!(validate_hot_water(&input, ctx, "dhw").is_empty());
        let result = assess_hot_water(&input, ctx).unwrap();
        assert!((result.annual_net_need_kwh - (1.4 * 800.0 + 12.5 * 200.0)).abs() < 1e-9);
        let jan = &result.months[0];
        assert!((jan.emission_input_kwh - jan.net_need_kwh / 0.8).abs() < 1e-9);
        // 13.24a: (1 − 0,75)·Q_em/η_dis.
        assert!((jan.conversion_loss_kwh - 0.25 * jan.emission_input_kwh).abs() < 1e-9);
        assert!((jan.auxiliary_electricity_kwh - 2.0 * 10.0 * 744.0 / 1000.0).abs() < 1e-9);
        assert!((jan.carrier_input_kwh - jan.generator_output_kwh).abs() < 1e-9);
        assert_eq!(StorageLabel::APlus.standing_loss_w(0.0), 5.5);
        assert_eq!(
            utility_specific_need(LabelFunction::HealthcareWithBeds),
            Some(15.3)
        );
        assert_eq!(table_13_4_psi(30.0, PipeInsulation::Unknown), 0.321);
        assert_eq!(table_13_4_psi(25.0, PipeInsulation::Mm20), 0.219);
    }

    #[test]
    fn annex_t_and_annex_u_replace_table_values() {
        use crate::hot_water_tests::{ShowerRun, ShowerTestClass, TestFuel};
        let report = AnnexTTest::CombiForfait {
            useful_mj: 20.0,
            fuel_input_mj: 25.0,
            electricity_kwh: 0.0,
            full_load_efficiency: 0.96,
            fuel: TestFuel::NaturalGas,
            source_reference: "Gaskeur report".into(),
        };
        let expected = round_down(report.efficiency().unwrap(), 0.025);
        let mut input = system(HotWaterGenerator::GasAppliance {
            appliance: GasAppliance::CombiGaskeurHrCw,
            measured_class: Some(ApplicationClass::Class1),
            kitchen_only: false,
            declared: None,
            annex_t: Some(report.clone()),
            annex_t_conditions: Some(AnnexTConditions {
                type_supplied_before_2021: true,
                appliance_indoors: true,
            }),
        });
        let run = |r: f64| ShowerRun::Energies {
            recovered_kj: r,
            shower_kj: 100.0,
        };
        input.shower_heat_recovery = Some(ShowerHeatRecovery {
            assignment_unknown: false,
            showers: vec![ShowerUnit::AnnexU {
                test: AnnexUTest {
                    class: ShowerTestClass::Class2,
                    runs: vec![run(50.0), run(51.0), run(52.0)],
                    source_reference: "lab".into(),
                },
            }],
            connection: ShowerConnection::MixerAndHeater,
            source_reference: "plan".into(),
        });
        assert!(validate_hot_water(&input, context(), "dhw").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        // Class 1 has c_W;gen = 1 at every demand (table 13.26).
        assert!((result.months[0].generation_efficiency - expected).abs() < 1e-9);
        assert_eq!(
            input.shower_heat_recovery.as_ref().unwrap().showers[0].efficiency(),
            0.5
        );
        // A water-heater report does not fit a combi appliance.
        if let HotWaterGenerator::GasAppliance { annex_t, .. } = &mut input.generator {
            *annex_t = Some(AnnexTTest::WaterHeater {
                useful_mj: 20.0,
                fuel_input_mj: 25.0,
                electricity_kwh: 0.0,
                fuel: TestFuel::NaturalGas,
                source_reference: "x".into(),
            });
        }
        let codes: Vec<_> = validate_hot_water(&input, context(), "dhw")
            .iter()
            .map(|item| item.code)
            .collect();
        assert!(codes.contains(&"annex_t_appliance_mismatch"));
    }

    #[test]
    fn annex_u_class_and_annex_t_conditions_are_checked() {
        use crate::hot_water_tests::ShowerTestClass;
        let mut input = system(combi());
        let declared = |class| ShowerUnit::Declared {
            efficiency: 0.61,
            test_class: Some(class),
            source_reference: "report".into(),
        };
        input.shower_heat_recovery = Some(ShowerHeatRecovery {
            assignment_unknown: false,
            showers: vec![declared(ShowerTestClass::Class2)],
            connection: ShowerConnection::MixerAndHeater,
            source_reference: "plan".into(),
        });
        let codes = |input: &HotWaterSystem, residential: bool| -> Vec<&'static str> {
            let mut ctx = context();
            ctx.residential = residential;
            validate_hot_water(input, ctx, "dhw")
                .iter()
                .map(|item| item.code)
                .collect()
        };
        // §13.5.3: a class 4 appliance needs the class 4 measurement.
        assert!(codes(&input, true).contains(&"annex_u_test_class_mismatch"));
        input.shower_heat_recovery.as_mut().unwrap().showers =
            vec![declared(ShowerTestClass::Class4)];
        assert!(!codes(&input, true).contains(&"annex_u_test_class_mismatch"));
        // Declared values are rounded down to 0,025.
        assert!(
            (input.shower_heat_recovery.as_ref().unwrap().showers[0].efficiency() - 0.6).abs()
                < 1e-12
        );
        // Class 1 appliance: class 2 measurement; utility: class 4 always.
        if let HotWaterGenerator::GasAppliance { measured_class, .. } = &mut input.generator {
            *measured_class = Some(ApplicationClass::Class1);
        }
        input.shower_heat_recovery.as_mut().unwrap().showers =
            vec![declared(ShowerTestClass::Class2)];
        assert!(!codes(&input, true).contains(&"annex_u_test_class_mismatch"));
        assert!(codes(&input, false).contains(&"annex_u_test_class_mismatch"));
        // Annex T needs the §13.8.4.3 conditions.
        let report = AnnexTTest::WaterHeater {
            useful_mj: 20.0,
            fuel_input_mj: 25.0,
            electricity_kwh: 0.0,
            fuel: crate::hot_water_tests::TestFuel::NaturalGas,
            source_reference: "report".into(),
        };
        let mut heater = system(HotWaterGenerator::GasAppliance {
            appliance: GasAppliance::WithoutGaskeur,
            measured_class: Some(ApplicationClass::Class3),
            kitchen_only: false,
            declared: None,
            annex_t: Some(report.clone()),
            annex_t_conditions: None,
        });
        assert!(codes(&heater, true).contains(&"annex_t_conditions_required"));
        if let HotWaterGenerator::GasAppliance {
            annex_t_conditions, ..
        } = &mut heater.generator
        {
            *annex_t_conditions = Some(AnnexTConditions {
                type_supplied_before_2021: false,
                appliance_indoors: true,
            });
        }
        assert!(codes(&heater, true).contains(&"annex_t_not_applicable"));
        // Note 3 d: the measured value is class-corrected for every type.
        let annual = 3000.0;
        let (efficiency, _) = generation(&heater.generator, annual).unwrap();
        let expected = round_down(report.efficiency().unwrap(), 0.025)
            * gas_class_correction(ApplicationClass::Class3, annual);
        assert!((efficiency - expected).abs() < 1e-12);
    }

    #[test]
    fn annex_t_u_fixture_is_valid() {
        let system: HotWaterSystem = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-hot-water-annex-t-u-synthetic.json"
        ))
        .unwrap();
        assert!(validate_hot_water(&system, context(), "dhw").is_empty());
        let shower = &system.shower_heat_recovery.as_ref().unwrap().showers[0];
        // (2870 + 2905 + 2850)/3/5880 = 0,4890 → 0,475.
        assert!((shower.efficiency() - 0.475).abs() < 1e-9);
        assert!(assess_hot_water(&system, context()).is_ok());
    }

    #[test]
    fn recoverable_losses_follow_13_13() {
        let mut input = system(HotWaterGenerator::ElectricBoiler);
        input.storage.push(StorageVessel {
            id: "vessel".into(),
            volume_l: 120.0,
            loss: StorageLoss::Label {
                label: StorageLabel::B,
            },
            connection_factor: 1,
            electric_boiler_insulated_pipe: false,
            in_heated_zone: true,
            unheated_ambient_c: None,
            not_in_appliance_test: false,
            source_reference: "label".into(),
        });
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        // 13.63: the whole loss of a vessel in a heated zone, below 500 m².
        assert!((jan.recoverable_loss_kwh - jan.storage_loss_kwh).abs() < 1e-9);
        input.storage[0].in_heated_zone = false;
        let outside = assess_hot_water(&input, context()).unwrap();
        assert_eq!(outside.months[0].recoverable_loss_kwh, 0.0);
        // 13.179: electric instantaneous heater, Q/η − Q.
        let instantaneous = system(HotWaterGenerator::ElectricInstantaneous);
        let result = assess_hot_water(&instantaneous, context()).unwrap();
        let jan = &result.months[0];
        let expected =
            jan.generator_output_kwh / jan.generation_efficiency - jan.generator_output_kwh;
        assert!((jan.recoverable_loss_kwh - expected).abs() < 1e-9);
    }

    /// §13.6.2 (2025+C1 p. 566): for a solar system the vessel and its
    /// backup part are calculated in 13.7, so a boiler heating the backup
    /// part of an integrated-backup solar vessel needs no vessel of its own
    /// (ISSO 54 v2.0 EP-W405d/e). A solar preheater does not lift the
    /// requirement: the boiler's own vessel stays.
    #[test]
    fn integrated_backup_solar_vessel_replaces_the_boiler_vessel() {
        use crate::solar_thermal::{
            CollectorEfficiency, CollectorField, CollectorType, LoopPipes, SolarMethod,
            SolarStorage, SolarType, SolarUse, SolarWaterHeater,
        };
        let solar = |solar_type| SolarWaterHeater {
            id: "zb".into(),
            solar_use: SolarUse::WaterHeating,
            count: 1,
            method: SolarMethod::Calculated {
                solar_type,
                collectors: CollectorField {
                    module_area_m2: 5.0,
                    module_count: 1,
                    orientation: crate::climate::Orientation::South,
                    tilt_deg: 30.0,
                    obstruction: crate::solar_shading::CollectorObstruction::Minimal,
                    efficiency: CollectorEfficiency::Forfait {
                        collector: CollectorType::Glazed,
                    },
                    heat_exchanger_w_per_k: None,
                    loop_pipes: LoopPipes::Forfait,
                    pump_power_w: None,
                },
                storage: SolarStorage {
                    total_volume_l: 220.0,
                    backup_volume_l: Some(100.0),
                    loss: StorageLoss::Label {
                        label: StorageLabel::B,
                    },
                    backup_loss_in_generator_efficiency: false,
                },
            },
            pvt: None,
            source_reference: "ISSO 54 v2.0 EP-W405e".into(),
        };
        let mut input = system(HotWaterGenerator::IndirectBoiler {
            boiler: IndirectBoiler::Hr107,
            oil: false,
            inside_boundary: true,
            also_space_heating: true,
            declared: None,
            pilot_flame: false,
        });
        let codes = |input: &HotWaterSystem| -> Vec<&str> {
            validate_hot_water(input, context(), "dhw")
                .iter()
                .map(|item| item.code)
                .collect()
        };
        assert!(codes(&input).contains(&"hot_water_storage_required"));
        input.solar = vec![solar(SolarType::IntegratedBackup)];
        assert!(!codes(&input).contains(&"hot_water_storage_required"));
        input.solar = vec![solar(SolarType::Preheater)];
        assert!(codes(&input).contains(&"hot_water_storage_required"));
    }

    #[test]
    fn validation_rejects_inconsistent_input() {
        let mut bad = system(HotWaterGenerator::ElectricBoiler);
        bad.emission = HotWaterEmission::Residential {
            served: ServedTaps::KitchenAndBathroom,
            kitchen_length_m: None,
            kitchen_pipe_diameter: None,
            bathroom_length_m: Some(-1.0),
            source_reference: String::new(),
        };
        let codes: Vec<_> = validate_hot_water(&bad, context(), "dhw")
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "hot_water_length_invalid",
            "hot_water_storage_required",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
        let mut combined = system(combi());
        combined.storage.push(StorageVessel {
            id: "v".into(),
            volume_l: 50.0,
            loss: StorageLoss::Label {
                label: StorageLabel::A,
            },
            connection_factor: 2,
            electric_boiler_insulated_pipe: false,
            in_heated_zone: true,
            unheated_ambient_c: None,
            not_in_appliance_test: false,
            source_reference: "label".into(),
        });
        assert!(validate_hot_water(&combined, context(), "dhw")
            .iter()
            .any(|item| item.code == "hot_water_storage_in_generator_efficiency"));
        // Note 1 of §13.6.2: a 13.8.4.3 heat pump tested without the vessel.
        let mut tested = system(HotWaterGenerator::HeatPumpEn16147 {
            profile: TappingProfile::L,
            delivered_kwh_per_day: 11.655,
            input_kwh_per_day: 4.0,
            exhaust_air_source: false,
            storage_without_legionella_cycle: false,
            outdoor_air_fraction: None,
            smart_control_factor: None,
            max_test_temperature_c: None,
            design_set_temperature_c: None,
            source_reference: "EN 16147 report".into(),
        });
        tested.storage = combined.storage.clone();
        let blocked = |input: &HotWaterSystem| {
            validate_hot_water(input, context(), "dhw")
                .iter()
                .any(|item| item.code == "hot_water_storage_in_generator_efficiency")
        };
        assert!(blocked(&tested));
        tested.storage[0].not_in_appliance_test = true;
        assert!(!blocked(&tested));
        let result = assess_hot_water(&tested, context()).unwrap();
        assert!(result.months[0].storage_loss_kwh > 0.0);
        // The flag does not open the table 13.25 route (loss in η).
        combined.storage[0].not_in_appliance_test = true;
        assert!(blocked(&combined));
    }

    #[test]
    fn measured_storage_loss_is_rounded_up_per_annex_x() {
        // 13.60: 1 000·1,2/(24·(65 − 20)) = 1,11 W/K → 1,2 (annex X).
        let standby = StorageLoss::MeasuredStandby {
            standby_kwh_per_day: 1.2,
            reference_storage_c: 65.0,
            reference_ambient_c: 20.0,
        };
        assert!((standby.measured_transmission_w_per_k().unwrap() - 1.2).abs() < 1e-12);
        // A measured 1,83 W/K becomes 1,9; X.2 takes the next higher value.
        let direct = StorageLoss::Measured {
            transmission_w_per_k: 1.83,
        };
        assert!((direct.measured_transmission_w_per_k().unwrap() - 1.9).abs() < 1e-12);
        let mut input = system(HotWaterGenerator::ElectricBoiler);
        input.storage.push(StorageVessel {
            id: "v".into(),
            volume_l: 120.0,
            loss: standby,
            connection_factor: 3,
            electric_boiler_insulated_pipe: false,
            in_heated_zone: true,
            unheated_ambient_c: None,
            not_in_appliance_test: false,
            source_reference: "test report".into(),
        });
        assert!(validate_hot_water(&input, context(), "dhw").is_empty());
        let from_standby = assess_hot_water(&input, context()).unwrap();
        input.storage[0].loss = StorageLoss::Measured {
            transmission_w_per_k: 1000.0 * 1.2 / (24.0 * 45.0),
        };
        let from_h = assess_hot_water(&input, context()).unwrap();
        assert!(
            (from_standby.months[0].storage_loss_kwh - from_h.months[0].storage_loss_kwh).abs()
                < 1e-12
        );
        input.storage[0].loss = StorageLoss::MeasuredStandby {
            standby_kwh_per_day: 1.2,
            reference_storage_c: 20.0,
            reference_ambient_c: 20.0,
        };
        assert!(validate_hot_water(&input, context(), "dhw")
            .iter()
            .any(|item| item.code == "hot_water_storage_loss_invalid"));
    }

    fn solar_heater(solar_use: crate::solar_thermal::SolarUse) -> SolarWaterHeater {
        use crate::solar_thermal::{
            CollectorEfficiency, CollectorField, CollectorType, LoopPipes, SolarStorage, SolarType,
        };
        SolarWaterHeater {
            id: "sol".into(),
            solar_use,
            count: 1,
            method: SolarMethod::Calculated {
                solar_type: SolarType::Preheater,
                collectors: CollectorField {
                    module_area_m2: 2.0,
                    module_count: 2,
                    orientation: crate::climate::Orientation::South,
                    tilt_deg: 45.0,
                    obstruction: crate::solar_shading::CollectorObstruction::Minimal,
                    efficiency: CollectorEfficiency::Forfait {
                        collector: CollectorType::Glazed,
                    },
                    heat_exchanger_w_per_k: None,
                    loop_pipes: LoopPipes::Forfait,
                    pump_power_w: None,
                },
                storage: SolarStorage {
                    total_volume_l: 200.0,
                    backup_volume_l: None,
                    loss: StorageLoss::Label {
                        label: StorageLabel::B,
                    },
                    backup_loss_in_generator_efficiency: false,
                },
            },
            pvt: None,
            source_reference: "datasheet".into(),
        }
    }

    #[test]
    fn solar_preheater_is_used_first_per_13_4() {
        use crate::solar_thermal::{SolarType, SolarUse};
        let plain = system(combi());
        let mut input = plain.clone();
        input.solar = vec![solar_heater(SolarUse::WaterHeating)];
        assert!(validate_hot_water(&input, context(), "dhw").is_empty());
        let base = assess_hot_water(&plain, context()).unwrap();
        let result = assess_hot_water(&input, context()).unwrap();
        let SolarMethod::Calculated {
            collectors,
            storage,
            ..
        } = &input.solar[0].method
        else {
            unreachable!()
        };
        let use_kwh: [f64; 12] =
            std::array::from_fn(|index| base.months[index].generator_output_kwh);
        let months = calculated_service(
            SolarType::Preheater,
            collectors,
            storage,
            &ServiceSettings {
                use_kwh,
                share: [1.0; 12],
                reference_c: std::array::from_fn(water_reference_c),
                low_c: [COLD_WATER_C; 12],
                high_c: 60.0,
                backup_set_c: 60.0,
                ambient_c: [20.0; 12],
                pump_hours: PUMP_HOURS_WATER,
                add_backup_loss_to_use: false,
            },
        );
        let july = &result.months[6];
        // 13.66 with f_prac;sol 0,95 and 13.4a.
        let expected = (0.95 * months[6].renewable_kwh).min(use_kwh[6]);
        assert!(expected > 0.0);
        assert!((july.solar_renewable_kwh - expected).abs() < 1e-9);
        assert!((july.generator_output_kwh - (use_kwh[6] - expected)).abs() < 1e-9);
        // Preheater: no backup part, no 13.8 loss.
        assert_eq!(july.solar_backup_storage_loss_kwh, 0.0);
        // 13.67 and 13.68 join the auxiliary energy and recoverable losses.
        assert!((july.solar_auxiliary_kwh - months[6].auxiliary_kwh).abs() < 1e-12);
        assert!(
            (july.auxiliary_electricity_kwh
                - base.months[6].auxiliary_electricity_kwh
                - months[6].auxiliary_kwh)
                .abs()
                < 1e-9
        );
        assert!((july.solar_recoverable_kwh - months[6].recoverable_kwh).abs() < 1e-12);
        // Less gas for the combi appliance.
        assert!(july.natural_gas_kwh < base.months[6].natural_gas_kwh);
        assert!(result.annual_solar_renewable_kwh > 0.0);
    }

    #[test]
    fn solar_combi_splits_by_f_use_and_feeds_the_node() {
        use crate::solar_thermal::SolarUse;
        let mut input = system(combi());
        input.solar = vec![solar_heater(SolarUse::Combi)];
        // Without space-heating data the combi supplies hot water only.
        let water_only = assess_hot_water(&input, context()).unwrap();
        assert_eq!(water_only.annual_solar_space_heating_kwh, 0.0);
        let mut with_heating = context();
        with_heating.space_heating = Some(SolarSpaceHeating {
            node_kwh: [
                800.0, 700.0, 500.0, 300.0, 100.0, 0.0, 0.0, 0.0, 100.0, 300.0, 500.0, 700.0,
            ],
            building_fraction: 1.0,
            design_supply_c: 55.0,
            design_return_c: 47.0,
        });
        let combi_result = assess_hot_water(&input, with_heating).unwrap();
        let march = &combi_result.months[2];
        assert!(march.solar_space_heating_kwh > 0.0);
        // July: no heating, so all collectors serve hot water (f_W;use = 1).
        assert_eq!(combi_result.months[6].solar_space_heating_kwh, 0.0);
        assert!(
            (combi_result.months[6].solar_renewable_kwh - water_only.months[6].solar_renewable_kwh)
                .abs()
                > 0.0
                || combi_result.months[6].solar_renewable_kwh > 0.0
        );
        // In March part of the collector serves the heating, so less for hot
        // water than with f_W;use = 1.
        assert!(march.solar_renewable_kwh < water_only.months[2].solar_renewable_kwh);
        // COMBI pumps run 2000 h a year in total (13.109/13.127).
        let pump: f64 = combi_result
            .months
            .iter()
            .map(|row| row.solar_auxiliary_kwh)
            .sum();
        assert!(pump > 0.0 && pump <= 15.0 * 2000.0 / 1000.0 + 1e-9);
    }

    #[test]
    fn tested_solar_system_outside_its_range_is_rejected() {
        use crate::solar_thermal::{SolarTestPoint, SolarType, SolarUse};
        let mut input = system(combi());
        let mut heater = solar_heater(SolarUse::WaterHeating);
        heater.method = SolarMethod::Tested {
            solar_type: SolarType::Preheater,
            orientation: crate::climate::Orientation::South,
            tilt_deg: 45.0,
            obstruction: crate::solar_shading::CollectorObstruction::Minimal,
            total_volume_l: 150.0,
            test_points: vec![
                SolarTestPoint {
                    annual_demand_kwh: 5000.0,
                    solar_output_kwh: Some(1500.0),
                    backup_output_kwh: None,
                    auxiliary_kwh: 40.0,
                },
                SolarTestPoint {
                    annual_demand_kwh: 9000.0,
                    solar_output_kwh: Some(2000.0),
                    backup_output_kwh: None,
                    auxiliary_kwh: 45.0,
                },
            ],
            backup_loss_in_generator_efficiency: false,
            source_reference: "EN 12976 report".into(),
        };
        input.solar = vec![heater];
        assert!(validate_hot_water(&input, context(), "dhw").is_empty());
        let error = assess_hot_water(&input, context()).unwrap_err();
        assert_eq!(error.code, "solar_test_out_of_range");
    }

    #[test]
    fn several_generators_follow_13_8_2() {
        // An individual exhaust-air heat pump (category a, 1,0 kW default)
        // first, then the gas combi for the rest.
        let mut input = system(HotWaterGenerator::HeatPump {
            declared: None,
            same_ground_source: false,
            exhaust_air_source: true,
            source_correction: None,
            measured_class: None,
            outdoor_air_fraction: None,
        });
        input.exhaust_air = Some(ExhaustAirUse {
            declared_flow_m3_per_h: None,
            ventilation_suitable: true,
            heating_time_fraction: vec![0.9, 0.9, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.9, 0.9],
        });
        input.additional_generators = vec![AdditionalHotWaterGenerator {
            declared_share: None,
            generator: combi(),
            nominal_power_kw: Some(24.0),
            exhaust_air: None,
            equipment_reference: "combi plate".into(),
        }];
        assert!(
            validate_hot_water(&input, context(), "dhw").is_empty(),
            "{:?}",
            validate_hot_water(&input, context(), "dhw")
        );
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        // 13.141/13.144a: 1·1·1,0 kW·744 h·(1 − 0,9) = 74,4 kWh.
        let pump = &result.generators[0];
        assert_eq!(pump.order, 0);
        let expected = jan.generator_output_kwh.min(74.4);
        assert!((pump.monthly_output_kwh[0] - expected).abs() < 1e-9);
        let gas = &result.generators[1];
        assert!((gas.monthly_output_kwh[0] - (jan.generator_output_kwh - expected)).abs() < 1e-9);
        // 13.150 shares add up to 1.
        assert!((pump.monthly_share[0] + gas.monthly_share[0] - 1.0).abs() < 1e-12);
        // 13.3 per carrier: electricity for the heat pump, gas for the rest.
        assert!(jan.electricity_kwh > 0.0 && jan.natural_gas_kwh > 0.0);
        assert!((jan.carrier_input_kwh - jan.electricity_kwh - jan.natural_gas_kwh).abs() < 1e-9);
        // July without heating: the heat pump covers everything.
        assert!(result.generators[1].monthly_output_kwh[6] < 1e-12);
        // A missing P_nom of a parallel gas appliance is rejected.
        input.additional_generators[0].nominal_power_kw = None;
        assert!(validate_hot_water(&input, context(), "dhw")
            .iter()
            .any(|item| item.code == "hot_water_nominal_power_required"));
    }

    #[test]
    fn hotfill_series_caps_the_first_appliance_at_80_percent() {
        let mut input = system(combi());
        input.series = Some(SeriesArrangement::HotfillElectricBoiler);
        input.additional_generators = vec![AdditionalHotWaterGenerator {
            declared_share: None,
            generator: HotWaterGenerator::ElectricBoiler,
            nominal_power_kw: None,
            exhaust_air: None,
            equipment_reference: "kitchen boiler".into(),
        }];
        input.storage = vec![StorageVessel {
            id: "boiler".into(),
            volume_l: 10.0,
            loss: StorageLoss::Label {
                label: StorageLabel::C,
            },
            connection_factor: 1,
            electric_boiler_insulated_pipe: false,
            in_heated_zone: true,
            unheated_ambient_c: None,
            not_in_appliance_test: false,
            source_reference: "label".into(),
        }];
        assert!(validate_hot_water(&input, context(), "dhw").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        // 13.141a.
        assert!(
            (result.generators[0].monthly_output_kwh[0] - 0.8 * jan.generator_output_kwh).abs()
                < 1e-9
        );
        assert!(
            (result.generators[1].monthly_output_kwh[0] - 0.2 * jan.generator_output_kwh).abs()
                < 1e-9
        );
        assert_eq!(jan.extra_electric_output_kwh, 0.0);
    }

    #[test]
    fn collective_series_uses_13_141b() {
        let mut input = system(HotWaterGenerator::IndirectBoiler {
            boiler: IndirectBoiler::Hr107,
            oil: false,
            inside_boundary: true,
            also_space_heating: true,
            declared: None,
            pilot_flame: false,
        });
        input.collective = Some(CollectiveHotWater {
            building_usable_floor_area_m2: 400.0,
            source_reference: "drawings".into(),
        });
        input.storage = vec![StorageVessel {
            id: "vessel".into(),
            volume_l: 300.0,
            loss: StorageLoss::Label {
                label: StorageLabel::B,
            },
            connection_factor: 1,
            electric_boiler_insulated_pipe: false,
            in_heated_zone: true,
            unheated_ambient_c: None,
            not_in_appliance_test: false,
            source_reference: "label".into(),
        }];
        input.additional_generators = vec![AdditionalHotWaterGenerator {
            declared_share: None,
            generator: HotWaterGenerator::ElectricBoiler,
            nominal_power_kw: None,
            exhaust_air: None,
            equipment_reference: "booster element".into(),
        }];
        let supply = vec![
            60.0, 58.0, 55.0, 50.0, 45.0, 40.0, 40.0, 40.0, 45.0, 50.0, 55.0, 58.0,
        ];
        input.series = Some(SeriesArrangement::CollectiveFirstAlsoHeating {
            maximum_supply_c: supply.clone(),
        });
        assert!(validate_hot_water(&input, context(), "dhw").is_empty());
        let result = assess_hot_water(&input, context()).unwrap();
        let year: f64 = MONTH_HOURS.iter().sum();
        let first: f64 = MONTH_HOURS
            .iter()
            .zip(&supply)
            .map(|(hours, value)| hours * (value - 10.0) / 60.0)
            .sum::<f64>()
            / year;
        let jan = &result.months[0];
        assert!((result.generators[0].monthly_share[0] - first).abs() < 1e-9);
        assert!(
            (result.generators[1].monthly_output_kwh[0] - (1.0 - first) * jan.generator_output_kwh)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn an_undersized_last_generator_gets_an_extra_electric_heater() {
        let mut input = system(combi());
        input.nominal_power_kw = Some(0.1);
        let result = assess_hot_water(&input, context()).unwrap();
        let jan = &result.months[0];
        // 13.141: 0,1 kW·744 h.
        let maximum = 0.1 * 744.0;
        assert!(jan.generator_output_kwh > maximum);
        assert!((result.generators[0].monthly_output_kwh[0] - maximum).abs() < 1e-9);
        let rest = jan.generator_output_kwh - maximum;
        assert!((jan.extra_electric_output_kwh - rest).abs() < 1e-9);
        assert!(jan.electricity_kwh >= rest / 0.95 - 1e-9);
    }

    #[test]
    fn individual_appliances_keep_recoverable_losses_above_500_m2() {
        use crate::annex_w::{BoosterClass, BoosterTest};
        let pump = BoosterHeatPump {
            low_test: BoosterTest {
                source_temperature_c: 24.0,
                cop: 3.0,
            },
            high_test: BoosterTest {
                source_temperature_c: 40.0,
                cop: 4.2,
            },
            measured_class: BoosterClass::Class2,
            standing_loss_kw: 0.02,
            source_temperatures_c: vec![30.0],
            cooling_extraction_kwh: None,
            heat_source: BoosterHeatSource::CollectiveGenerator {
                generation_efficiency: 0.9,
                carrier: BoosterSourceCarrier::Gas,
                source_reference: "collective boiler".into(),
            },
            test_report_reference: "synthetic".into(),
        };
        // 20 apartments of 60 m² with one booster each: 1200 m² in total.
        let mut input = system(HotWaterGenerator::BoosterHeatPump(Box::new(pump)));
        input.need = HotWaterNeed::Residential {
            dwelling_count: 20,
            source_reference: "block".into(),
        };
        let mut block = context();
        block.usable_floor_area_m2 = 1200.0;
        let result = assess_hot_water(&input, block).unwrap();
        assert!(result.months[0].recoverable_loss_kwh > 0.0);
        // A collective system above 500 m² loses its vessel recoverability.
        let mut collective = system(HotWaterGenerator::ElectricBoiler);
        collective.need = input.need.clone();
        collective.storage = vec![StorageVessel {
            id: "vessel".into(),
            volume_l: 500.0,
            loss: StorageLoss::Label {
                label: StorageLabel::C,
            },
            connection_factor: 1,
            electric_boiler_insulated_pipe: false,
            in_heated_zone: true,
            unheated_ambient_c: None,
            not_in_appliance_test: false,
            source_reference: "label".into(),
        }];
        collective.collective = Some(CollectiveHotWater {
            building_usable_floor_area_m2: 1200.0,
            source_reference: "drawings".into(),
        });
        let mut part = context();
        part.usable_floor_area_m2 = 1200.0;
        let collective_result = assess_hot_water(&collective, part).unwrap();
        assert_eq!(collective_result.months[0].recoverable_loss_kwh, 0.0);
        // The same vessel as individual appliances per dwelling counts.
        collective.collective = None;
        let individual = assess_hot_water(&collective, part).unwrap();
        assert!(individual.months[0].recoverable_loss_kwh > 0.0);
    }

    /// §13.8.4.7.2 (p. 640): a declared heat-pump value replaces 1,4·c_source,
    /// rounded down to 0,05; without a class no c_W;gen correction applies.
    #[test]
    fn declared_heat_pump_efficiency_replaces_the_table_value() {
        let declared = |class: Option<ApplicationClass>| HotWaterGenerator::HeatPump {
            exhaust_air_source: false,
            source_correction: None,
            measured_class: class,
            outdoor_air_fraction: None,
            same_ground_source: false,
            declared: Some(DeclaredEfficiency {
                value: 1.32,
                source_reference: "BCRG 0000/01".into(),
            }),
        };
        let (plain, practical) = generation(&declared(None), 2067.0).unwrap();
        assert!((plain - 1.30).abs() < 1e-9);
        // 13.152: a declared value is not a forfait value, so f_prac 0,95.
        assert!((practical - 0.95).abs() < 1e-12);
        let (classed, _) = generation(&declared(Some(ApplicationClass::Class4)), 2067.0).unwrap();
        assert!(classed < plain);
    }

    /// §13.8.4.7.2 (p. 640) rounds a declared value down to 0,05 (electric)
    /// or 0,025 (gas). A value below one step would become 0 and divide the
    /// carrier input by zero; it is refused, and `generation` never hands
    /// out a zero efficiency.
    #[test]
    fn declared_value_below_one_rounding_step_is_refused() {
        let heat_pump = |value: f64| HotWaterGenerator::HeatPump {
            exhaust_air_source: false,
            source_correction: None,
            measured_class: None,
            outdoor_air_fraction: None,
            same_ground_source: false,
            declared: Some(DeclaredEfficiency {
                value,
                source_reference: "BCRG 0000/01".into(),
            }),
        };
        let codes = |generator: &HotWaterGenerator| -> Vec<&'static str> {
            generator_issues(generator, "hotWater.generator")
                .into_iter()
                .map(|(code, _)| code)
                .collect()
        };
        assert!(codes(&heat_pump(0.04)).contains(&"hot_water_efficiency_invalid"));
        assert!(!codes(&heat_pump(0.05)).contains(&"hot_water_efficiency_invalid"));
        assert_eq!(
            generation(&heat_pump(1e-12), 2067.0),
            Err("hot_water_efficiency_invalid")
        );
        let indirect = |value: f64| HotWaterGenerator::IndirectBoiler {
            boiler: IndirectBoiler::Hr107,
            oil: false,
            inside_boundary: true,
            also_space_heating: true,
            declared: Some(DeclaredEfficiency {
                value,
                source_reference: "BCRG 0000/02".into(),
            }),
            pilot_flame: false,
        };
        assert!(codes(&indirect(0.02)).contains(&"hot_water_efficiency_invalid"));
        assert_eq!(
            generation(&indirect(0.02), 2067.0),
            Err("hot_water_efficiency_invalid")
        );
    }
}
