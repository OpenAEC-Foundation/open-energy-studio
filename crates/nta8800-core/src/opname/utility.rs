//! Basic survey (basisopname) of existing utility buildings, ISSO 75.1
//! (7e druk, printdatum 11-12-2025).
//!
//! The utility layer reuses the residential envelope, heating, hot-water
//! generator and PV translations where ISSO 75.1 prescribes the same tables
//! and recognition rules as ISSO 82.1, and rewrites their citations to the
//! ISSO 75.1 pages. Utility-specific parts are added here: use functions
//! (small functions merged into the main function, p. 39–40), building type
//! for infiltration (p. 55–56), BACS (table 7.3, p. 62–63), collective
//! heating (§9.3, p. 108–123), cooling (chapter 10, p. 128–138),
//! utility ventilation with AHU, recirculation and flow control (chapter 11,
//! p. 140–158), humidification (chapter 12, p. 161), utility hot water
//! (chapter 13, p. 164–178) and lighting (chapter 14, p. 182–189).
//!
//! One calculation zone per building. Small functions are merged into the
//! main function up to 25 % of A_g (p. 39–40); larger ones stay separate in
//! a mixed calculation zone with area-weighted values (NTA §6.5.3). When
//! afb. 6.6 (p. 53) requires separate calculation zones the survey stops
//! (`calculation_zone_split_required`).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::envelope::{derive_envelope_with_cooling, SurveyEnvelope};
use super::general::{infiltration_year, thermal_mass, Construction, Renovation};
use super::heating::{derive_heating, AirHeatingAnswer, SurveyHeating};
use super::hot_water::{
    derive_hot_water, GasApplianceType, GaskeurAnswer, HotWaterGeneratorAnswer,
    ShowerRecoveryAnswer, SurveyHotWater, TapsServed,
};
use super::production::{derive_pv, SurveyPv};
use super::ventilation::{
    apply_passive_cooling, ExchangerAnswer, MotorAnswer, PressureClass, RecoveryLayout,
    SurveyCombined, SurveyGrilleHeatingStrips, SurveyPassiveCooling, VentilationPrinciple,
};
use super::{
    loss_area, AppliedDefault, MeasuredInfiltration, OpnameAssessment, OpnameIssue, Recorder,
};
use crate::building_performance::{assess_building_performance, BuildingPerformanceInput};
use crate::humidification::{Humidification, Humidifier, SteamCarrier};
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

/// §9.3 (p. 108–109): installation type; the generator power as table 9.7
/// (p. 115) prescribes it.
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
    /// Compression chiller driven by a gas engine (table 10.2, p. 130);
    /// needs `gasEngine`.
    GasEngineCompression,
}

/// Table 10.2 (p. 130): manufacture year and electric power of the gas
/// engine of a gas-driven chiller.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyGasEngine {
    /// Manufactured from 2007; `None` unknown (up to 2006).
    #[serde(default)]
    pub from_2007: Option<bool>,
    /// Electric power P_el, kW; no default ("niet van toepassing").
    #[serde(default)]
    pub electric_power_kw: Option<f64>,
    /// P_el ≤ 2 kW with an HRe declaration.
    #[serde(default)]
    pub hre_declared: bool,
}

/// §10.4.1 (p. 133): where a direct-expansion evaporator delivers cold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectExpansionAnswer {
    /// In the room or the air duct (split units, room air conditioners).
    Room,
    /// In the air handling unit: cold is delivered through the AHU
    /// cooling coil (ventilation).
    AirHandlingUnit,
}

/// A further cooling generator on the same distribution (§10.3.2, p. 131).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyCoolingGenerator {
    pub generator: CoolingGeneratorAnswer,
    /// Nominal power, kW (table 10.3); needed for the priority split.
    #[serde(default)]
    pub capacity_kw: Option<f64>,
    #[serde(default)]
    pub aquifer_permit_year: Option<i32>,
    #[serde(default)]
    pub gas_engine: Option<SurveyGasEngine>,
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
    /// Table 10.6 (ISSO table 10.6): none, static or dynamic balancing;
    /// `true`/`false` of older surveys mean static / none.
    #[serde(default)]
    pub balanced: Option<CoolingBalanceAnswer>,
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
    /// NTA 10.84: a heat pump of the building uses this ground storage as
    /// its source; unknown follows the survey's heating heat pump.
    #[serde(default)]
    pub heat_pump_source: Option<bool>,
    /// ISSO 82.1 p. 129 / 75.1: the ground source is demonstrably always
    /// above 0 °C (e.g. an EED calculation).
    #[serde(default)]
    pub ground_above_zero_demonstrated: bool,
    /// Gas engine of a gas-driven chiller (table 10.2).
    #[serde(default)]
    pub gas_engine: Option<SurveyGasEngine>,
    /// Direct expansion (not water-based): in the room or in the AHU
    /// (§10.4.1, p. 133); `None` in the room.
    #[serde(default)]
    pub direct_expansion: Option<DirectExpansionAnswer>,
    /// Valves, brackets and fittings fully insulated; `None` unknown (not
    /// insulated, table 10.9).
    #[serde(default)]
    pub fittings_insulated: Option<bool>,
    /// Cold meters in the distribution; `None` unknown (present, table
    /// 10.11).
    #[serde(default)]
    pub cold_meters: Option<bool>,
    /// Actual pipe length L, m (table 10.10); `None` forfait.
    #[serde(default)]
    pub pipe_length_m: Option<f64>,
    /// Actual length of the pipes through uncooled spaces, m (table
    /// 10.10); `None` forfait.
    #[serde(default)]
    pub uncooled_pipe_length_m: Option<f64>,
    /// Further generators on the same distribution (§10.3.2, p. 131); the
    /// kernel ranks them by table 10.15 and sums equal priorities.
    #[serde(default)]
    pub additional_generators: Vec<SurveyCoolingGenerator>,
    pub source_reference: String,
}

/// Table 10.6 balancing of a water-based cooling distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CoolingBalanceAnswer {
    /// Older surveys: `true` static, `false` none.
    Flag(bool),
    Kind(CoolingBalanceKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingBalanceKind {
    None,
    Static,
    Dynamic,
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
    /// p. 149: heating connected to the AHU (reheating coil, NTA 11.118–
    /// 11.121); `None` unknown.
    #[serde(default)]
    pub heating_connected: Option<bool>,
    /// p. 149: cooling connected to the AHU (cooling coil, NTA 11.114–
    /// 11.117); `None` unknown.
    #[serde(default)]
    pub cooling_connected: Option<bool>,
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
    /// ISSO 75.1 §11.5.6 (p. 152): proven passive cooling.
    #[serde(default)]
    pub passive_cooling: Option<SurveyPassiveCooling>,
    /// Table 11.13 (p. 152): duct airtightness class; overrides
    /// `ductsLukaAbc`. `None`: that flag, else unknown (1,1).
    #[serde(default)]
    pub duct_airtightness: Option<DuctAirtightnessAnswer>,
    /// System E (§11.3.6, p. 145): decentral balanced units with heat
    /// recovery and CO₂ control in part of the zone; `principle` describes
    /// the other part and `heatRecovery` the decentral units.
    #[serde(default)]
    pub combined: Option<SurveyCombined>,
    /// Supply grilles with electric heating strips (§11.3.7, p. 145–146).
    #[serde(default)]
    pub grille_heating_strips: Option<SurveyGrilleHeatingStrips>,
    /// Installed ventilation capacity of the calculation zone, dm³/s
    /// (§11.4.1, p. 146–148); `None`: unknown (regulatory flow).
    #[serde(default)]
    pub installed_capacity_dm3_per_s: Option<f64>,
    /// Central or decentral heat recovery (table 11.6, p. 145); `None`:
    /// central.
    #[serde(default)]
    pub heat_recovery_layout: Option<RecoveryLayout>,
    /// Supply duct between outside and the heat-recovery unit (table
    /// 11.10, p. 150); `None`: not insulated.
    #[serde(default)]
    pub supply_duct_insulation: Option<SupplyDuctInsulationAnswer>,
    /// Length of that duct inside the envelope, m; `None`: kernel default
    /// by central/decentral system (11.109).
    #[serde(default)]
    pub supply_duct_length_m: Option<f64>,
    /// Constant-volume control at all flows (table 11.11, p. 151); `None`:
    /// unknown (none).
    #[serde(default)]
    pub constant_volume_control: Option<bool>,
    /// Partial bypass percentage, rounded down to tens (table 11.12, p.
    /// 151); `None`: bypass or percentage unknown (kernel "unknown").
    #[serde(default)]
    pub bypass_percent: Option<u32>,
    pub source_reference: String,
}

/// Table 11.13 (p. 152) duct airtightness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DuctAirtightnessAnswer {
    LukaAbc,
    LukaD,
    NoDucts,
    Unknown,
}

/// Table 11.10 (p. 150) insulation of the outside connection of the
/// heat-recovery unit.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SupplyDuctInsulationAnswer {
    Uninsulated,
    /// Insulated (R ≥ 0,3 m²K/W over ≥ 90 % of the length), properties
    /// unknown.
    Insulated,
    Specified {
        #[serde(rename = "thicknessM")]
        thickness_m: f64,
        #[serde(rename = "conductivityWPerMK")]
        conductivity_w_per_mk: f64,
    },
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
    /// Solar water heaters (ISSO 75.1 §15.3–15.4).
    #[serde(default)]
    pub solar: Vec<super::hot_water::SurveySolarWaterHeater>,
    /// Nominal power of the main generator, kW (NTA 13.8.2, 13.141).
    #[serde(default)]
    pub nominal_power_kw: Option<f64>,
    /// Further generators of the system (NTA 13.8.2), with the utility
    /// generator types.
    #[serde(default)]
    pub additional_generators: Vec<UtilityAdditionalHotWater>,
    /// NTA 13.20/13.20a: the use functions this system serves, for an
    /// additional hot-water system (required there). The main system serves
    /// the rest of the building.
    #[serde(default)]
    pub served_areas: Vec<FunctionArea>,
    pub source_reference: String,
}

/// A further utility hot-water generator with its nominal power.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UtilityAdditionalHotWater {
    pub generator: UtilityHotWaterGenerator,
    #[serde(default)]
    pub nominal_power_kw: Option<f64>,
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
    /// Toilet groups, stacked groups counted once (table 7.8, p. 68);
    /// used when `vertical_pipes` is not determinable.
    #[serde(default)]
    pub toilet_stacks: Option<u32>,
    /// Vertical pipes through the envelope (§7.2.4); `None`: not
    /// determinable, empty: none present.
    #[serde(default)]
    pub vertical_pipes: Option<Vec<super::SurveyVerticalPipe>>,
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
    /// Further hot-water systems, each with its served areas (NTA 13.2.4,
    /// 13.20/13.20a).
    #[serde(default)]
    pub additional_hot_water_systems: Vec<UtilityHotWater>,
    pub lighting: Vec<SurveyLightingZone>,
    #[serde(default)]
    pub pv: Vec<SurveyPv>,
    #[serde(default)]
    pub bacs: SurveyBacs,
    /// Building-bound electrical or thermal storage (§15.5).
    #[serde(default)]
    pub storage: Option<super::production::SurveyStorage>,
    /// §7.1.7 (p. 61–62): building-bound installations on the plot burning
    /// fossil fuel; `None` not established (the calculation decides).
    #[serde(default)]
    pub fossil_fuel_on_plot: Option<bool>,
    /// p. 65: A_g of the sport and swimming halls when a sport function is
    /// present and the building has A_g ≥ 1 000 m² (NTA 13.32a).
    #[serde(default)]
    pub sport_hall_area_m2: Option<f64>,
    /// p. 65: A_g of the room with a swimming pool (walkway plus basin);
    /// part of the sport function (NTA §11.2.2.5.1, q × 2).
    #[serde(default)]
    pub swimming_pool_area_m2: Option<f64>,
    /// Afb. 6.6 (p. 53): the residence areas are openly connected, so the
    /// ventilation-capacity criterion does not split the zone.
    #[serde(default)]
    pub openly_connected_residence_areas: bool,
    pub source_reference: String,
    /// Reason per applied default (path or rule) for falling back on the
    /// forfait (BRL 9500-U §4.2.2).
    #[serde(default, rename = "inklapRedenen")]
    pub collapse_reasons: std::collections::BTreeMap<String, String>,
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
        "vertical_pipe_insulation_unknown_uninsulated" => "ISSO 75.1 p. 68 (table 7.8)",
        "door_split_window_and_door" => "ISSO 75.1 p. 74",
        "cavity_width_unknown_table_8_26" => "ISSO 75.1 p. 88",
        "opaque_rc_forfait_annex_i" => "ISSO 75.1 p. 88–93 (tables 8.9–8.11), NTA annex I",
        "crawlspace_bottom" | "crawlspace_wall_from_facade" => "ISSO 75.1 p. 95",
        "thermal_bridges_forfait_delta_u" => "ISSO 75.1 p. 83–84; NTA 8.2/8.3",
        "frame_fraction_forfait" => "NTA 7.6.6.2 method B; ISSO 75.1 p. 48",
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
        "pipe_insulation_unknown_uninsulated"
        | "pipe_fittings_unknown_uninsulated"
        | "pipe_insulation_year_unknown_construction_year" => "ISSO 75.1 p. 120 (table 9.12)",
        "renovated_one_pipe_as_two_pipe" => {
            "ISSO 75.1 p. 117 (§9.4.2); NTA 8800 p. 317 (one-pipe rule only)"
        }
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

/// p. 39–40: the main function with the small functions merged into it,
/// and the functions that stay separate.
pub struct FunctionGroups {
    pub main: LabelFunction,
    pub total_m2: f64,
    /// The main function first (with the merged functions), then the
    /// functions kept apart, for the §6.5.3 mixed calculation zone.
    pub groups: Vec<(LabelFunction, f64)>,
}

/// p. 39–40: the main function is the largest; starting with the smallest
/// other function, functions are merged into it while their sum stays
/// within 25 % of A_g. The rest stays separate (NTA §6.5.3).
pub fn function_groups(
    functions: &[FunctionArea],
    recorder: &mut Recorder,
) -> Option<FunctionGroups> {
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
    let (main, main_area) = groups.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1))?;
    let mut others: Vec<(LabelFunction, f64)> = groups
        .iter()
        .copied()
        .filter(|(function, _)| *function != main)
        .collect();
    others.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut merged = 0.0;
    let mut separate = Vec::new();
    for (function, area) in others {
        if merged + area <= MERGE_LIMIT * total + 1e-9 {
            merged += area;
        } else {
            separate.push((function, area));
        }
    }
    if merged > 0.0 {
        recorder.record(
            "small_functions_merged_into_main",
            "functions",
            format!(
                "{} m² of other functions (≤ 25 %) counted as {}",
                round1(merged),
                label_name(main)
            ),
            "ISSO 75.1 p. 39–40",
        );
    }
    if !separate.is_empty() {
        recorder.record(
            "larger_functions_kept_separate",
            "functions",
            format!(
                "{} function(s) beyond the 25 % merge kept in a mixed calculation zone",
                separate.len()
            ),
            "ISSO 75.1 p. 39–40; NTA 8800 §6.5.3",
        );
    }
    let mut result = vec![(main, main_area + merged)];
    result.extend(separate);
    Some(FunctionGroups {
        main,
        total_m2: total,
        groups: result,
    })
}

