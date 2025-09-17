use super::{BrowserClient, BrowserInfo};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct ChromeClient {
    port: u16,
    timeout: Duration,
}

impl ChromeClient {
    pub fn new() -> Self {
        Self {
            port: 9222, // Default Chrome DevTools port
            timeout: Duration::from_millis(1000),
        }
    }

    #[allow(dead_code)]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Get list of open tabs from Chrome
    fn get_tabs(&self) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        // Simple HTTP GET request without external dependencies
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port))?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        let request = format!(
            "GET /json/list HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            self.port
        );

        stream.write_all(request.as_bytes())?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        // Parse HTTP response to get JSON body
        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .ok_or("Invalid HTTP response")?;

        let tabs: Vec<Value> = serde_json::from_str(body)?;
        Ok(tabs)
    }

    /// Find the currently active tab
    fn get_active_tab(&self) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let tabs = self.get_tabs()?;

        // Look for the active tab (type "page" and not a background page)
        for tab in tabs {
            if let Some(tab_type) = tab.get("type") {
                if tab_type == "page" {
                    // In Chrome, we can't easily determine which tab is "active"
                    // so we'll return the first page tab we find
                    // TODO: Could improve this by checking window focus or other criteria
                    return Ok(Some(tab));
                }
            }
        }

        Ok(None)
    }
}

impl BrowserClient for ChromeClient {
    fn get_active_tab_url(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match self.get_active_tab()? {
            Some(tab) => {
                if let Some(url) = tab.get("url") {
                    if let Some(url_str) = url.as_str() {
                        return Ok(Some(url_str.to_string()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    fn is_available(&self) -> bool {
        // Try to connect to Chrome DevTools port
        TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", self.port).parse().unwrap(),
            Duration::from_millis(500),
        )
        .is_ok()
    }

    fn get_browser_info(&self) -> BrowserInfo {
        BrowserInfo::new("Chrome").with_port(self.port)
    }
}

impl Default for ChromeClient {
    fn default() -> Self {
        Self::new()
    }
}
