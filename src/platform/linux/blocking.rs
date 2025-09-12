use crate::BlockableItem;

pub fn platform_start_blocking(
    _blocked_apps: &mut Vec<BlockableItem>,
    _redirect_url: &str,
    _blocklist_mode: bool,
) -> bool {
    // TODO: Implement application and website blocking for Linux
    // - Set up process monitoring for blocked applications
    // - Configure browser blocking (Chrome DevTools Protocol)
    // - Store blocking configuration
    log::warn!("platform_start_blocking not yet implemented for Linux");
    true // Assume success for now
}

pub fn platform_stop_blocking() {
    // TODO: Disable all blocking functionality
    log::warn!("platform_stop_blocking not yet implemented for Linux");
}

pub fn platform_get_application_icon_data(_bundle_id: &str) -> Option<String> {
    // TODO: Extract application icons from .desktop files or app directories
    // Return base64 encoded icon data
    log::warn!("platform_get_application_icon_data not yet implemented for Linux");
    None
}