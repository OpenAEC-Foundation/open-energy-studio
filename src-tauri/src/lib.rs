use tauri::image::Image;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

mod project_files;

use nta8800_core::norm_versions::{self, request, NormVersion};
use serde_json::Value;

/// Refuses a kernel result that holds a non-finite number. Tauri serializes
/// with `serde_json`, which would write NaN or infinity as `null`.
fn finite<T: serde::Serialize>(value: T) -> Result<T, String> {
    match nta8800_core::finite::first_non_finite(&value) {
        None => Ok(value),
        Some(path) => Err(format!("non_finite_result: {path}")),
    }
}

// Every NTA command takes an optional `normVersion` with the rules of the
// service (`nta8800_core::norm_versions::request`): an unknown identifier is
// `invalid_norm_version`; a command whose input carries its own edition gets
// the requested one written into it (never contradicting it); a diagnostic
// runs with the edition active; the result records `normVersion` and
// `targetNormVersion`, and an older edition gives `calculated_legacy_edition`.

const PROJECT_EDITION: &str = "/ntaCalculation/normVersion";

fn refused(error: request::EditionError) -> String {
    format!("{}: {}", error.code, error.message)
}

fn requested(norm_version: Option<Value>) -> Result<Option<NormVersion>, String> {
    request::parse_requested(norm_version.as_ref()).map_err(refused)
}

/// The finite result as JSON with the edition stamped on it.
fn stamped<T: serde::Serialize>(
    version: NormVersion,
    result: Result<T, String>,
) -> Result<Value, String> {
    let mut value = serde_json::to_value(finite(result?)?)
        .map_err(|error| format!("serialization_failed: {error}"))?;
    request::stamp(&mut value, version);
    Ok(value)
}

/// A diagnostic route: `run` with the requested edition active.
fn in_edition<T: serde::Serialize>(
    norm_version: Option<Value>,
    run: impl FnOnce() -> T,
) -> Result<Value, String> {
    let version = requested(norm_version)?.unwrap_or_default();
    if !version.implemented() {
        return Err(format!(
            "edition_not_implemented: The kernel has no profile for {}",
            version.label()
        ));
    }
    stamped(version, Ok(norm_versions::with_version(version, run)))
}

/// A route whose input carries its own edition at `pointer`: the requested
/// edition is written there, then `run` goes in the input's edition.
fn in_input_edition<T: serde::Serialize>(
    input: &mut Value,
    member: &str,
    pointer: &str,
    norm_version: Option<Value>,
    run: impl FnOnce(&Value) -> Result<T, String>,
) -> Result<Value, String> {
    if let Some(version) = requested(norm_version)? {
        request::place_in(input, member, pointer, version).map_err(refused)?;
    }
    let version = request::edition_at(input, pointer);
    let input = &*input;
    let result = norm_versions::with_version(version, || run(input));
    stamped(version, result)
}

/// Runs a kernel command off the main thread, so the window stays
/// responsive, and turns a panic into a `kernel_panic` error instead of
/// ending the app with every unsaved document. The edition is set inside
/// `run` by `with_version`, on the same blocking thread, and restored there
/// even on a panic.
async fn off_main<F>(run: F) -> Result<Value, String>
where
    F: FnOnce() -> Result<Value, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)).unwrap_or_else(|_| {
            Err("kernel_panic: the calculation stopped unexpectedly".to_string())
        })
    })
    .await
    .map_err(|error| format!("kernel_panic: {error}"))?
}

fn typed<T: serde::de::DeserializeOwned>(input: &Value) -> Result<T, String> {
    serde_json::from_value(input.clone()).map_err(|error| error.to_string())
}

#[tauri::command]
async fn validate_nta_project(
    project: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || validate_nta_project_blocking(project, norm_version)).await
}

fn validate_nta_project_blocking(
    mut project: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_input_edition(
        &mut project,
        "project",
        PROJECT_EDITION,
        norm_version,
        |project| nta8800_core::assess_json(project.clone()),
    )
}

/// Reference cases are compared in the current edition only.
#[tauri::command]
async fn compare_gas_heat_pump_chain_diagnostic(
    case: nta8800_core::gas_heat_pump_chain_reference::GasChainDiagnosticCase,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || compare_gas_heat_pump_chain_diagnostic_blocking(case, norm_version)).await
}

