use super::{ConfigSection, ConfigTab};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::config as keys, text as tr};
use crate::messages::{ConfigMessage, Message};
use crate::ui::icons::{self, Icon};
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{Space, button, checkbox, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Padding};

pub fn render(tab: &ConfigTab, compact: bool) -> Element<'_, Message> {
    let header_fn = if compact {
        ui::page_header_compact
    } else {
        ui::page_header
    };
    let header = header_fn(
        tr(keys::TITLE),
        tr(keys::SUBTITLE),
        vec![ui::primary_icon_button(
            Icon::Check,
            tr(keys::SAVE_CHANGES),
            Message::Config(ConfigMessage::Save),
        )],
    );

    let section_bar = section_tabs(&tab.active_section, compact);

    let section_body = match tab.active_section {
        ConfigSection::Apache => section_apache(tab),
        ConfigSection::Php => section_php(tab),
        ConfigSection::Projects => section_projects(tab),
        ConfigSection::Ui => section_ui(tab),
        ConfigSection::Ssh => section_ssh(tab),
        ConfigSection::Editor => section_editor(tab),
    };

    let save_bar = save_panel(tab);

    let status: Element<Message> = match &tab.status_msg {
        Some((ok, msg)) => {
            let (color, bg) = if *ok {
                (
                    theme::color(theme_keys::GREEN),
                    theme::color(theme_keys::GREEN_BG),
                )
            } else {
                (
                    theme::color(theme_keys::RED),
                    theme::color(theme_keys::RED_BG),
                )
            };
            container(
                row![
                    ui::status_dot(color),
                    Space::with_width(8),
                    text(msg.as_str())
                        .size(12)
                        .color(theme::color(theme_keys::TEXT_SECONDARY)),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([10, 14]))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(bg.into()),
                border: Border {
                    color: theme::color(theme_keys::BORDER_SUBTLE),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
        }
        None => Space::with_height(0).into(),
    };

    scrollable(
        column![
            header,
            Space::with_height(18),
            section_bar,
            Space::with_height(14),
            section_body,
            Space::with_height(16),
            status,
            Space::with_height(if tab.status_msg.is_some() { 12 } else { 0 }),
            save_bar,
            Space::with_height(24),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn save_panel(tab: &ConfigTab) -> Element<'_, Message> {
    let save_label = if tab.saving {
        tr(keys::SAVING)
    } else {
        tr(keys::SAVE_CHANGES)
    };
    let save_color = if tab.saving {
        theme::color(theme_keys::TEXT_MUTED)
    } else {
        theme::color(theme_keys::TEAL)
    };
    let save_btn = button(
        row![
            icons::solid(Icon::Check, 13.0, save_color),
            Space::with_width(8),
            text(save_label).size(13).color(save_color),
        ]
        .align_y(Alignment::Center),
    )
    .on_press_maybe(if tab.saving {
        None
    } else {
        Some(Message::Config(ConfigMessage::Save))
    })
    .padding(Padding::from([10, 24]))
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                text_color: theme::color(theme_keys::TEAL),
                border: Border {
                    color: theme::color(theme_keys::TEAL_BORDER),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(theme::color(theme_keys::TEAL_BG).into()),
            text_color: theme::color(theme_keys::TEAL),
            border: Border {
                color: theme::color(theme_keys::TEAL_BORDER),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        },
    });

    container(
        row![
            ui::status_dot(theme::color(theme_keys::YELLOW)),
            Space::with_width(12),
            column![
                text(tr(keys::SAVE_BAR_TITLE))
                    .size(14)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(3),
                text(tr(keys::SAVE_BAR_BODY))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .width(Length::Fill),
            save_btn,
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([12, 16]))
    .width(Length::Fill)
    .style(ui::surface_style())
    .into()
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

fn section_projects(tab: &ConfigTab) -> Element<'_, Message> {
    section_card(
        tr(keys::SECTION_PROJECTS),
        tr(keys::OPEN_COMMAND_HELP),
        Icon::Folder,
        column![setting_row(
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
        ),]
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
            setting_toggle_row(
                tr(keys::SHOW_SETUP_WARNINGS_LABEL),
                tr(keys::SHOW_SETUP_WARNINGS_HELP),
                tab.settings.ui_show_setup_log,
                |v| Message::Config(ConfigMessage::UiShowSetupLogChanged(v)),
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
                    |v| Message::Config(ConfigMessage::UiThemeChanged(v))
                ),
            ),
        ]
        .spacing(0),
    )
}

fn section_ssh(tab: &ConfigTab) -> Element<'_, Message> {
    let key_types = vec![
        "Ed25519".to_string(),
        "RSA 4096".to_string(),
        "ECDSA 521".to_string(),
    ];
    section_card(
        tr(keys::SECTION_SSH),
        tr(keys::DEFAULT_KEY_TYPE_HELP),
        Icon::Key,
        column![setting_row(
            tr(keys::DEFAULT_KEY_TYPE_LABEL),
            tr(keys::DEFAULT_KEY_TYPE_HELP),
            ui::dropdown(
                key_types,
                Some(tab.settings.ssh_default_key_type.clone()),
                |v| Message::Config(ConfigMessage::SshDefaultKeyTypeChanged(v))
            ),
        ),]
        .spacing(0),
    )
}

fn section_editor(tab: &ConfigTab) -> Element<'_, Message> {
    section_card(
        tr(keys::SECTION_EDITOR),
        tr(keys::EDITOR_COMMAND_HELP),
        Icon::Editor,
        column![
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

fn section_tabs(active: &ConfigSection, compact: bool) -> Element<'_, Message> {
    let sections = [
        (
            tr(keys::SECTION_APACHE),
            ConfigSection::Apache,
            Icon::Apache,
        ),
        (tr(keys::SECTION_PHP), ConfigSection::Php, Icon::Php),
        (
            tr(keys::SECTION_PROJECTS),
            ConfigSection::Projects,
            Icon::Folder,
        ),
        (tr(keys::SECTION_UI), ConfigSection::Ui, Icon::Config),
        (tr(keys::SECTION_SSH), ConfigSection::Ssh, Icon::Key),
        (
            tr(keys::SECTION_EDITOR),
            ConfigSection::Editor,
            Icon::Editor,
        ),
    ];
    let pills: Vec<Element<Message>> = sections
        .into_iter()
        .map(|(label, section, icon)| section_pill(label, icon, section, active))
        .collect();
    let content: Element<Message> = if compact {
        column(pills).spacing(8).into()
    } else {
        row(pills).spacing(8).into()
    };

    container(content)
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(ui::surface_style())
        .into()
}

fn section_pill<'a>(
    label: &'a str,
    icon: Icon,
    section: ConfigSection,
    active: &ConfigSection,
) -> Element<'a, Message> {
    let is_active = &section == active;
    let (color, bg, border) = if is_active {
        (
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_BORDER),
        )
    } else {
        (
            theme::color(theme_keys::TEXT_MUTED),
            theme::color(theme_keys::BG_SURFACE),
            theme::color(theme_keys::BORDER_SUBTLE),
        )
    };
    button(
        row![
            icons::solid_box(icon, 12.0, color, 14.0),
            Space::with_width(8),
            text(label).size(12).color(color),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::Config(ConfigMessage::SetSection(section)))
    .padding(Padding::from([10, 18]))
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                text_color: theme::color(theme_keys::TEAL),
                border: Border {
                    color: theme::color(theme_keys::TEAL_BORDER),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: color,
            border: Border {
                color: border,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        },
    })
    .into()
}

