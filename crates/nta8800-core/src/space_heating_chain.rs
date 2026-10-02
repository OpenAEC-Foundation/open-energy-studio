//! Monthly space-heating chain for one or more zones and one generator:
//! need (chapter 7) → emission (9.3) → distribution (9.4) → node (9.2.3)
//! → generator (9.6) → energy per carrier and auxiliary energy (9.6.8,
//! 9.4.4).
//!
//! With a single generator the table 9.1 dispatch gives `β = 1` and the
//! generator covers the whole node input. The recoverable losses of 9.2.5
//! are reported per zone (`zoneRecoverableLosses`) but are not fed back to
//! the chapter 7 need here; that coupling belongs to the demand calculation.
//! All results are unverified.

use crate::annex_m::{
    boiler_month, validate_product_boiler, BoilerFuel, BoilerMonth, BoilerPlacement, ProductBoiler,
};
use crate::annex_n::{
    heater_month, resolve_heater, validate_local_heater, HeaterMonth, LocalHeater,
};
use crate::annex_o::{auxiliary_constants, monthly_auxiliary_kwh, AppliancePowerMeasurements};
use crate::annex_q::{
    calculate_annex_q, validate_annex_q, AnnexQContext, AnnexQHeatPump, AnnexQResult, AnnexQSource,
    DemandClass,
};
use crate::annex_v::{
    calculate_regeneration, validate_regeneration, RegenerationContext, RegenerationInput,
};
use crate::boiler_forfait_draft::{
    assess_boiler_forfait_monthly_draft, BoilerForfaitDraftInput, BoilerForfaitMonthlyDraftInput,
    BoilerRole,
};
use crate::climate::{MONTH_HOURS, OUTDOOR_TEMPERATURE_C};
use crate::final_energy_draft::MonthlyEnergy;
use crate::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput;
use crate::forfait_heat_pump_monthly_draft::{
    assess_forfait_heat_pump_monthly_draft, ForfaitHeatPumpMonthlyDraftInput, SourceSystem,
};
use crate::generator_dispatch_draft::{
    Generator as DispatchGenerator, GeneratorDispatchDraftInput,
};
use crate::heating_aux_draft::{
    assess_heating_aux_measured_draft, GeneratorElectricityMonth, HeatingAuxMeasuredDraftInput,
};
use crate::heating_distribution::{
    balancing_factor, buffer_loss_kwh, design_flow_m3_per_h, design_pressure_kpa,
    equivalent_length_m, forfait_max_pipe_length_m, forfait_pipe_length_m,
    generator_resistance_kpa, heating_limit_c, heating_limit_hours, hydraulic_power_kw,
    label_standing_loss_w, pump_energy_factor, supply_return_c, DesignTemperatureClass,
    EmitterResistance, PipeTransmittance, ReductionFunction, StorageLabel,
    DEFAULT_UNHEATED_AMBIENT_C, DELIVERY_SET_MIN_MEAN_C, RECOVERABLE_AUX_FRACTION,
    UNKNOWN_UNHEATED_SHARE,
};
use crate::heating_emission::{
    balancing_consistent, monthly_loss_kwh, temperature_increment_k, EmissionInput, EmissionSystem,
    HydronicBalancing, DRAFT_SOURCE,
};
use crate::humidification::{
    calculate_humidity, HumidityFunction, HumidityFunctionArea, HumidityInput,
};
use crate::hybrid_heat_pump_monthly_draft::{
    assess_hybrid_heat_pump_monthly_draft, HybridHeatPumpAuxMeasurements,
    HybridHeatPumpMonthlyDraftInput,
};
use crate::monthly_demand::{
    apply_recoverable_losses, assess_monthly_demand, MonthlyDemandAssessment, MonthlyDemandInput,
};
use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};

pub const OMITTED_TERMS: &[&str] = &[
    "9.2.3 node gains from solar thermal systems, booster heat pumps and delivery sets",
    "9.6.1: generators with the same preference share their energy by nominal power; product-specific hybrid switching and domestic hot water priority are not modelled",
    "θ_int;op;H of 7.9.6 is taken equal to the heating setpoint for the in-zone pipe ambient",
    "annex Q: c_source (annex V) is not applied to method 1 (9.63 has no c_source; tables 9.27/9.29 only); the degree of regeneration is reported",
    "annex Q: W_H;aux;hp;an is not booked again as 9.6.3.2 auxiliary energy, because Q.4 already includes it in η_H;gen;hp (COP of 9.63)",
    "annex Q: F_H;gen = 1 (Q.1) is taken as met when every bin of table Q.6 is fully covered; the rounded table hours sum to 277,757 instead of 277,778",
    "annex Q: V.1 hot-water term not needed, because c_source is not applied to method 1",
    "annex N: E_H;gen;in converted to the gross calorific value (N.3) with f_Hs/Hi of table M.3 (biomass as wood 1,08)",
    "annex M: ϑ_brm (M.12) from 9.4.2: heating setpoint for a heated space, ϑ_ztu of the distribution system for an installation room; table M.6 otherwise",
];

/// 9.63 `f_prac` for heat pumps with annex Q data (method 1).
pub const ANNEX_Q_PRACTICE_FACTOR: f64 = 0.95;

