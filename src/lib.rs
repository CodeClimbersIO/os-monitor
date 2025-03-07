mod bindings;
mod error;
mod event;
mod platform;

pub use error::MonitorError;
pub use event::{
    EventCallback, KeyboardEvent, Monitor, MouseEvent, MouseEventType, Platform, WindowEvent,
    WindowEventType,
};
pub use platform::{
    create_screen_border, detect_changes, get_application_icon_data, has_accessibility_permissions,
    remove_screen_border, request_accessibility_permissions, request_automation_permission,
    run_loop_cycle, start_blocking, start_monitoring, stop_blocking,
};
