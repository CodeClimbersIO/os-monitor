use super::monitoring::MONITOR;
use crate::{BlockableItem, BlockedApp, BlockedAppEvent};
use std::ffi::{c_char, CStr, CString};

pub extern "C" fn app_blocked_callback(
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
            monitor.send_app_blocked_event(BlockedAppEvent {
                blocked_apps: blocked_apps,
            });
        }
    }
}

fn has_website_url(blocked_apps: &Vec<BlockableItem>) -> bool {
    blocked_apps.iter().any(|app| app.is_browser)
}

fn get_system_exceptions() -> Vec<BlockableItem> {
    vec![
        BlockableItem::new("com.apple.SystemFinder".to_string(), false),
        BlockableItem::new("com.spotify.client".to_string(), false),
        BlockableItem::new("com.apple.ActivityMonitor".to_string(), false),
        BlockableItem::new("com.apple.SystemPreferences".to_string(), false),
        BlockableItem::new("com.apple.finder".to_string(), false),
        BlockableItem::new("com.apple.Terminal".to_string(), false),
        BlockableItem::new("com.apple.Preview".to_string(), false),
        BlockableItem::new("com.apple.Music".to_string(), false),
        BlockableItem::new("com.nordvpn.macos".to_string(), false),
        BlockableItem::new("ebb.cool".to_string(), true),
        BlockableItem::new("com.ebb.app".to_string(), true),
    ]
}

fn get_browser_exceptions() -> Vec<BlockableItem> {
    vec![
        BlockableItem::new("com.google.Chrome".to_string(), false),
        BlockableItem::new("com.google.Chrome.beta".to_string(), false),
        BlockableItem::new("com.google.Chrome.dev".to_string(), false),
        BlockableItem::new("com.google.Chrome.canary".to_string(), false),
        BlockableItem::new("com.apple.Safari".to_string(), false),
        BlockableItem::new("com.microsoft.Edge".to_string(), false),
        BlockableItem::new("com.brave.Browser".to_string(), false),
        BlockableItem::new("company.thebrowser.Browser".to_string(), false),
    ]
}

fn get_exceptions(has_website_url: bool, blocklist_mode: bool) -> Vec<BlockableItem> {
    let mut exceptions = Vec::new();
    if blocklist_mode {
        return exceptions;
    }
    if has_website_url {
        exceptions.extend(get_browser_exceptions());
    }
    exceptions.extend(get_system_exceptions());
    exceptions
}

pub fn platform_start_blocking(
    blocked_apps: &Vec<BlockableItem>,
    redirect_url: &str,
    blocklist_mode: bool,
) -> bool {
    let mut all_items = blocked_apps.to_vec();

    let has_website_url = has_website_url(blocked_apps);
    let exceptions = get_exceptions(has_website_url, blocklist_mode);
    all_items.extend(exceptions);

    let c_urls: Vec<CString> = all_items
        .iter()
        .map(|app| CString::new(app.app_external_id.as_str()).unwrap())
        .collect();

    let c_urls_ptrs: Vec<*const c_char> = c_urls.iter().map(|url| url.as_ptr()).collect();

    let c_redirect_url = CString::new(redirect_url).unwrap();

    log::trace!("platform_start_blocking bindings call");
    unsafe {
        crate::bindings::start_blocking(
            c_urls_ptrs.as_ptr(),
            c_urls_ptrs.len() as i32,
            c_redirect_url.as_ptr(),
            blocklist_mode,
        )
    }
}

pub fn platform_stop_blocking() {
    unsafe {
        crate::bindings::stop_blocking();
    }
}

pub fn platform_get_application_icon_data(bundle_id: &str) -> Option<String> {
    unsafe {
        let c_bundle_id = CString::new(bundle_id).ok()?;
        let c_data = crate::bindings::get_app_icon_data(c_bundle_id.as_ptr());
        if c_data.is_null() {
            return None;
        }

        let data = CStr::from_ptr(c_data).to_str().ok()?.to_owned();
        crate::bindings::free_icon_data(c_data);
        Some(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_website_url() {
        let blocked_apps = vec![
            BlockableItem::new("com.example.app".to_string(), false),
            BlockableItem::new("google.com".to_string(), true),
        ];
        assert!(has_website_url(&blocked_apps));
    }

    #[test]
    fn test_has_website_url_false() {
        let blocked_apps = vec![
            BlockableItem::new("com.example.app".to_string(), false),
            BlockableItem::new("google.com".to_string(), false),
        ];
        assert!(!has_website_url(&blocked_apps));
    }

    #[test]
    fn test_get_browser_exceptions() {
        let exceptions = get_browser_exceptions();
        let contains_chrome = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.google.Chrome");
        assert!(contains_chrome);
    }

    #[test]
    fn test_get_system_exceptions() {
        let exceptions = get_system_exceptions();
        let contains_finder = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.apple.SystemFinder");
        assert!(contains_finder);
    }

    #[test]
    fn test_get_exceptions_allowlist_mode_with_browser() {
        let exceptions = get_exceptions(true, false);
        let contains_chrome = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.google.Chrome");
        let contains_finder = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.apple.SystemFinder");
        assert!(contains_chrome);
        assert!(contains_finder);
    }

    #[test]
    fn test_get_exceptions_allowlist_mode_no_browser() {
        let exceptions = get_exceptions(false, false);
        let contains_chrome = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.google.Chrome");
        let contains_finder = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.apple.SystemFinder");
        assert!(!contains_chrome);
        assert!(contains_finder);
    }

    #[test]
    fn test_get_exceptions_blocklist_mode_with_browser() {
        let exceptions = get_exceptions(true, true);
        let contains_chrome = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.google.Chrome");
        let contains_finder = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.apple.SystemFinder");
        assert!(!contains_chrome);
        assert!(!contains_finder);
    }

    #[test]
    fn test_get_exceptions_blocklist_mode_no_browser() {
        let exceptions = get_exceptions(false, true);
        let contains_chrome = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.google.Chrome");
        let contains_finder = exceptions
            .iter()
            .any(|app| app.app_external_id == "com.apple.SystemFinder");
        assert!(!contains_chrome);
        assert!(!contains_finder);
    }
}
