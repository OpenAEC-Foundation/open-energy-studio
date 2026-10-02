//! Heat pumps for space heating from measured product data, NTA 8800:2025+C1:2026
//! annex Q (pp. 1027–1069).
//!
//! The annual generation efficiency `η_H;gen;hp` (Q.2) and energy fraction
//! `F_H;gen` (Q.1) follow from a bin calculation over the outdoor
//! temperatures of table Q.6. Per bin the demanded power (Q.46) is compared
//! with the maximum (Q.37) or part-load (Q.27) condenser power, whose COP
//! comes from planes fitted through the NEN-EN 14511 / 14825 measurements
//! (Q.49–Q.83). Switch-off criteria (tables Q.1/Q.3), the outdoor-air
//! defrost correction (Q.17–Q.19, Q.32–Q.34) and the source pump (Q.5–Q.8)
//! are included. Q.84–Q.95 give the operating-time fractions and the
//! ventilation-air flow of exhaust-air heat pumps for chapter 11.
//!
//! Values and formulas are transcribed from the licensed norm; the norm text
//! itself is not part of this repository.

use serde::{Deserialize, Serialize};

/// Table Q.6 outdoor temperatures (bins), °C.
pub const BIN_TEMPERATURES_C: [f64; 27] = [
    16.0, 15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0,
    -1.0, -2.0, -3.0, -4.0, -5.0, -6.0, -7.0, -8.0, -9.0, -10.0,
];

/// Table Q.6 `t_H;hp;an;θi`, h.
pub const BIN_HOURS: [f64; 27] = [
    215.0, 207.0, 220.0, 303.0, 372.0, 367.0, 424.0, 376.0, 484.0, 456.0, 358.0, 399.0, 324.0,
    304.0, 236.0, 301.0, 183.0, 142.0, 95.0, 49.0, 45.0, 29.0, 11.0, 14.0, 5.0, 4.0, 0.0,
];

/// Table Q.6 `f_Q;H;θi` for WLE, WHE, ULE and UHE.
const DEMAND_FACTORS: [[f64; 27]; 4] = [
    [
        0.0, 0.0, 0.0, 0.0, 0.0078, 0.0157, 0.0235, 0.0313, 0.0392, 0.0470, 0.0549, 0.0627, 0.0705,
        0.0784, 0.0862, 0.0940, 0.1019, 0.1097, 0.1176, 0.1254, 0.1332, 0.1411, 0.1489, 0.1567,
        0.1646, 0.1724, 0.0,
    ],
    [
        0.0, 0.0053, 0.0107, 0.0160, 0.0214, 0.0267, 0.0321, 0.0374, 0.0428, 0.0481, 0.0535,
        0.0588, 0.0642, 0.0695, 0.0749, 0.0802, 0.0856, 0.0909, 0.0963, 0.1016, 0.1070, 0.1123,
        0.1177, 0.1230, 0.1284, 0.1337, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0128, 0.0257, 0.0385, 0.0514, 0.0642, 0.0771, 0.0899,
        0.1028, 0.1156, 0.1285, 0.1413, 0.1542, 0.1670, 0.1799, 0.1927, 0.2056, 0.2184, 0.2312,
        0.2441, 0.0,
    ],
    [
        0.0, 0.0, 0.0, 0.0068, 0.0136, 0.0205, 0.0273, 0.0341, 0.0409, 0.0477, 0.0546, 0.0614,
        0.0682, 0.0750, 0.0818, 0.0887, 0.0955, 0.1023, 0.1091, 0.1160, 0.1228, 0.1296, 0.1364,
        0.1432, 0.1501, 0.1569, 0.0,
    ],
];

/// Table Q.4: forfait evaporator inlet of brine/water heat pumps on ground
/// heat exchangers, per bin, °C.
const GROUND_EVAPORATOR_INLET_C: [f64; 27] = [
    7.4, 7.0, 6.9, 6.7, 5.9, 5.3, 4.6, 3.8, 3.4, 2.9, 2.6, 2.5, 2.1, 2.2, 2.0, 1.9, 1.6, 1.5, 1.5,
    2.0, 2.0, 1.1, 1.4, 1.1, 1.0, 1.0, 1.0,
];

/// Table Q.5 condenser inlet temperature per bin (rows) and design supply
/// temperature class (columns ≤30, ≤35, ≤40, ≤45, ≤50, ≤55, ≤65, ≤75 °C).
const CONDENSER_INLET_C: [[f64; 8]; 27] = [
    [20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0],
    [20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 21.3, 21.7],
    [20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 22.7, 23.5],
    [20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 24.0, 25.2],
    [20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 25.4, 26.9],
    [20.3, 20.5, 20.7, 20.9, 21.0, 21.2, 26.7, 28.7],
    [20.6, 20.9, 21.4, 21.8, 22.0, 22.5, 28.1, 30.4],
    [21.0, 21.4, 22.0, 22.7, 23.0, 23.7, 29.4, 32.1],
    [21.3, 21.8, 22.7, 23.6, 24.0, 24.9, 30.8, 33.8],
    [21.6, 22.3, 23.4, 24.5, 25.0, 26.1, 32.1, 35.6],
    [21.9, 22.7, 24.1, 25.5, 26.0, 27.4, 33.5, 37.3],
    [22.2, 23.2, 24.8, 26.4, 27.0, 28.6, 34.8, 39.0],
    [22.5, 23.6, 25.5, 27.3, 28.0, 29.8, 36.2, 40.8],
    [22.9, 24.1, 26.1, 28.2, 29.0, 31.0, 37.5, 42.5],
    [23.2, 24.5, 26.8, 29.1, 30.0, 32.3, 38.8, 44.2],
    [23.5, 25.0, 27.5, 30.0, 31.0, 33.5, 40.2, 46.0],
    [23.8, 25.5, 28.2, 30.9, 32.0, 34.7, 41.5, 47.7],
    [24.1, 25.9, 28.9, 31.8, 33.0, 36.0, 42.9, 49.4],
    [24.5, 26.4, 29.5, 32.7, 34.0, 37.2, 44.2, 51.2],
    [24.8, 26.8, 30.2, 33.6, 35.0, 38.4, 45.6, 52.9],
    [25.1, 27.3, 30.9, 34.5, 36.0, 39.6, 46.9, 54.6],
    [25.4, 27.7, 31.6, 35.5, 37.0, 40.9, 48.3, 56.3],
    [25.7, 28.2, 32.3, 36.4, 38.0, 42.1, 49.6, 58.1],
    [26.0, 28.6, 33.0, 37.3, 39.0, 43.3, 51.0, 59.8],
    [26.4, 29.1, 33.6, 38.2, 40.0, 44.5, 52.3, 61.5],
    [26.7, 29.5, 34.3, 39.1, 41.0, 45.8, 53.7, 63.3],
    [27.0, 30.0, 35.0, 40.0, 42.0, 47.0, 55.0, 65.0],
];

