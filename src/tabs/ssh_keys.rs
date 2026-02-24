// src/tabs/ssh_keys.rs

use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, radio, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

fn card_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
        ..Default::default()
    }
}

fn btn_style(
    bg: Color,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(Color::from_rgb(bg.r * 0.82, bg.g * 0.82, bg.b * 0.82).into()),
                text_color: Color::WHITE,
                border: Border { radius: 6.0.into(), ..Default::default() },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: Color::WHITE,
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        },
    }
}

fn ghost_style() -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(BG_HOVER.into()),
                text_color: TEXT_PRIMARY,
                border: Border { color: ACCENT, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(BG_SURFACE.into()),
            text_color: TEXT_SECONDARY,
            border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 6.0.into() },
            ..Default::default()
        },
    }
}

// ── Key type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Ed25519,
    Rsa4096,
    Ecdsa,
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Ed25519 => write!(f, "Ed25519"),
            KeyType::Rsa4096 => write!(f, "RSA 4096"),
            KeyType::Ecdsa   => write!(f, "ECDSA 521"),
        }
    }
}

// ── Status ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum StatusKind {
    None,
    Success,
    Error,
    #[allow(dead_code)]
    Info,
}

// ── State ─────────────────────────────────────────────────────────────────

pub struct SshKeysTab {
    pub email: String,
    pub key_name: String,
    pub key_type: KeyType,
    pub passphrase: String,
    pub show_passphrase: bool,
    pub status_message: String,
    pub status_kind: StatusKind,
    // list of detected keys
    pub keys_list: Vec<KeyEntry>,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub name: String,
    pub path: String,
    pub has_pub: bool,
}

