//! Tauri command modules.
//!
//! Each submodule groups a related set of `#[tauri::command]` functions. All
//! commands return `AppResult<T>` (or pure `T` where infallible) so the
//! frontend sees a uniform error shape.

pub mod calculate;
pub mod misc;
pub mod printers;
pub mod project;
pub mod schema;