fn section_card<'a>(
    title: &'a str,
    description: &'a str,
    icon: Icon,
    content: iced::widget::Column<'a, Message>,
) -> Element<'a, Message> {
    container(
        column![
            row![
                container(icons::solid(
                    icon,
                    16.0,
                    theme::color(theme_keys::TEXT_SECONDARY)
                ))
                .padding(Padding::from([9, 10]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(theme::color(theme_keys::BG_CARD).into()),
                    border: Border {
                        color: theme::color(theme_keys::BORDER_SUBTLE),
                        width: 1.0,
                        radius: 24.0.into(),
                    },
                    ..Default::default()
                }),
                Space::with_width(12),
                column![
                    text(format!("{} Settings", title))
                        .size(18)
                        .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    Space::with_height(4),
                    text(description)
                        .size(12)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .spacing(0),
            ]
            .align_y(Alignment::Center),
            Space::with_height(18),
            container(content).style(ui::surface_style()),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn setting_row<'a>(
    label: &'a str,
    hint: &'a str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    row![
        setting_text(label, hint).width(Length::FillPortion(1)),
        Space::with_width(22),
        container(control).width(Length::FillPortion(1)),
    ]
    .align_y(Alignment::Center)
    .padding(Padding::from([18, 20]))
    .into()
}

fn setting_toggle_row<'a, F>(
    label: &'a str,
    hint: &'a str,
    value: bool,
    on_toggle: F,
) -> Element<'a, Message>
where
    F: Fn(bool) -> Message + 'a,
{
    let state_text = if value {
        tr(keys::ENABLED)
    } else {
        tr(keys::DISABLED)
    };
    setting_row(
        label,
        hint,
        row![
            checkbox("", value).on_toggle(on_toggle).size(18),
            Space::with_width(10),
            text(state_text).size(12).color(if value {
                theme::color(theme_keys::GREEN)
            } else {
                theme::color(theme_keys::TEXT_MUTED)
            }),
        ]
        .align_y(Alignment::Center)
        .into(),
    )
}

fn setting_text<'a>(label: &'a str, hint: &'a str) -> iced::widget::Column<'a, Message> {
    column![
        text(label)
            .size(14)
            .color(theme::color(theme_keys::TEXT_PRIMARY)),
        Space::with_height(4),
        text(hint)
            .size(12)
            .color(theme::color(theme_keys::TEXT_MUTED)),
    ]
    .spacing(0)
}
