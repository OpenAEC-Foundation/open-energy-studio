use tauri::image::Image;
use tauri::Manager;

#[tauri::command]
fn validate_nta_project(
    project: serde_json::Value,
) -> Result<nta8800_core::InputAssessment, String> {
    nta8800_core::assess_json(project)
}

#[tauri::command]
fn diagnose_declared_heating_table(
    input: nta8800_core::declared_heating_table::DeclaredHeatingTableInput,
) -> nta8800_core::declared_heating_table::DeclaredHeatingTableAssessment {
    nta8800_core::declared_heating_table::assess_declared_heating_table(&input)
}

#[tauri::command]
fn diagnose_forfait_heat_pump_draft(
    input: nta8800_core::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput,
) -> nta8800_core::forfait_heat_pump_draft::ForfaitHeatPumpDraftAssessment {
    nta8800_core::forfait_heat_pump_draft::assess_forfait_heat_pump_draft(&input)
}

#[tauri::command]
fn diagnose_gas_heat_pump_forfait_draft(
    input: nta8800_core::gas_heat_pump_forfait_draft::GasHeatPumpForfaitDraftInput,
) -> nta8800_core::gas_heat_pump_forfait_draft::GasHeatPumpForfaitDraftAssessment {
    nta8800_core::gas_heat_pump_forfait_draft::assess_gas_heat_pump_forfait_draft(&input)
}

#[tauri::command]
fn diagnose_gas_heat_pump_aux_draft(
    input: nta8800_core::gas_heat_pump_aux_draft::GasHeatPumpAuxDraftInput,
) -> nta8800_core::gas_heat_pump_aux_draft::GasHeatPumpAuxDraftAssessment {
    nta8800_core::gas_heat_pump_aux_draft::assess_gas_heat_pump_aux_draft(&input)
}

#[tauri::command]
fn diagnose_gas_heat_pump_monthly_draft(
    input: nta8800_core::gas_heat_pump_monthly_draft::GasHeatPumpMonthlyDraftInput,
) -> nta8800_core::gas_heat_pump_monthly_draft::GasHeatPumpMonthlyDraftAssessment {
    nta8800_core::gas_heat_pump_monthly_draft::assess_gas_heat_pump_monthly_draft(&input)
}

#[tauri::command]
fn diagnose_gas_heat_pump_chain_draft(
    input: nta8800_core::gas_heat_pump_chain_draft::GasHeatPumpChainDraftInput,
) -> nta8800_core::gas_heat_pump_chain_draft::GasHeatPumpChainDraftAssessment {
    nta8800_core::gas_heat_pump_chain_draft::assess_gas_heat_pump_chain_draft(&input)
}

#[tauri::command]
fn diagnose_gas_collective_source_draft(
    input: nta8800_core::gas_collective_source_draft::GasCollectiveSourceDraftInput,
) -> nta8800_core::gas_collective_source_draft::GasCollectiveSourceDraftAssessment {
    nta8800_core::gas_collective_source_draft::assess_gas_collective_source_draft(&input)
}

#[tauri::command]
fn compare_gas_heat_pump_chain_diagnostic(
    case: nta8800_core::gas_heat_pump_chain_reference::GasChainDiagnosticCase,
) -> nta8800_core::gas_heat_pump_chain_reference::GasChainDiagnosticComparison {
    nta8800_core::gas_heat_pump_chain_reference::compare_gas_heat_pump_chain_diagnostic(case)
}

#[tauri::command]
fn diagnose_forfait_heat_pump_monthly_draft(
    input: nta8800_core::forfait_heat_pump_monthly_draft::ForfaitHeatPumpMonthlyDraftInput,
) -> nta8800_core::forfait_heat_pump_monthly_draft::ForfaitHeatPumpMonthlyDraftAssessment {
    nta8800_core::forfait_heat_pump_monthly_draft::assess_forfait_heat_pump_monthly_draft(&input)
}

#[tauri::command]
fn diagnose_generator_dispatch_draft(
    input: nta8800_core::generator_dispatch_draft::GeneratorDispatchDraftInput,
) -> nta8800_core::generator_dispatch_draft::GeneratorDispatchDraftAssessment {
    nta8800_core::generator_dispatch_draft::assess_generator_dispatch_draft(&input)
}