/// Table 6.4 (p. 54): heating setpoint (°C) and ventilation capacity
/// (dm³/(s·m²)) per use function; `None` for functions outside the table.
fn table_6_4(function: LabelFunction) -> Option<(f64, f64)> {
    Some(match function {
        LabelFunction::AssemblyWithDayCare => (21.0, 2.78),
        LabelFunction::AssemblyWithoutDayCare => (21.0, 1.71),
        LabelFunction::Cell => (21.0, 0.84),
        LabelFunction::HealthcareWithBeds => (22.0, 2.04),
        LabelFunction::HealthcareWithoutBeds => (21.0, 1.11),
        LabelFunction::Office => (21.0, 1.11),
        LabelFunction::Lodging => (21.0, 0.84),
        LabelFunction::Education => (21.0, 3.64),
        LabelFunction::Retail => (21.0, 0.28),
        LabelFunction::Sport => (16.0, 0.46),
        LabelFunction::Residential => return None,
    })
}

/// Afb. 6.6 with table 6.4 (p. 53–54): the criteria that force a split of
/// the climate zone into several calculation zones, on the use functions
/// left after the merge of p. 39–40. The basic survey derives one
/// calculation zone (the envelope is not surveyed per zone), so a required
/// split stops the survey (`calculation_zone_split_required`). The third
/// criterion (internal heat capacity differing by more than a factor 3)
/// cannot arise: the survey records one construction for the building.
fn check_zone_split(groups: &FunctionGroups, survey: &UtilitySurvey, recorder: &mut Recorder) {
    let rows: Vec<(f64, f64, f64)> = groups
        .groups
        .iter()
        .filter_map(|(function, area)| table_6_4(*function).map(|(t, q)| (t, q, *area)))
        .collect();
    if rows.len() < 2 || groups.total_m2 <= 0.0 {
        return;
    }
    let largest = rows.iter().map(|row| row.2).fold(0.0, f64::max);
    let (t_min, t_max) = rows
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), row| {
            (lo.min(row.0), hi.max(row.0))
        });
    // Setpoints more than 4 K apart, unless the largest function holds at
    // least 90 % of the zone.
    if t_max - t_min > 4.0 && largest < 0.9 * groups.total_m2 {
        recorder.issue("calculation_zone_split_required", "functions");
        return;
    }
    // Ventilation types A, B, C and E: capacities more than a factor 4
    // apart, unless the residence areas are openly connected or more than
    // 80 % of them has the same requirement.
    let vent = &survey.ventilation;
    let type_d = vent.principle == VentilationPrinciple::Balanced && vent.combined.is_none();
    if type_d || survey.openly_connected_residence_areas {
        return;
    }
    let (q_min, q_max) = rows
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), row| {
            (lo.min(row.1), hi.max(row.1))
        });
    let same_requirement = rows
        .iter()
        .map(|row| {
            rows.iter()
                .filter(|other| (other.1 - row.1).abs() < 1e-9)
                .map(|other| other.2)
                .sum::<f64>()
        })
        .fold(0.0, f64::max);
    if q_max > 4.0 * q_min && same_requirement <= 0.8 * groups.total_m2 {
        recorder.issue("calculation_zone_split_required", "functions");
    }
}

