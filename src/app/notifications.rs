use crate::app::{App, Toast};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::app as keys, text as tr};
use crate::messages::Message;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding, Task};

impl App {
    pub fn show_toast(&mut self, message: String, ok: bool) -> Task<Message> {
        let remaining_ms = self.config_tab.settings.ui_toast_duration_ms.max(1000);
        if let Some(db) = &self.db {
            let _ = db.add_notification(ok, &message);
        }
        self.notifications.push(Toast {
            message,
            ok,
            remaining_ms,
        });
        Task::none()
    }

    pub(crate) fn notification_overlay(&self) -> Element<'_, Message> {
        if self.notifications.is_empty() {
            return Space::with_height(0).into();
        }

        let mut cards: Vec<Element<Message>> = self
            .notifications
            .iter()
            .rev()
            .take(3)
            .map(notification_card)
            .collect();
        cards.insert(
            0,
            ui::compact_action_button(
                tr(keys::DISMISS_ALL),
                theme::color(theme_keys::TEXT_MUTED),
                theme::color(theme_keys::BG_SURFACE),
                theme::color(theme_keys::BG_HOVER),
                theme::color(theme_keys::BORDER_SUBTLE),
                Some(Message::DismissAllNotifications),
            ),
        );

        container(
            column![
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(Length::Fill),
                    column(cards)
                        .spacing(8)
                        .width(crate::core::app_config::panel_metrics().notification_width),
                ],
            ]
            .padding(Padding::from([20, 20])),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn notification_card(toast: &Toast) -> Element<'_, Message> {
    let (accent, border_color) = if toast.ok {
        (
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_HOVER),
        )
    } else {
        (
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BORDER),
        )
    };
    let seconds = (toast.remaining_ms / 1000).max(1);
    container(
        row![
            container(
                text(if toast.ok { "+" } else { "x" })
                    .size(crate::core::app_config::text_metrics().caption)
                    .color(theme::color(theme_keys::TEXT_ON_ACCENT))
            )
            .padding(Padding::from([3, 7]))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(accent.into()),
                border: Border {
                    radius: 20.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(10),
            text(toast.message.as_str())
                .size(crate::core::app_config::text_metrics().body)
                .color(theme::color(theme_keys::TEXT_PRIMARY))
                .width(Length::Fill),
            text(format!("{}s", seconds))
                .size(crate::core::app_config::text_metrics().tiny)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 14]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow {
            color: theme::color(theme_keys::SHADOW_HEAVY),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    })
    .into()
}