fn compare_gas_heat_pump_chain_diagnostic_blocking(
    case: nta8800_core::gas_heat_pump_chain_reference::GasChainDiagnosticCase,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    if requested(norm_version)?.is_some_and(|version| !version.is_default()) {
        return Err(
            "norm_version_not_applicable: Reference cases are compared in NTA 8800:2025+C1:2026 only"
                .into(),
        );
    }
    in_edition(None, || {
        nta8800_core::gas_heat_pump_chain_reference::compare_gas_heat_pump_chain_diagnostic(case)
    })
}

#[tauri::command]
async fn calculate_constructions(
    input: nta8800_core::envelope_elements::EnvelopeInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || calculate_constructions_blocking(input, norm_version)).await
}

fn calculate_constructions_blocking(
    input: nta8800_core::envelope_elements::EnvelopeInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::envelope_elements::assess_envelope(&input)
    })
}

#[tauri::command]
async fn assess_residential_survey(
    survey: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || assess_residential_survey_blocking(survey, norm_version)).await
}

fn assess_residential_survey_blocking(
    mut survey: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_input_edition(
        &mut survey,
        "survey",
        "/normVersion",
        norm_version,
        |survey| {
            Ok(nta8800_core::opname::assess_residential_survey(&typed(
                survey,
            )?))
        },
    )
}

#[tauri::command]
async fn assess_utility_survey(
    survey: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || assess_utility_survey_blocking(survey, norm_version)).await
}

fn assess_utility_survey_blocking(
    mut survey: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_input_edition(
        &mut survey,
        "survey",
        "/normVersion",
        norm_version,
        |survey| {
            Ok(nta8800_core::opname::utility::assess_utility_survey(
                &typed(survey)?,
            ))
        },
    )
}

/// Every variant is calculated in the base situation's edition.
#[tauri::command]
async fn assess_maatwerkadvies(input: Value, norm_version: Option<Value>) -> Result<Value, String> {
    off_main(move || assess_maatwerkadvies_blocking(input, norm_version)).await
}

fn assess_maatwerkadvies_blocking(
    mut input: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    let pointer = match input.pointer("/base/kind").and_then(Value::as_str) {
        Some("building") => "/base/input/normVersion",
        _ => "/base/project/ntaCalculation/normVersion",
    };
    in_input_edition(&mut input, "input", pointer, norm_version, |input| {
        nta8800_core::maatwerkadvies::assess_maatwerkadvies_json(input.clone())
    })
}

/// A relabel stays in the original's edition (BRL 9500-W §4.2.4); both
/// projects get the requested edition.
#[tauri::command]
async fn assess_relabel(
    original: Value,
    current: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || assess_relabel_blocking(original, current, norm_version)).await
}

fn assess_relabel_blocking(
    mut original: Value,
    mut current: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    if let Some(version) = requested(norm_version)? {
        request::place_in(&mut original, "original", PROJECT_EDITION, version).map_err(refused)?;
        request::place_in(&mut current, "current", PROJECT_EDITION, version).map_err(refused)?;
    }
    let version = request::edition_at(&original, PROJECT_EDITION);
    let result = norm_versions::with_version(version, || {
        nta8800_core::relabel::assess_relabel(&original, &current)
    });
    stamped(version, Ok(result))
}

#[tauri::command]
async fn calculate_building_performance(
    input: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || calculate_building_performance_blocking(input, norm_version)).await
}

fn calculate_building_performance_blocking(
    mut input: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_input_edition(&mut input, "input", "/normVersion", norm_version, |input| {
        Ok(nta8800_core::building_performance::assess_building_performance(&typed(input)?))
    })
}

#[tauri::command]
fn kernel_interpretations() -> Vec<nta8800_core::interpretations::InterpretationGroup> {
    nta8800_core::interpretations::kernel_interpretations()
}

#[tauri::command]
async fn calculate_project_performance(
    project: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || calculate_project_performance_blocking(project, norm_version)).await
}

