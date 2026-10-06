//! Space-heating emission losses, NTA 8800 §9.3 (NEN-EN 15316-2).
//!
//! Implements 9.9–9.12 and 9.12a with the forfait temperature corrections of
//! tables 9.2 (system), 9.3 (hydronic balancing) and 9.4 (control). Checked
//! against NTA 8800:2025+C1:2026 pages 296–297: identical to the 2026
//! consultation draft except that the draft's 9.16 is numbered 9.12a.

use serde::{Deserialize, Serialize};

pub const DRAFT_SOURCE: &str =
    "NTA 8800:2025+C1:2026, §9.3, formules 9.9–9.12a en tabellen 9.2–9.4 (p. 294–297)";

/// 9.12a: upper bound of the relative emission loss.
pub const MAX_RELATIVE_LOSS: f64 = 0.15;

/// Table 9.2 rows, emission system in the main room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionSystem {
    RadiatorsOrConvectors,
    FloorHeating,
    FanAssistedRadiatorsOrConvectors,
    AirHeating,
    /// Local heaters (stove, local electric heater): table 9.2 "overige", Δθ_hydr = 0.
    LocalHeater,
    OtherOrUnknown,
}

/// Table 9.3 rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HydronicBalancing {
    NoneOrUnknown,
    Static,
    Dynamic,
    /// Local heaters and air heating via ducts: `Δθ_hydr = 0` (footnote a).
    NotApplicable,
}

/// Table 9.4 rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionControl {
    MainRoomThermostat,
    CentralWithRoomValves,
    IndividualRoomThermostats,
    OtherOrUnknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmissionInput {
    pub system: EmissionSystem,
    pub balancing: HydronicBalancing,
    pub control: EmissionControl,
    pub source_reference: String,
    /// Fans for air circulation in the room (9.21/9.22, table 9.11);
    /// required for fan-assisted radiators or convectors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fans: Option<EmissionFans>,
    /// 9.23 with tables 9.12/9.13: fans and controls of direct-fired or
    /// indirect air heaters in the zone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub air_heaters: Option<AirHeaterAuxiliary>,
    /// NTA 8800:2023 9.3.2–9.3.3 (p. 273–285): the emission system in the
    /// terms of that edition (tables 9.2–9.10). Accepted under 2023 only;
    /// without it a 2023 run derives the forfait from the fields above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition2023: Option<Emission2023>,
}

/// Table 9.12 (direct-fired) or 9.13 (indirect) air heater type; unknown
/// properties take the highest value of the category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AirHeaterKind {
    /// Table 9.12: direct-fired air heater in the work space.
    Direct {
        #[serde(default, rename = "radialFan")]
        radial_fan: Option<bool>,
    },
    /// Table 9.13: indirect air heater on a central generator.
    Indirect {
        #[serde(default, rename = "roomHeightAbove8M")]
        room_height_above_8_m: Option<bool>,
        #[serde(default, rename = "warmAirReturn")]
        warm_air_return: Option<bool>,
        #[serde(default, rename = "ecMotor")]
        ec_motor: Option<bool>,
    },
}

