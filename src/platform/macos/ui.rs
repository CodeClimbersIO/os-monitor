pub fn platform_create_typewriter_window(opacity: f64) {
    unsafe {
        crate::bindings::create_typewriter_window(opacity);
    }
}

pub fn platform_sync_typewriter_window_order() {
    unsafe {
        crate::bindings::sync_typewriter_window_order();
    }
}

pub fn platform_remove_typewriter_window() {
    unsafe {
        crate::bindings::remove_typewriter_window();
    }
}
