use crate::core::theme::{self, theme_map as theme_keys};
use iced::widget::{Space, container};
use iced::{Border, Color, Element, Length};

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
