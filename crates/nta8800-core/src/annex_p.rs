//! External heat, hot-water and cold supply (gebiedsmaatregelen),
//! NTA 8800:2025+C1:2026 §5.8 (pp. 121–130) and annex P (pp. 932–1026).
//!
//! A collective energy system `XD` (heat `HD`, hot water `WD`, cold `CD`)
//! is valued with one primary energy factor `f_P;XD;tot`, a CO2 coefficient
//! `K_CO2;XD;tot` and a renewable factor `f_Pren;dX`. Two routes:
//!
//! - **declared**: the values of a registered quality declaration
//!   (EMG-verklaring, §5.8.0);
//! - **calculated**: the annex P method, P.7/P.9 with
//!   - the delivery `Q_XD;out;tot` supplied or summed over the plots of the
//!     area (P.72–P.83 with tables P.14/P.15 and the monthly split P.74);
//!   - the distribution efficiency P.10–P.12 from supplied flows, the
//!     forfait loss per connection (table P.0) or the pipe losses
//!     P.13–P.18 per month or per outdoor-temperature bin (tables P.1 and
//!     P.16), with buffer vessels per P.43/P.44;
//!   - `η_WD;gen;sto` supplied, forfait (P.6.6.4.3) or calculated from the
//!     vessels, charging pipes and external exchanger (P.43–P.46, table P.7);
//!   - energy fractions supplied or derived: heat from β (P.24/P.25, table
//!     P.2, cascade and P.23, β = 0,5 when unknown), hot water P.40–P.42,
//!     cold P.51–P.55 with table P.8, collective solar first (P.32/P.33);
//!   - generator factors P.19–P.22/P.36–P.39/P.47–P.50 per generator kind:
//!     declared efficiencies, table P.3/P.4 boilers, solid biomass
//!     (P.6.5.4.3), heat pumps with table P.5, CHP P.26–P.30 with table P.6,
//!     residual heat, geothermal energy, upstream systems, collective solar
//!     (13.7.2.2 with θ_0 = 70 °C), flex-mode electric generators
//!     (P.6.5.4.11 with tables 5.5/5.6), chillers and free cooling (table
//!     P.9) and sorption chillers (table P.10);
//!   - auxiliary energy supplied or calculated per P.56–P.70 (tables
//!     P.11–P.13);
//!   - the renewable factor 5.42/5.49/5.50 with 5.43–5.56;
//! - **measured**: P.6 and its CO2 counterpart.
//!
//! Supplied values always override the derived ones. P.71 (electricity
//! produced in the area with a direct physical connection) is reported by
//! [`area_electricity`]. `f_P;XD;tot` is rounded up and `f_Pren;dh`/`f_Pren;dw`
//! down to a multiple of 0,01; `f_Pren;dc` is not rounded, because 5.49
//! (p. 129) states no rounding, unlike 5.42 and 5.50.
//!
//! Readings where the printed norm is not explicit:
//! - P.62 and P.65 are applied dimensionally consistent with P.57 and P.60
//!   (`W = t_on·ΣP/1 000`, `t_on = Q·F·1,1/P` with P in kW);
//! - P.74 uses `max(0, θ_in;ref − θ_e;avg;mi)` so that summer months with a
//!   mean above 18 °C get no negative delivery;
//! - P.53 takes the efficiency of each compression chiller itself; for a
//!   gas-engine chiller the shaft power `P_in` takes the table P.9 COP,
//!   because its `η_CD;gen = COP·η_ge` relates to the fuel input;
//! - P.74 is applied only to the plots (parts) without monthly values;
//!   for heat every such part takes P.74, an annual-only sorption-cooling
//!   part included, so the supplied months of the other parts stay;
//! - P.78 (p. 1022) sums only `E_C;dc;mi`: dehumidification counts in the
//!   annual cold delivery (P.77) but not in the monthly profile used for
//!   `f_on;mi` and P.68, so the monthly values may sum below the annual;
//!   when that leaves a zero profile next to a positive annual total (only
//!   dehumidification) the months count as unknown;
//! - P.70 (p. 1016) writes `Q_CD;dis;tot;an` in the formula while its
//!   legend defines the network input `Q_XD;in;tot`; the legend is followed
//!   (the cold the chillers produce, distribution loss included);
//! - P.27 has no MAX(0): a renewable fuel in a CHP gives a negative
//!   `K_CO2;gen`, kept as printed with the warning `chp_co2_factor_negative`;
//! - P.13–P.18 for cold count heat gains in every month with a water
//!   temperature; months without cold delivery then raise
//!   `cold_network_gain_outside_cooling_months` (set those months to
//!   `null` in a monthly temperature profile);
//! - rule a) counts solid biomass as renewable (class 0); an electrode
//!   boiler in flex mode is ranked last; `priority` overrides both;
//! - P.25 takes the whole `Q_HD;in;tot`, collective solar included;
//! - the full-load deduction of P.6.6.5.2 ("aftrek van 5 %") is read as
//!   5 percentage points, as for heat; a full-load value on a preferred
//!   hot-water boiler is refused;
//! - without `networkProductionKwh` the 15 % flex cap of 5.8 is taken on
//!   the network heat of the function being calculated; f_Pren of a flex
//!   generator is clamped at 0;
//! - without monthly values the hot-water delivery is split by month
//!   length, as in P.82;
//! - the preference rules a)–e) of P.6.5.3.2 are also used for hot water;
//! - auxiliary defaults per generator kind follow P.6.8.4.3/P.6.9.4.3; kinds
//!   whose auxiliary energy is part of their factor (residual heat,
//!   geothermal energy, upstream systems, STEG/AVI) get none.

use crate::climate::{MONTH_HOURS, OUTDOOR_TEMPERATURE_C, YEAR_HOURS};
use crate::pv::{monthly_yield_kwh, validate_pv, PvSystem};
use crate::solar_thermal::{
    calculated_service, CollectorField, ServiceSettings, SolarStorage, SolarType, PUMP_HOURS_COMBI,
    PUMP_HOURS_WATER,
};
use serde::{Deserialize, Serialize};

type Months = [f64; 12];

/// Table 5.2 `f_P;del;el` and `f_P;exp;el`.
pub const F_P_EL: f64 = 1.45;
/// Table 5.3 `K_CO2;el`, kg/kWh.
pub const K_CO2_EL: f64 = 0.268;
/// Table 5.4 `f_Pren;renelect`.
pub const F_PREN_ELEC: f64 = 1.45;
/// P.6.5.4.7: specific auxiliary energy to make residual heat available.
pub const RESIDUAL_HEAT_AUX: f64 = 0.07;
/// P.29: `Δε_chp;el / ε_chp;th` for CHP with power loss.
pub const CHP_LOSS_RATIO: f64 = 0.18;
/// P.6.5.4.8: geothermal efficiency at 40 K cooling.
pub const GEOTHERMAL_EFFICIENCY_40K: f64 = 20.0;
/// P.6.4: forfait cold distribution loss share below 10 °C supply.
pub const COLD_FORFAIT_LOSS_SHARE: f64 = 0.15;
/// P.25: `P_HD;gen;ref = Q_HD;in;tot × 3,6 / 5 400`.
pub const REFERENCE_POWER_DIVISOR: f64 = 5400.0;
/// P.6.5.3.2: β of the most preferred generator when it is unknown.
pub const UNKNOWN_BETA: f64 = 0.5;
/// P.40: share of the day the preferred hot-water generators run at full load.
pub const WD_PREFERENT_DUTY: f64 = 0.60;
/// P.6.4: forfait `θ_WD;circ;mi`, °C.
pub const WD_CIRCULATION_C: f64 = 65.0;
/// Table P.1: `λ_g;j`, W/(m·K).
pub const GROUND_CONDUCTIVITY: f64 = 1.75;
/// Table P.1: `h_a;j` in an enclosed space, W/(m²·K).
pub const SURFACE_COEFFICIENT_ENCLOSED: f64 = 8.0;
/// Table P.1: `h_a;j` of all other pipes in air, W/(m²·K).
pub const SURFACE_COEFFICIENT_OTHER: f64 = 25.0;
/// P.74: `θ_in;ref`, °C.
pub const REFERENCE_INDOOR_C: f64 = 18.0;
/// P.6.5.4.11 note: forfait flex-mode hours at the installed power.
pub const FLEX_HOURS: f64 = 1500.0;
/// P.6.5.4.11 note: flex-mode heat at most 15 % of the network production.
pub const FLEX_MAX_SHARE: f64 = 0.15;
/// P.6.5.4.11: efficiency of an electrode boiler.
pub const ELECTRODE_BOILER_EFFICIENCY: f64 = 0.99;
/// P.6.5.4.3: higher/lower heating value ratio of solid biomass.
pub const BIOMASS_HHV_RATIO: f64 = 1.08;
/// P.6.5.4.3: deduction from the solid-biomass efficiency.
pub const BIOMASS_DEDUCTION: f64 = 0.05;
/// P.6.5.4.2/P.6.6.5.2: deduction from a full-load boiler efficiency.
pub const FULL_LOAD_DEDUCTION: f64 = 0.05;
/// Table P.10: COP of a sorption chiller on collective heat and on CHP.
pub const SORPTION_COP_COLLECTIVE: f64 = 0.7;
pub const SORPTION_COP_CHP: f64 = 1.0;
/// P.60/P.65/P.69: 10 % run-on time.
const RUN_TIME_MARGIN: f64 = 1.1;
/// Table P.12 (and the small heat systems of P.6.8.3.3), kWh_e/kWh_th.
pub const SECONDARY_AUX_SPECIFIC: f64 = 0.0018;
/// P.6.10.3.3, kWh_e/kWh_th.
pub const COLD_AUX_SPECIFIC: f64 = 0.009;
/// P.6.8.4.3/P.6.9.4.3: standby electronics per generator, W.
pub const STANDBY_W: f64 = 100.0;
/// P.6.10.4.3: standby of a cold generator, W.
pub const COLD_STANDBY_W: f64 = 10.0;
/// P.6.6.4.1 forfait water and ambient temperature of a vessel, °C.
pub const VESSEL_WATER_C: f64 = 70.0;
pub const VESSEL_AMBIENT_C: f64 = 20.0;
/// P.6.6.4.2 forfait values.
pub const CHARGE_CIRCULATION_C: f64 = 80.0;
pub const CHARGE_AMBIENT_C: f64 = 20.0;
pub const CHARGE_CORRECTION: f64 = 1.20;
/// P.55: `ρ_w` and `c_p;w`.
const WATER_DENSITY: f64 = 1000.0;
const WATER_HEAT_CAPACITY: f64 = 4190.0;
/// P.6.5.4.10.2: `θ_0;mi`, °C.
pub const COLLECTIVE_SOLAR_REFERENCE_C: f64 = 70.0;

/// Table P.2 (p. 966): `(β_HD;gen, F_HD;gen;gpref)`.
const TABLE_P2: [(f64, f64); 10] = [
    (0.0, 0.0),
    (0.1, 0.45),
    (0.2, 0.70),
    (0.3, 0.84),
    (0.4, 0.92),
    (0.5, 0.96),
    (0.6, 0.98),
    (0.7, 1.0),
    (0.8, 1.0),
    (0.9, 1.0),
];

/// Table P.8 (p. 998): `(β_CD;gen, F_CD;gen;gpref)`.
const TABLE_P8: [(f64, f64); 6] = [
    (0.0, 0.0),
    (0.1, 0.1),
    (0.2, 0.2),
    (0.3, 0.5),
    (0.5, 0.8),
    (1.0, 1.0),
];

/// Table P.16 (p. 1026): `(θ_ext;i, t_θei)` in °C and h; sums to 8 760 h.
pub const TABLE_P16: [(f64, f64); 42] = [
    (-9.0, 4.0),
    (-8.0, 5.0),
    (-7.0, 14.0),
    (-6.0, 11.0),
    (-5.0, 29.0),
    (-4.0, 45.0),
    (-3.0, 49.0),
    (-2.0, 95.0),
    (-1.0, 142.0),
    (0.0, 183.0),
    (1.0, 301.0),
    (2.0, 236.0),
    (3.0, 304.0),
    (4.0, 324.0),
    (5.0, 402.0),
    (6.0, 367.0),
    (7.0, 463.0),
    (8.0, 494.0),
    (9.0, 398.0),
    (10.0, 446.0),
    (11.0, 415.0),
    (12.0, 461.0),
    (13.0, 431.0),
    (14.0, 387.0),
    (15.0, 419.0),
    (16.0, 437.0),
    (17.0, 387.0),
    (18.0, 365.0),
    (19.0, 266.0),
    (20.0, 200.0),
    (21.0, 139.0),
    (22.0, 127.0),
    (23.0, 79.0),
    (24.0, 78.0),
    (25.0, 70.0),
    (26.0, 56.0),
    (27.0, 52.0),
    (28.0, 33.0),
    (29.0, 19.0),
    (30.0, 15.0),
    (31.0, 9.0),
    (32.0, 3.0),
];

fn interpolate(table: &[(f64, f64)], x: f64) -> f64 {
    let first = table[0];
    let last = table[table.len() - 1];
    if x <= first.0 {
        return first.1;
    }
    if x >= last.0 {
        return last.1;
    }
    for pair in table.windows(2) {
        let ((x1, y1), (x2, y2)) = (pair[0], pair[1]);
        if x <= x2 {
            return y1 + (y2 - y1) * (x - x1) / (x2 - x1);
        }
    }
    last.1
}

/// Table P.2 with linear interpolation; 1 above β = 0,9.
pub fn table_p2(beta: f64) -> f64 {
    interpolate(&TABLE_P2, beta)
}

/// Table P.8 with linear interpolation.
pub fn table_p8(beta: f64) -> f64 {
    interpolate(&TABLE_P8, beta)
}

/// Factors applied to one external supply carrier in chapter 5.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplyFactors {
    /// `f_P;del;dX`.
    pub primary_factor: f64,
    /// `f_Pren;dX`.
    pub renewable_factor: f64,
    /// `K_CO2;del;dX;tot`, kg/kWh.
    pub co2_kg_per_kwh: f64,
}

/// Table 5.2/5.3/5.4: external heat (dh, dw) without a declaration.
pub const HEAT_FORFAIT: SupplyFactors = SupplyFactors {
    primary_factor: 0.9,
    renewable_factor: 0.0,
    co2_kg_per_kwh: 0.09,
};

/// Table 5.2/5.3/5.4: external cold without a declaration.
pub const COLD_FORFAIT: SupplyFactors = SupplyFactors {
    primary_factor: F_P_EL / 3.0,
    renewable_factor: 0.0,
    co2_kg_per_kwh: K_CO2_EL / 3.0,
};

/// 9.6.8.1.1.2.3 a: collective heat-pump source below 20 °C.
pub const SOURCE_BELOW_20_FORFAIT: SupplyFactors = SupplyFactors {
    primary_factor: F_P_EL / 23.0,
    renewable_factor: 0.95,
    co2_kg_per_kwh: K_CO2_EL / 23.0,
};

/// Table 5.3 `K_CO2;el` of the active edition (2024 p. 94: 0,34;
/// 2025+C1 p. 96: 0,268).
pub fn k_co2_el() -> f64 {
    crate::norm_versions::profile().k_co2_electricity
}

/// [`HEAT_FORFAIT`] with `K_CO2` of the active edition (2024 p. 95: 0,17).
pub fn heat_forfait() -> SupplyFactors {
    SupplyFactors {
        co2_kg_per_kwh: crate::norm_versions::profile().k_co2_district_heat_forfait,
        ..HEAT_FORFAIT
    }
}

/// [`COLD_FORFAIT`] with `K_CO2;el / 3` of the active edition.
pub fn cold_forfait() -> SupplyFactors {
    SupplyFactors {
        co2_kg_per_kwh: k_co2_el() / 3.0,
        ..COLD_FORFAIT
    }
}

/// [`SOURCE_BELOW_20_FORFAIT`] with `K_CO2;el / 23` of the active edition.
pub fn source_below_20_forfait() -> SupplyFactors {
    SupplyFactors {
        co2_kg_per_kwh: k_co2_el() / 23.0,
        ..SOURCE_BELOW_20_FORFAIT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemFunction {
    /// Collective heat supply (HD, carrier dh).
    Heating,
    /// Collective hot-water circulation (WD, carrier dw).
    HotWater,
    /// Collective cold supply (CD, carrier dc).
    Cooling,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnnexPRoute {
    /// Values of a registered quality declaration (§5.8.0).
    Declared {
        #[serde(rename = "primaryFactor")]
        primary_factor: f64,
        #[serde(rename = "renewableFactor")]
        renewable_factor: f64,
        #[serde(rename = "co2KgPerKwh")]
        co2_kg_per_kwh: f64,
        #[serde(rename = "declarationReference")]
        declaration_reference: String,
        /// The declaration rests on measured values only (f_prac 1 in 9.84,
        /// 13.152, 10.78); otherwise 0,95.
        #[serde(default, rename = "measuredOnly")]
        measured_only: bool,
    },
    /// Annex P with calculated (and possibly measured) flows (P.7, P.9).
    Calculated(Box<CalculatedSystem>),
    /// Annex P with measured flows only (P.6 and its CO2 counterpart).
    Measured(Box<MeasuredSystem>),
}

/// Table 5.2/5.5 carriers delivered to the generators of the system.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SystemCarrier {
    NaturalGas,
    Oil,
    /// Grid electricity; `directRenewableShare` is the conservative share of
    /// renewable electricity with a direct physical connection (P.4.5).
    Electricity {
        #[serde(default, rename = "directRenewableShare")]
        direct_renewable_share: f64,
    },
    /// Table 5.5 bg: biogas and green gas on a local network.
    Biogas,
    /// Table 5.5 bm: biomass in a heat plant above 500 kW.
    BiomassAbove500Kw,
    /// Table 5.5 wi: waste incineration (AVI).
    WasteIncineration,
    /// Mix of biofuel and fossil gas (mbf, P.4.5) with a conservative share.
    BiofuelMix {
        #[serde(rename = "biofuelShare")]
        biofuel_share: f64,
    },
}

impl SystemCarrier {
    /// `f_P;del;ci` (tables 5.2 and 5.5).
    pub fn primary_factor(self) -> f64 {
        match self {
            Self::NaturalGas | Self::Oil => 1.0,
            Self::Electricity {
                direct_renewable_share,
            } => F_P_EL * (1.0 - direct_renewable_share),
            Self::Biogas | Self::BiomassAbove500Kw => 0.0,
            Self::WasteIncineration => 0.5,
            Self::BiofuelMix { biofuel_share } => 1.0 - biofuel_share,
        }
    }

    /// `K_CO2;del;ci` (tables 5.3 and 5.6).
    pub fn co2(self) -> f64 {
        match self {
            Self::NaturalGas => crate::norm_versions::profile().k_co2_gas,
            Self::Oil => crate::norm_versions::profile().k_co2_oil,
            Self::Electricity {
                direct_renewable_share,
            } => k_co2_el() * (1.0 - direct_renewable_share),
            Self::Biogas => 0.0 * 0.074,
            Self::BiomassAbove500Kw => 0.0 * 0.104,
            Self::WasteIncineration => crate::norm_versions::profile().k_co2_waste_incineration,
            Self::BiofuelMix { biofuel_share } => {
                crate::norm_versions::profile().k_co2_gas * (1.0 - biofuel_share)
            }
        }
    }

    fn renewable_electricity_share(self) -> f64 {
        match self {
            Self::Electricity {
                direct_renewable_share,
            } => direct_renewable_share,
            _ => 0.0,
        }
    }

    fn valid(self) -> bool {
        match self {
            Self::Electricity {
                direct_renewable_share: share,
            }
            | Self::BiofuelMix {
                biofuel_share: share,
            } => (0.0..=1.0).contains(&share),
            _ => true,
        }
    }
}

/// Table P.5 heat sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TableP5Source {
    ElectricGround,
    ElectricOutdoorAir,
    ElectricGroundwaterBelow15C,
    ElectricSurfaceWater,
    ElectricSource15To20C,
    ElectricSource20To40C,
    ElectricSourceAtLeast40C,
    GasGroundOrOutdoorAir,
    GasGroundwater,
    GasSurfaceWater,
}

/// Table P.5 (p. 973); `None` where the table has no value.
pub fn table_p5(source: TableP5Source, supply_temperature_c: f64) -> Option<f64> {
    use TableP5Source::*;
    let column = [30.0, 35.0, 40.0, 45.0, 50.0, 55.0, 60.0, 65.0, 70.0, 75.0]
        .iter()
        .position(|limit| supply_temperature_c <= *limit)?;
    let row: [Option<f64>; 10] = match source {
        ElectricGround => [3.55, 3.4, 3.25, 3.1, 2.95, 2.8, 2.4, 2.2, 2.1, 2.0].map(Some),
        ElectricOutdoorAir => [3.40, 3.25, 3.15, 3.05, 2.90, 2.80, 2.4, 2.2, 2.1, 2.0].map(Some),
        ElectricGroundwaterBelow15C => {
            [5.0, 4.7, 4.45, 4.2, 3.9, 3.6, 3.0, 2.8, 2.6, 2.5].map(Some)
        }
        ElectricSurfaceWater => [4.3, 4.1, 3.9, 3.7, 3.5, 3.3, 2.8, 2.6, 2.4, 2.3].map(Some),
        ElectricSource15To20C => [5.4, 5.0, 4.7, 4.4, 4.1, 3.7, 3.1, 2.9, 2.7, 2.5].map(Some),
        ElectricSource20To40C => [5.9, 5.3, 4.9, 4.6, 4.3, 3.9, 3.3, 3.0, 2.8, 2.6].map(Some),
        ElectricSourceAtLeast40C => [
            None,
            None,
            None,
            Some(10.3),
            Some(8.0),
            Some(6.5),
            Some(5.5),
            Some(4.8),
            Some(4.2),
            Some(3.8),
        ],
        GasGroundOrOutdoorAir => gas_row([1.65, 1.6, 1.55, 1.5, 1.45, 1.4]),
        GasGroundwater => gas_row([2.2, 2.1, 2.0, 1.9, 1.85, 1.8]),
        GasSurfaceWater => gas_row([1.95, 1.9, 1.85, 1.8, 1.75, 1.7]),
    };
    row[column]
}

fn gas_row(values: [f64; 6]) -> [Option<f64>; 10] {
    let mut row = [None; 10];
    for (index, value) in values.into_iter().enumerate() {
        row[index] = Some(value);
    }
    row
}

impl TableP5Source {
    fn electric(self) -> bool {
        !matches!(
            self,
            Self::GasGroundOrOutdoorAir | Self::GasGroundwater | Self::GasSurfaceWater
        )
    }
}

/// Table P.3 boiler classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoilerClass {
    /// Gas boiler without further designation, or any oil boiler.
    Conventional,
    Vr,
    Hr100,
    Hr104,
    Hr107,
}

/// Table P.4 classification of the heat generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureLevel {
    Low,
    High,
}

/// Table P.3 (p. 969).
pub fn table_p3(boiler: BoilerClass, level: TemperatureLevel) -> f64 {
    let (low, high) = match boiler {
        BoilerClass::Conventional => (0.70, 0.70),
        BoilerClass::Vr => (0.75, 0.75),
        BoilerClass::Hr100 => (0.875, 0.85),
        BoilerClass::Hr104 => (0.905, 0.875),
        BoilerClass::Hr107 => (0.925, 0.90),
    };
    match level {
        TemperatureLevel::Low => low,
        TemperatureLevel::High => high,
    }
}

/// Table P.4 emission systems with `θ_em;avg ≤ 50 °C`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmissionSystem {
    MixingWithoutReturnLimit,
    MixingWithReturnLimit,
    Direct,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmissionDesign {
    /// `θ_em;avg`: mean of the design supply and return temperature, °C.
    pub average_design_temperature_c: f64,
    pub system: EmissionSystem,
}

/// Table P.4 (p. 970).
pub fn table_p4(design: EmissionDesign) -> TemperatureLevel {
    if design.average_design_temperature_c > 50.0
        || design.system == EmissionSystem::MixingWithoutReturnLimit
    {
        TemperatureLevel::High
    } else {
        TemperatureLevel::Low
    }
}

/// Generation efficiency of a gas- or oil-fired boiler.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "method",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum BoilerEfficiency {
    /// Table P.3 with the level of table P.4 (hot water: always HT,
    /// P.6.6.5.2).
    TableP3 {
        boiler: BoilerClass,
        #[serde(default)]
        temperature_level: Option<TemperatureLevel>,
        #[serde(default)]
        emission: Option<EmissionDesign>,
    },
    /// Tested full-load efficiency at 80/60 (HT) or 60/40 (LT), less 5
    /// points when installed outdoors (P.6.5.4.2) or, for hot water, when
    /// not preferred (P.6.6.5.2).
    FullLoad {
        value: f64,
        #[serde(default)]
        outdoor_installation: bool,
        source_reference: String,
    },
}

/// Table P.6 input: a CHP by electrical power and build year.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TableP6 {
    pub electrical_power_kw: f64,
    pub installed_after_2006: bool,
    /// Table P.4; LT must be shown, so absent means HT.
    #[serde(default)]
    pub temperature_level: Option<TemperatureLevel>,
}

/// Table P.6 (p. 975): `(ε_chp;th, ε_chp;el)`; `None` outside 2 kW–25 MW.
pub fn table_p6(input: TableP6) -> Option<(f64, f64)> {
    // Upper limit kW; up to 2006 th, el; after 2006 th LT, th HT, el.
    const ROWS: [[f64; 6]; 5] = [
        [20.0, 0.57, 0.26, 0.57, 0.55, 0.28],
        [200.0, 0.54, 0.27, 0.51, 0.49, 0.30],
        [500.0, 0.50, 0.32, 0.52, 0.50, 0.32],
        [1000.0, 0.44, 0.35, 0.46, 0.44, 0.35],
        [25_000.0, 0.40, 0.36, 0.41, 0.39, 0.37],
    ];
    let power = input.electrical_power_kw;
    if power.is_nan() || power <= 2.0 {
        return None;
    }
    let row = ROWS.iter().find(|row| power <= row[0])?;
    Some(if input.installed_after_2006 {
        let thermal = match input.temperature_level {
            Some(TemperatureLevel::Low) => row[3],
            _ => row[4],
        };
        (thermal, row[5])
    } else {
        (row[1], row[2])
    })
}

/// Table P.9 compression chiller rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChillerVariant {
    Unspecified,
    HighTemperatureEmission,
    WetCooling,
    HighTemperatureEmissionAndWetCooling,
    LowTemperatureSource,
    HighTemperatureEmissionAndLowTemperatureSource,
}

/// Table P.9 (p. 1002): `COP_CD;gen;gi` of a compression chiller.
pub fn table_p9_cop(variant: ChillerVariant, gas_engine: bool) -> f64 {
    match variant {
        ChillerVariant::Unspecified => 3.0,
        ChillerVariant::HighTemperatureEmission | ChillerVariant::WetCooling => 4.0,
        ChillerVariant::HighTemperatureEmissionAndWetCooling => 5.0,
        ChillerVariant::LowTemperatureSource => 6.0,
        ChillerVariant::HighTemperatureEmissionAndLowTemperatureSource => {
            if gas_engine {
                7.0
            } else {
                8.0
            }
        }
    }
}

/// Table P.9 cold supply by an aquifer or another free source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FreeCoolingSource {
    /// Regeneration by heat-pump operation, storage principle, realised or
    /// permitted before 2013.
    AquiferStorageBefore2013,
    /// The same from 2013.
    AquiferStorageFrom2013,
    /// Regeneration by heat-pump operation, recirculation principle.
    AquiferRecirculation,
    /// Regeneration without using the heat for heating.
    AquiferWithoutHeatUse,
    /// Other free low-temperature cold sources.
    OtherLowTemperatureSource,
}

impl FreeCoolingSource {
    /// Table P.9 `η_CD;gen;gi`.
    pub fn efficiency(self) -> f64 {
        match self {
            Self::AquiferStorageBefore2013 => 18.0,
            Self::AquiferStorageFrom2013 | Self::OtherLowTemperatureSource => 23.0,
            Self::AquiferRecirculation => 14.0,
            Self::AquiferWithoutHeatUse => 9.0,
        }
    }
}

/// Shaft efficiency `η_ge` of a gas-engine chiller (table P.9).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "method",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum EngineEfficiency {
    Declared {
        value: f64,
        source_reference: String,
    },
    /// `ε_chp;el` of table P.6.
    TableP6(TableP6),
}

/// Table P.7 outer diameters (mm) and insulation thicknesses (mm).
const TABLE_P7_DIAMETERS: [f64; 10] = [10.0, 12.0, 15.0, 22.0, 28.0, 35.0, 42.0, 54.0, 67.0, 80.0];
const TABLE_P7_INSULATION: [f64; 5] = [0.0, 10.0, 15.0, 20.0, 25.0];
const TABLE_P7: [[f64; 5]; 10] = [
    [0.407, 0.165, 0.136, 0.114, 0.106],
    [0.453, 0.184, 0.154, 0.136, 0.124],
    [0.539, 0.211, 0.174, 0.154, 0.138],
    [0.728, 0.271, 0.219, 0.189, 0.169],
    [0.880, 0.321, 0.256, 0.219, 0.194],
    [1.049, 0.378, 0.299, 0.253, 0.223],
    [1.211, 0.435, 0.341, 0.287, 0.251],
    [1.477, 0.531, 0.412, 0.343, 0.299],
    [1.753, 0.635, 0.488, 0.404, 0.349],
    [2.018, 0.737, 0.563, 0.464, 0.399],
];