impl AirHeaterKind {
    /// P_H,aux = factor · Q_h;b / n_H,aux (tables 9.12/9.13).
    pub fn factor(self) -> f64 {
        match self {
            Self::Direct { radial_fan } => match radial_fan {
                Some(false) => 0.014,
                _ => 0.022,
            },
            Self::Indirect {
                room_height_above_8_m,
                warm_air_return,
                ec_motor,
            } => {
                let cell = |high: bool, ret: bool, ec: bool| match (high, ret, ec) {
                    (false, true, false) => 0.008,
                    (false, true, true) => 0.004,
                    (false, false, false) => 0.009,
                    (false, false, true) => 0.005,
                    (true, true, false) => 0.012,
                    (true, true, true) => 0.006,
                    (true, false, false) => 0.013,
                    (true, false, true) => 0.007,
                };
                let options = |value: Option<bool>| match value {
                    Some(value) => vec![value],
                    None => vec![false, true],
                };
                let mut highest: f64 = 0.0;
                for high in options(room_height_above_8_m) {
                    for ret in options(warm_air_return) {
                        for ec in options(ec_motor) {
                            highest = highest.max(cell(high, ret, ec));
                        }
                    }
                }
                highest
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AirHeaterAuxiliary {
    pub kind: AirHeaterKind,
    /// Q_h;b per EN 12831-1, W; absent means the table 9.12 estimate.
    #[serde(default)]
    pub design_heat_load_w: Option<f64>,
    pub source_reference: String,
}

/// Table 9.12 estimate of Q_h;b: Q_H;ht of January over t_jan scaled from
/// 21 °C to −10 °C, plus (√A_g·4·3 + A_g)·5 W for heating up.
pub fn estimated_design_heat_load_w(january_heat_transfer_kwh: f64, area_m2: f64) -> f64 {
    let january_outdoor = crate::climate::OUTDOOR_TEMPERATURE_C[0];
    let hours = crate::climate::MONTH_HOURS[0];
    let transmission =
        january_heat_transfer_kwh / (0.001 * hours) * (21.0 - (-10.0)) / (21.0 - january_outdoor);
    let heating_up = (area_m2.max(0.0).sqrt() * 4.0 * 3.0 + area_m2) * 5.0;
    transmission + heating_up
}

/// Table 9.11 decisive property of the room fans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionFanKind {
    /// Fan convector (fancoil), also the indoor unit of a (multi-)split.
    FanConvector,
    ElectricHeating,
    /// Local dynamic heat storage (soapstone, PCM).
    DynamicStorage,
    /// Not known: the highest value of table 9.11.
    Unknown,
}

impl EmissionFanKind {
    /// Table 9.11, W per fan.
    pub fn power_w(self) -> f64 {
        match self {
            Self::FanConvector | Self::ElectricHeating => 10.0,
            Self::DynamicStorage | Self::Unknown => 12.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmissionFans {
    pub kind: EmissionFanKind,
    /// n_fan;zi of 9.22.
    pub count: u32,
    /// Fan power of an assembly tested to NEN-EN 16430, W per fan; replaces
    /// table 9.11.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tested_power_w: Option<f64>,
    pub source_reference: String,
}

impl EmissionFans {
    pub fn power_w(&self) -> f64 {
        self.tested_power_w.unwrap_or_else(|| self.kind.power_w())
    }
}

pub fn system_correction_k(system: EmissionSystem) -> f64 {
    match system {
        EmissionSystem::RadiatorsOrConvectors
        | EmissionSystem::OtherOrUnknown
        | EmissionSystem::LocalHeater => 0.35,
        EmissionSystem::FloorHeating => 0.3,
        EmissionSystem::FanAssistedRadiatorsOrConvectors => -0.15,
        EmissionSystem::AirHeating => 0.0,
    }
}

pub fn balancing_correction_k(balancing: HydronicBalancing) -> f64 {
    match balancing {
        HydronicBalancing::NoneOrUnknown => 0.7,
        HydronicBalancing::Static => 0.4,
        HydronicBalancing::Dynamic => 0.2,
        HydronicBalancing::NotApplicable => 0.0,
    }
}

pub fn control_correction_k(control: EmissionControl) -> f64 {
    match control {
        EmissionControl::MainRoomThermostat | EmissionControl::OtherOrUnknown => 2.5,
        EmissionControl::CentralWithRoomValves => 2.0,
        EmissionControl::IndividualRoomThermostats => 1.5,
    }
}

/// 9.12: `Δθ_int;inc` in K; under NTA 8800:2023 the 9.12 sum of that
/// edition ([`Emission2023::increment_k`]).
pub fn temperature_increment_k(input: &EmissionInput) -> f64 {
    if crate::norm_versions::profile().emission_tables_2023 {
        return input
            .edition2023
            .unwrap_or_else(|| Emission2023::forfait_from(input))
            .increment_k();
    }
    system_correction_k(input.system)
        + balancing_correction_k(input.balancing)
        + control_correction_k(input.control)
}

/// Table 9.2 of NTA 8800:2023 (p. 275): one- or two-pipe heating; an
/// unknown system takes the one-pipe column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipeSystem2023 {
    OnePipe,
    TwoPipe,
    #[default]
    Unknown,
    /// Local heaters and air heating without water: Δθ_hydr = 0.
    NotHydronic,
}

/// Rows of table 9.2 (2023 p. 275) from top to bottom; the same row of
/// both columns is the same level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BalancingRow2023 {
    /// Without hydronic balancing, or unknown (also without a NEN-EN 14336
    /// G1 declaration).
    #[default]
    NoneOrUnknown,
    /// Static per circuit / static per panel without group balancing.
    Static,
    /// Dynamic per circuit / static per panel with group balancing.
    StaticOrDynamicWithGroups,
    /// Dynamic per circuit with load control (return temperature limit) /
    /// static per panel with a dynamic group balance.
    DynamicWithLoadControl,
    /// Dynamic per circuit on the supply-return difference / dynamic per
    /// panel.
    DynamicFull,
}

/// Δθ_roomaut of 9.3.3.2–9.3.3.4 (2023 p. 278, 280, 281).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomAutomation2023 {
    /// No information: −0,0 K.
    #[default]
    Unknown,
    IndividualPerRoom,
    IndividualWithManualOverride,
    NetworkWithOverrideAndAdaptive,
}

impl RoomAutomation2023 {
    pub fn delta_k(self) -> f64 {
        match self {
            Self::Unknown => 0.0,
            Self::IndividualPerRoom => -0.5,
            Self::IndividualWithManualOverride => -1.0,
            Self::NetworkWithOverrideAndAdaptive => -1.2,
        }
    }
}

/// Room temperature control of tables 9.3/9.4 (2023 p. 277, 279).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomControl2023 {
    /// Central supply temperature control without room control: 2,5 K. The
    /// table prints it in the Δθ_ctr,2 column only; it is read for both.
    Central,
    /// Main-room control, one-pipe heating or room temperature control (P,
    /// PI, PI with optimisation): Δθ_ctr,1 2,5, Δθ_ctr,2 1,5.
    Room,
}

/// Over-temperature rows of table 9.3 (2023 p. 277), Δθ_str,1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverTemperature2023 {
    MechanicalVentilation,
    FanAssisted,
    LocalHeater,
    TwoPipe60KOrUnknown,
    TwoPipe42K,
    TwoPipe30K,
    TwoPipe20K,
    OnePipe60KOrUnknown,
    OnePipe42K,
    /// The highest value of the category: 1,6.
    Unknown,
}

impl OverTemperature2023 {
    pub fn delta_k(self) -> f64 {
        match self {
            Self::MechanicalVentilation => 0.2,
            Self::FanAssisted => 0.0,
            Self::LocalHeater | Self::TwoPipe60KOrUnknown | Self::OnePipe42K => 1.2,
            Self::TwoPipe42K => 0.7,
            Self::TwoPipe30K => 0.5,
            Self::TwoPipe20K => 0.4,
            Self::OnePipe60KOrUnknown | Self::Unknown => 1.6,
        }
    }
}

/// Specific heat losses by outer components, table 9.3 (2023 p. 278),
/// Δθ_str,2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RadiatorPosition2023 {
    InnerWall,
    OuterWallGlassWithoutProtection,
    OuterWallGlassWithProtection,
    OuterWall,
    /// The highest value of the category: 1,7.
    Unknown,
}

