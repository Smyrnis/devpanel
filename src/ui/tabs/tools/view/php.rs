use super::shared::{search_box, section_header, small_action_btn, tool_item_row};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::tools::{PhpRelease, PhpStatus};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::icons::Icon;
use crate::ui::tabs::tools::ToolsTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn php_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let scan_lbl = if tab.scanning {
        tr(keys::SCANNING)
    } else {
        tr(keys::SCAN)
    };
    let header = section_header(
        tr(keys::SECTION_PHP_VERSIONS),
        tr(keys::PHP_VERSIONS_HELP),
        scan_lbl,
        if tab.scanning {
            None
        } else {
            Some(Message::Tools(ToolsMessage::ScanPhp))
        },
    );

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
            search_box(tr(keys::SEARCH_PLACEHOLDER), &tab.tool_search),
            Space::with_height(14),
            ui::thin_line(),
            Space::with_height(14),
            column(rows).spacing(8),
            Space::with_height(16),
            ui::info_banner(
                Icon::Info,
                column![
                    text(tr(keys::PHP_PPA_NOTE))
                        .size(crate::core::app_config::text_metrics().caption)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_height(3),
                    text(tr(keys::APACHE_MOD_NOTE))
                        .size(crate::core::app_config::text_metrics().caption)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .spacing(0)
                .into(),
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_BORDER),
            ),
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

    let mod_name = format!("php{}-fpm", r.version);
    let mod_status_lbl = if r.apache_mod_available {
        if r.apache_mod_enabled {
            tr(keys::STATUS_ENABLED)
        } else {
            tr(keys::STATUS_DISABLED)
        }
    } else {
        tr(keys::STATUS_NOT_AVAILABLE)
    };
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
                .size(crate::core::app_config::text_metrics().tiny)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        )
        .padding(Padding::from([6, 0]))
        .into()
    };

    let mut subtitle = format!("{}: {}", tr(keys::APACHE_MOD), mod_status_lbl);
    if r.is_active {
        subtitle = format!("{} · {}", tr(keys::ACTIVE), subtitle);
    }
    if is_php56 {
        subtitle = format!("{} · {}", tr(keys::EOL), subtitle);
    }

    let card: Element<Message> = tool_item_row(
        format!("PHP {}", r.version),
        subtitle,
        status_label,
        status_color,
        row![apache_btn, apt_btn]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
    );

    let ppa_hint: Element<Message> = if is_php56 && r.status != PhpStatus::Installed {
        column![
            Space::with_height(4),
            container(
                row![
                    text("!")
                        .size(crate::core::app_config::text_metrics().badge)
                        .color(theme::color(theme_keys::YELLOW)),
                    Space::with_width(6),
                    text(tr(keys::PHP56_PPA_HINT))
                        .size(crate::core::app_config::text_metrics().tiny)
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