/// The main function and the total area (see [`function_groups`]).
pub fn main_function(
    functions: &[FunctionArea],
    recorder: &mut Recorder,
) -> Option<(LabelFunction, f64)> {
    function_groups(functions, recorder).map(|groups| (groups.main, groups.total_m2))
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
/// Without `systemPowerKw` the surveyed heating and cooling systems decide
/// (`system_powers_kw`: the summed nominal power per system, `None` when a
/// generator's power is unknown; p. 63: generators of one system are
/// summed, systems are not). Only when a power is unknown does the served
/// A_g decide; with the power and the served area both unknown, the system
/// is taken to serve the whole building A_g (p. 62).
pub fn bacs_factor(
    bacs: &SurveyBacs,
    system_powers_kw: &[Option<f64>],
    building_area_m2: f64,
    recorder: &mut Recorder,
) -> (f64, String) {
    let surveyed_large = system_powers_kw
        .iter()
        .flatten()
        .any(|power| *power > 290.0);
    let all_known =
        !system_powers_kw.is_empty() && system_powers_kw.iter().all(|power| power.is_some());
    let large = match bacs.system_power_kw {
        Some(power) => power > 290.0,
        None if surveyed_large || all_known => {
            let largest = system_powers_kw
                .iter()
                .flatten()
                .copied()
                .fold(0.0, f64::max);
            recorder.record(
                "bacs_power_from_surveyed_systems",
                "bacs.systemPowerKw",
                format!("{largest} kW (largest surveyed heating or cooling system)"),
                "ISSO 75.1 p. 62–63 (table 7.3)",
            );
            largest > 290.0
        }
        None => {
            let area = bacs.served_area_m2.unwrap_or_else(|| {
                recorder.record(
                    "bacs_served_area_unknown_building",
                    "bacs.servedAreaM2",
                    format!("{building_area_m2} m² (building A_g)"),
                    "ISSO 75.1 p. 62 (interpretation: whole building served)",
                );
                building_area_m2
            });
            let large = area > 2500.0;
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

/// Table 7.3 (p. 63): the nominal power of each surveyed heating and
/// cooling system, the generators of one system summed; `None` when a
/// generator's power is unknown.
fn surveyed_system_powers(survey: &UtilitySurvey) -> Vec<Option<f64>> {
    let sum = |powers: Vec<Option<f64>>| -> Option<f64> { powers.into_iter().sum() };
    let heating = survey.heating_installation.capacity_kw.or_else(|| {
        let mut powers = vec![survey.heating.nominal_power_kw];
        powers.extend(
            survey
                .heating
                .additional_generators
                .iter()
                .map(|extra| extra.nominal_power_kw),
        );
        sum(powers)
    });
    let mut systems = vec![heating];
    if let Some(cooling) = &survey.cooling {
        let mut powers = vec![cooling.capacity_kw];
        powers.extend(
            cooling
                .additional_generators
                .iter()
                .map(|extra| extra.capacity_kw),
        );
        systems.push(sum(powers));
    }
    systems
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

/// Collective role, heat-pump scope, 9.91 auxiliaries and the large
/// installation location of one utility generator.
fn adjust_utility_generator(
    generator: &mut Value,
    installation: &HeatingInstallation,
    capacity: Option<f64>,
    area: f64,
    reference: &str,
    recorder: &mut Recorder,
) {
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
        match capacity {
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
        recorder.applied.retain(|item| {
            item.rule != "source_regeneration_none_c_source_1"
                && item.rule != "groundwater_system_unknown_recirculation"
        });
        generator["forfait"]["collectiveBuildingInstallation"] = json!(installation.collective);
        if let Some(power) = capacity {
            generator["forfait"]["thermalCapacityKw"] = json!(power);
            generator["forfait"]["capacitySourceReference"] = json!(reference);
        }
        if installation.collective {
            let mut auxiliary =
                json!({"electricallyConnectedDevices": 1, "sourceReference": reference});
            if let Some(power) = capacity {
                auxiliary["nominalPowerKw"] = json!(power);
            }
            generator["auxiliary"] = auxiliary;
        }
    }
    if kind == "gas_heat_pump" {
        // Table 9.29 "GWP" rows for utility; c_source is a table 9.27 item.
        generator["table"] = json!("utility_collective_or_above25_kw");
        generator["sourceCorrectionFactor"] = Value::Null;
        recorder
            .applied
            .retain(|item| item.rule != "groundwater_system_unknown_recirculation");
        if generator["auxiliary"]["nominalPowerKw"].is_null() {
            if let Some(power) = capacity {
                // The installation capacity supplies P_H;gen (9.91).
                generator["auxiliary"]["nominalPowerKw"] = json!(power);
                recorder
                    .issues
                    .retain(|item| item.code != "gas_heat_pump_capacity_required");
            }
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
    // Utility adjustments per generator, also inside `multiple` (9.6.1).
    if derived.generator["kind"] == "multiple" {
        if let Some(parts) = derived.generator["generators"].as_array_mut() {
            for part in parts.iter_mut() {
                let capacity = part["nominalPowerKw"].as_f64();
                adjust_utility_generator(
                    &mut part["generator"],
                    installation,
                    capacity,
                    area,
                    reference,
                    recorder,
                );
            }
        }
    } else {
        adjust_utility_generator(
            &mut derived.generator,
            installation,
            installation.capacity_kw,
            area,
            reference,
            recorder,
        );
    }
    // Afb. 9.1 (p. 121–122): heating pipes in a crawlspace or other
    // unheated space; not inspectable or unknown counts as present with the
    // forfait length (table 9.14).
    let unheated_spaces_present = survey.envelope.surfaces.iter().any(|surface| {
        matches!(
            surface.boundary,
            super::envelope::SurfaceBoundary::Crawlspace
                | super::envelope::SurfaceBoundary::UnheatedCellar
                | super::envelope::SurfaceBoundary::UnheatedSpace { .. }
        )
    });
    let unheated_pipe_length = if hydronic && unheated_spaces_present {
        match survey.heating.unheated_pipes {
            Some(super::heating::UnheatedPipesAnswer::Absent) => None,
            Some(super::heating::UnheatedPipesAnswer::Present { length_m }) => Some(length_m),
            None => {
                recorder.record(
                    "unheated_pipes_unknown_present",
                    "heating.unheatedPipes",
                    "present, forfait length (9.26)".into(),
                    "ISSO 75.1 p. 121–122 (afb. 9.1, table 9.14)",
                );
                Some(None)
            }
        }
    } else {
        None
    };
    // A collective boiler's pump is not in 9.85: the calculated
    // distribution (9.26–9.51) with the forfait pump applies; so do pipes
    // in unheated spaces.
    let needs_distribution = hydronic
        && (derived.distribution_system.is_some()
            || installation.collective
            || unheated_pipe_length.is_some());
    if needs_distribution {
        // The derived distribution class (several generators: the highest).
        let generator = &derived.generator;
        let mean = generator["boiler"]["averageDesignEmissionTemperatureC"]
            .as_f64()
            .or_else(|| generator["forfait"]["designSupplyTemperatureC"].as_f64())
            .or_else(|| {
                (!derived.design_class.is_empty()).then_some(match derived.design_class {
                    "45_40" => 42.5,
                    "55_47" => 51.0,
                    "70_60" => 65.0,
                    _ => 80.0,
                })
            });
        let mut system = derived.distribution_system.take().unwrap_or_else(|| {
            // Table 9.12 (p. 120) and the one-pipe loop of §9.4.2 (p. 117),
            // as in the residential survey.
            let (transmittance, valves) = super::heating::pipe_insulation(
                &survey.heating,
                survey.construction_year,
                recorder,
            );
            let one_pipe = super::heating::one_pipe_emitters(&survey.heating, recorder);
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
                "pipeTransmittance": transmittance,
                "valvesInsulated": valves,
                "pump": {"method": "calculated", "heatMeterPresent": true, "onePipeEmitterCount": one_pipe, "sourceReference": "basisopname forfait"},
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
        match unheated_pipe_length {
            Some(Some(length)) => system["unheatedPipeLengthM"] = json!(length),
            // Forfait: the kernel's 15 % of L (9.26).
            Some(None) => {}
            // Afb. 9.1: no unheated space, or no pipes there; without this
            // the kernel would assume 15 % of L in unheated spaces.
            None if system.get("unheatedPipeLengthM").is_none() => {
                system["unheatedPipeLengthM"] = json!(0.0);
                recorder.record(
                    "unheated_pipes_absent_length_0",
                    "heating.unheatedPipes",
                    "0 m in unheated spaces".into(),
                    "ISSO 75.1 p. 121–122 (afb. 9.1)",
                );
            }
            None => {}
        }
        derived.distribution_system = Some(system);
    }
    derived
}

/// The ISSO publication whose chapter 10 rules apply (same rules, other
/// pages).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoolingBook {
    /// ISSO 75.1 (utility buildings).
    Utility,
    /// ISSO 82.1 (dwellings).
    Residential,
}

impl CoolingBook {
    /// Table 10.34: an aquifer realised before 2013.
    fn aquifer_before_2013(self) -> &'static str {
        match self {
            Self::Residential => "aquifer_dwellings_before2013",
            Self::Utility => "aquifer_utility_before2013",
        }
    }

    fn cite(self, rule: &str) -> &'static str {
        match (self, rule) {
            (Self::Utility, "aquifer") => "ISSO 75.1 p. 131",
            (Self::Utility, "power") => "ISSO 75.1 p. 132 (table 10.3)",
            (Self::Utility, "concrete") => "ISSO 75.1 p. 138",
            (Self::Utility, "split") => "ISSO 75.1 p. 138",
            (Self::Utility, "balancing") => "ISSO 75.1 p. 133 (table 10.6)",
            (Self::Utility, "control") => "ISSO 75.1 p. 138 (table 10.12)",
            (Self::Utility, "design") => "ISSO 75.1 p. 132 (table 10.4)",
            (Self::Utility, "pipe") => "ISSO 75.1 p. 135 (table 10.7)",
            (Self::Utility, "fittings") => "ISSO 75.1 p. 136 (table 10.9)",
            (Self::Utility, "length") => "ISSO 75.1 p. 137 (table 10.10)",
            (Self::Utility, "meters") => "ISSO 75.1 p. 137 (table 10.11)",
            (Self::Utility, "gas_engine") => "ISSO 75.1 p. 130 (table 10.2)",
            (Self::Utility, "expansion") => "ISSO 75.1 p. 130, 133 (§10.3.1.1, §10.4.1)",
            (Self::Utility, _) => "ISSO 75.1 p. 136–137 (tables 10.8–10.11)",
            (Self::Residential, "fittings") => "ISSO 82.1 p. 135 (table 10.9)",
            (Self::Residential, "length") => "ISSO 82.1 p. 135 (table 10.10)",
            (Self::Residential, "meters") => "ISSO 82.1 p. 136 (table 10.11)",
            (Self::Residential, "gas_engine") => "ISSO 82.1 p. 128 (table 10.2)",
            (Self::Residential, "expansion") => "ISSO 82.1 p. 128, 131 (§10.3.1.1, §10.4.1)",
            (Self::Residential, "aquifer") => "ISSO 82.1 p. 129",
            (Self::Residential, "power") => "ISSO 82.1 p. 130 (table 10.3)",
            (Self::Residential, "concrete") => "ISSO 82.1 p. 136",
            (Self::Residential, "split") => "ISSO 82.1 p. 136",
            (Self::Residential, "balancing") => "ISSO 82.1 p. 132 (table 10.6)",
            (Self::Residential, "control") => "ISSO 82.1 p. 137 (table 10.12)",
            (Self::Residential, "design") => "ISSO 82.1 p. 130 (table 10.4)",
            (Self::Residential, "pipe") => "ISSO 82.1 p. 133 (table 10.7)",
            (Self::Residential, _) => "ISSO 82.1 p. 134–136 (tables 10.9–10.11)",
        }
    }
}

/// NTA 10.84 for free cooling from ground storage: the heat-pump source
/// flag follows the answer, or the survey's heating heat pump on a ground
/// source when unknown (ISSO 82.1 p. 129, 75.1 p. 131).
pub(crate) fn apply_cooling_heat_pump_source(
    cooling_value: &mut Value,
    cooling: &SurveyCooling,
    heating_generator: &Value,
    book: CoolingBook,
    recorder: &mut Recorder,
) {
    let ground_heat_pump = ground_source_heat_pump(heating_generator);
    let Some(generators) = cooling_value["generators"].as_array_mut() else {
        return;
    };
    for generator in generators {
        let kind = &mut generator["generator"];
        let ground_storage = kind["kind"] == json!("free_cooling")
            && matches!(
                kind["source"].as_str(),
                Some(
                    "aquifer_from2013"
                        | "aquifer_utility_before2013"
                        | "aquifer_dwellings_before2013"
                        | "closed_ground_loop"
                )
            );
        if !ground_storage {
            continue;
        }
        let source = cooling.heat_pump_source.unwrap_or_else(|| {
            recorder.record(
                "cooling_heat_pump_source_from_heating",
                "cooling.heatPumpSource",
                ground_heat_pump.to_string(),
                book.cite("aquifer"),
            );
            ground_heat_pump
        });
        kind["heatPumpSource"] = json!(source);
        kind["groundAboveZeroDemonstrated"] = json!(cooling.ground_above_zero_demonstrated);
    }
}

/// A heat pump on a ground or groundwater source in a derived heating
/// generator (forfait, annex Q or inside `multiple`).
fn ground_source_heat_pump(generator: &Value) -> bool {
    if let Some(parts) = generator["generators"].as_array() {
        if generator["kind"] == json!("multiple") {
            return parts
                .iter()
                .any(|part| ground_source_heat_pump(&part["generator"]));
        }
    }
    let forfait = generator["forfait"]["source"].as_str();
    let annex_q = generator["heatPump"]["source"].as_str();
    matches!(forfait, Some("ground" | "groundwater_below15_c"))
        || matches!(annex_q, Some("brine_water" | "water_water"))
}

/// One surveyed cooling generator as the kernel generator kind.
fn cooling_generator_value(
    answer: CoolingGeneratorAnswer,
    aquifer_permit_year: Option<i32>,
    gas_engine: Option<&SurveyGasEngine>,
    path: &str,
    book: CoolingBook,
    recorder: &mut Recorder,
) -> Value {
    match answer {
        CoolingGeneratorAnswer::Compression => json!({"kind": "compression"}),
        CoolingGeneratorAnswer::RoomAirConditioner => json!({"kind": "room_air_conditioner"}),
        CoolingGeneratorAnswer::GasAbsorption => json!({"kind": "gas_absorption"}),
        CoolingGeneratorAnswer::ExternalCold => json!({"kind": "external_cold"}),
        CoolingGeneratorAnswer::UnknownCollective => json!({"kind": "unknown_collective"}),
        // Table 10.34: before 2013 dwellings EER 14, utility buildings 16.
        CoolingGeneratorAnswer::AquiferBefore2013 => {
            json!({"kind": "free_cooling", "source": book.aquifer_before_2013()})
        }
        CoolingGeneratorAnswer::AquiferFrom2013 => {
            json!({"kind": "free_cooling", "source": "aquifer_from2013"})
        }
        CoolingGeneratorAnswer::AquiferYearUnknown => {
            let source = match aquifer_permit_year {
                Some(year) if year >= 2013 => "aquifer_from2013",
                _ => book.aquifer_before_2013(),
            };
            recorder.record(
                "aquifer_year_unknown",
                &format!("{path}.generator"),
                format!("{source} (permit year {aquifer_permit_year:?})"),
                book.cite("aquifer"),
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
        // Table 10.2: year unknown → up to 2006; the electric power has no
        // default. NTA table 10.29 with ε_ge from table 9.31.
        CoolingGeneratorAnswer::GasEngineCompression => {
            let engine = gas_engine.cloned().unwrap_or_default();
            let after_2006 = engine.from_2007.unwrap_or_else(|| {
                recorder.record(
                    "gas_engine_year_unknown_up_to_2006",
                    &format!("{path}.gasEngine.from2007"),
                    "up to 2006".into(),
                    book.cite("gas_engine"),
                );
                false
            });
            let power = match engine.electric_power_kw {
                Some(power) if power.is_finite() && power > 0.0 && power <= 25_000.0 => power,
                Some(_) => {
                    recorder.issue(
                        "gas_engine_power_invalid",
                        format!("{path}.gasEngine.electricPowerKw"),
                    );
                    0.0
                }
                None => {
                    recorder.issue(
                        "gas_engine_power_required",
                        format!("{path}.gasEngine.electricPowerKw"),
                    );
                    0.0
                }
            };
            json!({
                "kind": "gas_engine_compression",
                "gasEngine": {
                    "powerKw": power,
                    "builtAfter2006": after_2006,
                    "hreDeclared": engine.hre_declared,
                },
            })
        }
    }
}

pub(crate) fn cooling_value(
    cooling: &SurveyCooling,
    construction_year: i32,
    storeys: u32,
    book: CoolingBook,
    individual_dwelling: bool,
    recorder: &mut Recorder,
) -> Value {
    let reference = cooling.source_reference.as_str();
    let generator = cooling_generator_value(
        cooling.generator,
        cooling.aquifer_permit_year,
        cooling.gas_engine.as_ref(),
        "cooling",
        book,
        recorder,
    );
    let mut generators = vec![json!({
        "id": "koeling",
        "generator": generator,
        "capacityKw": cooling.capacity_kw,
        "equipmentReference": reference,
    })];
    for (index, extra) in cooling.additional_generators.iter().enumerate() {
        let path = format!("cooling.additionalGenerators[{index}]");
        let generator = cooling_generator_value(
            extra.generator,
            extra.aquifer_permit_year,
            extra.gas_engine.as_ref(),
            &path,
            book,
            recorder,
        );
        generators.push(json!({
            "id": format!("koeling-{}", index + 2),
            "generator": generator,
            "capacityKw": extra.capacity_kw,
            "equipmentReference": reference,
        }));
    }
    // 10.49/10.50 (§10.3.2, p. 131): the priority split needs the nominal
    // power of every generator.
    if !cooling.additional_generators.is_empty() {
        if cooling.capacity_kw.is_none() {
            recorder.issue("cooling_generator_capacity_required", "cooling.capacityKw");
        }
        for (index, extra) in cooling.additional_generators.iter().enumerate() {
            if extra.capacity_kw.is_none() {
                recorder.issue(
                    "cooling_generator_capacity_required",
                    format!("cooling.additionalGenerators[{index}].capacityKw"),
                );
            }
        }
    }
    if cooling.capacity_kw.is_none() {
        recorder.record(
            "cooling_power_unknown_forfait",
            "cooling.capacityKw",
            "forfait".into(),
            book.cite("power"),
        );
    }
    if cooling.water_based && cooling.direct_expansion.is_some() {
        recorder.issue(
            "cooling_direct_expansion_not_water_based",
            "cooling.directExpansion",
        );
    }
    if !cooling.water_based
        && cooling.direct_expansion == Some(DirectExpansionAnswer::AirHandlingUnit)
    {
        recorder.record(
            "cooling_direct_expansion_in_ahu",
            "cooling.directExpansion",
            "no distribution; cold through the AHU cooling coil".into(),
            book.cite("expansion"),
        );
    }
    let (emitter, radiant) = match cooling.emitter {
        CoolingEmitterAnswer::FloorCooling => ("floor_cooling", true),
        CoolingEmitterAnswer::ConcreteCoreActivation => {
            recorder.record(
                "concrete_core_activation_floor_cooling",
                "cooling.emitter",
                "floor_cooling".into(),
                book.cite("concrete"),
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
                book.cite("split"),
            );
            ("fan_coil_or_rac_on_outer_wall", false)
        }
        CoolingEmitterAnswer::SplitIndoorUnitsOnCeiling => {
            recorder.record(
                "split_indoor_units_fan_coils",
                "cooling.emitter",
                "fan_coil_or_rac_on_ceiling".into(),
                book.cite("split"),
            );
            ("fan_coil_or_rac_on_ceiling", false)
        }
        CoolingEmitterAnswer::Other => ("other_or_unknown", false),
    };
    // ISSO 82.1 p. 136 / 75.1: fan convectors and split indoor units need
    // their number for the fan energy (10.17).
    if emitter.starts_with("fan_coil") && cooling.fan_coil_count == 0 {
        recorder.issue("cooling_fan_coil_count_required", "cooling.fanCoilCount");
    }
    let balancing = if !cooling.water_based {
        "not_applicable"
    } else {
        match cooling.balanced {
            Some(CoolingBalanceAnswer::Flag(true))
            | Some(CoolingBalanceAnswer::Kind(CoolingBalanceKind::Static)) => "static",
            Some(CoolingBalanceAnswer::Kind(CoolingBalanceKind::Dynamic)) => "dynamic",
            Some(CoolingBalanceAnswer::Flag(false))
            | Some(CoolingBalanceAnswer::Kind(CoolingBalanceKind::None)) => "none_or_unknown",
            None => {
                recorder.record(
                    "cooling_balancing_unknown_none",
                    "cooling.balanced",
                    "none_or_unknown".into(),
                    book.cite("balancing"),
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
                book.cite("control"),
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
        "generators": generators,
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
                    book.cite("design"),
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
                        book.cite("pipe"),
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
                    book.cite("pipe"),
                );
                "uninsulated"
            }
        };
        let fittings = cooling.fittings_insulated.unwrap_or_else(|| {
            recorder.record(
                "cooling_fittings_unknown_uninsulated",
                "cooling.fittingsInsulated",
                "not insulated".into(),
                book.cite("fittings"),
            );
            false
        });
        // Table 10.11 (NTA table 10.12): cold meters unknown → present.
        let meters = cooling.cold_meters.unwrap_or_else(|| {
            recorder.record(
                "cooling_meters_unknown_present",
                "cooling.coldMeters",
                "present".into(),
                book.cite("meters"),
            );
            true
        });
        if cooling.pipe_length_m.is_none() || cooling.uncooled_pipe_length_m.is_none() {
            recorder.record(
                "cooling_pipe_length_unknown_forfait",
                "cooling.pipeLengthM",
                "forfait (10.27, 15 % in uncooled spaces)".into(),
                book.cite("length"),
            );
        }
        for (field, value) in [
            ("pipeLengthM", cooling.pipe_length_m),
            ("uncooledPipeLengthM", cooling.uncooled_pipe_length_m),
        ] {
            if value.is_some_and(|value| !value.is_finite() || value < 0.0) {
                recorder.issue("cooling_pipe_length_invalid", format!("cooling.{field}"));
            }
        }
        // NTA table 10.11: any performed balancing, static or dynamic
        // (ISSO table 10.6), gives f_HB 1,0.
        system["distribution"] = json!({
            "designTemperature": design,
            "pipe": {"kind": pipe},
            "fittingsInsulated": fittings,
            "pipeLengthM": cooling.pipe_length_m,
            "unconditionedPipeLengthM": cooling.uncooled_pipe_length_m,
            "pump": {
                "hydraulicallyBalanced": balancing != "none_or_unknown",
                "floorCount": storeys.max(1),
                "heatMeter": meters,
                "individualDwellingInstallation": individual_dwelling,
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

/// The kernel `heatRecovery` from tables 11.10–11.12 (p. 150–151).
fn recovery_value(
    vent: &UtilityVentilation,
    exchanger: &str,
    combined: bool,
    recorder: &mut Recorder,
) -> Value {
    let reference = vent.source_reference.as_str();
    // Table 11.6 (p. 145): central or decentral; a system E unit is
    // decentral (D.5b).
    let layout = if combined || vent.heat_recovery_layout == Some(RecoveryLayout::Decentral) {
        "decentral"
    } else {
        "central"
    };
    let insulation = match vent.supply_duct_insulation {
        Some(SupplyDuctInsulationAnswer::Uninsulated) => json!({"kind": "uninsulated"}),
        Some(SupplyDuctInsulationAnswer::Insulated) => json!({"kind": "insulated"}),
        Some(SupplyDuctInsulationAnswer::Specified {
            thickness_m,
            conductivity_w_per_mk,
        }) => {
            if !(thickness_m.is_finite()
                && thickness_m > 0.0
                && conductivity_w_per_mk.is_finite()
                && conductivity_w_per_mk > 0.0)
            {
                recorder.issue(
                    "supply_duct_insulation_invalid",
                    "ventilation.supplyDuctInsulation",
                );
            }
            json!({
                "kind": "specified",
                "thicknessM": thickness_m,
                "conductivityWPerMK": conductivity_w_per_mk,
            })
        }
        None => {
            recorder.record(
                "supply_duct_insulation_unknown",
                "ventilation.supplyDuctInsulation",
                "not insulated (R < 0,3)".into(),
                "ISSO 75.1 p. 150 (table 11.10)",
            );
            json!({"kind": "uninsulated"})
        }
    };
    let constant_volume = vent.constant_volume_control.unwrap_or_else(|| {
        recorder.record(
            "constant_volume_unknown_none",
            "ventilation.constantVolumeControl",
            "false".into(),
            "ISSO 75.1 p. 151 (table 11.11)",
        );
        false
    });
    // Table 11.12 (p. 151): a partial bypass rounded down to tens; with the
    // bypass or its share unknown the kernel's "unknown" (11.3.2.2).
    let bypass = match vent.bypass_percent {
        Some(percent) if percent > 100 => {
            recorder.issue("bypass_percent_invalid", "ventilation.bypassPercent");
            json!({"kind": "none"})
        }
        Some(percent) => match percent / 10 * 10 {
            0 => json!({"kind": "none"}),
            100 => json!({"kind": "full"}),
            tens => json!({"kind": "partial", "fraction": f64::from(tens) / 100.0}),
        },
        None => {
            recorder.record(
                "bypass_table_11_12",
                "ventilation.bypassPercent",
                "unknown (NTA 11.3.2.2 defaults)".into(),
                "ISSO 75.1 p. 151 (table 11.12)",
            );
            json!({"kind": "unknown", "bypassPresent": vent.bypass_present.unwrap_or(false)})
        }
    };
    let mut recovery = json!({
        "efficiency": {"method": "table", "exchanger": exchanger},
        "bypass": bypass,
        "layout": layout,
        "constantVolumeControl": constant_volume,
        "supplyDuctInsulation": insulation,
        "equipmentReference": reference,
    });
    match vent.supply_duct_length_m {
        Some(length) if length.is_finite() && length >= 0.0 => {
            recovery["supplyDuctLengthM"] = json!(length);
        }
        Some(_) => recorder.issue(
            "supply_duct_length_invalid",
            "ventilation.supplyDuctLengthM",
        ),
        None => recorder.record(
            "supply_duct_length_unknown_default",
            "ventilation.supplyDuctLengthM",
            format!("kernel default for a {layout} system (11.109)"),
            "ISSO 75.1 p. 150 (table 11.10)",
        ),
    }
    if let Some(year) = vent.unit_manufacture_year {
        recovery["manufactureYear"] = json!(year);
    }
    recovery
}

#[allow(clippy::too_many_arguments)]
fn ventilation_value(
    survey: &UtilitySurvey,
    ventilation_functions: &[(&'static str, f64)],
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
    // Table 11.13 (p. 152): the class from a measurement or the p. 153
    // recognition rules; unknown 1,1.
    let duct_class = vent.duct_airtightness.or(match vent.ducts_luka_abc {
        Some(true) => Some(DuctAirtightnessAnswer::LukaAbc),
        _ => None,
    });
    let ducts = if !mechanical {
        "no_ducts"
    } else {
        match duct_class {
            Some(DuctAirtightnessAnswer::LukaAbc) => "luka_a_b_c",
            Some(DuctAirtightnessAnswer::LukaD) => "luka_d",
            Some(DuctAirtightnessAnswer::NoDucts) => "no_ducts",
            Some(DuctAirtightnessAnswer::Unknown) | None => {
                recorder.record(
                    "duct_airtightness_unknown",
                    "ventilation.ductAirtightness",
                    "unknown (f_lea;du 1,1)".into(),
                    "ISSO 75.1 p. 152–153 (table 11.13)",
                );
                "unknown"
            }
        }
    };
    let mut unit = json!({"variant": variant, "ducts": ducts, "equipmentReference": reference});
    // System E (§11.3.6, p. 145): the decentral part has heat recovery,
    // the other part follows `principle`.
    let combined = vent.combined.as_ref();
    if combined.is_some() && vent.principle == VentilationPrinciple::Balanced {
        recorder.issue("combined_other_part_not_balanced", "ventilation.combined");
    }
    let mut recovery = None;
    if vent.principle == VentilationPrinciple::Balanced || combined.is_some() {
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
            Some(ExchangerAnswer::ColdStorageWithAhu) => Some("run_around_coil_ahu"),
        };
        if combined.is_some() && exchanger.is_none() {
            recorder.issue(
                "combined_requires_heat_recovery",
                "ventilation.heatRecovery",
            );
        }
        if let Some(exchanger) = exchanger {
            if vent.declared_variant.is_none() && combined.is_none() {
                unit["variant"] = json!("d2");
            }
            recovery = Some(recovery_value(
                vent,
                exchanger,
                combined.is_some(),
                recorder,
            ));
        }
    }
    if vent.ahu.is_none()
        && survey.cooling.as_ref().is_some_and(|cooling| {
            !cooling.water_based
                && cooling.direct_expansion == Some(DirectExpansionAnswer::AirHandlingUnit)
        })
    {
        recorder.issue(
            "cooling_direct_expansion_ahu_requires_ahu",
            "cooling.directExpansion",
        );
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
        // p. 149: heating and cooling connected to the AHU become the
        // reheating and cooling coils of NTA table 11.15.
        // ISSO 82.1 p. 123: air heating through the AHU is also entered with
        // the ventilation, so it implies the reheating coil.
        let via_ahu = matches!(
            survey.heating.air_heating,
            Some(AirHeatingAnswer::ViaAirHandlingUnit)
        );
        if via_ahu && ahu.heating_connected == Some(false) {
            recorder.issue(
                "air_heating_via_ahu_requires_heating_coil",
                "ventilation.ahu.heatingConnected",
            );
        }
        let heating_coil = ahu.heating_connected.unwrap_or_else(|| {
            if via_ahu {
                recorder.record(
                    "ahu_heating_from_air_heating",
                    "ventilation.ahu.heatingConnected",
                    "reheating coil: heating is air heating via the AHU".into(),
                    "ISSO 82.1 p. 123 (table 9.16b); ISSO 75.1 p. 149",
                );
                return true;
            }
            recorder.record(
                "ahu_heating_unknown_none",
                "ventilation.ahu.heatingConnected",
                "no reheating coil (interpretation: not determinable is not connected)".into(),
                "ISSO 75.1 p. 149",
            );
            false
        });
        // §10.4.1 (p. 133): direct expansion in the AHU delivers its cold
        // through the AHU cooling coil.
        let dx_in_ahu = survey.cooling.as_ref().is_some_and(|cooling| {
            !cooling.water_based
                && cooling.direct_expansion == Some(DirectExpansionAnswer::AirHandlingUnit)
        });
        if dx_in_ahu && ahu.cooling_connected == Some(false) {
            recorder.issue(
                "cooling_direct_expansion_ahu_requires_cooling_coil",
                "ventilation.ahu.coolingConnected",
            );
        }
        let cooling_coil = ahu.cooling_connected.unwrap_or_else(|| {
            if dx_in_ahu {
                recorder.record(
                    "ahu_cooling_from_direct_expansion",
                    "ventilation.ahu.coolingConnected",
                    "cooling coil: direct expansion in the AHU".into(),
                    "ISSO 75.1 p. 130, 133, 152",
                );
                return true;
            }
            recorder.record(
                "ahu_cooling_unknown_none",
                "ventilation.ahu.coolingConnected",
                "no cooling coil (interpretation: not determinable is not connected)".into(),
                "ISSO 75.1 p. 149",
            );
            false
        });
        if cooling_coil && survey.cooling.is_none() {
            recorder.issue(
                "ahu_cooling_requires_cooling_system",
                "ventilation.ahu.coolingConnected",
            );
        }
        unit["airHandlingUnit"] = json!({
            "insideThermalZone": inside,
            "supplyDuctsOutside": situation,
            "heatingCoil": heating_coil,
            "coolingCoil": cooling_coil,
        });
    }
    // Tables 11.7/11.8 (p. 148–149).
    let mut flow_reduction = json!({});
    let recirculation = match (vent.recirculation_percent, vent.recirculation) {
        // NTA 11.60: x = 20; a proven higher value, rounded down to tens.
        (Some(percent), _) => Some((percent / 10 * 10).max(20)),
        (None, Some(RecirculationAnswer::PresentPercentUnknown)) => {
            recorder.record(
                "recirculation_percent_unknown_x_20",
                "ventilation.recirculation",
                "< 20 %: NTA default x = 20".into(),
                "ISSO 75.1 p. 148 (table 11.7); NTA 8800 11.60",
            );
            Some(20)
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
            // NTA 11.61: x = 80; only a proven lower value replaces it.
            flow_reduction["flowControlPercent"] = json!(percent.min(80));
        }
        None => recorder.record(
            "flow_control_unknown_none",
            "ventilation.flowControl",
            "none".into(),
            "ISSO 75.1 p. 149 (table 11.8)",
        ),
    }
    // Notes 1 and 2 under 11.60/11.61: a surveyed, proven percentage is
    // backed by the ventilation evidence of the survey.
    let favourable = flow_reduction["recirculationPercent"]
        .as_u64()
        .is_some_and(|x| x > 20)
        || flow_reduction["flowControlPercent"]
            .as_u64()
            .is_some_and(|x| x < 80);
    if favourable {
        flow_reduction["evidenceReference"] = json!(vent.source_reference);
    }
    // Table 11.15: fan manufacture year unknown → construction year. This
    // specific rule takes precedence over the general installation-year
    // fallback (and is the conservative one).
    let fan_year = vent.unit_manufacture_year.unwrap_or_else(|| {
        recorder.record(
            "fan_year_unknown_construction_year",
            "ventilation.fans",
            year.to_string(),
            "ISSO 75.1 p. 154 (table 11.15; specific rule over p. 30)",
        );
        year
    });
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
    let system = match combined {
        Some(combined) => {
            recorder.record(
                "combined_system_e1",
                "ventilation.combined",
                "decentral part D.5b, other part per principle".into(),
                "ISSO 75.1 p. 145 (§11.3.6); NTA table 11.5 E.1",
            );
            let mut decentral = json!({
                "variant": "d5b",
                "ducts": "no_ducts",
                "equipmentReference": reference,
            });
            if let Some(recovery) = recovery {
                decentral["heatRecovery"] = recovery;
            }
            json!({
                "kind": "combined",
                "decentralAreaM2": combined.decentral_area_m2,
                "totalResidenceAreaM2": combined.total_residence_area_m2,
                "decentral": decentral,
                "other": unit,
            })
        }
        None => {
            if let Some(recovery) = recovery {
                unit["heatRecovery"] = recovery;
            }
            json!({"kind": "single", "unit": unit})
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
    let mut input = json!({
        "zoneId": "utiliteit",
        "usableFloorAreaM2": area,
        "category": "utility",
        "functions": ventilation_function_values(survey, ventilation_functions, recorder),
        "dwellingCount": 0,
        "buildingHeightM": survey.building_height_m,
        "constructionYear": year,
        "floorAboveCrawlspace": floor_above_crawlspace,
        "heatingSetpointC": heating_c,
        "coolingSetpointC": 24.0,
        "system": system,
        "flowReduction": flow_reduction,
        "infiltration": infiltration,
        "fans": {"method": "forfait", "current": current, "manufactureYear": fan_year},
        "sourceReference": format!("{reference}; basisopname"),
    });
    // §11.4.1 (p. 146–148): the installed capacity for systems B–E; with a
    // swimming pool in the zone the flow counts as unknown (p. 148).
    if let Some(total) = vent.installed_capacity_dm3_per_s {
        let pool = survey.swimming_pool_area_m2.is_some_and(|pool| pool > 0.0);
        if !mechanical {
            recorder.issue(
                "installed_capacity_requires_mechanical_ventilation",
                "ventilation.installedCapacityDm3PerS",
            );
        } else if !(total.is_finite() && total > 0.0) {
            recorder.issue(
                "installed_capacity_invalid",
                "ventilation.installedCapacityDm3PerS",
            );
        } else if pool {
            recorder.record(
                "swimming_pool_installed_capacity_unknown",
                "ventilation.installedCapacityDm3PerS",
                "unknown (regulatory flow): swimming pool in the zone".into(),
                "ISSO 75.1 p. 148",
            );
        } else {
            input["installedCapacity"] = json!({
                "totalDm3PerS": total,
                "sourceReference": reference,
            });
        }
    }
    apply_passive_cooling(
        &mut input,
        vent.passive_cooling.as_ref(),
        vent.principle,
        vent.bypass_present,
        recorder,
    );
    if let Some(strips) = &vent.grille_heating_strips {
        input["grillePreheating"] = grille_preheating_value(strips, recorder);
    }
    input
}

/// §11.3.7 (p. 145–146): the grille settings from product data; without
/// all four the kernel's 11.124 fallback. Share unknown: all grilles.
fn grille_preheating_value(strips: &SurveyGrilleHeatingStrips, recorder: &mut Recorder) -> Value {
    let control = match (
        strips.max_power_w_per_dm3_per_s,
        strips.max_temperature_rise_k,
        strips.switch_on_below_c,
        strips.max_supply_temperature_c,
    ) {
        (Some(power), Some(rise), Some(switch_on), Some(supply)) => json!({
            "method": "specified",
            "maxPowerWPerDm3PerS": power,
            "maxTemperatureRiseK": rise,
            "switchOnBelowC": switch_on,
            "maxSupplyTemperatureC": supply,
        }),
        _ => {
            recorder.record(
                "grille_heating_strip_settings_unknown",
                "ventilation.grilleHeatingStrips",
                "NTA 11.124 fallback".into(),
                "ISSO 75.1 p. 146 (§11.3.7)",
            );
            json!({"method": "fallback"})
        }
    };
    recorder.record(
        "grille_heating_strips_all_grilles",
        "ventilation.grilleHeatingStrips",
        "all grilles".into(),
        "ISSO 75.1 p. 146 (§11.3.7)",
    );
    json!({"control": control, "sourceReference": strips.source_reference})
}

/// The ventilation functions with the swimming-pool room split off the
/// sport function (p. 65; NTA §11.2.2.5.1).
fn ventilation_function_values(
    survey: &UtilitySurvey,
    functions: &[(&'static str, f64)],
    recorder: &mut Recorder,
) -> Vec<Value> {
    let mut values: Vec<Value> = functions
        .iter()
        .map(|(function, part)| json!({"function": function, "areaM2": part}))
        .collect();
    let Some(pool) = survey.swimming_pool_area_m2.filter(|pool| *pool > 0.0) else {
        return values;
    };
    let Some(sport) = values.iter_mut().find(|value| value["function"] == "sport") else {
        if survey
            .functions
            .iter()
            .any(|item| item.function == LabelFunction::Sport)
        {
            // p. 54: only the functions left after the merge of p. 39–40
            // occur in the calculation zone.
            recorder.record(
                "swimming_pool_in_merged_sport_function",
                "swimmingPoolAreaM2",
                "sport merged into the main function: no pool factor".into(),
                "ISSO 75.1 p. 39–40, 54",
            );
        } else {
            recorder.issue(
                "swimming_pool_requires_sport_function",
                "swimmingPoolAreaM2",
            );
        }
        return values;
    };
    let sport_area = sport["areaM2"].as_f64().unwrap_or(0.0);
    if !pool.is_finite() || pool > sport_area + 1e-6 {
        recorder.issue("swimming_pool_area_invalid", "swimmingPoolAreaM2");
        return values;
    }
    sport["areaM2"] = json!(sport_area - pool);
    values.retain(|value| value["areaM2"].as_f64().unwrap_or(0.0) > 1e-9);
    values.push(json!({"function": "sport", "areaM2": pool, "swimmingPool": true}));
    values
}

/// p. 65 and NTA 13.32a: the sport and swimming halls served by a
/// hot-water system with circulation, when the building has A_g ≥
/// 1 000 m²; unknown counts as 0 m² (the longer forfait loop).
fn sport_hall_area(
    survey: &UtilitySurvey,
    served: &[(LabelFunction, f64)],
    recorder: &mut Recorder,
) -> f64 {
    let total: f64 = survey.functions.iter().map(|item| item.area_m2).sum();
    let sport: f64 = survey
        .functions
        .iter()
        .filter(|item| item.function == LabelFunction::Sport)
        .map(|item| item.area_m2)
        .sum();
    if sport <= 0.0 || total < 1000.0 {
        return 0.0;
    }
    // The group holding the sport function: itself, or the main function
    // it was merged into (p. 39–40).
    let Some(groups) = function_groups(&survey.functions, &mut Recorder::default()) else {
        return 0.0;
    };
    let sport_group = if groups
        .groups
        .iter()
        .any(|(function, _)| *function == LabelFunction::Sport)
    {
        LabelFunction::Sport
    } else {
        groups.main
    };
    if !served.iter().any(|(function, _)| *function == sport_group) {
        return 0.0;
    }
    match survey.sport_hall_area_m2 {
        Some(area) if area.is_finite() && area >= 0.0 && area <= sport + 1e-6 => area,
        Some(_) => {
            recorder.issue("sport_hall_area_invalid", "sportHallAreaM2");
            0.0
        }
        None => {
            recorder.record(
                "sport_hall_area_unknown_0",
                "sportHallAreaM2",
                "0 m² (no reduction of A_g in 13.32a)".into(),
                "ISSO 75.1 p. 65 (interpretation: conservative)",
            );
            0.0
        }
    }
}

fn hot_water_value(
    survey: &UtilitySurvey,
    hot: &UtilityHotWater,
    main: LabelFunction,
    groups: &[(LabelFunction, f64)],
    recorder: &mut Recorder,
) -> Value {
    let reference = hot.source_reference.as_str();
    // Generators shared with the residential layer go through its mapping.
    let shared = shared_hot_water_answer(&hot.generator);
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
                nominal_power_kw: None,
                additional_generators: Vec::new(),
                collective: None,
                solar: Vec::new(),
                source_reference: reference.to_string(),
                connected_bathrooms: None,
                connected_kitchens: None,
            },
            recorder,
        ),
        None => {
            let generator =
                utility_only_generator(&hot.generator, survey, "hotWater.generator", recorder);
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
        "areas": groups
            .iter()
            .map(|(function, part)| json!({"function": label_name(*function), "areaM2": part}))
            .collect::<Vec<_>>(),
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
                "sportHallAreaM2": sport_hall_area(survey, groups, recorder),
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
    super::hot_water::apply_solar(
        &mut system,
        &hot.solar,
        survey.construction_year,
        true,
        recorder,
    );
    // NTA 13.8.2: several generators with their nominal powers.
    if let Some(power) = hot.nominal_power_kw {
        system["nominalPowerKw"] = json!(power);
    }
    if !hot.additional_generators.is_empty() {
        let extras: Vec<Value> = hot
            .additional_generators
            .iter()
            .enumerate()
            .map(|(index, extra)| {
                let path = format!("hotWater.additionalGenerators[{index}].generator");
                let generator = match shared_hot_water_answer(&extra.generator) {
                    Some(answer) => {
                        super::hot_water::convert_generator(&answer, false, &path, recorder)
                    }
                    None => utility_only_generator(&extra.generator, survey, &path, recorder),
                };
                let mut value = json!({"generator": generator, "equipmentReference": reference});
                if let Some(power) = extra.nominal_power_kw {
                    value["nominalPowerKw"] = json!(power);
                }
                value
            })
            .collect();
        system["additionalGenerators"] = json!(extras);
    }
    system
}

/// Utility generator types shared with the residential mapping.
fn shared_hot_water_answer(
    generator: &UtilityHotWaterGenerator,
) -> Option<HotWaterGeneratorAnswer> {
    match generator {
        UtilityHotWaterGenerator::None => Some(HotWaterGeneratorAnswer::None),
        UtilityHotWaterGenerator::GasAppliance {
            appliance_type,
            gaskeur,
            burner_load_kw,
        } => Some(HotWaterGeneratorAnswer::GasAppliance {
            appliance_type: *appliance_type,
            gaskeur: *gaskeur,
            burner_load_kw: *burner_load_kw,
            cw_class: None,
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
    }
}

/// Kernel generator of the utility-only types (gas storage heater p. 167,
/// unknown collective table 13.2 p. 165).
fn utility_only_generator(
    generator: &UtilityHotWaterGenerator,
    survey: &UtilitySurvey,
    path: &str,
    recorder: &mut Recorder,
) -> Value {
    match generator {
        UtilityHotWaterGenerator::GasStorageHeater {
            volume_l,
            before_1985,
            in_heated_zone,
        } => {
            let before = before_1985.unwrap_or_else(|| {
                recorder.record(
                    "gas_storage_year_construction_year",
                    &format!("{path}.before1985"),
                    survey.construction_year.to_string(),
                    "ISSO 75.1 p. 167",
                );
                survey.construction_year < 1985
            });
            let inside = in_heated_zone.unwrap_or_else(|| {
                recorder.record(
                    "gas_storage_location_unknown_outside",
                    &format!("{path}.inHeatedZone"),
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
                path,
                "other directly heated storage".into(),
                "ISSO 75.1 p. 165 (table 13.2)",
            );
            json!({"kind": "large_direct_storage", "gasFired": true})
        }
    }
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
    groups: &[(LabelFunction, f64)],
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
    // ISSO 75.1 p. 188: the LED value needs LED installed from 2017. Zones
    // with a measured power that move to the forfait count as LED only
    // when every lamp is an LED type (their year is the advisor's claim);
    // a luminaire power list carries no lamp type and counts as not LED.
    let measured_led = survey.lighting.iter().all(|zone| match &zone.power {
        LightingPowerAnswer::Unknown { .. } => true,
        LightingPowerAnswer::Lamps { lamps } => lamps.iter().all(|lamp| {
            matches!(
                lamp.lamp_type,
                LampTypeAnswer::LedInLuminaire | LampTypeAnswer::LedLamp
            )
        }),
        _ => false,
    });
    let whole_forfait =
        (!forfait_led.is_empty()).then(|| forfait_led.iter().all(|led| *led) && measured_led);
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
        "functions": groups
            .iter()
            .map(|(function, part)| json!({"function": label_name(*function), "areaM2": part}))
            .collect::<Vec<_>>(),
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
    let input = derive_utility_input_cited(survey, recorder);
    // The shared (ISSO 82.1) translations cite the ISSO 75.1 pages.
    remap_sources(&mut recorder.applied);
    input
}

fn derive_utility_input_cited(survey: &UtilitySurvey, recorder: &mut Recorder) -> Option<Value> {
    validate(survey, recorder);
    if !recorder.issues.is_empty() {
        return None;
    }
    let groups = function_groups(&survey.functions, recorder)?;
    check_zone_split(&groups, survey, recorder);
    if !recorder.issues.is_empty() {
        return None;
    }
    let (main, area) = (groups.main, groups.total_m2);
    let (usage, _, reduction_function) = kernel_names(main);
    // §6.5.3: area-weighted values of a mixed calculation zone.
    let function_areas: Vec<crate::monthly_demand::UsageFunctionArea> = groups
        .groups
        .iter()
        .map(
            |(function, part)| crate::monthly_demand::UsageFunctionArea {
                function: kernel_names(*function).0,
                area_m2: *part,
            },
        )
        .collect();
    let profile = crate::monthly_demand::FunctionProfile::weighted(&function_areas);
    let heating_c = profile.heating_setpoint_c;
    let cooling_c = profile.cooling_setpoint_c;
    let ventilation_functions: Vec<(&'static str, f64)> = groups
        .groups
        .iter()
        .map(|(function, part)| (kernel_names(*function).1, *part))
        .collect();
    let year = survey.construction_year;
    let airtightness = utility_airtightness(&survey.building_type, recorder);
    let infiltration = infiltration_year(year, survey.renovation.as_ref(), recorder);
    let (floor, wall, ceiling) = thermal_mass(&survey.construction);
    let mut envelope =
        derive_envelope_with_cooling(&survey.envelope, year, survey.cooling.is_some(), recorder);
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
        &ventilation_functions,
        area,
        heating_c,
        envelope.floor_above_crawlspace,
        airtightness,
        infiltration,
        recorder,
    );
    let heating = derive_utility_heating(survey, reduction_function, area, recorder);
    // NTA 13.20/13.20a: each additional system serves its own areas (a
    // merged small function counts as the main function); the main system
    // serves the rest.
    let to_group = |function: LabelFunction| {
        if groups.groups.iter().any(|(group, _)| *group == function) {
            function
        } else {
            main
        }
    };
    let mut main_served = groups.groups.clone();
    let mut extra_systems = Vec::new();
    for (index, system) in survey.additional_hot_water_systems.iter().enumerate() {
        let mut served: Vec<(LabelFunction, f64)> = Vec::new();
        for part in &system.served_areas {
            let function = to_group(part.function);
            match served.iter_mut().find(|(item, _)| *item == function) {
                Some(item) => item.1 += part.area_m2,
                None => served.push((function, part.area_m2)),
            }
            if let Some(rest) = main_served.iter_mut().find(|(item, _)| *item == function) {
                rest.1 -= part.area_m2;
            }
        }
        if served.is_empty() {
            recorder.issue(
                "hot_water_served_areas_required",
                format!("additionalHotWaterSystems[{index}].servedAreas"),
            );
            continue;
        }
        let mut value = hot_water_value(survey, system, main, &served, recorder);
        super::hot_water::apply_exhaust_air_use(
            &mut value,
            survey.ventilation.principle,
            survey.ventilation.heat_recovery.is_some(),
        );
        extra_systems.push(value);
    }
    if main_served.iter().any(|(_, part)| *part < -1e-6) {
        recorder.issue(
            "hot_water_served_areas_exceed_function",
            "additionalHotWaterSystems",
        );
    }
    main_served.retain(|(_, part)| *part > 1e-6);
    if !survey.hot_water.served_areas.is_empty() {
        recorder.warning(
            "hot_water_main_served_areas_ignored",
            "hotWater.servedAreas",
            "the main hot-water system serves the areas not taken by additional systems",
        );
    }
    let mut hot_water = hot_water_value(survey, &survey.hot_water, main, &main_served, recorder);
    super::hot_water::apply_exhaust_air_use(
        &mut hot_water,
        survey.ventilation.principle,
        survey.ventilation.heat_recovery.is_some(),
    );
    let lighting = lighting_value(survey, &groups.groups, recorder);
    let cooling = survey.cooling.as_ref().map(|cooling| {
        cooling_value(
            cooling,
            year,
            survey.storeys,
            CoolingBook::Utility,
            false,
            recorder,
        )
    });
    let pv: Vec<Value> = survey
        .pv
        .iter()
        .map(|item| derive_pv(item, year, recorder))
        .collect();
    let (bacs, bacs_reference) = bacs_factor(
        &survey.bacs,
        &surveyed_system_powers(survey),
        area,
        recorder,
    );
    let (storage_present, storage) =
        super::production::derive_storage(survey.storage.as_ref(), !survey.pv.is_empty(), recorder);
    // Table 7.8 (p. 68): one uninsulated pipe per toilet group through all
    // storeys when the pipes are not determinable.
    let stacks = match (&survey.vertical_pipes, survey.toilet_stacks) {
        (Some(_), _) => 0,
        (None, Some(stacks)) => {
            recorder.record(
                "vertical_pipes_unknown_one_per_toilet_group",
                "verticalPipes",
                format!(
                    "{stacks} uninsulated pipe(s) through {} storey(s)",
                    survey.storeys.max(1)
                ),
                "ISSO 75.1 p. 68 (table 7.8); NTA 8800 7.3.3",
            );
            stacks
        }
        (None, None) => {
            recorder.record(
                "vertical_pipes_unknown_one_toilet_group",
                "toiletStacks",
                "1 uninsulated pipe (interpretation: at least one toilet group)".into(),
                "ISSO 75.1 p. 68 (table 7.8); NTA 8800 7.3.3",
            );
            1
        }
    };
    if !recorder.issues.is_empty() {
        return None;
    }
    let demand = json!({
        "zoneId": "utiliteit",
        "usableFloorAreaM2": area,
        "areaSourceReference": survey.area_source_reference,
        "usageFunction": usage_name(usage),
        "functionAreas": if function_areas.len() > 1 {
            json!(function_areas
                .iter()
                .map(|part| json!({"function": usage_name(part.function), "areaM2": part.area_m2}))
                .collect::<Vec<_>>())
        } else {
            json!([])
        },
        "setpoints": {"heatingC": heating_c, "coolingC": cooling_c, "sourceReference": "NTA 8800 table 7.13 (§6.5.3 weighted)"},
        "transmission": {
            "method": "components",
            "direct": {"elements": envelope.direct_elements, "linearBridges": [], "pointBridges": []},
            "unheated": envelope.unheated,
            "groundFloors": envelope.ground_floors,
            "groundInventoryConfirmed": true,
            "verticalPipes": super::vertical_pipes(
                survey.vertical_pipes.as_deref(),
                survey.storeys,
                stacks,
                &survey.source_reference,
                recorder,
            ),
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
    if let Some(humidifier) = humidifier_value(survey) {
        chain["humidifiers"] = json!([humidifier]);
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
        "labelFunctions": if groups.groups.len() > 1 {
            json!(groups
                .groups
                .iter()
                .map(|(function, part)| json!({"function": label_name(*function), "areaM2": part}))
                .collect::<Vec<_>>())
        } else {
            json!([])
        },
        "lossAreaM2": loss_area(&survey.envelope),
        "lossAreaSourceReference": "basisopname: survey surfaces with f_ls (NTA 6.7.3)",
        "lighting": [lighting],
        "hotWater": hot_water,
        "additionalHotWaterSystems": extra_systems,
        "demandUsesFixedC1Ventilation": false,
        "batteryStoragePresent": storage_present,
    });
    if let Some(storage) = storage {
        input["storage"] = storage;
    }
    // §7.1.7 (p. 61–62): fossil installations on the plot that may fall
    // outside the calculation decide "lokaal koolstofemissievrij" (§5.5.7).
    match survey.fossil_fuel_on_plot {
        Some(present) => input["fossilAppliancesOutsideCalculation"] = json!(present),
        None => recorder.record(
            "fossil_fuel_on_plot_not_established",
            "fossilFuelOnPlot",
            "not established: only the calculated carriers decide".into(),
            "ISSO 75.1 p. 61–62 (§7.1.7)",
        ),
    }
    if let (Some(mut value), Some(answer)) = (cooling, survey.cooling.as_ref()) {
        apply_cooling_heat_pump_source(
            &mut value,
            answer,
            &input["spaceHeating"]["generator"],
            CoolingBook::Utility,
            recorder,
        );
        input["cooling"] = value;
    }
    if let Some(renewable) = heating.heat_pump_renewable {
        input["heatPumpRenewable"] = renewable;
    }
    Some(input)
}

/// Chapter 12 humidifier of the survey (p. 161) for the heating chain:
/// atomising humidifiers load the space-heating node (12.1, 9.4), steam
/// humidifiers book their own carrier (12.3) and recoverable loss (12.4).
fn humidifier_value(survey: &UtilitySurvey) -> Option<Value> {
    let humidification = survey.humidification.as_ref()?;
    let humidifier = match humidification.humidifier {
        HumidifierAnswer::ElectricSteam => Humidifier::Steam {
            carrier: SteamCarrier::Electricity,
        },
        HumidifierAnswer::NonElectricSteam => Humidifier::Steam {
            carrier: SteamCarrier::GasOrOil,
        },
        HumidifierAnswer::Adiabatic => Humidifier::Atomising,
    };
    let value = Humidification {
        humidifier,
        rotary_wheel: humidification.absorption_wheel,
        equipment_reference: humidification.source_reference.clone(),
    };
    Some(json!({
        "zoneId": "utiliteit",
        "humidification": serde_json::to_value(value).expect("typed humidification serializes"),
    }))
}

/// Survey → kernel input → building performance (utility).
pub fn assess_utility_survey(survey: &UtilitySurvey) -> OpnameAssessment {
    let mut recorder = Recorder::default();
    let derived = derive_utility_input(survey, &mut recorder);
    let derived_input =
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
    let performance = derived_input.as_ref().map(assess_building_performance);
    remap_sources(&mut recorder.applied);
    super::apply_collapse_reasons(&mut recorder, &survey.collapse_reasons);
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
    fn chp_thermal_power_follows_the_engine_of_table_9_7() {
        use super::super::heating::{ChpEngine, HeatingGenerator};
        for (engine, expected) in [
            (Some(ChpEngine::DieselEngine), 24.0),
            (Some(ChpEngine::MicroTurbine), 50.0),
            (None, 30.0),
        ] {
            let mut survey = fixture("1985");
            survey.heating.generator = HeatingGenerator::Chp {
                electrical_power_kw: 20.0,
                thermal_power_kw: None,
                engine,
                manufacture_year: Some(2012),
                hre_declared: false,
                low_temperature: false,
            };
            survey.heating.additional_generators =
                vec![super::super::heating::AdditionalHeatingGenerator {
                    generator: super::super::heating::HeatingGenerator::Boiler {
                        boiler_type: super::super::heating::BoilerType::Hr107,
                        pilot_flame: Some(false),
                        inside_thermal_boundary: true,
                        manufacture_year: Some(2012),
                        installation_year: None,
                    },
                    nominal_power_kw: Some(150.0),
                }];
            let result = assess_utility_survey(&survey);
            let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
            let parts = input["spaceHeating"]["generator"]["generators"]
                .as_array()
                .unwrap()
                .clone();
            let chp = parts
                .iter()
                .find(|part| part["generator"]["kind"] == "chp")
                .unwrap();
            assert_eq!(chp["nominalPowerKw"], expected, "{engine:?}");
            let unknown_recorded = result
                .applied_defaults
                .iter()
                .any(|item| item.rule == "chp_engine_unknown_gas");
            assert_eq!(unknown_recorded, engine.is_none());
        }
    }

    #[test]
    fn additional_hot_water_system_serves_its_own_areas() {
        let mut survey = fixture("1985");
        let main_area: f64 = survey.functions[0].area_m2;
        let mut extra = survey.hot_water.clone();
        extra.generator = UtilityHotWaterGenerator::ElectricInstantaneous;
        extra.storage.clear();
        extra.solar.clear();
        extra.additional_generators.clear();
        extra.nominal_power_kw = None;
        extra.served_areas = vec![FunctionArea {
            function: survey.functions[0].function,
            area_m2: 200.0,
        }];
        survey.additional_hot_water_systems = vec![extra];
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?}",
            (
                &result.issues,
                result.performance.as_ref().map(|item| &item.issues)
            )
        );
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        let extras = input["additionalHotWaterSystems"].as_array().unwrap();
        assert_eq!(extras.len(), 1);
        assert_eq!(extras[0]["need"]["areas"][0]["areaM2"], 200.0);
        // NTA 13.20a: the main system serves the rest of that function.
        let main_areas = input["hotWater"]["need"]["areas"].as_array().unwrap();
        let first = main_areas[0]["areaM2"].as_f64().unwrap();
        assert!(first < main_area + 1e-9 && first > 0.0);
        // Serving more than the function has is rejected.
        let mut over = survey.clone();
        over.additional_hot_water_systems[0].served_areas[0].area_m2 = 1.0e6;
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&over, &mut recorder).is_none());
        assert!(recorder
            .issues
            .iter()
            .any(|item| item.code == "hot_water_served_areas_exceed_function"));
    }

    #[test]
    fn utility_hot_water_takes_several_generators_of_its_own_types() {
        let mut survey = fixture("1985");
        survey.hot_water.generator = UtilityHotWaterGenerator::GasStorageHeater {
            volume_l: 200.0,
            before_1985: Some(false),
            in_heated_zone: Some(true),
        };
        survey.hot_water.storage.clear();
        survey.hot_water.nominal_power_kw = Some(30.0);
        survey.hot_water.additional_generators = vec![UtilityAdditionalHotWater {
            generator: UtilityHotWaterGenerator::ElectricInstantaneous,
            nominal_power_kw: Some(10.0),
        }];
        let result = assess_utility_survey(&survey);
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        let hot = &input["hotWater"];
        assert_eq!(hot["generator"]["kind"], "gas_storage_heater");
        assert_eq!(hot["nominalPowerKw"], 30.0);
        let extras = hot["additionalGenerators"].as_array().unwrap();
        assert_eq!(extras.len(), 1);
        assert_eq!(extras[0]["nominalPowerKw"], 10.0);
        assert_eq!(extras[0]["generator"]["kind"], "electric_instantaneous");
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
    }

    #[test]
    fn chp_with_peak_boilers_and_a_solar_water_heater() {
        use super::super::heating::{AdditionalHeatingGenerator, BoilerType, HeatingGenerator};
        let mut survey = fixture("1985");
        let boiler = survey.heating.generator.clone();
        assert!(matches!(boiler, HeatingGenerator::Boiler { .. }));
        survey.heating.generator = HeatingGenerator::Chp {
            electrical_power_kw: 20.0,
            thermal_power_kw: None,
            engine: None,
            manufacture_year: Some(2012),
            hre_declared: false,
            low_temperature: false,
        };
        survey.heating.additional_generators = vec![AdditionalHeatingGenerator {
            generator: HeatingGenerator::Boiler {
                boiler_type: BoilerType::Hr107,
                pilot_flame: Some(false),
                inside_thermal_boundary: true,
                manufacture_year: Some(2012),
                installation_year: None,
            },
            nominal_power_kw: Some(150.0),
        }];
        survey.hot_water.solar = vec![super::super::hot_water::SurveySolarWaterHeater {
            id: "zb".into(),
            collector: super::super::hot_water::CollectorAnswer::Glazed,
            collector_area_m2: 10.0,
            gross_area: false,
            collector_count: 4,
            orientation: crate::climate::Orientation::South,
            tilt_deg: 30.0,
            shading: None,
            backup: super::super::hot_water::SolarBackupAnswer::Unknown,
            storage_volume_l: 500.0,
            backup_volume_l: None,
            storage_label: None,
            storage_manufacture_year: Some(2012),
            also_space_heating: false,
            pvt: None,
            source_reference: "datasheet".into(),
        }];
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(generator["kind"], "multiple");
        let parts = generator["generators"].as_array().unwrap();
        let chp = parts
            .iter()
            .find(|part| part["generator"]["kind"] == "chp")
            .unwrap();
        // Table 9.7: thermal power 1,5 × 20 kW; p. 112: CHP before boilers.
        assert_eq!(chp["nominalPowerKw"], 30.0);
        assert_eq!(chp["preference"], 1);
        let boiler = parts
            .iter()
            .find(|part| part["generator"]["kind"] == "gas_boiler")
            .unwrap();
        // The collective utility adjustments reach the part.
        assert_eq!(boiler["generator"]["boiler"]["role"], "collective");
        assert_eq!(boiler["generator"]["auxiliary"]["nominalPowerKw"], 150.0);
        assert!(input["hotWater"]["solar"]
            .as_array()
            .is_some_and(|items| items.len() == 1));
        let performance = result.performance.unwrap();
        let produced: f64 = performance
            .electricity_balance
            .iter()
            .map(|month| month.produced_kwh)
            .sum();
        assert!(produced > 0.0, "CHP electricity (16.12)");
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
        // Above 25 % the function stays separate: a §6.5.3 mixed zone.
        let mut survey = fixture("1985");
        survey.functions[1].area_m2 = 500.0;
        survey.lighting[1].area_m2 = 600.0;
        let (input, recorder) = derive(&survey);
        assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
        assert!(applied(&recorder, "larger_functions_kept_separate"));
        let areas = input["spaceHeating"]["demand"]["functionAreas"]
            .as_array()
            .unwrap();
        assert_eq!(areas.len(), 2);
        assert_eq!(input["labelFunctions"].as_array().unwrap().len(), 2);
        assert_eq!(
            input["hotWater"]["need"]["areas"].as_array().unwrap().len(),
            2
        );
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
    }

    #[test]
    fn merging_starts_with_the_smallest_function() {
        let functions = [
            FunctionArea {
                function: LabelFunction::Office,
                area_m2: 600.0,
            },
            FunctionArea {
                function: LabelFunction::Retail,
                area_m2: 200.0,
            },
            FunctionArea {
                function: LabelFunction::Education,
                area_m2: 150.0,
            },
            FunctionArea {
                function: LabelFunction::Sport,
                area_m2: 50.0,
            },
        ];
        let mut recorder = Recorder::default();
        let groups = function_groups(&functions, &mut recorder).unwrap();
        // 25 % of 1000 m²: sport 50 + education 150 = 200 merge; retail stays.
        assert_eq!(groups.main, LabelFunction::Office);
        assert_eq!(
            groups.groups,
            vec![
                (LabelFunction::Office, 800.0),
                (LabelFunction::Retail, 200.0)
            ]
        );
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
        assert_eq!(bacs_factor(&small, &[], 4000.0, &mut recorder).0, 1.0);
        // Power and served area unknown: the building A_g decides.
        assert_eq!(
            bacs_factor(&SurveyBacs::default(), &[], 4000.0, &mut recorder).0,
            1.05
        );
        assert!(applied(&recorder, "bacs_served_area_unknown_building"));
        assert_eq!(
            bacs_factor(&SurveyBacs::default(), &[], 2000.0, &mut recorder).0,
            1.0
        );
        let unknown = SurveyBacs {
            served_area_m2: Some(2600.0),
            ..SurveyBacs::default()
        };
        assert_eq!(bacs_factor(&unknown, &[], 4000.0, &mut recorder).0, 1.05);
        assert!(applied(&recorder, "bacs_power_unknown_served_area"));
        assert!(applied(&recorder, "bacs_presence_unknown_no"));
        let present = SurveyBacs {
            system_power_kw: Some(400.0),
            present: Some(true),
            ..SurveyBacs::default()
        };
        assert_eq!(bacs_factor(&present, &[], 4000.0, &mut recorder).0, 1.05);
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
        assert_eq!(bacs_factor(&compliant, &[], 4000.0, &mut recorder).0, 1.0);
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
    fn pipes_in_unheated_spaces_follow_afb_9_1() {
        // No unheated space: no pipes there, length 0 (not the 15 % forfait).
        let (input, recorder) = derive(&fixture("1985"));
        let system = &input["spaceHeating"]["distributionSystem"];
        assert_eq!(system["unheatedPipeLengthM"], 0.0);
        assert!(applied(&recorder, "unheated_pipes_absent_length_0"));
        // A crawlspace with unknown pipes: present, forfait length.
        let mut survey = fixture("1985");
        survey.envelope.surfaces[0].boundary = super::super::envelope::SurfaceBoundary::Crawlspace;
        survey.envelope.surfaces[0].element = super::super::envelope::SurfaceElement::Floor;
        survey.envelope.surfaces[0].exposed_perimeter_m = Some(40.0);
        let mut recorder = Recorder::default();
        let input = derive_utility_input(&survey, &mut recorder).unwrap();
        let system = &input["spaceHeating"]["distributionSystem"];
        assert!(system.get("unheatedPipeLengthM").is_none());
        assert!(applied(&recorder, "unheated_pipes_unknown_present"));
        // A measured length is passed on.
        survey.heating.unheated_pipes = Some(super::super::heating::UnheatedPipesAnswer::Present {
            length_m: Some(12.0),
        });
        let mut recorder = Recorder::default();
        let input = derive_utility_input(&survey, &mut recorder).unwrap();
        assert_eq!(
            input["spaceHeating"]["distributionSystem"]["unheatedPipeLengthM"],
            12.0
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
            drive: Default::default(),
            groundwater_system: None,
            collective_source_reference: None,
            source_temperature_c: None,
            source_temperature_reference: None,
            source_quality_declaration_reference: None,
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

    #[test]
    fn gas_heat_pump_uses_the_utility_gwp_row() {
        let mut survey = fixture("2005");
        survey.heating.generator = super::super::heating::HeatingGenerator::HeatPump {
            source: super::super::heating::HeatPumpSource::OutdoorAir,
            air_sink: false,
            high_temperature: false,
            capacity_kw: None,
            source_regeneration_factor: None,
            high_efficiency_evidence: None,
            drive: Some(super::super::heating::HeatPumpDrive::GasEngine),
            groundwater_system: None,
            collective_source_reference: None,
            source_temperature_c: None,
            source_temperature_reference: None,
            source_quality_declaration_reference: None,
        };
        let (input, _) = derive(&survey);
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(generator["kind"], "gas_heat_pump");
        assert_eq!(generator["table"], "utility_collective_or_above25_kw");
        assert_eq!(generator["auxiliary"]["nominalPowerKw"], 200.0);
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|p| &p.issues)
        );
        // Table 9.29 GWP ground/outdoor air row: 1,4 at 55 °C … 1,65 ≤ 30 °C.
        let efficiency = result
            .performance
            .unwrap()
            .space_heating
            .generation_efficiency
            .unwrap();
        assert!([1.65, 1.6, 1.55, 1.5, 1.45, 1.4]
            .iter()
            .any(|value| (efficiency - value).abs() < 1e-9));
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
            heat_pump_source: None,
            ground_above_zero_demonstrated: false,
            gas_engine: None,
            direct_expansion: None,
            fittings_insulated: None,
            cold_meters: None,
            pipe_length_m: None,
            uncooled_pipe_length_m: None,
            additional_generators: Vec::new(),
            source_reference: "survey".into(),
        }
    }

    #[test]
    fn cooling_survey_book_balancing_fans_and_heat_pump_source() {
        // Table 10.34: an aquifer before 2013 is EER 14 for dwellings.
        let mut recorder = Recorder::default();
        let mut aquifer = cooling(CoolingEmitterAnswer::CeilingCooling);
        aquifer.generator = CoolingGeneratorAnswer::AquiferBefore2013;
        let value = cooling_value(
            &aquifer,
            2010,
            1,
            CoolingBook::Residential,
            true,
            &mut recorder,
        );
        assert_eq!(
            value["generators"][0]["generator"]["source"],
            "aquifer_dwellings_before2013"
        );
        let value = cooling_value(
            &aquifer,
            2010,
            1,
            CoolingBook::Utility,
            false,
            &mut recorder,
        );
        assert_eq!(
            value["generators"][0]["generator"]["source"],
            "aquifer_utility_before2013"
        );
        // Table 10.6: dynamic balancing, and the old boolean still works.
        aquifer.balanced = Some(CoolingBalanceAnswer::Kind(CoolingBalanceKind::Dynamic));
        let value = cooling_value(
            &aquifer,
            2010,
            1,
            CoolingBook::Utility,
            false,
            &mut recorder,
        );
        assert_eq!(value["emission"]["balancing"], "dynamic");
        let old: SurveyCooling = serde_json::from_value(json!({
            "generator": "compression", "emitter": "ceiling_cooling", "waterBased": true,
            "balanced": true, "sourceReference": "survey"
        }))
        .unwrap();
        let value = cooling_value(&old, 2010, 1, CoolingBook::Utility, false, &mut recorder);
        assert_eq!(value["emission"]["balancing"], "static");
        // 10.84: the heating heat pump on a ground source sets the flag.
        let mut value = cooling_value(
            &aquifer,
            2010,
            1,
            CoolingBook::Residential,
            true,
            &mut recorder,
        );
        let heating = json!({"kind": "heat_pump_forfait", "forfait": {"source": "ground"}});
        apply_cooling_heat_pump_source(
            &mut value,
            &aquifer,
            &heating,
            CoolingBook::Residential,
            &mut recorder,
        );
        assert_eq!(value["generators"][0]["generator"]["heatPumpSource"], true);
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "cooling_heat_pump_source_from_heating"));
        // ISSO p. 136: fan convectors need their number.
        let mut fans = Recorder::default();
        cooling_value(
            &cooling(CoolingEmitterAnswer::FanCoilOnCeiling),
            2010,
            1,
            CoolingBook::Utility,
            false,
            &mut fans,
        );
        assert!(fans
            .issues
            .iter()
            .any(|item| item.code == "cooling_fan_coil_count_required"));
    }

    #[test]
    fn cooling_unknowns_follow_chapter_10() {
        let mut recorder = Recorder::default();
        let fan_coils = cooling_value(
            &cooling(CoolingEmitterAnswer::FanCoilOnCeiling),
            1990,
            3,
            CoolingBook::Utility,
            false,
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
            "cooling_fittings_unknown_uninsulated",
            "cooling_meters_unknown_present",
            "cooling_pipe_length_unknown_forfait",
        ] {
            assert!(applied(&recorder, rule), "{rule}");
        }
        let core = cooling_value(
            &cooling(CoolingEmitterAnswer::ConcreteCoreActivation),
            1990,
            3,
            CoolingBook::Utility,
            false,
            &mut recorder,
        );
        assert_eq!(core["emission"]["emitter"], "floor_cooling");
        assert_eq!(core["distribution"]["designTemperature"], "t17_to21");
        let split = cooling_value(
            &cooling(CoolingEmitterAnswer::SplitIndoorUnitsOnWall),
            1990,
            3,
            CoolingBook::Utility,
            false,
            &mut recorder,
        );
        assert_eq!(
            split["emission"]["emitter"],
            "fan_coil_or_rac_on_outer_wall"
        );
        let mut insulated = cooling(CoolingEmitterAnswer::CeilingCooling);
        insulated.pipes_insulated = Some(true);
        let value = cooling_value(
            &insulated,
            1988,
            1,
            CoolingBook::Utility,
            false,
            &mut recorder,
        );
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
        let value = cooling_value(
            &aquifer,
            2010,
            1,
            CoolingBook::Utility,
            false,
            &mut recorder,
        );
        assert_eq!(
            value["generators"][0]["generator"]["source"],
            "aquifer_utility_before2013"
        );
        aquifer.aquifer_permit_year = Some(2014);
        let value = cooling_value(
            &aquifer,
            2010,
            1,
            CoolingBook::Utility,
            false,
            &mut recorder,
        );
        assert_eq!(
            value["generators"][0]["generator"]["source"],
            "aquifer_from2013"
        );
        let mut direct = cooling(CoolingEmitterAnswer::FanCoilOnOuterWall);
        direct.water_based = false;
        let value = cooling_value(&direct, 2010, 1, CoolingBook::Utility, false, &mut recorder);
        assert!(value.get("distribution").is_none());
        assert_eq!(value["emission"]["balancing"], "not_applicable");
    }

    #[test]
    fn recirculation_and_flow_control_follow_tables_11_7_and_11_8() {
        let (input, recorder) = derive(&fixture("1985"));
        let ventilation = &input["spaceHeating"]["demand"]["ventilation"];
        // NTA 11.60: x = 20 when the percentage is unknown.
        assert_eq!(ventilation["flowReduction"]["recirculationPercent"], 20);
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
        assert_eq!(
            reduction["evidenceReference"],
            json!(survey.ventilation.source_reference)
        );
        // 11.60/11.61: only proven higher x (recirculation) or lower x
        // (flow control) than the defaults 20 and 80 count.
        survey.ventilation.recirculation_percent = Some(15);
        survey.ventilation.flow_control = Some(SurveyFlowControl {
            method: FlowControlMethodAnswer::SpeedControl,
            minimum_percent: Some(95),
        });
        let (input, _) = derive(&survey);
        let reduction = &input["spaceHeating"]["demand"]["ventilation"]["flowReduction"];
        assert_eq!(reduction["recirculationPercent"], 20);
        assert_eq!(reduction["flowControlPercent"], 80);
        assert!(reduction.get("evidenceReference").is_none());
    }

    #[test]
    fn ahu_heating_and_cooling_become_coils() {
        let (input, recorder) = derive(&fixture("1970"));
        let ahu =
            &input["spaceHeating"]["demand"]["ventilation"]["system"]["unit"]["airHandlingUnit"];
        assert_eq!(ahu["heatingCoil"], false);
        assert_eq!(ahu["coolingCoil"], false);
        assert!(applied(&recorder, "ahu_heating_unknown_none"));
        let mut survey = fixture("1970");
        let unit = survey.ventilation.ahu.as_mut().unwrap();
        unit.heating_connected = Some(true);
        unit.cooling_connected = Some(true);
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let chain = &result.performance.as_ref().unwrap().space_heating;
        assert!(chain.monthly[0].ahu_heating_load_kwh > 0.0);
        // Cooling connected without a cooling system is rejected.
        let mut dry = survey.clone();
        dry.cooling = None;
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&dry, &mut recorder).is_none());
        assert!(recorder
            .issues
            .iter()
            .any(|item| item.code == "ahu_cooling_requires_cooling_system"));
        // Air heating via the AHU (ISSO 82.1 table 9.16b) implies the coil.
        let mut air = fixture("1970");
        air.heating.emitters = crate::opname::heating::Emitters::AirHeating;
        air.heating.air_heating = Some(AirHeatingAnswer::ViaAirHandlingUnit);
        let (input, recorder) = derive(&air);
        let ahu =
            &input["spaceHeating"]["demand"]["ventilation"]["system"]["unit"]["airHandlingUnit"];
        assert_eq!(ahu["heatingCoil"], true);
        assert!(applied(&recorder, "ahu_heating_from_air_heating"));
        air.ventilation.ahu.as_mut().unwrap().heating_connected = Some(false);
        let mut recorder = Recorder::default();
        derive_utility_input(&air, &mut recorder);
        assert!(recorder
            .issues
            .iter()
            .any(|item| item.code == "air_heating_via_ahu_requires_heating_coil"));
    }

    #[test]
    fn passive_cooling_maps_to_the_kernel_with_evidence() {
        let mut survey = fixture("1970");
        survey.ventilation.bypass_present = Some(true);
        survey.ventilation.passive_cooling = Some(SurveyPassiveCooling {
            evidence_reference: "supplier project document".into(),
            installed_capacity_dm3_per_s: None,
        });
        let (input, recorder) = derive(&survey);
        let ventilation = &input["spaceHeating"]["demand"]["ventilation"];
        assert_eq!(
            ventilation["maximumCapacityForCooling"],
            "supplier project document"
        );
        assert!(applied(
            &recorder,
            "passive_cooling_capacity_unknown_regulatory"
        ));
        let result = assess_utility_survey(&survey);
        assert!(result.issues.is_empty(), "{:?}", result.issues);
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
            heating_connected: None,
            cooling_connected: None,
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
    fn humidification_runs_in_the_heating_chain() {
        let result = assess_utility_survey(&fixture("1970"));
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let input = result.derived_input.as_ref().unwrap();
        assert_eq!(input.space_heating.humidifiers.len(), 1);
        // 12.3: steam energy on its own carrier in the chain, winter only.
        let chain = &result.performance.as_ref().unwrap().space_heating;
        let steam = |month: usize| {
            chain.monthly[month].humidification_electricity_kwh
                + chain.monthly[month].humidification_fuel_kwh
        };
        assert!(steam(0) > 0.0);
        assert_eq!(steam(6), 0.0);
        // 12.1/9.4: an adiabatic humidifier loads the heating node.
        let mut survey = fixture("1970");
        survey.humidification.as_mut().unwrap().humidifier = HumidifierAnswer::Adiabatic;
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert!(
            result.performance.as_ref().unwrap().space_heating.monthly[0].humidification_load_kwh
                > 0.0
        );
        assert!(!result
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
        // derive_utility_input cites ISSO 75.1 on its own as well.
        let (_, recorder) = derive(&fixture("1970"));
        assert!(recorder
            .applied
            .iter()
            .all(|item| !item.source.contains("82.1")));
    }

    #[test]
    fn bacs_uses_the_surveyed_generator_powers() {
        // Table 7.3 (p. 62–63): a 350 kW boiler without BACS data is a
        // system above 290 kW without BACS, whatever the served area.
        let mut recorder = Recorder::default();
        let (factor, _) = bacs_factor(
            &SurveyBacs::default(),
            &[Some(350.0)],
            1300.0,
            &mut recorder,
        );
        assert_eq!(factor, 1.05);
        assert!(applied(&recorder, "bacs_power_from_surveyed_systems"));
        // All systems known and at most 290 kW: 1,0 even above 2 500 m².
        let mut recorder = Recorder::default();
        let small = [Some(200.0), Some(150.0)];
        assert_eq!(
            bacs_factor(&SurveyBacs::default(), &small, 4000.0, &mut recorder).0,
            1.0
        );
        // One power unknown: the served A_g decides (p. 62).
        let mut recorder = Recorder::default();
        let partial = [Some(200.0), None];
        assert_eq!(
            bacs_factor(&SurveyBacs::default(), &partial, 4000.0, &mut recorder).0,
            1.05
        );
        assert!(applied(&recorder, "bacs_power_unknown_served_area"));
        // In the survey: the installation capacity, else the summed
        // generator powers, and the cooling capacities.
        let mut survey = fixture("1985");
        survey.bacs = SurveyBacs::default();
        survey.heating_installation.capacity_kw = Some(350.0);
        let (input, _) = derive(&survey);
        assert_eq!(input["bacsFactor"], 1.05);
        survey.heating_installation.capacity_kw = None;
        survey.heating.nominal_power_kw = Some(120.0);
        assert_eq!(surveyed_system_powers(&survey), vec![Some(120.0)]);
        let mut chiller = cooling(CoolingEmitterAnswer::FanCoilOnCeiling);
        chiller.fan_coil_count = 10;
        chiller.capacity_kw = Some(200.0);
        chiller.additional_generators.push(SurveyCoolingGenerator {
            generator: CoolingGeneratorAnswer::AquiferFrom2013,
            capacity_kw: Some(150.0),
            aquifer_permit_year: None,
            gas_engine: None,
        });
        survey.cooling = Some(chiller);
        assert_eq!(
            surveyed_system_powers(&survey),
            vec![Some(120.0), Some(350.0)]
        );
    }

    #[test]
    fn calculation_zone_split_follows_afb_6_6() {
        let split =
            |functions: Vec<(LabelFunction, f64)>, principle: VentilationPrinciple, open: bool| {
                let mut survey = fixture("2005");
                survey.functions = functions
                    .into_iter()
                    .map(|(function, area_m2)| FunctionArea { function, area_m2 })
                    .collect();
                survey.ventilation.principle = principle;
                survey.openly_connected_residence_areas = open;
                let mut recorder = Recorder::default();
                let groups = function_groups(&survey.functions, &mut recorder).unwrap();
                check_zone_split(&groups, &survey, &mut recorder);
                recorder
                    .issues
                    .iter()
                    .any(|item| item.code == "calculation_zone_split_required")
            };
        // Table 6.4: education 21 °C, sport 16 °C (5 K), largest 58 %.
        let mixed = vec![
            (LabelFunction::Education, 1400.0),
            (LabelFunction::Sport, 1000.0),
        ];
        assert!(split(mixed.clone(), VentilationPrinciple::Balanced, true));
        // A sport function within 25 % is merged first (p. 39–40), so the
        // criteria do not see it.
        let merged = vec![
            (LabelFunction::Education, 2300.0),
            (LabelFunction::Sport, 250.0),
        ];
        assert!(!split(
            merged,
            VentilationPrinciple::MechanicalExtract,
            false
        ));
        // Equal setpoints (healthcare beds 22, office 21: 1 K), capacities
        // 2,04/1,11 within 4 ×: no split.
        let care = vec![
            (LabelFunction::HealthcareWithBeds, 1000.0),
            (LabelFunction::Office, 800.0),
        ];
        assert!(!split(care, VentilationPrinciple::MechanicalExtract, false));
        // Type C with capacities 3,64/0,46 but openly connected areas: the
        // setpoint criterion still splits (16 vs 21 °C).
        let gym = vec![
            (LabelFunction::Education, 1000.0),
            (LabelFunction::Sport, 400.0),
        ];
        assert!(split(gym, VentilationPrinciple::MechanicalExtract, true));
        // Same setpoint, capacities 3,64 and 0,28 (13 ×): split under type
        // C, not under type D or with more than 80 % one requirement.
        let school_shop = vec![
            (LabelFunction::Education, 1000.0),
            (LabelFunction::Retail, 1000.0),
        ];
        assert!(split(
            school_shop.clone(),
            VentilationPrinciple::MechanicalExtract,
            false
        ));
        assert!(!split(school_shop, VentilationPrinciple::Balanced, false));
        let mostly_school = vec![
            (LabelFunction::Education, 1700.0),
            (LabelFunction::Retail, 400.0),
        ];
        // 400 m² > 25 % of 2 100? No: 19 %, merged into education.
        assert!(!split(
            mostly_school,
            VentilationPrinciple::MechanicalExtract,
            false
        ));
        // Office 1,11 and retail 0,28 (3,96 ×): no split.
        let office_shop = vec![
            (LabelFunction::Retail, 1800.0),
            (LabelFunction::Office, 1200.0),
        ];
        assert!(!split(
            office_shop,
            VentilationPrinciple::MechanicalExtract,
            false
        ));
        // The survey stops with the issue.
        let mut survey = fixture("2005");
        survey.functions = vec![
            FunctionArea {
                function: LabelFunction::Education,
                area_m2: 1400.0,
            },
            FunctionArea {
                function: LabelFunction::Sport,
                area_m2: 1000.0,
            },
        ];
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&survey, &mut recorder).is_none());
        assert_eq!(recorder.issues[0].code, "calculation_zone_split_required");
    }

    #[test]
    fn cooling_options_of_chapter_10() {
        // Table 10.6 with NTA table 10.11: dynamic balancing is balanced.
        let mut recorder = Recorder::default();
        let mut dynamic = cooling(CoolingEmitterAnswer::CeilingCooling);
        dynamic.balanced = Some(CoolingBalanceAnswer::Kind(CoolingBalanceKind::Dynamic));
        dynamic.fittings_insulated = Some(true);
        dynamic.cold_meters = Some(false);
        dynamic.pipe_length_m = Some(400.0);
        dynamic.uncooled_pipe_length_m = Some(30.0);
        let value = cooling_value(
            &dynamic,
            2000,
            3,
            CoolingBook::Utility,
            false,
            &mut recorder,
        );
        let distribution = &value["distribution"];
        assert_eq!(distribution["pump"]["hydraulicallyBalanced"], true);
        assert_eq!(distribution["pump"]["heatMeter"], false);
        assert_eq!(distribution["fittingsInsulated"], true);
        assert_eq!(distribution["pipeLengthM"], 400.0);
        assert_eq!(distribution["unconditionedPipeLengthM"], 30.0);
        assert!(!applied(&recorder, "cooling_pipe_length_unknown_forfait"));
        let mut none = dynamic.clone();
        none.balanced = Some(CoolingBalanceAnswer::Kind(CoolingBalanceKind::None));
        let value = cooling_value(&none, 2000, 3, CoolingBook::Utility, false, &mut recorder);
        assert_eq!(
            value["distribution"]["pump"]["hydraulicallyBalanced"],
            false
        );
        // Table 10.2: gas-engine chiller, year unknown → up to 2006.
        let mut gas = cooling(CoolingEmitterAnswer::CeilingCooling);
        gas.generator = CoolingGeneratorAnswer::GasEngineCompression;
        gas.gas_engine = Some(SurveyGasEngine {
            from_2007: None,
            electric_power_kw: Some(50.0),
            hre_declared: false,
        });
        let mut recorder = Recorder::default();
        let value = cooling_value(&gas, 2000, 3, CoolingBook::Utility, false, &mut recorder);
        let generator = &value["generators"][0]["generator"];
        assert_eq!(generator["kind"], "gas_engine_compression");
        assert_eq!(generator["gasEngine"]["powerKw"], 50.0);
        assert_eq!(generator["gasEngine"]["builtAfter2006"], false);
        assert!(applied(&recorder, "gas_engine_year_unknown_up_to_2006"));
        assert!(recorder.issues.is_empty());
        gas.gas_engine = None;
        let mut recorder = Recorder::default();
        cooling_value(&gas, 2000, 3, CoolingBook::Utility, false, &mut recorder);
        assert_eq!(recorder.issues[0].code, "gas_engine_power_required");
        // §10.3.2: further generators, each with its power.
        let mut two = cooling(CoolingEmitterAnswer::CeilingCooling);
        two.capacity_kw = Some(300.0);
        two.additional_generators.push(SurveyCoolingGenerator {
            generator: CoolingGeneratorAnswer::AquiferYearUnknown,
            capacity_kw: None,
            aquifer_permit_year: Some(2015),
            gas_engine: None,
        });
        let mut recorder = Recorder::default();
        let value = cooling_value(&two, 2000, 3, CoolingBook::Utility, false, &mut recorder);
        let generators = value["generators"].as_array().unwrap();
        assert_eq!(generators.len(), 2);
        assert_eq!(generators[1]["id"], "koeling-2");
        assert_eq!(generators[1]["generator"]["source"], "aquifer_from2013");
        assert_eq!(
            recorder.issues[0].code,
            "cooling_generator_capacity_required"
        );
        assert_eq!(
            recorder.issues[0].path,
            "cooling.additionalGenerators[0].capacityKw"
        );
        // §10.4.1: direct expansion only without water distribution.
        let mut dx = cooling(CoolingEmitterAnswer::Other);
        dx.direct_expansion = Some(DirectExpansionAnswer::AirHandlingUnit);
        let mut recorder = Recorder::default();
        cooling_value(&dx, 2000, 3, CoolingBook::Utility, false, &mut recorder);
        assert_eq!(
            recorder.issues[0].code,
            "cooling_direct_expansion_not_water_based"
        );
        dx.water_based = false;
        let mut recorder = Recorder::default();
        let value = cooling_value(&dx, 2000, 3, CoolingBook::Utility, false, &mut recorder);
        assert!(value.get("distribution").is_none());
        assert!(applied(&recorder, "cooling_direct_expansion_in_ahu"));
    }

    #[test]
    fn direct_expansion_in_the_ahu_needs_its_cooling_coil() {
        let mut survey = fixture("1985");
        let mut dx = cooling(CoolingEmitterAnswer::Other);
        dx.water_based = false;
        dx.direct_expansion = Some(DirectExpansionAnswer::AirHandlingUnit);
        survey.cooling = Some(dx);
        survey.ventilation.ahu = None;
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&survey, &mut recorder).is_none());
        assert!(recorder
            .issues
            .iter()
            .any(|item| item.code == "cooling_direct_expansion_ahu_requires_ahu"));
        survey.ventilation.ahu = Some(SurveyAhu {
            inside_thermal_zone: Some(false),
            ducts_outside_thermal_zone: Some(false),
            duct_length: None,
            ducts_insulated: None,
            heating_connected: Some(false),
            cooling_connected: None,
        });
        let (input, recorder) = derive(&survey);
        assert_eq!(
            input["spaceHeating"]["demand"]["ventilation"]["system"]["unit"]["airHandlingUnit"]
                ["coolingCoil"],
            true
        );
        assert!(applied(&recorder, "ahu_cooling_from_direct_expansion"));
    }

    #[test]
    fn utility_ventilation_options_of_chapter_11() {
        let mut survey = fixture("1985");
        let vent = &mut survey.ventilation;
        vent.principle = VentilationPrinciple::Balanced;
        vent.heat_recovery = Some(ExchangerAnswer::ColdStorageWithAhu);
        vent.duct_airtightness = Some(DuctAirtightnessAnswer::LukaD);
        vent.heat_recovery_layout = Some(RecoveryLayout::Decentral);
        vent.supply_duct_insulation = Some(SupplyDuctInsulationAnswer::Specified {
            thickness_m: 0.03,
            conductivity_w_per_mk: 0.035,
        });
        vent.supply_duct_length_m = Some(2.0);
        vent.constant_volume_control = Some(true);
        vent.bypass_percent = Some(45);
        vent.installed_capacity_dm3_per_s = Some(1500.0);
        vent.grille_heating_strips = Some(SurveyGrilleHeatingStrips {
            max_power_w_per_dm3_per_s: None,
            max_temperature_rise_k: Some(8.0),
            switch_on_below_c: None,
            max_supply_temperature_c: None,
            source_reference: "datasheet".into(),
        });
        let (input, recorder) = derive(&survey);
        let ventilation = &input["spaceHeating"]["demand"]["ventilation"];
        let unit = &ventilation["system"]["unit"];
        assert_eq!(unit["ducts"], "luka_d");
        let recovery = &unit["heatRecovery"];
        assert_eq!(recovery["efficiency"]["exchanger"], "run_around_coil_ahu");
        assert_eq!(recovery["layout"], "decentral");
        assert_eq!(recovery["constantVolumeControl"], true);
        assert_eq!(recovery["supplyDuctInsulation"]["kind"], "specified");
        assert_eq!(recovery["supplyDuctLengthM"], 2.0);
        // Table 11.12: 45 % rounds down to 40 %.
        assert_eq!(recovery["bypass"]["kind"], "partial");
        assert_eq!(recovery["bypass"]["fraction"], 0.4);
        assert_eq!(ventilation["installedCapacity"]["totalDm3PerS"], 1500.0);
        assert_eq!(
            ventilation["grillePreheating"]["control"]["method"],
            "fallback"
        );
        assert!(applied(&recorder, "grille_heating_strip_settings_unknown"));
        // System E: decentral D.5b with heat recovery, the rest type C.
        let mut survey = fixture("1985");
        survey.ventilation.principle = VentilationPrinciple::MechanicalExtract;
        survey.ventilation.ahu = None;
        survey.ventilation.heat_recovery = Some(ExchangerAnswer::CounterFlowPlastic);
        survey.ventilation.combined = Some(SurveyCombined {
            decentral_area_m2: 300.0,
            total_residence_area_m2: 1000.0,
        });
        let (input, recorder) = derive(&survey);
        let system = &input["spaceHeating"]["demand"]["ventilation"]["system"];
        assert_eq!(system["kind"], "combined");
        assert_eq!(system["decentral"]["variant"], "d5b");
        assert_eq!(system["decentral"]["heatRecovery"]["layout"], "decentral");
        assert!(system["other"]["variant"]
            .as_str()
            .unwrap()
            .starts_with('c'));
        assert!(applied(&recorder, "combined_system_e1"));
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        // The bypass percentage and capacity are checked.
        let mut survey = fixture("1985");
        survey.ventilation.bypass_percent = Some(120);
        let mut recorder = Recorder::default();
        derive_utility_input(&survey, &mut recorder);
        assert!(recorder
            .issues
            .iter()
            .any(|item| item.code == "bypass_percent_invalid"));
    }

    #[test]
    fn utility_distribution_reads_the_table_9_12_answers() {
        // The coordinator's review: the utility fallback distribution used
        // to ignore the pipe-insulation and one-pipe answers.
        let mut survey = fixture("1985");
        survey.heating.pipe_insulation = Some(super::super::heating::PipeInsulationAnswer {
            insulated: true,
            insulation_year: Some(1990),
            fittings_insulated: Some(true),
        });
        survey.heating.distribution_type =
            Some(super::super::heating::DistributionTypeAnswer::OnePipe { emitter_count: 12 });
        let (input, recorder) = derive(&survey);
        let system = &input["spaceHeating"]["distributionSystem"];
        assert_eq!(
            system["pipeTransmittance"]["insulation"]["state"],
            "insulated"
        );
        assert_eq!(
            system["pipeTransmittance"]["insulation"]["period"],
            "from1980_to1995"
        );
        assert_eq!(system["valvesInsulated"], true);
        assert_eq!(system["pump"]["onePipeEmitterCount"], 12);
        assert!(!applied(&recorder, "pipe_insulation_unknown_uninsulated"));
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
    }

    #[test]
    fn fossil_fuel_sport_halls_and_swimming_pool() {
        let mut survey = fixture("2005");
        survey.fossil_fuel_on_plot = Some(true);
        let (input, _) = derive(&survey);
        assert_eq!(input["fossilAppliancesOutsideCalculation"], true);
        let (input, recorder) = derive(&fixture("2005"));
        assert!(input.get("fossilAppliancesOutsideCalculation").is_none());
        assert!(applied(&recorder, "fossil_fuel_on_plot_not_established"));
        // p. 65: sport and swimming halls in a building of ≥ 1 000 m², and
        // the pool room ventilated as sport × 2 (NTA §11.2.2.5.1).
        // A sport function merged into education (20 %): the hall area
        // still shortens the loop; the pool factor lapses (p. 54).
        let mut survey = fixture("2005");
        survey.functions = vec![
            FunctionArea {
                function: LabelFunction::Education,
                area_m2: 1900.0,
            },
            FunctionArea {
                function: LabelFunction::Sport,
                area_m2: 500.0,
            },
        ];
        survey.hot_water.circulation = Some(true);
        survey.sport_hall_area_m2 = Some(450.0);
        survey.swimming_pool_area_m2 = Some(200.0);
        let (input, recorder) = derive(&survey);
        assert_eq!(input["hotWater"]["circulation"]["sportHallAreaM2"], 450.0);
        assert!(applied(&recorder, "swimming_pool_in_merged_sport_function"));
        // A separate sport function (an own zone needs no split here: the
        // pool hall is surveyed as a sports building).
        let mut survey = fixture("2005");
        survey.functions = vec![FunctionArea {
            function: LabelFunction::Sport,
            area_m2: 2400.0,
        }];
        survey.hot_water.circulation = Some(true);
        survey.sport_hall_area_m2 = Some(450.0);
        survey.swimming_pool_area_m2 = Some(200.0);
        let (input, _) = derive(&survey);
        assert_eq!(input["hotWater"]["circulation"]["sportHallAreaM2"], 450.0);
        let functions = input["spaceHeating"]["demand"]["ventilation"]["functions"]
            .as_array()
            .unwrap();
        let pool = functions
            .iter()
            .find(|item| item["swimmingPool"] == true)
            .unwrap();
        assert_eq!(pool["areaM2"], 200.0);
        let sport = functions
            .iter()
            .find(|item| item["function"] == "sport" && item.get("swimmingPool").is_none())
            .unwrap();
        assert_eq!(sport["areaM2"], 2200.0);
        let result = assess_utility_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        survey.swimming_pool_area_m2 = Some(2600.0);
        let mut recorder = Recorder::default();
        assert!(derive_utility_input(&survey, &mut recorder).is_none());
        assert!(recorder
            .issues
            .iter()
            .any(|item| item.code == "swimming_pool_area_invalid"));
    }
}
