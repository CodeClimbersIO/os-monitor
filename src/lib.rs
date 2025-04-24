mod bindings;
mod blocking;
mod error;
mod event;
mod platform;

pub use blocking::BlockableItem;
pub use error::MonitorError;
pub use event::{
    BlockedApp, BlockedAppEvent, EventCallback, KeyboardEvent, Monitor, MouseEvent, MouseEventType,
    Platform, WindowEvent, WindowEventType,
};
pub use platform::{
    create_typewriter_window, detect_changes, get_application_icon_data,
    has_accessibility_permissions, remove_typewriter_window, request_accessibility_permissions,
    request_automation_permission, run_loop_cycle, start_blocking, start_monitoring, stop_blocking,
    sync_typewriter_window_order,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocking_start_stop_cycle() {
        let blocked_apps = vec![
            BlockableItem::new("facebook.com".to_string(), true),
            BlockableItem::new("twitter.com".to_string(), true),
        ];

        // Test 3 cycles of start and stop
        for i in 0..3 {
            println!("Testing cycle {}", i + 1);

            // Start blocking
            start_blocking(&blocked_apps, "https://ebb.cool/vibes", true);
            println!("Started blocking cycle {}", i + 1);

            // Give it a moment to initialize
            std::thread::sleep(std::time::Duration::from_secs(1));

            // Stop blocking
            stop_blocking();
            println!("Stopped blocking cycle {}", i + 1);

            // Give it a moment to clean up
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    #[test]
    fn test_blocking_stop_multiple_times() {
        let blocked_apps = vec![
            BlockableItem::new("facebook.com".to_string(), true),
            BlockableItem::new("twitter.com".to_string(), true),
        ];

        // Start blocking
        start_blocking(&blocked_apps, "https://ebb.cool/vibes", true);
        // Give it a moment to initialize
        std::thread::sleep(std::time::Duration::from_secs(1));
        start_blocking(&blocked_apps, "https://ebb.cool/vibes", true);
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Stop blocking
        stop_blocking();

        // Give it a moment to clean up
        std::thread::sleep(std::time::Duration::from_secs(1));
        stop_blocking();
        stop_blocking();
    }
}
