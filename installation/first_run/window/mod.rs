mod actions;
mod header;
mod log;
mod packages;

use crate::core::theme::{self, theme_map as theme_keys};
use crate::installer::{FirstRunInstallOptions, FirstRunPackage, FirstRunSetupStatus};
use crate::lang::{lang_map::install as keys, text as tr};
use crate::messages::Message;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row, scrollable};
use iced::{Alignment, Element, Length, Padding};

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

    let content = column![
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
    .padding(Padding::from([32, 32]))
    .width(Length::Fixed(
        crate::core::app_config::installer_content_width(),
    ));

    container(
        scrollable(
            container(content)
                .width(Length::Fill)
                .center_x(Length::Fill)
                .padding(Padding::from([24, 0])),
        )
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_ELEVATED).into()),
        ..Default::default()
    })
    .into()
}

fn packages_heading<'a>() -> Element<'a, Message> {
    iced::widget::text(tr(keys::PACKAGES_TO_INSTALL))
        .size(crate::core::app_config::text_metrics().caption)
        .color(theme::color(theme_keys::TEXT_MUTED))
        .into()
}
