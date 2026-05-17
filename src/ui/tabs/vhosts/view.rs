use super::{FormMode, VHostEntry, VHostView, VHostsTab};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::vhosts as keys, text as tr};
use crate::messages::{Message, Tab, VHostsMessage};
use crate::ui::templates::prelude as ui;
use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, text, text_editor,
    text_input,
};
use iced::{Alignment, Border, Color, Element, Length, Padding};

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

pub fn render(tab: &VHostsTab) -> Element<'_, Message> {
    match tab.view_mode {
        VHostView::List => list_view(tab),
        VHostView::ConfigEditor => config_editor_view(tab),
    }
}

fn list_view(tab: &VHostsTab) -> Element<'_, Message> {
    let header = ui::page_header(
        tr(keys::TITLE),
        tr(keys::SUBTITLE),
        vec![
            ui::secondary_button(
                tr(keys::OPEN_FILE),
                Message::VHosts(VHostsMessage::OpenDevpanelConf),
            ),
            ui::secondary_button(tr(keys::RELOAD), Message::VHosts(VHostsMessage::Scan)),
            ui::primary_button(
                tr(keys::ADD_VHOST),
                Message::VHosts(VHostsMessage::ShowAddForm),
            ),
        ],
    );

    let path_bar = container(
        column![
            row![
                ui::metric_card(tr(keys::CONFIG_STATUS), tr(keys::NOT_CHECKED)),
                ui::metric_card(tr(keys::HOST_COUNT), tab.vhosts.len().to_string()),
                ui::metric_card(
                    tr(keys::PHP_PINNING),
                    if tab.available_php_versions.is_empty() {
                        tr(keys::NO_MOD_PHP).to_string()
                    } else {
                        tab.available_php_versions.join(", ")
                    }
                ),
            ]
            .spacing(12),
            Space::with_height(12),
            row![
                column![
                    text(tr(keys::CONFIG_FILE))
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_height(2),
                    text(tab.devpanel_conf.as_str())
                        .size(12)
                        .color(theme::color(theme_keys::TEXT_SECONDARY)),
                ]
                .spacing(0)
                .width(Length::Fill),
                ui::action_button(
                    tr(keys::EDIT_CONFIG),
                    theme::color(theme_keys::BLUE),
                    theme::color(theme_keys::BLUE_BG),
                    theme::color(theme_keys::BLUE_HOVER),
                    theme::color(theme_keys::BLUE_BORDER),
                    Some(Message::VHosts(VHostsMessage::OpenConfigEditor))
                ),
                Space::with_width(8),
                ui::action_button(
                    tr(keys::OPEN_FILE),
                    theme::color(theme_keys::BLUE),
                    theme::color(theme_keys::BLUE_BG),
                    theme::color(theme_keys::BLUE_HOVER),
                    theme::color(theme_keys::BLUE_BORDER),
                    Some(Message::VHosts(VHostsMessage::OpenDevpanelConf))
                ),
                Space::with_width(8),
                text(if tab.scanning { tr(keys::SCANNING) } else { "" })
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(0),
    )
    .padding(Padding::from([12, 16]))
    .width(Length::Fill)
    .style(ui::surface_style());

    let form_el: Element<Message> = if tab.form.mode == FormMode::Hidden {
        Space::with_height(0).into()
    } else {
        add_form_widget(tab)
    };

    let status: Element<Message> = match &tab.status_msg {
        Some((ok, msg)) => {
            let (color, bg) = if *ok {
                (
                    theme::color(theme_keys::GREEN),
                    theme::color(theme_keys::GREEN_BG),
                )
            } else {
                (
                    theme::color(theme_keys::RED),
                    theme::color(theme_keys::RED_BG),
                )
            };
            container(
                row![
                    container(Space::with_width(6)).width(6).height(6).style(
                        move |_: &iced::Theme| container::Style {
                            background: Some(color.into()),
                            border: Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    ),
                    Space::with_width(8),
                    text(msg.as_str())
                        .size(12)
                        .color(theme::color(theme_keys::TEXT_SECONDARY)),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([10, 14]))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(bg.into()),
                border: Border {
                    color: theme::color(theme_keys::BORDER_SUBTLE),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
        }
        None => Space::with_height(0).into(),
    };

    let php_info: Element<Message> = if tab.available_php_versions.is_empty() {
        container(
            row![
                text("i").size(10).color(theme::color(theme_keys::BLUE)),
                Space::with_width(8),
                text(tr(keys::NO_PHP_MODULES))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED))
                    .width(Length::Fill),
                ui::secondary_button(tr(keys::OPEN_PHP_VERSIONS), Message::SelectTab(Tab::Tools)),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 14]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BLUE_BG).into()),
            border: Border {
                color: theme::color(theme_keys::BLUE_BORDER),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_height(0).into()
    };

    let bulk_bar: Element<Message> = if tab.vhosts.is_empty() {
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

    let body: Element<Message> = if tab.vhosts.is_empty() && !tab.scanning {
        ui::empty_state(
            tr(keys::EMPTY_TITLE),
            tr(keys::EMPTY_BODY),
            vec![
                ui::primary_button(
                    tr(keys::ADD_VHOST),
                    Message::VHosts(VHostsMessage::ShowAddForm),
                ),
                ui::secondary_button(
                    tr(keys::OPEN_FILE),
                    Message::VHosts(VHostsMessage::OpenDevpanelConf),
                ),
            ],
        )
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
    } else {
        column(
            tab.vhosts
                .iter()
                .map(|v| vhost_row(tab, v))
                .collect::<Vec<_>>(),
        )
        .spacing(8)
        .into()
    };

    scrollable(
        column![
            header,
            Space::with_height(18),
            path_bar,
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

fn config_editor_view(tab: &VHostsTab) -> Element<'_, Message> {
    let header = row![
        column![
            text(tr(keys::CONFIG_EDITOR))
                .size(22)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(4),
            text(tab.devpanel_conf.as_str())
                .size(12)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .width(Length::Fill),
        ui::action_button(
            tr(keys::BACK),
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_HOVER),
            theme::color(theme_keys::TEAL_BORDER),
            Some(Message::VHosts(VHostsMessage::CloseConfigEditor))
        ),
        Space::with_width(8),
        ui::action_button(
            if tab.config_loading {
                tr(keys::SAVING)
            } else {
                tr(keys::SAVE)
            },
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            theme::color(theme_keys::GREEN_BG),
            if tab.config_loading {
                None
            } else {
                Some(Message::VHosts(VHostsMessage::SaveConfigFile))
            }
        ),
    ]
    .align_y(Alignment::Center);

    let dirty_badge: Element<Message> = if tab.config_dirty {
        container(
            text(tr(keys::UNSAVED_CHANGES))
                .size(10)
                .color(theme::color(theme_keys::YELLOW)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(
                Color {
                    r: 0.19,
                    g: 0.16,
                    b: 0.04,
                    a: 1.0,
                }
                .into(),
            ),
            border: Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_height(0).into()
    };

    let editor = text_editor(&tab.config_content)
        .on_action(|action| Message::VHosts(VHostsMessage::ConfigEditorAction(action)))
        .height(Length::Fill)
        .padding(Padding::from([12, 14]));

    let editor_container = container(editor)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

    column![
        Space::with_height(22),
        container(
            column![header, Space::with_height(8), dirty_badge]
                .spacing(0)
                .padding(Padding::from([0, 24]))
        )
        .width(Length::Fill),
        Space::with_height(12),
        container(editor_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([0, 24])),
        Space::with_height(16),
    ]
    .height(Length::Fill)
    .into()
}

fn vhost_row<'a>(tab: &'a VHostsTab, vh: &'a VHostEntry) -> Element<'a, Message> {
    let idx = vh.index;

    if matches!(tab.form.mode, FormMode::Edit(i) if i == idx) {
        return inline_edit_widget(tab, idx);
    }

    let sn = vh.server_name.clone();
    let selected = tab.selected.contains(&idx);

    let name_row = row![
        checkbox("", selected)
            .on_toggle(move |_| Message::VHosts(VHostsMessage::ToggleSelected(idx))),
        Space::with_width(8),
        text(vh.server_name.as_str())
            .size(14)
            .color(theme::color(theme_keys::TEXT_PRIMARY)),
        Space::with_width(Length::Fill),
        container(
            text(tr(keys::ACTIVE))
                .size(10)
                .color(theme::color(theme_keys::GREEN))
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::GREEN_BG).into()),
            border: Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    ]
    .align_y(Alignment::Center);

    let info_row = row![
        text(tr(keys::DOCUMENT_ROOT))
            .size(10)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_width(8),
        text(if vh.document_root.is_empty() {
            "—"
        } else {
            vh.document_root.as_str()
        })
        .size(12)
        .color(theme::color(theme_keys::TEXT_SECONDARY)),
    ]
    .align_y(Alignment::Center);

    let php_badge: Element<Message> = match &vh.php_version {
        Some(ver) => container(
            row![
                text(tr(keys::PHP))
                    .size(9)
                    .color(theme::color(theme_keys::PURPLE)),
                Space::with_width(4),
                text(ver.as_str())
                    .size(10)
                    .color(theme::color(theme_keys::PURPLE)),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::PURPLE_BG).into()),
            border: Border {
                color: theme::color(theme_keys::PURPLE_BORDER),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into(),
        None => container(
            text(tr(keys::GLOBAL_PHP))
                .size(9)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into(),
    };
    let https_badge: Element<Message> = if vh.https_enabled {
        container(
            text(tr(keys::HTTPS))
                .size(9)
                .color(theme::color(theme_keys::TEAL)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::TEAL_BG).into()),
            border: Border {
                color: theme::color(theme_keys::TEAL_BORDER),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else {
        container(
            text(tr(keys::HTTP_ONLY))
                .size(9)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into()
    };
    let tag_badge: Element<Message> = if vh.tag.trim().is_empty() {
        Space::with_width(0).into()
    } else {
        container(
            text(vh.tag.as_str())
                .size(9)
                .color(theme::color(theme_keys::YELLOW)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(
                Color {
                    r: 0.19,
                    g: 0.16,
                    b: 0.04,
                    a: 1.0,
                }
                .into(),
            ),
            border: Border {
                color: Color {
                    r: 0.24,
                    g: 0.20,
                    b: 0.05,
                    a: 1.0,
                },
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into()
    };

    let is_confirming = tab.confirm_delete == Some(idx);
    let del_btn: Element<Message> = if is_confirming {
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

    container(
        column![
            name_row,
            Space::with_height(4),
            info_row,
            Space::with_height(6),
            row![
                php_badge,
                Space::with_width(6),
                https_badge,
                Space::with_width(6),
                tag_badge
            ]
            .align_y(Alignment::Center),
            Space::with_height(14),
            ui::thin_line(),
            Space::with_height(12),
            row![
                ui::compact_action_button(
                    tr(keys::EDIT),
                    theme::color(theme_keys::BLUE),
                    theme::color(theme_keys::BLUE_BG),
                    theme::color(theme_keys::BLUE_HOVER),
                    theme::color(theme_keys::BLUE_BORDER),
                    Some(Message::VHosts(VHostsMessage::EditRequest(idx)))
                ),
                Space::with_width(6),
                ui::compact_action_button(
                    tr(keys::DUPLICATE),
                    theme::color(theme_keys::PURPLE),
                    theme::color(theme_keys::PURPLE_BG),
                    theme::color(theme_keys::PURPLE_HOVER),
                    theme::color(theme_keys::PURPLE_BORDER),
                    Some(Message::VHosts(VHostsMessage::DuplicateRequest(idx)))
                ),
                Space::with_width(6),
                ui::compact_action_button(
                    if vh.https_enabled {
                        tr(keys::HTTPS_OFF)
                    } else {
                        tr(keys::HTTPS_ON)
                    },
                    theme::color(theme_keys::TEAL),
                    theme::color(theme_keys::TEAL_BG),
                    theme::color(theme_keys::TEAL_HOVER),
                    theme::color(theme_keys::TEAL_BORDER),
                    Some(Message::VHosts(VHostsMessage::ToggleHttps(idx)))
                ),
                Space::with_width(6),
                ui::compact_action_button(
                    tr(keys::BROWSER),
                    theme::color(theme_keys::TEAL),
                    theme::color(theme_keys::TEAL_BG),
                    theme::color(theme_keys::TEAL_HOVER),
                    theme::color(theme_keys::TEAL_BORDER),
                    Some(Message::VHosts(VHostsMessage::OpenBrowser(sn)))
                ),
                Space::with_width(Length::Fill),
                del_btn,
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(0),
    )
    .padding(Padding::from([16, 18]))
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn inline_edit_widget<'a>(tab: &'a VHostsTab, _idx: usize) -> Element<'a, Message> {
    let can_save =
        !tab.form.server_name.trim().is_empty() && !tab.form.document_root.trim().is_empty();

    let submit_btn = button(text(tr(keys::SAVE_CHANGES)).size(13).color(if can_save {
        theme::color(theme_keys::GREEN)
    } else {
        theme::color(theme_keys::TEXT_MUTED)
    }))
    .padding(Padding::from([9, 18]))
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered if can_save => iced::widget::button::Style {
            background: Some(theme::color(theme_keys::GREEN_HOVER).into()),
            text_color: theme::color(theme_keys::GREEN),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        _ => iced::widget::button::Style {
            background: Some(
                if can_save {
                    theme::color(theme_keys::GREEN_BG)
                } else {
                    theme::color(theme_keys::BG_SURFACE)
                }
                .into(),
            ),
            text_color: if can_save {
                theme::color(theme_keys::GREEN)
            } else {
                theme::color(theme_keys::TEXT_MUTED)
            },
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        },
    });
    let submit_el: Element<Message> = if can_save {
        submit_btn
            .on_press(Message::VHosts(VHostsMessage::SaveEdit))
            .into()
    } else {
        submit_btn.into()
    };

    let php_picker =
        php_version_picker(&tab.available_php_versions, &tab.form.php_version, |sel| {
            Message::VHosts(VHostsMessage::FormPhpVersionChanged(selection_to_php(&sel)))
        });
    let https_toggle = checkbox(tr(keys::ENABLE_HTTPS_MKCERT), tab.form.https_enabled)
        .on_toggle(|v| Message::VHosts(VHostsMessage::FormHttpsChanged(v)))
        .size(13);

    container(
        column![
            row![
                text(tr(keys::EDITING_VHOST))
                    .size(13)
                    .color(theme::color(theme_keys::BLUE)),
                Space::with_width(Length::Fill),
                ui::compact_action_button(
                    tr(keys::CANCEL),
                    theme::color(theme_keys::TEXT_MUTED),
                    theme::color(theme_keys::BG_SURFACE),
                    theme::color(theme_keys::BG_HOVER),
                    theme::color(theme_keys::BORDER_SUBTLE),
                    Some(Message::VHosts(VHostsMessage::HideForm))
                ),
            ]
            .align_y(Alignment::Center),
            Space::with_height(14),
            ui::thin_line(),
            Space::with_height(14),
            row![
                column![
                    text(tr(keys::SERVER_NAME))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_height(5),
                    text_input(tr(keys::SERVER_NAME_PLACEHOLDER), &tab.form.server_name)
                        .on_input(|v| Message::VHosts(VHostsMessage::FormServerNameChanged(v)))
                        .size(13)
                        .padding(Padding::from([8, 10]))
                        .width(Length::Fill),
                ]
                .spacing(0)
                .width(Length::FillPortion(1)),
                Space::with_width(14),
                column![
                    text(tr(keys::DOCUMENT_ROOT))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_height(5),
                    text_input(tr(keys::DOCUMENT_ROOT_PLACEHOLDER), &tab.form.document_root)
                        .on_input(|v| Message::VHosts(VHostsMessage::FormDocRootChanged(v)))
                        .size(13)
                        .padding(Padding::from([8, 10]))
                        .width(Length::Fill),
                ]
                .spacing(0)
                .width(Length::FillPortion(2)),
            ]
            .align_y(Alignment::Start),
            Space::with_height(12),
            column![
                text(tr(keys::PHP_VERSION_OPTIONAL))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(5),
                php_picker,
            ]
            .spacing(0),
            Space::with_height(10),
            https_toggle,
            Space::with_height(14),
            submit_el,
        ]
        .spacing(0)
        .padding(Padding::from([16, 18])),
    )
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: theme::color(theme_keys::BLUE_BORDER),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn add_form_widget(tab: &VHostsTab) -> Element<'_, Message> {
    let is_edit = matches!(tab.form.mode, FormMode::Edit(_));
    let can_save =
        !tab.form.server_name.trim().is_empty() && !tab.form.document_root.trim().is_empty();
    let save_msg = if is_edit {
        Message::VHosts(VHostsMessage::SaveEdit)
    } else {
        Message::VHosts(VHostsMessage::Create)
    };
    let save_lbl = if is_edit {
        tr(keys::SAVE_CHANGES)
    } else {
        tr(keys::CREATE_VHOST)
    };

    let submit_btn = button(text(save_lbl).size(13).color(if can_save {
        theme::color(theme_keys::GREEN)
    } else {
        theme::color(theme_keys::TEXT_MUTED)
    }))
    .padding(Padding::from([9, 18]))
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered if can_save => iced::widget::button::Style {
            background: Some(theme::color(theme_keys::GREEN_HOVER).into()),
            text_color: theme::color(theme_keys::GREEN),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
        _ => iced::widget::button::Style {
            background: Some(
                if can_save {
                    theme::color(theme_keys::GREEN_BG)
                } else {
                    theme::color(theme_keys::BG_SURFACE)
                }
                .into(),
            ),
            text_color: if can_save {
                theme::color(theme_keys::GREEN)
            } else {
                theme::color(theme_keys::TEXT_MUTED)
            },
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        },
    });
    let submit_el: Element<Message> = if can_save {
        submit_btn.on_press(save_msg).into()
    } else {
        submit_btn.into()
    };

    let php_picker =
        php_version_picker(&tab.available_php_versions, &tab.form.php_version, |sel| {
            Message::VHosts(VHostsMessage::FormPhpVersionChanged(selection_to_php(&sel)))
        });
    let https_toggle = checkbox(tr(keys::ENABLE_HTTPS_MKCERT), tab.form.https_enabled)
        .on_toggle(|v| Message::VHosts(VHostsMessage::FormHttpsChanged(v)))
        .size(13);

    container(
        column![
            row![
                text(if is_edit {
                    tr(keys::EDIT_VHOST)
                } else {
                    tr(keys::ADD_VHOST_TITLE)
                })
                .size(14)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
                Space::with_width(Length::Fill),
                ui::compact_action_button(
                    tr(keys::CANCEL),
                    theme::color(theme_keys::TEXT_MUTED),
                    theme::color(theme_keys::BG_SURFACE),
                    theme::color(theme_keys::BG_HOVER),
                    theme::color(theme_keys::BORDER_SUBTLE),
                    Some(Message::VHosts(VHostsMessage::HideForm))
                ),
            ]
            .align_y(Alignment::Center),
            Space::with_height(16),
            ui::thin_line(),
            Space::with_height(16),
            column![
                text(tr(keys::SERVER_NAME_HELP))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(5),
                text_input(tr(keys::SERVER_NAME_ADD_PLACEHOLDER), &tab.form.server_name)
                    .on_input(|v| Message::VHosts(VHostsMessage::FormServerNameChanged(v)))
                    .size(13)
                    .padding(Padding::from([8, 10]))
                    .width(Length::Fill),
            ]
            .spacing(0),
            Space::with_height(12),
            column![
                text(tr(keys::DOCUMENT_ROOT_HELP))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(5),
                text_input(
                    tr(keys::DOCUMENT_ROOT_ADD_PLACEHOLDER),
                    &tab.form.document_root
                )
                .on_input(|v| Message::VHosts(VHostsMessage::FormDocRootChanged(v)))
                .size(13)
                .padding(Padding::from([8, 10]))
                .width(Length::Fill),
            ]
            .spacing(0),
            Space::with_height(12),
            column![
                row![
                    text(tr(keys::PHP_VERSION))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_width(6),
                    text(tr(keys::PHP_VERSION_HELP))
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center),
                Space::with_height(5),
                php_picker,
            ]
            .spacing(0),
            Space::with_height(10),
            https_toggle,
            Space::with_height(18),
            submit_el,
        ]
        .spacing(0)
        .padding(Padding::from([20, 22])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn php_version_picker<'a, F>(
    available: &'a [String],
    current: &'a Option<String>,
    on_select: F,
) -> Element<'a, Message>
where
    F: Fn(String) -> Message + 'a,
{
    let options = php_options(available);
    let selected = php_to_selection(current);

    let el = pick_list(options, Some(selected), move |s: String| on_select(s))
        .padding(Padding::from([8, 12]))
        .width(Length::Fixed(220.0))
        .style(|_theme, status| {
            use iced::widget::pick_list;
            let open = matches!(status, pick_list::Status::Opened);
            pick_list::Style {
                text_color: theme::color(theme_keys::PURPLE),
                placeholder_color: theme::color(theme_keys::TEXT_MUTED),
                handle_color: theme::color(theme_keys::PURPLE),
                background: iced::Background::Color(if open {
                    theme::color(theme_keys::PURPLE_HOVER)
                } else {
                    theme::color(theme_keys::PURPLE_BG)
                }),
                border: Border {
                    color: if open {
                        theme::color(theme_keys::PURPLE)
                    } else {
                        theme::color(theme_keys::PURPLE_BORDER)
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
            }
        });

    if available.is_empty() {
        column![
            el,
            Space::with_height(3),
            text(tr(keys::NO_MOD_PHP))
                .size(10)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .into()
    } else {
        el.into()
    }
}
