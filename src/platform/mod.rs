#[cfg(target_os = "macos")]
mod macos;
use std::sync::Arc;

#[cfg(target_os = "macos")]
pub(crate) use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

use crate::{Monitor, MonitorError};

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

pub fn start_blocking(urls: &[String], redirect_url: &str) -> bool {
    platform_start_blocking(urls, redirect_url)
}

pub fn stop_blocking() {
    platform_stop_blocking()
}

pub fn request_automation_permission(bundle_id: &str) -> bool {
    platform_request_automation_permission(bundle_id)
}
