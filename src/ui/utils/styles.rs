use crate::core::theme::{self, theme_map as theme_keys};
use iced::{Background, Border};

pub fn text_input_style(
    _: &iced::Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    let is_focused = matches!(status, iced::widget::text_input::Status::Focused);
    let is_hovered = matches!(status, iced::widget::text_input::Status::Hovered);
    let is_disabled = matches!(status, iced::widget::text_input::Status::Disabled);

    iced::widget::text_input::Style {
        background: Background::Color(if is_focused {
            theme::color(theme_keys::BG_CARD)
        } else if is_hovered {
            theme::color(theme_keys::BG_HOVER)
        } else {
            theme::color(theme_keys::BG_SURFACE)
        }),
        border: Border {
            color: if is_focused {
                theme::color(theme_keys::TEAL_BORDER)
            } else if is_hovered {
                theme::color(theme_keys::BORDER_MED)
            } else {
                theme::color(theme_keys::BORDER_SUBTLE)
            },
            width: if is_focused { 1.5 } else { 1.0 },
            radius: 8.0.into(),
        },
        icon: if is_focused {
            theme::color(theme_keys::TEAL)
        } else {
            theme::color(theme_keys::TEXT_MUTED)
        },
        placeholder: theme::color(theme_keys::TEXT_MUTED),
        value: if is_disabled {
            theme::color(theme_keys::TEXT_MUTED)
        } else {
            theme::color(theme_keys::TEXT_PRIMARY)
        },
        selection: theme::color(theme_keys::TEAL),
    }
}

pub fn pick_list_style(
    _: &iced::Theme,
    status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    let is_open = matches!(status, iced::widget::pick_list::Status::Opened);
    let is_hovered = matches!(status, iced::widget::pick_list::Status::Hovered);
    iced::widget::pick_list::Style {
        text_color: theme::color(theme_keys::TEXT_PRIMARY),
        placeholder_color: theme::color(theme_keys::TEXT_MUTED),
        handle_color: if is_open || is_hovered {
            theme::color(theme_keys::TEAL)
        } else {
            theme::color(theme_keys::TEXT_SECONDARY)
        },
        background: Background::Color(if is_open {
            theme::color(theme_keys::BG_HOVER)
        } else if is_hovered {
            theme::color(theme_keys::BG_CARD)
        } else {
            theme::color(theme_keys::BG_SURFACE)
        }),
        border: Border {
            color: if is_open || is_hovered {
                theme::color(theme_keys::TEAL_BORDER)
            } else {
                theme::color(theme_keys::BORDER_SUBTLE)
            },
            width: if is_open { 1.5 } else { 1.0 },
            radius: 8.0.into(),
        },
    }
}

pub fn pick_list_menu_style(_: &iced::Theme) -> iced::overlay::menu::Style {
    iced::overlay::menu::Style {
        background: Background::Color(theme::color(theme_keys::BG_CARD)),
        border: Border {
            color: theme::color(theme_keys::TEAL_BORDER),
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color: theme::color(theme_keys::TEXT_SECONDARY),
        selected_text_color: theme::color(theme_keys::TEXT_ON_ACCENT),
        selected_background: Background::Color(theme::color(theme_keys::TEAL)),
    }
}

pub fn checkbox_style(
    _: &iced::Theme,
    status: iced::widget::checkbox::Status,
) -> iced::widget::checkbox::Style {
    let is_checked = match status {
        iced::widget::checkbox::Status::Active { is_checked }
        | iced::widget::checkbox::Status::Hovered { is_checked }
        | iced::widget::checkbox::Status::Disabled { is_checked } => is_checked,
    };
    let is_hovered = matches!(status, iced::widget::checkbox::Status::Hovered { .. });
    let is_disabled = matches!(status, iced::widget::checkbox::Status::Disabled { .. });

    iced::widget::checkbox::Style {
        background: Background::Color(if is_checked {
            theme::color(theme_keys::TEAL)
        } else if is_hovered {
            theme::color(theme_keys::BG_HOVER)
        } else {
            theme::color(theme_keys::BG_SURFACE)
        }),
        icon_color: theme::color(theme_keys::TEXT_ON_ACCENT),
        border: Border {
            color: if is_checked || is_hovered {
                theme::color(theme_keys::TEAL_BORDER)
            } else {
                theme::color(theme_keys::BORDER_MED)
            },
            width: if is_hovered { 1.5 } else { 1.0 },
            radius: 4.0.into(),
        },
        text_color: Some(if is_disabled {
            theme::color(theme_keys::TEXT_MUTED)
        } else {
            theme::color(theme_keys::TEXT_SECONDARY)
        }),
    }
}
