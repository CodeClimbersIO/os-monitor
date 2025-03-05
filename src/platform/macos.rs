use crate::event::{Platform, WindowEvent};
use crate::MonitorError;
use crate::{bindings, event::EventCallback, Monitor};
use once_cell::sync::Lazy;
use std::ffi::c_char;
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
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

static URL_BLOCKING_IN_PROGRESS: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static CURRENT_BLOCKING_THREAD: Lazy<Mutex<Option<Arc<AtomicBool>>>> =
    Lazy::new(|| Mutex::new(None));

// List of known browser bundle IDs
const BROWSER_BUNDLE_IDS: &[&str] = &[
    "com.apple.Safari",
    "com.google.Chrome",
    "org.mozilla.firefox",
    "com.microsoft.edgemac",
    "com.brave.Browser",
    "com.operasoftware.Opera",
    "com.vivaldi.Vivaldi",
];

fn detect_focused_window() {
    unsafe {
        log::trace!("detect_focused_window start");
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
        log::trace!(
            "  detect_focused_window window_title: {:?} {:?}",
            app_name,
            title
        );
        log::trace!("  detect_focused_window bundle_id: {:?}", bundle_id);
        log::trace!("  detect_focused_window url: {:?}", url);

        {
            log::trace!("  detect_focused_window lock");
            let mut window_title_guard = FOCUSED_WINDOW.lock().unwrap();
            log::trace!("  detect_focused_window lock end");
            let monitor_guard = MONITOR.lock().unwrap();
            log::trace!("  detect_focused_window callback_guard");
            if app_name.to_string() != window_title_guard.app_name
                || title.to_string() != window_title_guard.title
            {
                log::trace!("    detect_focused_window callback");
                if let Some(monitor) = monitor_guard.as_ref() {
                    log::trace!("      detect_focused_window callback Some");
                    monitor.on_window_event(WindowEvent {
                        window_title: title.to_string(),
                        app_name: app_name.to_string(),
                        url: url,
                        bundle_id: bundle_id,
                        platform: Platform::Mac,
                    });
                    log::trace!("      detect_focused_window callback Some end");
                }
            }
            window_title_guard.title = title.to_string();
            window_title_guard.app_name = app_name.to_string();
            log::trace!("  detect_focused_window lock end");
        }
    }
}

/// Checks if the current URL is blocked and redirects if necessary
/// Returns true if URL was blocked and redirection was successful, false otherwise
fn check_and_block_url() -> bool {
    // Add a timeout to prevent hanging
    let now = std::time::Instant::now();
    unsafe {
        let window_title = bindings::detect_focused_window();
        if window_title.is_null() {
            return false;
        }

        // Check if the current app is a browser
        if let Some(bundle_id) = (*window_title).get_bundle_id() {
            if !BROWSER_BUNDLE_IDS.iter().any(|&id| id == bundle_id) {
                log::warn!(
                    "Current app is not a browser ({}), ignoring URL blocking",
                    bundle_id
                );
                return false;
            }
        } else {
            log::warn!("No bundle ID available, ignoring URL blocking");
            return false;
        }

        // Check if the URL is blocked
        if let Some(url) = (*window_title).get_url() {
            // Add debug logging
            log::warn!("Checking URL: {}", url);

            // Check for timeout
            if now.elapsed() > std::time::Duration::from_secs(5) {
                log::error!("URL checking timed out - aborting");
                return false;
            }

            let c_url = CString::new(url.clone()).unwrap();
            if bindings::is_url_blocked(c_url.as_ptr()) {
                log::trace!("URL is blocked, redirecting to vibes page: {}", url);
                let redirect_result = bindings::redirect_to_vibes_page();
                log::trace!("Redirect result: {}", redirect_result);
                return redirect_result;
            }
        }
    }

    false
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
        log::warn!("Enter key pressed");

        // Signal any existing thread to stop and start a new one
        {
            let mut thread_guard = CURRENT_BLOCKING_THREAD.lock().unwrap();
            if let Some(stop_flag) = thread_guard.as_ref() {
                // Signal the existing thread to stop
                stop_flag.store(true, Ordering::SeqCst);
                log::trace!("Signaled existing URL blocking thread to stop");
            }

            // Create a new stop flag for the new thread
            let stop_flag = Arc::new(AtomicBool::new(false));
            *thread_guard = Some(stop_flag.clone());

            // Spawn a new thread with the stop flag
            std::thread::spawn(move || {
                attempt_url_blocking(stop_flag);
            });
        }
    }
}

