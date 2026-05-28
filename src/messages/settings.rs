#[derive(Debug, Clone)]
pub enum ConfigMessage {
    ToggleSection(crate::domain::settings::ConfigSection),
    Save,
    SaveDone(bool, String),
    ApacheLogLevelChanged(String),
    ApacheAutoReloadChanged(bool),
    PhpDefaultVersionChanged(String),
    PhpDisplayErrorsChanged(bool),
    ProjectsOpenCommandChanged(String),
    UiConfirmDeletesChanged(bool),
    UiToastDurationChanged(u32),
    UiShowSetupLogChanged(bool),
    UiLanguageChanged(String),
    UiThemeChanged(String),
    SshDefaultKeyTypeChanged(String),
    EditorCommandChanged(String),
}
