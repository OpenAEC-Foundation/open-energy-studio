//! Domestic hot water, NTA 8800:2025+C1:2026 chapter 13 (pages 525–655),
//! for one hot-water system serving all zones with one generator.
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
//! - `Q_W;dis = Q_W;em/η_W;dis + Q_W;sto;ls + Q_W;dis;conv;ls` (13.7);
//! - `E_W = Q_W;dis;nren / (f_prac · η_W;gen)` (13.3, 13.152) with tables
//!   13.25–13.28 and 13.18, the gas storage heater 13.165–13.175, and
//!   generator auxiliaries 13.181;
//! - ambient heat of heat pumps 5.36/5.37.
//!
//! Not modelled: solar water heating (§13.7, `Q_W;ren;sol = 0`), several
//! generators per system (13.8.2, `F_W;gen = 1`), booster heat pumps
//! (13.8.4.4, annex W), delivery sets on a collective heating system
//! (13.8.4.9.3) and measured multi-pattern tests (13.8.4.2). The
//! recoverable losses of 13.13 (13.47, 13.49, 13.63, 13.179) are reported
//! per month; generator losses of heat pumps and combis with an integrated
//! vessel (13.160a) are 0. Annex T and U test reports are evaluated in
//! [`crate::hot_water_tests`].