/// Table Q.7 `θ_cond;nom` and `Δθ_cond` per design supply class.
const DESIGN_CONDENSER: [(f64, f64); 8] = [
    (28.0, 3.0),
    (31.7, 5.0),
    (36.7, 5.0),
    (41.7, 5.0),
    (44.7, 8.0),
    (49.7, 8.0),
    (58.3, 10.0),
    (68.3, 10.0),
];

/// Table Q.16 `B_pl;plj` for plj = 0…4.
pub const PART_LOAD_RATIOS: [f64; 5] = [1.00, 0.88, 0.54, 0.35, 0.15];

/// Q.8: mean on-time per start, s.
const MEAN_ON_TIME_S: f64 = 1800.0;
/// Q.24/Q.36: ρ·c of water, kJ/(m³·K).
const WATER_HEAT_CAPACITY: f64 = 4182.0;
/// Q.88 minimum and specific exhaust-air flow, dm³/s.
const EXHAUST_FLOW_MIN_DM3_S: f64 = 44.0;
const EXHAUST_FLOW_PER_M2: f64 = 0.44;

/// Table Q.17 `f_t;θi;mi`: share of each bin per month.
const BIN_MONTH_FRACTIONS: [[f64; 12]; 27] = [
    [
        0.0, 0.0164, 0.0013, 0.0153, 0.0578, 0.1028, 0.1237, 0.0712, 0.1208, 0.0605, 0.0, 0.0,
    ],
    [
        0.0, 0.0134, 0.0040, 0.0250, 0.0968, 0.0972, 0.1035, 0.0632, 0.1361, 0.0753, 0.0, 0.0,
    ],
    [
        0.0, 0.0223, 0.0108, 0.0264, 0.0847, 0.0917, 0.1035, 0.0672, 0.1278, 0.1129, 0.0042, 0.0,
    ],
    [
        0.0, 0.0119, 0.0134, 0.0528, 0.1102, 0.0694, 0.0632, 0.0605, 0.1069, 0.1035, 0.0181, 0.0013,
    ],
    [
        0.0, 0.0432, 0.0175, 0.0833, 0.1277, 0.0972, 0.0349, 0.0390, 0.0917, 0.1129, 0.0403, 0.0255,
    ],
    [
        0.0081, 0.0625, 0.0349, 0.0958, 0.0954, 0.0431, 0.0202, 0.0390, 0.0667, 0.1062, 0.1028,
        0.0188,
    ],
    [
        0.0296, 0.0357, 0.0887, 0.0875, 0.0860, 0.0194, 0.0081, 0.0242, 0.0472, 0.0874, 0.1153,
        0.0228,
    ],
    [
        0.0538, 0.0223, 0.1465, 0.1083, 0.0591, 0.0194, 0.0067, 0.0121, 0.0167, 0.0739, 0.1181,
        0.0497,
    ],
    [
        0.0712, 0.0595, 0.1008, 0.1569, 0.0296, 0.0208, 0.0027, 0.0081, 0.0069, 0.0699, 0.1250,
        0.0699,
    ],
    [
        0.1290, 0.0744, 0.1102, 0.0597, 0.0215, 0.0139, 0.0013, 0.0027, 0.0139, 0.0591, 0.0861,
        0.0766,
    ],
    [
        0.0726, 0.0655, 0.1331, 0.0431, 0.0134, 0.0153, 0.0, 0.0, 0.0, 0.0309, 0.0778, 0.0887,
    ],
    [
        0.0672, 0.0833, 0.1492, 0.0361, 0.0134, 0.0014, 0.0, 0.0, 0.0028, 0.0296, 0.1014, 0.1398,
    ],
    [
        0.0941, 0.0744, 0.0833, 0.0333, 0.0040, 0.0, 0.0, 0.0, 0.0, 0.0148, 0.0611, 0.0914,
    ],
    [
        0.0538, 0.1116, 0.0484, 0.0361, 0.0013, 0.0, 0.0, 0.0, 0.0, 0.0108, 0.0597, 0.0739,
    ],
    [
        0.0444, 0.0551, 0.0336, 0.0278, 0.0027, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0222, 0.0887,
    ],
    [
        0.0444, 0.0744, 0.0188, 0.0278, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0139, 0.1022,
    ],
    [
        0.0511, 0.0640, 0.0027, 0.0069, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0222, 0.0363,
    ],
    [
        0.0578, 0.0432, 0.0027, 0.0028, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0125, 0.0323,
    ],
    [
        0.0457, 0.0476, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0111, 0.0296,
    ],
    [
        0.0175, 0.0030, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0042, 0.0121,
    ],
    [
        0.0296, 0.0060, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0042, 0.0255,
    ],
    [
        0.0444, 0.0045, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0054,
    ],
    [
        0.0215, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0054,
    ],
    [
        0.0242, 0.0060, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0040,
    ],
    [
        0.0134, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0161, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
    [
        0.0108, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ],
];

pub const INTERPRETATIONS: &[&str] = &[
    "Q.48: θ_evap;nom of −10 °C is used for air/water heat pumps (the text lists L/L, which has no water flow)",
    "air/air heat pumps take a condenser inlet of 20 °C (the indoor test condition of table Q.10)",
    "interval int = 5 (B < 0,15) uses the 15 % part-load point (plj = 4); the formula refers to plj = 5, which is not measured",
    "the annual energy fraction F_H;gen is applied to every month",
    "switch-off criteria that are not given are not applied (note 1 of tables Q.1/Q.3)",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnexQSource {
    BrineWater,
    WaterWater,
    /// Air/water on outdoor air (also with exhaust air passed through a WTW
    /// first, Q.42).
    OutdoorAirWater,
    /// Air/water on exhaust (ventilation) air only (Q.41).
    ExhaustAirWater,
    /// Air/water on outdoor and exhaust air mixed (Q.43).
    CombinedAirWater,
    AirAir,
}

impl AnnexQSource {
    fn outdoor_air_cop_correction(self) -> bool {
        matches!(
            self,
            Self::OutdoorAirWater | Self::CombinedAirWater | Self::AirAir
        )
    }
    fn water_condenser(self) -> bool {
        !matches!(self, Self::AirAir)
    }
}

/// One NEN-EN 14511 / 14825 measurement (tables Q.11 and Q.15).
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeasuredPoint {
    pub evaporator_in_c: f64,
    pub evaporator_out_c: f64,
    pub condenser_in_c: f64,
    pub condenser_out_c: f64,
    pub cop: f64,
    pub heating_power_kw: f64,
}

impl MeasuredPoint {
    /// Q.69/Q.79.
    fn evaporator_c(&self) -> f64 {
        (self.evaporator_in_c + self.evaporator_out_c) / 2.0
    }
    /// Q.70/Q.80.
    fn condenser_c(&self) -> f64 {
        (2.0 * self.condenser_in_c + self.condenser_out_c) / 3.0
    }
    fn valid(&self) -> bool {
        [
            self.evaporator_in_c,
            self.evaporator_out_c,
            self.condenser_in_c,
            self.condenser_out_c,
        ]
        .iter()
        .all(|value| value.is_finite())
            && self.cop.is_finite()
            && self.cop > 0.0
            && self.heating_power_kw.is_finite()
            && self.heating_power_kw > 0.0
    }
}

/// Table Q.11: conditions 1–4 at maximum power.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaximumPowerTests {
    pub condition1: MeasuredPoint,
    #[serde(default)]
    pub condition2: Option<MeasuredPoint>,
    #[serde(default)]
    pub condition3: Option<MeasuredPoint>,
    /// Defrost dip (2/1 °C); optional for outdoor-air units.
    #[serde(default)]
    pub condition4: Option<MeasuredPoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Modulation {
    OnOff,
    Modulating {
        /// `P_H;hp;min`, kW.
        #[serde(rename = "minimumPowerKw")]
        minimum_power_kw: f64,
        /// Table Q.15, LT range: 100/88/54/35/15 %.
        #[serde(rename = "lowRange")]
        low_range: Vec<MeasuredPoint>,
        /// Table Q.15, IT/MT/HT range; not used for air/air.
        #[serde(default, rename = "highRange")]
        high_range: Option<Vec<MeasuredPoint>>,
        #[serde(rename = "condenserPumpModulating")]
        condenser_pump_modulating: bool,
        #[serde(rename = "sourcePumpModulating")]
        source_pump_modulating: bool,
    },
}

/// Tables Q.1/Q.3; an absent value disables the criterion.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SwitchOffCriteria {
    #[serde(default)]
    pub min_evaporator_in_c: Option<f64>,
    #[serde(default)]
    pub min_evaporator_out_c: Option<f64>,
    #[serde(default)]
    pub max_condenser_in_c: Option<f64>,
    #[serde(default)]
    pub max_condenser_out_c: Option<f64>,
    /// At least 1 (Q.4.4).
    #[serde(default)]
    pub min_cop: Option<f64>,
}

/// Q.4.4 source pump of brine/water and water/water units.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourcePump {
    /// `P_H;aux;ev;nom`, W; absent for water/water means 50 W per kW of
    /// evaporator power at condition 1.
    #[serde(default)]
    pub nominal_power_w: Option<f64>,
    /// `t_ev;xt`, s.
    pub overrun_s: f64,
}