fn calculate_project_performance_blocking(
    mut project: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_input_edition(
        &mut project,
        "project",
        PROJECT_EDITION,
        norm_version,
        |project| Ok(nta8800_core::project_performance::assess_project_performance(project)),
    )
}

/// What changes for the project in edition `norm_version` (default 2025+C1);
/// the project keeps its own edition.
#[tauri::command]
async fn compare_project_editions(
    project: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || compare_project_editions_blocking(project, norm_version)).await
}

fn compare_project_editions_blocking(
    project: Value,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    let target = requested(norm_version)?.unwrap_or_default();
    let report = nta8800_core::edition_switch::compare_project_editions(&project, target);
    serde_json::to_value(report).map_err(|error| error.to_string())
}

#[tauri::command]
async fn diagnose_declared_heating_table(
    input: nta8800_core::declared_heating_table::DeclaredHeatingTableInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_declared_heating_table_blocking(input, norm_version)).await
}

fn diagnose_declared_heating_table_blocking(
    input: nta8800_core::declared_heating_table::DeclaredHeatingTableInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::declared_heating_table::assess_declared_heating_table(&input)
    })
}

#[tauri::command]
async fn diagnose_forfait_heat_pump_draft(
    input: nta8800_core::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_forfait_heat_pump_draft_blocking(input, norm_version)).await
}

fn diagnose_forfait_heat_pump_draft_blocking(
    input: nta8800_core::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::forfait_heat_pump_draft::assess_forfait_heat_pump_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_gas_heat_pump_forfait_draft(
    input: nta8800_core::gas_heat_pump_forfait_draft::GasHeatPumpForfaitDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_gas_heat_pump_forfait_draft_blocking(input, norm_version)).await
}

fn diagnose_gas_heat_pump_forfait_draft_blocking(
    input: nta8800_core::gas_heat_pump_forfait_draft::GasHeatPumpForfaitDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::gas_heat_pump_forfait_draft::assess_gas_heat_pump_forfait_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_gas_heat_pump_aux_draft(
    input: nta8800_core::gas_heat_pump_aux_draft::GasHeatPumpAuxDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_gas_heat_pump_aux_draft_blocking(input, norm_version)).await
}

fn diagnose_gas_heat_pump_aux_draft_blocking(
    input: nta8800_core::gas_heat_pump_aux_draft::GasHeatPumpAuxDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::gas_heat_pump_aux_draft::assess_gas_heat_pump_aux_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_gas_heat_pump_monthly_draft(
    input: nta8800_core::gas_heat_pump_monthly_draft::GasHeatPumpMonthlyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_gas_heat_pump_monthly_draft_blocking(input, norm_version)).await
}

fn diagnose_gas_heat_pump_monthly_draft_blocking(
    input: nta8800_core::gas_heat_pump_monthly_draft::GasHeatPumpMonthlyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::gas_heat_pump_monthly_draft::assess_gas_heat_pump_monthly_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_gas_heat_pump_chain_draft(
    input: nta8800_core::gas_heat_pump_chain_draft::GasHeatPumpChainDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_gas_heat_pump_chain_draft_blocking(input, norm_version)).await
}

fn diagnose_gas_heat_pump_chain_draft_blocking(
    input: nta8800_core::gas_heat_pump_chain_draft::GasHeatPumpChainDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::gas_heat_pump_chain_draft::assess_gas_heat_pump_chain_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_gas_collective_source_draft(
    input: nta8800_core::gas_collective_source_draft::GasCollectiveSourceDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_gas_collective_source_draft_blocking(input, norm_version)).await
}

fn diagnose_gas_collective_source_draft_blocking(
    input: nta8800_core::gas_collective_source_draft::GasCollectiveSourceDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::gas_collective_source_draft::assess_gas_collective_source_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_forfait_heat_pump_monthly_draft(
    input: nta8800_core::forfait_heat_pump_monthly_draft::ForfaitHeatPumpMonthlyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_forfait_heat_pump_monthly_draft_blocking(input, norm_version)).await
}

fn diagnose_forfait_heat_pump_monthly_draft_blocking(
    input: nta8800_core::forfait_heat_pump_monthly_draft::ForfaitHeatPumpMonthlyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::forfait_heat_pump_monthly_draft::assess_forfait_heat_pump_monthly_draft(
            &input,
        )
    })
}

