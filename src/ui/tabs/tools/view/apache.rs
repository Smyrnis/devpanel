use super::shared::{section_header, small_action_btn, tool_item_row};
use crate::core::paths;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::tools::ApacheModule;
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::tabs::tools::ToolsTab;
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{Space, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn apache_mods_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let scan_lbl = if tab.mods_scanning {
        tr(keys::SCANNING)
    } else {
        tr(keys::SCAN)
    };
    let header = section_header(
        tr(keys::SECTION_APACHE_MODULES),
        format!(
            "{} {} - {}",
            tr(keys::APACHE_MODULES_HELP_PREFIX),
            paths::APACHE_MODS_AVAILABLE,
            tr(keys::APACHE_MODULES_HELP_SUFFIX),
        ),
        scan_lbl,
        if tab.mods_scanning {
            None
        } else {
            Some(Message::Tools(ToolsMessage::ScanApacheMods))
        },
    );

    let total = tab.apache_mods.len();
    let enabled = tab.apache_mods.iter().filter(|m| m.enabled).count();

    let filter_row = row![
        text_input(tr(keys::FILTER_MODULES), &tab.mod_filter)
            .on_input(|v| Message::Tools(ToolsMessage::ModFilterChanged(v)))
            .padding(Padding::from([7, 12]))
            .size(12)
            .style(styles::text_input_style)
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
    tool_item_row(
        format!("mod_{}", m.name),
        paths::APACHE_MODS_AVAILABLE,
        status_text,
        dot_color,
        action,
    )
}
