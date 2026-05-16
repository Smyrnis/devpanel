use super::DevPanelDb;
use rusqlite::{Result as SqlResult, params};

#[allow(dead_code)]
pub mod keys {
    pub const APACHE_LOG_LEVEL: &str = "apache.log_level";
    pub const APACHE_AUTO_RELOAD: &str = "apache.auto_reload_on_save";
    pub const PHP_DEFAULT_VERSION: &str = "php.default_version";
    pub const PHP_DISPLAY_ERRORS: &str = "php.display_errors_dev";
    pub const PROJECTS_ROOT: &str = "projects.root";
    pub const PROJECTS_OPEN_COMMAND: &str = "projects.open_command";
    pub const UI_CONFIRM_DELETES: &str = "ui.confirm_deletes";
    pub const UI_TOAST_DURATION_MS: &str = "ui.toast_duration_ms";
    pub const UI_SHOW_SETUP_LOG: &str = "ui.show_setup_log_on_warn";
    pub const SSH_DEFAULT_KEY_TYPE: &str = "ssh.default_key_type";
    pub const EDITOR_COMMAND: &str = "editor.command";
}

#[allow(dead_code)]
pub mod defaults {
    pub const APACHE_LOG_LEVEL: &str = "warn";
    pub const APACHE_AUTO_RELOAD: &str = "true";
    pub const PHP_DEFAULT_VERSION: &str = "";
    pub const PHP_DISPLAY_ERRORS: &str = "true";
    pub const PROJECTS_OPEN_COMMAND: &str = "xdg-open";
    pub const UI_CONFIRM_DELETES: &str = "true";
    pub const UI_TOAST_DURATION_MS: &str = "4000";
    pub const UI_SHOW_SETUP_LOG: &str = "true";
    pub const SSH_DEFAULT_KEY_TYPE: &str = "Ed25519";
    pub const EDITOR_COMMAND: &str = "xdg-open";
}

#[allow(dead_code)]
impl DevPanelDb {
    pub fn get(&self, key: &str) -> SqlResult<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .ok()
            .flatten()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn set(&self, key: &str, value: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn all_settings(&self) -> SqlResult<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        let default_str = if default { "true" } else { "false" };
        self.get_or(key, default_str) == "true"
    }

    pub fn set_bool(&self, key: &str, value: bool) -> SqlResult<()> {
        self.set(key, if value { "true" } else { "false" })
    }

    pub fn get_u32(&self, key: &str, default: u32) -> u32 {
        self.get_or(key, &default.to_string())
            .parse()
            .unwrap_or(default)
    }

    pub fn set_u32(&self, key: &str, value: u32) -> SqlResult<()> {
        self.set(key, &value.to_string())
    }
}
