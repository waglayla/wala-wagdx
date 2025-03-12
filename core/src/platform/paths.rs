use std::path::PathBuf;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::env;
use workflow_core::dirs;

/// Get appropriate log file path based on platform
pub fn get_log_file_path(service_name: &str) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if is_app_bundle() {
            // If running as a bundle on macOS, use Library/Logs
            if let Some(home) = dirs::home_dir() {
                let log_dir = home.join("Library/Logs/com.example.wala-wagdx");
                // Try to create the directory
                let _ = fs::create_dir_all(&log_dir);
                return log_dir.join(format!("{}.log", service_name));
            }
        }
    }

    // Default for all other platforms: relative to executable
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("logs").join(format!("{}.log", service_name));
        }
    }

    // Fallback if exe_path fails
    PathBuf::from(format!("{}.log", service_name))
}

/// Check if running as a macOS app bundle
#[cfg(target_os = "macos")]
pub fn is_app_bundle() -> bool {
    if let Ok(exe_path) = env::current_exe() {
        exe_path.to_string_lossy().contains(".app/Contents/MacOS")
    } else {
        false
    }
}

#[cfg(not(target_os = "macos"))]
pub fn is_app_bundle() -> bool {
    false
}

/// Open a log file with proper error handling
pub fn open_log_file(service_name: &str) -> io::Result<File> {
    let path = get_log_file_path(service_name);
    
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&path)
}