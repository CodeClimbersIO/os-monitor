use crate::blocking_state::{set_blocking_state, clear_blocking_state};
use crate::BlockableItem;

pub fn platform_start_blocking(
    blocked_apps: &mut Vec<BlockableItem>,
    redirect_url: &str,
    blocklist_mode: bool,
) -> bool {
    let websites: Vec<String> = blocked_apps
        .iter()
        .filter(|app| app.is_browser)
        .map(|app| app.app_external_id.clone())
        .collect();

    let apps: Vec<String> = blocked_apps
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
    log::info!("  Redirect URL: {}", redirect_url);
    log::info!("  Mode: {}", if blocklist_mode { "blocklist" } else { "allowlist" });

    // Website blocking happens reactively in browser/mod.rs when URLs are reported
    // via WebSocket from the browser extension. handle_url_change() checks against
    // the blocking state we just set above.

    // TODO: App blocking requires focus detection + process termination
    // Use sysinfo crate to enumerate processes and kill by name/pid
    if !apps.is_empty() {
        log::warn!("App blocking not yet implemented: {:?}", apps);
    }

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