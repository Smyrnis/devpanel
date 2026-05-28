use crate::core::theme::{self, theme_map as theme_keys};
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
    let action_area: Element<Message> = if actions.is_empty() {
        Space::with_width(0).into()
    } else {
        row(actions).spacing(8).align_y(Alignment::Center).into()
    };

    container(
        row![
            column![
                text(title)
                    .size(22)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(4),
                text(description)
                    .size(13)
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
    let content: Element<Message> = if actions.is_empty() {
        column![
            text(title)
                .size(22)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(4),
            text(description)
                .size(13)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .into()
    } else {
        column![
            text(title)
                .size(22)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(4),
            text(description)
                .size(13)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(12),
            row(actions).spacing(8).align_y(Alignment::Center),
        ]
        .spacing(0)
        .into()
    };

    container(content).width(Length::Fill).into()
}
