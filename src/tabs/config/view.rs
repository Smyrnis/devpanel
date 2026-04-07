use super::{ConfigSection, ConfigTab};
use crate::core::theme::*;
use crate::messages::{ConfigMessage, Message};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

const TEAL_BG:    Color = Color { r: 0.040, g: 0.160, b: 0.150, a: 1.0 };
const TEAL_BDR:   Color = Color { r: 0.060, g: 0.210, b: 0.200, a: 1.0 };
const TEAL_HOVER: Color = Color { r: 0.050, g: 0.185, b: 0.175, a: 1.0 };
const GREEN_BG:   Color = Color { r: 0.050, g: 0.160, b: 0.090, a: 1.0 };
const RED_BG:     Color = Color { r: 0.200, g: 0.060, b: 0.055, a: 1.0 };

pub fn render(tab: &ConfigTab) -> Element<'_, Message> {
    let header = column![
        text("Configuration").size(22).color(TEXT_PRIMARY),
        Space::with_height(4),
        text("Persistent preferences — stored in ~/.config/devpanel/devpanel.db")
            .size(13)
            .color(TEXT_MUTED),
    ]
    .spacing(0);

    let section_bar = row![
        section_pill("Apache",   ConfigSection::Apache,   &tab.active_section),
        section_pill("PHP",      ConfigSection::Php,      &tab.active_section),
        section_pill("Projects", ConfigSection::Projects, &tab.active_section),
        section_pill("UI",       ConfigSection::Ui,       &tab.active_section),
        section_pill("SSH",      ConfigSection::Ssh,      &tab.active_section),
        section_pill("Editor",   ConfigSection::Editor,   &tab.active_section),
    ]
    .spacing(8);

    let section_body = match tab.active_section {
        ConfigSection::Apache   => section_apache(tab),
        ConfigSection::Php      => section_php(tab),
        ConfigSection::Projects => section_projects(tab),
        ConfigSection::Ui       => section_ui(tab),
        ConfigSection::Ssh      => section_ssh(tab),
        ConfigSection::Editor   => section_editor(tab),
    };

    let save_btn = button(
        text(if tab.saving { "Saving…" } else { "Save Changes" })
            .size(13)
            .color(if tab.saving { TEXT_MUTED } else { TEAL }),
    )
    .on_press_maybe(if tab.saving {
        None
    } else {
        Some(Message::Config(ConfigMessage::Save))
    })
    .padding(Padding::from([10, 24]))
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
            iced::widget::button::Style {
                background: Some(TEAL_HOVER.into()),
                text_color: TEAL,
                border: Border { color: TEAL_BDR, width: 1.0, radius: 8.0.into() },
                ..Default::default()
            },
        _ => iced::widget::button::Style {
            background: Some(TEAL_BG.into()),
            text_color: TEAL,
            border: Border { color: TEAL_BDR, width: 1.0, radius: 8.0.into() },
            ..Default::default()
        },
    });

    let status: Element<Message> = match &tab.status_msg {
        Some((ok, msg)) => {
            let (color, bg) = if *ok { (GREEN, GREEN_BG) } else { (RED, RED_BG) };
            container(row![
                dot(color),
                Space::with_width(8),
                text(msg.as_str()).size(12).color(TEXT_SECONDARY),
            ]
            .align_y(Alignment::Center))
            .padding(Padding::from([10, 14]))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(bg.into()),
                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
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
            save_btn,
            Space::with_height(24),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn section_apache(tab: &ConfigTab) -> Element<'_, Message> {
    card(column![
        setting_label("Log level", "warn / error / info / debug"),
        text_input("warn", &tab.settings.apache_log_level)
            .on_input(|v| Message::Config(ConfigMessage::ApacheLogLevelChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
        Space::with_height(14),
        setting_toggle(
            "Auto-reload Apache on config save",
            "When enabled, Apache is reloaded every time devpanel.conf is saved.",
            tab.settings.apache_auto_reload,
            |v| Message::Config(ConfigMessage::ApacheAutoReloadChanged(v)),
        ),
    ]
    .spacing(6))
}

fn section_php(tab: &ConfigTab) -> Element<'_, Message> {
    card(column![
        setting_label("Default PHP version", "e.g. 8.2 — used as fallback in new VHosts"),
        text_input("8.2", &tab.settings.php_default_version)
            .on_input(|v| Message::Config(ConfigMessage::PhpDefaultVersionChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
        Space::with_height(14),
        setting_toggle(
            "Enable display_errors for development",
            "Sets display_errors = On in PHP INI for local dev convenience.",
            tab.settings.php_display_errors,
            |v| Message::Config(ConfigMessage::PhpDisplayErrorsChanged(v)),
        ),
    ]
    .spacing(6))
}

fn section_projects(tab: &ConfigTab) -> Element<'_, Message> {
    card(column![
        setting_label("Open command", "Command used to open a project folder (e.g. xdg-open, code, nautilus)"),
        text_input("xdg-open", &tab.settings.projects_open_command)
            .on_input(|v| Message::Config(ConfigMessage::ProjectsOpenCommandChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
    ]
    .spacing(6))
}

fn section_ui(tab: &ConfigTab) -> Element<'_, Message> {
    card(column![
        setting_toggle(
            "Confirm before deleting VHosts",
            "Shows a confirmation step before any virtual host is removed.",
            tab.settings.ui_confirm_deletes,
            |v| Message::Config(ConfigMessage::UiConfirmDeletesChanged(v)),
        ),
        Space::with_height(14),
        setting_label("Toast notification duration (ms)", "How long success/error banners stay visible"),
        text_input("4000", &tab.settings.ui_toast_duration_ms.to_string())
            .on_input(|v| {
                if let Ok(n) = v.parse::<u32>() {
                    Message::Config(ConfigMessage::UiToastDurationChanged(n))
                } else {
                    Message::Config(ConfigMessage::UiToastDurationChanged(4000))
                }
            })
            .padding(Padding::from([8, 10]))
            .size(13),
        Space::with_height(14),
        setting_toggle(
            "Show setup log warnings on startup",
            "Surfaces post-install WARN/ERROR entries from /var/log/devpanel/setup.log.",
            tab.settings.ui_show_setup_log,
            |v| Message::Config(ConfigMessage::UiShowSetupLogChanged(v)),
        ),
    ]
    .spacing(6))
}

fn section_ssh(tab: &ConfigTab) -> Element<'_, Message> {
    card(column![
        setting_label("Default key type", "Ed25519 / RSA 4096 / ECDSA 521"),
        text_input("Ed25519", &tab.settings.ssh_default_key_type)
            .on_input(|v| Message::Config(ConfigMessage::SshDefaultKeyTypeChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
    ]
    .spacing(6))
}

fn section_editor(tab: &ConfigTab) -> Element<'_, Message> {
    card(column![
        setting_label("Editor command", "Command used to open files (e.g. code, vim, gedit, xdg-open)"),
        text_input("xdg-open", &tab.settings.editor_command)
            .on_input(|v| Message::Config(ConfigMessage::EditorCommandChanged(v)))
            .padding(Padding::from([8, 10]))
            .size(13),
        Space::with_height(10),
        container(row![
            text("i").size(10).color(BLUE),
            Space::with_width(8),
            text("Used by 'Open File' buttons throughout the app.").size(11).color(TEXT_MUTED),
        ]
        .align_y(Alignment::Center))
        .padding(Padding::from([9, 12]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(Color { r: 0.047, g: 0.090, b: 0.157, a: 1.0 }.into()),
            border: Border { color: Color { r: 0.080, g: 0.140, b: 0.260, a: 1.0 }, width: 1.0, radius: 8.0.into() },
            ..Default::default()
        }),
    ]
    .spacing(6))
}

fn section_pill<'a>(
    label:   &'a str,
    section: ConfigSection,
    active:  &ConfigSection,
) -> Element<'a, Message> {
    let is_active = &section == active;
    let (color, bg, border) = if is_active {
        (TEAL, TEAL_BG, TEAL_BDR)
    } else {
        (TEXT_MUTED, BG_SURFACE, BORDER_SUBTLE)
    };
    button(text(label).size(12).color(color))
        .on_press(Message::Config(ConfigMessage::SetSection(section)))
        .padding(Padding::from([7, 16]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style {
                    background: Some(TEAL_HOVER.into()),
                    text_color: TEAL,
                    border: Border { color: TEAL_BDR, width: 1.0, radius: 8.0.into() },
                    ..Default::default()
                },
            _ => iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: color,
                border: Border { color: border, width: 1.0, radius: 8.0.into() },
                ..Default::default()
            },
        })
        .into()
}

fn card(content: iced::widget::Column<'_, Message>) -> Element<'_, Message> {
    container(content.padding(Padding::from([20, 22])))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_CARD.into()),
            border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 10.0.into() },
            ..Default::default()
        })
        .into()
}

fn setting_label<'a>(label: &'a str, hint: &'a str) -> Element<'a, Message> {
    column![
        text(label).size(13).color(TEXT_PRIMARY),
        Space::with_height(2),
        text(hint).size(11).color(TEXT_MUTED),
    ]
    .spacing(0)
    .into()
}

fn setting_toggle<'a, F>(
    label:   &'a str,
    hint:    &'a str,
    value:   bool,
    on_toggle: F,
) -> Element<'a, Message>
where
    F: Fn(bool) -> Message + 'a,
{
    row![
        column![
            text(label).size(13).color(TEXT_PRIMARY),
            Space::with_height(2),
            text(hint).size(11).color(TEXT_MUTED),
        ]
        .spacing(0)
        .width(Length::Fill),
        checkbox("", value).on_toggle(on_toggle).size(16),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn dot(color: Color) -> iced::widget::Container<'static, Message> {
    container(Space::with_width(6))
        .width(6)
        .height(6)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(color.into()),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        })
}
