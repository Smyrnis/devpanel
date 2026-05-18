pub mod backend;
pub mod view;

pub use backend::{
    add_vhost, bulk_delete_vhosts, delete_vhost, edit_vhost, load_config_file, save_config_file,
    scan_vhosts, toggle_https,
};

use crate::messages::Message;
use iced::Element;
use iced::widget::text_editor;

#[derive(Debug, Clone, PartialEq)]
pub struct VHostEntry {
    pub server_name: String,
    pub document_root: String,
    pub php_version: Option<String>,
    pub https_enabled: bool,
    pub tag: String,
    pub index: usize,
}

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
    pub fn open_edit(&mut self, e: &VHostEntry) {
        self.mode = FormMode::Edit(e.index);
        self.server_name = e.server_name.clone();
        self.document_root = e.document_root.clone();
        self.php_version = e.php_version.clone();
        self.https_enabled = e.https_enabled;
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

#[derive(Debug, Clone, PartialEq)]
pub enum VHostView {
    List,
    ConfigEditor,
}

pub struct VHostsTab {
    pub devpanel_conf: String,
    pub vhosts: Vec<VHostEntry>,
    pub scanning: bool,
    pub form: VHostForm,
    pub status_msg: Option<(bool, String)>,
    pub confirm_delete: Option<usize>,
    pub selected: Vec<usize>,
    pub bulk_tag: String,
    pub view_mode: VHostView,
    pub config_content: text_editor::Content,
    pub config_loading: bool,
    pub config_dirty: bool,
    /// Installed PHP versions that also have their Apache mod enabled.
    /// Populated from ToolsTab scan results and passed in from App.
    pub available_php_versions: Vec<String>,
}

impl VHostsTab {
    pub fn new(devpanel_conf: String) -> Self {
        Self {
            devpanel_conf,
            vhosts: Vec::new(),
            scanning: false,
            form: VHostForm::new(),
            status_msg: None,
            confirm_delete: None,
            selected: Vec::new(),
            bulk_tag: String::new(),
            view_mode: VHostView::List,
            config_content: text_editor::Content::new(),
            config_loading: false,
            config_dirty: false,
            available_php_versions: Vec::new(),
        }
    }

    pub fn set_vhosts(&mut self, v: Vec<VHostEntry>) {
        self.scanning = false;
        self.vhosts = v;
        self.selected
            .retain(|idx| self.vhosts.iter().any(|e| e.index == *idx));
    }

    /// Called whenever a PHP scan completes in ToolsTab.
    /// Keeps only versions that are installed AND have their Apache mod enabled.
    pub fn update_php_versions(&mut self, versions: Vec<String>) {
        self.available_php_versions = versions;
    }

    pub fn load_config_text(&mut self, text: String) {
        self.config_content = text_editor::Content::with_text(&text);
        self.config_loading = false;
        self.config_dirty = false;
    }

    pub fn view(&self, compact: bool) -> Element<'_, Message> {
        view::render(self, compact)
    }
}
