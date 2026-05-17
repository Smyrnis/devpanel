use super::shared::small_action_btn;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::tabs::tools::{PhpRelease, PhpStatus, ToolsTab};
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn php_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let scan_lbl = if tab.scanning {
        tr(keys::SCANNING)
    } else {
        tr(keys::SCAN)
    };
    let header = row![
        column![
            text(tr(keys::SECTION_PHP_VERSIONS))
                .size(14)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(3),
            text(tr(keys::PHP_VERSIONS_HELP))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .width(Length::Fill),
        button(
            text(scan_lbl)
                .size(12)
                .color(theme::color(theme_keys::TEAL))
        )
        .on_press_maybe(if tab.scanning {
            None
        } else {
            Some(Message::Tools(ToolsMessage::ScanPhp))
        })
        .padding(Padding::from([7, 14]))
        .style(|_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                    text_color: theme::color(theme_keys::TEAL),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            _ => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_BG).into()),
                text_color: theme::color(theme_keys::TEAL),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }),
    ]
    .align_y(Alignment::Center);

    let q = tab.tool_search.to_lowercase();
    let rows: Vec<Element<Message>> = tab
        .php_releases
        .iter()
        .filter(|r| q.is_empty() || format!("php {}", r.version).contains(&q))
        .map(php_row)
        .collect();

    container(
        column![
            header,
            Space::with_height(18),
            ui::thin_line(),
            Space::with_height(14),
            container(
                row![
                    Space::with_width(19 + 12),
                    text(tr(keys::VERSION_APT_STATUS))
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                        .width(Length::Fill),
                    text(tr(keys::APACHE_MOD))
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                        .width(160),
                    Space::with_width(12),
                    text(tr(keys::APT_ACTION))
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center)
            )
            .padding(Padding::from([0, 14])),
            Space::with_height(6),
            column(rows).spacing(8),
            Space::with_height(16),
            container(
                row![
                    text("i").size(10).color(theme::color(theme_keys::BLUE)),
                    Space::with_width(8),
                    column![
                        text(tr(keys::PHP_PPA_NOTE))
                            .size(11)
                            .color(theme::color(theme_keys::TEXT_MUTED)),
                        Space::with_height(3),
                        text(tr(keys::APACHE_MOD_NOTE))
                            .size(11)
                            .color(theme::color(theme_keys::TEXT_MUTED)),
                    ]
                    .spacing(0),
                ]
                .align_y(Alignment::Start)
            )
            .padding(Padding::from([10, 12]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::BLUE_BG).into()),
                border: Border {
                    color: theme::color(theme_keys::BLUE_BORDER),
                    width: 1.0,
                    radius: 6.0.into()
                },
                ..Default::default()
            }),
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn php_row<'a>(r: &'a PhpRelease) -> Element<'a, Message> {
    let (status_color, status_label) = match r.status {
        PhpStatus::Installed => (theme::color(theme_keys::GREEN), tr(keys::STATUS_INSTALLED)),
        PhpStatus::Available => (
            theme::color(theme_keys::TEXT_MUTED),
            tr(keys::STATUS_AVAILABLE),
        ),
        PhpStatus::Unknown => (
            theme::color(theme_keys::TEXT_MUTED),
            tr(keys::STATUS_UNKNOWN),
        ),
    };
    let is_php56 = r.version == "5.6";

    let active_badge: Element<Message> = if r.is_active {
        container(
            text(tr(keys::ACTIVE))
                .size(10)
                .color(theme::color(theme_keys::TEAL)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::TEAL_BG).into()),
            border: Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_width(0).into()
    };

    let eol_badge: Element<Message> = if is_php56 {
        container(
            text(tr(keys::EOL))
                .size(9)
                .color(theme::color(theme_keys::YELLOW)),
        )
        .padding(Padding::from([3, 7]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::YELLOW_BG).into()),
            border: Border {
                color: theme::color(theme_keys::YELLOW_BORDER),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_width(0).into()
    };

    let dot = ui::status_dot(status_color);

    let apt_btn: Element<Message> = match r.status {
        PhpStatus::Installed => small_action_btn(
            tr(keys::REMOVE),
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
            theme::color(theme_keys::RED_HOVER),
            Message::Tools(ToolsMessage::RemovePhp(r.version.clone())),
        ),
        _ => small_action_btn(
            tr(keys::INSTALL),
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            Message::Tools(ToolsMessage::InstallPhp(r.version.clone())),
        ),
    };

    let mod_name = format!("php{}", r.version);
    let (mod_dot_color, mod_status_lbl) = if r.apache_mod_available {
        if r.apache_mod_enabled {
            (theme::color(theme_keys::GREEN), tr(keys::STATUS_ENABLED))
        } else {
            (theme::color(theme_keys::YELLOW), tr(keys::STATUS_DISABLED))
        }
    } else {
        (
            theme::color(theme_keys::BORDER_SUBTLE),
            tr(keys::STATUS_NOT_AVAILABLE),
        )
    };
    let mod_dot = ui::status_dot(mod_dot_color);

    let apache_btn: Element<Message> = if r.apache_mod_available {
        if r.apache_mod_enabled {
            small_action_btn(
                tr(keys::DISABLE_MOD),
                theme::color(theme_keys::RED),
                theme::color(theme_keys::RED_BG),
                theme::color(theme_keys::RED_HOVER),
                Message::Tools(ToolsMessage::DisableApacheMod(mod_name)),
            )
        } else {
            small_action_btn(
                tr(keys::ENABLE_MOD),
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_HOVER),
                Message::Tools(ToolsMessage::EnableApacheMod(mod_name)),
            )
        }
    } else {
        container(
            text(tr(keys::NO_APACHE_MOD))
                .size(10)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        )
        .padding(Padding::from([6, 0]))
        .into()
    };

    let card: Element<Message> = container(
        row![
            dot,
            Space::with_width(12),
            column![
                row![
                    text(format!("PHP {}", r.version))
                        .size(14)
                        .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    Space::with_width(8),
                    active_badge,
                    Space::with_width(4),
                    eol_badge,
                ]
                .align_y(Alignment::Center),
                Space::with_height(2),
                text(status_label).size(11).color(status_color),
            ]
            .spacing(0)
            .width(Length::Fill),
            container(Space::with_width(1))
                .width(1)
                .height(34)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(theme::color(theme_keys::BORDER_SUBTLE).into()),
                    ..Default::default()
                }),
            Space::with_width(12),
            column![
                row![
                    mod_dot,
                    Space::with_width(6),
                    text(mod_status_lbl)
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                ]
                .align_y(Alignment::Center),
                Space::with_height(5),
                apache_btn,
            ]
            .spacing(0)
            .width(160),
            Space::with_width(12),
            apt_btn,
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into();

    let ppa_hint: Element<Message> = if is_php56 && r.status != PhpStatus::Installed {
        column![
            Space::with_height(4),
            container(
                row![
                    text("!").size(9).color(theme::color(theme_keys::YELLOW)),
                    Space::with_width(6),
                    text(tr(keys::PHP56_PPA_HINT))
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center)
            )
            .padding(Padding::from([6, 14]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::YELLOW_BG).into()),
                border: Border {
                    color: theme::color(theme_keys::YELLOW_BORDER),
                    width: 1.0,
                    radius: 6.0.into()
                },
                ..Default::default()
            }),
        ]
        .spacing(0)
        .into()
    } else {
        Space::with_height(0).into()
    };

    column![card, ppa_hint].spacing(0).into()
}
