#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

use crate::{event::Monitor, MonitorError};
use std::sync::Arc;

pub fn detect_changes() -> Result<(), MonitorError> {
    platform_detect_changes()
}

pub fn initialize_monitor(monitor: Arc<Monitor>) -> Result<(), MonitorError> {
    platform_initialize_monitor(monitor)
}

pub fn has_accessibility_permissions() -> bool {
    platform_has_accessibility_permissions()
}

pub fn request_accessibility_permissions() -> bool {
    platform_request_accessibility_permissions()
}

pub fn get_application_icon_path(bundle_id: &str) -> Option<String> {
    platform_get_application_icon_path(bundle_id)
}
