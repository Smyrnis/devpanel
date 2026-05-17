use super::state::{ModalState, SudoModal};
use crate::core::dry_run;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::sudo as keys, text as tr};
use crate::messages::Message;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, button, checkbox, column, container, row, text, text_input};
use iced::{Alignment, Border, Element, Length, Padding};

impl SudoModal {
    pub fn view(&self) -> Element<'_, Message> {
        let overlay_bg = theme::color(theme_keys::OVERLAY_MED);

        let dev_banner: Element<Message> = if dry_run::active() {
            container(
                row![
                    text(tr(keys::DEV_BADGE))
                        .size(9)
                        .color(theme::color(theme_keys::YELLOW)),
                    Space::with_width(6),
                    text(tr(keys::DRY_RUN_NOTE))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([7, 12]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::YELLOW_BG).into()),
                border: Border {
                    color: theme::color(theme_keys::YELLOW_BORDER),
                    width: 1.0,
                    radius: 7.0.into(),
                },
                ..Default::default()
            })
            .into()
        } else {
            Space::with_height(0).into()
        };

        let modal_content = match &self.state {
            ModalState::Hidden => return Space::with_width(0).into(),

            ModalState::Asking | ModalState::Failed => {
                let error_msg: Element<Message> = if self.state == ModalState::Failed {
                    error_message()
                } else {
                    Space::with_height(0).into()
                };

                use crate::messages::SudoMessage;

                let pass_input = text_input(
                    if self.show_password {
                        tr(keys::PASSWORD_PLACEHOLDER)
                    } else {
                        tr(keys::HIDDEN_PLACEHOLDER)
                    },
                    &self.password_input,
                )
                .on_input(|v| Message::Sudo(SudoMessage::PasswordChanged(v)))
                .on_submit(Message::Sudo(SudoMessage::Submit))
                .secure(!self.show_password)
                .padding(11)
                .size(13);

                let show_btn = button(
                    text(if self.show_password {
                        tr(keys::HIDE)
                    } else {
                        tr(keys::SHOW)
                    })
                    .size(12),
                )
                .on_press(Message::Sudo(SudoMessage::ToggleShow(!self.show_password)))
                .padding(Padding::from([11, 14]))
                .style(ui::ghost_button_style());

                let input_row = row![pass_input.width(Length::Fill), show_btn]
                    .spacing(8)
                    .align_y(Alignment::Center);

                let save_check = checkbox(tr(keys::SAVE_PASSWORD), self.save_password)
                    .on_toggle(|v| Message::Sudo(SudoMessage::ToggleSave(v)))
                    .size(14)
                    .text_size(13);

                let submit_btn = button(text(tr(keys::UNLOCK)).size(13))
                    .on_press(Message::Sudo(SudoMessage::Submit))
                    .padding(Padding::from([10, 28]))
                    .style(teal_btn_style());

                let cancel_btn = button(text(tr(keys::CANCEL)).size(13))
                    .on_press(Message::Sudo(SudoMessage::Cancel))
                    .padding(Padding::from([10, 18]))
                    .style(ui::ghost_button_style());

                column![
                    sudo_title(),
                    Space::with_height(6),
                    text(tr(keys::PROMPT))
                        .size(13)
                        .color(theme::color(theme_keys::TEXT_SECONDARY)),
                    Space::with_height(12),
                    dev_banner,
                    Space::with_height(20),
                    ui::divider(),
                    Space::with_height(16),
                    error_msg,
                    Space::with_height(if self.state == ModalState::Failed {
                        12
                    } else {
                        0
                    }),
                    text(tr(keys::PASSWORD_LABEL))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_height(6),
                    input_row,
                    Space::with_height(14),
                    save_check,
                    Space::with_height(24),
                    row![submit_btn, Space::with_width(10), cancel_btn].align_y(Alignment::Center),
                ]
                .spacing(0)
            }

            ModalState::Validating => column![
                text(tr(keys::TITLE))
                    .size(16)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
                Space::with_height(16),
                text(tr(keys::VERIFYING))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_SECONDARY)),
            ]
            .spacing(0),
        };

        let card = container(modal_content.padding(30))
            .width(420)
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::BG_ELEVATED).into()),
                border: Border {
                    color: theme::color(theme_keys::BORDER_MED),
                    width: 1.0,
                    radius: 16.0.into(),
                },
                shadow: iced::Shadow {
                    color: theme::color(theme_keys::SHADOW_HEAVY),
                    offset: iced::Vector::new(0.0, 12.0),
                    blur_radius: 48.0,
                },
                ..Default::default()
            });

        container(
            container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(overlay_bg.into()),
            ..Default::default()
        })
        .into()
    }
}

fn sudo_title() -> Element<'static, Message> {
    container(
        row![
            container(
                text(tr(keys::BADGE))
                    .size(10)
                    .color(theme::color(theme_keys::TEAL))
            )
            .padding(Padding::from([3, 8]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::TEAL_BG).into()),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(10),
            text(tr(keys::TITLE))
                .size(16)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
        ]
        .align_y(Alignment::Center),
    )
    .into()
}

fn error_message() -> Element<'static, Message> {
    container(
        row![
            container(text("x").size(10).color(theme::color(theme_keys::WHITE)))
                .padding(Padding::from([3, 6]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(theme::color(theme_keys::RED).into()),
                    border: Border {
                        radius: 20.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(10),
            text(tr(keys::INCORRECT_PASSWORD))
                .size(13)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 12]))
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::RED_BG).into()),
        border: Border {
            color: theme::color(theme_keys::RED_BORDER),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn teal_btn_style()
-> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_DIM).into()),
                text_color: theme::color(theme_keys::WHITE),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(theme::color(theme_keys::TEAL).into()),
            text_color: theme::color(theme_keys::TEXT_ON_ACCENT),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    }
}
