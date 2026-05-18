use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::tabs::tools::ToolsTab;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn log_panel(tab: &ToolsTab) -> Element<'_, Message> {
    if tab.install_log.is_empty() {
        return Space::with_height(0).into();
    }

    let rows: Vec<Element<Message>> = tab
        .install_log
        .iter()
        .map(|(ok, msg)| {
            let (prefix, color) = if *ok {
                (tr(keys::LOG_OK), theme::color(theme_keys::GREEN))
            } else {
                (tr(keys::LOG_ERR), theme::color(theme_keys::RED))
            };
            row![
                text(prefix).size(11).color(color),
                text(msg.as_str())
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_SECONDARY))
            ]
            .into()
        })
        .collect();

    container(
        column![
            row![
                text(tr(keys::ACTIVITY_LOG))
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_MUTED))
                    .width(Length::Fill),
                button(
                    text(tr(keys::CLEAR))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                )
                .on_press(Message::Tools(ToolsMessage::ClearLog))
                .padding(Padding::from([4, 10]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::BG_HOVER).into()),
                        text_color: theme::color(theme_keys::TEXT_PRIMARY),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: None,
                        text_color: theme::color(theme_keys::TEXT_MUTED),
                        ..Default::default()
                    },
                }),
            ]
            .align_y(Alignment::Center),
            Space::with_height(10),
            scrollable(column(rows).spacing(5).padding(Padding::from([4, 0]))).height(150),
        ]
        .spacing(0)
        .padding(Padding::from([16, 18])),
    )
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

pub(super) fn error_suggestion_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let php_version = tab
        .install_log
        .iter()
        .rev()
        .find_map(|(ok, msg)| {
            if !*ok && msg.contains("PHP") {
                msg.split("PHP ")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "8.2".to_string());

    let fix_commands = format!(
        "# Add the packages.sury.org/php repository.\n\
         sudo apt-get update\n\
         sudo apt-get install -y lsb-release ca-certificates apt-transport-https curl\n\
         sudo curl -sSLo /tmp/debsuryorg-archive-keyring.deb https://packages.sury.org/debsuryorg-archive-keyring.deb\n\
         sudo dpkg -i /tmp/debsuryorg-archive-keyring.deb\n\
         sudo sh -c 'echo \"deb [signed-by=/usr/share/keyrings/debsuryorg-archive-keyring.gpg] \
         https://packages.sury.org/php/ $(lsb_release -sc) main\" > /etc/apt/sources.list.d/php.list'\n\
         sudo apt-get update\n\n# Install PHP\nsudo apt-get install -y php{}",
        php_version
    );

    container(
        column![
            row![
                text(tr(keys::PHP_NOT_FOUND))
                    .size(13)
                    .color(theme::color(theme_keys::ORANGE)),
                Space::with_width(Length::Fill),
            ]
            .align_y(Alignment::Center),
            Space::with_height(10),
            text(tr(keys::PHP_PPA_MISSING))
                .size(12)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(12),
            container(
                scrollable(
                    text(fix_commands.clone())
                        .size(10)
                        .color(theme::color(theme_keys::BORDER_MED))
                )
                .height(180)
            )
            .padding(Padding::from([12, 14]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::BG_CARD).into()),
                border: Border {
                    color: theme::color(theme_keys::BORDER_SUBTLE),
                    width: 1.0,
                    radius: 6.0.into()
                },
                ..Default::default()
            }),
            Space::with_height(10),
            button(
                text(tr(keys::GET_TEXT_FILE))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_PRIMARY))
            )
            .on_press(Message::Tools(ToolsMessage::CopyFixCommands(fix_commands)))
            .padding(Padding::from([6, 12]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::BLUE_HOVER).into()),
                    text_color: theme::color(theme_keys::WHITE),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                _ => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::BLUE_BG).into()),
                    text_color: theme::color(theme_keys::TEXT_PRIMARY),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }),
        ]
        .spacing(0),
    )
    .width(Length::Fill)
    .padding(Padding::from([16, 18]))
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::YELLOW_BG).into()),
        border: Border {
            color: theme::color(theme_keys::YELLOW_BORDER),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
