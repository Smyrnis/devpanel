use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::ssh_keys::KeyEntry;
use crate::lang::{lang_map::ssh_keys as keys, text as tr};
use crate::messages::{Message, SshKeysMessage};
use crate::ui::icons::{self, Icon};
use crate::ui::tabs::ssh_keys::SshKeysTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn keys_panel(tab: &SshKeysTab) -> Element<'_, Message> {
    let entries: Vec<Element<Message>> = if tab.keys_list.is_empty() {
        vec![ssh_empty_state()]
    } else {
        tab.keys_list.iter().map(key_row).collect()
    };

    container(
        column![
            row![
                text(tr(keys::KEYS_PANEL_TITLE))
                    .size(14)
                    .color(theme::color(theme_keys::TEXT_SECONDARY))
                    .width(Length::Fill),
                container(
                    text(format!("{}", tab.keys_list.len()))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                )
                .padding(Padding::from([3, 8]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(theme::color(theme_keys::BG_SURFACE).into()),
                    border: Border {
                        color: theme::color(theme_keys::BORDER_SUBTLE),
                        width: 1.0,
                        radius: 20.0.into()
                    },
                    ..Default::default()
                }),
            ]
            .align_y(Alignment::Center),
            Space::with_height(16),
            scrollable(column(entries).spacing(6)).height(380),
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::FillPortion(1))
    .style(ui::card_style())
    .into()
}

fn key_row(k: &KeyEntry) -> Element<'_, Message> {
    let pub_badge: Element<Message> = if k.has_pub {
        badge(tr(keys::PUB_BADGE), theme_keys::GREEN, theme_keys::GREEN_BG)
    } else {
        Space::with_width(0).into()
    };
    let agent_badge: Element<Message> = if k.loaded_in_agent {
        badge(tr(keys::AGENT_BADGE), theme_keys::TEAL, theme_keys::TEAL_BG)
    } else {
        Space::with_width(0).into()
    };
    let copy_btn: Element<Message> = if k.has_pub {
        ui::compact_action_button(
            tr(keys::COPY),
            theme::color(theme_keys::TEXT_SECONDARY),
            theme::color(theme_keys::BG_SURFACE),
            theme::color(theme_keys::BG_HOVER),
            theme::color(theme_keys::BORDER_SUBTLE),
            Some(Message::SshKeys(SshKeysMessage::CopyPublicKey(
                k.path.clone(),
            ))),
        )
    } else {
        Space::with_width(0).into()
    };
    let meta = format!(
        "{}{}",
        k.created
            .as_deref()
            .map(|d| format!("{} {}", tr(keys::CREATED_PREFIX), d))
            .unwrap_or_else(|| tr(keys::CREATED_UNKNOWN).into()),
        k.fingerprint
            .as_deref()
            .map(|f| format!(" · {}", f))
            .unwrap_or_default(),
    );

    container(
        row![
            column![
                text(&k.name)
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(2),
                text(&k.path)
                    .size(10)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(2),
                text(meta)
                    .size(10)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .width(Length::Fill),
            pub_badge,
            agent_badge,
            copy_btn,
        ]
        .align_y(Alignment::Center)
        .spacing(8),
    )
    .padding(Padding::from([11, 14]))
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
}

fn badge<'a>(
    label: &'a str,
    text_key: &'static str,
    background_key: &'static str,
) -> Element<'a, Message> {
    container(text(label).size(10).color(theme::color(text_key)))
        .padding(Padding::from([3, 8]))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(theme::color(background_key).into()),
            border: Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn ssh_empty_state<'a>() -> Element<'a, Message> {
    container(
        column![
            Space::with_height(18),
            icons::solid(Icon::Key, 54.0, theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(18),
            text(tr(keys::NO_KEYS))
                .size(18)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(8),
            text(tr(keys::NO_KEYS_HELP))
                .size(12)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(22),
            row![
                ui::primary_icon_button(
                    Icon::Key,
                    tr(keys::GENERATE_KEY),
                    Message::SshKeys(SshKeysMessage::GenerateKey),
                ),
                ui::secondary_icon_button(
                    Icon::Plus,
                    tr(keys::ADD_EXISTING),
                    Message::SshKeys(SshKeysMessage::AddExisting),
                ),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            Space::with_height(18),
            container(
                row![
                    icons::solid(Icon::Info, 12.0, theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_width(8),
                    text(tr(keys::CONNECT_PROVIDERS_HELP))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([10, 14]))
            .width(Length::Fill)
            .style(ui::surface_style()),
            Space::with_height(10),
        ]
        .align_x(Alignment::Center)
        .spacing(0),
    )
    .width(Length::Fill)
    .into()
}
