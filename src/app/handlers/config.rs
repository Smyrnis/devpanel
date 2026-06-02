use iced::Task;

use crate::app::App;

use crate::core::db::{DevPanelDb, UserSettings};

use crate::domain::settings::ConfigSection;
use crate::messages::{ConfigMessage, Message, SshKeysMessage};

impl App {
    pub(crate) fn handle_config(&mut self, msg: ConfigMessage) -> Task<Message> {
        match msg {
            ConfigMessage::ToggleSection(s) => {
                let opening = self.config_tab.active_section.as_ref() != Some(&s);
                let should_load_ssh =
                    opening && s == ConfigSection::Advanced && self.ssh_keys.keys_list.is_empty();
                self.config_tab.active_section = if opening { Some(s) } else { None };
                if should_load_ssh {
                    Task::perform(crate::domain::ssh_keys::service::list_keys(), |keys| {
                        Message::SshKeys(SshKeysMessage::KeysListed(keys))
                    })
                } else {
                    Task::none()
                }
            }

            ConfigMessage::Save => {
                self.config_tab.saving = true;
                let settings = self.config_tab.settings.clone();
                let saved_settings = self.config_tab.saved_settings.clone();
                let ui_config = self.config_tab.ui_config.clone();
                let saved_ui_config = self.config_tab.saved_ui_config.clone();
                Task::perform(
                    async move {
                        let settings_changed = settings != saved_settings;
                        let ui_config_changed = ui_config != saved_ui_config;

                        if ui_config_changed && let Err(error) = ui_config.validate() {
                            return (false, format!("UI config invalid: {error}"));
                        }

                        if settings_changed {
                            match DevPanelDb::open() {
                                Ok(db) => {
                                    if let Err(e) = settings.save(&db) {
                                        return (false, format!("Save failed: {e}"));
                                    }
                                }
                                Err(e) => return (false, format!("DB error: {e}")),
                            }
                        }

                        if ui_config_changed
                            && let Err(error) =
                                crate::core::app_config::save_user_ui_config(&ui_config)
                        {
                            return (false, error);
                        }

                        let msg = if ui_config_changed {
                            "Settings saved. Window size changes apply on next launch.".to_string()
                        } else {
                            "Settings saved".to_string()
                        };
                        (true, msg)
                    },
                    |(ok, msg)| Message::Config(ConfigMessage::SaveDone(ok, msg)),
                )
            }

            ConfigMessage::SaveDone(ok, msg) => {
                self.config_tab.apply_save_result(ok, msg.clone());
                if ok && let Ok(db) = DevPanelDb::open() {
                    let loaded = UserSettings::load(&db);
                    crate::lang::set_language(&loaded.ui_language);
                    crate::core::theme::set_theme(&loaded.ui_theme);
                    self.config_tab.saved_settings = loaded.clone();
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
            ConfigMessage::UiLanguageChanged(v) => {
                self.config_tab.settings.ui_language = v;
                Task::none()
            }
            ConfigMessage::UiThemeChanged(v) => {
                self.config_tab.settings.ui_theme = v;
                Task::none()
            }
            ConfigMessage::UiConfigChanged(field, value) => {
                self.config_tab.ui_config.set_field(field, value);
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