/// 9.85 forfait constants for individual electric heat pumps (page 360).
pub const HEAT_PUMP_AUX_A_KWH: f64 = 43.8;
pub const HEAT_PUMP_AUX_B_KW: f64 = 0.132;
pub const HEAT_PUMP_AUX_C: f64 = 0.7;
pub const HEAT_PUMP_AUX_B_NOM_KW: f64 = 3.0;
/// 9.6.8.2.3 rekenwaarden (page 365).
pub const OTHER_AUX_STANDBY_W: f64 = 10.0;
pub const OTHER_AUX_GAS_OIL_W_PER_KW: f64 = 1.0;
pub const OTHER_AUX_AUTOMATIC_BIOMASS_W_PER_KW: f64 = 10.0;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpaceHeatingChainInput {
    /// Humidifiers per zone (chapter 12); atomising humidifiers load the
    /// space-heating node (9.4), steam humidifiers have their own carrier.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub humidifiers: Vec<ZoneHumidifier>,
    /// 9.2.3.4 `Q_H;ren;prac` of solar combi systems per month (13.66a),
    /// kWh; the node gain is capped at the node output plus losses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solar_heating_kwh: Vec<f64>,
    /// 13.185 `E_W;gen;in;conv;hj` per month: hot water made with heat from
    /// this system (§13.8.4.9.3), kWh; it loads the node like space heating.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hot_water_load_kwh: Vec<f64>,
    pub demand: MonthlyDemandInput,
    pub emission: EmissionInput,
    pub distribution: Distribution,
    /// Further calculation zones served by the same generator (9.2, sum over zones).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_zones: Vec<ChainZone>,
    pub generator: Generator,
    /// Hydraulic data of the distribution system (§9.4); required for the
    /// calculated distribution loss and for distribution pump energy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution_system: Option<DistributionSystem>,
    /// Part of a building on a collective installation: `f_gebouw;si;H` =
    /// zone area / `A_g;gebouw;H`. Absent means the whole building (1,0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collective_connection: Option<CollectiveConnection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChainZone {
    pub demand: MonthlyDemandInput,
    pub emission: EmissionInput,
    pub distribution: Distribution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectiveConnection {
    /// `A_g;gebouw;H`: usable area of the whole building on the installation.
    pub connected_usable_area_m2: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Distribution {
    /// 9.4.1: piping only in heated zones and only for space heating, so the
    /// in-zone length is set to 0 and the loss is neglected.
    HeatedZoneOnlySpaceHeating {
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Monthly `Q_H;dis;ls` determined elsewhere, kWh.
    Declared {
        #[serde(rename = "monthlyLossKwh")]
        monthly_loss_kwh: Vec<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 9.26–9.40 with the data of `distributionSystem`.
    Calculated {
        /// 9.28/9.29: `E_V;eldf + E_V;elvv + Q_H;ϑHstook;in;air` per month,
        /// added to the need for the heating limit; absent means 0.
        #[serde(default, rename = "heatingLimitExtraKwh")]
        heating_limit_extra_kwh: Option<Vec<f64>>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Installation {
    Individual,
    Collective,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DistributionSystem {
    /// Table 9.14; absent means unknown, which uses 90/70.
    #[serde(default)]
    pub design_temperature_class: Option<DesignTemperatureClass>,
    pub installation: Installation,
    /// Table 7.15 function for `f_H;red` (9.32b).
    pub usage_function: ReductionFunction,
    /// Pipes inside the connected parts serve heating and hot water: the
    /// operating time is the whole month and the in-zone loss counts.
    #[serde(default)]
    pub pipes_also_for_hot_water: bool,
    /// Collective installation making hot water through a delivery set:
    /// `ϑ_H,mean ≥ 65 °C` and the whole month in operation.
    #[serde(default)]
    pub collective_hot_water_delivery_set: bool,
    /// `n_si` of 9.37, at least 1.
    pub connected_storeys: u32,
    pub pipe_transmittance: PipeTransmittance,
    /// `Ψ_j` for pipes in unheated spaces; absent means the same as in zone.
    #[serde(default)]
    pub unheated_pipe_transmittance: Option<PipeTransmittance>,
    /// 9.27a/9.27b.
    pub valves_insulated: bool,
    /// Actual total pipe length `L_si`; absent means 9.36.
    #[serde(default)]
    pub actual_pipe_length_m: Option<f64>,
    /// `L_si;j` in unheated spaces; absent means 15 % of `L_si`.
    #[serde(default)]
    pub unheated_pipe_length_m: Option<f64>,
    /// `ϑ_ztu` per month of the unheated space (7.82); absent means 13 °C.
    #[serde(default)]
    pub unheated_ambient_c: Option<Vec<f64>>,
    /// Collective buffer vessel (9.2.3.3/9.2.3.5); only with calculated Ψ.
    #[serde(default)]
    pub buffer_vessel: Option<BufferVessel>,
    pub pump: DistributionPump,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BufferVessel {
    pub volume_l: f64,
    /// `S_sto;ls` from the energy label in W; absent uses table 13.9.
    #[serde(default)]
    pub standing_loss_w: Option<f64>,
    /// Label class for table 13.9; absent means C from 2018, otherwise G.
    #[serde(default)]
    pub label: Option<StorageLabel>,
    pub produced_from_2018: bool,
    pub in_heated_space: bool,
    /// Constant-temperature system: `ϑ_sto;set = ϑ_H,a;ontw`.
    #[serde(default)]
    pub constant_temperature: bool,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum DistributionPump {
    /// The pump is part of the 9.85 auxiliary energy (individual gas
    /// boilers and individual electric heat pumps).
    IncludedInGeneratorAuxiliary,
    /// No pump on the plot, e.g. external heat without a heat exchanger.
    NoneOnSite {
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 9.41–9.51.
    Calculated(PumpInput),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PumpInput {
    /// Table 9.21: heat meter in the circuit.
    pub heat_meter_present: bool,
    /// `L_max`; absent means 9.37.
    #[serde(default)]
    pub max_pipe_length_m: Option<f64>,
    /// Design flow from the design; absent means 9.45.
    #[serde(default)]
    pub design_flow_m3_per_h: Option<f64>,
    /// EEI under EU 622/2012; absent means 0,23 or 0,25.
    #[serde(default)]
    pub energy_efficiency_index: Option<f64>,
    /// Label or design electric power of all pumps (9.47), kW.
    #[serde(default)]
    pub electric_power_kw: Option<f64>,
    pub source_reference: String,
}

// Input data read once per calculation; variant size does not matter here.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Generator {
    GasBoiler(GasBoilerGenerator),
    HeatPumpForfait(HeatPumpGenerator),
    /// Heat pump with a supplementary boiler, split by table 9.1/9.23 (new build).
    HybridHeatPump(Box<HybridGenerator>),
    /// Individual electric heat pump with measured product data (annex Q),
    /// optionally with a supplementary heater for `1 − F_H;gen`.
    HeatPumpAnnexQ(Box<AnnexQGenerator>),
    /// External heat supply (9.6.7): heat is the energy carrier `dh`.
    ExternalHeat(ExternalHeatGenerator),
    /// Local or central electric resistance heating, COP 1,0 (table 9.27).
    ElectricResistance(ElectricResistanceGenerator),
    /// Solid-biomass stove or boiler, forfait efficiency of table 9.30.
    Biomass(BiomassGenerator),
    /// Gas, oil or biomass boiler with product values (annex M; 9.6.2.2 and
    /// 9.6.5.2 method 1).
    ProductBoiler(Box<ProductBoilerGenerator>),
    /// Local heater, air heater, radiant heater or stove with product or
    /// default values (annex N).
    LocalHeater(Box<LocalHeaterGenerator>),
    /// Table 9.25 "overige systemen": local gas or oil heating and
    /// direct-fired air heaters (forfait).
    ForfaitHeater(ForfaitHeaterGenerator),
    /// Building CHP with heat-led operation, method 2 (9.6.6.1, table 9.31),
    /// gas.
    Chp(ChpGenerator),
    /// Several unequal generators on one system, split by preference with
    /// 9.56–9.60 and table 9.23 (9.6.1).
    Multiple(Box<MultipleGenerators>),
}

/// 9.6.6.1 building CHP (gas, forfait conversion factors of table 9.31).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChpGenerator {
    /// Method 2 (9.6.6.1): table 9.31 class; exclusive with `method1`.
    #[serde(default)]
    pub chp: Option<crate::space_cooling::ChpClass>,
    /// Method 1 (9.6.6.2): NEN-EN 50465 test values of a micro-CHP.
    #[serde(default)]
    pub method1: Option<crate::micro_chp::MicroChp>,
    /// 9.6.8.2 (9.91) inputs.
    #[serde(default)]
    pub auxiliary: Option<OtherGeneratorAuxiliary>,
    pub equipment_reference: String,
}

/// 9.6.1: generators with their preference and nominal power.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MultipleGenerators {
    pub generators: Vec<PreferredGenerator>,
    /// 9.58/9.59: renovation or a changed installation where a preferred
    /// generator was added; β then relates the installed power times
    /// f_gebouw;si;H to Φ_H;tot = Σ Q_H;node;in / 1139.
    #[serde(default)]
    pub added_preferred_generator: bool,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreferredGenerator {
    /// 1 is the highest priority (9.2.2.1.3, table 9.1).
    pub preference: u32,
    /// Φ_H;gen;i nominal heating power, kW (type plate, NEN-EN 14511-2 for
    /// heat pumps, at most 40 % of a combi boiler's maximum).
    pub nominal_power_kw: f64,
    pub generator: Generator,
}

/// Table 9.23 f_H;gen;i;mi(β) for β = 0; 0,1; …; 1: October–April and
/// May–September.
const TABLE_9_23_WINTER: [f64; 11] = [
    0.0, 0.20, 0.40, 0.59, 0.75, 0.87, 0.95, 0.98, 0.99, 1.00, 1.00,
];
const TABLE_9_23_SUMMER: [f64; 11] = [
    0.0, 0.57, 0.87, 0.95, 0.98, 1.00, 1.00, 1.00, 1.00, 1.00, 1.00,
];
/// 9.59: 0,13 × the year length, rounded.
const FULL_LOAD_HOURS_9_59: f64 = 1139.0;

/// Table 9.23 with linear interpolation; `month_index` 0 = January.
pub fn preferred_energy_fraction(beta: f64, month_index: usize) -> f64 {
    let table = if (4..=8).contains(&month_index) {
        &TABLE_9_23_SUMMER
    } else {
        &TABLE_9_23_WINTER
    };
    let x = (beta.clamp(0.0, 1.0)) * 10.0;
    let low = (x.floor() as usize).min(9);
    let t = x - low as f64;
    table[low] + t * (table[low + 1] - table[low])
}

/// Annex M boiler in the chain.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductBoilerGenerator {
    pub boiler: ProductBoiler,
    /// Design temperature class (table 9.14) for `ϑ_H,out` when the
    /// distribution is not calculated with `distributionSystem`.
    #[serde(default)]
    pub design_temperature_class: Option<DesignTemperatureClass>,
    /// Biomass: appliance of at most 500 kW meeting annex R (bmB).
    #[serde(default)]
    pub annex_r_compliant_at_most_500_kw: Option<bool>,
    #[serde(default)]
    pub annex_r_reference: Option<String>,
}

/// Fuel of an annex N heater.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalHeaterFuel {
    NaturalGas,
    Oil,
    /// Solid biomass or wood pellets (carrier `bm`).
    Biomass,
}

impl LocalHeaterFuel {
    /// N.3 `H_s;fuel/H_i;fuel`, taken from table M.3 (biomass as wood).
    pub fn gross_to_net(self) -> f64 {
        match self {
            Self::NaturalGas => BoilerFuel::NaturalGas.gross_to_net(),
            Self::Oil => BoilerFuel::Oil.gross_to_net(),
            Self::Biomass => BoilerFuel::Wood.gross_to_net(),
        }
    }
}

/// Annex N heater in the chain.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalHeaterGenerator {
    pub heater: LocalHeater,
    pub fuel: LocalHeaterFuel,
    /// Biomass: annex R compliance (bmB) and §9.6.5 sole heating.
    #[serde(default)]
    pub annex_r_compliant_at_most_500_kw: Option<bool>,
    #[serde(default)]
    pub annex_r_reference: Option<String>,
    #[serde(default)]
    pub sole_heating_in_served_rooms: Option<bool>,
}

/// Table 9.25 rows of "overige systemen".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForfaitHeaterKind {
    /// Local gas heating incl. pilot, oil heating or steam boiler with flue: 0,65.
    LocalWithFlue,
    /// Without flue (incl. flueless decorative fires): 0,10.
    LocalWithoutFlue,
    AirHeaterConventional,
    AirHeaterVr,
    AirHeaterHr100,
    AirHeaterHr104,
    AirHeaterHr107,
}

impl ForfaitHeaterKind {
    /// Table 9.25.
    pub fn efficiency(self) -> f64 {
        match self {
            Self::LocalWithFlue => 0.65,
            Self::LocalWithoutFlue => 0.10,
            Self::AirHeaterConventional => 0.75,
            Self::AirHeaterVr => 0.80,
            Self::AirHeaterHr100 => 0.90,
            Self::AirHeaterHr104 => 0.925,
            Self::AirHeaterHr107 => 0.95,
        }
    }

    fn air_heater(self) -> bool {
        !matches!(self, Self::LocalWithFlue | Self::LocalWithoutFlue)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForfaitHeaterFuel {
    NaturalGas,
    Oil,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForfaitHeaterGenerator {
    /// `heaterKind`: `kind` is the generator tag.
    #[serde(rename = "heaterKind")]
    pub kind: ForfaitHeaterKind,
    pub fuel: ForfaitHeaterFuel,
    pub equipment_reference: String,
    /// Number of pilot flames of gas air heaters (§9.6.2.1: 695 kWh each per
    /// year; the table 9.25 values exclude them). Local heating with flue
    /// already includes its pilot.
    #[serde(default)]
    pub pilot_flames: u32,
    /// 9.91 inputs (devices and burner power).
    #[serde(default)]
    pub auxiliary: Option<OtherGeneratorAuxiliary>,
}

/// 9.91/9.92 inputs for generators outside 9.85.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OtherGeneratorAuxiliary {
    /// Devices or panels with an electricity connection (10 W stand-by each).
    pub electrically_connected_devices: u32,
    /// `P_H;gen;gi` in kW; required for burners and automatic biomass feed.
    #[serde(default)]
    pub nominal_power_kw: Option<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ElectricResistanceGenerator {
    pub equipment_reference: String,
    #[serde(default)]
    pub auxiliary: Option<OtherGeneratorAuxiliary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomassAppliance {
    FreestandingWoodStove,
    InsertStove,
    PelletStove,
    AccumulatingStove,
    CentralBoiler,
}

impl BiomassAppliance {
    fn is_stove(self) -> bool {
        self != Self::CentralBoiler
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomassLocation {
    InsideThermalBoundary,
    OutsideThermalBoundary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BiomassGenerator {
    pub appliance: BiomassAppliance,
    pub location: BiomassLocation,
    /// Table 9.30 and the bmB factors apply to appliances of at most 500 kW
    /// that meet the combustion and emission limits of annex R.
    pub annex_r_compliant_at_most_500_kw: bool,
    pub annex_r_reference: String,
    pub equipment_reference: String,
    /// §9.6.5: a stove counts only when it is the only heating in the rooms
    /// it serves.
    #[serde(default)]
    pub sole_heating_in_served_rooms: Option<bool>,
    /// Automatic fuel feed, ash removal or cleaning (10 W/kW in 9.91);
    /// central boilers are taken as automatically fired.
    #[serde(default)]
    pub automatic_fuel_feed: bool,
    #[serde(default)]
    pub auxiliary: Option<OtherGeneratorAuxiliary>,
}

/// Table 9.30 forfait efficiency; `None` where the table has no value.
pub fn biomass_efficiency(appliance: BiomassAppliance, location: BiomassLocation) -> Option<f64> {
    use BiomassAppliance::*;
    match (appliance, location) {
        (
            FreestandingWoodStove | InsertStove | AccumulatingStove,
            BiomassLocation::InsideThermalBoundary,
        ) => Some(0.600),
        (PelletStove, BiomassLocation::InsideThermalBoundary) => Some(0.725),
        (CentralBoiler, BiomassLocation::InsideThermalBoundary) => Some(0.800),
        (CentralBoiler, BiomassLocation::OutsideThermalBoundary) => Some(0.750),
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalHeatGenerator {
    /// Invoice, contract or other proof of external supply (9.6.7.1).
    pub supplier_reference: String,
    /// A quality declaration (annex P); its values are given in the
    /// building performance input (`externalSupply.heating`), which also
    /// calculates the paired forfait scenario (§5.3.1).
    pub quality_declaration_present: bool,
    /// §9.6.7.2 routes the auxiliary energy through 9.91.
    #[serde(default)]
    pub auxiliary: Option<OtherGeneratorAuxiliary>,
}

impl Generator {
    /// The electric heat pump of this generator, if any.
    pub fn heat_pump(&self) -> Option<(&ForfaitHeatPumpDraftInput, SourceSystem)> {
        match self {
            Self::GasBoiler(_) => None,
            Self::HeatPumpForfait(generator) => Some((&generator.forfait, generator.source_system)),
            Self::HybridHeatPump(generator) => Some((&generator.forfait, generator.source_system)),
            Self::HeatPumpAnnexQ(_)
            | Self::ExternalHeat(_)
            | Self::ElectricResistance(_)
            | Self::Biomass(_)
            | Self::ProductBoiler(_)
            | Self::LocalHeater(_)
            | Self::ForfaitHeater(_)
            | Self::Chp(_) => None,
            Self::Multiple(set) => set
                .generators
                .iter()
                .find_map(|part| part.generator.heat_pump()),
        }
    }

    /// The annex Q heat pump of this generator, if any.
    pub fn annex_q(&self) -> Option<&AnnexQGenerator> {
        match self {
            Self::HeatPumpAnnexQ(generator) => Some(generator),
            Self::Multiple(set) => set
                .generators
                .iter()
                .find_map(|part| part.generator.annex_q()),
            _ => None,
        }
    }

    /// 9.4.4: generators whose 9.85 auxiliary energy includes the pump.
    fn auxiliary_includes_pump(&self) -> bool {
        match self {
            // Q.1: the circulation pump is part of the annex Q efficiency.
            Self::HeatPumpAnnexQ(_) => true,
            Self::GasBoiler(generator) => generator.boiler.role != BoilerRole::Collective,
            Self::HeatPumpForfait(generator) => {
                generator.forfait.collective_building_installation != Some(true)
            }
            Self::HybridHeatPump(generator) => {
                generator.forfait.collective_building_installation != Some(true)
            }
            Self::ExternalHeat(_)
            | Self::ElectricResistance(_)
            | Self::Biomass(_)
            | Self::ProductBoiler(_)
            | Self::LocalHeater(_)
            | Self::ForfaitHeater(_)
            | Self::Chp(_) => false,
            Self::Multiple(set) => set
                .generators
                .iter()
                .all(|part| part.generator.auxiliary_includes_pump()),
        }
    }

    /// Table 9.21: full-load generator spread `Δϑ_h;g;ontw`; `None` means
    /// external heat, which takes the emitter design spread.
    fn generator_spread_k(&self) -> Option<f64> {
        match self {
            Self::HeatPumpForfait(_) | Self::HybridHeatPump(_) | Self::HeatPumpAnnexQ(_) => {
                Some(10.0)
            }
            Self::ExternalHeat(_) => None,
            // The preferred generator decides the design spread.
            Self::Multiple(set) => set
                .generators
                .iter()
                .min_by_key(|part| part.preference)
                .and_then(|part| part.generator.generator_spread_k()),
            _ => Some(20.0),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HybridGenerator {
    /// Only the new-build installed-power route of the draft is supported.
    pub design_context: String,
    /// Heat pump and boiler with class, power and priority efficiency (table 9.1).
    pub generators: Vec<DispatchGenerator>,
    pub forfait: ForfaitHeatPumpDraftInput,
    pub boiler: BoilerForfaitDraftInput,
    pub source_system: SourceSystem,
    pub source_system_reference: String,
    #[serde(default)]
    pub declared_operating_limits_present: bool,
    #[serde(default)]
    pub heat_pump_auxiliary_measurements: Option<HybridHeatPumpAuxMeasurements>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasBoilerGenerator {
    pub boiler: BoilerForfaitDraftInput,
    /// Individual appliances: component measurements of annex O for the
    /// constants A, B and C of 9.85 (9.86–9.90); absent means the forfait.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary_measurements: Option<AppliancePowerMeasurements>,
    /// Collective boilers: 9.91 inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary: Option<OtherGeneratorAuxiliary>,
}

/// Annex Q heat pump (§9.6.3, required above 55 °C).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnnexQGenerator {
    pub heat_pump: AnnexQHeatPump,
    /// `θ_sup` for tables Q.5 and Q.7, °C (up to 75).
    pub design_supply_temperature_c: f64,
    /// Supplementary heater; required when `F_H;gen < 1` (Q.1).
    #[serde(default)]
    pub backup: Option<AnnexQBackup>,
    /// Annex V regeneration of an individual ground heat exchanger.
    #[serde(default)]
    pub regeneration: Option<RegenerationInput>,
    pub equipment_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnnexQBackup {
    /// Electric resistance, COP 1,0 (table 9.27).
    ElectricResistance,
    /// Gas boiler with the table 9.25 forfait efficiency.
    GasBoiler { boiler: BoilerForfaitDraftInput },
}

/// Building data that annex Q (table Q.6) and annex V need from the zones.
#[derive(Debug, Clone, Copy)]
struct HeatPumpBuildingContext {
    residential: bool,
    heating_need_kwh: f64,
    cooling_need_kwh: f64,
    usable_floor_area_m2: f64,
}

/// `η_el = 1/f_P;del;el` (table 5.2) for V.1.
const ELECTRICITY_EFFICIENCY: f64 = 1.0 / 1.45;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatPumpGenerator {
    pub forfait: ForfaitHeatPumpDraftInput,
    pub source_system: SourceSystem,
    pub source_system_reference: String,
    /// Measured auxiliary powers of an individual heat pump (9.85–9.88);
    /// absent means the 9.85 forfait constants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary_measurements: Option<HybridHeatPumpAuxMeasurements>,
    /// Collective heat pumps: 9.91 inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary: Option<OtherGeneratorAuxiliary>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainMonth {
    pub month: u8,
    pub heating_need_kwh: f64,
    pub emission_loss_kwh: f64,
    pub emission_input_kwh: f64,
    pub distribution_loss_kwh: f64,
    /// 9.40: pump energy transferred to the medium.
    pub distribution_auxiliary_to_medium_kwh: f64,
    /// 9.2.3: buffer vessel loss at the node.
    pub node_loss_kwh: f64,
    /// 9.2.3.4 `Q_H;nod;gns` from solar combi systems, kWh.
    pub solar_gain_kwh: f64,
    pub generator_output_kwh: f64,
    /// Part of the generator output delivered by an electric heat pump.
    pub heat_pump_output_kwh: f64,
    pub natural_gas_kwh: f64,
    /// Delivered external heat, carrier `dh` (9.84).
    pub district_heat_kwh: f64,
    /// Solid biomass input, carrier `bm` (9.64).
    pub biomass_kwh: f64,
    /// Fuel oil input (annex M/N, table 9.25 oil appliances).
    pub oil_kwh: f64,
    /// Annex M/N/CHP generator losses recoverable in the space (M.16/M.19,
    /// 9.80); fed back into 7.3–7.8 through 9.7.
    pub generator_recoverable_loss_kwh: f64,
    pub generator_electricity_kwh: f64,
    /// 9.6: generator plus distribution auxiliary energy.
    pub auxiliary_electricity_kwh: Option<f64>,
    /// 9.51.
    pub distribution_auxiliary_electricity_kwh: f64,
    /// 9.21/9.22 W_H;em;aux of room fans, kWh.
    pub emission_fan_electricity_kwh: f64,
    /// 9.7: recoverable losses summed over the zones.
    pub recoverable_loss_kwh: f64,
    pub collective_source_heat_kwh: f64,
    /// 12.1: latent heat of atomising humidifiers delivered by this
    /// heating system (in `generator_output_kwh`), kWh.
    pub humidification_load_kwh: f64,
    /// 12.3: steam humidifier energy, kWh.
    pub humidification_electricity_kwh: f64,
    pub humidification_fuel_kwh: f64,
    /// 11.120 Q_H;AHU;in;req of air handling unit reheating coils, kWh.
    pub ahu_heating_load_kwh: f64,
    /// 16.12 E_el;chp;out;H: electricity of a heating CHP, kWh.
    pub chp_electricity_kwh: f64,
    /// 13.185 hot-water load from §13.8.4.9.3, kWh.
    pub hot_water_load_kwh: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZoneHumidifier {
    pub zone_id: String,
    pub humidification: crate::humidification::Humidification,
}

fn humidity_function(function: crate::monthly_demand::UsageFunction) -> HumidityFunction {
    use crate::monthly_demand::UsageFunction as U;
    match function {
        U::Residential => HumidityFunction::Residential,
        U::Sport => HumidityFunction::Sport,
        U::HealthcareWithBeds | U::OtherHealthcare => HumidityFunction::Healthcare,
        _ => HumidityFunction::General,
    }
}

/// 12.1–12.4 for one zone with its chapter 11 supply flows.
fn zone_humidity(
    humidifier: &ZoneHumidifier,
    zone_input: &MonthlyDemandInput,
    zone_demand: &MonthlyDemandAssessment,
    installation_area_m2: f64,
    path: &str,
    issues: &mut Vec<ChainIssue>,
) -> Option<Vec<crate::humidification::HumidityMonth>> {
    let Some(ventilation) = &zone_demand.ventilation else {
        issues.push(issue(
            "humidification_requires_chapter_11",
            path.to_string(),
        ));
        return None;
    };
    let functions = if zone_input.function_areas.is_empty() {
        vec![HumidityFunctionArea {
            function: humidity_function(zone_input.usage_function),
            area_m2: zone_input.usable_floor_area_m2,
        }]
    } else {
        zone_input
            .function_areas
            .iter()
            .map(|part| HumidityFunctionArea {
                function: humidity_function(part.function),
                area_m2: part.area_m2,
            })
            .collect()
    };
    let input = HumidityInput {
        zone_id: zone_input.zone_id.clone(),
        usable_floor_area_m2: zone_input.usable_floor_area_m2,
        installation_area_m2: Some(installation_area_m2),
        functions,
        humidification: Some(humidifier.humidification.clone()),
        supply_flow_m3_per_h: ventilation
            .months
            .iter()
            .map(|month| month.heating.mechanical_supply_m3_per_h)
            .collect(),
        cooling_design: None,
        cooling_need_kwh: Vec::new(),
    };
    match calculate_humidity(&input) {
        Ok(months) => Some(months),
        Err(found) => {
            issues.extend(
                found
                    .into_iter()
                    .map(|item| issue(item.code, format!("{path}.{}", item.path))),
            );
            None
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneRecoverableLoss {
    pub zone_id: String,
    /// `Q_H;ls;rbl;zi` per month (9.7: 9.38 + 9.39), kWh.
    pub monthly_kwh: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneDistributionSummary {
    pub zone_id: String,
    /// 9.28 step 5.
    pub heating_limit_c: i32,
    /// 9.32a.
    pub operating_hours: Vec<f64>,
    /// 9.30.
    pub mean_medium_temperature_c: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PumpSummary {
    pub max_pipe_length_m: f64,
    pub pressure_kpa: f64,
    pub design_flow_m3_per_h: f64,
    pub hydraulic_power_kw: f64,
    pub energy_factor: f64,
    pub balancing_factor: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionSummary {
    pub design_temperature_class: DesignTemperatureClass,
    pub building_fraction: f64,
    pub pipe_length_m: f64,
    pub unheated_pipe_length_m: f64,
    pub psi_zone_w_per_mk: f64,
    pub psi_unheated_w_per_mk: f64,
    pub zones: Vec<ZoneDistributionSummary>,
    pub pump: Option<PumpSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceHeatingChainAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub chapter_9_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub final_edition_verified: bool,
    pub reference_verified: bool,
    pub beng_calculation_available: bool,
    pub omitted_terms: &'static [&'static str],
    pub emission_temperature_increment_k: Option<f64>,
    pub generation_efficiency: Option<f64>,
    pub monthly: Vec<ChainMonth>,
    pub annual_natural_gas_kwh: Option<f64>,
    pub annual_generator_electricity_kwh: Option<f64>,
    pub annual_auxiliary_electricity_kwh: Option<f64>,
    pub annual_collective_source_heat_kwh: Option<f64>,
    pub annual_district_heat_kwh: Option<f64>,
    pub annual_biomass_kwh: Option<f64>,
    pub distribution: Option<DistributionSummary>,
    pub zone_recoverable_losses: Vec<ZoneRecoverableLoss>,
    /// Annex Q (and annex V) details of an annex Q heat pump.
    pub annex_q: Option<AnnexQOutput>,
    pub demand: MonthlyDemandAssessment,
    pub additional_zone_demands: Vec<MonthlyDemandAssessment>,
    pub issues: Vec<ChainIssue>,
}

fn issue(code: &'static str, path: impl Into<String>) -> ChainIssue {
    ChainIssue {
        code,
        path: path.into(),
    }
}

fn twelve_finite(values: &[f64]) -> bool {
    values.len() == 12 && values.iter().all(|value| value.is_finite())
}

/// Monthly terms of one valid zone.
struct ZoneTerms {
    zone_id: String,
    area_m2: f64,
    setpoint_c: f64,
    increment_k: f64,
    hydronic: bool,
    balanced: bool,
    emitter: EmitterResistance,
    need: [f64; 12],
    emission_loss: [f64; 12],
    /// Declared or zero loss; `None` for the calculated route.
    fixed_loss: Option<[f64; 12]>,
    heating_limit_extra: [f64; 12],
    /// Σ P_fan·n_fan of 9.22, W.
    fan_power_w: f64,
}

/// Validates one zone; `None` when invalid.
fn zone_terms(
    demand_input: &MonthlyDemandInput,
    demand: &MonthlyDemandAssessment,
    emission: &EmissionInput,
    distribution: &Distribution,
    prefix: &str,
    issues: &mut Vec<ChainIssue>,
) -> Option<ZoneTerms> {
    let prior = issues.len();
    issues.extend(
        demand
            .issues
            .iter()
            .map(|item| issue(item.code, format!("{prefix}demand.{}", item.path))),
    );
    if emission.source_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            format!("{prefix}emission.sourceReference"),
        ));
    }
    if !balancing_consistent(emission) {
        issues.push(issue(
            "emission_balancing_inconsistent",
            format!("{prefix}emission.balancing"),
        ));
    }
    // 9.21/9.22: room fans need their count and type (table 9.11).
    match &emission.fans {
        None if emission.system == EmissionSystem::FanAssistedRadiatorsOrConvectors => {
            issues.push(issue(
                "emission_fans_required",
                format!("{prefix}emission.fans"),
            ));
        }
        Some(fans) => {
            if fans.count == 0 {
                issues.push(issue(
                    "emission_fan_count_invalid",
                    format!("{prefix}emission.fans.count"),
                ));
            }
            if fans
                .tested_power_w
                .is_some_and(|power| !power.is_finite() || power < 0.0)
            {
                issues.push(issue(
                    "emission_fan_power_invalid",
                    format!("{prefix}emission.fans.testedPowerW"),
                ));
            }
            if fans.source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{prefix}emission.fans.sourceReference"),
                ));
            }
        }
        None => {}
    }
    let mut extra = [0.0; 12];
    let (fixed_loss, reference) = match distribution {
        Distribution::HeatedZoneOnlySpaceHeating { source_reference } => {
            (Some([0.0; 12]), source_reference)
        }
        Distribution::Declared {
            monthly_loss_kwh,
            source_reference,
        } => {
            if !twelve_finite(monthly_loss_kwh) || monthly_loss_kwh.iter().any(|value| *value < 0.0)
            {
                issues.push(issue(
                    "distribution_monthly_loss_invalid",
                    format!("{prefix}distribution.monthlyLossKwh"),
                ));
                (None, source_reference)
            } else {
                // 9.24: no zeroing outside the heating season; t_H,op is non-zero there.
                (
                    Some(std::array::from_fn(|index| monthly_loss_kwh[index])),
                    source_reference,
                )
            }
        }
        Distribution::Calculated {
            heating_limit_extra_kwh,
            source_reference,
        } => {
            if let Some(values) = heating_limit_extra_kwh {
                if !twelve_finite(values) || values.iter().any(|value| *value < 0.0) {
                    issues.push(issue(
                        "heating_limit_extra_invalid",
                        format!("{prefix}distribution.heatingLimitExtraKwh"),
                    ));
                } else if !demand.heating_limit_need_kwh.is_empty() {
                    // Chapter 11 already yields the 9.28/9.29 terms.
                    issues.push(issue(
                        "heating_limit_extra_and_chapter_11_exclusive",
                        format!("{prefix}distribution.heatingLimitExtraKwh"),
                    ));
                } else {
                    extra = std::array::from_fn(|index| values[index]);
                }
            } else if demand.heating_limit_need_kwh.len() == 12 {
                // 9.28/9.29 from the chapter 11 flows.
                extra = std::array::from_fn(|index| {
                    (demand.heating_limit_need_kwh[index] - demand.monthly[index].heating.need_kwh)
                        .max(0.0)
                });
            }
            (None, source_reference)
        }
    };
    if reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            format!("{prefix}distribution.sourceReference"),
        ));
    }
    if issues.len() != prior {
        return None;
    }
    let increment = temperature_increment_k(emission);
    let setpoint = demand_input.setpoints.heating_c;
    let need: [f64; 12] = std::array::from_fn(|index| demand.monthly[index].heating.need_kwh);
    // 9.10 and 9.12a.
    let emission_loss: [f64; 12] = std::array::from_fn(|index| {
        monthly_loss_kwh(
            need[index],
            setpoint,
            OUTDOOR_TEMPERATURE_C[index],
            increment,
        )
    });
    Some(ZoneTerms {
        zone_id: demand_input.zone_id.clone(),
        area_m2: demand_input.usable_floor_area_m2,
        setpoint_c: setpoint,
        increment_k: increment,
        hydronic: !matches!(
            emission.system,
            EmissionSystem::LocalHeater | EmissionSystem::AirHeating
        ),
        balanced: matches!(
            emission.balancing,
            HydronicBalancing::Static | HydronicBalancing::Dynamic
        ),
        emitter: match emission.system {
            EmissionSystem::RadiatorsOrConvectors
            | EmissionSystem::FanAssistedRadiatorsOrConvectors => {
                EmitterResistance::RadiatorConvectorOrDarkRadiator
            }
            _ => EmitterResistance::SurfaceAirOrOther,
        },
        need,
        emission_loss,
        fixed_loss,
        heating_limit_extra: extra,
        fan_power_w: emission
            .fans
            .as_ref()
            .map_or(0.0, |fans| fans.power_w() * f64::from(fans.count)),
    })
}

/// Results of the distribution calculation for all zones.
struct DistributionResult {
    /// Per zone and month: loss, recoverable loss, pump energy to medium.
    zone_loss: Vec<[f64; 12]>,
    zone_recoverable: Vec<[f64; 12]>,
    zone_aux_to_medium: Vec<[f64; 12]>,
    pump_electricity: [f64; 12],
    node_loss: [f64; 12],
    summary: Option<DistributionSummary>,
}

fn validate_distribution_system(system: &DistributionSystem, issues: &mut Vec<ChainIssue>) {
    let path = "distributionSystem";
    if system.source_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            format!("{path}.sourceReference"),
        ));
    }
    if system.connected_storeys == 0 {
        issues.push(issue(
            "connected_storeys_invalid",
            format!("{path}.connectedStoreys"),
        ));
    }
    for (value, field) in [
        (system.actual_pipe_length_m, "actualPipeLengthM"),
        (system.unheated_pipe_length_m, "unheatedPipeLengthM"),
    ] {
        if value.is_some_and(|length| !length.is_finite() || length < 0.0) {
            issues.push(issue("pipe_length_invalid", format!("{path}.{field}")));
        }
    }
    if let Some(values) = &system.unheated_ambient_c {
        if !twelve_finite(values) {
            issues.push(issue(
                "unheated_ambient_invalid",
                format!("{path}.unheatedAmbientC"),
            ));
        }
    }
    if system.collective_hot_water_delivery_set && system.installation != Installation::Collective {
        issues.push(issue(
            "delivery_set_requires_collective_installation",
            format!("{path}.collectiveHotWaterDeliverySet"),
        ));
    }
    if let Some(buffer) = &system.buffer_vessel {
        let buffer_path = format!("{path}.bufferVessel");
        if system.installation != Installation::Collective {
            issues.push(issue("buffer_vessel_collective_only", buffer_path.clone()));
        }
        if system.pipe_transmittance.is_forfait() {
            // Table 9.16 note 2: the forfait Ψ already includes buffer losses.
            issues.push(issue(
                "buffer_vessel_included_in_forfait_psi",
                buffer_path.clone(),
            ));
        }
        if !buffer.volume_l.is_finite() || buffer.volume_l <= 0.0 {
            issues.push(issue(
                "buffer_volume_invalid",
                format!("{buffer_path}.volumeL"),
            ));
        }
        if buffer
            .standing_loss_w
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            issues.push(issue(
                "buffer_standing_loss_invalid",
                format!("{buffer_path}.standingLossW"),
            ));
        }
        if buffer.source_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                format!("{buffer_path}.sourceReference"),
            ));
        }
    }
    match &system.pump {
        DistributionPump::IncludedInGeneratorAuxiliary => {}
        DistributionPump::NoneOnSite { source_reference } => {
            if source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{path}.pump.sourceReference"),
                ));
            }
        }
        DistributionPump::Calculated(pump) => {
            if pump.source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{path}.pump.sourceReference"),
                ));
            }
            for (value, field) in [
                (pump.max_pipe_length_m, "maxPipeLengthM"),
                (pump.design_flow_m3_per_h, "designFlowM3PerH"),
                (pump.energy_efficiency_index, "energyEfficiencyIndex"),
                (pump.electric_power_kw, "electricPowerKw"),
            ] {
                if value.is_some_and(|number| !number.is_finite() || number <= 0.0) {
                    issues.push(issue("pump_value_invalid", format!("{path}.pump.{field}")));
                }
            }
        }
    }
}

/// 9.26–9.51 and the 9.2.3 buffer loss. Call only with valid inputs.
fn calculate_distribution(
    zones: &[ZoneTerms],
    system: Option<&DistributionSystem>,
    building_fraction: f64,
    connected_area_m2: f64,
    generator: &Generator,
    issues: &mut Vec<ChainIssue>,
) -> DistributionResult {
    let mut result = DistributionResult {
        zone_loss: zones
            .iter()
            .map(|zone| zone.fixed_loss.unwrap_or([0.0; 12]))
            .collect(),
        zone_recoverable: vec![[0.0; 12]; zones.len()],
        zone_aux_to_medium: vec![[0.0; 12]; zones.len()],
        pump_electricity: [0.0; 12],
        node_loss: [0.0; 12],
        summary: None,
    };
    let Some(system) = system else {
        return result;
    };
    let class = system
        .design_temperature_class
        .unwrap_or(DesignTemperatureClass::C90);
    let (design_supply, design_spread) = class.design();
    let full_month = system.pipes_also_for_hot_water || system.collective_hot_water_delivery_set;
    // Table 9.X: pump fraction 0,10 for dwellings with an individual installation.
    let pump_fraction = if system.usage_function == ReductionFunction::Residential
        && system.installation == Installation::Individual
    {
        0.10
    } else {
        1.0
    };
    let reduction = system.usage_function.heating_reduction_factor();
    let zone_area: f64 = zones.iter().map(|zone| zone.area_m2).sum();

    // Per zone: heating limit, operating hours (9.32a) and medium temperatures.
    let mut limits = Vec::with_capacity(zones.len());
    let mut hours = Vec::with_capacity(zones.len());
    let mut supply_return = Vec::with_capacity(zones.len());
    for (index, zone) in zones.iter().enumerate() {
        let stook_need: [f64; 12] =
            std::array::from_fn(|month| zone.need[month] + zone.heating_limit_extra[month]);
        let Some(limit) = heating_limit_c(&stook_need, zone.setpoint_c) else {
            issues.push(issue(
                "heating_limit_undetermined",
                if index == 0 {
                    "demand".to_string()
                } else {
                    format!("additionalZones[{}].demand", index - 1)
                },
            ));
            return result;
        };
        limits.push(limit);
        hours.push(std::array::from_fn::<f64, 12, _>(|month| {
            if full_month {
                MONTH_HOURS[month]
            } else {
                heating_limit_hours(limit, month) * reduction * pump_fraction
            }
        }));
        supply_return.push(std::array::from_fn::<(f64, f64), 12, _>(|month| {
            supply_return_c(
                zone.setpoint_c,
                class,
                limit,
                OUTDOOR_TEMPERATURE_C[month],
                zone.increment_k,
            )
        }));
    }
    let mean = |zone: usize, month: usize| {
        let (supply, ret) = supply_return[zone][month];
        let value = (supply + ret) / 2.0;
        if system.collective_hot_water_delivery_set {
            value.max(DELIVERY_SET_MIN_MEAN_C)
        } else {
            value
        }
    };

    // Pipe lengths and Ψ (9.27, 9.36, table 9.16 or 9.33–9.35).
    let total_length = system
        .actual_pipe_length_m
        .unwrap_or_else(|| forfait_pipe_length_m(connected_area_m2));
    let unheated_length = system
        .unheated_pipe_length_m
        .unwrap_or(UNKNOWN_UNHEATED_SHARE * total_length);
    if unheated_length > total_length {
        issues.push(issue(
            "pipe_length_invalid",
            "distributionSystem.unheatedPipeLengthM",
        ));
        return result;
    }
    let shared = system.pipes_also_for_hot_water && system.installation == Installation::Collective;
    let psi_zone = system.pipe_transmittance.value(connected_area_m2, shared);
    let psi_unheated = system
        .unheated_pipe_transmittance
        .as_ref()
        .unwrap_or(&system.pipe_transmittance)
        .value(connected_area_m2, shared);
    let (Some(psi_zone), Some(psi_unheated)) = (psi_zone, psi_unheated) else {
        issues.push(issue(
            "pipe_transmittance_invalid",
            "distributionSystem.pipeTransmittance",
        ));
        return result;
    };
    let in_zone_total = total_length - unheated_length;
    let in_zone_with_fittings =
        in_zone_total + equivalent_length_m(in_zone_total, psi_zone, system.valves_insulated);
    let unheated_with_fittings = unheated_length
        + equivalent_length_m(unheated_length, psi_unheated, system.valves_insulated);
    let unheated_ambient = |month: usize| {
        system
            .unheated_ambient_c
            .as_ref()
            .map_or(DEFAULT_UNHEATED_AMBIENT_C, |values| values[month])
    };

    // 9.26 and 9.38 for zones on the calculated route.
    for (index, zone) in zones.iter().enumerate() {
        if zone.fixed_loss.is_some() {
            continue;
        }
        let share = zone.area_m2 / zone_area;
        // 9.4.1: L_zi = 0 when the pipes serve space heating only.
        let zone_length = if system.pipes_also_for_hot_water {
            in_zone_with_fittings * share
        } else {
            0.0
        };
        for (month, t) in hours[index].iter().copied().enumerate() {
            let theta = mean(index, month);
            let in_zone = psi_zone * (theta - zone.setpoint_c) * zone_length * t / 1000.0;
            let unheated =
                psi_unheated * (theta - unheated_ambient(month)) * unheated_with_fittings * t
                    / 1000.0
                    * share;
            result.zone_loss[index][month] = (in_zone + unheated) * building_fraction;
            // 9.38 with f_H;dis;rbl = 1; losses in unheated spaces are not recoverable.
            result.zone_recoverable[index][month] = in_zone * building_fraction;
        }
    }

    // System operating hours: the longest of the zones per month.
    let system_hours: [f64; 12] =
        std::array::from_fn(|month| hours.iter().map(|zone| zone[month]).fold(0.0_f64, f64::max));

    // 9.41–9.51.
    let mut pump_summary = None;
    if let DistributionPump::Calculated(pump) = &system.pump {
        let hydronic: Vec<usize> = (0..zones.len())
            .filter(|&zone| zones[zone].hydronic)
            .collect();
        let max_length = pump.max_pipe_length_m.unwrap_or_else(|| {
            forfait_max_pipe_length_m(system.connected_storeys, connected_area_m2)
        });
        let emitter = hydronic
            .iter()
            .map(|&zone| zones[zone].emitter.kpa())
            .fold(0.0_f64, f64::max);
        let generator_spread = generator.generator_spread_k().unwrap_or(design_spread);
        let additional = emitter
            + if pump.heat_meter_present { 10.0 } else { 0.0 }
            + generator_resistance_kpa(design_spread, generator_spread);
        let pressure = design_pressure_kpa(max_length, additional);
        let flow = match pump.design_flow_m3_per_h {
            Some(flow) => flow,
            None => {
                // 9.45 at the month with the highest need for the heating limit.
                let totals: [f64; 12] = std::array::from_fn(|month| {
                    zones
                        .iter()
                        .map(|zone| zone.need[month] + zone.heating_limit_extra[month])
                        .sum()
                });
                let peak = (0..12)
                    .max_by(|a, b| totals[*a].total_cmp(&totals[*b]))
                    .unwrap_or(0);
                let spread = if system.collective_hot_water_delivery_set {
                    design_spread
                } else {
                    hydronic
                        .iter()
                        .map(|&zone| supply_return[zone][peak].0 - supply_return[zone][peak].1)
                        .fold(f64::INFINITY, f64::min)
                };
                let t = system_hours[peak];
                if !(spread.is_finite() && spread > 0.0 && t > 0.0 && totals[peak] > 0.0) {
                    issues.push(issue(
                        "distribution_design_flow_undetermined",
                        "distributionSystem.pump.designFlowM3PerH",
                    ));
                    return result;
                }
                design_flow_m3_per_h(totals[peak], t, spread, building_fraction)
            }
        };
        let hydraulic = hydraulic_power_kw(pressure, flow);
        let factor = pump_energy_factor(
            hydraulic,
            pump.energy_efficiency_index,
            pump.electric_power_kw,
        );
        let balance = balancing_factor(
            !hydronic.is_empty() && hydronic.iter().all(|&zone| zones[zone].balanced),
        );
        for (month, operating) in system_hours.iter().enumerate() {
            // 9.41 with β = 1 and 9.51 summed over the zones.
            let hydraulic_energy = hydraulic * operating * balance;
            let total = hydraulic_energy * factor * building_fraction;
            result.pump_electricity[month] = total;
            for (index, zone) in zones.iter().enumerate() {
                let share = total * zone.area_m2 / zone_area;
                result.zone_recoverable[index][month] += RECOVERABLE_AUX_FRACTION * share;
                result.zone_aux_to_medium[index][month] = (1.0 - RECOVERABLE_AUX_FRACTION) * share;
            }
        }
        pump_summary = Some(PumpSummary {
            max_pipe_length_m: max_length,
            pressure_kpa: pressure,
            design_flow_m3_per_h: flow,
            hydraulic_power_kw: hydraulic,
            energy_factor: factor,
            balancing_factor: balance,
        });
    }

    // 9.2.3: buffer vessel of a collective installation.
    if let Some(buffer) = &system.buffer_vessel {
        let standing = buffer.standing_loss_w.unwrap_or_else(|| {
            let label = buffer.label.unwrap_or(if buffer.produced_from_2018 {
                StorageLabel::C
            } else {
                StorageLabel::G
            });
            label_standing_loss_w(label, buffer.volume_l)
        });
        for month in 0..12 {
            let set = if buffer.constant_temperature {
                design_supply
            } else {
                supply_return
                    .iter()
                    .map(|zone| zone[month].0)
                    .fold(f64::NEG_INFINITY, f64::max)
            };
            let ambient = if buffer.in_heated_space {
                zones[0].setpoint_c
            } else {
                unheated_ambient(month)
            };
            result.node_loss[month] = buffer_loss_kwh(
                standing,
                set,
                ambient,
                system_hours[month],
                building_fraction,
            )
            .max(0.0);
        }
    }

    result.summary = Some(DistributionSummary {
        design_temperature_class: class,
        building_fraction,
        pipe_length_m: total_length,
        unheated_pipe_length_m: unheated_length,
        psi_zone_w_per_mk: psi_zone,
        psi_unheated_w_per_mk: psi_unheated,
        zones: zones
            .iter()
            .enumerate()
            .map(|(index, zone)| ZoneDistributionSummary {
                zone_id: zone.zone_id.clone(),
                heating_limit_c: limits[index],
                operating_hours: hours[index].to_vec(),
                mean_medium_temperature_c: (0..12).map(|month| mean(index, month)).collect(),
            })
            .collect(),
        pump: pump_summary,
    });
    result
}

fn validate_other_auxiliary(
    auxiliary: Option<&OtherGeneratorAuxiliary>,
    needs_power: bool,
    issues: &mut Vec<ChainIssue>,
) {
    match auxiliary {
        None => issues.push(issue(
            "generator_auxiliary_input_required",
            "generator.auxiliary",
        )),
        Some(auxiliary) => {
            if auxiliary.source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    "generator.auxiliary.sourceReference",
                ));
            }
            match auxiliary.nominal_power_kw {
                Some(power) if !power.is_finite() || power <= 0.0 => issues.push(issue(
                    "generator_nominal_power_invalid",
                    "generator.auxiliary.nominalPowerKw",
                )),
                None if needs_power => issues.push(issue(
                    "generator_nominal_power_required",
                    "generator.auxiliary.nominalPowerKw",
                )),
                _ => {}
            }
        }
    }
}

