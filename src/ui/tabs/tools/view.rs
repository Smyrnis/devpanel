use super::{
    ApacheModule, InstalledTools, PhpExtension, PhpRelease, PhpStatus, ToolSection, ToolsTab,
};
use crate::core::paths;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::templates::view as ui;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub fn render(tab: &ToolsTab) -> Element<'_, Message> {
    scrollable(
        column![
            column![
                text(tr(keys::TITLE))
                    .size(22)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(4),
                text(tr(keys::SUBTITLE))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0),
            Space::with_height(18),
            section_tabs(tab),
            Space::with_height(10),
            text_input(tr(keys::SEARCH_PLACEHOLDER), &tab.tool_search)
                .on_input(|v| Message::Tools(ToolsMessage::ToolSearchChanged(v)))
                .padding(Padding::from([7, 12]))
                .size(12)
                .style(|_, _| iced::widget::text_input::Style {
                    background: theme::color(theme_keys::BG_SURFACE).into(),
                    border: Border {
                        color: theme::color(theme_keys::BORDER_SUBTLE),
                        width: 1.0,
                        radius: 8.0.into()
                    },
                    icon: theme::color(theme_keys::TEXT_MUTED),
                    placeholder: theme::color(theme_keys::TEXT_MUTED),
                    value: theme::color(theme_keys::TEXT_PRIMARY),
                    selection: theme::color(theme_keys::TEAL),
                }),
            Space::with_height(16),
            match tab.active_section {
                ToolSection::Php => php_panel(tab),
                ToolSection::ApacheMods => apache_mods_panel(tab),
                ToolSection::PhpExts => php_exts_panel(tab),
                ToolSection::Runtimes => runtimes_panel(tab),
                ToolSection::Database => db_panel(tab),
            },
            Space::with_height(16),
            log_panel(tab),
            if tab.last_php_error.is_some() {
                Space::with_height(16)
            } else {
                Space::with_height(0)
            },
            if tab.last_php_error.is_some() {
                error_suggestion_panel(tab)
            } else {
                Space::with_height(0).into()
            },
            Space::with_height(22),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn section_tabs(tab: &ToolsTab) -> Element<'_, Message> {
    let sections = [
        (ToolSection::Php, tr(keys::SECTION_PHP_VERSIONS)),
        (ToolSection::ApacheMods, tr(keys::SECTION_APACHE_MODULES)),
        (ToolSection::PhpExts, tr(keys::SECTION_PHP_EXTENSIONS)),
        (ToolSection::Runtimes, tr(keys::SECTION_RUNTIMES)),
        (ToolSection::Database, tr(keys::SECTION_DATABASE)),
    ];
    let tabs: Vec<Element<Message>> = sections
        .iter()
        .map(|(sec, label)| {
            let active = *sec == tab.active_section;
            let (color, bg, bg_hover) = if active {
                (
                    theme::color(theme_keys::TEAL),
                    theme::color(theme_keys::TEAL_BG),
                    theme::color(theme_keys::TEAL_HOVER),
                )
            } else {
                (
                    theme::color(theme_keys::TEXT_MUTED),
                    theme::color(theme_keys::BG_SURFACE),
                    theme::color(theme_keys::BG_HOVER),
                )
            };
            let msg = match sec {
                ToolSection::Php => Message::Tools(ToolsMessage::SetSection(ToolSection::Php)),
                ToolSection::ApacheMods => {
                    Message::Tools(ToolsMessage::SetSection(ToolSection::ApacheMods))
                }
                ToolSection::PhpExts => {
                    Message::Tools(ToolsMessage::SetSection(ToolSection::PhpExts))
                }
                ToolSection::Runtimes => {
                    Message::Tools(ToolsMessage::SetSection(ToolSection::Runtimes))
                }
                ToolSection::Database => {
                    Message::Tools(ToolsMessage::SetSection(ToolSection::Database))
                }
            };
            button(text(*label).size(12).color(color))
                .on_press(msg)
                .padding(Padding::from([7, 16]))
                .style(move |_, status| match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(bg_hover.into()),
                        text_color: color,
                        border: Border {
                            color: if active {
                                theme::color(theme_keys::TEAL_BORDER)
                            } else {
                                theme::color(theme_keys::BORDER_SUBTLE)
                            },
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: Some(bg.into()),
                        text_color: color,
                        border: Border {
                            color: if active {
                                theme::color(theme_keys::TEAL_BORDER)
                            } else {
                                theme::color(theme_keys::BORDER_SUBTLE)
                            },
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    },
                })
                .into()
        })
        .collect();
    row(tabs).spacing(8).into()
}

fn php_panel(tab: &ToolsTab) -> Element<'_, Message> {
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
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            _ => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_BG).into()),
                text_color: theme::color(theme_keys::TEAL),
                border: Border {
                    radius: 8.0.into(),
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
                    radius: 8.0.into()
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
            radius: 8.0.into(),
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

fn apache_mods_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let scan_lbl = if tab.mods_scanning {
        tr(keys::SCANNING)
    } else {
        tr(keys::SCAN)
    };
    let header = row![
        column![
            text(tr(keys::SECTION_APACHE_MODULES))
                .size(14)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(3),
            text(format!(
                "{} {} - {}",
                tr(keys::APACHE_MODULES_HELP_PREFIX),
                paths::APACHE_MODS_AVAILABLE,
                tr(keys::APACHE_MODULES_HELP_SUFFIX),
            ))
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
        .on_press_maybe(if tab.mods_scanning {
            None
        } else {
            Some(Message::Tools(ToolsMessage::ScanApacheMods))
        })
        .padding(Padding::from([7, 14]))
        .style(|_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                    text_color: theme::color(theme_keys::TEAL),
                    border: Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            _ => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_BG).into()),
                text_color: theme::color(theme_keys::TEAL),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }),
    ]
    .align_y(Alignment::Center);

    let total = tab.apache_mods.len();
    let enabled = tab.apache_mods.iter().filter(|m| m.enabled).count();

    let filter_row = row![
        text_input(tr(keys::FILTER_MODULES), &tab.mod_filter)
            .on_input(|v| Message::Tools(ToolsMessage::ModFilterChanged(v)))
            .padding(Padding::from([7, 12]))
            .size(12)
            .style(|_, _| iced::widget::text_input::Style {
                background: theme::color(theme_keys::BG_SURFACE).into(),
                border: Border {
                    color: theme::color(theme_keys::BORDER_SUBTLE),
                    width: 1.0,
                    radius: 8.0.into()
                },
                icon: theme::color(theme_keys::TEXT_MUTED),
                placeholder: theme::color(theme_keys::TEXT_MUTED),
                value: theme::color(theme_keys::TEXT_PRIMARY),
                selection: theme::color(theme_keys::TEAL),
            })
            .width(Length::Fill),
    ];

    let q = if tab.tool_search.is_empty() {
        tab.mod_filter.to_lowercase()
    } else {
        tab.tool_search.to_lowercase()
    };
    let filtered: Vec<&ApacheModule> = tab
        .apache_mods
        .iter()
        .filter(|m| q.is_empty() || m.name.contains(&q))
        .collect();

    let body: Element<Message> = if total == 0 {
        container(
            column![
                text(tr(keys::NO_MODULES))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(6),
                text(format!(
                    "{} {}",
                    tr(keys::CLICK_SCAN_TO_READ),
                    paths::APACHE_MODS_AVAILABLE
                ))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0),
        )
        .padding(Padding::from([20, 0]))
        .into()
    } else {
        let rows: Vec<Element<Message>> = filtered.iter().map(|m| apache_mod_row(m)).collect();
        scrollable(column(rows).spacing(5)).height(420).into()
    };

    container(
        column![
            header,
            Space::with_height(14),
            if total > 0 {
                row![
                    ui::status_dot(theme::color(theme_keys::GREEN)),
                    Space::with_width(6),
                    text(format!("{} {}", enabled, tr(keys::ENABLED_SUFFIX)))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_width(18),
                    ui::status_dot(theme::color(theme_keys::BORDER_MED)),
                    Space::with_width(6),
                    text(format!("{} {}", total - enabled, tr(keys::DISABLED_SUFFIX)))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_width(18),
                    text(format!("{} {}", total, tr(keys::TOTAL_SUFFIX)))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center)
            } else {
                row![Space::with_width(0)]
            },
            Space::with_height(10),
            filter_row,
            Space::with_height(14),
            ui::thin_line(),
            Space::with_height(10),
            body,
            Space::with_height(16),
            container(
                row![
                    text("!").size(10).color(theme::color(theme_keys::YELLOW)),
                    Space::with_width(8),
                    text(tr(keys::MODULES_SUDO_NOTE))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center)
            )
            .padding(Padding::from([10, 12]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::YELLOW_BG).into()),
                border: Border {
                    color: theme::color(theme_keys::YELLOW_BORDER),
                    width: 1.0,
                    radius: 8.0.into()
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

fn apache_mod_row<'a>(m: &'a ApacheModule) -> Element<'a, Message> {
    let (dot_color, status_text) = if m.enabled {
        (theme::color(theme_keys::GREEN), tr(keys::STATUS_ENABLED))
    } else {
        (
            theme::color(theme_keys::BORDER_MED),
            tr(keys::STATUS_DISABLED),
        )
    };
    let action: Element<Message> = if m.enabled {
        small_action_btn(
            tr(keys::DISABLE),
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
            theme::color(theme_keys::RED_HOVER),
            Message::Tools(ToolsMessage::DisableApacheMod(m.name.clone())),
        )
    } else {
        small_action_btn(
            tr(keys::ENABLE),
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            Message::Tools(ToolsMessage::EnableApacheMod(m.name.clone())),
        )
    };
    container(
        row![
            ui::status_dot(dot_color),
            Space::with_width(12),
            column![
                text(format!("mod_{}", m.name))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(2),
                text(status_text).size(11).color(dot_color),
            ]
            .spacing(0)
            .width(Length::Fill),
            action,
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 14]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(
            if m.enabled {
                theme::color(theme_keys::BG_SURFACE)
            } else {
                theme::color(theme_keys::BG_BASE)
            }
            .into(),
        ),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn php_exts_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let active_ver: Option<String> = tab
        .php_releases
        .iter()
        .find(|r| r.is_active)
        .map(|r| r.version.clone());
    let ver_label = active_ver
        .as_deref()
        .unwrap_or(tr(keys::ACTIVE_VERSION_FALLBACK));

    let header = row![
        column![
            text(tr(keys::SECTION_PHP_EXTENSIONS))
                .size(14)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(3),
            text(format!(
                "{} {} {}",
                tr(keys::PHP_EXTENSIONS_HELP_PREFIX),
                ver_label,
                tr(keys::PHP_EXTENSIONS_HELP_SUFFIX)
            ))
            .size(11)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .width(Length::Fill),
        button(
            text(tr(keys::SCAN))
                .size(12)
                .color(theme::color(theme_keys::TEAL))
        )
        .on_press(Message::Tools(ToolsMessage::ScanPhpExts))
        .padding(Padding::from([7, 14]))
        .style(|_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                    text_color: theme::color(theme_keys::TEAL),
                    border: Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            _ => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_BG).into()),
                text_color: theme::color(theme_keys::TEAL),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }),
    ]
    .align_y(Alignment::Center);

    let q = tab.tool_search.to_lowercase();
    let rows: Vec<Element<Message>> = tab
        .php_exts
        .iter()
        .filter(|e| q.is_empty() || e.name.contains(&q) || e.pkg_suffix.contains(&q))
        .map(|e| php_ext_row(e, &active_ver))
        .collect();
    container(
        column![
            header,
            Space::with_height(18),
            ui::thin_line(),
            Space::with_height(14),
            column(rows).spacing(8),
            Space::with_height(16),
            container(
                row![
                    text("i").size(10).color(theme::color(theme_keys::BLUE)),
                    Space::with_width(8),
                    text(tr(keys::PHP_EXTENSIONS_NOTE))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center)
            )
            .padding(Padding::from([10, 12]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::BLUE_BG).into()),
                border: Border {
                    color: theme::color(theme_keys::BLUE_BORDER),
                    width: 1.0,
                    radius: 8.0.into()
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

fn php_ext_row<'a>(ext: &'a PhpExtension, active_ver: &Option<String>) -> Element<'a, Message> {
    let (dot_color, status_text) = if ext.installed {
        (theme::color(theme_keys::GREEN), tr(keys::STATUS_INSTALLED))
    } else {
        (
            theme::color(theme_keys::TEXT_MUTED),
            tr(keys::STATUS_NOT_INSTALLED),
        )
    };
    let pkg = match active_ver {
        Some(ver) => format!("php{}-{}", ver, ext.name),
        None => ext.pkg_suffix.clone(),
    };
    let action: Element<Message> = if ext.installed {
        small_action_btn(
            tr(keys::REMOVE),
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
            theme::color(theme_keys::RED_HOVER),
            Message::Tools(ToolsMessage::RemovePhpExt(pkg)),
        )
    } else {
        small_action_btn(
            tr(keys::INSTALL),
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            Message::Tools(ToolsMessage::InstallPhpExt(pkg)),
        )
    };
    container(
        row![
            ui::status_dot(dot_color),
            Space::with_width(12),
            column![
                row![
                    text(ext.name.as_str())
                        .size(13)
                        .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    Space::with_width(8),
                    text(ext.pkg_suffix.as_str())
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center),
                Space::with_height(2),
                text(status_text).size(11).color(dot_color),
            ]
            .spacing(0)
            .width(Length::Fill),
            action,
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
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn db_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let note = container(
        row![
            text("").size(10).color(theme::color(theme_keys::YELLOW)),
            Space::with_width(8),
            text(tr(keys::TERMINAL_ROOT_NOTE))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::YELLOW_BG).into()),
        border: Border {
            color: theme::color(theme_keys::YELLOW_BORDER),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    let status_row: Element<Message> = if !tab.db_status.is_empty() {
        container(
            text(&tab.db_status)
                .size(12)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
        )
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_height(0).into()
    };

    container(
        column![
            text(tr(keys::SECTION_DATABASE))
                .size(14)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(3),
            text(tr(keys::DATABASE_HELP))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(18),
            ui::thin_line(),
            Space::with_height(14),
            db_btn(
                tr(keys::MYSQL_MARIADB),
                tr(keys::MYSQL_MARIADB_HELP),
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_HOVER),
                theme::color(theme_keys::BLUE_BORDER),
                Message::Tools(ToolsMessage::OpenMysqlCli)
            ),
            Space::with_height(8),
            db_btn(
                tr(keys::MARIADB_EXPLICIT),
                tr(keys::MARIADB_EXPLICIT_HELP),
                theme::color(theme_keys::PURPLE),
                theme::color(theme_keys::PURPLE_BG),
                theme::color(theme_keys::PURPLE_HOVER),
                theme::color(theme_keys::PURPLE_BORDER),
                Message::Tools(ToolsMessage::OpenMariadbCli)
            ),
            Space::with_height(8),
            db_btn(
                tr(keys::MYSQL_SOCKET),
                tr(keys::MYSQL_SOCKET_HELP),
                theme::color(theme_keys::TEAL),
                theme::color(theme_keys::TEAL_BG),
                theme::color(theme_keys::TEAL_HOVER),
                theme::color(theme_keys::TEAL_BORDER),
                Message::Tools(ToolsMessage::OpenMysqlSocket)
            ),
            Space::with_height(16),
            status_row,
            if tab.db_status.is_empty() {
                Space::with_height(0)
            } else {
                Space::with_height(12)
            },
            note,
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn runtimes_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let tools = &tab.installed_tools;
    let q = tab.tool_search.to_lowercase();
    let mut cards: Vec<Element<Message>> = Vec::new();
    let candidates = [
        ("composer", runtime_composer_card(tools)),
        ("node npm nvm javascript", runtime_node_card(tools)),
        ("redis cache memory", runtime_redis_card(tools)),
    ];
    for (terms, card) in candidates {
        if q.is_empty() || terms.contains(&q) {
            cards.push(card);
        }
    }
    if cards.is_empty() {
        cards.push(
            container(
                text(tr(keys::NO_TOOLS_MATCH))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            )
            .padding(Padding::from([20, 16]))
            .into(),
        );
    }

    container(
        column![
            row![
                column![
                    text(tr(keys::COMPOSER_NODE_REDIS))
                        .size(14)
                        .color(theme::color(theme_keys::TEXT_SECONDARY)),
                    Space::with_height(3),
                    text(tr(keys::RUNTIMES_HELP))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .spacing(0)
                .width(Length::Fill),
                button(
                    text(if tab.tools_scanning {
                        tr(keys::SCANNING)
                    } else {
                        tr(keys::SCAN)
                    })
                    .size(12)
                    .color(theme::color(theme_keys::TEAL))
                )
                .on_press_maybe(if tab.tools_scanning {
                    None
                } else {
                    Some(Message::Tools(ToolsMessage::ScanInstalledTools))
                })
                .padding(Padding::from([7, 14]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                        text_color: theme::color(theme_keys::TEAL),
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::TEAL_BG).into()),
                        text_color: theme::color(theme_keys::TEAL),
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                }),
            ]
            .align_y(Alignment::Center),
            Space::with_height(18),
            ui::thin_line(),
            Space::with_height(14),
            column(cards).spacing(8),
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn runtime_composer_card(tools: &InstalledTools) -> Element<'_, Message> {
    let installed = tools.composer_version.is_some();
    let subtitle = tools
        .composer_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_INSTALLED));
    let action = if installed {
        small_action_btn(
            tr(keys::UPDATE),
            theme::color(theme_keys::BLUE),
            theme::color(theme_keys::BLUE_BG),
            theme::color(theme_keys::BLUE_HOVER),
            Message::Tools(ToolsMessage::UpdateComposer),
        )
    } else {
        small_action_btn(
            tr(keys::INSTALL),
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            Message::Tools(ToolsMessage::InstallComposer),
        )
    };
    runtime_card(
        tr(keys::COMPOSER).into(),
        subtitle.into(),
        if installed {
            theme::color(theme_keys::GREEN)
        } else {
            theme::color(theme_keys::TEXT_MUTED)
        },
        action,
    )
}

fn runtime_node_card(tools: &InstalledTools) -> Element<'_, Message> {
    let node = tools
        .node_version
        .as_deref()
        .unwrap_or(tr(keys::NODE_NOT_INSTALLED));
    let npm = tools
        .npm_version
        .as_deref()
        .unwrap_or(tr(keys::NPM_NOT_INSTALLED));
    let nvm = if tools.nvm_available {
        tr(keys::NVM_AVAILABLE)
    } else {
        tr(keys::NVM_NOT_FOUND)
    };
    let action = small_action_btn(
        tr(keys::NVM_COMMAND),
        theme::color(theme_keys::PURPLE),
        theme::color(theme_keys::PURPLE_BG),
        theme::color(theme_keys::PURPLE_HOVER),
        Message::Tools(ToolsMessage::CopyNvmInstallCommand),
    );
    runtime_card(
        tr(keys::NODE_JS).into(),
        format!("{} / {} / {}", node, npm, nvm),
        if tools.node_version.is_some() {
            theme::color(theme_keys::GREEN)
        } else {
            theme::color(theme_keys::TEXT_MUTED)
        },
        action,
    )
}

fn runtime_redis_card(tools: &InstalledTools) -> Element<'_, Message> {
    let status = if !tools.redis_installed {
        tr(keys::REDIS_NOT_INSTALLED).to_string()
    } else if tools.redis_running {
        format!(
            "{}{}",
            tr(keys::REDIS_RUNNING),
            tools
                .redis_memory
                .as_deref()
                .map(|m| format!(" / {}", m))
                .unwrap_or_default()
        )
    } else {
        tr(keys::REDIS_STOPPED).into()
    };
    let action = if tools.redis_running {
        small_action_btn(
            tr(keys::STOP),
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
            theme::color(theme_keys::RED_HOVER),
            Message::Tools(ToolsMessage::RedisStop),
        )
    } else {
        small_action_btn(
            tr(keys::START),
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            Message::Tools(ToolsMessage::RedisStart),
        )
    };
    runtime_card(
        tr(keys::REDIS).into(),
        status,
        if tools.redis_running {
            theme::color(theme_keys::GREEN)
        } else {
            theme::color(theme_keys::TEXT_MUTED)
        },
        action,
    )
}

fn runtime_card(
    title: String,
    subtitle: String,
    color: Color,
    action: Element<'_, Message>,
) -> Element<'_, Message> {
    container(
        row![
            ui::status_dot(color),
            Space::with_width(12),
            column![
                text(title)
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(2),
                text(subtitle)
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .width(Length::Fill),
            action,
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
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn log_panel(tab: &ToolsTab) -> Element<'_, Message> {
    if tab.install_log.is_empty() {
        return Space::with_height(0).into();
    }

    let rows: Vec<Element<Message>> = tab
        .install_log
        .iter()
        .map(|(ok, msg)| {
            let (prefix, color) = if *ok {
                (tr(keys::LOG_OK), theme::color(theme_keys::GREEN))
            } else {
                (tr(keys::LOG_ERR), theme::color(theme_keys::RED))
            };
            row![
                text(prefix).size(11).color(color),
                text(msg.as_str())
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_SECONDARY))
            ]
            .into()
        })
        .collect();

    container(
        column![
            row![
                text(tr(keys::ACTIVITY_LOG))
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_MUTED))
                    .width(Length::Fill),
                button(
                    text(tr(keys::CLEAR))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                )
                .on_press(Message::Tools(ToolsMessage::ClearLog))
                .padding(Padding::from([4, 10]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::BG_HOVER).into()),
                        text_color: theme::color(theme_keys::TEXT_PRIMARY),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: None,
                        text_color: theme::color(theme_keys::TEXT_MUTED),
                        ..Default::default()
                    },
                }),
            ]
            .align_y(Alignment::Center),
            Space::with_height(10),
            scrollable(column(rows).spacing(5).padding(Padding::from([4, 0]))).height(150),
        ]
        .spacing(0)
        .padding(Padding::from([16, 18])),
    )
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn error_suggestion_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let php_version = tab
        .install_log
        .iter()
        .rev()
        .find_map(|(ok, msg)| {
            if !*ok && msg.contains("PHP") {
                msg.split("PHP ")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "8.2".to_string());

    let fix_commands = format!(
        "# Add the packages.sury.org/php repository.\n\
         sudo apt-get update\n\
         sudo apt-get install -y lsb-release ca-certificates apt-transport-https curl\n\
         sudo curl -sSLo /tmp/debsuryorg-archive-keyring.deb https://packages.sury.org/debsuryorg-archive-keyring.deb\n\
         sudo dpkg -i /tmp/debsuryorg-archive-keyring.deb\n\
         sudo sh -c 'echo \"deb [signed-by=/usr/share/keyrings/debsuryorg-archive-keyring.gpg] \
         https://packages.sury.org/php/ $(lsb_release -sc) main\" > /etc/apt/sources.list.d/php.list'\n\
         sudo apt-get update\n\n# Install PHP\nsudo apt-get install -y php{}",
        php_version
    );

    container(
        column![
            row![
                text(tr(keys::PHP_NOT_FOUND)).size(13).color(Color {
                    r: 1.0,
                    g: 0.650,
                    b: 0.0,
                    a: 1.0
                }),
                Space::with_width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            Space::with_height(10),
            text(tr(keys::PHP_PPA_MISSING))
                .size(12)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(12),
            container(
                scrollable(
                    text(fix_commands.clone())
                        .size(10)
                        .color(theme::color(theme_keys::BORDER_MED))
                )
                .height(180)
            )
            .padding(Padding::from([12, 14]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(
                    Color {
                        r: 0.08,
                        g: 0.08,
                        b: 0.08,
                        a: 1.0
                    }
                    .into()
                ),
                border: Border {
                    color: theme::color(theme_keys::BORDER_SUBTLE),
                    width: 1.0,
                    radius: 6.0.into()
                },
                ..Default::default()
            }),
            Space::with_height(10),
            button(
                text(tr(keys::GET_TEXT_FILE))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_PRIMARY))
            )
            .on_press(Message::Tools(ToolsMessage::CopyFixCommands(fix_commands)))
            .padding(Padding::from([6, 12]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::BLUE_HOVER).into()),
                    text_color: theme::color(theme_keys::WHITE),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                _ => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::BLUE_BG).into()),
                    text_color: theme::color(theme_keys::TEXT_PRIMARY),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .padding(Padding::from([16, 18]))
    .style(|_: &iced::Theme| container::Style {
        background: Some(
            Color {
                r: 0.200,
                g: 0.120,
                b: 0.080,
                a: 1.0,
            }
            .into(),
        ),
        border: Border {
            color: Color {
                r: 1.0,
                g: 0.650,
                b: 0.0,
                a: 1.0,
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn small_action_btn<'a>(
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    msg: Message,
) -> Element<'a, Message> {
    button(text(label).size(12).color(color))
        .on_press(msg)
        .padding(Padding::from([6, 14]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(bg_hover.into()),
                    text_color: color,
                    border: Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            _ => iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: color,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .into()
}
fn db_btn<'a>(
    title: &'a str,
    subtitle: &'a str,
    accent: Color,
    bg: Color,
    bg_hover: Color,
    _border: Color,
    msg: Message,
) -> Element<'a, Message> {
    button(
        row![
            container(Space::with_width(4))
                .width(4)
                .height(28)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(accent.into()),
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(12),
            column![
                text(title)
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(2),
                text(subtitle)
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .width(Length::Fill),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(msg)
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(bg_hover.into()),
                text_color: theme::color(theme_keys::TEXT_PRIMARY),
                border: Border {
                    color: theme::color(theme_keys::BORDER_MED),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: theme::color(theme_keys::TEXT_PRIMARY),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
    })
    .into()
}
