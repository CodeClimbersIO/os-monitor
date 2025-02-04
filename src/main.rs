use std::sync::Arc;

use os_monitor::{
    detect_changes, enable_log, initialize_monitor, log, KeyboardEvent, Monitor, MonitorError,
    MouseEvent, WindowEvent,
};

fn on_keyboard_events(events: Vec<KeyboardEvent>) {
    log(&format!("Keyboard event: {:?}", events));
}

fn on_mouse_events(events: Vec<MouseEvent>) {
    log(&format!("Mouse event: {:?}", events));
}

fn on_window_event(event: WindowEvent) {
    log(&format!("Window event: {:?}", event));
}

fn main() -> Result<(), MonitorError> {
    enable_log();
    log("main.rs starting");

    let monitor = Monitor::new();

    monitor.register_keyboard_callback(Box::new(on_keyboard_events));
    monitor.register_mouse_callback(Box::new(on_mouse_events));
    monitor.register_window_callback(Box::new(on_window_event));

    initialize_monitor(Arc::new(monitor)).expect("Failed to initialize monitor");
    loop {
        detect_changes().expect("Failed to detect changes");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
