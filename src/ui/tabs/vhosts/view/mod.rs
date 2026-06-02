mod config_editor;
mod form;
mod list;
mod row;

use super::{VHostView, VHostsTab};
use crate::lang::{lang_map::vhosts as keys, text as tr};
use crate::messages::Message;
use iced::Element;

fn php_options(available: &[String]) -> Vec<String> {
    let mut opts = vec![tr(keys::PHP_GLOBAL).to_string()];
    opts.extend(available.iter().cloned());
    opts
}

fn selection_to_php(s: &str) -> Option<String> {
    if s == tr(keys::PHP_GLOBAL) {
        None
    } else {
        Some(s.to_string())
    }
}

fn php_to_selection(v: &Option<String>) -> String {
    v.clone()
        .unwrap_or_else(|| tr(keys::PHP_GLOBAL).to_string())
}

pub fn render(tab: &VHostsTab, compact: bool) -> Element<'_, Message> {
    match tab.view_mode {
        VHostView::List => list::list_view(tab, compact),
        VHostView::ConfigEditor => config_editor::config_editor_view(tab),
    }
}
