pub mod view;

pub use crate::domain::dashboard::DashboardService;

use crate::core::paths;
use crate::domain::tools::InstalledTools;
use crate::messages::Message;
use iced::Element;

pub struct DashboardTab {
    pub apache_running: bool,
    pub mysql_running: bool,
    pub php_versions: Vec<String>,
    pub active_php_version: Option<String>,
    pub web_root: String,
    pub apache_uptime: Option<String>,
    pub mysql_uptime: Option<String>,
    pub recent_failures: Vec<String>,
    pub php_info: Option<String>,
    pub php_info_loading: bool,
    pub expanded_service: Option<DashboardService>,
    pub installed_tools: InstalledTools,
    pub runtimes_scanning: bool,
    pub selected_composer_version: String,
    pub selected_node_version: String,
}

impl DashboardTab {
    pub fn new() -> Self {
        Self {
            apache_running: false,
            mysql_running: false,
            php_versions: Vec::new(),
            active_php_version: None,
            web_root: paths::WEB_ROOT.into(),
            apache_uptime: None,
            mysql_uptime: None,
            recent_failures: Vec::new(),
            php_info: None,
            php_info_loading: false,
            expanded_service: Some(DashboardService::Apache),
            installed_tools: InstalledTools::default(),
            runtimes_scanning: false,
            selected_composer_version: crate::core::app_config::default_composer_version(),
            selected_node_version: crate::core::app_config::default_node_version(),
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

    pub fn apply_runtime_scan(&mut self, tools: InstalledTools) {
        self.installed_tools = tools;
        self.runtimes_scanning = false;
    }

    pub fn view(&self, compact: bool) -> Element<'_, Message> {
        view::render(self, compact)
    }
}

impl Default for DashboardTab {
    fn default() -> Self {
        Self::new()
    }
}
