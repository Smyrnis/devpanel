use iced::Task;

use crate::app::App;

use crate::core::db::{DevPanelDb, UserSettings};

use crate::messages::{ConfigMessage, Message};

impl App {
    pub(crate) fn handle_config(&mut self, msg: ConfigMessage) -> Task<Message> {
        match msg {
            ConfigMessage::SetSection(s) => {
                self.config_tab.active_section = s;
                Task::none()
            }

            ConfigMessage::Save => {
                self.config_tab.saving = true;
                let settings = self.config_tab.settings.clone();
                Task::perform(
                    async move {
                        match DevPanelDb::open() {
                            Ok(db) => match settings.save(&db) {
                                Ok(_) => (true, "Settings saved".to_string()),
                                Err(e) => (false, format!("Save failed: {}", e)),
                            },
                            Err(e) => (false, format!("DB error: {}", e)),
                        }
                    },
                    |(ok, msg)| Message::Config(ConfigMessage::SaveDone(ok, msg)),
                )
            }

            ConfigMessage::SaveDone(ok, msg) => {
                self.config_tab.apply_save_result(ok, msg.clone());
                if ok && let Ok(db) = DevPanelDb::open() {
                    let loaded = UserSettings::load(&db);
                    self.config_tab.settings = loaded;
                    self.db = Some(db);
                }
                self.show_toast(msg, ok)
            }

            ConfigMessage::ApacheLogLevelChanged(v) => {
                self.config_tab.settings.apache_log_level = v;
                Task::none()
            }
            ConfigMessage::ApacheAutoReloadChanged(v) => {
                self.config_tab.settings.apache_auto_reload = v;
                Task::none()
            }
            ConfigMessage::PhpDefaultVersionChanged(v) => {
                self.config_tab.settings.php_default_version = v;
                Task::none()
            }
            ConfigMessage::PhpDisplayErrorsChanged(v) => {
                self.config_tab.settings.php_display_errors = v;
                Task::none()
            }
            ConfigMessage::ProjectsOpenCommandChanged(v) => {
                self.config_tab.settings.projects_open_command = v;
                Task::none()
            }
            ConfigMessage::UiConfirmDeletesChanged(v) => {
                self.config_tab.settings.ui_confirm_deletes = v;
                Task::none()
            }
            ConfigMessage::UiToastDurationChanged(v) => {
                self.config_tab.settings.ui_toast_duration_ms = v;
                Task::none()
            }
            ConfigMessage::UiShowSetupLogChanged(v) => {
                self.config_tab.settings.ui_show_setup_log = v;
                Task::none()
            }
            ConfigMessage::SshDefaultKeyTypeChanged(v) => {
                self.config_tab.settings.ssh_default_key_type = v;
                Task::none()
            }
            ConfigMessage::EditorCommandChanged(v) => {
                self.config_tab.settings.editor_command = v;
                Task::none()
            }
        }
    }
}
