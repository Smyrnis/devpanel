use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::vhosts as keys, text as tr};
use crate::messages::{Message, VHostsMessage};
use crate::ui::tabs::vhosts::VHostsTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, column, container, row, text, text_editor};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn config_editor_view(tab: &VHostsTab) -> Element<'_, Message> {
    let header = row![
        column![
            text(tr(keys::CONFIG_EDITOR))
                .size(crate::core::app_config::text_metrics().title)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(4),
            text(tab.devpanel_conf.as_str())
                .size(crate::core::app_config::text_metrics().caption)
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
                .size(crate::core::app_config::text_metrics().tiny)
                .color(theme::color(theme_keys::YELLOW)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::YELLOW_BG).into()),
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
