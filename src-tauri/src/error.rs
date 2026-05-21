//! Single error type exposed across the Tauri command boundary.
//!
//! Serializes to a stable `{ kind, message }` shape so the frontend can branch
//! on `kind` without parsing prose.

use serde::Serialize;

/// All errors surfaced to the frontend.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Filesystem I/O failure (read, write, missing file, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON (de)serialization failure on project files or command payloads.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Calculation error bubbling up from `isso51_core`.
    #[error("calculation error: {0}")]
    Calc(String),

    /// `ProjectV2 → isso51_core::Project` projection failed.
    #[error("project view error: {0}")]
    View(String),

    /// Legacy `.oes` could not be migrated to `ProjectV2`.
    #[error("legacy migration error: {0}")]
    Migration(String),

    /// Input failed semantic validation (e.g. ratio out of range).
    #[error("validation error: {0}")]
    Validation(String),

    /// Catch-all for anything not yet modelled explicitly.
    #[error("{0}")]
    Other(String),
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            AppError::Io(_) => "io",
            AppError::Serde(_) => "serde",
            AppError::Calc(_) => "calc",
            AppError::View(_) => "view",
            AppError::Migration(_) => "migration",
            AppError::Validation(_) => "validation",
            AppError::Other(_) => "other",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

/// Convenience alias used throughout the commands modules.
pub type AppResult<T> = std::result::Result<T, AppError>;
