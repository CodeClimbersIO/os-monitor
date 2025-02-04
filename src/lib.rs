mod bindings;
mod error;
mod event;
mod platform;
mod utils;

pub use error::MonitorError;
pub use event::{
    EventCallback, KeyboardEvent, Monitor, MouseEvent, MouseEventType, Platform, WindowEvent,
    WindowEventType,
};
pub use platform::{detect_changes, initialize_monitor};
pub use utils::log::{disable_log, enable_log, log};
