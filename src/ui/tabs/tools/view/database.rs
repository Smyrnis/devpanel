use super::shared::{section_header, small_action_btn, tool_item_row};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::icons::Icon;
use crate::ui::tabs::tools::ToolsTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, text};
use iced::{Border, Element, Length, Padding};

pub(super) fn db_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let note = ui::info_banner(
        Icon::Info,
        text(tr(keys::TERMINAL_ROOT_NOTE))
            .size(11)
            .color(theme::color(theme_keys::TEXT_MUTED))
            .into(),
        theme::color(theme_keys::YELLOW),
        theme::color(theme_keys::YELLOW_BG),
        theme::color(theme_keys::YELLOW_BORDER),
    );

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
            section_header(
                tr(keys::SECTION_DATABASE),
                tr(keys::DATABASE_HELP),
                tr(keys::MYSQL_MARIADB),
                Some(Message::Tools(ToolsMessage::OpenMysqlCli)),
            ),
            Space::with_height(18),
            ui::thin_line(),
            Space::with_height(14),
            db_action_row(
                tr(keys::MYSQL_MARIADB),
                tr(keys::MYSQL_MARIADB_HELP),
                theme::color(theme_keys::BLUE),
                Message::Tools(ToolsMessage::OpenMysqlCli)
            ),
            Space::with_height(8),
            db_action_row(
                tr(keys::MARIADB_EXPLICIT),
                tr(keys::MARIADB_EXPLICIT_HELP),
                theme::color(theme_keys::PURPLE),
                Message::Tools(ToolsMessage::OpenMariadbCli)
            ),
            Space::with_height(8),
            db_action_row(
                tr(keys::MYSQL_SOCKET),
                tr(keys::MYSQL_SOCKET_HELP),
                theme::color(theme_keys::TEAL),
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

fn db_action_row<'a>(
    title: &'a str,
    subtitle: &'a str,
    color: iced::Color,
    msg: Message,
) -> Element<'a, Message> {
    let action = small_action_btn(
        tr(keys::OPEN),
        color,
        theme::color(theme_keys::BG_SURFACE),
        theme::color(theme_keys::BG_HOVER),
        msg,
    );
    tool_item_row(title, subtitle, tr(keys::AVAILABLE), color, action)
}
