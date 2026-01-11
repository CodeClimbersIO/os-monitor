mod bindings;
mod blocking;
pub mod blocking_state;
pub mod browser;
mod error;
pub mod event;
mod platform;

pub use blocking::BlockableItem;
pub use browser::{
    start_browser_monitoring, handle_url_change, block_website, 
    update_blocking_rules, get_active_tab_info, is_browser_connected,
    ActiveTabInfo, BlockingRule, BrowserError
};
pub use error::MonitorError;
pub use event::{
    AppEvent, BlockedApp, BlockedAppEvent, KeyboardEvent, Monitor, MouseEvent, MouseEventType,
    Platform, WindowEvent, WindowEventType,
};
pub use platform::{
    create_typewriter_window, detect_changes, get_application_icon_data,
    has_accessibility_permissions, remove_typewriter_window, request_accessibility_permissions,
    run_loop_cycle, start_blocking, start_monitoring, stop_blocking, sync_typewriter_window_order,
};
