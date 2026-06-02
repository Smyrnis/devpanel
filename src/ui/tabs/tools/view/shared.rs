use crate::core::theme::{self, theme_map as theme_keys};
use crate::messages::Message;
use crate::messages::ToolsMessage;
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{Space, column, container, row, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub(super) fn small_action_btn<'a>(
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    msg: Message,
) -> Element<'a, Message> {
    ui::compact_action_button(label, color, bg, bg_hover, bg_hover, Some(msg))
}

pub(super) fn section_header<'a>(
    title: &'a str,
    subtitle: impl Into<String>,
    action_label: &'a str,
    action: Option<Message>,
) -> Element<'a, Message> {
    row![
        column![
            text(title)
                .size(crate::core::app_config::text_metrics().section_title)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(3),
            text(subtitle.into())
                .size(crate::core::app_config::text_metrics().caption)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .width(Length::Fill),
        ui::compact_action_button(
            action_label,
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_HOVER),
            theme::color(theme_keys::TEAL_BORDER),
            action,
        ),
    ]
    .align_y(Alignment::Center)
    .into()
}

pub(super) fn tool_item_row<'a>(
    title: impl Into<String>,
    subtitle: impl Into<String>,
    status_label: impl Into<String>,
    status_color: Color,
    action: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        row![
            ui::status_dot(status_color),
            Space::with_width(12),
            column![
                text(title.into())
                    .size(crate::core::app_config::text_metrics().body)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(2),
                text(subtitle.into())
                    .size(crate::core::app_config::text_metrics().caption)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .width(Length::Fill),
            ui::small_badge(status_label, status_tone(status_color)),
            Space::with_width(8),
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

pub(super) fn search_box<'a>(placeholder: &'a str, value: &'a str) -> Element<'a, Message> {
    text_input(placeholder, value)
        .on_input(|v| Message::Tools(ToolsMessage::ToolSearchChanged(v)))
        .padding(Padding::from([7, 12]))
        .size(crate::core::app_config::text_metrics().caption)
        .style(styles::text_input_style)
        .width(Length::Fill)
        .into()
}

fn status_tone(color: Color) -> ui::BadgeTone {
    if color == theme::color(theme_keys::GREEN) {
        ui::BadgeTone::Success
    } else if color == theme::color(theme_keys::YELLOW) {
        ui::BadgeTone::Warning
    } else if color == theme::color(theme_keys::RED) {
        ui::BadgeTone::Danger
    } else {
        ui::BadgeTone::Neutral
    }
}
