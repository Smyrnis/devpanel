#![allow(dead_code)]

use crate::core::app_config;
use crate::core::theme::{self, theme_map as theme_keys};
use iced::widget::{container, text};
use iced::{Border, Color, Element, Length, Padding};

#[derive(Clone, Copy)]
pub enum BadgeTone {
    Neutral,
    Success,
    Danger,
    Warning,
    Info,
}

impl BadgeTone {
    fn colors(self) -> (Color, Color, Color) {
        match self {
            Self::Neutral => (
                theme::color(theme_keys::TEXT_SECONDARY),
                theme::color(theme_keys::BG_CARD),
                theme::color(theme_keys::BORDER_SUBTLE),
            ),
            Self::Success => (
                theme::color(theme_keys::GREEN),
                theme::color(theme_keys::GREEN_BG),
                theme::color(theme_keys::GREEN_DIM),
            ),
            Self::Danger => (
                theme::color(theme_keys::RED),
                theme::color(theme_keys::RED_BG),
                theme::color(theme_keys::RED_BORDER),
            ),
            Self::Warning => (
                theme::color(theme_keys::ORANGE),
                theme::color(theme_keys::YELLOW_BG),
                theme::color(theme_keys::YELLOW_BORDER),
            ),
            Self::Info => (
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_BORDER),
            ),
        }
    }
}

pub fn status_badge<'a, Message>(label: impl Into<String>, tone: BadgeTone) -> Element<'a, Message>
where
    Message: 'a,
{
    pill(label, tone, app_config::text_metrics().caption, [4, 9])
}

pub fn small_badge<'a, Message>(label: impl Into<String>, tone: BadgeTone) -> Element<'a, Message>
where
    Message: 'a,
{
    pill(label, tone, app_config::text_metrics().tiny, [3, 7])
}

pub fn path_chip<'a, Message>(path: &'a str) -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        text(path)
            .size(app_config::text_metrics().caption)
            .font(iced::Font::MONOSPACE)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
    )
    .padding(Padding::from([5, 8]))
    .width(Length::Shrink)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_CARD).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn pill<'a, Message>(
    label: impl Into<String>,
    tone: BadgeTone,
    size: u16,
    padding: [u16; 2],
) -> Element<'a, Message>
where
    Message: 'a,
{
    let (text_color, bg, border) = tone.colors();
    container(text(label.into()).size(size).color(text_color))
        .padding(Padding::from(padding))
        .width(Length::Shrink)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(bg.into()),
            border: Border {
                color: border,
                width: 1.0,
                radius: 999.0.into(),
            },
            ..Default::default()
        })
        .into()
}