/// Q.2.14.2: evaporator inlet of ground and water sources.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum EvaporatorInlet {
    /// Table Q.4 (brine/water) or 10 °C (water/water).
    Forfait,
    /// History (≥ 3 years) or a dynamic ground calculation (≥ 25 years):
    /// one value, or one per bin of table Q.6.
    Declared {
        #[serde(rename = "temperaturesC")]
        temperatures_c: Vec<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnnexQHeatPump {
    pub source: AnnexQSource,
    /// `f_buitenlucht` (11.24) for a combined outdoor/exhaust-air source.
    #[serde(default)]
    pub outdoor_air_fraction: Option<f64>,
    pub maximum_power: MaximumPowerTests,
    pub modulation: Modulation,
    #[serde(default)]
    pub switch_off: SwitchOffCriteria,
    #[serde(default)]
    pub source_pump: Option<SourcePump>,
    #[serde(default)]
    pub evaporator_inlet: Option<EvaporatorInlet>,
    pub test_report_reference: String,
}

/// Table Q.6 column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DemandClass {
    ResidentialLow,
    ResidentialHigh,
    UtilityLow,
    UtilityHigh,
}

impl DemandClass {
    /// Table Q.6 notes a–d: `Q_H;nd/A_g;tot` against 41,7 or 69,4 kWh/m².
    pub fn from_need(residential: bool, need_kwh_per_m2: f64) -> Self {
        match (residential, need_kwh_per_m2) {
            (true, value) if value <= 41.7 => Self::ResidentialLow,
            (true, _) => Self::ResidentialHigh,
            (false, value) if value <= 69.4 => Self::UtilityLow,
            (false, _) => Self::UtilityHigh,
        }
    }
    fn index(self) -> usize {
        match self {
            Self::ResidentialLow => 0,
            Self::ResidentialHigh => 1,
            Self::UtilityLow => 2,
            Self::UtilityHigh => 3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AnnexQContext {
    /// `Σ_mi Q_H;node,in;mi` of the system, kWh.
    pub annual_node_input_kwh: f64,
    /// Design supply temperature `θ_sup` (tables Q.5/Q.7), °C.
    pub design_supply_temperature_c: f64,
    pub demand_class: DemandClass,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnexQIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnexQBin {
    pub outdoor_c: f64,
    pub hours: f64,
    pub demand_kw: f64,
    pub maximum_power_kw: f64,
    pub delivered_kw: f64,
    pub cop: f64,
    pub switch_off_factor: f64,
    pub on_hours: f64,
    /// `f_H;t;hp-on;θi` (Q.91/Q.93).
    pub on_fraction: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnexQResult {
    /// `F_H;gen;si,gpref` (Q.1).
    pub energy_fraction: f64,
    /// `η_H;gen;hp;si` (Q.2), source pump included.
    pub generation_efficiency: f64,
    /// `Q_H;hp;pr;an` (Q.3), kWh.
    pub delivered_kwh: f64,
    /// `Q_H;hp;us;an;el` (Q.4), kWh.
    pub electricity_kwh: f64,
    /// `W_H;aux;hp;an` (Q.5), kWh.
    pub source_pump_kwh: f64,
    pub bins: Vec<AnnexQBin>,
    /// `f_H;t;hp-on;mi` (Q.90).
    pub monthly_on_fraction: Vec<f64>,
    pub interpretations: Vec<&'static str>,
}

fn issue(code: &'static str, path: impl Into<String>) -> AnnexQIssue {
    AnnexQIssue {
        code,
        path: path.into(),
    }
}

fn design_class(supply_c: f64) -> usize {
    [30.0, 35.0, 40.0, 45.0, 50.0, 55.0, 65.0]
        .iter()
        .position(|limit| supply_c <= *limit)
        .unwrap_or(7)
}

pub fn validate_annex_q(
    pump: &AnnexQHeatPump,
    design_supply_temperature_c: f64,
    path: &str,
) -> Vec<AnnexQIssue> {
    let mut issues = Vec::new();
    let tests = &pump.maximum_power;
    let required: &[(u8, Option<&MeasuredPoint>)] = match pump.source {
        AnnexQSource::ExhaustAirWater => &[(2, tests.condition2.as_ref())],
        AnnexQSource::AirAir => &[(3, tests.condition3.as_ref())],
        _ => &[
            (2, tests.condition2.as_ref()),
            (3, tests.condition3.as_ref()),
        ],
    };
    for (number, point) in required {
        if point.is_none() {
            issues.push(issue(
                "annex_q_condition_required",
                format!("{path}.maximumPower.condition{number}"),
            ));
        }
    }
    for (number, point) in [
        (1, Some(&tests.condition1)),
        (2, tests.condition2.as_ref()),
        (3, tests.condition3.as_ref()),
        (4, tests.condition4.as_ref()),
    ] {
        if point.is_some_and(|item| !item.valid()) {
            issues.push(issue(
                "annex_q_measurement_invalid",
                format!("{path}.maximumPower.condition{number}"),
            ));
        }
    }
    if issues.is_empty() && plane(pump, |point| point.cop).is_none() {
        issues.push(issue(
            "annex_q_conditions_degenerate",
            format!("{path}.maximumPower"),
        ));
    }
    if let Modulation::Modulating {
        minimum_power_kw,
        low_range,
        high_range,
        ..
    } = &pump.modulation
    {
        if !(minimum_power_kw.is_finite() && *minimum_power_kw > 0.0) {
            issues.push(issue(
                "annex_q_minimum_power_invalid",
                format!("{path}.modulation.minimumPowerKw"),
            ));
        }
        let five =
            |range: &Vec<MeasuredPoint>| range.len() == 5 && range.iter().all(MeasuredPoint::valid);
        if !five(low_range) {
            issues.push(issue(
                "annex_q_part_load_series_invalid",
                format!("{path}.modulation.lowRange"),
            ));
        }
        match (pump.source, high_range) {
            (AnnexQSource::AirAir, _) => {}
            (_, Some(range)) if five(range) => {
                if low_range.len() == 5
                    && range
                        .iter()
                        .zip(low_range)
                        .any(|(high, low)| (high.condenser_c() - low.condenser_c()).abs() < 1e-9)
                {
                    issues.push(issue(
                        "annex_q_part_load_series_degenerate",
                        format!("{path}.modulation.highRange"),
                    ));
                }
            }
            _ => issues.push(issue(
                "annex_q_part_load_series_invalid",
                format!("{path}.modulation.highRange"),
            )),
        }
    }
    if pump
        .switch_off
        .min_cop
        .is_some_and(|value| !(value.is_finite() && value >= 1.0))
    {
        issues.push(issue(
            "annex_q_min_cop_invalid",
            format!("{path}.switchOff.minCop"),
        ));
    }
    match (pump.source, pump.outdoor_air_fraction) {
        (AnnexQSource::CombinedAirWater, Some(value)) if (0.0..=1.0).contains(&value) => {}
        (AnnexQSource::CombinedAirWater, _) => issues.push(issue(
            "annex_q_outdoor_air_fraction_required",
            format!("{path}.outdoorAirFraction"),
        )),
        (_, Some(_)) => issues.push(issue(
            "annex_q_outdoor_air_fraction_not_applicable",
            format!("{path}.outdoorAirFraction"),
        )),
        _ => {}
    }
    let ground_or_water = matches!(
        pump.source,
        AnnexQSource::BrineWater | AnnexQSource::WaterWater
    );
    match (&pump.source_pump, ground_or_water) {
        (None, true) => issues.push(issue(
            "annex_q_source_pump_required",
            format!("{path}.sourcePump"),
        )),
        (Some(source_pump), true) => {
            if !(source_pump.overrun_s.is_finite() && source_pump.overrun_s >= 0.0) {
                issues.push(issue(
                    "annex_q_source_pump_invalid",
                    format!("{path}.sourcePump.overrunS"),
                ));
            }
            match source_pump.nominal_power_w {
                Some(value) if !(value.is_finite() && value >= 0.0) => issues.push(issue(
                    "annex_q_source_pump_invalid",
                    format!("{path}.sourcePump.nominalPowerW"),
                )),
                None if pump.source == AnnexQSource::BrineWater => issues.push(issue(
                    "annex_q_source_pump_power_required",
                    format!("{path}.sourcePump.nominalPowerW"),
                )),
                _ => {}
            }
        }
        (Some(_), false) => issues.push(issue(
            "annex_q_source_pump_not_applicable",
            format!("{path}.sourcePump"),
        )),
        (None, false) => {}
    }
    match (&pump.evaporator_inlet, ground_or_water) {
        (Some(EvaporatorInlet::Declared { .. }), false) => issues.push(issue(
            "annex_q_evaporator_inlet_not_applicable",
            format!("{path}.evaporatorInlet"),
        )),
        (
            Some(EvaporatorInlet::Declared {
                temperatures_c,
                source_reference,
            }),
            true,
        ) => {
            if !(temperatures_c.len() == 1 || temperatures_c.len() == 27)
                || temperatures_c.iter().any(|value| !value.is_finite())
            {
                issues.push(issue(
                    "annex_q_evaporator_inlet_invalid",
                    format!("{path}.evaporatorInlet.temperaturesC"),
                ));
            }
            if source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{path}.evaporatorInlet.sourceReference"),
                ));
            }
        }
        _ => {}
    }
    if !(design_supply_temperature_c.is_finite()
        && design_supply_temperature_c > 0.0
        && design_supply_temperature_c <= 75.0)
    {
        issues.push(issue(
            "annex_q_design_supply_temperature_invalid",
            format!("{path}.designSupplyTemperatureC"),
        ));
    }
    if pump.test_report_reference.trim().is_empty() {
        issues.push(issue(
            "source_reference_required",
            format!("{path}.testReportReference"),
        ));
    }
    issues
}

/// Constants `(k1, k2, k3)` of `value = k1·θ_evap + k2·θ_cond + k3` at
/// maximum power (Q.51–Q.68).
fn plane(pump: &AnnexQHeatPump, value: impl Fn(&MeasuredPoint) -> f64) -> Option<[f64; 3]> {
    let tests = &pump.maximum_power;
    let p1 = &tests.condition1;
    match pump.source {
        AnnexQSource::ExhaustAirWater => {
            // Q.54–Q.56 / Q.63–Q.65.
            let p2 = tests.condition2.as_ref()?;
            let (c1, c2) = (p1.condenser_c(), p2.condenser_c());
            let denominator = c1 - c2;
            if denominator.abs() < 1e-9 {
                return None;
            }
            Some([
                0.0,
                (value(p1) - value(p2)) / denominator,
                (c1 * value(p2) - c2 * value(p1)) / denominator,
            ])
        }
        AnnexQSource::AirAir => {
            // Q.57–Q.59 / Q.66–Q.68.
            let p3 = tests.condition3.as_ref()?;
            let (e1, e3) = (p1.evaporator_c(), p3.evaporator_c());
            let denominator = e1 - e3;
            if denominator.abs() < 1e-9 {
                return None;
            }
            Some([
                (value(p1) - value(p3)) / denominator,
                0.0,
                (e1 * value(p3) - e3 * value(p1)) / denominator,
            ])
        }
        _ => {
            // Q.51–Q.53 / Q.60–Q.62.
            let p2 = tests.condition2.as_ref()?;
            let p3 = tests.condition3.as_ref()?;
            let (e1, e2, e3) = (p1.evaporator_c(), p2.evaporator_c(), p3.evaporator_c());
            let (c1, c2, c3) = (p1.condenser_c(), p2.condenser_c(), p3.condenser_c());
            let (v1, v2, v3) = (value(p1), value(p2), value(p3));
            let denominator = e1 * (c2 - c3) + e2 * (c3 - c1) + e3 * (c1 - c2);
            if denominator.abs() < 1e-9 {
                return None;
            }
            Some([
                (v1 * (c2 - c3) + v2 * (c3 - c1) + v3 * (c1 - c2)) / denominator,
                (e1 * (v2 - v3) + e2 * (v3 - v1) + e3 * (v1 - v2)) / denominator,
                (e1 * (v3 * c2 - v2 * c3) + e2 * (v1 * c3 - v3 * c1) + e3 * (v2 * c1 - v1 * c2))
                    / denominator,
            ])
        }
    }
}

fn maximum_points(pump: &AnnexQHeatPump) -> Vec<&MeasuredPoint> {
    let tests = &pump.maximum_power;
    std::iter::once(&tests.condition1)
        .chain(tests.condition2.as_ref())
        .chain(tests.condition3.as_ref())
        .chain(tests.condition4.as_ref())
        .collect()
}

/// Q.49 and Q.50: mean condenser rise and evaporator drop at maximum power.
fn mean_spreads(pump: &AnnexQHeatPump) -> (f64, f64) {
    let points = maximum_points(pump);
    let n = points.len() as f64;
    let condenser = points
        .iter()
        .map(|point| point.condenser_out_c - point.condenser_in_c)
        .sum::<f64>()
        / n;
    let evaporator = points
        .iter()
        .map(|point| point.evaporator_in_c - point.evaporator_out_c)
        .sum::<f64>()
        / n;
    (condenser, evaporator)
}

/// Q.81–Q.83.
fn maximum_defrost_correction(pump: &AnnexQHeatPump, cop: [f64; 3]) -> f64 {
    if !pump.source.outdoor_air_cop_correction() {
        return 1.0;
    }
    match &pump.maximum_power.condition4 {
        None => 0.75,
        Some(point) => {
            let model = cop[0] * point.evaporator_c() + cop[1] * point.condenser_c() + cop[2];
            if model > 0.0 {
                (point.cop / model).min(1.0)
            } else {
                0.75
            }
        }
    }
}

/// Q.17–Q.19 / Q.32–Q.34.
fn defrost_correction(pump: &AnnexQHeatPump, maximum: f64, outdoor_c: f64) -> f64 {
    if !pump.source.outdoor_air_cop_correction() || outdoor_c <= -7.0 || outdoor_c >= 7.0 {
        1.0
    } else if outdoor_c >= 2.0 {
        (1.0 - maximum) / 5.0 * outdoor_c + (7.0 * maximum - 2.0) / 5.0
    } else {
        (maximum - 1.0) / 9.0 * outdoor_c + (7.0 * maximum + 2.0) / 9.0
    }
}

struct PartLoadModel {
    /// `c_plj;i` for plj = 0…4.
    constants: [[f64; 3]; 5],
    /// `F_C;max` (air/air only).
    carnot_max: Option<f64>,
}

/// Q.71–Q.78.
fn part_load_model(
    pump: &AnnexQHeatPump,
    cop_max: [f64; 3],
    defrost_max: f64,
    low: &[MeasuredPoint],
    high: Option<&Vec<MeasuredPoint>>,
) -> PartLoadModel {
    let mut constants = [[0.0; 3]; 5];
    let mut carnot_max: Option<f64> = None;
    for plj in 0..5 {
        let correction = if plj == 2 { defrost_max } else { 1.0 };
        let t1 = &low[plj];
        if pump.source == AnnexQSource::AirAir {
            let c1 = cop_max[0];
            constants[plj] = [c1, cop_max[1], t1.cop / correction - c1 * t1.evaporator_c()];
            let carnot =
                t1.cop * (t1.condenser_c() - t1.evaporator_c()) / (273.15 + t1.condenser_c());
            carnot_max = Some(carnot_max.map_or(carnot, |value: f64| value.max(carnot)));
        } else {
            let t2 = &high.expect("validated high range")[plj];
            let (c_t1, c_t2) = (t1.condenser_c(), t2.condenser_c());
            let c1 = cop_max[0];
            let c2 = (t1.cop - t2.cop) / (c_t1 - c_t2) / correction;
            let c3 = (c_t1 * t2.cop - c_t2 * t1.cop) / (c_t1 - c_t2) / correction
                - c1 * t1.evaporator_c();
            constants[plj] = [c1, c2, c3];
        }
    }
    PartLoadModel {
        constants,
        carnot_max,
    }
}

/// Table Q.2.
fn load_interval(utilisation: f64) -> usize {
    if utilisation >= 1.0 {
        0
    } else if utilisation >= 0.88 {
        1
    } else if utilisation >= 0.54 {
        2
    } else if utilisation >= 0.35 {
        3
    } else if utilisation >= 0.15 {
        4
    } else {
        5
    }
}

struct Temperatures {
    evaporator_in: f64,
    evaporator_out: f64,
    condenser_in: f64,
    condenser_out: f64,
}

/// Tables Q.1/Q.3: product of the criteria factors.
fn switch_off_factor(criteria: &SwitchOffCriteria, temperatures: &Temperatures, cop: f64) -> f64 {
    let checks = [
        criteria
            .min_evaporator_in_c
            .map(|limit| temperatures.evaporator_in > limit),
        criteria
            .min_evaporator_out_c
            .map(|limit| temperatures.evaporator_out > limit),
        criteria
            .max_condenser_in_c
            .map(|limit| temperatures.condenser_in <= limit),
        criteria
            .max_condenser_out_c
            .map(|limit| temperatures.condenser_out <= limit),
        criteria.min_cop.map(|limit| cop >= limit),
    ];
    if checks.contains(&Some(false)) {
        0.0
    } else {
        1.0
    }
}

/// Q.41–Q.44 and table Q.4.
fn evaporator_inlet(pump: &AnnexQHeatPump, bin: usize, outdoor_c: f64) -> f64 {
    let declared = match &pump.evaporator_inlet {
        Some(EvaporatorInlet::Declared { temperatures_c, .. }) => {
            Some(if temperatures_c.len() == 1 {
                temperatures_c[0]
            } else {
                temperatures_c[bin]
            })
        }
        _ => None,
    };
    match pump.source {
        AnnexQSource::ExhaustAirWater => 20.0,
        AnnexQSource::OutdoorAirWater | AnnexQSource::AirAir => outdoor_c,
        AnnexQSource::CombinedAirWater => {
            let fraction = pump.outdoor_air_fraction.unwrap_or(0.0);
            fraction * outdoor_c + (1.0 - fraction) * 20.0
        }
        AnnexQSource::WaterWater => declared.unwrap_or(10.0),
        AnnexQSource::BrineWater => declared.unwrap_or(GROUND_EVAPORATOR_INLET_C[bin]),
    }
}

/// Annex Q for one heat pump; call after [`validate_annex_q`] returned no
/// issues.
pub fn calculate_annex_q(pump: &AnnexQHeatPump, context: AnnexQContext) -> AnnexQResult {
    let cop_max = plane(pump, |point| point.cop).expect("validated conditions");
    let power_max = plane(pump, |point| point.heating_power_kw).expect("validated conditions");
    let (spread_condenser, spread_evaporator) = mean_spreads(pump);
    let defrost_max = maximum_defrost_correction(pump, cop_max);
    let class = design_class(context.design_supply_temperature_c);
    let (design_condenser_c, design_spread) = DESIGN_CONDENSER[class];
    // Q.47/Q.48.
    let nominal_evaporator = match pump.source {
        AnnexQSource::WaterWater => 8.5,
        AnnexQSource::BrineWater => -1.5,
        _ => -10.0,
    };
    let nominal_power =
        power_max[0] * nominal_evaporator + power_max[1] * design_condenser_c + power_max[2];
    let nominal_flow = nominal_power / (design_spread * WATER_HEAT_CAPACITY);
    let part_load = match &pump.modulation {
        Modulation::Modulating {
            low_range,
            high_range,
            ..
        } => Some(part_load_model(
            pump,
            cop_max,
            defrost_max,
            low_range,
            high_range.as_ref(),
        )),
        Modulation::OnOff => None,
    };
    // Q.4.4: water/water source pump 50 W per kW evaporator power.
    let source_pump_w = pump.source_pump.as_ref().map(|source| {
        source.nominal_power_w.unwrap_or_else(|| {
            let point = &pump.maximum_power.condition1;
            50.0 * point.heating_power_kw * (1.0 - 1.0 / point.cop)
        })
    });
    let annual_mj = 3.6 * context.annual_node_input_kwh;
    let mut delivered_mj = 0.0;
    let mut electricity_mj = 0.0;
    let mut source_pump_mj = 0.0;
    let mut bins = Vec::with_capacity(27);
    for (bin, outdoor) in BIN_TEMPERATURES_C.iter().copied().enumerate() {
        let hours = BIN_HOURS[bin];
        // Q.46.
        let demand = annual_mj * DEMAND_FACTORS[context.demand_class.index()][bin] / 1000.0;
        let evaporator_in = evaporator_inlet(pump, bin, outdoor);
        let condenser_in = if pump.source.water_condenser() {
            CONDENSER_INLET_C[bin][class]
        } else {
            20.0
        };
        // Q.37–Q.40, Q.45.
        let evaporator_out_max = evaporator_in - spread_evaporator;
        let evaporator_max = (evaporator_in + evaporator_out_max) / 2.0;
        let condenser_estimate = condenser_in + spread_condenser / 3.0;
        let maximum =
            (power_max[0] * evaporator_max + power_max[1] * condenser_estimate + power_max[2])
                .max(0.0);
        let correction = defrost_correction(pump, defrost_max, outdoor);
        let (delivered, cop, factor, on_hours, on_fraction, operating_power) =
            match (&pump.modulation, &part_load) {
                (
                    Modulation::Modulating {
                        minimum_power_kw,
                        condenser_pump_modulating,
                        source_pump_modulating,
                        ..
                    },
                    Some(model),
                ) => {
                    // Q.27/Q.28.
                    let operating = minimum_power_kw.max(maximum.min(demand));
                    let utilisation = if maximum > 0.0 {
                        minimum_power_kw.max(demand) / maximum
                    } else {
                        0.0
                    };
                    let ratio = if maximum > 0.0 {
                        operating / maximum
                    } else {
                        0.0
                    };
                    // Q.21/Q.22.
                    let evaporator_out = if *source_pump_modulating {
                        evaporator_in - spread_evaporator
                    } else {
                        evaporator_in - ratio * spread_evaporator
                    };
                    let evaporator = (evaporator_in + evaporator_out) / 2.0;
                    // Q.24–Q.26, Q.24A.
                    let condenser_out = if pump.source.water_condenser() {
                        let flow = if *condenser_pump_modulating {
                            nominal_flow * ratio
                        } else {
                            nominal_flow
                        };
                        condenser_in + operating / (flow * WATER_HEAT_CAPACITY)
                    } else {
                        condenser_in + spread_condenser
                    };
                    let condenser = (2.0 * condenser_in + condenser_out) / 3.0;
                    let at = |plj: usize| {
                        let [c1, c2, c3] = model.constants[plj];
                        correction * (c1 * evaporator + c2 * condenser + c3)
                    };
                    // Q.13–Q.16.
                    let interval = load_interval(utilisation);
                    let mut cop = match interval {
                        0 => at(0),
                        5 => at(4),
                        int => {
                            let (upper, lower) = (at(int - 1), at(int));
                            lower
                                + (upper - lower) * (utilisation - PART_LOAD_RATIOS[int])
                                    / (PART_LOAD_RATIOS[int - 1] - PART_LOAD_RATIOS[int])
                        }
                    };
                    // Q.12A/Q.12B: air/air limited by the Carnot factor.
                    if let Some(carnot_max) = model.carnot_max {
                        let lift = condenser - evaporator;
                        if lift > 0.0 && cop * lift / (273.15 + condenser) > carnot_max {
                            cop = carnot_max * (273.15 + condenser) / lift;
                        }
                    }
                    let factor = switch_off_factor(
                        &pump.switch_off,
                        &Temperatures {
                            evaporator_in,
                            evaporator_out,
                            condenser_in,
                            condenser_out,
                        },
                        cop,
                    );
                    // Q.11, Q.10, Q.93.
                    let delivered = factor * operating.min(demand);
                    let on_hours = if operating > 0.0 {
                        hours * delivered / operating
                    } else {
                        0.0
                    };
                    let on_fraction = factor * (demand / minimum_power_kw).min(1.0);
                    (delivered, cop, factor, on_hours, on_fraction, operating)
                }
                _ => {
                    // Q.35/Q.36.
                    let condenser_out = if pump.source.water_condenser() {
                        condenser_in + maximum / (nominal_flow * WATER_HEAT_CAPACITY)
                    } else {
                        condenser_in + spread_condenser
                    };
                    let condenser = (2.0 * condenser_in + condenser_out) / 3.0;
                    // Q.31.
                    let cop = correction
                        * (cop_max[0] * evaporator_max + cop_max[1] * condenser + cop_max[2]);
                    let factor = switch_off_factor(
                        &pump.switch_off,
                        &Temperatures {
                            evaporator_in,
                            evaporator_out: evaporator_out_max,
                            condenser_in,
                            condenser_out,
                        },
                        cop,
                    );
                    // Q.29, Q.9, Q.91.
                    let delivered = factor * maximum.min(demand);
                    let on_hours = if maximum > 0.0 {
                        hours * delivered / maximum
                    } else {
                        0.0
                    };
                    let on_fraction = if maximum > 0.0 {
                        factor * (demand / maximum).min(1.0)
                    } else {
                        0.0
                    };
                    (delivered, cop, factor, on_hours, on_fraction, maximum)
                }
            };
        // Q.3/Q.4.
        delivered_mj += 3.6 * hours * delivered;
        if delivered > 0.0 && cop > 0.0 {
            electricity_mj += 3.6 * hours * delivered / cop;
        }
        // Q.5–Q.8.
        if let (Some(nominal), Some(source)) = (source_pump_w, &pump.source_pump) {
            let modulating_pump = matches!(
                pump.modulation,
                Modulation::Modulating {
                    source_pump_modulating: true,
                    ..
                }
            );
            let power = if modulating_pump && maximum > 0.0 {
                operating_power / maximum * nominal
            } else {
                nominal
            };
            let seconds = 3600.0 * on_hours * (1.0 + source.overrun_s / MEAN_ON_TIME_S);
            source_pump_mj += seconds * power / 1e6;
        }
        bins.push(AnnexQBin {
            outdoor_c: outdoor,
            hours,
            demand_kw: demand,
            maximum_power_kw: maximum,
            delivered_kw: delivered,
            cop,
            switch_off_factor: factor,
            on_hours,
            on_fraction,
        });
    }
    electricity_mj += source_pump_mj;
    let monthly_on_fraction = (0..12)
        .map(|month| {
            bins.iter()
                .enumerate()
                .map(|(bin, item)| item.on_fraction * BIN_MONTH_FRACTIONS[bin][month])
                .sum()
        })
        .collect();
    AnnexQResult {
        energy_fraction: if annual_mj > 0.0 {
            (delivered_mj / annual_mj).min(1.0)
        } else {
            1.0
        },
        generation_efficiency: if electricity_mj > 0.0 {
            delivered_mj / electricity_mj
        } else {
            0.0
        },
        delivered_kwh: delivered_mj / 3.6,
        electricity_kwh: electricity_mj / 3.6,
        source_pump_kwh: source_pump_mj / 3.6,
        bins,
        monthly_on_fraction,
        interpretations: INTERPRETATIONS.to_vec(),
    }
}

/// Q.86/Q.87/Q.88: time fraction for hot water of an exhaust-air heat pump,
/// with `Q_W` the monthly hot-water energy to distribution (kWh), `F_W` the
/// heat-pump share and `η_W;gen` its generation efficiency.
pub fn hot_water_on_fraction(
    hot_water_kwh: f64,
    heat_pump_share: f64,
    generation_efficiency: f64,
    month_hours: f64,
    source_power_kw: f64,
) -> f64 {
    if source_power_kw <= 0.0 || generation_efficiency <= 0.0 {
        return 0.0;
    }
    // Q.86 in MJ and Ms: t_mi in Ms = h·0,0036.
    let mj = 3.6 * hot_water_kwh;
    mj * heat_pump_share * (generation_efficiency - 1.0) * 1.2
        / (month_hours * 0.0036 * source_power_kw * 1000.0 * generation_efficiency)
}

/// Q.87/Q.88: maximum source power for hot water, kW, from the evaporator
/// air flow (dm³/s; `None` gives 0,44·A_g;tot with at least 44 dm³/s) and
/// the evaporator outlet (default 3 °C).
pub fn exhaust_air_source_power_kw(
    air_flow_dm3_s: Option<f64>,
    usable_floor_area_m2: f64,
    evaporator_out_c: Option<f64>,
) -> f64 {
    let flow = air_flow_dm3_s.unwrap_or_else(|| {
        (EXHAUST_FLOW_PER_M2 * usable_floor_area_m2).max(EXHAUST_FLOW_MIN_DM3_S)
    });
    1.2 * 1000.0 * flow * (20.0 - evaporator_out_c.unwrap_or(3.0)) * 1e-6
}

/// Q.84: total time fraction.
pub fn total_on_fraction(hot_water: f64, heating: f64) -> f64 {
    (hot_water + heating).min(1.0)
}

/// Q.94/Q.95: ventilation air used by the heat pump, dm³/s.
pub fn exhaust_air_used_dm3_s(total_air_flow_dm3_s: f64, outdoor_air_fraction: f64) -> f64 {
    total_air_flow_dm3_s * (1.0 - outdoor_air_fraction)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(
        evap_in: f64,
        evap_out: f64,
        cond_in: f64,
        cond_out: f64,
        cop: f64,
        power: f64,
    ) -> MeasuredPoint {
        MeasuredPoint {
            evaporator_in_c: evap_in,
            evaporator_out_c: evap_out,
            condenser_in_c: cond_in,
            condenser_out_c: cond_out,
            cop,
            heating_power_kw: power,
        }
    }

    fn brine_water() -> AnnexQHeatPump {
        AnnexQHeatPump {
            source: AnnexQSource::BrineWater,
            outdoor_air_fraction: None,
            maximum_power: MaximumPowerTests {
                // Table Q.8 B0/W35, B0/W45, B5/W35.
                condition1: point(0.0, -3.0, 30.0, 35.0, 4.5, 8.0),
                condition2: Some(point(0.0, -3.0, 40.0, 45.0, 3.5, 7.6)),
                condition3: Some(point(5.0, 2.0, 30.0, 35.0, 5.2, 9.2)),
                condition4: None,
            },
            modulation: Modulation::OnOff,
            switch_off: SwitchOffCriteria::default(),
            source_pump: Some(SourcePump {
                nominal_power_w: Some(100.0),
                overrun_s: 180.0,
            }),
            evaporator_inlet: Some(EvaporatorInlet::Forfait),
            test_report_reference: "synthetic test report".into(),
        }
    }

    fn context() -> AnnexQContext {
        AnnexQContext {
            annual_node_input_kwh: 6000.0,
            design_supply_temperature_c: 35.0,
            demand_class: DemandClass::ResidentialLow,
        }
    }

    #[test]
    fn plane_reproduces_the_three_measurements() {
        let pump = brine_water();
        assert!(validate_annex_q(&pump, 35.0, "p").is_empty());
        let cop = plane(&pump, |item| item.cop).unwrap();
        for item in maximum_points(&pump) {
            let model = cop[0] * item.evaporator_c() + cop[1] * item.condenser_c() + cop[2];
            assert!((model - item.cop).abs() < 1e-9);
        }
        // Q.49/Q.50: all three spreads are 5 K and 3 K.
        assert_eq!(mean_spreads(&pump), (5.0, 3.0));
    }

    #[test]
    fn on_off_bin_follows_q29_q31_and_q36() {
        let pump = brine_water();
        let result = calculate_annex_q(&pump, context());
        let cop = plane(&pump, |item| item.cop).unwrap();
        let power = plane(&pump, |item| item.heating_power_kw).unwrap();
        // Bin −5 °C (index 21): evaporator inlet 1,1 °C (table Q.4),
        // condenser inlet 27,7 °C (table Q.5, ≤ 35 °C).
        let bin = &result.bins[21];
        let evaporator = (1.1 + (1.1 - 3.0)) / 2.0;
        let maximum = power[0] * evaporator + power[1] * (27.7 + 5.0 / 3.0) + power[2];
        assert!((bin.maximum_power_kw - maximum).abs() < 1e-9);
        let nominal = power[0] * -1.5 + power[1] * 31.7 + power[2];
        let flow = nominal / (5.0 * 4182.0);
        let condenser = (2.0 * 27.7 + 27.7 + maximum / (flow * 4182.0)) / 3.0;
        let expected_cop = cop[0] * evaporator + cop[1] * condenser + cop[2];
        assert!((bin.cop - expected_cop).abs() < 1e-9);
        let demand = 3.6 * 6000.0 * 0.1411 / 1000.0;
        assert!((bin.demand_kw - demand).abs() < 1e-12);
        assert!((bin.delivered_kw - demand.min(maximum)).abs() < 1e-12);
        // Q.8: source pump time with 180 s overrun.
        let on = 29.0 * bin.delivered_kw / maximum;
        assert!((bin.on_hours - on).abs() < 1e-9);
        // The whole demand is covered: F = 1 within the table rounding.
        assert!((result.energy_fraction - 1.0).abs() < 1e-3);
        let expected_efficiency = result.delivered_kwh / result.electricity_kwh;
        assert!((result.generation_efficiency - expected_efficiency).abs() < 1e-12);
        assert!(result.generation_efficiency > 3.0 && result.generation_efficiency < 6.0);
        assert!(result.source_pump_kwh > 0.0);
    }

    #[test]
    fn switch_off_reduces_the_energy_fraction() {
        let mut pump = brine_water();
        pump.switch_off.min_evaporator_in_c = Some(2.0);
        let result = calculate_annex_q(&pump, context());
        assert!(result.energy_fraction < 0.9);
        let off = result
            .bins
            .iter()
            .find(|item| item.outdoor_c == -5.0)
            .unwrap();
        assert_eq!(off.switch_off_factor, 0.0);
        assert_eq!(off.delivered_kw, 0.0);
    }

    #[test]
    fn defrost_correction_matches_q32_to_q34() {
        let pump = AnnexQHeatPump {
            source: AnnexQSource::OutdoorAirWater,
            ..brine_water()
        };
        assert_eq!(defrost_correction(&pump, 0.75, 7.0), 1.0);
        assert!((defrost_correction(&pump, 0.75, 2.0) - 0.75).abs() < 1e-12);
        assert!((defrost_correction(&pump, 0.75, -7.0 + 1e-12) - 1.0).abs() < 1e-9);
        assert!(
            (defrost_correction(&pump, 0.75, 4.5) - (0.25 / 5.0 * 4.5 + 3.25 / 5.0)).abs() < 1e-12
        );
        assert_eq!(defrost_correction(&brine_water(), 0.75, 2.0), 1.0);
    }

    #[test]
    fn modulating_cop_interpolates_between_part_load_points() {
        let mut pump = brine_water();
        let series = |cond_out: [f64; 5], cop: [f64; 5]| -> Vec<MeasuredPoint> {
            (0..5)
                .map(|index| {
                    point(
                        0.0,
                        -3.0,
                        cond_out[index] - 5.0,
                        cond_out[index],
                        cop[index],
                        8.0 * PART_LOAD_RATIOS[index],
                    )
                })
                .collect()
        };
        pump.modulation = Modulation::Modulating {
            minimum_power_kw: 2.0,
            low_range: series([35.0, 34.0, 30.0, 27.0, 24.0], [4.5, 4.7, 5.2, 5.4, 5.0]),
            high_range: Some(series(
                [55.0, 52.0, 42.0, 36.0, 30.0],
                [3.0, 3.2, 3.9, 4.3, 4.1],
            )),
            condenser_pump_modulating: false,
            source_pump_modulating: false,
        };
        assert!(validate_annex_q(&pump, 35.0, "p").is_empty());
        let cop_max = plane(&pump, |item| item.cop).unwrap();
        let Modulation::Modulating {
            low_range,
            high_range,
            ..
        } = &pump.modulation
        else {
            unreachable!()
        };
        let model = part_load_model(&pump, cop_max, 1.0, low_range, high_range.as_ref());
        // Q.72 at plj = 0: (4,5 − 3,0)/(θ_t1 − θ_t2).
        let (t1, t2) = (
            low_range[0].condenser_c(),
            high_range.as_ref().unwrap()[0].condenser_c(),
        );
        assert!((model.constants[0][1] - (4.5 - 3.0) / (t1 - t2)).abs() < 1e-12);
        let result = calculate_annex_q(&pump, context());
        assert!(result.generation_efficiency > 0.0);
        // A bin with B between 0,54 and 0,88 lies between the plj 1 and 2 COPs.
        assert_eq!(load_interval(0.6), 2);
        assert_eq!(load_interval(0.1), 5);
    }

    #[test]
    fn exhaust_air_helpers_follow_q84_to_q95() {
        // Q.88: 100 m² gives 44 dm³/s; Q.87 with 20 → 3 °C.
        let power = exhaust_air_source_power_kw(None, 100.0, None);
        assert!((power - 1.2 * 1000.0 * 44.0 * 17.0 * 1e-6).abs() < 1e-12);
        assert_eq!(total_on_fraction(0.7, 0.6), 1.0);
        assert!((exhaust_air_used_dm3_s(50.0, 0.3) - 35.0).abs() < 1e-12);
        let fraction = hot_water_on_fraction(200.0, 1.0, 2.5, 744.0, power);
        let expected = 720.0 * 1.5 * 1.2 / (744.0 * 0.0036 * power * 1000.0 * 2.5);
        assert!((fraction - expected).abs() < 1e-12);
    }

    #[test]
    fn validation_reports_missing_conditions() {
        let mut pump = brine_water();
        pump.maximum_power.condition3 = None;
        pump.source_pump = None;
        let codes: Vec<_> = validate_annex_q(&pump, 80.0, "p")
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "annex_q_condition_required",
            "annex_q_source_pump_required",
            "annex_q_design_supply_temperature_invalid",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    #[test]
    fn demand_class_follows_table_q6_notes() {
        assert_eq!(
            DemandClass::from_need(true, 41.7),
            DemandClass::ResidentialLow
        );
        assert_eq!(
            DemandClass::from_need(true, 41.8),
            DemandClass::ResidentialHigh
        );
        assert_eq!(
            DemandClass::from_need(false, 69.5),
            DemandClass::UtilityHigh
        );
        let sum: f64 = BIN_HOURS
            .iter()
            .zip(DEMAND_FACTORS[0])
            .map(|(hours, factor)| hours * factor)
            .sum();
        // Σ t·f_Q converts MJ to W·h: ≈ 1e6/3600.
        assert!((sum - 277.78).abs() < 0.1);
    }
}