#[tauri::command]
async fn diagnose_generator_dispatch_draft(
    input: nta8800_core::generator_dispatch_draft::GeneratorDispatchDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_generator_dispatch_draft_blocking(input, norm_version)).await
}

fn diagnose_generator_dispatch_draft_blocking(
    input: nta8800_core::generator_dispatch_draft::GeneratorDispatchDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::generator_dispatch_draft::assess_generator_dispatch_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_hybrid_heat_pump_monthly_draft(
    input: nta8800_core::hybrid_heat_pump_monthly_draft::HybridHeatPumpMonthlyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_hybrid_heat_pump_monthly_draft_blocking(input, norm_version)).await
}

fn diagnose_hybrid_heat_pump_monthly_draft_blocking(
    input: nta8800_core::hybrid_heat_pump_monthly_draft::HybridHeatPumpMonthlyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::hybrid_heat_pump_monthly_draft::assess_hybrid_heat_pump_monthly_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_boiler_forfait_draft(
    input: nta8800_core::boiler_forfait_draft::BoilerForfaitDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_boiler_forfait_draft_blocking(input, norm_version)).await
}

fn diagnose_boiler_forfait_draft_blocking(
    input: nta8800_core::boiler_forfait_draft::BoilerForfaitDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::boiler_forfait_draft::assess_boiler_forfait_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_boiler_forfait_monthly_draft(
    input: nta8800_core::boiler_forfait_draft::BoilerForfaitMonthlyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_boiler_forfait_monthly_draft_blocking(input, norm_version)).await
}

fn diagnose_boiler_forfait_monthly_draft_blocking(
    input: nta8800_core::boiler_forfait_draft::BoilerForfaitMonthlyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::boiler_forfait_draft::assess_boiler_forfait_monthly_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_declared_dhw(
    input: nta8800_core::heat_pumps::HeatPumpInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_declared_dhw_blocking(input, norm_version)).await
}

fn diagnose_declared_dhw_blocking(
    input: nta8800_core::heat_pumps::HeatPumpInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::declared_dhw::assess_declared_dhw(&input)
    })
}

#[tauri::command]
async fn diagnose_final_energy_draft(
    input: nta8800_core::final_energy_draft::FinalEnergyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_final_energy_draft_blocking(input, norm_version)).await
}

fn diagnose_final_energy_draft_blocking(
    input: nta8800_core::final_energy_draft::FinalEnergyDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::final_energy_draft::assess_final_energy_draft(&input)
    })
}

#[tauri::command]
async fn calculate_monthly_demand(
    input: nta8800_core::monthly_demand::MonthlyDemandInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || calculate_monthly_demand_blocking(input, norm_version)).await
}

fn calculate_monthly_demand_blocking(
    input: nta8800_core::monthly_demand::MonthlyDemandInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::monthly_demand::assess_monthly_demand(&input)
    })
}

#[tauri::command]
async fn calculate_ventilation(
    input: nta8800_core::ventilation::VentilationInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || calculate_ventilation_blocking(input, norm_version)).await
}

fn calculate_ventilation_blocking(
    input: nta8800_core::ventilation::VentilationInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::ventilation::assess_ventilation(&input)
    })
}

#[tauri::command]
async fn calculate_space_heating_chain(
    input: nta8800_core::space_heating_chain::SpaceHeatingChainInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || calculate_space_heating_chain_blocking(input, norm_version)).await
}

fn calculate_space_heating_chain_blocking(
    input: nta8800_core::space_heating_chain::SpaceHeatingChainInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::space_heating_chain::assess_space_heating_chain(&input)
    })
}

#[tauri::command]
async fn diagnose_epus_draft(
    input: nta8800_core::epus_draft::EpusDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_epus_draft_blocking(input, norm_version)).await
}

fn diagnose_epus_draft_blocking(
    input: nta8800_core::epus_draft::EpusDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::epus_draft::assess_epus_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_bacs_draft(
    input: nta8800_core::bacs_draft::BacsDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_bacs_draft_blocking(input, norm_version)).await
}

