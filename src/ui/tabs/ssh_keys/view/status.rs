use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::ssh_keys::StatusKind;
use crate::messages::Message;
use crate::ui::tabs::ssh_keys::SshKeysTab;
use iced::widget::{Space, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn status_bar(tab: &SshKeysTab) -> Element<'_, Message> {
    if tab.status_kind == StatusKind::None || tab.status_message.is_empty() {
        return Space::with_height(0).into();
    }
    let (color, border_color, icon) = match tab.status_kind {
        StatusKind::Success => (
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            "+",
        ),
        StatusKind::Error => (
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BORDER),
            "x",
        ),
        StatusKind::Info => (
            theme::color(theme_keys::BLUE),
            theme::color(theme_keys::BLUE_BORDER),
            "i",
        ),
        StatusKind::None => (
            theme::color(theme_keys::TEXT_MUTED),
            theme::color(theme_keys::BORDER_SUBTLE),
            "",
        ),
    };
    container(
        row![
            container(
                text(icon)
                    .size(crate::core::app_config::text_metrics().tiny)
                    .color(theme::color(theme_keys::TEXT_ON_ACCENT))
            )
            .padding(Padding::from([3, 6]))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(color.into()),
                border: Border {
                    radius: 20.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(10),
            text(&tab.status_message)
                .size(crate::core::app_config::text_metrics().body)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([12, 16]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
