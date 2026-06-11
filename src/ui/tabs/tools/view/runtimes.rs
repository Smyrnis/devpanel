use super::shared::{search_box, section_header, small_action_btn};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::tools::InstalledTools;
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::tabs::tools::ToolsTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn runtimes_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let tools = &tab.installed_tools;
    let q = tab.tool_search.to_lowercase();
    let mut cards: Vec<Element<Message>> = Vec::new();
    let candidates = [("redis cache memory", runtime_redis_card(tools))];
    for (terms, card) in candidates {
        if q.is_empty() || terms.contains(&q) {
            cards.push(card);
        }
    }
    if cards.is_empty() {
        cards.push(
            container(
                text(tr(keys::NO_TOOLS_MATCH))
                    .size(crate::core::app_config::text_metrics().body)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            )
            .padding(Padding::from([20, 16]))
            .into(),
        );
    }

    container(
        column![
            section_header(
                tr(keys::REDIS_RUNTIME),
                tr(keys::RUNTIMES_HELP),
                if tab.tools_scanning {
                    tr(keys::SCANNING)
                } else {
                    tr(keys::SCAN)
                },
                if tab.tools_scanning {
                    None
                } else {
                    Some(Message::Tools(ToolsMessage::ScanInstalledTools))
                },
            ),
            Space::with_height(18),
            search_box(tr(keys::SEARCH_PLACEHOLDER), &tab.tool_search),
            Space::with_height(14),
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
    color: iced::Color,
    action: Element<'_, Message>,
) -> Element<'_, Message> {
    let status = if color == theme::color(theme_keys::GREEN) {
        tr(keys::STATUS_INSTALLED)
    } else {
        tr(keys::STATUS_NOT_INSTALLED)
    };
    container(
        column![
            row![
                ui::status_dot(color),
                Space::with_width(12),
                column![
                    text(title)
                        .size(crate::core::app_config::text_metrics().body)
                        .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    Space::with_height(2),
                    text(subtitle)
                        .size(crate::core::app_config::text_metrics().caption)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .spacing(0)
                .width(Length::Fill),
                ui::small_badge(status, runtime_status_tone(color)),
            ]
            .align_y(Alignment::Center),
            action,
        ]
        .spacing(10),
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
    .into()
}

fn runtime_status_tone(color: iced::Color) -> ui::BadgeTone {
    if color == theme::color(theme_keys::GREEN) {
        ui::BadgeTone::Success
    } else {
        ui::BadgeTone::Neutral
    }
}
