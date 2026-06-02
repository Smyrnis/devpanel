use super::controls::{section_card, setting_row, setting_toggle_row};
use crate::core::app_config::UiConfigField;
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
        ConfigSection::UiConfig => section_ui_config(tab),
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
    let php_versions: Vec<String> = crate::core::app_config::php_version_numbers()
        .into_iter()
        .rev()
        .collect();
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

fn section_ui_config(tab: &ConfigTab) -> Element<'_, Message> {
    section_card(
        tr(keys::SECTION_UI_CONFIG),
        tr(keys::UI_CONFIG_RESTART_NOTE),
        Icon::Config,
        column![ui_config_settings(tab)].spacing(0),
    )
}

fn ui_config_settings<'a>(tab: &'a ConfigTab) -> Element<'a, Message> {
    container(
        column![
            ui::info_banner(
                Icon::Info,
                text(tr(keys::UI_CONFIG_RESTART_NOTE))
                    .size(crate::core::app_config::text_metrics().caption)
                    .color(theme::color(theme_keys::TEXT_MUTED))
                    .into(),
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_BORDER),
            ),
            ui::panel_section(
                tr(keys::UI_CONFIG_WINDOW_SECTION),
                vec![
                    ui_config_input(
                        "window.width",
                        UiConfigField::WindowWidth,
                        &tab.ui_config.window_width
                    ),
                    ui_config_input(
                        "window.height",
                        UiConfigField::WindowHeight,
                        &tab.ui_config.window_height
                    ),
                    ui_config_input(
                        "window.min_width",
                        UiConfigField::WindowMinWidth,
                        &tab.ui_config.window_min_width
                    ),
                    ui_config_input(
                        "window.min_height",
                        UiConfigField::WindowMinHeight,
                        &tab.ui_config.window_min_height
                    ),
                    ui_config_input(
                        "layout.sidebar_collapse_width",
                        UiConfigField::SidebarCollapseWidth,
                        &tab.ui_config.sidebar_collapse_width
                    ),
                    ui_config_input(
                        "installer.content_width",
                        UiConfigField::InstallerContentWidth,
                        &tab.ui_config.installer_content_width
                    ),
                ],
            ),
            ui::panel_section(
                tr(keys::UI_CONFIG_TEXT_SECTION),
                vec![
                    ui_config_input(
                        "text.title",
                        UiConfigField::TextTitle,
                        &tab.ui_config.text_title
                    ),
                    ui_config_input(
                        "text.modal_title",
                        UiConfigField::TextModalTitle,
                        &tab.ui_config.text_modal_title
                    ),
                    ui_config_input(
                        "text.dialog_title",
                        UiConfigField::TextDialogTitle,
                        &tab.ui_config.text_dialog_title
                    ),
                    ui_config_input(
                        "text.section_title",
                        UiConfigField::TextSectionTitle,
                        &tab.ui_config.text_section_title
                    ),
                    ui_config_input(
                        "text.body",
                        UiConfigField::TextBody,
                        &tab.ui_config.text_body
                    ),
                    ui_config_input(
                        "text.caption",
                        UiConfigField::TextCaption,
                        &tab.ui_config.text_caption
                    ),
                    ui_config_input(
                        "text.tiny",
                        UiConfigField::TextTiny,
                        &tab.ui_config.text_tiny
                    ),
                    ui_config_input(
                        "text.badge",
                        UiConfigField::TextBadge,
                        &tab.ui_config.text_badge
                    ),
                ],
            ),
            ui::panel_section(
                tr(keys::UI_CONFIG_CONTROLS_SECTION),
                vec![
                    ui_config_input(
                        "controls.button_height",
                        UiConfigField::ControlButtonHeight,
                        &tab.ui_config.control_button_height
                    ),
                    ui_config_input(
                        "controls.summary_row_height",
                        UiConfigField::ControlSummaryRowHeight,
                        &tab.ui_config.control_summary_row_height
                    ),
                    ui_config_input(
                        "controls.detail_label_width",
                        UiConfigField::ControlDetailLabelWidth,
                        &tab.ui_config.control_detail_label_width
                    ),
                    ui_config_input(
                        "controls.checkbox_size",
                        UiConfigField::ControlCheckboxSize,
                        &tab.ui_config.control_checkbox_size
                    ),
                    ui_config_input(
                        "controls.large_checkbox_size",
                        UiConfigField::ControlLargeCheckboxSize,
                        &tab.ui_config.control_large_checkbox_size
                    ),
                    ui_config_input(
                        "controls.modal_log_height",
                        UiConfigField::ControlModalLogHeight,
                        &tab.ui_config.control_modal_log_height
                    ),
                    ui_config_input(
                        "controls.form_dropdown_width",
                        UiConfigField::ControlFormDropdownWidth,
                        &tab.ui_config.control_form_dropdown_width
                    ),
                ],
            ),
            ui::panel_section(
                tr(keys::UI_CONFIG_PANELS_SECTION),
                vec![
                    ui_config_input(
                        "panels.notification_width",
                        UiConfigField::PanelNotificationWidth,
                        &tab.ui_config.panel_notification_width
                    ),
                    ui_config_input(
                        "panels.sudo_dialog_width",
                        UiConfigField::PanelSudoDialogWidth,
                        &tab.ui_config.panel_sudo_dialog_width
                    ),
                    ui_config_input(
                        "panels.installer_log_height",
                        UiConfigField::PanelInstallerLogHeight,
                        &tab.ui_config.panel_installer_log_height
                    ),
                    ui_config_input(
                        "panels.ssh_keys_list_height",
                        UiConfigField::PanelSshKeysListHeight,
                        &tab.ui_config.panel_ssh_keys_list_height
                    ),
                    ui_config_input(
                        "panels.tools_list_height",
                        UiConfigField::PanelToolsListHeight,
                        &tab.ui_config.panel_tools_list_height
                    ),
                    ui_config_input(
                        "panels.tools_log_height",
                        UiConfigField::PanelToolsLogHeight,
                        &tab.ui_config.panel_tools_log_height
                    ),
                    ui_config_input(
                        "panels.tools_compact_log_height",
                        UiConfigField::PanelToolsCompactLogHeight,
                        &tab.ui_config.panel_tools_compact_log_height
                    ),
                ],
            ),
            ui::panel_section(
                tr(keys::UI_CONFIG_ICONS_SECTION),
                vec![ui_config_input(
                    "icons.sidebar_logo",
                    UiConfigField::IconSidebarLogo,
                    &tab.ui_config.icon_sidebar_logo
                )],
            ),
        ]
        .spacing(14),
    )
    .padding(Padding::from([18, 20]))
    .into()
}

fn ui_config_input<'a>(
    label: &'static str,
    field: UiConfigField,
    value: &'a str,
) -> Element<'a, Message> {
    setting_row(
        label,
        tr(keys::UI_CONFIG_NUMBER_HELP),
        text_input("0", value)
            .on_input(move |v| Message::Config(ConfigMessage::UiConfigChanged(field, v)))
            .padding(Padding::from([10, 12]))
            .size(crate::core::app_config::text_metrics().body)
            .style(styles::text_input_style)
            .into(),
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
                .size(crate::core::app_config::text_metrics().body)
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
                .size(crate::core::app_config::text_metrics().body)
                .style(styles::text_input_style)
                .into(),
            ),
            Space::with_height(12),
            ui::info_banner(
                Icon::Info,
                text(tr(keys::EDITOR_OPEN_FILE_NOTE))
                    .size(crate::core::app_config::text_metrics().caption)
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
