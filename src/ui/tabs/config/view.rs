use super::{ConfigSection, ConfigTab};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::config as keys, text as tr};
use crate::messages::{ConfigMessage, Message};
use crate::ui::templates::view as ui;
use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub fn render(tab: &ConfigTab) -> Element<'_, Message> {
    let header = ui::page_header(
        tr(keys::TITLE),
        tr(keys::SUBTITLE),
        vec![ui::primary_button(
            tr(keys::SAVE_CHANGES),
            Message::Config(ConfigMessage::Save),
        )],
    );

    let section_bar = row![
        section_pill(
            tr(keys::SECTION_APACHE),
            ConfigSection::Apache,
            &tab.active_section
        ),
        section_pill(
            tr(keys::SECTION_PHP),
            ConfigSection::Php,
            &tab.active_section
        ),
        section_pill(
            tr(keys::SECTION_PROJECTS),
            ConfigSection::Projects,
            &tab.active_section
        ),
        section_pill(tr(keys::SECTION_UI), ConfigSection::Ui, &tab.active_section),
        section_pill(
            tr(keys::SECTION_SSH),
            ConfigSection::Ssh,
            &tab.active_section
        ),
        section_pill(
            tr(keys::SECTION_EDITOR),
            ConfigSection::Editor,
            &tab.active_section
        ),
    ]
    .spacing(8);

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
    let save_btn = button(
        text(if tab.saving {
            tr(keys::SAVING)
        } else {
            tr(keys::SAVE_CHANGES)
        })
        .size(13)
        .color(if tab.saving {
            theme::color(theme_keys::TEXT_MUTED)
        } else {
            theme::color(theme_keys::TEAL)
        }),
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
            column![
                text(tr(keys::SAVE_BAR_TITLE))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_SECONDARY)),
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
    card(
        column![
            setting_label(tr(keys::LOG_LEVEL_LABEL), tr(keys::LOG_LEVEL_HELP)),
            Space::with_height(5),
            text_input(
                tr(keys::LOG_LEVEL_PLACEHOLDER),
                &tab.settings.apache_log_level
            )
            .on_input(|v| Message::Config(ConfigMessage::ApacheLogLevelChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
            Space::with_height(16),
            setting_toggle(
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
    card(
        column![
            setting_label(
                tr(keys::PHP_DEFAULT_VERSION_LABEL),
                tr(keys::PHP_DEFAULT_VERSION_HELP)
            ),
            Space::with_height(5),
            text_input("8.2", &tab.settings.php_default_version)
                .on_input(|v| Message::Config(ConfigMessage::PhpDefaultVersionChanged(v)))
                .padding(Padding::from([8, 10]))
                .size(13),
            Space::with_height(16),
            setting_toggle(
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
    card(
        column![
            setting_label(tr(keys::OPEN_COMMAND_LABEL), tr(keys::OPEN_COMMAND_HELP)),
            Space::with_height(5),
            text_input(
                tr(keys::OPEN_COMMAND_PLACEHOLDER),
                &tab.settings.projects_open_command
            )
            .on_input(|v| Message::Config(ConfigMessage::ProjectsOpenCommandChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
        ]
        .spacing(0),
    )
}

fn section_ui(tab: &ConfigTab) -> Element<'_, Message> {
    card(
        column![
            setting_toggle(
                tr(keys::CONFIRM_DELETE_LABEL),
                tr(keys::CONFIRM_DELETE_HELP),
                tab.settings.ui_confirm_deletes,
                |v| Message::Config(ConfigMessage::UiConfirmDeletesChanged(v)),
            ),
            Space::with_height(16),
            setting_label(
                tr(keys::TOAST_DURATION_LABEL),
                tr(keys::TOAST_DURATION_HELP)
            ),
            Space::with_height(5),
            text_input("4000", &tab.settings.ui_toast_duration_ms.to_string())
                .on_input(|v| {
                    let n = v.parse::<u32>().unwrap_or(4000);
                    Message::Config(ConfigMessage::UiToastDurationChanged(n))
                })
                .padding(Padding::from([8, 10]))
                .size(13),
            Space::with_height(16),
            setting_toggle(
                tr(keys::SHOW_SETUP_WARNINGS_LABEL),
                tr(keys::SHOW_SETUP_WARNINGS_HELP),
                tab.settings.ui_show_setup_log,
                |v| Message::Config(ConfigMessage::UiShowSetupLogChanged(v)),
            ),
            Space::with_height(16),
            setting_label(tr(keys::LANGUAGE_LABEL), tr(keys::LANGUAGE_HELP)),
            Space::with_height(5),
            pick_list(
                &tab.available_languages[..],
                Some(tab.settings.ui_language.clone()),
                |v| Message::Config(ConfigMessage::UiLanguageChanged(v))
            )
            .padding(Padding::from([8, 10]))
            .width(Length::Fixed(220.0)),
            Space::with_height(16),
            setting_label(tr(keys::THEME_LABEL), tr(keys::THEME_HELP)),
            Space::with_height(5),
            pick_list(
                &tab.available_themes[..],
                Some(tab.settings.ui_theme.clone()),
                |v| Message::Config(ConfigMessage::UiThemeChanged(v))
            )
            .padding(Padding::from([8, 10]))
            .width(Length::Fixed(220.0)),
        ]
        .spacing(0),
    )
}

fn section_ssh(tab: &ConfigTab) -> Element<'_, Message> {
    card(
        column![
            setting_label(
                tr(keys::DEFAULT_KEY_TYPE_LABEL),
                tr(keys::DEFAULT_KEY_TYPE_HELP)
            ),
            Space::with_height(5),
            text_input(
                tr(keys::DEFAULT_KEY_TYPE_PLACEHOLDER),
                &tab.settings.ssh_default_key_type
            )
            .on_input(|v| Message::Config(ConfigMessage::SshDefaultKeyTypeChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
        ]
        .spacing(0),
    )
}

fn section_editor(tab: &ConfigTab) -> Element<'_, Message> {
    let blue_bg = Color {
        r: 0.047,
        g: 0.090,
        b: 0.157,
        a: 1.0,
    };
    let blue_bdr = Color {
        r: 0.080,
        g: 0.140,
        b: 0.260,
        a: 1.0,
    };

    card(
        column![
            setting_label(
                tr(keys::EDITOR_COMMAND_LABEL),
                tr(keys::EDITOR_COMMAND_HELP)
            ),
            Space::with_height(5),
            text_input(
                tr(keys::OPEN_COMMAND_PLACEHOLDER),
                &tab.settings.editor_command
            )
            .on_input(|v| Message::Config(ConfigMessage::EditorCommandChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
            Space::with_height(12),
            container(
                row![
                    text("i").size(10).color(theme::color(theme_keys::BLUE)),
                    Space::with_width(8),
                    text(tr(keys::EDITOR_OPEN_FILE_NOTE))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center)
            )
            .padding(Padding::from([9, 12]))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(blue_bg.into()),
                border: Border {
                    color: blue_bdr,
                    width: 1.0,
                    radius: 6.0.into()
                },
                ..Default::default()
            }),
        ]
        .spacing(0),
    )
}

fn section_pill<'a>(
    label: &'a str,
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
    button(text(label).size(12).color(color))
        .on_press(Message::Config(ConfigMessage::SetSection(section)))
        .padding(Padding::from([7, 16]))
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

fn card(content: iced::widget::Column<'_, Message>) -> Element<'_, Message> {
    container(content.padding(Padding::from([20, 22])))
        .width(Length::Fill)
        .style(ui::card_style())
        .into()
}

fn setting_label<'a>(label: &'a str, hint: &'a str) -> Element<'a, Message> {
    column![
        text(label)
            .size(13)
            .color(theme::color(theme_keys::TEXT_PRIMARY)),
        Space::with_height(2),
        text(hint)
            .size(11)
            .color(theme::color(theme_keys::TEXT_MUTED)),
    ]
    .spacing(0)
    .into()
}

fn setting_toggle<'a, F>(
    label: &'a str,
    hint: &'a str,
    value: bool,
    on_toggle: F,
) -> Element<'a, Message>
where
    F: Fn(bool) -> Message + 'a,
{
    row![
        column![
            text(label)
                .size(13)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(2),
            text(hint)
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .width(Length::Fill),
        Space::with_width(16),
        checkbox("", value).on_toggle(on_toggle).size(16),
    ]
    .align_y(Alignment::Center)
    .into()
}
