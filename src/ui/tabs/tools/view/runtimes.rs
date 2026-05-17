use super::shared::small_action_btn;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::tabs::tools::{InstalledTools, ToolsTab};
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub(super) fn runtimes_panel(tab: &ToolsTab) -> Element<'_, Message> {
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
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
