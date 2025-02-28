use crate::event::{Platform, WindowEvent};
use crate::MonitorError;
use crate::{bindings, event::EventCallback, Monitor};
use once_cell::sync::Lazy;
use std::ffi::c_char;
use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct WindowTitle {
    app_name: String,
    title: String,
}

static MONITOR: Lazy<Mutex<Option<Arc<Monitor>>>> = Lazy::new(|| Mutex::new(None));

static HAS_MOUSE_ACTIVITY: Mutex<bool> = Mutex::new(false);
static HAS_KEYBOARD_ACTIVITY: Mutex<bool> = Mutex::new(false);

static LAST_SEND: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

static FOCUSED_WINDOW: Mutex<WindowTitle> = Mutex::new(WindowTitle {
    app_name: String::new(),
    title: String::new(),
});

fn detect_focused_window() {
    unsafe {
        log::info!("detect_focused_window start");
        let window_title: *const bindings::RawWindowTitle = bindings::detect_focused_window();
        if window_title.is_null() {
            log::warn!("  detect_focused_window null");
            return;
        }

        let title = std::ffi::CStr::from_ptr((*window_title).window_title)
            .to_str()
            .unwrap();

        let app_name = std::ffi::CStr::from_ptr((*window_title).app_name)
            .to_str()
            .unwrap();

        let bundle_id = (*window_title).get_bundle_id();
        let url = (*window_title).get_url();
        log::info!(
            "  detect_focused_window window_title: {:?} {:?}",
            app_name,
            title
        );
        log::info!("  detect_focused_window bundle_id: {:?}", bundle_id);
        log::info!("  detect_focused_window url: {:?}", url);

        {
            log::info!("  detect_focused_window lock");
            let mut window_title_guard = FOCUSED_WINDOW.lock().unwrap();
            log::info!("  detect_focused_window lock end");
            let monitor_guard = MONITOR.lock().unwrap();
            log::info!("  detect_focused_window callback_guard");
            if app_name.to_string() != window_title_guard.app_name
                || title.to_string() != window_title_guard.title
            {
                log::info!("    detect_focused_window callback");
                if let Some(monitor) = monitor_guard.as_ref() {
                    log::info!("      detect_focused_window callback Some");
                    monitor.on_window_event(WindowEvent {
                        window_title: title.to_string(),
                        app_name: app_name.to_string(),
                        url: url,
                        bundle_id: bundle_id,
                        platform: Platform::Mac,
                    });
                    log::info!("      detect_focused_window callback Some end");
                }
            }
            window_title_guard.title = title.to_string();
            window_title_guard.app_name = app_name.to_string();
            log::info!("  detect_focused_window lock end");
        }
    }
}

/// Checks if the current URL is blocked and redirects if necessary
fn check_and_block_url() {
    // Add a timeout to prevent hanging
    let now = std::time::Instant::now();

    unsafe {
        let window_title = bindings::detect_focused_window();
        if window_title.is_null() {
            return;
        }

        // Check if the URL is blocked
        if let Some(url) = (*window_title).get_url() {
            // Add debug logging
            log::info!("Checking URL: {}", url);

            // Check for timeout
            if now.elapsed() > std::time::Duration::from_secs(5) {
                log::error!("URL checking timed out - aborting");
                return;
            }

            let c_url = CString::new(url.clone()).unwrap();
            if bindings::is_url_blocked(c_url.as_ptr()) {
                log::info!("URL is blocked, redirecting to vibes page: {}", url);
                let redirect_result = bindings::redirect_to_vibes_page();
                log::info!("Redirect result: {}", redirect_result);
            }
        }
    }
}

extern "C" fn mouse_event_callback(_: f64, _: f64, _: i32, _: i32) {
    // Store event in vector
    let mut has_activity = HAS_MOUSE_ACTIVITY.lock().unwrap();
    *has_activity = true;
}

extern "C" fn keyboard_event_callback(keycode: i32) {
    let mut has_activity = HAS_KEYBOARD_ACTIVITY.lock().unwrap();
    *has_activity = true;

    // Check if Enter key was pressed (keycode 36)
    if keycode == 36 {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            check_and_block_url();

            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                check_and_block_url();
            });
        });
    }
}

fn send_buffered_events() {
    let monitor_guard = MONITOR.lock().unwrap();
    if let Some(monitor) = monitor_guard.as_ref() {
        {
            let mut has_activity = HAS_KEYBOARD_ACTIVITY.lock().unwrap();
            monitor.on_keyboard_events(has_activity.clone());
            *has_activity = false;
        }

        {
            let mut has_activity = HAS_MOUSE_ACTIVITY.lock().unwrap();
            monitor.on_mouse_events(has_activity.clone());
            *has_activity = false;
        }
    }
}

pub(crate) fn platform_detect_changes() -> Result<(), MonitorError> {
    log::info!("platform_detect_changes start");
    detect_focused_window();
    log::info!("detected focused window");

    let mut last_send = LAST_SEND.lock().unwrap();
    log::info!("last_send: {:?}", last_send.elapsed());

    if last_send.elapsed() >= Duration::from_secs(30) {
        log::info!("sending buffered events");
        send_buffered_events();
        log::info!("sent buffered events");
        *last_send = Instant::now();
    }
    log::info!("platform_detect_changes end");
    Ok(())
}

pub(crate) fn platform_has_accessibility_permissions() -> bool {
    unsafe { bindings::has_accessibility_permissions() }
}

pub(crate) fn platform_request_accessibility_permissions() -> bool {
    unsafe { bindings::request_accessibility_permissions() }
}

pub(crate) fn platform_get_application_icon_data(bundle_id: &str) -> Option<String> {
    unsafe {
        let c_bundle_id = CString::new(bundle_id).ok()?;
        let c_data = bindings::get_app_icon_data(c_bundle_id.as_ptr());
        if c_data.is_null() {
            return None;
        }

        let data = CStr::from_ptr(c_data).to_str().ok()?.to_owned();
        bindings::free_icon_data(c_data);
        Some(data)
    }
}

pub(crate) fn platform_start_monitoring(monitor: Arc<Monitor>) {
    log::info!("platform_start_monitoring start");
    {
        let mut monitor_guard = MONITOR.lock().unwrap();
        *monitor_guard = Some(monitor);
    }
    log::info!("platform_start_monitoring end");
    unsafe {
        bindings::start_monitoring(mouse_event_callback, keyboard_event_callback);
    }
    log::info!("bindings::start_monitoring end");
}

pub(crate) fn platform_start_site_blocking(urls: &[String], redirect_url: &str) -> bool {
    log::info!("platform_start_site_blocking start");
    let c_urls: Vec<CString> = urls
        .iter()
        .map(|url| CString::new(url.as_str()).unwrap())
        .collect();

    let c_urls_ptrs: Vec<*const c_char> = c_urls.iter().map(|url| url.as_ptr()).collect();

    let c_redirect_url = CString::new(redirect_url).unwrap();

    log::info!("platform_start_site_blocking start");
    unsafe {
        bindings::start_site_blocking(
            c_urls_ptrs.as_ptr(),
            c_urls_ptrs.len() as i32,
            c_redirect_url.as_ptr(),
        )
    }
}

pub(crate) fn platform_stop_site_blocking() {
    unsafe {
        bindings::stop_site_blocking();
    }
}

pub(crate) fn platform_request_automation_permission(bundle_id: &str) -> bool {
    unsafe {
        match CString::new(bundle_id) {
            Ok(c_bundle_id) => bindings::request_automation_permission(c_bundle_id.as_ptr()),
            Err(_) => false,
        }
    }
}
