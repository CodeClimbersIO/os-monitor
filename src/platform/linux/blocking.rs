//! App and website blocking implementation for Linux
//!
//! Mirrors the macOS blocking.rs structure: stores blocking state, provides
//! system/browser exceptions, and handles app termination with event callbacks.

use crate::blocking_state::{set_blocking_state, clear_blocking_state, BLOCKING_STATE};
use crate::{BlockableItem, BlockedApp, BlockedAppEvent};
use super::monitoring::MONITOR;
use sysinfo::{System, Signal};

/// Check if any blocked apps contain websites (for browser exception logic)
fn has_website_url(blocked_apps: &Vec<BlockableItem>) -> bool {
    blocked_apps.iter().any(|app| app.is_browser)
}

/// System apps that should never be blocked (Linux equivalents of macOS exceptions)
fn get_system_exceptions() -> Vec<BlockableItem> {
    vec![
        // File managers
        BlockableItem::new("nautilus".to_string(), false),
        BlockableItem::new("dolphin".to_string(), false),
        BlockableItem::new("thunar".to_string(), false),
        BlockableItem::new("nemo".to_string(), false),
        BlockableItem::new("pcmanfm".to_string(), false),
        // Terminals
        BlockableItem::new("gnome-terminal".to_string(), false),
        BlockableItem::new("konsole".to_string(), false),
        BlockableItem::new("xterm".to_string(), false),
        BlockableItem::new("alacritty".to_string(), false),
        BlockableItem::new("kitty".to_string(), false),
        BlockableItem::new("foot".to_string(), false),
        BlockableItem::new("wezterm".to_string(), false),
        // System utilities
        BlockableItem::new("gnome-control-center".to_string(), false),
        BlockableItem::new("systemsettings".to_string(), false),
        BlockableItem::new("gnome-system-monitor".to_string(), false),
        BlockableItem::new("ksysguard".to_string(), false),
        // Ebb itself
        BlockableItem::new("ebb".to_string(), false),
        BlockableItem::new("ebb.cool".to_string(), true),
    ]
}

/// Browsers that should be allowed when website blocking is active
/// (we block websites via extension, not by killing the browser)
fn get_browser_exceptions() -> Vec<BlockableItem> {
    vec![
        BlockableItem::new("google-chrome".to_string(), false),
        BlockableItem::new("chromium".to_string(), false),
        BlockableItem::new("firefox".to_string(), false),
        BlockableItem::new("brave".to_string(), false),
        BlockableItem::new("vivaldi".to_string(), false),
        BlockableItem::new("opera".to_string(), false),
        BlockableItem::new("microsoft-edge".to_string(), false),
        BlockableItem::new("epiphany".to_string(), false),
    ]
}

/// Get exceptions based on blocking mode and whether websites are being blocked
fn get_exceptions(has_website_url: bool, blocklist_mode: bool) -> Vec<BlockableItem> {
    let mut exceptions = Vec::new();
    if blocklist_mode {
        return exceptions;
    }
    if has_website_url {
        exceptions.extend(get_browser_exceptions());
    }
    exceptions.extend(get_system_exceptions());
    exceptions
}

/// Check if an app should be blocked based on current blocking state
pub fn is_blocked(app_name: &str) -> bool {
    let state = match BLOCKING_STATE.lock() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if !state.active || state.blocked_apps.is_empty() {
        return false;
    }

    let app_lower = app_name.to_lowercase();

    // Case-insensitive exact match (like macOS bundle ID matching)
    let app_in_list = state.blocked_apps.iter().any(|blocked| {
        blocked.to_lowercase() == app_lower
    });

    if state.blocklist_mode {
        app_in_list // Block if in blocklist
    } else {
        !app_in_list // Block if NOT in allowlist
    }
}

