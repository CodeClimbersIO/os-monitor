//! Browser integration module for Ebb OS Monitor
//! 
//! This module provides a platform-agnostic interface for communicating with
//! the Ebb browser extension via WebSocket. It handles:
//! - WebSocket server for browser communication
//! - URL monitoring and reporting
//! - Website blocking commands
//! - Real-time bidirectional communication

use std::sync::{Arc, Mutex};
use std::io;
use serde::{Deserialize, Serialize};

pub mod websocket_server;

// Re-export for convenience
pub use websocket_server::{start_websocket_server, get_websocket_port, broadcast_to_extensions, HostMessage, BlockingRule};

/// Represents browser tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTabInfo {
    pub url: String,
    pub title: String,
    pub tab_id: u32,
    pub timestamp: u64,
}

// BlockingRule is now defined in websocket_server.rs and re-exported

/// Browser interface error types
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("WebSocket connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Failed to send message to browser: {0}")]
    SendFailed(String),
    
    #[error("No browser extensions connected")]
    NoExtensionsConnected,
    
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    #[error("Browser operation timeout")]
    Timeout,
    
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Global browser state
static BROWSER_STATE: once_cell::sync::Lazy<Arc<Mutex<BrowserState>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(BrowserState::new())));

/// Browser state for OS monitor integration
#[derive(Debug)]
pub struct BrowserState {
    /// Current active tab information
    current_tab: Option<ActiveTabInfo>,
    /// Active blocking rules
    blocking_rules: std::collections::HashMap<u32, BlockingRule>,
}

impl BrowserState {
    /// Create a new browser state
    pub fn new() -> Self {
        Self {
            current_tab: None,
            blocking_rules: std::collections::HashMap::new(),
        }
    }
    
    /// Update current active tab information
    pub fn update_active_tab(&mut self, tab_info: ActiveTabInfo) {
        self.current_tab = Some(tab_info);
    }
    
    /// Get current active tab information
    pub fn get_active_tab(&self) -> Option<&ActiveTabInfo> {
        self.current_tab.as_ref()
    }
    
    /// Add or update a blocking rule
    pub fn add_blocking_rule(&mut self, rule: BlockingRule) {
        self.blocking_rules.insert(rule.id, rule);
    }
    
    /// Remove a blocking rule
    pub fn remove_blocking_rule(&mut self, rule_id: u32) {
        self.blocking_rules.remove(&rule_id);
    }
    
    /// Get all blocking rules
    pub fn get_all_rules(&self) -> Vec<&BlockingRule> {
        self.blocking_rules.values().collect()
    }
}

// Platform-agnostic browser interface functions

/// Initialize browser monitoring with WebSocket server
pub async fn start_browser_monitoring() -> Result<u16, BrowserError> {
    println!("🚀 Starting Ebb browser monitoring...");
    
    // Start WebSocket server and return the port it's running on
    let port = start_websocket_server().await?;
    println!("✅ Browser monitoring ready on port {}", port);
    
    Ok(port)
}

/// Handle URL change from browser extension
pub async fn handle_url_change(url: &str, title: &str, tab_id: u32) -> Result<(), BrowserError> {
    use crate::blocking_state::{should_block_domain, get_redirect_url, is_blocking_active};

    let tab_info = ActiveTabInfo {
        url: url.to_string(),
        title: title.to_string(),
        tab_id,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };

    // Update global state
    {
        let mut state = BROWSER_STATE.lock()
            .map_err(|e| BrowserError::ConnectionFailed(format!("Mutex poisoned: {}", e)))?;
        state.update_active_tab(tab_info);
    }

    // Check if URL should be blocked using dynamic blocklist
    if !is_blocking_active() {
        return Ok(());
    }

    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(domain) = parsed_url.domain() {
            if should_block_domain(domain) {
                let redirect_url = get_redirect_url();
                log::info!("Blocked domain detected: {} -> redirecting to {}", domain, redirect_url);

                // Send immediate redirect for current tab
                // The extension is a pure interface - no persistent blocking rules
                if let Err(e) = redirect_tab(tab_id, &redirect_url).await {
                    log::error!("Failed to redirect tab: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Block a website with optional redirect
pub async fn block_website(url: &str, redirect_to: Option<&str>) -> Result<(), BrowserError> {
    let redirect_url = redirect_to.unwrap_or("about:blank");
    
    let message = HostMessage::BlockUrl {
        url: url.to_string(),
        redirect_to: redirect_url.to_string(),
    };
    
    broadcast_to_extensions(message).await?;
    println!("🚫 Blocked website: {} -> {}", url, redirect_url);
    
    Ok(())
}

/// Redirect a specific tab immediately
pub async fn redirect_tab(tab_id: u32, redirect_to: &str) -> Result<(), BrowserError> {
    let message = HostMessage::RedirectTab {
        tab_id,
        redirect_to: redirect_to.to_string(),
    };
    
    broadcast_to_extensions(message).await?;
    println!("🔄 Redirected tab {} to: {}", tab_id, redirect_to);
    
    Ok(())
}

/// Update blocking rules
pub async fn update_blocking_rules(rules: &[BlockingRule]) -> Result<(), BrowserError> {
    // Update global state
    {
        let mut state = BROWSER_STATE.lock()
            .map_err(|e| BrowserError::ConnectionFailed(format!("Mutex poisoned: {}", e)))?;
            
        // Clear existing rules and add new ones
        state.blocking_rules.clear();
        
        for rule in rules {
            state.blocking_rules.insert(rule.id, rule.clone());
        }
    }
    
    // Send update to all connected extensions
    let message = HostMessage::UpdateRules {
        rules: rules.to_vec(),
    };
    
    broadcast_to_extensions(message).await?;
    println!("📋 Updated {} blocking rules", rules.len());
    
    Ok(())
}

/// Get current active tab information
pub async fn get_active_tab_info() -> Result<Option<ActiveTabInfo>, BrowserError> {
    let state = BROWSER_STATE.lock()
        .map_err(|e| BrowserError::ConnectionFailed(format!("Mutex poisoned: {}", e)))?;
        
    Ok(state.get_active_tab().cloned())
}

/// Check if browser extensions are connected
pub async fn is_browser_connected() -> bool {
    use crate::browser::websocket_server::{WS_SERVER, CONNECTION_COUNT};
    
    // Check if WebSocket server is running and has active connections
    if let Ok(global_sender) = WS_SERVER.lock() {
        if global_sender.is_some() {
            // Check if we have active connections
            if let Ok(count) = CONNECTION_COUNT.lock() {
                return *count > 0;
            }
        }
    }
    false
}