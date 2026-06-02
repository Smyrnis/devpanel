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
    UiConfigChanged(crate::core::app_config::UiConfigField, String),
    SshDefaultKeyTypeChanged(String),
    EditorCommandChanged(String),
}
