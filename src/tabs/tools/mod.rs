pub mod backend;
pub mod view;

pub use backend::{
    apt_package_op, apt_php_op, scan_apache_modules, scan_php_extensions, scan_php_versions,
    switch_php, toggle_apache_module,
};

use crate::messages::Message;
use iced::Element;

#[derive(Debug, Clone, PartialEq)]
pub enum PhpStatus {
    Installed,
    Available,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PhpRelease {
    pub version: String,
    pub status: PhpStatus,
    pub is_active: bool,
    pub apache_mod_available: bool,
    pub apache_mod_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ApacheModule {
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PhpExtension {
    pub name: String,
    pub pkg_suffix: String,
    pub installed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolSection {
    Php,
    ApacheMods,
    PhpExts,
    Database,
}

pub struct ToolsTab {
    pub php_releases: Vec<PhpRelease>,
    pub apache_mods: Vec<ApacheModule>,
    pub php_exts: Vec<PhpExtension>,
    pub scanning: bool,
    pub mods_scanning: bool,
    pub install_log: Vec<(bool, String)>,
    pub db_status: String,
    pub last_php_error: Option<String>,
    pub active_section: ToolSection,
    pub mod_filter: String,
}

impl ToolsTab {
    pub fn new() -> Self {
        Self {
            php_releases: vec![
                PhpRelease {
                    version: "5.6".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                    apache_mod_available: false,
                    apache_mod_enabled: false,
                },
                PhpRelease {
                    version: "7.4".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                    apache_mod_available: false,
                    apache_mod_enabled: false,
                },
                PhpRelease {
                    version: "8.0".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                    apache_mod_available: false,
                    apache_mod_enabled: false,
                },
                PhpRelease {
                    version: "8.1".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                    apache_mod_available: false,
                    apache_mod_enabled: false,
                },
                PhpRelease {
                    version: "8.2".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                    apache_mod_available: false,
                    apache_mod_enabled: false,
                },
                PhpRelease {
                    version: "8.3".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                    apache_mod_available: false,
                    apache_mod_enabled: false,
                },
                PhpRelease {
                    version: "8.4".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                    apache_mod_available: false,
                    apache_mod_enabled: false,
                },
            ],
            apache_mods: Vec::new(),
            php_exts: vec![
                PhpExtension {
                    name: "curl".into(),
                    pkg_suffix: "php-curl".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "gd".into(),
                    pkg_suffix: "php-gd".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "mbstring".into(),
                    pkg_suffix: "php-mbstring".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "xml".into(),
                    pkg_suffix: "php-xml".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "zip".into(),
                    pkg_suffix: "php-zip".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "mysql".into(),
                    pkg_suffix: "php-mysql".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "pgsql".into(),
                    pkg_suffix: "php-pgsql".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "redis".into(),
                    pkg_suffix: "php-redis".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "intl".into(),
                    pkg_suffix: "php-intl".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "bcmath".into(),
                    pkg_suffix: "php-bcmath".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "soap".into(),
                    pkg_suffix: "php-soap".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "imagick".into(),
                    pkg_suffix: "php-imagick".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "xdebug".into(),
                    pkg_suffix: "php-xdebug".into(),
                    installed: false,
                },
                PhpExtension {
                    name: "sqlite3".into(),
                    pkg_suffix: "php-sqlite3".into(),
                    installed: false,
                },
            ],
            scanning: false,
            mods_scanning: false,
            install_log: Vec::new(),
            db_status: String::new(),
            last_php_error: None,
            active_section: ToolSection::Php,
            mod_filter: String::new(),
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
        self.install_log.push((ok, msg));
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }
}
