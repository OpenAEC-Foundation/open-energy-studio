//! Tauri shell entry point.
//!
//! The library form (`app_lib`) is invoked from `src/main.rs` (and from the
//! mobile entry point on supported targets). All commands live in submodules
//! under [`commands`]; this file only wires the builder.

mod commands;
mod error;

use tauri::image::Image;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // calculation
            commands::calculate::calculate,
            // project I/O
            commands::project::load_project,
            commands::project::save_project,
            commands::project::migrate_legacy,
            // schemas
            commands::schema::project_schema,
            commands::schema::result_schema,
            // printer surface (preserved from previous monolithic lib.rs)
            commands::printers::list_printers,
            commands::printers::open_printer_properties,
            // misc
            commands::misc::play_system_beep,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Set window icon for the OS taskbar / dock.
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
