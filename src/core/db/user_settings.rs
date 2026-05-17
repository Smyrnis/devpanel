use super::DevPanelDb;
use super::settings::{defaults, keys};
use rusqlite::Result as SqlResult;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserSettings {
    pub apache_log_level: String,
    pub apache_auto_reload: bool,
    pub php_default_version: String,
    pub php_display_errors: bool,
    pub projects_open_command: String,
    pub ui_confirm_deletes: bool,
    pub ui_toast_duration_ms: u32,
    pub ui_show_setup_log: bool,
    pub ui_language: String,
    pub ui_theme: String,
    pub ssh_default_key_type: String,
    pub editor_command: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        UserSettings {
            apache_log_level: defaults::APACHE_LOG_LEVEL.to_string(),
            apache_auto_reload: true,
            php_default_version: defaults::PHP_DEFAULT_VERSION.to_string(),
            php_display_errors: true,
            projects_open_command: defaults::PROJECTS_OPEN_COMMAND.to_string(),
            ui_confirm_deletes: true,
            ui_toast_duration_ms: 4000,
            ui_show_setup_log: true,
            ui_language: defaults::UI_LANGUAGE.to_string(),
            ui_theme: defaults::UI_THEME.to_string(),
            ssh_default_key_type: defaults::SSH_DEFAULT_KEY_TYPE.to_string(),
            editor_command: defaults::EDITOR_COMMAND.to_string(),
        }
    }
}

#[allow(dead_code)]
impl UserSettings {
    pub fn load(db: &DevPanelDb) -> Self {
        UserSettings {
            apache_log_level: db.get_or(keys::APACHE_LOG_LEVEL, defaults::APACHE_LOG_LEVEL),
            apache_auto_reload: db.get_bool(keys::APACHE_AUTO_RELOAD, true),
            php_default_version: db
                .get_or(keys::PHP_DEFAULT_VERSION, defaults::PHP_DEFAULT_VERSION),
            php_display_errors: db.get_bool(keys::PHP_DISPLAY_ERRORS, true),
            projects_open_command: db
                .get_or(keys::PROJECTS_OPEN_COMMAND, defaults::PROJECTS_OPEN_COMMAND),
            ui_confirm_deletes: db.get_bool(keys::UI_CONFIRM_DELETES, true),
            ui_toast_duration_ms: db.get_u32(keys::UI_TOAST_DURATION_MS, 4000),
            ui_show_setup_log: db.get_bool(keys::UI_SHOW_SETUP_LOG, true),
            ui_language: db.get_or(keys::UI_LANGUAGE, defaults::UI_LANGUAGE),
            ui_theme: db.get_or(keys::UI_THEME, defaults::UI_THEME),
            ssh_default_key_type: db
                .get_or(keys::SSH_DEFAULT_KEY_TYPE, defaults::SSH_DEFAULT_KEY_TYPE),
            editor_command: db.get_or(keys::EDITOR_COMMAND, defaults::EDITOR_COMMAND),
        }
    }

    pub fn save(&self, db: &DevPanelDb) -> SqlResult<()> {
        db.set(keys::APACHE_LOG_LEVEL, &self.apache_log_level)?;
        db.set_bool(keys::APACHE_AUTO_RELOAD, self.apache_auto_reload)?;
        db.set(keys::PHP_DEFAULT_VERSION, &self.php_default_version)?;
        db.set_bool(keys::PHP_DISPLAY_ERRORS, self.php_display_errors)?;
        db.set(keys::PROJECTS_OPEN_COMMAND, &self.projects_open_command)?;
        db.set_bool(keys::UI_CONFIRM_DELETES, self.ui_confirm_deletes)?;
        db.set_u32(keys::UI_TOAST_DURATION_MS, self.ui_toast_duration_ms)?;
        db.set_bool(keys::UI_SHOW_SETUP_LOG, self.ui_show_setup_log)?;
        db.set(keys::UI_LANGUAGE, &self.ui_language)?;
        db.set(keys::UI_THEME, &self.ui_theme)?;
        db.set(keys::SSH_DEFAULT_KEY_TYPE, &self.ssh_default_key_type)?;
        db.set(keys::EDITOR_COMMAND, &self.editor_command)?;
        Ok(())
    }
}
