use crate::event::{Platform, WindowEvent};
use crate::utils::log::log;
use crate::{bindings, event::EventCallback, Monitor};
use crate::{KeyboardEvent, MonitorError, MouseEvent, MouseEventType};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct WindowTitle {
    app_name: String,
    title: String,
}

static MONITOR: Lazy<Mutex<Option<Arc<Monitor>>>> = Lazy::new(|| Mutex::new(None));

static MOUSE_EVENTS: Mutex<Vec<MouseEvent>> = Mutex::new(Vec::new());
static KEYBOARD_EVENTS: Mutex<Vec<KeyboardEvent>> = Mutex::new(Vec::new());

static LAST_SEND: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

static FOCUSED_WINDOW: Mutex<WindowTitle> = Mutex::new(WindowTitle {
    app_name: String::new(),
    title: String::new(),
});

fn detect_focused_window() {
    unsafe {
        log("detect_focused_window start");
        let window_title: *const bindings::RawWindowTitle = bindings::detect_focused_window();
        log("detect_focused_window end");
        if window_title.is_null() {
            log("  detect_focused_window null");
            return;
        }

        let title = std::ffi::CStr::from_ptr((*window_title).window_title)
            .to_str()
            .unwrap();
        let app_name = std::ffi::CStr::from_ptr((*window_title).app_name)
            .to_str()
            .unwrap();

        let url = (*window_title).get_url();

        {
            log("  detect_focused_window lock");
            let mut window_title_guard = FOCUSED_WINDOW.lock().unwrap();
            log("  detect_focused_window lock end");
            let monitor_guard = MONITOR.lock().unwrap();
            log("  detect_focused_window callback_guard");
            if app_name.to_string() != window_title_guard.app_name
                || title.to_string() != window_title_guard.title
            {
                log("    detect_focused_window callback");
                if let Some(monitor) = monitor_guard.as_ref() {
                    log("      detect_focused_window callback Some");
                    monitor.on_window_event(WindowEvent {
                        window_title: title.to_string(),
                        app_name: app_name.to_string(),
                        url: url,
                        platform: Platform::Mac,
                    });
                    log("      detect_focused_window callback Some end");
                }
            }
            window_title_guard.title = title.to_string();
            window_title_guard.app_name = app_name.to_string();
            log("  detect_focused_window lock end");
        }
    }
}

extern "C" fn mouse_event_callback(x: f64, y: f64, event_type: i32, scroll_delta: i32) {
    let mouse_event = MouseEvent {
        x,
        y,
        event_type: MouseEventType::try_from(event_type).unwrap(),
        scroll_delta,
        platform: Platform::Mac,
    };
    // Store event in vector
    let mut events = MOUSE_EVENTS.lock().unwrap();
    events.push(mouse_event);
}

extern "C" fn keyboard_event_callback(key_code: i32) {
    let keyboard_event = KeyboardEvent {
        key_code,
        platform: Platform::Mac,
    };
    // Store event in vector
    let mut events = KEYBOARD_EVENTS.lock().unwrap();
    events.push(keyboard_event);
}

fn send_buffered_events() {
    let monitor_guard = MONITOR.lock().unwrap();
    if let Some(monitor) = monitor_guard.as_ref() {
        // Send mouse events
        let mut mouse_events = MOUSE_EVENTS.lock().unwrap();
        monitor.on_mouse_events(mouse_events.drain(..).collect());

        // Send keyboard events
        let mut keyboard_events = KEYBOARD_EVENTS.lock().unwrap();
        monitor.on_keyboard_events(keyboard_events.drain(..).collect());
    }
}

pub(crate) fn platform_initialize_monitor(monitor: Arc<Monitor>) -> Result<(), MonitorError> {
    let mut monitor_guard = MONITOR.lock().unwrap();

    *monitor_guard = Some(monitor);

    unsafe {
        bindings::start_mouse_monitoring(mouse_event_callback);
        bindings::start_keyboard_monitoring(keyboard_event_callback);
    }
    Ok(())
}

pub(crate) fn platform_detect_changes() -> Result<(), MonitorError> {
    log("platform_detect_changes start");
    unsafe {
        bindings::process_events();
    }
    log("processed events");
    detect_focused_window();
    log("detected focused window");

    let mut last_send = LAST_SEND.lock().unwrap();
    log(&format!("last_send: {:?}", last_send.elapsed()));

    if last_send.elapsed() >= Duration::from_secs(30) {
        log("sending buffered events");
        send_buffered_events();
        log("sent buffered events");
        *last_send = Instant::now();
    }
    log("platform_detect_changes end");
    Ok(())
}