/// 9.91/9.92 with the rekenwaarden of 9.6.8.2.3 and `f_H;gen;ctr = 1`.
/// `output_kwh` is the output of the calculated building part.
pub fn other_generator_auxiliary_kwh(
    auxiliary: &OtherGeneratorAuxiliary,
    burner_w_per_kw: f64,
    output_kwh: f64,
    hours: f64,
    building_fraction: f64,
) -> f64 {
    let standby = OTHER_AUX_STANDBY_W * f64::from(auxiliary.electrically_connected_devices) * hours;
    let burner = match auxiliary.nominal_power_kw {
        Some(power) if burner_w_per_kw > 0.0 => {
            // 9.92 with Q_gen;out of the whole installation.
            let on = (output_kwh / building_fraction * 1.1 / power).min(hours);
            burner_w_per_kw * power * on
        }
        _ => 0.0,
    };
    (standby + burner) * building_fraction / 1000.0
}

/// 9.85 with the forfait constants for an individual electric heat pump.
pub fn heat_pump_forfait_auxiliary_kwh(electricity_kwh: f64) -> f64 {
    HEAT_PUMP_AUX_A_KWH / 12.0
        + HEAT_PUMP_AUX_B_KW * electricity_kwh / (HEAT_PUMP_AUX_C * HEAT_PUMP_AUX_B_NOM_KW)
}

pub fn assess_space_heating_chain(input: &SpaceHeatingChainInput) -> SpaceHeatingChainAssessment {
    match exhaust_air_recalculation(input) {
        Some(result) => result,
        None => assess_chain_once(input),
    }
}

/// Q.5.3: an annex Q heat pump using ventilation air whose chapter 11
/// overventilation leaves `heatingTimeFraction` empty. Steps 1–3 run the
/// chain with f_H;t;hp-on = 0, with the annex Q fraction of that run
/// (Q.90), and with the correction of Q.96/Q.97; Q.84 caps f_H + f_W at 1.
fn exhaust_air_recalculation(
    input: &SpaceHeatingChainInput,
) -> Option<SpaceHeatingChainAssessment> {
    if !matches!(input.generator, Generator::HeatPumpAnnexQ(_)) {
        return None;
    }
    let over = input
        .demand
        .ventilation
        .as_ref()?
        .overventilation
        .as_ref()?;
    if !over.heating_time_fraction.is_empty() || over.hot_water_time_fraction.len() != 12 {
        return None;
    }
    let hot_water = over.hot_water_time_fraction.clone();
    let cap = |month: usize, value: f64| value.clamp(0.0, (1.0 - hot_water[month]).max(0.0));
    let run = |fractions: &[f64; 12]| {
        let mut step = input.clone();
        if let Some(over) = step
            .demand
            .ventilation
            .as_mut()
            .and_then(|ventilation| ventilation.overventilation.as_mut())
        {
            over.heating_time_fraction = fractions.to_vec();
        }
        assess_chain_once(&step)
    };
    let node = |result: &SpaceHeatingChainAssessment| -> Option<[f64; 12]> {
        (result.monthly.len() == 12)
            .then(|| std::array::from_fn(|month| result.monthly[month].generator_output_kwh))
    };
    // Step 1.
    let first = run(&[0.0; 12]);
    let (Some(node_1), Some(annex_q)) = (node(&first), first.annex_q.as_ref()) else {
        return Some(first);
    };
    // Step 2 (Q.90 from the step 1 demand).
    let f_2: [f64; 12] =
        std::array::from_fn(|month| cap(month, annex_q.annex_q.monthly_on_fraction[month]));
    let second = run(&f_2);
    let Some(node_2) = node(&second) else {
        return Some(second);
    };
    // Step 3 (Q.96/Q.97).
    let f_3: [f64; 12] = std::array::from_fn(|month| {
        if node_2[month] > 0.0 {
            let ff = f_2[month] / node_2[month];
            cap(month, f_2[month] + (node_2[month] - node_1[month]) * ff)
        } else {
            f_2[month]
        }
    });
    let mut third = run(&f_3);
    // The fingerprint identifies the caller's input, not the derived step.
    third.input_fingerprint =
        input_fingerprint(&serde_json::to_value(input).expect("typed input serializes"));
    if let Some(output) = third.annex_q.as_mut() {
        output.exhaust_air_heating_time_fraction = Some(f_3.to_vec());
    }
    Some(third)
}

/// 9.7: the recoverable generator losses (9.6) belong to Q_H;ls;rbl of
/// 7.3–7.8, but follow from the generator output. A first pass gives them;
/// a second pass feeds them back, split over the zones by usable area (one
/// substitution step, documented as an interpretation).
fn assess_chain_once(input: &SpaceHeatingChainInput) -> SpaceHeatingChainAssessment {
    let first = assess_chain_pass(input, None);
    if first.monthly.len() != 12
        || first
            .monthly
            .iter()
            .all(|row| row.generator_recoverable_loss_kwh <= 0.0)
    {
        return first;
    }
    let losses: [f64; 12] =
        std::array::from_fn(|index| first.monthly[index].generator_recoverable_loss_kwh);
    assess_chain_pass(input, Some(&losses))
}

