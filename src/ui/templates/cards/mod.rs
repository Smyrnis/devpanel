use crate::core::theme::{self, theme_map as theme_keys};
use crate::ui::icons::{self, Icon};
use iced::widget::{Space, column, container, row, text};
use iced::{Border, Color, Element, Length, Padding};

pub fn card_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub fn card_style_with_border(border_color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

pub fn surface_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(theme::color(theme_keys::BG_CARD).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn metric_icon<'a, Message: 'a>(icon: Icon) -> Element<'a, Message> {
    icons::solid_box(icon, 15.0, theme::color(theme_keys::TEXT_MUTED), 18.0)
}

pub fn metric_card_icon<'a, Message>(
    icon: Icon,
    label: &'a str,
    value: impl Into<String>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        row![
            metric_icon(icon),
            Space::with_width(10),
            column![
                text(label)
                    .size(10)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(4),
                text(value.into())
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
            ]
            .spacing(0),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::FillPortion(1))
    .style(card_style())
    .into()
}

pub fn surface_metric_icon<'a, Message>(
    icon: Icon,
    label: &'a str,
    value: impl Into<String>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        row![
            metric_icon(icon),
            Space::with_width(10),
            column![
                text(label)
                    .size(10)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(4),
                text(value.into())
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
            ]
            .spacing(0),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::FillPortion(1))
    .style(surface_style())
    .into()
}
