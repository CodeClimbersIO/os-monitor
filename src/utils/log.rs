use std::sync::Mutex;

use once_cell::sync::Lazy;

static IS_LOG_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub fn log(message: &str) {
    if *IS_LOG_ENABLED.lock().unwrap() {
        cc_logger::log(message);
    }
}

pub fn enable_log() {
    *IS_LOG_ENABLED.lock().unwrap() = true;
}

pub fn disable_log() {
    *IS_LOG_ENABLED.lock().unwrap() = false;
}
