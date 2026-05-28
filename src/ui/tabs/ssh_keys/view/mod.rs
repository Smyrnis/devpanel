mod generate;
mod list;
mod status;

use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::ssh_keys as keys, text as tr};
use crate::messages::{Message, SshKeysMessage};
use crate::ui::icons::Icon;
use crate::ui::tabs::ssh_keys::SshKeysTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, row, text};
use iced::{Alignment, Element, Length};

pub fn settings_panel(tab: &SshKeysTab, compact: bool) -> Element<'_, Message> {
    let panels: Element<Message> = if compact {
        column![
            list::keys_panel(tab),
            Space::with_height(14),
            generate::generate_panel(tab)
        ]
        .spacing(0)
        .into()
    } else {
        row![
            list::keys_panel(tab),
            Space::with_width(14),
            generate::generate_panel(tab)
        ]
        .align_y(Alignment::Start)
        .into()
    };

    column![
        row![
            text(tr(keys::TITLE))
                .size(13)
                .color(theme::color(theme_keys::TEXT_SECONDARY))
                .width(Length::Fill),
            ui::secondary_icon_button(
                Icon::Refresh,
                tr(keys::REFRESH),
                Message::SshKeys(SshKeysMessage::ListKeys),
            ),
            ui::secondary_icon_button(
                Icon::Folder,
                tr(keys::OPEN_SSH_DIR),
                Message::SshKeys(SshKeysMessage::OpenDir),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        Space::with_height(12),
        panels,
        Space::with_height(12),
        status::status_bar(tab),
    ]
    .spacing(0)
    .into()
}
