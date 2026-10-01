//! Space-cooling generation, NTA 8800 §10.5 (partial), from the chapter 7
//! cooling need.
//!
//! `Q_C;gen;pref = Q_C;nd / (η_C;em · η_C;dis · f_reg)` with supplied emission,
//! distribution and control efficiencies. Generation (forfait method 3):
//! compression `E_el = Q / (EER · f_prpr)` (10.76, table 10.29, EER 3,00),
//! gas absorption `E_gas = Q / (ζ · f_prpr)` (10.77, table 10.30, ζ 0,80),
//! free cooling `W_el = Q / EER_fc` (10.86, table 10.34) with an optional
//! compression backup for the remaining fraction. Free cooling with
//! EER ≥ 8 delivers ambient cold `rencold` (5.34, `f_Pren` 1,0).
//!
//! Transcription source: Open Heatloss Studio analysis F3b (2026-07-11);
//! the generator priority split of tables 10.15/10.16 is not transcribed,
//! so the free-cooling fraction is supplied with a source.

use serde::{Deserialize, Serialize};

/// Table 10.29: forfait EER of an electric compression chiller.
pub const EER_COMPRESSION_FORFAIT: f64 = 3.0;
/// Table 10.30: forfait heat ratio of gas-fired absorption cooling.
pub const ZETA_GAS_ABSORPTION: f64 = 0.8;
/// 5.34: minimum EER for ambient cold to count as renewable.
pub const RENCOLD_MIN_EER: f64 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FreeCoolingSource {
    AquiferDwellingsFrom2013,
    AquiferUtilityBefore2013,
    AquiferDwellingsBefore2013,
    SurfaceWater,
    ClosedGroundLoop,
    DewPointCooling,
}

