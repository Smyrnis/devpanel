use crate::core::theme::{self, theme_map as theme_keys};
use iced::widget::{button, text};
use iced::{Border, Color, Element, Padding};

pub fn primary_button<'a, Message>(label: &'a str, msg: Message) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    button(
        text(label)
            .size(13)
            .color(theme::color(theme_keys::TEXT_ON_ACCENT)),
    )
    .on_press(msg)
    .padding(Padding::from([9, 16]))
    .style(|_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                text_color: theme::color(theme_keys::TEXT_ON_ACCENT),
                border: Border {
                    color: theme::color(theme_keys::TEAL_BORDER),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(theme::color(theme_keys::TEAL).into()),
            text_color: theme::color(theme_keys::TEXT_ON_ACCENT),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    })
    .into()
}

pub fn secondary_button<'a, Message>(label: &'a str, msg: Message) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    button(
        text(label)
            .size(13)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
    )
    .on_press(msg)
    .padding(Padding::from([9, 16]))
    .style(ghost_button_style())
    .into()
}

pub fn action_button<'a, Message>(
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    border: Color,
    on_press: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    action_button_with_style(
        label,
        on_press,
        ActionButtonStyle {
            color,
            bg,
            bg_hover,
            border,
            padding: [7, 14],
            radius: 6.0,
        },
    )
}

pub fn compact_action_button<'a, Message>(
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    border: Color,
    on_press: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    action_button_with_style(
        label,
        on_press,
        ActionButtonStyle {
            color,
            bg,
            bg_hover,
            border,
            padding: [6, 12],
            radius: 6.0,
        },
    )
}

struct ActionButtonStyle {
    color: Color,
    bg: Color,
    bg_hover: Color,
    border: Color,
    padding: [u16; 2],
    radius: f32,
}

fn action_button_with_style<'a, Message>(
    label: &'a str,
    on_press: Option<Message>,
    style: ActionButtonStyle,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let btn = button(text(label).size(12).color(style.color))
        .padding(Padding::from(style.padding))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(style.bg_hover.into()),
                    text_color: style.color,
                    border: Border {
                        color: style.border,
                        width: 1.0,
                        radius: style.radius.into(),
                    },
                    ..Default::default()
                }
            }
            _ => iced::widget::button::Style {
                background: Some(style.bg.into()),
                text_color: style.color,
                border: Border {
                    color: style.border,
                    width: 1.0,
                    radius: style.radius.into(),
                },
                ..Default::default()
            },
        });

    if let Some(msg) = on_press {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

pub fn ghost_button_style()
-> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(theme::color(theme_keys::BG_HOVER).into()),
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
            background: Some(theme::color(theme_keys::BG_CARD).into()),
            text_color: theme::color(theme_keys::TEXT_SECONDARY),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        },
    }
}
