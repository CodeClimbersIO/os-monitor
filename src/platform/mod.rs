#[cfg(target_os = "macos")]
mod macos;
use std::sync::Arc;

#[cfg(target_os = "macos")]
pub(crate) use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

use crate::{BlockableItem, Monitor, MonitorError};

pub fn detect_changes() -> Result<(), MonitorError> {
    platform_detect_changes()
}

pub fn has_accessibility_permissions() -> bool {
    platform_has_accessibility_permissions()
}

pub fn request_accessibility_permissions() -> bool {
    platform_request_accessibility_permissions()
}

pub fn get_application_icon_data(bundle_id: &str) -> Option<String> {
    platform_get_application_icon_data(bundle_id)
}

pub fn start_monitoring(monitor: Arc<Monitor>) {
    platform_start_monitoring(monitor);
}

pub fn start_blocking(
    blocked_apps: &[BlockableItem],
    redirect_url: &str,
    blocklist_mode: bool,
) -> bool {
    // test if any of the blocked apps are website urls
    let is_website_url = blocked_apps.iter().any(|app| app.is_browser);
    let mut all_items = blocked_apps.to_vec();

    if !blocklist_mode {
        // if we are in allowlist, add exceptions like system finder, spotify, activity monitor, other mac system apps
        all_items.extend(vec![
            BlockableItem::new("com.apple.SystemFinder".to_string(), false),
            BlockableItem::new("com.spotify.client".to_string(), false),
            BlockableItem::new("com.apple.ActivityMonitor".to_string(), false),
            BlockableItem::new("com.apple.SystemPreferences".to_string(), false),
            BlockableItem::new("com.apple.finder".to_string(), false),
            BlockableItem::new("ebb.cool".to_string(), true),
            BlockableItem::new("com.ebb.app".to_string(), true),
        ]);
    }
    if is_website_url && !blocklist_mode {
        let browser_bundle_ids = vec![
            "com.google.Chrome",
            "com.google.Chrome.beta",
            "com.google.Chrome.dev",
            "com.google.Chrome.canary",
            "com.apple.Safari",
            "com.microsoft.Edge",
            "com.brave.Browser",
            "company.thebrowser.Browser",
        ];
        all_items.extend(
            browser_bundle_ids
                .iter()
                .map(|id| BlockableItem::new(id.to_string(), false)),
        );
    }
    platform_start_blocking(&all_items, redirect_url, blocklist_mode)
}

pub fn stop_blocking() {
    platform_stop_blocking()
}

pub fn request_automation_permission(bundle_id: &str) -> bool {
    platform_request_automation_permission(bundle_id)
}
