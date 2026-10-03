//! ISSO 82.1 (7e druk) chapter 9: space heating in the basic survey.
//!
//! - No generator present during a renovation: the previous one, else a
//!   conventional boiler (p. 107).
//! - Pilot flame unknown: with pilot flame (table 9.3, p. 108); a hydrogen
//!   boiler counts as HR-107 on natural gas (p. 108).
//! - Heat-pump water source unknown: ground; individual surface water:
//!   ground (table 9.6, p. 110–111).
//! - Design temperature class unknown: table 9.9 (p. 114, erratum §4).
//! - Several emitters: surface heating before radiators/convectors before
//!   air heating (p. 123–124).
//! - Hydronic balancing unknown: not balanced (table 9.10, p. 116).
//! - Emission control unknown: other/unknown (table 9.17, p. 124).
//! - Pipe insulation unknown: not insulated (table 9.12, p. 118); the heat
//!   meter of table 9.16a (p. 122) applies to collective installations
//!   only, so individual systems have none; pumps unknown: forfait
//!   (table 9.11, p. 117).
//! - Ground or groundwater source without solar regeneration: c_source
//!   1,0 (p. 110; NTA table 9.27 footnote a).
//! - Exhaust-air heat pump: a second generator is required (WD 2025
//!   p. 43); without `additionalGenerators` it is rejected.
//! - Several generators (§9.3.2, p. 112–113): every unequal generator is
//!   entered with its nominal power (table 9.7); the preference follows the
//!   p. 112 order (exhaust-air heat pumps, other heat pumps, CHP, biomass,
//!   external heat, electric, gas/oil boilers). External heat of unknown
//!   power takes 50 % (table 9.7); an added preferred generator (§9.3.6)
//!   uses NTA 9.58/9.59.
//! - Collective installations (p. 106, 121–122): the connected usable area
//!   unknown → dwellings × the dwelling's area (p. 121); heat meters
//!   unknown → present (table 9.16, p. 122); pipes forfait (table 9.14).
//! - Biomass annex R compliance unknown: not compliant (p. 111–112, 28).
//! - Oil boilers, local gas/oil heating, steam boilers and gas air heaters
//!   (table 9.3, p. 108) follow NTA table 9.25; pilot flame unknown →
//!   present; electricity connection unknown → present.
//! - Heat-pump sources (table 9.6, p. 110–111): see `convert_heat_pump`.
//! - Pipe insulation (table 9.12, p. 118): year unknown → construction
//!   year, fittings unknown → not insulated; one-pipe loops (p. 115) add
//!   the table 9.21 resistance per emitter.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::Recorder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoilerType {
    Conventional,
    Vr,
    Hr100,
    Hr104,
    Hr107,
    /// Boiler on hydrogen: HR-107 on natural gas (p. 108).
    Hydrogen,
    /// Oil-fired central boiler (table 9.3, p. 108): conventional, the NTA
    /// table 9.25 definition of an oil boiler.
    Oil,
}

/// Table 9.3 (p. 108): local gas heating incl. pilot flame, oil heating
/// or a steam boiler; NTA table 9.25 "overige systemen".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalFiredAppliance {
    GasHeater,
    OilHeater,
    SteamBoiler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FiredFuel {
    NaturalGas,
    Oil,
}

/// Table 9.3 type of a direct-fired gas air heater (NTA table 9.25).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AirHeaterType {
    Conventional,
    Vr,
    Hr100,
    Hr104,
    Hr107,
}

/// §9.3.1.3 (p. 109): how the heat pump is driven.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatPumpDrive {
    #[default]
    Electric,
    GasEngine,
    GasAbsorption,
}

/// Table 9.6 (p. 110): groundwater source system; unknown → recirculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundwaterSystem {
    Doublet,
    Recirculation,
}

/// §9.4.2 (p. 115, WD 2025 p. 50): water-based distribution system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DistributionTypeAnswer {
    /// Including a Tichelmann system (p. 116).
    TwoPipe,
    OnePipe {
        #[serde(rename = "emitterCount")]
        emitter_count: u32,
    },
    RenovatedOnePipe,
}

