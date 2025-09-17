#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BrowserInfo {
    pub name: String,
    pub version: Option<String>,
    pub debug_port: Option<u16>,
}

impl BrowserInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: None,
            debug_port: None,
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.debug_port = Some(port);
        self
    }

    #[allow(dead_code)]
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
}
