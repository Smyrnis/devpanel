use super::{KeyType, SshKeysTab, StatusKind};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::ssh_keys as keys, text as tr};
use crate::messages::{Message, SshKeysMessage};
use crate::ui::templates::view as ui;
use iced::widget::{Space, button, column, container, radio, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub fn render(tab: &SshKeysTab) -> Element<'_, Message> {
    scrollable(
        column![
            ui::page_header(
                tr(keys::TITLE),
                tr(keys::SUBTITLE),
                vec![
                    ui::secondary_button(
                        tr(keys::OPEN_SSH_DIR),
                        Message::SshKeys(SshKeysMessage::OpenDir),
                    ),
                    ui::secondary_button(
                        tr(keys::ADD_EXISTING),
                        Message::SshKeys(SshKeysMessage::AddExisting),
                    ),
                    ui::primary_button(
                        tr(keys::GENERATE_KEY),
                        Message::SshKeys(SshKeysMessage::GenerateKey),
                    ),
                ],
            ),
            Space::with_height(18),
            ssh_summary(tab),
            Space::with_height(16),
            row![keys_panel(tab), Space::with_width(14), generate_panel(tab)]
                .align_y(Alignment::Start),
            Space::with_height(16),
            status_bar(tab),
            Space::with_height(20),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn ssh_summary(tab: &SshKeysTab) -> Element<'_, Message> {
    let public_keys = tab.keys_list.iter().filter(|key| key.has_pub).count();
    let agent_loaded = tab
        .keys_list
        .iter()
        .filter(|key| key.loaded_in_agent)
        .count();
    let default_key = tab
        .keys_list
        .first()
        .map(|key| key.name.clone())
        .unwrap_or_else(|| tr(keys::NONE).to_string());

    container(
        column![
            text(tr(keys::STATUS_SUMMARY))
                .size(13)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(12),
            row![
                ui::metric_card(tr(keys::KEYS_FOUND), tab.keys_list.len().to_string()),
                ui::metric_card(tr(keys::DEFAULT_KEY), default_key),
                ui::metric_card(tr(keys::PUBLIC_KEYS), public_keys.to_string()),
                ui::metric_card(tr(keys::AGENT_LOADED), agent_loaded.to_string()),
            ]
            .spacing(12),
        ]
        .spacing(0),
    )
    .padding(Padding::from([14, 16]))
    .width(Length::Fill)
    .style(ui::surface_style())
    .into()
}

fn generate_panel(tab: &SshKeysTab) -> Element<'_, Message> {
    let type_row = row![
        radio(
            tr(keys::KEY_ED25519),
            KeyType::Ed25519,
            Some(tab.key_type),
            |_| { Message::SshKeys(SshKeysMessage::KeyTypeChanged(KeyType::Ed25519)) }
        ),
        Space::with_width(14),
        radio(
            tr(keys::KEY_RSA_4096),
            KeyType::Rsa4096,
            Some(tab.key_type),
            |_| { Message::SshKeys(SshKeysMessage::KeyTypeChanged(KeyType::Rsa4096)) }
        ),
        Space::with_width(14),
        radio(
            tr(keys::KEY_ECDSA),
            KeyType::Ecdsa,
            Some(tab.key_type),
            |_| { Message::SshKeys(SshKeysMessage::KeyTypeChanged(KeyType::Ecdsa)) }
        ),
    ]
    .align_y(Alignment::Center);

    let pass_row = row![
        text_input(
            if tab.show_passphrase {
                tr(keys::PASSPHRASE_PLACEHOLDER)
            } else {
                tr(keys::HIDDEN_PLACEHOLDER)
            },
            &tab.passphrase
        )
        .on_input(|v| Message::SshKeys(SshKeysMessage::PassphraseChanged(v)))
        .secure(!tab.show_passphrase)
        .padding(10)
        .size(13)
        .width(Length::Fill),
        button(
            text(if tab.show_passphrase {
                tr(keys::HIDE)
            } else {
                tr(keys::SHOW)
            })
            .size(12)
        )
        .on_press(Message::SshKeys(SshKeysMessage::TogglePassphrase(
            !tab.show_passphrase
        )))
        .padding(Padding::from([10, 14]))
        .style(ui::ghost_button_style()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let divider = container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BORDER_SUBTLE).into()),
            ..Default::default()
        });

    container(
        column![
            text(tr(keys::GENERATE_PANEL_TITLE))
                .size(14)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(18),
            lbl(tr(keys::EMAIL_LABEL)),
            Space::with_height(5),
            text_input(tr(keys::EMAIL_PLACEHOLDER), &tab.email)
                .on_input(|v| Message::SshKeys(SshKeysMessage::EmailChanged(v)))
                .padding(10)
                .size(13),
            Space::with_height(14),
            lbl(tr(keys::KEY_FILENAME_LABEL)),
            Space::with_height(5),
            text_input(tr(keys::KEY_FILENAME_PLACEHOLDER), &tab.key_name)
                .on_input(|v| Message::SshKeys(SshKeysMessage::KeyNameChanged(v)))
                .padding(10)
                .size(13),
            Space::with_height(14),
            lbl(tr(keys::KEY_TYPE_LABEL)),
            Space::with_height(8),
            type_row,
            Space::with_height(14),
            lbl(tr(keys::PASSPHRASE_LABEL)),
            Space::with_height(5),
            pass_row,
            Space::with_height(20),
            divider,
            Space::with_height(16),
            button(text(tr(keys::GENERATE_KEY)).size(13))
                .on_press(Message::SshKeys(SshKeysMessage::GenerateKey))
                .padding(Padding::from([10, 22]))
                .style(btn_style(theme::color(theme_keys::ACCENT))),
            Space::with_height(16),
            text(tr(keys::QUICK_ACTIONS))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(8),
            row![
                button(text(tr(keys::ADD_EXISTING)).size(12))
                    .on_press(Message::SshKeys(SshKeysMessage::AddExisting))
                    .padding(Padding::from([8, 14]))
                    .style(ui::ghost_button_style()),
                button(text(tr(keys::OPEN_SSH_DIR)).size(12))
                    .on_press(Message::SshKeys(SshKeysMessage::OpenDir))
                    .padding(Padding::from([8, 14]))
                    .style(ui::ghost_button_style()),
                button(text(tr(keys::REFRESH)).size(12))
                    .on_press(Message::SshKeys(SshKeysMessage::ListKeys))
                    .padding(Padding::from([8, 14]))
                    .style(ui::ghost_button_style()),
            ]
            .spacing(8),
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::FillPortion(2))
    .style(ui::card_style())
    .into()
}

