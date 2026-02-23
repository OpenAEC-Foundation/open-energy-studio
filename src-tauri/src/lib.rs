use tauri::image::Image;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
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
