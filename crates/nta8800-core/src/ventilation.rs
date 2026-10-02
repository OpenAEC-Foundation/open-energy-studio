//! Ventilation, infiltration and fan energy, NTA 8800:2025+C1:2026 chapter 11.
//!
//! Per zone and month, separately for the heating and the cooling balance
//! (11.2), this module determines:
//!
//! - the required outdoor-air flow `q_V;ODA;req` (11.22/11.23) from the
//!   system variant (tables 11.5–11.7), the specific capacity of table 11.8
//!   (11.55–11.65) and the duct/AHU leakage factors (table 11.9);
//! - the reference flows at 1 Pa for purge ventilation (11.66–11.69,
//!   table 11.10), ventilative cooling (11.70–11.79), open combustion
//!   appliances (11.80–11.83, tables 11.11/11.12) and infiltration
//!   (11.84–11.86, tables 11.13/11.14);
//! - the internal reference pressure from the mass balance 11.5 with the
//!   prescribed iteration and bisection routine of 11.2.1.6 over the
//!   openings of table 11.1 (11.1–11.18), and from it the effective flows
//!   (11.19–11.21, table 11.4);
//! - the supply-air temperatures (11.99–11.130) including frost protection
//!   (table 11.16), heat recovery (11.106a–11.109, tables 11.17/11.18),
//!   recirculation, fan heat and duct losses (table 11.19), and electric
//!   preheating in natural supply grilles (11.123–11.128);
//! - the fan, frost-protection and grille-preheating electricity
//!   (11.105/11.106, 11.125–11.142, tables 11.20–11.23).
//!
//! The effective flows and their temperatures are returned as chapter 7
//! ventilation flows (`H_ve;k = ρ_a·c_a·q_V;k/3600`, 7.19).
//!
//! Air-handling units that heat or cool the supply air (table 11.15,
//! 11.100/11.101, 11.114–11.121) and specific gas appliances with a
//! variable flue (table 11.11 footnote a) are rejected as unsupported.
//!
//! Transcribed from the licensed NTA 8800:2025+C1:2026, pp. 436–520.
//! Interpretation choices are listed in [`INTERPRETATIONS`].

use serde::{Deserialize, Serialize};

use crate::climate::{
    ARGII_TEMPERATURE_C as VENTILATIVE_COOLING_TEMPERATURE_C, COLD_RECOVERY_SUPPLY_TEMPERATURE_C,
    MONTH_HOURS, OUTDOOR_TEMPERATURE_C, WIND_SPEED_M_PER_S,
};
use crate::monthly_demand::{VentilationFlow, VentilationMonth};

/// ρ_a;ref, kg/m³ (11.1).
pub const AIR_DENSITY_REF: f64 = 1.205;
/// T_e;ref and T_ref, K.
pub const REFERENCE_TEMPERATURE_K: f64 = 293.0;
pub const GRAVITY: f64 = 9.81;
/// ρ_a·c_a for the chapter 7 conductance (7.19), J/(m³·K).
pub const VOLUMETRIC_HEAT_CAPACITY: f64 = 1.205 * 1005.0;
/// f_prac;req (11.22).
pub const PRACTICE_FACTOR_REQUIRED: f64 = 0.95;
/// f_prac;vent (11.132).
pub const PRACTICE_FACTOR_FANS: f64 = 0.9;

/// Table 11.2 flow exponents.
pub const EXPONENT_LEAKAGE: f64 = 0.67;
pub const EXPONENT_VENT: f64 = 0.5;
pub const EXPONENT_PURGE: f64 = 0.5;
pub const EXPONENT_COMBUSTION: f64 = 0.5;

/// Table 11.7 temperature-weighted time fractions.
pub const TAU_SYS_C: [f64; 12] = [
    0.01, 0.11, 0.08, 0.30, 0.73, 0.88, 1.00, 0.93, 0.89, 0.56, 0.17, 0.03,
];
pub const TAU_BYPASS: [f64; 12] = TAU_SYS_C;
pub const TAU_PURGE_COOLING: [f64; 12] = [
    0.01, 0.01, 0.02, 0.04, 0.06, 0.08, 0.08, 0.08, 0.06, 0.04, 0.02, 0.01,
];
pub const TAU_VENTILATIVE_COOLING: [f64; 12] = [
    0.00, 0.02, 0.00, 0.00, 0.46, 0.75, 0.81, 0.79, 0.75, 0.26, 0.05, 0.00,
];
pub const TAU_COLD_RECOVERY: [f64; 12] = [
    0.00, 0.00, 0.00, 0.00, 0.15, 0.08, 0.17, 0.17, 0.08, 0.00, 0.00, 0.00,
];
/// τ_argI;H for every function and τ_argI;C for utility (11.2.3.2).
pub const TAU_PURGE_DEFAULT: f64 = 0.01;

/// Table 11.16 ΔT_defrost for the residential function, K.
pub const DEFROST_RESIDENTIAL_K: [f64; 12] =
    [0.2, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2, 0.2];

/// Table 11.19 ΔT_SUP;du;nc;in, K (situations 2 and 3; situation 1 is 0).
const DUCT_LOSS_SITUATION_2_K: [f64; 12] = [
    2.82, 2.47, 2.29, 1.73, 0.86, 0.63, 0.32, 0.25, 0.71, 1.56, 1.95, 2.6,
];
const DUCT_LOSS_SITUATION_3_K: [f64; 12] = [
    5.72, 4.99, 4.63, 3.51, 1.73, 1.28, 0.64, 0.5, 1.44, 3.16, 3.95, 5.26,
];

/// Table 11.15 θ_SUP;dis;out of an AHU with heating and cooling, °C, for
/// functions other than sport (sport: 16 °C all year).
const AHU_SUPPLY_OTHER_C: [f64; 12] = [
    18.0, 18.0, 17.5, 17.5, 17.0, 16.5, 16.5, 16.5, 17.0, 17.0, 17.5, 18.0,
];
const AHU_SUPPLY_SPORT_C: f64 = 16.0;
/// η of the AHU heating and cooling coils (11.116, 11.120).
const AHU_COIL_EFFICIENCY: f64 = 0.98;
/// c_a for 11.115/11.119, kWh/(kg·K).
const AIR_HEAT_CAPACITY_KWH: f64 = 0.000_027_9;

/// Interpretation choices where the norm text leaves room; recorded in the
/// verification dossier.
pub const INTERPRETATIONS: &[&str] = &[
    "table 11.8 f_τ for dwellings uses the whole-dwelling area when given, otherwise the mean dwelling area A_g;zi/N_woon",
    "fan energy (11.132) uses q_V;ODA;req of the heating balance",
    "ventilative cooling flows are computed for the whole zone and split over airflow zones by 11.6–11.10",
    "ΔC_p for cross ventilation uses the table 11.3 class of the building height",
    "the 11.14 accuracy uses zone totals for every airflow zone",
    "at H = 50 m the zone is split into two airflow zones (11.6/11.7)",
    "9.29 is applied literally: Q_air uses θ_SUP;dis;out − Δθ_hr − Δθ_rca − Δθ_fan, added to Q_H;ve without changing H_ve or τ",
    "q_V;comb;out counts as an outflow in the mass balance (11.3), although 11.82 defines it as positive",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Residential,
    Utility,
}

/// Use functions as distinguished by tables 11.8, 11.10, 11.15.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VentilationFunction {
    Residential,
    AssemblyChildCare,
    OtherAssembly,
    Cell,
    HealthcareBedArea,
    OtherHealthcare,
    Office,
    LodgingBuilding,
    Education,
    Sport,
    Retail,
}

impl VentilationFunction {
    /// Table 11.8 q_usi;spec, dm³/(s·m²).
    fn specific_capacity(self) -> f64 {
        match self {
            Self::Residential => 0.50,
            Self::AssemblyChildCare => 2.78,
            Self::OtherAssembly => 1.71,
            Self::Cell => 0.84,
            Self::HealthcareBedArea => 2.04,
            Self::OtherHealthcare => 1.11,
            Self::Office => 1.11,
            Self::LodgingBuilding => 0.84,
            Self::Education => 3.64,
            Self::Sport => 0.46,
            Self::Retail => 0.28,
        }
    }

    /// Table 11.8 f_τ; dwellings depend on the dwelling area.
    fn occupancy_factor(self, dwelling_area_m2: f64) -> f64 {
        match self {
            Self::Residential => (0.38 + dwelling_area_m2 * 0.006).min(0.8),
            Self::AssemblyChildCare => 0.30,
            Self::OtherAssembly => 0.15,
            Self::Cell => 0.80,
            Self::HealthcareBedArea => 0.80,
            Self::OtherHealthcare => 0.30,
            Self::Office => 0.30,
            Self::LodgingBuilding => 0.40,
            Self::Education => 0.30,
            Self::Sport => 0.30,
            Self::Retail => 0.40,
        }
    }

