//! Basic survey (basisopname) of existing utility buildings, ISSO 75.1
//! (7e druk, printdatum 11-12-2025).
//!
//! The utility layer reuses the residential envelope, heating, hot-water
//! generator and PV translations where ISSO 75.1 prescribes the same tables
//! and recognition rules as ISSO 82.1, and rewrites their citations to the
//! ISSO 75.1 pages. Utility-specific parts are added here: use functions
//! (small functions merged into the main function, p. 39–40), building type
//! for infiltration (p. 55–56), BACS (table 7.3, p. 62–63), collective
//! heating (table 9.7, p. 108–115), cooling (chapter 10, p. 128–138),
//! utility ventilation with AHU, recirculation and flow control (chapter 11,
//! p. 140–158), humidification (chapter 12, p. 161), utility hot water
//! (chapter 13, p. 164–178) and lighting (chapter 14, p. 182–189).
//!
//! One calculation zone per building: the main use function carries the
//! whole A_g. Buildings where the other functions exceed 25 % of A_g are
//! rejected (`mixed_functions_require_zones`) until the kernel supports
//! area-weighted multi-function zones (§6.5.3).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::envelope::{derive_envelope, SurveyEnvelope};
use super::general::{infiltration_year, thermal_mass, Construction, Renovation};
use super::heating::{derive_heating, SurveyHeating};
use super::hot_water::{
    derive_hot_water, GasApplianceType, GaskeurAnswer, HotWaterGeneratorAnswer,
    ShowerRecoveryAnswer, SurveyHotWater, TapsServed,
};
use super::production::{derive_pv, SurveyPv};
use super::ventilation::{ExchangerAnswer, MotorAnswer, PressureClass, VentilationPrinciple};
use super::{
    loss_area, AppliedDefault, MeasuredInfiltration, OpnameAssessment, OpnameIssue, Recorder,
};
use crate::building_performance::{
    assess_building_performance, BuildingPerformanceAssessment, BuildingPerformanceInput,
};
use crate::humidification::{
    calculate_humidity, CoolingDesignTemperature as DehumidificationDesign, Humidification,
    Humidifier, HumidityFunction, HumidityFunctionArea, HumidityInput, SteamCarrier,
};
use crate::label_class::LabelFunction;
use crate::monthly_demand::UsageFunction;
use crate::ventilation::AirtightnessType;

pub const ISSO_UTILITY_SOURCE: &str = "ISSO 75.1 (7e druk, printdatum 11-12-2025)";

