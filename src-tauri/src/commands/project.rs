//! Project file I/O commands.
//!
//! The on-disk format is a JSON envelope:
//!
//! ```json
//! {
//!   "schema_version": "2.0",
//!   "exported_at": "...",
//!   "app_version": "...",
//!   "project": { /* ProjectV2 */ },
//!   "result": { /* ProjectResult | null */ }
//! }
//! ```
//!
//! Files written by the previous TypeScript engine are detected by the
//! absence of `schema_version` and converted via
//! [`openaec_project_shared::migration::from_legacy_v1`].

use std::fs;
use std::path::Path;

use isso51_core::result::ProjectResult;
use openaec_project_shared::{migration, ProjectV2};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// JSON envelope wrapping a project plus its (optional) last computed result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectEnvelope {
    /// Schema marker, always `"2.0"` for files written by the new engine.
    pub schema_version: String,
    /// ISO 8601 timestamp of when the envelope was last written.
    pub exported_at: String,
    /// `Cargo.toml` package version of the writing app.
    pub app_version: String,
    /// The project payload.
    pub project: ProjectV2,
    /// Optional cached calculation result.
    pub result: Option<ProjectResult>,
}

impl ProjectEnvelope {
    fn now_iso() -> String {
        // Avoid pulling in a heavy date crate for one timestamp; rely on the
        // OS-provided clock and format manually.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("@{now}")
    }

    fn wrap(project: ProjectV2, result: Option<ProjectResult>) -> Self {
        Self {
            schema_version: "2.0".to_string(),
            exported_at: Self::now_iso(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            project,
            result,
        }
    }
}

/// Create an empty `ProjectV2` with the given name. Phase B's "New project"
/// action calls this so the default shape lives in Rust alongside the schema.
#[tauri::command]
pub fn new_project(name: String) -> ProjectV2 {
    ProjectV2::new(name)
}

/// Load a project from disk.
///
/// If `schema_version` is missing or starts with `"1"`, the file is treated as
/// a legacy `.oes` and run through the migration. Otherwise the envelope is
/// deserialized directly.
#[tauri::command]
pub fn load_project(path: String) -> AppResult<ProjectV2> {
    let raw = fs::read_to_string(Path::new(&path))?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;

    let schema_v = value
        .get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if schema_v.is_empty() || schema_v.starts_with('1') {
        return migrate_legacy_inner(&raw);
    }

    let env: ProjectEnvelope = serde_json::from_str(&raw)?;
    Ok(env.project)
}

/// Save a project to disk, optionally with a cached result.
#[tauri::command]
pub fn save_project(
    path: String,
    project: ProjectV2,
    result: Option<ProjectResult>,
) -> AppResult<()> {
    let env = ProjectEnvelope::wrap(project, result);
    let json = serde_json::to_string_pretty(&env)?;
    fs::write(Path::new(&path), json)?;
    Ok(())
}

/// Force-migrate a legacy `.oes` file into a `ProjectV2` without writing.
#[tauri::command]
pub fn migrate_legacy(path: String) -> AppResult<ProjectV2> {
    let raw = fs::read_to_string(Path::new(&path))?;
    migrate_legacy_inner(&raw)
}

fn migrate_legacy_inner(raw: &str) -> AppResult<ProjectV2> {
    migration::from_legacy_v1(raw)
        .map_err(|e| AppError::Migration(e.to_string()))
}
