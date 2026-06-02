use super::VHostEntry;

#[derive(Debug, Clone, PartialEq)]
pub enum FormMode {
    Hidden,
    Add,
    Edit(usize),
}

#[derive(Debug, Clone)]
pub struct VHostForm {
    pub mode: FormMode,
    pub server_name: String,
    pub document_root: String,
    pub php_version: Option<String>,
    pub https_enabled: bool,
}

impl VHostForm {
    pub fn new() -> Self {
        Self {
            mode: FormMode::Hidden,
            server_name: String::new(),
            document_root: String::new(),
            php_version: None,
            https_enabled: false,
        }
    }

    pub fn open_add(&mut self) {
        self.mode = FormMode::Add;
        self.server_name.clear();
        self.document_root.clear();
        self.php_version = None;
        self.https_enabled = false;
    }

    pub fn open_edit(&mut self, entry: &VHostEntry) {
        self.mode = FormMode::Edit(entry.index);
        self.server_name = entry.server_name.clone();
        self.document_root = entry.document_root.clone();
        self.php_version = entry.php_version.clone();
        self.https_enabled = entry.https_enabled;
    }

    pub fn hide(&mut self) {
        self.mode = FormMode::Hidden;
    }
}

impl Default for VHostForm {
    fn default() -> Self {
        Self::new()
    }
}