/// Table P.7 (p. 992), W/(m·K): the next larger tabulated diameter and the
/// next thinner tabulated insulation; `None` above 80 mm.
pub fn table_p7(outer_diameter_mm: f64, insulation_mm: f64) -> Option<f64> {
    let row = TABLE_P7_DIAMETERS
        .iter()
        .position(|d| outer_diameter_mm <= *d + 1e-9)?;
    let column = TABLE_P7_INSULATION
        .iter()
        .rposition(|t| insulation_mm + 1e-9 >= *t)?;
    Some(TABLE_P7[row][column])
}

/// Table P.14 dwelling types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DwellingHeatType {
    /// Apartment or gallery dwelling.
    Apartment,
    /// Terraced dwelling (and utility buildings).
    TerracedOrUtility,
    /// Corner or semi-detached dwelling.
    CornerOrSemiDetached,
    Detached,
}

/// Table P.14 (p. 1023, informative): `Q_H;dh;spec;mi`, kWh/m².
pub fn table_p14(kind: DwellingHeatType) -> Months {
    const ROWS: [[f64; 4]; 12] = [
        [4.33, 3.81, 7.86, 8.94],
        [3.19, 2.81, 5.81, 6.61],
        [2.97, 2.58, 5.36, 6.11],
        [2.08, 1.83, 3.78, 4.31],
        [0.92, 0.81, 1.69, 1.92],
        [0.0; 4],
        [0.0; 4],
        [0.0; 4],
        [0.53, 0.44, 0.94, 1.08],
        [1.50, 1.31, 2.72, 3.11],
        [2.81, 2.44, 5.08, 5.78],
        [3.86, 3.39, 7.03, 8.00],
    ];
    let column = match kind {
        DwellingHeatType::Apartment => 0,
        DwellingHeatType::TerracedOrUtility => 1,
        DwellingHeatType::CornerOrSemiDetached => 2,
        DwellingHeatType::Detached => 3,
    };
    ROWS.map(|row| row[column])
}

/// Table P.15 uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HotWaterUse {
    DwellingLowTemperature,
    DwellingHighTemperature,
    AssemblyWithAlcohol,
    Assembly,
    Cell,
    HealthcareClinical,
    HealthcareNonClinical,
    Office,
    Lodging,
    Education,
    Sport,
    Retail,
}

