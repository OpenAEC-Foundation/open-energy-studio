//! Calculation commands.
//!
//! Phase A scope: a single `calculate` command that converts a `ProjectV2`
//! (the user-facing schema) into an `isso51_core::Project` and runs the heat
//! loss engine. Phase B will add NTA 8800 / BENG orchestration across the
//! `nta8800-*` crates.

use isso51_core::result::ProjectResult;
use openaec_project_shared::{view, ProjectV2};

use crate::error::{AppError, AppResult};

/// Run the heat loss calculation for a `ProjectV2`.
///
/// Returns the `isso51_core::result::ProjectResult` directly. Phase B will
/// replace this with a richer `OesProjectResult` that aggregates BENG
/// indicators across the full NTA 8800 chain.
#[tauri::command]
pub fn calculate(project: ProjectV2) -> AppResult<ProjectResult> {
    let isso = view::to_isso51_project(&project)
        .map_err(|e| AppError::View(e.to_string()))?;
    isso51_core::calculate(&isso).map_err(|e| AppError::Calc(e.to_string()))
}
