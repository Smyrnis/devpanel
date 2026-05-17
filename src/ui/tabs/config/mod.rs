pub mod view;

use crate::core::db::UserSettings;
use crate::messages::Message;
use iced::Element;

pub struct ConfigTab {
    /// In-memory snapshot of all user settings.
    /// The Config tab binds directly to this; on "Save" the whole struct
    /// is flushed back to SQLite via UserSettings::save().
    pub settings: UserSettings,

    /// True while a save operation is running (shows a spinner / disables button).
    pub saving: bool,

    /// Status message shown after save (ok=true → green, ok=false → red).
    pub status_msg: Option<(bool, String)>,

    /// Which section of the config is currently expanded.
    pub active_section: ConfigSection,

    pub available_languages: Vec<String>,
    pub available_themes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSection {
    Apache,
    Php,
    Projects,
    Ui,
    Ssh,
    Editor,
}

impl ConfigTab {
    pub fn new(settings: UserSettings) -> Self {
        Self {
            settings,
            saving: false,
            status_msg: None,
            active_section: ConfigSection::Apache,
            available_languages: crate::lang::available_languages(),
            available_themes: crate::core::theme::available_themes(),
        }
    }

    pub fn apply_save_result(&mut self, ok: bool, msg: String) {
        self.saving = false;
        self.status_msg = Some((ok, msg));
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }
}
