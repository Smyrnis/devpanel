use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::dashboard as keys, text as tr};
use crate::messages::Message;
use crate::ui::icons::Icon;
use crate::ui::templates::prelude as ui;
use iced::widget::row;
use iced::{Alignment, Element};

pub(super) fn action_row<'a>(actions: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    row(actions).spacing(8).align_y(Alignment::Center).into()
}

pub(super) fn service_power_button<'a>(
    running: bool,
    start: Message,
    stop: Message,
) -> Element<'a, Message> {
    if running {
        ui::action_icon_button(
            Icon::Stop,
            tr(keys::STOP),
            theme::color(theme_keys::RED),
            theme::color(theme_keys::RED_BG),
            theme::color(theme_keys::RED_HOVER),
            theme::color(theme_keys::RED_BORDER),
            Some(stop),
        )
    } else {
        ui::action_icon_button(
            Icon::Play,
            tr(keys::START),
            theme::color(theme_keys::GREEN),
            theme::color(theme_keys::GREEN_BG),
            theme::color(theme_keys::GREEN_HOVER),
            theme::color(theme_keys::GREEN_DIM),
            Some(start),
        )
    }
}

pub(super) fn restart_button<'a>(message: Message) -> Element<'a, Message> {
    ui::action_icon_button(
        Icon::Refresh,
        tr(keys::RESTART),
        theme::color(theme_keys::ORANGE),
        theme::color(theme_keys::YELLOW_BG),
        theme::color(theme_keys::BG_HOVER),
        theme::color(theme_keys::YELLOW_BORDER),
        Some(message),
    )
}

pub(super) fn service_status(running: bool, uptime: Option<&str>) -> String {
    if running {
        uptime
            .map(|value| format!("{} {}", tr(keys::UPTIME_PREFIX), value))
            .unwrap_or_else(|| tr(keys::RUNNING).to_string())
    } else {
        tr(keys::STOPPED).to_string()
    }
}

pub(super) fn service_tone(running: bool) -> ui::BadgeTone {
    if running {
        ui::BadgeTone::Success
    } else {
        ui::BadgeTone::Danger
    }
}

pub(super) fn running_text(running: bool) -> &'static str {
    if running {
        tr(keys::RUNNING)
    } else {
        tr(keys::STOPPED)
    }
}
