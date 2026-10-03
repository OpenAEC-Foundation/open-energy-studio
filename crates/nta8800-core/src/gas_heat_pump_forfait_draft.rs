//! Gas-engine and gas-absorption heat-pump COP lookup from NTA 8800:2025+C1:2026
//! tables 9.27/9.29 (p. 333–334, 337–338), first transcribed from the
//! consultation draft and checked against the final edition.
//! This selects a row only; gas input and auxiliary energy require a separate carrier audit.

use crate::{input_fingerprint, KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::json;

const DRAFT_SOURCE: &str = "NTA 8800:2025+C1:2026, tabellen 9.27 (p. 333–334) en 9.29 (p. 337–338)";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GasPumpDrive {
    GasEngine,
    Absorption,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GasPumpApplication {
    ResidentialCollectiveAtMost25Kw,
    Utility,
    CollectiveBuilding,
    Over25Kw,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GasPumpSource {
    Ground,
    OutdoorAir,
    ExhaustAir,
    GroundwaterAquifer,
    SurfaceWater,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GasHeatPumpForfaitDraftInput {
    pub generator_id: String,
    pub drive: GasPumpDrive,
    pub application: GasPumpApplication,
    pub application_reference: String,
    pub collective_building_installation: bool,
    pub external_heat_supply: bool,
    pub thermal_capacity_kw: f64,
    pub capacity_reference: String,
    pub source: GasPumpSource,
    pub source_reference: String,
    pub design_supply_temperature_c: f64,
    pub design_supply_reference: String,
    #[serde(default)]
    pub source_correction_factor: Option<f64>,
    #[serde(default)]
    pub source_correction_reference: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasPumpIssue {
    pub code: &'static str,
    pub path: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasHeatPumpForfaitDraftAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub consultation_source: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub table: &'static str,
    pub temperature_band_upper_c: Option<u8>,
    pub forfait_cop: Option<f64>,
    pub corrected_cop: Option<f64>,
    pub gas_input_energy_available: bool,
    pub auxiliary_energy_available: bool,
    pub final_edition_verified: bool,
    pub beng_calculation_available: bool,
    pub issues: Vec<GasPumpIssue>,
}

pub fn assess_gas_heat_pump_forfait_draft(
    input: &GasHeatPumpForfaitDraftInput,
) -> GasHeatPumpForfaitDraftAssessment {
    let mut issues = Vec::new();
    let mut add = |code, path| issues.push(GasPumpIssue { code, path });
    if input.generator_id.trim().is_empty() {
        add("generator_id_required", "generatorId");
    }
    for (path, reference) in [
        ("applicationReference", &input.application_reference),
        ("capacityReference", &input.capacity_reference),
        ("sourceReference", &input.source_reference),
        ("designSupplyReference", &input.design_supply_reference),
    ] {
        if reference.trim().is_empty() {
            add("source_required", path);
        }
    }
    if !input.thermal_capacity_kw.is_finite() || input.thermal_capacity_kw <= 0.0 {
        add("thermal_capacity_invalid", "thermalCapacityKw");
    }
    if matches!(
        input.application,
        GasPumpApplication::ResidentialCollectiveAtMost25Kw
            | GasPumpApplication::CollectiveBuilding
    ) && !input.collective_building_installation
    {
        add(
            "collective_installation_required",
            "collectiveBuildingInstallation",
        );
    }
    if input.application == GasPumpApplication::Over25Kw && input.thermal_capacity_kw <= 25.0 {
        add("over25_capacity_required", "thermalCapacityKw");
    }
    let residential = input.application == GasPumpApplication::ResidentialCollectiveAtMost25Kw;
    if residential && input.thermal_capacity_kw > 25.0 {
        add("residential_capacity_above25", "thermalCapacityKw");
    }
    if residential
        && matches!(
            input.source,
            GasPumpSource::ExhaustAir | GasPumpSource::SurfaceWater
        )
    {
        add("residential_source_unavailable", "source");
    }
    if input.application == GasPumpApplication::CollectiveBuilding
        && input.thermal_capacity_kw <= 25.0
    {
        add("collective_table_overlap_unresolved", "application");
    }
    let correction_required = residential
        && matches!(
            input.source,
            GasPumpSource::Ground | GasPumpSource::GroundwaterAquifer
        );
    if correction_required {
        match input.source_correction_factor {
            Some(factor) if factor.is_finite() && factor > 0.0 => {}
            _ => add("source_correction_required", "sourceCorrectionFactor"),
        }
        if input
            .source_correction_reference
            .as_deref()
            .map_or(true, |value| value.trim().is_empty())
        {
            add(
                "source_correction_reference_required",
                "sourceCorrectionReference",
            );
        }
    } else if input.source_correction_factor.is_some()
        || input.source_correction_reference.is_some()
    {
        add("source_correction_not_applicable", "sourceCorrectionFactor");
    }
    if input.external_heat_supply {
        add(
            "external_heat_supply_route_unavailable",
            "externalHeatSupply",
        );
    }
    let band = if input.design_supply_temperature_c.is_finite()
        && input.design_supply_temperature_c > 0.0
    {
        [30_u8, 35, 40, 45, 50, 55]
            .into_iter()
            .position(|upper| input.design_supply_temperature_c <= f64::from(upper))
    } else {
        None
    };
    if band.is_none() {
        add(
            "design_supply_temperature_out_of_scope",
            "designSupplyTemperatureC",
        );
    }
    let values = if residential {
        match input.source {
            GasPumpSource::Ground | GasPumpSource::GroundwaterAquifer => {
                [1.35, 1.3, 1.25, 1.2, 1.15, 1.1]
            }
            GasPumpSource::OutdoorAir => [1.25, 1.2, 1.15, 1.1, 1.05, 1.0],
            GasPumpSource::ExhaustAir | GasPumpSource::SurfaceWater => [0.0; 6],
        }
    } else {
        match input.source {
            GasPumpSource::Ground | GasPumpSource::OutdoorAir => [1.65, 1.6, 1.55, 1.5, 1.45, 1.4],
            GasPumpSource::ExhaustAir => [2.7, 2.6, 2.4, 2.2, 2.1, 2.0],
            GasPumpSource::GroundwaterAquifer => [2.2, 2.1, 2.0, 1.9, 1.85, 1.8],
            GasPumpSource::SurfaceWater => [1.95, 1.9, 1.85, 1.8, 1.75, 1.7],
        }
    };
    let corrected = band.map(|index| values[index] * input.source_correction_factor.unwrap_or(1.0));
    if corrected.is_some_and(|value| !value.is_finite() || value <= 0.0) {
        add("corrected_cop_invalid", "sourceCorrectionFactor");
    }
    let valid = issues.is_empty();
    GasHeatPumpForfaitDraftAssessment {
        status: if valid { "diagnostic_valid" } else { "invalid" },
        scope: "public_chapter_9_draft_tables_9_27_9_29_gas_engine_or_absorption_lookup_only",
        consultation_source: DRAFT_SOURCE,
        target_norm_version: TARGET_NORM_VERSION,
        kernel_version: KERNEL_VERSION,
        input_fingerprint: input_fingerprint(&json!(input)),
        table: if residential { "9.27" } else { "9.29" },
        temperature_band_upper_c: if valid {
            band.map(|index| [30, 35, 40, 45, 50, 55][index])
        } else {
            None
        },
        forfait_cop: if valid {
            band.map(|index| values[index])
        } else {
            None
        },
        corrected_cop: if valid { corrected } else { None },
        gas_input_energy_available: false,
        auxiliary_energy_available: false,
        final_edition_verified: false,
        beng_calculation_available: false,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SourceTableFixture {
        kind: String,
        source_url: String,
        source_pdf_sha256: String,
        temperature_band_upper_c: Vec<u8>,
        rows: Vec<SourceTableRow>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SourceTableRow {
        table: String,
        application: GasPumpApplication,
        source: GasPumpSource,
        cop: Vec<f64>,
        csource_required: bool,
    }

    fn sample() -> GasHeatPumpForfaitDraftInput {
        serde_json::from_value(json!({
            "generatorId":"gwp-1", "drive":"gas_engine", "application":"collective_building",
            "applicationReference":"building installation schedule", "collectiveBuildingInstallation":true,
            "externalHeatSupply":false, "thermalCapacityKw":40.0, "capacityReference":"plate",
            "source":"outdoor_air", "sourceReference":"product schedule",
            "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design"
        })).unwrap()
    }

    #[test]
    fn table_929_selects_gas_drives_sources_and_temperature_edges() {
        let mut input = sample();
        let result = assess_gas_heat_pump_forfait_draft(&input);
        assert_eq!(result.forfait_cop, Some(1.6));
        assert_eq!(result.temperature_band_upper_c, Some(35));
        assert!(!result.gas_input_energy_available && !result.beng_calculation_available);
        input.drive = GasPumpDrive::Absorption;
        input.source = GasPumpSource::ExhaustAir;
        input.design_supply_temperature_c = 40.0;
        assert_eq!(
            assess_gas_heat_pump_forfait_draft(&input).forfait_cop,
            Some(2.4)
        );
        input.source = GasPumpSource::SurfaceWater;
        input.design_supply_temperature_c = 30.0;
        assert_eq!(
            assess_gas_heat_pump_forfait_draft(&input).forfait_cop,
            Some(1.95)
        );
        input.source = GasPumpSource::GroundwaterAquifer;
        input.design_supply_temperature_c = 55.0;
        assert_eq!(
            assess_gas_heat_pump_forfait_draft(&input).forfait_cop,
            Some(1.8)
        );
    }

    #[test]
    fn every_public_gas_row_and_temperature_band_is_preserved() {
        let fixture: SourceTableFixture = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-2026-consultation-gas-cop-tables.json"
        ))
        .unwrap();
        assert_eq!(fixture.kind, "source_table_transcription_not_edr_reference");
        // The transcription was made from the consultation draft; the final
        // edition (DRAFT_SOURCE) has the same gas rows.
        assert_eq!(
            fixture.source_url,
            "https://www.internetconsultatie.nl/epg2026/document/14150"
        );
        assert_eq!(
            fixture.source_pdf_sha256,
            "92053f92d0490bc2fc5ae9fac3c99904f4d80aaf80a3bacc0f6c00f41661118a"
        );
        assert_eq!(fixture.temperature_band_upper_c, [30, 35, 40, 45, 50, 55]);
        assert_eq!(fixture.rows.len(), 8);
        for row in &fixture.rows {
            assert_eq!(row.cop.len(), fixture.temperature_band_upper_c.len());
            for drive in [GasPumpDrive::GasEngine, GasPumpDrive::Absorption] {
                for (&temperature, &cop) in fixture.temperature_band_upper_c.iter().zip(&row.cop) {
                    let mut input = sample();
                    input.drive = drive;
                    input.application = row.application;
                    input.thermal_capacity_kw =
                        if row.application == GasPumpApplication::ResidentialCollectiveAtMost25Kw {
                            25.0
                        } else {
                            40.0
                        };
                    input.source = row.source;
                    input.design_supply_temperature_c = f64::from(temperature);
                    if row.csource_required {
                        input.source_correction_factor = Some(1.0);
                        input.source_correction_reference =
                            Some("appendix V: no regeneration".into());
                    }
                    let actual = assess_gas_heat_pump_forfait_draft(&input);
                    assert_eq!(
                        actual.status, "diagnostic_valid",
                        "{} {drive:?} {temperature} °C",
                        row.table
                    );
                    assert_eq!(actual.table, row.table);
                    assert_eq!(actual.temperature_band_upper_c, Some(temperature));
                    assert_eq!(
                        actual.forfait_cop,
                        Some(cop),
                        "{} {drive:?} {:?} {temperature} °C",
                        row.table,
                        row.source
                    );
                    assert!(!actual.final_edition_verified && !actual.beng_calculation_available);
                }
            }
        }
    }

    #[test]
    fn residential_collective_table_927_requires_capacity_and_source_correction() {
        let mut input = sample();
        input.application = GasPumpApplication::ResidentialCollectiveAtMost25Kw;
        input.thermal_capacity_kw = 25.0;
        input.source = GasPumpSource::Ground;
        input.source_correction_factor = Some(1.0);
        input.source_correction_reference = Some("no regeneration; design record".into());
        let ground = assess_gas_heat_pump_forfait_draft(&input);
        assert_eq!(ground.table, "9.27");
        assert_eq!(ground.forfait_cop, Some(1.3));
        assert_eq!(ground.corrected_cop, Some(1.3));
        input.source = GasPumpSource::Ground;
        input.source_correction_factor = Some(1.1);
        input.source_correction_reference = Some("appendix V evidence".into());
        input.design_supply_temperature_c = 35.0;
        let corrected = assess_gas_heat_pump_forfait_draft(&input);
        assert_eq!(corrected.forfait_cop, Some(1.3));
        assert_eq!(corrected.corrected_cop, Some(1.3 * 1.1));
        input.source_correction_factor = None;
        assert!(assess_gas_heat_pump_forfait_draft(&input)
            .forfait_cop
            .is_none());
        input.source_correction_factor = Some(1.0);
        input.thermal_capacity_kw = 25.01;
        assert!(assess_gas_heat_pump_forfait_draft(&input)
            .forfait_cop
            .is_none());
        input.thermal_capacity_kw = 25.0;
        input.source = GasPumpSource::ExhaustAir;
        assert!(assess_gas_heat_pump_forfait_draft(&input)
            .forfait_cop
            .is_none());
    }

    #[test]
    fn invalid_scope_and_temperature_suppress_the_lookup() {
        let mut input = sample();
        input.application = GasPumpApplication::Over25Kw;
        input.thermal_capacity_kw = 25.0;
        input.design_supply_temperature_c = 55.01;
        let result = assess_gas_heat_pump_forfait_draft(&input);
        assert_eq!(result.status, "invalid");
        assert!(result.forfait_cop.is_none());
        input.application = GasPumpApplication::Utility;
        input.thermal_capacity_kw = 10.0;
        input.design_supply_temperature_c = 35.0;
        input.external_heat_supply = true;
        assert!(assess_gas_heat_pump_forfait_draft(&input)
            .forfait_cop
            .is_none());
        input.external_heat_supply = false;
        input.application = GasPumpApplication::CollectiveBuilding;
        input.collective_building_installation = true;
        input.thermal_capacity_kw = 25.0;
        assert!(assess_gas_heat_pump_forfait_draft(&input)
            .issues
            .iter()
            .any(|issue| issue.code == "collective_table_overlap_unresolved"));
    }
}
