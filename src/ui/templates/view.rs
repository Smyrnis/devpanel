use crate::core::theme::{self, theme_map as theme_keys};
use iced::widget::{Space, container};
use iced::{Border, Color, Element, Length};

pub fn card_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(theme::color(theme_keys::BG_CARD).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

pub fn card_style_with_border(border_color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(theme::color(theme_keys::BG_CARD).into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

pub fn surface_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

pub fn thin_line<'a, Message: 'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_| container::Style {
            background: Some(theme::color(theme_keys::BORDER_SUBTLE).into()),
            ..Default::default()
        })
}

pub fn divider<'a, Message>() -> Element<'a, Message>
where
    Message: 'a,
{
    thin_line().into()
}

pub fn dot<Message: 'static>(color: Color, size: f32) -> iced::widget::Container<'static, Message> {
    container(Space::with_width(size))
        .width(size)
        .height(size)
        .style(move |_| container::Style {
            background: Some(color.into()),
            border: Border {
                radius: (size / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
}

pub fn status_dot<Message: 'static>(color: Color) -> iced::widget::Container<'static, Message> {
    dot(color, 6.0)
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
                    radius: 8.0.into(),
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
                radius: 8.0.into(),
            },
            ..Default::default()
        },
    }
}
