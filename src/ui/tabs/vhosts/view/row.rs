use super::form::inline_edit_widget;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::vhosts::{FormMode, VHostEntry};
use crate::lang::{lang_map::vhosts as keys, text as tr};
use crate::messages::{Message, VHostsMessage};
use crate::ui::icons::Icon;
use crate::ui::tabs::vhosts::VHostsTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, checkbox, column, row};
use iced::{Alignment, Element};

pub(super) fn vhost_row<'a>(tab: &'a VHostsTab, vh: &'a VHostEntry) -> Element<'a, Message> {
    let idx = vh.index;

    if matches!(tab.form.mode, FormMode::Edit(i) if i == idx) {
        return inline_edit_widget(tab, idx);
    }

    let selected = tab.selected.contains(&idx);
    let expanded = tab.expanded_vhost == Some(idx);
    let status = format!(
        "{} · {}",
        vh.php_version
            .as_deref()
            .map(|v| format!("PHP {}", v))
            .unwrap_or_else(|| tr(keys::GLOBAL_PHP).to_string()),
        if vh.https_enabled {
            tr(keys::HTTPS)
        } else {
            tr(keys::HTTP_ONLY)
        }
    );
    let tone = if vh.https_enabled {
        ui::BadgeTone::Success
    } else {
        ui::BadgeTone::Neutral
    };

    let row = ui::summary_row(
        Icon::Globe,
        vh.server_name.as_str(),
        status,
        tone,
        vec![
            checkbox("", selected)
                .on_toggle(move |_| Message::VHosts(VHostsMessage::ToggleSelected(idx)))
                .into(),
            ui::compact_action_button(
                tr(keys::BROWSER),
                theme::color(theme_keys::TEAL),
                theme::color(theme_keys::TEAL_BG),
                theme::color(theme_keys::TEAL_HOVER),
                theme::color(theme_keys::TEAL_BORDER),
                Some(Message::VHosts(VHostsMessage::OpenBrowser(
                    vh.server_name.clone(),
                ))),
            ),
            ui::compact_action_button(
                tr(keys::EDIT),
                theme::color(theme_keys::BLUE),
                theme::color(theme_keys::BLUE_BG),
                theme::color(theme_keys::BLUE_HOVER),
                theme::color(theme_keys::BLUE_BORDER),
                Some(Message::VHosts(VHostsMessage::EditRequest(idx))),
            ),
        ],
        expanded,
        Some(Message::VHosts(VHostsMessage::ToggleExpanded(idx))),
    );

    if !expanded {
        return row;
    }

    column![row, vhost_expanded_panel(tab, vh)]
        .spacing(6)
        .into()
}

fn vhost_expanded_panel<'a>(tab: &'a VHostsTab, vh: &'a VHostEntry) -> Element<'a, Message> {
    let idx = vh.index;
    let is_confirming = tab.confirm_delete == Some(idx);
    let delete_action: Element<Message> = if is_confirming {
        row![
            ui::compact_action_button(
                tr(keys::CONFIRM_DELETE),
                theme::color(theme_keys::RED),
                theme::color(theme_keys::RED_BG),
                theme::color(theme_keys::RED_HOVER),
                theme::color(theme_keys::RED_BG),
                Some(Message::VHosts(VHostsMessage::DeleteConfirm(idx)))
            ),
            Space::with_width(6),
            ui::compact_action_button(
                tr(keys::CANCEL),
                theme::color(theme_keys::TEXT_MUTED),
                theme::color(theme_keys::BG_SURFACE),
                theme::color(theme_keys::BG_HOVER),
                theme::color(theme_keys::BORDER_SUBTLE),
                Some(Message::VHosts(VHostsMessage::DeleteCancel))
            ),
        ]
        .align_y(Alignment::Center)
        .into()
    } else {
        ui::compact_action_button(
            tr(keys::DELETE),
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
            theme::color(theme_keys::RED_HOVER),
            theme::color(theme_keys::RED_BG),
            Some(Message::VHosts(VHostsMessage::DeleteRequest(idx))),
        )
    };

    ui::expanded_panel(vec![
        ui::panel_section(
            "Details",
            vec![
                ui::detail_row(
                    tr(keys::DOCUMENT_ROOT),
                    ui::path_chip(vh.document_root.as_str()),
                ),
                ui::detail_row(
                    tr(keys::CONFIG_FILE),
                    ui::path_chip(tab.devpanel_conf.as_str()),
                ),
                if vh.tag.trim().is_empty() {
                    ui::detail_row("Tag", ui::detail_text("—"))
                } else {
                    ui::detail_row(
                        "Tag",
                        ui::small_badge(vh.tag.as_str(), ui::BadgeTone::Warning),
                    )
                },
            ],
        ),
        ui::panel_section(
            "Quick Actions",
            vec![
                row![
                    ui::secondary_icon_button(
                        Icon::Folder,
                        tr(keys::DOCUMENT_ROOT),
                        Message::VHosts(VHostsMessage::OpenDocumentRoot(vh.document_root.clone()))
                    ),
                    ui::secondary_icon_button(
                        Icon::Copy,
                        tr(keys::DUPLICATE),
                        Message::VHosts(VHostsMessage::DuplicateRequest(idx))
                    ),
                    ui::secondary_icon_button(
                        Icon::Shield,
                        if vh.https_enabled {
                            tr(keys::HTTPS_OFF)
                        } else {
                            tr(keys::HTTPS_ON)
                        },
                        Message::VHosts(VHostsMessage::ToggleHttps(idx))
                    ),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into(),
            ],
        ),
        ui::panel_section(
            "Advanced",
            vec![
                row![
                    ui::secondary_icon_button(
                        Icon::Code,
                        tr(keys::EDIT_CONFIG),
                        Message::VHosts(VHostsMessage::OpenConfigEditor)
                    ),
                    delete_action,
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into(),
            ],
        ),
    ])
}
