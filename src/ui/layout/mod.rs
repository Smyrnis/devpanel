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
            row(actions).spacing(8).align_y(Alignment::Center),
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
    container(
        column![
            text(title)
                .size(22)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(4),
            text(description)
                .size(13)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(if actions.is_empty() { 0 } else { 12 }),
            row(actions).spacing(8).align_y(Alignment::Center),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .into()
}
