use crate::core::{
    app_config,
    theme::{self, theme_map as theme_keys},
};
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Element, Length};

pub fn page_header<'a, Message>(
    title: &'a str,
    description: &'a str,
    actions: Vec<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let text_metrics = app_config::text_metrics();
    let action_area: Element<Message> = if actions.is_empty() {
        Space::with_width(0).into()
    } else {
        row(actions).spacing(8).align_y(Alignment::Center).into()
    };

    container(
        row![
            column![
                text(title)
                    .size(text_metrics.title)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(4),
                text(description)
                    .size(text_metrics.body)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .width(Length::Fill),
            action_area,
        ]
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}

pub fn page_header_compact<'a, Message>(
    title: &'a str,
    description: &'a str,
    actions: Vec<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let text_metrics = app_config::text_metrics();
    let content: Element<Message> = if actions.is_empty() {
        column![
            text(title)
                .size(text_metrics.title)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(4),
            text(description)
                .size(text_metrics.body)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .into()
    } else {
        column![
            text(title)
                .size(text_metrics.title)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(4),
            text(description)
                .size(text_metrics.body)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(12),
            row(actions).spacing(8).align_y(Alignment::Center),
        ]
        .spacing(0)
        .into()
    };

    container(content).width(Length::Fill).into()
}
