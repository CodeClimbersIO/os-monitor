use super::{BrowserClient, BrowserInfo};

pub struct FirefoxClient {
    port: u16,
}

impl FirefoxClient {
    pub fn new() -> Self {
        Self {
            port: 6000, // Default Firefox remote debugging port
        }
    }

    #[allow(dead_code)]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

impl BrowserClient for FirefoxClient {
    fn get_active_tab_url(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // TODO: Implement Firefox remote debugging protocol
        // Firefox uses a different protocol than Chrome
        log::warn!("Firefox URL extraction not yet implemented");
        Ok(None)
    }

    fn is_available(&self) -> bool {
        // TODO: Check if Firefox remote debugging is available
        log::debug!("Firefox availability check not yet implemented");
        false
    }

    fn get_browser_info(&self) -> BrowserInfo {
        BrowserInfo::new("Firefox").with_port(self.port)
    }
}

impl Default for FirefoxClient {
    fn default() -> Self {
        Self::new()
    }
}