/// Table 9.12 (p. 118): insulation of the distribution pipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PipeInsulationAnswer {
    pub insulated: bool,
    /// Year of insulation; unknown → the construction year.
    #[serde(default)]
    pub insulation_year: Option<i32>,
    /// Valves and brackets; unknown → not insulated.
    #[serde(default)]
    pub fittings_insulated: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatPumpSource {
    OutdoorAir,
    ExhaustAir,
    OutdoorAndExhaustAir,
    Ground,
    Groundwater,
    /// Individual systems: counts as ground (p. 111).
    SurfaceWater,
    /// Water-based source of unknown kind: ground (table 9.6).
    WaterBasedUnknown,
    /// Heat-pump panel (table 9.6): the outdoor-air row (NTA p. 336,
    /// note 3 below table 9.28).
    HeatPumpPanel,
    /// Collective high-temperature source (table 9.6, p. 110–111).
    HighTemperature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomassApplianceAnswer {
    FreestandingWoodStove,
    InsertStove,
    PelletStove,
    AccumulatingStove,
    CentralBoiler,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HeatingGenerator {
    Boiler {
        #[serde(rename = "boilerType")]
        boiler_type: BoilerType,
        #[serde(default, rename = "pilotFlame")]
        pilot_flame: Option<bool>,
        #[serde(rename = "insideThermalBoundary")]
        inside_thermal_boundary: bool,
        #[serde(default, rename = "manufactureYear")]
        manufacture_year: Option<i32>,
        #[serde(default, rename = "installationYear")]
        installation_year: Option<i32>,
    },
    HeatPump {
        source: HeatPumpSource,
        /// Indoor air as the sink (air/air heat pump).
        #[serde(default, rename = "airSink")]
        air_sink: bool,
        /// Suitable for high temperatures (table 9.9).
        #[serde(default, rename = "highTemperature")]
        high_temperature: bool,
        #[serde(default, rename = "capacityKw")]
        capacity_kw: Option<f64>,
        /// Annex V c_source of a ground source regenerated by a solar
        /// system (ISSO 82.1 p. 110); `None`: no regeneration, 1,0.
        #[serde(default, rename = "sourceRegenerationFactor")]
        source_regeneration_factor: Option<f64>,
        /// Product data showing the table 9.5 (NTA table 9.28) COP values
        /// (ISSO 82.1 p. 109); passed to the kernel as
        /// `highEfficiencyEvidence`.
        #[serde(default, rename = "highEfficiencyEvidence")]
        high_efficiency_evidence: Option<Value>,
        /// Electric (`None`) or gas driven (p. 109).
        #[serde(default)]
        drive: Option<HeatPumpDrive>,
        /// Table 9.6: doublet or recirculation; `None` unknown →
        /// recirculation.
        #[serde(default, rename = "groundwaterSystem")]
        groundwater_system: Option<GroundwaterSystem>,
        /// Invoices or design data showing a collective water-based source
        /// (p. 111); `None`: an individual source.
        #[serde(default, rename = "collectiveSourceReference")]
        collective_source_reference: Option<String>,
        /// Source temperature from design data or winter measurements
        /// (p. 111), °C; classifies groundwater and high-temperature rows.
        #[serde(default, rename = "sourceTemperatureC")]
        source_temperature_c: Option<f64>,
        #[serde(default, rename = "sourceTemperatureReference")]
        source_temperature_reference: Option<String>,
        /// Quality declaration of a source of 20 °C or more (WD 2025 p. 45).
        #[serde(default, rename = "sourceQualityDeclarationReference")]
        source_quality_declaration_reference: Option<String>,
    },
    DistrictHeat,
    Electric {
        #[serde(rename = "connectedDevices")]
        connected_devices: u32,
    },
    Biomass {
        appliance: BiomassApplianceAnswer,
        #[serde(rename = "insideThermalBoundary")]
        inside_thermal_boundary: bool,
        #[serde(rename = "soleHeatingInServedRooms")]
        sole_heating_in_served_rooms: bool,
        /// Annex R compliance (ISSO 82.1 p. 111–112, "indien bekend");
        /// `None`: not compliant (conservative, p. 28).
        #[serde(default, rename = "annexRCompliant")]
        annex_r_compliant: Option<bool>,
    },
    /// Building CHP (NTA 9.6.6.1, table 9.31; ISSO table 9.7).
    Chp {
        /// Electrical power P_el, kW (table 9.31 row).
        #[serde(rename = "electricalPowerKw")]
        electrical_power_kw: f64,
        /// Thermal power, kW; unknown → table 9.7 rule of thumb on P_el
        /// by `engine`.
        #[serde(default, rename = "thermalPowerKw")]
        thermal_power_kw: Option<f64>,
        /// Table 9.7: gas engine (1,5), diesel engine (1,2) or micro
        /// turbine (2,5); unknown → gas engine.
        #[serde(default)]
        engine: Option<ChpEngine>,
        #[serde(default, rename = "manufactureYear")]
        manufacture_year: Option<i32>,
        /// HRe declaration for P_el ≤ 2 kW (table 9.31).
        #[serde(default, rename = "hreDeclared")]
        hre_declared: bool,
        /// LT operation demonstrated with design data (table 9.31 note a).
        #[serde(default, rename = "lowTemperature")]
        low_temperature: bool,
    },
    /// Table 9.3: local gas heating incl. pilot flame, oil heating or a
    /// steam boiler, with or without a flue.
    LocalFired {
        appliance: LocalFiredAppliance,
        /// Steam boilers: gas or oil (p. 107); the heaters follow their
        /// appliance.
        #[serde(default)]
        fuel: Option<FiredFuel>,
        #[serde(rename = "flueGasExhaust")]
        flue_gas_exhaust: bool,
        /// Electricity connection (NTA 9.6.8.2.3); `None` → connected.
        #[serde(default, rename = "electricityConnected")]
        electricity_connected: Option<bool>,
    },
    /// Table 9.3: direct-fired gas air heater(s).
    GasAirHeater {
        #[serde(rename = "heaterType")]
        heater_type: AirHeaterType,
        #[serde(default, rename = "pilotFlame")]
        pilot_flame: Option<bool>,
        /// Number of air heaters; `None`: the table 9.16 count, else 1.
        #[serde(default)]
        count: Option<u32>,
    },
    /// No generator present (renovation), previous one unknown.
    NonePresent,
}

/// Prime mover of a CHP for the table 9.7 rule of thumb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChpEngine {
    GasEngine,
    DieselEngine,
    MicroTurbine,
}

impl ChpEngine {
    /// Table 9.7 (ISSO 82.1 p. 113): thermal / electrical power.
    pub fn thermal_ratio(self) -> f64 {
        match self {
            Self::GasEngine => 1.5,
            Self::DieselEngine => 1.2,
            Self::MicroTurbine => 2.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Emitters {
    Radiators,
    LowTemperatureRadiators,
    FloorHeating,
    FloorHeatingAndRadiators,
    AirHeating,
    LocalHeaters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlAnswer {
    RoomThermostat,
    CentralWithRadiatorValves,
    IndividualRoomControl,
    Unknown,
}

/// Table 9.9 classes (supply/return, °C).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignClass {
    C45_40,
    C55_47,
    C70_50,
    C90_70,
}

impl DesignClass {
    fn supply_c(self) -> f64 {
        match self {
            Self::C45_40 => 45.0,
            Self::C55_47 => 55.0,
            Self::C70_50 => 70.0,
            Self::C90_70 => 90.0,
        }
    }

    fn mean_c(self) -> f64 {
        match self {
            Self::C45_40 => 42.5,
            Self::C55_47 => 51.0,
            Self::C70_50 => 60.0,
            Self::C90_70 => 80.0,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::C45_40 => "45/40",
            Self::C55_47 => "55/47",
            Self::C70_50 => "70/50",
            Self::C90_70 => "90/70",
        }
    }

    /// NTA table 9.14 class for the distribution; ISSO 70/50 has no NTA
    /// row and takes 70/60 (same design supply temperature).
    fn kernel(self) -> &'static str {
        match self {
            Self::C45_40 => "45_40",
            Self::C55_47 => "55_47",
            Self::C70_50 => "70_60",
            Self::C90_70 => "90_70",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyHeating {
    pub generator: HeatingGenerator,
    pub emitters: Emitters,
    #[serde(default)]
    pub design_class: Option<DesignClass>,
    /// `None`: not determinable.
    #[serde(default)]
    pub balanced: Option<bool>,
    pub control: ControlAnswer,
    /// Pipes in unheated spaces (afb. 9.1); `None`: spaces not inspectable
    /// and unknown whether pipes run there → present, forfait length.
    #[serde(default)]
    pub unheated_pipes: Option<UnheatedPipesAnswer>,
    /// Storeys served by the distribution (9.37).
    #[serde(default = "one")]
    pub storeys: u32,
    /// Nominal power of the main generator (table 9.7), kW; required with
    /// `additionalGenerators`.
    #[serde(default)]
    pub nominal_power_kw: Option<f64>,
    /// Further unequal generators on the same distribution (§9.3.2).
    #[serde(default)]
    pub additional_generators: Vec<AdditionalHeatingGenerator>,
    /// §9.3.6: a preferred generator added after delivery (renovation).
    #[serde(default)]
    pub added_preferred_generator: bool,
    /// Collective installation serving several dwellings (p. 106, 121).
    #[serde(default)]
    pub collective: Option<CollectiveHeating>,
    /// ISSO 82.1 table 9.16 / 75.1: the air-heating type with emitters
    /// `air_heating`.
    #[serde(default)]
    pub air_heating: Option<AirHeatingAnswer>,
    /// §9.4.2: one- or two-pipe system; `None` → two-pipe.
    #[serde(default)]
    pub distribution_type: Option<DistributionTypeAnswer>,
    /// Table 9.12; `None` unknown → not insulated.
    #[serde(default)]
    pub pipe_insulation: Option<PipeInsulationAnswer>,
    pub source_reference: String,
}

/// ISSO table 9.16: direct or indirect air heaters, or air heating
/// through the air-handling unit (then entered with the ventilation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AirHeatingAnswer {
    Direct {
        /// Radial recirculation fan; `None` unknown (radial).
        #[serde(default, rename = "radialFan")]
        radial_fan: Option<bool>,
        /// Number of air heaters (table 9.16).
        #[serde(default)]
        count: Option<u32>,
    },
    Indirect {
        #[serde(default, rename = "roomHeightAbove8M")]
        room_height_above_8_m: Option<bool>,
        #[serde(default, rename = "warmAirReturn")]
        warm_air_return: Option<bool>,
        #[serde(default, rename = "ecMotor")]
        ec_motor: Option<bool>,
        #[serde(default)]
        count: Option<u32>,
    },
    ViaAirHandlingUnit,
}

/// One further generator with its table 9.7 nominal power.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdditionalHeatingGenerator {
    pub generator: HeatingGenerator,
    #[serde(default)]
    pub nominal_power_kw: Option<f64>,
}

/// A collective heating installation (ISSO 82.1 p. 106, 121–122).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectiveHeating {
    /// Usable area of the building on the installation, m².
    #[serde(default)]
    pub connected_usable_area_m2: Option<f64>,
    /// Dwellings on the installation, for the p. 121 rule.
    #[serde(default)]
    pub connected_dwellings: Option<u32>,
    /// Storeys on the installation (9.37); `None`: the dwelling's storeys.
    #[serde(default)]
    pub connected_storeys: Option<u32>,
    /// Table 9.16: `None` unknown → present.
    #[serde(default)]
    pub heat_meters_present: Option<bool>,
}

fn one() -> u32 {
    1
}

/// Table 9.9 when the class is unknown.
pub fn design_class(
    emitters: Emitters,
    generator: &HeatingGenerator,
    recorder: &mut Recorder,
) -> DesignClass {
    let class = match (emitters, generator) {
        (Emitters::FloorHeating, _) => DesignClass::C45_40,
        (Emitters::LowTemperatureRadiators, _) => DesignClass::C55_47,
        (
            _,
            HeatingGenerator::HeatPump {
                high_temperature: false,
                ..
            },
        ) => DesignClass::C55_47,
        (
            _,
            HeatingGenerator::HeatPump {
                high_temperature: true,
                ..
            },
        ) => DesignClass::C70_50,
        _ => DesignClass::C90_70,
    };
    recorder.record(
        "design_temperature_class_unknown",
        "heating.designClass",
        class.label().into(),
        "ISSO 82.1 p. 114 (table 9.9, erratum §4)",
    );
    class
}

/// Kernel fragments for the space-heating chain.
pub struct DerivedHeating {
    pub emission: Value,
    pub generator: Value,
    pub distribution_system: Option<Value>,
    pub heat_pump_renewable: Option<Value>,
    /// `externalSupply.collectiveHeatPumpSource` (NTA 9.6.8.1.1.2.3).
    pub collective_heat_pump_source: Option<Value>,
    /// Water-based emission (a distribution system exists).
    pub hydronic: bool,
    /// NTA table 9.14 class of the distribution.
    pub design_class: &'static str,
    pub construction_year: i32,
}

/// Pipes in unheated spaces (afb. 9.1, p. 120).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UnheatedPipesAnswer {
    /// No central-heating pipes or manifolds in unheated spaces.
    Absent,
    /// Present; `lengthM` the supply plus return length there, else forfait.
    Present {
        #[serde(default, rename = "lengthM")]
        length_m: Option<f64>,
    },
}

pub fn derive_heating(
    heating: &SurveyHeating,
    construction_year: i32,
    recorder: &mut Recorder,
) -> DerivedHeating {
    let reference = heating.source_reference.as_str();
    let generator = match &heating.generator {
        HeatingGenerator::NonePresent => {
            recorder.record(
                "no_generator_conventional_boiler",
                "heating.generator",
                "conventional boiler".into(),
                "ISSO 82.1 p. 107",
            );
            HeatingGenerator::Boiler {
                boiler_type: BoilerType::Conventional,
                pilot_flame: None,
                inside_thermal_boundary: true,
                manufacture_year: None,
                installation_year: None,
            }
        }
        other => other.clone(),
    };
    // Table 9.9 (p. 114): with several generators a non-heat-pump one
    // decides the class ("als opwekker geen warmtepomp is").
    let class_generator = std::iter::once(&generator)
        .chain(
            heating
                .additional_generators
                .iter()
                .map(|item| &item.generator),
        )
        .find(|item| !is_heat_pump(item))
        .unwrap_or(&generator);
    let design = heating
        .design_class
        .unwrap_or_else(|| design_class(heating.emitters, class_generator, recorder));
    let (system, hydronic) = match heating.emitters {
        Emitters::Radiators | Emitters::LowTemperatureRadiators => {
            ("radiators_or_convectors", true)
        }
        Emitters::FloorHeating => ("floor_heating", true),
        Emitters::FloorHeatingAndRadiators => {
            recorder.record(
                "several_emitters_surface_heating_first",
                "heating.emitters",
                "floor_heating".into(),
                "ISSO 82.1 p. 123–124",
            );
            ("floor_heating", true)
        }
        Emitters::AirHeating => ("air_heating", false),
        Emitters::LocalHeaters => ("local_heater", false),
    };
    let balancing = if !hydronic {
        "not_applicable"
    } else {
        match heating.balanced {
            Some(true) => "static",
            Some(false) => "none_or_unknown",
            None => {
                recorder.record(
                    "hydronic_balancing_unknown_none",
                    "heating.balanced",
                    "none_or_unknown".into(),
                    "ISSO 82.1 p. 116 (table 9.10)",
                );
                "none_or_unknown"
            }
        }
    };
    let control = match heating.control {
        ControlAnswer::RoomThermostat => "main_room_thermostat",
        ControlAnswer::CentralWithRadiatorValves => "central_with_room_valves",
        ControlAnswer::IndividualRoomControl => "individual_room_thermostats",
        ControlAnswer::Unknown => {
            recorder.record(
                "emission_control_unknown_other",
                "heating.control",
                "other_or_unknown".into(),
                "ISSO 82.1 p. 124 (table 9.17)",
            );
            "other_or_unknown"
        }
    };
    let mut emission = json!({
        "system": system, "balancing": balancing, "control": control,
        "sourceReference": format!("{reference}; basisopname"),
    });
    // Table 9.16 → NTA 9.23 (tables 9.12/9.13); unknown properties take the
    // highest factor, as the kernel does for a missing value.
    match (heating.emitters, heating.air_heating) {
        (Emitters::AirHeating, Some(answer)) => {
            let kind = match answer {
                AirHeatingAnswer::Direct { radial_fan, .. } => {
                    if radial_fan.is_none() {
                        recorder.record(
                            "air_heater_fan_unknown_radial",
                            "heating.airHeating.radialFan",
                            "radial recirculation fan".into(),
                            "ISSO 82.1 p. 123 (table 9.16)",
                        );
                    }
                    Some(json!({"kind": "direct", "radialFan": radial_fan}))
                }
                AirHeatingAnswer::Indirect {
                    room_height_above_8_m,
                    warm_air_return,
                    ec_motor,
                    ..
                } => {
                    if room_height_above_8_m.is_none()
                        || warm_air_return.is_none()
                        || ec_motor.is_none()
                    {
                        recorder.record(
                            "air_heater_indirect_unknown_highest",
                            "heating.airHeating",
                            "AC fans, room higher than 8 m, without warm-air return".into(),
                            "ISSO 82.1 p. 123 (table 9.16)",
                        );
                    }
                    Some(json!({
                        "kind": "indirect",
                        "roomHeightAbove8M": room_height_above_8_m,
                        "warmAirReturn": warm_air_return,
                        "ecMotor": ec_motor,
                    }))
                }
                // The AHU fans count in chapter 11.
                AirHeatingAnswer::ViaAirHandlingUnit => None,
            };
            if let Some(kind) = kind {
                emission["airHeaters"] = json!({
                    "kind": kind,
                    "sourceReference": format!("{reference}; ISSO table 9.16"),
                });
            }
        }
        (Emitters::AirHeating, None) => {
            recorder.record(
                "air_heating_type_unknown",
                "heating.airHeating",
                "not applicable: no air-heater fan energy".into(),
                "ISSO 82.1 p. 123 (table 9.16)",
            );
        }
        (_, Some(_)) => recorder.issue(
            "air_heating_requires_air_heating_emitters",
            "heating.airHeating",
        ),
        (_, None) => {}
    }

    let collective = heating.collective.is_some();
    // Table 9.9 per generator (p. 114): with several generators a heat
    // pump keeps its own class (footnote 8: a hybrid heat pump is not
    // high-temperature by default); the distribution takes the highest.
    let several = !heating.additional_generators.is_empty();
    let generator_design = |item: &HeatingGenerator, recorder: &mut Recorder| {
        if several && is_heat_pump(item) && heating.design_class.is_none() {
            design_class(heating.emitters, item, recorder)
        } else {
            design
        }
    };
    let main_design = generator_design(&generator, recorder);
    let main = convert_generator(
        &generator,
        main_design,
        hydronic,
        collective,
        heating.nominal_power_kw,
        heating.air_heating,
        "opwekker-1",
        reference,
        construction_year,
        recorder,
    );
    let mut needs_pump = main.needs_pump;
    let mut heat_pump_renewable = main.heat_pump_renewable;
    let mut collective_heat_pump_source = main.collective_source;
    let generator_value = if heating.additional_generators.is_empty() {
        if is_exhaust_air(&generator) {
            // WD 2025 p. 43: an exhaust-air heat pump needs a second
            // generator.
            recorder.issue(
                "exhaust_air_heat_pump_second_generator_required",
                "heating.generator.source",
            );
        }
        main.value
    } else {
        let mut parts = vec![(generator.clone(), heating.nominal_power_kw, main.value)];
        for (index, extra) in heating.additional_generators.iter().enumerate() {
            let generator = match &extra.generator {
                HeatingGenerator::NonePresent => {
                    recorder.issue(
                        "additional_generator_missing",
                        format!("heating.additionalGenerators[{index}].generator"),
                    );
                    continue;
                }
                other => other.clone(),
            };
            let extra_design = generator_design(&generator, recorder);
            let converted = convert_generator(
                &generator,
                extra_design,
                hydronic,
                collective,
                extra.nominal_power_kw,
                heating.air_heating,
                &format!("opwekker-{}", index + 2),
                reference,
                construction_year,
                recorder,
            );
            needs_pump |= converted.needs_pump;
            heat_pump_renewable = heat_pump_renewable.or(converted.heat_pump_renewable);
            collective_heat_pump_source =
                collective_heat_pump_source.or(converted.collective_source);
            parts.push((generator, extra.nominal_power_kw, converted.value));
        }
        multiple_generators(
            parts,
            heating.added_preferred_generator,
            reference,
            recorder,
        )
    };
    let distribution_system = needs_pump.then(|| {
        let (transmittance, valves) = pipe_insulation(heating, construction_year, recorder);
        let one_pipe = one_pipe_emitters(heating, recorder);
        recorder.record(
            "pump_unknown_forfait",
            "heating.distribution.pump",
            "forfait (9.41–9.51)".into(),
            "ISSO 82.1 p. 117 (table 9.11)",
        );
        match &heating.collective {
            Some(collective) => {
                // Table 9.16 (p. 122): heat meters unknown → present.
                let meters = collective.heat_meters_present.unwrap_or_else(|| {
                    recorder.record(
                        "heat_meters_unknown_present",
                        "heating.collective.heatMetersPresent",
                        "present".into(),
                        "ISSO 82.1 p. 123 (table 9.16)",
                    );
                    true
                });
                json!({
                    "designTemperatureClass": design.kernel(),
                    "installation": "collective",
                    "usageFunction": "residential",
                    "connectedStoreys": collective.connected_storeys.unwrap_or(heating.storeys).max(1),
                    "pipeTransmittance": transmittance,
                    "valvesInsulated": valves,
                    "pump": {"method": "calculated", "heatMeterPresent": meters, "onePipeEmitterCount": one_pipe, "sourceReference": "basisopname forfait; collective installation"},
                    "sourceReference": format!("{reference}; basisopname"),
                })
            }
            None => json!({
                "designTemperatureClass": design.kernel(),
                "installation": "individual",
                "usageFunction": "residential",
                "connectedStoreys": heating.storeys.max(1),
                "pipeTransmittance": transmittance,
                "valvesInsulated": valves,
                // Table 9.16a (heat meter unknown: present) covers
                // collective installations only.
                "pump": {"method": "calculated", "heatMeterPresent": false, "onePipeEmitterCount": one_pipe, "sourceReference": "basisopname forfait; individual installation without heat meter"},
                "sourceReference": format!("{reference}; basisopname"),
            }),
        }
    });
    DerivedHeating {
        emission,
        generator: generator_value,
        distribution_system,
        heat_pump_renewable,
        collective_heat_pump_source,
        hydronic,
        design_class: design.kernel(),
        construction_year,
    }
}

/// Table 9.12 (p. 118) as NTA table 9.16 forfait Ψ and the 9.27a/9.27b
/// valves: not insulated when unknown, the year of insulation unknown →
/// the construction year, fittings unknown → not insulated.
fn pipe_insulation(
    heating: &SurveyHeating,
    construction_year: i32,
    recorder: &mut Recorder,
) -> (Value, bool) {
    let Some(answer) = heating.pipe_insulation else {
        recorder.record(
            "pipe_insulation_unknown_uninsulated",
            "heating.distribution",
            "uninsulated".into(),
            "ISSO 82.1 p. 118 (table 9.12)",
        );
        return (
            json!({"method": "forfait", "insulation": {"state": "uninsulated"}}),
            false,
        );
    };
    let fittings = answer.fittings_insulated.unwrap_or_else(|| {
        recorder.record(
            "pipe_fittings_unknown_uninsulated",
            "heating.pipeInsulation.fittingsInsulated",
            "false".into(),
            "ISSO 82.1 p. 118 (table 9.12)",
        );
        false
    });
    if !answer.insulated {
        return (
            json!({"method": "forfait", "insulation": {"state": "uninsulated"}}),
            fittings,
        );
    }
    let year = answer.insulation_year.unwrap_or_else(|| {
        recorder.record(
            "pipe_insulation_year_unknown_construction_year",
            "heating.pipeInsulation.insulationYear",
            construction_year.to_string(),
            "ISSO 82.1 p. 118 (table 9.12)",
        );
        construction_year
    });
    let period = if year >= 1995 {
        "from1995"
    } else if year >= 1980 {
        "from1980_to1995"
    } else {
        "before1980_or_unknown"
    };
    (
        json!({"method": "forfait", "insulation": {"state": "insulated", "period": period}}),
        fittings,
    )
}

/// §9.4.2 (p. 115): the emitters of a one-pipe loop for table 9.21; a
/// renovated one-pipe system (WD 2025 p. 50) has no NTA rule of its own
/// and counts as two-pipe.
fn one_pipe_emitters(heating: &SurveyHeating, recorder: &mut Recorder) -> Option<u32> {
    match heating.distribution_type {
        Some(DistributionTypeAnswer::OnePipe { emitter_count }) => {
            if emitter_count == 0 {
                recorder.issue(
                    "one_pipe_emitter_count_invalid",
                    "heating.distributionType.emitterCount",
                );
            }
            Some(emitter_count)
        }
        Some(DistributionTypeAnswer::RenovatedOnePipe) => {
            recorder.record(
                "renovated_one_pipe_as_two_pipe",
                "heating.distributionType",
                "two_pipe".into(),
                "ISSO 82.1 WD 2025 p. 50; NTA 8800 p. 317 (one-pipe rule only)",
            );
            None
        }
        Some(DistributionTypeAnswer::TwoPipe) | None => None,
    }
}

/// NTA 9.2 f_gebouw;si;H of a collective installation: the connected usable
/// area, or dwellings × the dwelling's area (ISSO 82.1 p. 121).
pub fn collective_connection(
    heating: &SurveyHeating,
    dwelling_area_m2: f64,
    recorder: &mut Recorder,
) -> Option<Value> {
    let collective = heating.collective.as_ref()?;
    let area = match (
        collective.connected_usable_area_m2,
        collective.connected_dwellings,
    ) {
        (Some(area), _) => area,
        (None, Some(dwellings)) => {
            let area = f64::from(dwellings) * dwelling_area_m2;
            recorder.record(
                "collective_area_dwellings_times_area",
                "heating.collective.connectedUsableAreaM2",
                format!("{dwellings} × {dwelling_area_m2} m² = {area} m²"),
                "ISSO 82.1 p. 121",
            );
            area
        }
        (None, None) => {
            recorder.issue(
                "collective_connected_area_required",
                "heating.collective.connectedUsableAreaM2",
            );
            return None;
        }
    };
    if !(area.is_finite() && area >= dwelling_area_m2) {
        recorder.issue(
            "collective_connected_area_invalid",
            "heating.collective.connectedUsableAreaM2",
        );
        return None;
    }
    Some(json!({
        "connectedUsableAreaM2": area,
        "sourceReference": format!("{}; ISSO 82.1 p. 121", heating.source_reference),
    }))
}

/// Afb. 9.1 (p. 120): heating pipes in unheated spaces turn the
/// distribution into the calculated route (9.26–9.40) with the 15 %
/// forfait share of 9.26 unless a length is given.
pub fn apply_unheated_pipes(
    heating: &SurveyHeating,
    derived: &mut DerivedHeating,
    unheated_spaces_present: bool,
    recorder: &mut Recorder,
) -> Option<Value> {
    if !derived.hydronic || !unheated_spaces_present {
        return None;
    }
    let length = match heating.unheated_pipes {
        Some(UnheatedPipesAnswer::Absent) => return None,
        Some(UnheatedPipesAnswer::Present { length_m }) => length_m,
        None => {
            recorder.record(
                "unheated_pipes_unknown_present",
                "heating.unheatedPipes",
                "present, forfait length (15 % of L, 9.26)".into(),
                "ISSO 82.1 p. 120 (afb. 9.1), p. 121 (table 9.14)",
            );
            None
        }
    };
    let reference = heating.source_reference.as_str();
    let year = derived.construction_year;
    let mut system = derived.distribution_system.take().unwrap_or_else(|| {
        let (transmittance, valves) = pipe_insulation(heating, year, recorder);
        json!({
            "designTemperatureClass": derived.design_class,
            "installation": "individual",
            "usageFunction": "residential",
            "connectedStoreys": heating.storeys.max(1),
            "pipeTransmittance": transmittance,
            "valvesInsulated": valves,
            "pump": {"method": "included_in_generator_auxiliary"},
            "sourceReference": format!("{reference}; basisopname"),
        })
    });
    if let Some(length) = length {
        system["unheatedPipeLengthM"] = json!(length);
    }
    derived.distribution_system = Some(system);
    Some(json!({
        "method": "calculated",
        "sourceReference": format!("{reference}; basisopname: pipes in unheated spaces (afb. 9.1)"),
    }))
}

/// Table 9.7 (ISSO 82.1 p. 113, 75.1 equally): the thermal power of a
/// CHP; unknown → the rule of thumb on the electrical power: 1,5 for gas
/// engines, 1,2 for diesel engines and 2,5 for micro turbines (engine
/// unknown → gas engine).
fn chp_thermal_power(
    electrical: f64,
    thermal: Option<f64>,
    engine: Option<ChpEngine>,
    recorder: &mut Recorder,
) -> f64 {
    thermal.unwrap_or_else(|| {
        let engine = engine.unwrap_or_else(|| {
            recorder.record(
                "chp_engine_unknown_gas",
                "heating.generator.engine",
                "gas_engine".into(),
                "ISSO 82.1 p. 113 (table 9.7)",
            );
            ChpEngine::GasEngine
        });
        let ratio = engine.thermal_ratio();
        recorder.record(
            "chp_thermal_power_from_electrical",
            "heating.generator.thermalPowerKw",
            format!("{ratio} × {electrical} kW"),
            "ISSO 82.1 p. 113 (table 9.7)",
        );
        ratio * electrical
    })
}

fn is_exhaust_air(generator: &HeatingGenerator) -> bool {
    matches!(
        generator,
        HeatingGenerator::HeatPump {
            source: HeatPumpSource::ExhaustAir,
            ..
        }
    )
}

fn is_heat_pump(generator: &HeatingGenerator) -> bool {
    matches!(generator, HeatingGenerator::HeatPump { .. })
}

/// ISSO 82.1 p. 112 order (NTA 9.2.2.1.3, table 9.1): 1 exhaust-air heat
/// pumps, 2 other heat pumps, 3 CHP, 4 biomass, 5 external heat, 6
/// electric heating, 7 gas or oil boilers.
fn priority_class(generator: &HeatingGenerator) -> u32 {
    match generator {
        HeatingGenerator::HeatPump {
            source: HeatPumpSource::ExhaustAir,
            ..
        } => 1,
        HeatingGenerator::HeatPump { .. } => 2,
        HeatingGenerator::Chp { .. } => 3,
        HeatingGenerator::Biomass { .. } => 4,
        HeatingGenerator::DistrictHeat => 5,
        HeatingGenerator::Electric { .. } => 6,
        HeatingGenerator::Boiler { .. }
        | HeatingGenerator::LocalFired { .. }
        | HeatingGenerator::GasAirHeater { .. }
        | HeatingGenerator::NonePresent => 7,
    }
}

/// §9.3.2 / NTA 9.6.1: the kernel `multiple` generator with preferences
/// from the p. 112 order and the table 9.7 powers.
fn multiple_generators(
    parts: Vec<(HeatingGenerator, Option<f64>, Value)>,
    added_preferred_generator: bool,
    reference: &str,
    recorder: &mut Recorder,
) -> Value {
    // Table 9.7: a CHP enters with its thermal power.
    let parts: Vec<(HeatingGenerator, Option<f64>, Value)> = parts
        .into_iter()
        .map(|(generator, power, value)| {
            let power = power.or_else(|| {
                matches!(generator, HeatingGenerator::Chp { .. })
                    .then(|| value["auxiliary"]["nominalPowerKw"].as_f64())
                    .flatten()
            });
            (generator, power, value)
        })
        .collect();
    let mut classes: Vec<u32> = parts
        .iter()
        .map(|(generator, _, _)| priority_class(generator))
        .collect();
    classes.sort_unstable();
    classes.dedup();
    let known: f64 = parts
        .iter()
        .filter(|(generator, _, _)| !matches!(generator, HeatingGenerator::DistrictHeat))
        .filter_map(|(_, power, _)| *power)
        .sum();
    let mut generators = Vec::with_capacity(parts.len());
    for (index, (generator, power, value)) in parts.into_iter().enumerate() {
        let path = if index == 0 {
            "heating.nominalPowerKw".to_string()
        } else {
            format!("heating.additionalGenerators[{}].nominalPowerKw", index - 1)
        };
        let power = match power {
            Some(power) => power,
            None if matches!(generator, HeatingGenerator::DistrictHeat) && known > 0.0 => {
                // Table 9.7: external heat of unknown power delivers 50 %.
                recorder.record(
                    "external_heat_power_unknown_half",
                    &path,
                    format!("{known} kW (50 % of the total)"),
                    "ISSO 82.1 p. 113 (table 9.7)",
                );
                known
            }
            None => {
                recorder.issue("generator_power_required", path);
                0.0
            }
        };
        let preference = classes
            .iter()
            .position(|class| *class == priority_class(&generator))
            .map_or(1, |position| position + 1);
        generators.push(json!({
            "preference": preference,
            "nominalPowerKw": power,
            "generator": value,
        }));
    }
    recorder.record(
        "several_generators_preference_order",
        "heating.additionalGenerators",
        "heat pumps, CHP, biomass, external heat, electric, boilers".into(),
        "ISSO 82.1 p. 112 (§9.3.2)",
    );
    json!({
        "kind": "multiple",
        "generators": generators,
        "addedPreferredGenerator": added_preferred_generator,
        "sourceReference": format!("{reference}; ISSO 82.1 §9.3.2"),
    })
}

/// One survey generator as a kernel generator.
struct Converted {
    value: Value,
    /// The generator's auxiliary energy excludes the distribution pump.
    needs_pump: bool,
    heat_pump_renewable: Option<Value>,
    collective_source: Option<Value>,
}

#[allow(clippy::too_many_arguments)]
fn convert_generator(
    generator: &HeatingGenerator,
    design: DesignClass,
    hydronic: bool,
    collective: bool,
    nominal_power_kw: Option<f64>,
    air_heating: Option<AirHeatingAnswer>,
    id: &str,
    reference: &str,
    construction_year: i32,
    recorder: &mut Recorder,
) -> Converted {
    let mut heat_pump_renewable = None;
    let mut collective_source = None;
    let mut needs_pump = false;
    let value = match generator {
        HeatingGenerator::Boiler {
            boiler_type,
            pilot_flame,
            inside_thermal_boundary,
            manufacture_year,
            installation_year,
        } => {
            let oil = *boiler_type == BoilerType::Oil;
            let kind = match boiler_type {
                BoilerType::Conventional | BoilerType::Oil => "conventional",
                BoilerType::Vr => "vr",
                BoilerType::Hr100 => "hr100",
                BoilerType::Hr104 => "hr104",
                BoilerType::Hr107 => "hr107",
                BoilerType::Hydrogen => {
                    recorder.record(
                        "hydrogen_boiler_hr107",
                        "heating.generator.boilerType",
                        "hr107".into(),
                        "ISSO 82.1 p. 108",
                    );
                    "hr107"
                }
            };
            let pilot = if oil {
                // Table 9.25: the pilot flame is a gas-boiler item.
                if *pilot_flame == Some(true) {
                    recorder.issue("pilot_flame_gas_only", "heating.generator.pilotFlame");
                }
                false
            } else {
                pilot_flame.unwrap_or_else(|| {
                    recorder.record(
                        "pilot_flame_unknown_present",
                        "heating.generator.pilotFlame",
                        "true".into(),
                        "ISSO 82.1 p. 108 (table 9.3)",
                    );
                    true
                })
            };
            let year = super::general::device_year(
                *manufacture_year,
                *installation_year,
                construction_year,
                recorder,
                "heating.generator",
            );
            json!({
                "kind": "gas_boiler",
                "boiler": {
                    "generatorId": id,
                    "role": if collective { "collective" } else { "individual_main" },
                    "location": if *inside_thermal_boundary { "inside_thermal_boundary" } else { "outside_thermal_boundary" },
                    "kind": kind,
                    "fuel": if oil { "oil" } else { "natural_gas" },
                    "averageDesignEmissionTemperatureC": design.mean_c(),
                    "emissionCircuit": "direct",
                    "equipmentReference": reference,
                    "locationReference": reference,
                    "temperatureAndCircuitReference": format!("{reference}; table 9.9 class {}", design.label()),
                    "pilotFlamePresent": pilot,
                    "installationYear": year,
                    "installationYearReference": "basisopname (ISSO 82.1 p. 28)",
                }
            })
        }
        HeatingGenerator::HeatPump { .. } => {
            let converted = convert_heat_pump(
                generator,
                design,
                collective,
                nominal_power_kw,
                id,
                reference,
                recorder,
            );
            heat_pump_renewable = converted.heat_pump_renewable;
            collective_source = converted.collective_source;
            // A gas-driven heat pump is outside 9.85: the pump is separate.
            needs_pump = hydronic && converted.value["kind"] == "gas_heat_pump";
            converted.value
        }
        HeatingGenerator::DistrictHeat => {
            needs_pump = hydronic;
            json!({
                "kind": "external_heat",
                "supplierReference": reference,
                "qualityDeclarationPresent": false,
                "auxiliary": {"electricallyConnectedDevices": 1, "sourceReference": "basisopname: one delivery set"},
            })
        }
        HeatingGenerator::Electric { connected_devices } => {
            needs_pump = hydronic;
            json!({
                "kind": "electric_resistance",
                "equipmentReference": reference,
                "auxiliary": {"electricallyConnectedDevices": connected_devices, "sourceReference": reference},
            })
        }
        HeatingGenerator::Biomass {
            appliance,
            inside_thermal_boundary,
            sole_heating_in_served_rooms,
            annex_r_compliant,
        } => {
            needs_pump = hydronic;
            let compliant = annex_r_compliant.unwrap_or_else(|| {
                recorder.record(
                    "biomass_annex_r_unknown_not_compliant",
                    "heating.generator.annexRCompliant",
                    "false".into(),
                    "ISSO 82.1 p. 111–112, p. 28",
                );
                false
            });
            let appliance = match appliance {
                BiomassApplianceAnswer::FreestandingWoodStove => "freestanding_wood_stove",
                BiomassApplianceAnswer::InsertStove => "insert_stove",
                BiomassApplianceAnswer::PelletStove => "pellet_stove",
                BiomassApplianceAnswer::AccumulatingStove => "accumulating_stove",
                BiomassApplianceAnswer::CentralBoiler => "central_boiler",
            };
            json!({
                "kind": "biomass",
                "appliance": appliance,
                "location": if *inside_thermal_boundary { "inside_thermal_boundary" } else { "outside_thermal_boundary" },
                "annexRCompliantAtMost500Kw": compliant,
                "annexRReference": reference,
                "equipmentReference": reference,
                "soleHeatingInServedRooms": sole_heating_in_served_rooms,
                "auxiliary": {"electricallyConnectedDevices": 1, "sourceReference": reference},
            })
        }
        HeatingGenerator::Chp {
            electrical_power_kw,
            thermal_power_kw,
            engine,
            manufacture_year,
            hre_declared,
            low_temperature,
        } => {
            needs_pump = hydronic;
            let year = super::general::device_year(
                *manufacture_year,
                None,
                construction_year,
                recorder,
                "heating.generator",
            );
            let thermal =
                chp_thermal_power(*electrical_power_kw, *thermal_power_kw, *engine, recorder);
            json!({
                "kind": "chp",
                "chp": {
                    "powerKw": electrical_power_kw,
                    "builtAfter2006": year > 2006,
                    "hreDeclared": hre_declared,
                    "lowTemperature": low_temperature,
                },
                // 9.6.8.2 (9.91) with the thermal power as burner power.
                "auxiliary": {"electricallyConnectedDevices": 1, "nominalPowerKw": thermal, "sourceReference": reference},
                "equipmentReference": reference,
            })
        }
        HeatingGenerator::LocalFired { .. } | HeatingGenerator::GasAirHeater { .. } => {
            needs_pump = hydronic;
            convert_fired_heater(
                generator,
                air_heating,
                nominal_power_kw,
                reference,
                recorder,
            )
        }
        HeatingGenerator::NonePresent => unreachable!("replaced by the caller"),
    };
    let mut value = value;
    if collective {
        // A collective boiler or heat pump: its auxiliary energy (9.91)
        // excludes the distribution pump; a collective boiler needs its
        // nominal power (table 9.7).
        match value["kind"].as_str() {
            Some("gas_boiler") => {
                if nominal_power_kw.is_none() {
                    recorder.issue(
                        "collective_generator_power_required",
                        "heating.nominalPowerKw",
                    );
                }
                value["auxiliary"] = json!({
                    "electricallyConnectedDevices": 1,
                    "nominalPowerKw": nominal_power_kw,
                    "sourceReference": reference,
                });
                needs_pump = hydronic;
            }
            Some("heat_pump_forfait" | "gas_heat_pump") => needs_pump = hydronic,
            _ => {}
        }
    }
    Converted {
        value,
        needs_pump,
        heat_pump_renewable,
        collective_source,
    }
}

/// A survey heat pump as a kernel generator.
struct ConvertedHeatPump {
    value: Value,
    heat_pump_renewable: Option<Value>,
    collective_source: Option<Value>,
}

/// Table 9.6 (p. 110–111, WD 2025 p. 43–45) → NTA tables 9.27/9.29 (p.
/// 333–338) and 9.6.8.1.1.2.3 (p. 362).
///
/// - Table 9.29 applies to collective building installations and heat
///   pumps above 25 kW (its title), table 9.27 otherwise.
/// - Heat-pump panel: the outdoor-air row (p. 336, note 3).
/// - Groundwater or a collective source without a known temperature: the
///   ground row (p. 335).
/// - A collective groundwater source of the doublet type: c_source 1,04
///   (table V.3, p. 1117); recirculation or unknown: 1,00.
/// - Gas-driven heat pumps (p. 109): the "GWP" rows of tables 9.27/9.29;
///   exhaust air, combined air and high-temperature sources are no option
///   (table 9.6 footnotes 4–6).
fn convert_heat_pump(
    generator: &HeatingGenerator,
    design: DesignClass,
    collective: bool,
    nominal_power_kw: Option<f64>,
    id: &str,
    reference: &str,
    recorder: &mut Recorder,
) -> ConvertedHeatPump {
    let HeatingGenerator::HeatPump {
        source,
        air_sink,
        capacity_kw,
        source_regeneration_factor,
        high_efficiency_evidence,
        drive,
        groundwater_system,
        collective_source_reference,
        source_temperature_c,
        source_temperature_reference,
        source_quality_declaration_reference,
        ..
    } = generator
    else {
        unreachable!("called for heat pumps only");
    };
    let capacity = capacity_kw.or(nominal_power_kw);
    let large = capacity.is_some_and(|value| value > 25.0);
    let collective_source = collective_source_reference
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    let water_based = matches!(
        source,
        HeatPumpSource::Ground
            | HeatPumpSource::Groundwater
            | HeatPumpSource::SurfaceWater
            | HeatPumpSource::WaterBasedUnknown
            | HeatPumpSource::HighTemperature
    );
    if collective_source.is_some() && !water_based {
        recorder.issue(
            "collective_source_water_based_only",
            "heating.generator.collectiveSourceReference",
        );
    }
    if matches!(source, HeatPumpSource::HighTemperature) && collective_source.is_none() {
        // p. 111: a high-temperature source is a collective source.
        recorder.issue(
            "high_temperature_source_collective_only",
            "heating.generator.collectiveSourceReference",
        );
    }
    let temperature = source_temperature_c.filter(|value| value.is_finite());
    let temperature_reference = source_temperature_reference
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| reference.to_string());
    let path = "heating.generator.source";
    let drive = drive.unwrap_or_default();
    let gas = drive != HeatPumpDrive::Electric;
    // Collective sources below 15 °C enter as groundwater (WD 2025 p. 45);
    // an unknown temperature takes the ground row (NTA p. 335).
    let groundwater_row = |recorder: &mut Recorder| match temperature {
        Some(value) if value < 15.0 => "groundwater_below15_c",
        _ => {
            recorder.record(
                "source_temperature_unknown_ground",
                path,
                "ground".into(),
                "NTA 8800 p. 335 (below table 9.27)",
            );
            "ground"
        }
    };
    let table_source: &str = match source {
        HeatPumpSource::OutdoorAir => "outdoor_air",
        HeatPumpSource::ExhaustAir => "exhaust_air",
        HeatPumpSource::OutdoorAndExhaustAir => {
            recorder.record(
                "combined_air_source_outdoor_row",
                path,
                "outdoor_air".into(),
                "NTA table 9.27 footnote c; ISSO 82.1 WD p. 43–45",
            );
            "outdoor_air"
        }
        HeatPumpSource::HeatPumpPanel => {
            recorder.record(
                "heat_pump_panel_outdoor_air_row",
                path,
                "outdoor_air".into(),
                "NTA 8800 p. 336 (note 3 below table 9.28)",
            );
            "outdoor_air"
        }
        HeatPumpSource::Ground => "ground",
        // The gas rows have no temperature condition for groundwater.
        HeatPumpSource::Groundwater if gas => "groundwater_below15_c",
        HeatPumpSource::Groundwater => groundwater_row(recorder),
        HeatPumpSource::SurfaceWater if collective_source.is_none() => {
            recorder.record(
                "individual_surface_water_as_ground",
                path,
                "ground".into(),
                "ISSO 82.1 p. 111",
            );
            "ground"
        }
        HeatPumpSource::SurfaceWater if collective || large => "surface_water",
        HeatPumpSource::SurfaceWater if gas => "groundwater_below15_c",
        // Table 9.27 has no surface-water row.
        HeatPumpSource::SurfaceWater => groundwater_row(recorder),
        HeatPumpSource::WaterBasedUnknown => {
            recorder.record(
                "heat_pump_water_source_unknown_ground",
                path,
                "ground".into(),
                "ISSO 82.1 p. 110 (table 9.6)",
            );
            "ground"
        }
        HeatPumpSource::HighTemperature => match temperature {
            Some(value) if value >= 40.0 => "collective_at_least40_c",
            Some(value) if value >= 20.0 => "collective20_to40_c",
            Some(value) if value >= 15.0 => "collective15_to20_c",
            _ => groundwater_row(recorder),
        },
    };
    let ground_row = matches!(table_source, "ground" | "groundwater_below15_c");
    // Table V.3 (p. 1117): collective groundwater source, doublet 1,04.
    let doublet = |recorder: &mut Recorder| -> f64 {
        if collective_source.is_none() || table_source != "groundwater_below15_c" {
            return 1.0;
        }
        match groundwater_system {
            Some(GroundwaterSystem::Doublet) => 1.04,
            Some(GroundwaterSystem::Recirculation) => 1.0,
            None => {
                recorder.record(
                    "groundwater_system_unknown_recirculation",
                    "heating.generator.groundwaterSystem",
                    "recirculation (c_source 1,00)".into(),
                    "ISSO 82.1 p. 110 (table 9.6); NTA table V.3",
                );
                1.0
            }
        }
    };
    let renewable = json!({
        "sourceBelow20C": !matches!(table_source, "collective20_to40_c" | "collective_at_least40_c"),
        "exhaustAirSource": matches!(source, HeatPumpSource::ExhaustAir),
        "combinedOutdoorAndExhaustAir": matches!(source, HeatPumpSource::OutdoorAndExhaustAir),
        "sourceReference": reference,
    });

    if gas {
        if matches!(
            source,
            HeatPumpSource::ExhaustAir
                | HeatPumpSource::OutdoorAndExhaustAir
                | HeatPumpSource::HighTemperature
        ) {
            recorder.issue("gas_heat_pump_source_not_allowed", path);
        }
        if *air_sink {
            recorder.issue(
                "gas_heat_pump_air_sink_unsupported",
                "heating.generator.airSink",
            );
        }
        if collective_source.is_some() {
            // 9.62 f_cor.bron.col is modelled for electric heat pumps only.
            recorder.issue(
                "gas_heat_pump_collective_source_unsupported",
                "heating.generator.collectiveSourceReference",
            );
        }
        if high_efficiency_evidence.is_some() {
            recorder.issue(
                "high_efficiency_evidence_electric_only",
                "heating.generator.highEfficiencyEvidence",
            );
        }
        if capacity.is_none() {
            // NTA §9.6.3 (p. 331) and 9.91/9.92 need P_H;gen.
            recorder.issue(
                "gas_heat_pump_capacity_required",
                "heating.generator.capacityKw",
            );
        }
        // Table 9.27 "GWP" rows: dwellings up to 25 kW (collective
        // installations included); table 9.29 above 25 kW.
        let table = if large {
            "utility_collective_or_above25_kw"
        } else {
            "residential_at_most25_kw"
        };
        let gas_source = match table_source {
            "groundwater_below15_c" => "groundwater",
            "outdoor_air" => "outdoor_air",
            "surface_water" => "surface_water",
            _ => "ground",
        };
        let correction_allowed = !large && matches!(gas_source, "ground" | "groundwater");
        if source_regeneration_factor.is_some() && !correction_allowed {
            recorder.issue(
                "source_regeneration_not_applicable",
                "heating.generator.sourceRegenerationFactor",
            );
        }
        let correction = correction_allowed
            .then(|| source_regeneration_factor.unwrap_or_else(|| doublet(recorder)));
        let drive_label = match drive {
            HeatPumpDrive::GasEngine => "gas engine",
            _ => "gas absorption",
        };
        return ConvertedHeatPump {
            value: json!({
                "kind": "gas_heat_pump",
                "table": table,
                "source": gas_source,
                "designSupplyTemperatureC": design.supply_c(),
                "sourceCorrectionFactor": correction,
                "auxiliary": {"electricallyConnectedDevices": 1, "nominalPowerKw": capacity, "sourceReference": reference},
                "equipmentReference": format!("{reference}; ISSO 82.1 p. 109 ({drive_label})"),
            }),
            heat_pump_renewable: Some(renewable),
            collective_source: None,
        };
    }

    let utility_table = collective || large;
    if utility_table {
        recorder.record(
            "heat_pump_table_9_29",
            "heating.generator",
            if collective {
                "collective building installation".into()
            } else {
                format!("{} kW > 25 kW", capacity.unwrap_or_default())
            },
            "NTA 8800 p. 337 (title of table 9.29)",
        );
    }
    // Table 9.27 footnote a: c_source for the ground and groundwater rows.
    let (correction, correction_reference) = if ground_row && !*air_sink && !utility_table {
        match source_regeneration_factor {
            Some(factor) => (json!(factor), json!(format!("{reference}; annex V"))),
            None => {
                let factor = doublet(recorder);
                if factor == 1.0 {
                    recorder.record(
                        "source_regeneration_none_c_source_1",
                        "heating.generator.sourceRegenerationFactor",
                        "1.0".into(),
                        "ISSO 82.1 p. 110; NTA table 9.27 footnote a",
                    );
                    (
                        json!(1.0),
                        json!("NTA 8800 table 9.27 footnote a: no regeneration"),
                    )
                } else {
                    (
                        json!(factor),
                        json!(format!("{reference}; NTA table V.3 doublet")),
                    )
                }
            }
        }
    } else {
        if source_regeneration_factor.is_some() {
            recorder.issue(
                "source_regeneration_not_applicable",
                "heating.generator.sourceRegenerationFactor",
            );
        }
        (Value::Null, Value::Null)
    };
    let mut forfait = json!({
        "generatorId": id,
        "classificationSourceReference": reference,
        "scope": if utility_table { "utility_collective_or_over25_kw" } else { "residential_at_most25_kw" },
        "source": table_source,
        "sink": if *air_sink { "indoor_air" } else { "hydronic" },
        "designSupplyTemperatureC": if *air_sink { Value::Null } else { json!(design.supply_c()) },
        "sourceCorrectionFactor": correction,
        "sourceCorrectionReference": correction_reference,
        "collectiveBuildingInstallation": collective,
    });
    if let Some(capacity) = capacity {
        forfait["thermalCapacityKw"] = json!(capacity);
        forfait["capacitySourceReference"] = json!(reference);
    }
    if let Some(evidence) = high_efficiency_evidence {
        forfait["rowVariant"] = json!("table_9_28_high_efficiency");
        forfait["highEfficiencyEvidence"] = evidence.clone();
    }
    if matches!(
        table_source,
        "groundwater_below15_c"
            | "collective15_to20_c"
            | "collective20_to40_c"
            | "collective_at_least40_c"
    ) {
        forfait["sourceTemperatureC"] = json!(temperature);
        forfait["sourceTemperatureEvidenceReference"] = json!(temperature_reference);
    }
    if let Some(declaration) = source_quality_declaration_reference
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        forfait["sourceQualityDeclarationReference"] = json!(declaration);
    }
    let (source_system, collective_value) = match collective_source {
        None => ("individual", None),
        Some(supplier) => {
            // 9.6.8.1.1.2.3 note 2 (p. 362): only ground or groundwater
            // sources are below 20 °C; surface water, warm or unknown
            // sources take tables 5.2–5.4.
            let below_20 = matches!(
                table_source,
                "ground" | "groundwater_below15_c" | "collective15_to20_c"
            ) && !matches!(source, HeatPumpSource::SurfaceWater)
                && !(matches!(source, HeatPumpSource::HighTemperature) && temperature.is_none());
            (
                if table_source == "ground" {
                    "collective_ground"
                } else {
                    "collective_groundwater_surface_or_at_least15_c"
                },
                Some(json!({
                    "temperatureClass": if below_20 { "below20_c" } else { "at_least20_c_or_surface_water_or_unknown" },
                    "supplierReference": supplier,
                })),
            )
        }
    };
    let mut value = json!({
        "kind": "heat_pump_forfait",
        "forfait": forfait,
        "sourceSystem": source_system,
        "sourceSystemReference": collective_source.unwrap_or(reference),
    });
    if collective {
        // 9.91 for a collective heat pump (9.85 covers individual ones).
        value["auxiliary"] = json!({
            "electricallyConnectedDevices": 1,
            "nominalPowerKw": capacity,
            "sourceReference": reference,
        });
    }
    ConvertedHeatPump {
        value,
        heat_pump_renewable: Some(renewable),
        collective_source: collective_value,
    }
}

