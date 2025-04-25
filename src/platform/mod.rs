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

pub fn start_monitoring(monitor: Arc<Monitor>) {
    platform_start_monitoring(monitor);
}

pub fn start_blocking(
    blocked_apps: &[BlockableItem],
    redirect_url: &str,
    blocklist_mode: bool,
) -> bool {
    platform_start_blocking(&mut blocked_apps.to_vec(), redirect_url, blocklist_mode)
}

pub fn stop_blocking() {
    platform_stop_blocking()
}

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

pub fn run_loop_cycle() {
    platform_run_loop_cycle()
}

pub fn create_typewriter_window(opacity: f64) {
    platform_create_typewriter_window(opacity)
}

pub fn sync_typewriter_window_order() {
    platform_sync_typewriter_window_order();
}

pub fn remove_typewriter_window() {
    platform_remove_typewriter_window();
}
