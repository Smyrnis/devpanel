use crate::core::theme::{self, theme_map as theme_keys};
use crate::messages::Message;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub(super) fn small_action_btn<'a>(
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    msg: Message,
) -> Element<'a, Message> {
    button(text(label).size(12).color(color))
        .on_press(msg)
        .padding(Padding::from([6, 14]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(bg_hover.into()),
                    text_color: color,
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            _ => iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: color,
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .into()
}
pub(super) fn db_btn<'a>(
    title: &'a str,
    subtitle: &'a str,
    accent: Color,
    bg: Color,
    bg_hover: Color,
    _border: Color,
    msg: Message,
) -> Element<'a, Message> {
    button(
        row![
            container(Space::with_width(4))
                .width(4)
                .height(28)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(accent.into()),
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
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
        ]
        .align_y(Alignment::Center),
    )
    .on_press(msg)
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(bg_hover.into()),
                text_color: theme::color(theme_keys::TEXT_PRIMARY),
                border: Border {
                    color: theme::color(theme_keys::BORDER_MED),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: theme::color(theme_keys::TEXT_PRIMARY),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        },
    })
    .into()
}
