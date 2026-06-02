use crate::core::{
    app_config, dry_run,
    theme::{self, theme_map as theme_keys},
};
use crate::lang::{lang_map::install as keys, text as tr};
use crate::messages::Message;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Element, Padding};

pub(super) fn header<'a>() -> Element<'a, Message> {
    let text_metrics = app_config::text_metrics();
    let dry_run_note: Element<Message> = if dry_run::active() {
        column![
            Space::with_height(4),
            text(tr(keys::DRY_RUN_NOTE))
                .size(text_metrics.caption)
                .color(theme::color(theme_keys::YELLOW)),
        ]
        .spacing(0)
        .into()
    } else {
        Space::with_height(0).into()
    };

    column![
        row![
            container(
                text(tr(keys::BADGE_NEW))
                    .size(text_metrics.badge)
                    .color(theme::color(theme_keys::TEAL))
            )
            .padding(Padding::from([3, 8]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::TEAL_BG).into()),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(10),
            text(tr(keys::WELCOME_TITLE))
                .size(text_metrics.title)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
        ]
        .align_y(Alignment::Center),
        Space::with_height(8),
        text(tr(keys::WELCOME_BODY))
            .size(text_metrics.body)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
        Space::with_height(4),
        text(tr(keys::SUDO_NOTE))
            .size(text_metrics.caption)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        dry_run_note,
    ]
    .spacing(0)
    .into()
}