/// p. 39–40: other functions may be merged into the main function up to
/// 25 % of A_g in total.
pub const MERGE_LIMIT: f64 = 0.25;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FunctionArea {
    pub function: LabelFunction,
    pub area_m2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SingleLayerPosition {
    Detached,
    EndOrCorner,
    Terraced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UtilityRoof {
    Pitched,
    /// At least 50 % flat; only for detached buildings (p. 56).
    PartlyFlat,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreyLevel {
    Bottom,
    Intermediate,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreyPosition {
    EndOrCorner,
    Middle,
    WholeStorey,
}

/// Building type and position for the infiltration value (p. 55–56).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UtilityBuildingType {
    SingleLayer {
        position: SingleLayerPosition,
        roof: UtilityRoof,
    },
    /// Whole multi-storey building.
    MultiLayerWhole,
    /// Part or unit of a multi-storey building on one storey level.
    MultiLayerPart {
        level: StoreyLevel,
        position: StoreyPosition,
    },
}

/// Table 7.3 (p. 62–63).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyBacs {
    /// Summed nominal power of the largest heating or cooling system, kW.
    #[serde(default)]
    pub system_power_kw: Option<f64>,
    /// A_g served by that system, m² (used when the power is unknown).
    #[serde(default)]
    pub served_area_m2: Option<f64>,
    #[serde(default)]
    pub present: Option<bool>,
    #[serde(default)]
    pub automation_class_c_or_better: Option<bool>,
    #[serde(default)]
    pub management_class_b_or_better: Option<bool>,
    #[serde(default)]
    pub evidence_reference: Option<String>,
}

/// Table 9.7/§9.3 (p. 108–115): installation type and generator power.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatingInstallation {
    pub collective: bool,
    /// Nominal power, kW (boiler at 50/30, heat pump at A7/W35, B0/W35 or
    /// W10/W35); required for a collective boiler.
    #[serde(default)]
    pub capacity_kw: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingGeneratorAnswer {
    Compression,
    RoomAirConditioner,
    GasAbsorption,
    ExternalCold,
    /// Collective installation, generator not determinable.
    UnknownCollective,
    AquiferBefore2013,
    AquiferFrom2013,
    /// ATES, realisation year unknown: permit year, else before 2013.
    AquiferYearUnknown,
    SurfaceWater,
    ClosedGroundLoop,
    DewPointCooling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingEmitterAnswer {
    FloorCooling,
    /// Concrete core activation counts as floor cooling (p. 138).
    ConcreteCoreActivation,
    WallCooling,
    CeilingCooling,
    FanCoilOnOuterWall,
    FanCoilOnCeiling,
    /// Split or VRF indoor units: fan convectors (p. 138).
    SplitIndoorUnitsOnWall,
    SplitIndoorUnitsOnCeiling,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingControlAnswer {
    Standalone,
    CentralWithRoomControl,
    OtherOrUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingDesignAnswer {
    T6To12,
    T12To16,
    T12To18,
    T17To21,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyCooling {
    pub generator: CoolingGeneratorAnswer,
    #[serde(default)]
    pub capacity_kw: Option<f64>,
    pub emitter: CoolingEmitterAnswer,
    #[serde(default)]
    pub fan_coil_count: u32,
    /// Water-based distribution (false for direct expansion).
    pub water_based: bool,
    #[serde(default)]
    pub design_temperature: Option<CoolingDesignAnswer>,
    #[serde(default)]
    pub balanced: Option<bool>,
    #[serde(default)]
    pub control: Option<CoolingControlAnswer>,
    /// Pipes insulated; `None` unknown (table 10.7).
    #[serde(default)]
    pub pipes_insulated: Option<bool>,
    #[serde(default)]
    pub pipe_insulation_year: Option<i32>,
    /// Permit year of an ATES of unknown realisation year (p. 131).
    #[serde(default)]
    pub aquifer_permit_year: Option<i32>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecirculationAnswer {
    None,
    PresentPercentUnknown,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowControlMethodAnswer {
    Throttle,
    InletVane,
    BladePitch,
    SpeedControl,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyFlowControl {
    pub method: FlowControlMethodAnswer,
    /// Lowest flow as a percentage of the maximum; `None` unknown (≥ 80 %).
    #[serde(default)]
    pub minimum_percent: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DuctLengthAnswer {
    AtMost20M,
    From20To40M,
    AtLeast40M,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyAhu {
    #[serde(default)]
    pub inside_thermal_zone: Option<bool>,
    /// Supply ducts run outside the thermal zone; `None` unknown.
    #[serde(default)]
    pub ducts_outside_thermal_zone: Option<bool>,
    #[serde(default)]
    pub duct_length: Option<DuctLengthAnswer>,
    /// R ≥ 1,0 m²K/W; `None` unknown (table 11.14).
    #[serde(default)]
    pub ducts_insulated: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UtilityVentilation {
    pub principle: VentilationPrinciple,
    /// Table 11.5 variant (e.g. "c4c") when controls are proven.
    #[serde(default)]
    pub declared_variant: Option<String>,
    #[serde(default)]
    pub self_regulating_vents: Option<bool>,
    #[serde(default)]
    pub pressure_class: Option<PressureClass>,
    #[serde(default)]
    pub installation_year: Option<i32>,
    #[serde(default)]
    pub heat_recovery: Option<ExchangerAnswer>,
    #[serde(default)]
    pub bypass_present: Option<bool>,
    #[serde(default)]
    pub unit_manufacture_year: Option<i32>,
    #[serde(default)]
    pub motor: Option<MotorAnswer>,
    /// LUKA A–C proven (f_lea;du 1,05); `None` unknown (1,1).
    #[serde(default)]
    pub ducts_luka_abc: Option<bool>,
    #[serde(default)]
    pub ahu: Option<SurveyAhu>,
    #[serde(default)]
    pub recirculation: Option<RecirculationAnswer>,
    /// Proven recirculation percentage (rounded down to tens).
    #[serde(default)]
    pub recirculation_percent: Option<u32>,
    #[serde(default)]
    pub flow_control: Option<SurveyFlowControl>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HumidifierAnswer {
    ElectricSteam,
    NonElectricSteam,
    Adiabatic,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyHumidification {
    pub humidifier: HumidifierAnswer,
    /// Absorption (sorption) heat wheel; recirculation does not count.
    #[serde(default)]
    pub absorption_wheel: bool,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UtilityHotWaterGenerator {
    /// No hot-water system in the building: electric instantaneous (p. 164).
    None,
    GasAppliance {
        #[serde(rename = "applianceType")]
        appliance_type: GasApplianceType,
        gaskeur: GaskeurAnswer,
        #[serde(default, rename = "burnerLoadKw")]
        burner_load_kw: Option<f64>,
    },
    ElectricBoiler,
    ElectricInstantaneous,
    HeatPump {
        #[serde(rename = "exhaustAirSource")]
        exhaust_air_source: bool,
    },
    DistrictHeat,
    /// Gas-fired storage heater up to 150 kW (p. 167).
    GasStorageHeater {
        #[serde(rename = "volumeL")]
        volume_l: f64,
        /// Vessel manufactured before 1985; `None` follows the
        /// construction year.
        #[serde(default, rename = "before1985")]
        before_1985: Option<bool>,
        /// `None`: outside the thermal zone (p. 167).
        #[serde(default, rename = "inHeatedZone")]
        in_heated_zone: Option<bool>,
    },
    /// Collective generator not determinable: other directly heated
    /// storage (table 13.2, p. 165).
    CollectiveUnknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UtilityHotWater {
    pub generator: UtilityHotWaterGenerator,
    /// Mean draw-off length, m; `None` unknown (> 3 m, p. 177).
    #[serde(default)]
    pub mean_draw_off_length_m: Option<f64>,
    /// Circulation present; `None` unknown (no circulation entered).
    #[serde(default)]
    pub circulation: Option<bool>,
    #[serde(default = "one")]
    pub showers: u32,
    pub shower_heat_recovery: ShowerRecoveryAnswer,
    /// Vessels whose losses are not part of the generator efficiency
    /// (electric and indirectly heated boilers, p. 172).
    #[serde(default)]
    pub storage: Vec<SurveyStorage>,
    pub source_reference: String,
}

/// Connection of the hot pipes to a vessel (p. 173).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VesselConnection {
    /// Four or more connections, tees, valves and unused points insulated.
    FullyInsulated,
    /// Four connections, only straight pipe insulated, no heat trap.
    StraightOnlyFour,
    /// More than four connections, only straight pipe insulated.
    StraightOnlyMoreThanFour,
    Uninsulated,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyStorage {
    pub id: String,
    pub volume_l: f64,
    /// Energy label (vessels ≤ 500 l); `None` follows the manufacture year.
    #[serde(default)]
    pub label: Option<crate::domestic_hot_water::StorageLabel>,
    /// Manufactured from 2018; `None` unknown (up to 2017).
    #[serde(default)]
    pub produced_from_2018: Option<bool>,
    #[serde(default)]
    pub connection: Option<VesselConnection>,
    /// `None` unknown: outside the thermal zone (table 13.10).
    #[serde(default)]
    pub in_heated_zone: Option<bool>,
    pub source_reference: String,
}

fn one() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LampTypeAnswer {
    T12,
    T8Conventional,
    T8HighFrequency,
    T5,
    LedInLuminaire,
    CflPlugIn,
    CflIntegrated,
    IncandescentOrHalogen,
    LedLamp,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LuminaireCount {
    pub count: u32,
    pub power_w: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LampCount {
    pub count: u32,
    pub lamp_power_w: f64,
    pub lamp_type: LampTypeAnswer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum LightingPowerAnswer {
    Luminaires {
        luminaires: Vec<LuminaireCount>,
    },
    Lamps {
        lamps: Vec<LampCount>,
    },
    /// Power not determinable or no lighting present (table 14.5).
    Unknown {
        #[serde(default, rename = "ledFrom2017")]
        led_from_2017: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceControlAnswer {
    NoneOrCentralOn,
    /// Room switches visible, no sensors.
    RoomSwitch,
    RoomSwitchWithSweep,
    /// Sensors visible, type unknown: automatic on, dimmed (p. 183).
    SensorsTypeUnknown,
    AutoOnDimmed,
    AutoOnAutoOff,
    ManualOnDimmed,
    ManualOnAutoOff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DaylightAnswer {
    None,
    Switching,
    Dimming,
    /// Sensors present, type unknown: switching (table 14.3).
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyLightingZone {
    pub id: String,
    pub area_m2: f64,
    pub power: LightingPowerAnswer,
    pub presence: PresenceControlAnswer,
    #[serde(default = "daylight_none")]
    pub daylight: DaylightAnswer,
    /// Office switching zone larger than 30 m² (p. 183).
    #[serde(default)]
    pub large_office_group: bool,
    /// At least 70 % of the power in extracted luminaires (p. 186).
    #[serde(default)]
    pub extracted_luminaires: bool,
    pub source_reference: String,
}

fn daylight_none() -> DaylightAnswer {
    DaylightAnswer::None
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UtilitySurvey {
    pub id: String,
    pub construction_year: i32,
    #[serde(default)]
    pub renovation: Option<Renovation>,
    pub building_type: UtilityBuildingType,
    /// EP-liable use functions with A_g per NEN 2580 (p. 63–64).
    pub functions: Vec<FunctionArea>,
    pub area_source_reference: String,
    pub building_height_m: f64,
    /// Storeys served by the installations (≥ 1,5 m clear height, p. 65).
    #[serde(default = "one")]
    pub storeys: u32,
    pub construction: Construction,
    #[serde(default)]
    pub measured_infiltration: Option<MeasuredInfiltration>,
    pub envelope: SurveyEnvelope,
    /// Windows with evident solar-control glass or film (g 0,4, table 8.14).
    #[serde(default)]
    pub solar_control_window_ids: Vec<String>,
    pub heating: SurveyHeating,
    pub heating_installation: HeatingInstallation,
    #[serde(default)]
    pub cooling: Option<SurveyCooling>,
    pub ventilation: UtilityVentilation,
    #[serde(default)]
    pub humidification: Option<SurveyHumidification>,
    pub hot_water: UtilityHotWater,
    pub lighting: Vec<SurveyLightingZone>,
    #[serde(default)]
    pub pv: Vec<SurveyPv>,
    #[serde(default)]
    pub bacs: SurveyBacs,
    pub source_reference: String,
}

/// ISSO 75.1 page for each rule of the shared (ISSO 82.1) translations.
fn utility_source(rule: &str) -> Option<&'static str> {
    Some(match rule {
        "device_year_from_installation" | "device_year_from_construction_year" => "ISSO 75.1 p. 30",
        "renovation_year_not_applicable" => "ISSO 75.1 p. 57–58 (afb. 7.2)",
        "renovation_year_unknown_next_class" => "ISSO 75.1 p. 59",
        "glazing_without_frame_wood_plastic"
        | "glazing_coating_type_unknown_hr"
        | "glazing_double_with_coating_hr"
        | "glazing_double_with_secondary_hr"
        | "glazing_hr_with_secondary_hr_plus_plus"
        | "glazing_hr_plus_plus_with_secondary_triple"
        | "glazing_leaded_light_single"
        | "glazing_glass_blocks_double" => "ISSO 75.1 p. 96",
        "door_insulation_unknown_uninsulated" => "ISSO 75.1 p. 30 (conservative), p. 97",
        "door_split_window_and_door" => "ISSO 75.1 p. 74",
        "cavity_width_unknown_table_8_26" => "ISSO 75.1 p. 88",
        "opaque_rc_forfait_annex_i" => "ISSO 75.1 p. 88–93 (tables 8.9–8.11), NTA annex I",
        "crawlspace_bottom" | "crawlspace_wall_from_facade" => "ISSO 75.1 p. 95",
        "thermal_bridges_forfait_delta_u" => "ISSO 75.1 p. 83–84; NTA 8.2/8.3",
        "frame_fraction_forfait" => "NTA 7.6.6.2 method B; ISSO 75.1 p. 87",
        "unheated_space_basic_survey_h_ue" => "NTA I.8 (I.2.4) and 8.53; ISSO 75.1 basisopname",
        "no_generator_conventional_boiler" => "ISSO 75.1 p. 109",
        "pilot_flame_unknown_present" | "hydrogen_boiler_hr107" => "ISSO 75.1 p. 110",
        "individual_surface_water_as_ground" | "heat_pump_water_source_unknown_ground" => {
            "ISSO 75.1 p. 112 (table 9.6)"
        }
        "design_temperature_class_unknown" => "ISSO 75.1 p. 116–117 (table 9.9)",
        "several_emitters_surface_heating_first" => "ISSO 75.1 p. 125",
        "hydronic_balancing_unknown_none" => "ISSO 75.1 p. 118",
        "emission_control_unknown_other" => "ISSO 75.1 p. 126",
        "pipe_insulation_unknown_uninsulated" => "ISSO 75.1 p. 120",
        "heat_meter_unknown_present" => "ISSO 75.1 p. 123",
        "pump_unknown_forfait" => "ISSO 75.1 p. 119",
        "no_hot_water_system_electric_instantaneous" => "ISSO 75.1 p. 164",
        "gas_appliance_type_unknown_bath_geyser"
        | "kitchen_geyser_above_13_kw_bath_geyser"
        | "gaskeur_unknown_none"
        | "cw_class_unknown_cw_4_5_6" => "ISSO 75.1 p. 168",
        "shower_heat_recovery_unknown_not_connected" => "ISSO 75.1 p. 178",
        "pv_year_unknown_construction_year"
        | "pv_type_unknown_polycrystalline"
        | "pv_amorphous_unknown_multi_junction" => "ISSO 75.1 p. 197",
        "pv_mounting_unknown_not_ventilated" => "ISSO 75.1 p. 196–197",
        _ => return None,
    })
}

fn remap_sources(applied: &mut [AppliedDefault]) {
    for item in applied.iter_mut() {
        if let Some(source) = utility_source(item.rule) {
            item.source = source;
        }
    }
}

fn label_name(function: LabelFunction) -> &'static str {
    match function {
        LabelFunction::Residential => "residential",
        LabelFunction::Office => "office",
        LabelFunction::AssemblyWithoutDayCare => "assembly_without_day_care",
        LabelFunction::AssemblyWithDayCare => "assembly_with_day_care",
        LabelFunction::Education => "education",
        LabelFunction::HealthcareWithoutBeds => "healthcare_without_beds",
        LabelFunction::HealthcareWithBeds => "healthcare_with_beds",
        LabelFunction::Retail => "retail",
        LabelFunction::Sport => "sport",
        LabelFunction::Lodging => "lodging",
        LabelFunction::Cell => "cell",
    }
}

/// Kernel names per chapter for one use function: (usage function of
/// chapter 7, ventilation function, distribution reduction function).
fn kernel_names(function: LabelFunction) -> (UsageFunction, &'static str, &'static str) {
    match function {
        LabelFunction::Office => (UsageFunction::Office, "office", "office"),
        LabelFunction::AssemblyWithoutDayCare => {
            (UsageFunction::OtherAssembly, "other_assembly", "assembly")
        }
        LabelFunction::AssemblyWithDayCare => (
            UsageFunction::AssemblyChildCare,
            "assembly_child_care",
            "assembly",
        ),
        LabelFunction::Education => (UsageFunction::Education, "education", "education"),
        LabelFunction::HealthcareWithoutBeds => (
            UsageFunction::OtherHealthcare,
            "other_healthcare",
            "healthcare_other",
        ),
        LabelFunction::HealthcareWithBeds => (
            UsageFunction::HealthcareWithBeds,
            "healthcare_bed_area",
            "healthcare_with_beds",
        ),
        LabelFunction::Retail => (UsageFunction::Retail, "retail", "retail"),
        LabelFunction::Sport => (UsageFunction::Sport, "sport", "sport"),
        LabelFunction::Lodging => (UsageFunction::Lodging, "lodging_building", "lodging"),
        LabelFunction::Cell => (UsageFunction::Cell, "cell", "cell"),
        LabelFunction::Residential => (UsageFunction::Residential, "residential", "residential"),
    }
}

fn usage_name(function: UsageFunction) -> &'static str {
    match function {
        UsageFunction::AssemblyChildCare => "assembly_child_care",
        UsageFunction::OtherAssembly => "other_assembly",
        UsageFunction::Cell => "cell",
        UsageFunction::HealthcareWithBeds => "healthcare_with_beds",
        UsageFunction::OtherHealthcare => "other_healthcare",
        UsageFunction::Office => "office",
        UsageFunction::Lodging => "lodging",
        UsageFunction::Education => "education",
        UsageFunction::Sport => "sport",
        UsageFunction::Retail => "retail",
        UsageFunction::Residential => "residential",
    }
}

/// p. 39–40: the main function and whether the others fit within 25 %.
pub fn main_function(
    functions: &[FunctionArea],
    recorder: &mut Recorder,
) -> Option<(LabelFunction, f64)> {
    let total: f64 = functions.iter().map(|item| item.area_m2).sum();
    let mut groups: Vec<(LabelFunction, f64)> = Vec::new();
    for item in functions {
        match groups
            .iter_mut()
            .find(|(function, _)| *function == item.function)
        {
            Some(group) => group.1 += item.area_m2,
            None => groups.push((item.function, item.area_m2)),
        }
    }
    let (main, area) = groups.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1))?;
    let others = total - area;
    if others > MERGE_LIMIT * total + 1e-9 {
        recorder.issue("mixed_functions_require_zones", "functions");
        return None;
    }
    if others > 0.0 {
        recorder.record(
            "small_functions_merged_into_main",
            "functions",
            format!(
                "{} m² of other functions (≤ 25 %) counted as {}",
                round1(others),
                label_name(main)
            ),
            "ISSO 75.1 p. 39–40",
        );
    }
    Some((main, total))
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

/// NTA table 11.14 type for the utility building (p. 55–56).
pub fn utility_airtightness(
    building: &UtilityBuildingType,
    recorder: &mut Recorder,
) -> AirtightnessType {
    match building {
        UtilityBuildingType::SingleLayer { position, roof } => {
            let roof = match (roof, position) {
                (UtilityRoof::PartlyFlat, SingleLayerPosition::Detached) => UtilityRoof::PartlyFlat,
                (UtilityRoof::PartlyFlat, _) => {
                    recorder.record(
                        "partly_flat_roof_only_detached",
                        "buildingType.roof",
                        "pitched".into(),
                        "ISSO 75.1 p. 56",
                    );
                    UtilityRoof::Pitched
                }
                (other, _) => *other,
            };
            match (roof, position) {
                (UtilityRoof::Pitched, SingleLayerPosition::Terraced) => {
                    AirtightnessType::PitchedRoofTerraced
                }
                (UtilityRoof::Pitched, SingleLayerPosition::EndOrCorner) => {
                    AirtightnessType::PitchedRoofEndOrCorner
                }
                (UtilityRoof::Pitched, SingleLayerPosition::Detached) => {
                    AirtightnessType::PitchedRoofDetached
                }
                (UtilityRoof::PartlyFlat, _) => AirtightnessType::PitchedRoofDetachedPartlyFlat,
                (UtilityRoof::Flat, SingleLayerPosition::Terraced) => {
                    AirtightnessType::FlatRoofTerraced
                }
                (UtilityRoof::Flat, SingleLayerPosition::EndOrCorner) => {
                    AirtightnessType::FlatRoofEndOrCorner
                }
                (UtilityRoof::Flat, SingleLayerPosition::Detached) => {
                    AirtightnessType::FlatRoofDetached
                }
            }
        }
        UtilityBuildingType::MultiLayerWhole => AirtightnessType::MultiStoreyWholeBuilding,
        UtilityBuildingType::MultiLayerPart { level, position } => match (level, position) {
            (StoreyLevel::Top, StoreyPosition::Middle) => AirtightnessType::StoreyMiddleTop,
            (StoreyLevel::Top, StoreyPosition::EndOrCorner) => AirtightnessType::StoreyEndTop,
            (StoreyLevel::Top, StoreyPosition::WholeStorey) => {
                AirtightnessType::MultiStoreyWholeTopLayer
            }
            (_, StoreyPosition::Middle) => AirtightnessType::StoreyMiddleLowerOrIntermediate,
            (_, StoreyPosition::EndOrCorner) => AirtightnessType::StoreyEndLowerOrIntermediate,
            (StoreyLevel::Bottom, StoreyPosition::WholeStorey) => {
                AirtightnessType::MultiStoreyWholeBottomLayer
            }
            (StoreyLevel::Intermediate, StoreyPosition::WholeStorey) => {
                AirtightnessType::MultiStoreyWholeIntermediateLayer
            }
        },
    }
}

/// Table 7.3 and §5.5.8: f_BACS 1,05 when a system above 290 kW (or
/// serving more than 2 500 m² when the power is unknown) lacks a compliant
/// BACS.
pub fn bacs_factor(bacs: &SurveyBacs, recorder: &mut Recorder) -> (f64, String) {
    let large = match bacs.system_power_kw {
        Some(power) => power > 290.0,
        None => {
            let large = bacs.served_area_m2.is_some_and(|area| area > 2500.0);
            recorder.record(
                "bacs_power_unknown_served_area",
                "bacs.systemPowerKw",
                format!("> 290 kW: {large} (served A_g > 2 500 m²)"),
                "ISSO 75.1 p. 62–63 (table 7.3)",
            );
            large
        }
    };
    if !large {
        return (1.0, "f_BACS 1,0: no system above 290 kW (§5.5.8)".into());
    }
    let present = bacs.present.unwrap_or_else(|| {
        recorder.record(
            "bacs_presence_unknown_no",
            "bacs.present",
            "false".into(),
            "ISSO 75.1 p. 63 (table 7.3)",
        );
        false
    });
    let automation = present
        && bacs.automation_class_c_or_better.unwrap_or_else(|| {
            recorder.record(
                "bacs_automation_class_unknown_d",
                "bacs.automationClassCOrBetter",
                "class D".into(),
                "ISSO 75.1 p. 63 (table 7.3)",
            );
            false
        });
    let management = present
        && bacs.management_class_b_or_better.unwrap_or_else(|| {
            recorder.record(
                "bacs_management_class_unknown_c_d",
                "bacs.managementClassBOrBetter",
                "class C or D".into(),
                "ISSO 75.1 p. 63 (table 7.3)",
            );
            false
        });
    if automation && management {
        if bacs
            .evidence_reference
            .as_deref()
            .map_or(true, |item| item.trim().is_empty())
        {
            recorder.issue("source_reference_required", "bacs.evidenceReference");
        }
        (1.0, "f_BACS 1,0: compliant BACS (§5.5.8)".into())
    } else {
        (
            1.05,
            "f_BACS 1,05: system above 290 kW without compliant BACS (§5.5.8, ISSO 75.1 table 7.3)"
                .into(),
        )
    }
}

fn design_class_from_mean(mean: f64) -> &'static str {
    if mean <= 45.0 {
        "45_40"
    } else if mean <= 55.0 {
        "55_47"
    } else if mean <= 65.0 {
        "70_60"
    } else {
        "90_70"
    }
}

/// Utility heating: the residential translation plus collective role,
/// heat-pump scope, 9.91 auxiliaries and the calculated distribution.
fn derive_utility_heating(
    survey: &UtilitySurvey,
    reduction_function: &str,
    area: f64,
    recorder: &mut Recorder,
) -> super::heating::DerivedHeating {
    let mut derived = derive_heating(&survey.heating, survey.construction_year, recorder);
    let installation = &survey.heating_installation;
    let reference = survey.heating.source_reference.as_str();
    let hydronic = !matches!(
        survey.heating.emitters,
        super::heating::Emitters::AirHeating | super::heating::Emitters::LocalHeaters
    );
    let generator = &mut derived.generator;
    let kind = generator["kind"].as_str().unwrap_or_default().to_string();
    // p. 17/117: the technical room of a large installation (A_g served
    // > 500 m²) lies outside the thermal zone by definition.
    let location = if kind == "gas_boiler" {
        Some(&mut generator["boiler"]["location"])
    } else if kind == "biomass" {
        Some(&mut generator["location"])
    } else {
        None
    };
    if let Some(location) = location {
        if area > 500.0 && *location == json!("inside_thermal_boundary") {
            *location = json!("outside_thermal_boundary");
            recorder.record(
                "large_installation_outside_thermal_zone",
                "heating.generator.insideThermalBoundary",
                format!("outside (A_g {area} m² > 500 m²)"),
                "ISSO 75.1 p. 17, 117",
            );
        }
    }
    if kind == "gas_boiler" && installation.collective {
        generator["boiler"]["role"] = json!("collective");
        match installation.capacity_kw {
            Some(power) => {
                generator["auxiliary"] = json!({
                    "electricallyConnectedDevices": 1,
                    "nominalPowerKw": power,
                    "sourceReference": reference,
                });
            }
            None => recorder.issue(
                "collective_generator_power_required",
                "heatingInstallation.capacityKw",
            ),
        }
    }
    if kind == "heat_pump_forfait" {
        generator["forfait"]["scope"] = json!("utility_collective_or_over25_kw");
        // c_source (table 9.27 footnote a) and the table 9.28 row are for
        // the residential table only.
        generator["forfait"]["sourceCorrectionFactor"] = Value::Null;
        generator["forfait"]["sourceCorrectionReference"] = Value::Null;
        recorder
            .applied
            .retain(|item| item.rule != "source_regeneration_none_c_source_1");
        generator["forfait"]["collectiveBuildingInstallation"] = json!(installation.collective);
        if let Some(power) = installation.capacity_kw {
            generator["forfait"]["thermalCapacityKw"] = json!(power);
            generator["forfait"]["capacitySourceReference"] = json!(reference);
        }
        if installation.collective {
            let mut auxiliary =
                json!({"electricallyConnectedDevices": 1, "sourceReference": reference});
            if let Some(power) = installation.capacity_kw {
                auxiliary["nominalPowerKw"] = json!(power);
            }
            generator["auxiliary"] = auxiliary;
        }
    }
    if installation.collective && matches!(kind.as_str(), "gas_boiler" | "heat_pump_forfait") {
        recorder.record(
            "collective_generator_one_connected_device",
            "heating.generator.auxiliary",
            "1 electrically connected device (10 W stand-by)".into(),
            "NTA 8800 9.91 (interpretation: one device per surveyed generator)",
        );
    }
    // A collective boiler's pump is not in 9.85: the calculated
    // distribution (9.26–9.51) with the forfait pump applies.
    let needs_distribution =
        hydronic && (derived.distribution_system.is_some() || installation.collective);
    if needs_distribution {
        let mean = generator["boiler"]["averageDesignEmissionTemperatureC"]
            .as_f64()
            .or_else(|| generator["forfait"]["designSupplyTemperatureC"].as_f64());
        let mut system = derived.distribution_system.take().unwrap_or_else(|| {
            recorder.record(
                "pipe_insulation_unknown_uninsulated",
                "heating.distribution",
                "uninsulated".into(),
                "ISSO 75.1 p. 120",
            );
            recorder.record(
                "heat_meter_unknown_present",
                "heating.distribution.pump",
                "true".into(),
                "ISSO 75.1 p. 123",
            );
            recorder.record(
                "pump_unknown_forfait",
                "heating.distribution.pump",
                "forfait (9.41–9.51)".into(),
                "ISSO 75.1 p. 119",
            );
            json!({
                "designTemperatureClass": mean.map(design_class_from_mean).unwrap_or("90_70"),
                "pipeTransmittance": {"method": "forfait", "insulation": {"state": "uninsulated"}},
                "valvesInsulated": false,
                "pump": {"method": "calculated", "heatMeterPresent": true, "sourceReference": "basisopname forfait"},
                "sourceReference": format!("{reference}; basisopname"),
            })
        });
        system["installation"] = json!(if installation.collective {
            "collective"
        } else {
            "individual"
        });
        // Table 9.16a (heat meter unknown: present) is for collective
        // installations only.
        system["pump"]["heatMeterPresent"] = json!(installation.collective);
        system["usageFunction"] = json!(reduction_function);
        system["connectedStoreys"] = json!(survey.storeys.max(1));
        derived.distribution_system = Some(system);
    }
    derived
}

fn cooling_value(
    cooling: &SurveyCooling,
    construction_year: i32,
    storeys: u32,
    recorder: &mut Recorder,
) -> Value {
    let reference = cooling.source_reference.as_str();
    let generator = match cooling.generator {
        CoolingGeneratorAnswer::Compression => json!({"kind": "compression"}),
        CoolingGeneratorAnswer::RoomAirConditioner => json!({"kind": "room_air_conditioner"}),
        CoolingGeneratorAnswer::GasAbsorption => json!({"kind": "gas_absorption"}),
        CoolingGeneratorAnswer::ExternalCold => json!({"kind": "external_cold"}),
        CoolingGeneratorAnswer::UnknownCollective => json!({"kind": "unknown_collective"}),
        CoolingGeneratorAnswer::AquiferBefore2013 => {
            json!({"kind": "free_cooling", "source": "aquifer_utility_before2013"})
        }
        CoolingGeneratorAnswer::AquiferFrom2013 => {
            json!({"kind": "free_cooling", "source": "aquifer_from2013"})
        }
        CoolingGeneratorAnswer::AquiferYearUnknown => {
            let source = match cooling.aquifer_permit_year {
                Some(year) if year >= 2013 => "aquifer_from2013",
                _ => "aquifer_utility_before2013",
            };
            recorder.record(
                "aquifer_year_unknown",
                "cooling.generator",
                format!("{source} (permit year {:?})", cooling.aquifer_permit_year),
                "ISSO 75.1 p. 131",
            );
            json!({"kind": "free_cooling", "source": source})
        }
        CoolingGeneratorAnswer::SurfaceWater => {
            json!({"kind": "free_cooling", "source": "surface_water"})
        }
        CoolingGeneratorAnswer::ClosedGroundLoop => {
            json!({"kind": "free_cooling", "source": "closed_ground_loop"})
        }
        CoolingGeneratorAnswer::DewPointCooling => {
            json!({"kind": "free_cooling", "source": "dew_point_cooling"})
        }
    };
    if cooling.capacity_kw.is_none() {
        recorder.record(
            "cooling_power_unknown_forfait",
            "cooling.capacityKw",
            "forfait".into(),
            "ISSO 75.1 p. 132 (table 10.3)",
        );
    }
    let (emitter, radiant) = match cooling.emitter {
        CoolingEmitterAnswer::FloorCooling => ("floor_cooling", true),
        CoolingEmitterAnswer::ConcreteCoreActivation => {
            recorder.record(
                "concrete_core_activation_floor_cooling",
                "cooling.emitter",
                "floor_cooling".into(),
                "ISSO 75.1 p. 138",
            );
            ("floor_cooling", true)
        }
        CoolingEmitterAnswer::WallCooling => ("wall_cooling", true),
        CoolingEmitterAnswer::CeilingCooling => ("ceiling_cooling", true),
        CoolingEmitterAnswer::FanCoilOnOuterWall => ("fan_coil_or_rac_on_outer_wall", false),
        CoolingEmitterAnswer::FanCoilOnCeiling => ("fan_coil_or_rac_on_ceiling", false),
        CoolingEmitterAnswer::SplitIndoorUnitsOnWall => {
            recorder.record(
                "split_indoor_units_fan_coils",
                "cooling.emitter",
                "fan_coil_or_rac_on_outer_wall".into(),
                "ISSO 75.1 p. 138",
            );
            ("fan_coil_or_rac_on_outer_wall", false)
        }
        CoolingEmitterAnswer::SplitIndoorUnitsOnCeiling => {
            recorder.record(
                "split_indoor_units_fan_coils",
                "cooling.emitter",
                "fan_coil_or_rac_on_ceiling".into(),
                "ISSO 75.1 p. 138",
            );
            ("fan_coil_or_rac_on_ceiling", false)
        }
        CoolingEmitterAnswer::Other => ("other_or_unknown", false),
    };
    let balancing = if !cooling.water_based {
        "not_applicable"
    } else {
        match cooling.balanced {
            Some(true) => "static",
            Some(false) => "none_or_unknown",
            None => {
                recorder.record(
                    "cooling_balancing_unknown_none",
                    "cooling.balanced",
                    "none_or_unknown".into(),
                    "ISSO 75.1 p. 133–134 (table 10.5)",
                );
                "none_or_unknown"
            }
        }
    };
    let control = match cooling.control {
        Some(CoolingControlAnswer::Standalone) => "standalone_per_room",
        Some(CoolingControlAnswer::CentralWithRoomControl) => "central_with_room_control",
        Some(CoolingControlAnswer::OtherOrUnknown) => "unknown_or_other",
        None => {
            recorder.record(
                "cooling_control_unknown_other",
                "cooling.control",
                "unknown_or_other".into(),
                "ISSO 75.1 p. 138 (table 10.12)",
            );
            "unknown_or_other"
        }
    };
    let mut system = json!({
        "emission": {
            "emitter": emitter, "balancing": balancing, "control": control,
            "fanCoilCount": cooling.fan_coil_count,
            "sourceReference": format!("{reference}; basisopname"),
        },
        "generators": [{
            "id": "koeling",
            "generator": generator,
            "capacityKw": cooling.capacity_kw,
            "equipmentReference": reference,
        }],
    });
    if cooling.water_based {
        let design = match cooling.design_temperature {
            Some(CoolingDesignAnswer::T6To12) => "t6_to12_or_unknown",
            Some(CoolingDesignAnswer::T12To16) => "t12_to16",
            Some(CoolingDesignAnswer::T12To18) => "t12_to18",
            Some(CoolingDesignAnswer::T17To21) => "t17_to21",
            None => {
                let design = if radiant {
                    "t17_to21"
                } else {
                    "t6_to12_or_unknown"
                };
                recorder.record(
                    "cooling_design_temperature_unknown",
                    "cooling.designTemperature",
                    design.into(),
                    "ISSO 75.1 p. 132 (table 10.4)",
                );
                design
            }
        };
        let pipe = match cooling.pipes_insulated {
            Some(true) => {
                let year = cooling.pipe_insulation_year.unwrap_or_else(|| {
                    recorder.record(
                        "cooling_pipe_insulation_year_construction_year",
                        "cooling.pipeInsulationYear",
                        construction_year.to_string(),
                        "ISSO 75.1 p. 135 (table 10.7)",
                    );
                    construction_year
                });
                if year >= 1995 {
                    "insulated_from1995"
                } else if year >= 1980 {
                    "insulated1980_to1995"
                } else {
                    "insulated_before1980_or_unknown_age"
                }
            }
            Some(false) => "uninsulated",
            None => {
                recorder.record(
                    "cooling_pipes_insulation_unknown_no",
                    "cooling.pipesInsulated",
                    "uninsulated".into(),
                    "ISSO 75.1 p. 135 (table 10.7)",
                );
                "uninsulated"
            }
        };
        recorder.record(
            "cooling_fittings_unknown_uninsulated_meters_present",
            "cooling.distribution",
            "fittings uninsulated, length forfait, cold meters present".into(),
            "ISSO 75.1 p. 136–137 (tables 10.8–10.11)",
        );
        system["distribution"] = json!({
            "designTemperature": design,
            "pipe": {"kind": pipe},
            "fittingsInsulated": false,
            "pump": {
                "hydraulicallyBalanced": balancing == "static",
                "floorCount": storeys.max(1),
                "heatMeter": true,
                "individualDwellingInstallation": false,
                "sourceReference": "basisopname forfait",
            },
            "sourceReference": format!("{reference}; basisopname"),
        });
    }
    system
}

fn pressure_variant(
    survey: &UtilityVentilation,
    construction_year: i32,
    prefix: char,
    recorder: &mut Recorder,
) -> String {
    let year = survey.installation_year.unwrap_or(construction_year);
    let class = match (survey.self_regulating_vents, survey.pressure_class) {
        (Some(false), _) => None,
        (Some(true), Some(class)) => Some(class),
        (Some(true), None) => {
            let class = if year <= 2003 {
                PressureClass::From5To10Pa
            } else {
                PressureClass::AtMost1Pa
            };
            recorder.record(
                "vent_pressure_class_unknown",
                "ventilation.pressureClass",
                format!("{class:?} (installation year {year})"),
                "ISSO 75.1 p. 142–144 (tables 11.3/11.5)",
            );
            Some(class)
        }
        (None, _) => {
            let class = (year >= 2013).then_some(PressureClass::AtMost1Pa);
            recorder.record(
                "self_regulating_vents_unknown",
                "ventilation.selfRegulatingVents",
                format!("{class:?} (installation year {year})"),
                "ISSO 75.1 p. 142–144 (tables 11.3/11.5)",
            );
            class
        }
    };
    let suffix = match class {
        None => "1",
        Some(PressureClass::AtMost1Pa) => "2a",
        Some(PressureClass::From1To5Pa) => "2b",
        Some(PressureClass::From5To10Pa) => "2c",
    };
    format!("{prefix}{suffix}")
}

#[allow(clippy::too_many_arguments)]
fn ventilation_value(
    survey: &UtilitySurvey,
    ventilation_function: &str,
    area: f64,
    heating_c: f64,
    floor_above_crawlspace: bool,
    airtightness: AirtightnessType,
    infiltration_year: Option<i32>,
    recorder: &mut Recorder,
) -> Value {
    let vent = &survey.ventilation;
    let year = survey.construction_year;
    let reference = vent.source_reference.as_str();
    let variant = match &vent.declared_variant {
        Some(variant) => variant.clone(),
        None => {
            recorder.record(
                "ventilation_controls_unknown_none",
                "ventilation",
                "no time control, CO₂ measurement or control, no zoning".into(),
                "ISSO 75.1 p. 143–145 (tables 11.4–11.6)",
            );
            match vent.principle {
                VentilationPrinciple::Natural => pressure_variant(vent, year, 'a', recorder),
                VentilationPrinciple::MechanicalExtract => {
                    pressure_variant(vent, year, 'c', recorder)
                }
                VentilationPrinciple::MechanicalSupply => "b1".into(),
                VentilationPrinciple::Balanced => "d1".into(),
            }
        }
    };
    let mechanical = vent.principle != VentilationPrinciple::Natural;
    let ducts = if !mechanical {
        "no_ducts"
    } else {
        match vent.ducts_luka_abc {
            Some(true) => "luka_a_b_c",
            Some(false) | None => {
                recorder.record(
                    "duct_airtightness_unknown",
                    "ventilation.ductsLukaAbc",
                    "unknown (f_lea;du 1,1)".into(),
                    "ISSO 75.1 p. 152–153 (table 11.13)",
                );
                "unknown"
            }
        }
    };
    let mut unit = json!({"variant": variant, "ducts": ducts, "equipmentReference": reference});
    if vent.principle == VentilationPrinciple::Balanced {
        let exchanger = match vent.heat_recovery {
            None | Some(ExchangerAnswer::Unknown) => {
                recorder.record(
                    "heat_recovery_unknown_none",
                    "ventilation.heatRecovery",
                    "none".into(),
                    "ISSO 75.1 p. 150 (table 11.9)",
                );
                None
            }
            Some(ExchangerAnswer::CounterFlowUnknownMaterial) => {
                recorder.record(
                    "counterflow_material_unknown_aluminium",
                    "ventilation.heatRecovery",
                    "counter_flow_aluminium".into(),
                    "ISSO 75.1 p. 151",
                );
                Some("counter_flow_aluminium")
            }
            Some(ExchangerAnswer::CounterFlowAluminium) => Some("counter_flow_aluminium"),
            Some(ExchangerAnswer::CounterFlowPlastic) => Some("counter_flow_plastic"),
            Some(ExchangerAnswer::CrossFlow) => Some("cross_flow"),
            Some(ExchangerAnswer::PlateOrTube) => Some("plate_or_tube"),
            Some(ExchangerAnswer::Rotary) => Some("rotary"),
            Some(ExchangerAnswer::Enthalpy) => Some("enthalpy"),
            Some(ExchangerAnswer::HeatPipe) => Some("heat_pipe"),
            Some(ExchangerAnswer::TwoElement) => Some("two_element"),
        };
        if let Some(exchanger) = exchanger {
            if vent.declared_variant.is_none() {
                unit["variant"] = json!("d2");
            }
            recorder.record(
                "supply_duct_insulation_unknown",
                "ventilation.heatRecovery",
                "uninsulated; length by kernel default (4 m utility)".into(),
                "ISSO 75.1 p. 150 (table 11.10)",
            );
            recorder.record(
                "constant_volume_unknown_none",
                "ventilation.heatRecovery",
                "false".into(),
                "ISSO 75.1 p. 151 (table 11.11)",
            );
            recorder.record(
                "bypass_table_11_12",
                "ventilation.bypass",
                "unknown (partial bypass of unknown share: 0 %)".into(),
                "ISSO 75.1 p. 151 (table 11.12)",
            );
            let mut recovery = json!({
                "efficiency": {"method": "table", "exchanger": exchanger},
                "bypass": {"kind": "unknown", "bypassPresent": vent.bypass_present.unwrap_or(false)},
                "layout": "central",
                "supplyDuctInsulation": {"kind": "uninsulated"},
                "equipmentReference": reference,
            });
            if let Some(year) = vent.unit_manufacture_year {
                recovery["manufactureYear"] = json!(year);
            }
            unit["heatRecovery"] = recovery;
        }
    }
    if let Some(ahu) = &vent.ahu {
        if !matches!(
            vent.principle,
            VentilationPrinciple::MechanicalSupply | VentilationPrinciple::Balanced
        ) {
            recorder.issue("ahu_requires_mechanical_supply", "ventilation.ahu");
        }
        let inside = match ahu.inside_thermal_zone {
            Some(true) if area > 500.0 => {
                recorder.record(
                    "large_installation_outside_thermal_zone",
                    "ventilation.ahu.insideThermalZone",
                    format!("outside (A_g {area} m² > 500 m²)"),
                    "ISSO 75.1 p. 17",
                );
                false
            }
            Some(inside) => inside,
            None => {
                recorder.record(
                    "ahu_location_unknown_outside",
                    "ventilation.ahu.insideThermalZone",
                    "outside the thermal zone".into(),
                    "ISSO 75.1 p. 17 (interpretation: conservative for small installations)",
                );
                false
            }
        };
        let situation = match ahu.ducts_outside_thermal_zone {
            Some(false) => "none",
            _ => {
                let length = ahu.duct_length.unwrap_or_else(|| {
                    recorder.record(
                        "ahu_duct_length_unknown_40_m",
                        "ventilation.ahu.ductLength",
                        "≥ 40 m".into(),
                        "ISSO 75.1 p. 153 (table 11.14)",
                    );
                    DuctLengthAnswer::AtLeast40M
                });
                let insulated = ahu.ducts_insulated.unwrap_or_else(|| {
                    recorder.record(
                        "ahu_duct_insulation_unknown_no",
                        "ventilation.ahu.ductsInsulated",
                        "R < 1,0".into(),
                        "ISSO 75.1 p. 153 (table 11.14)",
                    );
                    false
                });
                // NTA table 11.19 situations.
                match (length, insulated) {
                    (DuctLengthAnswer::AtMost20M, true) => "situation1",
                    (DuctLengthAnswer::From20To40M, true)
                    | (DuctLengthAnswer::AtMost20M, false) => "situation2",
                    _ => "situation3",
                }
            }
        };
        unit["airHandlingUnit"] = json!({
            "insideThermalZone": inside,
            "supplyDuctsOutside": situation,
        });
    }
    // Tables 11.7/11.8 (p. 148–149).
    let mut flow_reduction = json!({});
    let recirculation = match (vent.recirculation_percent, vent.recirculation) {
        (Some(percent), _) => Some(percent / 10 * 10),
        (None, Some(RecirculationAnswer::PresentPercentUnknown)) => {
            recorder.record(
                "recirculation_percent_unknown_below_20",
                "ventilation.recirculation",
                "< 20 % (x = 10)".into(),
                "ISSO 75.1 p. 148 (table 11.7)",
            );
            Some(10)
        }
        (None, Some(RecirculationAnswer::Unknown)) => {
            recorder.record(
                "recirculation_unknown_none",
                "ventilation.recirculation",
                "none".into(),
                "ISSO 75.1 p. 148 (table 11.7)",
            );
            None
        }
        _ => None,
    };
    if let Some(percent) = recirculation.filter(|percent| *percent > 0) {
        flow_reduction["recirculationPercent"] = json!(percent);
    }
    // Forfait fans (11.139–11.142) do not use the control method; only the
    // flow reduction of 11.61 follows from it.
    match &vent.flow_control {
        Some(control) => {
            let percent = control.minimum_percent.map_or_else(
                || {
                    recorder.record(
                        "flow_control_percent_unknown_80",
                        "ventilation.flowControl.minimumPercent",
                        "80 %".into(),
                        "ISSO 75.1 p. 149 (table 11.8)",
                    );
                    80
                },
                |percent| percent.div_ceil(10) * 10,
            );
            flow_reduction["flowControlPercent"] = json!(percent.min(100));
        }
        None => recorder.record(
            "flow_control_unknown_none",
            "ventilation.flowControl",
            "none".into(),
            "ISSO 75.1 p. 149 (table 11.8)",
        ),
    }
    let fan_year = super::general::device_year(
        vent.unit_manufacture_year,
        vent.installation_year,
        year,
        recorder,
        "ventilation.fans",
    );
    let current = match vent.motor.unwrap_or(MotorAnswer::Unknown) {
        MotorAnswer::Ac => "ac",
        MotorAnswer::Dc => "dc",
        MotorAnswer::Unknown => {
            let installed = vent.installation_year.unwrap_or(year);
            let current = if installed <= 2006 { "ac" } else { "dc" };
            recorder.record(
                "fan_motor_unknown",
                "ventilation.motor",
                format!("{current} (installed {installed})"),
                "ISSO 75.1 p. 154 (table 11.15)",
            );
            current
        }
    };
    let infiltration = match &survey.measured_infiltration {
        Some(item) => json!({
            "method": "measured", "qv10DmPerSM2": item.qv10_dm3_per_s_m2,
            "sourceReference": item.source_reference,
        }),
        None => {
            let mut value = json!({"method": "reference", "buildingType": airtightness});
            if let Some(year) = infiltration_year {
                value["renovationYear"] = json!(year);
            }
            value
        }
    };
    json!({
        "zoneId": "utiliteit",
        "usableFloorAreaM2": area,
        "category": "utility",
        "functions": [{"function": ventilation_function, "areaM2": area}],
        "dwellingCount": 0,
        "buildingHeightM": survey.building_height_m,
        "constructionYear": year,
        "floorAboveCrawlspace": floor_above_crawlspace,
        "heatingSetpointC": heating_c,
        "coolingSetpointC": 24.0,
        "system": {"kind": "single", "unit": unit},
        "flowReduction": flow_reduction,
        "infiltration": infiltration,
        "fans": {"method": "forfait", "current": current, "manufactureYear": fan_year},
        "sourceReference": format!("{reference}; basisopname"),
    })
}

fn hot_water_value(
    survey: &UtilitySurvey,
    main: LabelFunction,
    area: f64,
    recorder: &mut Recorder,
) -> Value {
    let hot = &survey.hot_water;
    let reference = hot.source_reference.as_str();
    // Generators shared with the residential layer go through its mapping.
    let shared = match &hot.generator {
        UtilityHotWaterGenerator::None => Some(HotWaterGeneratorAnswer::None),
        UtilityHotWaterGenerator::GasAppliance {
            appliance_type,
            gaskeur,
            burner_load_kw,
        } => Some(HotWaterGeneratorAnswer::GasAppliance {
            appliance_type: *appliance_type,
            gaskeur: *gaskeur,
            burner_load_kw: *burner_load_kw,
        }),
        UtilityHotWaterGenerator::ElectricBoiler => Some(HotWaterGeneratorAnswer::ElectricBoiler),
        UtilityHotWaterGenerator::ElectricInstantaneous => {
            Some(HotWaterGeneratorAnswer::ElectricInstantaneous)
        }
        UtilityHotWaterGenerator::HeatPump { exhaust_air_source } => {
            Some(HotWaterGeneratorAnswer::HeatPump {
                exhaust_air_source: *exhaust_air_source,
            })
        }
        UtilityHotWaterGenerator::DistrictHeat => Some(HotWaterGeneratorAnswer::DistrictHeat),
        _ => None,
    };
    let shower_functions = matches!(
        main,
        LabelFunction::Cell
            | LabelFunction::HealthcareWithBeds
            | LabelFunction::HealthcareWithoutBeds
            | LabelFunction::Lodging
            | LabelFunction::Sport
    );
    let mut system = match shared {
        Some(generator) => derive_hot_water(
            &SurveyHotWater {
                generator,
                served: TapsServed::KitchenAndBathroom,
                kitchen_length_m: None,
                bathroom_length_m: None,
                showers: hot.showers,
                shower_heat_recovery: if shower_functions {
                    hot.shower_heat_recovery
                } else {
                    ShowerRecoveryAnswer::None
                },
                boiler_vessel: None,
                source_reference: reference.to_string(),
            },
            recorder,
        ),
        None => {
            let generator = match &hot.generator {
                UtilityHotWaterGenerator::GasStorageHeater {
                    volume_l,
                    before_1985,
                    in_heated_zone,
                } => {
                    let before = before_1985.unwrap_or_else(|| {
                        recorder.record(
                            "gas_storage_year_construction_year",
                            "hotWater.generator.before1985",
                            survey.construction_year.to_string(),
                            "ISSO 75.1 p. 167",
                        );
                        survey.construction_year < 1985
                    });
                    let inside = in_heated_zone.unwrap_or_else(|| {
                        recorder.record(
                            "gas_storage_location_unknown_outside",
                            "hotWater.generator.inHeatedZone",
                            "outside the thermal zone".into(),
                            "ISSO 75.1 p. 167",
                        );
                        false
                    });
                    json!({"kind": "gas_storage_heater", "volumeL": volume_l,
                           "before1985": before, "inHeatedZone": inside})
                }
                _ => {
                    recorder.record(
                        "collective_hot_water_generator_unknown",
                        "hotWater.generator",
                        "other directly heated storage".into(),
                        "ISSO 75.1 p. 165 (table 13.2)",
                    );
                    json!({"kind": "large_direct_storage", "gasFired": true})
                }
            };
            json!({"generator": generator, "equipmentReference": reference})
        }
    };
    if !shower_functions && hot.shower_heat_recovery != ShowerRecoveryAnswer::None {
        recorder.warning(
            "shower_heat_recovery_not_for_function",
            "hotWater.showerHeatRecovery",
            "ISSO 75.1 p. 178: only for cell, healthcare, lodging and sport functions",
        );
    }
    let electric_boiler = matches!(hot.generator, UtilityHotWaterGenerator::ElectricBoiler);
    let storage: Vec<Value> = hot
        .storage
        .iter()
        .map(|vessel| storage_value(vessel, electric_boiler, recorder))
        .collect();
    if !storage.is_empty() {
        system["storage"] = json!(storage);
    }
    system["need"] = json!({
        "method": "utility",
        "areas": [{"function": label_name(main), "areaM2": area}],
        "sourceReference": "NTA table 13.1",
    });
    let length = hot.mean_draw_off_length_m.unwrap_or_else(|| {
        recorder.record(
            "draw_off_length_unknown_over_3_m",
            "hotWater.meanDrawOffLengthM",
            "> 3 m".into(),
            "ISSO 75.1 p. 177",
        );
        4.0
    });
    system["emission"] = json!({
        "method": "utility", "meanLengthM": length,
        "sourceReference": format!("{reference}; basisopname"),
    });
    match hot.circulation {
        Some(true) => {
            recorder.record(
                "circulation_forfait",
                "hotWater.circulation",
                "length, insulation and diameter forfait; fittings uninsulated".into(),
                "ISSO 75.1 p. 174–175",
            );
            system["circulation"] = json!({
                "insulation": "unknown",
                "fittingsInsulated": false,
                "floorCount": survey.storeys.max(1),
                "pump": {"control": "uncontrolled_or_unknown"},
                "sourceReference": format!("{reference}; basisopname"),
            });
        }
        Some(false) => {}
        None => recorder.warning(
            "circulation_unknown_not_entered",
            "hotWater.circulation",
            "presence of a circulation loop was not established; none entered",
        ),
    }
    system
}

/// Table 13.10 (p. 173) and NTA 13.6.3 `f_sto;dis;ls`.
fn storage_value(vessel: &SurveyStorage, electric_boiler: bool, recorder: &mut Recorder) -> Value {
    let path = format!("hotWater.storage[{}]", vessel.id);
    let loss = match vessel.label {
        Some(label) => json!({"method": "label", "label": label}),
        None => {
            let from_2018 = vessel.produced_from_2018.unwrap_or_else(|| {
                recorder.record(
                    "vessel_year_unknown_until_2017",
                    &path,
                    "manufactured up to 2017".into(),
                    "ISSO 75.1 p. 173 (table 13.10)",
                );
                false
            });
            json!({"method": "unknown_label", "producedFrom2018": from_2018})
        }
    };
    let factor = if electric_boiler {
        // NTA 13.6.3: electric boilers use 2 with or without insulation.
        2
    } else {
        match vessel.connection {
            Some(VesselConnection::FullyInsulated) => 2,
            Some(VesselConnection::StraightOnlyFour) => 3,
            Some(VesselConnection::StraightOnlyMoreThanFour) => 4,
            Some(VesselConnection::Uninsulated) => 5,
            None => {
                recorder.record(
                    "vessel_connection_unknown_uninsulated",
                    &path,
                    "f_sto;dis;ls 5".into(),
                    "ISSO 75.1 p. 173",
                );
                5
            }
        }
    };
    let inside = vessel.in_heated_zone.unwrap_or_else(|| {
        recorder.record(
            "vessel_location_unknown_outside",
            &path,
            "outside the thermal zone".into(),
            "ISSO 75.1 p. 173 (table 13.10)",
        );
        false
    });
    json!({
        "id": vessel.id,
        "volumeL": vessel.volume_l,
        "loss": loss,
        "connectionFactor": factor,
        "inHeatedZone": inside,
        "sourceReference": vessel.source_reference,
    })
}

fn lighting_value(
    survey: &UtilitySurvey,
    main: LabelFunction,
    area: f64,
    recorder: &mut Recorder,
) -> Value {
    let mut zones = Vec::new();
    // NTA 14.3.4: the power forfait applies to all lighting zones of the
    // calculation zone; LED from 2017 only when every forfait zone has it.
    let forfait_led: Vec<bool> = survey
        .lighting
        .iter()
        .filter_map(|zone| match zone.power {
            LightingPowerAnswer::Unknown { led_from_2017 } => Some(led_from_2017),
            _ => None,
        })
        .collect();
    let whole_forfait = (!forfait_led.is_empty()).then(|| forfait_led.iter().all(|led| *led));
    if whole_forfait.is_some() && forfait_led.len() < survey.lighting.len() {
        recorder.record(
            "lighting_forfait_whole_zone",
            "lighting",
            "forfait power for all lighting zones".into(),
            "NTA 8800 14.3.4; ISSO 75.1 p. 188 (table 14.5)",
        );
    }
    for zone in &survey.lighting {
        let path = format!("lighting[{}]", zone.id);
        let forfait = whole_forfait.is_some();
        let unknown_power = LightingPowerAnswer::Unknown {
            led_from_2017: whole_forfait.unwrap_or(false),
        };
        let zone_power = if forfait { &unknown_power } else { &zone.power };
        let power = match zone_power {
            LightingPowerAnswer::Luminaires { luminaires } => json!({
                "method": "installed",
                "luminaires": luminaires.iter().map(|item| json!({
                    "count": item.count, "power": {"method": "system", "powerW": item.power_w},
                })).collect::<Vec<_>>(),
                "sourceReference": zone.source_reference,
            }),
            LightingPowerAnswer::Lamps { lamps } => json!({
                "method": "installed",
                "luminaires": lamps.iter().map(|item| {
                    let technology = match item.lamp_type {
                        LampTypeAnswer::T12 => "fluorescent_t12",
                        LampTypeAnswer::T8Conventional => "fluorescent_t8_conventional",
                        LampTypeAnswer::T8HighFrequency => "fluorescent_t8_electronic",
                        LampTypeAnswer::T5 => "fluorescent_t5",
                        LampTypeAnswer::LedInLuminaire => "led",
                        LampTypeAnswer::CflPlugIn => "compact_fluorescent_not_integrated",
                        LampTypeAnswer::CflIntegrated
                        | LampTypeAnswer::IncandescentOrHalogen
                        | LampTypeAnswer::LedLamp => "no_separate_ballast",
                        LampTypeAnswer::Unknown => "unknown_or_other",
                    };
                    json!({"count": 1, "power": {
                        "method": "lamps", "lampPowerW": item.lamp_power_w,
                        "lampCount": item.count, "technology": technology,
                    }})
                }).collect::<Vec<_>>(),
                "sourceReference": zone.source_reference,
            }),
            LightingPowerAnswer::Unknown { led_from_2017 } => {
                recorder.record(
                    "lighting_power_unknown_forfait",
                    &path,
                    format!("table 14.5 (LED from 2017: {led_from_2017})"),
                    "ISSO 75.1 p. 188 (table 14.5)",
                );
                json!({"method": "forfait", "ledFrom2017": led_from_2017})
            }
        };
        if let LightingPowerAnswer::Lamps { lamps } = zone_power {
            if lamps
                .iter()
                .any(|item| item.lamp_type == LampTypeAnswer::Unknown)
            {
                recorder.record(
                    "lamp_type_unknown_20_percent",
                    &path,
                    "+20 %".into(),
                    "ISSO 75.1 p. 188 (table 14.4)",
                );
            }
        }
        let (control, central) = if forfait {
            recorder.record(
                "forfait_lighting_central_on",
                &path,
                "central on-control, no daylight control".into(),
                "ISSO 75.1 p. 183–184 (tables 14.2/14.3)",
            );
            ("manual_or_unknown", true)
        } else {
            match zone.presence {
                PresenceControlAnswer::NoneOrCentralOn => ("manual_or_unknown", true),
                PresenceControlAnswer::RoomSwitch => ("manual_or_unknown", false),
                PresenceControlAnswer::RoomSwitchWithSweep => ("manual_with_sweep", false),
                PresenceControlAnswer::SensorsTypeUnknown => {
                    recorder.record(
                        "sensors_type_unknown_auto_on_dimmed",
                        &path,
                        "auto_on_dimmed".into(),
                        "ISSO 75.1 p. 183",
                    );
                    ("auto_on_dimmed", false)
                }
                PresenceControlAnswer::AutoOnDimmed => ("auto_on_dimmed", false),
                PresenceControlAnswer::AutoOnAutoOff => ("auto_on_auto_off", false),
                PresenceControlAnswer::ManualOnDimmed => ("manual_on_dimmed", false),
                PresenceControlAnswer::ManualOnAutoOff => ("manual_on_auto_off", false),
            }
        };
        let daylight = if forfait {
            json!({"method": "none"})
        } else {
            match zone.daylight {
                DaylightAnswer::None => json!({"method": "none"}),
                DaylightAnswer::Unknown => {
                    recorder.record(
                        "daylight_control_unknown_switching",
                        &path,
                        "switching (forfait daylight method 14.43/14.44)".into(),
                        "ISSO 75.1 p. 184 (table 14.3)",
                    );
                    json!({"method": "forfait", "daylightControl": true})
                }
                DaylightAnswer::Switching | DaylightAnswer::Dimming => {
                    json!({"method": "forfait", "daylightControl": true})
                }
            }
        };
        zones.push(json!({
            "id": zone.id,
            "areaM2": zone.area_m2,
            "power": power,
            "parasitic": {"method": "forfait"},
            "occupancy": {
                "control": control, "centralOnControl": central,
                "largeOfficeGroup": zone.large_office_group,
            },
            "daylight": daylight,
            "extractedLuminaires": zone.extracted_luminaires && !forfait,
        }));
    }
    recorder.record(
        "lighting_parasitic_unknown_forfait",
        "lighting",
        "forfait (14.14)".into(),
        "ISSO 75.1 p. 189",
    );
    json!({
        "zoneId": "utiliteit",
        "functions": [{"function": label_name(main), "areaM2": area}],
        "lightingZones": zones,
        "sourceReference": format!("{}; basisopname", survey.source_reference),
    })
}

fn validate(survey: &UtilitySurvey, recorder: &mut Recorder) {
    if survey.id.trim().is_empty() {
        recorder.issue("survey_id_required", "id");
    }
    if !(1600..=2100).contains(&survey.construction_year) {
        recorder.issue("construction_year_invalid", "constructionYear");
    }
    if survey.functions.is_empty()
        || survey
            .functions
            .iter()
            .any(|item| !(item.area_m2.is_finite() && item.area_m2 > 0.0))
    {
        recorder.issue("function_area_invalid", "functions");
    }
    if survey
        .functions
        .iter()
        .any(|item| item.function == LabelFunction::Residential)
    {
        recorder.issue("residential_function_use_residential_survey", "functions");
    }
    if !(survey.building_height_m.is_finite() && survey.building_height_m > 0.0) {
        recorder.issue("building_height_invalid", "buildingHeightM");
    }
    for (field, value) in [
        ("sourceReference", &survey.source_reference),
        ("areaSourceReference", &survey.area_source_reference),
    ] {
        if value.trim().is_empty() {
            recorder.issue("source_reference_required", field);
        }
    }
    if let Some(renovation) = &survey.renovation {
        if renovation.evidence_reference.trim().is_empty() {
            recorder.issue(
                "renovation_evidence_required",
                "renovation.evidenceReference",
            );
        }
    }
    if survey.lighting.is_empty() {
        recorder.issue("lighting_zone_required", "lighting");
    }
    if matches!(
        survey.hot_water.generator,
        UtilityHotWaterGenerator::ElectricBoiler
    ) && survey.hot_water.storage.is_empty()
    {
        recorder.issue("hot_water_storage_required", "hotWater.storage");
    }
    let total: f64 = survey.functions.iter().map(|item| item.area_m2).sum();
    let lit: f64 = survey.lighting.iter().map(|item| item.area_m2).sum();
    if !survey.lighting.is_empty() && (lit - total).abs() > 0.01 * total {
        recorder.issue("lighting_area_mismatch", "lighting");
    }
    for window in &survey.solar_control_window_ids {
        if !survey
            .envelope
            .windows
            .iter()
            .any(|item| &item.id == window)
        {
            recorder.issue("solar_control_window_unknown", "solarControlWindowIds");
        }
    }
}

/// Kernel input derived from the utility survey.
pub fn derive_utility_input(survey: &UtilitySurvey, recorder: &mut Recorder) -> Option<Value> {
    validate(survey, recorder);
    if !recorder.issues.is_empty() {
        return None;
    }
    let (main, area) = main_function(&survey.functions, recorder)?;
    let (usage, ventilation_function, reduction_function) = kernel_names(main);
    let heating_c = usage.heating_setpoint_c();
    let year = survey.construction_year;
    let airtightness = utility_airtightness(&survey.building_type, recorder);
    let infiltration = infiltration_year(year, survey.renovation.as_ref(), recorder);
    let (floor, wall, ceiling) = thermal_mass(&survey.construction);
    let mut envelope = derive_envelope(&survey.envelope, year, recorder);
    for window in envelope.windows.iter_mut() {
        let id = window["id"].as_str().unwrap_or_default().to_string();
        if survey.solar_control_window_ids.contains(&id) {
            window["gPerpendicular"] = json!(0.4);
            recorder.record(
                "solar_control_glass_g_0_4",
                &format!("envelope.windows[{id}]"),
                "0,4".into(),
                "ISSO 75.1 p. 96 (table 8.14)",
            );
        }
    }
    let ventilation = ventilation_value(
        survey,
        ventilation_function,
        area,
        heating_c,
        envelope.floor_above_crawlspace,
        airtightness,
        infiltration,
        recorder,
    );
    let heating = derive_utility_heating(survey, reduction_function, area, recorder);
    let hot_water = hot_water_value(survey, main, area, recorder);
    let lighting = lighting_value(survey, main, area, recorder);
    let cooling = survey
        .cooling
        .as_ref()
        .map(|cooling| cooling_value(cooling, year, survey.storeys, recorder));
    let pv: Vec<Value> = survey
        .pv
        .iter()
        .map(|item| derive_pv(item, year, recorder))
        .collect();
    let (bacs, bacs_reference) = bacs_factor(&survey.bacs, recorder);
    if !recorder.issues.is_empty() {
        return None;
    }
    let demand = json!({
        "zoneId": "utiliteit",
        "usableFloorAreaM2": area,
        "areaSourceReference": survey.area_source_reference,
        "usageFunction": usage_name(usage),
        "setpoints": {"heatingC": heating_c, "coolingC": 24.0, "sourceReference": "NTA 8800 table 7.13"},
        "transmission": {
            "method": "components",
            "direct": {"elements": envelope.direct_elements, "linearBridges": [], "pointBridges": []},
            "unheated": envelope.unheated,
            "groundFloors": envelope.ground_floors,
            "groundInventoryConfirmed": true,
        },
        "ventilationFlows": [],
        "ventilation": ventilation,
        "thermalMass": {
            "floor": floor, "wall": wall, "ceiling": ceiling,
            "sourceReference": format!("{}; ISSO 75.1 table 7.5", survey.construction.source_reference),
        },
        "internalGains": {
            "method": "utility",
            "lighting": {"method": "chapter14"},
            "sourceReference": "NTA tables 7.2/7.3; lighting from chapter 14",
        },
        "windowInventoryComplete": true,
        "windows": envelope.windows,
        "opaqueInventoryComplete": true,
        "opaqueElements": envelope.opaque_elements,
    });
    let distribution = if heating.distribution_system.is_some() {
        json!({"method": "calculated", "sourceReference": format!("{}; basisopname forfait", survey.heating.source_reference)})
    } else {
        json!({"method": "heated_zone_only_space_heating", "sourceReference": format!("{}; basisopname: pipes in the heated zone", survey.heating.source_reference)})
    };
    let mut chain = json!({
        "demand": demand,
        "emission": heating.emission,
        "distribution": distribution,
        "generator": heating.generator,
    });
    if let Some(system) = heating.distribution_system {
        chain["distributionSystem"] = system;
    }
    let mut input = json!({
        "calculationScope": "utility",
        "totalUsableFloorAreaM2": area,
        "areaSourceReference": survey.area_source_reference,
        "spaceHeating": chain,
        "bacsFactor": bacs,
        "bacsSourceReference": bacs_reference,
        "useInventoryComplete": true,
        "declaredUses": [],
        "productionInventoryComplete": true,
        "onSiteProduction": [],
        "pvSystems": pv,
        "labelFunction": label_name(main),
        "lossAreaM2": loss_area(&survey.envelope),
        "lossAreaSourceReference": "basisopname: survey surfaces with f_ls (NTA 6.7.3)",
        "lighting": [lighting],
        "hotWater": hot_water,
        "demandUsesFixedC1Ventilation": false,
        "batteryStoragePresent": false,
    });
    if let Some(cooling) = cooling {
        input["cooling"] = cooling;
    }
    if let Some(renewable) = heating.heat_pump_renewable {
        input["heatPumpRenewable"] = renewable;
    }
    Some(input)
}

/// Monthly steam-humidifier energy (12.1–12.3) as a declared use, from the
/// mechanical supply flow of the heating balance and the cooling need.
fn humidification_use(
    survey: &UtilitySurvey,
    main: LabelFunction,
    performance: &BuildingPerformanceAssessment,
    recorder: &mut Recorder,
) -> Option<Value> {
    let humidification = survey.humidification.as_ref()?;
    let demand = &performance.space_heating.demand;
    let Some(ventilation) = demand.ventilation.as_ref() else {
        recorder.issue("humidification_requires_chapter_11", "humidification");
        return None;
    };
    let function = match main {
        LabelFunction::HealthcareWithBeds | LabelFunction::HealthcareWithoutBeds => {
            HumidityFunction::Healthcare
        }
        LabelFunction::Sport => HumidityFunction::Sport,
        _ => HumidityFunction::General,
    };
    let humidifier = match humidification.humidifier {
        HumidifierAnswer::ElectricSteam => Humidifier::Steam {
            carrier: SteamCarrier::Electricity,
        },
        HumidifierAnswer::NonElectricSteam => Humidifier::Steam {
            carrier: SteamCarrier::GasOrOil,
        },
        HumidifierAnswer::Adiabatic => Humidifier::Atomising,
    };
    let input = HumidityInput {
        zone_id: "utiliteit".into(),
        usable_floor_area_m2: survey.functions.iter().map(|item| item.area_m2).sum(),
        functions: vec![HumidityFunctionArea {
            function,
            area_m2: survey.functions.iter().map(|item| item.area_m2).sum(),
        }],
        humidification: Some(Humidification {
            humidifier,
            rotary_wheel: humidification.absorption_wheel,
            equipment_reference: humidification.source_reference.clone(),
        }),
        supply_flow_m3_per_h: ventilation
            .months
            .iter()
            .map(|month| month.heating.mechanical_supply_m3_per_h)
            .collect(),
        cooling_design: None::<DehumidificationDesign>,
        cooling_need_kwh: Vec::new(),
    };
    let months = match calculate_humidity(&input) {
        Ok(months) => months,
        Err(issues) => {
            for issue in issues {
                recorder.issue(issue.code, format!("humidification.{}", issue.path));
            }
            return None;
        }
    };
    match humidification.humidifier {
        HumidifierAnswer::Adiabatic => {
            recorder.warning(
                "adiabatic_humidification_load_not_in_heating_chain",
                "humidification",
                "12.2: the latent load of atomising humidifiers is supplied by the heating system; the kernel does not add it to the heating chain yet",
            );
            None
        }
        HumidifierAnswer::ElectricSteam | HumidifierAnswer::NonElectricSteam => {
            let electric = humidification.humidifier == HumidifierAnswer::ElectricSteam;
            Some(json!({
                "id": "bevochtiging",
                "service": "humidification",
                "carrier": if electric { "el" } else { "gas" },
                "monthlyKwh": months.iter().map(|month| if electric {
                    month.steam_electricity_kwh
                } else {
                    month.steam_fuel_kwh
                }).collect::<Vec<_>>(),
                "sourceReference": format!("NTA 12.1–12.3 (kernel humidification); {}", humidification.source_reference),
            }))
        }
    }
}

/// Survey → kernel input → building performance (utility).
pub fn assess_utility_survey(survey: &UtilitySurvey) -> OpnameAssessment {
    let mut recorder = Recorder::default();
    let derived = derive_utility_input(survey, &mut recorder);
    let mut derived_input =
        derived.and_then(
            |value| match serde_json::from_value::<BuildingPerformanceInput>(value) {
                Ok(input) => Some(input),
                Err(error) => {
                    recorder.issues.push(OpnameIssue {
                        code: "derived_input_shape_invalid",
                        path: error.to_string(),
                    });
                    None
                }
            },
        );
    let mut performance = derived_input.as_ref().map(assess_building_performance);
    // 12.1–12.3: steam humidification energy needs the chapter 11 flows of
    // the first run; it is then added as a declared use.
    if let (Some(input), Some(result)) = (derived_input.as_mut(), performance.as_ref()) {
        if result.status == "calculated_unverified" {
            let main = main_function(&survey.functions, &mut Recorder::default())
                .map(|(main, _)| main)
                .unwrap_or(LabelFunction::Office);
            if let Some(value) = humidification_use(survey, main, result, &mut recorder) {
                match serde_json::from_value(value) {
                    Ok(item) => {
                        input.declared_uses.push(item);
                        performance = Some(assess_building_performance(input));
                    }
                    Err(error) => recorder.issues.push(OpnameIssue {
                        code: "derived_input_shape_invalid",
                        path: error.to_string(),
                    }),
                }
            }
        }
    }
    remap_sources(&mut recorder.applied);
    let status = match &performance {
        Some(_) if !recorder.issues.is_empty() => "invalid",
        Some(result) if result.status == "calculated_unverified" => "calculated_unverified",
        Some(_) => "derived_input_rejected",
        None => "invalid",
    };
    OpnameAssessment {
        status,
        scope: "isso_75_1_basisopname_utility_unverified",
        source: ISSO_UTILITY_SOURCE,
        applied_defaults: recorder.applied,
        warnings: recorder.warnings,
        issues: recorder.issues,
        derived_input,
        performance,
        reference_verified: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> UtilitySurvey {
        let raw = match name {
            "1985" => {
                include_str!("../../../../training-data/nta8800-opname-utility-1985-office.json")
            }
            "2005" => {
                include_str!("../../../../training-data/nta8800-opname-utility-2005-school.json")
            }
            _ => include_str!("../../../../training-data/nta8800-opname-utility-1970-retail.json"),
        };
        serde_json::from_str(raw).unwrap()
    }

    fn rules(result: &OpnameAssessment) -> Vec<&'static str> {
        result
            .applied_defaults
            .iter()
            .map(|item| item.rule)
            .collect()
    }

    #[test]
    fn fixtures_calculate_with_documented_defaults() {
        for name in ["1985", "2005", "1970"] {
            let result = assess_utility_survey(&fixture(name));
            let performance = result.performance.as_ref();
            assert_eq!(
                result.status,
                "calculated_unverified",
                "{name}: {:?} {:?}",
                result.issues,
                performance.map(|item| &item.issues)
            );
            let performance = performance.unwrap();
            let ep = performance
                .primary_fossil_indicator_kwh_per_m2_year
                .unwrap();
            eprintln!(
                "{name}: EP2 {ep:.1} kWh/m², label {:?}, rules {:?}",
                performance.indicative_label_class,
                rules(&result)
            );
            assert!(ep > 0.0);
        }
    }

    fn derive(survey: &UtilitySurvey) -> (Value, Recorder) {
        let mut recorder = Recorder::default();
        let input = derive_utility_input(survey, &mut recorder);
        assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
        (input.unwrap(), recorder)
    }

    fn applied(recorder: &Recorder, rule: &str) -> bool {
        recorder.applied.iter().any(|item| item.rule == rule)
    }

    #[test]
    fn small_functions_merge_up_to_25_percent() {
        let (input, recorder) = derive(&fixture("1985"));
        assert_eq!(input["labelFunction"], "office");
        assert_eq!(input["totalUsableFloorAreaM2"], 1300.0);
        assert!(applied(&recorder, "small_functions_merged_into_main"));
        let mut survey = fixture("1985");
        survey.functions[1].area_m2 = 500.0;
        survey.lighting[1].area_m2 = 600.0;
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&survey, &mut recorder).is_none());
        assert_eq!(recorder.issues[0].code, "mixed_functions_require_zones");
    }

    #[test]
    fn residential_function_is_rejected() {
        let mut survey = fixture("2005");
        survey.functions[0].function = LabelFunction::Residential;
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&survey, &mut recorder).is_none());
        assert_eq!(
            recorder.issues[0].code,
            "residential_function_use_residential_survey"
        );
    }

    #[test]
    fn building_type_gives_table_11_14_row() {
        let mut recorder = Recorder::default();
        let row = utility_airtightness(
            &UtilityBuildingType::SingleLayer {
                position: SingleLayerPosition::Terraced,
                roof: UtilityRoof::PartlyFlat,
            },
            &mut recorder,
        );
        assert_eq!(row, AirtightnessType::PitchedRoofTerraced);
        assert!(applied(&recorder, "partly_flat_roof_only_detached"));
        let row = utility_airtightness(
            &UtilityBuildingType::MultiLayerPart {
                level: StoreyLevel::Top,
                position: StoreyPosition::WholeStorey,
            },
            &mut recorder,
        );
        assert_eq!(row, AirtightnessType::MultiStoreyWholeTopLayer);
    }

    #[test]
    fn bacs_follows_table_7_3() {
        let mut recorder = Recorder::default();
        let small = SurveyBacs {
            system_power_kw: Some(290.0),
            ..SurveyBacs::default()
        };
        assert_eq!(bacs_factor(&small, &mut recorder).0, 1.0);
        let unknown = SurveyBacs {
            served_area_m2: Some(2600.0),
            ..SurveyBacs::default()
        };
        assert_eq!(bacs_factor(&unknown, &mut recorder).0, 1.05);
        assert!(applied(&recorder, "bacs_power_unknown_served_area"));
        assert!(applied(&recorder, "bacs_presence_unknown_no"));
        let present = SurveyBacs {
            system_power_kw: Some(400.0),
            present: Some(true),
            ..SurveyBacs::default()
        };
        assert_eq!(bacs_factor(&present, &mut recorder).0, 1.05);
        assert!(applied(&recorder, "bacs_automation_class_unknown_d"));
        assert!(applied(&recorder, "bacs_management_class_unknown_c_d"));
        let compliant = SurveyBacs {
            system_power_kw: Some(400.0),
            present: Some(true),
            automation_class_c_or_better: Some(true),
            management_class_b_or_better: Some(true),
            evidence_reference: Some("BACS inspection".into()),
            ..SurveyBacs::default()
        };
        assert_eq!(bacs_factor(&compliant, &mut recorder).0, 1.0);
        assert!(recorder.issues.is_empty());
    }

    #[test]
    fn collective_boiler_gets_role_auxiliary_and_distribution() {
        let (input, recorder) = derive(&fixture("1985"));
        let chain = &input["spaceHeating"];
        let boiler = &chain["generator"];
        assert_eq!(boiler["boiler"]["role"], "collective");
        assert_eq!(boiler["auxiliary"]["nominalPowerKw"], 150.0);
        // p. 17/117: A_g 1 300 m² > 500 m², technical room outside.
        assert_eq!(boiler["boiler"]["location"], "outside_thermal_boundary");
        assert!(applied(
            &recorder,
            "large_installation_outside_thermal_zone"
        ));
        let system = &chain["distributionSystem"];
        assert_eq!(system["installation"], "collective");
        assert_eq!(system["usageFunction"], "office");
        assert_eq!(system["connectedStoreys"], 2);
        assert_eq!(chain["distribution"]["method"], "calculated");
        let mut survey = fixture("1985");
        survey.heating_installation.capacity_kw = None;
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&survey, &mut recorder).is_none());
        assert_eq!(
            recorder.issues[0].code,
            "collective_generator_power_required"
        );
    }

    #[test]
    fn heat_pump_uses_utility_scope() {
        let mut survey = fixture("2005");
        survey.heating.generator = super::super::heating::HeatingGenerator::HeatPump {
            source: super::super::heating::HeatPumpSource::Ground,
            air_sink: false,
            high_temperature: false,
            capacity_kw: None,
            source_regeneration_factor: None,
            high_efficiency_evidence: None,
        };
        let (input, _) = derive(&survey);
        let forfait = &input["spaceHeating"]["generator"]["forfait"];
        assert_eq!(forfait["scope"], "utility_collective_or_over25_kw");
        assert_eq!(forfait["collectiveBuildingInstallation"], true);
        assert_eq!(forfait["thermalCapacityKw"], 200.0);
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?}",
            result.performance.map(|p| p.issues)
        );
    }

    fn cooling(emitter: CoolingEmitterAnswer) -> SurveyCooling {
        SurveyCooling {
            generator: CoolingGeneratorAnswer::Compression,
            capacity_kw: None,
            emitter,
            fan_coil_count: 0,
            water_based: true,
            design_temperature: None,
            balanced: None,
            control: None,
            pipes_insulated: None,
            pipe_insulation_year: None,
            aquifer_permit_year: None,
            source_reference: "survey".into(),
        }
    }

    #[test]
    fn cooling_unknowns_follow_chapter_10() {
        let mut recorder = Recorder::default();
        let fan_coils = cooling_value(
            &cooling(CoolingEmitterAnswer::FanCoilOnCeiling),
            1990,
            3,
            &mut recorder,
        );
        assert_eq!(
            fan_coils["distribution"]["designTemperature"],
            "t6_to12_or_unknown"
        );
        assert_eq!(fan_coils["distribution"]["pipe"]["kind"], "uninsulated");
        assert_eq!(fan_coils["emission"]["balancing"], "none_or_unknown");
        assert_eq!(fan_coils["emission"]["control"], "unknown_or_other");
        assert_eq!(fan_coils["distribution"]["pump"]["heatMeter"], true);
        for rule in [
            "cooling_power_unknown_forfait",
            "cooling_design_temperature_unknown",
            "cooling_pipes_insulation_unknown_no",
            "cooling_balancing_unknown_none",
            "cooling_control_unknown_other",
            "cooling_fittings_unknown_uninsulated_meters_present",
        ] {
            assert!(applied(&recorder, rule), "{rule}");
        }
        let core = cooling_value(
            &cooling(CoolingEmitterAnswer::ConcreteCoreActivation),
            1990,
            3,
            &mut recorder,
        );
        assert_eq!(core["emission"]["emitter"], "floor_cooling");
        assert_eq!(core["distribution"]["designTemperature"], "t17_to21");
        let split = cooling_value(
            &cooling(CoolingEmitterAnswer::SplitIndoorUnitsOnWall),
            1990,
            3,
            &mut recorder,
        );
        assert_eq!(
            split["emission"]["emitter"],
            "fan_coil_or_rac_on_outer_wall"
        );
        let mut insulated = cooling(CoolingEmitterAnswer::CeilingCooling);
        insulated.pipes_insulated = Some(true);
        let value = cooling_value(&insulated, 1988, 1, &mut recorder);
        assert_eq!(
            value["distribution"]["pipe"]["kind"],
            "insulated1980_to1995"
        );
        assert!(applied(
            &recorder,
            "cooling_pipe_insulation_year_construction_year"
        ));
        let mut aquifer = cooling(CoolingEmitterAnswer::CeilingCooling);
        aquifer.generator = CoolingGeneratorAnswer::AquiferYearUnknown;
        let value = cooling_value(&aquifer, 2010, 1, &mut recorder);
        assert_eq!(
            value["generators"][0]["generator"]["source"],
            "aquifer_utility_before2013"
        );
        aquifer.aquifer_permit_year = Some(2014);
        let value = cooling_value(&aquifer, 2010, 1, &mut recorder);
        assert_eq!(
            value["generators"][0]["generator"]["source"],
            "aquifer_from2013"
        );
        let mut direct = cooling(CoolingEmitterAnswer::FanCoilOnOuterWall);
        direct.water_based = false;
        let value = cooling_value(&direct, 2010, 1, &mut recorder);
        assert!(value.get("distribution").is_none());
        assert_eq!(value["emission"]["balancing"], "not_applicable");
    }

    #[test]
    fn recirculation_and_flow_control_follow_tables_11_7_and_11_8() {
        let (input, recorder) = derive(&fixture("1985"));
        let ventilation = &input["spaceHeating"]["demand"]["ventilation"];
        assert_eq!(ventilation["flowReduction"]["recirculationPercent"], 10);
        assert!(ventilation["flowReduction"]
            .get("flowControlPercent")
            .is_none());
        assert!(applied(&recorder, "flow_control_unknown_none"));
        let (input, recorder) = derive(&fixture("1970"));
        let ventilation = &input["spaceHeating"]["demand"]["ventilation"];
        assert!(ventilation["flowReduction"]
            .get("recirculationPercent")
            .is_none());
        assert!(applied(&recorder, "recirculation_unknown_none"));
        assert_eq!(ventilation["flowReduction"]["flowControlPercent"], 80);
        let mut survey = fixture("1985");
        survey.ventilation.recirculation_percent = Some(37);
        survey.ventilation.flow_control = Some(SurveyFlowControl {
            method: FlowControlMethodAnswer::SpeedControl,
            minimum_percent: Some(43),
        });
        let (input, _) = derive(&survey);
        let reduction = &input["spaceHeating"]["demand"]["ventilation"]["flowReduction"];
        // Recirculation rounds down, the minimum flow up, both to tens.
        assert_eq!(reduction["recirculationPercent"], 30);
        assert_eq!(reduction["flowControlPercent"], 50);
    }

    #[test]
    fn ahu_ducts_follow_table_11_14() {
        let (input, recorder) = derive(&fixture("1970"));
        let ahu =
            &input["spaceHeating"]["demand"]["ventilation"]["system"]["unit"]["airHandlingUnit"];
        assert_eq!(ahu["insideThermalZone"], false);
        assert_eq!(ahu["supplyDuctsOutside"], "situation3");
        assert!(applied(&recorder, "ahu_duct_length_unknown_40_m"));
        assert!(applied(&recorder, "ahu_duct_insulation_unknown_no"));
        assert!(applied(&recorder, "ahu_location_unknown_outside"));
        let mut survey = fixture("1970");
        survey.ventilation.ahu = Some(SurveyAhu {
            inside_thermal_zone: Some(true),
            ducts_outside_thermal_zone: Some(true),
            duct_length: Some(DuctLengthAnswer::AtMost20M),
            ducts_insulated: Some(true),
        });
        let (input, recorder) = derive(&survey);
        let ahu =
            &input["spaceHeating"]["demand"]["ventilation"]["system"]["unit"]["airHandlingUnit"];
        assert_eq!(ahu["supplyDuctsOutside"], "situation1");
        // A_g 3 000 m² > 500 m²: outside by definition.
        assert_eq!(ahu["insideThermalZone"], false);
        assert!(applied(
            &recorder,
            "large_installation_outside_thermal_zone"
        ));
    }

    #[test]
    fn hot_water_unknowns_follow_chapter_13() {
        let (input, recorder) = derive(&fixture("1970"));
        let hot = &input["hotWater"];
        assert_eq!(hot["generator"]["kind"], "large_direct_storage");
        assert_eq!(hot["need"]["method"], "utility");
        assert_eq!(hot["need"]["areas"][0]["function"], "retail");
        assert_eq!(hot["emission"]["meanLengthM"], 4.0);
        assert_eq!(hot["circulation"]["insulation"], "unknown");
        for rule in [
            "collective_hot_water_generator_unknown",
            "draw_off_length_unknown_over_3_m",
            "circulation_forfait",
        ] {
            assert!(applied(&recorder, rule), "{rule}");
        }
        let (input, recorder) = derive(&fixture("1985"));
        let vessel = &input["hotWater"]["storage"][0];
        assert_eq!(vessel["connectionFactor"], 2);
        assert_eq!(vessel["loss"]["method"], "unknown_label");
        assert_eq!(vessel["loss"]["producedFrom2018"], false);
        assert!(applied(&recorder, "vessel_year_unknown_until_2017"));
        let mut survey = fixture("1985");
        survey.hot_water.storage.clear();
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&survey, &mut recorder).is_none());
        assert_eq!(recorder.issues[0].code, "hot_water_storage_required");
    }

    #[test]
    fn indirect_vessel_and_gas_storage_defaults() {
        let mut recorder = Recorder::default();
        let vessel = storage_value(
            &SurveyStorage {
                id: "vat".into(),
                volume_l: 300.0,
                label: None,
                produced_from_2018: Some(true),
                connection: None,
                in_heated_zone: None,
                source_reference: "survey".into(),
            },
            false,
            &mut recorder,
        );
        assert_eq!(vessel["connectionFactor"], 5);
        assert_eq!(vessel["inHeatedZone"], false);
        assert!(applied(&recorder, "vessel_connection_unknown_uninsulated"));
        assert!(applied(&recorder, "vessel_location_unknown_outside"));
        let mut survey = fixture("1970");
        survey.hot_water.generator = UtilityHotWaterGenerator::GasStorageHeater {
            volume_l: 200.0,
            before_1985: None,
            in_heated_zone: None,
        };
        let (input, recorder) = derive(&survey);
        let generator = &input["hotWater"]["generator"];
        assert_eq!(generator["before1985"], true);
        assert_eq!(generator["inHeatedZone"], false);
        assert!(applied(&recorder, "gas_storage_year_construction_year"));
        assert!(applied(&recorder, "gas_storage_location_unknown_outside"));
        assert_eq!(
            assess_utility_survey(&survey).status,
            "calculated_unverified"
        );
    }

    #[test]
    fn shower_recovery_only_for_showering_functions() {
        let mut survey = fixture("2005");
        survey.hot_water.shower_heat_recovery = ShowerRecoveryAnswer::Vertical;
        let (input, recorder) = derive(&survey);
        assert!(input["hotWater"].get("showerHeatRecovery").is_none());
        assert_eq!(
            recorder.warnings[0].code,
            "shower_heat_recovery_not_for_function"
        );
    }

    #[test]
    fn lighting_unknowns_follow_chapter_14() {
        let (input, recorder) = derive(&fixture("2005"));
        let zone = &input["lighting"][0]["lightingZones"][0];
        assert_eq!(zone["occupancy"]["control"], "auto_on_dimmed");
        assert_eq!(zone["daylight"]["daylightControl"], true);
        assert!(applied(&recorder, "sensors_type_unknown_auto_on_dimmed"));
        assert!(applied(&recorder, "daylight_control_unknown_switching"));
        let (input, recorder) = derive(&fixture("1970"));
        let zone = &input["lighting"][0]["lightingZones"][0];
        assert_eq!(zone["power"]["method"], "forfait");
        assert_eq!(zone["occupancy"]["centralOnControl"], true);
        assert_eq!(zone["daylight"]["method"], "none");
        assert!(applied(&recorder, "forfait_lighting_central_on"));
        // One zone with unknown power puts the whole calculation zone on
        // the forfait (14.3.4).
        let mut survey = fixture("1985");
        survey.lighting[1].power = LightingPowerAnswer::Unknown {
            led_from_2017: false,
        };
        survey.lighting[0].power = LightingPowerAnswer::Lamps {
            lamps: vec![LampCount {
                count: 10,
                lamp_power_w: 36.0,
                lamp_type: LampTypeAnswer::Unknown,
            }],
        };
        let (input, recorder) = derive(&survey);
        let zones = &input["lighting"][0]["lightingZones"];
        assert_eq!(zones[0]["power"]["method"], "forfait");
        assert_eq!(zones[1]["power"]["method"], "forfait");
        assert!(applied(&recorder, "lighting_forfait_whole_zone"));
        assert!(!applied(&recorder, "lamp_type_unknown_20_percent"));
        let mut survey = fixture("1985");
        survey.lighting[0].power = LightingPowerAnswer::Lamps {
            lamps: vec![LampCount {
                count: 10,
                lamp_power_w: 36.0,
                lamp_type: LampTypeAnswer::Unknown,
            }],
        };
        let (input, recorder) = derive(&survey);
        let lamp = &input["lighting"][0]["lightingZones"][0]["power"]["luminaires"][0];
        assert_eq!(lamp["power"]["technology"], "unknown_or_other");
        assert!(applied(&recorder, "lamp_type_unknown_20_percent"));
    }

    #[test]
    fn solar_control_glass_uses_g_0_4() {
        let (input, recorder) = derive(&fixture("1970"));
        let windows = input["spaceHeating"]["demand"]["windows"]
            .as_array()
            .unwrap();
        let shop = windows.iter().find(|item| item["id"] == "etalage").unwrap();
        assert_eq!(shop["gPerpendicular"], 0.4);
        assert!(applied(&recorder, "solar_control_glass_g_0_4"));
        let mut survey = fixture("1970");
        survey.solar_control_window_ids = vec!["missing".into()];
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&survey, &mut recorder).is_none());
        assert_eq!(recorder.issues[0].code, "solar_control_window_unknown");
    }

    #[test]
    fn steam_humidification_becomes_a_declared_use() {
        let result = assess_utility_survey(&fixture("1970"));
        let input = result.derived_input.as_ref().unwrap();
        let steam = input
            .declared_uses
            .iter()
            .find(|item| item.id == "bevochtiging")
            .expect("steam use");
        assert_eq!(
            steam.service,
            crate::building_performance::Service::Humidification
        );
        assert!(steam.monthly_kwh[0] > 0.0);
        assert_eq!(steam.monthly_kwh[6], 0.0);
        let mut survey = fixture("1970");
        survey.humidification.as_mut().unwrap().humidifier = HumidifierAnswer::Adiabatic;
        let result = assess_utility_survey(&survey);
        assert!(result
            .warnings
            .iter()
            .any(|item| item.code == "adiabatic_humidification_load_not_in_heating_chain"));
        let without = assess_utility_survey(&fixture("1970"));
        let mut dry = fixture("1970");
        dry.humidification = None;
        let dry = assess_utility_survey(&dry);
        assert!(
            without
                .performance
                .unwrap()
                .primary_fossil_indicator_kwh_per_m2_year
                .unwrap()
                > dry
                    .performance
                    .unwrap()
                    .primary_fossil_indicator_kwh_per_m2_year
                    .unwrap()
        );
    }

    #[test]
    fn shared_rules_cite_isso_75_1() {
        let result = assess_utility_survey(&fixture("1970"));
        for item in &result.applied_defaults {
            assert!(
                !item.source.contains("82.1"),
                "{} {}",
                item.rule,
                item.source
            );
        }
        let rule = result
            .applied_defaults
            .iter()
            .find(|item| item.rule == "pilot_flame_unknown_present")
            .unwrap();
        assert_eq!(rule.source, "ISSO 75.1 p. 110");
    }
}
