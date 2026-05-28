use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::ssh_keys::KeyType;
use crate::lang::{lang_map::ssh_keys as keys, text as tr};
use crate::messages::{Message, SshKeysMessage};
use crate::ui::icons::Icon;
use crate::ui::tabs::ssh_keys::SshKeysTab;
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{Space, column, container, radio, row, text, text_input};
use iced::{Alignment, Element, Length, Padding};

pub(super) fn generate_panel(tab: &SshKeysTab) -> Element<'_, Message> {
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
        .style(styles::text_input_style)
        .width(Length::Fill),
        ui::ghost_text_button(
            if tab.show_passphrase {
                tr(keys::HIDE)
            } else {
                tr(keys::SHOW)
            },
            Message::SshKeys(SshKeysMessage::TogglePassphrase(!tab.show_passphrase)),
        ),
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
            label(tr(keys::EMAIL_LABEL)),
            Space::with_height(5),
            text_input(tr(keys::EMAIL_PLACEHOLDER), &tab.email)
                .on_input(|v| Message::SshKeys(SshKeysMessage::EmailChanged(v)))
                .padding(10)
                .size(13)
                .style(styles::text_input_style),
            Space::with_height(14),
            label(tr(keys::KEY_FILENAME_LABEL)),
            Space::with_height(5),
            text_input(tr(keys::KEY_FILENAME_PLACEHOLDER), &tab.key_name)
                .on_input(|v| Message::SshKeys(SshKeysMessage::KeyNameChanged(v)))
                .padding(10)
                .size(13)
                .style(styles::text_input_style),
            Space::with_height(14),
            label(tr(keys::KEY_TYPE_LABEL)),
            Space::with_height(8),
            type_row,
            Space::with_height(14),
            label(tr(keys::PASSPHRASE_LABEL)),
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

fn label<'a>(s: &'a str) -> Element<'a, Message> {
    text(s)
        .size(11)
        .color(theme::color(theme_keys::TEXT_MUTED))
        .into()
}
