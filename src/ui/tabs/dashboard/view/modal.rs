use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::dashboard as keys, text as tr};
use crate::messages::{DashboardMessage, Message};
use crate::ui::icons::Icon;
use crate::ui::tabs::dashboard::DashboardTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length, Padding};

pub(super) fn php_info_modal(tab: &DashboardTab) -> Element<'_, Message> {
    let body = tab
        .php_info
        .as_deref()
        .unwrap_or(tr(keys::LOADING_PHP_INFO));

    container(
        column![
            row![
                text(tr(keys::PHP_INFO))
                    .size(18)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_width(Length::Fill),
                ui::secondary_icon_button(
                    Icon::Stop,
                    tr(keys::CLOSE),
                    Message::Dashboard(DashboardMessage::ClosePhpInfo)
                ),
            ]
            .align_y(Alignment::Center),
            Space::with_height(10),
            scrollable(
                text(body)
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_SECONDARY))
            )
            .height(Length::Fixed(240.0)),
        ]
        .spacing(0),
    )
    .padding(Padding::from([16, 18]))
    .width(Length::Fill)
    .style(ui::card_style_with_border(theme::color(
        theme_keys::PURPLE_BORDER,
    )))
    .into()
}
