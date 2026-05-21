//! Printer enumeration and properties dialog.
//!
//! Cross-platform: Windows uses CIM + the native Printing Preferences dialog
//! via `windows-rs`; macOS / Linux use `lpstat` and best-effort opening of
//! the system print settings UI.
//!
//! This module preserves the printer commands the previous monolithic
//! `lib.rs` exposed, with no behavioural changes.

use serde::Serialize;

/// One installed printer, as reported by the OS.
#[derive(Serialize)]
pub struct PrinterInfo {
    /// Display name.
    pub name: String,
    /// `true` if the OS marks this printer as the default.
    pub is_default: bool,
}

/// Paper dimensions and orientation returned by the Windows
/// Printing Preferences dialog.
#[derive(Serialize)]
pub struct PrinterSettings {
    /// Paper width in millimetres (rotated to landscape if `landscape` is true).
    pub paper_width_mm: f64,
    /// Paper height in millimetres (rotated to landscape if `landscape` is true).
    pub paper_height_mm: f64,
    /// `true` if the user selected landscape orientation.
    pub landscape: bool,
}

/// List all printers known to the OS.
#[tauri::command]
pub fn list_printers() -> Vec<PrinterInfo> {
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
        unix_list_printers()
    }
    #[cfg(target_os = "linux")]
    {
        unix_list_printers()
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn unix_list_printers() -> Vec<PrinterInfo> {
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
            if let Some(rest) = line.strip_prefix("printer ") {
                if let Some(name) = rest.split_whitespace().next() {
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

/// Open the Printing Preferences dialog for the named printer and return the
/// chosen paper size + orientation, or `None` if the user cancelled.
#[tauri::command]
pub fn open_printer_properties(
    window: tauri::WebviewWindow,
    printer: String,
) -> Result<Option<PrinterSettings>, String> {
    #[cfg(target_os = "windows")]
    {
        win::open_printer_properties_impl(window, printer)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = window;
        let _ = printer;
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
        let _ = printer;
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
mod win {
    use super::PrinterSettings;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::DEVMODEW;
    use windows::Win32::Graphics::Printing::{ClosePrinter, DocumentPropertiesW, OpenPrinterW};

    const DM_IN_PROMPT: u32 = 4;
    const DM_OUT_BUFFER: u32 = 2;
    const IDOK: i32 = 1;

    pub fn open_printer_properties_impl(
        window: tauri::WebviewWindow,
        printer: String,
    ) -> Result<Option<PrinterSettings>, String> {
        unsafe {
            let hwnd = window.hwnd().map_err(|e| format!("Failed to get HWND: {e}"))?;
            let hwnd = HWND(hwnd.0 as *mut _);

            let wide_name: Vec<u16> = printer.encode_utf16().chain(std::iter::once(0)).collect();

            let mut h_printer = windows::Win32::Graphics::Printing::PRINTER_HANDLE::default();
            OpenPrinterW(PCWSTR(wide_name.as_ptr()), &mut h_printer, None)
                .map_err(|e| format!("OpenPrinterW failed: {e}"))?;

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
                return Ok(None);
            }

            let devmode = &*devmode_ptr;
            let landscape = devmode.Anonymous1.Anonymous1.dmOrientation == 2;

            let (w_tenths, h_tenths) = (
                devmode.Anonymous1.Anonymous1.dmPaperWidth as f64,
                devmode.Anonymous1.Anonymous1.dmPaperLength as f64,
            );

            let (paper_w, paper_h) = if w_tenths > 0.0 && h_tenths > 0.0 {
                (w_tenths / 10.0, h_tenths / 10.0)
            } else {
                paper_size_from_id(devmode.Anonymous1.Anonymous1.dmPaperSize)
            };

            Ok(Some(PrinterSettings {
                paper_width_mm: if landscape { paper_h } else { paper_w },
                paper_height_mm: if landscape { paper_w } else { paper_h },
                landscape,
            }))
        }
    }

    fn paper_size_from_id(id: i16) -> (f64, f64) {
        match id {
            1 => (215.9, 279.4),
            3 => (279.4, 431.8),
            5 => (215.9, 355.6),
            6 => (139.7, 215.9),
            7 => (184.15, 266.7),
            8 => (297.0, 420.0),
            9 => (210.0, 297.0),
            10 => (210.0, 297.0),
            11 => (148.0, 210.0),
            12 => (250.0, 353.0),
            13 => (182.0, 257.0),
            14 => (215.9, 330.2),
            24 => (1000.0, 1414.0),
            25 => (279.4, 431.8),
            28 => (162.0, 229.0),
            29 => (324.0, 458.0),
            30 => (229.0, 324.0),
            34 => (176.0, 250.0),
            37 => (98.4, 190.5),
            38 => (98.4, 225.4),
            20 => (104.8, 241.3),
            27 => (110.0, 220.0),
            66 => (841.0, 1189.0),
            67 => (594.0, 841.0),
            68 => (420.0, 594.0),
            70 => (105.0, 148.0),
            _ => (210.0, 297.0),
        }
    }
}
