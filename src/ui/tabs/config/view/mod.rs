mod controls;
mod rows;
mod sections;

use crate::lang::{lang_map::config as keys, text as tr};
use crate::messages::Message;
use crate::ui::tabs::config::ConfigTab;
use crate::ui::tabs::ssh_keys::SshKeysTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, scrollable};
use iced::{Element, Padding};

pub fn render<'a>(
    tab: &'a ConfigTab,
    ssh_keys_tab: &'a SshKeysTab,
    compact: bool,
) -> Element<'a, Message> {
    let header_fn = if compact {
        ui::page_header_compact
    } else {
        ui::page_header
    };
    let header = header_fn(tr(keys::TITLE), tr(keys::SUBTITLE), vec![]);

    scrollable(
        column![
            header,
            Space::with_height(18),
            rows::config_sections(tab, ssh_keys_tab, compact),
            Space::with_height(16),
            status_panel(&tab.status_msg),
            Space::with_height(24),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn status_panel<'a>(status_msg: &'a Option<(bool, String)>) -> Element<'a, Message> {
    match status_msg {
        Some((ok, msg)) => ui::status_banner(*ok, msg.as_str()),
        None => Space::with_height(0).into(),
    }
}