fn keys_panel(tab: &SshKeysTab) -> Element<'_, Message> {
    let entries: Vec<Element<Message>> = if tab.keys_list.is_empty() {
        vec![ui::empty_state(
            tr(keys::NO_KEYS),
            tr(keys::NO_KEYS_HELP),
            vec![
                ui::primary_button(
                    tr(keys::GENERATE_KEY),
                    Message::SshKeys(SshKeysMessage::GenerateKey),
                ),
                ui::secondary_button(
                    tr(keys::ADD_EXISTING),
                    Message::SshKeys(SshKeysMessage::AddExisting),
                ),
            ],
        )]
    } else {
        tab.keys_list
            .iter()
            .map(|k| {
                let pub_badge: Element<Message> = if k.has_pub {
                    container(
                        text(tr(keys::PUB_BADGE))
                            .size(10)
                            .color(theme::color(theme_keys::GREEN)),
                    )
                    .padding(Padding::from([3, 8]))
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(theme::color(theme_keys::GREEN_BG).into()),
                        border: Border {
                            radius: 20.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .into()
                } else {
                    Space::with_width(0).into()
                };
                let agent_badge: Element<Message> = if k.loaded_in_agent {
                    container(
                        text(tr(keys::AGENT_BADGE))
                            .size(10)
                            .color(theme::color(theme_keys::TEAL)),
                    )
                    .padding(Padding::from([3, 8]))
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(theme::color(theme_keys::TEAL_BG).into()),
                        border: Border {
                            radius: 20.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .into()
                } else {
                    Space::with_width(0).into()
                };
                let copy_btn: Element<Message> = if k.has_pub {
                    button(
                        text(tr(keys::COPY))
                            .size(11)
                            .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    )
                    .on_press(Message::SshKeys(SshKeysMessage::CopyPublicKey(
                        k.path.clone(),
                    )))
                    .padding(Padding::from([5, 10]))
                    .style(ui::ghost_button_style())
                    .into()
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
            })
            .collect()
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
    .width(Length::FillPortion(3))
    .style(ui::card_style())
    .into()
}

fn status_bar(tab: &SshKeysTab) -> Element<'_, Message> {
    if tab.status_kind == StatusKind::None || tab.status_message.is_empty() {
        return Space::with_height(0).into();
    }
    let (color, border_color, icon) = match tab.status_kind {
        StatusKind::Success => (
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            "+",
        ),
        StatusKind::Error => (
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BORDER),
            "x",
        ),
        StatusKind::Info => (
            theme::color(theme_keys::BLUE),
            theme::color(theme_keys::BLUE_BORDER),
            "i",
        ),
        StatusKind::None => (
            theme::color(theme_keys::TEXT_MUTED),
            theme::color(theme_keys::BORDER_SUBTLE),
            "",
        ),
    };
    container(
        row![
            container(text(icon).size(10).color(theme::color(theme_keys::WHITE)))
                .padding(Padding::from([3, 6]))
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(color.into()),
                    border: Border {
                        radius: 20.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(10),
            text(&tab.status_message)
                .size(13)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([12, 16]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn btn_style(
    bg: Color,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(Color::from_rgba(bg.r, bg.g, bg.b, 0.82).into()),
                text_color: theme::color(theme_keys::WHITE),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                shadow: iced::Shadow {
                    color: Color { a: 0.3, ..bg },
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
            }
        }
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: theme::color(theme_keys::WHITE),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    }
}
fn lbl<'a>(s: &'a str) -> Element<'a, Message> {
    text(s)
        .size(11)
        .color(theme::color(theme_keys::TEXT_MUTED))
        .into()
}