use crate::annex_w::{
    calculate_booster, validate_booster, BoosterHeatPump, BoosterHeatSource, BoosterSourceCarrier,
};
use crate::building_performance::Carrier;
use crate::climate::MONTH_HOURS;
use crate::hot_water_tests::{AnnexTTest, AnnexUTest};
use crate::label_class::LabelFunction;
use crate::monthly_demand::occupants_per_dwelling;
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
    /// 13.58 with a measured `H_sto;ls` (13.60), W/K; `f_sto;dis;ls = 1`.
    Measured {
        #[serde(rename = "transmissionWPerK")]
        transmission_w_per_k: f64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageVessel {
    pub id: String,
    pub volume_l: f64,
    pub loss: StorageLoss,
    /// `f_sto;dis;ls` 1–5 (§13.6.3); ignored for a measured `H_sto;ls`.
    pub connection_factor: u8,
    /// Placed in a heated zone; otherwise 13 °C or the given ambient.
    pub in_heated_zone: bool,
    #[serde(default)]
    pub unheated_ambient_c: Option<f64>,
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
            Self::Declared { efficiency, .. } => *efficiency,
            Self::AnnexU { test } => test.efficiency().unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShowerHeatRecovery {
    /// One entry per shower of the system (13.53).
    pub showers: Vec<ShowerUnit>,
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
    #[serde(default)]
    pub collective: Option<CollectiveHotWater>,
    pub equipment_reference: String,
}

impl HotWaterSystem {
    pub fn carrier(&self) -> HotWaterCarrier {
        match &self.generator {
            HotWaterGenerator::GasAppliance { .. } | HotWaterGenerator::GasStorageHeater { .. } => {
                HotWaterCarrier::Fuel(Carrier::Gas)
            }
            HotWaterGenerator::LargeDirectStorage { gas_fired: true } => {
                HotWaterCarrier::Fuel(Carrier::Gas)
            }
            HotWaterGenerator::IndirectBoiler { oil, .. } => {
                HotWaterCarrier::Fuel(if *oil { Carrier::Oil } else { Carrier::Gas })
            }
            HotWaterGenerator::ExternalHeat => HotWaterCarrier::DistrictHeat,
            HotWaterGenerator::BoosterHeatPump(pump) => match &pump.heat_source {
                BoosterHeatSource::ExternalHeat => HotWaterCarrier::DistrictHeat,
                BoosterHeatSource::CollectiveGenerator { carrier, .. } => {
                    HotWaterCarrier::Fuel(match carrier {
                        BoosterSourceCarrier::Gas => Carrier::Gas,
                        BoosterSourceCarrier::Oil => Carrier::Oil,
                        BoosterSourceCarrier::Electricity => Carrier::El,
                    })
                }
            },
            _ => HotWaterCarrier::Fuel(Carrier::El),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotWaterCarrier {
    Fuel(Carrier),
    DistrictHeat,
}

/// Context from outside chapter 13.
#[derive(Debug, Clone, Copy)]
pub struct HotWaterContext {
    pub residential: bool,
    /// `A_g;tot` of the assessed building (part), m².
    pub usable_floor_area_m2: f64,
    /// Heating setpoint of the heated zones (`ϑ_int;set;H`), °C.
    pub heated_ambient_c: f64,
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
    /// Q_W;ls;rbl (13.13) for the zones served, kWh.
    pub recoverable_loss_kwh: f64,
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

/// Table 13.18 `c_W,EU;gen`; `None` above the measured profile.
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
    Some(interpolate(points, annual_kwh))
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
                || (total - context.usable_floor_area_m2).abs()
                    > 1e-6 * context.usable_floor_area_m2.max(1.0)
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
        for (index, unit) in shower.showers.iter().enumerate() {
            if let ShowerUnit::AnnexU { test } = unit {
                for code in test.issues() {
                    push(code, &format!("showerHeatRecovery.showers[{index}].test"));
                }
            }
            if let ShowerUnit::Declared {
                efficiency,
                source_reference,
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
        if circulation.floor_count == 0 {
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
        if vessel.source_reference.trim().is_empty() {
            push(
                "source_reference_required",
                &format!("{base}.sourceReference"),
            );
        }
    }
    // §13.6.2: the storage loss is part of the efficiency of complete
    // appliances; separate vessels belong to boilers and indirect systems.
    let needs_storage = matches!(
        system.generator,
        HotWaterGenerator::ElectricBoiler
            | HotWaterGenerator::IndirectBoiler { .. }
            | HotWaterGenerator::IndirectHeatPump { .. }
    );
    let allows_storage =
        needs_storage || matches!(system.generator, HotWaterGenerator::ExternalHeat);
    if needs_storage && system.storage.is_empty() {
        push("hot_water_storage_required", "storage");
    }
    if !allows_storage && !system.storage.is_empty() {
        push("hot_water_storage_in_generator_efficiency", "storage");
    }
    if let HotWaterGenerator::GasAppliance {
        appliance,
        measured_class,
        declared,
        annex_t: Some(test),
        ..
    } = &system.generator
    {
        for code in test.issues() {
            push(code, "generator.annexT");
        }
        if declared.is_some() {
            push("hot_water_efficiency_declared_twice", "generator.annexT");
        }
        if measured_class.is_none() {
            push("annex_t_measured_class_required", "generator.measuredClass");
        }
        let combi = matches!(
            appliance,
            GasAppliance::CombiGaskeur | GasAppliance::CombiGaskeurHrCw
        );
        if combi != test.is_combi() {
            push("annex_t_appliance_mismatch", "generator.annexT.method");
        }
    }
    match &system.generator {
        HotWaterGenerator::GasAppliance { declared, .. }
        | HotWaterGenerator::IndirectBoiler { declared, .. } => {
            if let Some(item) = declared {
                if !positive(item.value) || item.value > 1.2 {
                    push("hot_water_efficiency_invalid", "generator.declared.value");
                }
                if item.source_reference.trim().is_empty() {
                    push(
                        "source_reference_required",
                        "generator.declared.sourceReference",
                    );
                }
            }
        }
        HotWaterGenerator::HeatPump {
            source_correction,
            outdoor_air_fraction,
            ..
        } => {
            if source_correction.is_some_and(|value| !positive(value) || value > 1.0) {
                push(
                    "hot_water_source_correction_invalid",
                    "generator.sourceCorrection",
                );
            }
            if outdoor_air_fraction.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
                push(
                    "hot_water_outdoor_fraction_invalid",
                    "generator.outdoorAirFraction",
                );
            }
        }
        HotWaterGenerator::HeatPumpEn16147 {
            delivered_kwh_per_day,
            input_kwh_per_day,
            outdoor_air_fraction,
            source_reference,
            ..
        } => {
            if !positive(*delivered_kwh_per_day) || !positive(*input_kwh_per_day) {
                push("hot_water_test_values_invalid", "generator");
            }
            if outdoor_air_fraction.is_some_and(|value| !(0.0..=1.0).contains(&value)) {
                push(
                    "hot_water_outdoor_fraction_invalid",
                    "generator.outdoorAirFraction",
                );
            }
            if source_reference.trim().is_empty() {
                push("source_reference_required", "generator.sourceReference");
            }
        }
        HotWaterGenerator::BoosterHeatPump(pump) => {
            for found in validate_booster(pump, "generator") {
                push(found.code, &found.path);
            }
        }
        HotWaterGenerator::GasStorageHeater {
            volume_l,
            measured_standby_kwh_per_day,
            ..
        } => {
            if !positive(*volume_l) {
                push("hot_water_storage_volume_invalid", "generator.volumeL");
            }
            if measured_standby_kwh_per_day.is_some_and(|value| !positive(value)) {
                push(
                    "hot_water_storage_loss_invalid",
                    "generator.measuredStandbyKwhPerDay",
                );
            }
        }
        _ => {}
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
            bathroom_length_m,
            ..
        } => {
            let kitchen = kitchen_emission(kitchen_length_m.unwrap_or(0.0));
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
fn generation(system: &HotWaterSystem, annual_output_kwh: f64) -> Result<(f64, f64), &'static str> {
    match &system.generator {
        HotWaterGenerator::GasAppliance {
            appliance,
            measured_class,
            kitchen_only,
            declared,
            annex_t,
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
            let correction = if *kitchen_only || !corrected {
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
            ..
        } => {
            let correction = heat_pump_class_correction(
                measured_class.unwrap_or(ApplicationClass::Class4),
                annual_output_kwh,
            )
            .ok_or("hot_water_heat_pump_class_exceeded")?;
            Ok((1.4 * source_correction.unwrap_or(1.0) * correction, 1.0))
        }
        HotWaterGenerator::HeatPumpEn16147 {
            profile,
            delivered_kwh_per_day,
            input_kwh_per_day,
            storage_without_legionella_cycle,
            ..
        } => {
            let correction = european_profile_correction(*profile, annual_output_kwh)
                .ok_or("hot_water_heat_pump_class_exceeded")?;
            let efficiency =
                round_down(delivered_kwh_per_day * correction / input_kwh_per_day, 0.05);
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
        HotWaterGenerator::ExternalHeat => Ok((1.0, 1.0)),
        // Annex W per month; see the month loop.
        HotWaterGenerator::BoosterHeatPump(_) => Ok((1.0, 1.0)),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotWaterAssessment {
    pub annual_net_need_kwh: f64,
    pub emission_efficiency: f64,
    pub annual_generator_output_kwh: f64,
    pub months: Vec<HotWaterMonth>,
}

/// Monthly chain; call only after [`validate_hot_water`] returned no issues.
pub fn assess_hot_water(
    system: &HotWaterSystem,
    context: HotWaterContext,
) -> Result<HotWaterAssessment, HotWaterIssue> {
    let area = context.usable_floor_area_m2;
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
    let year_hours: f64 = MONTH_HOURS.iter().sum();
    let eta_em = emission_efficiency(system);
    // 13.51/13.52 with 13.53.
    let recovery_factor = system.shower_heat_recovery.as_ref().map_or(0.0, |item| {
        let mean = item.showers.iter().map(ShowerUnit::efficiency).sum::<f64>()
            / item.showers.len() as f64;
        shower
            * mean
            * SHOWER_PRACTICAL_FACTOR
            * SHOWER_TEMPERATURE_FACTOR
            * item.connection.factor()
    });
    let need: [f64; 12] = std::array::from_fn(|index| annual * MONTH_HOURS[index] / year_hours);
    let emission_input: [f64; 12] =
        std::array::from_fn(|index| need[index] / eta_em - recovery_factor * need[index]);

    // Circulation (13.26–13.44).
    let mut circulation_loss = [0.0; 12];
    let mut pump = [0.0; 12];
    if let Some(circulation) = &system.circulation {
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
        let psi = circulation
            .declared_psi_w_per_mk
            .unwrap_or_else(|| table_13_4_psi(diameter, circulation.insulation));
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
        let unheated_ambient = circulation.unheated_ambient_c.unwrap_or(UNHEATED_AMBIENT_C);
        for (index, loss) in circulation_loss.iter_mut().enumerate() {
            *loss = f_building * MONTH_HOURS[index] / 1000.0
                * psi
                * ((CIRCULATION_MEAN_C - context.heated_ambient_c) * equivalent(heated)
                    + (CIRCULATION_MEAN_C - unheated_ambient) * equivalent(unheated));
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
        let ambient = if vessel.in_heated_zone {
            context.heated_ambient_c
        } else {
            vessel.unheated_ambient_c.unwrap_or(UNHEATED_AMBIENT_C)
        };
        let factor = f64::from(vessel.connection_factor);
        let label_c = StorageLabel::C.standing_loss_w(vessel.volume_l);
        let watts = match vessel.loss {
            StorageLoss::Measured {
                transmission_w_per_k,
            } => transmission_w_per_k * (set_temperature - ambient),
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
            let value = f_building * MONTH_HOURS[index] / 1000.0 * watts;
            *loss += value;
            if vessel.in_heated_zone {
                heated_storage_loss[index] += value;
            }
        }
    }

    // Delivery sets (13.24/13.24a, 13.46).
    let sets = system.delivery_sets.as_ref().map_or(0, |item| item.count);

    let mut months = Vec::with_capacity(12);
    let mut outputs = [0.0; 12];
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
        // 13.7 with Q_W;ren;sol = 0 (13.4).
        let output = emission / eta_dis + storage_loss[index] + conversion;
        outputs[index] = output;
        months.push(HotWaterMonth {
            month: index as u8 + 1,
            net_need_kwh: need[index],
            recovered_kwh: recovery_factor * need[index],
            emission_input_kwh: emission,
            circulation_loss_kwh: circulation_loss[index],
            storage_loss_kwh: storage_loss[index],
            conversion_loss_kwh: conversion,
            distribution_efficiency: eta_dis,
            generator_output_kwh: output,
            auxiliary_electricity_kwh: pump[index]
                + f64::from(sets) * STANDBY_ELECTRONICS_W * MONTH_HOURS[index] / 1000.0,
            ..HotWaterMonth::default()
        });
    }
    let annual_output: f64 = outputs.iter().sum();
    let (base, practical) = generation(system, annual_output).map_err(|code| HotWaterIssue {
        code,
        path: "hotWater.generator".into(),
    })?;
    for (index, row) in months.iter_mut().enumerate() {
        let output = row.generator_output_kwh;
        let mut efficiency = base;
        if let HotWaterGenerator::GasStorageHeater {
            volume_l,
            measured_standby_kwh_per_day,
            before_1985,
            in_heated_zone,
        } = &system.generator
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
            let ambient = if *in_heated_zone {
                context.heated_ambient_c
            } else {
                UNHEATED_AMBIENT_C
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
        row.generation_efficiency = practical_efficiency;
        row.carrier_input_kwh = output / practical_efficiency;
        // 13.181 for generators whose auxiliaries are not in the efficiency.
        let (electronics, burner) = match &system.generator {
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
        row.auxiliary_electricity_kwh +=
            (electronics * MONTH_HOURS[index] * f_building + burner * output * 1.1) / 1000.0;
        // 5.36/5.37
        let renewable_share = match &system.generator {
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
            _ => 0.0,
        };
        if renewable_share > 0.0 && practical_efficiency >= 1.0 {
            row.ambient_heat_kwh = output * (1.0 - 1.0 / practical_efficiency) * renewable_share;
        }
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
        // Above 500 m² the generator and vessels sit in a separate zone.
        let storage = if building_area > 500.0 {
            0.0
        } else {
            heated_storage_loss[index]
        };
        // 13.179: electric instantaneous heaters (individual appliances).
        let generation = match &system.generator {
            HotWaterGenerator::ElectricInstantaneous => output / practical_efficiency - output,
            _ => 0.0,
        };
        row.recoverable_loss_kwh = distribution + storage + generation;
    }
    // Annex W: the carrier input is the heat drawn from the collective
    // heating system (W.2) divided by its generation efficiency; the BWP
    // electricity (W.1) is auxiliary energy.
    if let HotWaterGenerator::BoosterHeatPump(pump) = &system.generator {
        let booster = calculate_booster(pump, &outputs);
        let source_efficiency = match &pump.heat_source {
            BoosterHeatSource::ExternalHeat => 1.0,
            BoosterHeatSource::CollectiveGenerator {
                generation_efficiency,
                ..
            } => *generation_efficiency,
        };
        for (row, month) in months.iter_mut().zip(booster) {
            row.carrier_input_kwh = month.heating_system_heat_kwh / source_efficiency;
            row.auxiliary_electricity_kwh += month.electricity_kwh;
            let input = month.heating_system_heat_kwh + month.electricity_kwh;
            row.generation_efficiency = if input > 0.0 {
                row.generator_output_kwh / input
            } else {
                0.0
            };
            row.ambient_heat_kwh = 0.0;
        }
    }
    Ok(HotWaterAssessment {
        annual_net_need_kwh: annual,
        emission_efficiency: eta_em,
        annual_generator_output_kwh: annual_output,
        months,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> HotWaterContext {
        HotWaterContext {
            residential: true,
            usable_floor_area_m2: 100.0,
            heated_ambient_c: 20.0,
        }
    }

    fn system(generator: HotWaterGenerator) -> HotWaterSystem {
        HotWaterSystem {
            need: HotWaterNeed::Residential {
                dwelling_count: 1,
                source_reference: "one dwelling".into(),
            },
            emission: HotWaterEmission::Residential {
                served: ServedTaps::KitchenAndBathroom,
                kitchen_length_m: Some(3.0),
                bathroom_length_m: Some(5.0),
                source_reference: "drawing".into(),
            },
            shower_heat_recovery: None,
            circulation: None,
            storage: Vec::new(),
            delivery_sets: None,
            boiling_water_tap: false,
            generator,
            collective: None,
            equipment_reference: "plate".into(),
        }
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
    }

    fn combi() -> HotWaterGenerator {
        HotWaterGenerator::GasAppliance {
            appliance: GasAppliance::CombiGaskeurHrCw,
            measured_class: Some(ApplicationClass::Class4),
            kitchen_only: false,
            declared: None,
            annex_t: None,
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
    fn shower_recovery_is_subtracted_after_emission_efficiency() {
        let mut input = system(combi());
        input.shower_heat_recovery = Some(ShowerHeatRecovery {
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
    fn circulation_storage_and_pump_follow_13_25_to_13_59() {
        let mut input = system(HotWaterGenerator::IndirectBoiler {
            boiler: IndirectBoiler::Hr107,
            oil: false,
            inside_boundary: true,
            also_space_heating: true,
            declared: None,
        });
        input.circulation = Some(Circulation {
            outer_diameter_mm: Some(15.0),
            insulation: PipeInsulation::Mm15,
            declared_psi_w_per_mk: None,
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
            in_heated_zone: true,
            unheated_ambient_c: None,
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
            source_reference: "EN 16147 report".into(),
        };
        let result = assess_hot_water(&input, context()).unwrap();
        assert_eq!(result.months[0].ambient_heat_kwh, 0.0);
        let correction =
            european_profile_correction(TappingProfile::Xl, result.annual_generator_output_kwh)
                .unwrap();
        let eta = round_down(11.0 * correction / 4.0, 0.05) * 0.9;
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
    }

    #[test]
    fn utility_need_storage_label_and_delivery_sets() {
        let ctx = HotWaterContext {
            residential: false,
            usable_floor_area_m2: 1000.0,
            heated_ambient_c: 21.0,
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
        });
        let run = |r: f64| ShowerRun::Energies {
            recovered_kj: r,
            shower_kj: 100.0,
        };
        input.shower_heat_recovery = Some(ShowerHeatRecovery {
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
            in_heated_zone: true,
            unheated_ambient_c: None,
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

    #[test]
    fn validation_rejects_inconsistent_input() {
        let mut bad = system(HotWaterGenerator::ElectricBoiler);
        bad.emission = HotWaterEmission::Residential {
            served: ServedTaps::KitchenAndBathroom,
            kitchen_length_m: None,
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
            in_heated_zone: true,
            unheated_ambient_c: None,
            source_reference: "label".into(),
        });
        assert!(validate_hot_water(&combined, context(), "dhw")
            .iter()
            .any(|item| item.code == "hot_water_storage_in_generator_efficiency"));
    }
}
