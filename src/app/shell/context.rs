use crate::app::App;
use crate::core::{
    dry_run,
    theme::{self, theme_map as theme_keys},
};
use crate::lang::{lang_map::app as keys, text as tr};
use crate::messages::Message;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, container, row, text};
use iced::{Alignment, Border, Element, Length, Padding};

impl App {
    pub(super) fn context_bar(&self) -> Element<'_, Message> {
        let compact = self.is_compact();
        let php_separator: Element<Message> = if compact {
            Space::with_width(0).into()
        } else {
            context_separator()
        };

        container(
            row![
                dry_run_badge(),
                if dry_run::active() {
                    context_separator()
                } else {
                    Space::with_width(0).into()
                },
                service_context_item(
                    tr(keys::APACHE),
                    self.dashboard.apache_running,
                    self.dashboard.apache_uptime.as_deref(),
                ),
                context_separator(),
                service_context_item(
                    "MySQL",
                    self.dashboard.mysql_running,
                    self.dashboard.mysql_uptime.as_deref()
                ),
                php_separator,
                php_context_item(
                    self.dashboard
                        .active_php_version
                        .as_deref()
                        .unwrap_or(tr(keys::NOT_AVAILABLE_SHORT)),
                ),
                Space::with_width(Length::Fill),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 18]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

fn service_context_item<'a>(
    label: &'a str,
    running: bool,
    uptime: Option<&'a str>,
) -> Element<'a, Message> {
    let color = if running {
        theme::color(theme_keys::GREEN)
    } else {
        theme::color(theme_keys::RED)
    };
    let value = if running {
        uptime.unwrap_or("running")
    } else {
        "stopped"
    };

    row![
        ui::status_dot(color),
        Space::with_width(7),
        text(label)
            .size(crate::core::app_config::text_metrics().tiny)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_width(5),
        text(value)
            .size(crate::core::app_config::text_metrics().caption)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn php_context_item<'a>(version: &'a str) -> Element<'a, Message> {
    row![
        ui::status_dot(theme::color(theme_keys::PURPLE)),
        Space::with_width(7),
        text(tr(keys::PHP))
            .size(crate::core::app_config::text_metrics().tiny)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_width(5),
        text(version)
            .size(crate::core::app_config::text_metrics().caption)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn dry_run_badge<'a>() -> Element<'a, Message> {
    if !dry_run::active() {
        return Space::with_width(0).into();
    }

    ui::small_badge("DRY RUN", ui::BadgeTone::Warning)
}

fn context_separator<'a>() -> Element<'a, Message> {
    container(Space::with_width(1))
        .width(1)
        .height(18)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BORDER_SUBTLE).into()),
            ..Default::default()
        })
        .into()
}
