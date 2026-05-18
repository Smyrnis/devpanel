use super::{KeyType, SshKeysTab, StatusKind};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::ssh_keys as keys, text as tr};
use crate::messages::{Message, SshKeysMessage};
use crate::ui::icons::{self, Icon};
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, button, column, container, radio, row, scrollable, text, text_input};
use iced::{Alignment, Border, Element, Length, Padding};

pub fn render(tab: &SshKeysTab, compact: bool) -> Element<'_, Message> {
    let header_fn = if compact {
        ui::page_header_compact
    } else {
        ui::page_header
    };
    let main_panels: Element<Message> = if compact {
        column![keys_panel(tab), Space::with_height(14), generate_panel(tab)]
            .spacing(0)
            .into()
    } else {
        row![keys_panel(tab), Space::with_width(14), generate_panel(tab)]
            .align_y(Alignment::Start)
            .into()
    };

    scrollable(
        column![
            header_fn(
                tr(keys::TITLE),
                tr(keys::SUBTITLE),
                vec![
                    ui::secondary_icon_button(
                        Icon::Folder,
                        tr(keys::OPEN_SSH_DIR),
                        Message::SshKeys(SshKeysMessage::OpenDir),
                    ),
                    ui::secondary_icon_button(
                        Icon::Plus,
                        tr(keys::ADD_EXISTING),
                        Message::SshKeys(SshKeysMessage::AddExisting),
                    ),
                    ui::primary_icon_button(
                        Icon::Key,
                        tr(keys::GENERATE_KEY),
                        Message::SshKeys(SshKeysMessage::GenerateKey),
                    ),
                ],
            ),
            Space::with_height(18),
            ssh_summary(tab, compact),
            Space::with_height(16),
            main_panels,
            Space::with_height(16),
            support_panels(tab, compact),
            Space::with_height(16),
            status_bar(tab),
            Space::with_height(20),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn support_panels(tab: &SshKeysTab, compact: bool) -> Element<'_, Message> {
    let providers = container(
        column![
            row![
                icons::solid(Icon::Repo, 15.0, theme::color(theme_keys::TEXT_SECONDARY)),
                Space::with_width(8),
                text(tr(keys::CONNECT_PROVIDERS))
                    .size(15)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
            ]
            .align_y(Alignment::Center),
            Space::with_height(6),
            text(tr(keys::CONNECT_PROVIDERS_HELP))
                .size(12)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(14),
            provider_hint(tr(keys::GITHUB), tr(keys::NOT_CONNECTED), Icon::Repo),
            Space::with_height(8),
            provider_hint(tr(keys::BITBUCKET), tr(keys::NOT_CONNECTED), Icon::Repo),
        ]
        .spacing(0),
    )
    .padding(Padding::from([18, 20]))
    .width(Length::FillPortion(1))
    .style(ui::card_style());

    let agent_loaded = tab
        .keys_list
        .iter()
        .filter(|key| key.loaded_in_agent)
        .count();
    let agent = container(
        column![
            row![
                icons::solid(
                    Icon::Terminal,
                    15.0,
                    theme::color(theme_keys::TEXT_SECONDARY)
                ),
                Space::with_width(8),
                text(tr(keys::SSH_AGENT))
                    .size(15)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
            ]
            .align_y(Alignment::Center),
            Space::with_height(6),
            text(tr(keys::SSH_AGENT_HELP))
                .size(12)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(16),
            row![
                text(tr(keys::STATUS_SUMMARY))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_width(Length::Fill),
                ui::status_dot(if agent_loaded > 0 {
                    theme::color(theme_keys::GREEN)
                } else {
                    theme::color(theme_keys::TEXT_MUTED)
                }),
                Space::with_width(7),
                text(if agent_loaded > 0 {
                    format!("{} {}", agent_loaded, tr(keys::AGENT_LOADED))
                } else {
                    tr(keys::NOT_CONNECTED).to_string()
                })
                .size(12)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(0),
    )
    .padding(Padding::from([18, 20]))
    .width(Length::FillPortion(1))
    .style(ui::card_style());

    if compact {
        column![providers, Space::with_height(12), agent].into()
    } else {
        row![providers, Space::with_width(14), agent]
            .align_y(Alignment::Start)
            .into()
    }
}

fn provider_hint<'a>(label: &'a str, status: &'a str, icon: Icon) -> Element<'a, Message> {
    container(
        row![
            icons::solid(icon, 15.0, theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_width(10),
            text(label)
                .size(12)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_width(Length::Fill),
            container(
                text(status)
                    .size(10)
                    .color(theme::color(theme_keys::TEXT_MUTED))
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
            }),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(ui::surface_style())
    .into()
}

fn ssh_summary(tab: &SshKeysTab, compact: bool) -> Element<'_, Message> {
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
    let summary_grid: Element<Message> = if compact {
        column![
            ui::metric_card_icon(
                Icon::Key,
                tr(keys::KEYS_FOUND),
                tab.keys_list.len().to_string()
            ),
            ui::metric_card_icon(Icon::Key, tr(keys::DEFAULT_KEY), default_key),
            ui::metric_card_icon(Icon::Copy, tr(keys::PUBLIC_KEYS), public_keys.to_string()),
            ui::metric_card_icon(
                Icon::Terminal,
                tr(keys::AGENT_LOADED),
                agent_loaded.to_string()
            ),
        ]
        .spacing(8)
        .into()
    } else {
        row![
            ui::metric_card_icon(
                Icon::Key,
                tr(keys::KEYS_FOUND),
                tab.keys_list.len().to_string()
            ),
            ui::metric_card_icon(Icon::Shield, tr(keys::DEFAULT_KEY), default_key),
            ui::metric_card_icon(Icon::Copy, tr(keys::PUBLIC_KEYS), public_keys.to_string()),
            ui::metric_card_icon(
                Icon::Terminal,
                tr(keys::AGENT_LOADED),
                agent_loaded.to_string()
            ),
        ]
        .spacing(12)
        .into()
    };

    container(
        column![
            text(tr(keys::STATUS_SUMMARY))
                .size(13)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(12),
            summary_grid,
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
            ui::primary_icon_button(
                Icon::Key,
                tr(keys::GENERATE_KEY),
                Message::SshKeys(SshKeysMessage::GenerateKey),
            ),
            Space::with_height(16),
            text(tr(keys::QUICK_ACTIONS))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(8),
            row![
                ui::secondary_icon_button(
                    Icon::Plus,
                    tr(keys::ADD_EXISTING),
                    Message::SshKeys(SshKeysMessage::AddExisting),
                ),
                ui::secondary_icon_button(
                    Icon::Folder,
                    tr(keys::OPEN_SSH_DIR),
                    Message::SshKeys(SshKeysMessage::OpenDir),
                ),
                ui::secondary_icon_button(
                    Icon::Refresh,
                    tr(keys::REFRESH),
                    Message::SshKeys(SshKeysMessage::ListKeys),
                ),
            ]
            .spacing(8),
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::FillPortion(1))
    .style(ui::card_style())
    .into()
}

fn keys_panel(tab: &SshKeysTab) -> Element<'_, Message> {
    let entries: Vec<Element<Message>> = if tab.keys_list.is_empty() {
        vec![ssh_empty_state()]
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
    .width(Length::FillPortion(1))
    .style(ui::card_style())
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

fn lbl<'a>(s: &'a str) -> Element<'a, Message> {
    text(s)
        .size(11)
        .color(theme::color(theme_keys::TEXT_MUTED))
        .into()
}
