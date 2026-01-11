use std::sync::Arc;
use std::path::Path;

use os_monitor::{
    create_typewriter_window, detect_changes, get_application_icon_data,
    has_accessibility_permissions, remove_typewriter_window, request_accessibility_permissions,
    run_loop_cycle, start_blocking, start_monitoring, start_browser_monitoring, sync_typewriter_window_order, AppEvent,
    BlockableItem, Monitor, is_browser_connected,
};

/// Load blocklist from JSON config file in repo root
/// Falls back to defaults if file doesn't exist or is invalid
fn load_blocklist_config() -> Vec<BlockableItem> {
    let config_path = "blocklist.json";

    if Path::new(config_path).exists() {
        match std::fs::read_to_string(config_path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<BlockableItem>>(&content) {
                    Ok(items) => {
                        log::info!("Loaded {} items from {}", items.len(), config_path);
                        return items;
                    }
                    Err(e) => {
                        log::error!("Failed to parse {}: {}", config_path, e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read {}: {}", config_path, e);
            }
        }
    }

    // Fallback to default test items
    log::info!("No blocklist.json found, using default test blocklist");
    vec![
        BlockableItem::new("facebook.com".to_string(), true),
        BlockableItem::new("twitter.com".to_string(), true),
        BlockableItem::new("instagram.com".to_string(), true),
        BlockableItem::new("x.com".to_string(), true),
    ]
}

#[tokio::main]
async fn main() {
    env_logger::init();
    log::trace!("main.rs starting");

    // No longer need native messaging host mode - WebSocket server runs alongside main app

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
                AppEvent::Mouse(_has_activity) => {
                    // log::warn!("Mouse event: {}", has_activity);
                }
                AppEvent::Keyboard(_has_activity) => {
                    // log::warn!("Keyboard event: {}", has_activity);
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

    // Start browser monitoring
    tokio::spawn(async move {
        println!("🚀 Starting browser monitoring...");
        loop {
            match start_browser_monitoring().await {
                Ok(port) => {
                    println!("✅ Browser monitoring started on port {}", port);
                    
                    // Wait for browser extension to connect properly
                    println!("⏳ Waiting for browser extension to connect...");
                    let mut attempts = 0;
                    loop {
                        if is_browser_connected().await {
                            println!("✅ Browser extension connected and ready");
                            break;
                        }
                        attempts += 1;
                        if attempts > 30 {
                            println!("ℹ️  Browser extension not connected yet (will accept connections when available)");
                            break; // Don't return, just break to retry loop
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                    
                    println!("📡 Listening for URL changes from browser extension...");
                    // Keep the task alive to maintain the WebSocket server
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
                Err(e) => {
                    println!("❌ Failed to start browser monitoring: {}", e);
                    println!("🔄 Retrying in 5 seconds...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    // Start blocking in a separate thread
    std::thread::spawn(move || {
        let blocked_apps = load_blocklist_config();
        let redirect_url = "https://ebb.cool/vibes";
        let blocklist_mode = true; // true = block items in list, false = allow only items in list

        println!("Starting blocking with {} items", blocked_apps.len());
        for item in &blocked_apps {
            println!("  - {} ({})", item.app_external_id, if item.is_browser { "website" } else { "app" });
        }

        start_blocking(&blocked_apps, redirect_url, blocklist_mode);
        println!("Blocking started");
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
