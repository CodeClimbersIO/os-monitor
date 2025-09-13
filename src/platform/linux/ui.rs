// Typewriter mode UI functions - deprecated but required for platform interface

/// deprecated: no-op
pub fn platform_create_typewriter_window(_opacity: f64) {
    log::warn!("deprecated platform_create_typewriter_window called - no-op");
}

/// deprecated: no-op
pub fn platform_sync_typewriter_window_order() {
    log::trace!("deprecated platform_sync_typewriter_window_order called - no-op");
}

/// deprecated: no-op
pub fn platform_remove_typewriter_window() {
    log::warn!("deprecated platform_remove_typewriter_window called - no-op");
}
