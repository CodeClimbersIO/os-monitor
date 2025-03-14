use crate::event::{BlockedApp, Platform, WindowEvent};
use crate::{bindings, event::EventCallback, Monitor};
use crate::{BlockedAppEvent, MonitorError};
use once_cell::sync::Lazy;
use std::ffi::{c_char, CStr, CString};
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

        // Check if URL is blocked and redirect if needed
        if let Some(url_str) = &url {
            log::info!("is blocked? {}", url_str);
            let c_url = CString::new(url_str.clone()).unwrap_or_default();
            log::info!("c_url: {:?}", c_url);
            if bindings::is_blocked(c_url.as_ptr()) {
                log::info!("Url is blocked, redirecting to vibes page: {}", url_str);
                let redirect_result = bindings::redirect_to_vibes_page();

                let blocked_app = BlockedApp {
                    app_name: app_name.to_string(),
                    app_external_id: url_str.to_string(),
                    is_site: true,
                };

                let monitor_guard = MONITOR.lock().unwrap();
                if let Some(monitor) = monitor_guard.as_ref() {
                    monitor.on_app_blocked(BlockedAppEvent {
                        blocked_apps: vec![blocked_app],
                    });
                }

                log::info!("Redirect result: {}", redirect_result);
            }
        }
        if let Some(bundle_id) = bundle_id.clone() {
            let c_bundle_id = CString::new(bundle_id.clone()).unwrap_or_default();
            if bindings::is_blocked(c_bundle_id.as_ptr()) {
                log::info!("App is blocked, closing app: {:?}", bundle_id);
                let close_result = bindings::close_app(c_bundle_id.as_ptr(), true);
                log::info!("Close result: {}", close_result);
            }
        }

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

extern "C" fn mouse_event_callback(_: f64, _: f64, _: i32, _: i32) {
    // Store event in vector
    let mut has_activity = HAS_MOUSE_ACTIVITY.lock().unwrap();
    *has_activity = true;
}

extern "C" fn keyboard_event_callback(_: i32) {
    log::trace!("keyboard_event_callback");
    let mut has_activity = HAS_KEYBOARD_ACTIVITY.lock().unwrap();
    *has_activity = true;
}

extern "C" fn app_blocked_callback(
    app_names: *const *const c_char,
    bundle_ids: *const *const c_char,
    count: i32,
) {
    unsafe {
        let monitor_guard = MONITOR.lock().unwrap();
        if let Some(monitor) = monitor_guard.as_ref() {
            // Create a vector to hold all blocked app events
            let mut blocked_apps = Vec::with_capacity(count as usize);

            for i in 0..count as isize {
                let app_name_ptr = *app_names.offset(i);
                let bundle_id_ptr = *bundle_ids.offset(i);

                let app_name_str = if !app_name_ptr.is_null() {
                    CStr::from_ptr(app_name_ptr).to_string_lossy().into_owned()
                } else {
                    String::from("Unknown App")
                };

                let bundle_id_str = if !bundle_id_ptr.is_null() {
                    CStr::from_ptr(bundle_id_ptr).to_string_lossy().into_owned()
                } else {
                    String::from("unknown.bundle.id")
                };

                log::info!("App blocked: {} ({})", app_name_str, bundle_id_str);
                blocked_apps.push(BlockedApp {
                    app_name: app_name_str,
                    app_external_id: bundle_id_str,
                    is_site: false,
                });
            }

            log::trace!("app_blocked_callback blocked_apps: {:?}", blocked_apps);
            monitor.on_app_blocked(BlockedAppEvent {
                blocked_apps: blocked_apps,
            });
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

    unsafe {
        bindings::register_app_blocked_callback(app_blocked_callback);
        bindings::start_monitoring(mouse_event_callback, keyboard_event_callback);
    }
    log::trace!("bindings::start_monitoring end");
}

pub(crate) fn platform_start_blocking(urls: &[String], redirect_url: &str) -> bool {
    log::trace!("platform_start_blocking start");
    let c_urls: Vec<CString> = urls
        .iter()
        .map(|url| CString::new(url.as_str()).unwrap())
        .collect();

    let c_urls_ptrs: Vec<*const c_char> = c_urls.iter().map(|url| url.as_ptr()).collect();

    let c_redirect_url = CString::new(redirect_url).unwrap();

    log::trace!("platform_start_blocking start");
    unsafe {
        bindings::start_blocking(
            c_urls_ptrs.as_ptr(),
            c_urls_ptrs.len() as i32,
            c_redirect_url.as_ptr(),
        )
    }
}

pub(crate) fn platform_stop_blocking() {
    unsafe {
        bindings::stop_blocking();
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
