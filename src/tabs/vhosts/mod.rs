// src/tabs/vhosts/mod.rs — state, data types, public API

pub mod backend;
pub mod view;

// Re-export backend functions so callers can use `tabs::vhosts::scan_vhosts(...)` etc.
pub use backend::{
    add_vhost, delete_vhost, edit_vhost,
    load_config_file, save_config_file, scan_vhosts,
};

use iced::widget::text_editor;
use iced::Element;
use crate::Message;


#[derive(Debug, Clone)]
pub struct VHostEntry {
    pub server_name:   String,
    pub document_root: String,
    pub index:         usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormMode { Hidden, Add, Edit(usize) }

#[derive(Debug, Clone)]
pub struct VHostForm {
    pub mode:          FormMode,
    pub server_name:   String,
    pub document_root: String,
}

impl VHostForm {
    pub fn new() -> Self {
        Self { mode: FormMode::Hidden, server_name: String::new(), document_root: String::new() }
    }
    pub fn open_add(&mut self) {
        self.mode = FormMode::Add;
        self.server_name.clear();
        self.document_root.clear();
    }
    pub fn open_edit(&mut self, e: &VHostEntry) {
        self.mode          = FormMode::Edit(e.index);
        self.server_name   = e.server_name.clone();
        self.document_root = e.document_root.clone();
    }
    pub fn hide(&mut self) { self.mode = FormMode::Hidden; }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VHostView { List, ConfigEditor }


pub struct VHostsTab {
    pub devpanel_conf:  String,
    pub vhosts:         Vec<VHostEntry>,
    pub scanning:       bool,
    pub form:           VHostForm,
    pub status_msg:     Option<(bool, String)>,
    pub confirm_delete: Option<usize>,
    pub view_mode:      VHostView,
    pub config_content: text_editor::Content,
    pub config_loading: bool,
    pub config_dirty:   bool,
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
            view_mode: VHostView::List,
            config_content: text_editor::Content::new(),
            config_loading: false,
            config_dirty: false,
        }
    }

    pub fn set_vhosts(&mut self, v: Vec<VHostEntry>) {
        self.scanning = false;
        self.vhosts   = v;
    }

    pub fn load_config_text(&mut self, text: String) {
        self.config_content = text_editor::Content::with_text(&text);
        self.config_loading = false;
        self.config_dirty   = false;
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }
}