impl RadiatorPosition2023 {
    pub fn delta_k(self) -> f64 {
        match self {
            Self::InnerWall => 1.3,
            Self::OuterWallGlassWithoutProtection | Self::Unknown => 1.7,
            Self::OuterWallGlassWithProtection => 1.2,
            Self::OuterWall => 0.3,
        }
    }
}

/// System rows of table 9.4 (2023 p. 279).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceSystem2023 {
    MechanicalVentilation,
    FloorWetOrUnknown,
    FloorDry,
    FloorThinScreed,
    Wall,
    Ceiling,
    /// The highest values of the category: 0,7 and 0,7.
    Unknown,
}

impl SurfaceSystem2023 {
    /// (Δθ_str, Δθ_emb,1), K.
    pub fn deltas_k(self) -> (f64, f64) {
        match self {
            Self::MechanicalVentilation | Self::FloorWetOrUnknown | Self::FloorDry => (0.0, 0.7),
            Self::FloorThinScreed => (0.0, 0.4),
            Self::Wall => (0.4, 0.2),
            Self::Ceiling | Self::Unknown => (0.7, 0.7),
        }
    }
}

/// Heat losses by horizontal emitting surfaces, table 9.4 (2023 p. 280),
/// Δθ_emb,2; `Unknown` applies 9.18a.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceInsulation2023 {
    WithoutInsulation,
    MinimalInsulation,
    DoubleInsulation,
    Unknown,
}

