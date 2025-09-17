use crate::{Monitor, MonitorError, WindowEvent};
use evdev::{Device, EventType};
#[allow(deprecated)]
use nix::sys::epoll::{
    epoll_create1, epoll_ctl, epoll_wait, EpollCreateFlags, EpollEvent, EpollFlags, EpollOp,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::sync::{Arc, Mutex};
use x11::xlib::*;

static LAST_FOCUSED_WINDOW: Lazy<Mutex<Option<Window>>> = Lazy::new(|| Mutex::new(None));
static MONITOR: Lazy<Mutex<Option<Arc<Monitor>>>> = Lazy::new(|| Mutex::new(None));
static OPENED_DEVICES: Lazy<Mutex<HashMap<i32, Device>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static EPOLL_FD: Lazy<Mutex<Option<i32>>> = Lazy::new(|| Mutex::new(None));

pub fn platform_start_monitoring(monitor: Arc<Monitor>) {
    *MONITOR.lock().unwrap() = Some(monitor);

    initialize_input_monitoring();

    log::info!("Linux monitoring started");
}

pub fn platform_detect_changes() -> Result<(), MonitorError> {
    // Check permissions first
    if !platform_has_accessibility_permissions() {
        return Err(MonitorError::PermissionDenied(
            "Root privileges required for input monitoring on Linux".to_string(),
        ));
    }

    match detect_focused_window() {
        Ok((window_id, app_name)) => {
            let mut last_focused_window = LAST_FOCUSED_WINDOW.lock().unwrap();

            if last_focused_window.map_or(true, |last_id| last_id != window_id) {
                if let Some(monitor) = MONITOR.lock().unwrap().clone() {
                    monitor.send_window_event(WindowEvent {
                        app_name,
                        window_title: String::new(),
                        bundle_id: None, // .desktop files on Linux (?)
                        url: None,
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
        if let Some(monitor) = MONITOR.lock().unwrap().clone() {
            if has_keyboard_activity {
                monitor.send_keyboard_event(true);
            }

            if has_mouse_activity {
                monitor.send_mouse_event(true);
            }
        }
    }

    Ok(())
}

pub fn platform_has_accessibility_permissions() -> bool {
    // On Linux, we need root privileges to access /dev/input/event* devices
    let is_root = unsafe { libc::geteuid() == 0 };

    if !is_root {
        log::warn!("Not running as root - input event monitoring will not work");
    }

    is_root
}

pub fn platform_request_accessibility_permissions() -> bool {
    log::error!("Input monitoring on Linux requires root privileges.");
    log::error!("Please run the application with sudo:");
    log::error!("  sudo cargo run");
    log::error!("  or");
    log::error!("  sudo ./target/release/os-monitor");

    false // Cannot automatically grant root permissions
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

        Ok((focus_window, title))
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

    None
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

    Some(class_name)
}

fn detect_input_activity() -> (bool, bool) {
    let mut has_keyboard_activity = false;
    let mut has_mouse_activity = false;

    // Get epoll file descriptor
    let epoll_fd = {
        let guard = EPOLL_FD.lock().unwrap();
        match *guard {
            Some(fd) => fd,
            _ => {
                log::debug!("Epoll not initialized");
                return (false, false);
            }
        }
    };

    // Check for events with 0 timeout (non-blocking)
    let mut events = [EpollEvent::empty(); 32];
    #[allow(deprecated)]
    let num_events = match epoll_wait(epoll_fd, &mut events, 0) {
        Ok(count) => count,
        Err(e) => {
            log::debug!("Epoll wait failed: {}", e);
            return (false, false);
        }
    };

    if num_events == 0 {
        return (false, false);
    }

    log::debug!("Epoll found {} devices with events", num_events);

    // Get devices map
    let mut devices = match OPENED_DEVICES.lock() {
        Ok(guard) => guard,
        Err(_) => {
            log::error!("Failed to lock devices");
            return (false, false);
        }
    };

    // Process events from each ready device
    for event in &events[..num_events] {
        let fd = event.data() as i32;

        let device = match devices.get_mut(&fd) {
            Some(dev) => dev,
            _ => {
                log::debug!("Unknown device fd: {}", fd);
                continue;
            }
        };

        let device_events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                log::debug!("Failed to fetch events from device {}: {}", fd, e);
                continue;
            }
        };

        for ev in device_events {
            match ev.event_type() {
                EventType::KEY => {
                    has_keyboard_activity = true;
                    log::debug!("Keyboard event detected");
                }
                EventType::RELATIVE | EventType::ABSOLUTE => {
                    has_mouse_activity = true;
                    log::debug!("Mouse event detected");
                }
                _ => {}
            }
        }
    }

    (has_keyboard_activity, has_mouse_activity)
}

fn initialize_input_monitoring() {
    // Check if already initialized
    {
        let guard = EPOLL_FD.lock().unwrap();
        if guard.is_some() {
            return;
        }
    }

    // Create epoll instance
    #[allow(deprecated)]
    let epoll_fd = match epoll_create1(EpollCreateFlags::empty()) {
        Ok(fd) => fd,
        Err(e) => {
            log::error!("Failed to create epoll: {}", e);
            return;
        }
    };

    // Read /dev/input directory
    let entries = match fs::read_dir("/dev/input") {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("Failed to read /dev/input: {}", e);
            return;
        }
    };

    let mut devices = OPENED_DEVICES.lock().unwrap();
    let mut device_count = 0;

    // Open and register each input device
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        if !file_name.starts_with("event") {
            continue;
        }

        // Open device
        let device = match Device::open(&path) {
            Ok(dev) => dev,
            Err(e) => {
                log::debug!("Failed to open device {}: {}", path.display(), e);
                continue;
            }
        };

        // Set non-blocking mode
        if let Err(e) = device.set_nonblocking(true) {
            log::warn!(
                "Failed to set device {} to non-blocking: {}",
                path.display(),
                e
            );
            continue;
        }

        // Get file descriptor and register with epoll
        let fd = device.as_raw_fd();
        let mut event = EpollEvent::new(EpollFlags::EPOLLIN, fd as u64);

        #[allow(deprecated)]
        if let Err(e) = epoll_ctl(epoll_fd, EpollOp::EpollCtlAdd, fd, &mut event) {
            log::warn!("Failed to add device {} to epoll: {}", path.display(), e);
            continue;
        }

        // Store device
        devices.insert(fd, device);
        device_count += 1;
        log::debug!("Added device {} to epoll monitoring", path.display());
    }

    // Store epoll fd
    {
        let mut guard = EPOLL_FD.lock().unwrap();
        *guard = Some(epoll_fd);
    }

    log::info!("Input monitoring initialized with {} devices", device_count);
}

