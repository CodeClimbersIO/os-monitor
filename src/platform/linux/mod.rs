mod blocking;
mod events;
mod monitoring;

pub use blocking::{
    platform_get_application_icon_data, platform_start_blocking, platform_stop_blocking,
};
pub use events::platform_run_loop_cycle;
pub use monitoring::{
    platform_detect_changes, platform_has_accessibility_permissions,
    platform_request_accessibility_permissions, platform_start_monitoring,
};
