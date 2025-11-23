use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "linux")]
pub fn get_asset_path() -> PathBuf {
    PathBuf::from("/usr/share/vitray-widget")
}

#[cfg(target_os = "windows")]
pub fn get_asset_path() -> PathBuf {
    // For now, assume assets are relative to the executable or in a specific folder
    // This might need adjustment based on installation method
    std::env::current_exe()
        .map(|p| p.parent().unwrap_or(&p).join("assets"))
        .unwrap_or_else(|_| PathBuf::from("assets"))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn get_asset_path() -> PathBuf {
    PathBuf::from("assets")
}

#[cfg(target_os = "linux")]
pub fn get_doc_path() -> String {
    "/usr/share/doc/vitray-widget/".to_string()
}

#[cfg(target_os = "windows")]
pub fn get_doc_path() -> String {
    // Placeholder or online URL
    "https://zacxxx.github.io/vitray-widget".to_string()
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn get_doc_path() -> String {
    "https://zacxxx.github.io/vitray-widget".to_string()
}

#[cfg(target_os = "linux")]
pub fn get_default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string())
}

#[cfg(target_os = "windows")]
pub fn get_default_shell() -> String {
    "powershell.exe".to_string()
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn get_default_shell() -> String {
    "bash".to_string()
}

pub fn open_external_terminal(command: Option<&str>) {
    #[cfg(target_os = "windows")]
    {
        // Try to launch Warp if requested or configured
        // For now, just launch PowerShell or CMD
        // If 'command' is "warp", try to launch warp
        
        let shell = if let Some(cmd) = command {
            if cmd.eq_ignore_ascii_case("warp") {
                "warp.exe" // Assuming warp is in PATH
            } else {
                "powershell.exe"
            }
        } else {
            "powershell.exe"
        };

        let _ = Command::new("cmd")
            .args(["/C", "start", shell])
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, we might want to launch an external terminal too
        if let Some(cmd) = command {
             let _ = Command::new(cmd).spawn();
        }
    }
}
