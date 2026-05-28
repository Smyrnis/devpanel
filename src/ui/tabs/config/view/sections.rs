use super::controls::{section_card, setting_row, setting_toggle_row};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::settings::ConfigSection;
use crate::lang::{lang_map::config as keys, text as tr};
use crate::messages::{ConfigMessage, Message};
use crate::ui::icons::Icon;
use crate::ui::tabs::config::ConfigTab;
use crate::ui::tabs::ssh_keys::{self, SshKeysTab};
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{Space, column, container, text, text_input};
use iced::{Element, Padding};

pub(super) fn section_body<'a>(
    tab: &'a ConfigTab,
    ssh_keys_tab: &'a SshKeysTab,
    section: ConfigSection,
    compact: bool,
) -> Element<'a, Message> {
    match section {
        ConfigSection::Ui => section_ui(tab),
        ConfigSection::Apache => section_apache(tab),
        ConfigSection::Php => section_php(tab),
        ConfigSection::Editor => section_editor(tab),
        ConfigSection::Advanced => section_advanced(tab, ssh_keys_tab, compact),
    }
}

fn section_apache(tab: &ConfigTab) -> Element<'_, Message> {
    let log_levels = vec![
        "error".to_string(),
        "warn".to_string(),
        "info".to_string(),
        "debug".to_string(),
    ];
    section_card(
        tr(keys::SECTION_APACHE),
        tr(keys::APACHE_AUTO_RELOAD_HELP),
        Icon::Apache,
        column![
            setting_row(
                tr(keys::LOG_LEVEL_LABEL),
                tr(keys::LOG_LEVEL_HELP),
                ui::dropdown(
                    log_levels,
                    Some(tab.settings.apache_log_level.clone()),
                    |v| Message::Config(ConfigMessage::ApacheLogLevelChanged(v))
                ),
            ),
            ui::divider(),
            setting_toggle_row(
                tr(keys::APACHE_AUTO_RELOAD_LABEL),
                tr(keys::APACHE_AUTO_RELOAD_HELP),
                tab.settings.apache_auto_reload,
                |v| Message::Config(ConfigMessage::ApacheAutoReloadChanged(v)),
            ),
        ]
        .spacing(0),
    )
}

fn section_php(tab: &ConfigTab) -> Element<'_, Message> {
    let php_versions = vec![
        "8.4".to_string(),
        "8.3".to_string(),
        "8.2".to_string(),
        "8.1".to_string(),
        "8.0".to_string(),
        "7.4".to_string(),
        "5.6".to_string(),
    ];
    section_card(
        tr(keys::SECTION_PHP),
        tr(keys::PHP_DEFAULT_VERSION_HELP),
        Icon::Php,
        column![
            setting_row(
                tr(keys::PHP_DEFAULT_VERSION_LABEL),
                tr(keys::PHP_DEFAULT_VERSION_HELP),
                ui::dropdown(
                    php_versions,
                    Some(tab.settings.php_default_version.clone()),
                    |v| Message::Config(ConfigMessage::PhpDefaultVersionChanged(v))
                ),
            ),
            ui::divider(),
            setting_toggle_row(
                tr(keys::DISPLAY_ERRORS_LABEL),
                tr(keys::DISPLAY_ERRORS_HELP),
                tab.settings.php_display_errors,
                |v| Message::Config(ConfigMessage::PhpDisplayErrorsChanged(v)),
            ),
        ]
        .spacing(0),
    )
}