fn attempt_url_blocking(stop_flag: Arc<AtomicBool>) {
    // Mark URL blocking as in progress
    {
        let mut blocking_in_progress = URL_BLOCKING_IN_PROGRESS.lock().unwrap();
        *blocking_in_progress = true;
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut retry_count = 0;
    let max_retries = 5;

    while retry_count < max_retries {
        // Check if we've been signaled to stop
        if stop_flag.load(Ordering::SeqCst) {
            log::trace!("URL blocking thread received stop signal, terminating");

            // Clear the in-progress flag only if we're the thread being stopped
            // and not a new thread that's just starting
            let thread_guard = CURRENT_BLOCKING_THREAD.lock().unwrap();
            if let Some(current_flag) = thread_guard.as_ref() {
                if Arc::ptr_eq(current_flag, &stop_flag) {
                    let mut blocking_in_progress = URL_BLOCKING_IN_PROGRESS.lock().unwrap();
                    *blocking_in_progress = false;
                }
            }

            return;
        }

        log::warn!("URL blocking attempt {}", retry_count);
        if check_and_block_url() {
            break;
        }

        // Exponential backoff: 500ms, 1000ms, 2000ms, etc.
        let backoff_ms = 500 * (2_u64.pow(retry_count));
        log::trace!("URL blocking attempt failed, retrying in {}ms", backoff_ms);

        // Sleep in small chunks so we can check for stop signals
        let start_time = std::time::Instant::now();
        let sleep_chunk = std::time::Duration::from_millis(100);

        while start_time.elapsed() < std::time::Duration::from_millis(backoff_ms) {
            std::thread::sleep(sleep_chunk);

            // Check if we've been signaled to stop
            if stop_flag.load(Ordering::SeqCst) {
                log::trace!("URL blocking thread received stop signal during backoff, terminating");

                // Clear the in-progress flag only if we're the thread being stopped
                let thread_guard = CURRENT_BLOCKING_THREAD.lock().unwrap();
                if let Some(current_flag) = thread_guard.as_ref() {
                    if Arc::ptr_eq(current_flag, &stop_flag) {
                        let mut blocking_in_progress = URL_BLOCKING_IN_PROGRESS.lock().unwrap();
                        *blocking_in_progress = false;
                    }
                }

                return;
            }
        }

        retry_count += 1;
    }

    if retry_count == max_retries {
        log::warn!("Failed to block URL after {} attempts", max_retries);
    }

    // Mark URL blocking as no longer in progress, but only if we're still the current thread
    let mut thread_guard = CURRENT_BLOCKING_THREAD.lock().unwrap();
    if let Some(current_flag) = thread_guard.as_ref() {
        if Arc::ptr_eq(current_flag, &stop_flag) {
            let mut blocking_in_progress = URL_BLOCKING_IN_PROGRESS.lock().unwrap();
            *blocking_in_progress = false;
            *thread_guard = None;
        }
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
    log::trace!("platform_detect_changes start");
    detect_focused_window();
    log::trace!("detected focused window");

    let mut last_send = LAST_SEND.lock().unwrap();
    log::trace!("last_send: {:?}", last_send.elapsed());

    if last_send.elapsed() >= Duration::from_secs(30) {
        log::trace!("sending buffered events");
        send_buffered_events();
        log::trace!("sent buffered events");
        *last_send = Instant::now();
    }
    log::trace!("platform_detect_changes end");
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
    log::trace!("platform_start_monitoring start");
    {
        let mut monitor_guard = MONITOR.lock().unwrap();
        *monitor_guard = Some(monitor);
    }
    log::trace!("platform_start_monitoring end");

    // Start observer-based window monitoring
    platform_start_window_observer_monitoring();

    // Also keep the existing monitoring for mouse/keyboard events
    unsafe {
        bindings::start_monitoring(mouse_event_callback, keyboard_event_callback);
    }
    log::trace!("bindings::start_monitoring end");
}

pub(crate) fn platform_start_site_blocking(urls: &[String], redirect_url: &str) -> bool {
    log::trace!("platform_start_site_blocking start");
    let c_urls: Vec<CString> = urls
        .iter()
        .map(|url| CString::new(url.as_str()).unwrap())
        .collect();

    let c_urls_ptrs: Vec<*const c_char> = c_urls.iter().map(|url| url.as_ptr()).collect();

    let c_redirect_url = CString::new(redirect_url).unwrap();

    log::trace!("platform_start_site_blocking start");
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

extern "C" fn window_observer_callback(
    app_name: *const c_char,
    window_title: *const c_char,
    bundle_id: *const c_char,
    url: *const c_char,
) {
    log::warn!("window_observer_callback start");

    let app_name = unsafe {
        if app_name.is_null() {
            String::new()
        } else {
            CStr::from_ptr(app_name).to_string_lossy().to_string()
        }
    };

    let title = unsafe {
        if window_title.is_null() {
            String::new()
        } else {
            CStr::from_ptr(window_title).to_string_lossy().to_string()
        }
    };

    let bundle_id = unsafe {
        if bundle_id.is_null() {
            None
        } else {
            Some(CStr::from_ptr(bundle_id).to_string_lossy().to_string())
        }
    };
    let url = unsafe {
        if url.is_null() {
            None
        } else {
            Some(CStr::from_ptr(url).to_string_lossy().to_string())
        }
    };

    let monitor_guard = MONITOR.lock().unwrap();
    if let Some(monitor) = monitor_guard.as_ref() {
        monitor.on_window_event(WindowEvent {
            window_title: title,
            app_name,
            url,
            bundle_id,
            platform: Platform::Mac,
        });
    }
}

pub(crate) fn platform_start_window_observer_monitoring() -> bool {
    log::trace!("Starting window observer monitoring");
    unsafe { bindings::start_window_observer_monitoring(window_observer_callback) }
}

pub(crate) fn platform_stop_window_observer_monitoring() {
    log::trace!("Stopping window observer monitoring");
    unsafe { bindings::stop_window_observer_monitoring() }
}

pub(crate) fn platform_is_window_observer_monitoring() -> bool {
    unsafe { bindings::is_window_observer_monitoring() }
}