fn diagnose_bacs_draft_blocking(
    input: nta8800_core::bacs_draft::BacsDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::bacs_draft::assess_bacs_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_indicators_draft(
    input: nta8800_core::indicators_draft::IndicatorsDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_indicators_draft_blocking(input, norm_version)).await
}

fn diagnose_indicators_draft_blocking(
    input: nta8800_core::indicators_draft::IndicatorsDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::indicators_draft::assess_indicators_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_heating_aux_draft(
    input: nta8800_core::heating_aux_draft::HeatingAuxDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_heating_aux_draft_blocking(input, norm_version)).await
}

fn diagnose_heating_aux_draft_blocking(
    input: nta8800_core::heating_aux_draft::HeatingAuxDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::heating_aux_draft::assess_heating_aux_draft(&input)
    })
}

#[tauri::command]
async fn diagnose_heating_aux_measured_draft(
    input: nta8800_core::heating_aux_draft::HeatingAuxMeasuredDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    off_main(move || diagnose_heating_aux_measured_draft_blocking(input, norm_version)).await
}

fn diagnose_heating_aux_measured_draft_blocking(
    input: nta8800_core::heating_aux_draft::HeatingAuxMeasuredDraftInput,
    norm_version: Option<Value>,
) -> Result<Value, String> {
    in_edition(norm_version, || {
        nta8800_core::heating_aux_draft::assess_heating_aux_measured_draft(&input)
    })
}

#[derive(serde::Serialize)]
struct PrinterInfo {
    name: String,
    is_default: bool,
}

#[derive(serde::Serialize)]
struct PrinterSettings {
    paper_width_mm: f64,
    paper_height_mm: f64,
    landscape: bool,
}

#[tauri::command]
fn list_printers() -> Vec<PrinterInfo> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance -ClassName Win32_Printer | Select-Object Name, Default | ConvertTo-Json -Compress",
            ])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // PowerShell returns a single object (not array) when there's only one printer
            if let Ok(printers) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                return printers
                    .iter()
                    .map(|p| PrinterInfo {
                        name: p["Name"].as_str().unwrap_or("").to_string(),
                        is_default: p["Default"].as_bool().unwrap_or(false),
                    })
                    .collect();
            } else if let Ok(single) = serde_json::from_str::<serde_json::Value>(&stdout) {
                return vec![PrinterInfo {
                    name: single["Name"].as_str().unwrap_or("").to_string(),
                    is_default: single["Default"].as_bool().unwrap_or(false),
                }];
            }
        }
        vec![]
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let mut printers = Vec::new();

        // Get default printer
        let default_name = Command::new("lpstat")
            .args(["-d"])
            .output()
            .ok()
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout).to_string();
                s.split(": ").nth(1).map(|n| n.trim().to_string())
            })
            .unwrap_or_default();

        // List all printers
        if let Ok(output) = Command::new("lpstat").args(["-p"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(name) = line.strip_prefix("printer ") {
                    if let Some(name) = name.split_whitespace().next() {
                        printers.push(PrinterInfo {
                            is_default: name == default_name,
                            name: name.to_string(),
                        });
                    }
                }
            }
        }
        printers
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let mut printers = Vec::new();

        let default_name = Command::new("lpstat")
            .args(["-d"])
            .output()
            .ok()
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout).to_string();
                s.split(": ").nth(1).map(|n| n.trim().to_string())
            })
            .unwrap_or_default();

        if let Ok(output) = Command::new("lpstat").args(["-p"]).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(name) = line.strip_prefix("printer ") {
                    if let Some(name) = name.split_whitespace().next() {
                        printers.push(PrinterInfo {
                            is_default: name == default_name,
                            name: name.to_string(),
                        });
                    }
                }
            }
        }
        printers
    }
}

