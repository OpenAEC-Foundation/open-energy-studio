//! External heat, hot-water and cold supply (gebiedsmaatregelen),
//! NTA 8800:2025+C1:2026 §5.8 (pp. 128–138) and annex P (pp. 940–1012).
//!
//! A collective energy system `XD` (heat `HD`, hot water `WD`, cold `CD`)
//! is valued with one primary energy factor `f_P;XD;tot`, a CO2 coefficient
//! `K_CO2;XD;tot` and a renewable factor `f_Pren;dX`. Two routes:
//!
//! - **declared**: the values of a registered quality declaration
//!   (EMG-verklaring, §5.8.0);
//! - **calculated**: the annex P method with calculated (and optionally
//!   measured) flows, P.7/P.9 with the distribution efficiency P.10–P.12 or
//!   the forfait loss per connection (table P.0), the generator factors
//!   P.19–P.22 per generator kind (P.20/P.22 fuels and heat pumps with
//!   table P.5, P.26–P.30 CHP, P.6.5.4.7 residual heat, P.6.5.4.8
//!   geothermal energy, P.6.5.4.9 an upstream system) and the renewable
//!   factor 5.42/5.49/5.50 with 5.43–5.56; or the measured-flows formula
//!   P.6/P.6 (CO2).
//!
//! `f_P;XD;tot` is rounded up and `f_Pren;dX` down to a multiple of 0,01.
//! Not transcribed: the β-factor and capacity-based energy fractions
//! (P.6.5.3, P.6.7.3; fractions are supplied with a source), the detailed
//! pipe-loss method P.13–P.18 (losses are supplied as flows or taken from
//! table P.0), the forfait boiler/chiller efficiency tables (efficiencies
//! are declared), collective solar collectors and flex-mode electric
//! generators (supplied as declared generator factors).

