pub mod chrome;
pub mod firefox;
pub mod common;

pub use chrome::ChromeClient;
pub use firefox::FirefoxClient;
pub use common::BrowserInfo;

/// Common interface for all browser clients
pub trait BrowserClient {
    /// Get the URL of the currently active tab
    fn get_active_tab_url(&self) -> Result<Option<String>, Box<dyn std::error::Error>>;
    
    /// Check if the browser is reachable
    fn is_available(&self) -> bool;
    
    /// Get browser information
    #[allow(dead_code)]
    fn get_browser_info(&self) -> BrowserInfo;
}

/// Factory function to create appropriate browser client based on app name
pub fn create_browser_client(app_name: &str) -> Option<Box<dyn BrowserClient>> {
    match app_name.to_lowercase().as_str() {
        "chromium-browser" | "chrome" | "google-chrome" => {
            Some(Box::new(ChromeClient::new()))
        }
        "firefox" | "firefox-esr" => {
            Some(Box::new(FirefoxClient::new()))
        }
        _ => None,
    }
}
