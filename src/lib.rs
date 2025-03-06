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
    detect_changes, get_application_icon_data, has_accessibility_permissions,
    request_accessibility_permissions, request_automation_permission, start_monitoring,
    start_site_blocking, stop_site_blocking,
};
