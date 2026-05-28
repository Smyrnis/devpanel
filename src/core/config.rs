use crate::core::paths;
use crate::infra::system::get_home;

#[derive(Debug, Clone)]
pub struct DevPanelConfig {
    pub devpanel_conf: String,
    #[allow(dead_code)]
    pub hosts_file: String,
}

impl DevPanelConfig {
    pub fn load() -> Self {
        let config_path = get_home().join(".config/devpanel/config.toml");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            DevPanelConfig {
                devpanel_conf: parse_toml_str(&content, "devpanel_conf")
                    .unwrap_or_else(default_devpanel_conf),
                hosts_file: parse_toml_str(&content, "hosts_file")
                    .unwrap_or_else(|| paths::HOSTS_FILE.to_string()),
            }
        } else {
            DevPanelConfig {
                devpanel_conf: default_devpanel_conf(),
                hosts_file: paths::HOSTS_FILE.to_string(),
            }
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) {
        let dir = get_home().join(".config/devpanel");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(
            dir.join("config.toml"),
            format!(
                "devpanel_conf = \"{}\"\nhosts_file    = \"{}\"\n",
                self.devpanel_conf, self.hosts_file,
            ),
        );
    }
}

fn parse_toml_str(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if let Some(eq_pos) = t.find('=') {
            let before_eq = t[..eq_pos].trim();
            if before_eq == key {
                let val = t[eq_pos + 1..].trim();
                if val.starts_with('"') && val.ends_with('"') {
                    return Some(val[1..val.len() - 1].to_string());
                }
            }
        }
    }
    None
}

fn default_devpanel_conf() -> String {
    paths::DEVPANEL_CONF.to_string()
}