#[tauri::command]
fn diagnose_hybrid_heat_pump_monthly_draft(
    input: nta8800_core::hybrid_heat_pump_monthly_draft::HybridHeatPumpMonthlyDraftInput,
) -> nta8800_core::hybrid_heat_pump_monthly_draft::HybridHeatPumpMonthlyDraftAssessment {
    nta8800_core::hybrid_heat_pump_monthly_draft::assess_hybrid_heat_pump_monthly_draft(&input)
}

#[tauri::command]
fn diagnose_boiler_forfait_draft(
    input: nta8800_core::boiler_forfait_draft::BoilerForfaitDraftInput,
) -> nta8800_core::boiler_forfait_draft::BoilerForfaitDraftAssessment {
    nta8800_core::boiler_forfait_draft::assess_boiler_forfait_draft(&input)
}

#[tauri::command]
fn diagnose_boiler_forfait_monthly_draft(
    input: nta8800_core::boiler_forfait_draft::BoilerForfaitMonthlyDraftInput,
) -> nta8800_core::boiler_forfait_draft::BoilerForfaitMonthlyDraftAssessment {
    nta8800_core::boiler_forfait_draft::assess_boiler_forfait_monthly_draft(&input)
}

#[tauri::command]
fn diagnose_declared_dhw(
    input: nta8800_core::heat_pumps::HeatPumpInput,
) -> nta8800_core::declared_dhw::DeclaredDhwAssessment {
    nta8800_core::declared_dhw::assess_declared_dhw(&input)
}

#[tauri::command]
fn diagnose_final_energy_draft(
    input: nta8800_core::final_energy_draft::FinalEnergyDraftInput,
) -> nta8800_core::final_energy_draft::FinalEnergyDraftAssessment {
    nta8800_core::final_energy_draft::assess_final_energy_draft(&input)
}

#[tauri::command]
fn calculate_monthly_demand(
    input: nta8800_core::monthly_demand::MonthlyDemandInput,
) -> nta8800_core::monthly_demand::MonthlyDemandAssessment {
    nta8800_core::monthly_demand::assess_monthly_demand(&input)
}

#[tauri::command]
fn calculate_space_heating_chain(
    input: nta8800_core::space_heating_chain::SpaceHeatingChainInput,
) -> nta8800_core::space_heating_chain::SpaceHeatingChainAssessment {
    nta8800_core::space_heating_chain::assess_space_heating_chain(&input)
}

#[tauri::command]
fn calculate_building_performance(
    input: nta8800_core::building_performance::BuildingPerformanceInput,
) -> nta8800_core::building_performance::BuildingPerformanceAssessment {
    nta8800_core::building_performance::assess_building_performance(&input)
}

#[tauri::command]
fn calculate_project_performance(
    project: serde_json::Value,
) -> nta8800_core::project_performance::ProjectPerformanceAssessment {
    nta8800_core::project_performance::assess_project_performance(&project)
}

#[tauri::command]
fn diagnose_epus_draft(
    input: nta8800_core::epus_draft::EpusDraftInput,
) -> nta8800_core::epus_draft::EpusDraftAssessment {
    nta8800_core::epus_draft::assess_epus_draft(&input)
}

#[tauri::command]
fn diagnose_bacs_draft(
    input: nta8800_core::bacs_draft::BacsDraftInput,
) -> nta8800_core::bacs_draft::BacsDraftAssessment {
    nta8800_core::bacs_draft::assess_bacs_draft(&input)
}

#[tauri::command]
fn diagnose_indicators_draft(
    input: nta8800_core::indicators_draft::IndicatorsDraftInput,
) -> nta8800_core::indicators_draft::IndicatorsDraftAssessment {
    nta8800_core::indicators_draft::assess_indicators_draft(&input)
}

#[tauri::command]
fn diagnose_heating_aux_draft(
    input: nta8800_core::heating_aux_draft::HeatingAuxDraftInput,
) -> nta8800_core::heating_aux_draft::HeatingAuxDraftAssessment {
    nta8800_core::heating_aux_draft::assess_heating_aux_draft(&input)
}

#[tauri::command]
fn diagnose_heating_aux_measured_draft(
    input: nta8800_core::heating_aux_draft::HeatingAuxMeasuredDraftInput,
) -> nta8800_core::heating_aux_draft::HeatingAuxMeasuredDraftAssessment {
    nta8800_core::heating_aux_draft::assess_heating_aux_measured_draft(&input)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            play_system_beep,
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
            calculate_building_performance,
            calculate_project_performance,
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