/// Terminate a blocked app by name and send BlockedAppEvent
/// Returns true if the app was found and terminated
pub fn close_app(app_name: &str) -> bool {
    let mut system = System::new_all();
    system.refresh_processes();

    let app_lower = app_name.to_lowercase();
    let mut killed = false;

    for (pid, process) in system.processes() {
        let process_name = process.name().to_lowercase();

        // Exact match on process name (like macOS bundle ID matching)
        if process_name == app_lower {
            log::info!("Terminating blocked application: {} (PID {})", process.name(), pid);

            if process.kill_with(Signal::Term).unwrap_or(false) {
                killed = true;

                // Send BlockedAppEvent (like macOS callback)
                if let Ok(monitor_guard) = MONITOR.lock() {
                    if let Some(monitor) = monitor_guard.as_ref() {
                        monitor.send_app_blocked_event(BlockedAppEvent {
                            blocked_apps: vec![BlockedApp {
                                app_name: process.name().to_string(),
                                app_external_id: process.name().to_string(),
                                is_site: false,
                            }],
                        });
                    }
                }

                log::info!("Successfully terminated blocked application: {}", process.name());
            } else {
                log::warn!("Failed to terminate process {} (PID {})", process.name(), pid);
            }
        }
    }

    killed
}

pub fn platform_start_blocking(
    blocked_apps: &Vec<BlockableItem>,
    redirect_url: &str,
    blocklist_mode: bool,
) -> bool {
    let mut all_items = blocked_apps.to_vec();

    let has_website_url = has_website_url(blocked_apps);
    let exceptions = get_exceptions(has_website_url, blocklist_mode);
    all_items.extend(exceptions);

    let websites: Vec<String> = all_items
        .iter()
        .filter(|app| app.is_browser)
        .map(|app| app.app_external_id.clone())
        .collect();

    let apps: Vec<String> = all_items
        .iter()
        .filter(|app| !app.is_browser)
        .map(|app| app.app_external_id.clone())
        .collect();

    // Store blocking config in shared state for other modules to check against
    set_blocking_state(
        websites.clone(),
        apps.clone(),
        redirect_url.to_string(),
        blocklist_mode,
    );

    log::info!("Linux blocking started:");
    log::info!("  Websites: {:?}", websites);
    log::info!("  Apps: {:?}", apps);
    log::info!("  Redirect URL: {}", redirect_url);
    log::info!("  Mode: {}", if blocklist_mode { "blocklist" } else { "allowlist" });

    // Website blocking happens reactively in browser/mod.rs when URLs are reported
    // via WebSocket from the browser extension. handle_url_change() checks against
    // the blocking state.

    // App blocking happens in monitoring.rs when focus changes - it calls
    // is_blocked() and close_app() from this module.

    true
}

pub fn platform_stop_blocking() {
    log::info!("Linux blocking stopped");
    clear_blocking_state();
}

pub fn platform_get_application_icon_data(_bundle_id: &str) -> Option<String> {
    // TODO: Extract application icons from .desktop files or app directories
    // Return base64 encoded icon data
    log::warn!("platform_get_application_icon_data not yet implemented for Linux");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_website_url() {
        let blocked_apps = vec![
            BlockableItem::new("com.example.app".to_string(), false),
            BlockableItem::new("google.com".to_string(), true),
        ];
        assert!(has_website_url(&blocked_apps));
    }

    #[test]
    fn test_has_website_url_false() {
        let blocked_apps = vec![
            BlockableItem::new("com.example.app".to_string(), false),
            BlockableItem::new("google.com".to_string(), false),
        ];
        assert!(!has_website_url(&blocked_apps));
    }

    #[test]
    fn test_get_browser_exceptions() {
        let exceptions = get_browser_exceptions();
        let contains_chrome = exceptions
            .iter()
            .any(|app| app.app_external_id == "google-chrome");
        assert!(contains_chrome);
    }

    #[test]
    fn test_get_system_exceptions() {
        let exceptions = get_system_exceptions();
        let contains_nautilus = exceptions
            .iter()
            .any(|app| app.app_external_id == "nautilus");
        assert!(contains_nautilus);
    }

    #[test]
    fn test_get_exceptions_allowlist_mode_with_browser() {
        let exceptions = get_exceptions(true, false);
        let contains_chrome = exceptions
            .iter()
            .any(|app| app.app_external_id == "google-chrome");
        let contains_nautilus = exceptions
            .iter()
            .any(|app| app.app_external_id == "nautilus");
        assert!(contains_chrome);
        assert!(contains_nautilus);
    }

    #[test]
    fn test_get_exceptions_blocklist_mode() {
        let exceptions = get_exceptions(true, true);
        assert!(exceptions.is_empty());
    }
}
