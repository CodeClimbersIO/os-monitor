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
    detect_changes, get_application_icon_path, has_accessibility_permissions, initialize_monitor,
    request_accessibility_permissions,
};
