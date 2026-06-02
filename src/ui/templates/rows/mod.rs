#![allow(dead_code)]

use crate::core::app_config;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::ui::icons::{self, Icon};
use crate::ui::templates::badges::{BadgeTone, status_badge};
use iced::widget::{Space, button, column, container, mouse_area, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding, mouse};

pub fn summary_row<'a, Message>(
    icon: Icon,
    title: &'a str,
    status: impl Into<String>,
    tone: BadgeTone,
    actions: Vec<Element<'a, Message>>,
    expanded: bool,
    on_toggle: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let arrow = if expanded {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    };

    let main_content = row![
        icons::solid_box(icon, 15.0, theme::color(theme_keys::TEXT_SECONDARY), 22.0),
        Space::with_width(12),
        text(title)
            .size(app_config::text_metrics().section_title)
            .color(theme::color(theme_keys::TEXT_PRIMARY))
            .width(Length::FillPortion(2)),
        status_badge(status, tone),
        Space::with_width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let main_content: Element<Message> = if let Some(message) = on_toggle.clone() {
        mouse_area(container(main_content).width(Length::Fill))
            .on_press(message)
            .interaction(mouse::Interaction::Pointer)
            .into()
    } else {
        container(main_content).width(Length::Fill).into()
    };

    let toggle: Element<Message> = if let Some(message) = on_toggle {
        button(icons::solid_box(
            arrow,
            13.0,
            theme::color(theme_keys::TEXT_SECONDARY),
            20.0,
        ))
        .on_press(message)
        .padding(Padding::from([7, 8]))
        .height(Length::Fixed(
            app_config::control_metrics().summary_row_height,
        ))
        .style(crate::ui::templates::buttons::ghost_button_style())
        .into()
    } else {
        icons::solid_box(arrow, 13.0, theme::color(theme_keys::TEXT_MUTED), 34.0)
    };

    container(
        row![
            main_content,
            row(actions).spacing(8).align_y(Alignment::Center),
            toggle,
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| row_style(expanded))
    .into()
}

pub fn expanded_panel<'a, Message>(children: Vec<Element<'a, Message>>) -> Element<'a, Message>
where
    Message: 'a,
{
    container(column(children).spacing(14))
        .padding(Padding::from([14, 16]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_CARD).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}

pub fn row_group<'a, Message>(rows: Vec<Element<'a, Message>>) -> Element<'a, Message>
where
    Message: 'a,
{
    container(column(rows).spacing(8))
        .width(Length::Fill)
        .into()
}

pub fn panel_section<'a, Message>(
    title: &'a str,
    children: Vec<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    column![
        text(title)
            .size(app_config::text_metrics().caption)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        column(children).spacing(8),
    ]
    .spacing(8)
    .into()
}

pub fn detail_row<'a, Message>(label: &'a str, value: Element<'a, Message>) -> Element<'a, Message>
where
    Message: 'a,
{
    row![
        text(label)
            .size(app_config::text_metrics().caption)
            .color(theme::color(theme_keys::TEXT_MUTED))
            .width(Length::Fixed(
                app_config::control_metrics().detail_label_width
            )),
        value,
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

pub fn detail_text<'a, Message>(value: &'a str) -> Element<'a, Message>
where
    Message: 'a,
{
    text(value)
        .size(app_config::text_metrics().caption)
        .color(theme::color(theme_keys::TEXT_SECONDARY))
        .into()
}

pub fn status_banner<'a, Message>(ok: bool, message: &'a str) -> Element<'a, Message>
where
    Message: 'a,
{
    let (color, bg) = if ok {
        (
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
        )
    } else {
        (
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
        )
    };

    container(
        row![
            status_dot(color),
            Space::with_width(8),
            text(message)
                .size(app_config::text_metrics().caption)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 14]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(bg.into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn status_dot<'a, Message>(color: Color) -> Element<'a, Message>
where
    Message: 'a,
{
    container(Space::with_width(6))
        .width(6)
        .height(6)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(color.into()),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn row_style(expanded: bool) -> container::Style {
    container::Style {
        background: Some(
            if expanded {
                theme::color(theme_keys::BG_HOVER)
            } else {
                theme::color(theme_keys::BG_SURFACE)
            }
            .into(),
        ),
        border: Border {
            color: if expanded {
                theme::color(theme_keys::BORDER_MED)
            } else {
                theme::color(theme_keys::BORDER_SUBTLE)
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
