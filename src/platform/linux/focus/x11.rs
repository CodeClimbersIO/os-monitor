//! X11 focus detection backend
//!
//! Works on any X11 session and via XWayland on Wayland compositors.

use super::{FocusBackend, FocusedWindow};
use std::ffi::CStr;
use std::ptr;
use x11::xlib::*;

pub struct X11FocusBackend {
    display: *mut Display,
}

// SAFETY: X11 display can be used from multiple threads if properly synchronized.
// Our usage is single-threaded within the monitoring loop.
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
