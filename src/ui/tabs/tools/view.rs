use super::ToolsTab;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::tools::{PhpStatus, ToolSection};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::icons::Icon;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Padding};

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

    scrollable(
        column![
            header_fn(tr(keys::TITLE), tr(keys::SUBTITLE), vec![],),
            Space::with_height(18),
            tools_control_row(tab),
            Space::with_height(10),
            tool_sections(tab),
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

fn tools_control_row(tab: &ToolsTab) -> Element<'_, Message> {
    container(
        column![
            row![
                column![
                    text(
                        tab.active_section
                            .as_ref()
                            .map(section_label)
                            .unwrap_or_else(|| tr(keys::TITLE))
                    )
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    Space::with_height(3),
                    text(tools_summary(tab))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .spacing(0)
                .width(Length::Fill),
                text(if tab.tools_scanning || tab.scanning || tab.mods_scanning {
                    tr(keys::SCANNING)
                } else {
                    ""
                })
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(0),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(ui::surface_style())
    .into()
}

fn tool_sections(tab: &ToolsTab) -> Element<'_, Message> {
    let sections = [
        ToolSection::Php,
        ToolSection::PhpExts,
        ToolSection::ApacheMods,
        ToolSection::Database,
        ToolSection::Runtimes,
    ];

    ui::row_group(
        sections
            .iter()
            .map(|section| tool_section_block(tab, section.clone()))
            .collect(),
    )
}

fn tool_section_block(tab: &ToolsTab, section: ToolSection) -> Element<'_, Message> {
    let expanded = tab.active_section.as_ref() == Some(&section);
    let row = ui::summary_row(
        section_icon(&section),
        section_label(&section),
        section_summary(tab, &section),
        section_tone(tab, &section),
        vec![section_action(tab, &section)],
        expanded,
        Some(Message::Tools(ToolsMessage::ToggleSection(section.clone()))),
    );

    if !expanded {
        return row;
    }

    column![row, active_panel(tab, section)].spacing(6).into()
}

fn active_panel(tab: &ToolsTab, section: ToolSection) -> Element<'_, Message> {
    match section {
        ToolSection::Php => php_panel(tab),
        ToolSection::ApacheMods => apache_mods_panel(tab),
        ToolSection::PhpExts => php_exts_panel(tab),
        ToolSection::Runtimes => runtimes_panel(tab),
        ToolSection::Database => db_panel(tab),
    }
}

fn section_action<'a>(tab: &ToolsTab, section: &ToolSection) -> Element<'a, Message> {
    if tab.active_section.as_ref() != Some(section) {
        return ui::compact_action_button(
            tr(keys::MANAGE),
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_HOVER),
            theme::color(theme_keys::TEAL_BORDER),
            Some(Message::Tools(ToolsMessage::ToggleSection(section.clone()))),
        );
    }

    match section {
        ToolSection::Php => ui::compact_action_button(
            tr(keys::SCAN),
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_HOVER),
            theme::color(theme_keys::TEAL_BORDER),
            if tab.scanning {
                None
            } else {
                Some(Message::Tools(ToolsMessage::ScanPhp))
            },
        ),
        ToolSection::PhpExts => ui::compact_action_button(
            tr(keys::SCAN),
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_HOVER),
            theme::color(theme_keys::TEAL_BORDER),
            Some(Message::Tools(ToolsMessage::ScanPhpExts)),
        ),
        ToolSection::ApacheMods => ui::compact_action_button(
            tr(keys::SCAN),
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_HOVER),
            theme::color(theme_keys::TEAL_BORDER),
            if tab.mods_scanning {
                None
            } else {
                Some(Message::Tools(ToolsMessage::ScanApacheMods))
            },
        ),
        ToolSection::Database => ui::compact_action_button(
            tr(keys::MYSQL_MARIADB),
            theme::color(theme_keys::BLUE),
            theme::color(theme_keys::BLUE_BG),
            theme::color(theme_keys::BLUE_HOVER),
            theme::color(theme_keys::BLUE_BORDER),
            Some(Message::Tools(ToolsMessage::OpenMysqlCli)),
        ),
        ToolSection::Runtimes => ui::compact_action_button(
            tr(keys::SCAN),
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_HOVER),
            theme::color(theme_keys::TEAL_BORDER),
            if tab.tools_scanning {
                None
            } else {
                Some(Message::Tools(ToolsMessage::ScanInstalledTools))
            },
        ),
    }
}