    /// Table 11.10 q_ve;spec;spui, dm³/(s·m²).
    fn purge_capacity(self) -> f64 {
        match self {
            Self::Residential => 4.2,
            Self::AssemblyChildCare => 4.8,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FunctionArea {
    pub function: VentilationFunction,
    pub area_m2: f64,
    /// Room with a swimming pool: q_usi;spec of the sport function × 2
    /// (§11.2.2.5.1, p. 470).
    #[serde(default)]
    pub swimming_pool: bool,
}

/// Table 11.5 system variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemVariant {
    A1,
    A2a,
    A2b,
    A2c,
    B1,
    B2,
    B3,
    C1,
    C2a,
    C2b,
    C2c,
    C3a,
    C3b,
    C3c,
    C4a,
    C4b,
    C4c,
    C5a,
    C5b,
    D1,
    D2,
    D3,
    D4a,
    D4b,
    D5a,
    D5b,
    D5c,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VentSysOp {
    Natural,
    Supply,
    Extract,
    Balanced,
}

impl SystemVariant {
    pub fn op(self) -> VentSysOp {
        use SystemVariant::*;
        match self {
            A1 | A2a | A2b | A2c => VentSysOp::Natural,
            B1 | B2 | B3 => VentSysOp::Supply,
            C1 | C2a | C2b | C2c | C3a | C3b | C3c | C4a | C4b | C4c | C5a | C5b => {
                VentSysOp::Extract
            }
            D1 | D2 | D3 | D4a | D4b | D5a | D5b | D5c => VentSysOp::Balanced,
        }
    }

    /// Table 11.5 f_ctrl; `None` where the table says "N.v.t.".
    pub fn control_factor(self, category: Category) -> Option<f64> {
        use SystemVariant::*;
        let (residential, utility) = match self {
            A1 => (Some(1.00), Some(0.95)),
            A2a => (Some(0.85), Some(0.81)),
            A2b => (Some(0.90), Some(0.85)),
            A2c => (Some(0.93), Some(0.88)),
            B1 => (Some(1.00), Some(1.02)),
            B2 => (Some(0.85), Some(0.87)),
            B3 => (Some(0.57), Some(0.58)),
            C1 => (Some(1.00), Some(1.32)),
            C2a => (Some(0.83), Some(1.10)),
            C2b => (Some(0.88), Some(1.17)),
            C2c => (Some(0.93), Some(1.23)),
            C3a => (Some(0.90), Some(1.10)),
            C3b => (Some(0.75), Some(0.92)),
            C3c => (Some(0.68), Some(0.84)),
            C4a => (Some(0.80), None),
            C4b => (Some(0.52), Some(0.61)),
            C4c => (Some(0.59), Some(0.82)),
            C5a => (Some(0.56), None),
            C5b => (Some(0.55), None),
            D1 => (Some(1.00), Some(1.00)),
            D2 => (Some(1.00), Some(1.00)),
            D3 => (Some(0.80), Some(1.00)),
            D4a => (Some(0.90), Some(1.00)),
            D4b => (Some(0.80), Some(1.00)),
            D5a => (Some(0.52), Some(0.67)),
            D5b => (Some(0.52), Some(0.67)),
            D5c => (Some(0.59), None),
        };
        match category {
            Category::Residential => residential,
            Category::Utility => utility,
        }
    }

    /// 11.2.2.4.2 note 3: demand-controlled variants.
    pub fn demand_controlled(self) -> bool {
        use SystemVariant::*;
        matches!(
            self,
            B2 | B3
                | C3a
                | C3b
                | C3c
                | C4a
                | C4b
                | C4c
                | C5a
                | C5b
                | D3
                | D4a
                | D4b
                | D5a
                | D5b
                | D5c
        )
    }

    /// 11.142 f_systype.
    fn fan_system_factor(self) -> f64 {
        match self.op() {
            VentSysOp::Natural => 0.0,
            VentSysOp::Supply | VentSysOp::Extract => 1.0,
            VentSysOp::Balanced => 2.0,
        }
    }
}

/// Table 11.9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DuctAirtightness {
    Unknown,
    LukaABC,
    LukaD,
    NoDucts,
}

impl DuctAirtightness {
    fn factor(self) -> f64 {
        match self {
            Self::Unknown => 1.10,
            Self::LukaABC => 1.05,
            Self::LukaD | Self::NoDucts => 1.00,
        }
    }
}

/// Table 11.19 situations for supply ducts from the AHU outside the
/// thermal zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DuctOutsideSituation {
    None,
    Situation1,
    Situation2,
    Situation3,
}

impl DuctOutsideSituation {
    /// Table 11.19 (p. 508): situation 1 gives 0,0 K; situations 2 and 3
    /// the two monthly columns.
    fn delta_k(self, month_index: usize) -> f64 {
        match self {
            Self::None | Self::Situation1 => 0.0,
            Self::Situation2 => DUCT_LOSS_SITUATION_2_K[month_index],
            Self::Situation3 => DUCT_LOSS_SITUATION_3_K[month_index],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AirHandlingUnit {
    pub inside_thermal_zone: bool,
    pub supply_ducts_outside: DuctOutsideSituation,
    /// Reheating coil in the supply air (11.3.2, 11.118–11.121).
    #[serde(default)]
    pub heating_coil: bool,
    /// Cooling coil in the supply air (11.3.2, 11.114–11.117).
    #[serde(default)]
    pub cooling_coil: bool,
}

/// Table 11.18 exchanger types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatExchanger {
    None,
    RunAroundCoilAhu,
    PlateOrTube,
    CrossFlow,
    TwoElement,
    HeatPipe,
    Rotary,
    Enthalpy,
    CounterFlowAluminium,
    CounterFlowPlastic,
}

impl HeatExchanger {
    fn efficiency(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::RunAroundCoilAhu => 0.40,
            Self::PlateOrTube => 0.65,
            Self::CrossFlow => 0.55,
            Self::TwoElement | Self::HeatPipe => 0.60,
            Self::Rotary => 0.70,
            Self::Enthalpy | Self::CounterFlowAluminium => 0.75,
            Self::CounterFlowPlastic => 0.80,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EfficiencyStandard {
    En13141_7,
    En13141_8,
    En13142,
    En13053,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum HeatRecoveryEfficiency {
    Declared {
        value: f64,
        standard: EfficiencyStandard,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    Table {
        exchanger: HeatExchanger,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Bypass {
    None,
    /// 100 % bypass; cold recovery (11.106a) needs the BCRG evidence.
    Full {
        #[serde(default, rename = "coldRecoveryEvidence")]
        cold_recovery_evidence: Option<String>,
    },
    Partial {
        fraction: f64,
    },
    /// 11.3.2.2 defaults when the bypass cannot be established.
    Unknown {
        #[serde(rename = "bypassPresent")]
        bypass_present: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitLayout {
    Central,
    Decentral,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DuctInsulation {
    Uninsulated,
    Insulated,
    Unknown,
    Specified {
        #[serde(rename = "thicknessM")]
        thickness_m: f64,
        #[serde(rename = "conductivityWPerMK")]
        conductivity_w_per_mk: f64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatRecovery {
    pub efficiency: HeatRecoveryEfficiency,
    pub bypass: Bypass,
    pub layout: UnitLayout,
    #[serde(default)]
    pub constant_volume_control: bool,
    /// L_bu inside the envelope; `None` takes the 11.109 default.
    #[serde(default)]
    pub supply_duct_length_m: Option<f64>,
    pub supply_duct_insulation: DuctInsulation,
    #[serde(default)]
    pub manufacture_year: Option<i32>,
    pub equipment_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemUnit {
    pub variant: SystemVariant,
    #[serde(default)]
    pub heat_recovery: Option<HeatRecovery>,
    pub ducts: DuctAirtightness,
    #[serde(default)]
    pub air_handling_unit: Option<AirHandlingUnit>,
    pub equipment_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VentilationSystem {
    Single {
        unit: SystemUnit,
    },
    /// E.1: decentral balanced units with heat recovery and CO2 control
    /// (D.5b) in part of the zone (11.29–11.45, 11.50–11.54).
    Combined {
        #[serde(rename = "decentralAreaM2")]
        decentral_area_m2: f64,
        #[serde(rename = "totalResidenceAreaM2")]
        total_residence_area_m2: f64,
        decentral: SystemUnit,
        other: SystemUnit,
    },
}

/// Table 11.14 building types and execution variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AirtightnessType {
    PitchedRoofTerraced,
    PitchedRoofEndOrCorner,
    PitchedRoofDetached,
    PitchedRoofDetachedPartlyFlat,
    FlatRoofTerraced,
    FlatRoofEndOrCorner,
    FlatRoofDetached,
    StoreyMiddleLowerOrIntermediate,
    StoreyEndLowerOrIntermediate,
    StoreyMiddleTop,
    StoreyEndTop,
    MultiStoreyWholeBuilding,
    MultiStoreyWholeTopLayer,
    MultiStoreyWholeIntermediateLayer,
    MultiStoreyWholeBottomLayer,
}

impl AirtightnessType {
    /// (q_v10;spec;calc, f_type).
    fn values(self) -> (f64, f64) {
        use AirtightnessType::*;
        match self {
            PitchedRoofTerraced => (1.0, 1.0),
            PitchedRoofEndOrCorner => (1.0, 1.2),
            PitchedRoofDetached => (1.0, 1.4),
            PitchedRoofDetachedPartlyFlat => (1.0, 1.2),
            FlatRoofTerraced => (0.7, 1.0),
            FlatRoofEndOrCorner => (0.7, 1.2),
            FlatRoofDetached => (0.7, 1.4),
            StoreyMiddleLowerOrIntermediate => (0.5, 1.0),
            StoreyEndLowerOrIntermediate => (0.5, 1.3),
            StoreyMiddleTop => (0.5, 1.2),
            StoreyEndTop => (0.5, 1.4),
            MultiStoreyWholeBuilding => (0.5, 1.2),
            MultiStoreyWholeTopLayer => (0.5, 1.3),
            MultiStoreyWholeIntermediateLayer => (0.5, 1.2),
            MultiStoreyWholeBottomLayer => (0.5, 1.1),
        }
    }
}

/// Table 11.8 q_usi;spec, dm³/(s·m²).
pub fn specific_capacity(function: VentilationFunction) -> f64 {
    function.specific_capacity()
}

/// Table 11.13 f_y.
pub fn year_factor(year: i32) -> f64 {
    match year {
        y if y < 1970 => 3.0,
        y if y < 1980 => 2.5,
        y if y < 1990 => 2.0,
        y if y < 2000 => 1.5,
        y if y < 2010 => 1.0,
        _ => 0.7,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Infiltration {
    /// NEN 2686 measurement or quality-assurance value, dm³/(s·m²) at 10 Pa.
    Measured {
        #[serde(rename = "qv10DmPerSM2")]
        qv10_dm3_per_s_m2: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 11.86 with tables 11.13/11.14.
    Reference {
        #[serde(rename = "buildingType")]
        building_type: AirtightnessType,
        /// Year of (near-)complete renovation, otherwise the construction year.
        #[serde(default, rename = "renovationYear")]
        renovation_year: Option<i32>,
    },
}

/// Table 11.12 appliances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CombustionApplianceKind {
    GasGeyser,
    GasOpenAtmospheric,
    GasBathGeyser,
    GasBoilerStorage,
    GasCentralHeatingBoiler,
    GasFireNaturalFlueTypeI,
    GasFireMechanicalFlueTypeII,
    GasFireTypeII,
    OpenFireplaceSolidFuel,
    GasStove,
    OilStove,
    CoalStove,
    SolidFuelStove,
}

impl CombustionApplianceKind {
    /// (P_h;fi;min kW, f_τ;verbr heating, f_τ;verbr cooling, f_ff dm³/(s·kW), flueless).
    fn values(self) -> (f64, f64, f64, f64, bool) {
        use CombustionApplianceKind::*;
        match self {
            GasGeyser => (13.0, 0.05, 0.05, 1.62, true),
            GasOpenAtmospheric => (10.0, 0.20, 0.05, 1.62, true),
            GasBathGeyser => (35.0, 0.05, 0.05, 0.78, false),
            GasBoilerStorage => (15.0, 0.05, 0.05, 0.78, false),
            GasCentralHeatingBoiler => (30.0, 0.20, 0.00, 0.78, false),
            GasFireNaturalFlueTypeI => (10.0, 0.20, 0.05, 0.78, false),
            GasFireMechanicalFlueTypeII => (10.0, 0.20, 0.05, 1.34, false),
            GasFireTypeII => (15.0, 0.20, 0.05, 3.35, false),
            OpenFireplaceSolidFuel => (25.0, 0.20, 0.05, 2.80, false),
            GasStove => (10.0, 0.20, 0.00, 0.78, false),
            OilStove => (10.0, 0.20, 0.00, 0.32, false),
            CoalStove => (15.0, 0.20, 0.00, 0.52, false),
            SolidFuelStove => (25.0, 0.20, 0.00, 2.80, false),
        }
    }
}

/// Table 11.11 appliance classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplianceClass {
    KitchenStove,
    GasTypeA,
    OpenFireplace,
    GasTypeB,
    SpecificGasAppliance,
    RoomSealed,
}

impl ApplianceClass {
    fn factor(self) -> Option<f64> {
        match self {
            Self::GasTypeA | Self::GasTypeB => Some(1.0),
            Self::KitchenStove | Self::OpenFireplace | Self::RoomSealed => Some(0.0),
            Self::SpecificGasAppliance => None,
        }
    }
}

/// Only appliances that generate space heat (chapter 9) or hot water
/// (chapter 13); decorative appliances are furnishing (11.2.4).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombustionAppliance {
    pub id: String,
    pub kind: CombustionApplianceKind,
    pub class: ApplianceClass,
    /// `None` uses P_h;fi;min.
    #[serde(default)]
    pub nominal_input_kw: Option<f64>,
    pub source_reference: String,
}

/// ISSO 82.2/75.2 practice factors; `None` takes the standard values.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VentilationPractice {
    /// f_prac;vent;sys: system A 0,25, C 0,5, D 0,75 (B is not listed and
    /// takes the C value).
    #[serde(default)]
    pub system: Option<f64>,
    /// f_prac;argl on purge ventilation, standard 0,5.
    #[serde(default)]
    pub purge: Option<f64>,
    /// f_prac;lea on infiltration, standard 0,5.
    #[serde(default)]
    pub leakage: Option<f64>,
}

impl VentilationPractice {
    fn system_factor(&self, op: VentSysOp) -> f64 {
        self.system.unwrap_or(match op {
            VentSysOp::Natural => 0.25,
            VentSysOp::Supply | VentSysOp::Extract => 0.5,
            VentSysOp::Balanced => 0.75,
        })
    }
}

/// 11.23/11.23a: exhaust-air heat pump overventilation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Overventilation {
    /// f_H;τ;hp-on;mi from Q.5.3. Left empty, the space-heating chain of an
    /// annex Q heat pump derives it with the Q.5.3 recalculation.
    #[serde(default)]
    pub heating_time_fraction: Vec<f64>,
    /// f_W;τ;hp-on;mi from 13.8.2.4 (13.149). Left empty, the building
    /// performance fills it from the hot-water system.
    #[serde(default)]
    pub hot_water_time_fraction: Vec<f64>,
    /// q_V;hp;H from the supplier, m³/h.
    #[serde(default)]
    pub heating_flow_m3_per_h: Option<f64>,
    /// A_g;zi/Σ A_g of the zones served by the heat pump (11.23a).
    #[serde(default = "one")]
    pub heating_area_share: f64,
    /// q_V;hp;W;mi from 13.8.2.4 (13.148/13.148a), m³/h. Left empty, the
    /// building performance fills it from the hot-water system.
    #[serde(default)]
    pub hot_water_flow_m3_per_h: Vec<f64>,
    pub source_reference: String,
}

fn one() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum OpeningArea {
    Declared {
        #[serde(rename = "netAreaM2")]
        net_area_m2: f64,
    },
    /// 11.71a with C_d;k and C_e;k per NEN-EN 13030.
    Discharge {
        #[serde(rename = "grossAreaM2")]
        gross_area_m2: f64,
        #[serde(rename = "dischargeCoefficient")]
        discharge_coefficient: f64,
        #[serde(rename = "entryLossCoefficient")]
        entry_loss_coefficient: f64,
    },
    /// 11.71b with R_w;arg from the maximum opening angle (note 6).
    OpeningAngle {
        #[serde(rename = "maxNetAreaM2")]
        max_net_area_m2: f64,
        #[serde(rename = "maxAngleDeg")]
        max_angle_deg: f64,
    },
}

impl OpeningArea {
    fn net_area(&self) -> f64 {
        match self {
            Self::Declared { net_area_m2 } => *net_area_m2,
            Self::Discharge {
                gross_area_m2,
                discharge_coefficient,
                entry_loss_coefficient,
            } => gross_area_m2 * ((discharge_coefficient + entry_loss_coefficient) / 2.0) / 0.67,
            Self::OpeningAngle {
                max_net_area_m2,
                max_angle_deg,
            } => (1.46 * max_angle_deg / (max_angle_deg + 41.0)).min(1.0) * max_net_area_m2,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoolingOpening {
    pub id: String,
    pub area: OpeningArea,
    /// h_w;path: height of the opening centre above the lowest ground level.
    pub centre_height_m: f64,
    /// h_w;fa: height of the net opening.
    pub opening_height_m: f64,
    /// 0 = north, clockwise (as in the rest of the kernel).
    pub azimuth_deg: f64,
    /// 0 = horizontal, 90 = vertical.
    pub tilt_deg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoolingOperation {
    Manual,
    Automatic,
    AutomaticWithTemperature,
}

impl CoolingOperation {
    fn factor(self) -> f64 {
        match self {
            Self::Manual => 0.35,
            Self::Automatic => 0.50,
            Self::AutomaticWithTemperature => 1.0,
        }
    }
}

/// 11.2.3.3: only when all conditions (burglary, insects, rain, operable
/// below 1.8 m) are met; the evidence names the documents.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VentilativeCooling {
    pub openings: Vec<CoolingOpening>,
    pub operation: CoolingOperation,
    pub conditions_evidence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum GrillePreheatingControl {
    /// 11.123.
    Specified {
        #[serde(rename = "maxPowerWPerDm3PerS")]
        max_power_w_per_dm3_per_s: f64,
        #[serde(rename = "maxTemperatureRiseK")]
        max_temperature_rise_k: f64,
        #[serde(rename = "switchOnBelowC")]
        switch_on_below_c: f64,
        #[serde(rename = "maxSupplyTemperatureC")]
        max_supply_temperature_c: f64,
    },
    /// 11.124: insufficient information.
    Fallback,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrillePreheating {
    pub control: GrillePreheatingControl,
    /// q_V;sys;nat;elvv, m³/h; `None` gives f_pre;nat;elvv = 1.
    #[serde(default)]
    pub preheated_design_flow_m3_per_h: Option<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FanCurrent {
    Ac,
    Dc,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum FanPower {
    /// P_nom;el from the type plate or design, W.
    Nominal {
        #[serde(rename = "nominalPowerW")]
        nominal_power_w: f64,
    },
    /// 11.135 with table 11.20 or 11.136.
    Motor {
        #[serde(rename = "motorPowerW")]
        motor_power_w: f64,
        #[serde(default, rename = "manufactureYear")]
        manufacture_year: Option<i32>,
        /// U·I·e measured; `None` uses table 11.20.
        #[serde(default, rename = "electricalInputW")]
        electrical_input_w: Option<f64>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowControlMethod {
    Throttle,
    InletVaneOrBladePitch,
    SpeedControl,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum FanControl {
    /// Table 11.21 (dwellings).
    ResidentialTable,
    /// Quality statement: monthly f_regfan.
    Declared {
        monthly: Vec<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Table 11.22 × f_τ (utility and collective systems).
    FlowControl { control: FlowControlMethod },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Fan {
    pub id: String,
    pub power: FanPower,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Fans {
    /// 11.139–11.142 with table 11.23.
    Forfait {
        current: FanCurrent,
        #[serde(default, rename = "manufactureYear")]
        manufacture_year: Option<i32>,
    },
    /// 11.133 with the zone's own fans.
    Declared {
        fans: Vec<Fan>,
        control: FanControl,
        /// f_gebouw;si;V.
        #[serde(default = "one", rename = "buildingShare")]
        building_share: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlowReduction {
    /// Utility buildings and residential buildings with a collective system.
    #[serde(default)]
    pub collective: bool,
    /// x of 11.60 (multiple of 10); `None` without recirculation.
    #[serde(default)]
    pub recirculation_percent: Option<u32>,
    /// x of 11.61 (multiple of 10); `None` without flow control.
    #[serde(default)]
    pub flow_control_percent: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstalledCapacity {
    /// q_V;inst (11.62), dm³/s.
    pub total_dm3_per_s: f64,
    /// q_V;inst;1a + q_V;inst;1b, dm³/s (for 11.128).
    #[serde(default)]
    pub natural_supply_dm3_per_s: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VentilationInput {
    pub zone_id: String,
    pub usable_floor_area_m2: f64,
    pub category: Category,
    pub functions: Vec<FunctionArea>,
    /// N_woon;zi (6.6.6); 0 for utility zones.
    #[serde(default)]
    pub dwelling_count: u32,
    /// A_g of the whole dwelling when a dwelling is split over zones (11.65).
    #[serde(default)]
    pub whole_dwelling_area_m2: Option<f64>,
    /// Apartment building (L_bu default 0,5·H in 11.109).
    #[serde(default)]
    pub apartment_building: bool,
    /// H: from the lowest ground level to the highest point, m.
    pub building_height_m: f64,
    pub construction_year: i32,
    /// Ground floor over a crawlspace (table 11.1, built before 1992).
    #[serde(default)]
    pub floor_above_crawlspace: bool,
    /// θ_int;set;H;stc and θ_int;set;C;stc (7.9.4.1).
    pub heating_setpoint_c: f64,
    pub cooling_setpoint_c: f64,
    pub system: VentilationSystem,
    /// Evidence of automatic control on measured indoor and outdoor
    /// temperature (11.2.2.3.2); `None` means τ_sysC = 0.
    #[serde(default)]
    pub maximum_capacity_for_cooling: Option<String>,
    #[serde(default)]
    pub installed_capacity: Option<InstalledCapacity>,
    #[serde(default)]
    pub flow_reduction: FlowReduction,
    pub infiltration: Infiltration,
    #[serde(default)]
    pub combustion_appliances: Vec<CombustionAppliance>,
    #[serde(default)]
    pub overventilation: Option<Overventilation>,
    /// Maatwerkadvies only (ISSO 82.2 table 2.7, 75.2 table 2.8): practice
    /// factors on the actual system flow, purge and infiltration. Never set
    /// for the energy label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub practice: Option<VentilationPractice>,
    #[serde(default)]
    pub ventilative_cooling: Option<VentilativeCooling>,
    #[serde(default)]
    pub grille_preheating: Option<GrillePreheating>,
    pub fans: Fans,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VentilationIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceFlows {
    /// q_V;ODA;req (11.22/11.23), m³/h.
    pub required_outdoor_air_m3_per_h: f64,
    /// q_V;ODA;eff (11.2.2.2), m³/h.
    pub effective_outdoor_air_m3_per_h: f64,
    pub infiltration_m3_per_h: f64,
    pub natural_supply_m3_per_h: f64,
    pub purge_m3_per_h: f64,
    pub ventilative_cooling_m3_per_h: f64,
    pub combustion_m3_per_h: f64,
    pub mechanical_supply_m3_per_h: f64,
    pub mechanical_extract_m3_per_h: f64,
    /// Temperature of the natural supply (11.99), °C.
    pub natural_supply_temperature_c: f64,
    /// Flow-weighted mechanical supply temperature (11.104), °C.
    pub mechanical_supply_temperature_c: f64,
    /// Temperature of ventilative cooling air (table 11.4), °C.
    pub ventilative_cooling_temperature_c: f64,
    /// H_ve (7.19) with b_v folded into the supply temperatures, W/K.
    pub conductance_w_per_k: f64,
    /// H_ve·(θ_set − θ_sup) summed over flows, W.
    pub heat_flow_per_setpoint_w: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VentilationMonthResult {
    pub month: u8,
    pub heating: BalanceFlows,
    pub cooling: BalanceFlows,
    /// Internal reference pressure per airflow zone, heating balance, Pa.
    pub heating_reference_pressure_pa: Vec<f64>,
    pub cooling_reference_pressure_pa: Vec<f64>,
    pub fan_electricity_kwh: f64,
    pub frost_protection_electricity_kwh: f64,
    pub grille_preheating_electricity_kwh: f64,
    /// 9.29 Q_H;ϑHstook;in;air of the heating balance, kWh.
    pub heating_limit_air_kwh: f64,
    /// 10.20 Q_C;ϑkoelgrens;in;air of the cooling balance, kWh: the extra
    /// ventilation transfer for the cooling limit (10.19).
    pub cooling_limit_air_kwh: f64,
    /// Q_H;AHU;in;req (11.120) of the heating balance, kWh, for the
    /// space-heating node (9.4).
    pub ahu_heating_kwh: f64,
    /// Q_C;ahu;in;req (11.116) of the cooling balance, kWh, for the cooling
    /// generator (10.5).
    pub ahu_cooling_kwh: f64,
    /// f_buitenlucht (11.24) for annex Q, when overventilation applies.
    pub outdoor_air_fraction: Option<f64>,
}

/// One chapter 7 flow `k` with its conductance and supply temperature per
/// balance.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandFlowMonth {
    pub month: u8,
    pub heating_conductance_w_per_k: f64,
    pub heating_supply_temperature_c: f64,
    pub cooling_conductance_w_per_k: f64,
    pub cooling_supply_temperature_c: f64,
    /// Heating supply temperature for the heating limit (9.29), °C.
    pub heating_limit_supply_temperature_c: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandFlow {
    pub id: String,
    pub months: Vec<DemandFlowMonth>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VentilationResult {
    pub zone_id: String,
    pub ventilation_system_op: Vec<VentSysOp>,
    pub infiltration_q_v1_m3_per_h: f64,
    pub months: Vec<VentilationMonthResult>,
    pub demand_flows: Vec<DemandFlow>,
    pub annual_fan_electricity_kwh: f64,
    pub annual_frost_protection_electricity_kwh: f64,
    pub annual_grille_preheating_electricity_kwh: f64,
    pub interpretations: Vec<&'static str>,
}

// ---------------------------------------------------------------- validation

fn issue(code: &'static str, path: impl Into<String>) -> VentilationIssue {
    VentilationIssue {
        code,
        path: path.into(),
    }
}

fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn units(system: &VentilationSystem) -> Vec<(&'static str, &SystemUnit)> {
    match system {
        VentilationSystem::Single { unit } => vec![("system.unit", unit)],
        VentilationSystem::Combined {
            decentral, other, ..
        } => vec![("system.decentral", decentral), ("system.other", other)],
    }
}

pub fn validate_ventilation(input: &VentilationInput) -> Vec<VentilationIssue> {
    let mut issues = Vec::new();
    if input.zone_id.trim().is_empty() {
        issues.push(issue("zone_id_required", "zoneId"));
    }
    if input.source_reference.trim().is_empty() {
        issues.push(issue("source_reference_required", "sourceReference"));
    }
    if !finite_positive(input.usable_floor_area_m2) {
        issues.push(issue("usable_floor_area_invalid", "usableFloorAreaM2"));
    }
    if !finite_positive(input.building_height_m) {
        issues.push(issue("building_height_invalid", "buildingHeightM"));
    }
    if !(1800..=2100).contains(&input.construction_year) {
        issues.push(issue("construction_year_invalid", "constructionYear"));
    }
    for (field, value) in [
        ("heatingSetpointC", input.heating_setpoint_c),
        ("coolingSetpointC", input.cooling_setpoint_c),
    ] {
        if !value.is_finite() || !(5.0..=35.0).contains(&value) {
            issues.push(issue("setpoint_invalid", field));
        }
    }
    if input.functions.is_empty() {
        issues.push(issue("function_area_required", "functions"));
    }
    let mut function_area = 0.0;
    for (index, item) in input.functions.iter().enumerate() {
        if !finite_positive(item.area_m2) {
            issues.push(issue(
                "function_area_invalid",
                format!("functions[{index}].areaM2"),
            ));
        }
        function_area += item.area_m2;
        if item.swimming_pool && item.function != VentilationFunction::Sport {
            issues.push(issue(
                "swimming_pool_requires_sport_function",
                format!("functions[{index}].swimmingPool"),
            ));
        }
        let residential = item.function == VentilationFunction::Residential;
        if residential != (input.category == Category::Residential) {
            issues.push(issue(
                "function_category_mismatch",
                format!("functions[{index}].function"),
            ));
        }
    }
    if finite_positive(input.usable_floor_area_m2)
        && !input.functions.is_empty()
        && (function_area - input.usable_floor_area_m2).abs() > 0.01 * input.usable_floor_area_m2
    {
        issues.push(issue("function_area_sum_mismatch", "functions"));
    }
    if input.category == Category::Residential && input.dwelling_count == 0 {
        issues.push(issue("dwelling_count_required", "dwellingCount"));
    }
    if let Some(area) = input.whole_dwelling_area_m2 {
        if !finite_positive(area) || area + 1e-9 < input.usable_floor_area_m2 {
            issues.push(issue("whole_dwelling_area_invalid", "wholeDwellingAreaM2"));
        }
    }
    for (path, unit) in units(&input.system) {
        validate_unit(unit, path, input.category, &mut issues);
    }
    if let VentilationSystem::Combined {
        decentral_area_m2,
        total_residence_area_m2,
        decentral,
        other,
    } = &input.system
    {
        if decentral.variant != SystemVariant::D5b {
            issues.push(issue(
                "combined_decentral_variant_must_be_d5b",
                "system.decentral.variant",
            ));
        }
        if matches!(other.variant, SystemVariant::D5b) {
            issues.push(issue(
                "combined_other_variant_invalid",
                "system.other.variant",
            ));
        }
        if !finite_positive(*decentral_area_m2)
            || !finite_positive(*total_residence_area_m2)
            || decentral_area_m2 > total_residence_area_m2
        {
            issues.push(issue("combined_area_invalid", "system.decentralAreaM2"));
        }
    }
    if let Some(evidence) = &input.maximum_capacity_for_cooling {
        if evidence.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                "maximumCapacityForCooling",
            ));
        }
    }
    if let Some(capacity) = &input.installed_capacity {
        if !finite_positive(capacity.total_dm3_per_s)
            || !capacity.natural_supply_dm3_per_s.is_finite()
            || capacity.natural_supply_dm3_per_s < 0.0
        {
            issues.push(issue("installed_capacity_invalid", "installedCapacity"));
        }
        if capacity.source_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                "installedCapacity.sourceReference",
            ));
        }
    }
    for (field, value) in [
        (
            "flowReduction.recirculationPercent",
            input.flow_reduction.recirculation_percent,
        ),
        (
            "flowReduction.flowControlPercent",
            input.flow_reduction.flow_control_percent,
        ),
    ] {
        if let Some(value) = value {
            if value > 100 || value % 10 != 0 {
                issues.push(issue("flow_reduction_percent_invalid", field));
            }
            if !input.flow_reduction.collective && input.category == Category::Residential {
                issues.push(issue("flow_reduction_requires_collective_system", field));
            }
        }
    }
    match &input.infiltration {
        Infiltration::Measured {
            qv10_dm3_per_s_m2,
            source_reference,
        } => {
            if !qv10_dm3_per_s_m2.is_finite() || *qv10_dm3_per_s_m2 < 0.0 {
                issues.push(issue("infiltration_invalid", "infiltration.qv10DmPerSM2"));
            }
            if source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    "infiltration.sourceReference",
                ));
            }
        }
        Infiltration::Reference {
            renovation_year, ..
        } => {
            if let Some(year) = renovation_year {
                if *year < input.construction_year || *year > 2100 {
                    issues.push(issue(
                        "renovation_year_invalid",
                        "infiltration.renovationYear",
                    ));
                }
            }
        }
    }
    for (index, appliance) in input.combustion_appliances.iter().enumerate() {
        let path = format!("combustionAppliances[{index}]");
        let (_, _, _, _, flueless) = appliance.kind.values();
        let class_fits = match appliance.class {
            // Table 11.11: type A and kitchen stoves have no flue.
            ApplianceClass::GasTypeA | ApplianceClass::KitchenStove => flueless,
            ApplianceClass::GasTypeB
            | ApplianceClass::OpenFireplace
            | ApplianceClass::RoomSealed => !flueless,
            ApplianceClass::SpecificGasAppliance => true,
        };
        if !class_fits {
            issues.push(issue(
                "appliance_class_kind_mismatch",
                format!("{path}.class"),
            ));
        }
        if appliance.class == ApplianceClass::SpecificGasAppliance {
            issues.push(issue(
                "specific_gas_appliance_unsupported",
                format!("{path}.class"),
            ));
        }
        if let Some(power) = appliance.nominal_input_kw {
            if !finite_positive(power) {
                issues.push(issue(
                    "appliance_power_invalid",
                    format!("{path}.nominalInputKw"),
                ));
            }
        }
        if appliance.source_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                format!("{path}.sourceReference"),
            ));
        }
    }
    if let Some(over) = &input.overventilation {
        for (field, values) in [
            (
                "overventilation.heatingTimeFraction",
                &over.heating_time_fraction,
            ),
            (
                "overventilation.hotWaterTimeFraction",
                &over.hot_water_time_fraction,
            ),
        ] {
            if values.len() != 12 || values.iter().any(|v| !(0.0..=1.0).contains(v)) {
                issues.push(issue("overventilation_fraction_invalid", field));
            }
        }
        if over.heating_time_fraction.len() == 12
            && over.hot_water_time_fraction.len() == 12
            && over
                .heating_time_fraction
                .iter()
                .zip(&over.hot_water_time_fraction)
                .any(|(h, w)| h + w > 1.0 + 1e-9)
        {
            issues.push(issue(
                "overventilation_fraction_sum_exceeds_one",
                "overventilation",
            ));
        }
        if over.hot_water_flow_m3_per_h.len() != 12
            || over
                .hot_water_flow_m3_per_h
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0)
        {
            issues.push(issue(
                "overventilation_flow_invalid",
                "overventilation.hotWaterFlowM3PerH",
            ));
        }
        if let Some(flow) = over.heating_flow_m3_per_h {
            if !flow.is_finite() || flow < 0.0 {
                issues.push(issue(
                    "overventilation_flow_invalid",
                    "overventilation.heatingFlowM3PerH",
                ));
            }
        }
        if !(over.heating_area_share > 0.0 && over.heating_area_share <= 1.0) {
            issues.push(issue(
                "overventilation_area_share_invalid",
                "overventilation.heatingAreaShare",
            ));
        }
        if over.source_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                "overventilation.sourceReference",
            ));
        }
    }
    if let Some(cooling) = &input.ventilative_cooling {
        if cooling.openings.is_empty() {
            issues.push(issue(
                "ventilative_cooling_opening_required",
                "ventilativeCooling.openings",
            ));
        }
        if cooling.conditions_evidence.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                "ventilativeCooling.conditionsEvidence",
            ));
        }
        for (index, opening) in cooling.openings.iter().enumerate() {
            let path = format!("ventilativeCooling.openings[{index}]");
            let area_ok = match &opening.area {
                OpeningArea::Declared { net_area_m2 } => finite_positive(*net_area_m2),
                OpeningArea::Discharge {
                    gross_area_m2,
                    discharge_coefficient,
                    entry_loss_coefficient,
                } => {
                    finite_positive(*gross_area_m2)
                        && discharge_coefficient.is_finite()
                        && entry_loss_coefficient.is_finite()
                        && (discharge_coefficient + entry_loss_coefficient) > 0.0
                }
                OpeningArea::OpeningAngle {
                    max_net_area_m2,
                    max_angle_deg,
                } => finite_positive(*max_net_area_m2) && finite_positive(*max_angle_deg),
            };
            if !area_ok {
                issues.push(issue("opening_area_invalid", format!("{path}.area")));
            }
            if !opening.centre_height_m.is_finite()
                || opening.centre_height_m < 0.0
                || !finite_positive(opening.opening_height_m)
            {
                issues.push(issue(
                    "opening_height_invalid",
                    format!("{path}.centreHeightM"),
                ));
            }
            if !opening.azimuth_deg.is_finite() || !(0.0..=90.0).contains(&opening.tilt_deg) {
                issues.push(issue(
                    "opening_orientation_invalid",
                    format!("{path}.tiltDeg"),
                ));
            }
        }
    }
    if let Some(preheat) = &input.grille_preheating {
        if preheat.source_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                "grillePreheating.sourceReference",
            ));
        }
        if let GrillePreheatingControl::Specified {
            max_power_w_per_dm3_per_s,
            max_temperature_rise_k,
            switch_on_below_c,
            max_supply_temperature_c,
        } = preheat.control
        {
            if !finite_positive(max_power_w_per_dm3_per_s)
                || !finite_positive(max_temperature_rise_k)
                || !switch_on_below_c.is_finite()
                || !max_supply_temperature_c.is_finite()
            {
                issues.push(issue(
                    "grille_preheating_invalid",
                    "grillePreheating.control",
                ));
            }
        }
        if let Some(flow) = preheat.preheated_design_flow_m3_per_h {
            let natural = input
                .installed_capacity
                .as_ref()
                .map(|c| c.natural_supply_dm3_per_s)
                .unwrap_or(0.0);
            if !finite_positive(flow) || natural <= 0.0 {
                issues.push(issue(
                    "grille_preheating_flow_requires_installed_natural_capacity",
                    "grillePreheating.preheatedDesignFlowM3PerH",
                ));
            }
        }
        if !all_ops(&input.system)
            .iter()
            .any(|op| matches!(op, VentSysOp::Natural | VentSysOp::Extract))
        {
            issues.push(issue(
                "grille_preheating_requires_natural_supply",
                "grillePreheating",
            ));
        }
    }
    match &input.fans {
        Fans::Forfait {
            current,
            manufacture_year,
        } => {
            if *current == FanCurrent::Ac && manufacture_year.is_some_and(|y| y > 2006) {
                issues.push(issue("forfait_fan_ac_after_2006_unsupported", "fans"));
            }
        }
        Fans::Declared {
            fans,
            control,
            building_share,
            source_reference,
        } => {
            if fans.is_empty() {
                issues.push(issue("fan_required", "fans.fans"));
            }
            for (index, fan) in fans.iter().enumerate() {
                let ok = match &fan.power {
                    FanPower::Nominal { nominal_power_w } => finite_positive(*nominal_power_w),
                    FanPower::Motor {
                        motor_power_w,
                        electrical_input_w,
                        ..
                    } => {
                        finite_positive(*motor_power_w)
                            && electrical_input_w
                                .map_or(true, |w| finite_positive(w) && w >= *motor_power_w)
                    }
                };
                if !ok {
                    issues.push(issue(
                        "fan_power_invalid",
                        format!("fans.fans[{index}].power"),
                    ));
                }
            }
            if !(*building_share > 0.0 && *building_share <= 1.0) {
                issues.push(issue("fan_building_share_invalid", "fans.buildingShare"));
            }
            if source_reference.trim().is_empty() {
                issues.push(issue("source_reference_required", "fans.sourceReference"));
            }
            match control {
                FanControl::Declared {
                    monthly,
                    source_reference,
                } => {
                    if monthly.len() != 12 || monthly.iter().any(|v| !(0.0..=1.0).contains(v)) {
                        issues.push(issue("fan_control_invalid", "fans.control.monthly"));
                    }
                    if source_reference.trim().is_empty() {
                        issues.push(issue(
                            "source_reference_required",
                            "fans.control.sourceReference",
                        ));
                    }
                }
                FanControl::ResidentialTable => {
                    if input.category != Category::Residential {
                        issues.push(issue("fan_control_table_residential_only", "fans.control"));
                    }
                }
                FanControl::FlowControl { .. } => {}
            }
        }
    }
    issues
}

fn validate_unit(
    unit: &SystemUnit,
    path: &str,
    category: Category,
    issues: &mut Vec<VentilationIssue>,
) {
    if unit.equipment_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            format!("{path}.equipmentReference"),
        ));
    }
    if unit.variant.control_factor(category).is_none() {
        issues.push(issue(
            "variant_not_applicable_to_category",
            format!("{path}.variant"),
        ));
    }
    let op = unit.variant.op();
    if let Some(recovery) = &unit.heat_recovery {
        let rpath = format!("{path}.heatRecovery");
        if op != VentSysOp::Balanced {
            issues.push(issue(
                "heat_recovery_requires_balanced_ventilation",
                rpath.clone(),
            ));
        }
        match &recovery.efficiency {
            HeatRecoveryEfficiency::Declared {
                value,
                source_reference,
                ..
            } => {
                if !(0.0..=1.0).contains(value) {
                    issues.push(issue(
                        "heat_recovery_efficiency_invalid",
                        format!("{rpath}.efficiency.value"),
                    ));
                }
                if source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        format!("{rpath}.efficiency.sourceReference"),
                    ));
                }
            }
            HeatRecoveryEfficiency::Table { .. } => {}
        }
        match &recovery.bypass {
            Bypass::Partial { fraction } if !(0.0..=1.0).contains(fraction) => issues.push(issue(
                "bypass_fraction_invalid",
                format!("{rpath}.bypass.fraction"),
            )),
            Bypass::Full {
                cold_recovery_evidence: Some(evidence),
            } if evidence.trim().is_empty() => issues.push(issue(
                "source_reference_required",
                format!("{rpath}.bypass.coldRecoveryEvidence"),
            )),
            _ => {}
        }
        if let Some(length) = recovery.supply_duct_length_m {
            if !length.is_finite() || length < 0.0 {
                issues.push(issue(
                    "supply_duct_length_invalid",
                    format!("{rpath}.supplyDuctLengthM"),
                ));
            }
        }
        if let DuctInsulation::Specified {
            thickness_m,
            conductivity_w_per_mk,
        } = recovery.supply_duct_insulation
        {
            if !finite_positive(thickness_m) || !finite_positive(conductivity_w_per_mk) {
                issues.push(issue(
                    "duct_insulation_invalid",
                    format!("{rpath}.supplyDuctInsulation"),
                ));
            }
        }
        if recovery.equipment_reference.trim().is_empty() {
            issues.push(issue(
                "source_reference_required",
                format!("{rpath}.equipmentReference"),
            ));
        }
    }
    if unit.air_handling_unit.is_some() && !matches!(op, VentSysOp::Supply | VentSysOp::Balanced) {
        issues.push(issue(
            "ahu_requires_mechanical_supply",
            format!("{path}.airHandlingUnit"),
        ));
    }
    if op == VentSysOp::Natural && unit.ducts != DuctAirtightness::NoDucts {
        issues.push(issue(
            "natural_ventilation_ducts_must_be_none",
            format!("{path}.ducts"),
        ));
    }
}

fn all_ops(system: &VentilationSystem) -> Vec<VentSysOp> {
    units(system)
        .iter()
        .map(|(_, unit)| unit.variant.op())
        .collect()
}

// --------------------------------------------------------------- calculation

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Balance {
    Heating,
    Cooling,
}

fn density(temperature_c: f64) -> f64 {
    REFERENCE_TEMPERATURE_K / (temperature_c + 273.0) * AIR_DENSITY_REF
}

fn sign(value: f64) -> f64 {
    if value < 0.0 {
        -1.0
    } else {
        1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathKind {
    Leakage,
    Vent,
    Purge,
    Combustion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Windward,
    Leeward,
    Roof,
    Floor,
}

#[derive(Debug, Clone, Copy)]
struct Path {
    kind: PathKind,
    height_m: f64,
    coefficient: f64,
    exponent: f64,
    side: Side,
}

/// Table 11.3.
fn pressure_coefficient(side: Side, height_m: f64) -> f64 {
    let class = if height_m < 15.0 {
        0
    } else if height_m < 50.0 {
        1
    } else {
        2
    };
    match (side, class) {
        (Side::Windward, 0) => 0.25,
        (Side::Windward, 1) => 0.45,
        (Side::Windward, _) => 0.80,
        (Side::Leeward, 2) => -0.70,
        (Side::Leeward, _) => -0.50,
        (Side::Roof, 2) => -0.70,
        (Side::Roof, _) => -0.60,
        (Side::Floor, _) => -0.20,
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Coefficients {
    leakage: f64,
    vent_in: f64,
    vent_out: f64,
    purge_in: f64,
    purge_out: f64,
    combustion_in: f64,
}

struct AirflowZone {
    paths: Vec<Path>,
    /// Share of the mechanical and fixed flows (11.6–11.10).
    share: f64,
}

/// Table 11.1.
fn airflow_zones(height: f64, old_crawlspace: bool, c: Coefficients) -> Vec<AirflowZone> {
    let others = [
        (PathKind::Vent, c.vent_in, EXPONENT_VENT),
        (PathKind::Vent, c.vent_out, EXPONENT_VENT),
        (PathKind::Purge, c.purge_in, EXPONENT_PURGE),
        (PathKind::Purge, c.purge_out, EXPONENT_PURGE),
        (PathKind::Combustion, c.combustion_in, EXPONENT_COMBUSTION),
    ];
    let pair = |paths: &mut Vec<Path>, kind, coefficient: f64, exponent, h| {
        for side in [Side::Windward, Side::Leeward] {
            paths.push(Path {
                kind,
                height_m: h,
                coefficient,
                exponent,
                side,
            });
        }
    };
    if height < 15.0 {
        let mut paths = Vec::new();
        let (facade, roof, floor) = if old_crawlspace {
            (0.35, 0.15, Some(0.15))
        } else {
            (0.4, 0.2, None)
        };
        pair(
            &mut paths,
            PathKind::Leakage,
            facade * c.leakage,
            EXPONENT_LEAKAGE,
            0.5 * height,
        );
        paths.push(Path {
            kind: PathKind::Leakage,
            height_m: height,
            coefficient: roof * c.leakage,
            exponent: EXPONENT_LEAKAGE,
            side: Side::Roof,
        });
        if let Some(floor) = floor {
            paths.push(Path {
                kind: PathKind::Leakage,
                height_m: 0.0,
                coefficient: floor * c.leakage,
                exponent: EXPONENT_LEAKAGE,
                side: Side::Floor,
            });
        }
        for (kind, coefficient, exponent) in others {
            pair(&mut paths, kind, 0.5 * coefficient, exponent, 0.5 * height);
        }
        return vec![AirflowZone { paths, share: 1.0 }];
    }
    let layers: Vec<(f64, f64)> = if height <= 50.0 {
        vec![
            (7.5, 15.0 / height),
            (15.0 + (height - 15.0) / 2.0, (height - 15.0) / height),
        ]
    } else {
        vec![
            (7.5, 15.0 / height),
            (32.5, 35.0 / height),
            (50.0 + (height - 50.0) / 2.0, (height - 50.0) / height),
        ]
    };
    layers
        .into_iter()
        .map(|(h, fraction)| {
            let mut paths = Vec::new();
            pair(
                &mut paths,
                PathKind::Leakage,
                0.5 * fraction * c.leakage,
                EXPONENT_LEAKAGE,
                h,
            );
            for (kind, coefficient, exponent) in others {
                pair(&mut paths, kind, 0.5 * fraction * coefficient, exponent, h);
            }
            AirflowZone {
                paths,
                share: fraction,
            }
        })
        .collect()
}

struct PressureContext {
    outdoor_c: f64,
    indoor_c: f64,
    wind_m_per_s: f64,
}

impl PressureContext {
    /// 11.1.
    fn external(&self, path: &Path) -> f64 {
        let cp = pressure_coefficient(path.side, path.height_m);
        AIR_DENSITY_REF * REFERENCE_TEMPERATURE_K / (self.outdoor_c + 273.0)
            * (0.5 * cp * self.wind_m_per_s.powi(2) - path.height_m * GRAVITY)
    }

    /// 11.11/11.12.
    fn difference(&self, path: &Path, reference: f64) -> f64 {
        let internal = reference
            - AIR_DENSITY_REF * path.height_m * GRAVITY * REFERENCE_TEMPERATURE_K
                / (self.indoor_c + 273.0);
        self.external(path) - internal
    }

    /// 11.2/11.3/11.19, kg/h.
    fn mass_flow(&self, path: &Path, reference: f64) -> f64 {
        let dp = self.difference(path, reference);
        let volume = path.coefficient * dp.abs().powf(path.exponent);
        if dp > 0.0 {
            density(self.outdoor_c) * volume
        } else {
            -density(self.indoor_c) * volume
        }
    }
}

/// 11.2.1.6 steps 1–12; returns the reference pressure, or `None` when
/// the routine finds no pressure within the accuracy of 11.14.
fn solve_reference_pressure(
    zone: &AirflowZone,
    context: &PressureContext,
    fixed_mass_flow: f64,
    accuracy: f64,
) -> Option<f64> {
    let active: Vec<&Path> = zone.paths.iter().filter(|p| p.coefficient > 0.0).collect();
    if active.is_empty() {
        return Some(0.0);
    }
    let sum = |reference: f64| -> f64 {
        fixed_mass_flow
            + active
                .iter()
                .map(|path| context.mass_flow(path, reference))
                .sum::<f64>()
    };
    // Step 1 (11.15).
    let externals: Vec<f64> = active.iter().map(|p| context.external(p)).collect();
    let min = externals.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = externals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let leak_height = active
        .iter()
        .find(|p| p.kind == PathKind::Leakage && p.side == Side::Windward)
        .map(|p| p.height_m)
        .unwrap_or(0.0);
    let mut p_a = (min + max) / 2.0 + density(context.indoor_c) * leak_height * GRAVITY;
    // Step 2.
    let mut s_a = sum(p_a);
    if s_a.abs() <= accuracy {
        return Some(p_a);
    }
    // Steps 3–5.
    let mut p_b = p_a + 2.0;
    let mut s_b = sum(p_b);
    if s_b.abs() <= accuracy {
        return Some(p_b);
    }
    // Steps 6–7.
    let mut guard = 0;
    while sign(s_a) == sign(s_b) {
        let r = sign(p_b - p_a) * sign(s_b - s_a);
        if s_a.abs() > s_b.abs() {
            p_a = p_b;
            s_a = s_b;
        }
        p_b = p_a - 2.0 * sign(s_a) * r;
        s_b = sum(p_b);
        if s_b.abs() <= accuracy {
            return Some(p_b);
        }
        guard += 1;
        if guard > 100_000 || !s_b.is_finite() {
            return None;
        }
    }
    // Steps 8–12 (bisection).
    for _ in 0..200 {
        let p_c = (p_a + p_b) / 2.0;
        let s_c = sum(p_c);
        if s_c.abs() <= accuracy {
            return Some(p_c);
        }
        if sign(s_c) == sign(s_a) {
            p_a = p_c;
            s_a = s_c;
        } else {
            p_b = p_c;
        }
    }
    None
}

/// 11.14.
fn accuracy_kg_per_h(volume_m3_per_h: f64) -> f64 {
    if volume_m3_per_h <= 1000.0 {
        0.9
    } else {
        (volume_m3_per_h / 1000.0).floor() * 0.9
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct EffectiveFlows {
    leakage_in: f64,
    vent_in: f64,
    purge_in: f64,
    combustion_in: f64,
}

struct Part<'a> {
    unit: &'a SystemUnit,
    fraction: f64,
}

fn parts(system: &VentilationSystem) -> Vec<Part<'_>> {
    match system {
        VentilationSystem::Single { unit } => vec![Part {
            unit,
            fraction: 1.0,
        }],
        VentilationSystem::Combined {
            decentral_area_m2,
            total_residence_area_m2,
            decentral,
            other,
        } => {
            let f = decentral_area_m2 / total_residence_area_m2;
            vec![
                Part {
                    unit: decentral,
                    fraction: f,
                },
                Part {
                    unit: other,
                    fraction: 1.0 - f,
                },
            ]
        }
    }
}

/// Table 11.5 f_ctrl for the zone (11.51/11.52 for E.1).
fn control_factor_table(input: &VentilationInput) -> f64 {
    match &input.system {
        VentilationSystem::Single { unit } => {
            unit.variant.control_factor(input.category).unwrap_or(1.0)
        }
        VentilationSystem::Combined {
            decentral_area_m2,
            total_residence_area_m2,
            other,
            ..
        } => {
            let hru = match input.category {
                Category::Residential => 0.52,
                Category::Utility => 0.67,
            };
            let other_factor = other.variant.control_factor(input.category).unwrap_or(1.0);
            (decentral_area_m2 * hru + (total_residence_area_m2 - decentral_area_m2) * other_factor)
                / total_residence_area_m2
        }
    }
}

fn demand_controlled(system: &VentilationSystem) -> bool {
    match system {
        VentilationSystem::Single { unit } => unit.variant.demand_controlled(),
        VentilationSystem::Combined { .. } => true,
    }
}

/// Highest f_lea;du and f_lea;ahu over the units (11.3.1.1).
fn leakage_factors(input: &VentilationInput) -> (f64, f64) {
    units(&input.system)
        .iter()
        .fold((1.0_f64, 1.0_f64), |(du, ahu), (_, unit)| {
            let unit_du = match unit.variant.op() {
                VentSysOp::Natural => 1.0,
                _ => unit.ducts.factor(),
            };
            let unit_ahu = if unit.air_handling_unit.is_some() {
                1.02
            } else {
                1.0
            };
            (du.max(unit_du), ahu.max(unit_ahu))
        })
}

fn area_weighted(input: &VentilationInput, value: impl Fn(VentilationFunction) -> f64) -> f64 {
    let total: f64 = input.functions.iter().map(|f| f.area_m2).sum();
    input
        .functions
        .iter()
        .map(|f| value(f.function) * f.area_m2)
        .sum::<f64>()
        / total
}

/// Settings that the C1 variant (§5.4.3) overrides.
#[derive(Debug, Clone, Copy)]
struct Policy {
    lower_bound: bool,
    use_installed: bool,
}

struct ZoneConstants {
    occupancy_factor: f64,
    design_flow_regulatory_m3_per_h: f64,
    leakage_q1_m3_per_h: f64,
    flea_du: f64,
    flea_ahu: f64,
}

fn zone_constants(input: &VentilationInput, policy: Policy) -> ZoneConstants {
    let area = input.usable_floor_area_m2;
    // Table 11.8 f_τ for dwellings: the area of the whole dwelling when a
    // dwelling is split over zones, otherwise the mean dwelling area.
    let dwelling_area = match (input.whole_dwelling_area_m2, input.dwelling_count) {
        (Some(whole), _) => whole,
        (None, count) if count > 0 => area / f64::from(count),
        _ => area,
    };
    let occupancy_factor = area_weighted(input, |f| f.occupancy_factor(dwelling_area));
    let total_area: f64 = input.functions.iter().map(|f| f.area_m2).sum();
    let specific = input
        .functions
        .iter()
        .map(|part| {
            let pool = if part.swimming_pool { 2.0 } else { 1.0 };
            pool * part.function.specific_capacity() * part.area_m2
        })
        .sum::<f64>()
        / total_area;
    let mut capacity_dm3_per_s = specific * area;
    if policy.lower_bound && input.category == Category::Residential {
        // 11.63–11.65.
        let minimum = match input.whole_dwelling_area_m2 {
            Some(whole) => 35.0 * area / whole,
            None => 35.0 * f64::from(input.dwelling_count.max(1)),
        };
        capacity_dm3_per_s = capacity_dm3_per_s.max(minimum);
    }
    let (flea_du, flea_ahu) = leakage_factors(input);
    let design_flow_regulatory_m3_per_h =
        flea_du * flea_ahu * occupancy_factor * capacity_dm3_per_s * 3.6;
    let qv10 = match &input.infiltration {
        Infiltration::Measured {
            qv10_dm3_per_s_m2, ..
        } => *qv10_dm3_per_s_m2,
        Infiltration::Reference {
            building_type,
            renovation_year,
        } => {
            let (calc, f_type) = building_type.values();
            f_type * year_factor(renovation_year.unwrap_or(input.construction_year)) * calc
        }
    };
    // 11.85.
    let leakage_q1_m3_per_h = qv10 / 10f64.powf(EXPONENT_LEAKAGE) * area * 3.6;
    ZoneConstants {
        occupancy_factor,
        design_flow_regulatory_m3_per_h,
        leakage_q1_m3_per_h,
        flea_du,
        flea_ahu,
    }
}

/// 11.55–11.58 for one balance.
fn design_flow(
    input: &VentilationInput,
    constants: &ZoneConstants,
    balance: Balance,
    policy: Policy,
) -> f64 {
    let regulatory = constants.design_flow_regulatory_m3_per_h;
    let Some(installed) = input
        .installed_capacity
        .as_ref()
        .filter(|_| policy.use_installed)
    else {
        return regulatory;
    };
    let mut installed_dm3_per_s = installed.total_dm3_per_s;
    if balance == Balance::Heating
        && input.category == Category::Residential
        && demand_controlled(&input.system)
    {
        installed_dm3_per_s = 0.0;
    }
    // 11.59–11.61: only heating in utility or collective systems.
    let reducible = balance == Balance::Heating
        && (input.category == Category::Utility || input.flow_reduction.collective);
    let reduction = if reducible {
        let recirculation = input
            .flow_reduction
            .recirculation_percent
            .map_or(1.0, |x| 1.0 - f64::from(x) / 100.0);
        let flow = input
            .flow_reduction
            .flow_control_percent
            .map_or(1.0, |x| f64::from(x) / 100.0);
        recirculation * flow
    } else {
        1.0
    };
    let from_installed = constants.occupancy_factor * reduction * installed_dm3_per_s * 3.6;
    from_installed.max(regulatory)
}

fn heat_recovery_practice_factor(
    recovery: &HeatRecovery,
    input: &VentilationInput,
    variant: SystemVariant,
) -> f64 {
    let length = recovery
        .supply_duct_length_m
        .unwrap_or(match input.category {
            Category::Utility => 4.0,
            Category::Residential if input.apartment_building => 0.5 * input.building_height_m,
            Category::Residential => 4.0,
        });
    let insulation = match recovery.supply_duct_insulation {
        DuctInsulation::Specified {
            thickness_m,
            conductivity_w_per_mk,
        } => 0.01 / (0.25 + thickness_m / conductivity_w_per_mk) * length,
        DuctInsulation::Insulated => 0.02 * length,
        DuctInsulation::Uninsulated | DuctInsulation::Unknown => 0.04 * length,
    };
    let imbalance = if recovery.constant_volume_control {
        0.0
    } else if recovery.layout == UnitLayout::Decentral || variant == SystemVariant::D5b {
        0.02
    } else {
        0.05
    };
    let condensation = match &recovery.efficiency {
        HeatRecoveryEfficiency::Table { .. } => 0.0,
        HeatRecoveryEfficiency::Declared {
            standard: EfficiencyStandard::En13053,
            ..
        } => 0.0,
        HeatRecoveryEfficiency::Declared { value, .. } => {
            if *value >= 0.95 {
                0.10
            } else if *value >= 0.90 {
                0.075
            } else if *value >= 0.85 {
                0.05
            } else if *value >= 0.80 {
                0.025
            } else {
                0.10
            }
        }
    };
    let cap = match input.category {
        Category::Residential => 0.9,
        Category::Utility => 0.95,
    };
    (1.0 - insulation - imbalance - condensation).clamp(0.0, cap)
}

fn recovery_efficiency(recovery: &HeatRecovery) -> f64 {
    match &recovery.efficiency {
        HeatRecoveryEfficiency::Declared { value, .. } => *value,
        HeatRecoveryEfficiency::Table { exchanger } => exchanger.efficiency(),
    }
}

fn dissipation_included(recovery: &HeatRecovery) -> bool {
    matches!(
        recovery.efficiency,
        HeatRecoveryEfficiency::Table { .. }
            | HeatRecoveryEfficiency::Declared {
                standard: EfficiencyStandard::En13141_7 | EfficiencyStandard::En13141_8,
                ..
            }
    )
}

/// f_bypass with the 11.3.2.2 defaults.
fn bypass_fraction(recovery: &HeatRecovery, input: &VentilationInput) -> f64 {
    match &recovery.bypass {
        Bypass::None => 0.0,
        Bypass::Full { .. } => 1.0,
        Bypass::Partial { fraction } => *fraction,
        Bypass::Unknown { bypass_present } => match input.category {
            Category::Utility => 0.0,
            Category::Residential => {
                if input.construction_year >= 2010
                    || recovery.manufacture_year.is_some_and(|year| year >= 2010)
                {
                    1.0
                } else if *bypass_present {
                    0.7
                } else {
                    0.0
                }
            }
        },
    }
}

struct SupplyContext<'a> {
    input: &'a VentilationInput,
    month_index: usize,
    balance: Balance,
    outdoor_c: f64,
    indoor_c: f64,
}

/// 11.103/11.104 for a mechanical supply unit; returns (θ_SUP;dis;out,
/// ΔT_defrost applied, θ_SUP;dis;out − Δθ_hr − Δθ_rca − Δθ_fan of 9.29).
fn mechanical_supply_temperature(
    unit: &SystemUnit,
    context: &SupplyContext<'_>,
    oda_eff_m3_per_h: f64,
    flea_du: f64,
) -> SupplyTemperature {
    let input = context.input;
    let m = context.month_index;
    let op = unit.variant.op();
    let defrost = if op == VentSysOp::Balanced
        && unit.heat_recovery.is_some()
        && input.category == Category::Residential
    {
        DEFROST_RESIDENTIAL_K[m]
    } else {
        0.0
    };
    let oda_preh = context.outdoor_c + defrost;
    let (duct_outside, flea_ahu, fins_ahu) = match &unit.air_handling_unit {
        Some(ahu) => (
            ahu.supply_ducts_outside.delta_k(m),
            1.02,
            if ahu.inside_thermal_zone { 0.0 } else { 0.02 },
        ),
        None => (0.0, 1.0, 0.0),
    };
    // 11.129/11.130.
    let extract_out = context.indoor_c - duct_outside;
    let mut recovery_rise = 0.0;
    let mut fan_rise = match context.balance {
        Balance::Heating => 0.7,
        Balance::Cooling => match input.category {
            Category::Residential => 0.4,
            Category::Utility => 0.7,
        },
    };
    if let (VentSysOp::Balanced, Some(recovery)) = (op, &unit.heat_recovery) {
        let efficiency = recovery_efficiency(recovery);
        let practice = heat_recovery_practice_factor(recovery, input, unit.variant);
        let bypass = bypass_fraction(recovery, input);
        recovery_rise = match context.balance {
            Balance::Heating => {
                (practice * efficiency - (flea_ahu - 1.0) - fins_ahu) * (extract_out - oda_preh)
            }
            Balance::Cooling if bypass >= 1.0 => {
                let cold_recovery = matches!(
                    &recovery.bypass,
                    Bypass::Full {
                        cold_recovery_evidence: Some(_)
                    }
                );
                let wtwc = COLD_RECOVERY_SUPPLY_TEMPERATURE_C[m];
                if cold_recovery && context.indoor_c < wtwc {
                    (context.indoor_c - wtwc) * efficiency * TAU_COLD_RECOVERY[m]
                } else {
                    0.0
                }
            }
            Balance::Cooling => {
                let eta_bypass = 1.0 - TAU_BYPASS[m] * bypass;
                (practice * efficiency * eta_bypass - (flea_ahu - 1.0) - fins_ahu)
                    * (extract_out - oda_preh)
            }
        };
        if dissipation_included(recovery) {
            match context.balance {
                Balance::Heating => fan_rise = 0.0,
                Balance::Cooling if bypass == 0.0 => fan_rise = 0.0,
                Balance::Cooling => {}
            }
        }
    }
    // 11.110–11.113, recirculation in the heating balance (f_terugregel = 1
    // for cooling).
    let mut recirculation_rise = 0.0;
    if op == VentSysOp::Balanced && context.balance == Balance::Heating {
        if let Some(x) = input.flow_reduction.recirculation_percent {
            if input.flow_reduction.collective || input.category == Category::Utility {
                let f_recirculation = 1.0 - f64::from(x) / 100.0;
                let supply_in = oda_eff_m3_per_h / flea_ahu;
                let extract_out_flow = supply_in / flea_du;
                let q_rca = extract_out_flow * (1.0 - f_recirculation);
                let mixed = oda_eff_m3_per_h + q_rca;
                if mixed > 0.0 {
                    let sup_hr = oda_preh + recovery_rise;
                    recirculation_rise = q_rca / mixed * (extract_out - sup_hr);
                }
            }
        }
    }
    let dis_in = oda_preh + recovery_rise + recirculation_rise + fan_rise;
    let formula_out = dis_in - duct_outside;
    // 11.100/11.101 and the heating-only counterpart: the heating balance
    // has no cooling coil, the cooling balance no reheating (note 1).
    let mut coil_k = 0.0;
    if let Some(ahu) = &unit.air_handling_unit {
        let table = ahu_supply_temperature_c(input, m);
        match context.balance {
            Balance::Heating if ahu.heating_coil && table > formula_out => {
                coil_k = table - formula_out;
            }
            Balance::Cooling if ahu.cooling_coil && table < formula_out => {
                coil_k = table - formula_out;
            }
            _ => {}
        }
    }
    // 9.29: the supply temperature without heat recovery, recirculation and
    // fan heat, for the heating limit.
    SupplyTemperature {
        dis_out_c: formula_out + coil_k,
        defrost_k: defrost,
        heating_limit_c: oda_preh - duct_outside,
        // 10.20 as printed: θ_SUP;dis;out − Δθ_hr − Δθ_rca + Δθ_fan.
        cooling_limit_c: formula_out + coil_k - recovery_rise - recirculation_rise + fan_rise,
        before_coil_c: dis_in - fan_rise,
        coil_k,
        // 11.115/11.119: from ϑ_SUP;RCA or ϑ_SUP;hu (without the fan rise)
        // to ϑ_SUP;dis;in = table value + ΔT_du.
        coil_energy_k: if coil_k != 0.0 {
            coil_k + fan_rise
        } else {
            0.0
        },
    }
}

/// Mechanical supply temperatures of one unit and month (11.103/11.104).
struct SupplyTemperature {
    dis_out_c: f64,
    defrost_k: f64,
    heating_limit_c: f64,
    /// Supply temperature of 10.20 for the cooling limit, °C.
    cooling_limit_c: f64,
    /// ϑ_SUP;hu or ϑ_SUP;RCA: before the coil and the fan, °C.
    before_coil_c: f64,
    /// Change of ϑ_SUP;dis;out by the AHU coil, K (positive heating).
    coil_k: f64,
    /// ϑ_SUP;dis;in − ϑ_SUP;hu/RCA of 11.115/11.119, K (positive heating).
    coil_energy_k: f64,
}

/// Table 11.15, area-weighted between sport and the other functions.
fn ahu_supply_temperature_c(input: &VentilationInput, month_index: usize) -> f64 {
    area_weighted(input, |function| match function {
        VentilationFunction::Sport => AHU_SUPPLY_SPORT_C,
        _ => AHU_SUPPLY_OTHER_C[month_index],
    })
}

/// 11.123/11.124 for the heating balance.
fn grille_temperature_rise(input: &VentilationInput, outdoor_c: f64, balance: Balance) -> f64 {
    let Some(preheat) = &input.grille_preheating else {
        return 0.0;
    };
    if balance == Balance::Cooling {
        return 0.0;
    }
    let rise = match preheat.control {
        GrillePreheatingControl::Specified {
            max_power_w_per_dm3_per_s,
            max_temperature_rise_k,
            switch_on_below_c,
            max_supply_temperature_c,
        } => {
            if outdoor_c >= switch_on_below_c || outdoor_c >= max_supply_temperature_c {
                0.0
            } else {
                (max_power_w_per_dm3_per_s * 1000.0 / (1.205 * 1005.0)).min(max_temperature_rise_k)
            }
        }
        GrillePreheatingControl::Fallback => (input.heating_setpoint_c - 4.0) - outdoor_c,
    };
    rise.max(0.0)
}

/// 11.70–11.79; returns (q_in, q_out) per zone for the cooling balance.
fn ventilative_cooling_flows(
    input: &VentilationInput,
    month_index: usize,
    indoor_c: f64,
    outdoor_c: f64,
) -> (f64, f64) {
    let Some(cooling) = &input.ventilative_cooling else {
        return (0.0, 0.0);
    };
    let tau = TAU_VENTILATIVE_COOLING[month_index];
    let Some(argii_c) = VENTILATIVE_COOLING_TEMPERATURE_C[month_index] else {
        return (0.0, 0.0);
    };
    if tau == 0.0 {
        return (0.0, 0.0);
    }
    let areas: Vec<f64> = cooling.openings.iter().map(|o| o.area.net_area()).collect();
    let total: f64 = areas.iter().sum();
    let top = cooling
        .openings
        .iter()
        .map(|o| o.centre_height_m + o.opening_height_m / 2.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let bottom = cooling
        .openings
        .iter()
        .map(|o| o.centre_height_m - o.opening_height_m / 2.0)
        .fold(f64::INFINITY, f64::min);
    let free_height = top - bottom;
    let wind = WIND_SPEED_M_PER_S[month_index];
    let factor = tau * cooling.operation.factor();
    let rho_z = density(indoor_c);
    let rho_e = density(argii_c);
    let tz = indoor_c + 273.0;
    let stack_argii = (0.0035 * free_height * (tz - (argii_c + 273.0)).abs()).sqrt();
    let stack_avg = (0.0035 * free_height * (tz - (outdoor_c + 273.0)).abs()).sqrt();
    if cross_ventilation(&cooling.openings) {
        let cross = cross_area(&cooling.openings, &areas, total);
        let delta_cp = pressure_coefficient(Side::Windward, input.building_height_m)
            - pressure_coefficient(Side::Leeward, input.building_height_m);
        let wind_term = 0.67 * cross * wind.min(3.0) * delta_cp.sqrt();
        let q_in =
            3600.0 * AIR_DENSITY_REF / rho_e * wind_term.max(total / 2.0 * stack_argii) * factor;
        let q_out =
            -3600.0 * AIR_DENSITY_REF / rho_z * wind_term.max(total / 2.0 * stack_avg) * factor;
        (q_in, q_out)
    } else {
        let driver =
            (0.001 * wind * wind).max(0.0035 * free_height * (tz - (argii_c + 273.0)).abs());
        let q_in = 3600.0 * AIR_DENSITY_REF / rho_e * total / 2.0 * driver.sqrt() * factor;
        let q_out = -3600.0 * AIR_DENSITY_REF / rho_z * total / 2.0 * driver.sqrt() * factor;
        (q_in, q_out)
    }
}

fn angular_difference(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// 11.2.3.3.1: two façades at least 90° apart, or a façade and a roof
/// (tilt below 60°).
fn cross_ventilation(openings: &[CoolingOpening]) -> bool {
    let facades: Vec<&CoolingOpening> = openings.iter().filter(|o| o.tilt_deg >= 60.0).collect();
    let roof = openings.iter().any(|o| o.tilt_deg < 60.0);
    if roof && !facades.is_empty() {
        return true;
    }
    facades.iter().enumerate().any(|(i, a)| {
        facades[i + 1..]
            .iter()
            .any(|b| angular_difference(a.azimuth_deg, b.azimuth_deg) >= 90.0)
    })
}

/// 11.77–11.79.
fn cross_area(openings: &[CoolingOpening], areas: &[f64], total: f64) -> f64 {
    let mut result = f64::INFINITY;
    for i in 0..2 {
        let mut sum = 0.0;
        for j in 0..4 {
            let reference = f64::from(i) * 45.0 + f64::from(j) * 90.0;
            let oriented: f64 = openings
                .iter()
                .zip(areas)
                .filter(|(o, _)| {
                    o.tilt_deg < 60.0 || angular_difference(o.azimuth_deg, reference) <= 45.0
                })
                .map(|(_, a)| a)
                .sum();
            if oriented > 0.0 {
                let rest = total - oriented;
                if rest > 0.0 {
                    sum += 1.0 / (1.0 / oriented.powi(2) + 1.0 / rest.powi(2)).sqrt();
                }
            }
        }
        result = result.min(sum / 4.0);
    }
    result
}

/// 11.83 per appliance; returns (in, out) at 1 Pa for the balance.
fn combustion_flows(input: &VentilationInput, balance: Balance) -> (f64, f64) {
    input
        .combustion_appliances
        .iter()
        .fold((0.0, 0.0), |(q_in, q_out), appliance| {
            let (p_min, tau_h, tau_c, f_ff, flueless) = appliance.kind.values();
            let tau = match balance {
                Balance::Heating => tau_h,
                Balance::Cooling => tau_c,
            };
            let power = appliance.nominal_input_kw.unwrap_or(p_min).max(p_min);
            let f_as = appliance.class.factor().unwrap_or(0.0);
            let q = tau * f_as * f_ff * power * 3.6;
            if flueless {
                (q_in + q, q_out)
            } else {
                (q_in, q_out + q)
            }
        })
}

struct MonthBalance {
    flows: BalanceFlows,
    pressures: Vec<f64>,
    /// (flow id, q, θ_sup) per chapter 7 flow.
    demand_flows: Vec<(String, f64, f64)>,
    /// θ_sup per flow for the heating limit (9.29).
    limit_temperatures: Vec<f64>,
    frost_protection_kwh: f64,
    grille_preheating_kwh: f64,
    heating_limit_air_kwh: f64,
    cooling_limit_air_kwh: f64,
    ahu_heating_kwh: f64,
    ahu_cooling_kwh: f64,
    outdoor_air_fraction: Option<f64>,
    /// False when 11.2.1.6 found no reference pressure in some airflow zone.
    converged: bool,
}

fn balance_month(
    input: &VentilationInput,
    constants: &ZoneConstants,
    month_index: usize,
    balance: Balance,
    policy: Policy,
) -> MonthBalance {
    let m = month_index;
    let hours = MONTH_HOURS[m];
    let outdoor = OUTDOOR_TEMPERATURE_C[m];
    let indoor = match balance {
        Balance::Heating => input.heating_setpoint_c,
        Balance::Cooling => input.cooling_setpoint_c,
    };
    // 11.46–11.49 (f_sys = 1,00 in tables 11.5 and 11.6).
    let f_ctrl_table = control_factor_table(input);
    let tau_sys = if balance == Balance::Cooling && input.maximum_capacity_for_cooling.is_some() {
        TAU_SYS_C[m]
    } else {
        0.0
    };
    let f_ctrl = if 1.0 >= f_ctrl_table {
        (1.0 - tau_sys) * f_ctrl_table + tau_sys
    } else {
        f_ctrl_table
    };
    let design = design_flow(input, constants, balance, policy);
    let mut required = f_ctrl / PRACTICE_FACTOR_REQUIRED * design;
    let mut outdoor_air_fraction = None;
    if let Some(over) = &input.overventilation {
        // 11.23/11.23a/11.24.
        let f_h = over.heating_time_fraction[m];
        let f_w = over.hot_water_time_fraction[m];
        let q_h = over
            .heating_flow_m3_per_h
            .map_or(required, |q| (q * over.heating_area_share).max(required));
        let q_w = over.hot_water_flow_m3_per_h[m];
        let base = required;
        required = (1.0 - f_h - f_w) * base + f_h * q_h + f_w * q_w;
        if over.heating_flow_m3_per_h.is_some() && q_h > 0.0 {
            outdoor_air_fraction = Some(1.0 - base / q_h);
        }
    }

    // 11.2.2.2: per part.
    let (flea_du, flea_ahu) = (constants.flea_du, constants.flea_ahu);
    let mut coefficients = Coefficients::default();
    let mut supply_parts: Vec<(usize, f64, &SystemUnit)> = Vec::new();
    let mut extract = 0.0;
    let mut balanced_oda = 0.0;
    let mut natural_parts = false;
    // Maatwerkadvies practice factors apply to the actual system only.
    let practice = input.practice.as_ref().filter(|_| policy.use_installed);
    for (index, part) in parts(&input.system).into_iter().enumerate() {
        let factor = practice.map_or(1.0, |item| item.system_factor(part.unit.variant.op()));
        let q = required * part.fraction * factor;
        match part.unit.variant.op() {
            VentSysOp::Natural => {
                coefficients.vent_in += q;
                coefficients.vent_out += q;
                natural_parts = true;
            }
            VentSysOp::Supply => {
                supply_parts.push((index, q, part.unit));
                coefficients.vent_out += q;
                balanced_oda += q;
            }
            VentSysOp::Extract => {
                extract -= q;
                coefficients.vent_in += q;
                natural_parts = true;
            }
            VentSysOp::Balanced => {
                let supply_out = q / (flea_du * flea_ahu);
                let supply_in = supply_out * flea_du;
                supply_parts.push((index, supply_out, part.unit));
                extract -= supply_in;
                balanced_oda += q;
            }
        }
    }

    // 11.68/11.69, table 11.10.
    let tau_purge = match (input.category, balance) {
        (Category::Residential, Balance::Cooling) => TAU_PURGE_COOLING[m],
        _ => TAU_PURGE_DEFAULT,
    };
    let purge = tau_purge
        * area_weighted(input, |f| f.purge_capacity())
        * input.usable_floor_area_m2
        * 3.6
        * practice.map_or(1.0, |item| item.purge.unwrap_or(0.5));
    coefficients.purge_in = purge;
    coefficients.purge_out = purge;

    let (combustion_in, combustion_out) = if policy.use_installed {
        combustion_flows(input, balance)
    } else {
        (0.0, 0.0)
    };
    coefficients.combustion_in = combustion_in;
    coefficients.leakage =
        constants.leakage_q1_m3_per_h * practice.map_or(1.0, |item| item.leakage.unwrap_or(0.5));

    let (argii_in, argii_out) = if balance == Balance::Cooling {
        ventilative_cooling_flows(input, m, indoor, outdoor)
    } else {
        (0.0, 0.0)
    };
    let argii_temperature = VENTILATIVE_COOLING_TEMPERATURE_C[m].unwrap_or(outdoor);

    // Supply temperatures.
    let context = SupplyContext {
        input,
        month_index: m,
        balance,
        outdoor_c: outdoor,
        indoor_c: indoor,
    };
    let grille_rise = if policy.use_installed {
        grille_temperature_rise(input, outdoor, balance)
    } else {
        0.0
    };
    let natural_temperature = outdoor + grille_rise;
    let mut supply_flows = Vec::new();
    let mut frost_protection_kwh = 0.0;
    let mut heating_limit_air_kwh = 0.0;
    let mut cooling_limit_air_kwh = 0.0;
    let mut ahu_heating_kwh = 0.0;
    let mut ahu_cooling_kwh = 0.0;
    for (index, q_supply, unit) in &supply_parts {
        let oda_eff = required * parts(&input.system)[*index].fraction;
        let supply = mechanical_supply_temperature(unit, &context, oda_eff, flea_du);
        let (temperature, defrost, limit_temperature) =
            (supply.dis_out_c, supply.defrost_k, supply.heating_limit_c);
        if supply.coil_k != 0.0 {
            // 11.115/11.116 and 11.119/11.120 with q_V;SUP;dis;in (11.88),
            // literally from the temperature before the fan.
            let air =
                q_supply * flea_du * density(supply.before_coil_c) * AIR_HEAT_CAPACITY_KWH * hours
                    / AHU_COIL_EFFICIENCY;
            if supply.coil_k > 0.0 {
                ahu_heating_kwh += air * supply.coil_energy_k.max(0.0);
            } else {
                ahu_cooling_kwh += air * (-supply.coil_energy_k).max(0.0);
            }
        }
        if balance == Balance::Heating && defrost > 0.0 {
            // 11.105/11.106.
            let power = oda_eff * 1.205 * 1006.0 / 3600.0 * defrost;
            frost_protection_kwh += power * hours / 1000.0;
        }
        supply_flows.push((*index, *q_supply, temperature, limit_temperature));
        // 9.29 with q_V;SUP;dis;out of 11.87.
        let dis_out = oda_eff / (flea_du * flea_ahu);
        heating_limit_air_kwh +=
            dis_out * 1.205 * 1005.0 / 3600.0 * (limit_temperature - outdoor) * hours / 1000.0;
        // 10.20 with q_V;SUP;dis;out.
        cooling_limit_air_kwh +=
            dis_out * 1.205 * 1005.0 / 3600.0 * (supply.cooling_limit_c - outdoor) * hours / 1000.0;
    }

    // Mass balance per airflow zone.
    let pressure = PressureContext {
        outdoor_c: outdoor,
        indoor_c: indoor,
        wind_m_per_s: WIND_SPEED_M_PER_S[m],
    };
    let old_crawlspace = input.floor_above_crawlspace && input.construction_year < 1992;
    let zones = airflow_zones(input.building_height_m, old_crawlspace, coefficients);
    let supply_mass: f64 = supply_flows
        .iter()
        .map(|(_, q, t, _)| density(*t) * q)
        .sum();
    let extract_mass = density(indoor) * extract;
    let fixed_total = supply_mass + extract_mass - density(indoor) * combustion_out
        + density(argii_temperature) * argii_in
        + density(indoor) * argii_out;
    let accuracy = accuracy_kg_per_h(
        required + combustion_in - combustion_out
            + purge
            + argii_in
            + constants.leakage_q1_m3_per_h,
    );
    let mut effective = EffectiveFlows::default();
    let mut pressures = Vec::new();
    let mut converged = true;
    for zone in &zones {
        let reference =
            solve_reference_pressure(zone, &pressure, fixed_total * zone.share, accuracy)
                .unwrap_or_else(|| {
                    converged = false;
                    0.0
                });
        pressures.push(reference);
        for path in &zone.paths {
            let dp = pressure.difference(path, reference);
            if dp > 0.0 && path.coefficient > 0.0 {
                let q = path.coefficient * dp.powf(path.exponent);
                match path.kind {
                    PathKind::Leakage => effective.leakage_in += q,
                    PathKind::Vent => effective.vent_in += q,
                    PathKind::Purge => effective.purge_in += q,
                    PathKind::Combustion => effective.combustion_in += q,
                }
            }
        }
    }

    let effective_oda = balanced_oda
        + if natural_parts {
            effective.vent_in
        } else {
            0.0
        };
    let grille_preheating_kwh = match &input.grille_preheating {
        Some(preheat) if grille_rise > 0.0 => {
            // 11.125–11.128.
            let share = match (
                preheat.preheated_design_flow_m3_per_h,
                &input.installed_capacity,
            ) {
                (Some(flow), Some(capacity)) if capacity.natural_supply_dm3_per_s > 0.0 => {
                    (flow / (capacity.natural_supply_dm3_per_s * 3.6)).min(1.0)
                }
                _ => 1.0,
            };
            let power = effective.vent_in * share * 1.205 * 1006.0 / 3600.0 * grille_rise;
            power * hours / 1000.0
        }
        _ => 0.0,
    };

    let mut demand_flows = vec![
        ("infiltration".to_string(), effective.leakage_in, outdoor),
        (
            "natural_supply".to_string(),
            effective.vent_in,
            natural_temperature,
        ),
        ("purge".to_string(), effective.purge_in, outdoor),
        (
            "ventilative_cooling".to_string(),
            argii_in,
            argii_temperature,
        ),
        ("combustion".to_string(), effective.combustion_in, outdoor),
    ];
    let mut limit_temperatures: Vec<f64> = demand_flows.iter().map(|(_, _, t)| *t).collect();
    let mut supply_total = 0.0;
    let mut supply_weighted = 0.0;
    for (index, q, temperature, limit_temperature) in &supply_flows {
        let id = match &input.system {
            VentilationSystem::Single { .. } => "mechanical_supply".to_string(),
            VentilationSystem::Combined { .. } if *index == 0 => {
                "mechanical_supply_decentral".to_string()
            }
            VentilationSystem::Combined { .. } => "mechanical_supply_other".to_string(),
        };
        demand_flows.push((id, *q, *temperature));
        limit_temperatures.push(*limit_temperature);
        supply_total += q;
        supply_weighted += q * temperature;
    }
    let factor = VOLUMETRIC_HEAT_CAPACITY / 3600.0;
    let conductance: f64 = demand_flows.iter().map(|(_, q, _)| q * factor).sum();
    let heat_flow: f64 = demand_flows
        .iter()
        .map(|(_, q, t)| q * factor * (indoor - t))
        .sum();
    MonthBalance {
        flows: BalanceFlows {
            required_outdoor_air_m3_per_h: required,
            effective_outdoor_air_m3_per_h: effective_oda,
            infiltration_m3_per_h: effective.leakage_in,
            natural_supply_m3_per_h: effective.vent_in,
            purge_m3_per_h: effective.purge_in,
            ventilative_cooling_m3_per_h: argii_in,
            combustion_m3_per_h: effective.combustion_in,
            mechanical_supply_m3_per_h: supply_total,
            mechanical_extract_m3_per_h: -extract,
            natural_supply_temperature_c: natural_temperature,
            mechanical_supply_temperature_c: if supply_total > 0.0 {
                supply_weighted / supply_total
            } else {
                outdoor
            },
            ventilative_cooling_temperature_c: argii_temperature,
            conductance_w_per_k: conductance,
            heat_flow_per_setpoint_w: heat_flow,
        },
        pressures,
        demand_flows,
        limit_temperatures,
        converged,
        frost_protection_kwh,
        grille_preheating_kwh,
        heating_limit_air_kwh: if balance == Balance::Heating {
            heating_limit_air_kwh
        } else {
            0.0
        },
        cooling_limit_air_kwh: if balance == Balance::Cooling {
            cooling_limit_air_kwh
        } else {
            0.0
        },
        ahu_heating_kwh,
        ahu_cooling_kwh,
        outdoor_air_fraction,
    }
}

/// Table 11.20 η_elm.
fn motor_efficiency(power_kw: f64, year: Option<i32>) -> f64 {
    let recent = year.is_some_and(|y| y >= 2005);
    let bands = [
        (1.0, 0.65, 0.70),
        (2.0, 0.70, 0.75),
        (4.0, 0.75, 0.80),
        (10.0, 0.80, 0.85),
        (30.0, 0.85, 0.875),
        (60.0, 0.875, 0.90),
        (120.0, 0.90, 0.925),
        (f64::INFINITY, 0.925, 0.95),
    ];
    let (_, old, new) = bands
        .iter()
        .find(|(limit, _, _)| power_kw < *limit)
        .copied()
        .unwrap_or((f64::INFINITY, 0.925, 0.95));
    if recent {
        new
    } else {
        old
    }
}

fn nominal_fan_power(fan: &Fan) -> f64 {
    match &fan.power {
        FanPower::Nominal { nominal_power_w } => *nominal_power_w,
        FanPower::Motor {
            motor_power_w,
            manufacture_year,
            electrical_input_w,
        } => {
            let efficiency = match electrical_input_w {
                // 11.136, rounded down to a multiple of 0,025.
                Some(input) => ((motor_power_w / input) / 0.025 + 1e-9).floor() * 0.025,
                None => motor_efficiency(motor_power_w / 1000.0, *manufacture_year),
            }
            .min(1.0);
            0.8 * motor_power_w / efficiency
        }
    }
}

/// Table 11.23 f_SFP, W/(m³/h).
fn specific_fan_power(current: FanCurrent, year: Option<i32>) -> f64 {
    let (ac, dc) = match year {
        None => (4.00, 2.20),
        Some(y) if y <= 1980 => (4.00, 2.20),
        Some(y) if y <= 1985 => (2.80, 1.50),
        Some(y) if y <= 1990 => (2.40, 1.30),
        Some(y) if y <= 1998 => (2.00, 1.10),
        Some(y) if y <= 2006 => (1.60, 0.90),
        Some(_) => (f64::NAN, 0.45),
    };
    match current {
        FanCurrent::Ac => ac / 3.6,
        FanCurrent::Dc => dc / 3.6,
    }
}

/// Table 11.22 f_regfan;aan.
fn flow_control_factor(method: FlowControlMethod, reduction: f64) -> f64 {
    let column = if reduction > 0.8 {
        0
    } else if reduction > 0.6 {
        1
    } else if reduction > 0.4 {
        2
    } else {
        3
    };
    let row = match method {
        FlowControlMethod::Throttle => [1.0, 0.95, 0.9, 0.85],
        FlowControlMethod::InletVaneOrBladePitch => [1.0, 0.75, 0.65, 0.6],
        FlowControlMethod::SpeedControl => [1.0, 0.65, 0.45, 0.35],
        FlowControlMethod::Other => [1.0, 1.0, 1.0, 1.0],
    };
    row[column]
}

/// 11.132–11.142, kWh per month.
fn fan_electricity(
    input: &VentilationInput,
    constants: &ZoneConstants,
    month_index: usize,
    required_m3_per_h: f64,
) -> f64 {
    let hours = MONTH_HOURS[month_index];
    let power = match &input.fans {
        Fans::Forfait {
            current,
            manufacture_year,
        } => {
            let sfp = specific_fan_power(*current, *manufacture_year);
            parts(&input.system)
                .iter()
                .map(|part| {
                    sfp * part.unit.variant.fan_system_factor() * required_m3_per_h * part.fraction
                })
                .sum::<f64>()
        }
        Fans::Declared {
            fans,
            control,
            building_share,
            ..
        } => {
            let regulation = match control {
                // 11.137 with table 11.21.
                // 11.137/11.138 (p. 516): Σ f_q;k²·t_d;k with table 11.21.
                FanControl::ResidentialTable => {
                    1.0f64.powi(2) * 0.10 + 0.6f64.powi(2) * 0.60 + 0.4f64.powi(2) * 0.30
                }
                FanControl::Declared { monthly, .. } => monthly[month_index],
                FanControl::FlowControl { control } => {
                    let reduction = input
                        .flow_reduction
                        .flow_control_percent
                        .map_or(1.0, |x| f64::from(x) / 100.0);
                    flow_control_factor(*control, reduction) * constants.occupancy_factor
                }
            };
            fans.iter().map(nominal_fan_power).sum::<f64>() * regulation * building_share
        }
    };
    power * hours / PRACTICE_FACTOR_FANS / 1000.0
}

fn calculate_with_policy(
    input: &VentilationInput,
    policy: Policy,
) -> Result<VentilationResult, Vec<VentilationIssue>> {
    let issues = validate_ventilation(input);
    if !issues.is_empty() {
        return Err(issues);
    }
    let constants = zone_constants(input, policy);
    let mut months = Vec::with_capacity(12);
    let mut flow_rows: Vec<(String, Vec<DemandFlowMonth>)> = Vec::new();
    for m in 0..12 {
        let heating = balance_month(input, &constants, m, Balance::Heating, policy);
        let cooling = balance_month(input, &constants, m, Balance::Cooling, policy);
        for (name, balance) in [("heating", &heating), ("cooling", &cooling)] {
            if !balance.converged {
                return Err(vec![issue(
                    "pressure_balance_not_converged",
                    format!("months[{m}].{name}"),
                )]);
            }
        }
        for (((id, q_h, t_h), (_, q_c, t_c)), t_limit) in heating
            .demand_flows
            .iter()
            .zip(&cooling.demand_flows)
            .zip(&heating.limit_temperatures)
        {
            let factor = VOLUMETRIC_HEAT_CAPACITY / 3600.0;
            let row = DemandFlowMonth {
                month: m as u8 + 1,
                heating_conductance_w_per_k: q_h * factor,
                heating_supply_temperature_c: *t_h,
                cooling_conductance_w_per_k: q_c * factor,
                cooling_supply_temperature_c: *t_c,
                heating_limit_supply_temperature_c: *t_limit,
            };
            match flow_rows.iter_mut().find(|(existing, _)| existing == id) {
                Some((_, rows)) => rows.push(row),
                None => flow_rows.push((id.clone(), vec![row])),
            }
        }
        let fan = fan_electricity(
            input,
            &constants,
            m,
            heating.flows.required_outdoor_air_m3_per_h,
        );
        months.push(VentilationMonthResult {
            month: m as u8 + 1,
            heating: heating.flows,
            cooling: cooling.flows,
            heating_reference_pressure_pa: heating.pressures,
            cooling_reference_pressure_pa: cooling.pressures,
            fan_electricity_kwh: if policy.use_installed { fan } else { 0.0 },
            frost_protection_electricity_kwh: heating.frost_protection_kwh,
            grille_preheating_electricity_kwh: heating.grille_preheating_kwh,
            heating_limit_air_kwh: heating.heating_limit_air_kwh,
            cooling_limit_air_kwh: cooling.cooling_limit_air_kwh,
            ahu_heating_kwh: heating.ahu_heating_kwh,
            ahu_cooling_kwh: cooling.ahu_cooling_kwh,
            outdoor_air_fraction: heating.outdoor_air_fraction,
        });
    }
    let demand_flows = flow_rows
        .into_iter()
        .filter(|(_, rows)| {
            rows.iter()
                .any(|r| r.heating_conductance_w_per_k > 0.0 || r.cooling_conductance_w_per_k > 0.0)
        })
        .map(|(id, months)| DemandFlow {
            id: format!("{}:{id}", input.zone_id),
            months,
        })
        .collect();
    Ok(VentilationResult {
        zone_id: input.zone_id.clone(),
        ventilation_system_op: all_ops(&input.system),
        infiltration_q_v1_m3_per_h: constants.leakage_q1_m3_per_h,
        annual_fan_electricity_kwh: months.iter().map(|m| m.fan_electricity_kwh).sum(),
        annual_frost_protection_electricity_kwh: months
            .iter()
            .map(|m| m.frost_protection_electricity_kwh)
            .sum(),
        annual_grille_preheating_electricity_kwh: months
            .iter()
            .map(|m| m.grille_preheating_electricity_kwh)
            .sum(),
        months,
        demand_flows,
        interpretations: INTERPRETATIONS.to_vec(),
    })
}

impl VentilationResult {
    /// E_V;eldf + E_V;elvv per month (9.28), kWh.
    pub fn heating_limit_electricity_kwh(&self) -> Vec<f64> {
        self.months
            .iter()
            .map(|row| row.frost_protection_electricity_kwh + row.grille_preheating_electricity_kwh)
            .collect()
    }

    /// The effective flows as chapter 7 ventilation flows (7.19/7.20).
    pub fn monthly_demand_flows(&self, source_reference: &str) -> Vec<VentilationFlow> {
        self.demand_flows
            .iter()
            .map(|flow| VentilationFlow {
                id: flow.id.clone(),
                source_reference: source_reference.to_string(),
                months: flow
                    .months
                    .iter()
                    .map(|row| VentilationMonth {
                        month: row.month,
                        conductance_w_per_k: row.heating_conductance_w_per_k,
                        supply_temperature_c: Some(row.heating_supply_temperature_c),
                        cooling_conductance_w_per_k: Some(row.cooling_conductance_w_per_k),
                        cooling_supply_temperature_c: Some(row.cooling_supply_temperature_c),
                    })
                    .collect(),
            })
            .collect()
    }
}

/// Chapter 11 with the zone's actual ventilation system.
pub fn calculate_ventilation(
    input: &VentilationInput,
) -> Result<VentilationResult, Vec<VentilationIssue>> {
    calculate_with_policy(
        input,
        Policy {
            lower_bound: true,
            use_installed: true,
        },
    )
}

/// §5.4.3: the fixed C1 system for the BENG 1 demand run. Infiltration,
/// purge ventilation and the building geometry stay; the system becomes
/// EXTRACT_OP with f_ctrl 1,00 (dwellings) or 1,32 (utility), f_lea;du
/// 1,05, no AHU, no installed capacity, no lower bound of 11.63–11.65, no
/// combustion air, no overventilation, no grille preheating and no full use
/// of the capacity for cooling. Ventilative cooling openings stay (they are
/// building features, not part of the ventilation system).
pub fn c1_variant(input: &VentilationInput) -> VentilationInput {
    let mut fixed = input.clone();
    fixed.system = VentilationSystem::Single {
        unit: SystemUnit {
            variant: SystemVariant::C1,
            heat_recovery: None,
            ducts: DuctAirtightness::LukaABC,
            air_handling_unit: None,
            equipment_reference: "NTA 8800 §5.4.3 fixed C1 system".into(),
        },
    };
    fixed.maximum_capacity_for_cooling = None;
    fixed.installed_capacity = None;
    fixed.flow_reduction = FlowReduction::default();
    fixed.combustion_appliances.clear();
    fixed.overventilation = None;
    fixed.practice = None;
    fixed.grille_preheating = None;
    fixed
}

/// Chapter 11 for the BENG 1 run with the fixed C1 system (§5.4.3).
pub fn calculate_c1_ventilation(
    input: &VentilationInput,
) -> Result<VentilationResult, Vec<VentilationIssue>> {
    calculate_with_policy(
        &c1_variant(input),
        Policy {
            lower_bound: false,
            use_installed: false,
        },
    )
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VentilationAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub issues: Vec<VentilationIssue>,
    /// Actual system (EP_Tot, RER, TOjuli).
    pub actual: Option<VentilationResult>,
    /// Fixed C1 system for BENG 1 (§5.4.3).
    pub fixed_c1: Option<VentilationResult>,
    pub reference_verified: bool,
}

/// Both chapter 11 runs for one zone; unverified until the reference
/// cases of the verification protocol are reproduced.
pub fn assess_ventilation(input: &VentilationInput) -> VentilationAssessment {
    let scope = "nta8800_chapter_11_ventilation_unverified";
    match (
        calculate_ventilation(input),
        calculate_c1_ventilation(input),
    ) {
        (Ok(actual), Ok(fixed_c1)) => VentilationAssessment {
            status: "calculated_unverified",
            scope,
            issues: Vec::new(),
            actual: Some(actual),
            fixed_c1: Some(fixed_c1),
            reference_verified: false,
        },
        (Err(issues), _) | (_, Err(issues)) => VentilationAssessment {
            status: "invalid",
            scope,
            issues,
            actual: None,
            fixed_c1: None,
            reference_verified: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(variant: SystemVariant) -> SystemUnit {
        SystemUnit {
            variant,
            heat_recovery: None,
            ducts: if variant.op() == VentSysOp::Natural {
                DuctAirtightness::NoDucts
            } else {
                DuctAirtightness::LukaABC
            },
            air_handling_unit: None,
            equipment_reference: "design".into(),
        }
    }

    fn dwelling(variant: SystemVariant) -> VentilationInput {
        VentilationInput {
            zone_id: "z1".into(),
            usable_floor_area_m2: 100.0,
            category: Category::Residential,
            functions: vec![FunctionArea {
                function: VentilationFunction::Residential,
                area_m2: 100.0,
                swimming_pool: false,
            }],
            dwelling_count: 1,
            whole_dwelling_area_m2: None,
            apartment_building: false,
            building_height_m: 9.0,
            construction_year: 2015,
            floor_above_crawlspace: false,
            heating_setpoint_c: 20.0,
            cooling_setpoint_c: 24.0,
            system: VentilationSystem::Single {
                unit: unit(variant),
            },
            maximum_capacity_for_cooling: None,
            installed_capacity: None,
            flow_reduction: FlowReduction::default(),
            infiltration: Infiltration::Reference {
                building_type: AirtightnessType::PitchedRoofTerraced,
                renovation_year: None,
            },
            combustion_appliances: Vec::new(),
            overventilation: None,
            practice: None,
            ventilative_cooling: None,
            grille_preheating: None,
            fans: Fans::Forfait {
                current: FanCurrent::Dc,
                manufacture_year: Some(2015),
            },
            source_reference: "drawings".into(),
        }
    }

    fn close(a: f64, b: f64, tolerance: f64) {
        assert!((a - b).abs() <= tolerance, "{a} vs {b}");
    }

    #[test]
    fn required_flow_follows_11_22_and_table_11_8() {
        let result = calculate_ventilation(&dwelling(SystemVariant::C1)).unwrap();
        // f_τ = min(0,38 + 100·0,006; 0,8) = 0,8; q_usi·A_g = 50 dm³/s ≥ 35.
        let design = 1.05 * 0.8 * 50.0 * 3.6;
        close(
            result.months[0].heating.required_outdoor_air_m3_per_h,
            design / 0.95,
            1e-9,
        );
        // Lower bound 11.63 for a small dwelling: 40 m² gives 20 dm³/s → 35.
        let mut small = dwelling(SystemVariant::C1);
        small.usable_floor_area_m2 = 40.0;
        small.functions[0].area_m2 = 40.0;
        let result = calculate_ventilation(&small).unwrap();
        let tau = 0.38 + 40.0 * 0.006;
        close(
            result.months[0].heating.required_outdoor_air_m3_per_h,
            1.05 * tau * 35.0 * 3.6 / 0.95,
            1e-9,
        );
        // The C1 run drops the lower bound (§5.4.3.1).
        let c1 = calculate_c1_ventilation(&small).unwrap();
        close(
            c1.months[0].heating.required_outdoor_air_m3_per_h,
            1.05 * tau * 20.0 * 3.6 / 0.95,
            1e-9,
        );
    }

    #[test]
    fn infiltration_reference_value_follows_11_85_and_11_86() {
        let result = calculate_ventilation(&dwelling(SystemVariant::C1)).unwrap();
        let expected = 1.0 * 0.7 * 1.0 / 10f64.powf(0.67) * 100.0 * 3.6;
        close(result.infiltration_q_v1_m3_per_h, expected, 1e-9);
        assert_eq!(year_factor(1969), 3.0);
        assert_eq!(year_factor(2009), 1.0);
        assert_eq!(year_factor(2010), 0.7);
    }

    #[test]
    fn mass_balance_closes_within_the_11_14_accuracy() {
        for variant in [
            SystemVariant::A1,
            SystemVariant::B1,
            SystemVariant::C1,
            SystemVariant::D1,
        ] {
            let input = dwelling(variant);
            let constants = zone_constants(
                &input,
                Policy {
                    lower_bound: true,
                    use_installed: true,
                },
            );
            for m in 0..12 {
                for balance in [Balance::Heating, Balance::Cooling] {
                    let result = balance_month(
                        &input,
                        &constants,
                        m,
                        balance,
                        Policy {
                            lower_bound: true,
                            use_installed: true,
                        },
                    );
                    assert_eq!(result.pressures.len(), 1);
                    let flows = result.flows;
                    assert!(flows.infiltration_m3_per_h > 0.0, "{variant:?} {m}");
                    if variant == SystemVariant::C1 {
                        // Extracted air is replaced by natural supply and
                        // infiltration, up to the density difference.
                        let inflow = flows.natural_supply_m3_per_h
                            + flows.infiltration_m3_per_h
                            + flows.purge_m3_per_h;
                        assert!(inflow > 0.9 * flows.mechanical_extract_m3_per_h);
                    }
                }
            }
        }
    }

    #[test]
    fn pressure_routine_meets_the_accuracy() {
        let zone = airflow_zones(
            9.0,
            false,
            Coefficients {
                leakage: 60.0,
                vent_in: 150.0,
                ..Coefficients::default()
            },
        )
        .remove(0);
        let context = PressureContext {
            outdoor_c: 2.61,
            indoor_c: 20.0,
            wind_m_per_s: 3.04,
        };
        let fixed = -density(20.0) * 150.0;
        let p = solve_reference_pressure(&zone, &context, fixed, 0.9).unwrap();
        let sum: f64 = fixed
            + zone
                .paths
                .iter()
                .filter(|path| path.coefficient > 0.0)
                .map(|path| context.mass_flow(path, p))
                .sum::<f64>();
        assert!(sum.abs() <= 0.9, "{sum}");
        assert_eq!(accuracy_kg_per_h(999.0), 0.9);
        close(accuracy_kg_per_h(2500.0), 1.8, 1e-12);
    }

    #[test]
    fn airflow_zones_follow_table_11_1() {
        let c = Coefficients {
            leakage: 100.0,
            ..Coefficients::default()
        };
        let low = airflow_zones(10.0, true, c);
        let leak: Vec<_> = low[0]
            .paths
            .iter()
            .filter(|p| p.kind == PathKind::Leakage)
            .map(|p| (p.side, p.height_m, p.coefficient))
            .collect();
        assert_eq!(
            leak,
            vec![
                (Side::Windward, 5.0, 35.0),
                (Side::Leeward, 5.0, 35.0),
                (Side::Roof, 10.0, 15.0),
                (Side::Floor, 0.0, 15.0)
            ]
        );
        let middle = airflow_zones(30.0, false, c);
        assert_eq!(middle.len(), 2);
        close(middle[0].share, 0.5, 1e-12);
        close(middle[1].paths[0].height_m, 22.5, 1e-12);
        close(middle[1].paths[0].coefficient, 0.5 * 0.5 * 100.0, 1e-12);
        let high = airflow_zones(100.0, false, c);
        assert_eq!(high.len(), 3);
        close(high[2].paths[0].height_m, 75.0, 1e-12);
        close(high.iter().map(|z| z.share).sum::<f64>(), 1.0, 1e-12);
        assert_eq!(pressure_coefficient(Side::Windward, 75.0), 0.80);
    }

    #[test]
    fn balanced_supply_temperature_follows_11_103_and_11_107() {
        let mut input = dwelling(SystemVariant::D2);
        if let VentilationSystem::Single { unit } = &mut input.system {
            unit.heat_recovery = Some(HeatRecovery {
                efficiency: HeatRecoveryEfficiency::Table {
                    exchanger: HeatExchanger::CounterFlowPlastic,
                },
                bypass: Bypass::Full {
                    cold_recovery_evidence: None,
                },
                layout: UnitLayout::Central,
                constant_volume_control: false,
                supply_duct_length_m: None,
                supply_duct_insulation: DuctInsulation::Insulated,
                manufacture_year: Some(2015),
                equipment_reference: "unit".into(),
            });
        }
        let result = calculate_ventilation(&input).unwrap();
        // f_prac;hr = 1 − 0,02·4 − 0,05 − 0 = 0,87; ΔT_defrost = 0,2 K.
        let preheated = 2.61 + 0.2;
        let expected = preheated + 0.87 * 0.80 * (20.0 - preheated);
        close(
            result.months[0].heating.mechanical_supply_temperature_c,
            expected,
            1e-9,
        );
        // Full bypass in cooling: no recovery, fan heat 0,4 K for dwellings.
        close(
            result.months[6].cooling.mechanical_supply_temperature_c,
            18.05 + 0.4,
            1e-9,
        );
        // 10.20 as printed: (ϑ_SUP;dis;out − 0 − 0 + 0,4) − ϑ_e = 0,8 K.
        let q = result.months[6].cooling.mechanical_supply_m3_per_h;
        close(
            result.months[6].cooling_limit_air_kwh,
            q * 1.205 * 1005.0 / 3600.0 * 0.8 * 744.0 / 1000.0,
            1e-9,
        );
        // Frost protection 11.105/11.106 in January.
        let q = result.months[0].heating.required_outdoor_air_m3_per_h;
        close(
            result.months[0].frost_protection_electricity_kwh,
            q * 1.205 * 1006.0 / 3600.0 * 0.2 * 744.0 / 1000.0,
            1e-9,
        );
        // Forfait fans: D-system f_systype 2, DC after 2006 0,45/3,6.
        close(
            result.months[0].fan_electricity_kwh,
            0.45 / 3.6 * 2.0 * q * 744.0 / 0.9 / 1000.0,
            1e-9,
        );
    }

    #[test]
    fn ahu_coils_clamp_to_table_11_15() {
        let mut office = dwelling(SystemVariant::D2);
        office.category = Category::Utility;
        office.functions[0].function = VentilationFunction::Office;
        office.dwelling_count = 0;
        let base = calculate_ventilation(&office).unwrap();
        let set_coils = |input: &mut VentilationInput, heating: bool, cooling: bool| {
            if let VentilationSystem::Single { unit } = &mut input.system {
                unit.air_handling_unit = Some(AirHandlingUnit {
                    inside_thermal_zone: true,
                    supply_ducts_outside: DuctOutsideSituation::None,
                    heating_coil: heating,
                    cooling_coil: cooling,
                });
            }
        };
        let mut plain = office.clone();
        set_coils(&mut plain, false, false);
        let plain = calculate_ventilation(&plain).unwrap();
        let mut coils = office.clone();
        set_coils(&mut coils, true, true);
        let coils = calculate_ventilation(&coils).unwrap();
        // Without coils the AHU changes nothing here.
        close(
            plain.months[0].heating.mechanical_supply_temperature_c,
            base.months[0].heating.mechanical_supply_temperature_c,
            1e-9,
        );
        assert_eq!(plain.months[0].ahu_heating_kwh, 0.0);
        // January heating: reheated to 18 °C (table 11.15).
        let before = plain.months[0].heating.mechanical_supply_temperature_c;
        assert!(before < 18.0);
        close(
            coils.months[0].heating.mechanical_supply_temperature_c,
            18.0,
            1e-9,
        );
        // 11.119/11.120 with q_SUP;dis;in ≥ q_SUP;dis;out, from ϑ_SUP;hu
        // (before the 0,7 K fan rise) to ϑ_SUP;dis;in = 18 °C (no ducts).
        let q_out = coils.months[0].heating.mechanical_supply_m3_per_h;
        let hu = before - 0.7;
        let per_flow = density(hu) * 0.000_027_9 * 744.0 * (18.0 - hu) / 0.98;
        let flow = coils.months[0].ahu_heating_kwh / per_flow;
        assert!(
            flow >= q_out - 1e-9 && flow <= q_out * 1.2,
            "{flow} vs {q_out}"
        );
        // July cooling: cooled to 16,5 °C; no reheating in the cooling balance.
        let july = plain.months[6].cooling.mechanical_supply_temperature_c;
        assert!(july > 16.5);
        close(
            coils.months[6].cooling.mechanical_supply_temperature_c,
            16.5,
            1e-9,
        );
        assert!(coils.months[6].ahu_cooling_kwh > 0.0);
        assert_eq!(coils.months[6].ahu_heating_kwh, 0.0);
        // Heating coil only: the cooling balance keeps the formula value.
        let mut heat_only = office.clone();
        set_coils(&mut heat_only, true, false);
        let heat_only = calculate_ventilation(&heat_only).unwrap();
        close(
            heat_only.months[6].cooling.mechanical_supply_temperature_c,
            july,
            1e-9,
        );
        assert_eq!(heat_only.months[6].ahu_cooling_kwh, 0.0);
        // Sport: 16 °C all year.
        let mut hall = office.clone();
        hall.functions[0].function = VentilationFunction::Sport;
        set_coils(&mut hall, true, true);
        let hall = calculate_ventilation(&hall).unwrap();
        close(
            hall.months[6].cooling.mechanical_supply_temperature_c,
            16.0,
            1e-9,
        );
    }

    #[test]
    fn maatwerkadvies_practice_factors_scale_the_actual_flows() {
        let input = dwelling(SystemVariant::C1);
        let mut practice = input.clone();
        practice.practice = Some(VentilationPractice::default());
        let base = calculate_ventilation(&input).unwrap();
        let fitted = calculate_ventilation(&practice).unwrap();
        // System C: f_prac;vent;sys = 0,5 on the mechanical extract.
        close(
            fitted.months[0].heating.mechanical_extract_m3_per_h,
            0.5 * base.months[0].heating.mechanical_extract_m3_per_h,
            1e-9,
        );
        assert!(
            fitted.months[0].heating.conductance_w_per_k
                < base.months[0].heating.conductance_w_per_k
        );
        // The fixed C1 run of §5.4.3 ignores the practice factors.
        let c1 = calculate_c1_ventilation(&practice).unwrap();
        let c1_base = calculate_c1_ventilation(&input).unwrap();
        close(
            c1.months[0].heating.conductance_w_per_k,
            c1_base.months[0].heating.conductance_w_per_k,
            1e-9,
        );
    }

    #[test]
    fn c1_variant_uses_policy_values() {
        let mut office = dwelling(SystemVariant::D5a);
        office.category = Category::Utility;
        office.functions[0].function = VentilationFunction::Office;
        office.dwelling_count = 0;
        let c1 = calculate_c1_ventilation(&office).unwrap();
        // f_ctrl 1,32 for utility, f_τ 0,30, q_usi 1,11, f_lea;du 1,05.
        let design = 1.05 * 0.30 * 1.11 * 100.0 * 3.6;
        close(
            c1.months[0].heating.required_outdoor_air_m3_per_h,
            1.32 / 0.95 * design,
            1e-9,
        );
        assert_eq!(c1.ventilation_system_op, vec![VentSysOp::Extract]);
        assert_eq!(c1.annual_fan_electricity_kwh, 0.0);
    }

    #[test]
    fn combustion_and_purge_reference_flows() {
        let mut input = dwelling(SystemVariant::C1);
        input.combustion_appliances.push(CombustionAppliance {
            id: "boiler".into(),
            kind: CombustionApplianceKind::GasCentralHeatingBoiler,
            class: ApplianceClass::GasTypeB,
            nominal_input_kw: None,
            source_reference: "survey".into(),
        });
        let (q_in, q_out) = combustion_flows(&input, Balance::Heating);
        close(q_in, 0.0, 1e-12);
        close(q_out, 0.20 * 1.0 * 0.78 * 30.0 * 3.6, 1e-9);
        assert_eq!(combustion_flows(&input, Balance::Cooling).1, 0.0);
        let constants = zone_constants(
            &input,
            Policy {
                lower_bound: true,
                use_installed: true,
            },
        );
        let july = balance_month(
            &input,
            &constants,
            6,
            Balance::Cooling,
            Policy {
                lower_bound: true,
                use_installed: true,
            },
        );
        // τ_argIC July 0,08 · 4,2 · 100 · 3,6 at 1 Pa; the effective inflow
        // depends on the pressures but stays positive.
        assert!(july.flows.purge_m3_per_h > 0.0);
    }

    #[test]
    fn cross_ventilation_area_follows_11_77_to_11_79() {
        let opening = |azimuth: f64| CoolingOpening {
            id: "w".into(),
            area: OpeningArea::Declared { net_area_m2: 1.0 },
            centre_height_m: 1.5,
            opening_height_m: 1.0,
            azimuth_deg: azimuth,
            tilt_deg: 90.0,
        };
        let openings = vec![opening(0.0), opening(180.0)];
        assert!(cross_ventilation(&openings));
        let area = cross_area(&openings, &[1.0, 1.0], 2.0);
        close(area, 2.0 / 2f64.sqrt() / 4.0, 1e-12);
        assert!(!cross_ventilation(&[opening(0.0), opening(45.0)]));
        let angle = OpeningArea::OpeningAngle {
            max_net_area_m2: 2.0,
            max_angle_deg: 30.0,
        };
        close(angle.net_area(), 1.46 * 30.0 / 71.0 * 2.0, 1e-12);
        let discharge = OpeningArea::Discharge {
            gross_area_m2: 1.0,
            discharge_coefficient: 0.6,
            entry_loss_coefficient: 0.5,
        };
        close(discharge.net_area(), 0.55 / 0.67, 1e-12);
    }

    #[test]
    fn single_sided_ventilative_cooling_follows_11_73() {
        let mut input = dwelling(SystemVariant::C1);
        input.ventilative_cooling = Some(VentilativeCooling {
            openings: vec![CoolingOpening {
                id: "hatch".into(),
                area: OpeningArea::Declared { net_area_m2: 0.5 },
                centre_height_m: 2.0,
                opening_height_m: 1.0,
                azimuth_deg: 180.0,
                tilt_deg: 90.0,
            }],
            operation: CoolingOperation::Manual,
            conditions_evidence: "NEN 5096 report".into(),
        });
        let (q_in, q_out) = ventilative_cooling_flows(&input, 6, 24.0, 18.05);
        let driver = (0.001 * 2.63f64.powi(2)).max(0.0035 * 1.0 * (24.0f64 - 17.51).abs());
        let expected = 3600.0 * 1.205 / density(17.51) * 0.25 * driver.sqrt() * 0.81 * 0.35;
        close(q_in, expected, 1e-9);
        assert!(q_out < 0.0);
        assert_eq!(ventilative_cooling_flows(&input, 0, 24.0, 2.61), (0.0, 0.0));
    }

    #[test]
    fn assessment_returns_actual_and_c1_runs() {
        let assessment = assess_ventilation(&dwelling(SystemVariant::D5c));
        assert_eq!(assessment.status, "calculated_unverified");
        let actual = assessment.actual.unwrap();
        let c1 = assessment.fixed_c1.unwrap();
        assert!(
            actual.months[0].heating.required_outdoor_air_m3_per_h
                < c1.months[0].heating.required_outdoor_air_m3_per_h
        );
        let mut broken = dwelling(SystemVariant::C1);
        broken.usable_floor_area_m2 = 0.0;
        assert_eq!(assess_ventilation(&broken).status, "invalid");
    }

    #[test]
    fn validation_reports_unsupported_and_missing_inputs() {
        let mut input = dwelling(SystemVariant::C4a);
        input.category = Category::Utility;
        input.dwelling_count = 0;
        input.functions[0].function = VentilationFunction::Office;
        input.combustion_appliances.push(CombustionAppliance {
            id: "x".into(),
            kind: CombustionApplianceKind::GasStove,
            class: ApplianceClass::SpecificGasAppliance,
            nominal_input_kw: None,
            source_reference: String::new(),
        });
        let codes: Vec<_> = validate_ventilation(&input)
            .into_iter()
            .map(|i| i.code)
            .collect();
        for code in [
            "variant_not_applicable_to_category",
            "specific_gas_appliance_unsupported",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    #[test]
    fn residential_fan_regulation_and_duct_losses_follow_11_137_and_table_11_19() {
        let mut input = dwelling(SystemVariant::C1);
        input.fans = Fans::Declared {
            fans: vec![Fan {
                id: "f".into(),
                power: FanPower::Nominal {
                    nominal_power_w: 80.0,
                },
            }],
            control: FanControl::ResidentialTable,
            building_share: 1.0,
            source_reference: "plate".into(),
        };
        let result = calculate_ventilation(&input).unwrap();
        // Σ f_q²·t_d = 0,1 + 0,216 + 0,048 = 0,364.
        close(
            result.months[0].fan_electricity_kwh,
            80.0 * 0.364 * 744.0 / 0.9 / 1000.0,
            1e-9,
        );
        assert_eq!(DuctOutsideSituation::Situation1.delta_k(0), 0.0);
        assert_eq!(DuctOutsideSituation::Situation2.delta_k(0), 2.82);
        assert_eq!(DuctOutsideSituation::Situation3.delta_k(0), 5.72);
    }

    #[test]
    fn fan_motor_power_follows_11_135_and_table_11_20() {
        let fan = Fan {
            id: "f".into(),
            power: FanPower::Motor {
                motor_power_w: 1500.0,
                manufacture_year: Some(2010),
                electrical_input_w: None,
            },
        };
        close(nominal_fan_power(&fan), 0.8 * 1500.0 / 0.75, 1e-9);
        let measured = Fan {
            id: "f".into(),
            power: FanPower::Motor {
                motor_power_w: 1500.0,
                manufacture_year: None,
                electrical_input_w: Some(1900.0),
            },
        };
        // 1500/1900 = 0,789 → 0,775.
        close(nominal_fan_power(&measured), 0.8 * 1500.0 / 0.775, 1e-9);
        assert_eq!(
            flow_control_factor(FlowControlMethod::SpeedControl, 0.8),
            0.65
        );
    }
}