impl FreeCoolingSource {
    /// Table 10.34.
    pub fn eer(self) -> f64 {
        match self {
            Self::AquiferDwellingsFrom2013 => 23.0,
            Self::AquiferUtilityBefore2013 => 16.0,
            Self::AquiferDwellingsBefore2013 => 14.0,
            Self::SurfaceWater | Self::ClosedGroundLoop => 10.0,
            Self::DewPointCooling => 8.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CoolingGenerator {
    Compression,
    GasAbsorption,
    FreeCooling {
        source: FreeCoolingSource,
        /// Share of the cold delivered by free cooling; the rest by a
        /// compression backup at the forfait EER.
        #[serde(rename = "freeCoolingFraction")]
        free_cooling_fraction: f64,
        #[serde(rename = "fractionSourceReference")]
        fraction_source_reference: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoolingSystem {
    pub emission_efficiency: f64,
    pub distribution_efficiency: f64,
    pub control_factor: f64,
    pub efficiency_source_reference: String,
    pub generator: CoolingGenerator,
    pub equipment_reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoolingIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoolingMonth {
    pub cold_delivered_kwh: f64,
    pub electricity_kwh: f64,
    pub natural_gas_kwh: f64,
    pub ambient_cold_kwh: f64,
}

pub fn validate_cooling(system: &CoolingSystem, path: &str) -> Vec<CoolingIssue> {
    let mut issues = Vec::new();
    let mut push = |code, field: &str| {
        issues.push(CoolingIssue {
            code,
            path: format!("{path}.{field}"),
        })
    };
    for (field, value) in [
        ("emissionEfficiency", system.emission_efficiency),
        ("distributionEfficiency", system.distribution_efficiency),
        ("controlFactor", system.control_factor),
    ] {
        if !value.is_finite() || value <= 0.0 || value > 1.0 {
            push("cooling_efficiency_invalid", field);
        }
    }
    for (field, value) in [
        (
            "efficiencySourceReference",
            &system.efficiency_source_reference,
        ),
        ("equipmentReference", &system.equipment_reference),
    ] {
        if value.trim().is_empty() {
            push("source_reference_required", field);
        }
    }
    if let CoolingGenerator::FreeCooling {
        free_cooling_fraction,
        fraction_source_reference,
        ..
    } = &system.generator
    {
        if !(0.0..=1.0).contains(free_cooling_fraction) {
            push(
                "free_cooling_fraction_invalid",
                "generator.freeCoolingFraction",
            );
        }
        if fraction_source_reference.trim().is_empty() {
            push(
                "source_reference_required",
                "generator.fractionSourceReference",
            );
        }
    }
    issues
}

/// One month; call only after [`validate_cooling`] returned no issues.
pub fn cooling_month(system: &CoolingSystem, cooling_need_kwh: f64) -> CoolingMonth {
    let cold = cooling_need_kwh
        / (system.emission_efficiency * system.distribution_efficiency * system.control_factor);
    match &system.generator {
        CoolingGenerator::Compression => CoolingMonth {
            cold_delivered_kwh: cold,
            electricity_kwh: cold / EER_COMPRESSION_FORFAIT,
            natural_gas_kwh: 0.0,
            ambient_cold_kwh: 0.0,
        },
        CoolingGenerator::GasAbsorption => CoolingMonth {
            cold_delivered_kwh: cold,
            electricity_kwh: 0.0,
            natural_gas_kwh: cold / ZETA_GAS_ABSORPTION,
            ambient_cold_kwh: 0.0,
        },
        CoolingGenerator::FreeCooling {
            source,
            free_cooling_fraction,
            ..
        } => {
            let free = cold * free_cooling_fraction;
            let eer = source.eer();
            CoolingMonth {
                cold_delivered_kwh: cold,
                electricity_kwh: free / eer + (cold - free) / EER_COMPRESSION_FORFAIT,
                natural_gas_kwh: 0.0,
                ambient_cold_kwh: if eer >= RENCOLD_MIN_EER { free } else { 0.0 },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system(generator: CoolingGenerator) -> CoolingSystem {
        CoolingSystem {
            emission_efficiency: 0.9,
            distribution_efficiency: 1.0,
            control_factor: 1.0,
            efficiency_source_reference: "design".into(),
            generator,
            equipment_reference: "plate".into(),
        }
    }

    #[test]
    fn generation_routes_follow_10_76_10_77_and_10_86() {
        let compression = cooling_month(&system(CoolingGenerator::Compression), 90.0);
        assert!((compression.cold_delivered_kwh - 100.0).abs() < 1e-12);
        assert!((compression.electricity_kwh - 100.0 / 3.0).abs() < 1e-12);
        let absorption = cooling_month(&system(CoolingGenerator::GasAbsorption), 90.0);
        assert!((absorption.natural_gas_kwh - 125.0).abs() < 1e-12);
        let free = cooling_month(
            &system(CoolingGenerator::FreeCooling {
                source: FreeCoolingSource::ClosedGroundLoop,
                free_cooling_fraction: 0.4,
                fraction_source_reference: "design".into(),
            }),
            90.0,
        );
        assert!((free.electricity_kwh - (40.0 / 10.0 + 60.0 / 3.0)).abs() < 1e-12);
        assert!((free.ambient_cold_kwh - 40.0).abs() < 1e-12);
        assert_eq!(FreeCoolingSource::AquiferDwellingsFrom2013.eer(), 23.0);
        assert_eq!(FreeCoolingSource::DewPointCooling.eer(), 8.0);
    }

    #[test]
    fn validation_rejects_bad_efficiencies_and_fraction() {
        let mut bad = system(CoolingGenerator::FreeCooling {
            source: FreeCoolingSource::SurfaceWater,
            free_cooling_fraction: 1.2,
            fraction_source_reference: String::new(),
        });
        bad.control_factor = 0.0;
        let codes: Vec<_> = validate_cooling(&bad, "cooling")
            .iter()
            .map(|item| item.code)
            .collect();
        for code in [
            "cooling_efficiency_invalid",
            "free_cooling_fraction_invalid",
            "source_reference_required",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }
}
