pub mod view;

use crate::core::{app_config::UiConfigDraft, db::UserSettings};
use crate::domain::settings::ConfigSection;
use crate::messages::Message;
use crate::ui::tabs::ssh_keys::SshKeysTab;
use iced::Element;

pub struct ConfigTab {
    /// In-memory snapshot of all user settings.
    /// The Config tab binds directly to this; on "Save" the whole struct
    /// is flushed back to SQLite via UserSettings::save().
    pub settings: UserSettings,
    pub saved_settings: UserSettings,

    /// True while a save operation is running (shows a spinner / disables button).
    pub saving: bool,

    /// Status message shown after save (ok=true → green, ok=false → red).
    pub status_msg: Option<(bool, String)>,

    /// Which section of the config is currently expanded.
    pub active_section: Option<ConfigSection>,

    pub available_languages: Vec<String>,
    pub available_themes: Vec<String>,
    pub ui_config: UiConfigDraft,
    pub saved_ui_config: UiConfigDraft,
}

impl ConfigTab {
    pub fn new(settings: UserSettings) -> Self {
        let saved_settings = settings.clone();
        let ui_config = UiConfigDraft::current();
        let saved_ui_config = ui_config.clone();
        Self {
            settings,
            saved_settings,
            saving: false,
            status_msg: None,
            active_section: Some(ConfigSection::Ui),
            available_languages: crate::lang::available_languages(),
            available_themes: crate::core::theme::available_themes(),
            ui_config,
            saved_ui_config,
        }
    }

    pub fn apply_save_result(&mut self, ok: bool, msg: String) {
        self.saving = false;
        if ok {
            self.saved_settings = self.settings.clone();
            self.saved_ui_config = self.ui_config.clone();
        }
        self.status_msg = Some((ok, msg));
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.settings != self.saved_settings || self.ui_config != self.saved_ui_config
    }

    pub fn view<'a>(&'a self, ssh_keys: &'a SshKeysTab, compact: bool) -> Element<'a, Message> {
        view::render(self, ssh_keys, compact)
    }
}
