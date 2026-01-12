//! Hyprland focus detection backend
//!
//! Uses hyprctl CLI to get the active window information.

use super::{FocusBackend, FocusedWindow};
use std::process::Command;

pub struct HyprlandFocusBackend {
    // No state needed - we call hyprctl each time
}

impl HyprlandFocusBackend {
    pub fn new() -> Option<Self> {
        // Verify hyprctl is available
        match Command::new("hyprctl").arg("version").output() {
            Ok(output) if output.status.success() => {
                log::info!("Hyprland backend initialized");
                Some(Self {})
            }
            _ => {
                log::error!("hyprctl not available");
                None
            }
        }
    }
}

impl FocusBackend for HyprlandFocusBackend {
    fn get_focused_window(&self) -> Option<FocusedWindow> {
        // Run: hyprctl activewindow -j
        let output = Command::new("hyprctl")
            .args(["activewindow", "-j"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8_lossy(&output.stdout);

        // Parse JSON response
        // Format: {"address":"0x...","class":"firefox","title":"...","pid":1234,...}
        let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        let address = json.get("address")?.as_str()?;
        let class = json.get("class")?.as_str().unwrap_or("Unknown");
        let title = json.get("title")?.as_str().unwrap_or("");

        // Convert address (hex string like "0x5678abcd") to u64
        let id = u64::from_str_radix(address.trim_start_matches("0x"), 16).unwrap_or(0);

        Some(FocusedWindow {
            id,
            app_name: class.to_string(),
            title: title.to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "Hyprland"
    }
}
