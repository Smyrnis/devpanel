// src/config.rs — DevPanel configuration: load, save, and TOML helpers

use crate::system::get_home;

#[derive(Debug, Clone)]
pub struct DevPanelConfig {
    pub repos_root:    String,
    pub devpanel_conf: String,
    pub hosts_file:    String,
}

impl DevPanelConfig {
    pub fn load() -> Self {
        let config_path = get_home().join(".config/devpanel/config.toml");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            DevPanelConfig {
                repos_root:    parse_toml_str(&content, "repos_root")
                    .unwrap_or_else(default_repos_root),
                devpanel_conf: parse_toml_str(&content, "devpanel_conf")
                    .unwrap_or_else(default_devpanel_conf),
                hosts_file:    parse_toml_str(&content, "hosts_file")
                    .unwrap_or_else(|| "/etc/hosts".to_string()),
            }
        } else {
            DevPanelConfig {
                repos_root:    default_repos_root(),
                devpanel_conf: default_devpanel_conf(),
                hosts_file:    "/etc/hosts".to_string(),
            }
        }
    }

    pub fn save(&self) {
        let dir = get_home().join(".config/devpanel");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(
            dir.join("config.toml"),
            format!(
                "repos_root    = \"{}\"\ndevpanel_conf = \"{}\"\nhosts_file    = \"{}\"\n",
                self.repos_root, self.devpanel_conf, self.hosts_file,
            ),
        );
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn parse_toml_str(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with(&format!("{} =", key)) || t.starts_with(&format!("{}=", key)) {
            if let Some(eq) = t.find('=') {
                let val = t[eq + 1..].trim();
                if val.starts_with('"') && val.ends_with('"') {
                    return Some(val[1..val.len() - 1].to_string());
                }
            }
        }
    }
    None
}

fn default_repos_root() -> String {
    let candidate = get_home().join("projects");
    if candidate.exists() {
        candidate.to_string_lossy().to_string()
    } else {
        "/var/www/html".to_string()
    }
}

fn default_devpanel_conf() -> String {
    "/etc/apache2/sites-available/devpanel.conf".to_string()
}