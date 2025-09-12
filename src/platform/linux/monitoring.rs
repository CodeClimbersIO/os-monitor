use crate::{Monitor, MonitorError};
use std::ffi::CStr;
use std::ptr;
use std::sync::Arc;
use x11::xlib::*;

pub fn platform_start_monitoring(_monitor: Arc<Monitor>) {
    // TODO: Initialize X11 connection and event monitoring
    log::warn!("platform_start_monitoring not yet implemented for Linux");
}

pub fn platform_detect_changes() -> Result<(), MonitorError> {
    match detect_focused_window() {
        Ok((window_id, title)) => {
            log::info!("Focused window: {} (ID: {})", title, window_id);
            // TODO: send window event to the event system
        }
        Err(e) => {
            log::error!("Failed to detect focused window: {}", e);
        }
    }

    Ok(())
}

pub fn platform_has_accessibility_permissions() -> bool {
    // TODO: Check if running with appropriate permissions (likely root or input group)
    log::warn!("platform_has_accessibility_permissions not yet implemented for Linux");
    true // Assume we have permissions for now
}

pub fn platform_request_accessibility_permissions() -> bool {
    // TODO: Guide user to run with appropriate permissions
    log::warn!("platform_request_accessibility_permissions not yet implemented for Linux");
    true // Assume success for now
}

fn detect_focused_window() -> Result<(Window, String), Box<dyn std::error::Error>> {
    unsafe {
        let display = XOpenDisplay(ptr::null());
        if display.is_null() {
            return Err("Failed to open X11 display".into());
        }

        let mut focus_window: Window = 0;
        let mut revert_to: i32 = 0;

        XGetInputFocus(display, &mut focus_window, &mut revert_to);

        let title = get_window_title(display, focus_window)
            .or_else(|| get_parent_window_title(display, focus_window))
            .unwrap_or_else(|| "Unknown".to_string());

        XCloseDisplay(display);

        return Ok((focus_window, title));
    }
}

unsafe fn get_window_title(display: *mut Display, window: Window) -> Option<String> {
    let mut window_name: *mut i8 = ptr::null_mut();
    let status = XFetchName(display, window, &mut window_name);

    if status != 0 && !window_name.is_null() {
        let c_str = CStr::from_ptr(window_name);
        let title = c_str.to_string_lossy().to_string();

        XFree(window_name as *mut _);

        if !title.is_empty() {
            return Some(title);
        }
    }

    return None;
}

unsafe fn get_parent_window_title(display: *mut Display, window: Window) -> Option<String> {
    let mut root: Window = 0;
    let mut parent: Window = 0;
    let mut children: *mut Window = ptr::null_mut();
    let mut nchildren: u32 = 0;

    let status = XQueryTree(
        display,
        window,
        &mut root,
        &mut parent,
        &mut children,
        &mut nchildren,
    );

    if status != 0 && parent != root && parent != 0 {
        if !children.is_null() {
            XFree(children as *mut _);
        }
        return get_window_title(display, parent);
    }

    if !children.is_null() {
        XFree(children as *mut _);
    }
    None
}