/// Table 9.6 control (2023 p. 282): electric air heating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ElectricAirControl2023 {
    PPerZone,
    CentralWithLocalP,
    PPerRoom,
    PiPerRoom,
    /// The highest value of the wall area.
    Unknown,
}

/// Table 9.6 area (2023 p. 282).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WallArea2023 {
    OuterWall,
    InnerWall,
    Unknown,
}

/// Table 9.7 (2023 p. 283): air heating by ventilation systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VentilationAirHeating2023 {
    ReheatRoomAir,
    ReheatCascade,
    ReheatExtractAir,
    /// Reheat with an unknown control: the highest value, 1,9/1,5.
    ReheatUnknown,
    Recirculation,
}

/// Emitters of tables 9.8 and 9.10 (2023 p. 284–285), rooms above 4 m.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HighRoomEmitter2023 {
    WarmAirHorizontal,
    WarmAirHorizontalLowTemperature,
    WarmAirFromCeiling,
    WarmAirFromCeilingLowTemperature,
    RecirculationTwoStep,
    RecirculationPi,
    DarkRadiators,
    HighTemperatureRadiators,
    CeilingPanels,
    FloorUninsulatedSpacingUpTo20Cm,
    FloorUninsulatedSpacingAbove20Cm,
    FloorMinimalInsulationUpTo10Cm,
    FloorMinimalInsulationAbove10Cm,
    FloorThermallyDecoupled,
    /// Floor heating of unknown build-up: the highest Δθ_emb, 1,9.
    FloorUnknown,
}

impl HighRoomEmitter2023 {
    /// Table 9.8 θ'_str, K/m.
    pub fn gradient_k_per_m(self) -> f64 {
        match self {
            Self::WarmAirHorizontal => 1.0,
            Self::WarmAirHorizontalLowTemperature
            | Self::WarmAirFromCeilingLowTemperature
            | Self::RecirculationTwoStep => 0.35,
            Self::WarmAirFromCeiling => 0.60,
            Self::RecirculationPi => 0.25,
            Self::DarkRadiators | Self::HighTemperatureRadiators => 0.20,
            Self::CeilingPanels => 0.40,
            Self::FloorUninsulatedSpacingUpTo20Cm
            | Self::FloorUninsulatedSpacingAbove20Cm
            | Self::FloorMinimalInsulationUpTo10Cm
            | Self::FloorMinimalInsulationAbove10Cm
            | Self::FloorThermallyDecoupled
            | Self::FloorUnknown => 0.10,
        }
    }

    /// Table 9.10 Δθ_emb, K.
    pub fn embedded_k(self) -> f64 {
        match self {
            Self::CeilingPanels | Self::FloorMinimalInsulationUpTo10Cm => 0.5,
            Self::FloorUninsulatedSpacingUpTo20Cm => 1.4,
            Self::FloorUninsulatedSpacingAbove20Cm | Self::FloorUnknown => 1.9,
            Self::FloorMinimalInsulationAbove10Cm => 1.0,
            _ => 0.0,
        }
    }

    /// The low-temperature warm-air rows hold for rooms up to 6 m.
    fn up_to_6_m(self) -> bool {
        matches!(
            self,
            Self::WarmAirHorizontalLowTemperature | Self::WarmAirFromCeilingLowTemperature
        )
    }

