use std::sync::Arc;

use os_monitor::{
    create_typewriter_window, detect_changes, get_application_icon_data,
    has_accessibility_permissions, remove_typewriter_window, request_accessibility_permissions,
    run_loop_cycle, start_blocking, start_monitoring, sync_typewriter_window_order, BlockableItem,
    BlockedAppEvent, Monitor, WindowEvent,
};

fn on_keyboard_events(has_activity: bool) {
    log::warn!("Keyboard event: {}", has_activity);
}

fn on_mouse_events(has_activity: bool) {
    log::warn!("Mouse event: {}", has_activity);
}

fn on_window_event(event: WindowEvent) {
    log::warn!("Window event: {:?}", event);
    sync_typewriter_window_order()
}

fn on_app_blocked(event: BlockedAppEvent) {
    log::warn!("Apps blocked:");
    for app in &event.blocked_apps {
        log::warn!("  - {} ({})", app.app_name, app.app_external_id);
    }
}

fn main() {
    env_logger::init();
    log::trace!("main.rs starting");

    let has_permissions = has_accessibility_permissions();
    log::trace!("has_permissions: {}", has_permissions);
    if !has_permissions {
        let request_permissions = request_accessibility_permissions();
        log::trace!("request_permissions: {}", request_permissions);
    }

    // Create a grayscale effect with 0.7 opacity
    create_typewriter_window(0.5);
    run_loop_cycle();

    let icon_data = get_application_icon_data("md.obsidian");
    log::trace!("icon_data: {}", icon_data.unwrap().len());

    let monitor = Monitor::new();
    monitor.register_keyboard_callback(Box::new(on_keyboard_events));
    monitor.register_mouse_callback(Box::new(on_mouse_events));
    monitor.register_window_callback(Box::new(on_window_event));
    monitor.register_app_blocked_callback(Box::new(on_app_blocked));

    std::thread::spawn(move || {
        start_monitoring(Arc::new(monitor));
        println!("started_monitoring");
    });
    std::thread::spawn(move || {
        let blocked_apps = vec![
            BlockableItem::new("facebook.com".to_string(), true),
            BlockableItem::new("twitter.com".to_string(), true),
            BlockableItem::new("instagram.com".to_string(), true),
            BlockableItem::new("linkedin.com".to_string(), true),
            BlockableItem::new("x.com".to_string(), true),
            BlockableItem::new("com.todesktop.230313mzl4w4u92".to_string(), false),
            BlockableItem::new("com.google.Chrome".to_string(), false),
        ];

        start_blocking(&blocked_apps, "https://ebb.cool/vibes", false);
        println!("started_blocking");
    });
    std::thread::spawn(move || {
        // initialize_monitor(monitor_clone).expect("Failed to initialize monitor");
        loop {
            log::trace!("detect_changes start");
            detect_changes().expect("Failed to detect changes");
            log::trace!("detect_changes end");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    std::thread::sleep(std::time::Duration::from_secs(5));
    remove_typewriter_window();
    run_loop_cycle();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