#[tauri::command]
fn open_printer_properties(
    window: tauri::WebviewWindow,
    printer: String,
) -> Result<Option<PrinterSettings>, String> {
    #[cfg(not(target_os = "windows"))]
    let _ = &printer;
    #[cfg(target_os = "windows")]
    {
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Gdi::DEVMODEW;
        use windows::Win32::Graphics::Printing::{ClosePrinter, DocumentPropertiesW, OpenPrinterW};

        const DM_IN_PROMPT: u32 = 4;
        const DM_OUT_BUFFER: u32 = 2;
        const IDOK: i32 = 1;

        unsafe {
            let hwnd = window
                .hwnd()
                .map_err(|e| format!("Failed to get HWND: {e}"))?;
            let hwnd = HWND(hwnd.0 as *mut _);

            let wide_name: Vec<u16> = printer.encode_utf16().chain(std::iter::once(0)).collect();

            let mut h_printer = windows::Win32::Graphics::Printing::PRINTER_HANDLE::default();
            OpenPrinterW(PCWSTR(wide_name.as_ptr()), &mut h_printer, None)
                .map_err(|e| format!("OpenPrinterW failed: {e}"))?;

            // Get required DEVMODE buffer size
            let buf_size = DocumentPropertiesW(
                Some(hwnd),
                h_printer,
                PCWSTR(wide_name.as_ptr()),
                None,
                None,
                0,
            );
            if buf_size < 0 {
                let _ = ClosePrinter(h_printer);
                return Err("DocumentPropertiesW failed to get buffer size".into());
            }

            // Allocate buffer and show Printing Preferences dialog
            let mut buffer = vec![0u8; buf_size as usize];
            let devmode_ptr = buffer.as_mut_ptr() as *mut DEVMODEW;

            let result = DocumentPropertiesW(
                Some(hwnd),
                h_printer,
                PCWSTR(wide_name.as_ptr()),
                Some(devmode_ptr),
                None,
                DM_IN_PROMPT | DM_OUT_BUFFER,
            );

            let _ = ClosePrinter(h_printer);

            if result != IDOK {
                return Ok(None); // User cancelled
            }

            // Parse DEVMODE for paper size and orientation
            let devmode = &*devmode_ptr;
            let landscape = devmode.Anonymous1.Anonymous1.dmOrientation == 2;

            // dmPaperWidth/dmPaperLength are in tenths of mm (portrait dimensions)
            let (w_tenths, h_tenths) = (
                devmode.Anonymous1.Anonymous1.dmPaperWidth as f64,
                devmode.Anonymous1.Anonymous1.dmPaperLength as f64,
            );

            let (paper_w, paper_h) = if w_tenths > 0.0 && h_tenths > 0.0 {
                (w_tenths / 10.0, h_tenths / 10.0)
            } else {
                // Fallback: look up from dmPaperSize
                paper_size_from_id(devmode.Anonymous1.Anonymous1.dmPaperSize)
            };

            Ok(Some(PrinterSettings {
                paper_width_mm: if landscape { paper_h } else { paper_w },
                paper_height_mm: if landscape { paper_w } else { paper_h },
                landscape,
            }))
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = window;
        use std::process::Command;
        let _ = Command::new("open")
            .args([
                "-a",
                "System Preferences",
                "/System/Library/PreferencePanes/PrintAndScan.prefPane",
            ])
            .output();
        Ok(None)
    }
    #[cfg(target_os = "linux")]
    {
        let _ = window;
        use std::process::Command;
        let result = Command::new("system-config-printer").output();
        if result.is_err() {
            let _ = Command::new("gnome-control-center")
                .arg("printers")
                .output();
        }
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn paper_size_from_id(id: i16) -> (f64, f64) {
    match id {
        1 => (215.9, 279.4),    // Letter
        3 => (279.4, 431.8),    // Tabloid
        5 => (215.9, 355.6),    // Legal
        6 => (139.7, 215.9),    // Statement
        7 => (184.15, 266.7),   // Executive
        8 => (297.0, 420.0),    // A3
        9 => (210.0, 297.0),    // A4
        10 => (210.0, 297.0),   // A4 Small
        11 => (148.0, 210.0),   // A5
        12 => (250.0, 353.0),   // B4 (JIS)
        13 => (182.0, 257.0),   // B5 (JIS)
        14 => (215.9, 330.2),   // Folio
        24 => (1000.0, 1414.0), // ISO C (placeholder for large)
        25 => (279.4, 431.8),   // Tabloid Extra
        28 => (162.0, 229.0),   // C5 Envelope
        29 => (324.0, 458.0),   // C3 Envelope
        30 => (229.0, 324.0),   // C4 Envelope
        34 => (176.0, 250.0),   // B5 Envelope
        37 => (98.4, 190.5),    // Monarch Envelope
        38 => (98.4, 225.4),    // 6 3/4 Envelope
        20 => (104.8, 241.3),   // #10 Envelope
        27 => (110.0, 220.0),   // DL Envelope
        66 => (841.0, 1189.0),  // A0
        67 => (594.0, 841.0),   // A1
        68 => (420.0, 594.0),   // A2
        70 => (105.0, 148.0),   // A6
        _ => (210.0, 297.0),    // Default to A4
    }
}

#[tauri::command]
fn play_system_beep() {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _ = Command::new("powershell")
            .args(["-c", "[System.Media.SystemSounds]::Exclamation.Play()"])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("afplay")
            .args(["/System/Library/Sounds/Funk.aiff"])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let _ = Command::new("paplay")
            .args(["/usr/share/sounds/freedesktop/stereo/dialog-warning.oga"])
            .spawn();
    }
}

fn recent_directory(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|failure| project_files::error("app_data_unavailable", failure))
}

fn in_dialog_scope(app: &tauri::AppHandle, path: &std::path::Path) -> bool {
    app.try_fs_scope()
        .is_some_and(|scope| scope.is_allowed(path))
}

/// Saves a project file atomically (temporary file, flush, rename, `.bak`).
/// Only a `.json` file picked in a dialog this session or on the recent list
/// is written; the path is remembered for later sessions.
#[tauri::command]
async fn save_project(app: tauri::AppHandle, path: String, text: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(path);
        project_files::check_project_path(&path)?;
        let directory = recent_directory(&app)?;
        let mut recent = project_files::RecentFiles::load(&directory);
        project_files::allowed(&path, in_dialog_scope(&app, &path), &recent)?;
        project_files::atomic_write(&path, &text)
            .map_err(|failure| project_files::error("project_save_failed", failure))?;
        recent.remember(&path);
        let _ = recent.store(&directory);
        Ok(())
    })
    .await
    .map_err(|failure| project_files::error("project_save_failed", failure))?
}

