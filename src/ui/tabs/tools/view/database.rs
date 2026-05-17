use super::shared::db_btn;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::tabs::tools::ToolsTab;
use crate::ui::templates::view as ui;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn db_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let note = container(
        row![
            text("").size(10).color(theme::color(theme_keys::YELLOW)),
            Space::with_width(8),
            text(tr(keys::TERMINAL_ROOT_NOTE))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::YELLOW_BG).into()),
        border: Border {
            color: theme::color(theme_keys::YELLOW_BORDER),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    let status_row: Element<Message> = if !tab.db_status.is_empty() {
        container(
            text(&tab.db_status)
                .size(12)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
        )
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_height(0).into()
    };

    container(
        column![
            text(tr(keys::SECTION_DATABASE))
                .size(14)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(3),
            text(tr(keys::DATABASE_HELP))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(18),
            ui::thin_line(),
            Space::with_height(14),
            db_btn(
                tr(keys::MYSQL_MARIADB),
                tr(keys::MYSQL_MARIADB_HELP),
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_HOVER),
                theme::color(theme_keys::BLUE_BORDER),
                Message::Tools(ToolsMessage::OpenMysqlCli)
            ),
            Space::with_height(8),
            db_btn(
                tr(keys::MARIADB_EXPLICIT),
                tr(keys::MARIADB_EXPLICIT_HELP),
                theme::color(theme_keys::PURPLE),
                theme::color(theme_keys::PURPLE_BG),
                theme::color(theme_keys::PURPLE_HOVER),
                theme::color(theme_keys::PURPLE_BORDER),
                Message::Tools(ToolsMessage::OpenMariadbCli)
            ),
            Space::with_height(8),
            db_btn(
                tr(keys::MYSQL_SOCKET),
                tr(keys::MYSQL_SOCKET_HELP),
                theme::color(theme_keys::TEAL),
                theme::color(theme_keys::TEAL_BG),
                theme::color(theme_keys::TEAL_HOVER),
                theme::color(theme_keys::TEAL_BORDER),
                Message::Tools(ToolsMessage::OpenMysqlSocket)
            ),
            Space::with_height(16),
            status_row,
            if tab.db_status.is_empty() {
                Space::with_height(0)
            } else {
                Space::with_height(12)
            },
            note,
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}
