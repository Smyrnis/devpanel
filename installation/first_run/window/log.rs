use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::install as keys, text as tr};
use crate::messages::Message;
use iced::widget::{Space, column, container, scrollable, text};
use iced::{Border, Element, Length, Padding};

pub(super) fn setup_log_panel<'a>(log_lines: &'a [String]) -> Element<'a, Message> {
    let rows: Vec<Element<Message>> = log_lines
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|line| {
            text(line.as_str())
                .size(10)
                .color(theme::color(theme_keys::TEXT_MUTED))
                .into()
        })
        .collect();
    container(
        column![
            text(tr(keys::SETUP_LOG))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(6),
            scrollable(column(rows).spacing(3)).height(110),
        ]
        .spacing(0),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_BASE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}
