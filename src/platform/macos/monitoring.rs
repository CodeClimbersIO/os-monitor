use super::events::{send_buffered_events, LAST_SEND};
use crate::event::{Platform, WindowEvent};
use crate::BlockedAppEvent;
use crate::{bindings, Monitor};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub static MONITOR: Lazy<Mutex<Option<Arc<Monitor>>>> = Lazy::new(|| Mutex::new(None));

pub struct WindowTitle {
    pub app_name: String,
    pub title: String,
}

pub static FOCUSED_WINDOW: Mutex<WindowTitle> = Mutex::new(WindowTitle {
    app_name: String::new(),
    title: String::new(),
});

pub fn detect_focused_window() {
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

        if let Some(url_str) = &url {
            log::info!("is blocked? {}", url_str);

            let c_url = std::ffi::CString::new(url_str.clone()).unwrap_or_default();
            log::info!("c_url: {:?}", c_url);
            if bindings::is_blocked(c_url.as_ptr()) {
                log::info!("Url is blocked, redirecting to vibes page: {}", url_str);
                let redirect_result = bindings::redirect_to_vibes_page();

                let blocked_app = crate::BlockedApp {
                    app_name: app_name.to_string(),
                    app_external_id: url_str.to_string(),
                    is_site: true,
                };

                let monitor_guard = MONITOR.lock().unwrap();
                if let Some(monitor) = monitor_guard.as_ref() {
                    monitor.send_app_blocked_event(BlockedAppEvent {
                        blocked_apps: vec![blocked_app],
                    });
                }

                log::info!("Redirect result: {}", redirect_result);
            }
        }
        if let Some(bundle_id) = bundle_id.clone() {
            let c_bundle_id = std::ffi::CString::new(bundle_id.clone()).unwrap_or_default();
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
                    monitor.send_window_event(WindowEvent {
                        window_title: title.to_string(),
                        app_name: app_name.to_string(),
                        url: url,
                        bundle_id: bundle_id,
                        platform: Platform::Mac,
                    });
                    log::trace!("      detect_focused_window callback Some end");
                }
                // bindings::update_typewriter_windows();
            }
            window_title_guard.title = title.to_string();
            window_title_guard.app_name = app_name.to_string();
            log::trace!("  detect_focused_window lock end");
        }
    }
}

pub trait FocusedWindowDetector {
    fn detect_focused_window(&self);
}

pub trait EventSender {
    fn send_buffered_events(&self);
    fn should_send_events(&self) -> bool;
    fn mark_events_sent(&self);
}

pub struct DefaultDependencies;

impl FocusedWindowDetector for DefaultDependencies {
    fn detect_focused_window(&self) {
        detect_focused_window()
    }
}

impl EventSender for DefaultDependencies {
    fn send_buffered_events(&self) {
        send_buffered_events()
    }

    fn should_send_events(&self) -> bool {
        let last_send = LAST_SEND.lock().unwrap();
        last_send.elapsed() >= Duration::from_secs(30)
    }

    fn mark_events_sent(&self) {
        let mut last_send = LAST_SEND.lock().unwrap();
        *last_send = Instant::now();
    }
}

pub fn platform_detect_changes() -> Result<(), crate::MonitorError> {
    platform_detect_changes_with_deps(&DefaultDependencies {})
}

pub fn platform_detect_changes_with_deps<T>(deps: &T) -> Result<(), crate::MonitorError>
where
    T: FocusedWindowDetector + EventSender,
{
    log::trace!("platform_detect_changes start");
    deps.detect_focused_window();
    log::trace!("detected focused window");

    if deps.should_send_events() {
        log::trace!("sending buffered events");
        deps.send_buffered_events();
        log::trace!("sent buffered events");
        deps.mark_events_sent();
    }

    log::trace!("platform_detect_changes end");
    Ok(())
}

pub fn platform_has_accessibility_permissions() -> bool {
    unsafe { bindings::has_accessibility_permissions() }
}

pub fn platform_request_accessibility_permissions() -> bool {
    unsafe { bindings::request_accessibility_permissions() }
}

pub fn platform_start_monitoring(monitor: Arc<Monitor>) {
    log::trace!("platform_start_monitoring start");
    {
        let mut monitor_guard = MONITOR.lock().unwrap();
        *monitor_guard = Some(monitor);
    }
    log::trace!("platform_start_monitoring end");

    unsafe {
        bindings::register_app_blocked_callback(super::blocking::app_blocked_callback);
        bindings::start_monitoring(
            super::events::mouse_event_callback,
            super::events::keyboard_event_callback,
        );
    }
    log::trace!("bindings::start_monitoring end");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    struct MockDependencies {
        focused_window_detected: Rc<Cell<bool>>,
        should_send_events: bool,
        events_sent: Rc<Cell<bool>>,
    }

    impl FocusedWindowDetector for MockDependencies {
        fn detect_focused_window(&self) {
            self.focused_window_detected.set(true);
        }
    }

    impl EventSender for MockDependencies {
        fn send_buffered_events(&self) {
            self.events_sent.set(true);
        }

        fn should_send_events(&self) -> bool {
            self.should_send_events
        }

        fn mark_events_sent(&self) {
            // Nothing to do in the mock
        }
    }

    #[test]
    fn test_platform_detect_changes() {
        // Setup
        let focused_window_detected = Rc::new(Cell::new(false));
        let events_sent = Rc::new(Cell::new(false));

        let deps = MockDependencies {
            focused_window_detected: focused_window_detected.clone(),
            should_send_events: true,
            events_sent: events_sent.clone(),
        };

        // Execute
        let result = platform_detect_changes_with_deps(&deps);

        // Verify
        assert!(result.is_ok());
        assert!(
            focused_window_detected.get(),
            "detect_focused_window should be called"
        );
        assert!(
            events_sent.get(),
            "send_buffered_events should be called when should_send_events is true"
        );

        // Test when events shouldn't be sent
        let focused_window_detected = Rc::new(Cell::new(false));
        let events_sent = Rc::new(Cell::new(false));

        let deps = MockDependencies {
            focused_window_detected: focused_window_detected.clone(),
            should_send_events: false,
            events_sent: events_sent.clone(),
        };

        let result = platform_detect_changes_with_deps(&deps);

        assert!(result.is_ok());
        assert!(
            focused_window_detected.get(),
            "detect_focused_window should always be called"
        );
        assert!(
            !events_sent.get(),
            "send_buffered_events should not be called when should_send_events is false"
        );
    }
}