fn section_ui(tab: &ConfigTab) -> Element<'_, Message> {
    let toast_durations = vec![
        "2000".to_string(),
        "3000".to_string(),
        "4000".to_string(),
        "5000".to_string(),
        "8000".to_string(),
    ];
    section_card(
        tr(keys::SECTION_UI),
        tr(keys::THEME_HELP),
        Icon::Config,
        column![
            setting_toggle_row(
                tr(keys::CONFIRM_DELETE_LABEL),
                tr(keys::CONFIRM_DELETE_HELP),
                tab.settings.ui_confirm_deletes,
                |v| Message::Config(ConfigMessage::UiConfirmDeletesChanged(v)),
            ),
            ui::divider(),
            setting_row(
                tr(keys::TOAST_DURATION_LABEL),
                tr(keys::TOAST_DURATION_HELP),
                ui::dropdown(
                    toast_durations,
                    Some(tab.settings.ui_toast_duration_ms.to_string()),
                    |v| {
                        let n = v.parse::<u32>().unwrap_or(4000);
                        Message::Config(ConfigMessage::UiToastDurationChanged(n))
                    }
                ),
            ),
            ui::divider(),
            setting_row(
                tr(keys::LANGUAGE_LABEL),
                tr(keys::LANGUAGE_HELP),
                ui::dropdown(
                    &tab.available_languages[..],
                    Some(tab.settings.ui_language.clone()),
                    |v| Message::Config(ConfigMessage::UiLanguageChanged(v))
                ),
            ),
            ui::divider(),
            setting_row(
                tr(keys::THEME_LABEL),
                tr(keys::THEME_HELP),
                ui::dropdown(
                    &tab.available_themes[..],
                    Some(tab.settings.ui_theme.clone()),
                    |v| { Message::Config(ConfigMessage::UiThemeChanged(v)) }
                ),
            ),
        ]
        .spacing(0),
    )
}

fn section_advanced<'a>(
    tab: &'a ConfigTab,
    ssh_keys_tab: &'a SshKeysTab,
    compact: bool,
) -> Element<'a, Message> {
    let key_types = vec![
        "Ed25519".to_string(),
        "RSA 4096".to_string(),
        "ECDSA 521".to_string(),
    ];
    section_card(
        tr(keys::SECTION_ADVANCED),
        tr(keys::DEFAULT_KEY_TYPE_HELP),
        Icon::Shield,
        column![
            setting_row(
                tr(keys::DEFAULT_KEY_TYPE_LABEL),
                tr(keys::DEFAULT_KEY_TYPE_HELP),
                ui::dropdown(
                    key_types,
                    Some(tab.settings.ssh_default_key_type.clone()),
                    |v| Message::Config(ConfigMessage::SshDefaultKeyTypeChanged(v))
                ),
            ),
            ui::divider(),
            setting_toggle_row(
                tr(keys::SHOW_SETUP_WARNINGS_LABEL),
                tr(keys::SHOW_SETUP_WARNINGS_HELP),
                tab.settings.ui_show_setup_log,
                |v| Message::Config(ConfigMessage::UiShowSetupLogChanged(v)),
            ),
            ui::divider(),
            ssh_settings_block(ssh_keys_tab, compact),
        ]
        .spacing(0),
    )
}

fn ssh_settings_block<'a>(ssh_keys_tab: &'a SshKeysTab, compact: bool) -> Element<'a, Message> {
    container(ssh_keys::view::settings_panel(ssh_keys_tab, compact))
        .padding(Padding::from([18, 20]))
        .into()
}

fn section_editor(tab: &ConfigTab) -> Element<'_, Message> {
    section_card(
        tr(keys::SECTION_EDITOR),
        tr(keys::EDITOR_COMMAND_HELP),
        Icon::Editor,
        column![
            setting_row(
                tr(keys::OPEN_COMMAND_LABEL),
                tr(keys::OPEN_COMMAND_HELP),
                text_input(
                    tr(keys::OPEN_COMMAND_PLACEHOLDER),
                    &tab.settings.projects_open_command
                )
                .on_input(|v| Message::Config(ConfigMessage::ProjectsOpenCommandChanged(v)))
                .padding(Padding::from([10, 12]))
                .size(13)
                .style(styles::text_input_style)
                .into(),
            ),
            ui::divider(),
            setting_row(
                tr(keys::EDITOR_COMMAND_LABEL),
                tr(keys::EDITOR_COMMAND_HELP),
                text_input(
                    tr(keys::OPEN_COMMAND_PLACEHOLDER),
                    &tab.settings.editor_command
                )
                .on_input(|v| Message::Config(ConfigMessage::EditorCommandChanged(v)))
                .padding(Padding::from([10, 12]))
                .size(13)
                .style(styles::text_input_style)
                .into(),
            ),
            Space::with_height(12),
            ui::info_banner(
                Icon::Info,
                text(tr(keys::EDITOR_OPEN_FILE_NOTE))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED))
                    .into(),
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_BORDER),
            ),
        ]
        .spacing(0),
    )
}