/// Table 9.3 (p. 108) local or air heater as NTA table 9.25 "overige
/// systemen" (`forfait_heater`).
fn convert_fired_heater(
    generator: &HeatingGenerator,
    air_heating: Option<AirHeatingAnswer>,
    nominal_power_kw: Option<f64>,
    reference: &str,
    recorder: &mut Recorder,
) -> Value {
    match generator {
        HeatingGenerator::LocalFired {
            appliance,
            fuel,
            flue_gas_exhaust,
            electricity_connected,
        } => {
            let fuel = match (appliance, fuel) {
                (LocalFiredAppliance::GasHeater, None | Some(FiredFuel::NaturalGas)) => {
                    "natural_gas"
                }
                (LocalFiredAppliance::OilHeater, None | Some(FiredFuel::Oil)) => "oil",
                (LocalFiredAppliance::SteamBoiler, Some(FiredFuel::NaturalGas)) => "natural_gas",
                (LocalFiredAppliance::SteamBoiler, Some(FiredFuel::Oil)) => "oil",
                (LocalFiredAppliance::SteamBoiler, None) => {
                    recorder.issue("steam_boiler_fuel_required", "heating.generator.fuel");
                    "natural_gas"
                }
                _ => {
                    recorder.issue("local_heater_fuel_contradiction", "heating.generator.fuel");
                    "natural_gas"
                }
            };
            let connected = electricity_connected.unwrap_or_else(|| {
                recorder.record(
                    "fired_heater_electricity_unknown_connected",
                    "heating.generator.electricityConnected",
                    "true (10 W stand-by)".into(),
                    "NTA 8800 p. 365 (9.6.8.2.3)",
                );
                true
            });
            json!({
                "kind": "forfait_heater",
                "heaterKind": if *flue_gas_exhaust { "local_with_flue" } else { "local_without_flue" },
                "fuel": fuel,
                "equipmentReference": format!("{reference}; ISSO 82.1 table 9.3"),
                "auxiliary": {
                    "electricallyConnectedDevices": u32::from(connected),
                    "nominalPowerKw": nominal_power_kw,
                    "sourceReference": reference,
                },
            })
        }
        HeatingGenerator::GasAirHeater {
            heater_type,
            pilot_flame,
            count,
        } => {
            let count = count
                .or(match air_heating {
                    Some(AirHeatingAnswer::Direct { count, .. })
                    | Some(AirHeatingAnswer::Indirect { count, .. }) => count,
                    _ => None,
                })
                .unwrap_or_else(|| {
                    recorder.record(
                        "air_heater_count_unknown_one",
                        "heating.generator.count",
                        "1".into(),
                        "ISSO 82.1 p. 108 (table 9.3)",
                    );
                    1
                });
            if count == 0 {
                recorder.issue("air_heater_count_invalid", "heating.generator.count");
            }
            let pilot = pilot_flame.unwrap_or_else(|| {
                recorder.record(
                    "pilot_flame_unknown_present",
                    "heating.generator.pilotFlame",
                    "true".into(),
                    "ISSO 82.1 p. 108 (table 9.3)",
                );
                true
            });
            let kind = match heater_type {
                AirHeaterType::Conventional => "air_heater_conventional",
                AirHeaterType::Vr => "air_heater_vr",
                AirHeaterType::Hr100 => "air_heater_hr100",
                AirHeaterType::Hr104 => "air_heater_hr104",
                AirHeaterType::Hr107 => "air_heater_hr107",
            };
            json!({
                "kind": "forfait_heater",
                "heaterKind": kind,
                "fuel": "natural_gas",
                "equipmentReference": format!("{reference}; ISSO 82.1 table 9.3"),
                "pilotFlames": if pilot { count } else { 0 },
                "auxiliary": {
                    "electricallyConnectedDevices": count,
                    "nominalPowerKw": nominal_power_kw,
                    "sourceReference": reference,
                },
            })
        }
        _ => unreachable!("called for fired heaters only"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heating(generator: HeatingGenerator, emitters: Emitters) -> SurveyHeating {
        SurveyHeating {
            generator,
            emitters,
            design_class: None,
            balanced: None,
            control: ControlAnswer::Unknown,
            unheated_pipes: None,
            storeys: 2,
            nominal_power_kw: None,
            additional_generators: Vec::new(),
            added_preferred_generator: false,
            collective: None,
            air_heating: None,
            distribution_type: None,
            pipe_insulation: None,
            source_reference: "survey".into(),
        }
    }

    #[test]
    fn air_heating_maps_table_9_16_to_air_heaters() {
        let mut recorder = Recorder::default();
        let mut survey = heating(HeatingGenerator::NonePresent, Emitters::AirHeating);
        survey.air_heating = Some(AirHeatingAnswer::Direct {
            radial_fan: None,
            count: Some(3),
        });
        let derived = derive_heating(&survey, 1990, &mut recorder);
        assert_eq!(derived.emission["airHeaters"]["kind"]["kind"], "direct");
        assert!(derived.emission["airHeaters"]["kind"]["radialFan"].is_null());
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "air_heater_fan_unknown_radial"));

        let mut recorder = Recorder::default();
        survey.air_heating = Some(AirHeatingAnswer::Indirect {
            room_height_above_8_m: Some(false),
            warm_air_return: Some(true),
            ec_motor: Some(true),
            count: None,
        });
        let derived = derive_heating(&survey, 1990, &mut recorder);
        let heaters = &derived.emission["airHeaters"]["kind"];
        assert_eq!(heaters["kind"], "indirect");
        assert_eq!(heaters["ecMotor"], true);
        assert!(recorder
            .applied
            .iter()
            .all(|item| !item.rule.starts_with("air_heater")));

        // Via the AHU: no air-heater fans; unknown type: not applicable.
        survey.air_heating = Some(AirHeatingAnswer::ViaAirHandlingUnit);
        let derived = derive_heating(&survey, 1990, &mut recorder);
        assert!(derived.emission.get("airHeaters").is_none());
        survey.air_heating = None;
        let mut recorder = Recorder::default();
        derive_heating(&survey, 1990, &mut recorder);
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "air_heating_type_unknown"));

        // An air-heating answer with other emitters is inconsistent.
        let mut recorder = Recorder::default();
        let mut radiators = heating(HeatingGenerator::NonePresent, Emitters::Radiators);
        radiators.air_heating = Some(AirHeatingAnswer::ViaAirHandlingUnit);
        derive_heating(&radiators, 1990, &mut recorder);
        assert_eq!(
            recorder.issues[0].code,
            "air_heating_requires_air_heating_emitters"
        );
    }

    #[test]
    fn missing_generator_is_a_conventional_boiler_with_pilot_flame() {
        let mut recorder = Recorder::default();
        let derived = derive_heating(
            &heating(HeatingGenerator::NonePresent, Emitters::Radiators),
            1960,
            &mut recorder,
        );
        assert_eq!(derived.generator["boiler"]["kind"], "conventional");
        assert_eq!(derived.generator["boiler"]["pilotFlamePresent"], true);
        assert_eq!(derived.generator["boiler"]["installationYear"], 1960);
        assert_eq!(
            derived.generator["boiler"]["averageDesignEmissionTemperatureC"],
            80.0
        );
        assert_eq!(derived.emission["balancing"], "none_or_unknown");
        assert_eq!(derived.emission["control"], "other_or_unknown");
        assert!(derived.distribution_system.is_none());
    }

    #[test]
    fn design_class_follows_table_9_9() {
        let mut recorder = Recorder::default();
        let hp = HeatingGenerator::HeatPump {
            source: HeatPumpSource::WaterBasedUnknown,
            air_sink: false,
            high_temperature: false,
            capacity_kw: Some(6.0),
            source_regeneration_factor: None,
            high_efficiency_evidence: None,
            drive: Default::default(),
            groundwater_system: None,
            collective_source_reference: None,
            source_temperature_c: None,
            source_temperature_reference: None,
            source_quality_declaration_reference: None,
        };
        assert_eq!(
            design_class(Emitters::FloorHeating, &hp, &mut recorder),
            DesignClass::C45_40
        );
        assert_eq!(
            design_class(Emitters::Radiators, &hp, &mut recorder),
            DesignClass::C55_47
        );
        let derived = derive_heating(&heating(hp, Emitters::Radiators), 2015, &mut recorder);
        assert_eq!(derived.generator["forfait"]["source"], "ground");
        assert_eq!(
            derived.generator["forfait"]["designSupplyTemperatureC"],
            55.0
        );
        assert!(derived.heat_pump_renewable.is_some());
        // p. 110: no solar regeneration, c_source 1,0.
        assert_eq!(derived.generator["forfait"]["sourceCorrectionFactor"], 1.0);
    }

    #[test]
    fn exhaust_air_heat_pump_needs_a_second_generator() {
        let mut recorder = Recorder::default();
        let hp = HeatingGenerator::HeatPump {
            source: HeatPumpSource::ExhaustAir,
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
        let derived = derive_heating(&heating(hp, Emitters::Radiators), 2015, &mut recorder);
        assert_eq!(
            derived.generator["forfait"]["sourceCorrectionFactor"],
            Value::Null
        );
        assert_eq!(
            recorder.issues[0].code,
            "exhaust_air_heat_pump_second_generator_required"
        );
    }

    #[test]
    fn biomass_annex_r_unknown_is_not_compliant() {
        let mut recorder = Recorder::default();
        let stove = HeatingGenerator::Biomass {
            appliance: BiomassApplianceAnswer::PelletStove,
            inside_thermal_boundary: true,
            sole_heating_in_served_rooms: true,
            annex_r_compliant: None,
        };
        let derived = derive_heating(&heating(stove, Emitters::LocalHeaters), 2010, &mut recorder);
        assert_eq!(derived.generator["annexRCompliantAtMost500Kw"], false);
    }

    #[test]
    fn district_heat_with_radiators_gets_a_forfait_pump() {
        let mut recorder = Recorder::default();
        let derived = derive_heating(
            &heating(HeatingGenerator::DistrictHeat, Emitters::Radiators),
            1975,
            &mut recorder,
        );
        let system = derived.distribution_system.unwrap();
        assert_eq!(system["pump"]["heatMeterPresent"], false);
        assert_eq!(system["designTemperatureClass"], "90_70");
    }
}
