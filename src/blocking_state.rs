//! Shared blocking state module
//!
//! This module provides a thread-safe global state for blocking configuration
//! that can be accessed from both platform-specific blocking code and the
//! browser module.

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Global blocking state accessible from any module
pub static BLOCKING_STATE: Lazy<Arc<Mutex<BlockingState>>> =
    Lazy::new(|| Arc::new(Mutex::new(BlockingState::default())));

/// Blocking configuration state
#[derive(Default, Clone, Debug)]
pub struct BlockingState {
    /// Domains to block (from items with is_browser=true)
    pub blocked_websites: Vec<String>,
    /// App IDs to block (from items with is_browser=false)
    pub blocked_apps: Vec<String>,
    /// URL to redirect blocked sites to
    pub redirect_url: String,
    /// true = blocklist mode (block if in list), false = allowlist mode (block if NOT in list)
    pub blocklist_mode: bool,
    /// Whether blocking is currently active
    pub active: bool,
}

impl BlockingState {
    /// Check if a domain should be blocked based on current configuration
    pub fn should_block_domain(&self, domain: &str) -> bool {
        if !self.active {
            return false;
        }

        let domain_lower = domain.to_lowercase();
        let matches = self.blocked_websites.iter().any(|blocked| {
            let blocked_lower = blocked.to_lowercase();
            // Match exact domain or subdomain (e.g., "www.facebook.com" matches "facebook.com")
            domain_lower == blocked_lower || domain_lower.ends_with(&format!(".{}", blocked_lower))
        });

        if self.blocklist_mode {
            matches // Block if in list
        } else {
            !matches // Block if NOT in list (allowlist mode)
        }
    }
}

/// Set the blocking state with new configuration
pub fn set_blocking_state(
    websites: Vec<String>,
    apps: Vec<String>,
    redirect_url: String,
    blocklist_mode: bool,
) {
    if let Ok(mut state) = BLOCKING_STATE.lock() {
        state.blocked_websites = websites;
        state.blocked_apps = apps;
        state.redirect_url = redirect_url;
        state.blocklist_mode = blocklist_mode;
        state.active = true;
        log::debug!("Blocking state updated: {:?}", *state);
    }
}

/// Clear the blocking state (stop blocking)
pub fn clear_blocking_state() {
    if let Ok(mut state) = BLOCKING_STATE.lock() {
        state.active = false;
        state.blocked_websites.clear();
        state.blocked_apps.clear();
        log::debug!("Blocking state cleared");
    }
}

/// Get the current redirect URL
pub fn get_redirect_url() -> String {
    BLOCKING_STATE
        .lock()
        .map(|s| s.redirect_url.clone())
        .unwrap_or_else(|_| "https://ebb.cool/vibes".to_string())
}

/// Check if a domain should be blocked (convenience function)
pub fn should_block_domain(domain: &str) -> bool {
    BLOCKING_STATE
        .lock()
        .map(|s| s.should_block_domain(domain))
        .unwrap_or(false)
}

/// Check if blocking is currently active
pub fn is_blocking_active() -> bool {
    BLOCKING_STATE
        .lock()
        .map(|s| s.active)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_block_exact_domain() {
        let state = BlockingState {
            blocked_websites: vec!["facebook.com".to_string()],
            blocked_apps: vec![],
            redirect_url: "https://example.com".to_string(),
            blocklist_mode: true,
            active: true,
        };

        assert!(state.should_block_domain("facebook.com"));
        assert!(!state.should_block_domain("twitter.com"));
    }

    #[test]
    fn test_should_block_subdomain() {
        let state = BlockingState {
            blocked_websites: vec!["facebook.com".to_string()],
            blocked_apps: vec![],
            redirect_url: "https://example.com".to_string(),
            blocklist_mode: true,
            active: true,
        };

        assert!(state.should_block_domain("www.facebook.com"));
        assert!(state.should_block_domain("m.facebook.com"));
        assert!(!state.should_block_domain("notfacebook.com"));
    }

    #[test]
    fn test_allowlist_mode() {
        let state = BlockingState {
            blocked_websites: vec!["allowed.com".to_string()],
            blocked_apps: vec![],
            redirect_url: "https://example.com".to_string(),
            blocklist_mode: false, // allowlist mode
            active: true,
        };

        // In allowlist mode, block everything NOT in the list
        assert!(!state.should_block_domain("allowed.com")); // allowed
        assert!(state.should_block_domain("blocked.com")); // blocked
    }

    #[test]
    fn test_inactive_state() {
        let state = BlockingState {
            blocked_websites: vec!["facebook.com".to_string()],
            blocked_apps: vec![],
            redirect_url: "https://example.com".to_string(),
            blocklist_mode: true,
            active: false,
        };

        // Nothing should be blocked when inactive
        assert!(!state.should_block_domain("facebook.com"));
    }

    #[test]
    fn test_case_insensitive() {
        let state = BlockingState {
            blocked_websites: vec!["Facebook.COM".to_string()],
            blocked_apps: vec![],
            redirect_url: "https://example.com".to_string(),
            blocklist_mode: true,
            active: true,
        };

        assert!(state.should_block_domain("facebook.com"));
        assert!(state.should_block_domain("FACEBOOK.COM"));
        assert!(state.should_block_domain("www.Facebook.com"));
    }
}
