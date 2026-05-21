//! JSON Schema exposure for type generation.
//!
//! The frontend's TypeScript types are produced from these schemas via
//! `npm run generate-types`. Keeping schema derivation in Rust means the
//! single source of truth for input/output shapes is the crate definitions.

use crate::error::AppResult;

/// JSON Schema for the calculation input (currently `isso51_core::Project`).
///
/// Phase B will switch this to `openaec_project_shared::ProjectV2` once the
/// full BENG orchestrator is in place — the frontend already drives ProjectV2
/// shape, but the calculation pipeline converts to the isso51 shape.
#[tauri::command]
pub fn project_schema() -> AppResult<String> {
    Ok(isso51_core::project_schema())
}

/// JSON Schema for the calculation result (`isso51_core::ProjectResult`).
#[tauri::command]
pub fn result_schema() -> AppResult<String> {
    Ok(isso51_core::result_schema())
}