    /// 9.20 applies to dark and high-temperature radiators.
    pub fn radiant(self) -> bool {
        matches!(self, Self::DarkRadiators | Self::HighTemperatureRadiators)
    }
}

/// Table 9.10 control (2023 p. 285).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HighRoomControl2023 {
    NotControlled,
    /// 2-step, P, PI or PI with optimisation.
    Controlled,
}

/// Radiant product data of 9.20 (2023 p. 284).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RadiantProduct2023 {
    /// RF per NEN-EN 416-2/419-2; omitted means 0,55.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radiation_factor: Option<f64>,
    /// p_h, W/m².
    pub specific_power_w_per_m2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Emission2023Kind {
    /// 9.3.3.2, table 9.3 (p. 276–279).
    Radiators {
        control: RoomControl2023,
        #[serde(rename = "overTemperature")]
        over_temperature: OverTemperature2023,
        position: RadiatorPosition2023,
    },
    /// 9.3.3.3, table 9.4 (p. 279–281).
    Surface {
        control: RoomControl2023,
        system: SurfaceSystem2023,
        insulation: SurfaceInsulation2023,
    },
    /// 9.3.3.4: non-electric air heating of dwellings (p. 281).
    DwellingAir { control: RoomControl2023 },
    /// 9.3.3.5, table 9.6 (p. 281–282).
    ElectricAir {
        wall: WallArea2023,
        control: ElectricAirControl2023,
    },
    /// 9.3.3.6, table 9.7 (p. 282–283).
    VentilationAir {
        configuration: VentilationAirHeating2023,
    },
    /// 9.3.3.7, 9.19/9.20 and tables 9.8/9.10 (p. 283–285): rooms above
    /// 4 m.
    HighRoom {
        #[serde(rename = "heightM")]
        height_m: f64,
        emitter: HighRoomEmitter2023,
        control: HighRoomControl2023,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        radiant: Option<RadiantProduct2023>,
    },
}

/// NTA 8800:2023 9.3.2–9.3.3 description of a heating emission system.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Emission2023 {
    pub kind: Emission2023Kind,
    /// Controls certified to NEN-EN 215/NEN-EN 15500: Δθ_ctr,2 of tables
    /// 9.3/9.4/9.10, variation a of table 9.6, the high-quality column of
    /// table 9.7.
    #[serde(default)]
    pub certified_control: bool,
    #[serde(default)]
    pub room_automation: RoomAutomation2023,
    #[serde(default)]
    pub pipe_system: PipeSystem2023,
    #[serde(default)]
    pub balancing: BalancingRow2023,
}

impl Emission2023 {
    /// Table 9.2 (2023 p. 275).
    pub fn hydronic_k(&self) -> f64 {
        let row = match self.balancing {
            BalancingRow2023::NoneOrUnknown => 0,
            BalancingRow2023::Static => 1,
            BalancingRow2023::StaticOrDynamicWithGroups => 2,
            BalancingRow2023::DynamicWithLoadControl => 3,
            BalancingRow2023::DynamicFull => 4,
        };
        match self.pipe_system {
            PipeSystem2023::NotHydronic => 0.0,
            PipeSystem2023::TwoPipe => [0.6, 0.4, 0.3, 0.2, 0.0][row],
            PipeSystem2023::OnePipe | PipeSystem2023::Unknown => [0.7, 0.4, 0.3, 0.2, 0.1][row],
        }
    }

    fn room_control_k(&self, control: RoomControl2023) -> f64 {
        match (control, self.certified_control) {
            (RoomControl2023::Room, true) => 1.5,
            _ => 2.5,
        }
    }

