use super::monitoring::MONITOR;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::Instant;

// Global state for tracking activity
pub static HAS_MOUSE_ACTIVITY: Mutex<bool> = Mutex::new(false);
pub static HAS_KEYBOARD_ACTIVITY: Mutex<bool> = Mutex::new(false);
pub static LAST_SEND: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

// Event callbacks
pub extern "C" fn mouse_event_callback(_: f64, _: f64, _: i32, _: i32) {
    let mut has_activity = HAS_MOUSE_ACTIVITY.lock().unwrap();
    *has_activity = true;
}

pub extern "C" fn keyboard_event_callback(_: i32) {
    let mut has_activity = HAS_KEYBOARD_ACTIVITY.lock().unwrap();
    *has_activity = true;
}

pub fn send_buffered_events() {
    let monitor_guard = MONITOR.lock().unwrap();
    if let Some(monitor) = monitor_guard.as_ref() {
        {
            let mut has_activity = HAS_KEYBOARD_ACTIVITY.lock().unwrap();
            monitor.send_keyboard_event(has_activity.clone());
            *has_activity = false;
        }

        {
            let mut has_activity = HAS_MOUSE_ACTIVITY.lock().unwrap();
            monitor.send_mouse_event(has_activity.clone());
            *has_activity = false;
        }
    }
}

pub fn platform_run_loop_cycle() {
    unsafe {
        crate::bindings::run_loop_cycle();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_event_callback() {
        {
            let mut has_activity = HAS_MOUSE_ACTIVITY.lock().unwrap();
            *has_activity = false;
        }

        mouse_event_callback(0.0, 0.0, 0, 0);

        {
            let has_activity = HAS_MOUSE_ACTIVITY.lock().unwrap();
            assert!(*has_activity, "Mouse activity should be set to true");
        }
    }

    #[test]
    fn test_keyboard_event_callback() {
        // Ensure activity is initially false
        {
            let mut has_activity = HAS_KEYBOARD_ACTIVITY.lock().unwrap();
            *has_activity = false;
        }

        // Call the callback
        keyboard_event_callback(0);

        // Verify activity is now true
        {
            let has_activity = HAS_KEYBOARD_ACTIVITY.lock().unwrap();
            assert!(*has_activity, "Keyboard activity should be set to true");
        }
    }
}