fn assess_chain_pass(
    input: &SpaceHeatingChainInput,
    generator_recoverable: Option<&[f64; 12]>,
) -> SpaceHeatingChainAssessment {
    let fingerprint =
        input_fingerprint(&serde_json::to_value(input).expect("typed input serializes"));
    let mut issues: Vec<ChainIssue> = Vec::new();
    let mut demand = assess_monthly_demand(&input.demand);
    let primary = zone_terms(
        &input.demand,
        &demand,
        &input.emission,
        &input.distribution,
        "",
        &mut issues,
    );
    let mut additional_zone_demands: Vec<MonthlyDemandAssessment> = input
        .additional_zones
        .iter()
        .map(|zone| assess_monthly_demand(&zone.demand))
        .collect();
    let mut zones = vec![primary];
    for (index, (zone, assessed)) in input
        .additional_zones
        .iter()
        .zip(&additional_zone_demands)
        .enumerate()
    {
        zones.push(zone_terms(
            &zone.demand,
            assessed,
            &zone.emission,
            &zone.distribution,
            &format!("additionalZones[{index}]."),
            &mut issues,
        ));
    }
    let mut zone_ids = std::collections::HashSet::new();
    for (index, id) in std::iter::once(&input.demand.zone_id)
        .chain(
            input
                .additional_zones
                .iter()
                .map(|zone| &zone.demand.zone_id),
        )
        .enumerate()
    {
        if !zone_ids.insert(id.as_str()) {
            issues.push(issue("zone_id_duplicate", format!("zones[{index}].zoneId")));
        }
    }

    // f_gebouw;si;H.
    let zone_area: f64 = std::iter::once(&input.demand)
        .chain(input.additional_zones.iter().map(|zone| &zone.demand))
        .map(|zone| zone.usable_floor_area_m2)
        .sum();
    if !input.solar_heating_kwh.is_empty()
        && (input.solar_heating_kwh.len() != 12
            || input
                .solar_heating_kwh
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0))
    {
        issues.push(issue("solar_heating_invalid", "solarHeatingKwh"));
    }
    if !input.hot_water_load_kwh.is_empty()
        && (input.hot_water_load_kwh.len() != 12
            || input
                .hot_water_load_kwh
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0))
    {
        issues.push(issue("hot_water_load_invalid", "hotWaterLoadKwh"));
    }
    let (connected_area, building_fraction) = match &input.collective_connection {
        None => (zone_area, 1.0),
        Some(connection) => {
            if connection.source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    "collectiveConnection.sourceReference",
                ));
            }
            let area = connection.connected_usable_area_m2;
            if !area.is_finite() || area <= 0.0 || area + 1e-9 < zone_area {
                issues.push(issue(
                    "connected_area_invalid",
                    "collectiveConnection.connectedUsableAreaM2",
                ));
                (zone_area, 1.0)
            } else {
                (area, zone_area / area)
            }
        }
    };

    let calculated_route = std::iter::once(&input.distribution)
        .chain(input.additional_zones.iter().map(|zone| &zone.distribution))
        .any(|item| matches!(item, Distribution::Calculated { .. }));
    let any_hydronic = std::iter::once(&input.emission)
        .chain(input.additional_zones.iter().map(|zone| &zone.emission))
        .any(|emission| {
            !matches!(
                emission.system,
                EmissionSystem::LocalHeater | EmissionSystem::AirHeating
            )
        });
    match &input.distribution_system {
        Some(system) => {
            validate_distribution_system(system, &mut issues);
            if matches!(system.pump, DistributionPump::IncludedInGeneratorAuxiliary)
                && !input.generator.auxiliary_includes_pump()
            {
                // 9.4.4: only 9.85 auxiliary energy includes the pump.
                issues.push(issue(
                    "distribution_pump_not_in_generator_auxiliary",
                    "distributionSystem.pump",
                ));
            }
        }
        None => {
            if calculated_route {
                issues.push(issue("distribution_system_required", "distributionSystem"));
            }
            if any_hydronic && !input.generator.auxiliary_includes_pump() {
                issues.push(issue(
                    "distribution_pump_input_required",
                    "distributionSystem",
                ));
            }
        }
    }

    // A single increment is only meaningful for one zone.
    let increment =
        (input.additional_zones.is_empty()).then(|| temperature_increment_k(&input.emission));
    let mut monthly = Vec::with_capacity(12);
    let mut generation_efficiency = None;
    let mut distribution_summary = None;
    let mut zone_recoverable_losses = Vec::new();
    let mut annex_q_result: Option<AnnexQOutput> = None;
    if issues.is_empty() {
        let valid_zones: Vec<ZoneTerms> = zones.into_iter().flatten().collect();
        let distribution = calculate_distribution(
            &valid_zones,
            input.distribution_system.as_ref(),
            building_fraction,
            connected_area,
            &input.generator,
            &mut issues,
        );
        let conditions = generator_conditions(input, &valid_zones, &distribution);
        // 7.3/7.7: the recoverable losses (9.2.5) reduce the heating need and
        // add to the cooling need; the heating limit (9.28) and the
        // distribution itself use the need without them.
        let zone_sources: Vec<(&MonthlyDemandInput, &EmissionInput, &Distribution)> =
            std::iter::once((&input.demand, &input.emission, &input.distribution))
                .chain(
                    input
                        .additional_zones
                        .iter()
                        .map(|zone| (&zone.demand, &zone.emission, &zone.distribution)),
                )
                .collect();
        // Chapter 12: humidifiers per zone.
        // Q_H;hum;rbl (12.4) of steam humidifiers joins the recoverable
        // losses of its zone (7.3–7.8).
        let mut humidification = [[0.0_f64; 3]; 12];
        let mut humidifier_recoverable = vec![[0.0_f64; 12]; zone_sources.len()];
        for (h_index, humidifier) in input.humidifiers.iter().enumerate() {
            let path = format!("humidifiers[{h_index}]");
            let found = zone_sources
                .iter()
                .zip(std::iter::once(&demand).chain(&additional_zone_demands))
                .enumerate()
                .find(|(_, ((zone_input, _, _), _))| zone_input.zone_id == humidifier.zone_id);
            let Some((zone_index, ((zone_input, _, _), zone_demand))) = found else {
                issues.push(issue("humidifier_zone_unknown", format!("{path}.zoneId")));
                continue;
            };
            if let Some(months) = zone_humidity(
                humidifier,
                zone_input,
                zone_demand,
                connected_area,
                &path,
                &mut issues,
            ) {
                for (index, month) in months.iter().enumerate() {
                    humidification[index][0] += month.heating_system_load_kwh;
                    humidification[index][1] += month.steam_electricity_kwh;
                    humidification[index][2] += month.steam_fuel_kwh;
                    humidifier_recoverable[zone_index][index] += month.recoverable_kwh;
                }
            }
        }
        let adjusted: Vec<MonthlyDemandAssessment> = std::iter::once(&demand)
            .chain(&additional_zone_demands)
            .zip(&distribution.zone_recoverable)
            .zip(&humidifier_recoverable)
            .zip(&zone_sources)
            .map(
                |(((assessed, recoverable), humidifier), (zone_input, _, _))| {
                    let share = if zone_area > 0.0 {
                        zone_input.usable_floor_area_m2 / zone_area
                    } else {
                        0.0
                    };
                    let total: Vec<f64> = (0..12)
                        .map(|index| {
                            recoverable.get(index).copied().unwrap_or(0.0)
                                + humidifier[index]
                                + generator_recoverable.map_or(0.0, |losses| losses[index] * share)
                        })
                        .collect();
                    apply_recoverable_losses(assessed, &total, &[])
                },
            )
            .collect();
        let mut scratch = Vec::new();
        let valid_zones: Vec<ZoneTerms> = zone_sources
            .iter()
            .zip(&adjusted)
            .zip(valid_zones)
            .map(|(((demand_input, emission, route), assessed), original)| {
                zone_terms(demand_input, assessed, emission, route, "", &mut scratch)
                    .unwrap_or(original)
            })
            .collect();
        let mut adjusted = adjusted.into_iter();
        demand = adjusted.next().expect("primary zone");
        additional_zone_demands = adjusted.collect();
        // 9.4: reheating coils of air handling units (11.120) also draw on
        // the node.
        let ahu_heating: [f64; 12] = std::array::from_fn(|index| {
            std::iter::once(&demand)
                .chain(&additional_zone_demands)
                .filter_map(|zone| zone.ventilation.as_ref())
                .filter_map(|result| result.months.get(index))
                .map(|month| month.ahu_heating_kwh)
                .sum()
        });
        let mut outputs = Vec::with_capacity(12);
        for index in 0..12 {
            let mut need = 0.0;
            let mut emission_loss = 0.0;
            let mut distribution_loss = 0.0;
            let mut to_medium = 0.0;
            let mut distribution_input = 0.0;
            for (zone_index, zone) in valid_zones.iter().enumerate() {
                let emission_input = zone.need[index] + zone.emission_loss[index];
                let loss = distribution.zone_loss[zone_index][index];
                let aux = distribution.zone_aux_to_medium[zone_index][index];
                need += zone.need[index];
                emission_loss += zone.emission_loss[index];
                distribution_loss += loss;
                to_medium += aux;
                // 9.24 per zone, clamped at 0.
                distribution_input += (emission_input + loss - aux).max(0.0);
            }
            // 9.5 node: generator output covers all zones plus the buffer loss.
            // 9.4: the node also supplies atomising humidification (12.1).
            // 13.185: hot water from this system (§13.8.4.9.3).
            let hot_water_load = input.hot_water_load_kwh.get(index).copied().unwrap_or(0.0);
            let node_output = distribution_input
                + distribution.node_loss[index]
                + humidification[index][0]
                + ahu_heating[index]
                + hot_water_load;
            // 9.2.3.4/9.5: solar heat reduces the node input.
            let solar_gain = input
                .solar_heating_kwh
                .get(index)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, node_output.max(0.0));
            let generator_output = node_output - solar_gain;
            let month = index as u8 + 1;
            outputs.push(MonthlyEnergy {
                month,
                energy_kwh: generator_output,
            });
            monthly.push(ChainMonth {
                month,
                heating_need_kwh: need,
                emission_loss_kwh: emission_loss,
                emission_input_kwh: need + emission_loss,
                distribution_loss_kwh: distribution_loss,
                distribution_auxiliary_to_medium_kwh: to_medium,
                node_loss_kwh: distribution.node_loss[index],
                solar_gain_kwh: solar_gain,
                generator_output_kwh: generator_output,
                heat_pump_output_kwh: 0.0,
                natural_gas_kwh: 0.0,
                district_heat_kwh: 0.0,
                biomass_kwh: 0.0,
                generator_electricity_kwh: 0.0,
                auxiliary_electricity_kwh: None,
                distribution_auxiliary_electricity_kwh: distribution.pump_electricity[index],
                emission_fan_electricity_kwh: 0.0,
                recoverable_loss_kwh: distribution
                    .zone_recoverable
                    .iter()
                    .map(|zone| zone[index])
                    .sum(),
                collective_source_heat_kwh: 0.0,
                oil_kwh: 0.0,
                generator_recoverable_loss_kwh: 0.0,
                humidification_load_kwh: humidification[index][0],
                ahu_heating_load_kwh: ahu_heating[index],
                chp_electricity_kwh: 0.0,
                hot_water_load_kwh: hot_water_load,
                humidification_electricity_kwh: humidification[index][1],
                humidification_fuel_kwh: humidification[index][2],
            });
        }
        zone_recoverable_losses = valid_zones
            .iter()
            .zip(&distribution.zone_recoverable)
            .map(|(zone, values)| ZoneRecoverableLoss {
                zone_id: zone.zone_id.clone(),
                monthly_kwh: values.to_vec(),
            })
            .collect();
        distribution_summary = distribution.summary;
        if issues.is_empty() {
            let zones = || std::iter::once(&demand).chain(&additional_zone_demands);
            let building = HeatPumpBuildingContext {
                residential: input.demand.usage_function.is_residential(),
                heating_need_kwh: zones()
                    .map(|zone| zone.annual_heating_need_kwh.unwrap_or(0.0))
                    .sum(),
                cooling_need_kwh: zones()
                    .map(|zone| zone.annual_cooling_need_kwh.unwrap_or(0.0))
                    .sum(),
                usable_floor_area_m2: zone_area,
            };
            generation_efficiency = generate(
                input,
                &outputs,
                building_fraction,
                &conditions,
                building,
                &mut monthly,
                &mut annex_q_result,
                &mut issues,
            );
        }
        // 9.22 with t_H;op;si;mi (9.32a): the longest operating time of the
        // zones on the system, t_H·f_H;red·f_H;red;pmp;op as for the pump.
        let fan_power: f64 = valid_zones.iter().map(|zone| zone.fan_power_w).sum();
        if fan_power > 0.0 {
            let system_hours: [f64; 12] = match &distribution_summary {
                Some(summary) => std::array::from_fn(|month| {
                    summary
                        .zones
                        .iter()
                        .map(|zone| zone.operating_hours[month])
                        .fold(0.0_f64, f64::max)
                }),
                None => {
                    // Without hydraulic data: the use function of the primary
                    // zone, an individual installation for dwellings.
                    let function = reduction_function(input.demand.usage_function);
                    let pump = if function == ReductionFunction::Residential {
                        0.10
                    } else {
                        1.0
                    };
                    let factor = function.heating_reduction_factor() * pump;
                    let limits: Vec<i32> = valid_zones
                        .iter()
                        .filter_map(|zone| {
                            let need: [f64; 12] = std::array::from_fn(|month| {
                                zone.need[month] + zone.heating_limit_extra[month]
                            });
                            heating_limit_c(&need, zone.setpoint_c)
                        })
                        .collect();
                    std::array::from_fn(|month| {
                        limits
                            .iter()
                            .map(|limit| heating_limit_hours(*limit, month) * factor)
                            .fold(0.0_f64, f64::max)
                    })
                }
            };
            for (index, row) in monthly.iter_mut().enumerate() {
                row.emission_fan_electricity_kwh = fan_power * system_hours[index] / 1000.0;
            }
        }
        // 9.6: total auxiliary energy includes the distribution pump and the
        // emission fans (9.21).
        for row in monthly.iter_mut() {
            row.auxiliary_electricity_kwh = row.auxiliary_electricity_kwh.map(|value| {
                value
                    + row.distribution_auxiliary_electricity_kwh
                    + row.emission_fan_electricity_kwh
            });
        }
    }
    let valid = issues.is_empty();
    if !valid {
        monthly.clear();
        zone_recoverable_losses.clear();
        distribution_summary = None;
    }
    let sum = |field: fn(&ChainMonth) -> f64| valid.then(|| monthly.iter().map(field).sum::<f64>());
    let auxiliary = if valid
        && monthly
            .iter()
            .all(|row| row.auxiliary_electricity_kwh.is_some())
    {
        Some(
            monthly
                .iter()
                .map(|row| row.auxiliary_electricity_kwh.unwrap_or(0.0))
                .sum(),
        )
    } else {
        None
    };
    SpaceHeatingChainAssessment {
        status: if valid {
            "calculated_unverified"
        } else {
            "invalid"
        },
        scope: "nta8800_space_heating_zones_single_generator_unverified",
        chapter_9_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        final_edition_verified: false,
        reference_verified: false,
        beng_calculation_available: false,
        omitted_terms: OMITTED_TERMS,
        emission_temperature_increment_k: increment.filter(|_| valid),
        generation_efficiency: generation_efficiency.filter(|_| valid),
        annual_natural_gas_kwh: sum(|row| row.natural_gas_kwh),
        annual_generator_electricity_kwh: sum(|row| row.generator_electricity_kwh),
        annual_auxiliary_electricity_kwh: auxiliary,
        annual_collective_source_heat_kwh: sum(|row| row.collective_source_heat_kwh),
        annual_district_heat_kwh: sum(|row| row.district_heat_kwh),
        annual_biomass_kwh: sum(|row| row.biomass_kwh),
        distribution: distribution_summary,
        zone_recoverable_losses,
        annex_q: annex_q_result.filter(|_| valid),
        monthly,
        demand,
        additional_zone_demands,
        issues,
    }
}

/// Measured 9.85–9.88 auxiliary energy of an individual heat pump.
fn measured_heat_pump_auxiliary(
    aux: &HybridHeatPumpAuxMeasurements,
    generator_id: &str,
    fingerprint: &str,
    electricity: &[(u8, f64)],
    path: &str,
    issues: &mut Vec<ChainIssue>,
) -> Option<Vec<f64>> {
    if aux.generator_id != generator_id {
        issues.push(issue(
            "heat_pump_auxiliary_generator_mismatch",
            format!("{path}.generatorId"),
        ));
        return None;
    }
    let assessed = assess_heating_aux_measured_draft(&HeatingAuxMeasuredDraftInput {
        generator_id: aux.generator_id.clone(),
        generator_source_reference: aux.generator_source_reference.clone(),
        measurements: aux.measurements.clone(),
        input_energy_source_reference: format!("heat_pump_monthly_sha256:{fingerprint}"),
        months: electricity
            .iter()
            .map(|(month, kwh)| GeneratorElectricityMonth {
                month: *month,
                generator_input_electricity_kwh: *kwh,
            })
            .collect(),
    });
    match assessed
        .auxiliary
        .as_ref()
        .filter(|_| assessed.status != "invalid")
    {
        None => {
            issues.extend(
                assessed
                    .issues
                    .iter()
                    .map(|item| issue(item.code, format!("{path}.{}", item.path))),
            );
            None
        }
        Some(aux) => Some(
            (1..=12u8)
                .map(|month| {
                    aux.monthly_auxiliary_electricity_kwh
                        .iter()
                        .find(|item| item.month == month)
                        .map_or(0.0, |item| item.electricity_kwh)
                })
                .collect(),
        ),
    }
}

/// Generator step; fills the carrier columns and the generator auxiliary
/// energy of `monthly` and returns the generation efficiency.
/// Generator operating conditions of annexes M and N.
struct GeneratorConditions {
    /// `t_H;op;si;mi` of table 9.15 at the heating limit (longest over the
    /// zones); `None` when the heating limit cannot be determined.
    hours: Option<[f64; 12]>,
    /// `ϑ_H,out` (9.32), area-weighted over the zones; `None` without a
    /// design temperature class.
    return_c: Option<[f64; 12]>,
    /// Area-weighted heating setpoint, °C.
    indoor_c: f64,
    /// `t_H;op;si;mi` per 9.32a (with f_H;red and f_H;red;pmp;op), longest
    /// over the zones.
    operating_hours: Option<[f64; 12]>,
}

fn generator_conditions(
    input: &SpaceHeatingChainInput,
    zones: &[ZoneTerms],
    distribution: &DistributionResult,
) -> GeneratorConditions {
    let area: f64 = zones.iter().map(|zone| zone.area_m2).sum();
    let indoor_c = if area > 0.0 {
        zones
            .iter()
            .map(|zone| zone.setpoint_c * zone.area_m2)
            .sum::<f64>()
            / area
    } else {
        20.0
    };
    let limits: Option<Vec<i32>> = match &distribution.summary {
        Some(summary) => Some(
            summary
                .zones
                .iter()
                .map(|zone| zone.heating_limit_c)
                .collect(),
        ),
        None => zones
            .iter()
            .map(|zone| {
                let need: [f64; 12] =
                    std::array::from_fn(|month| zone.need[month] + zone.heating_limit_extra[month]);
                heating_limit_c(&need, zone.setpoint_c)
            })
            .collect(),
    };
    let class = input
        .distribution_system
        .as_ref()
        .map(|system| {
            system
                .design_temperature_class
                .unwrap_or(DesignTemperatureClass::C90)
        })
        .or(match &input.generator {
            Generator::ProductBoiler(generator) => generator.design_temperature_class,
            _ => None,
        });
    let hours = limits.as_ref().map(|limits| {
        std::array::from_fn(|month| {
            limits
                .iter()
                .map(|limit| heating_limit_hours(*limit, month))
                .fold(0.0, f64::max)
        })
    });
    let return_c = match (&limits, class) {
        (Some(limits), Some(class)) if area > 0.0 => Some(std::array::from_fn(|month| {
            zones
                .iter()
                .zip(limits)
                .map(|(zone, limit)| {
                    let (_, ret) = supply_return_c(
                        zone.setpoint_c,
                        class,
                        *limit,
                        OUTDOOR_TEMPERATURE_C[month],
                        zone.increment_k,
                    );
                    ret * zone.area_m2
                })
                .sum::<f64>()
                / area
        })),
        _ => None,
    };
    // 9.32a from the distribution summary, else table 9.15 with the use
    // function of the primary zone and an individual installation.
    let operating_hours = match &distribution.summary {
        Some(summary) => Some(std::array::from_fn(|month| {
            summary
                .zones
                .iter()
                .map(|zone| zone.operating_hours[month])
                .fold(0.0_f64, f64::max)
        })),
        None => hours.map(|hours: [f64; 12]| {
            let function = reduction_function(input.demand.usage_function);
            let pump = if function == ReductionFunction::Residential {
                0.10
            } else {
                1.0
            };
            let factor = function.heating_reduction_factor() * pump;
            std::array::from_fn(|month| hours[month] * factor)
        }),
    };
    GeneratorConditions {
        hours,
        return_c,
        indoor_c,
        operating_hours,
    }
}

/// Biomass checks shared by annex M boilers and annex N stoves.
fn validate_biomass_evidence(
    compliant: Option<bool>,
    reference: Option<&String>,
    issues: &mut Vec<ChainIssue>,
) {
    if compliant != Some(true) {
        issues.push(issue(
            "biomass_class_unsupported",
            "generator.annexRCompliantAtMost500Kw",
        ));
    }
    if reference.map_or(true, |value| value.trim().is_empty()) {
        issues.push(issue(
            "source_reference_required",
            "generator.annexRReference",
        ));
    }
}

/// Annex Q result and the annex V correction as reported by the chain.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnexQOutput {
    pub annex_q: AnnexQResult,
    pub demand_class: DemandClass,
    /// Annex V degree of regeneration, when given (reported only).
    pub regeneration_degree: Option<f64>,
    /// `c_source`; always 1, because 9.63 (method 1) has no source
    /// correction.
    pub source_correction: f64,
    /// Q.5.3 f_H;t;hp-on;mi(3) used for chapter 11, when derived.
    pub exhaust_air_heating_time_fraction: Option<Vec<f64>>,
    /// 9.63 `f_prac`.
    pub practice_factor: f64,
    /// `COP · f_prac` of 9.63: heat-pump output per unit of electricity.
    pub corrected_efficiency: f64,
}

#[allow(clippy::too_many_arguments)]
fn generate_annex_q(
    generator: &AnnexQGenerator,
    outputs: &[MonthlyEnergy],
    building: HeatPumpBuildingContext,
    building_fraction: f64,
    monthly: &mut [ChainMonth],
    annex_q_result: &mut Option<AnnexQOutput>,
    issues: &mut Vec<ChainIssue>,
) -> Option<f64> {
    let prior = issues.len();
    issues.extend(
        validate_annex_q(
            &generator.heat_pump,
            generator.design_supply_temperature_c,
            "generator.heatPump",
        )
        .into_iter()
        .map(|item| issue(item.code, item.path)),
    );
    if generator.equipment_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            "generator.equipmentReference",
        ));
    }
    if let Some(regeneration) = &generator.regeneration {
        if generator.heat_pump.source != AnnexQSource::BrineWater {
            // Table V.1 covers individual ground heat exchangers only.
            issues.push(issue(
                "regeneration_requires_ground_source",
                "generator.regeneration",
            ));
        }
        issues.extend(
            validate_regeneration(regeneration, "generator.regeneration")
                .into_iter()
                .map(|item| issue(item.code, item.path)),
        );
    }
    if issues.len() > prior {
        return None;
    }
    let annual: f64 = outputs.iter().map(|item| item.energy_kwh).sum();
    let demand_class = DemandClass::from_need(
        building.residential,
        if building.usable_floor_area_m2 > 0.0 {
            building.heating_need_kwh / building.usable_floor_area_m2
        } else {
            0.0
        },
    );
    let result = calculate_annex_q(
        &generator.heat_pump,
        AnnexQContext {
            annual_node_input_kwh: annual,
            design_supply_temperature_c: generator.design_supply_temperature_c,
            demand_class,
        },
    );
    let fraction = result.energy_fraction;
    let efficiency = result.generation_efficiency;
    if !(efficiency.is_finite() && efficiency > 0.0) {
        issues.push(issue("annex_q_efficiency_invalid", "generator.heatPump"));
        return None;
    }
    // Annex V on the heat-pump heat (hot water on the same source is not
    // part of the space-heating chain).
    let regeneration = generator.regeneration.as_ref().map(|input| {
        calculate_regeneration(
            input,
            RegenerationContext {
                heating_kwh: fraction * annual,
                heating_efficiency: efficiency,
                hot_water_kwh: 0.0,
                hot_water_efficiency: 0.0,
                annual_cooling_need_kwh: building.cooling_need_kwh,
                electricity_efficiency: ELECTRICITY_EFFICIENCY,
            },
        )
    });
    // 9.63: E = Q_out / (COP · f_prac); no c_source for method 1.
    let correction = 1.0;
    let corrected = efficiency * ANNEX_Q_PRACTICE_FACTOR;
    if result
        .bins
        .iter()
        .any(|bin| bin.delivered_kw > 0.0 && !(bin.cop.is_finite() && bin.cop > 0.0))
    {
        // Delivered heat needs a positive COP (Q.4).
        issues.push(issue("annex_q_cop_not_positive", "generator.heatPump"));
        return None;
    }
    // Q.1: the rounded hours of table Q.6 keep F just below 1 even when
    // every bin is covered; full coverage per bin counts as F = 1.
    let covers_all_bins = result
        .bins
        .iter()
        .all(|bin| bin.delivered_kw >= bin.demand_kw * (1.0 - 1e-9));
    let fraction = if covers_all_bins && generator.backup.is_none() {
        1.0
    } else {
        fraction
    };
    if fraction < 1.0 - 1e-9 && generator.backup.is_none() {
        // Q.1: without supplementary heating the fraction must be 1.
        issues.push(issue("annex_q_backup_required", "generator.backup"));
        return None;
    }
    let backup_outputs: Vec<MonthlyEnergy> = outputs
        .iter()
        .map(|item| MonthlyEnergy {
            month: item.month,
            energy_kwh: (1.0 - fraction) * item.energy_kwh,
        })
        .collect();
    let mut backup_gas: Option<Vec<(f64, Option<f64>)>> = None;
    if let Some(AnnexQBackup::GasBoiler { boiler }) = &generator.backup {
        let assessed = assess_boiler_forfait_monthly_draft(&BoilerForfaitMonthlyDraftInput {
            boiler: boiler.clone(),
            generator_output_kwh: backup_outputs.clone(),
            generator_output_reference: "annex Q supplementary share".into(),
        });
        issues.extend(
            assessed
                .issues
                .iter()
                .map(|item| issue(item.code, format!("generator.backup.boiler.{}", item.path))),
        );
        if assessed.monthly.len() != 12 {
            if issues.len() == prior {
                issues.push(issue("generator_result_incomplete", "generator.backup"));
            }
            return None;
        }
        backup_gas = Some(
            assessed
                .monthly
                .iter()
                .map(|item| {
                    // §9.6.2.1: pilot flame scaled with f_gebouw;H when collective.
                    let pilot_share = if boiler.role == BoilerRole::Collective {
                        building_fraction
                    } else {
                        1.0
                    };
                    (
                        item.input_natural_gas_kwh + pilot_share * item.pilot_flame_natural_gas_kwh,
                        item.auxiliary_electricity_kwh,
                    )
                })
                .collect(),
        );
    }
    for (index, row) in monthly.iter_mut().enumerate() {
        let output = outputs[index].energy_kwh;
        let pump = fraction * output;
        let backup = output - pump;
        row.heat_pump_output_kwh = pump;
        row.generator_electricity_kwh = pump / corrected;
        // Q.4: the source pump is in η_H;gen;hp; 9.6.3.2 W_aux is not
        // booked a second time (see OMITTED_TERMS).
        let mut auxiliary = 0.0;
        match (&generator.backup, &backup_gas) {
            (Some(AnnexQBackup::ElectricResistance), _) => {
                row.generator_electricity_kwh += backup;
            }
            (Some(AnnexQBackup::GasBoiler { .. }), Some(gas)) => {
                row.natural_gas_kwh = gas[index].0;
                auxiliary += gas[index].1.unwrap_or(0.0);
            }
            _ => {}
        }
        row.auxiliary_electricity_kwh = Some(auxiliary);
    }
    *annex_q_result = Some(AnnexQOutput {
        annex_q: result,
        demand_class,
        regeneration_degree: regeneration.map(|item| item.degree),
        source_correction: correction,
        exhaust_air_heating_time_fraction: None,
        practice_factor: ANNEX_Q_PRACTICE_FACTOR,
        corrected_efficiency: corrected,
    });
    Some(corrected)
}