    /// Whether the input can be evaluated: a room above 4 m (at most 6 m
    /// for the low-temperature warm-air rows) and the product data of 9.20
    /// for radiant heaters.
    pub fn valid(&self) -> bool {
        match self.kind {
            Emission2023Kind::HighRoom {
                height_m,
                emitter,
                radiant,
                ..
            } => {
                let height = height_m.is_finite()
                    && height_m > 4.0
                    && (!emitter.up_to_6_m() || height_m <= 6.0);
                let product = !emitter.radiant()
                    || radiant.is_some_and(|product| {
                        let rf = product.radiation_factor.unwrap_or(0.55);
                        rf.is_finite()
                            && rf > 0.0
                            && rf <= 1.0
                            && product.specific_power_w_per_m2.is_finite()
                            && product.specific_power_w_per_m2 > 0.0
                    });
                height && product
            }
            _ => true,
        }
    }

    /// 9.12 of NTA 8800:2023 (p. 273): Δθ_str + Δθ_ctr + Δθ_emb + Δθ_rad +
    /// Δθ_im + Δθ_hydr + Δθ_roomaut, K. `NaN` when [`Self::valid`] fails.
    pub fn increment_k(&self) -> f64 {
        if !self.valid() {
            return f64::NAN;
        }
        let hydr = self.hydronic_k();
        let roomaut = self.room_automation.delta_k();
        match self.kind {
            Emission2023Kind::Radiators {
                control,
                over_temperature,
                position,
            } => {
                // 9.17 and Δθ_im;emt −0,3 (p. 278).
                let stratification = (over_temperature.delta_k() + position.delta_k()) / 2.0;
                stratification + self.room_control_k(control) - 0.3 + hydr + roomaut
            }
            Emission2023Kind::Surface {
                control,
                system,
                insulation,
            } => {
                let (stratification, emb1) = system.deltas_k();
                // 9.18/9.18a and Δθ_im;emt −0,2 (p. 280).
                let embedded = match insulation {
                    SurfaceInsulation2023::WithoutInsulation => (emb1 + 1.4) / 2.0,
                    SurfaceInsulation2023::MinimalInsulation => (emb1 + 0.5) / 2.0,
                    SurfaceInsulation2023::DoubleInsulation => (emb1 + 0.1) / 2.0,
                    SurfaceInsulation2023::Unknown => emb1,
                };
                stratification + self.room_control_k(control) + embedded - 0.2 + hydr + roomaut
            }
            // p. 281: Δθ_str = Δθ_emb = Δθ_im = 0, Δθ_ctr per table 9.3.
            Emission2023Kind::DwellingAir { control } => {
                self.room_control_k(control) + hydr + roomaut
            }
            Emission2023Kind::ElectricAir { wall, control } => {
                use ElectricAirControl2023 as C;
                // Table 9.6: (variation a, variation b).
                let row = |wall: WallArea2023, control: C| -> (f64, f64) {
                    let outer = wall == WallArea2023::OuterWall;
                    match (outer, control) {
                        (true, C::PPerZone) => (1.2, 1.2),
                        (true, C::CentralWithLocalP) => (1.1, 1.2),
                        (true, C::PPerRoom) => (1.1, 1.1),
                        (true, C::PiPerRoom) => (0.7, 1.1),
                        (false, C::PPerZone) => (1.6, 1.6),
                        (false, C::CentralWithLocalP | C::PPerRoom) => (1.5, 1.5),
                        (false, C::PiPerRoom) => (1.1, 3.1),
                        (_, C::Unknown) => (f64::NAN, f64::NAN),
                    }
                };
                // Unknown properties: the highest value per type (p. 282).
                let walls: &[WallArea2023] = match wall {
                    WallArea2023::Unknown => &[WallArea2023::OuterWall, WallArea2023::InnerWall],
                    _ => std::slice::from_ref(&wall),
                };
                let controls: &[C] = match control {
                    C::Unknown => &[C::PPerZone, C::CentralWithLocalP, C::PPerRoom, C::PiPerRoom],
                    _ => std::slice::from_ref(&control),
                };
                let mut highest = f64::NEG_INFINITY;
                for wall in walls {
                    for control in controls {
                        let (a, b) = row(*wall, *control);
                        highest = highest.max(if self.certified_control { a } else { b });
                    }
                }
                // Δθ_im = −0,3 K; Δθ_str = Δθ_emb = 0 (p. 281–282).
                highest - 0.3 + hydr
            }
            Emission2023Kind::VentilationAir { configuration } => {
                // Table 9.7: (low, high) quality of control; Δθ_im = 0.
                let (low, high) = match configuration {
                    VentilationAirHeating2023::ReheatRoomAir => (1.8, 1.3),
                    VentilationAirHeating2023::ReheatCascade => (1.2, 1.0),
                    VentilationAirHeating2023::ReheatExtractAir
                    | VentilationAirHeating2023::ReheatUnknown => (1.9, 1.5),
                    VentilationAirHeating2023::Recirculation => (1.1, 0.7),
                };
                (if self.certified_control { high } else { low }) + hydr
            }
            Emission2023Kind::HighRoom {
                height_m,
                emitter,
                control,
                radiant,
            } => {
                // 9.19 with a = 16 K and b = 1,1 m.
                let stratification =
                    10.0 * emitter.gradient_k_per_m() / 16.0 * (0.5 * height_m - 1.1);
                // 9.20 for dark and high-temperature radiators.
                let radiation = match radiant {
                    Some(product) if emitter.radiant() => {
                        let rf = product.radiation_factor.unwrap_or(0.55);
                        let p_h = product.specific_power_w_per_m2;
                        10.0 * (0.36 / (rf + 0.2)
                            + 0.354 * (70.0 / p_h).powf(0.12) * (10.0 / height_m).powf(0.15)
                            - 0.9)
                    }
                    _ => 0.0,
                };
                // Table 9.10: Δθ_ctr,1 2,5; Δθ_ctr,2 2,5 uncontrolled and
                // 0,7 controlled.
                let control_k = match (control, self.certified_control) {
                    (HighRoomControl2023::Controlled, true) => 0.7,
                    _ => 2.5,
                };
                stratification + control_k + emitter.embedded_k() + radiation + hydr
            }
        }
    }

