use os_monitor::{
    detect_changes, get_application_icon_data, has_accessibility_permissions,
    request_accessibility_permissions, start_monitoring,
};

fn main() {
    env_logger::init();
    log::info!("main.rs starting");

    let has_permissions = has_accessibility_permissions();
    println!("has_permissions: {}", has_permissions);
    if !has_permissions {
        let request_permissions = request_accessibility_permissions();
        println!("request_permissions: {}", request_permissions);
    }

    let icon_data = get_application_icon_data("md.obsidian");
    println!("icon_data: {}", icon_data.unwrap());

    std::thread::spawn(move || {
        start_monitoring();
    });
    std::thread::spawn(move || {
        // initialize_monitor(monitor_clone).expect("Failed to initialize monitor");
        loop {
            detect_changes().expect("Failed to detect changes");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