/// M.12 ϑ_brm as ϑ_H,amb of 9.4.2: the heating setpoint (taken for
/// θ_int;op;H, 7.9.6) in a heated space, ϑ_ztu of the distribution system
/// (7.82) in an installation room when entered; otherwise table M.6.
fn boiler_ambient_c(
    placement: BoilerPlacement,
    input: &SpaceHeatingChainInput,
    indoor_c: f64,
    month: usize,
) -> Option<f64> {
    match placement {
        BoilerPlacement::HeatedSpace => Some(indoor_c),
        BoilerPlacement::InstallationRoom => input
            .distribution_system
            .as_ref()
            .and_then(|system| system.unheated_ambient_c.as_ref())
            .and_then(|values| values.get(month).copied()),
        BoilerPlacement::Outdoors | BoilerPlacement::UnderRoof => None,
    }
}

/// Table 7.15 use function of a monthly-demand usage function.
fn reduction_function(usage: crate::monthly_demand::UsageFunction) -> ReductionFunction {
    use crate::monthly_demand::UsageFunction as U;
    match usage {
        U::AssemblyChildCare | U::OtherAssembly => ReductionFunction::Assembly,
        U::Cell => ReductionFunction::Cell,
        U::HealthcareWithBeds => ReductionFunction::HealthcareWithBeds,
        U::OtherHealthcare => ReductionFunction::HealthcareOther,
        U::Office => ReductionFunction::Office,
        U::Lodging => ReductionFunction::Lodging,
        U::Education => ReductionFunction::Education,
        U::Sport => ReductionFunction::Sport,
        U::Retail => ReductionFunction::Retail,
        U::Residential => ReductionFunction::Residential,
    }
}

/// 9.6.1: energy fractions per preference (9.56–9.60, table 9.23) and the
/// generators run on their share of the node output. Generators with the
/// same preference share by nominal power; the lowest preference takes the
/// remainder (note 4: a fictitious identical generator covers missing
/// power).
#[allow(clippy::too_many_arguments)]
fn generate_multiple(
    input: &SpaceHeatingChainInput,
    set: &MultipleGenerators,
    outputs: &[MonthlyEnergy],
    building_fraction: f64,
    conditions: &GeneratorConditions,
    building: HeatPumpBuildingContext,
    monthly: &mut [ChainMonth],
    annex_q_result: &mut Option<AnnexQOutput>,
    issues: &mut Vec<ChainIssue>,
) -> Option<f64> {
    let prior = issues.len();
    if set.generators.len() < 2 {
        issues.push(issue(
            "multiple_generators_require_two",
            "generator.generators",
        ));
    }
    if set.source_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            "generator.sourceReference",
        ));
    }
    let mut preferences: Vec<u32> = set.generators.iter().map(|part| part.preference).collect();
    preferences.sort_unstable();
    preferences.dedup();
    if preferences.first() != Some(&1) || preferences.windows(2).any(|pair| pair[1] != pair[0] + 1)
    {
        issues.push(issue(
            "generator_preferences_not_consecutive",
            "generator.generators",
        ));
    }
    for (index, part) in set.generators.iter().enumerate() {
        if !part.nominal_power_kw.is_finite() || part.nominal_power_kw <= 0.0 {
            issues.push(issue(
                "generator_nominal_power_invalid",
                format!("generator.generators[{index}].nominalPowerKw"),
            ));
        }
        if matches!(
            part.generator,
            Generator::Multiple(_) | Generator::HybridHeatPump(_)
        ) || part
            .generator
            .annex_q()
            .is_some_and(|generator| generator.backup.is_some())
        {
            // These already split the load themselves.
            issues.push(issue(
                "generator_nested_split_unsupported",
                format!("generator.generators[{index}].generator"),
            ));
        }
    }
    if issues.len() > prior {
        return None;
    }
    let total_power: f64 = set
        .generators
        .iter()
        .map(|part| part.nominal_power_kw)
        .sum();
    let power_up_to = |preference: u32| -> f64 {
        set.generators
            .iter()
            .filter(|part| part.preference <= preference)
            .map(|part| part.nominal_power_kw)
            .sum()
    };
    // 9.56/9.57, or 9.58/9.59 with Φ_H;tot from the node input.
    let reference = if set.added_preferred_generator {
        let annual: f64 = outputs.iter().map(|item| item.energy_kwh).sum();
        annual / FULL_LOAD_HOURS_9_59
    } else {
        total_power
    };
    let scale = if set.added_preferred_generator {
        building_fraction
    } else {
        1.0
    };
    let beta = |preference: u32| -> f64 {
        if preference == 0 || reference <= 0.0 {
            0.0
        } else {
            power_up_to(preference) * scale / reference
        }
    };
    let last = *preferences.last().expect("validated preferences");
    // 9.60 per preference and month.
    let fraction = |preference: u32, month: usize| -> f64 {
        let below = preferred_energy_fraction(beta(preference - 1), month);
        if preference == last {
            1.0 - below
        } else {
            (preferred_energy_fraction(beta(preference), month) - below).max(0.0)
        }
    };
    let mut hp_efficiency = None;
    let mut total_input = 0.0;
    let mut auxiliary_known = true;
    for row in monthly.iter_mut() {
        row.auxiliary_electricity_kwh = Some(0.0);
    }
    for (index, part) in set.generators.iter().enumerate() {
        let same: f64 = set
            .generators
            .iter()
            .filter(|other| other.preference == part.preference)
            .map(|other| other.nominal_power_kw)
            .sum();
        let share = part.nominal_power_kw / same;
        let sub_outputs: Vec<MonthlyEnergy> = outputs
            .iter()
            .enumerate()
            .map(|(month, item)| MonthlyEnergy {
                month: item.month,
                energy_kwh: item.energy_kwh * fraction(part.preference, month) * share,
            })
            .collect();
        let mut sub_rows: Vec<ChainMonth> = monthly
            .iter()
            .zip(&sub_outputs)
            .map(|(row, output)| ChainMonth {
                month: row.month,
                generator_output_kwh: output.energy_kwh,
                ..ChainMonth::default()
            })
            .collect();
        let mut sub_input = input.clone();
        sub_input.generator = part.generator.clone();
        let mut sub_issues = Vec::new();
        let efficiency = generate(
            &sub_input,
            &sub_outputs,
            building_fraction,
            conditions,
            building,
            &mut sub_rows,
            annex_q_result,
            &mut sub_issues,
        );
        let prefix = format!("generator.generators[{index}].generator.");
        issues.extend(sub_issues.into_iter().map(|item| ChainIssue {
            code: item.code,
            path: match item.path.strip_prefix("generator.") {
                Some(rest) => format!("{prefix}{rest}"),
                None => format!("{prefix}{}", item.path),
            },
        }));
        if part.generator.heat_pump().is_some() || part.generator.annex_q().is_some() {
            hp_efficiency = hp_efficiency.or(efficiency);
        }
        for (row, sub) in monthly.iter_mut().zip(&sub_rows) {
            row.heat_pump_output_kwh += sub.heat_pump_output_kwh;
            row.natural_gas_kwh += sub.natural_gas_kwh;
            row.district_heat_kwh += sub.district_heat_kwh;
            row.biomass_kwh += sub.biomass_kwh;
            row.oil_kwh += sub.oil_kwh;
            row.generator_recoverable_loss_kwh += sub.generator_recoverable_loss_kwh;
            row.generator_electricity_kwh += sub.generator_electricity_kwh;
            row.collective_source_heat_kwh += sub.collective_source_heat_kwh;
            row.chp_electricity_kwh += sub.chp_electricity_kwh;
            match (row.auxiliary_electricity_kwh, sub.auxiliary_electricity_kwh) {
                (Some(total), Some(value)) => row.auxiliary_electricity_kwh = Some(total + value),
                _ => auxiliary_known = false,
            }
            total_input += sub.natural_gas_kwh
                + sub.district_heat_kwh
                + sub.biomass_kwh
                + sub.oil_kwh
                + sub.generator_electricity_kwh;
        }
    }
    if !auxiliary_known {
        for row in monthly.iter_mut() {
            row.auxiliary_electricity_kwh = None;
        }
    }
    if issues.len() > prior {
        return None;
    }
    // The heat pump's efficiency feeds the ambient heat (5.30/5.31) of its
    // own output; otherwise the combined output over input.
    let output: f64 = outputs.iter().map(|item| item.energy_kwh).sum();
    hp_efficiency.or((total_input > 0.0).then(|| output / total_input))
}