    /// The 2023 forfait for an input described in 2024 terms: properties
    /// the 2024 input does not carry take the "unknown" (highest) value of
    /// their category (2023 p. 276, 279) and an uncertified control;
    /// documented in docs/nta8800-normversies.md.
    pub fn forfait_from(input: &EmissionInput) -> Self {
        let room_automation = match input.control {
            EmissionControl::IndividualRoomThermostats => RoomAutomation2023::IndividualPerRoom,
            _ => RoomAutomation2023::Unknown,
        };
        let balancing = match input.balancing {
            HydronicBalancing::NoneOrUnknown | HydronicBalancing::NotApplicable => {
                BalancingRow2023::NoneOrUnknown
            }
            HydronicBalancing::Static => BalancingRow2023::Static,
            HydronicBalancing::Dynamic => BalancingRow2023::StaticOrDynamicWithGroups,
        };
        let pipe_system = if input.balancing == HydronicBalancing::NotApplicable {
            PipeSystem2023::NotHydronic
        } else {
            PipeSystem2023::Unknown
        };
        let control = match input.control {
            EmissionControl::OtherOrUnknown => RoomControl2023::Central,
            _ => RoomControl2023::Room,
        };
        let radiators = |over_temperature| Emission2023Kind::Radiators {
            control,
            over_temperature,
            position: RadiatorPosition2023::Unknown,
        };
        let kind = match input.system {
            EmissionSystem::RadiatorsOrConvectors | EmissionSystem::OtherOrUnknown => {
                radiators(OverTemperature2023::Unknown)
            }
            EmissionSystem::FanAssistedRadiatorsOrConvectors => {
                radiators(OverTemperature2023::FanAssisted)
            }
            EmissionSystem::LocalHeater => radiators(OverTemperature2023::LocalHeater),
            EmissionSystem::FloorHeating => Emission2023Kind::Surface {
                control,
                system: SurfaceSystem2023::FloorWetOrUnknown,
                insulation: SurfaceInsulation2023::Unknown,
            },
            EmissionSystem::AirHeating => Emission2023Kind::DwellingAir { control },
        };
        Self {
            kind,
            certified_control: false,
            room_automation,
            pipe_system,
            balancing,
        }
    }
}

