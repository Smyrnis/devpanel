use super::shared::small_action_btn;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::tools as keys, text as tr};
use crate::messages::{Message, ToolsMessage};
use crate::ui::tabs::tools::{PhpExtension, ToolsTab};
use crate::ui::templates::view as ui;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn php_exts_panel(tab: &ToolsTab) -> Element<'_, Message> {
    let active_ver: Option<String> = tab
        .php_releases
        .iter()
        .find(|r| r.is_active)
        .map(|r| r.version.clone());
    let ver_label = active_ver
        .as_deref()
        .unwrap_or(tr(keys::ACTIVE_VERSION_FALLBACK));

    let header = row![
        column![
            text(tr(keys::SECTION_PHP_EXTENSIONS))
                .size(14)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(3),
            text(format!(
                "{} {} {}",
                tr(keys::PHP_EXTENSIONS_HELP_PREFIX),
                ver_label,
                tr(keys::PHP_EXTENSIONS_HELP_SUFFIX)
            ))
            .size(11)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .width(Length::Fill),
        button(
            text(tr(keys::SCAN))
                .size(12)
                .color(theme::color(theme_keys::TEAL))
        )
        .on_press(Message::Tools(ToolsMessage::ScanPhpExts))
        .padding(Padding::from([7, 14]))
        .style(|_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                    text_color: theme::color(theme_keys::TEAL),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            _ => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_BG).into()),
                text_color: theme::color(theme_keys::TEAL),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }),
    ]
    .align_y(Alignment::Center);

    let q = tab.tool_search.to_lowercase();
    let rows: Vec<Element<Message>> = tab
        .php_exts
        .iter()
        .filter(|e| q.is_empty() || e.name.contains(&q) || e.pkg_suffix.contains(&q))
        .map(|e| php_ext_row(e, &active_ver))
        .collect();
    container(
        column![
            header,
            Space::with_height(18),
            ui::thin_line(),
            Space::with_height(14),
            column(rows).spacing(8),
            Space::with_height(16),
            container(
                row![
                    text("i").size(10).color(theme::color(theme_keys::BLUE)),
                    Space::with_width(8),
                    text(tr(keys::PHP_EXTENSIONS_NOTE))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center)
            )
            .padding(Padding::from([10, 12]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::BLUE_BG).into()),
                border: Border {
                    color: theme::color(theme_keys::BLUE_BORDER),
                    width: 1.0,
                    radius: 6.0.into()
                },
                ..Default::default()
            }),
        ]
        .spacing(0)
        .padding(Padding::from([22, 22])),
    )
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn php_ext_row<'a>(ext: &'a PhpExtension, active_ver: &Option<String>) -> Element<'a, Message> {
    let (dot_color, status_text) = if ext.installed {
        (theme::color(theme_keys::GREEN), tr(keys::STATUS_INSTALLED))
    } else {
        (
            theme::color(theme_keys::TEXT_MUTED),
            tr(keys::STATUS_NOT_INSTALLED),
        )
    };
    let pkg = match active_ver {
        Some(ver) => format!("php{}-{}", ver, ext.name),
        None => ext.pkg_suffix.clone(),
    };
    let action: Element<Message> = if ext.installed {
        small_action_btn(
            tr(keys::REMOVE),
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
            theme::color(theme_keys::RED_HOVER),
            Message::Tools(ToolsMessage::RemovePhpExt(pkg)),
        )
    } else {
        small_action_btn(
            tr(keys::INSTALL),
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            Message::Tools(ToolsMessage::InstallPhpExt(pkg)),
        )
    };
    container(
        row![
            ui::status_dot(dot_color),
            Space::with_width(12),
            column![
                row![
                    text(ext.name.as_str())
                        .size(13)
                        .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    Space::with_width(8),
                    text(ext.pkg_suffix.as_str())
                        .size(10)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center),
                Space::with_height(2),
                text(status_text).size(11).color(dot_color),
            ]
            .spacing(0)
            .width(Length::Fill),
            action,
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([12, 14]))
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