use serde::{Deserialize, Serialize};

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
            Self::NaturalGas => 0.218,
            Self::Oil => 0.326,
            Self::Electricity {
                direct_renewable_share,
            } => K_CO2_EL * (1.0 - direct_renewable_share),
            Self::Biogas => 0.0 * 0.074,
            Self::BiomassAbove500Kw => 0.0 * 0.104,
            Self::WasteIncineration => 0.138,
            Self::BiofuelMix { biofuel_share } => 0.218 * (1.0 - biofuel_share),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum HeatPumpEfficiency {
    Declared {
        value: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
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
    /// P.26/P.27: CHP without power loss.
    ChpWithoutLoss {
        carrier: SystemCarrier,
        #[serde(rename = "thermalEfficiency")]
        thermal_efficiency: f64,
        #[serde(rename = "electricalEfficiency")]
        electrical_efficiency: f64,
        #[serde(rename = "efficiencyReference")]
        efficiency_reference: String,
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemGenerator {
    pub id: String,
    /// `F_XD;gen;gi` (P.6.5.3/P.6.7.3), from operating data or a design
    /// calculation.
    pub energy_fraction: f64,
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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalculatedSystem {
    pub function: SystemFunction,
    /// `η_WD;gen;sto` (P.34); required for hot water (WD), not allowed
    /// otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hot_water_storage: Option<WdStorage>,
    /// `Q_XD;out;tot` (P.8), kWh per year.
    pub delivered_kwh: f64,
    pub distribution: SystemDistribution,
    pub generators: Vec<SystemGenerator>,
    /// `W_XD;aux;tot` (P.6.8–P.6.10), kWh per year.
    pub auxiliary_electricity_kwh: f64,
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
}

fn issue(code: &'static str, path: impl Into<String>) -> AnnexPIssue {
    AnnexPIssue {
        code,
        path: path.into(),
    }
}

fn round_up(value: f64) -> f64 {
    (value * 100.0 - 1e-9).ceil() / 100.0
}

fn round_down(value: f64) -> f64 {
    (value * 100.0 + 1e-9).floor() / 100.0
}

fn reference(value: &str, path: String, issues: &mut Vec<AnnexPIssue>) {
    if value.trim().is_empty() {
        issues.push(issue("source_reference_required", path));
    }
}

/// `(f_XD;gen;gi, K_CO2;gen;gi, f_Pren;XD;gi)`.
fn generator_factors(
    kind: &GeneratorKind,
    function: SystemFunction,
    path: &str,
    issues: &mut Vec<AnnexPIssue>,
) -> Option<(f64, f64, f64)> {
    let positive = |value: f64| value.is_finite() && value > 0.0;
    match kind {
        GeneratorKind::Combustion {
            carrier,
            efficiency,
            efficiency_reference,
        } => {
            reference(
                efficiency_reference,
                format!("{path}.kind.efficiencyReference"),
                issues,
            );
            if !carrier.valid() {
                issues.push(issue(
                    "carrier_share_invalid",
                    format!("{path}.kind.carrier"),
                ));
            }
            if !positive(*efficiency) {
                issues.push(issue(
                    "generator_efficiency_invalid",
                    format!("{path}.kind.efficiency"),
                ));
                return None;
            }
            Some((
                carrier.primary_factor() / efficiency,
                carrier.co2() / efficiency,
                carrier_renewable(carrier),
            ))
        }
        GeneratorKind::HeatPump { efficiency, drive } => {
            let eta = match efficiency {
                HeatPumpEfficiency::Declared {
                    value,
                    source_reference,
                } => {
                    reference(
                        source_reference,
                        format!("{path}.kind.efficiency.sourceReference"),
                        issues,
                    );
                    Some(*value)
                }
                HeatPumpEfficiency::TableP5 {
                    source,
                    supply_temperature_c,
                } => {
                    let electric_drive = matches!(drive, SystemCarrier::Electricity { .. });
                    if source.electric() != electric_drive {
                        issues.push(issue(
                            "heat_pump_drive_table_mismatch",
                            format!("{path}.kind.drive"),
                        ));
                    }
                    // P.6.6.5.4: no forfait values for hot water.
                    if function != SystemFunction::Heating {
                        issues.push(issue(
                            "table_p5_heating_only",
                            format!("{path}.kind.efficiency"),
                        ));
                    }
                    table_p5(*source, *supply_temperature_c)
                }
            };
            if !drive.valid()
                || !matches!(
                    drive,
                    SystemCarrier::Electricity { .. } | SystemCarrier::NaturalGas
                )
            {
                issues.push(issue(
                    "heat_pump_drive_invalid",
                    format!("{path}.kind.drive"),
                ));
            }
            let Some(eta) = eta.filter(|value| positive(*value)) else {
                issues.push(issue(
                    "generator_efficiency_invalid",
                    format!("{path}.kind.efficiency"),
                ));
                return None;
            };
            let renewable = match function {
                // 5.49: cold generators with η ≥ 8 count fully.
                SystemFunction::Cooling => {
                    if eta >= 8.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                // 5.44/5.52 for η ≥ 1.
                _ => {
                    if eta >= 1.0 {
                        1.0 - 1.0 / eta
                    } else {
                        0.0
                    }
                }
            };
            Some((drive.primary_factor() / eta, drive.co2() / eta, renewable))
        }
        GeneratorKind::ChpWithoutLoss {
            carrier,
            thermal_efficiency,
            electrical_efficiency,
            efficiency_reference,
        } => {
            reference(
                efficiency_reference,
                format!("{path}.kind.efficiencyReference"),
                issues,
            );
            if !positive(*thermal_efficiency)
                || !electrical_efficiency.is_finite()
                || *electrical_efficiency < 0.0
            {
                issues.push(issue(
                    "generator_efficiency_invalid",
                    format!("{path}.kind.thermalEfficiency"),
                ));
                return None;
            }
            // P.26/P.27 (P.27 has no MAX(0): a renewable fuel gives a
            // negative K_CO2), renewable share 5.45/5.46.
            Some((
                (carrier.primary_factor() - electrical_efficiency * F_P_EL).max(0.0)
                    / thermal_efficiency,
                (carrier.co2() - electrical_efficiency * K_CO2_EL) / thermal_efficiency,
                carrier_renewable(carrier),
            ))
        }
        GeneratorKind::ChpWithLoss {
            carrier,
            loss_ratio,
            loss_ratio_reference,
        } => {
            let ratio = match loss_ratio {
                Some(ratio) => {
                    if loss_ratio_reference
                        .as_deref()
                        .map_or(true, |value| value.trim().is_empty())
                    {
                        issues.push(issue(
                            "source_reference_required",
                            format!("{path}.kind.lossRatioReference"),
                        ));
                    }
                    *ratio
                }
                None => CHP_LOSS_RATIO,
            };
            if !positive(ratio) {
                issues.push(issue(
                    "chp_loss_ratio_invalid",
                    format!("{path}.kind.lossRatio"),
                ));
                return None;
            }
            // P.28/P.30; AVI heat uses f_P;del;wi (P.6.5.4.6); renewable
            // share 5.45/5.46.
            Some((
                carrier.primary_factor() * ratio * F_P_EL,
                carrier.co2() * ratio * F_P_EL,
                carrier_renewable(carrier),
            ))
        }
        GeneratorKind::ResidualHeat {
            auxiliary_specific,
            auxiliary_reference,
        } => {
            let aux = match auxiliary_specific {
                Some(value) => {
                    if auxiliary_reference
                        .as_deref()
                        .map_or(true, |value| value.trim().is_empty())
                    {
                        issues.push(issue(
                            "source_reference_required",
                            format!("{path}.kind.auxiliaryReference"),
                        ));
                    }
                    *value
                }
                None => RESIDUAL_HEAT_AUX,
            };
            if !aux.is_finite() || !(0.0..1.0).contains(&aux) {
                issues.push(issue(
                    "residual_heat_auxiliary_invalid",
                    format!("{path}.kind.auxiliarySpecific"),
                ));
                return None;
            }
            // P.6.5.4.7 and 5.47/5.55.
            Some((aux * F_P_EL, aux * K_CO2_EL, 1.0 - aux))
        }
        GeneratorKind::Geothermal {
            source_temperature_c,
            return_temperature_c,
        } => {
            let delta = source_temperature_c - return_temperature_c - 3.0;
            let eta = GEOTHERMAL_EFFICIENCY_40K * delta / 40.0;
            if !positive(eta) || function == SystemFunction::Cooling {
                issues.push(issue(
                    "geothermal_temperature_invalid",
                    format!("{path}.kind.sourceTemperatureC"),
                ));
                return None;
            }
            // 5.48 generalised to 1 − 1/η.
            Some((
                F_P_EL / eta,
                K_CO2_EL / eta,
                if eta >= 1.0 { 1.0 - 1.0 / eta } else { 0.0 },
            ))
        }
        GeneratorKind::Declared {
            primary_factor,
            co2_kg_per_kwh,
            renewable_factor,
            source_reference,
        } => {
            reference(
                source_reference,
                format!("{path}.kind.sourceReference"),
                issues,
            );
            if !primary_factor.is_finite()
                || *primary_factor < 0.0
                || !co2_kg_per_kwh.is_finite()
                || !(0.0..=1.0).contains(renewable_factor)
            {
                issues.push(issue("generator_factor_invalid", format!("{path}.kind")));
                return None;
            }
            Some((*primary_factor, *co2_kg_per_kwh, *renewable_factor))
        }
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

/// The drive electricity of a generator that counts as directly renewable
/// (`W_XD;gen;ren`, 5.42), kWh.
fn renewable_drive(kind: &GeneratorKind, heat_kwh: f64, efficiency: Option<f64>) -> f64 {
    match (kind, efficiency) {
        (GeneratorKind::HeatPump { drive, .. }, Some(eta))
        | (GeneratorKind::Combustion { carrier: drive, .. }, Some(eta)) => {
            heat_kwh / eta * drive.renewable_electricity_share()
        }
        _ => 0.0,
    }
}

fn efficiency_of(kind: &GeneratorKind) -> Option<f64> {
    match kind {
        GeneratorKind::Combustion { efficiency, .. } => Some(*efficiency),
        GeneratorKind::HeatPump {
            efficiency: HeatPumpEfficiency::Declared { value, .. },
            ..
        } => Some(*value),
        GeneratorKind::HeatPump {
            efficiency:
                HeatPumpEfficiency::TableP5 {
                    source,
                    supply_temperature_c,
                },
            ..
        } => table_p5(*source, *supply_temperature_c),
        _ => None,
    }
}

fn calculated(system: &CalculatedSystem, path: &str) -> Result<SystemResult, Vec<AnnexPIssue>> {
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
    for (field, value) in [
        ("auxiliaryElectricityKwh", system.auxiliary_electricity_kwh),
        ("auxiliaryRenewableShare", system.auxiliary_renewable_share),
    ] {
        if !value.is_finite() || value < 0.0 {
            issues.push(issue("value_invalid", format!("{path}.{field}")));
        }
    }
    if system.auxiliary_renewable_share > 1.0 {
        issues.push(issue(
            "value_invalid",
            format!("{path}.auxiliaryRenewableShare"),
        ));
    }
    // P.10–P.12 and P.6.4.
    let input = match &system.distribution {
        SystemDistribution::Flows {
            input_kwh,
            loss_kwh,
            source_reference,
        } => {
            reference(
                source_reference,
                format!("{path}.distribution.sourceReference"),
                &mut issues,
            );
            match (input_kwh, loss_kwh) {
                (Some(input), None) => Some(*input),
                (None, Some(loss)) => Some(out + loss),
                _ => {
                    issues.push(issue(
                        "distribution_two_of_three_flows_required",
                        format!("{path}.distribution"),
                    ));
                    None
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
                issues.push(issue(
                    "small_system_forfait_invalid",
                    format!("{path}.distribution"),
                ));
            }
            if !other_loss_kwh.is_finite() || *other_loss_kwh < 0.0 {
                issues.push(issue(
                    "value_invalid",
                    format!("{path}.distribution.otherLossKwh"),
                ));
            }
            // Table P.0 footnote b: WD systems use 90/60.
            if system.function == SystemFunction::HotWater
                && design_temperature.is_some_and(|t| t != NetworkTemperature::T90To60)
            {
                issues.push(issue(
                    "hot_water_small_system_requires_90_60",
                    format!("{path}.distribution.designTemperature"),
                ));
            }
            let temperature = design_temperature.unwrap_or(NetworkTemperature::T90To60);
            Some(
                out + table_p0(temperature, *connection_type) * f64::from(*connections)
                    + other_loss_kwh,
            )
        }
        SystemDistribution::SmallColdForfait { supply_below_10_c } => {
            if system.function != SystemFunction::Cooling {
                issues.push(issue(
                    "small_cold_forfait_requires_cooling",
                    format!("{path}.distribution"),
                ));
            }
            Some(if *supply_below_10_c {
                out * (1.0 + COLD_FORFAIT_LOSS_SHARE)
            } else {
                out
            })
        }
    };
    if let Some(input) = input {
        if !input.is_finite() || input < out {
            issues.push(issue(
                "distribution_flows_inconsistent",
                format!("{path}.distribution"),
            ));
        }
    }
    // P.34/P.35 and P.6.6.4.3.
    let storage_efficiency = match (&system.hot_water_storage, system.function) {
        (None, SystemFunction::HotWater) => {
            issues.push(issue(
                "hot_water_storage_efficiency_required",
                format!("{path}.hotWaterStorage"),
            ));
            None
        }
        (None, _) => Some(1.0),
        (Some(_), SystemFunction::Heating | SystemFunction::Cooling) => {
            issues.push(issue(
                "hot_water_storage_only_for_hot_water",
                format!("{path}.hotWaterStorage"),
            ));
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
            reference(
                source_reference,
                format!("{path}.hotWaterStorage.sourceReference"),
                &mut issues,
            );
            let valid = |value: f64| value.is_finite() && value >= 0.0;
            match input {
                Some(input) if input > 0.0 && valid(*storage_loss_kwh) && valid(*pipe_loss_kwh) => {
                    Some(input / (input + storage_loss_kwh + pipe_loss_kwh))
                }
                _ => {
                    issues.push(issue("value_invalid", format!("{path}.hotWaterStorage")));
                    None
                }
            }
        }
    };
    if system.generators.is_empty() {
        issues.push(issue("generator_required", format!("{path}.generators")));
    }
    let fraction_sum: f64 = system.generators.iter().map(|g| g.energy_fraction).sum();
    if (fraction_sum - 1.0).abs() > 1e-6 {
        issues.push(issue(
            "energy_fractions_must_sum_to_one",
            format!("{path}.generators"),
        ));
    }
    let mut factors = Vec::new();
    for (index, generator) in system.generators.iter().enumerate() {
        let gpath = format!("{path}.generators[{index}]");
        if !(0.0..=1.0).contains(&generator.energy_fraction) {
            issues.push(issue(
                "energy_fraction_invalid",
                format!("{gpath}.energyFraction"),
            ));
        }
        factors.push(generator_factors(
            &generator.kind,
            system.function,
            &gpath,
            &mut issues,
        ));
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let input = input.expect("validated");
    let storage_efficiency = storage_efficiency.expect("validated");
    let efficiency = out / input;
    // P.19/P.21.
    let mut f_gen = 0.0;
    let mut k_gen = 0.0;
    let mut renewable_energy = 0.0;
    let mut generators = Vec::new();
    for (generator, factor) in system.generators.iter().zip(factors) {
        let (f, k, pren) = factor.expect("validated");
        f_gen += generator.energy_fraction * f;
        k_gen += generator.energy_fraction * k;
        // P.34: for WD the generators also cover the storage and pipe losses.
        let heat = generator.energy_fraction * input / storage_efficiency;
        // 5.42/5.49/5.50: Q_gen;gi·f_Pren;gi + W_gen;ren·f_Pren;elec.
        renewable_energy += heat * pren
            + renewable_drive(&generator.kind, heat, efficiency_of(&generator.kind)) * F_PREN_ELEC;
        generators.push(GeneratorResult {
            id: generator.id.clone(),
            primary_factor: f,
            co2_kg_per_kwh: k,
            renewable_factor: pren,
            heat_kwh: heat,
        });
    }
    // P.34: f_WD;gen;tot = f_WD;gen;tot;ex / η_WD;gen;sto (1 for HD/CD).
    let f_gen = f_gen / storage_efficiency;
    let k_gen = k_gen / storage_efficiency;
    let aux = system.auxiliary_electricity_kwh;
    let aux_share = system.auxiliary_renewable_share;
    renewable_energy += aux * aux_share * F_PREN_ELEC;
    // P.7 and P.9 with the (weighted) electricity factors.
    let primary = round_up(f_gen / efficiency + aux / out * F_P_EL * (1.0 - aux_share));
    let co2 = k_gen / efficiency + aux / out * K_CO2_EL * (1.0 - aux_share);
    let e_prim = out * primary;
    let renewable = if renewable_energy + e_prim > 0.0 {
        round_down(renewable_energy / (renewable_energy + e_prim))
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
        storage_efficiency: (system.function == SystemFunction::HotWater)
            .then_some(storage_efficiency),
        generators,
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
    let mut co2 = -system.exported_electricity_kwh * K_CO2_EL;
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
            renewable_factor: round_down(system.renewable_factor),
            co2_kg_per_kwh: co2 / out,
        },
        distribution_efficiency: None,
        generation_primary_factor: None,
        storage_efficiency: None,
        generators: Vec::new(),
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
                    renewable_factor: round_down(*renewable_factor),
                    co2_kg_per_kwh: *co2_kg_per_kwh,
                },
                distribution_efficiency: None,
                generation_primary_factor: None,
                storage_efficiency: None,
                generators: Vec::new(),
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
    let forfait = match source.temperature_class {
        SourceTemperatureClass::Below20C => SOURCE_BELOW_20_FORFAIT,
        SourceTemperatureClass::AtLeast20COrSurfaceWaterOrUnknown => HEAT_FORFAIT,
    };
    let declared_factors = declared.as_ref().map_or(forfait, |item| item.factors);
    let forfait_scenario = match source.temperature_class {
        SourceTemperatureClass::Below20C => declared_factors,
        SourceTemperatureClass::AtLeast20COrSurfaceWaterOrUnknown => HEAT_FORFAIT,
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
            delivered_kwh: 1_000_000.0,
            distribution: SystemDistribution::Flows {
                input_kwh: None,
                loss_kwh: Some(250_000.0),
                source_reference: "network model".into(),
            },
            generators,
            auxiliary_electricity_kwh: 20_000.0,
            auxiliary_renewable_share: 0.0,
            source_reference: "design".into(),
        }
    }

    #[test]
    fn calculated_route_follows_p7_p9_and_5_42() {
        let generators = vec![
            SystemGenerator {
                id: "hp".into(),
                energy_fraction: 0.7,
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
                energy_fraction: 0.3,
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
                thermal_efficiency: 0.51,
                electrical_efficiency: 0.30,
                efficiency_reference: "table P.6".into(),
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
            energy_fraction: 1.0,
            kind: GeneratorKind::Combustion {
                carrier: SystemCarrier::NaturalGas,
                efficiency: 1.0,
                efficiency_reference: "x".into(),
            },
        }]);
        input.delivered_kwh = 100_000.0;
        input.auxiliary_electricity_kwh = 0.0;
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
            annex_p: None,
        };
        let (factors, _) = source_factors(&source, "s").unwrap();
        close(factors.declared.primary_factor, 1.45 / 23.0);
        assert_eq!(factors.declared, factors.forfait);
        let warm = CollectiveHeatPumpSource {
            temperature_class: SourceTemperatureClass::AtLeast20COrSurfaceWaterOrUnknown,
            supplier_reference: "invoice".into(),
            annex_p: Some(AnnexPRoute::Declared {
                primary_factor: 0.3,
                renewable_factor: 0.5,
                co2_kg_per_kwh: 0.05,
                declaration_reference: "EMG".into(),
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
            energy_fraction: 0.5,
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
            energy_fraction: fraction,
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
        wd.auxiliary_electricity_kwh = 0.0;
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
            energy_fraction: 1.0,
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
                thermal_efficiency: 0.5,
                electrical_efficiency: 0.36,
                efficiency_reference: "test".into(),
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
}
