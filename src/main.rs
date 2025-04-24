use std::sync::Arc;

use os_monitor::{
    create_typewriter_window, detect_changes, get_application_icon_data,
    has_accessibility_permissions, remove_typewriter_window, request_accessibility_permissions,
    run_loop_cycle, start_blocking, start_monitoring, sync_typewriter_window_order, AppEvent,
    BlockableItem, Monitor,
};

fn main() {
    env_logger::init();
    log::trace!("main.rs starting");

    let has_permissions = has_accessibility_permissions();
    log::trace!("has_permissions: {}", has_permissions);
    if !has_permissions {
        let request_permissions = request_accessibility_permissions();
        log::trace!("request_permissions: {}", request_permissions);
    }

    create_typewriter_window(0.5);
    run_loop_cycle();

    let icon_data = get_application_icon_data("md.obsidian");
    if let Some(data) = icon_data {
        log::trace!("icon_data: {}", data.len());
    }

    let monitor = Monitor::new();

    let mut main_receiver = monitor.subscribe();

    let monitor_arc = Arc::new(monitor);
    std::thread::spawn(move || {
        start_monitoring(monitor_arc);
        println!("started_monitoring");
    });

    std::thread::spawn(move || {
        println!("Main event processor thread started");
        while let Ok(event) = main_receiver.blocking_recv() {
            match event {
                AppEvent::Mouse(has_activity) => {
                    log::warn!("Mouse event: {}", has_activity);
                }
                AppEvent::Keyboard(has_activity) => {
                    log::warn!("Keyboard event: {}", has_activity);
                }
                AppEvent::Window(event) => {
                    log::warn!("Window event: {:?}", event);
                    sync_typewriter_window_order();
                }
                AppEvent::AppBlocked(event) => {
                    log::warn!("Apps blocked:");
                    for app in &event.blocked_apps {
                        log::warn!("  - {} ({})", app.app_name, app.app_external_id);
                    }
                }
            }
        }
        log::warn!("Main event receiver channel closed");
    });

    // Start blocking in a separate thread
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

    // Detect changes thread
    std::thread::spawn(move || loop {
        log::trace!("detect_changes start");
        detect_changes().expect("Failed to detect changes");
        log::trace!("detect_changes end");
        std::thread::sleep(std::time::Duration::from_secs(1));
    });

    std::thread::sleep(std::time::Duration::from_secs(5));
    remove_typewriter_window();
    run_loop_cycle();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
