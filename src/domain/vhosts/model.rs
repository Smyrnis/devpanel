#[derive(Debug, Clone, PartialEq)]
pub struct VHostEntry {
    pub server_name: String,
    pub document_root: String,
    pub php_version: Option<String>,
    pub https_enabled: bool,
    pub tag: String,
    pub index: usize,
}