/// Reads a project file picked in a dialog this session or on the recent
/// list (also after a restart). A missing file gives
/// `project_file_not_found`; only then should the app forget the entry.
#[tauri::command]
async fn read_project_file(app: tauri::AppHandle, path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::PathBuf::from(path);
        let directory = recent_directory(&app)?;
        let mut recent = project_files::RecentFiles::load(&directory);
        project_files::allowed(&path, in_dialog_scope(&app, &path), &recent)?;
        let text = project_files::read_project(&path)?;
        recent.remember(&path);
        let _ = recent.store(&directory);
        Ok(text)
    })
    .await
    .map_err(|failure| project_files::error("project_file_unreadable", failure))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            play_system_beep,
            save_project,
            read_project_file,
            list_printers,
            open_printer_properties,
            validate_nta_project,
            diagnose_declared_heating_table,
            diagnose_forfait_heat_pump_draft,
            diagnose_gas_heat_pump_forfait_draft,
            diagnose_gas_heat_pump_aux_draft,
            diagnose_gas_heat_pump_monthly_draft,
            diagnose_gas_heat_pump_chain_draft,
            diagnose_gas_collective_source_draft,
            compare_gas_heat_pump_chain_diagnostic,
            diagnose_forfait_heat_pump_monthly_draft,
            diagnose_generator_dispatch_draft,
            diagnose_hybrid_heat_pump_monthly_draft,
            diagnose_boiler_forfait_draft,
            diagnose_boiler_forfait_monthly_draft,
            diagnose_declared_dhw,
            diagnose_final_energy_draft,
            diagnose_epus_draft,
            calculate_monthly_demand,
            calculate_space_heating_chain,
            calculate_ventilation,
            assess_residential_survey,
            assess_utility_survey,
            assess_relabel,
            assess_maatwerkadvies,
            calculate_constructions,
            calculate_building_performance,
            calculate_project_performance,
            compare_project_editions,
            kernel_interpretations,
            diagnose_bacs_draft,
            diagnose_indicators_draft,
            diagnose_heating_aux_draft,
            diagnose_heating_aux_measured_draft
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Set window icon for taskbar
            if let Some(window) = app.get_webview_window("main") {
                let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
                    .expect("failed to load icon");
                window.set_icon(icon).expect("failed to set window icon");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(json: &str) -> Value {
        serde_json::from_str(json).unwrap()
    }

    const CHAIN: &str =
        include_str!("../../training-data/nta8800-space-heating-chain-synthetic.json");
    const DWELLING: &str =
        include_str!("../../training-data/nta8800-example-terraced-dwelling.json");

    #[test]
    fn diagnostic_runs_in_the_requested_edition() {
        let input = || serde_json::from_str(CHAIN).unwrap();
        let current = calculate_space_heating_chain_blocking(input(), None).unwrap();
        assert_eq!(current["normVersion"], "2025+C1");
        assert_eq!(current["status"], "calculated_unverified");
        let legacy = calculate_space_heating_chain_blocking(input(), Some(json!("2024"))).unwrap();
        assert_eq!(legacy["normVersion"], "2024");
        assert_eq!(legacy["targetNormVersion"], "NTA 8800:2024 met INT-V1:2024");
        assert_eq!(legacy["status"], "calculated_legacy_edition");
        let error =
            calculate_space_heating_chain_blocking(input(), Some(json!("2019"))).unwrap_err();
        assert!(error.starts_with("invalid_norm_version: "), "{error}");
    }

    #[test]
    fn project_gets_the_requested_edition_without_contradicting_it() {
        let result =
            calculate_project_performance_blocking(fixture(DWELLING), Some(json!("2022"))).unwrap();
        assert_eq!(result["normVersion"], "2022");
        assert_eq!(result["status"], "calculated_legacy_edition");
        assert_eq!(result["registrationEligible"], false);
        let current = calculate_project_performance_blocking(fixture(DWELLING), None).unwrap();
        assert_eq!(current["normVersion"], "2025+C1");
        let mut project = fixture(DWELLING);
        project["ntaCalculation"]["normVersion"] = json!("2023");
        let error =
            calculate_project_performance_blocking(project, Some(json!("2024"))).unwrap_err();
        assert!(error.starts_with("norm_version_conflict: "), "{error}");
    }

    #[test]
    fn edition_comparison_keeps_the_project_and_targets_the_request() {
        let report =
            compare_project_editions_blocking(fixture(DWELLING), Some(json!("2022"))).unwrap();
        assert_eq!(report["fromNormVersion"], "2025+C1");
        assert_eq!(report["toNormVersion"], "2022");
        assert_eq!(
            report["adjustedProject"]["ntaCalculation"]["normVersion"],
            "2022"
        );
        let error =
            compare_project_editions_blocking(fixture(DWELLING), Some(json!("2019"))).unwrap_err();
        assert!(error.starts_with("invalid_norm_version: "), "{error}");
    }

    #[test]
    fn validation_follows_the_project_edition() {
        let result = validate_nta_project_blocking(fixture(DWELLING), Some(json!("2023"))).unwrap();
        assert_eq!(result["normVersion"], "2023");
        let error =
            validate_nta_project_blocking(fixture(DWELLING), Some(json!(2023))).unwrap_err();
        assert!(error.starts_with("invalid_norm_version: "), "{error}");
    }

    #[test]
    fn a_panicking_command_becomes_an_error() {
        let result = tauri::async_runtime::block_on(off_main(|| panic!("boom")));
        assert_eq!(
            result.unwrap_err(),
            "kernel_panic: the calculation stopped unexpectedly"
        );
        // The thread-local edition is back to the default afterwards.
        let edition = tauri::async_runtime::block_on(off_main(|| {
            Ok(serde_json::json!(norm_versions::current().label()))
        }))
        .unwrap();
        assert_eq!(edition, serde_json::json!(NormVersion::default().label()));
    }

    #[test]
    fn commands_run_off_the_main_thread_with_the_edition() {
        let input = serde_json::from_str(CHAIN).unwrap();
        let legacy = tauri::async_runtime::block_on(calculate_space_heating_chain(
            input,
            Some(json!("2024")),
        ))
        .unwrap();
        assert_eq!(legacy["normVersion"], "2024");
    }
}