/// Returns `true` when the balancing choice contradicts footnote a of
/// table 9.3 (air heating needs `NotApplicable`; wet systems must state it).
pub fn balancing_consistent(input: &EmissionInput) -> bool {
    // Table 9.3 footnote a: local heaters and air heating have Δθ_hydr = 0.
    let non_hydronic = matches!(
        input.system,
        EmissionSystem::AirHeating | EmissionSystem::LocalHeater
    );
    non_hydronic == (input.balancing == HydronicBalancing::NotApplicable)
}

/// 9.12a for one month: `Q_H;em;ls` in kWh for `Q_H;em;out = Q_H;nd`.
pub fn monthly_loss_kwh(
    heat_output_kwh: f64,
    setpoint_c: f64,
    outdoor_c: f64,
    increment_k: f64,
) -> f64 {
    // 9.12a condition: θ_H;int;ini − θ_e;comb > 0, with θ_e;comb = θ_e;avg;mi.
    if setpoint_c - outdoor_c <= 0.0 {
        return 0.0;
    }
    // 9.11
    let inclusive = setpoint_c + increment_k;
    let ratio = increment_k / (inclusive - outdoor_c);
    heat_output_kwh * ratio.min(MAX_RELATIVE_LOSS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        system: EmissionSystem,
        balancing: HydronicBalancing,
        control: EmissionControl,
    ) -> EmissionInput {
        EmissionInput {
            system,
            balancing,
            control,
            source_reference: "test".into(),
            fans: None,
            air_heaters: None,
            edition2023: None,
        }
    }

    #[test]
    fn increments_follow_tables_9_2_to_9_4() {
        let radiators = input(
            EmissionSystem::RadiatorsOrConvectors,
            HydronicBalancing::NoneOrUnknown,
            EmissionControl::MainRoomThermostat,
        );
        assert!((temperature_increment_k(&radiators) - 3.55).abs() < 1e-12);
        let floor = input(
            EmissionSystem::FloorHeating,
            HydronicBalancing::Dynamic,
            EmissionControl::IndividualRoomThermostats,
        );
        assert!((temperature_increment_k(&floor) - 2.0).abs() < 1e-12);
        let air = input(
            EmissionSystem::AirHeating,
            HydronicBalancing::NotApplicable,
            EmissionControl::CentralWithRoomValves,
        );
        assert!((temperature_increment_k(&air) - 2.0).abs() < 1e-12);
        assert!(balancing_consistent(&air));
        assert!(balancing_consistent(&floor));
        let wrong = input(
            EmissionSystem::AirHeating,
            HydronicBalancing::Static,
            EmissionControl::OtherOrUnknown,
        );
        assert!(!balancing_consistent(&wrong));
        let local = input(
            EmissionSystem::LocalHeater,
            HydronicBalancing::NotApplicable,
            EmissionControl::OtherOrUnknown,
        );
        assert!(balancing_consistent(&local));
        assert!((temperature_increment_k(&local) - 2.85).abs() < 1e-12);
    }

    #[test]
    fn loss_uses_ratio_and_cap() {
        // January De Bilt, floor heating with dynamic balancing and room
        // thermostats: 2,0/(22,0 − 2,61) stays below the cap.
        let loss = monthly_loss_kwh(1000.0, 20.0, 2.61, 2.0);
        assert!((loss - 1000.0 * 2.0 / (22.0 - 2.61)).abs() < 1e-9);
        // Radiators without balancing: 3,55/(23,55 − 2,61) = 0,17 → capped.
        assert!((monthly_loss_kwh(1000.0, 20.0, 2.61, 3.55) - 150.0).abs() < 1e-9);
        // Mild month hits the 0,15 cap.
        assert!((monthly_loss_kwh(100.0, 20.0, 18.0, 3.55) - 15.0).abs() < 1e-12);
        // Outdoor at or above setpoint: no loss.
        assert_eq!(monthly_loss_kwh(100.0, 20.0, 20.0, 3.55), 0.0);
    }
}
