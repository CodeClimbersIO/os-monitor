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
        // File managers - GNOME
        BlockableItem::new("nautilus".to_string(), false),
        BlockableItem::new("org.gnome.Nautilus".to_string(), false),
        // File managers - KDE
        BlockableItem::new("dolphin".to_string(), false),
        BlockableItem::new("org.kde.dolphin".to_string(), false),
        // File managers - XFCE/other
        BlockableItem::new("thunar".to_string(), false),
        BlockableItem::new("nemo".to_string(), false),
        BlockableItem::new("pcmanfm".to_string(), false),
        BlockableItem::new("pcmanfm-qt".to_string(), false),
        BlockableItem::new("caja".to_string(), false),
        // Terminals - GNOME
        BlockableItem::new("gnome-terminal".to_string(), false),
        BlockableItem::new("org.gnome.Terminal".to_string(), false),
        BlockableItem::new("gnome-console".to_string(), false),
        BlockableItem::new("org.gnome.Console".to_string(), false),
        // Terminals - KDE
        BlockableItem::new("konsole".to_string(), false),
        BlockableItem::new("org.kde.konsole".to_string(), false),
        // Terminals - other
        BlockableItem::new("xterm".to_string(), false),
        BlockableItem::new("alacritty".to_string(), false),
        BlockableItem::new("kitty".to_string(), false),
        BlockableItem::new("foot".to_string(), false),
        BlockableItem::new("wezterm".to_string(), false),
        BlockableItem::new("tilix".to_string(), false),
        BlockableItem::new("terminator".to_string(), false),
        BlockableItem::new("urxvt".to_string(), false),
        BlockableItem::new("st".to_string(), false),
        // System settings - GNOME
        BlockableItem::new("gnome-control-center".to_string(), false),
        BlockableItem::new("org.gnome.Settings".to_string(), false),
        BlockableItem::new("gnome-system-monitor".to_string(), false),
        BlockableItem::new("org.gnome.SystemMonitor".to_string(), false),
        // System settings - KDE
        BlockableItem::new("systemsettings".to_string(), false),
        BlockableItem::new("systemsettings5".to_string(), false),
        BlockableItem::new("org.kde.systemsettings".to_string(), false),
        BlockableItem::new("ksysguard".to_string(), false),
        BlockableItem::new("plasma-systemmonitor".to_string(), false),
        BlockableItem::new("org.kde.plasma-systemmonitor".to_string(), false),
        // System settings - XFCE
        BlockableItem::new("xfce4-settings-manager".to_string(), false),
        BlockableItem::new("xfce4-taskmanager".to_string(), false),
        // Launchers/runners
        BlockableItem::new("rofi".to_string(), false),
        BlockableItem::new("wofi".to_string(), false),
        BlockableItem::new("dmenu".to_string(), false),
        BlockableItem::new("krunner".to_string(), false),
        // Ebb itself
        BlockableItem::new("ebb".to_string(), false),
        BlockableItem::new("ebb.cool".to_string(), true),
    ]
}

/// Browsers that should be allowed when website blocking is active
/// (we block websites via extension, not by killing the browser)
fn get_browser_exceptions() -> Vec<BlockableItem> {
    vec![
        // Chrome variants
        BlockableItem::new("google-chrome".to_string(), false),
        BlockableItem::new("google-chrome-stable".to_string(), false),
        BlockableItem::new("chrome".to_string(), false),
        BlockableItem::new("chromium".to_string(), false),
        BlockableItem::new("chromium-browser".to_string(), false),
        // Firefox
        BlockableItem::new("firefox".to_string(), false),
        BlockableItem::new("firefox-esr".to_string(), false),
        BlockableItem::new("librewolf".to_string(), false),
        // Other Chromium-based
        BlockableItem::new("brave".to_string(), false),
        BlockableItem::new("brave-browser".to_string(), false),
        BlockableItem::new("vivaldi".to_string(), false),
        BlockableItem::new("vivaldi-stable".to_string(), false),
        BlockableItem::new("opera".to_string(), false),
        BlockableItem::new("microsoft-edge".to_string(), false),
        BlockableItem::new("microsoft-edge-stable".to_string(), false),
        // GNOME/GTK browsers
        BlockableItem::new("epiphany".to_string(), false),
        BlockableItem::new("org.gnome.Epiphany".to_string(), false),
        // KDE browser
        BlockableItem::new("falkon".to_string(), false),
        BlockableItem::new("org.kde.falkon".to_string(), false),
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

/// Check if a process name should be blocked based on current blocking state
fn should_block_process(process_name: &str) -> bool {
    let state = match BLOCKING_STATE.lock() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if !state.active || state.blocked_apps.is_empty() {
        return false;
    }

    let name_lower = process_name.to_lowercase();

    // Case-insensitive exact match
    let in_list = state.blocked_apps.iter().any(|blocked| {
        blocked.to_lowercase() == name_lower
    });

    if state.blocklist_mode {
        in_list // Block if in blocklist
    } else {
        !in_list // Block if NOT in allowlist
    }
}

/// Scan all running processes and kill any that are blocked.
/// Called every polling cycle - works universally across all display servers.
pub fn kill_blocked_processes() {
    let state = match BLOCKING_STATE.lock() {
        Ok(s) => s,
        Err(_) => return,
    };

    if !state.active || state.blocked_apps.is_empty() {
        return;
    }

    // Debug: Log blocking state once per cycle
    static LOGGED_STATE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED_STATE.load(std::sync::atomic::Ordering::Relaxed) {
        log::info!("=== Blocking Active ===");
        log::info!("Mode: {}", if state.blocklist_mode { "blocklist" } else { "allowlist" });
        log::info!("Blocked apps: {:?}", state.blocked_apps);
        LOGGED_STATE.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    drop(state); // Release lock before potentially slow process enumeration

    let mut system = System::new_all();
    system.refresh_processes();

    let mut killed_apps: Vec<BlockedApp> = Vec::new();

    // Debug: Log all process names once to see what we're working with
    static LOGGED_PROCESSES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED_PROCESSES.load(std::sync::atomic::Ordering::Relaxed) {
        log::info!("=== All Running Processes ===");
        for (pid, proc) in system.processes() {
            log::info!("PID {}: {}", pid, proc.name());
        }
        log::info!("=== End Process List ===");
        LOGGED_PROCESSES.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    for (pid, process) in system.processes() {
        let process_name = process.name().to_string();

        if should_block_process(&process_name) {
            log::info!("🚫 Terminating blocked application: {} (PID {})", process_name, pid);

            if process.kill_with(Signal::Term).unwrap_or(false) {
                killed_apps.push(BlockedApp {
                    app_name: process_name.clone(),
                    app_external_id: process_name.clone(),
                    is_site: false,
                });
                log::info!("✅ Successfully terminated: {}", process_name);
            } else {
                log::warn!("❌ Failed to terminate: {} (PID {})", process_name, pid);
            }
        }
    }

    // Send BlockedAppEvent for all killed apps (batched like macOS)
    if !killed_apps.is_empty() {
        if let Ok(monitor_guard) = MONITOR.lock() {
            if let Some(monitor) = monitor_guard.as_ref() {
                monitor.send_app_blocked_event(BlockedAppEvent {
                    blocked_apps: killed_apps,
                });
            }
        }
    }
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