impl HotWaterUse {
    /// Table P.15 (p. 1025): `Q_W;dh;spec`, kWh/m² per year.
    pub fn specific_kwh_per_m2(self) -> f64 {
        match self {
            Self::DwellingLowTemperature => 29.17,
            Self::DwellingHighTemperature => 33.33,
            Self::AssemblyWithAlcohol | Self::Cell => 4.17,
            Self::Assembly | Self::HealthcareNonClinical => 2.78,
            Self::HealthcareClinical => 15.28,
            Self::Office | Self::Education | Self::Retail => 1.39,
            Self::Lodging | Self::Sport => 12.50,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum HeatPumpEfficiency {
    Declared {
        value: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
        /// P.6.8.4.3/P.6.9.4.3 (p. 1009): the source pump or fan is
        /// included in the declared efficiency, so its default is 0 W/kW
        /// instead of 10 W/kW.
        #[serde(default, rename = "sourcePumpIncluded")]
        source_pump_included: bool,
    },
    /// Table P.5 with the design supply temperature of the network.
    TableP5 {
        source: TableP5Source,
        #[serde(rename = "supplyTemperatureC")]
        supply_temperature_c: f64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GeneratorKind {
    /// Boilers and other fuel-fired generators (P.20/P.22) with a declared
    /// generation efficiency.
    Combustion {
        carrier: SystemCarrier,
        efficiency: f64,
        #[serde(rename = "efficiencyReference")]
        efficiency_reference: String,
    },
    /// Electric or gas heat pumps, chillers (P.20/P.22, 5.44/5.52).
    HeatPump {
        efficiency: HeatPumpEfficiency,
        /// Drive energy: electricity (optionally with a direct renewable
        /// share) or natural gas.
        drive: SystemCarrier,
    },
    /// P.26/P.27: CHP without power loss; declared conversion numbers
    /// override those of table P.6.
    ChpWithoutLoss {
        carrier: SystemCarrier,
        #[serde(default, rename = "thermalEfficiency")]
        thermal_efficiency: Option<f64>,
        #[serde(default, rename = "electricalEfficiency")]
        electrical_efficiency: Option<f64>,
        #[serde(default, rename = "efficiencyReference")]
        efficiency_reference: Option<String>,
        #[serde(default, rename = "tableP6")]
        table_p6: Option<TableP6>,
    },
    /// P.28–P.30: CHP with power loss; `lossRatio` defaults to 0,18 (P.29).
    ChpWithLoss {
        carrier: SystemCarrier,
        #[serde(default, rename = "lossRatio")]
        loss_ratio: Option<f64>,
        #[serde(default, rename = "lossRatioReference")]
        loss_ratio_reference: Option<String>,
    },
    /// P.6.5.4.7 with `f_rw;aux;spec` 0,07 unless declared.
    ResidualHeat {
        #[serde(default, rename = "auxiliarySpecific")]
        auxiliary_specific: Option<f64>,
        #[serde(default, rename = "auxiliaryReference")]
        auxiliary_reference: Option<String>,
    },
    /// P.6.5.4.8: `η = 20·Δθ_bron/40` with `Δθ_bron = θ_geo − θ_ret − 3`.
    Geothermal {
        #[serde(rename = "sourceTemperatureC")]
        source_temperature_c: f64,
        #[serde(rename = "returnTemperatureC")]
        return_temperature_c: f64,
    },
    /// Generator values from another determination (P.6.5.4.9 upstream
    /// system, solar collectors, flex mode, measured CHP data).
    Declared {
        #[serde(rename = "primaryFactor")]
        primary_factor: f64,
        #[serde(rename = "co2KgPerKwh")]
        co2_kg_per_kwh: f64,
        #[serde(rename = "renewableFactor")]
        renewable_factor: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// P.6.5.4.2/P.6.6.5.2: gas- or oil-fired boiler.
    Boiler {
        carrier: SystemCarrier,
        efficiency: BoilerEfficiency,
    },
    /// P.6.5.4.3: solid biomass, efficiency on the lower heating value per
    /// NEN-EN 303-5 divided by 1,08, less 0,05.
    SolidBiomassBoiler {
        carrier: SystemCarrier,
        #[serde(rename = "netEfficiency")]
        net_efficiency: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// P.6.5.4.10/P.6.6.5.10: collective solar collectors, always preferred
    /// with `F = ΣQ_sol;mi / Q_XD;in;tot` (P.32), factor 0.
    CollectiveSolar { contribution: SolarContribution },
    /// P.6.5.4.11: electric generators in flex mode (5.8, tables 5.5/5.6);
    /// the thermal power is `nominalPowerKw`.
    ElectricFlex {
        generator: FlexGenerator,
        /// Flex-mode heat from hourly records; absent: 1 500 h at the
        /// installed power, at most 15 % of the network production.
        #[serde(default, rename = "flexHeatKwh")]
        flex_heat_kwh: Option<f64>,
        #[serde(default, rename = "flexReference")]
        flex_reference: Option<String>,
        /// Annual heat production of the whole network, heating and hot
        /// water together, for the 15 % cap (5.8); absent: the network heat
        /// of the function being calculated.
        #[serde(default, rename = "networkProductionKwh")]
        network_production_kwh: Option<f64>,
        /// Connections of the heat network (at least 500, 5.8).
        connections: u32,
        /// A heat buffer that decouples production from demand (5.8).
        #[serde(rename = "heatBuffer")]
        heat_buffer: bool,
        /// Transparent registration of the flex-mode production (5.8).
        #[serde(rename = "registrationReference")]
        registration_reference: String,
    },
    /// Table P.9: electric or gas-engine compression chiller.
    CompressionChiller {
        variant: ChillerVariant,
        drive: SystemCarrier,
        /// Gas engine only: `η_ge`.
        #[serde(default, rename = "engineEfficiency")]
        engine_efficiency: Option<EngineEfficiency>,
    },
    /// Table P.9: aquifer or other free low-temperature cold source.
    FreeCooling {
        source: FreeCoolingSource,
        drive: SystemCarrier,
    },
    /// Table P.10: sorption chiller on collective heat or CHP.
    SorptionChiller { heat: SorptionHeat },
}

/// The solar contribution `Q_XD;sol;mi` of collective collectors.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "method",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum SolarContribution {
    /// From another 13.7.2.2 determination: twelve monthly or one annual
    /// value, kWh.
    Declared {
        #[serde(default)]
        monthly_kwh: Vec<f64>,
        #[serde(default)]
        annual_kwh: Option<f64>,
        source_reference: String,
    },
    /// 13.7.2.2 on the monthly network input `Q_XD;in;mi` (P.33) with
    /// `θ_0;mi` = 70 °C.
    Calculated(Box<CollectiveSolarField>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectiveSolarField {
    pub solar_type: SolarType,
    pub collectors: CollectorField,
    pub storage: SolarStorage,
    /// Network design supply temperature (backup set point), °C.
    pub network_supply_c: f64,
    /// Network design return temperature, °C.
    pub network_return_c: f64,
    /// Ambient temperature of the solar store, °C.
    pub storage_ambient_c: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum FlexGenerator {
    /// Efficiency 0,99 unless declared.
    ElectrodeBoiler {
        #[serde(default)]
        efficiency: Option<f64>,
        #[serde(default)]
        efficiency_reference: Option<String>,
    },
    HeatPump {
        efficiency: HeatPumpEfficiency,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "source",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum SorptionHeat {
    /// `f_CD;gen = f_P;HD;tot / 0,7`.
    CollectiveHeat {
        primary_factor: f64,
        co2_kg_per_kwh: f64,
        source_reference: String,
    },
    /// `f_CD;gen = f_HD;gen;chp / 1,0` with P.26/P.27.
    Chp(ChpData),
}

/// CHP data for P.26/P.27: declared conversion numbers or table P.6.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChpData {
    pub carrier: SystemCarrier,
    #[serde(default)]
    pub thermal_efficiency: Option<f64>,
    #[serde(default)]
    pub electrical_efficiency: Option<f64>,
    #[serde(default)]
    pub efficiency_reference: Option<String>,
    #[serde(default)]
    pub table_p6: Option<TableP6>,
}

/// Cooling power of a cold generator for β (P.52–P.55).
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(
    tag = "method",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum CoolingPower {
    /// P.53: `η_CD;gen × P_in` from the motor shaft power.
    CompressorShaft { shaft_power_kw: f64 },
    /// P.55: `φ·ρ_w·c_p;w·(θ_sup − θ_ret)·10⁻³`.
    Aquifer {
        flow_m3_per_s: f64,
        supply_c: f64,
        return_c: f64,
    },
}

/// Table P.13 heat rejection of chillers using outdoor air.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatRejection {
    /// Cooling tower or evaporative condenser, closed circuit: 33 W/kW.
    ClosedCoolingTower,
    /// Open circuit: 18 W/kW.
    OpenCoolingTower,
    /// Dry cooler: 45 W/kW.
    DryCooler,
}

impl HeatRejection {
    /// Table P.13 (p. 1017), W/kW.
    pub fn specific_w_per_kw(self) -> f64 {
        match self {
            Self::ClosedCoolingTower => 33.0,
            Self::OpenCoolingTower => 18.0,
            Self::DryCooler => 45.0,
        }
    }
}

/// Auxiliary-energy values of one generator (P.6.8.4.3, P.6.9.4.3,
/// P.6.10.4.3); absent values take the forfait of the generator kind.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratorAuxiliary {
    /// `P_XD;aux;gen;e` (cold: `P_CD;gen;aux;gi`), W.
    #[serde(default)]
    pub standby_w: Option<f64>,
    /// `P_XD;aux;gen;v;spec`, W/kW.
    #[serde(default)]
    pub burner_w_per_kw: Option<f64>,
    /// `P_XD;aux;gen;hs;spec`, W/kW.
    #[serde(default)]
    pub source_w_per_kw: Option<f64>,
    /// `P_XD;aux;gen;sp;spec`, W/kW.
    #[serde(default)]
    pub solution_pump_w_per_kw: Option<f64>,
    /// Cold: heat rejection to outdoor air (P.69/P.70, table P.13).
    #[serde(default)]
    pub heat_rejection: Option<HeatRejection>,
    /// Cold: `P_CD;aux;gen;spec` instead of table P.13, W/kW.
    #[serde(default)]
    pub heat_rejection_w_per_kw: Option<f64>,
    /// Measured component data behind deviating values.
    #[serde(default)]
    pub source_reference: Option<String>,
    /// Hot water: the generator also serves the heating supply, whose
    /// function carries its standby (P.6.8.4.1 a), P.6.9.4.3): the forfait
    /// `P_WD;aux;gen;e` becomes 0 W.
    #[serde(default)]
    pub also_serves_heating: bool,
    /// Hot water: the generator works without auxiliary energy, such as a
    /// traditional gas boiler (P.6.9.4.3): forfait 0 W and 0 W/kW.
    #[serde(default)]
    pub without_auxiliary_energy: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemGenerator {
    pub id: String,
    /// `F_XD;gen;gi` from operating data or a design calculation;
    /// absent for all generators: derived per P.6.5.3/P.6.6.3/P.6.7.3.
    #[serde(default)]
    pub energy_fraction: Option<f64>,
    /// `P_XD;gen;gi`: nominal thermal power, kW (cold: the cooling power,
    /// overriding `coolingPower`).
    #[serde(default)]
    pub nominal_power_kw: Option<f64>,
    /// Cold: P.53/P.55 cooling power.
    #[serde(default)]
    pub cooling_power: Option<CoolingPower>,
    /// Fixed order of preference (1 first); equal values form one group.
    /// Absent for all: the rules of P.6.5.3.2/P.6.7.3.2.
    #[serde(default)]
    pub priority: Option<u32>,
    #[serde(default)]
    pub auxiliary: Option<GeneratorAuxiliary>,
    pub kind: GeneratorKind,
}

/// Table P.0 design temperature classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkTemperature {
    /// 90/60 °C; also for hot-water circulation and unknown levels.
    T90To60,
    T90To50,
    T70To40,
    T50To40,
    T35To25,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    /// One connection per plot.
    GroundBound,
    /// Several connections within one building.
    WithinBuilding,
}

/// Table P.0 (p. 961), kWh per connection per year.
pub fn table_p0(temperature: NetworkTemperature, connection: ConnectionType) -> f64 {
    use NetworkTemperature::*;
    let (ground, within) = match temperature {
        T90To60 => (3410.0, 1750.0),
        T90To50 => (2870.0, 1515.0),
        T70To40 => (2350.0, 1180.0),
        T50To40 => (2240.0, 1125.0),
        T35To25 => (985.0, 480.0),
    };
    match connection {
        ConnectionType::GroundBound => ground,
        ConnectionType::WithinBuilding => within,
    }
}

/// Table P.0 footnote a: the next higher class for a design pair.
pub fn table_p0_class(supply_c: f64, return_c: f64) -> NetworkTemperature {
    use NetworkTemperature::*;
    [
        (35.0, 25.0, T35To25),
        (50.0, 40.0, T50To40),
        (70.0, 40.0, T70To40),
        (90.0, 50.0, T90To50),
    ]
    .into_iter()
    .find(|(sup, ret, _)| supply_c <= *sup && return_c <= *ret)
    .map_or(T90To60, |(_, _, class)| class)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum SystemDistribution {
    /// P.10–P.12: two of the three annual flows.
    Flows {
        #[serde(default, rename = "inputKwh")]
        input_kwh: Option<f64>,
        #[serde(default, rename = "lossKwh")]
        loss_kwh: Option<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// P.6.4: forfait loss per small connection (table P.0) plus the losses
    /// of large consumers and a primary network.
    SmallSystemForfait {
        connections: u32,
        #[serde(rename = "connectionType")]
        connection_type: ConnectionType,
        #[serde(default, rename = "designTemperature")]
        design_temperature: Option<NetworkTemperature>,
        #[serde(default, rename = "otherLossKwh")]
        other_loss_kwh: f64,
    },
    /// P.6.4: small cold system; 15 % loss below 10 °C supply, else 0.
    SmallColdForfait {
        #[serde(rename = "supplyBelow10C")]
        supply_below_10_c: bool,
    },
    /// P.13–P.18: pipe losses per month (P.14) or per outdoor-temperature
    /// bin (P.15, table P.16), plus buffer vessels (P.43/P.44 at the actual
    /// temperatures) and other losses (large consumers, a primary network).
    Pipes {
        segments: Vec<PipeSegment>,
        /// `θ_XD;circ`; absent for hot water: 65 °C.
        #[serde(default, rename = "waterTemperature")]
        water_temperature: Option<NetworkWaterTemperature>,
        #[serde(default)]
        buffers: Vec<StorageVessel>,
        /// Cold: a supply below 10 °C; otherwise the loss is 0 (P.6.2.2).
        #[serde(default, rename = "supplyBelow10C")]
        supply_below_10_c: Option<bool>,
        #[serde(default, rename = "otherLossKwh")]
        other_loss_kwh: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

/// Surroundings of a pipe for `θ_ext;avg;mi,j`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PipeAmbient {
    /// Outdoor air: `θ_e;avg;mi` (table 17.1) or the bin temperature.
    Outdoor,
    /// Crawl space: P.16.
    Crawlspace,
    /// Indoor space with a design temperature, °C.
    Indoor { temperature_c: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PipePlacement {
    /// P.17; `coverDepthM` is `h_j`.
    Buried {
        cover_depth_m: f64,
        /// Deviating from table P.1 (1,75): a measured soil value.
        #[serde(default)]
        ground_conductivity: Option<f64>,
        ambient: PipeAmbient,
    },
    /// P.18; `h_a;j` 8 in enclosed spaces, 25 elsewhere (table P.1).
    InAir {
        ambient: PipeAmbient,
        /// Deviating from table P.1: a measured `h_a;j`.
        #[serde(default)]
        surface_coefficient: Option<f64>,
    },
}

/// Table P.1 cases for `f_x;j`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipeCorrection {
    /// Two pipes in one trench without rigid connections: 1,05.
    TwoPipesInTrench,
    /// Two pipes in one trench with rigid connections: 1,00.
    TwoPipesInTrenchRigid,
    /// Two old pipes in a sliding system: 0,80.
    OldSlidingSystem,
    /// Surface-mounted or recessed: 0,90.
    SurfaceOrRecessed,
    /// One pipe in a trench without rigid connections: 1,00. Deviating:
    /// table P.1 has no single-pipe case; it takes the neutral factor.
    SinglePipeInTrench,
}

impl PipeCorrection {
    pub fn factor(self) -> f64 {
        match self {
            Self::TwoPipesInTrench => 1.05,
            Self::TwoPipesInTrenchRigid | Self::SinglePipeInTrench => 1.0,
            Self::OldSlidingSystem => 0.80,
            Self::SurfaceOrRecessed => 0.90,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PipeLayer {
    /// `λ_k;j,i` per NEN-EN-ISO 8497 (rounded up to 0,001), W/(m·K).
    pub conductivity_w_per_mk: f64,
    pub inner_diameter_m: f64,
    pub outer_diameter_m: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PipeSegment {
    pub length_m: f64,
    /// Insulation and other layers, inside out; the last outer diameter is
    /// `D_u;j`.
    #[serde(default)]
    pub layers: Vec<PipeLayer>,
    pub placement: PipePlacement,
    /// Table P.1; absent in air: surface-mounted or recessed.
    #[serde(default)]
    pub correction: Option<PipeCorrection>,
    #[serde(default)]
    pub correction_factor: Option<f64>,
    /// `R_XD;dis;j` instead of P.17/P.18, K·m/W.
    #[serde(default)]
    pub resistance_km_per_w: Option<f64>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurvePoint {
    pub outdoor_c: f64,
    pub water_c: f64,
}

/// Mean network water temperature during operation.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "method",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum NetworkWaterTemperature {
    /// The whole year at one temperature.
    Constant { temperature_c: f64 },
    /// P.14: twelve monthly means; `null` when out of operation.
    Monthly { temperatures_c: Vec<Option<f64>> },
    /// P.15: a heating curve over the outdoor bins of table P.16, out of
    /// operation above `offAboveOutdoorC`.
    OutdoorBins {
        curve: Vec<CurvePoint>,
        #[serde(default)]
        off_above_outdoor_c: Option<f64>,
    },
}

/// P.6.6.4.1 insulation of a vessel (`α_sto`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VesselInsulation {
    /// 10 W/(m²·K).
    None,
    /// At least 1 cm around the whole vessel: 2,5 W/(m²·K).
    AtLeast10Mm,
    /// At least 2 cm: 1,5 W/(m²·K).
    AtLeast20Mm,
    /// At least 3 cm: 1,0 W/(m²·K).
    AtLeast30Mm,
}

impl VesselInsulation {
    pub fn loss_factor(self) -> f64 {
        match self {
            Self::None => 10.0,
            Self::AtLeast10Mm => 2.5,
            Self::AtLeast20Mm => 1.5,
            Self::AtLeast30Mm => 1.0,
        }
    }
}

/// An indirectly heated vessel or buffer (P.43/P.44).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageVessel {
    /// `V_sto`, l.
    #[serde(default)]
    pub volume_l: Option<f64>,
    /// `A_sto`, m²; absent: `V/100` (diameter ≥ 50 cm) or `V/200`.
    #[serde(default)]
    pub surface_m2: Option<f64>,
    #[serde(default)]
    pub diameter_at_least_50_cm: Option<bool>,
    #[serde(default)]
    pub insulation: Option<VesselInsulation>,
    /// `α_sto` instead of the insulation class, W/(m²·K).
    #[serde(default)]
    pub loss_factor_w_per_m2k: Option<f64>,
    /// Measured `Q_sto;s-b`, kWh/day, at `standbyTestDifferenceK`.
    #[serde(default)]
    pub standby_loss_kwh_per_day: Option<f64>,
    /// `Δθ_sto;s-b`, K.
    #[serde(default)]
    pub standby_test_difference_k: Option<f64>,
    /// `θ_WD;sto`; absent: 70 °C.
    #[serde(default)]
    pub water_temperature_c: Option<f64>,
    /// `θ_WD;amb;sto`; absent: 20 °C.
    #[serde(default)]
    pub ambient_temperature_c: Option<f64>,
}

/// P.45: a pipe between the exchanger and the generators.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChargingPipe {
    pub length_m: f64,
    /// `U_j`, W/(m·K); absent: table P.7.
    #[serde(default)]
    pub u_value_w_per_mk: Option<f64>,
    #[serde(default)]
    pub outer_diameter_mm: Option<f64>,
    #[serde(default)]
    pub insulation_mm: Option<f64>,
    /// `θ_WD;amb;j`; absent: 20 °C.
    #[serde(default)]
    pub ambient_temperature_c: Option<f64>,
}

/// P.46: an external plate heat exchanger.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalExchanger {
    pub nominal_power_kw: f64,
    /// At least 20 mm all round: 0,2 W/kW, else 1,3 W/kW.
    #[serde(default)]
    pub insulated: bool,
    #[serde(default)]
    pub specific_loss_w_per_kw: Option<f64>,
}

/// P.6.6.4.3 insulation classes for the forfait `η_WD;gen;sto`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WdStorageInsulation {
    /// At least 20 mm around storage, pipes and any external exchanger: 0,90.
    AtLeast20Mm,
    /// At least 10 mm around storage and pipes, exchanger uninsulated: 0,80.
    AtLeast10Mm,
    /// No insulation: 0,50.
    None,
}

impl WdStorageInsulation {
    pub fn efficiency(self) -> f64 {
        match self {
            Self::AtLeast20Mm => 0.90,
            Self::AtLeast10Mm => 0.80,
            Self::None => 0.50,
        }
    }
}

/// `η_WD;gen;sto` of a collective hot-water circulation system (P.34/P.35).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum WdStorage {
    /// P.35 with the annual losses of P.6.6.4.1 (storage) and P.6.6.4.2
    /// (pipes and external exchanger).
    Losses {
        #[serde(rename = "storageLossKwh")]
        storage_loss_kwh: f64,
        #[serde(rename = "pipeLossKwh")]
        pipe_loss_kwh: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// P.6.6.4.3 forfait value.
    Forfait { insulation: WdStorageInsulation },
    /// P.35 with P.43–P.46 and table P.7.
    Calculated {
        #[serde(default)]
        vessels: Vec<StorageVessel>,
        #[serde(default)]
        pipes: Vec<ChargingPipe>,
        #[serde(default)]
        exchanger: Option<ExternalExchanger>,
        /// `θ_WD;circ`; absent: 80 °C.
        #[serde(default, rename = "circulationTemperatureC")]
        circulation_temperature_c: Option<f64>,
        /// `f_x`; absent: 1,20.
        #[serde(default, rename = "correctionFactor")]
        correction_factor: Option<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

/// One plot flow: an annual value, twelve monthly values or both.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlotFlow {
    #[serde(default)]
    pub annual_kwh: Option<f64>,
    #[serde(default)]
    pub monthly_kwh: Vec<f64>,
}

/// One connected plot (P.8). Values from an energy performance
/// calculation override the forfait values of P.8.5.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AreaPlot {
    pub id: String,
    /// `A_g;woon;pi` or `A_g;tot;pi`, m².
    #[serde(default)]
    pub usable_area_m2: Option<f64>,
    /// `E_H;dh;pi` (chapter 9).
    #[serde(default)]
    pub heating: Option<PlotFlow>,
    /// Table P.14 for P.79/P.80.
    #[serde(default)]
    pub heating_forfait: Option<DwellingHeatType>,
    /// `E_C;dh;pi`: heat for sorption cooling (chapter 10).
    #[serde(default)]
    pub sorption_cooling: Option<PlotFlow>,
    /// `E_W;dh;pi` (chapter 13).
    #[serde(default)]
    pub hot_water: Option<PlotFlow>,
    /// Table P.15 for P.81–P.83.
    #[serde(default)]
    pub hot_water_forfait: Option<HotWaterUse>,
    /// Heat supply: the hot water is made on the plot with a delivery set,
    /// so it counts in P.72.
    #[serde(default)]
    pub hot_water_via_delivery_set: bool,
    /// `E_C;dc;pi` (chapter 10).
    #[serde(default)]
    pub cooling: Option<PlotFlow>,
    /// `E_dhum;dc;pi` (chapter 12).
    #[serde(default)]
    pub dehumidification: Option<PlotFlow>,
    pub source_reference: String,
}

/// P.8: the connected plots of the area.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AreaDemand {
    pub plots: Vec<AreaPlot>,
}

/// P.11 network types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuxiliaryNetwork {
    PrimaryAndSecondary,
    Primary,
    Secondary,
    /// A small system for external heat or hot water (secondary value).
    SmallSystem,
}

/// `W_XD;aux;dis` (P.57/P.58, P.62/P.63, P.67/P.68).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "method",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum DistributionAuxiliary {
    /// `t_on/1 000 × ΣP_pump`; `t_on` defaults to 8 760 h for heat and hot
    /// water and is required for cold.
    Pumps {
        pump_powers_w: Vec<f64>,
        #[serde(default)]
        operating_hours: Option<f64>,
        source_reference: String,
    },
    /// P.68 (cold): pumps on in the months with cold delivery.
    PumpsMonthly {
        pump_powers_w: Vec<f64>,
        source_reference: String,
    },
    /// Tables P.11/P.12 and P.6.10.3.3 times `Q_XD;in;tot`.
    Forfait {
        #[serde(default)]
        network: Option<AuxiliaryNetwork>,
        /// `L`: production unit to the farthest point of the primary
        /// network, km.
        #[serde(default)]
        farthest_distance_km: Option<f64>,
    },
}

/// P.56/P.61/P.66 inputs; the generator terms come from the generators.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuxiliaryInput {
    pub distribution: DistributionAuxiliary,
    /// `W_XD;aux;sol;an` (13.7.2); absent: from the calculated collectors.
    #[serde(default)]
    pub solar_kwh: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalculatedSystem {
    pub function: SystemFunction,
    /// `η_WD;gen;sto` (P.34); required for hot water (WD), not allowed
    /// otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hot_water_storage: Option<WdStorage>,
    /// `Q_XD;out;tot`, kWh per year; overrides the area demand.
    #[serde(default)]
    pub delivered_kwh: Option<f64>,
    /// P.72–P.83.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_demand: Option<AreaDemand>,
    pub distribution: SystemDistribution,
    pub generators: Vec<SystemGenerator>,
    /// `P_HD;gen;ref` from at least three years of peak records instead of
    /// P.25, kW.
    #[serde(default)]
    pub reference_power_kw: Option<f64>,
    /// `W_XD;aux;tot`, kWh per year; overrides `auxiliary`.
    #[serde(default)]
    pub auxiliary_electricity_kwh: Option<f64>,
    /// P.56–P.70.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary: Option<AuxiliaryInput>,
    /// Conservative direct renewable share of the auxiliary electricity.
    #[serde(default)]
    pub auxiliary_renewable_share: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeasuredInput {
    pub carrier: SystemCarrier,
    pub kwh: f64,
    /// Use of a CHP with power loss (`E_XD;in2`), with `Δε_chp;el`.
    #[serde(default)]
    pub chp_loss_electrical: Option<f64>,
}

/// P.6: measured annual flows over at least three years.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeasuredSystem {
    pub function: SystemFunction,
    pub delivered_kwh: f64,
    pub inputs: Vec<MeasuredInput>,
    /// `E_XD;exp1;el`, kWh.
    #[serde(default)]
    pub exported_electricity_kwh: f64,
    /// `f_Pren;dX` from 5.42/5.49/5.50 as documented for the system.
    pub renewable_factor: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnnexPIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorResult {
    pub id: String,
    /// `f_XD;gen;gi`.
    pub primary_factor: f64,
    pub co2_kg_per_kwh: f64,
    /// `f_Pren;XD;gi`.
    pub renewable_factor: f64,
    /// `Q_XD;gen;gi`, kWh.
    pub heat_kwh: f64,
    /// `F_XD;gen;gi`, supplied or derived.
    pub energy_fraction: f64,
    /// `η_XD;gen;gi` where the kind has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub efficiency: Option<f64>,
    /// `W_XD;aux;gen;gi` when calculated, kWh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auxiliary_kwh: Option<f64>,
}

/// P.56/P.61/P.66 terms, kWh per year.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuxiliaryResult {
    pub distribution_kwh: f64,
    pub solar_kwh: f64,
    pub generators_kwh: f64,
    pub total_kwh: f64,
}

/// Intermediate values of the calculated route.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationDetails {
    /// `Q_XD;out;tot`, kWh.
    pub delivered_kwh: f64,
    /// `Q_XD;in;tot`, kWh.
    pub input_kwh: f64,
    /// `Q_XD;dis;ls`, kWh.
    pub distribution_loss_kwh: f64,
    /// `Q_XD;in;mi` (P.33), kWh. For cold the months follow P.78, which
    /// leaves dehumidification out, so they can sum below `input_kwh`.
    pub monthly_input_kwh: Months,
    /// `W_XD;aux;tot` used, kWh.
    pub auxiliary_electricity_kwh: f64,
    /// Present when the auxiliary energy was calculated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auxiliary: Option<AuxiliaryResult>,
    /// β of the preferred generators (P.24/P.52) when fractions were derived.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beta: Option<f64>,
    /// `P_HD;gen;ref` (P.25), kW.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_power_kw: Option<f64>,
    /// Ids of the preferred generators.
    pub preferred: Vec<String>,
    pub fractions_derived: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemResult {
    pub factors: SupplyFactors,
    /// `η_XD;dis` (P.10); `None` for declared values.
    pub distribution_efficiency: Option<f64>,
    /// `f_XD;gen;tot` (P.19, or P.34 for WD including `η_WD;gen;sto`).
    pub generation_primary_factor: Option<f64>,
    /// `η_WD;gen;sto` (P.35 or P.6.6.4.3) for hot water.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_efficiency: Option<f64>,
    pub generators: Vec<GeneratorResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calculation: Option<CalculationDetails>,
    /// Non-blocking findings: the factors follow the norm, but a literal
    /// formula or an input gives a value worth checking.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<AnnexPIssue>,
}

fn issue(code: &'static str, path: impl Into<String>) -> AnnexPIssue {
    AnnexPIssue {
        code,
        path: path.into(),
    }
}

// `+ 0.0` turns a `-0.0` (for example `ceil(-1e-9)`) into `0.0`.
fn round_up(value: f64) -> f64 {
    (value * 100.0 - 1e-9).ceil() / 100.0 + 0.0
}

fn round_down(value: f64) -> f64 {
    (value * 100.0 + 1e-9).floor() / 100.0 + 0.0
}

/// `f_Pren;dX`: 5.42 (heat) and 5.50 (hot water) are rounded down to a
/// multiple of 0,01; 5.49 for cold (§5.8.3.2, p. 129) states no rounding,
/// so `f_Pren;dc` is kept unrounded.
fn round_renewable(function: SystemFunction, value: f64) -> f64 {
    match function {
        SystemFunction::Cooling => value + 0.0,
        _ => round_down(value),
    }
}

/// Annex P and §5.8 readings where the printed norm is not explicit (for
/// the report appendix; the module documentation lists all of them).
pub const INTERPRETATIONS: &[&str] = &[
    "P.74 (p. 1020) is applied per plot part without monthly values, with max(0, θ_in;ref − θ_e;avg;mi); an annual-only sorption-cooling part of a heat network also takes P.74, so the supplied months of the other parts stay",
    "P.77/P.78 (p. 1022): dehumidification E_dhum;dc counts in the annual cold delivery but not in the monthly profile, because P.78 sums only E_C;dc;mi",
    "5.49 (§5.8.3.2, p. 129) gives no rounding for f_Pren;dc, unlike 5.42 and 5.50 (rounded down to 0,01); f_Pren;dc is kept unrounded",
    "P.70 (p. 1016) writes Q_CD;dis;tot;an in the formula while the legend defines Q_XD;in;tot; the legend (cold produced by the chillers) is followed",
    "P.27 has no MAX(0); a CHP on a renewable fuel gives a negative K_CO2;gen, kept as printed with a warning",
    "P.13–P.18 for a cold network count heat gains in every month with a water temperature; months without cold delivery give a warning and should have no water temperature",
    "P.6.8.4.3 (p. 1009): the 10 W/kW source pump of a heat pump with a declared efficiency is 0 when the declaration includes it (sourcePumpIncluded)",
];

fn reference(value: &str, path: String, issues: &mut Vec<AnnexPIssue>) {
    if value.trim().is_empty() {
        issues.push(issue("source_reference_required", path));
    }
}

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn optional_reference(value: Option<&String>, path: String, issues: &mut Vec<AnnexPIssue>) {
    if value.map_or(true, |value| value.trim().is_empty()) {
        issues.push(issue("source_reference_required", path));
    }
}

/// Factors of one generator.
#[derive(Debug, Clone, Copy, Default)]
struct GenFactors {
    /// `f_XD;gen;gi`.
    f: f64,
    /// `K_CO2;gen;gi`.
    k: f64,
    /// `f_Pren;XD;gi`.
    pren: f64,
    eta: Option<f64>,
    /// `W_XD;gen;ren` per kWh of heat or cold.
    renewable_drive: f64,
    /// `COP_CD;gen;gi` (P.70).
    cop: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
struct FactorContext {
    /// Hot water: a non-preferred boiler loses 5 points (P.6.6.5.2).
    preferred: bool,
    /// `Q_XD;gen;gi`, kWh.
    heat_kwh: f64,
    /// Production of all generators of the network, kWh.
    network_heat_kwh: f64,
    nominal_power_kw: Option<f64>,
}

#[cfg(test)]
const STANDALONE: FactorContext = FactorContext {
    preferred: true,
    heat_kwh: 0.0,
    network_heat_kwh: 0.0,
    nominal_power_kw: None,
};

/// `(f_XD;gen;gi, K_CO2;gen;gi, f_Pren;XD;gi)` outside a system.
#[cfg(test)]
fn generator_factors(
    kind: &GeneratorKind,
    function: SystemFunction,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<(f64, f64, f64)> {
    factors(kind, function, STANDALONE, path, issues).map(|item| (item.f, item.k, item.pren))
}

fn heat_pump_efficiency(efficiency: &HeatPumpEfficiency) -> Option<f64> {
    match efficiency {
        HeatPumpEfficiency::Declared { value, .. } => Some(*value),
        HeatPumpEfficiency::TableP5 {
            source,
            supply_temperature_c,
        } => table_p5(*source, *supply_temperature_c),
    }
}

/// Table P.3/P.4 or a full-load value (P.6.5.4.2, P.6.6.5.2).
fn boiler_efficiency(
    efficiency: &BoilerEfficiency,
    function: SystemFunction,
    preferred: bool,
) -> Option<f64> {
    match efficiency {
        BoilerEfficiency::TableP3 {
            boiler,
            temperature_level,
            emission,
        } => {
            let level = if function == SystemFunction::HotWater {
                Some(TemperatureLevel::High)
            } else {
                temperature_level.or(emission.map(table_p4))
            };
            level.map(|level| table_p3(*boiler, level))
        }
        BoilerEfficiency::FullLoad {
            value,
            outdoor_installation,
            ..
        } => {
            let deduct =
                *outdoor_installation || (function == SystemFunction::HotWater && !preferred);
            Some(value - if deduct { FULL_LOAD_DEDUCTION } else { 0.0 })
        }
    }
}

fn biomass_efficiency(net_efficiency: f64) -> f64 {
    net_efficiency / BIOMASS_HHV_RATIO - BIOMASS_DEDUCTION
}

fn geothermal_efficiency(source_c: f64, return_c: f64) -> f64 {
    GEOTHERMAL_EFFICIENCY_40K * (source_c - return_c - 3.0) / 40.0
}

fn flex_efficiency(generator: &FlexGenerator) -> Option<f64> {
    match generator {
        FlexGenerator::ElectrodeBoiler { efficiency, .. } => {
            Some(efficiency.unwrap_or(ELECTRODE_BOILER_EFFICIENCY))
        }
        FlexGenerator::HeatPump { efficiency } => heat_pump_efficiency(efficiency),
    }
}

fn engine_efficiency(engine: Option<&EngineEfficiency>) -> Option<f64> {
    match engine? {
        EngineEfficiency::Declared { value, .. } => Some(*value),
        EngineEfficiency::TableP6(input) => table_p6(*input).map(|(_, electrical)| electrical),
    }
}

/// Table P.9: `(η_CD;gen;gi, COP_CD;gen;gi)`.
fn chiller_efficiency(
    variant: ChillerVariant,
    drive: &SystemCarrier,
    engine: Option<&EngineEfficiency>,
) -> Option<(f64, f64)> {
    match drive {
        SystemCarrier::Electricity { .. } => {
            let cop = table_p9_cop(variant, false);
            Some((cop, cop))
        }
        SystemCarrier::NaturalGas | SystemCarrier::Biogas | SystemCarrier::BiofuelMix { .. } => {
            let cop = table_p9_cop(variant, true);
            engine_efficiency(engine).map(|shaft| (cop * shaft, cop))
        }
        _ => None,
    }
}

/// The generation efficiency of a generator where its kind has one.
fn efficiency_of(kind: &GeneratorKind, function: SystemFunction) -> Option<f64> {
    match kind {
        GeneratorKind::Combustion { efficiency, .. } => Some(*efficiency),
        GeneratorKind::HeatPump { efficiency, .. } => heat_pump_efficiency(efficiency),
        GeneratorKind::Boiler { efficiency, .. } => boiler_efficiency(efficiency, function, true),
        GeneratorKind::SolidBiomassBoiler { net_efficiency, .. } => {
            Some(biomass_efficiency(*net_efficiency))
        }
        GeneratorKind::Geothermal {
            source_temperature_c,
            return_temperature_c,
        } => Some(geothermal_efficiency(
            *source_temperature_c,
            *return_temperature_c,
        )),
        GeneratorKind::ElectricFlex { generator, .. } => flex_efficiency(generator),
        GeneratorKind::CompressionChiller {
            variant,
            drive,
            engine_efficiency,
        } => chiller_efficiency(*variant, drive, engine_efficiency.as_ref()).map(|(eta, _)| eta),
        GeneratorKind::FreeCooling { source, .. } => Some(source.efficiency()),
        _ => None,
    }
}

/// P.20/P.22 (P.37/P.39, P.48/P.50) for a fuel or electricity.
fn fuel_factors(carrier: &SystemCarrier, eta: f64) -> GenFactors {
    GenFactors {
        f: carrier.primary_factor() / eta,
        k: carrier.co2() / eta,
        pren: carrier_renewable(carrier),
        eta: Some(eta),
        renewable_drive: carrier.renewable_electricity_share() / eta,
        cop: None,
    }
}

/// 5.44/5.52 for heat pumps (η ≥ 1), 5.49 for cold (η ≥ 8).
fn heat_pump_renewable(function: SystemFunction, eta: f64) -> f64 {
    match function {
        SystemFunction::Cooling => {
            if eta >= 8.0 {
                1.0
            } else {
                0.0
            }
        }
        _ => {
            if eta >= 1.0 {
                1.0 - 1.0 / eta
            } else {
                0.0
            }
        }
    }
}

/// `(ε_chp;th, ε_chp;el)`: declared values override table P.6.
fn chp_conversion(
    thermal: Option<f64>,
    electrical: Option<f64>,
    reference_value: Option<&String>,
    table: Option<TableP6>,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<(f64, f64)> {
    let tabled = table.and_then(|input| {
        let values = table_p6(input);
        if values.is_none() {
            issues.push(issue("table_p6_out_of_range", format!("{path}.tableP6")));
        }
        values
    });
    if thermal.is_some() || electrical.is_some() {
        optional_reference(
            reference_value,
            format!("{path}.efficiencyReference"),
            issues,
        );
    }
    match (
        thermal.or(tabled.map(|v| v.0)),
        electrical.or(tabled.map(|v| v.1)),
    ) {
        (Some(th), Some(el)) if positive(th) && el.is_finite() && el >= 0.0 => Some((th, el)),
        (Some(_), Some(_)) => {
            issues.push(issue(
                "generator_efficiency_invalid",
                format!("{path}.thermalEfficiency"),
            ));
            None
        }
        _ => {
            if table.is_none() {
                issues.push(issue(
                    "chp_efficiency_required",
                    format!("{path}.thermalEfficiency"),
                ));
            }
            None
        }
    }
}

/// P.26/P.27 (P.27 has no MAX(0): a renewable fuel gives a negative K_CO2).
fn chp_factors(carrier: &SystemCarrier, thermal: f64, electrical: f64) -> (f64, f64) {
    (
        (carrier.primary_factor() - electrical * F_P_EL).max(0.0) / thermal,
        (carrier.co2() - electrical * k_co2_el()) / thermal,
    )
}

fn factors(
    kind: &GeneratorKind,
    function: SystemFunction,
    ctx: FactorContext,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<GenFactors> {
    let kpath = format!("{path}.kind");
    let cooling_only = matches!(
        kind,
        GeneratorKind::CompressionChiller { .. }
            | GeneratorKind::FreeCooling { .. }
            | GeneratorKind::SorptionChiller { .. }
    );
    let heat_only = matches!(
        kind,
        GeneratorKind::Boiler { .. }
            | GeneratorKind::SolidBiomassBoiler { .. }
            | GeneratorKind::CollectiveSolar { .. }
            | GeneratorKind::ElectricFlex { .. }
            | GeneratorKind::Geothermal { .. }
    );
    if cooling_only && function != SystemFunction::Cooling {
        issues.push(issue("generator_kind_requires_cooling", kpath));
        return None;
    }
    if heat_only && function == SystemFunction::Cooling {
        issues.push(issue("generator_kind_not_for_cooling", kpath));
        return None;
    }
    match kind {
        GeneratorKind::Combustion {
            carrier,
            efficiency,
            efficiency_reference,
        } => {
            reference(
                efficiency_reference,
                format!("{kpath}.efficiencyReference"),
                issues,
            );
            if !carrier.valid() {
                issues.push(issue("carrier_share_invalid", format!("{kpath}.carrier")));
            }
            if !positive(*efficiency) {
                issues.push(issue(
                    "generator_efficiency_invalid",
                    format!("{kpath}.efficiency"),
                ));
                return None;
            }
            Some(fuel_factors(carrier, *efficiency))
        }
        GeneratorKind::HeatPump { efficiency, drive } => {
            match efficiency {
                HeatPumpEfficiency::Declared {
                    source_reference, ..
                } => reference(
                    source_reference,
                    format!("{kpath}.efficiency.sourceReference"),
                    issues,
                ),
                HeatPumpEfficiency::TableP5 { source, .. } => {
                    let electric_drive = matches!(drive, SystemCarrier::Electricity { .. });
                    if source.electric() != electric_drive {
                        issues.push(issue(
                            "heat_pump_drive_table_mismatch",
                            format!("{kpath}.drive"),
                        ));
                    }
                    // P.6.6.5.4: no forfait values for hot water.
                    if function != SystemFunction::Heating {
                        issues.push(issue(
                            "table_p5_heating_only",
                            format!("{kpath}.efficiency"),
                        ));
                    }
                }
            }
            if !drive.valid()
                || !matches!(
                    drive,
                    SystemCarrier::Electricity { .. } | SystemCarrier::NaturalGas
                )
            {
                issues.push(issue("heat_pump_drive_invalid", format!("{kpath}.drive")));
            }
            let Some(eta) = heat_pump_efficiency(efficiency).filter(|value| positive(*value))
            else {
                issues.push(issue(
                    "generator_efficiency_invalid",
                    format!("{kpath}.efficiency"),
                ));
                return None;
            };
            Some(GenFactors {
                pren: heat_pump_renewable(function, eta),
                cop: Some(eta),
                ..fuel_factors(drive, eta)
            })
        }
        GeneratorKind::ChpWithoutLoss {
            carrier,
            thermal_efficiency,
            electrical_efficiency,
            efficiency_reference,
            table_p6,
        } => {
            let (thermal, electrical) = chp_conversion(
                *thermal_efficiency,
                *electrical_efficiency,
                efficiency_reference.as_ref(),
                *table_p6,
                &kpath,
                issues,
            )?;
            let (f, k) = chp_factors(carrier, thermal, electrical);
            // Renewable share 5.45/5.46.
            Some(GenFactors {
                f,
                k,
                pren: carrier_renewable(carrier),
                ..GenFactors::default()
            })
        }
        GeneratorKind::ChpWithLoss {
            carrier,
            loss_ratio,
            loss_ratio_reference,
        } => {
            let ratio = match loss_ratio {
                Some(ratio) => {
                    optional_reference(
                        loss_ratio_reference.as_ref(),
                        format!("{kpath}.lossRatioReference"),
                        issues,
                    );
                    *ratio
                }
                None => CHP_LOSS_RATIO,
            };
            if !positive(ratio) {
                issues.push(issue(
                    "chp_loss_ratio_invalid",
                    format!("{kpath}.lossRatio"),
                ));
                return None;
            }
            // P.28/P.30; AVI heat uses f_P;del;wi (P.6.5.4.6); renewable
            // share 5.45/5.46.
            Some(GenFactors {
                f: carrier.primary_factor() * ratio * F_P_EL,
                k: carrier.co2() * ratio * F_P_EL,
                pren: carrier_renewable(carrier),
                ..GenFactors::default()
            })
        }
        GeneratorKind::ResidualHeat {
            auxiliary_specific,
            auxiliary_reference,
        } => {
            let aux = match auxiliary_specific {
                Some(value) => {
                    optional_reference(
                        auxiliary_reference.as_ref(),
                        format!("{kpath}.auxiliaryReference"),
                        issues,
                    );
                    *value
                }
                None => RESIDUAL_HEAT_AUX,
            };
            if !aux.is_finite() || !(0.0..1.0).contains(&aux) {
                issues.push(issue(
                    "residual_heat_auxiliary_invalid",
                    format!("{kpath}.auxiliarySpecific"),
                ));
                return None;
            }
            // P.6.5.4.7 and 5.47/5.55.
            Some(GenFactors {
                f: aux * F_P_EL,
                k: aux * k_co2_el(),
                pren: 1.0 - aux,
                ..GenFactors::default()
            })
        }
        GeneratorKind::Geothermal {
            source_temperature_c,
            return_temperature_c,
        } => {
            let eta = geothermal_efficiency(*source_temperature_c, *return_temperature_c);
            if !positive(eta) {
                issues.push(issue(
                    "geothermal_temperature_invalid",
                    format!("{kpath}.sourceTemperatureC"),
                ));
                return None;
            }
            // 5.48 generalised to 1 − 1/η.
            Some(GenFactors {
                f: F_P_EL / eta,
                k: k_co2_el() / eta,
                pren: heat_pump_renewable(function, eta),
                eta: Some(eta),
                ..GenFactors::default()
            })
        }
        GeneratorKind::Declared {
            primary_factor,
            co2_kg_per_kwh,
            renewable_factor,
            source_reference,
        } => {
            reference(source_reference, format!("{kpath}.sourceReference"), issues);
            if !primary_factor.is_finite()
                || *primary_factor < 0.0
                || !co2_kg_per_kwh.is_finite()
                || !(0.0..=1.0).contains(renewable_factor)
            {
                issues.push(issue("generator_factor_invalid", kpath));
                return None;
            }
            Some(GenFactors {
                f: *primary_factor,
                k: *co2_kg_per_kwh,
                pren: *renewable_factor,
                ..GenFactors::default()
            })
        }
        GeneratorKind::Boiler {
            carrier,
            efficiency,
        } => {
            if !carrier.valid()
                || !matches!(
                    carrier,
                    SystemCarrier::NaturalGas
                        | SystemCarrier::Oil
                        | SystemCarrier::Biogas
                        | SystemCarrier::BiofuelMix { .. }
                )
            {
                issues.push(issue("boiler_carrier_invalid", format!("{kpath}.carrier")));
            }
            match efficiency {
                BoilerEfficiency::TableP3 {
                    boiler,
                    temperature_level,
                    emission,
                } => {
                    if function == SystemFunction::Heating
                        && temperature_level.is_none()
                        && emission.is_none()
                    {
                        issues.push(issue(
                            "temperature_level_required",
                            format!("{kpath}.efficiency"),
                        ));
                        return None;
                    }
                    // Table P.3: an oil boiler is a conventional boiler.
                    if *carrier == SystemCarrier::Oil && *boiler != BoilerClass::Conventional {
                        issues.push(issue(
                            "oil_boiler_conventional_only",
                            format!("{kpath}.efficiency.boiler"),
                        ));
                    }
                }
                BoilerEfficiency::FullLoad {
                    source_reference, ..
                } => {
                    reference(
                        source_reference,
                        format!("{kpath}.efficiency.sourceReference"),
                        issues,
                    );
                    // P.6.6.5.2 (p. 994): deviating hot-water values only for
                    // non-preferred boilers (bijstook).
                    if function == SystemFunction::HotWater && ctx.preferred {
                        issues.push(issue(
                            "hot_water_full_load_requires_non_preferred",
                            format!("{kpath}.efficiency"),
                        ));
                    }
                }
            }
            let Some(eta) =
                boiler_efficiency(efficiency, function, ctx.preferred).filter(|v| positive(*v))
            else {
                issues.push(issue(
                    "generator_efficiency_invalid",
                    format!("{kpath}.efficiency"),
                ));
                return None;
            };
            Some(fuel_factors(carrier, eta))
        }
        GeneratorKind::SolidBiomassBoiler {
            carrier,
            net_efficiency,
            source_reference,
        } => {
            reference(source_reference, format!("{kpath}.sourceReference"), issues);
            if !carrier.valid() {
                issues.push(issue("carrier_share_invalid", format!("{kpath}.carrier")));
            }
            let eta = biomass_efficiency(*net_efficiency);
            if !positive(*net_efficiency) || !positive(eta) {
                issues.push(issue(
                    "generator_efficiency_invalid",
                    format!("{kpath}.netEfficiency"),
                ));
                return None;
            }
            Some(fuel_factors(carrier, eta))
        }
        // P.6.5.4.10.2: free heat, factor 0, fully renewable.
        GeneratorKind::CollectiveSolar { .. } => Some(GenFactors {
            pren: 1.0,
            ..GenFactors::default()
        }),
        GeneratorKind::ElectricFlex {
            generator,
            flex_heat_kwh,
            flex_reference,
            network_production_kwh,
            connections,
            heat_buffer,
            registration_reference,
        } => {
            reference(
                registration_reference,
                format!("{kpath}.registrationReference"),
                issues,
            );
            // 5.8: at least 500 connections and a heat buffer.
            if *connections < 500 || !*heat_buffer {
                issues.push(issue("flex_mode_conditions_not_met", kpath.clone()));
            }
            match generator {
                FlexGenerator::ElectrodeBoiler {
                    efficiency: Some(_),
                    efficiency_reference,
                } => optional_reference(
                    efficiency_reference.as_ref(),
                    format!("{kpath}.generator.efficiencyReference"),
                    issues,
                ),
                FlexGenerator::HeatPump {
                    efficiency:
                        HeatPumpEfficiency::Declared {
                            source_reference, ..
                        },
                } => reference(
                    source_reference,
                    format!("{kpath}.generator.efficiency.sourceReference"),
                    issues,
                ),
                _ => {}
            }
            let Some(eta) = flex_efficiency(generator).filter(|v| positive(*v)) else {
                issues.push(issue(
                    "generator_efficiency_invalid",
                    format!("{kpath}.generator.efficiency"),
                ));
                return None;
            };
            let flex = match flex_heat_kwh {
                Some(value) => {
                    optional_reference(
                        flex_reference.as_ref(),
                        format!("{kpath}.flexReference"),
                        issues,
                    );
                    if !value.is_finite() || *value < 0.0 {
                        issues.push(issue("value_invalid", format!("{kpath}.flexHeatKwh")));
                        return None;
                    }
                    *value
                }
                None => match ctx.nominal_power_kw {
                    Some(power) if positive(power) => {
                        let network = match network_production_kwh {
                            Some(value) if value.is_finite() && *value >= 0.0 => *value,
                            Some(_) => {
                                issues.push(issue(
                                    "value_invalid",
                                    format!("{kpath}.networkProductionKwh"),
                                ));
                                return None;
                            }
                            None => ctx.network_heat_kwh,
                        };
                        (power * FLEX_HOURS).min(FLEX_MAX_SHARE * network)
                    }
                    _ => {
                        issues.push(issue(
                            "generator_power_required",
                            format!("{path}.nominalPowerKw"),
                        ));
                        return None;
                    }
                },
            };
            let share = if ctx.heat_kwh > 0.0 {
                (flex / ctx.heat_kwh).min(1.0)
            } else {
                0.0
            };
            // P.6.5.4.11 with f_P;del;flex = 0, K_CO2;del;flex = 0 (tables
            // 5.5/5.6) and 5.8.3.1 g). f_Pren = 1 − (1 − share)/η is
            // clamped at 0: below the break-even share the electricity
            // outside the flex hours exceeds the heat, which is no
            // negative renewable share (interpretation).
            Some(GenFactors {
                f: (1.0 - share) * F_P_EL / eta,
                k: (1.0 - share) * k_co2_el() / eta,
                pren: (1.0 - (1.0 - share) / eta).max(0.0),
                eta: Some(eta),
                ..GenFactors::default()
            })
        }
        GeneratorKind::CompressionChiller {
            variant,
            drive,
            engine_efficiency,
        } => {
            if !drive.valid() {
                issues.push(issue("carrier_share_invalid", format!("{kpath}.drive")));
            }
            match (drive, engine_efficiency) {
                (SystemCarrier::Electricity { .. }, None) => {}
                (SystemCarrier::Electricity { .. }, Some(_)) => issues.push(issue(
                    "engine_efficiency_only_for_gas",
                    format!("{kpath}.engineEfficiency"),
                )),
                (
                    SystemCarrier::NaturalGas
                    | SystemCarrier::Biogas
                    | SystemCarrier::BiofuelMix { .. },
                    engine,
                ) => match engine {
                    None => issues.push(issue(
                        "engine_efficiency_required",
                        format!("{kpath}.engineEfficiency"),
                    )),
                    Some(EngineEfficiency::Declared {
                        source_reference, ..
                    }) => reference(
                        source_reference,
                        format!("{kpath}.engineEfficiency.sourceReference"),
                        issues,
                    ),
                    Some(EngineEfficiency::TableP6(input)) => {
                        if table_p6(*input).is_none() {
                            issues.push(issue(
                                "table_p6_out_of_range",
                                format!("{kpath}.engineEfficiency"),
                            ));
                        }
                    }
                },
                _ => issues.push(issue("chiller_drive_invalid", format!("{kpath}.drive"))),
            }
            let (eta, cop) = chiller_efficiency(*variant, drive, engine_efficiency.as_ref())
                .filter(|(eta, _)| positive(*eta))?;
            Some(GenFactors {
                pren: heat_pump_renewable(function, eta),
                cop: Some(cop),
                ..fuel_factors(drive, eta)
            })
        }
        GeneratorKind::FreeCooling { source, drive } => {
            if !drive.valid() || !matches!(drive, SystemCarrier::Electricity { .. }) {
                issues.push(issue("chiller_drive_invalid", format!("{kpath}.drive")));
                return None;
            }
            let eta = source.efficiency();
            Some(GenFactors {
                pren: heat_pump_renewable(function, eta),
                ..fuel_factors(drive, eta)
            })
        }
        GeneratorKind::SorptionChiller { heat } => match heat {
            SorptionHeat::CollectiveHeat {
                primary_factor,
                co2_kg_per_kwh,
                source_reference,
            } => {
                reference(
                    source_reference,
                    format!("{kpath}.heat.sourceReference"),
                    issues,
                );
                if !primary_factor.is_finite()
                    || *primary_factor < 0.0
                    || !co2_kg_per_kwh.is_finite()
                {
                    issues.push(issue("generator_factor_invalid", format!("{kpath}.heat")));
                    return None;
                }
                Some(GenFactors {
                    f: primary_factor / SORPTION_COP_COLLECTIVE,
                    k: co2_kg_per_kwh / SORPTION_COP_COLLECTIVE,
                    cop: Some(SORPTION_COP_COLLECTIVE),
                    ..GenFactors::default()
                })
            }
            SorptionHeat::Chp(data) => {
                let (thermal, electrical) = chp_conversion(
                    data.thermal_efficiency,
                    data.electrical_efficiency,
                    data.efficiency_reference.as_ref(),
                    data.table_p6,
                    &format!("{kpath}.heat"),
                    issues,
                )?;
                let (f, k) = chp_factors(&data.carrier, thermal, electrical);
                Some(GenFactors {
                    f: f / SORPTION_COP_CHP,
                    k: k / SORPTION_COP_CHP,
                    cop: Some(SORPTION_COP_CHP),
                    ..GenFactors::default()
                })
            }
        },
    }
}

/// `f_Pren;XD;gi` of a fuel-fired generator or CHP: 5.43, 5.45 (note 6: a
/// fully biogenic fuel counts 1), 5.46 (and 5.51/5.53/5.54).
fn carrier_renewable(carrier: &SystemCarrier) -> f64 {
    match carrier {
        SystemCarrier::Biogas | SystemCarrier::BiomassAbove500Kw => 1.0,
        SystemCarrier::BiofuelMix { biofuel_share } => *biofuel_share,
        SystemCarrier::WasteIncineration => 1.0 - carrier.primary_factor(),
        _ => 0.0,
    }
}

/// Annual delivery and, where known, its monthly split.
#[derive(Debug, Clone, Copy)]
struct Demand {
    annual: f64,
    monthly: Option<Months>,
}

fn hours_profile(annual: f64) -> Months {
    std::array::from_fn(|index| annual * MONTH_HOURS[index] / YEAR_HOURS)
}

/// P.74 with `max(0, θ_in;ref − θ_e;avg;mi)`.
fn degree_profile(annual: f64) -> Months {
    let weights: Months =
        std::array::from_fn(|index| (REFERENCE_INDOOR_C - OUTDOOR_TEMPERATURE_C[index]).max(0.0));
    let sum: f64 = weights.iter().sum();
    weights.map(|weight| annual * weight / sum)
}

/// Monthly values from an annual value: P.74 for heat, by month length
/// for hot water (as P.82), none for cold.
fn annual_profile(function: SystemFunction, annual: f64) -> Option<Months> {
    match function {
        SystemFunction::Heating => Some(degree_profile(annual)),
        SystemFunction::HotWater => Some(hours_profile(annual)),
        SystemFunction::Cooling => None,
    }
}

type Part = Option<(f64, Option<Months>)>;

const ZERO_PART: Part = Some((0.0, Some([0.0; 12])));

fn plot_flow(flow: &PlotFlow, path: String, issues: &mut Vec<AnnexPIssue>) -> Part {
    let valid = |value: f64| value.is_finite() && value >= 0.0;
    let monthly = match flow.monthly_kwh.len() {
        0 => None,
        12 => {
            if !flow.monthly_kwh.iter().all(|value| valid(*value)) {
                issues.push(issue("value_invalid", format!("{path}.monthlyKwh")));
                return None;
            }
            let mut months = [0.0; 12];
            months.copy_from_slice(&flow.monthly_kwh);
            Some(months)
        }
        _ => {
            issues.push(issue(
                "monthly_values_require_twelve",
                format!("{path}.monthlyKwh"),
            ));
            return None;
        }
    };
    match (flow.annual_kwh, monthly) {
        (Some(annual), _) if !valid(annual) => {
            issues.push(issue("value_invalid", format!("{path}.annualKwh")));
            None
        }
        (Some(annual), Some(months)) => {
            let sum: f64 = months.iter().sum();
            if (sum - annual).abs() > 1e-6 * annual.max(1.0) {
                issues.push(issue("monthly_sum_mismatch", path));
                return None;
            }
            Some((annual, Some(months)))
        }
        (Some(annual), None) => Some((annual, None)),
        (None, Some(months)) => Some((months.iter().sum(), Some(months))),
        (None, None) => {
            issues.push(issue("plot_flow_required", path));
            None
        }
    }
}

fn plot_area(plot: &AreaPlot, path: &str, issues: &mut Vec<AnnexPIssue>) -> Option<f64> {
    match plot.usable_area_m2 {
        Some(area) if positive(area) => Some(area),
        _ => {
            issues.push(issue(
                "usable_area_required",
                format!("{path}.usableAreaM2"),
            ));
            None
        }
    }
}

/// `E_H;dh;pi`: supplied, or P.79/P.80 with table P.14.
fn heating_part(plot: &AreaPlot, path: &str, issues: &mut Vec<AnnexPIssue>) -> Part {
    if let Some(flow) = &plot.heating {
        return plot_flow(flow, format!("{path}.heating"), issues);
    }
    match plot.heating_forfait {
        Some(kind) => {
            let area = plot_area(plot, path, issues)?;
            let months = table_p14(kind).map(|specific| specific * area);
            Some((months.iter().sum(), Some(months)))
        }
        None => ZERO_PART,
    }
}

/// `E_W;dh;pi`: supplied, or P.81–P.83 with table P.15.
fn hot_water_part(plot: &AreaPlot, path: &str, issues: &mut Vec<AnnexPIssue>) -> Part {
    if let Some(flow) = &plot.hot_water {
        return plot_flow(flow, format!("{path}.hotWater"), issues);
    }
    match plot.hot_water_forfait {
        Some(usage) => {
            let area = plot_area(plot, path, issues)?;
            let months: Months = std::array::from_fn(|index| {
                usage.specific_kwh_per_m2() * area * MONTH_HOURS[index] / YEAR_HOURS
            });
            Some((months.iter().sum(), Some(months)))
        }
        None => ZERO_PART,
    }
}

fn optional_part(flow: Option<&PlotFlow>, path: String, issues: &mut Vec<AnnexPIssue>) -> Part {
    match flow {
        Some(flow) => plot_flow(flow, path, issues),
        None => ZERO_PART,
    }
}

/// P.72/P.73 (heat), P.75/P.76 (hot water), P.77/P.78 (cold).
fn area_demand(
    area: &AreaDemand,
    function: SystemFunction,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<Demand> {
    if area.plots.is_empty() {
        issues.push(issue("area_plot_required", format!("{path}.plots")));
        return None;
    }
    let mut annual = 0.0;
    let mut monthly = [0.0; 12];
    let mut all_monthly = true;
    let mut ok = true;
    for (index, plot) in area.plots.iter().enumerate() {
        let ppath = format!("{path}.plots[{index}]");
        reference(
            &plot.source_reference,
            format!("{ppath}.sourceReference"),
            issues,
        );
        // Each part with the function whose profile it takes when it has
        // no months, and whether it belongs to the monthly sum. For heat
        // every part without months takes P.74, the only monthly rule the
        // norm gives for Q_HD;nd;tot (p. 1020), so an annual-only sorption
        // part does not replace the months of the other parts. P.78
        // (p. 1022) sums only E_C;dc;mi, so dehumidification counts in the
        // annual total (P.77) but not in the monthly cold profile.
        let parts = match function {
            SystemFunction::Heating => vec![
                (
                    heating_part(plot, &ppath, issues),
                    SystemFunction::Heating,
                    true,
                ),
                (
                    optional_part(
                        plot.sorption_cooling.as_ref(),
                        format!("{ppath}.sorptionCooling"),
                        issues,
                    ),
                    SystemFunction::Heating,
                    true,
                ),
                (
                    if plot.hot_water_via_delivery_set {
                        hot_water_part(plot, &ppath, issues)
                    } else {
                        ZERO_PART
                    },
                    SystemFunction::HotWater,
                    true,
                ),
            ],
            SystemFunction::HotWater => vec![(
                hot_water_part(plot, &ppath, issues),
                SystemFunction::HotWater,
                true,
            )],
            SystemFunction::Cooling => vec![
                (
                    optional_part(plot.cooling.as_ref(), format!("{ppath}.cooling"), issues),
                    SystemFunction::Cooling,
                    true,
                ),
                (
                    optional_part(
                        plot.dehumidification.as_ref(),
                        format!("{ppath}.dehumidification"),
                        issues,
                    ),
                    SystemFunction::Cooling,
                    false,
                ),
            ],
        };
        for (part, profile, in_monthly) in parts {
            match part {
                Some((value, months)) => {
                    annual += value;
                    if !in_monthly {
                        continue;
                    }
                    // P.74 (P.82 for hot water) only for the parts without
                    // months; the supplied months of the others stay.
                    match months.or_else(|| annual_profile(profile, value)) {
                        Some(months) => {
                            for (total, value) in monthly.iter_mut().zip(months) {
                                *total += value;
                            }
                        }
                        None => all_monthly = false,
                    }
                }
                None => ok = false,
            }
        }
    }
    // Parts left out of the monthly sum (dehumidification, P.78) can leave a
    // zero profile next to a positive annual total; those months are not
    // known, so the annual profile applies as for parts without months.
    let known = all_monthly && !(monthly.iter().sum::<f64>() <= 0.0 && annual > 0.0);
    ok.then(|| Demand {
        annual,
        monthly: if known {
            Some(monthly)
        } else {
            annual_profile(function, annual)
        },
    })
}

/// `Q_XD;out;tot`: supplied (keeping the shape of the area profile) or from
/// the area.
fn delivered(
    system: &CalculatedSystem,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<Demand> {
    let area = system
        .area_demand
        .as_ref()
        .and_then(|area| area_demand(area, system.function, &format!("{path}.areaDemand"), issues));
    match system.delivered_kwh {
        Some(value) => {
            if !positive(value) {
                issues.push(issue(
                    "delivered_energy_invalid",
                    format!("{path}.deliveredKwh"),
                ));
                return None;
            }
            let monthly = match area {
                Some(Demand {
                    annual,
                    monthly: Some(months),
                }) if annual > 0.0 => Some(months.map(|month| month * value / annual)),
                _ => annual_profile(system.function, value),
            };
            Some(Demand {
                annual: value,
                monthly,
            })
        }
        None => match area {
            Some(demand) if positive(demand.annual) => Some(demand),
            Some(_) => {
                issues.push(issue(
                    "delivered_energy_invalid",
                    format!("{path}.areaDemand"),
                ));
                None
            }
            None => {
                if system.area_demand.is_none() {
                    issues.push(issue(
                        "delivered_energy_required",
                        format!("{path}.deliveredKwh"),
                    ));
                }
                None
            }
        },
    }
}

fn ambient_c(ambient: PipeAmbient, outdoor_c: f64) -> f64 {
    match ambient {
        PipeAmbient::Outdoor => outdoor_c,
        // P.16.
        PipeAmbient::Crawlspace => 11.2 + 0.08 * outdoor_c,
        PipeAmbient::Indoor { temperature_c } => temperature_c,
    }
}

fn placement_ambient(placement: PipePlacement) -> PipeAmbient {
    match placement {
        PipePlacement::Buried { ambient, .. } | PipePlacement::InAir { ambient, .. } => ambient,
    }
}

/// P.17/P.18: `R_XD;dis;j`, K·m/W.
pub fn pipe_resistance(
    segment: &PipeSegment,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<f64> {
    if let Some(resistance) = segment.resistance_km_per_w {
        if positive(resistance) {
            return Some(resistance);
        }
        issues.push(issue("value_invalid", format!("{path}.resistanceKmPerW")));
        return None;
    }
    let Some(last) = segment.layers.last() else {
        issues.push(issue("pipe_layers_required", format!("{path}.layers")));
        return None;
    };
    let mut layers = 0.0;
    for (index, layer) in segment.layers.iter().enumerate() {
        if !positive(layer.conductivity_w_per_mk)
            || !positive(layer.inner_diameter_m)
            || !(layer.outer_diameter_m.is_finite()
                && layer.outer_diameter_m > layer.inner_diameter_m)
        {
            issues.push(issue(
                "pipe_layer_invalid",
                format!("{path}.layers[{index}]"),
            ));
            return None;
        }
        // λ rounded up to 0,001.
        let lambda = (layer.conductivity_w_per_mk * 1000.0 - 1e-9).ceil() / 1000.0;
        layers += (layer.outer_diameter_m / layer.inner_diameter_m).ln()
            / (2.0 * std::f64::consts::PI * lambda);
    }
    let outer = last.outer_diameter_m;
    let correction = match (segment.correction_factor, segment.correction) {
        (Some(factor), _) if positive(factor) => factor,
        (Some(_), _) => {
            issues.push(issue("value_invalid", format!("{path}.correctionFactor")));
            return None;
        }
        (None, Some(case)) => case.factor(),
        (None, None) => match segment.placement {
            PipePlacement::InAir { .. } => PipeCorrection::SurfaceOrRecessed.factor(),
            PipePlacement::Buried { .. } => {
                issues.push(issue(
                    "pipe_correction_required",
                    format!("{path}.correction"),
                ));
                return None;
            }
        },
    };
    let outside = match segment.placement {
        PipePlacement::Buried {
            cover_depth_m,
            ground_conductivity,
            ambient,
        } => {
            if matches!(ambient, PipeAmbient::Indoor { .. }) {
                issues.push(issue(
                    "buried_pipe_ambient_invalid",
                    format!("{path}.placement.ambient"),
                ));
                return None;
            }
            let lambda = ground_conductivity.unwrap_or(GROUND_CONDUCTIVITY);
            let ratio = 4.0 * cover_depth_m / outer;
            if !positive(lambda) || !(ratio.is_finite() && ratio > 1.0) {
                issues.push(issue("value_invalid", format!("{path}.placement")));
                return None;
            }
            ratio.ln() / (2.0 * std::f64::consts::PI * lambda)
        }
        PipePlacement::InAir {
            ambient,
            surface_coefficient,
        } => {
            let coefficient = surface_coefficient.unwrap_or(match ambient {
                PipeAmbient::Outdoor => SURFACE_COEFFICIENT_OTHER,
                _ => SURFACE_COEFFICIENT_ENCLOSED,
            });
            if !positive(coefficient) {
                issues.push(issue("value_invalid", format!("{path}.placement")));
                return None;
            }
            1.0 / (std::f64::consts::PI * coefficient * outer)
        }
    };
    Some(correction * (layers + outside))
}

/// P.13–P.15 per month, kWh.
fn pipe_losses(
    segments: &[PipeSegment],
    temperature: &NetworkWaterTemperature,
    cold: bool,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<Months> {
    let mut items = Vec::new();
    let mut ok = true;
    for (index, segment) in segments.iter().enumerate() {
        let spath = format!("{path}.segments[{index}]");
        if !positive(segment.length_m) {
            issues.push(issue("value_invalid", format!("{spath}.lengthM")));
            ok = false;
        }
        match pipe_resistance(segment, &spath, issues) {
            Some(resistance) => items.push((
                segment.length_m,
                resistance,
                placement_ambient(segment.placement),
            )),
            None => ok = false,
        }
    }
    match temperature {
        NetworkWaterTemperature::Monthly { temperatures_c } if temperatures_c.len() != 12 => {
            issues.push(issue(
                "monthly_values_require_twelve",
                format!("{path}.waterTemperature.temperaturesC"),
            ));
            ok = false;
        }
        // P.14: null marks a month out of operation; a network that is
        // never in operation has no loss, so an all-null row is an
        // unfilled input rather than a network out of operation.
        NetworkWaterTemperature::Monthly { temperatures_c }
            if temperatures_c.iter().all(Option::is_none) =>
        {
            issues.push(issue(
                "network_temperature_required",
                format!("{path}.waterTemperature.temperaturesC"),
            ));
            ok = false;
        }
        NetworkWaterTemperature::OutdoorBins { curve, .. } if curve.is_empty() => {
            issues.push(issue(
                "heating_curve_required",
                format!("{path}.waterTemperature.curve"),
            ));
            ok = false;
        }
        _ => {}
    }
    if !ok {
        return None;
    }
    // W for a water and an outdoor temperature; cold networks gain heat.
    let watts = |water: f64, outdoor: f64| -> f64 {
        items
            .iter()
            .map(|(length, resistance, ambient)| {
                let ambient = ambient_c(*ambient, outdoor);
                let difference = if cold {
                    ambient - water
                } else {
                    water - ambient
                };
                length * difference / resistance
            })
            .sum()
    };
    let monthly = |water: Option<f64>, index: usize| -> f64 {
        water.map_or(0.0, |water| {
            (watts(water, OUTDOOR_TEMPERATURE_C[index]) * MONTH_HOURS[index] / 1000.0).max(0.0)
        })
    };
    Some(match temperature {
        NetworkWaterTemperature::Constant { temperature_c } => {
            std::array::from_fn(|index| monthly(Some(*temperature_c), index))
        }
        NetworkWaterTemperature::Monthly { temperatures_c } => {
            std::array::from_fn(|index| monthly(temperatures_c[index], index))
        }
        NetworkWaterTemperature::OutdoorBins {
            curve,
            off_above_outdoor_c,
        } => {
            let mut points: Vec<(f64, f64)> = curve
                .iter()
                .map(|point| (point.outdoor_c, point.water_c))
                .collect();
            points.sort_by(|a, b| a.0.total_cmp(&b.0));
            let annual: f64 = TABLE_P16
                .iter()
                .filter(|(outdoor, _)| off_above_outdoor_c.map_or(true, |off| *outdoor <= off))
                .map(|(outdoor, hours)| {
                    watts(interpolate(&points, *outdoor), *outdoor) * hours / 1000.0
                })
                .sum();
            hours_profile(annual.max(0.0))
        }
    })
}

/// P.43 with P.44, kWh per year.
pub fn vessel_loss_kwh(
    vessel: &StorageVessel,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<f64> {
    let water = vessel.water_temperature_c.unwrap_or(VESSEL_WATER_C);
    let ambient = vessel.ambient_temperature_c.unwrap_or(VESSEL_AMBIENT_C);
    if !water.is_finite() || !ambient.is_finite() {
        issues.push(issue("value_invalid", format!("{path}.waterTemperatureC")));
        return None;
    }
    let difference = water - ambient;
    if let Some(standby) = vessel.standby_loss_kwh_per_day {
        return match vessel.standby_test_difference_k {
            Some(test) if positive(test) && standby.is_finite() && standby >= 0.0 => {
                Some(difference / test * standby * 365.0)
            }
            _ => {
                issues.push(issue(
                    "standby_test_difference_required",
                    format!("{path}.standbyTestDifferenceK"),
                ));
                None
            }
        };
    }
    let surface = match (
        vessel.surface_m2,
        vessel.volume_l,
        vessel.diameter_at_least_50_cm,
    ) {
        (Some(surface), _, _) if positive(surface) => surface,
        (None, Some(volume), Some(large)) if positive(volume) => {
            volume / if large { 100.0 } else { 200.0 }
        }
        _ => {
            issues.push(issue(
                "vessel_surface_required",
                format!("{path}.surfaceM2"),
            ));
            return None;
        }
    };
    let alpha = match (vessel.loss_factor_w_per_m2k, vessel.insulation) {
        (Some(alpha), _) if alpha.is_finite() && alpha >= 0.0 => alpha,
        (None, Some(insulation)) => insulation.loss_factor(),
        _ => {
            issues.push(issue(
                "vessel_insulation_required",
                format!("{path}.insulation"),
            ));
            return None;
        }
    };
    Some(surface * alpha * difference * 24.0 / 1000.0 * 365.0)
}

/// P.45/P.46 with table P.7, kWh per year.
fn charging_losses(
    pipes: &[ChargingPipe],
    exchanger: Option<&ExternalExchanger>,
    circulation_c: f64,
    correction: f64,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<f64> {
    let mut watts = 0.0;
    let mut ok = true;
    for (index, pipe) in pipes.iter().enumerate() {
        let u_value = match (
            pipe.u_value_w_per_mk,
            pipe.outer_diameter_mm,
            pipe.insulation_mm,
        ) {
            (Some(u), _, _) => Some(u).filter(|u| u.is_finite() && *u >= 0.0),
            (None, Some(diameter), Some(insulation)) => table_p7(diameter, insulation),
            _ => None,
        };
        match u_value {
            Some(u) if positive(pipe.length_m) => {
                let ambient = pipe.ambient_temperature_c.unwrap_or(CHARGE_AMBIENT_C);
                watts += pipe.length_m * u * (circulation_c - ambient) * correction;
            }
            _ => {
                issues.push(issue(
                    "charging_pipe_invalid",
                    format!("{path}.pipes[{index}]"),
                ));
                ok = false;
            }
        }
    }
    if let Some(exchanger) = exchanger {
        let specific = exchanger
            .specific_loss_w_per_kw
            .unwrap_or(if exchanger.insulated { 0.2 } else { 1.3 });
        if positive(exchanger.nominal_power_kw) && specific.is_finite() && specific >= 0.0 {
            watts += specific * exchanger.nominal_power_kw;
        } else {
            issues.push(issue("value_invalid", format!("{path}.exchanger")));
            ok = false;
        }
    }
    ok.then_some(watts * YEAR_HOURS / 1000.0)
}

/// `Q_XD;in;tot` and the monthly loss where the method gives one.
struct DistributionResult {
    input: f64,
    monthly_loss: Option<Months>,
}

/// P.10–P.18 and P.6.4.
fn distribution(
    system: &CalculatedSystem,
    out: f64,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<DistributionResult> {
    let dpath = format!("{path}.distribution");
    let result = match &system.distribution {
        SystemDistribution::Flows {
            input_kwh,
            loss_kwh,
            source_reference,
        } => {
            reference(source_reference, format!("{dpath}.sourceReference"), issues);
            match (input_kwh, loss_kwh) {
                (Some(input), None) => Some(DistributionResult {
                    input: *input,
                    monthly_loss: None,
                }),
                (None, Some(loss)) => Some(DistributionResult {
                    input: out + loss,
                    monthly_loss: None,
                }),
                _ => {
                    issues.push(issue("distribution_two_of_three_flows_required", dpath));
                    return None;
                }
            }
        }
        SystemDistribution::SmallSystemForfait {
            connections,
            connection_type,
            design_temperature,
            other_loss_kwh,
        } => {
            if system.function == SystemFunction::Cooling || *connections == 0 {
                issues.push(issue("small_system_forfait_invalid", dpath.clone()));
            }
            if !other_loss_kwh.is_finite() || *other_loss_kwh < 0.0 {
                issues.push(issue("value_invalid", format!("{dpath}.otherLossKwh")));
            }
            // Table P.0 footnote b: WD systems use 90/60.
            if system.function == SystemFunction::HotWater
                && design_temperature.is_some_and(|t| t != NetworkTemperature::T90To60)
            {
                issues.push(issue(
                    "hot_water_small_system_requires_90_60",
                    format!("{dpath}.designTemperature"),
                ));
            }
            let temperature = design_temperature.unwrap_or(NetworkTemperature::T90To60);
            Some(DistributionResult {
                input: out
                    + table_p0(temperature, *connection_type) * f64::from(*connections)
                    + other_loss_kwh,
                monthly_loss: None,
            })
        }
        SystemDistribution::SmallColdForfait { supply_below_10_c } => {
            if system.function != SystemFunction::Cooling {
                issues.push(issue("small_cold_forfait_requires_cooling", dpath.clone()));
            }
            Some(DistributionResult {
                input: if *supply_below_10_c {
                    out * (1.0 + COLD_FORFAIT_LOSS_SHARE)
                } else {
                    out
                },
                monthly_loss: None,
            })
        }
        SystemDistribution::Pipes {
            segments,
            water_temperature,
            buffers,
            supply_below_10_c,
            other_loss_kwh,
            source_reference,
        } => {
            reference(source_reference, format!("{dpath}.sourceReference"), issues);
            let cold = system.function == SystemFunction::Cooling;
            if segments.is_empty() {
                issues.push(issue("pipe_segment_required", format!("{dpath}.segments")));
            }
            if cold && supply_below_10_c.is_none() {
                issues.push(issue(
                    "cold_supply_temperature_required",
                    format!("{dpath}.supplyBelow10C"),
                ));
            }
            if !other_loss_kwh.is_finite() || *other_loss_kwh < 0.0 {
                issues.push(issue("value_invalid", format!("{dpath}.otherLossKwh")));
            }
            let temperature = match (water_temperature, system.function) {
                (Some(temperature), _) => Some(temperature.clone()),
                (None, SystemFunction::HotWater) => Some(NetworkWaterTemperature::Constant {
                    temperature_c: WD_CIRCULATION_C,
                }),
                (None, _) => {
                    issues.push(issue(
                        "network_temperature_required",
                        format!("{dpath}.waterTemperature"),
                    ));
                    None
                }
            };
            let pipes = pipe_losses(segments, temperature.as_ref()?, cold, &dpath, issues);
            let mut buffer = 0.0;
            for (index, vessel) in buffers.iter().enumerate() {
                buffer += vessel_loss_kwh(vessel, &format!("{dpath}.buffers[{index}]"), issues)?;
            }
            let pipes = pipes?;
            // P.6.2.2: a cold network at 10 °C or more has no loss.
            let losses: Months = if cold && *supply_below_10_c != Some(true) {
                [0.0; 12]
            } else {
                let spread = hours_profile(buffer + other_loss_kwh);
                std::array::from_fn(|index| pipes[index] + spread[index])
            };
            Some(DistributionResult {
                input: out + losses.iter().sum::<f64>(),
                monthly_loss: Some(losses),
            })
        }
    };
    if let Some(item) = &result {
        if !item.input.is_finite() || item.input < out {
            issues.push(issue("distribution_flows_inconsistent", dpath));
        }
    }
    result
}

/// `η_WD;gen;sto`: P.34/P.35 and P.6.6.4.3; 1 for heat and cold.
fn storage_efficiency(
    system: &CalculatedSystem,
    input: f64,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<f64> {
    let spath = format!("{path}.hotWaterStorage");
    match (&system.hot_water_storage, system.function) {
        (None, SystemFunction::HotWater) => {
            issues.push(issue("hot_water_storage_efficiency_required", spath));
            None
        }
        (None, _) => Some(1.0),
        (Some(_), SystemFunction::Heating | SystemFunction::Cooling) => {
            issues.push(issue("hot_water_storage_only_for_hot_water", spath));
            None
        }
        (Some(WdStorage::Forfait { insulation }), _) => Some(insulation.efficiency()),
        (
            Some(WdStorage::Losses {
                storage_loss_kwh,
                pipe_loss_kwh,
                source_reference,
            }),
            _,
        ) => {
            reference(source_reference, format!("{spath}.sourceReference"), issues);
            let valid = |value: f64| value.is_finite() && value >= 0.0;
            if input > 0.0 && valid(*storage_loss_kwh) && valid(*pipe_loss_kwh) {
                Some(input / (input + storage_loss_kwh + pipe_loss_kwh))
            } else {
                issues.push(issue("value_invalid", spath));
                None
            }
        }
        (
            Some(WdStorage::Calculated {
                vessels,
                pipes,
                exchanger,
                circulation_temperature_c,
                correction_factor,
                source_reference,
            }),
            _,
        ) => {
            reference(source_reference, format!("{spath}.sourceReference"), issues);
            // P.35: the calculated route sums the losses of the components
            // present; without any it would give η = 1, better than the
            // forfait of P.6.6.4.3, so at least one is required.
            if vessels.is_empty() && pipes.is_empty() && exchanger.is_none() {
                issues.push(issue("storage_components_required", spath.clone()));
                return None;
            }
            let mut storage = Some(0.0);
            for (index, vessel) in vessels.iter().enumerate() {
                let loss = vessel_loss_kwh(vessel, &format!("{spath}.vessels[{index}]"), issues);
                storage = storage.zip(loss).map(|(sum, loss)| sum + loss);
            }
            let circulation = circulation_temperature_c.unwrap_or(CHARGE_CIRCULATION_C);
            let correction = correction_factor.unwrap_or(CHARGE_CORRECTION);
            if !circulation.is_finite() || !positive(correction) {
                issues.push(issue("value_invalid", spath.clone()));
                return None;
            }
            let pipes = charging_losses(
                pipes,
                exchanger.as_ref(),
                circulation,
                correction,
                &spath,
                issues,
            );
            let (storage, pipes) = storage.zip(pipes)?;
            let loss = storage + pipes;
            if input > 0.0 && loss >= 0.0 {
                Some(input / (input + loss))
            } else {
                issues.push(issue("value_invalid", spath));
                None
            }
        }
    }
}

/// The solar contribution `ΣQ_XD;sol;mi` and `W_XD;aux;sol;an`, kWh.
fn solar_contribution(
    contribution: &SolarContribution,
    function: SystemFunction,
    monthly_input: &Months,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<(f64, f64)> {
    match contribution {
        SolarContribution::Declared {
            monthly_kwh,
            annual_kwh,
            source_reference,
        } => {
            reference(source_reference, format!("{path}.sourceReference"), issues);
            let flow = PlotFlow {
                annual_kwh: *annual_kwh,
                monthly_kwh: monthly_kwh.clone(),
            };
            plot_flow(&flow, path.to_string(), issues).map(|(annual, _)| (annual, 0.0))
        }
        SolarContribution::Calculated(field) => {
            let collectors = &field.collectors;
            if !positive(collectors.module_area_m2) || collectors.module_count == 0 {
                issues.push(issue("solar_area_invalid", format!("{path}.collectors")));
                return None;
            }
            if !(0.0..=180.0).contains(&collectors.tilt_deg) {
                issues.push(issue(
                    "tilt_unsupported",
                    format!("{path}.collectors.tiltDeg"),
                ));
                return None;
            }
            if !positive(field.storage.total_volume_l) {
                issues.push(issue(
                    "value_invalid",
                    format!("{path}.storage.totalVolumeL"),
                ));
                return None;
            }
            if !field.storage_ambient_c.is_finite()
                || !field.network_return_c.is_finite()
                || !field.network_supply_c.is_finite()
                || field.network_supply_c <= field.network_return_c
            {
                issues.push(issue(
                    "network_temperature_invalid",
                    format!("{path}.networkSupplyC"),
                ));
                return None;
            }
            let ambient = [field.storage_ambient_c; 12];
            let months = calculated_service(
                field.solar_type,
                collectors,
                &field.storage,
                &ServiceSettings {
                    use_kwh: *monthly_input,
                    share: [1.0; 12],
                    reference_c: [COLLECTIVE_SOLAR_REFERENCE_C; 12],
                    low_c: ambient,
                    high_c: field.network_return_c,
                    backup_set_c: field.network_supply_c,
                    ambient_c: ambient,
                    pump_hours: if function == SystemFunction::HotWater {
                        PUMP_HOURS_WATER
                    } else {
                        PUMP_HOURS_COMBI
                    },
                    add_backup_loss_to_use: false,
                },
            );
            Some((
                months.iter().map(|month| month.renewable_kwh).sum(),
                months.iter().map(|month| month.auxiliary_kwh).sum(),
            ))
        }
    }
}

/// `P_XD;gen;gi` for β and P.23/P.42/P.51; cold: P.53/P.55.
fn generator_power(
    generator: &SystemGenerator,
    eta: Option<f64>,
    function: SystemFunction,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<f64> {
    if let Some(power) = generator.nominal_power_kw {
        if positive(power) {
            return Some(power);
        }
        issues.push(issue("value_invalid", format!("{path}.nominalPowerKw")));
        return None;
    }
    let cooling = generator.cooling_power?;
    if function != SystemFunction::Cooling {
        issues.push(issue(
            "cooling_power_requires_cooling",
            format!("{path}.coolingPower"),
        ));
        return None;
    }
    // P.53 multiplies the shaft power P_in by the cooling COP. For a
    // gas-engine chiller η_CD;gen = COP·η_ge relates to the fuel input, so
    // the shaft power takes the table P.9 COP itself (p. 1000).
    let shaft_cop = match &generator.kind {
        GeneratorKind::CompressionChiller {
            variant,
            drive,
            engine_efficiency,
        } => chiller_efficiency(*variant, drive, engine_efficiency.as_ref()).map(|(_, cop)| cop),
        _ => eta,
    };
    let power = match cooling {
        CoolingPower::CompressorShaft { shaft_power_kw } => shaft_cop
            .filter(|cop| positive(*cop))
            .map(|cop| cop * shaft_power_kw),
        CoolingPower::Aquifer {
            flow_m3_per_s,
            supply_c,
            return_c,
        } => Some(
            flow_m3_per_s
                * WATER_DENSITY
                * WATER_HEAT_CAPACITY
                * (supply_c - return_c).abs()
                * 1e-3,
        ),
    };
    match power {
        Some(power) if positive(power) => Some(power),
        _ => {
            issues.push(issue("value_invalid", format!("{path}.coolingPower")));
            None
        }
    }
}

struct FractionInput<'a> {
    function: SystemFunction,
    generators: &'a [SystemGenerator],
    efficiencies: &'a [Option<f64>],
    powers: &'a [Option<f64>],
    solar_kwh: &'a [Option<f64>],
    input_kwh: f64,
    storage_efficiency: f64,
    reference_power_kw: Option<f64>,
}

struct Fractions {
    values: Vec<f64>,
    preferred: Vec<bool>,
    beta: Option<f64>,
    reference_power_kw: Option<f64>,
    derived: bool,
}

/// Rules a)–e) of P.6.5.3.2 and a)–b) of P.6.7.3.2: 0 renewable or free
/// cooling, 1 heat pumps and CHP, 2 the highest efficiency, 3 an electrode
/// boiler in flex mode.
///
/// Rule a) lists renewable generators open-endedly ("zoals geothermie en
/// collectieve zonnecollectoren", p. 966); solid biomass is renewable heat
/// and takes class 0. An electrode boiler in flex mode runs only in the
/// flex hours (5.8) and is ranked last instead of winning rule e) on its
/// 0,99 efficiency; `priority` overrides both choices.
fn preference_class(kind: &GeneratorKind, function: SystemFunction) -> u8 {
    match (function, kind) {
        (SystemFunction::Cooling, GeneratorKind::FreeCooling { .. }) => 0,
        (SystemFunction::Cooling, _) => 2,
        (_, GeneratorKind::Geothermal { .. } | GeneratorKind::SolidBiomassBoiler { .. }) => 0,
        (
            _,
            GeneratorKind::ElectricFlex {
                generator: FlexGenerator::ElectrodeBoiler { .. },
                ..
            },
        ) => 3,
        (
            _,
            GeneratorKind::HeatPump { .. }
            | GeneratorKind::ChpWithoutLoss { .. }
            | GeneratorKind::ChpWithLoss { .. }
            | GeneratorKind::ElectricFlex {
                generator: FlexGenerator::HeatPump { .. },
                ..
            },
        ) => 1,
        _ => 2,
    }
}

fn chiller_carrier(kind: &GeneratorKind) -> Option<SystemCarrier> {
    match kind {
        GeneratorKind::CompressionChiller { drive, .. }
        | GeneratorKind::FreeCooling { drive, .. } => Some(*drive),
        _ => None,
    }
}

/// Groups of non-solar generators in order of preference: the given
/// priorities, or the preferred generators and the rest.
fn preference_groups(
    input: &FractionInput,
    others: &[usize],
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<Vec<Vec<usize>>> {
    let generators = input.generators;
    if others.iter().any(|&i| generators[i].priority.is_some()) {
        let mut priorities = Vec::new();
        for &i in others {
            match generators[i].priority {
                Some(priority) => priorities.push(priority),
                None => issues.push(issue(
                    "generator_priority_required",
                    format!("{path}.generators[{i}].priority"),
                )),
            }
        }
        if priorities.len() != others.len() {
            return None;
        }
        priorities.sort_unstable();
        priorities.dedup();
        return Some(
            priorities
                .iter()
                .map(|priority| {
                    others
                        .iter()
                        .copied()
                        .filter(|&i| generators[i].priority == Some(*priority))
                        .collect()
                })
                .collect(),
        );
    }
    if others.len() == 1 {
        return Some(vec![others.to_vec()]);
    }
    let class = others
        .iter()
        .map(|&i| preference_class(&generators[i].kind, input.function))
        .min()?;
    let preferred: Vec<usize> = if class != 2 {
        others
            .iter()
            .copied()
            .filter(|&i| preference_class(&generators[i].kind, input.function) == class)
            .collect()
    } else {
        // Rule e) among the class-2 generators only.
        let candidates: Vec<usize> = others
            .iter()
            .copied()
            .filter(|&i| preference_class(&generators[i].kind, input.function) == 2)
            .collect();
        if candidates.iter().any(|&i| input.efficiencies[i].is_none()) {
            issues.push(issue(
                "generator_priority_required",
                format!("{path}.generators"),
            ));
            return None;
        }
        let best = candidates
            .iter()
            .filter_map(|&i| input.efficiencies[i])
            .fold(f64::NEG_INFINITY, f64::max);
        let best_units: Vec<usize> = candidates
            .iter()
            .copied()
            .filter(|&i| input.efficiencies[i].is_some_and(|eta| eta >= best - 1e-9))
            .collect();
        // P.6.7.3: chillers form one preferred group only with the same
        // efficiency and the same carrier.
        match input.function {
            SystemFunction::Cooling => {
                let carrier = chiller_carrier(&generators[best_units[0]].kind);
                best_units
                    .into_iter()
                    .filter(|&i| chiller_carrier(&generators[i].kind) == carrier)
                    .collect()
            }
            _ => best_units,
        }
    };
    let is_last = |i: usize| preference_class(&generators[i].kind, input.function) == 3;
    let rest: Vec<usize> = others
        .iter()
        .copied()
        .filter(|&i| !preferred.contains(&i) && !is_last(i))
        .collect();
    let last: Vec<usize> = others
        .iter()
        .copied()
        .filter(|&i| !preferred.contains(&i) && is_last(i))
        .collect();
    let mut groups = vec![preferred];
    groups.extend([rest, last].into_iter().filter(|group| !group.is_empty()));
    Some(groups)
}

fn group_power(input: &FractionInput, members: &[usize]) -> Option<f64> {
    members
        .iter()
        .map(|&i| input.powers[i].filter(|power| positive(*power)))
        .sum()
}

fn require_powers(
    input: &FractionInput,
    members: &[usize],
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> bool {
    let mut ok = true;
    for &i in members {
        if !input.powers[i].is_some_and(positive) {
            issues.push(issue(
                "generator_power_required",
                format!("{path}.generators[{i}].nominalPowerKw"),
            ));
            ok = false;
        }
    }
    ok
}

/// Split `total` over `members` by nominal power (P.23/P.42/P.51).
fn split(
    input: &FractionInput,
    total: f64,
    members: &[usize],
    values: &mut [f64],
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> bool {
    if let [only] = members {
        values[*only] += total;
        return true;
    }
    if !require_powers(input, members, path, issues) {
        return false;
    }
    let sum: f64 = members.iter().filter_map(|&i| input.powers[i]).sum();
    for &i in members {
        values[i] += total * input.powers[i].unwrap_or(0.0) / sum;
    }
    true
}

/// `F_XD;gen;gi`: supplied for all generators, or derived (P.6.5.3,
/// P.6.6.3, P.6.7.3) with collective solar first (P.32).
fn energy_fractions(
    input: &FractionInput,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<Fractions> {
    let generators = input.generators;
    let count = generators.len();
    let solar: Vec<usize> = (0..count)
        .filter(|&i| matches!(generators[i].kind, GeneratorKind::CollectiveSolar { .. }))
        .collect();
    let others: Vec<usize> = (0..count).filter(|i| !solar.contains(i)).collect();
    if generators.iter().all(|g| g.energy_fraction.is_some()) {
        let values: Vec<f64> = generators
            .iter()
            .map(|g| g.energy_fraction.unwrap_or(0.0))
            .collect();
        let mut ok = true;
        for (index, value) in values.iter().enumerate() {
            if !(0.0..=1.0).contains(value) {
                issues.push(issue(
                    "energy_fraction_invalid",
                    format!("{path}.generators[{index}].energyFraction"),
                ));
                ok = false;
            }
        }
        if (values.iter().sum::<f64>() - 1.0).abs() > 1e-6 {
            issues.push(issue(
                "energy_fractions_must_sum_to_one",
                format!("{path}.generators"),
            ));
            ok = false;
        }
        // The preference only matters for P.6.6.5.2.
        let mut preferred = vec![true; count];
        if let Some(groups) = preference_groups(input, &others, path, &mut Vec::new()) {
            for &i in groups.iter().skip(1).flatten() {
                preferred[i] = false;
            }
        }
        return ok.then_some(Fractions {
            values,
            preferred,
            beta: None,
            reference_power_kw: None,
            derived: false,
        });
    }
    if generators.iter().any(|g| g.energy_fraction.is_some()) {
        issues.push(issue(
            "energy_fraction_partial",
            format!("{path}.generators"),
        ));
        return None;
    }
    let mut values = vec![0.0; count];
    let mut preferred = vec![false; count];
    for &i in &solar {
        values[i] = input.solar_kwh[i]? / input.input_kwh;
        preferred[i] = true;
    }
    let solar_sum: f64 = values.iter().sum();
    if solar_sum > 1.0 + 1e-9 {
        issues.push(issue(
            "solar_fraction_exceeds_one",
            format!("{path}.generators"),
        ));
        return None;
    }
    let rest = 1.0 - solar_sum;
    let mut fractions = Fractions {
        values: Vec::new(),
        preferred: Vec::new(),
        beta: None,
        reference_power_kw: None,
        derived: true,
    };
    if others.is_empty() {
        if rest > 1e-6 {
            issues.push(issue(
                "energy_fractions_must_sum_to_one",
                format!("{path}.generators"),
            ));
            return None;
        }
        fractions.values = values;
        fractions.preferred = preferred;
        return Some(fractions);
    }
    let groups = preference_groups(input, &others, path, issues)?;
    for &i in &groups[0] {
        preferred[i] = true;
    }
    let ok = if groups.len() == 1 {
        split(input, rest, &groups[0], &mut values, path, issues)
    } else {
        match input.function {
            SystemFunction::Heating => {
                // P.24/P.25 on the heat left after solar, table P.2, the
                // cascade of P.6.5.3.2 and P.23 for the remainder.
                // P.25 literally uses Q_HD;in;tot, the whole heat delivered
                // by all generators, collective solar included (p. 968).
                let reference_power = input
                    .reference_power_kw
                    .unwrap_or(input.input_kwh * 3.6 / REFERENCE_POWER_DIVISOR);
                fractions.reference_power_kw = Some(reference_power);
                let last = groups.len() - 1;
                let mut cumulative = 0.0;
                let mut cumulative_power = 0.0;
                let mut known = true;
                let mut ok = true;
                for (k, group) in groups.iter().enumerate() {
                    let power = group_power(input, group);
                    if k == last || (k > 0 && (power.is_none() || !known)) {
                        let members: Vec<usize> = groups[k..].concat();
                        ok &= split(
                            input,
                            rest * (1.0 - cumulative),
                            &members,
                            &mut values,
                            path,
                            issues,
                        );
                        break;
                    }
                    let beta = match power {
                        Some(power) => {
                            cumulative_power += power;
                            cumulative_power / reference_power
                        }
                        None => {
                            known = false;
                            UNKNOWN_BETA
                        }
                    };
                    if k == 0 {
                        fractions.beta = Some(beta);
                    }
                    let fraction = table_p2(beta).max(cumulative);
                    ok &= split(
                        input,
                        rest * (fraction - cumulative),
                        group,
                        &mut values,
                        path,
                        issues,
                    );
                    cumulative = fraction;
                }
                ok
            }
            SystemFunction::HotWater => {
                // P.40–P.42.
                let members: Vec<usize> = groups[1..].concat();
                match group_power(input, &groups[0]) {
                    Some(power) => {
                        let fraction =
                            (WD_PREFERENT_DUTY * power * YEAR_HOURS * input.storage_efficiency
                                / input.input_kwh)
                                .min(rest);
                        split(input, fraction, &groups[0], &mut values, path, issues)
                            & split(input, rest - fraction, &members, &mut values, path, issues)
                    }
                    None => require_powers(input, &groups[0], path, issues),
                }
            }
            SystemFunction::Cooling => {
                // P.52 with table P.8 and P.51.
                let members: Vec<usize> = groups[1..].concat();
                match (group_power(input, &groups[0]), group_power(input, &members)) {
                    (Some(preferred_power), Some(other_power)) => {
                        let beta = preferred_power / (preferred_power + other_power);
                        fractions.beta = Some(beta);
                        let fraction = table_p8(beta) * rest;
                        split(input, fraction, &groups[0], &mut values, path, issues)
                            & split(input, rest - fraction, &members, &mut values, path, issues)
                    }
                    _ => {
                        require_powers(input, &groups[0], path, issues)
                            & require_powers(input, &members, path, issues)
                    }
                }
            }
        }
    };
    if !ok {
        return None;
    }
    fractions.values = values;
    fractions.preferred = preferred;
    Some(fractions)
}

/// Forfait `[P_e (W), P_v;spec, P_hs;spec, P_sp;spec (W/kW)]` per kind
/// (P.6.8.4.3/P.6.9.4.3); cold: the standby of P.6.10.4.3.
fn default_auxiliary(kind: &GeneratorKind, function: SystemFunction) -> [f64; 4] {
    if function == SystemFunction::Cooling {
        let standby = if matches!(kind, GeneratorKind::Declared { .. }) {
            0.0
        } else {
            COLD_STANDBY_W
        };
        return [standby, 0.0, 0.0, 0.0];
    }
    let fuel = |carrier: &SystemCarrier| match carrier {
        // STEG and AVI: none (P.6.8.4.1).
        SystemCarrier::WasteIncineration => [0.0; 4],
        SystemCarrier::BiomassAbove500Kw => [STANDBY_W, 10.0, 0.0, 0.0],
        SystemCarrier::Electricity { .. } => [STANDBY_W, 0.0, 0.0, 0.0],
        _ => [STANDBY_W, 1.0, 0.0, 0.0],
    };
    // 0 W/kW with forfait efficiencies or when the declared efficiency
    // includes the source pump or fan, 10 W/kW otherwise.
    let source = |efficiency: &HeatPumpEfficiency| match efficiency {
        HeatPumpEfficiency::TableP5 { .. } => 0.0,
        HeatPumpEfficiency::Declared {
            source_pump_included: true,
            ..
        } => 0.0,
        HeatPumpEfficiency::Declared { .. } => 10.0,
    };
    match kind {
        GeneratorKind::Combustion { carrier, .. }
        | GeneratorKind::Boiler { carrier, .. }
        | GeneratorKind::ChpWithoutLoss { carrier, .. } => fuel(carrier),
        GeneratorKind::SolidBiomassBoiler { .. } => [STANDBY_W, 10.0, 0.0, 0.0],
        GeneratorKind::HeatPump { efficiency, .. }
        | GeneratorKind::ElectricFlex {
            generator: FlexGenerator::HeatPump { efficiency },
            ..
        } => [STANDBY_W, 0.0, source(efficiency), 0.0],
        GeneratorKind::ElectricFlex { .. } => [STANDBY_W, 0.0, 0.0, 0.0],
        _ => [0.0; 4],
    }
}

struct AuxiliaryContext<'a> {
    function: SystemFunction,
    generators: &'a [SystemGenerator],
    fractions: &'a [f64],
    factors: &'a [GenFactors],
    input_kwh: f64,
    monthly_input: Months,
    cold_months: [bool; 12],
    monthly_cold_known: bool,
    solar_kwh: f64,
}

/// P.56–P.70: the total and the term per generator, kWh.
fn auxiliary_energy(
    ctx: &AuxiliaryContext,
    input: &AuxiliaryInput,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<(AuxiliaryResult, Vec<f64>)> {
    let apath = format!("{path}.auxiliary.distribution");
    let cold = ctx.function == SystemFunction::Cooling;
    let pumps = |powers: &[f64], issues: &mut Vec<AnnexPIssue>| -> Option<f64> {
        if powers
            .iter()
            .all(|power| power.is_finite() && *power >= 0.0)
        {
            Some(powers.iter().sum())
        } else {
            issues.push(issue("value_invalid", format!("{apath}.pumpPowersW")));
            None
        }
    };
    let distribution = match &input.distribution {
        DistributionAuxiliary::Pumps {
            pump_powers_w,
            operating_hours,
            source_reference,
        } => {
            reference(source_reference, format!("{apath}.sourceReference"), issues);
            let power = pumps(pump_powers_w, issues);
            let hours = match operating_hours {
                Some(hours) if (0.0..=YEAR_HOURS).contains(hours) => Some(*hours),
                Some(_) => {
                    issues.push(issue("value_invalid", format!("{apath}.operatingHours")));
                    None
                }
                None if cold => {
                    issues.push(issue(
                        "operating_hours_required",
                        format!("{apath}.operatingHours"),
                    ));
                    None
                }
                None => Some(YEAR_HOURS),
            };
            // P.57/P.62/P.67.
            power
                .zip(hours)
                .map(|(power, hours)| hours * power / 1000.0)
        }
        DistributionAuxiliary::PumpsMonthly {
            pump_powers_w,
            source_reference,
        } => {
            reference(source_reference, format!("{apath}.sourceReference"), issues);
            if !cold {
                issues.push(issue("monthly_pumps_require_cooling", apath.clone()));
            } else if !ctx.monthly_cold_known {
                issues.push(issue("monthly_cold_delivery_required", apath.clone()));
            }
            // P.68 with f_on;mi of P.6.10.3.3.
            pumps(pump_powers_w, issues).map(|power| {
                (0..12)
                    .filter(|&month| ctx.cold_months[month])
                    .map(|month| power / 1000.0 * MONTH_HOURS[month])
                    .sum::<f64>()
            })
        }
        DistributionAuxiliary::Forfait {
            network,
            farthest_distance_km,
        } => {
            let specific = match ctx.function {
                SystemFunction::HotWater => Some(SECONDARY_AUX_SPECIFIC),
                SystemFunction::Cooling => Some(COLD_AUX_SPECIFIC),
                SystemFunction::Heating => match network {
                    None => {
                        issues.push(issue(
                            "auxiliary_network_required",
                            format!("{apath}.network"),
                        ));
                        None
                    }
                    Some(AuxiliaryNetwork::Secondary | AuxiliaryNetwork::SmallSystem) => {
                        Some(SECONDARY_AUX_SPECIFIC)
                    }
                    Some(network) => match farthest_distance_km {
                        Some(distance) if distance.is_finite() && *distance >= 0.0 => {
                            // Table P.11.
                            match (network, *distance <= 3.0) {
                                (AuxiliaryNetwork::PrimaryAndSecondary, true) => Some(0.0072),
                                (_, true) => Some(0.0054),
                                (AuxiliaryNetwork::Primary, false) => Some(0.0018 * distance),
                                _ => {
                                    issues.push(issue(
                                        "table_p11_no_value",
                                        format!("{apath}.network"),
                                    ));
                                    None
                                }
                            }
                        }
                        _ => {
                            issues.push(issue(
                                "farthest_distance_required",
                                format!("{apath}.farthestDistanceKm"),
                            ));
                            None
                        }
                    },
                },
            };
            specific.map(|specific| specific * ctx.input_kwh)
        }
    };
    let mut per_generator = Vec::new();
    let mut ok = true;
    for (index, generator) in ctx.generators.iter().enumerate() {
        let gpath = format!("{path}.generators[{index}]");
        let mut defaults = default_auxiliary(&generator.kind, ctx.function);
        let aux = generator.auxiliary.clone().unwrap_or_default();
        if ctx.function == SystemFunction::HotWater {
            // P.6.9.4.3: no standby when the heating function carries it,
            // none at all for a generator without auxiliary energy.
            if aux.also_serves_heating || aux.without_auxiliary_energy {
                defaults[0] = 0.0;
            }
            if aux.without_auxiliary_energy {
                defaults[1] = 0.0;
            }
        }
        let overrides = [
            aux.standby_w,
            aux.burner_w_per_kw,
            aux.source_w_per_kw,
            aux.solution_pump_w_per_kw,
            aux.heat_rejection_w_per_kw,
        ];
        if overrides.iter().any(Option::is_some) {
            optional_reference(
                aux.source_reference.as_ref(),
                format!("{gpath}.auxiliary.sourceReference"),
                issues,
            );
        }
        if overrides
            .iter()
            .flatten()
            .any(|value| !(value.is_finite() && *value >= 0.0))
        {
            issues.push(issue("value_invalid", format!("{gpath}.auxiliary")));
            ok = false;
            per_generator.push(0.0);
            continue;
        }
        let standby = aux.standby_w.unwrap_or(defaults[0]);
        let fraction = ctx.fractions[index];
        let energy = if cold {
            // P.69 standby term and P.70.
            let standby_kwh: f64 = (0..12)
                .filter(|&month| ctx.cold_months[month])
                .map(|month| standby * MONTH_HOURS[month] / 1000.0)
                .sum();
            let rejection = aux
                .heat_rejection_w_per_kw
                .or(aux.heat_rejection.map(HeatRejection::specific_w_per_kw));
            let air = match (rejection, ctx.factors[index].cop) {
                (None, _) => 0.0,
                (Some(specific), Some(cop)) if positive(cop) => {
                    specific * ctx.input_kwh * fraction * (1.0 + cop) / cop * RUN_TIME_MARGIN
                        / 1000.0
                }
                (Some(_), _) => {
                    issues.push(issue(
                        "cop_required",
                        format!("{gpath}.auxiliary.heatRejection"),
                    ));
                    ok = false;
                    0.0
                }
            };
            standby_kwh + air
        } else {
            if aux.heat_rejection.is_some() || aux.heat_rejection_w_per_kw.is_some() {
                issues.push(issue(
                    "heat_rejection_requires_cooling",
                    format!("{gpath}.auxiliary.heatRejection"),
                ));
                ok = false;
            }
            // P.59/P.60 and P.64/P.65. The norm cites P.32 under P.59 where
            // the monthly input of P.33 is meant (p. 1008); P.33 is used.
            let specific = aux.burner_w_per_kw.unwrap_or(defaults[1])
                + aux.source_w_per_kw.unwrap_or(defaults[2])
                + aux.solution_pump_w_per_kw.unwrap_or(defaults[3]);
            let running = if specific > 0.0 && fraction > 0.0 {
                match generator.nominal_power_kw {
                    Some(power) if positive(power) => (0..12)
                        .map(|month| {
                            let on = (ctx.monthly_input[month] * fraction * RUN_TIME_MARGIN
                                / power)
                                .clamp(0.0, MONTH_HOURS[month]);
                            specific * power * on / 1000.0
                        })
                        .sum::<f64>(),
                    _ => {
                        issues.push(issue(
                            "generator_power_required",
                            format!("{gpath}.nominalPowerKw"),
                        ));
                        ok = false;
                        0.0
                    }
                }
            } else {
                0.0
            };
            standby * YEAR_HOURS / 1000.0 + running
        };
        per_generator.push(energy);
    }
    let solar = if cold {
        if input.solar_kwh.is_some() {
            issues.push(issue("value_invalid", format!("{path}.auxiliary.solarKwh")));
            ok = false;
        }
        0.0
    } else {
        let solar = input.solar_kwh.unwrap_or(ctx.solar_kwh);
        if !(solar.is_finite() && solar >= 0.0) {
            issues.push(issue("value_invalid", format!("{path}.auxiliary.solarKwh")));
            ok = false;
        }
        solar
    };
    let distribution = distribution?;
    if !ok {
        return None;
    }
    let generators: f64 = per_generator.iter().sum();
    Some((
        AuxiliaryResult {
            distribution_kwh: distribution,
            solar_kwh: solar,
            generators_kwh: generators,
            total_kwh: distribution + solar + generators,
        },
        per_generator,
    ))
}

fn calculated(system: &CalculatedSystem, path: &str) -> Result<SystemResult, Vec<AnnexPIssue>> {
    let mut issues = Vec::new();
    let function = system.function;
    reference(
        &system.source_reference,
        format!("{path}.sourceReference"),
        &mut issues,
    );
    // P.72–P.83 or the supplied Q_XD;out;tot.
    let demand = delivered(system, path, &mut issues);
    match system.auxiliary_electricity_kwh {
        Some(value) if !(value.is_finite() && value >= 0.0) => issues.push(issue(
            "value_invalid",
            format!("{path}.auxiliaryElectricityKwh"),
        )),
        None if system.auxiliary.is_none() => issues.push(issue(
            "auxiliary_energy_required",
            format!("{path}.auxiliaryElectricityKwh"),
        )),
        _ => {}
    }
    if !(0.0..=1.0).contains(&system.auxiliary_renewable_share) {
        issues.push(issue(
            "value_invalid",
            format!("{path}.auxiliaryRenewableShare"),
        ));
    }
    if system
        .reference_power_kw
        .is_some_and(|value| !positive(value))
    {
        issues.push(issue("value_invalid", format!("{path}.referencePowerKw")));
    }
    let out = demand.map_or(1.0, |demand| demand.annual);
    // P.10–P.18.
    let distributed = distribution(system, out, path, &mut issues);
    let input = distributed.as_ref().map_or(out, |item| item.input);
    let monthly_loss = distributed
        .as_ref()
        .and_then(|item| item.monthly_loss)
        .unwrap_or_else(|| hours_profile(input - out));
    let demand_months = demand.and_then(|demand| demand.monthly);
    // P.33.
    let monthly_input: Months = match demand_months {
        Some(months) => std::array::from_fn(|index| months[index] + monthly_loss[index]),
        None => hours_profile(input),
    };
    let cold = function == SystemFunction::Cooling;
    // P.6.10.3.3: f_on;mi from the monthly cold delivery where known.
    let cold_months: [bool; 12] = match demand_months {
        Some(months) if cold => months.map(|value| value > 0.0),
        _ => [true; 12],
    };
    let mut warnings = Vec::new();
    // P.13–P.18 for cold: the pipes gain heat in every month with a water
    // temperature, also when no cold is delivered (the clamp at 0 only
    // removes negative gains). Months without cooling should then have no
    // water temperature (`null` in a monthly profile).
    let pipe_months = distributed.as_ref().and_then(|item| item.monthly_loss);
    if cold
        && demand_months.is_some()
        && pipe_months
            .is_some_and(|loss| (0..12).any(|month| !cold_months[month] && loss[month] > 0.0))
    {
        warnings.push(issue(
            "cold_network_gain_outside_cooling_months",
            format!("{path}.distribution"),
        ));
    }
    let storage_efficiency = storage_efficiency(system, input, path, &mut issues);
    let eta_sto = storage_efficiency.unwrap_or(1.0);
    let generators = &system.generators;
    if generators.is_empty() {
        issues.push(issue("generator_required", format!("{path}.generators")));
    }
    let efficiencies: Vec<Option<f64>> = generators
        .iter()
        .map(|generator| efficiency_of(&generator.kind, function))
        .collect();
    let mut powers = Vec::new();
    let mut solar_kwh = Vec::new();
    let mut solar_aux = 0.0;
    for (index, generator) in generators.iter().enumerate() {
        let gpath = format!("{path}.generators[{index}]");
        powers.push(generator_power(
            generator,
            efficiencies[index],
            function,
            &gpath,
            &mut issues,
        ));
        solar_kwh.push(match &generator.kind {
            GeneratorKind::CollectiveSolar { contribution } => solar_contribution(
                contribution,
                function,
                &monthly_input,
                &format!("{gpath}.kind.contribution"),
                &mut issues,
            )
            .map(|(heat, aux)| {
                solar_aux += aux;
                heat
            }),
            _ => None,
        });
    }
    let fractions = if generators.is_empty() {
        None
    } else {
        energy_fractions(
            &FractionInput {
                function,
                generators,
                efficiencies: &efficiencies,
                powers: &powers,
                solar_kwh: &solar_kwh,
                input_kwh: input,
                storage_efficiency: eta_sto,
                reference_power_kw: system.reference_power_kw,
            },
            path,
            &mut issues,
        )
    };
    // P.34: for WD the generators also cover the storage and pipe losses.
    let network_heat = input / eta_sto;
    let gen_factors: Vec<Option<GenFactors>> = generators
        .iter()
        .enumerate()
        .map(|(index, generator)| {
            let fraction = fractions.as_ref().map_or(0.0, |item| item.values[index]);
            factors(
                &generator.kind,
                function,
                FactorContext {
                    preferred: fractions
                        .as_ref()
                        .map_or(true, |item| item.preferred[index]),
                    heat_kwh: fraction * network_heat,
                    network_heat_kwh: network_heat,
                    nominal_power_kw: generator.nominal_power_kw,
                },
                &format!("{path}.generators[{index}]"),
                &mut issues,
            )
        })
        .collect();
    let calculated_aux = match (
        &system.auxiliary,
        &fractions,
        system.auxiliary_electricity_kwh,
    ) {
        (Some(aux), Some(fractions), None) if gen_factors.iter().all(Option::is_some) => {
            let factors: Vec<GenFactors> = gen_factors.iter().flatten().copied().collect();
            auxiliary_energy(
                &AuxiliaryContext {
                    function,
                    generators,
                    fractions: &fractions.values,
                    factors: &factors,
                    input_kwh: input,
                    monthly_input,
                    cold_months,
                    monthly_cold_known: cold && demand_months.is_some(),
                    solar_kwh: solar_aux,
                },
                aux,
                path,
                &mut issues,
            )
        }
        _ => None,
    };
    if !issues.is_empty() {
        return Err(issues);
    }
    let (Some(fractions), Some(storage_efficiency), Some(aux)) = (
        fractions,
        storage_efficiency,
        system
            .auxiliary_electricity_kwh
            .or(calculated_aux.as_ref().map(|(result, _)| result.total_kwh)),
    ) else {
        return Err(vec![issue("calculation_incomplete", path.to_string())]);
    };
    let efficiency = out / input;
    // P.19/P.21 (P.36/P.38, P.47/P.49).
    let mut f_gen = 0.0;
    let mut k_gen = 0.0;
    let mut renewable_energy = 0.0;
    let mut results = Vec::new();
    for (index, generator) in generators.iter().enumerate() {
        let item = gen_factors[index].unwrap_or_default();
        let fraction = fractions.values[index];
        f_gen += fraction * item.f;
        k_gen += fraction * item.k;
        let heat = fraction * network_heat;
        // P.27 has no MAX(0): a renewable fuel (biogas) in a CHP gives a
        // negative K_CO2;gen, which is kept as printed.
        if item.k < 0.0
            && matches!(
                generator.kind,
                GeneratorKind::ChpWithoutLoss { .. } | GeneratorKind::ChpWithLoss { .. }
            )
        {
            warnings.push(issue(
                "chp_co2_factor_negative",
                format!("{path}.generators[{index}]"),
            ));
        }
        // 5.42/5.49/5.50: Q_gen;gi·f_Pren;gi + W_gen;ren·f_Pren;elec.
        renewable_energy += heat * item.pren + heat * item.renewable_drive * F_PREN_ELEC;
        results.push(GeneratorResult {
            id: generator.id.clone(),
            primary_factor: item.f,
            co2_kg_per_kwh: item.k,
            renewable_factor: item.pren,
            heat_kwh: heat,
            energy_fraction: fraction,
            efficiency: item.eta,
            auxiliary_kwh: calculated_aux.as_ref().map(|(_, per)| per[index]),
        });
    }
    // P.34: f_WD;gen;tot = f_WD;gen;tot;ex / η_WD;gen;sto (1 for HD/CD).
    let f_gen = f_gen / storage_efficiency;
    let k_gen = k_gen / storage_efficiency;
    let aux_share = system.auxiliary_renewable_share;
    renewable_energy += aux * aux_share * F_PREN_ELEC;
    // P.7 and P.9 with the (weighted) electricity factors.
    let primary = round_up(f_gen / efficiency + aux / out * F_P_EL * (1.0 - aux_share));
    let co2 = k_gen / efficiency + aux / out * k_co2_el() * (1.0 - aux_share);
    let e_prim = out * primary;
    let renewable = if renewable_energy + e_prim > 0.0 {
        round_renewable(function, renewable_energy / (renewable_energy + e_prim))
    } else {
        0.0
    };
    Ok(SystemResult {
        factors: SupplyFactors {
            primary_factor: primary,
            renewable_factor: renewable,
            co2_kg_per_kwh: co2,
        },
        distribution_efficiency: Some(efficiency),
        generation_primary_factor: Some(f_gen),
        storage_efficiency: (function == SystemFunction::HotWater).then_some(storage_efficiency),
        calculation: Some(CalculationDetails {
            delivered_kwh: out,
            input_kwh: input,
            distribution_loss_kwh: input - out,
            monthly_input_kwh: monthly_input,
            auxiliary_electricity_kwh: aux,
            auxiliary: calculated_aux.map(|(result, _)| result),
            beta: fractions.beta,
            reference_power_kw: fractions.reference_power_kw,
            preferred: generators
                .iter()
                .zip(&fractions.preferred)
                .filter(|(_, preferred)| **preferred)
                .map(|(generator, _)| generator.id.clone())
                .collect(),
            fractions_derived: fractions.derived,
        }),
        generators: results,
        warnings,
    })
}

/// P.7: a producer of electricity in the area with a direct physical
/// connection to the users.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AreaElectricityGenerator {
    /// P.7.3.1 with chapter 16.
    Pv(Box<PvSystem>),
    /// Wind (DIN V 18599-9), hydro and other producers.
    Declared {
        id: String,
        annual_kwh: f64,
        source_reference: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaElectricityItem {
    pub id: String,
    /// `E_dei;pr;el;gi`, kWh.
    pub annual_kwh: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaElectricityResult {
    /// `E_dei;pr;el;tot` (P.71), kWh.
    pub total_kwh: f64,
    pub generators: Vec<AreaElectricityItem>,
}

/// P.71.
pub fn area_electricity(
    generators: &[AreaElectricityGenerator],
    path: &str,
) -> Result<AreaElectricityResult, Vec<AnnexPIssue>> {
    let mut issues = Vec::new();
    let mut items = Vec::new();
    for (index, generator) in generators.iter().enumerate() {
        let gpath = format!("{path}[{index}]");
        match generator {
            AreaElectricityGenerator::Pv(system) => {
                let found = validate_pv(system, &gpath);
                if system.collective.is_some() {
                    issues.push(issue(
                        "area_pv_not_collective",
                        format!("{gpath}.collective"),
                    ));
                } else if found.is_empty() {
                    items.push(AreaElectricityItem {
                        id: system.id.clone(),
                        annual_kwh: monthly_yield_kwh(system, 0.0).iter().sum(),
                    });
                }
                issues.extend(found.into_iter().map(|item| issue(item.code, item.path)));
            }
            AreaElectricityGenerator::Declared {
                id,
                annual_kwh,
                source_reference,
            } => {
                reference(
                    source_reference,
                    format!("{gpath}.sourceReference"),
                    &mut issues,
                );
                if !annual_kwh.is_finite() || *annual_kwh < 0.0 {
                    issues.push(issue("value_invalid", format!("{gpath}.annualKwh")));
                }
                items.push(AreaElectricityItem {
                    id: id.clone(),
                    annual_kwh: *annual_kwh,
                });
            }
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    Ok(AreaElectricityResult {
        total_kwh: items.iter().map(|item| item.annual_kwh).sum(),
        generators: items,
    })
}

fn measured(system: &MeasuredSystem, path: &str) -> Result<SystemResult, Vec<AnnexPIssue>> {
    let mut issues = Vec::new();
    reference(
        &system.source_reference,
        format!("{path}.sourceReference"),
        &mut issues,
    );
    let out = system.delivered_kwh;
    if !(out.is_finite() && out > 0.0) {
        issues.push(issue(
            "delivered_energy_invalid",
            format!("{path}.deliveredKwh"),
        ));
    }
    if system.inputs.is_empty() {
        issues.push(issue("measured_input_required", format!("{path}.inputs")));
    }
    for (index, item) in system.inputs.iter().enumerate() {
        if !item.kwh.is_finite() || item.kwh < 0.0 || !item.carrier.valid() {
            issues.push(issue("value_invalid", format!("{path}.inputs[{index}]")));
        }
        if item
            .chp_loss_electrical
            .is_some_and(|v| !v.is_finite() || v <= 0.0)
        {
            issues.push(issue(
                "chp_loss_ratio_invalid",
                format!("{path}.inputs[{index}].chpLossElectrical"),
            ));
        }
    }
    if !system.exported_electricity_kwh.is_finite() || system.exported_electricity_kwh < 0.0 {
        issues.push(issue(
            "value_invalid",
            format!("{path}.exportedElectricityKwh"),
        ));
    }
    if !(0.0..=1.0).contains(&system.renewable_factor) {
        issues.push(issue("value_invalid", format!("{path}.renewableFactor")));
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    // P.6 and its CO2 counterpart.
    let mut primary = -system.exported_electricity_kwh * F_P_EL;
    let mut co2 = -system.exported_electricity_kwh * k_co2_el();
    for item in &system.inputs {
        match item.chp_loss_electrical {
            None => {
                primary += item.kwh * item.carrier.primary_factor();
                co2 += item.kwh * item.carrier.co2();
            }
            Some(loss) => {
                primary += item.kwh * item.carrier.primary_factor() * loss * F_P_EL;
                co2 += item.kwh * item.carrier.co2() * loss * F_P_EL;
            }
        }
    }
    Ok(SystemResult {
        factors: SupplyFactors {
            primary_factor: round_up((primary / out).max(0.0)),
            renewable_factor: round_renewable(system.function, system.renewable_factor),
            co2_kg_per_kwh: co2 / out,
        },
        distribution_efficiency: None,
        generation_primary_factor: None,
        storage_efficiency: None,
        generators: Vec::new(),
        calculation: None,
        warnings: Vec::new(),
    })
}

/// The supply factors of one external supply route.
pub fn assess_route(
    route: &AnnexPRoute,
    function: SystemFunction,
    path: &str,
) -> Result<SystemResult, Vec<AnnexPIssue>> {
    match route {
        AnnexPRoute::Declared {
            primary_factor,
            renewable_factor,
            co2_kg_per_kwh,
            declaration_reference,
            ..
        } => {
            let mut issues = Vec::new();
            reference(
                declaration_reference,
                format!("{path}.declarationReference"),
                &mut issues,
            );
            if !primary_factor.is_finite() || *primary_factor < 0.0 {
                issues.push(issue("value_invalid", format!("{path}.primaryFactor")));
            }
            if !(0.0..=1.0).contains(renewable_factor) {
                issues.push(issue("value_invalid", format!("{path}.renewableFactor")));
            }
            if !co2_kg_per_kwh.is_finite() {
                issues.push(issue("value_invalid", format!("{path}.co2KgPerKwh")));
            }
            if !issues.is_empty() {
                return Err(issues);
            }
            Ok(SystemResult {
                factors: SupplyFactors {
                    primary_factor: round_up(*primary_factor),
                    renewable_factor: round_renewable(function, *renewable_factor),
                    co2_kg_per_kwh: *co2_kg_per_kwh,
                },
                distribution_efficiency: None,
                generation_primary_factor: None,
                storage_efficiency: None,
                generators: Vec::new(),
                calculation: None,
                warnings: Vec::new(),
            })
        }
        AnnexPRoute::Calculated(system) => {
            if system.function != function {
                return Err(vec![issue(
                    "system_function_mismatch",
                    format!("{path}.function"),
                )]);
            }
            calculated(system, path)
        }
        AnnexPRoute::Measured(system) => {
            if system.function != function {
                return Err(vec![issue(
                    "system_function_mismatch",
                    format!("{path}.function"),
                )]);
            }
            measured(system, path)
        }
    }
}

/// 9.6.8.1.1.2.3: temperature class of a collective heat-pump source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceTemperatureClass {
    /// Ground heat exchanger, groundwater, aquifer or WKO only.
    Below20C,
    /// At least 20 °C, surface water or unknown: tables 5.2–5.4.
    AtLeast20COrSurfaceWaterOrUnknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectiveHeatPumpSource {
    pub temperature_class: SourceTemperatureClass,
    /// Invoices or design data showing a collective source (note 1).
    pub supplier_reference: String,
    /// Annex P values for the source; absent means the forfait values.
    #[serde(default)]
    pub annex_p: Option<AnnexPRoute>,
    /// NTA 8800:2024 9.6.8.1.1.2.3 (p. 346): source (WKO) realised or
    /// permitted from 2013, EER_bron 23; otherwise or unknown 16. Only in
    /// the 2024 edition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realised_from_2013: Option<bool>,
}

/// Declared and forfait (EMGforf) factors of one supply.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioFactors {
    pub declared: SupplyFactors,
    pub forfait: SupplyFactors,
}

/// Collective heat-pump source factors for both scenarios: below 20 °C the
/// declared or forfait value applies in both (9.6.8.1.1.2.3).
pub fn source_factors(
    source: &CollectiveHeatPumpSource,
    path: &str,
) -> Result<(ScenarioFactors, Option<SystemResult>), Vec<AnnexPIssue>> {
    let mut issues = Vec::new();
    reference(
        &source.supplier_reference,
        format!("{path}.supplierReference"),
        &mut issues,
    );
    let declared = match &source.annex_p {
        Some(route) => {
            match assess_route(route, SystemFunction::Heating, &format!("{path}.annexP")) {
                Ok(result) => Some(result),
                Err(found) => {
                    issues.extend(found);
                    None
                }
            }
        }
        None => None,
    };
    if !issues.is_empty() {
        return Err(issues);
    }
    let route_2024 = crate::norm_versions::profile().heat_pump_source_route
        == crate::norm_versions::HeatPumpSourceRoute::AnySourceFrom15C2024;
    let forfait = match source.temperature_class {
        // NTA 8800:2024 9.6.3.1.3 (p. 323, INT-V1 p. 5): the source heat
        // takes f_P;del of table 5.2 (p. 93: 0,9) or annex P, whatever its
        // temperature; the electricity of the source system is booked
        // separately (9.6.8.1.1.2.3, p. 346).
        _ if route_2024 => heat_forfait(),
        SourceTemperatureClass::Below20C => source_below_20_forfait(),
        SourceTemperatureClass::AtLeast20COrSurfaceWaterOrUnknown => heat_forfait(),
    };
    let declared_factors = declared.as_ref().map_or(forfait, |item| item.factors);
    let forfait_scenario = match source.temperature_class {
        // 2025+C1 5.3.1.2 (p. 72): below 20 °C EMGforf uses the same value;
        // 2024 has no such rule (p. 73).
        SourceTemperatureClass::Below20C if !route_2024 => declared_factors,
        _ => heat_forfait(),
    };
    Ok((
        ScenarioFactors {
            declared: declared_factors,
            forfait: forfait_scenario,
        },
        declared,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} vs {b}");
    }

    fn system(generators: Vec<SystemGenerator>) -> CalculatedSystem {
        CalculatedSystem {
            function: SystemFunction::Heating,
            hot_water_storage: None,
            delivered_kwh: Some(1_000_000.0),
            area_demand: None,
            reference_power_kw: None,
            auxiliary: None,
            distribution: SystemDistribution::Flows {
                input_kwh: None,
                loss_kwh: Some(250_000.0),
                source_reference: "network model".into(),
            },
            generators,
            auxiliary_electricity_kwh: Some(20_000.0),
            auxiliary_renewable_share: 0.0,
            source_reference: "design".into(),
        }
    }

    #[test]
    fn calculated_route_follows_p7_p9_and_5_42() {
        let generators = vec![
            SystemGenerator {
                id: "hp".into(),
                energy_fraction: Some(0.7),
                nominal_power_kw: None,
                cooling_power: None,
                priority: None,
                auxiliary: None,
                kind: GeneratorKind::HeatPump {
                    efficiency: HeatPumpEfficiency::TableP5 {
                        source: TableP5Source::ElectricGroundwaterBelow15C,
                        supply_temperature_c: 70.0,
                    },
                    drive: SystemCarrier::Electricity {
                        direct_renewable_share: 0.0,
                    },
                },
            },
            SystemGenerator {
                id: "boiler".into(),
                energy_fraction: Some(0.3),
                nominal_power_kw: None,
                cooling_power: None,
                priority: None,
                auxiliary: None,
                kind: GeneratorKind::Combustion {
                    carrier: SystemCarrier::NaturalGas,
                    efficiency: 0.9,
                    efficiency_reference: "type plate".into(),
                },
            },
        ];
        let result = calculated(&system(generators), "s").unwrap();
        // Table P.5: 65–70 °C groundwater 2,6.
        let f_gen = 0.7 * 1.45 / 2.6 + 0.3 * 1.0 / 0.9;
        let eta = 1_000_000.0 / 1_250_000.0;
        let primary: f64 = f_gen / eta + 20_000.0 / 1_000_000.0 * 1.45;
        close(
            result.factors.primary_factor,
            (primary * 100.0).ceil() / 100.0,
        );
        let co2 = (0.7 * 0.268 / 2.6 + 0.3 * 0.218 / 0.9) / eta + 0.02 * 0.268;
        close(result.factors.co2_kg_per_kwh, co2);
        // 5.42: only the heat pump is renewable.
        let e_ren = 0.7 * 1_250_000.0 * (1.0 - 1.0 / 2.6);
        let e_prim = 1_000_000.0 * result.factors.primary_factor;
        close(
            result.factors.renewable_factor,
            (e_ren / (e_ren + e_prim) * 100.0).floor() / 100.0,
        );
    }

    #[test]
    fn generator_kinds_follow_annex_p() {
        let mut issues = Vec::new();
        let chp = generator_factors(
            &GeneratorKind::ChpWithoutLoss {
                carrier: SystemCarrier::NaturalGas,
                thermal_efficiency: Some(0.51),
                electrical_efficiency: Some(0.30),
                efficiency_reference: Some("table P.6".into()),
                table_p6: None,
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(chp.0, (1.0f64 - 0.30 * 1.45).max(0.0) / 0.51);
        // P.29 note: STEG 0,261.
        let steg = generator_factors(
            &GeneratorKind::ChpWithLoss {
                carrier: SystemCarrier::NaturalGas,
                loss_ratio: None,
                loss_ratio_reference: None,
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(steg.0, 0.261);
        let residual = generator_factors(
            &GeneratorKind::ResidualHeat {
                auxiliary_specific: None,
                auxiliary_reference: None,
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(residual.0, 0.07 * 1.45);
        close(residual.2, 0.93);
        // Geothermal 83/40 °C: Δθ = 40 K → η = 20, f_Pren 0,95 (5.48).
        let geo = generator_factors(
            &GeneratorKind::Geothermal {
                source_temperature_c: 83.0,
                return_temperature_c: 40.0,
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(geo.0, 1.45 / 20.0);
        close(geo.2, 0.95);
        let avi = generator_factors(
            &GeneratorKind::Combustion {
                carrier: SystemCarrier::WasteIncineration,
                efficiency: 1.0,
                efficiency_reference: "x".into(),
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(avi.0, 0.5);
        close(avi.1, 0.138);
        close(avi.2, 0.5);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(
            table_p5(TableP5Source::ElectricSourceAtLeast40C, 40.0),
            None
        );
        assert_eq!(
            table_p5(TableP5Source::ElectricSourceAtLeast40C, 42.0),
            Some(10.3)
        );
        assert_eq!(table_p5(TableP5Source::GasGroundwater, 60.0), None);
    }

    #[test]
    fn small_system_forfait_uses_table_p0() {
        let mut input = system(vec![SystemGenerator {
            id: "boiler".into(),
            energy_fraction: Some(1.0),
            nominal_power_kw: None,
            cooling_power: None,
            priority: None,
            auxiliary: None,
            kind: GeneratorKind::Combustion {
                carrier: SystemCarrier::NaturalGas,
                efficiency: 1.0,
                efficiency_reference: "x".into(),
            },
        }]);
        input.delivered_kwh = Some(100_000.0);
        input.auxiliary_electricity_kwh = Some(0.0);
        input.distribution = SystemDistribution::SmallSystemForfait {
            connections: 10,
            connection_type: ConnectionType::WithinBuilding,
            design_temperature: Some(table_p0_class(65.0, 40.0)),
            other_loss_kwh: 0.0,
        };
        let result = calculated(&input, "s").unwrap();
        let eta = 100_000.0 / (100_000.0 + 11_800.0);
        close(result.distribution_efficiency.unwrap(), eta);
        close(
            result.factors.primary_factor,
            ((1.0 / eta) * 100.0).ceil() / 100.0,
        );
        assert_eq!(table_p0_class(95.0, 70.0), NetworkTemperature::T90To60);
        assert_eq!(table_p0_class(40.0, 30.0), NetworkTemperature::T50To40);
    }

    #[test]
    fn measured_route_follows_p6() {
        let system = MeasuredSystem {
            function: SystemFunction::Heating,
            delivered_kwh: 800.0,
            inputs: vec![MeasuredInput {
                carrier: SystemCarrier::NaturalGas,
                kwh: 1000.0,
                chp_loss_electrical: None,
            }],
            exported_electricity_kwh: 200.0,
            renewable_factor: 0.123,
            source_reference: "three-year records".into(),
        };
        let result = measured(&system, "m").unwrap();
        close(
            result.factors.primary_factor,
            ((1000.0 - 290.0) / 800.0 * 100.0f64).ceil() / 100.0,
        );
        close(result.factors.renewable_factor, 0.12);
    }

    #[test]
    fn source_below_20_uses_one_value_in_both_scenarios() {
        let source = CollectiveHeatPumpSource {
            temperature_class: SourceTemperatureClass::Below20C,
            supplier_reference: "invoice".into(),
            realised_from_2013: None,
            annex_p: None,
        };
        let (factors, _) = source_factors(&source, "s").unwrap();
        close(factors.declared.primary_factor, 1.45 / 23.0);
        assert_eq!(factors.declared, factors.forfait);
        let warm = CollectiveHeatPumpSource {
            temperature_class: SourceTemperatureClass::AtLeast20COrSurfaceWaterOrUnknown,
            supplier_reference: "invoice".into(),
            realised_from_2013: None,
            annex_p: Some(AnnexPRoute::Declared {
                primary_factor: 0.3,
                renewable_factor: 0.5,
                co2_kg_per_kwh: 0.05,
                declaration_reference: "EMG".into(),
                measured_only: false,
            }),
        };
        let (factors, _) = source_factors(&warm, "s").unwrap();
        close(factors.declared.primary_factor, 0.3);
        assert_eq!(factors.forfait, HEAT_FORFAIT);
    }

    #[test]
    fn validation_reports_inconsistent_inputs() {
        let mut input = system(vec![SystemGenerator {
            id: "x".into(),
            energy_fraction: Some(0.5),
            nominal_power_kw: None,
            cooling_power: None,
            priority: None,
            auxiliary: None,
            kind: GeneratorKind::Combustion {
                carrier: SystemCarrier::BiofuelMix { biofuel_share: 1.5 },
                efficiency: 0.0,
                efficiency_reference: String::new(),
            },
        }]);
        input.distribution = SystemDistribution::Flows {
            input_kwh: Some(1.0),
            loss_kwh: Some(1.0),
            source_reference: "x".into(),
        };
        let codes: Vec<_> = calculated(&input, "s")
            .unwrap_err()
            .into_iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "distribution_two_of_three_flows_required",
            "energy_fractions_must_sum_to_one",
            "carrier_share_invalid",
            "generator_efficiency_invalid",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    fn boiler(fraction: f64, carrier: SystemCarrier) -> SystemGenerator {
        SystemGenerator {
            id: "boiler".into(),
            energy_fraction: Some(fraction),
            nominal_power_kw: None,
            cooling_power: None,
            priority: None,
            auxiliary: None,
            kind: GeneratorKind::Combustion {
                carrier,
                efficiency: 0.9,
                efficiency_reference: "test".into(),
            },
        }
    }

    #[test]
    fn hot_water_storage_efficiency_divides_the_generation_factor() {
        let mut wd = system(vec![boiler(1.0, SystemCarrier::NaturalGas)]);
        wd.function = SystemFunction::HotWater;
        wd.auxiliary_electricity_kwh = Some(0.0);
        // Required for WD.
        let codes: Vec<_> = calculated(&wd, "p")
            .unwrap_err()
            .iter()
            .map(|i| i.code)
            .collect();
        assert!(codes.contains(&"hot_water_storage_efficiency_required"));
        // P.6.6.4.3 forfait 0,90 (P.34).
        wd.hot_water_storage = Some(WdStorage::Forfait {
            insulation: WdStorageInsulation::AtLeast20Mm,
        });
        let result = calculated(&wd, "p").unwrap();
        close(result.storage_efficiency.unwrap(), 0.9);
        close(result.generation_primary_factor.unwrap(), 1.0 / 0.9 / 0.9);
        // P.10: η_dis = 1 000 000 / 1 250 000.
        close(
            result.factors.primary_factor,
            round_up(1.0 / 0.9 / 0.9 / 0.8),
        );
        // P.35 from the losses.
        wd.hot_water_storage = Some(WdStorage::Losses {
            storage_loss_kwh: 100_000.0,
            pipe_loss_kwh: 150_000.0,
            source_reference: "P.43/P.45".into(),
        });
        let result = calculated(&wd, "p").unwrap();
        close(
            result.storage_efficiency.unwrap(),
            1_250_000.0 / 1_500_000.0,
        );
        close(result.generators[0].heat_kwh, 1_500_000.0);
        // Not allowed for heat supply.
        let mut hd = system(vec![boiler(1.0, SystemCarrier::NaturalGas)]);
        hd.hot_water_storage = wd.hot_water_storage.clone();
        assert!(calculated(&hd, "p").is_err());
    }

    #[test]
    fn hot_water_small_system_forfait_requires_90_60() {
        let mut wd = system(vec![boiler(1.0, SystemCarrier::NaturalGas)]);
        wd.function = SystemFunction::HotWater;
        wd.hot_water_storage = Some(WdStorage::Forfait {
            insulation: WdStorageInsulation::AtLeast20Mm,
        });
        wd.distribution = SystemDistribution::SmallSystemForfait {
            connections: 10,
            connection_type: ConnectionType::WithinBuilding,
            design_temperature: Some(NetworkTemperature::T35To25),
            other_loss_kwh: 0.0,
        };
        let codes: Vec<_> = calculated(&wd, "p")
            .unwrap_err()
            .iter()
            .map(|i| i.code)
            .collect();
        assert_eq!(codes, vec!["hot_water_small_system_requires_90_60"]);
        if let SystemDistribution::SmallSystemForfait {
            design_temperature, ..
        } = &mut wd.distribution
        {
            *design_temperature = None;
        }
        let result = calculated(&wd, "p").unwrap();
        close(
            result.distribution_efficiency.unwrap(),
            1_000_000.0 / (1_000_000.0 + 17_500.0),
        );
    }

    #[test]
    fn table_p5_and_chp_renewable_rules() {
        // P.6.6.5.4: no table P.5 for hot water.
        let mut wd = system(vec![SystemGenerator {
            id: "hp".into(),
            energy_fraction: Some(1.0),
            nominal_power_kw: None,
            cooling_power: None,
            priority: None,
            auxiliary: None,
            kind: GeneratorKind::HeatPump {
                efficiency: HeatPumpEfficiency::TableP5 {
                    source: TableP5Source::ElectricGroundwaterBelow15C,
                    supply_temperature_c: 70.0,
                },
                drive: SystemCarrier::Electricity {
                    direct_renewable_share: 0.0,
                },
            },
        }]);
        wd.function = SystemFunction::HotWater;
        wd.hot_water_storage = Some(WdStorage::Forfait {
            insulation: WdStorageInsulation::AtLeast20Mm,
        });
        let codes: Vec<_> = calculated(&wd, "p")
            .unwrap_err()
            .iter()
            .map(|i| i.code)
            .collect();
        assert!(codes.contains(&"table_p5_heating_only"));
        // 5.45 note 6: a biogas CHP counts fully renewable.
        let mut issues = Vec::new();
        let (_, co2, pren) = generator_factors(
            &GeneratorKind::ChpWithoutLoss {
                carrier: SystemCarrier::Biogas,
                thermal_efficiency: Some(0.5),
                electrical_efficiency: Some(0.36),
                efficiency_reference: Some("test".into()),
                table_p6: None,
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(pren, 1.0);
        assert!(co2 < 0.0);
        let (_, _, pren) = generator_factors(
            &GeneratorKind::ChpWithLoss {
                carrier: SystemCarrier::BiofuelMix { biofuel_share: 0.4 },
                loss_ratio: None,
                loss_ratio_reference: None,
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(pren, 0.4);
        assert!(issues.is_empty());
    }

    #[test]
    fn plot_profiles_follow_p74_and_p78() {
        let mut issues = Vec::new();
        let heat_months: Vec<f64> = (1..=12).map(|m| 100.0 * m as f64).collect();
        // P.73/P.74: an annual-only sorption part takes P.74 itself; the
        // supplied heating months stay.
        let plot = AreaPlot {
            heating: Some(PlotFlow {
                annual_kwh: None,
                monthly_kwh: heat_months.clone(),
            }),
            heating_forfait: None,
            sorption_cooling: Some(PlotFlow {
                annual_kwh: Some(5000.0),
                monthly_kwh: Vec::new(),
            }),
            ..area_plot()
        };
        let area = AreaDemand { plots: vec![plot] };
        let heat = area_demand(&area, SystemFunction::Heating, "a", &mut issues).unwrap();
        near(heat.annual, 7800.0 + 5000.0, 1e-9);
        let weights: Vec<f64> = OUTDOOR_TEMPERATURE_C
            .iter()
            .map(|t| (18.0 - t).max(0.0))
            .collect();
        let sum: f64 = weights.iter().sum();
        let months = heat.monthly.unwrap();
        for index in 0..12 {
            near(
                months[index],
                heat_months[index] + 5000.0 * weights[index] / sum,
                1e-9,
            );
        }
        // July: no P.74 share, the supplied 700 kWh stays.
        near(months[6], 700.0, 1e-9);
        // P.77/P.78: annual-only dehumidification counts in the annual
        // total only; the monthly cold profile stays known.
        let cold_months = [
            0.0, 0.0, 0.0, 0.0, 50.0, 100.0, 200.0, 200.0, 50.0, 0.0, 0.0, 0.0,
        ];
        let plot = AreaPlot {
            heating_forfait: None,
            cooling: Some(PlotFlow {
                annual_kwh: None,
                monthly_kwh: cold_months.to_vec(),
            }),
            dehumidification: Some(PlotFlow {
                annual_kwh: Some(120.0),
                monthly_kwh: Vec::new(),
            }),
            ..area_plot()
        };
        let area = AreaDemand { plots: vec![plot] };
        let cold = area_demand(&area, SystemFunction::Cooling, "a", &mut issues).unwrap();
        near(cold.annual, 600.0 + 120.0, 1e-9);
        assert_eq!(cold.monthly.unwrap(), cold_months);
        assert!(issues.is_empty(), "{issues:?}");
        // A plot with only dehumidification: no zero profile marked as
        // known; the months are unknown and the annual profile applies.
        let plot = AreaPlot {
            heating_forfait: None,
            cooling: None,
            dehumidification: Some(PlotFlow {
                annual_kwh: Some(120.0),
                monthly_kwh: Vec::new(),
            }),
            ..area_plot()
        };
        let area = AreaDemand { plots: vec![plot] };
        let cold = area_demand(&area, SystemFunction::Cooling, "a", &mut issues).unwrap();
        near(cold.annual, 120.0, 1e-9);
        assert_eq!(cold.monthly, None);
    }

    #[test]
    fn cold_renewable_factor_is_not_rounded_and_zero_is_positive() {
        let route = |renewable_factor: f64| AnnexPRoute::Declared {
            primary_factor: 0.0,
            renewable_factor,
            co2_kg_per_kwh: 0.0,
            declaration_reference: "EMG".into(),
            measured_only: false,
        };
        let cold = assess_route(&route(0.782), SystemFunction::Cooling, "c").unwrap();
        close(cold.factors.renewable_factor, 0.782);
        let heat = assess_route(&route(0.782), SystemFunction::Heating, "h").unwrap();
        close(heat.factors.renewable_factor, 0.78);
        // ceil(-1e-9) is -0,0; the output must be +0,0.
        assert!(cold.factors.primary_factor.is_sign_positive());
        assert!(round_up(0.0).is_sign_positive());
        assert!(round_down(0.0).is_sign_positive());
    }

    #[test]
    fn biogas_chp_warns_on_negative_co2_and_source_pump_flag_applies() {
        let chp = SystemGenerator {
            id: "chp".into(),
            energy_fraction: Some(1.0),
            nominal_power_kw: None,
            cooling_power: None,
            priority: None,
            auxiliary: None,
            kind: GeneratorKind::ChpWithoutLoss {
                carrier: SystemCarrier::Biogas,
                thermal_efficiency: Some(0.5),
                electrical_efficiency: Some(0.36),
                efficiency_reference: Some("test".into()),
                table_p6: None,
            },
        };
        let result = calculated(&system(vec![chp]), "s").unwrap();
        assert!(result.factors.co2_kg_per_kwh < 0.0);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].code, "chp_co2_factor_negative");
        assert_eq!(result.warnings[0].path, "s.generators[0]");
        // P.6.8.4.3: 10 W/kW source pump unless included in the declaration.
        let declared = |included: bool| GeneratorKind::HeatPump {
            efficiency: HeatPumpEfficiency::Declared {
                value: 4.0,
                source_reference: "test".into(),
                source_pump_included: included,
            },
            drive: SystemCarrier::Electricity {
                direct_renewable_share: 0.0,
            },
        };
        assert_eq!(
            default_auxiliary(&declared(false), SystemFunction::Heating)[2],
            10.0
        );
        assert_eq!(
            default_auxiliary(&declared(true), SystemFunction::Heating)[2],
            0.0
        );
    }

    fn near(a: f64, b: f64, tolerance: f64) {
        assert!((a - b).abs() < tolerance, "{a} vs {b}");
    }

    fn generator(id: &str, power: Option<f64>, kind: GeneratorKind) -> SystemGenerator {
        SystemGenerator {
            id: id.into(),
            energy_fraction: None,
            nominal_power_kw: power,
            cooling_power: None,
            priority: None,
            auxiliary: None,
            kind,
        }
    }

    fn gas(efficiency: f64) -> GeneratorKind {
        GeneratorKind::Combustion {
            carrier: SystemCarrier::NaturalGas,
            efficiency,
            efficiency_reference: "test".into(),
        }
    }

    fn groundwater_heat_pump() -> GeneratorKind {
        GeneratorKind::HeatPump {
            efficiency: HeatPumpEfficiency::TableP5 {
                source: TableP5Source::ElectricGroundwaterBelow15C,
                supply_temperature_c: 70.0,
            },
            drive: SystemCarrier::Electricity {
                direct_renewable_share: 0.0,
            },
        }
    }

    fn codes(result: Result<SystemResult, Vec<AnnexPIssue>>) -> Vec<&'static str> {
        result.unwrap_err().iter().map(|item| item.code).collect()
    }

    #[test]
    fn forfait_tables_of_annex_p() {
        // Table P.2: 0,70 + 0,5·(0,84 − 0,70).
        close(table_p2(0.25), 0.77);
        close(table_p2(0.95), 1.0);
        // Table P.8: 0,5 + 0,5·(0,8 − 0,5).
        close(table_p8(0.4), 0.65);
        close(table_p3(BoilerClass::Hr107, TemperatureLevel::Low), 0.925);
        close(table_p3(BoilerClass::Hr104, TemperatureLevel::High), 0.875);
        let design = |average, system| EmissionDesign {
            average_design_temperature_c: average,
            system,
        };
        assert_eq!(
            table_p4(design(55.0, EmissionSystem::Direct)),
            TemperatureLevel::High
        );
        assert_eq!(
            table_p4(design(45.0, EmissionSystem::MixingWithoutReturnLimit)),
            TemperatureLevel::High
        );
        assert_eq!(
            table_p4(design(45.0, EmissionSystem::MixingWithReturnLimit)),
            TemperatureLevel::Low
        );
        let chp = |power, after, level| TableP6 {
            electrical_power_kw: power,
            installed_after_2006: after,
            temperature_level: level,
        };
        assert_eq!(table_p6(chp(150.0, true, None)), Some((0.49, 0.30)));
        assert_eq!(
            table_p6(chp(150.0, true, Some(TemperatureLevel::Low))),
            Some((0.51, 0.30))
        );
        assert_eq!(table_p6(chp(800.0, false, None)), Some((0.44, 0.35)));
        assert_eq!(table_p6(chp(2.0, false, None)), None);
        assert_eq!(table_p6(chp(30_000.0, false, None)), None);
        // Table P.7: 20 mm → 22 mm row, 12 mm → 10 mm column.
        assert_eq!(table_p7(20.0, 12.0), Some(0.271));
        assert_eq!(table_p7(80.0, 25.0), Some(0.399));
        assert_eq!(table_p7(90.0, 0.0), None);
        close(
            table_p9_cop(
                ChillerVariant::HighTemperatureEmissionAndLowTemperatureSource,
                true,
            ),
            7.0,
        );
        close(FreeCoolingSource::AquiferRecirculation.efficiency(), 14.0);
        close(
            TABLE_P16.iter().map(|(_, hours)| hours).sum::<f64>(),
            8760.0,
        );
        // Table P.14 detached: the listed months add up to 45,86 (total
        // printed as 45,83).
        near(
            table_p14(DwellingHeatType::Detached).iter().sum(),
            45.86,
            1e-9,
        );
        close(HotWaterUse::HealthcareClinical.specific_kwh_per_m2(), 15.28);
        close(HeatRejection::DryCooler.specific_w_per_kw(), 45.0);
    }

    #[test]
    fn heat_fractions_follow_beta_and_table_p2() {
        // Q_in = 1 250 000 kWh → P_ref = 1 250 000·3,6/5 400 = 833,33 kW.
        let mut input = system(vec![
            generator("boiler", Some(1000.0), gas(0.9)),
            generator("hp", Some(200.0), groundwater_heat_pump()),
        ]);
        let result = calculated(&input, "s").unwrap();
        let details = result.calculation.clone().unwrap();
        // β = 200/833,33 = 0,24 → F = 0,70 + 0,4·0,14 = 0,756 (rule b).
        close(details.reference_power_kw.unwrap(), 2500.0 / 3.0);
        close(details.beta.unwrap(), 0.24);
        assert_eq!(details.preferred, vec!["hp".to_string()]);
        assert!(details.fractions_derived);
        close(result.generators[1].energy_fraction, 0.756);
        close(result.generators[0].energy_fraction, 0.244);
        close(result.generators[1].heat_kwh, 0.756 * 1_250_000.0);
        let f_gen = 0.756 * 1.45 / 2.6 + 0.244 / 0.9;
        close(result.generation_primary_factor.unwrap(), f_gen);
        close(
            result.factors.primary_factor,
            round_up(f_gen / 0.8 + 0.02 * 1.45),
        );
        // A supplied reference power (historical peaks) overrides P.25.
        input.reference_power_kw = Some(400.0);
        let result = calculated(&input, "s").unwrap();
        // β = 0,5 → 0,96.
        close(result.generators[1].energy_fraction, 0.96);
        // Supplied fractions override the derivation.
        input.generators[0].energy_fraction = Some(0.3);
        input.generators[1].energy_fraction = Some(0.7);
        let result = calculated(&input, "s").unwrap();
        close(result.generators[1].energy_fraction, 0.7);
        assert!(!result.calculation.unwrap().fractions_derived);
        // Only one of them supplied is rejected.
        input.generators[0].energy_fraction = None;
        assert_eq!(
            codes(calculated(&input, "s")),
            vec!["energy_fraction_partial"]
        );
    }

    #[test]
    fn heat_fractions_cascade_and_unknown_beta() {
        let mut chp = generator(
            "chp",
            Some(200.0),
            GeneratorKind::ChpWithoutLoss {
                carrier: SystemCarrier::NaturalGas,
                thermal_efficiency: None,
                electrical_efficiency: None,
                efficiency_reference: None,
                table_p6: Some(TableP6 {
                    electrical_power_kw: 150.0,
                    installed_after_2006: true,
                    temperature_level: None,
                }),
            },
        );
        chp.priority = Some(2);
        let mut hp = generator("hp", Some(100.0), groundwater_heat_pump());
        hp.priority = Some(1);
        let mut boiler = generator("boiler", Some(600.0), gas(0.9));
        boiler.priority = Some(3);
        let mut input = system(vec![boiler, chp, hp]);
        let result = calculated(&input, "s").unwrap();
        // β1 = 100/833,33 = 0,12 → 0,45 + 0,2·0,25 = 0,50;
        // β1+2 = 300/833,33 = 0,36 → 0,84 + 0,6·0,08 = 0,888.
        close(result.generators[2].energy_fraction, 0.5);
        close(result.generators[1].energy_fraction, 0.388);
        close(result.generators[0].energy_fraction, 0.112);
        // CHP with table P.6 (0,49/0,30): (1 − 0,30·1,45)/0,49.
        close(
            result.generators[1].primary_factor,
            (1.0 - 0.3 * 1.45) / 0.49,
        );
        // A missing priority is reported.
        input.generators[0].priority = None;
        assert_eq!(
            codes(calculated(&input, "s")),
            vec!["generator_priority_required"]
        );
        // Unknown power of the preferred heat pump: β = 0,5 → 0,96.
        let input = system(vec![
            generator("hp", None, groundwater_heat_pump()),
            generator("boiler", None, gas(0.9)),
        ]);
        let result = calculated(&input, "s").unwrap();
        close(result.generators[0].energy_fraction, 0.96);
        close(result.generators[1].energy_fraction, 0.04);
        // Two non-preferred generators need their power (P.23).
        let input = system(vec![
            generator("hp", None, groundwater_heat_pump()),
            generator("boiler", Some(500.0), gas(0.9)),
            generator("oil", None, gas(0.85)),
        ]);
        assert_eq!(
            codes(calculated(&input, "s")),
            vec!["generator_power_required"]
        );
        // Rule e: the highest efficiency; without one a priority is needed.
        let input = system(vec![
            generator("a", Some(100.0), gas(0.9)),
            generator("b", Some(900.0), gas(0.95)),
        ]);
        let result = calculated(&input, "s").unwrap();
        assert_eq!(result.calculation.unwrap().preferred, vec!["b".to_string()]);
        let input = system(vec![
            generator("a", Some(100.0), gas(0.9)),
            generator(
                "rest",
                Some(900.0),
                GeneratorKind::ResidualHeat {
                    auxiliary_specific: None,
                    auxiliary_reference: None,
                },
            ),
        ]);
        assert_eq!(
            codes(calculated(&input, "s")),
            vec!["generator_priority_required"]
        );
    }

    #[test]
    fn collective_solar_comes_first() {
        let solar = generator(
            "solar",
            None,
            GeneratorKind::CollectiveSolar {
                contribution: SolarContribution::Declared {
                    monthly_kwh: Vec::new(),
                    annual_kwh: Some(125_000.0),
                    source_reference: "13.7.2.2".into(),
                },
            },
        );
        let input = system(vec![
            generator("boiler", Some(1000.0), gas(0.9)),
            generator("hp", Some(200.0), groundwater_heat_pump()),
            solar,
        ]);
        let result = calculated(&input, "s").unwrap();
        // P.32: 125 000/1 250 000 = 0,1; P.25 on the whole Q_HD;in;tot:
        // P_ref = 1 250 000·3,6/5 400 = 833,3 kW, β = 0,24 → 0,70 +
        // (0,04/0,1)·0,14.
        close(result.generators[2].energy_fraction, 0.1);
        close(result.generators[2].primary_factor, 0.0);
        close(result.generators[2].renewable_factor, 1.0);
        let preferred = 0.70 + (0.24 - 0.2) / 0.1 * 0.14;
        close(result.generators[1].energy_fraction, 0.9 * preferred);
        close(
            result.generators[0].energy_fraction,
            0.9 * (1.0 - preferred),
        );
        close(
            result.calculation.unwrap().reference_power_kw.unwrap(),
            1_250_000.0 * 3.6 / 5400.0,
        );
    }

    fn collectors() -> CollectorField {
        CollectorField {
            module_area_m2: 2.5,
            module_count: 200,
            orientation: crate::climate::Orientation::South,
            tilt_deg: 45.0,
            obstruction: crate::solar_shading::CollectorObstruction::Minimal,
            efficiency: crate::solar_thermal::CollectorEfficiency::Forfait {
                collector: crate::solar_thermal::CollectorType::Glazed,
            },
            heat_exchanger_w_per_k: None,
            loop_pipes: crate::solar_thermal::LoopPipes::Forfait,
            pump_power_w: None,
        }
    }

    #[test]
    fn calculated_collective_solar_uses_13_7_2_2() {
        let field = CollectiveSolarField {
            solar_type: SolarType::Preheater,
            collectors: collectors(),
            storage: SolarStorage {
                total_volume_l: 25_000.0,
                backup_volume_l: None,
                loss: crate::domestic_hot_water::StorageLoss::Measured {
                    transmission_w_per_k: 20.0,
                },
                backup_loss_in_generator_efficiency: false,
            },
            network_supply_c: 70.0,
            network_return_c: 40.0,
            storage_ambient_c: 15.0,
        };
        let solar = generator(
            "solar",
            None,
            GeneratorKind::CollectiveSolar {
                contribution: SolarContribution::Calculated(Box::new(field.clone())),
            },
        );
        let mut input = system(vec![generator("boiler", Some(500.0), gas(0.9)), solar]);
        input.auxiliary_electricity_kwh = None;
        input.auxiliary = Some(AuxiliaryInput {
            distribution: DistributionAuxiliary::Forfait {
                network: Some(AuxiliaryNetwork::Secondary),
                farthest_distance_km: None,
            },
            solar_kwh: None,
        });
        let result = calculated(&input, "s").unwrap();
        let details = result.calculation.unwrap();
        // P.33 monthly input sums to Q_in.
        close(details.monthly_input_kwh.iter().sum(), 1_250_000.0);
        // The same 13.7.2.2 call on Q_HD;in;mi with θ_0 = 70 °C.
        let months = calculated_service(
            SolarType::Preheater,
            &field.collectors,
            &field.storage,
            &ServiceSettings {
                use_kwh: details.monthly_input_kwh,
                share: [1.0; 12],
                reference_c: [70.0; 12],
                low_c: [15.0; 12],
                high_c: 40.0,
                backup_set_c: 70.0,
                ambient_c: [15.0; 12],
                pump_hours: PUMP_HOURS_COMBI,
                add_backup_loss_to_use: false,
            },
        );
        let heat: f64 = months.iter().map(|m| m.renewable_kwh).sum();
        let pump: f64 = months.iter().map(|m| m.auxiliary_kwh).sum();
        assert!(heat > 0.0 && pump > 0.0);
        close(result.generators[1].energy_fraction, heat / 1_250_000.0);
        close(details.auxiliary.unwrap().solar_kwh, pump);
    }

    #[test]
    fn hot_water_fractions_follow_p40_to_p42() {
        let boiler = |id: &str, power, class| {
            generator(
                id,
                Some(power),
                GeneratorKind::Boiler {
                    carrier: SystemCarrier::NaturalGas,
                    efficiency: BoilerEfficiency::TableP3 {
                        boiler: class,
                        temperature_level: None,
                        emission: None,
                    },
                },
            )
        };
        let mut wd = system(vec![
            boiler("hr", 20.0, BoilerClass::Hr107),
            boiler("old", 100.0, BoilerClass::Conventional),
        ]);
        wd.function = SystemFunction::HotWater;
        wd.delivered_kwh = Some(500_000.0);
        wd.distribution = SystemDistribution::Flows {
            input_kwh: None,
            loss_kwh: Some(100_000.0),
            source_reference: "x".into(),
        };
        wd.hot_water_storage = Some(WdStorage::Forfait {
            insulation: WdStorageInsulation::AtLeast20Mm,
        });
        let result = calculated(&wd, "w").unwrap();
        // P.40: 0,60·20·8 760·0,9/600 000 = 0,15768; HT per P.6.6.5.2.
        let preferred = 0.6 * 20.0 * 8760.0 * 0.9 / 600_000.0;
        close(result.generators[0].energy_fraction, preferred);
        close(result.generators[1].energy_fraction, 1.0 - preferred);
        close(result.generators[0].efficiency.unwrap(), 0.90);
        close(
            result.generation_primary_factor.unwrap(),
            (preferred / 0.9 + (1.0 - preferred) / 0.7) / 0.9,
        );
        // P.41: capped at 1.
        wd.generators[0].nominal_power_kw = Some(500.0);
        let result = calculated(&wd, "w").unwrap();
        close(result.generators[0].energy_fraction, 1.0);
        close(result.generators[1].energy_fraction, 0.0);
        // P.6.6.5.2: a non-preferred full-load boiler loses 5 points.
        wd.generators[0].nominal_power_kw = Some(20.0);
        wd.generators[1].kind = GeneratorKind::Boiler {
            carrier: SystemCarrier::NaturalGas,
            efficiency: BoilerEfficiency::FullLoad {
                value: 0.88,
                outdoor_installation: false,
                source_reference: "test 80/60".into(),
            },
        };
        let result = calculated(&wd, "w").unwrap();
        close(result.generators[1].efficiency.unwrap(), 0.83);
    }

    #[test]
    fn hot_water_full_load_and_auxiliary_flags() {
        let full_load = GeneratorKind::Boiler {
            carrier: SystemCarrier::NaturalGas,
            efficiency: BoilerEfficiency::FullLoad {
                value: 0.88,
                outdoor_installation: false,
                source_reference: "test 80/60".into(),
            },
        };
        let mut wd = system(vec![
            generator("hr", Some(20.0), full_load.clone()),
            generator("old", Some(100.0), full_load),
        ]);
        wd.function = SystemFunction::HotWater;
        wd.delivered_kwh = Some(500_000.0);
        wd.distribution = SystemDistribution::Flows {
            input_kwh: None,
            loss_kwh: Some(100_000.0),
            source_reference: "x".into(),
        };
        wd.hot_water_storage = Some(WdStorage::Forfait {
            insulation: WdStorageInsulation::AtLeast20Mm,
        });
        // P.6.6.5.2: a preferred hot-water boiler takes table P.3 HT.
        assert!(codes(calculated(&wd, "w")).contains(&"hot_water_full_load_requires_non_preferred"));
        wd.generators[0].kind = GeneratorKind::Boiler {
            carrier: SystemCarrier::NaturalGas,
            efficiency: BoilerEfficiency::TableP3 {
                boiler: BoilerClass::Hr107,
                temperature_level: None,
                emission: None,
            },
        };
        wd.auxiliary_electricity_kwh = None;
        wd.auxiliary = Some(AuxiliaryInput {
            distribution: DistributionAuxiliary::Pumps {
                pump_powers_w: vec![0.0],
                operating_hours: None,
                source_reference: "pump schedule".into(),
            },
            solar_kwh: None,
        });
        let base = calculated(&wd, "w").unwrap();
        // P.6.9.4.3: no standby when the boiler also serves heating, nothing
        // at all without auxiliary energy.
        wd.generators[0].auxiliary = Some(GeneratorAuxiliary {
            also_serves_heating: true,
            ..GeneratorAuxiliary::default()
        });
        wd.generators[1].auxiliary = Some(GeneratorAuxiliary {
            without_auxiliary_energy: true,
            ..GeneratorAuxiliary::default()
        });
        let flagged = calculated(&wd, "w").unwrap();
        close(
            flagged.generators[0].auxiliary_kwh.unwrap(),
            base.generators[0].auxiliary_kwh.unwrap() - 876.0,
        );
        assert!(base.generators[1].auxiliary_kwh.unwrap() > 876.0);
        close(flagged.generators[1].auxiliary_kwh.unwrap(), 0.0);
    }

    #[test]
    fn biomass_first_and_electrode_boiler_last() {
        // Rule a): solid biomass is renewable and preferred over gas.
        let biomass = generator(
            "bio",
            Some(200.0),
            GeneratorKind::SolidBiomassBoiler {
                carrier: SystemCarrier::BiomassAbove500Kw,
                net_efficiency: 0.95,
                source_reference: "EN 303-5".into(),
            },
        );
        let input = system(vec![generator("boiler", Some(1000.0), gas(0.95)), biomass]);
        let result = calculated(&input, "s").unwrap();
        // P.25: 1 250 000·3,6/5 400 = 833,3 kW, β = 0,24.
        close(result.generators[1].energy_fraction, table_p2(0.24));
        // An electrode boiler in flex mode comes after the gas boiler.
        let flex = GeneratorKind::ElectricFlex {
            generator: FlexGenerator::ElectrodeBoiler {
                efficiency: None,
                efficiency_reference: None,
            },
            flex_heat_kwh: None,
            network_production_kwh: None,
            flex_reference: None,
            connections: 600,
            heat_buffer: true,
            registration_reference: "hourly register".into(),
        };
        let input = system(vec![
            generator("e-boiler", Some(400.0), flex),
            generator("boiler", Some(500.0), gas(0.9)),
        ]);
        let result = calculated(&input, "s").unwrap();
        // β = 500/833,3 = 0,6 for the gas boiler.
        close(result.generators[1].energy_fraction, table_p2(0.6));
        close(result.generators[0].energy_fraction, 1.0 - table_p2(0.6));
    }

    #[test]
    fn cold_fractions_follow_p52_to_p55() {
        let mut aquifer = generator(
            "wko",
            None,
            GeneratorKind::FreeCooling {
                source: FreeCoolingSource::AquiferStorageFrom2013,
                drive: SystemCarrier::Electricity {
                    direct_renewable_share: 0.0,
                },
            },
        );
        aquifer.cooling_power = Some(CoolingPower::Aquifer {
            flow_m3_per_s: 0.01,
            supply_c: 6.0,
            return_c: 16.0,
        });
        let mut chiller = generator(
            "ckm",
            None,
            GeneratorKind::CompressionChiller {
                variant: ChillerVariant::Unspecified,
                drive: SystemCarrier::Electricity {
                    direct_renewable_share: 0.0,
                },
                engine_efficiency: None,
            },
        );
        chiller.cooling_power = Some(CoolingPower::CompressorShaft {
            shaft_power_kw: 200.0,
        });
        let mut cd = system(vec![chiller, aquifer]);
        cd.function = SystemFunction::Cooling;
        cd.delivered_kwh = Some(400_000.0);
        cd.distribution = SystemDistribution::SmallColdForfait {
            supply_below_10_c: true,
        };
        cd.auxiliary_electricity_kwh = Some(0.0);
        let result = calculated(&cd, "c").unwrap();
        // P.55: 0,01·1 000·4 190·10·10⁻³ = 419 kW; P.53: 3·200 = 600 kW;
        // β = 419/1 019 → table P.8 between 0,3 (0,5) and 0,5 (0,8).
        let beta = 419.0 / 1019.0;
        close(result.calculation.clone().unwrap().beta.unwrap(), beta);
        let preferred = 0.5 + (beta - 0.3) / 0.2 * 0.3;
        close(result.generators[1].energy_fraction, preferred);
        close(result.generators[0].energy_fraction, 1.0 - preferred);
        // Table P.9: η 23 (fully renewable, 5.49) and 3.
        close(result.generators[1].primary_factor, 1.45 / 23.0);
        close(result.generators[1].renewable_factor, 1.0);
        close(result.generators[0].renewable_factor, 0.0);
        close(
            result.generation_primary_factor.unwrap(),
            preferred * 1.45 / 23.0 + (1.0 - preferred) * 1.45 / 3.0,
        );
    }

    #[test]
    fn chillers_group_only_with_the_same_carrier() {
        // P.6.7.3: equal η but another carrier is no common group.
        let chiller = |id: &str, share, shaft| SystemGenerator {
            cooling_power: Some(CoolingPower::CompressorShaft {
                shaft_power_kw: shaft,
            }),
            ..generator(
                id,
                None,
                GeneratorKind::CompressionChiller {
                    variant: ChillerVariant::Unspecified,
                    drive: SystemCarrier::Electricity {
                        direct_renewable_share: share,
                    },
                    engine_efficiency: None,
                },
            )
        };
        let mut cd = system(vec![chiller("a", 0.0, 200.0), chiller("b", 0.5, 100.0)]);
        cd.function = SystemFunction::Cooling;
        cd.delivered_kwh = Some(400_000.0);
        cd.distribution = SystemDistribution::SmallColdForfait {
            supply_below_10_c: true,
        };
        cd.auxiliary_electricity_kwh = Some(0.0);
        let result = calculated(&cd, "c").unwrap();
        let beta = 600.0 / 900.0;
        close(result.calculation.clone().unwrap().beta.unwrap(), beta);
        close(result.generators[0].energy_fraction, table_p8(beta));
        close(result.generators[1].energy_fraction, 1.0 - table_p8(beta));
    }

    #[test]
    fn gas_engine_chiller_power_uses_the_cop_on_the_shaft() {
        // P.53: P_CD = COP × P_in; η_CD;gen = COP·η_ge relates to fuel.
        let gas = SystemGenerator {
            cooling_power: Some(CoolingPower::CompressorShaft {
                shaft_power_kw: 100.0,
            }),
            ..generator(
                "gas",
                None,
                GeneratorKind::CompressionChiller {
                    variant: ChillerVariant::Unspecified,
                    drive: SystemCarrier::NaturalGas,
                    engine_efficiency: Some(EngineEfficiency::Declared {
                        value: 0.30,
                        source_reference: "datasheet".into(),
                    }),
                },
            )
        };
        let mut issues = Vec::new();
        let power = generator_power(
            &gas,
            efficiency_of(&gas.kind, SystemFunction::Cooling),
            SystemFunction::Cooling,
            "g",
            &mut issues,
        )
        .unwrap();
        close(
            power,
            table_p9_cop(ChillerVariant::Unspecified, true) * 100.0,
        );
        assert!(issues.is_empty());
    }

    #[test]
    fn chiller_and_boiler_kinds() {
        let mut issues = Vec::new();
        // Gas-engine chiller: COP 7 × ε_chp;el 0,30 (table P.6) = 2,1.
        let (f, _, pren) = generator_factors(
            &GeneratorKind::CompressionChiller {
                variant: ChillerVariant::HighTemperatureEmissionAndLowTemperatureSource,
                drive: SystemCarrier::NaturalGas,
                engine_efficiency: Some(EngineEfficiency::TableP6(TableP6 {
                    electrical_power_kw: 150.0,
                    installed_after_2006: true,
                    temperature_level: None,
                })),
            },
            SystemFunction::Cooling,
            "g",
            &mut issues,
        )
        .unwrap();
        close(f, 1.0 / 2.1);
        close(pren, 0.0);
        // Table P.10: f_P;HD;tot / 0,7.
        let (f, _, _) = generator_factors(
            &GeneratorKind::SorptionChiller {
                heat: SorptionHeat::CollectiveHeat {
                    primary_factor: 0.5,
                    co2_kg_per_kwh: 0.05,
                    source_reference: "EMG".into(),
                },
            },
            SystemFunction::Cooling,
            "g",
            &mut issues,
        )
        .unwrap();
        close(f, 0.5 / 0.7);
        // P.6.5.4.3: 0,90/1,08 − 0,05; biomass above 500 kW (table 5.5).
        let (f, _, pren) = generator_factors(
            &GeneratorKind::SolidBiomassBoiler {
                carrier: SystemCarrier::BiomassAbove500Kw,
                net_efficiency: 0.9,
                source_reference: "NEN-EN 303-5".into(),
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(f, 0.0);
        close(pren, 1.0);
        close(biomass_efficiency(0.9), 0.9 / 1.08 - 0.05);
        // Table P.3 with table P.4 (direct, 40 °C → LT).
        let (f, _, _) = generator_factors(
            &GeneratorKind::Boiler {
                carrier: SystemCarrier::NaturalGas,
                efficiency: BoilerEfficiency::TableP3 {
                    boiler: BoilerClass::Hr107,
                    temperature_level: None,
                    emission: Some(EmissionDesign {
                        average_design_temperature_c: 40.0,
                        system: EmissionSystem::Direct,
                    }),
                },
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(f, 1.0 / 0.925);
        // Outdoors: the full-load value less 5 points.
        let (f, _, _) = generator_factors(
            &GeneratorKind::Boiler {
                carrier: SystemCarrier::NaturalGas,
                efficiency: BoilerEfficiency::FullLoad {
                    value: 0.97,
                    outdoor_installation: true,
                    source_reference: "test".into(),
                },
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        )
        .unwrap();
        close(f, 1.0 / 0.92);
        assert!(issues.is_empty(), "{issues:?}");
        for (kind, code) in [
            (
                GeneratorKind::Boiler {
                    carrier: SystemCarrier::Oil,
                    efficiency: BoilerEfficiency::TableP3 {
                        boiler: BoilerClass::Hr107,
                        temperature_level: Some(TemperatureLevel::High),
                        emission: None,
                    },
                },
                "oil_boiler_conventional_only",
            ),
            (
                GeneratorKind::Boiler {
                    carrier: SystemCarrier::NaturalGas,
                    efficiency: BoilerEfficiency::TableP3 {
                        boiler: BoilerClass::Hr107,
                        temperature_level: None,
                        emission: None,
                    },
                },
                "temperature_level_required",
            ),
            (
                GeneratorKind::CompressionChiller {
                    variant: ChillerVariant::Unspecified,
                    drive: SystemCarrier::NaturalGas,
                    engine_efficiency: None,
                },
                "engine_efficiency_required",
            ),
        ] {
            let function = if matches!(kind, GeneratorKind::CompressionChiller { .. }) {
                SystemFunction::Cooling
            } else {
                SystemFunction::Heating
            };
            let mut issues = Vec::new();
            generator_factors(&kind, function, "g", &mut issues);
            assert!(
                issues.iter().any(|item| item.code == code),
                "{code}: {issues:?}"
            );
        }
        let mut issues = Vec::new();
        generator_factors(
            &GeneratorKind::FreeCooling {
                source: FreeCoolingSource::OtherLowTemperatureSource,
                drive: SystemCarrier::Electricity {
                    direct_renewable_share: 0.0,
                },
            },
            SystemFunction::Heating,
            "g",
            &mut issues,
        );
        assert_eq!(issues[0].code, "generator_kind_requires_cooling");
    }

    #[test]
    fn flex_mode_follows_p6_5_4_11() {
        let flex = GeneratorKind::ElectricFlex {
            generator: FlexGenerator::ElectrodeBoiler {
                efficiency: None,
                efficiency_reference: None,
            },
            flex_heat_kwh: None,
            network_production_kwh: None,
            flex_reference: None,
            connections: 600,
            heat_buffer: true,
            registration_reference: "hourly register".into(),
        };
        let mut boiler = generator("boiler", None, gas(0.9));
        boiler.energy_fraction = Some(0.8);
        let mut electric = generator("e-boiler", Some(100.0), flex);
        electric.energy_fraction = Some(0.2);
        let mut input = system(vec![boiler, electric]);
        let result = calculated(&input, "s").unwrap();
        // Q_tot = 0,2·1 250 000 = 250 000; Q_flex = min(100·1 500,
        // 0,15·1 250 000) = 150 000 → 60 % flex (tables 5.5/5.6: 0).
        let item = &result.generators[1];
        close(item.primary_factor, 0.4 * 1.45 / 0.99);
        close(item.co2_kg_per_kwh, 0.4 * 0.268 / 0.99);
        // 5.8.3.1 g).
        close(item.renewable_factor, 1.0 - 0.4 / 0.99);
        // The 15 % cap on a supplied network total (heat and hot water):
        // min(150 000, 0,15·600 000) = 90 000 → 36 % flex.
        if let GeneratorKind::ElectricFlex {
            network_production_kwh,
            ..
        } = &mut input.generators[1].kind
        {
            *network_production_kwh = Some(600_000.0);
        }
        let item = &calculated(&input, "s").unwrap().generators[1];
        close(item.primary_factor, 0.64 * 1.45 / 0.99);
        // Fewer than 500 connections (5.8).
        if let GeneratorKind::ElectricFlex { connections, .. } = &mut input.generators[1].kind {
            *connections = 100;
        }
        assert_eq!(
            codes(calculated(&input, "s")),
            vec!["flex_mode_conditions_not_met"]
        );
    }

    fn buried_segment() -> PipeSegment {
        PipeSegment {
            length_m: 1000.0,
            layers: vec![PipeLayer {
                conductivity_w_per_mk: 0.0275,
                inner_diameter_m: 0.1,
                outer_diameter_m: 0.2,
            }],
            placement: PipePlacement::Buried {
                cover_depth_m: 0.8,
                ground_conductivity: None,
                ambient: PipeAmbient::Outdoor,
            },
            correction: Some(PipeCorrection::TwoPipesInTrench),
            correction_factor: None,
            resistance_km_per_w: None,
        }
    }

    #[test]
    fn pipe_resistance_follows_p17_and_p18() {
        let mut issues = Vec::new();
        // P.17: λ rounded up to 0,028; 1,05·(ln 2/(2π·0,028) + ln 16/(2π·1,75)).
        let pi = std::f64::consts::PI;
        let buried = 1.05 * (2f64.ln() / (2.0 * pi * 0.028) + 16f64.ln() / (2.0 * pi * 1.75));
        close(
            pipe_resistance(&buried_segment(), "p", &mut issues).unwrap(),
            buried,
        );
        // P.18 in a crawl space: h_a 8, f_x 0,90 by default.
        let mut air = buried_segment();
        air.correction = None;
        air.placement = PipePlacement::InAir {
            ambient: PipeAmbient::Crawlspace,
            surface_coefficient: None,
        };
        let in_air = 0.9 * (2f64.ln() / (2.0 * pi * 0.028) + 1.0 / (pi * 8.0 * 0.2));
        close(pipe_resistance(&air, "p", &mut issues).unwrap(), in_air);
        assert!(issues.is_empty());
        let mut no_case = buried_segment();
        no_case.correction = None;
        assert!(pipe_resistance(&no_case, "p", &mut issues).is_none());
        assert_eq!(issues[0].code, "pipe_correction_required");
    }

    #[test]
    fn pipe_losses_per_month_and_per_bin() {
        let resistance = pipe_resistance(&buried_segment(), "p", &mut Vec::new()).unwrap();
        let mut input = system(vec![generator("boiler", None, gas(0.9))]);
        input.distribution = SystemDistribution::Pipes {
            segments: vec![buried_segment()],
            water_temperature: Some(NetworkWaterTemperature::Constant {
                temperature_c: 70.0,
            }),
            buffers: Vec::new(),
            supply_below_10_c: None,
            other_loss_kwh: 0.0,
            source_reference: "network design".into(),
        };
        let result = calculated(&input, "s").unwrap();
        // P.13/P.14: Σ t_mi/1 000·L·(70 − θ_e;avg;mi)/R with L = 1 000 m.
        let loss: f64 = (0..12)
            .map(|i| MONTH_HOURS[i] * (70.0 - OUTDOOR_TEMPERATURE_C[i]) / resistance)
            .sum();
        let details = result.calculation.unwrap();
        close(details.distribution_loss_kwh, loss);
        close(details.input_kwh, 1_000_000.0 + loss);
        // P.15 with table P.16 at 60 °C: Σ t·θ = 94 294 h·K by hand.
        if let SystemDistribution::Pipes {
            water_temperature, ..
        } = &mut input.distribution
        {
            *water_temperature = Some(NetworkWaterTemperature::OutdoorBins {
                curve: vec![CurvePoint {
                    outdoor_c: 0.0,
                    water_c: 60.0,
                }],
                off_above_outdoor_c: None,
            });
        }
        let result = calculated(&input, "s").unwrap();
        close(
            result.calculation.unwrap().distribution_loss_kwh,
            (60.0 * 8760.0 - 94_294.0) / resistance,
        );
        // Out of operation from June to August.
        if let SystemDistribution::Pipes {
            water_temperature, ..
        } = &mut input.distribution
        {
            let mut months = vec![Some(70.0); 12];
            for month in &mut months[5..8] {
                *month = None;
            }
            *water_temperature = Some(NetworkWaterTemperature::Monthly {
                temperatures_c: months,
            });
        }
        let result = calculated(&input, "s").unwrap();
        let summer: f64 = (5..8)
            .map(|i| MONTH_HOURS[i] * (70.0 - OUTDOOR_TEMPERATURE_C[i]) / resistance)
            .sum();
        close(
            result.calculation.unwrap().distribution_loss_kwh,
            loss - summer,
        );
        // A row without any month in operation is an unfilled input.
        if let SystemDistribution::Pipes {
            water_temperature, ..
        } = &mut input.distribution
        {
            *water_temperature = Some(NetworkWaterTemperature::Monthly {
                temperatures_c: vec![None; 12],
            });
        }
        let issues = calculated(&input, "s").unwrap_err();
        assert!(
            issues
                .iter()
                .any(|item| item.code == "network_temperature_required"),
            "{issues:?}"
        );
    }

    #[test]
    fn storage_losses_follow_p43_to_p46() {
        let mut issues = Vec::new();
        // P.44: 1 000 l, Ø ≥ 50 cm → 10 m²; 1,5·50·24/1 000 = 1,8 kWh/day.
        let vessel = StorageVessel {
            volume_l: Some(1000.0),
            diameter_at_least_50_cm: Some(true),
            insulation: Some(VesselInsulation::AtLeast20Mm),
            ..StorageVessel::default()
        };
        close(
            vessel_loss_kwh(&vessel, "v", &mut issues).unwrap(),
            10.0 * 1.5 * 50.0 * 24.0 / 1000.0 * 365.0,
        );
        // P.43 with a measured standby loss of 2 kWh/day at 45 K; 60/15 °C.
        let measured = StorageVessel {
            standby_loss_kwh_per_day: Some(2.0),
            standby_test_difference_k: Some(45.0),
            water_temperature_c: Some(60.0),
            ambient_temperature_c: Some(15.0),
            ..StorageVessel::default()
        };
        close(vessel_loss_kwh(&measured, "v", &mut issues).unwrap(), 730.0);
        assert!(issues.is_empty());
        let mut wd = system(vec![boiler(1.0, SystemCarrier::NaturalGas)]);
        wd.function = SystemFunction::HotWater;
        wd.hot_water_storage = Some(WdStorage::Calculated {
            vessels: vec![vessel],
            pipes: vec![ChargingPipe {
                length_m: 20.0,
                u_value_w_per_mk: None,
                outer_diameter_mm: Some(22.0),
                insulation_mm: Some(10.0),
                ambient_temperature_c: None,
            }],
            exchanger: Some(ExternalExchanger {
                nominal_power_kw: 100.0,
                insulated: true,
                specific_loss_w_per_kw: None,
            }),
            circulation_temperature_c: None,
            correction_factor: None,
            source_reference: "design".into(),
        });
        let result = calculated(&wd, "w").unwrap();
        // Without any component the calculated route would give η = 1.
        let mut empty = wd.clone();
        empty.hot_water_storage = Some(WdStorage::Calculated {
            vessels: Vec::new(),
            pipes: Vec::new(),
            exchanger: None,
            circulation_temperature_c: None,
            correction_factor: None,
            source_reference: "design".into(),
        });
        let issues = calculated(&empty, "w").unwrap_err();
        assert!(
            issues
                .iter()
                .any(|item| item.code == "storage_components_required"),
            "{issues:?}"
        );
        // P.45/P.46: (20·0,271·60·1,20 + 0,2·100)·8,76.
        let pipes = (20.0 * 0.271 * 60.0 * 1.2 + 20.0) * 8.76;
        close(
            result.storage_efficiency.unwrap(),
            1_250_000.0 / (1_250_000.0 + 6570.0 + pipes),
        );
    }

    fn area_plot() -> AreaPlot {
        AreaPlot {
            id: "p".into(),
            usable_area_m2: Some(80.0),
            heating: None,
            heating_forfait: Some(DwellingHeatType::Apartment),
            sorption_cooling: None,
            hot_water: None,
            hot_water_forfait: None,
            hot_water_via_delivery_set: false,
            cooling: None,
            dehumidification: None,
            source_reference: "plot".into(),
        }
    }

    #[test]
    fn area_demand_follows_p72_to_p83() {
        let detached = AreaPlot {
            id: "p1".into(),
            usable_area_m2: Some(100.0),
            heating_forfait: Some(DwellingHeatType::Detached),
            hot_water_forfait: Some(HotWaterUse::DwellingLowTemperature),
            hot_water_via_delivery_set: true,
            ..area_plot()
        };
        let mut issues = Vec::new();
        let area = AreaDemand {
            plots: vec![detached.clone()],
        };
        // P.80 + P.82: 45,86·100 + 29,17·100 kWh, monthly from the tables.
        let heat = area_demand(&area, SystemFunction::Heating, "a", &mut issues).unwrap();
        near(heat.annual, 4586.0 + 2917.0, 1e-6);
        near(
            heat.monthly.unwrap()[0],
            894.0 + 2917.0 * 744.0 / 8760.0,
            1e-6,
        );
        let forfait_months = heat.monthly.unwrap();
        let water = area_demand(&area, SystemFunction::HotWater, "a", &mut issues).unwrap();
        near(water.annual, 2917.0, 1e-6);
        // A supplied annual value overrides the forfait; without months P.74
        // splits that plot only, the other plot keeps its months.
        let supplied = AreaPlot {
            id: "p2".into(),
            heating: Some(PlotFlow {
                annual_kwh: Some(10_000.0),
                monthly_kwh: Vec::new(),
            }),
            heating_forfait: Some(DwellingHeatType::Apartment),
            hot_water_forfait: None,
            hot_water_via_delivery_set: false,
            ..detached
        };
        let area = AreaDemand {
            plots: vec![area.plots[0].clone(), supplied],
        };
        let heat = area_demand(&area, SystemFunction::Heating, "a", &mut issues).unwrap();
        near(heat.annual, 17_503.0, 1e-6);
        let weights: Vec<f64> = OUTDOOR_TEMPERATURE_C
            .iter()
            .map(|t| (18.0 - t).max(0.0))
            .collect();
        let sum: f64 = weights.iter().sum();
        near(
            heat.monthly.unwrap()[0],
            forfait_months[0] + 10_000.0 * weights[0] / sum,
            1e-6,
        );
        // July and August are above 18 °C: only the first plot's months.
        near(heat.monthly.unwrap()[6], forfait_months[6], 1e-6);
        assert!(issues.is_empty(), "{issues:?}");
        // As the delivery of a heat system.
        let mut input = system(vec![generator("boiler", None, gas(0.9))]);
        input.delivered_kwh = None;
        input.area_demand = Some(area);
        let result = calculated(&input, "s").unwrap();
        near(result.calculation.unwrap().delivered_kwh, 17_503.0, 1e-6);
        // Forfait values need the usable area.
        let mut missing = AreaDemand {
            plots: vec![AreaPlot {
                usable_area_m2: None,
                ..area_plot()
            }],
        };
        assert!(area_demand(&missing, SystemFunction::Heating, "a", &mut issues).is_none());
        assert_eq!(issues[0].code, "usable_area_required");
        missing.plots.clear();
        assert!(area_demand(&missing, SystemFunction::Heating, "a", &mut Vec::new()).is_none());
    }

    #[test]
    fn heat_auxiliary_follows_p56_to_p60() {
        let mut input = system(vec![generator("boiler", Some(500.0), gas(0.9))]);
        input.auxiliary_electricity_kwh = None;
        input.auxiliary = Some(AuxiliaryInput {
            distribution: DistributionAuxiliary::Forfait {
                network: Some(AuxiliaryNetwork::Primary),
                farthest_distance_km: Some(5.0),
            },
            solar_kwh: Some(1000.0),
        });
        let result = calculated(&input, "s").unwrap();
        let aux = result.calculation.clone().unwrap().auxiliary.unwrap();
        // Table P.11: 0,0018·5 km·1 250 000.
        close(aux.distribution_kwh, 0.0018 * 5.0 * 1_250_000.0);
        // P.59/P.60: 100 W·8 760 h + 1 W/kW·500 kW·Σ t_on with
        // Σ t_on = 1 250 000·1,1/500 (no month reaches t_mi).
        close(aux.generators_kwh, 876.0 + 1_250_000.0 * 1.1 / 1000.0);
        close(aux.solar_kwh, 1000.0);
        close(
            result.calculation.unwrap().auxiliary_electricity_kwh,
            aux.total_kwh,
        );
        close(
            result.generators[0].auxiliary_kwh.unwrap(),
            aux.generators_kwh,
        );
        // P.57: two pumps of 1 kW and 0,5 kW all year; a heat pump on table
        // P.5 has no source-pump term (0 W/kW), standby only.
        input.auxiliary = Some(AuxiliaryInput {
            distribution: DistributionAuxiliary::Pumps {
                pump_powers_w: vec![1000.0, 500.0],
                operating_hours: None,
                source_reference: "pump schedule".into(),
            },
            solar_kwh: None,
        });
        input.generators[0] = generator("hp", Some(500.0), groundwater_heat_pump());
        let aux = calculated(&input, "s")
            .unwrap()
            .calculation
            .unwrap()
            .auxiliary
            .unwrap();
        close(aux.distribution_kwh, 1.5 * 8760.0);
        close(aux.generators_kwh, 876.0);
        // A supplied total overrides the calculation.
        input.auxiliary_electricity_kwh = Some(5.0);
        let details = calculated(&input, "s").unwrap().calculation.unwrap();
        close(details.auxiliary_electricity_kwh, 5.0);
        assert!(details.auxiliary.is_none());
        // Table P.11 has no value for a combined network beyond 3 km.
        input.auxiliary_electricity_kwh = None;
        input.auxiliary = Some(AuxiliaryInput {
            distribution: DistributionAuxiliary::Forfait {
                network: Some(AuxiliaryNetwork::PrimaryAndSecondary),
                farthest_distance_km: Some(4.0),
            },
            solar_kwh: None,
        });
        assert_eq!(codes(calculated(&input, "s")), vec!["table_p11_no_value"]);
        input.auxiliary = None;
        assert_eq!(
            codes(calculated(&input, "s")),
            vec!["auxiliary_energy_required"]
        );
    }

    #[test]
    fn cold_auxiliary_follows_p66_to_p70() {
        let mut chiller = generator(
            "ckm",
            None,
            GeneratorKind::CompressionChiller {
                variant: ChillerVariant::Unspecified,
                drive: SystemCarrier::Electricity {
                    direct_renewable_share: 0.0,
                },
                engine_efficiency: None,
            },
        );
        chiller.auxiliary = Some(GeneratorAuxiliary {
            heat_rejection: Some(HeatRejection::DryCooler),
            ..GeneratorAuxiliary::default()
        });
        let mut cd = system(vec![chiller]);
        cd.function = SystemFunction::Cooling;
        cd.delivered_kwh = None;
        let mut months = vec![0.0; 12];
        months[4..9].copy_from_slice(&[10_000.0, 20_000.0, 30_000.0, 30_000.0, 10_000.0]);
        cd.area_demand = Some(AreaDemand {
            plots: vec![AreaPlot {
                heating_forfait: None,
                cooling: Some(PlotFlow {
                    annual_kwh: None,
                    monthly_kwh: months,
                }),
                ..area_plot()
            }],
        });
        cd.distribution = SystemDistribution::SmallColdForfait {
            supply_below_10_c: false,
        };
        cd.auxiliary_electricity_kwh = None;
        cd.auxiliary = Some(AuxiliaryInput {
            distribution: DistributionAuxiliary::PumpsMonthly {
                pump_powers_w: vec![500.0],
                source_reference: "pump".into(),
            },
            solar_kwh: None,
        });
        let result = calculated(&cd, "c").unwrap();
        let aux = result.calculation.unwrap().auxiliary.unwrap();
        // P.68: on from May to September (3 672 h) at 0,5 kW.
        let hours = 744.0 + 720.0 + 744.0 + 744.0 + 720.0;
        close(aux.distribution_kwh, 0.5 * hours);
        // P.69/P.70: 10 W standby in those months and 45 W/kW on the heat
        // rejected, 100 000·(1 + 3)/3, plus 10 % run-on.
        close(
            aux.generators_kwh,
            10.0 * hours / 1000.0 + 45.0 * 100_000.0 * 4.0 / 3.0 * 1.1 / 1000.0,
        );
        // P.67 needs the operating hours for cold.
        cd.auxiliary = Some(AuxiliaryInput {
            distribution: DistributionAuxiliary::Pumps {
                pump_powers_w: vec![500.0],
                operating_hours: None,
                source_reference: "pump".into(),
            },
            solar_kwh: None,
        });
        assert_eq!(
            codes(calculated(&cd, "c")),
            vec!["operating_hours_required"]
        );
        // P.6.10.3.3: forfait 0,009 kWh_e/kWh_th.
        cd.auxiliary = Some(AuxiliaryInput {
            distribution: DistributionAuxiliary::Forfait {
                network: None,
                farthest_distance_km: None,
            },
            solar_kwh: None,
        });
        let aux = calculated(&cd, "c")
            .unwrap()
            .calculation
            .unwrap()
            .auxiliary
            .unwrap();
        close(aux.distribution_kwh, 0.009 * 100_000.0);
        assert!(result_warnings_of(&cd).is_empty());
        // P.13–P.18 for cold: crawlspace pipes at a constant 6 °C gain heat
        // in winter, when no cold is delivered.
        let mut segment = buried_segment();
        segment.placement = PipePlacement::Buried {
            cover_depth_m: 0.8,
            ground_conductivity: None,
            ambient: PipeAmbient::Crawlspace,
        };
        cd.distribution = SystemDistribution::Pipes {
            segments: vec![segment],
            water_temperature: Some(NetworkWaterTemperature::Constant { temperature_c: 6.0 }),
            buffers: Vec::new(),
            supply_below_10_c: Some(true),
            other_loss_kwh: 0.0,
            source_reference: "network design".into(),
        };
        assert_eq!(
            result_warnings_of(&cd),
            vec!["cold_network_gain_outside_cooling_months"]
        );
        // Without a water temperature outside the cooling months, no gain.
        if let SystemDistribution::Pipes {
            water_temperature, ..
        } = &mut cd.distribution
        {
            let months = (0..12)
                .map(|month| (4..9).contains(&month).then_some(6.0))
                .collect();
            *water_temperature = Some(NetworkWaterTemperature::Monthly {
                temperatures_c: months,
            });
        }
        assert!(result_warnings_of(&cd).is_empty());
    }

    fn result_warnings_of(system: &CalculatedSystem) -> Vec<&'static str> {
        calculated(system, "c")
            .unwrap()
            .warnings
            .iter()
            .map(|item| item.code)
            .collect()
    }

    #[test]
    fn area_electricity_follows_p71() {
        let generators: Vec<AreaElectricityGenerator> = serde_json::from_str(
            r#"[
                {"kind": "declared", "id": "wind", "annualKwh": 12000,
                 "sourceReference": "DIN V 18599-9"},
                {"kind": "pv", "id": "field",
                 "peakPower": {"method": "panels", "panelPeakPowerW": 400, "panelCount": 100},
                 "azimuthDeg": 180, "tiltDeg": 30, "obstructionFactors": [1.0],
                 "sourceReference": "design"}
            ]"#,
        )
        .unwrap();
        let result = area_electricity(&generators, "e").unwrap();
        let AreaElectricityGenerator::Pv(pv) = &generators[1] else {
            unreachable!()
        };
        let pv_kwh: f64 = monthly_yield_kwh(pv, 0.0).iter().sum();
        assert!(pv_kwh > 0.0);
        close(result.total_kwh, 12_000.0 + pv_kwh);
        let bad: Vec<AreaElectricityGenerator> = serde_json::from_str(
            r#"[{"kind": "declared", "id": "x", "annualKwh": -1, "sourceReference": ""}]"#,
        )
        .unwrap();
        let found: Vec<_> = area_electricity(&bad, "e")
            .unwrap_err()
            .iter()
            .map(|item| item.code)
            .collect();
        assert_eq!(found, vec!["source_reference_required", "value_invalid"]);
    }

    #[test]
    fn calculated_route_reads_the_new_fields() {
        let route: AnnexPRoute = serde_json::from_str(
            r#"{
                "method": "calculated",
                "function": "heating",
                "areaDemand": {"plots": [
                    {"id": "a", "usableAreaM2": 5000, "heatingForfait": "terraced_or_utility",
                     "hotWaterForfait": "dwelling_low_temperature",
                     "hotWaterViaDeliverySet": true, "sourceReference": "plan"}
                ]},
                "distribution": {"method": "pipes", "sourceReference": "design",
                    "waterTemperature": {"method": "monthly", "temperaturesC":
                        [70, 70, 70, 70, 70, null, null, null, 70, 70, 70, 70]},
                    "segments": [{"lengthM": 300, "correction": "two_pipes_in_trench",
                        "layers": [{"conductivityWPerMk": 0.03, "innerDiameterM": 0.06,
                                    "outerDiameterM": 0.14}],
                        "placement": {"kind": "buried", "coverDepthM": 0.6,
                                      "ambient": {"kind": "outdoor"}}}]},
                "generators": [
                    {"id": "hp", "nominalPowerKw": 40, "kind": {"kind": "heat_pump",
                        "efficiency": {"method": "table_p5", "source": "electric_ground",
                                       "supplyTemperatureC": 55},
                        "drive": {"kind": "electricity"}}},
                    {"id": "ketel", "nominalPowerKw": 300, "kind": {"kind": "boiler",
                        "carrier": {"kind": "natural_gas"},
                        "efficiency": {"method": "table_p3", "boiler": "hr107",
                                       "temperatureLevel": "high"}},
                     "auxiliary": {"standbyW": 50, "sourceReference": "datasheet"}}
                ],
                "auxiliary": {"distribution": {"method": "forfait", "network": "secondary"}},
                "sourceReference": "annex P study"
            }"#,
        )
        .unwrap();
        let result = assess_route(&route, SystemFunction::Heating, "r").unwrap();
        let details = result.calculation.clone().unwrap();
        // P.80/P.82 on 5 000 m².
        near(
            details.delivered_kwh,
            table_p14(DwellingHeatType::TerracedOrUtility)
                .iter()
                .sum::<f64>()
                * 5000.0
                + 29.17 * 5000.0,
            1e-6,
        );
        assert_eq!(details.preferred, vec!["hp".to_string()]);
        assert!(details.distribution_loss_kwh > 0.0);
        let aux = details.auxiliary.unwrap();
        close(aux.distribution_kwh, 0.0018 * details.input_kwh);
        assert!(result.factors.primary_factor > 0.0);
        // Round trip.
        let text = serde_json::to_string(&route).unwrap();
        let again: AnnexPRoute = serde_json::from_str(&text).unwrap();
        let second = assess_route(&again, SystemFunction::Heating, "r").unwrap();
        close(second.factors.primary_factor, result.factors.primary_factor);
    }
}