fn section_summary(tab: &ToolsTab, section: &ToolSection) -> String {
    match section {
        ToolSection::Php => {
            let installed = installed_php_count(tab);
            let active = active_php_label(tab);
            format!("{} {}, {}", installed, tr(keys::STATUS_INSTALLED), active)
        }
        ToolSection::PhpExts => {
            let installed = tab
                .php_exts
                .iter()
                .filter(|extension| extension.installed)
                .count();
            format!("{} {}", installed, tr(keys::STATUS_INSTALLED))
        }
        ToolSection::ApacheMods => {
            let enabled = enabled_mod_count(tab);
            format!("{} {}", enabled, tr(keys::STATUS_ENABLED))
        }
        ToolSection::Database => {
            if tab.db_status.is_empty() {
                tr(keys::MYSQL_MARIADB).to_string()
            } else {
                tab.db_status.clone()
            }
        }
        ToolSection::Runtimes => runtimes_summary(tab),
    }
}

fn section_tone(tab: &ToolsTab, section: &ToolSection) -> ui::BadgeTone {
    match section {
        ToolSection::Php if installed_php_count(tab) > 0 => ui::BadgeTone::Success,
        ToolSection::PhpExts if tab.php_exts.iter().any(|extension| extension.installed) => {
            ui::BadgeTone::Success
        }
        ToolSection::ApacheMods if enabled_mod_count(tab) > 0 => ui::BadgeTone::Success,
        ToolSection::Database => ui::BadgeTone::Info,
        ToolSection::Runtimes
            if tab.installed_tools.composer_version.is_some()
                || tab.installed_tools.node_version.is_some()
                || tab.installed_tools.redis_installed =>
        {
            ui::BadgeTone::Success
        }
        _ => ui::BadgeTone::Neutral,
    }
}

fn section_icon(section: &ToolSection) -> Icon {
    match section {
        ToolSection::Php => Icon::Php,
        ToolSection::ApacheMods => Icon::Apache,
        ToolSection::PhpExts => Icon::Code,
        ToolSection::Runtimes => Icon::Tools,
        ToolSection::Database => Icon::Database,
    }
}

fn tools_summary(tab: &ToolsTab) -> String {
    format!(
        "{} PHP, {} Apache modules, {} extensions",
        installed_php_count(tab),
        enabled_mod_count(tab),
        tab.php_exts
            .iter()
            .filter(|extension| extension.installed)
            .count()
    )
}

fn installed_php_count(tab: &ToolsTab) -> usize {
    tab.php_releases
        .iter()
        .filter(|release| release.status == PhpStatus::Installed)
        .count()
}

fn enabled_mod_count(tab: &ToolsTab) -> usize {
    tab.apache_mods
        .iter()
        .filter(|module| module.enabled)
        .count()
}

fn active_php_label(tab: &ToolsTab) -> String {
    tab.php_releases
        .iter()
        .find(|release| release.is_active)
        .map(|release| format!("PHP {} {}", release.version, tr(keys::ACTIVE)))
        .unwrap_or_else(|| tr(keys::SCAN_NEEDED).to_string())
}

fn runtimes_summary(tab: &ToolsTab) -> String {
    if tab.installed_tools.redis_installed {
        return if tab.installed_tools.redis_running {
            tr(keys::REDIS_RUNNING).to_string()
        } else {
            tr(keys::REDIS_STOPPED).to_string()
        };
    }

    if let Some(version) = tab.installed_tools.node_version.as_deref() {
        return format!("Node {}", version);
    }

    if let Some(version) = tab.installed_tools.composer_version.as_deref() {
        return format!("Composer {}", version);
    }

    tr(keys::SCAN_NEEDED).to_string()
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
