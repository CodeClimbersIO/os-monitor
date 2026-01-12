//! Focus detection backends for different Linux display servers/compositors
//!
//! This module abstracts window focus detection so we can support multiple
//! backends (X11, Hyprland, etc.) without changing the rest of the codebase.

use std::env;

/// Information about the currently focused window
#[derive(Debug, Clone)]
pub struct FocusedWindow {
    /// Window/surface identifier (X11 window ID, Hyprland address, etc.)
    pub id: u64,
    /// Application name/class (used for blocking)
    pub app_name: String,
    /// Window title
    pub title: String,
}

/// Trait for focus detection backends
pub trait FocusBackend: Send + Sync {
    /// Get the currently focused window, if any
    fn get_focused_window(&self) -> Option<FocusedWindow>;

    /// Backend name for logging
    fn name(&self) -> &'static str;
}

/// Detected display server/compositor type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    X11,
    Hyprland,
    // Future: Sway, GNOME Wayland, KDE Wayland, etc.
    Unknown,
}

/// Detect which display server/compositor is running
pub fn detect_display_server() -> DisplayServer {
    // Check for Hyprland first (it also sets WAYLAND_DISPLAY)
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return DisplayServer::Hyprland;
    }

    // Check for X11
    if env::var("DISPLAY").is_ok() && env::var("WAYLAND_DISPLAY").is_err() {
        return DisplayServer::X11;
    }

    // XWayland case: both DISPLAY and WAYLAND_DISPLAY are set
    // For now, prefer the native Wayland compositor if we support it
    if env::var("WAYLAND_DISPLAY").is_ok() {
        // We don't have a generic Wayland backend yet, fall back to X11 via XWayland
        if env::var("DISPLAY").is_ok() {
            log::info!("Wayland detected but no native backend, using XWayland");
            return DisplayServer::X11;
        }
    }

    DisplayServer::Unknown
}

/// Create the appropriate focus backend for the current environment
pub fn create_focus_backend() -> Option<Box<dyn FocusBackend>> {
    let display_server = detect_display_server();
    log::info!("Detected display server: {:?}", display_server);

    match display_server {
        DisplayServer::X11 => {
            Some(Box::new(x11::X11FocusBackend::new()?))
        }
        DisplayServer::Hyprland => {
            // TODO: Implement Hyprland backend
            log::warn!("Hyprland backend not yet implemented, falling back to X11 via XWayland");
            Some(Box::new(x11::X11FocusBackend::new()?))
        }
        DisplayServer::Unknown => {
            log::error!("Could not detect display server");
            None
        }
    }
}

pub mod x11 {
    use super::{FocusBackend, FocusedWindow};
    use std::ffi::CStr;
    use std::ptr;
    use x11::xlib::*;

    pub struct X11FocusBackend {
        display: *mut Display,
    }

    // SAFETY: X11 display can be used from multiple threads if properly synchronized
    // Our usage is single-threaded within the monitoring loop
    unsafe impl Send for X11FocusBackend {}
    unsafe impl Sync for X11FocusBackend {}

    impl X11FocusBackend {
        pub fn new() -> Option<Self> {
            let display = unsafe { XOpenDisplay(ptr::null()) };
            if display.is_null() {
                log::error!("Failed to open X11 display");
                return None;
            }
            Some(Self { display })
        }

        unsafe fn get_window_class(&self, window: Window) -> Option<String> {
            let mut class_hint = std::mem::zeroed::<XClassHint>();
            let status = XGetClassHint(self.display, window, &mut class_hint);

            if status == 0 || class_hint.res_class.is_null() {
                return None;
            }

            let class_name = CStr::from_ptr(class_hint.res_class)
                .to_string_lossy()
                .to_string();

            XFree(class_hint.res_class as *mut _);
            if !class_hint.res_name.is_null() {
                XFree(class_hint.res_name as *mut _);
            }

            Some(class_name)
        }

        unsafe fn get_window_title(&self, window: Window) -> Option<String> {
            let mut window_name: *mut i8 = ptr::null_mut();
            let status = XFetchName(self.display, window, &mut window_name);

            if status != 0 && !window_name.is_null() {
                let title = CStr::from_ptr(window_name)
                    .to_string_lossy()
                    .to_string();
                XFree(window_name as *mut _);

                if !title.is_empty() {
                    return Some(title);
                }
            }

            None
        }

        unsafe fn get_parent_window_title(&self, window: Window) -> Option<String> {
            let mut root: Window = 0;
            let mut parent: Window = 0;
            let mut children: *mut Window = ptr::null_mut();
            let mut nchildren: u32 = 0;

            let status = XQueryTree(
                self.display,
                window,
                &mut root,
                &mut parent,
                &mut children,
                &mut nchildren,
            );

            if !children.is_null() {
                XFree(children as *mut _);
            }

            if status != 0 && parent != root && parent != 0 {
                return self.get_window_title(parent);
            }

            None
        }
    }

    impl Drop for X11FocusBackend {
        fn drop(&mut self) {
            if !self.display.is_null() {
                unsafe { XCloseDisplay(self.display) };
            }
        }
    }

    impl FocusBackend for X11FocusBackend {
        fn get_focused_window(&self) -> Option<FocusedWindow> {
            unsafe {
                let mut focus_window: Window = 0;
                let mut revert_to: i32 = 0;

                XGetInputFocus(self.display, &mut focus_window, &mut revert_to);

                if focus_window == 0 {
                    return None;
                }

                let app_name = self.get_window_class(focus_window)
                    .or_else(|| self.get_window_title(focus_window))
                    .or_else(|| self.get_parent_window_title(focus_window))
                    .unwrap_or_else(|| "Unknown".to_string());

                let title = self.get_window_title(focus_window)
                    .unwrap_or_default();

                Some(FocusedWindow {
                    id: focus_window,
                    app_name,
                    title,
                })
            }
        }

        fn name(&self) -> &'static str {
            "X11"
        }
    }
}
