//! App blocking implementation for Linux
//!
//! Terminates blocked applications when they gain focus.
//! Uses the focus backend abstraction to support different compositors.

use crate::blocking_state::BLOCKING_STATE;
use super::focus::FocusBackend;
use sysinfo::{System, Signal};

/// Check if an app should be blocked based on current blocking state
fn should_block_app(app_name: &str) -> bool {
    let state = match BLOCKING_STATE.lock() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if !state.active || state.blocked_apps.is_empty() {
        return false;
    }

    let app_lower = app_name.to_lowercase();

    let matches = state.blocked_apps.iter().any(|blocked| {
        let blocked_lower = blocked.to_lowercase();
        // Match by exact name or if app name contains the blocked string
        app_lower == blocked_lower || app_lower.contains(&blocked_lower)
    });

    if state.blocklist_mode {
        matches // Block if in blocklist
    } else {
        !matches // Block if NOT in allowlist
    }
}

/// Find and kill processes matching the app name
fn kill_app_processes(app_name: &str) -> bool {
    let mut system = System::new_all();
    system.refresh_processes();

    let app_lower = app_name.to_lowercase();
    let mut killed = false;

    for (pid, process) in system.processes() {
        let process_name = process.name().to_lowercase();

        // Match process name against app name
        if process_name.contains(&app_lower) || app_lower.contains(&process_name) {
            log::info!("Killing blocked app: {} (PID {})", process.name(), pid);

            if process.kill_with(Signal::Term).unwrap_or(false) {
                killed = true;
            } else {
                log::warn!("Failed to kill process {} (PID {})", process.name(), pid);
            }
        }
    }

    killed
}

/// Check the focused window and kill it if it's blocked
/// Returns true if an app was blocked and killed
pub fn check_and_block_focused_app(backend: &dyn FocusBackend) -> bool {
    let focused = match backend.get_focused_window() {
        Some(w) => w,
        None => return false,
    };

    if should_block_app(&focused.app_name) {
        log::info!("Blocking app: {} (window: {})", focused.app_name, focused.title);
        kill_app_processes(&focused.app_name)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocking_state::set_blocking_state;

    #[test]
    fn test_should_block_app_blocklist_mode() {
        set_blocking_state(
            vec![],
            vec!["discord".to_string(), "slack".to_string()],
            "https://example.com".to_string(),
            true, // blocklist mode
        );

        assert!(should_block_app("discord"));
        assert!(should_block_app("Discord")); // case insensitive
        assert!(should_block_app("slack"));
        assert!(!should_block_app("firefox"));
    }

    #[test]
    fn test_should_block_app_allowlist_mode() {
        set_blocking_state(
            vec![],
            vec!["code".to_string(), "terminal".to_string()],
            "https://example.com".to_string(),
            false, // allowlist mode
        );

        // In allowlist mode, block everything NOT in the list
        assert!(!should_block_app("code")); // allowed
        assert!(!should_block_app("terminal")); // allowed
        assert!(should_block_app("discord")); // blocked
    }
}
