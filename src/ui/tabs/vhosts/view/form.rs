use super::{php_options, php_to_selection, selection_to_php};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::domain::vhosts::FormMode;
use crate::lang::{lang_map::vhosts as keys, text as tr};
use crate::messages::{Message, VHostsMessage};
use crate::ui::tabs::vhosts::VHostsTab;
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{Space, checkbox, column, container, row, text, text_input};
use iced::{Alignment, Border, Element, Length, Padding};

pub(super) fn inline_edit_widget<'a>(tab: &'a VHostsTab, _idx: usize) -> Element<'a, Message> {
    vhost_form_panel(
        tab,
        tr(keys::EDITING_VHOST),
        tr(keys::SAVE_CHANGES),
        Message::VHosts(VHostsMessage::SaveEdit),
        theme::color(theme_keys::BLUE_BORDER),
    )
}

pub(super) fn add_form_widget(tab: &VHostsTab) -> Element<'_, Message> {
    let is_edit = matches!(tab.form.mode, FormMode::Edit(_));
    let title = if is_edit {
        tr(keys::EDIT_VHOST)
    } else {
        tr(keys::ADD_VHOST_TITLE)
    };
    let save_msg = if is_edit {
        Message::VHosts(VHostsMessage::SaveEdit)
    } else {
        Message::VHosts(VHostsMessage::Create)
    };
    let save_lbl = if is_edit {
        tr(keys::SAVE_CHANGES)
    } else {
        tr(keys::CREATE_VHOST)
    };

    vhost_form_panel(
        tab,
        title,
        save_lbl,
        save_msg,
        theme::color(theme_keys::BORDER_SUBTLE),
    )
}

fn vhost_form_panel<'a>(
    tab: &'a VHostsTab,
    title: &'a str,
    save_label: &'a str,
    save_msg: Message,
    border_color: iced::Color,
) -> Element<'a, Message> {
    let can_save =
        !tab.form.server_name.trim().is_empty() && !tab.form.document_root.trim().is_empty();

    let php_picker =
        php_version_picker(&tab.available_php_versions, &tab.form.php_version, |sel| {
            Message::VHosts(VHostsMessage::FormPhpVersionChanged(selection_to_php(&sel)))
        });

    container(
        column![
            row![
                text(title)
                    .size(crate::core::app_config::text_metrics().body)
                    .color(theme::color(theme_keys::TEXT_SECONDARY)),
                Space::with_width(Length::Fill),
                ui::compact_action_button(
                    tr(keys::CANCEL),
                    theme::color(theme_keys::TEXT_MUTED),
                    theme::color(theme_keys::BG_SURFACE),
                    theme::color(theme_keys::BG_HOVER),
                    theme::color(theme_keys::BORDER_SUBTLE),
                    Some(Message::VHosts(VHostsMessage::HideForm))
                ),
                ui::compact_action_button(
                    save_label,
                    theme::color(theme_keys::GREEN),
                    theme::color(theme_keys::GREEN_BG),
                    theme::color(theme_keys::GREEN_HOVER),
                    theme::color(theme_keys::GREEN_DIM),
                    if can_save { Some(save_msg) } else { None }
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            Space::with_height(14),
            ui::thin_line(),
            Space::with_height(14),
            row![
                form_input(
                    tr(keys::SERVER_NAME),
                    tr(keys::SERVER_NAME_PLACEHOLDER),
                    &tab.form.server_name,
                    |v| Message::VHosts(VHostsMessage::FormServerNameChanged(v))
                )
                .width(Length::FillPortion(1)),
                Space::with_width(14),
                form_input(
                    tr(keys::DOCUMENT_ROOT),
                    tr(keys::DOCUMENT_ROOT_PLACEHOLDER),
                    &tab.form.document_root,
                    |v| Message::VHosts(VHostsMessage::FormDocRootChanged(v))
                )
                .width(Length::FillPortion(1)),
            ]
            .align_y(Alignment::Start),
            Space::with_height(12),
            row![
                column![
                    text(tr(keys::PHP_VERSION_OPTIONAL))
                        .size(crate::core::app_config::text_metrics().caption)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_height(5),
                    php_picker,
                ]
                .spacing(0)
                .width(Length::FillPortion(1)),
                Space::with_width(14),
                column![
                    text(tr(keys::HTTPS))
                        .size(crate::core::app_config::text_metrics().caption)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                    Space::with_height(9),
                    row![
                        checkbox("", tab.form.https_enabled)
                            .on_toggle(|v| Message::VHosts(VHostsMessage::FormHttpsChanged(v)))
                            .size(crate::core::app_config::control_metrics().checkbox_size),
                        Space::with_width(8),
                        text(tr(keys::ENABLE_HTTPS_MKCERT))
                            .size(crate::core::app_config::text_metrics().caption)
                            .color(theme::color(theme_keys::TEXT_SECONDARY)),
                    ]
                    .align_y(Alignment::Center),
                ]
                .spacing(0)
                .width(Length::FillPortion(1)),
            ]
            .align_y(Alignment::Start),
        ]
        .spacing(0)
        .padding(Padding::from([16, 18])),
    )
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

fn form_input<'a, F>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: F,
) -> iced::widget::Column<'a, Message>
where
    F: Fn(String) -> Message + 'a,
{
    column![
        text(label)
            .size(crate::core::app_config::text_metrics().caption)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_height(5),
        text_input(placeholder, value)
            .on_input(on_input)
            .size(crate::core::app_config::text_metrics().body)
            .padding(Padding::from([8, 10]))
            .style(styles::text_input_style)
            .width(Length::Fill),
    ]
    .spacing(0)
}

fn php_version_picker<'a, F>(
    available: &'a [String],
    current: &'a Option<String>,
    on_select: F,
) -> Element<'a, Message>
where
    F: Fn(String) -> Message + 'a,
{
    let options = php_options(available);
    let selected = php_to_selection(current);

    let el = ui::dropdown_width(
        options,
        Some(selected),
        move |s: String| on_select(s),
        Length::Fixed(crate::core::app_config::control_metrics().form_dropdown_width),
    );

    if available.is_empty() {
        column![
            el,
            Space::with_height(3),
            text(tr(keys::NO_MOD_PHP))
                .size(crate::core::app_config::text_metrics().tiny)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0)
        .into()
    } else {
        el
    }
}
