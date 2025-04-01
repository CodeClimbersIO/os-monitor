#[derive(Clone, Debug)]
pub struct BlockableItem {
    pub app_external_id: String,
    pub is_browser: bool,
}

impl BlockableItem {
    pub fn new(app_external_id: String, is_browser: bool) -> Self {
        Self {
            app_external_id,
            is_browser,
        }
    }
}
