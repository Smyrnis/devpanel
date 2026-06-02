pub mod view;

pub use crate::domain::tools::service::{
    apt_package_op, apt_php_op, composer_op, redis_service_op, scan_apache_modules,
    scan_installed_tools, scan_php_extensions, scan_php_versions, switch_php, toggle_apache_module,
};
pub use crate::domain::tools::{
    ApacheModule, InstalledTools, PhpExtension, PhpRelease, PhpStatus, ToolSection,
    default_php_extensions, default_php_releases,
};

use crate::messages::Message;
use iced::Element;

pub struct ToolsTab {
    pub php_releases: Vec<PhpRelease>,
    pub apache_mods: Vec<ApacheModule>,
    pub php_exts: Vec<PhpExtension>,
    pub scanning: bool,
    pub mods_scanning: bool,
    pub install_log: Vec<(bool, String)>,
    pub db_status: String,
    pub last_php_error: Option<String>,
    pub active_section: Option<ToolSection>,
    pub mod_filter: String,
    pub tool_search: String,
    pub installed_tools: InstalledTools,
    pub tools_scanning: bool,
    pub log_expanded: bool,
}

impl ToolsTab {
    pub fn new() -> Self {
        Self {
            php_releases: default_php_releases(),
            apache_mods: Vec::new(),
            php_exts: default_php_extensions(),
            scanning: false,
            mods_scanning: false,
            install_log: Vec::new(),
            db_status: String::new(),
            last_php_error: None,
            active_section: Some(ToolSection::Php),
            mod_filter: String::new(),
            tool_search: String::new(),
            installed_tools: InstalledTools::default(),
            tools_scanning: false,
            log_expanded: false,
        }
    }

    pub fn apply_scan(&mut self, results: Vec<(String, PhpStatus, bool, bool, bool)>) {
        self.scanning = false;
        for r in &mut self.php_releases {
            if let Some((_, status, active, mod_avail, mod_en)) =
                results.iter().find(|(v, _, _, _, _)| v == &r.version)
            {
                r.status = status.clone();
                r.is_active = *active;
                r.apache_mod_available = *mod_avail;
                r.apache_mod_enabled = *mod_en;
            }
        }
    }

    pub fn apply_mod_scan(&mut self, results: Vec<ApacheModule>) {
        self.mods_scanning = false;
        self.apache_mods = results;
        self.apache_mods.sort_by(|a, b| a.name.cmp(&b.name));
    }

    pub fn apply_ext_scan(&mut self, results: Vec<(String, bool)>) {
        for e in &mut self.php_exts {
            if let Some((_, inst)) = results.iter().find(|(n, _)| n == &e.name) {
                e.installed = *inst;
            }
        }
    }

    pub fn set_mod_enabled(&mut self, name: &str, enabled: bool) {
        for m in &mut self.apache_mods {
            if m.name == name {
                m.enabled = enabled;
            }
        }
        for r in &mut self.php_releases {
            let mod_name = format!("php{}", r.version);
            let mod_name_alt = if r.version == "5.6" {
                Some("php5")
            } else {
                None
            };
            if mod_name == name || mod_name_alt == Some(name) {
                r.apache_mod_enabled = enabled;
            }
        }
    }

    pub fn push_log(&mut self, ok: bool, msg: String) {
        if !ok && msg.contains("PHP") {
            self.last_php_error = Some(msg.clone());
        }
        if !ok {
            self.log_expanded = true;
        }
        self.install_log.push((ok, msg));
    }

    pub fn apply_tools_scan(&mut self, tools: InstalledTools) {
        self.installed_tools = tools;
        self.tools_scanning = false;
    }

    pub fn view(&self, compact: bool) -> Element<'_, Message> {
        view::render(self, compact)
    }
}

impl Default for ToolsTab {
    fn default() -> Self {
        Self::new()
    }
}
