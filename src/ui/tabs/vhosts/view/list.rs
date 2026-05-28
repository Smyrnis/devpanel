use super::{form::add_form_widget, row::vhost_row};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::vhosts::FormMode;
use crate::lang::{lang_map::vhosts as keys, text as tr};
use crate::messages::{Message, Tab, VHostsMessage};
use crate::ui::icons::Icon;
use crate::ui::tabs::vhosts::VHostsTab;
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{Space, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Padding};

pub(super) fn list_view(tab: &VHostsTab, compact: bool) -> Element<'_, Message> {
    let header_fn = if compact {
        ui::page_header_compact
    } else {
        ui::page_header
    };
    let header = header_fn(
        tr(keys::TITLE),
        tr(keys::SUBTITLE),
        vec![ui::primary_icon_button(
            Icon::Plus,
            tr(keys::ADD_VHOST),
            Message::VHosts(VHostsMessage::ShowAddForm),
        )],
    );

    let form_el: Element<Message> = if tab.form.mode == FormMode::Hidden {
        Space::with_height(0).into()
    } else {
        add_form_widget(tab)
    };

    let status: Element<Message> = match &tab.status_msg {
        Some((ok, msg)) => ui::status_banner(*ok, msg.as_str()),
        None => Space::with_height(0).into(),
    };

    let php_info: Element<Message> = if tab.available_php_versions.is_empty() {
        ui::info_banner(
            Icon::Info,
            row![
                text(tr(keys::NO_PHP_MODULES))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED))
                    .width(Length::Fill),
                ui::secondary_icon_button(
                    Icon::Php,
                    tr(keys::OPEN_PHP_VERSIONS),
                    Message::SelectTab(Tab::Tools),
                ),
            ]
            .align_y(Alignment::Center)
            .into(),
            theme::color(theme_keys::BLUE),
            theme::color(theme_keys::BLUE_BG),
            theme::color(theme_keys::BLUE_BORDER),
        )
    } else {
        Space::with_height(0).into()
    };

    let bulk_bar: Element<Message> = if tab.selected.is_empty() {
        Space::with_height(0).into()
    } else {
        container(
            row![
                text(format!(
                    "{} {}",
                    tab.selected.len(),
                    tr(keys::SELECTED_SUFFIX)
                ))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_width(8),
                ui::compact_action_button(
                    tr(keys::SELECT_ALL),
                    theme::color(theme_keys::TEAL),
                    theme::color(theme_keys::TEAL_BG),
                    theme::color(theme_keys::TEAL_HOVER),
                    theme::color(theme_keys::TEAL_BORDER),
                    Some(Message::VHosts(VHostsMessage::SelectAll))
                ),
                Space::with_width(6),
                ui::compact_action_button(
                    tr(keys::CLEAR),
                    theme::color(theme_keys::TEXT_MUTED),
                    theme::color(theme_keys::BG_SURFACE),
                    theme::color(theme_keys::BG_HOVER),
                    theme::color(theme_keys::BORDER_SUBTLE),
                    Some(Message::VHosts(VHostsMessage::ClearSelection))
                ),
                Space::with_width(14),
                text_input(tr(keys::TAG_SELECTED_PLACEHOLDER), &tab.bulk_tag)
                    .on_input(|v| Message::VHosts(VHostsMessage::BulkTagChanged(v)))
                    .size(12)
                    .padding(Padding::from([6, 10]))
                    .style(styles::text_input_style)
                    .width(Length::FillPortion(1)),
                Space::with_width(6),
                ui::compact_action_button(
                    tr(keys::APPLY_TAG),
                    theme::color(theme_keys::BLUE),
                    theme::color(theme_keys::BLUE_BG),
                    theme::color(theme_keys::BLUE_HOVER),
                    theme::color(theme_keys::BLUE_BORDER),
                    if tab.selected.is_empty() || tab.bulk_tag.trim().is_empty() {
                        None
                    } else {
                        Some(Message::VHosts(VHostsMessage::ApplyBulkTag))
                    }
                ),
                Space::with_width(6),
                ui::compact_action_button(
                    tr(keys::DELETE_SELECTED),
                    theme::color(theme_keys::RED),
                    theme::color(theme_keys::RED_BG),
                    theme::color(theme_keys::RED_HOVER),
                    theme::color(theme_keys::RED_BG),
                    if tab.selected.is_empty() {
                        None
                    } else {
                        Some(Message::VHosts(VHostsMessage::BulkDeleteConfirm))
                    }
                ),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 14]))
        .width(Length::Fill)
        .style(ui::surface_style())
        .into()
    };

    let visible_vhosts = filtered_vhosts(tab);

    let body: Element<Message> = if tab.vhosts.is_empty() && !tab.scanning {
        empty_vhosts_panel()
    } else if tab.scanning {
        container(
            text(tr(keys::SCANNING))
                .size(14)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        )
        .width(Length::Fill)
        .padding(Padding::from([40, 0]))
        .center_x(Length::Fill)
        .into()
    } else if visible_vhosts.is_empty() {
        empty_search_panel()
    } else {
        ui::row_group(
            visible_vhosts
                .into_iter()
                .map(|v| vhost_row(tab, v))
                .collect(),
        )
    };

    scrollable(
        column![
            header,
            Space::with_height(18),
            vhost_control_row(tab),
            Space::with_height(10),
            php_info,
            if tab.available_php_versions.is_empty() {
                Space::with_height(10)
            } else {
                Space::with_height(0)
            },
            bulk_bar,
            if tab.vhosts.is_empty() {
                Space::with_height(0)
            } else {
                Space::with_height(10)
            },
            form_el,
            if tab.form.mode != FormMode::Hidden {
                Space::with_height(10)
            } else {
                Space::with_height(0)
            },
            status,
            if tab.status_msg.is_some() {
                Space::with_height(12)
            } else {
                Space::with_height(0)
            },
            body,
            Space::with_height(24),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn vhost_control_row(tab: &VHostsTab) -> Element<'_, Message> {
    container(
        row![
            text_input(tr(keys::SEARCH_PLACEHOLDER), &tab.search_query)
                .on_input(|v| Message::VHosts(VHostsMessage::SearchChanged(v)))
                .size(12)
                .padding(Padding::from([8, 10]))
                .style(styles::text_input_style)
                .width(Length::FillPortion(2)),
            Space::with_width(4),
            column![
                text(vhost_count_label(tab))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(3),
                text(if tab.selected.is_empty() {
                    tr(keys::CONFIG_SOURCE_HINT).to_string()
                } else {
                    format!("{} {}", tab.selected.len(), tr(keys::SELECTED_SUFFIX))
                })
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .width(Length::Fill),
            text(if tab.scanning { tr(keys::SCANNING) } else { "" })
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            ui::secondary_icon_button(
                Icon::Refresh,
                tr(keys::RELOAD),
                Message::VHosts(VHostsMessage::Scan),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(ui::surface_style())
    .into()
}

fn filtered_vhosts(tab: &VHostsTab) -> Vec<&crate::domain::vhosts::VHostEntry> {
    let query = tab.search_query.trim().to_lowercase();
    if query.is_empty() {
        return tab.vhosts.iter().collect();
    }

    tab.vhosts
        .iter()
        .filter(|vhost| {
            vhost.server_name.to_lowercase().contains(&query)
                || vhost.document_root.to_lowercase().contains(&query)
                || vhost
                    .php_version
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&query)
        })
        .collect()
}

fn vhost_count_label(tab: &VHostsTab) -> String {
    let visible = filtered_vhosts(tab).len();
    if tab.search_query.trim().is_empty() {
        format!("{} {}", tab.vhosts.len(), tr(keys::TITLE))
    } else {
        format!("{}/{} {}", visible, tab.vhosts.len(), tr(keys::TITLE))
    }
}

fn empty_vhosts_panel<'a>() -> Element<'a, Message> {
    container(
        row![
            column![
                text(tr(keys::EMPTY_TITLE))
                    .size(16)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(5),
                text(tr(keys::EMPTY_BODY))
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .width(Length::Fill),
            ui::primary_icon_button(
                Icon::Plus,
                tr(keys::ADD_VHOST),
                Message::VHosts(VHostsMessage::ShowAddForm),
            ),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([14, 16]))
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn empty_search_panel<'a>() -> Element<'a, Message> {
    container(
        text(tr(keys::NO_FILTER_MATCH))
            .size(13)
            .color(theme::color(theme_keys::TEXT_MUTED)),
    )
    .padding(Padding::from([22, 24]))
    .width(Length::Fill)
    .center_x(Length::Fill)
    .style(ui::surface_style())
    .into()
}
