use super::{InstalledTools, PhpStatus, ToolSection, ToolsTab};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::icons::{self, Icon};
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

pub fn render(tab: &ToolsTab, compact: bool) -> Element<'_, Message> {
    let header_fn = if compact {
        ui::page_header_compact
    } else {
        ui::page_header
    };
    let active_panel = match tab.active_section {
        ToolSection::Php => php_panel(tab),
        ToolSection::ApacheMods => apache_mods_panel(tab),
        ToolSection::PhpExts => php_exts_panel(tab),
        ToolSection::Runtimes => runtimes_panel(tab),
        ToolSection::Database => db_panel(tab),
    };
    let tools_body: Element<Message> = if compact {
        column![
            active_panel,
            Space::with_height(14),
            tool_detail_panel(tab, compact)
        ]
        .spacing(0)
        .into()
    } else {
        row![
            container(active_panel).width(Length::FillPortion(1)),
            Space::with_width(14),
            container(tool_detail_panel(tab, compact)).width(Length::FillPortion(1)),
        ]
        .align_y(Alignment::Start)
        .into()
    };

    scrollable(
        column![
            header_fn(
                tr(keys::TITLE),
                tr(keys::SUBTITLE),
                vec![ui::primary_icon_button(
                    Icon::Refresh,
                    tr(keys::SCAN_SYSTEM),
                    Message::Tools(ToolsMessage::ScanInstalledTools),
                )],
            ),
            Space::with_height(18),
            scan_summary(tab, compact),
            Space::with_height(14),
            section_tabs(tab, compact),
            Space::with_height(10),
            text_input(tr(keys::SEARCH_PLACEHOLDER), &tab.tool_search)
                .on_input(|v| Message::Tools(ToolsMessage::ToolSearchChanged(v)))
                .padding(Padding::from([7, 12]))
                .size(12)
                .style(styles::text_input_style),
            Space::with_height(16),
            tools_body,
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

fn tool_detail_panel(tab: &ToolsTab, _compact: bool) -> Element<'_, Message> {
    let active_php = tab
        .php_releases
        .iter()
        .find(|release| release.is_active)
        .or_else(|| {
            tab.php_releases
                .iter()
                .find(|release| release.status == PhpStatus::Installed)
        });
    let active_php_label = active_php
        .map(|release| format!("PHP {}", release.version))
        .unwrap_or_else(|| tr(keys::SCAN_NEEDED).to_string());
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

    container(
        column![
            row![
                icons::solid(Icon::Info, 15.0, theme::color(theme_keys::TEXT_SECONDARY)),
                Space::with_width(8),
                text(tr(keys::OVERVIEW))
                    .size(15)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
            ]
            .align_y(Alignment::Center),
            Space::with_height(14),
            detail_row(
                Icon::Php,
                tr(keys::ACTIVE_PHP),
                active_php_label,
                theme::color(theme_keys::PURPLE),
            ),
            detail_row(
                Icon::Tools,
                tr(keys::ACTIVE_SECTION),
                section_label(&tab.active_section).to_string(),
                theme::color(theme_keys::TEAL),
            ),
            detail_row(
                Icon::Php,
                tr(keys::INSTALLED_COUNT),
                installed_php.to_string(),
                theme::color(theme_keys::GREEN),
            ),
            detail_row(
                Icon::Apache,
                tr(keys::ENABLED_COUNT),
                enabled_mods.to_string(),
                theme::color(theme_keys::BLUE),
            ),
            Space::with_height(18),
            ui::divider(),
            Space::with_height(16),
            text(tr(keys::QUICK_ACTIONS))
                .size(13)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(10),
            quick_action_grid(tab),
            Space::with_height(18),
            ui::divider(),
            Space::with_height(16),
            text(tr(keys::COMMAND_PREVIEW))
                .size(13)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(4),
            text(tr(keys::COMMAND_PREVIEW_HELP))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(10),
            command_preview(tab),
        ]
        .spacing(0)
        .padding(Padding::from([18, 18])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn detail_row<'a>(
    icon: Icon,
    label: &'a str,
    value: String,
    accent: iced::Color,
) -> Element<'a, Message> {
    container(
        row![
            icons::solid(icon, 13.0, theme::color(theme_keys::TEXT_MUTED)),
            Space::with_width(10),
            text(label)
                .size(12)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_width(Length::Fill),
            text(value).size(12).color(accent),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([9, 10]))
    .width(Length::Fill)
    .style(ui::surface_style())
    .into()
}

fn quick_action_grid(tab: &ToolsTab) -> Element<'_, Message> {
    let scan_msg = match tab.active_section {
        ToolSection::Php => Message::Tools(ToolsMessage::ScanPhp),
        ToolSection::ApacheMods => Message::Tools(ToolsMessage::ScanApacheMods),
        ToolSection::PhpExts => Message::Tools(ToolsMessage::ScanPhpExts),
        ToolSection::Runtimes | ToolSection::Database => {
            Message::Tools(ToolsMessage::ScanInstalledTools)
        }
    };

    column![
        ui::secondary_icon_button(Icon::Refresh, tr(keys::SCAN), scan_msg),
        Space::with_height(8),
        ui::primary_icon_button(
            Icon::Refresh,
            tr(keys::SCAN_SYSTEM),
            Message::Tools(ToolsMessage::ScanInstalledTools),
        ),
    ]
    .spacing(0)
    .into()
}

fn command_preview(tab: &ToolsTab) -> Element<'_, Message> {
    let active_php = tab
        .php_releases
        .iter()
        .find(|release| release.is_active)
        .map(|release| release.version.as_str())
        .unwrap_or("8.2");
    let commands = match tab.active_section {
        ToolSection::Php => format!(
            "sudo a2dismod php*\nsudo a2enmod php{}\nsudo systemctl reload apache2\nsudo update-alternatives --set php /usr/bin/php{}",
            active_php, active_php
        ),
        ToolSection::ApacheMods => {
            "sudo a2enmod <module>\nsudo systemctl reload apache2".to_string()
        }
        ToolSection::PhpExts => format!("sudo apt-get install php{}-<extension>", active_php),
        ToolSection::Runtimes => {
            "composer self-update\nnode --version\nredis-cli INFO memory".to_string()
        }
        ToolSection::Database => "mysql -u root\nmariadb -u root".to_string(),
    };

    container(
        text(commands)
            .size(11)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
    )
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_CARD).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn scan_summary(tab: &ToolsTab, compact: bool) -> Element<'_, Message> {
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

    let metric_grid: Element<Message> = if compact {
        column![
            ui::surface_metric_icon(
                Icon::Php,
                tr(keys::PHP_DETECTED),
                format!("{} {}", installed_php, tr(keys::STATUS_INSTALLED)),
            ),
            ui::surface_metric_icon(
                Icon::Apache,
                tr(keys::APACHE_MODULES_DETECTED),
                format!("{} {}", enabled_mods, tr(keys::STATUS_ENABLED)),
            ),
            ui::surface_metric_icon(
                Icon::Database,
                tr(keys::REDIS),
                redis_status(&tab.installed_tools),
            ),
            ui::surface_metric_icon(Icon::Tools, tr(keys::RUNTIME_STATUS), runtime_status),
        ]
        .spacing(8)
        .into()
    } else {
        row![
            ui::surface_metric_icon(
                Icon::Php,
                tr(keys::PHP_DETECTED),
                format!("{} {}", installed_php, tr(keys::STATUS_INSTALLED)),
            ),
            ui::surface_metric_icon(
                Icon::Apache,
                tr(keys::APACHE_MODULES_DETECTED),
                format!("{} {}", enabled_mods, tr(keys::STATUS_ENABLED)),
            ),
            ui::surface_metric_icon(
                Icon::Database,
                tr(keys::REDIS),
                redis_status(&tab.installed_tools),
            ),
            ui::surface_metric_icon(Icon::Tools, tr(keys::RUNTIME_STATUS), runtime_status),
        ]
        .spacing(12)
        .into()
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
            metric_grid,
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

fn section_label(section: &ToolSection) -> &'static str {
    match section {
        ToolSection::Php => tr(keys::SECTION_PHP_VERSIONS),
        ToolSection::ApacheMods => tr(keys::SECTION_APACHE_MODULES),
        ToolSection::PhpExts => tr(keys::SECTION_PHP_EXTENSIONS),
        ToolSection::Runtimes => tr(keys::SECTION_RUNTIMES),
        ToolSection::Database => tr(keys::SECTION_DATABASE),
    }
}

fn section_tabs(tab: &ToolsTab, compact: bool) -> Element<'_, Message> {
    let sections = [
        (ToolSection::Php, tr(keys::SECTION_PHP_VERSIONS), Icon::Php),
        (
            ToolSection::ApacheMods,
            tr(keys::SECTION_APACHE_MODULES),
            Icon::Apache,
        ),
        (
            ToolSection::PhpExts,
            tr(keys::SECTION_PHP_EXTENSIONS),
            Icon::Code,
        ),
        (
            ToolSection::Runtimes,
            tr(keys::SECTION_RUNTIMES),
            Icon::Tools,
        ),
        (
            ToolSection::Database,
            tr(keys::SECTION_DATABASE),
            Icon::Database,
        ),
    ];
    let tabs: Vec<Element<Message>> = sections
        .iter()
        .map(|(sec, label, icon)| {
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
            button(
                row![
                    icons::solid_box(*icon, 12.0, color, 14.0),
                    Space::with_width(7),
                    text(*label).size(12).color(color),
                ]
                .align_y(Alignment::Center),
            )
            .on_press(msg)
            .padding(Padding::from([7, 16]))
            .style(move |_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                    iced::widget::button::Style {
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
                    }
                }
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
    if compact {
        column(tabs).spacing(8).into()
    } else {
        row(tabs).spacing(8).into()
    }
}
