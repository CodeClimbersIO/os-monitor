mod bindings;
mod blocking;
mod error;
pub mod event;
mod platform;

pub use blocking::BlockableItem;
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
