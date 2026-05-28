mod actions;
mod header;
mod log;
mod packages;

use crate::core::theme::{self, theme_map as theme_keys};
use crate::installer::{FirstRunInstallOptions, FirstRunPackage, FirstRunSetupStatus};
use crate::lang::{lang_map::install as keys, text as tr};
use crate::messages::Message;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row};
use iced::{Alignment, Border, Element, Length, Padding};

pub fn view<'a>(
    options: FirstRunInstallOptions,
    status: FirstRunSetupStatus,
    expanded: Option<FirstRunPackage>,
    installing: bool,
    log_lines: &'a [String],
) -> Element<'a, Message> {
    let log_panel: Element<Message> = if installing || !log_lines.is_empty() {
        log::setup_log_panel(log_lines)
    } else {
        Space::with_height(0).into()
    };

    let card = container(
        column![
            header::header(),
            Space::with_height(20),
            ui::divider(),
            Space::with_height(16),
            packages_heading(),
            Space::with_height(8),
            packages::install_rows(options, status, expanded, installing),
            Space::with_height(if installing || !log_lines.is_empty() {
                14
            } else {
                0
            }),
            log_panel,
            Space::with_height(24),
            row![
                actions::continue_button(installing),
                Space::with_width(10),
                actions::exit_button(installing)
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(0)
        .padding(Padding::from([32, 32])),
    )
    .width(560)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_ELEVATED).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_MED),
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: theme::color(theme_keys::SHADOW_HEAVY),
            offset: iced::Vector::new(0.0, 16.0),
            blur_radius: 56.0,
        },
        ..Default::default()
    });

    container(
        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::OVERLAY_STRONG).into()),
        ..Default::default()
    })
    .into()
}

fn packages_heading<'a>() -> Element<'a, Message> {
    iced::widget::text(tr(keys::PACKAGES_TO_INSTALL))
        .size(11)
        .color(theme::color(theme_keys::TEXT_MUTED))
        .into()
}
