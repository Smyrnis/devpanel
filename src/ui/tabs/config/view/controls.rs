use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::config as keys, text as tr};
use crate::messages::Message;
use crate::ui::icons::Icon;
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, checkbox, column, container, row, text};
use iced::{Alignment, Element, Length, Padding};

pub(super) fn section_card<'a>(
    _title: &'a str,
    _description: &'a str,
    _icon: Icon,
    content: iced::widget::Column<'a, Message>,
) -> Element<'a, Message> {
    ui::expanded_panel(vec![container(content).style(ui::surface_style()).into()])
}

pub(super) fn setting_row<'a>(
    label: &'a str,
    hint: &'a str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    row![
        setting_text(label, hint).width(Length::FillPortion(1)),
        Space::with_width(22),
        container(control).width(Length::FillPortion(1)),
    ]
    .align_y(Alignment::Center)
    .padding(Padding::from([18, 20]))
    .into()
}

pub(super) fn setting_toggle_row<'a, F>(
    label: &'a str,
    hint: &'a str,
    value: bool,
    on_toggle: F,
) -> Element<'a, Message>
where
    F: Fn(bool) -> Message + 'a,
{
    let state_text = if value {
        tr(keys::ENABLED)
    } else {
        tr(keys::DISABLED)
    };
    setting_row(
        label,
        hint,
        row![
            checkbox("", value)
                .on_toggle(on_toggle)
                .size(crate::core::app_config::control_metrics().large_checkbox_size),
            Space::with_width(10),
            text(state_text)
                .size(crate::core::app_config::text_metrics().caption)
                .color(if value {
                    theme::color(theme_keys::GREEN)
                } else {
                    theme::color(theme_keys::TEXT_MUTED)
                }),
        ]
        .align_y(Alignment::Center)
        .into(),
    )
}

fn setting_text<'a>(label: &'a str, hint: &'a str) -> iced::widget::Column<'a, Message> {
    column![
        text(label)
            .size(crate::core::app_config::text_metrics().section_title)
            .color(theme::color(theme_keys::TEXT_PRIMARY)),
        Space::with_height(4),
        text(hint)
            .size(crate::core::app_config::text_metrics().caption)
            .color(theme::color(theme_keys::TEXT_MUTED)),
    ]
    .spacing(0)
}
