use super::{KeyType, SshKeysTab, StatusKind};
use crate::core::theme::*;
use crate::messages::{Message, SshKeysMessage};
use iced::widget::{Space, button, column, container, radio, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub fn render(tab: &SshKeysTab) -> Element<'_, Message> {
    scrollable(
        column![
            column![
                text("SSH Key Manager").size(22).color(TEXT_PRIMARY),
                Space::with_height(4),
                text("Generate and manage SSH keys for this machine")
                    .size(13)
                    .color(TEXT_MUTED),
            ]
            .spacing(0),
            Space::with_height(22),
            row![generate_panel(tab), Space::with_width(14), keys_panel(tab),]
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

fn generate_panel(tab: &SshKeysTab) -> Element<'_, Message> {
    let type_row = row![
        radio("Ed25519", KeyType::Ed25519, Some(tab.key_type), |_| {
            Message::SshKeys(SshKeysMessage::KeyTypeChanged(KeyType::Ed25519))
        }),
        Space::with_width(14),
        radio("RSA 4096", KeyType::Rsa4096, Some(tab.key_type), |_| {
            Message::SshKeys(SshKeysMessage::KeyTypeChanged(KeyType::Rsa4096))
        }),
        Space::with_width(14),
        radio("ECDSA", KeyType::Ecdsa, Some(tab.key_type), |_| {
            Message::SshKeys(SshKeysMessage::KeyTypeChanged(KeyType::Ecdsa))
        }),
    ]
    .align_y(Alignment::Center);

    let pass_row = row![
        text_input(
            if tab.show_passphrase {
                "passphrase"
            } else {
                "(hidden)"
            },
            &tab.passphrase,
        )
        .on_input(|v| Message::SshKeys(SshKeysMessage::PassphraseChanged(v)))
        .secure(!tab.show_passphrase)
        .padding(10)
        .size(13)
        .width(Length::Fill),
        button(text(if tab.show_passphrase { "Hide" } else { "Show" }).size(12))
            .on_press(Message::SshKeys(SshKeysMessage::TogglePassphrase(
                !tab.show_passphrase
            )))
            .padding(Padding::from([10, 14]))
            .style(ghost_style()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let divider = container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_SUBTLE.into()),
            ..Default::default()
        });

    container(
        column![
            text("Generate New Key").size(14).color(TEXT_SECONDARY),
            Space::with_height(18),
            lbl("Email address"),
            Space::with_height(5),
            text_input("user@example.com", &tab.email)
                .on_input(|v| Message::SshKeys(SshKeysMessage::EmailChanged(v)))
                .padding(10)
                .size(13),
            Space::with_height(14),
            lbl("Key filename"),
            Space::with_height(5),
            text_input("id_ed25519", &tab.key_name)
                .on_input(|v| Message::SshKeys(SshKeysMessage::KeyNameChanged(v)))
                .padding(10)
                .size(13),
            Space::with_height(14),
            lbl("Key type"),
            Space::with_height(8),
            type_row,
            Space::with_height(14),
            lbl("Passphrase (optional)"),
            Space::with_height(5),
            pass_row,
            Space::with_height(20),
            divider,
            Space::with_height(16),
            button(text("Generate Key").size(13))
                .on_press(Message::SshKeys(SshKeysMessage::GenerateKey))
                .padding(Padding::from([10, 22]))
                .style(btn_style(ACCENT)),
            Space::with_height(16),
            text("Quick Actions").size(11).color(TEXT_MUTED),
            Space::with_height(8),
            row![
                button(text("Add Existing").size(12))
                    .on_press(Message::SshKeys(SshKeysMessage::AddExisting))
                    .padding(Padding::from([8, 14]))
                    .style(ghost_style()),
                button(text("Open ~/.ssh").size(12))
                    .on_press(Message::SshKeys(SshKeysMessage::OpenDir))
                    .padding(Padding::from([8, 14]))
                    .style(ghost_style()),
                button(text("Refresh").size(12))
                    .on_press(Message::SshKeys(SshKeysMessage::ListKeys))
                    .padding(Padding::from([8, 14]))
                    .style(ghost_style()),
            ]
            .spacing(8),
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::FillPortion(3))
    .style(card_style())
    .into()
}

fn keys_panel(tab: &SshKeysTab) -> Element<'_, Message> {
    let entries: Vec<Element<Message>> = if tab.keys_list.is_empty() {
        vec![
            container(
                text("No keys found. Click Refresh.")
                    .size(13)
                    .color(TEXT_MUTED),
            )
            .padding(Padding::from([20, 16]))
            .into(),
        ]
    } else {
        tab.keys_list
            .iter()
            .map(|k| {
                let pub_badge: Element<Message> = if k.has_pub {
                    container(text(".pub").size(10).color(GREEN))
                        .padding(Padding::from([3, 8]))
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(
                                Color {
                                    r: 0.050,
                                    g: 0.160,
                                    b: 0.090,
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
                    Space::with_width(0).into()
                };

                container(
                    row![
                        column![
                            text(&k.name).size(13).color(TEXT_PRIMARY),
                            Space::with_height(2),
                            text(&k.path).size(10).color(TEXT_MUTED),
                        ]
                        .spacing(0)
                        .width(Length::Fill),
                        pub_badge,
                    ]
                    .align_y(Alignment::Center)
                    .spacing(8),
                )
                .padding(Padding::from([11, 14]))
                .width(Length::Fill)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(BG_SURFACE.into()),
                    border: Border {
                        color: BORDER_SUBTLE,
                        width: 1.0,
                        radius: 8.0.into(),
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
                text("SSH Keys")
                    .size(14)
                    .color(TEXT_SECONDARY)
                    .width(Length::Fill),
                container(
                    text(format!("{}", tab.keys_list.len()))
                        .size(11)
                        .color(TEXT_MUTED)
                )
                .padding(Padding::from([3, 8]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(BG_SURFACE.into()),
                    border: Border {
                        color: BORDER_SUBTLE,
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
    .width(Length::FillPortion(2))
    .style(card_style())
    .into()
}

fn status_bar(tab: &SshKeysTab) -> Element<'_, Message> {
    if tab.status_kind == StatusKind::None || tab.status_message.is_empty() {
        return Space::with_height(0).into();
    }
    let (color, border_color, icon) = match tab.status_kind {
        StatusKind::Success => (
            GREEN,
            Color {
                r: 0.070,
                g: 0.210,
                b: 0.110,
                a: 1.0,
            },
            "+",
        ),
        StatusKind::Error => (
            RED,
            Color {
                r: 0.300,
                g: 0.090,
                b: 0.080,
                a: 1.0,
            },
            "x",
        ),
        StatusKind::Info => (
            BLUE,
            Color {
                r: 0.080,
                g: 0.140,
                b: 0.260,
                a: 1.0,
            },
            "i",
        ),
        StatusKind::None => (TEXT_MUTED, BORDER_SUBTLE, ""),
    };
    container(
        row![
            container(text(icon).size(10).color(Color::WHITE))
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
            text(&tab.status_message).size(13).color(TEXT_PRIMARY),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([12, 16]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn card_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}
fn btn_style(
    bg: Color,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(Color::from_rgba(bg.r, bg.g, bg.b, 0.82).into()),
                text_color: Color::WHITE,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                shadow: iced::Shadow {
                    color: Color { a: 0.3, ..bg },
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: Color::WHITE,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    }
}
fn ghost_style()
-> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(BG_HOVER.into()),
                text_color: TEXT_PRIMARY,
                border: Border {
                    color: BORDER_MED,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(BG_CARD.into()),
            text_color: TEXT_SECONDARY,
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
    }
}
fn lbl<'a>(s: &'a str) -> Element<'a, Message> {
    text(s).size(11).color(TEXT_MUTED).into()
}
