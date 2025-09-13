use crate::{Monitor, MonitorError, WindowEvent};
use evdev::{Device, EventType};
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::fs;
use std::ptr;
use std::sync::{Arc, Mutex};
use x11::xlib::*;

static LAST_FOCUSED_WINDOW: Lazy<Mutex<Option<Window>>> = Lazy::new(|| Mutex::new(None));
static MONITOR: Lazy<Mutex<Option<Arc<Monitor>>>> = Lazy::new(|| Mutex::new(None));

pub fn platform_start_monitoring(monitor: Arc<Monitor>) {
    *MONITOR.lock().unwrap() = Some(monitor);

    log::info!("Linux monitoring started");
}

pub fn platform_detect_changes() -> Result<(), MonitorError> {
    match detect_focused_window() {
        Ok((window_id, app_name)) => {
            let mut last_focused_window = LAST_FOCUSED_WINDOW.lock().unwrap();

            if last_focused_window.map_or(true, |last_id| last_id != window_id) {
                if let Some(monitor) = MONITOR.lock().unwrap().clone() {
                    monitor.send_window_event(WindowEvent {
                        app_name,
                        window_title: String::new(),
                        bundle_id: None, // .desktop files on Linux (?)
                        url: None,       // TODO: browser integration
                        platform: crate::Platform::Linux,
                    })
                }

                *last_focused_window = Some(window_id);
            }
        }
        Err(e) => log::error!("Failed to detect focused window: {}", e),
    }

    let (has_keyboard_activity, has_mouse_activity) = detect_input_activity();

    if has_keyboard_activity || has_mouse_activity {
        log::info!(
            "input activity detected: keyboard={}, mouse={}",
            has_keyboard_activity,
            has_mouse_activity
        );

        // TODO: send keeb/mouse event to monitor
    }

    return Ok(());
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

        let title = get_window_class(display, focus_window)
            .or_else(|| get_window_title(display, focus_window))
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

unsafe fn get_window_class(display: *mut Display, window: Window) -> Option<String> {
    let mut class_hint = std::mem::zeroed::<XClassHint>();

    let status_code = XGetClassHint(display, window, &mut class_hint);

    if status_code == 0 || class_hint.res_class.is_null() {
        return None;
    }

    let c_str = CStr::from_ptr(class_hint.res_class);
    let class_name = c_str.to_string_lossy().to_string();

    XFree(class_hint.res_class as *mut _);

    if !class_hint.res_name.is_null() {
        XFree(class_hint.res_name as *mut _);
    }

    return Some(class_name);
}

fn detect_input_activity() -> (bool, bool) {
    let mut has_keyboard_activity = false;
    let mut has_mouse_activity = false;

    let entries = match fs::read_dir("/dev/input") {
        Ok(entries) => entries,
        Err(_) => return (false, false),
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        if !file_name.starts_with("event") {
            continue;
        }

        let mut device = match Device::open(&path) {
            Ok(dev) => {
                // FIXME: this should work per the examples but it doesn't exist?
                // dev.set_nonblocking(true)?;
                dev
            }
            Err(_) => continue,
        };

        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(_) => continue,
        };

        for event in events {
            match event.event_type() {
                EventType::KEY => has_keyboard_activity = true,
                EventType::RELATIVE | EventType::ABSOLUTE => has_mouse_activity = true,
                _ => {}
            }
        }
    }

    return (has_keyboard_activity, has_mouse_activity);
}