#[allow(clippy::too_many_arguments)]
fn generate(
    input: &SpaceHeatingChainInput,
    outputs: &[MonthlyEnergy],
    building_fraction: f64,
    conditions: &GeneratorConditions,
    building: HeatPumpBuildingContext,
    monthly: &mut [ChainMonth],
    annex_q_result: &mut Option<AnnexQOutput>,
    issues: &mut Vec<ChainIssue>,
) -> Option<f64> {
    let mut generation_efficiency = None;
    match &input.generator {
        Generator::Multiple(set) => {
            return generate_multiple(
                input,
                set,
                outputs,
                building_fraction,
                conditions,
                building,
                monthly,
                annex_q_result,
                issues,
            );
        }
        Generator::Chp(generator) => {
            if generator.equipment_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    "generator.equipmentReference",
                ));
            }
            match (&generator.chp, &generator.method1) {
                (Some(class), None) => {
                    validate_other_auxiliary(generator.auxiliary.as_ref(), true, issues);
                    let Some((thermal, electric)) = class.factors() else {
                        issues.push(issue("chp_class_invalid", "generator.chp"));
                        return None;
                    };
                    if !issues.is_empty() {
                        return None;
                    }
                    // 9.65 with η = ε_chp;th (gross value) and f_prac = 1.
                    generation_efficiency = Some(thermal);
                    let auxiliary = generator.auxiliary.as_ref().expect("validated auxiliary");
                    for (index, row) in monthly.iter_mut().enumerate() {
                        row.natural_gas_kwh = row.generator_output_kwh / thermal;
                        // 16.12: credited in chapter 16.
                        row.chp_electricity_kwh = row.generator_output_kwh * electric / thermal;
                        row.auxiliary_electricity_kwh = Some(other_generator_auxiliary_kwh(
                            auxiliary,
                            OTHER_AUX_GAS_OIL_W_PER_KW,
                            row.generator_output_kwh,
                            MONTH_HOURS[index],
                            building_fraction,
                        ));
                    }
                }
                (None, Some(product)) => {
                    issues.extend(
                        crate::micro_chp::validate_micro_chp(product, "generator.method1")
                            .into_iter()
                            .map(|item| issue(item.code, item.path)),
                    );
                    let measured_aux = product.net_production_measured
                        || (product.standby_auxiliary_kw.is_some()
                            && product.chp_only.auxiliary_power_kw.is_some()
                            && product.full_load.auxiliary_power_kw.is_some());
                    // 9.6.8 when NEN-EN 50465 gives no auxiliary power.
                    if !measured_aux {
                        validate_other_auxiliary(generator.auxiliary.as_ref(), true, issues);
                    }
                    let Some(hours) = conditions.operating_hours else {
                        issues.push(issue("heating_limit_undetermined", "generator"));
                        return None;
                    };
                    if !issues.is_empty() {
                        return None;
                    }
                    let collective = building_fraction < 1.0;
                    let mut output = 0.0;
                    let mut input_total = 0.0;
                    for (index, row) in monthly.iter_mut().enumerate() {
                        let month = crate::micro_chp::micro_chp_month(
                            product,
                            row.generator_output_kwh,
                            hours[index],
                            MONTH_HOURS[index],
                            building_fraction,
                            collective,
                        )
                        .expect("validated micro-CHP");
                        match product.fuel {
                            crate::micro_chp::MicroChpFuel::NaturalGas => {
                                row.natural_gas_kwh = month.input_kwh
                            }
                            crate::micro_chp::MicroChpFuel::Oil => row.oil_kwh = month.input_kwh,
                        }
                        // 16.15.
                        row.chp_electricity_kwh = month.electricity_kwh;
                        row.generator_recoverable_loss_kwh = month.recoverable_kwh;
                        let auxiliary = match month.auxiliary_kwh {
                            Some(value) => value,
                            None => other_generator_auxiliary_kwh(
                                generator.auxiliary.as_ref().expect("validated auxiliary"),
                                OTHER_AUX_GAS_OIL_W_PER_KW,
                                row.generator_output_kwh,
                                MONTH_HOURS[index],
                                building_fraction,
                            ),
                        };
                        row.auxiliary_electricity_kwh = Some(auxiliary);
                        output += row.generator_output_kwh;
                        input_total += month.input_kwh;
                    }
                    generation_efficiency = (input_total > 0.0).then(|| output / input_total);
                }
                _ => {
                    issues.push(issue("chp_method_required", "generator.chp"));
                    return None;
                }
            }
        }
        Generator::GasBoiler(generator) => {
            let collective = generator.boiler.role == BoilerRole::Collective;
            if collective {
                validate_other_auxiliary(generator.auxiliary.as_ref(), true, issues);
            } else if generator.auxiliary.is_some() {
                issues.push(issue(
                    "generator_auxiliary_individual_uses_9_85",
                    "generator.auxiliary",
                ));
            }
            let result = assess_boiler_forfait_monthly_draft(&BoilerForfaitMonthlyDraftInput {
                boiler: generator.boiler.clone(),
                generator_output_kwh: outputs.to_vec(),
                generator_output_reference: "derived by space_heating_chain".into(),
            });
            issues.extend(
                result
                    .issues
                    .iter()
                    .map(|item| issue(item.code, format!("generator.boiler.{}", item.path))),
            );
            generation_efficiency = result.generation_efficiency;
            for (row, boiler) in monthly.iter_mut().zip(&result.monthly) {
                row.natural_gas_kwh = boiler.input_natural_gas_kwh;
                row.auxiliary_electricity_kwh = boiler.auxiliary_electricity_kwh;
            }
            if let Some(measurements) = &generator.auxiliary_measurements {
                if collective {
                    issues.push(issue(
                        "boiler_auxiliary_measurements_individual_only",
                        "generator.auxiliaryMeasurements",
                    ));
                } else {
                    // Annex O and 9.86–9.88 instead of the 9.85 forfait.
                    match auxiliary_constants(measurements, "generator.auxiliaryMeasurements") {
                        Ok(constants) => {
                            for row in monthly.iter_mut() {
                                row.auxiliary_electricity_kwh =
                                    Some(monthly_auxiliary_kwh(&constants, row.natural_gas_kwh));
                            }
                        }
                        Err(found) => {
                            issues.extend(found.into_iter().map(|item| issue(item.code, item.path)))
                        }
                    }
                }
            }
            // §9.6.2.1 pilot flame, scaled with f_gebouw;H for a collective boiler.
            let pilot_share = if collective { building_fraction } else { 1.0 };
            for (row, boiler) in monthly.iter_mut().zip(&result.monthly) {
                row.natural_gas_kwh += pilot_share * boiler.pilot_flame_natural_gas_kwh;
            }
            if result.monthly.len() != 12 && issues.is_empty() {
                issues.push(issue("generator_result_incomplete", "generator"));
            }
            if let (true, Some(auxiliary)) = (collective, &generator.auxiliary) {
                if issues.is_empty() {
                    for (index, row) in monthly.iter_mut().enumerate() {
                        row.auxiliary_electricity_kwh = Some(other_generator_auxiliary_kwh(
                            auxiliary,
                            OTHER_AUX_GAS_OIL_W_PER_KW,
                            row.generator_output_kwh,
                            MONTH_HOURS[index],
                            building_fraction,
                        ));
                    }
                }
            }
        }
        Generator::HeatPumpForfait(generator) => {
            if generator
                .forfait
                .design_supply_temperature_c
                .is_some_and(|temperature| temperature > 55.0)
            {
                // §9.6.3: systems above 55 °C need annex Q, also without a boiler.
                issues.push(issue(
                    "heat_pump_above55_requires_annex_q",
                    "generator.forfait.designSupplyTemperatureC",
                ));
            }
            let collective = generator.forfait.collective_building_installation == Some(true);
            if collective {
                validate_other_auxiliary(generator.auxiliary.as_ref(), false, issues);
                if generator.auxiliary_measurements.is_some() {
                    issues.push(issue(
                        "heat_pump_auxiliary_individual_only",
                        "generator.auxiliaryMeasurements",
                    ));
                }
            } else if generator.auxiliary.is_some() {
                issues.push(issue(
                    "generator_auxiliary_individual_uses_9_85",
                    "generator.auxiliary",
                ));
            }
            let result =
                assess_forfait_heat_pump_monthly_draft(&ForfaitHeatPumpMonthlyDraftInput {
                    forfait: generator.forfait.clone(),
                    generator_output_kwh: outputs.to_vec(),
                    generator_output_reference: "derived by space_heating_chain".into(),
                    source_system: generator.source_system,
                    source_system_reference: generator.source_system_reference.clone(),
                });
            issues.extend(
                result
                    .issues
                    .iter()
                    .map(|item| issue(item.code, format!("generator.{}", item.path))),
            );
            generation_efficiency = result.corrected_cop;
            for (row, pump) in monthly.iter_mut().zip(&result.monthly) {
                row.generator_electricity_kwh = pump.generator_input_electricity_kwh;
                row.collective_source_heat_kwh = pump.collective_source_heat_kwh;
                row.heat_pump_output_kwh = pump.generator_output_kwh;
            }
            if result.monthly.len() != 12 {
                if issues.is_empty() {
                    issues.push(issue("generator_result_incomplete", "generator"));
                }
                return generation_efficiency;
            }
            if !issues.is_empty() {
                return generation_efficiency;
            }
            let electricity: Vec<(u8, f64)> = monthly
                .iter()
                .map(|row| (row.month, row.generator_electricity_kwh))
                .collect();
            let auxiliary: Option<Vec<f64>> = if collective {
                generator.auxiliary.as_ref().map(|auxiliary| {
                    monthly
                        .iter()
                        .enumerate()
                        .map(|(index, row)| {
                            other_generator_auxiliary_kwh(
                                auxiliary,
                                0.0,
                                row.generator_output_kwh,
                                MONTH_HOURS[index],
                                building_fraction,
                            )
                        })
                        .collect()
                })
            } else if let Some(aux) = &generator.auxiliary_measurements {
                measured_heat_pump_auxiliary(
                    aux,
                    &generator.forfait.generator_id,
                    &result.input_fingerprint,
                    &electricity,
                    "generator.auxiliaryMeasurements",
                    issues,
                )
            } else {
                Some(
                    electricity
                        .iter()
                        .map(|(_, kwh)| heat_pump_forfait_auxiliary_kwh(*kwh))
                        .collect(),
                )
            };
            if let Some(values) = auxiliary {
                for (row, value) in monthly.iter_mut().zip(values) {
                    row.auxiliary_electricity_kwh = Some(value);
                }
            }
        }
        Generator::HybridHeatPump(generator) => {
            let result = assess_hybrid_heat_pump_monthly_draft(&HybridHeatPumpMonthlyDraftInput {
                dispatch: GeneratorDispatchDraftInput {
                    node_input_kwh: outputs.to_vec(),
                    node_input_reference: "derived by space_heating_chain".into(),
                    design_context: generator.design_context.clone(),
                    generators: generator.generators.clone(),
                },
                forfait: generator.forfait.clone(),
                boiler: generator.boiler.clone(),
                source_system: generator.source_system,
                source_system_reference: generator.source_system_reference.clone(),
                declared_operating_limits_present: generator.declared_operating_limits_present,
                heat_pump_auxiliary_measurements: generator
                    .heat_pump_auxiliary_measurements
                    .clone(),
            });
            issues.extend(
                result
                    .issues
                    .iter()
                    .map(|item| issue(item.code, format!("generator.{}", item.path))),
            );
            if let (Some(pump), Some(boiler)) = (&result.heat_pump, &result.boiler) {
                generation_efficiency = pump.corrected_cop;
                let pump_aux = result
                    .heat_pump_auxiliary
                    .as_ref()
                    .and_then(|aux| aux.auxiliary.as_ref())
                    .map(|aux| &aux.monthly_auxiliary_electricity_kwh);
                for (index, row) in monthly.iter_mut().enumerate() {
                    let pump_month = &pump.monthly[index];
                    let boiler_month = &boiler.monthly[index];
                    row.generator_electricity_kwh = pump_month.generator_input_electricity_kwh;
                    row.collective_source_heat_kwh = pump_month.collective_source_heat_kwh;
                    row.heat_pump_output_kwh = pump_month.generator_output_kwh;
                    row.natural_gas_kwh = boiler_month.input_natural_gas_kwh
                        + boiler_month.pilot_flame_natural_gas_kwh;
                    // Measured 9.85–9.88 values, otherwise the 9.85 forfait.
                    let pump_aux_month = match pump_aux {
                        Some(months) => months
                            .iter()
                            .find(|item| item.month == row.month)
                            .map_or(0.0, |item| item.electricity_kwh),
                        None => heat_pump_forfait_auxiliary_kwh(
                            pump_month.generator_input_electricity_kwh,
                        ),
                    };
                    row.auxiliary_electricity_kwh = boiler_month
                        .auxiliary_electricity_kwh
                        .map(|value| value + pump_aux_month);
                }
            } else if issues.is_empty() {
                issues.push(issue("generator_result_incomplete", "generator"));
            }
        }
        Generator::HeatPumpAnnexQ(generator) => {
            generation_efficiency = generate_annex_q(
                generator,
                outputs,
                building,
                building_fraction,
                monthly,
                annex_q_result,
                issues,
            );
        }
        Generator::ExternalHeat(generator) => {
            if generator.supplier_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    "generator.supplierReference",
                ));
            }
            validate_other_auxiliary(generator.auxiliary.as_ref(), false, issues);
            // 9.84 with η = 1,0 and f_prac = 1 for the fixed factor 0,9.
            generation_efficiency = Some(1.0);
            for (index, row) in monthly.iter_mut().enumerate() {
                row.district_heat_kwh = row.generator_output_kwh;
                row.auxiliary_electricity_kwh = generator.auxiliary.as_ref().map(|auxiliary| {
                    other_generator_auxiliary_kwh(
                        auxiliary,
                        0.0,
                        row.generator_output_kwh,
                        MONTH_HOURS[index],
                        building_fraction,
                    )
                });
            }
        }
        Generator::ElectricResistance(generator) => {
            if generator.equipment_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    "generator.equipmentReference",
                ));
            }
            validate_other_auxiliary(generator.auxiliary.as_ref(), false, issues);
            // Table 9.27: electric heating COP 1,0.
            generation_efficiency = Some(1.0);
            for (index, row) in monthly.iter_mut().enumerate() {
                row.generator_electricity_kwh = row.generator_output_kwh;
                row.auxiliary_electricity_kwh = generator.auxiliary.as_ref().map(|auxiliary| {
                    other_generator_auxiliary_kwh(
                        auxiliary,
                        0.0,
                        row.generator_output_kwh,
                        MONTH_HOURS[index],
                        building_fraction,
                    )
                });
            }
        }
        Generator::Biomass(generator) => {
            for (value, field) in [
                (&generator.annex_r_reference, "generator.annexRReference"),
                (
                    &generator.equipment_reference,
                    "generator.equipmentReference",
                ),
            ] {
                if value.trim().is_empty() {
                    issues.push(issue("source_reference_required", field));
                }
            }
            if !generator.annex_r_compliant_at_most_500_kw {
                issues.push(issue(
                    "biomass_class_unsupported",
                    "generator.annexRCompliantAtMost500Kw",
                ));
            }
            if generator.appliance.is_stove() {
                match generator.sole_heating_in_served_rooms {
                    None => issues.push(issue(
                        "biomass_sole_heating_confirmation_required",
                        "generator.soleHeatingInServedRooms",
                    )),
                    // §9.6.5: the other system is then the only heat supplier.
                    Some(false) => issues.push(issue(
                        "biomass_stove_not_sole_heating",
                        "generator.soleHeatingInServedRooms",
                    )),
                    Some(true) => {}
                }
            }
            // Central boilers are taken as automatically fired (table 9.30 note).
            let automatic = generator.automatic_fuel_feed || !generator.appliance.is_stove();
            let burner = if automatic {
                OTHER_AUX_AUTOMATIC_BIOMASS_W_PER_KW
            } else {
                0.0
            };
            validate_other_auxiliary(generator.auxiliary.as_ref(), automatic, issues);
            match biomass_efficiency(generator.appliance, generator.location) {
                None => issues.push(issue(
                    "biomass_efficiency_unavailable",
                    "generator.location",
                )),
                Some(efficiency) => {
                    // 9.64 with f_prac = 1,0.
                    generation_efficiency = Some(efficiency);
                    for (index, row) in monthly.iter_mut().enumerate() {
                        row.biomass_kwh = row.generator_output_kwh / efficiency;
                        row.auxiliary_electricity_kwh =
                            generator.auxiliary.as_ref().map(|auxiliary| {
                                other_generator_auxiliary_kwh(
                                    auxiliary,
                                    burner,
                                    row.generator_output_kwh,
                                    MONTH_HOURS[index],
                                    building_fraction,
                                )
                            });
                    }
                }
            }
        }
        Generator::ProductBoiler(generator) => {
            issues.extend(
                validate_product_boiler(&generator.boiler, "generator.boiler")
                    .into_iter()
                    .map(|item| issue(item.code, item.path)),
            );
            if generator.boiler.fuel == BoilerFuel::Wood {
                validate_biomass_evidence(
                    generator.annex_r_compliant_at_most_500_kw,
                    generator.annex_r_reference.as_ref(),
                    issues,
                );
            }
            let Some(hours) = conditions.hours else {
                issues.push(issue("heating_limit_undetermined", "demand"));
                return None;
            };
            let Some(return_c) = conditions.return_c else {
                issues.push(issue(
                    "design_temperature_class_required",
                    "generator.designTemperatureClass",
                ));
                return None;
            };
            if !issues.is_empty() {
                return None;
            }
            let mut output_total = 0.0;
            let mut input_total = 0.0;
            for (index, row) in monthly.iter_mut().enumerate() {
                // M.25 uses the output of the whole installation.
                let result = boiler_month(
                    &generator.boiler,
                    BoilerMonth {
                        heat_output_kwh: row.generator_output_kwh / building_fraction,
                        operating_hours: hours[index],
                        month_hours: MONTH_HOURS[index],
                        return_temperature_c: return_c[index],
                        outdoor_temperature_c: OUTDOOR_TEMPERATURE_C[index],
                        ambient_temperature_c: boiler_ambient_c(
                            generator.boiler.placement,
                            input,
                            conditions.indoor_c,
                            index,
                        ),
                    },
                );
                let fuel = result.input_kwh * building_fraction;
                match generator.boiler.fuel {
                    BoilerFuel::NaturalGas => row.natural_gas_kwh = fuel,
                    BoilerFuel::Oil => row.oil_kwh = fuel,
                    BoilerFuel::Wood => row.biomass_kwh = fuel,
                }
                row.auxiliary_electricity_kwh =
                    Some(result.auxiliary_electricity_kwh * building_fraction);
                row.generator_recoverable_loss_kwh =
                    result.recoverable_to_space_kwh * building_fraction;
                output_total += row.generator_output_kwh;
                input_total += fuel;
            }
            generation_efficiency = (input_total > 0.0).then(|| output_total / input_total);
        }
        Generator::LocalHeater(generator) => {
            issues.extend(
                validate_local_heater(&generator.heater, "generator.heater")
                    .into_iter()
                    .map(|item| issue(item.code, item.path)),
            );
            if generator.fuel == LocalHeaterFuel::Biomass {
                validate_biomass_evidence(
                    generator.annex_r_compliant_at_most_500_kw,
                    generator.annex_r_reference.as_ref(),
                    issues,
                );
                match generator.sole_heating_in_served_rooms {
                    None => issues.push(issue(
                        "biomass_sole_heating_confirmation_required",
                        "generator.soleHeatingInServedRooms",
                    )),
                    Some(false) => issues.push(issue(
                        "biomass_stove_not_sole_heating",
                        "generator.soleHeatingInServedRooms",
                    )),
                    Some(true) => {}
                }
            }
            let Some(hours) = conditions.hours else {
                issues.push(issue("heating_limit_undetermined", "demand"));
                return None;
            };
            if !issues.is_empty() {
                return None;
            }
            let Ok(resolved) = resolve_heater(&generator.heater, "generator.heater") else {
                return None;
            };
            let mut output_total = 0.0;
            let mut input_total = 0.0;
            for (index, row) in monthly.iter_mut().enumerate() {
                let result = heater_month(
                    &generator.heater,
                    &resolved,
                    HeaterMonth {
                        heat_required_kwh: row.generator_output_kwh,
                        operating_hours: hours[index],
                        indoor_c: conditions.indoor_c,
                        outdoor_c: OUTDOOR_TEMPERATURE_C[index],
                    },
                );
                if result.shortfall_kwh > 1e-6 {
                    issues.push(issue(
                        "local_heater_capacity_insufficient",
                        format!("monthly[{index}]"),
                    ));
                }
                // N.3: annex N works on the net calorific value; chapter 5
                // counts fuel on the gross value (f_Hs/Hi of table M.3).
                let fuel = result.input_kwh * generator.fuel.gross_to_net();
                match generator.fuel {
                    LocalHeaterFuel::NaturalGas => row.natural_gas_kwh = fuel,
                    LocalHeaterFuel::Oil => row.oil_kwh = fuel,
                    LocalHeaterFuel::Biomass => row.biomass_kwh = fuel,
                }
                row.auxiliary_electricity_kwh = Some(result.auxiliary_electricity_kwh);
                output_total += row.generator_output_kwh;
                input_total += fuel;
            }
            generation_efficiency = (input_total > 0.0).then(|| output_total / input_total);
        }
        Generator::ForfaitHeater(generator) => {
            if generator.equipment_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    "generator.equipmentReference",
                ));
            }
            if generator.kind.air_heater() && generator.fuel != ForfaitHeaterFuel::NaturalGas {
                // Table 9.25 defines the air-heater classes for gas only.
                issues.push(issue("forfait_air_heater_gas_only", "generator.fuel"));
            }
            validate_other_auxiliary(generator.auxiliary.as_ref(), true, issues);
            if generator.pilot_flames > 0
                && (!generator.kind.air_heater() || generator.fuel != ForfaitHeaterFuel::NaturalGas)
            {
                issues.push(issue("pilot_flame_not_applicable", "generator.pilotFlames"));
            }
            if !issues.is_empty() {
                return None;
            }
            let efficiency = generator.kind.efficiency();
            generation_efficiency = Some(efficiency);
            let auxiliary = generator.auxiliary.as_ref().expect("validated auxiliary");
            for (index, row) in monthly.iter_mut().enumerate() {
                let fuel = row.generator_output_kwh / efficiency;
                match generator.fuel {
                    ForfaitHeaterFuel::NaturalGas => {
                        // §9.6.2.1 pilot flames, scaled with f_gebouw;H.
                        row.natural_gas_kwh = fuel
                            + f64::from(generator.pilot_flames)
                                * crate::boiler_forfait_draft::PILOT_FLAME_ANNUAL_KWH
                                * MONTH_HOURS[index]
                                / crate::climate::YEAR_HOURS
                                * building_fraction
                    }
                    ForfaitHeaterFuel::Oil => row.oil_kwh = fuel,
                }
                row.auxiliary_electricity_kwh = Some(other_generator_auxiliary_kwh(
                    auxiliary,
                    OTHER_AUX_GAS_OIL_W_PER_KW,
                    row.generator_output_kwh,
                    MONTH_HOURS[index],
                    building_fraction,
                ));
            }
        }
    }
    generation_efficiency
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forfait_heat_pump_draft::{TableRowVariant, TableScope, TableSink, TableSource};
    use serde_json::json;

    fn demand() -> MonthlyDemandInput {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-monthly-demand-synthetic.json"
        ))
        .unwrap()
    }

    fn emission() -> EmissionInput {
        serde_json::from_value(json!({
            "system": "radiators_or_convectors", "balancing": "none_or_unknown",
            "control": "main_room_thermostat", "sourceReference": "installation survey"
        }))
        .unwrap()
    }

    #[test]
    fn humidifiers_load_the_node_or_use_their_own_carrier() {
        use crate::humidification::{Humidification, Humidifier, SteamCarrier};
        let mut input = boiler_chain();
        let demand = &mut input.demand;
        demand.usage_function = crate::monthly_demand::UsageFunction::Office;
        demand.dwelling_type = None;
        demand.setpoints.heating_c = 21.0;
        demand.internal_gains = crate::monthly_demand::InternalGains::Declared {
            heat_flux_w_per_m2: 5.5,
            source_reference: "tables 7.2/7.3".into(),
        };
        demand.ventilation_flows.clear();
        demand.ventilation = Some(
            serde_json::from_value(serde_json::json!({
                "zoneId": demand.zone_id,
                "usableFloorAreaM2": demand.usable_floor_area_m2,
                "category": "utility",
                "functions": [{"function": "office", "areaM2": demand.usable_floor_area_m2}],
                "buildingHeightM": 9.0,
                "constructionYear": 2020,
                "heatingSetpointC": 21.0,
                "coolingSetpointC": 24.0,
                "system": {"kind": "single", "unit": {"variant": "d1", "ducts": "luka_a_b_c", "equipmentReference": "ahu"}},
                "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "test"},
                "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020},
                "sourceReference": "test"
            }))
            .unwrap(),
        );
        let base = assess_space_heating_chain(&input);
        assert_eq!(base.status, "calculated_unverified", "{:?}", base.issues);
        let zone_id = input.demand.zone_id.clone();
        input.humidifiers = vec![ZoneHumidifier {
            zone_id: zone_id.clone(),
            humidification: Humidification {
                humidifier: Humidifier::Steam {
                    carrier: SteamCarrier::Electricity,
                },
                rotary_wheel: false,
                equipment_reference: "steam unit".into(),
            },
        }];
        let steam = assess_space_heating_chain(&input);
        assert_eq!(steam.status, "calculated_unverified", "{:?}", steam.issues);
        let jan = &steam.monthly[0];
        let supply = steam.demand.ventilation.as_ref().unwrap().months[0]
            .heating
            .mechanical_supply_m3_per_h;
        let need = 2538.2 * 1.205 * supply / 3600.0 * 0.82;
        assert!((jan.humidification_electricity_kwh - need / 0.8).abs() < 1e-6);
        // 12.4: (1 − η) of the steam humidifier is recoverable (7.3–7.8) and
        // lowers the heating need.
        assert!(jan.heating_need_kwh < base.monthly[0].heating_need_kwh);
        assert!(jan.generator_output_kwh < base.monthly[0].generator_output_kwh);
        input.humidifiers[0].humidification.humidifier = Humidifier::Atomising;
        let atomising = assess_space_heating_chain(&input);
        let jan = &atomising.monthly[0];
        assert!((jan.humidification_load_kwh - need).abs() < 1e-6);
        assert!(
            (jan.generator_output_kwh - base.monthly[0].generator_output_kwh - need).abs() < 1e-6
        );
    }

    fn boiler_chain() -> SpaceHeatingChainInput {
        SpaceHeatingChainInput {
            humidifiers: Vec::new(),
            solar_heating_kwh: Vec::new(),
            hot_water_load_kwh: Vec::new(),
            demand: demand(),
            emission: emission(),
            distribution: Distribution::HeatedZoneOnlySpaceHeating {
                source_reference: "pipes inside envelope".into(),
            },
            additional_zones: Vec::new(),
            distribution_system: None,
            collective_connection: None,
            generator: serde_json::from_value(json!({
                "kind": "gas_boiler",
                "boiler": {
                    "generatorId":"boiler", "role":"individual_main",
                    "location":"inside_thermal_boundary", "kind":"hr107", "fuel":"natural_gas",
                    "averageDesignEmissionTemperatureC":45.0, "emissionCircuit":"direct",
                    "equipmentReference":"type plate", "locationReference":"building plan",
                    "temperatureAndCircuitReference":"system design", "pilotFlamePresent":false,
                    "installationYear": 2020, "installationYearReference": "invoice"
                }
            }))
            .unwrap(),
        }
    }

    fn heat_pump() -> ForfaitHeatPumpDraftInput {
        ForfaitHeatPumpDraftInput {
            generator_id: "hp".into(),
            classification_source_reference: "system design".into(),
            scope: TableScope::ResidentialAtMost25Kw,
            source: TableSource::OutdoorAir,
            sink: TableSink::Hydronic,
            design_supply_temperature_c: Some(35.0),
            source_correction_factor: None,
            source_correction_reference: None,
            thermal_capacity_kw: Some(8.0),
            capacity_source_reference: Some("rated capacity".into()),
            collective_building_installation: Some(false),
            row_variant: TableRowVariant::Base,
            high_efficiency_evidence: None,
            source_temperature_c: None,
            source_temperature_evidence_reference: None,
            source_quality_declaration_reference: None,
        }
    }

    #[test]
    fn boiler_chain_carries_need_through_emission_to_gas() {
        let result = assess_space_heating_chain(&boiler_chain());
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        let need = result.demand.monthly[0].heating.need_kwh;
        let loss = need * (3.55_f64 / (23.55 - 2.61)).min(0.15);
        assert!((jan.emission_loss_kwh - loss).abs() < 1e-9);
        assert_eq!(jan.distribution_loss_kwh, 0.0);
        assert!((jan.generator_output_kwh - (need + loss)).abs() < 1e-9);
        let efficiency = result.generation_efficiency.unwrap();
        assert!((jan.natural_gas_kwh - (need + loss) / efficiency).abs() < 1e-9);
        assert!(result.annual_natural_gas_kwh.unwrap() > 0.0);
        assert_eq!(result.annual_generator_electricity_kwh, Some(0.0));
        assert!(!result.beng_calculation_available);
    }

    #[test]
    fn micro_chp_method_1_books_input_electricity_and_aux() {
        use crate::micro_chp::{
            micro_chp_month, ChpTestPoint, MicroChp, MicroChpFuel, MicroChpLocation, MicroChpType,
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
            test_report_reference: "EN 50465 report".into(),
        };
        let mut input = boiler_chain();
        input.generator = Generator::Chp(ChpGenerator {
            chp: None,
            method1: Some(product.clone()),
            auxiliary: None,
            equipment_reference: "micro-CHP".into(),
        });
        input.distribution_system = Some(system(calculated_pump()));
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        // The chain uses t_H;op of 9.32a from the distribution summary.
        let hours = result.distribution.as_ref().unwrap().zones[0].operating_hours[0];
        let jan = &result.monthly[0];
        let expected =
            micro_chp_month(&product, jan.generator_output_kwh, hours, 744.0, 1.0, false).unwrap();
        assert!((jan.natural_gas_kwh - expected.input_kwh).abs() < 1e-9);
        assert!((jan.chp_electricity_kwh - expected.electricity_kwh).abs() < 1e-9);
        assert!(jan.chp_electricity_kwh > 0.0);
        assert!((jan.generator_recoverable_loss_kwh - 0.4 * hours).abs() < 1e-9);
        // 9.7: the recoverable generator loss lowers the heating need
        // against a pass without it (7.3–7.8).
        let without = assess_chain_pass(&input, None);
        assert!(jan.heating_need_kwh < without.monthly[0].heating_need_kwh);
        // Both methods at once, or none, are rejected.
        if let Generator::Chp(generator) = &mut input.generator {
            generator.method1 = None;
        }
        assert!(codes(&input).contains(&"chp_method_required"));
    }

    #[test]
    fn chp_generator_follows_9_65_and_16_12() {
        let mut input = boiler_chain();
        input.generator = Generator::Chp(ChpGenerator {
            chp: Some(crate::space_cooling::ChpClass {
                power_kw: 50.0,
                built_after_2006: true,
                hre_declared: false,
                low_temperature: false,
            }),
            method1: None,
            auxiliary: Some(OtherGeneratorAuxiliary {
                electrically_connected_devices: 1,
                nominal_power_kw: Some(80.0),
                source_reference: "datasheet".into(),
            }),
            equipment_reference: "CHP datasheet".into(),
        });
        // A CHP's 9.91 auxiliary energy excludes the circulation pump.
        input.distribution_system = Some(system(calculated_pump()));
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        // Table 9.31, 20–200 kW after 2006 HT: ε_th 0,49, ε_el 0,30.
        let jan = &result.monthly[0];
        assert!((jan.natural_gas_kwh - jan.generator_output_kwh / 0.49).abs() < 1e-9);
        assert!((jan.chp_electricity_kwh - jan.generator_output_kwh * 0.30 / 0.49).abs() < 1e-9);
        assert_eq!(result.generation_efficiency, Some(0.49));
    }

    #[test]
    fn multiple_generators_split_by_table_9_23() {
        // β per 9.56: heat pump 4 kW of 24 kW → β 1/6; winter
        // 0,20 + (1/6 − 0,1)·10·0,20 = 1/3; July 0,87 + 0,5·0,08 = 0,91.
        assert!((preferred_energy_fraction(1.0 / 6.0, 0) - 1.0 / 3.0).abs() < 1e-12);
        assert!((preferred_energy_fraction(0.25, 6) - 0.91).abs() < 1e-12);
        assert_eq!(preferred_energy_fraction(1.5, 0), 1.0);
        let mut input = boiler_chain();
        let boiler = input.generator.clone();
        let heat_pump_generator = Generator::HeatPumpForfait(HeatPumpGenerator {
            forfait: heat_pump(),
            source_system: SourceSystem::Individual,
            source_system_reference: "own outdoor unit".into(),
            auxiliary_measurements: None,
            auxiliary: None,
        });
        input.generator = Generator::Multiple(Box::new(MultipleGenerators {
            generators: vec![
                PreferredGenerator {
                    preference: 2,
                    nominal_power_kw: 20.0,
                    generator: boiler.clone(),
                },
                PreferredGenerator {
                    preference: 1,
                    nominal_power_kw: 4.0,
                    generator: heat_pump_generator.clone(),
                },
            ],
            added_preferred_generator: false,
            source_reference: "installation survey".into(),
        }));
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let beta = 4.0 / 24.0;
        for (month, row) in result.monthly.iter().enumerate() {
            let f = preferred_energy_fraction(beta, month);
            assert!((row.heat_pump_output_kwh - f * row.generator_output_kwh).abs() < 1e-6);
        }
        // Each part equals its own single-generator chain on that share:
        // the heat pump's COP is the reported efficiency.
        let mut alone = boiler_chain();
        alone.generator = heat_pump_generator;
        let cop = assess_space_heating_chain(&alone)
            .generation_efficiency
            .unwrap();
        assert!((result.generation_efficiency.unwrap() - cop).abs() < 1e-9);
        let jan = &result.monthly[0];
        assert!((jan.generator_electricity_kwh - jan.heat_pump_output_kwh / cop).abs() < 1e-6);
        assert!(jan.natural_gas_kwh > 0.0);
        // Preferences must start at 1 and be consecutive.
        if let Generator::Multiple(set) = &mut input.generator {
            set.generators[0].preference = 3;
        }
        assert!(assess_space_heating_chain(&input)
            .issues
            .iter()
            .any(|item| item.code == "generator_preferences_not_consecutive"));
    }

    #[test]
    fn heat_pump_chain_uses_forfait_cop() {
        let mut input = boiler_chain();
        input.generator = Generator::HeatPumpForfait(HeatPumpGenerator {
            forfait: heat_pump(),
            source_system: SourceSystem::Individual,
            source_system_reference: "own outdoor unit".into(),
            auxiliary_measurements: None,
            auxiliary: None,
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let cop = result.generation_efficiency.unwrap();
        assert!(cop > 1.0);
        let jan = &result.monthly[0];
        assert!((jan.generator_electricity_kwh - jan.generator_output_kwh / cop).abs() < 1e-9);
        assert_eq!(result.annual_natural_gas_kwh, Some(0.0));
    }

    fn annex_q_chain() -> SpaceHeatingChainInput {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-space-heating-chain-annex-q-synthetic.json"
        ))
        .unwrap()
    }

    #[test]
    fn exhaust_air_heat_pump_recalculates_the_demand_per_q_5_3() {
        let mut input = annex_q_chain();
        let demand = &input.demand;
        let ventilation = json!({
            "zoneId": demand.zone_id,
            "usableFloorAreaM2": demand.usable_floor_area_m2,
            "category": "residential",
            "functions": [{"function": "residential", "areaM2": demand.usable_floor_area_m2}],
            "dwellingCount": 1,
            "buildingHeightM": 9.0,
            "constructionYear": 2020,
            "heatingSetpointC": demand.setpoints.heating_c,
            "coolingSetpointC": demand.setpoints.cooling_c,
            "system": {"kind": "single", "unit": {"variant": "c1", "ducts": "luka_a_b_c", "equipmentReference": "synthetic"}},
            "infiltration": {"method": "measured", "qv10DmPerSM2": 0.4, "sourceReference": "synthetic"},
            "fans": {"method": "forfait", "current": "dc", "manufactureYear": 2020},
            "overventilation": {
                "hotWaterTimeFraction": vec![0.1; 12],
                "heatingFlowM3PerH": 900.0,
                "hotWaterFlowM3PerH": vec![900.0; 12],
                "sourceReference": "supplier"
            },
            "sourceReference": "synthetic"
        });
        input.demand.ventilation_flows.clear();
        input.demand.ventilation = Some(serde_json::from_value(ventilation).unwrap());
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let fractions = result
            .annex_q
            .as_ref()
            .and_then(|output| output.exhaust_air_heating_time_fraction.clone())
            .expect("Q.5.3 fractions");
        assert!(fractions[0] > 0.0);
        assert!(fractions.iter().all(|f| (0.0..=0.9 + 1e-12).contains(f)));
        // The used fractions reproduce the result when given explicitly.
        let mut explicit = input.clone();
        explicit
            .demand
            .ventilation
            .as_mut()
            .unwrap()
            .overventilation
            .as_mut()
            .unwrap()
            .heating_time_fraction = fractions.clone();
        let again = assess_space_heating_chain(&explicit);
        assert!(
            (again.monthly[0].generator_output_kwh - result.monthly[0].generator_output_kwh).abs()
                < 1e-9
        );
        // Overventilation raises the January demand above step 1 (f_H = 0).
        explicit
            .demand
            .ventilation
            .as_mut()
            .unwrap()
            .overventilation
            .as_mut()
            .unwrap()
            .heating_time_fraction = vec![0.0; 12];
        let step_1 = assess_space_heating_chain(&explicit);
        assert!(result.monthly[0].heating_need_kwh > step_1.monthly[0].heating_need_kwh);
        assert_ne!(result.input_fingerprint, again.input_fingerprint);
    }

    #[test]
    fn annex_q_heat_pump_above_55_degrees_is_calculated() {
        let input = annex_q_chain();
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let details = result.annex_q.as_ref().unwrap();
        let fraction = details.annex_q.energy_fraction;
        let efficiency = details.corrected_efficiency;
        assert_eq!(details.source_correction, 1.0);
        assert_eq!(result.generation_efficiency, Some(efficiency));
        for row in &result.monthly {
            let pump = fraction * row.generator_output_kwh;
            assert!((row.heat_pump_output_kwh - pump).abs() < 1e-9);
            // Electric backup covers 1 − F at COP 1.
            let expected = pump / efficiency + (row.generator_output_kwh - pump);
            assert!((row.generator_electricity_kwh - expected).abs() < 1e-9);
            assert_eq!(row.auxiliary_electricity_kwh, Some(0.0));
        }
        // The annual node input of Q.1 equals the chain output.
        let annual: f64 = result
            .monthly
            .iter()
            .map(|row| row.generator_output_kwh)
            .sum();
        assert!((details.annex_q.delivered_kwh - fraction * annual).abs() < 1e-6 * annual.max(1.0));
    }

    #[test]
    fn annex_q_applies_f_prac_and_accepts_monovalent_full_coverage() {
        let mut input = annex_q_chain();
        let Generator::HeatPumpAnnexQ(generator) = &mut input.generator else {
            panic!("annex Q generator expected");
        };
        generator.backup = None;
        let power = &mut generator.heat_pump.maximum_power;
        power.condition1.heating_power_kw *= 10.0;
        for condition in [&mut power.condition2, &mut power.condition3]
            .into_iter()
            .flatten()
        {
            condition.heating_power_kw *= 10.0;
        }
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let details = result.annex_q.as_ref().unwrap();
        // Table Q.6 rounding keeps the literal F just below 1.
        assert!(details.annex_q.energy_fraction < 1.0);
        assert!(details.annex_q.energy_fraction > 0.999);
        // 9.63: E = Q_out / (COP · 0,95), no c_source.
        assert_eq!(details.practice_factor, 0.95);
        assert_eq!(details.source_correction, 1.0);
        let efficiency = details.annex_q.generation_efficiency;
        assert!((details.corrected_efficiency - 0.95 * efficiency).abs() < 1e-12);
        for row in &result.monthly {
            assert!((row.heat_pump_output_kwh - row.generator_output_kwh).abs() < 1e-9);
            let expected = row.generator_output_kwh / (0.95 * efficiency);
            assert!((row.generator_electricity_kwh - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn annex_q_without_backup_needs_full_coverage() {
        let mut input = annex_q_chain();
        let Generator::HeatPumpAnnexQ(generator) = &mut input.generator else {
            panic!("annex Q generator expected");
        };
        generator.backup = None;
        // Switching off below 3 °C evaporator inlet leaves demand uncovered.
        generator.heat_pump.switch_off.min_evaporator_in_c = Some(3.0);
        assert!(codes(&input).contains(&"annex_q_backup_required"));
        let Generator::HeatPumpAnnexQ(generator) = &mut input.generator else {
            unreachable!()
        };
        generator.heat_pump.source = AnnexQSource::OutdoorAirWater;
        assert!(codes(&input).contains(&"regeneration_requires_ground_source"));
    }

    #[test]
    fn declared_distribution_loss_is_added_in_every_month() {
        let mut input = boiler_chain();
        input.distribution = Distribution::Declared {
            monthly_loss_kwh: vec![10.0; 12],
            source_reference: "pipe calculation".into(),
        };
        let result = assess_space_heating_chain(&input);
        // 9.24: the loss is not zeroed outside the heating season.
        for row in &result.monthly {
            assert_eq!(row.distribution_loss_kwh, 10.0);
            assert!((row.generator_output_kwh - (row.emission_input_kwh + 10.0)).abs() < 1e-9);
        }
    }

    #[test]
    fn fan_assisted_emitters_need_fans_and_inconsistent_balancing_is_rejected() {
        let mut input = boiler_chain();
        input.emission.system = EmissionSystem::FanAssistedRadiatorsOrConvectors;
        let result = assess_space_heating_chain(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.monthly.is_empty());
        assert!(result.annual_natural_gas_kwh.is_none());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "emission_fans_required"));
        // 9.22 with table 9.11: 4 fan convectors of 10 W over t_H;op.
        input.emission.fans = Some(crate::heating_emission::EmissionFans {
            kind: crate::heating_emission::EmissionFanKind::FanConvector,
            count: 4,
            tested_power_w: None,
            source_reference: "survey".into(),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        assert!(jan.emission_fan_electricity_kwh > 0.0);
        // 9.32a for a dwelling with an individual installation:
        // t_H·f_H;red·0,10 with f_H;red = 1 − 70/168.
        let hours = jan.emission_fan_electricity_kwh * 1000.0 / 40.0;
        let factor = ReductionFunction::Residential.heating_reduction_factor() * 0.10;
        let limit = heating_limit_hours(
            heating_limit_c(
                &std::array::from_fn(|month| result.demand.monthly[month].heating.need_kwh),
                input.demand.setpoints.heating_c,
            )
            .unwrap(),
            0,
        );
        assert!((hours - limit * factor).abs() < 1e-6 * limit.max(1.0) || hours <= 744.0 * factor);
        let mut plain = input.clone();
        if let Some(fans) = plain.emission.fans.as_mut() {
            fans.tested_power_w = Some(0.0);
        }
        let plain = assess_space_heating_chain(&plain);
        let extra = result.annual_auxiliary_electricity_kwh.unwrap()
            - plain.annual_auxiliary_electricity_kwh.unwrap();
        let fans: f64 = result
            .monthly
            .iter()
            .map(|row| row.emission_fan_electricity_kwh)
            .sum();
        assert!((extra - fans).abs() < 1e-6);
        input.emission.fans = None;
        input.emission.system = EmissionSystem::AirHeating;
        let result = assess_space_heating_chain(&input);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "emission_balancing_inconsistent"));
    }

    #[test]
    fn demand_errors_are_prefixed() {
        let mut input = boiler_chain();
        input.demand.usable_floor_area_m2 = 0.0;
        let result = assess_space_heating_chain(&input);
        assert!(result
            .issues
            .iter()
            .any(|item| item.path == "demand.usableFloorAreaM2"));
    }

    #[test]
    fn second_zone_adds_to_generator_output_with_its_own_emission() {
        let single = assess_space_heating_chain(&boiler_chain());
        let mut input = boiler_chain();
        let mut second = demand();
        second.zone_id = "rz-2".into();
        let mut floor = emission();
        floor.system = EmissionSystem::FloorHeating;
        input.additional_zones.push(ChainZone {
            demand: second,
            emission: floor,
            distribution: Distribution::HeatedZoneOnlySpaceHeating {
                source_reference: "inside".into(),
            },
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert_eq!(result.additional_zone_demands.len(), 1);
        assert!(result.emission_temperature_increment_k.is_none());
        let jan = &result.monthly[0];
        let need = single.monthly[0].heating_need_kwh;
        assert!((jan.heating_need_kwh - 2.0 * need).abs() < 1e-9);
        // Floor heating (3,5 K) is capped as well in January: 0,15 · need.
        assert!((jan.emission_loss_kwh - 2.0 * 0.15 * need).abs() < 1e-9);
        let duplicate = {
            let mut copy = input.clone();
            copy.additional_zones[0].demand.zone_id = "rz-1".into();
            assess_space_heating_chain(&copy)
        };
        assert!(duplicate
            .issues
            .iter()
            .any(|item| item.code == "zone_id_duplicate"));
        let mut broken = input;
        broken.additional_zones[0].emission.source_reference.clear();
        let result = assess_space_heating_chain(&broken);
        assert!(result
            .issues
            .iter()
            .any(|item| item.path == "additionalZones[0].emission.sourceReference"));
    }

    #[test]
    fn hybrid_generator_splits_output_between_heat_pump_and_boiler() {
        use crate::forfait_heat_pump_draft::assess_forfait_heat_pump_draft;
        let mut pump = heat_pump();
        pump.thermal_capacity_kw = Some(4.0);
        let cop = assess_forfait_heat_pump_draft(&pump).corrected_cop.unwrap();
        let mut input = boiler_chain();
        let Generator::GasBoiler(boiler) = &input.generator else {
            unreachable!()
        };
        let mut boiler = boiler.boiler.clone();
        boiler.role = crate::boiler_forfait_draft::BoilerRole::IndividualSupplementary;
        let boiler_efficiency = crate::boiler_forfait_draft::assess_boiler_forfait_draft(&boiler)
            .generation_efficiency
            .unwrap();
        input.generator = serde_json::from_value(json!({
            "kind": "hybrid_heat_pump",
            "designContext": "new_build",
            "generators": [
                {"id": "hp", "class": "heat_pump", "classificationReference": "design",
                 "nominalThermalPowerKw": 4.0, "powerReference": "plate",
                 "priorityEfficiency": cop, "efficiencyReference": "table"},
                {"id": "boiler", "class": "other_boiler", "classificationReference": "design",
                 "nominalThermalPowerKw": 6.0, "powerReference": "plate",
                 "priorityEfficiency": boiler_efficiency, "efficiencyReference": "table 9.25"}
            ],
            "forfait": pump,
            "boiler": boiler,
            "sourceSystem": "individual",
            "sourceSystemReference": "own unit"
        }))
        .unwrap();
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        assert!(
            jan.heat_pump_output_kwh > 0.0 && jan.heat_pump_output_kwh < jan.generator_output_kwh
        );
        let boiler_heat = jan.generator_output_kwh - jan.heat_pump_output_kwh;
        assert!((jan.natural_gas_kwh - boiler_heat / boiler_efficiency).abs() < 1e-6);
        assert!((jan.generator_electricity_kwh - jan.heat_pump_output_kwh / cop).abs() < 1e-6);
        assert_eq!(result.generation_efficiency, Some(cop));
        assert!(input.generator.heat_pump().is_some());
    }

    #[test]
    fn single_heat_pump_can_add_measured_auxiliary_energy() {
        use crate::heating_aux_draft::ElectricHeatPumpAuxMeasurements;
        let mut input = boiler_chain();
        input.generator = Generator::HeatPumpForfait(HeatPumpGenerator {
            forfait: heat_pump(),
            source_system: SourceSystem::Individual,
            source_system_reference: "own unit".into(),
            auxiliary_measurements: Some(HybridHeatPumpAuxMeasurements {
                generator_id: "hp".into(),
                generator_source_reference: "plate".into(),
                measurements: ElectricHeatPumpAuxMeasurements {
                    standby_electronics_w: 10.0,
                    delivery_pump_during_compressor_w: 200.0,
                    delivery_pump_pre_post_w: 90.0,
                    pump_pre_run_seconds: 300.0,
                    pump_post_run_seconds: 300.0,
                    average_compressor_on_seconds: 600.0,
                    mean_compressor_modulation: 0.5,
                    nominal_electric_drive_kw: 2.0,
                    measurement_source_reference: "measured".into(),
                    timing_source_reference: "measured".into(),
                },
            }),
            auxiliary: None,
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert!(result.annual_auxiliary_electricity_kwh.unwrap() > 0.0);
        let Generator::HeatPumpForfait(generator) = &mut input.generator else {
            unreachable!()
        };
        generator
            .auxiliary_measurements
            .as_mut()
            .unwrap()
            .generator_id = "other".into();
        let result = assess_space_heating_chain(&input);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_auxiliary_generator_mismatch"));
    }

    fn product_boiler() -> ProductBoilerGenerator {
        serde_json::from_value(json!({
            "boiler": {
                "technology": "condensing_gas", "fuel": "natural_gas",
                "placement": "heated_space", "draught": "fan_assisted",
                "control": "wall_hung_outdoor_compensated",
                "product": {
                    "nominalPowerKw": 24.0,
                    "fullLoad": {"method": "condensing", "efficiencyAt60": 0.97, "efficiencyAt30": 1.06},
                    "partLoadEfficiency": 1.08,
                    "standbyLossFactor": 0.006, "standbyTestTemperatureC": 50.0,
                    "auxiliaryStandbyW": 3.0, "auxiliaryIntermediateW": 15.0, "auxiliaryFullW": 40.0,
                    "sourceReference": "product sheet"
                },
                "equipmentReference": "type plate"
            }
        }))
        .unwrap()
    }

    #[test]
    fn product_boiler_follows_annex_m_with_table_9_15_hours() {
        let mut input = boiler_chain();
        let mut distribution = system(calculated_pump());
        distribution.installation = Installation::Individual;
        distribution.design_temperature_class = Some(DesignTemperatureClass::C45);
        input.distribution_system = Some(distribution);
        input.generator = Generator::ProductBoiler(Box::new(product_boiler()));
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let zone = &result.distribution.as_ref().unwrap().zones[0];
        let increment = result.emission_temperature_increment_k.unwrap();
        let Generator::ProductBoiler(generator) = &input.generator else {
            unreachable!()
        };
        for (index, row) in result.monthly.iter().enumerate() {
            let (_, ret) = supply_return_c(
                20.0,
                DesignTemperatureClass::C45,
                zone.heating_limit_c,
                OUTDOOR_TEMPERATURE_C[index],
                increment,
            );
            let expected = boiler_month(
                &generator.boiler,
                BoilerMonth {
                    heat_output_kwh: row.generator_output_kwh,
                    operating_hours: heating_limit_hours(zone.heating_limit_c, index),
                    month_hours: MONTH_HOURS[index],
                    return_temperature_c: ret,
                    outdoor_temperature_c: OUTDOOR_TEMPERATURE_C[index],
                    // 9.4.2: heated space at the setpoint.
                    ambient_temperature_c: Some(20.0),
                },
            );
            assert!((row.natural_gas_kwh - expected.input_kwh).abs() < 1e-9);
            let aux =
                row.auxiliary_electricity_kwh.unwrap() - row.distribution_auxiliary_electricity_kwh;
            assert!((aux - expected.auxiliary_electricity_kwh).abs() < 1e-9);
        }
        // M.12 in an installation room: ϑ_ztu of the distribution system.
        let mut room = input.clone();
        if let Generator::ProductBoiler(generator) = &mut room.generator {
            generator.boiler.placement = BoilerPlacement::InstallationRoom;
        }
        let mut cold = room.clone();
        cold.distribution_system
            .as_mut()
            .unwrap()
            .unheated_ambient_c = Some(vec![5.0; 12]);
        let warm_gas = assess_space_heating_chain(&room)
            .annual_natural_gas_kwh
            .unwrap();
        let cold_gas = assess_space_heating_chain(&cold)
            .annual_natural_gas_kwh
            .unwrap();
        // Table M.6 gives 13 °C; 5 °C raises the stand-by loss.
        assert!(cold_gas > warm_gas);
        // Without a calculated distribution the class must be given.
        input.distribution_system = None;
        assert!(codes(&input).contains(&"distribution_pump_input_required"));
    }

    #[test]
    fn product_boiler_fixture_calculates() {
        let input: SpaceHeatingChainInput = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-space-heating-chain-product-boiler-synthetic.json"
        ))
        .unwrap();
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let efficiency = result.generation_efficiency.unwrap();
        // Gross basis (f_Hs/Hi 1,11) with f_prac 0,95 and f_ctr;ls 1,03:
        // part load 1,08 gives about 1/(1,03·1,11/(0,95·1,08)) ≈ 0,90.
        assert!(efficiency > 0.8 && efficiency < 0.95, "{efficiency}");
        assert!(result
            .monthly
            .iter()
            .any(|row| row.generator_recoverable_loss_kwh > 0.0));
    }

    #[test]
    fn local_air_heater_follows_annex_n() {
        let mut input = boiler_chain();
        input.emission = serde_json::from_value(json!({
            "system": "local_heater", "balancing": "not_applicable",
            "control": "main_room_thermostat", "sourceReference": "survey"
        }))
        .unwrap();
        input.generator = serde_json::from_value(json!({
            "kind": "local_heater",
            "fuel": "natural_gas",
            "heater": {
                "heaterType": "air_heater_fan_burner", "control": "on_off",
                "productionPeriod": "after2005", "condensing": false, "pilotFlame": false,
                "ventilation": "none", "location": "heated_space_free", "fan": "axial",
                "envelopeInsulation": "well_insulated_maintained",
                "product": {"inputFullKw": 20.0},
                "sourceReference": "type plate"
            }
        }))
        .unwrap();
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let Generator::LocalHeater(generator) = &input.generator else {
            unreachable!()
        };
        let resolved = resolve_heater(&generator.heater, "h").unwrap();
        let need: [f64; 12] = std::array::from_fn(|m| result.monthly[m].heating_need_kwh);
        let limit = heating_limit_c(&need, 20.0).unwrap();
        let jan = &result.monthly[0];
        let expected = heater_month(
            &generator.heater,
            &resolved,
            HeaterMonth {
                heat_required_kwh: jan.generator_output_kwh,
                operating_hours: heating_limit_hours(limit, 0),
                indoor_c: 20.0,
                outdoor_c: OUTDOOR_TEMPERATURE_C[0],
            },
        );
        // N.3: gross calorific value with f_Hs/Hi 1,11 (table M.3).
        assert!((jan.natural_gas_kwh - 1.11 * expected.input_kwh).abs() < 1e-9);
        assert!(jan.natural_gas_kwh > jan.generator_output_kwh);
    }

    #[test]
    fn forfait_local_heater_uses_table_9_25() {
        let mut input = boiler_chain();
        input.emission = serde_json::from_value(json!({
            "system": "local_heater", "balancing": "not_applicable",
            "control": "main_room_thermostat", "sourceReference": "survey"
        }))
        .unwrap();
        input.generator = Generator::ForfaitHeater(ForfaitHeaterGenerator {
            kind: ForfaitHeaterKind::LocalWithFlue,
            fuel: ForfaitHeaterFuel::Oil,
            equipment_reference: "survey".into(),
            pilot_flames: 0,
            auxiliary: other_aux(1, Some(8.0)),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        assert!((jan.oil_kwh - jan.generator_output_kwh / 0.65).abs() < 1e-9);
        // The generator round-trips through JSON despite the `kind` tag.
        let json = serde_json::to_value(&input.generator).unwrap();
        assert_eq!(json["kind"], "forfait_heater");
        assert_eq!(json["heaterKind"], "local_with_flue");
        let parsed: Generator = serde_json::from_value(json).unwrap();
        assert!(matches!(parsed, Generator::ForfaitHeater(_)));
        assert_eq!(jan.natural_gas_kwh, 0.0);
        input.generator = Generator::ForfaitHeater(ForfaitHeaterGenerator {
            kind: ForfaitHeaterKind::AirHeaterHr107,
            fuel: ForfaitHeaterFuel::Oil,
            equipment_reference: "survey".into(),
            pilot_flames: 0,
            auxiliary: other_aux(1, Some(8.0)),
        });
        assert!(codes(&input).contains(&"forfait_air_heater_gas_only"));
        // §9.6.2.1: two pilot flames of gas air heaters, 695 kWh each.
        input.generator = Generator::ForfaitHeater(ForfaitHeaterGenerator {
            kind: ForfaitHeaterKind::AirHeaterConventional,
            fuel: ForfaitHeaterFuel::NaturalGas,
            equipment_reference: "survey".into(),
            pilot_flames: 2,
            auxiliary: other_aux(1, Some(8.0)),
        });
        let gas = assess_space_heating_chain(&input);
        let jan = &gas.monthly[0];
        let pilot = 2.0 * 695.0 * 744.0 / 8760.0;
        assert!((jan.natural_gas_kwh - (jan.generator_output_kwh / 0.75 + pilot)).abs() < 1e-9);
    }

    #[test]
    fn individual_boiler_auxiliary_from_annex_o_measurements() {
        let mut input = boiler_chain();
        let Generator::GasBoiler(boiler) = &mut input.generator else {
            unreachable!()
        };
        boiler.auxiliary_measurements = Some(
            serde_json::from_value(json!({
                "nominalLoadKw": 24.0, "standbyElectronicsW": 2.0, "gasValveW": 5.0,
                "fan": {"method": "single_speed", "powerW": 20.0},
                "pump": {"method": "staged", "operationW": 40.0, "prePostRunW": 30.0},
                "pumpPostRunS": 60.0,
                "loadCurve": [{"timeS": 200.0, "meanLoad": 0.5}, {"timeS": 300.0, "meanLoad": 0.5}],
                "sourceReference": "test report"
            }))
            .unwrap(),
        );
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let Generator::GasBoiler(boiler) = &input.generator else {
            unreachable!()
        };
        let constants =
            auxiliary_constants(boiler.auxiliary_measurements.as_ref().unwrap(), "a").unwrap();
        let jan = &result.monthly[0];
        let expected = monthly_auxiliary_kwh(&constants, jan.natural_gas_kwh);
        assert!((jan.auxiliary_electricity_kwh.unwrap() - expected).abs() < 1e-9);
    }

    fn other_aux(devices: u32, power: Option<f64>) -> Option<OtherGeneratorAuxiliary> {
        Some(OtherGeneratorAuxiliary {
            electrically_connected_devices: devices,
            nominal_power_kw: power,
            source_reference: "plate".into(),
        })
    }

    fn calculated_pump() -> DistributionPump {
        DistributionPump::Calculated(PumpInput {
            heat_meter_present: true,
            max_pipe_length_m: None,
            design_flow_m3_per_h: None,
            energy_efficiency_index: None,
            electric_power_kw: None,
            source_reference: "design".into(),
        })
    }

    fn system(pump: DistributionPump) -> DistributionSystem {
        DistributionSystem {
            design_temperature_class: None,
            installation: Installation::Collective,
            usage_function: ReductionFunction::Residential,
            pipes_also_for_hot_water: false,
            collective_hot_water_delivery_set: false,
            connected_storeys: 4,
            pipe_transmittance: PipeTransmittance::Forfait {
                insulation: crate::heating_distribution::PipeInsulation::Insulated {
                    period: crate::heating_distribution::InsulationPeriod::From1995,
                },
            },
            unheated_pipe_transmittance: None,
            valves_insulated: false,
            actual_pipe_length_m: None,
            unheated_pipe_length_m: None,
            unheated_ambient_c: None,
            buffer_vessel: None,
            pump,
            source_reference: "installation survey".into(),
        }
    }

    fn codes(input: &SpaceHeatingChainInput) -> Vec<&'static str> {
        assess_space_heating_chain(input)
            .issues
            .iter()
            .map(|item| item.code)
            .collect()
    }

    #[test]
    fn external_heat_delivers_generator_output_as_dh() {
        let mut input = boiler_chain();
        input.generator = Generator::ExternalHeat(ExternalHeatGenerator {
            supplier_reference: "heat supply contract".into(),
            quality_declaration_present: false,
            auxiliary: other_aux(1, None),
        });
        // 9.4.4: external heat needs its distribution pump stated.
        assert_eq!(codes(&input), vec!["distribution_pump_input_required"]);
        input.distribution_system = Some(system(DistributionPump::NoneOnSite {
            source_reference: "delivery without heat exchanger".into(),
        }));
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        assert_eq!(jan.district_heat_kwh, jan.generator_output_kwh);
        assert_eq!(jan.natural_gas_kwh, 0.0);
        assert_eq!(result.generation_efficiency, Some(1.0));
        // 9.91: one device, 10 W stand-by, no burner term.
        assert!((jan.auxiliary_electricity_kwh.unwrap() - 7.44).abs() < 1e-9);
        input.generator = Generator::ExternalHeat(ExternalHeatGenerator {
            supplier_reference: String::new(),
            quality_declaration_present: true,
            auxiliary: None,
        });
        let found = codes(&input);
        assert!(!found.contains(&"external_heat_declaration_unsupported"));
        assert!(found.contains(&"source_reference_required"));
        assert!(found.contains(&"generator_auxiliary_input_required"));
        input.distribution_system = Some(system(DistributionPump::IncludedInGeneratorAuxiliary));
        assert!(codes(&input).contains(&"distribution_pump_not_in_generator_auxiliary"));
    }

    #[test]
    fn electric_resistance_and_biomass_generators() {
        let mut input = boiler_chain();
        input.emission.system = EmissionSystem::LocalHeater;
        input.emission.balancing = HydronicBalancing::NotApplicable;
        input.generator = Generator::ElectricResistance(ElectricResistanceGenerator {
            equipment_reference: "panel heaters".into(),
            auxiliary: other_aux(4, None),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert_eq!(
            result.monthly[0].generator_electricity_kwh,
            result.monthly[0].generator_output_kwh
        );
        // 9.91: four panels at 10 W in January.
        assert!((result.monthly[0].auxiliary_electricity_kwh.unwrap() - 4.0 * 7.44).abs() < 1e-9);
        assert!(input.generator.heat_pump().is_none());

        input.emission = emission();
        input.distribution_system = Some(system(calculated_pump()));
        input.generator = Generator::Biomass(BiomassGenerator {
            appliance: BiomassAppliance::CentralBoiler,
            location: BiomassLocation::OutsideThermalBoundary,
            annex_r_compliant_at_most_500_kw: true,
            annex_r_reference: "type test".into(),
            equipment_reference: "plate".into(),
            sole_heating_in_served_rooms: None,
            automatic_fuel_feed: false,
            auxiliary: other_aux(1, Some(20.0)),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        assert!((jan.biomass_kwh - jan.generator_output_kwh / 0.75).abs() < 1e-9);
        // 9.91/9.92: 10 W stand-by + 10 W/kW for the automatically fired boiler.
        let on = (jan.generator_output_kwh * 1.1 / 20.0).min(744.0);
        let generator_aux = (10.0 * 744.0 + 10.0 * 20.0 * on) / 1000.0;
        assert!(
            (jan.auxiliary_electricity_kwh.unwrap()
                - generator_aux
                - jan.distribution_auxiliary_electricity_kwh)
                .abs()
                < 1e-9
        );
        assert!(jan.distribution_auxiliary_electricity_kwh > 0.0);
        assert_eq!(
            biomass_efficiency(
                BiomassAppliance::PelletStove,
                BiomassLocation::InsideThermalBoundary
            ),
            Some(0.725)
        );
        assert_eq!(
            biomass_efficiency(
                BiomassAppliance::PelletStove,
                BiomassLocation::OutsideThermalBoundary
            ),
            None
        );

        input.generator = Generator::Biomass(BiomassGenerator {
            appliance: BiomassAppliance::PelletStove,
            location: BiomassLocation::OutsideThermalBoundary,
            annex_r_compliant_at_most_500_kw: false,
            annex_r_reference: "x".into(),
            equipment_reference: "x".into(),
            sole_heating_in_served_rooms: Some(false),
            automatic_fuel_feed: true,
            auxiliary: other_aux(1, None),
        });
        let found = codes(&input);
        assert!(found.contains(&"biomass_class_unsupported"));
        assert!(found.contains(&"biomass_efficiency_unavailable"));
        assert!(found.contains(&"biomass_stove_not_sole_heating"));
        assert!(found.contains(&"generator_nominal_power_required"));
    }

    #[test]
    fn heat_pump_forfait_auxiliary_follows_9_85() {
        let mut input = boiler_chain();
        input.generator = Generator::HeatPumpForfait(HeatPumpGenerator {
            forfait: heat_pump(),
            source_system: SourceSystem::Individual,
            source_system_reference: "own unit".into(),
            auxiliary_measurements: None,
            auxiliary: None,
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let jan = &result.monthly[0];
        let expected = 43.8 / 12.0 + 0.132 * jan.generator_electricity_kwh / (0.7 * 3.0);
        assert!((jan.auxiliary_electricity_kwh.unwrap() - expected).abs() < 1e-9);
        assert!((heat_pump_forfait_auxiliary_kwh(0.0) - 3.65).abs() < 1e-12);

        let Generator::HeatPumpForfait(generator) = &mut input.generator else {
            unreachable!()
        };
        generator.forfait.design_supply_temperature_c = Some(60.0);
        assert!(codes(&input).contains(&"heat_pump_above55_requires_annex_q"));
    }

    #[test]
    fn other_generator_auxiliary_for_a_building_part() {
        let auxiliary = other_aux(2, Some(100.0)).unwrap();
        // Part of 25 %: system output 4× part output; on-time capped at t.
        let value = other_generator_auxiliary_kwh(&auxiliary, 1.0, 5000.0, 720.0, 0.25);
        let on = (5000.0 / 0.25 * 1.1 / 100.0_f64).min(720.0);
        assert!((value - (20.0 * 720.0 + 100.0 * on) * 0.25 / 1000.0).abs() < 1e-12);
        let capped = other_generator_auxiliary_kwh(&auxiliary, 1.0, 100_000.0, 720.0, 1.0);
        assert!((capped - (20.0 * 720.0 + 100.0 * 720.0) / 1000.0).abs() < 1e-12);
    }

    fn collective_boiler_chain() -> SpaceHeatingChainInput {
        let mut input = boiler_chain();
        let Generator::GasBoiler(boiler) = &mut input.generator else {
            unreachable!()
        };
        boiler.boiler.role = BoilerRole::Collective;
        boiler.auxiliary = other_aux(1, Some(200.0));
        input.distribution = Distribution::Calculated {
            heating_limit_extra_kwh: None,
            source_reference: "9.26".into(),
        };
        input.distribution_system = Some(system(calculated_pump()));
        input
    }

    #[test]
    fn calculated_distribution_follows_9_26_to_9_51() {
        let mut input = collective_boiler_chain();
        input.collective_connection = Some(CollectiveConnection {
            connected_usable_area_m2: 1000.0,
            source_reference: "building plan".into(),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let summary = result.distribution.as_ref().unwrap();
        let fraction = input.demand.usable_floor_area_m2 / 1000.0;
        assert!((summary.building_fraction - fraction).abs() < 1e-12);
        assert!((summary.pipe_length_m - 640.0).abs() < 1e-9);
        assert!((summary.unheated_pipe_length_m - 96.0).abs() < 1e-9);
        assert_eq!(summary.psi_zone_w_per_mk, 0.3);
        let zone = &summary.zones[0];
        let limit = zone.heating_limit_c;
        assert!((8..=16).contains(&limit));
        // Collective residential: f_pmp = 1, f_H;red = 1 − 70/168.
        let reduction = 1.0 - 70.0 / 168.0;
        let t_jan = heating_limit_hours(limit, 0) * reduction;
        assert!((zone.operating_hours[0] - t_jan).abs() < 1e-9);
        // 9.26: only the unheated part (space heating only), 15 % of L_si.
        let increment = temperature_increment_k(&input.emission);
        let (supply, ret) = supply_return_c(
            input.demand.setpoints.heating_c,
            DesignTemperatureClass::C90,
            limit,
            OUTDOOR_TEMPERATURE_C[0],
            increment,
        );
        let mean = (supply + ret) / 2.0;
        let unheated = 96.0 + 0.15 / 0.3 * 96.0;
        let loss = 0.3 * (mean - 13.0) * unheated * t_jan / 1000.0 * fraction;
        let jan = &result.monthly[0];
        assert!((jan.distribution_loss_kwh - loss).abs() < 1e-9);
        // Pump: L_max = 35 + 24 + 0,13·1000/4; Δp_add radiator + meter + 90/70 boiler.
        let pump = summary.pump.as_ref().unwrap();
        assert!((pump.max_pipe_length_m - 91.5).abs() < 1e-9);
        assert!((pump.pressure_kpa - (1.4 * 0.1 * 91.5 + 2.0 + 10.0 + 10.0)).abs() < 1e-9);
        assert_eq!(pump.balancing_factor, 1.15);
        let pump_energy = pump.hydraulic_power_kw * t_jan * 1.15 * pump.energy_factor * fraction;
        assert!((jan.distribution_auxiliary_electricity_kwh - pump_energy).abs() < 1e-9);
        assert!((jan.distribution_auxiliary_to_medium_kwh - 0.75 * pump_energy).abs() < 1e-9);
        // 9.24: generator output includes the loss minus the pump heat to the medium.
        assert!(
            (jan.generator_output_kwh - (jan.emission_input_kwh + loss - 0.75 * pump_energy)).abs()
                < 1e-9
        );
        // Recoverable: no in-zone pipe loss, a quarter of the pump energy.
        assert!((jan.recoverable_loss_kwh - 0.25 * pump_energy).abs() < 1e-9);
        assert_eq!(result.zone_recoverable_losses.len(), 1);
        // Summer: t_H,op > 0, so the distribution loss continues in July.
        let july = &result.monthly[6];
        assert!(july.distribution_loss_kwh > 0.0 && july.generator_output_kwh > 0.0);
        // Total auxiliary = 9.91 collective boiler + pump.
        let Generator::GasBoiler(boiler) = &input.generator else {
            unreachable!()
        };
        let generator_aux = other_generator_auxiliary_kwh(
            boiler.auxiliary.as_ref().unwrap(),
            1.0,
            jan.generator_output_kwh,
            744.0,
            fraction,
        );
        assert!(
            (jan.auxiliary_electricity_kwh.unwrap() - generator_aux - pump_energy).abs() < 1e-9
        );
    }

    #[test]
    fn hot_water_pipes_count_in_zone_and_buffer_adds_node_loss() {
        let mut input = collective_boiler_chain();
        input.distribution = Distribution::Calculated {
            heating_limit_extra_kwh: Some(vec![0.0; 12]),
            source_reference: "9.26".into(),
        };
        let distribution = input.distribution_system.as_mut().unwrap();
        distribution.pipes_also_for_hot_water = true;
        distribution.collective_hot_water_delivery_set = true;
        distribution.pipe_transmittance = PipeTransmittance::InsulatedInAir {
            pipe_outer_diameter_m: 0.028,
            insulated_diameter_m: 0.068,
            insulation_lambda: 0.035,
            surface_coefficient: None,
        };
        distribution.buffer_vessel = Some(BufferVessel {
            volume_l: 500.0,
            standing_loss_w: None,
            label: None,
            produced_from_2018: true,
            in_heated_space: false,
            constant_temperature: true,
            source_reference: "plant room".into(),
        });
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let summary = result.distribution.as_ref().unwrap();
        let zone = &summary.zones[0];
        // Whole month in operation and ϑ_mean ≥ 65 °C.
        assert_eq!(zone.operating_hours[6], 744.0);
        assert!(zone
            .mean_medium_temperature_c
            .iter()
            .all(|value| *value >= 65.0));
        let jan = &result.monthly[0];
        assert!(jan.recoverable_loss_kwh > 0.25 * jan.distribution_auxiliary_electricity_kwh);
        // Buffer: label C, 90 °C set (constant), 13 °C, whole month.
        let standing = 14.33 + 7.13 * 500.0_f64.powf(0.4);
        let expected = 744.0 / 1000.0 * standing / 45.0 * (90.0 - 13.0);
        assert!((jan.node_loss_kwh - expected).abs() < 1e-9);

        // Forfait Ψ already contains the buffer loss.
        let mut forfait = input.clone();
        forfait
            .distribution_system
            .as_mut()
            .unwrap()
            .pipe_transmittance = system(calculated_pump()).pipe_transmittance;
        assert!(codes(&forfait).contains(&"buffer_vessel_included_in_forfait_psi"));
        // The calculated route needs the system data.
        input.distribution_system = None;
        let found = codes(&input);
        assert!(found.contains(&"distribution_system_required"));
        assert!(found.contains(&"distribution_pump_input_required"));
    }
    #[test]
    fn collective_fixture_calculates_distribution_and_pump() {
        let input: SpaceHeatingChainInput = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-space-heating-chain-collective-synthetic.json"
        ))
        .unwrap();
        let result = assess_space_heating_chain(&input);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let summary = result.distribution.as_ref().unwrap();
        assert!((summary.building_fraction - 100.0 / 1200.0).abs() < 1e-12);
        assert_eq!(
            summary.design_temperature_class,
            DesignTemperatureClass::C70
        );
        assert!(summary.pump.is_some());
        assert!(result
            .monthly
            .iter()
            .all(|row| row.auxiliary_electricity_kwh.is_some()));
        assert!(result.annual_auxiliary_electricity_kwh.unwrap() > 0.0);
    }
}
