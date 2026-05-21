//! Miscellaneous commands not large enough to warrant their own module.

/// Play the OS "alert" sound. Non-blocking.
#[tauri::command]
pub fn play_system_beep() {
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
