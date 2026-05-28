pub mod view;

pub use crate::domain::vhosts::service::{
    add_vhost, bulk_delete_vhosts, delete_vhost, edit_vhost, load_config_file, save_config_file,
    scan_vhosts, toggle_https,
};
pub use crate::domain::vhosts::{VHostEntry, VHostForm};

use crate::messages::Message;
use iced::Element;
use iced::widget::text_editor;

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
    pub search_query: String,
    pub expanded_vhost: Option<usize>,
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
            search_query: String::new(),
            expanded_vhost: None,
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
        if self
            .expanded_vhost
            .is_some_and(|idx| !self.vhosts.iter().any(|e| e.index == idx))
        {
            self.expanded_vhost = None;
        }
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
