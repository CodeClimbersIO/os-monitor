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

pub fn create_screen_border(red: f64, green: f64, blue: f64, width: f64, opacity: f64) {
    platform_create_screen_border(red, green, blue, width, opacity)
}

pub fn remove_screen_border(border_window: &str) {
    platform_remove_screen_border(border_window)
}

pub fn run_loop_cycle() {
    platform_run_loop_cycle()
}

pub fn create_screen_grayscale(opacity: f64) {
    platform_create_screen_grayscale(opacity)
}

pub fn remove_screen_grayscale(grayscale_window: &str) {
    platform_remove_screen_grayscale(grayscale_window)
}

pub fn create_screen_false_color(
    opacity: f64,
    color0_r: f64,
    color0_g: f64,
    color0_b: f64,
    color1_r: f64,
    color1_g: f64,
    color1_b: f64,
) {
    platform_create_screen_false_color(
        opacity, color0_r, color0_g, color0_b, color1_r, color1_g, color1_b,
    );
}

pub fn remove_screen_false_color(false_color_window: &str) {
    platform_remove_screen_false_color(false_color_window);
}
