pub mod backend;
pub mod view;

pub use backend::probe_services;

use crate::messages::Message;
use iced::Element;

pub struct DashboardTab {
    pub apache_running: bool,
    pub mysql_running: bool,
    pub php_versions: Vec<String>,
    pub active_php_version: Option<String>,
    pub distro: String,
    pub web_root: String,
    pub apache_conf_dir: String,
    pub apache_uptime: Option<String>,
    pub mysql_uptime: Option<String>,
    pub recent_failures: Vec<String>,
    pub php_info: Option<String>,
    pub php_info_loading: bool,
}

impl DashboardTab {
    pub fn new() -> Self {
        Self {
            apache_running: false,
            mysql_running: false,
            php_versions: Vec::new(),
            active_php_version: None,
            distro: backend::detect_distro(),
            web_root: "/var/www/html".into(),
            apache_conf_dir: "/etc/apache2".into(),
            apache_uptime: None,
            mysql_uptime: None,
            recent_failures: Vec::new(),
            php_info: None,
            php_info_loading: false,
        }
    }

    pub fn update_status(
        &mut self,
        apache: bool,
        mysql: bool,
        php: Option<String>,
        apache_uptime: Option<String>,
        mysql_uptime: Option<String>,
        recent_failures: Vec<String>,
    ) {
        self.apache_running = apache;
        self.mysql_running = mysql;
        self.active_php_version = php;
        self.apache_uptime = apache_uptime;
        self.mysql_uptime = mysql_uptime;
        self.recent_failures = recent_failures;
    }

    pub fn set_php_versions(&mut self, versions: Vec<String>) {
        self.php_versions = versions;
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }
}

impl Default for DashboardTab {
    fn default() -> Self {
        Self::new()
    }
}
