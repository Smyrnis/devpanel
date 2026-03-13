// src/tabs/dashboard/mod.rs — state, data types, public API

pub mod backend;
pub mod view;

pub use backend::{detect_php, probe_services, service_active};

use iced::Element;
use crate::Message;

// ── State ─────────────────────────────────────────────────────────────────

pub struct DashboardTab {
    pub apache_running:      bool,
    pub mysql_running:       bool,
    pub php_versions:        Vec<String>,
    pub active_php_version:  Option<String>,
    pub distro:              String,
    pub web_root:            String,
    pub apache_conf_dir:     String,
}

impl DashboardTab {
    pub fn new() -> Self {
        Self {
            apache_running:     false,
            mysql_running:      false,
            php_versions:       Vec::new(),
            active_php_version: None,
            distro:             backend::detect_distro(),
            web_root:           "/var/www/html".into(),
            apache_conf_dir:    "/etc/apache2".into(),
        }
    }

    pub fn update_status(&mut self, apache: bool, mysql: bool, php: Option<String>) {
        self.apache_running     = apache;
        self.mysql_running      = mysql;
        self.active_php_version = php;
    }

    pub fn set_php_versions(&mut self, versions: Vec<String>) {
        self.php_versions = versions;
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }
}
