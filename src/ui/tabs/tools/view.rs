use super::{InstalledTools, PhpStatus, ToolSection, ToolsTab};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Padding};

mod apache;
mod database;
mod extensions;
mod logs;
mod php;
mod runtimes;
mod shared;

use apache::apache_mods_panel;
use database::db_panel;
use extensions::php_exts_panel;
use logs::{error_suggestion_panel, log_panel};
use php::php_panel;
use runtimes::runtimes_panel;

pub fn render(tab: &ToolsTab) -> Element<'_, Message> {
    scrollable(
        column![
            ui::page_header(
                tr(keys::TITLE),
                tr(keys::SUBTITLE),
                vec![ui::primary_button(
                    tr(keys::SCAN_SYSTEM),
                    Message::Tools(ToolsMessage::ScanInstalledTools),
                )],
            ),
            Space::with_height(18),
            scan_summary(tab),
            Space::with_height(14),
            section_tabs(tab),
            Space::with_height(10),
            text_input(tr(keys::SEARCH_PLACEHOLDER), &tab.tool_search)
                .on_input(|v| Message::Tools(ToolsMessage::ToolSearchChanged(v)))
                .padding(Padding::from([7, 12]))
                .size(12)
                .style(styles::text_input_style),
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

fn scan_summary(tab: &ToolsTab) -> Element<'_, Message> {
    let installed_php = tab
        .php_releases
        .iter()
        .filter(|release| release.status == PhpStatus::Installed)
        .count();
    let enabled_mods = tab
        .apache_mods
        .iter()
        .filter(|module| module.enabled)
        .count();
    let runtime_status = if tab.installed_tools.composer_version.is_some()
        || tab.installed_tools.node_version.is_some()
        || tab.installed_tools.npm_version.is_some()
        || tab.installed_tools.redis_installed
    {
        runtime_summary(&tab.installed_tools)
    } else {
        tr(keys::SCAN_NEEDED).to_string()
    };

    container(
        column![
            row![
                text(tr(keys::SYSTEM_SCAN))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_SECONDARY)),
                Space::with_width(Length::Fill),
                if tab.tools_scanning {
                    text(tr(keys::SCANNING))
                        .size(11)
                        .color(theme::color(theme_keys::TEAL))
                } else {
                    text(tr(keys::SCAN_NEEDED))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                },
            ]
            .align_y(Alignment::Center),
            Space::with_height(12),
            row![
                ui::surface_metric(
                    tr(keys::PHP_DETECTED),
                    format!("{} {}", installed_php, tr(keys::STATUS_INSTALLED)),
                ),
                ui::surface_metric(
                    tr(keys::APACHE_MODULES_DETECTED),
                    format!("{} {}", enabled_mods, tr(keys::STATUS_ENABLED)),
                ),
                ui::surface_metric(tr(keys::REDIS), redis_status(&tab.installed_tools)),
                ui::surface_metric(tr(keys::RUNTIME_STATUS), runtime_status),
            ]
            .spacing(12),
        ]
        .spacing(0),
    )
    .padding(Padding::from([14, 16]))
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn runtime_summary(tools: &InstalledTools) -> String {
    if let Some(version) = tools.node_version.as_deref() {
        return format!("Node {}", version);
    }

    if let Some(version) = tools.composer_version.as_deref() {
        return format!("Composer {}", version);
    }

    redis_status(tools).to_string()
}

fn redis_status(tools: &InstalledTools) -> &'static str {
    if !tools.redis_installed {
        tr(keys::REDIS_NOT_INSTALLED)
    } else if tools.redis_running {
        tr(keys::REDIS_RUNNING)
    } else {
        tr(keys::REDIS_STOPPED)
    }
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
                            radius: 6.0.into(),
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
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    },
                })
                .into()
        })
        .collect();
    row(tabs).spacing(8).into()
}