impl SshKeysTab {
    pub fn new() -> Self {
        Self {
            email: String::new(),
            key_name: String::new(),
            key_type: KeyType::Ed25519,
            passphrase: String::new(),
            show_passphrase: false,
            status_message: String::new(),
            status_kind: StatusKind::None,
            keys_list: Vec::new(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let left = self.generate_panel();
        let right = self.keys_panel();

        scrollable(
            column![
                column![
                    text("SSH Key Manager").size(26).color(TEXT_PRIMARY),
                    text("Generate, manage and add SSH keys for Debian/Ubuntu")
                        .size(13)
                        .color(TEXT_MUTED),
                ]
                .spacing(3),
                Space::with_height(16),
                row![left, Space::with_width(14), right].align_y(Alignment::Start),
                Space::with_height(16),
                self.status_bar(),
            ]
            .spacing(0)
            .padding(Padding::from([20, 24])),
        )
        .into()
    }

    // ── Generate panel ────────────────────────────────────────────────────

    fn generate_panel(&self) -> Element<'_, Message> {
        let type_row = row![
            radio("Ed25519",  KeyType::Ed25519, Some(self.key_type), |_| Message::SSH_KeyTypeChanged(KeyType::Ed25519)),
            Space::with_width(16),
            radio("RSA 4096", KeyType::Rsa4096, Some(self.key_type), |_| Message::SSH_KeyTypeChanged(KeyType::Rsa4096)),
            Space::with_width(16),
            radio("ECDSA",    KeyType::Ecdsa,   Some(self.key_type), |_| Message::SSH_KeyTypeChanged(KeyType::Ecdsa)),
        ]
        .align_y(Alignment::Center);

        let pass_row = row![
            text_input(
                if self.show_passphrase { "passphrase" } else { "(hidden)" },
                &self.passphrase
            )
            .on_input(Message::SSH_PassphraseChanged)
            .secure(!self.show_passphrase)
            .padding(10)
            .size(14)
            .width(Length::Fill),
            button(text(if self.show_passphrase { "Hide" } else { "Show" }).size(12))
                .on_press(Message::SSH_TogglePassphrase(!self.show_passphrase))
                .padding(Padding::from([10, 14]))
                .style(ghost_style()),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        container(
            column![
                text("Generate New Key").size(15).color(TEXT_SECONDARY),
                Space::with_height(14),
                lbl("Email address"),
                Space::with_height(4),
                text_input("user@example.com", &self.email)
                    .on_input(Message::SSH_EmailChanged)
                    .padding(10)
                    .size(14),
                Space::with_height(10),
                lbl("Key filename"),
                Space::with_height(4),
                text_input("id_ed25519", &self.key_name)
                    .on_input(Message::SSH_KeyNameChanged)
                    .padding(10)
                    .size(14),
                Space::with_height(10),
                lbl("Key type"),
                Space::with_height(6),
                type_row,
                Space::with_height(10),
                lbl("Passphrase (optional)"),
                Space::with_height(4),
                pass_row,
                Space::with_height(16),
                button(text(">> Generate Key").size(14))
                    .on_press(Message::SSH_GenerateKey)
                    .padding(Padding::from([10, 22]))
                    .style(btn_style(ACCENT)),
                Space::with_height(12),
                // quick actions
                text("Quick Actions").size(12).color(TEXT_MUTED),
                Space::with_height(6),
                row![
                    button(text("+ Add Existing").size(12))
                        .on_press(Message::SSH_AddExisting)
                        .padding(Padding::from([8, 14]))
                        .style(ghost_style()),
                    button(text("@ Open ~/.ssh").size(12))
                        .on_press(Message::SSH_OpenDir)
                        .padding(Padding::from([8, 14]))
                        .style(ghost_style()),
                    button(text("~ Refresh List").size(12))
                        .on_press(Message::SSH_ListKeys)
                        .padding(Padding::from([8, 14]))
                        .style(ghost_style()),
                ]
                .spacing(8),
            ]
            .spacing(0)
            .padding(20),
        )
        .width(Length::FillPortion(3))
        .style(card_style())
        .into()
    }

    // ── Keys list panel ───────────────────────────────────────────────────

    fn keys_panel(&self) -> Element<'_, Message> {
        let entries: Vec<Element<Message>> = if self.keys_list.is_empty() {
            vec![text("No keys found. Click Refresh.")
                .size(13)
                .color(TEXT_MUTED)
                .into()]
        } else {
            self.keys_list
                .iter()
                .map(|k| {
                    let pub_badge: Element<Message> = if k.has_pub {
                        container(text(".pub [ok]").size(10).color(GREEN))
                            .padding(Padding::from([2, 6]))
                            .style(|_: &iced::Theme| container::Style {
                                background: Some(Color { a: 0.12, ..GREEN }.into()),
                                border: Border {
                                    color: Color { a: 0.35, ..GREEN },
                                    width: 1.0,
                                    radius: 3.0.into(),
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
                                text(&k.path).size(10).color(TEXT_MUTED),
                            ]
                            .spacing(2)
                            .width(Length::Fill),
                            pub_badge,
                        ]
                        .align_y(Alignment::Center)
                        .spacing(8),
                    )
                    .padding(Padding::from([10, 12]))
                    .width(Length::Fill)
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(BG_SURFACE.into()),
                        border: Border {
                            color: BORDER_SUBTLE,
                            width: 1.0,
                            radius: 5.0.into(),
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
                    text("SSH Keys").size(15).color(TEXT_SECONDARY).width(Length::Fill),
                    text(format!("{} found", self.keys_list.len()))
                        .size(11)
                        .color(TEXT_MUTED),
                ]
                .align_y(Alignment::Center),
                Space::with_height(12),
                scrollable(column(entries).spacing(6)).height(360),
            ]
            .spacing(0)
            .padding(20),
        )
        .width(Length::FillPortion(2))
        .style(card_style())
        .into()
    }

    // ── Status bar ────────────────────────────────────────────────────────

    fn status_bar(&self) -> Element<'_, Message> {
        if self.status_kind == StatusKind::None || self.status_message.is_empty() {
            return Space::with_height(0).into();
        }
        let (color, icon) = match self.status_kind {
            StatusKind::Success => (GREEN, "[+]"),
            StatusKind::Error   => (RED,   "[-]"),
            StatusKind::Info    => (BLUE,  "[i]"),
            StatusKind::None    => (TEXT_MUTED, ""),
        };
        container(
            row![
                text(icon).size(14).color(color),
                text(format!("  {}", self.status_message))
                    .size(13)
                    .color(color),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 16]))
        .width(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(Color { a: 0.08, ..color }.into()),
            border: Border {
                color: Color { a: 0.35, ..color },
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

fn lbl<'a>(s: &'a str) -> Element<'a, Message> {
    text(s).size(12).color(TEXT_MUTED).into()
}
