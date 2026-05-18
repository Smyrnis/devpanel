use crate::core::theme::{self, theme_map as theme_keys};
use crate::ui::icons::{self, Icon};
use iced::widget::{Space, container, row};
use iced::{Alignment, Border, Color, Element, Length, Padding};

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

pub fn info_banner<'a, Message>(
    icon: Icon,
    content: Element<'a, Message>,
    color: Color,
    background: Color,
    border: Color,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        row![
            icons::solid(icon, 13.0, color),
            Space::with_width(9),
            content
        ]
        .align_y(Alignment::Start),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(background.into()),
        border: Border {
            color: border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
