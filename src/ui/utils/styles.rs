use crate::core::theme::{self, theme_map as theme_keys};
use iced::Border;

pub fn text_input_style(
    _: &iced::Theme,
    _: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: theme::color(theme_keys::BG_SURFACE).into(),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: theme::color(theme_keys::TEXT_MUTED),
        placeholder: theme::color(theme_keys::TEXT_MUTED),
        value: theme::color(theme_keys::TEXT_PRIMARY),
        selection: theme::color(theme_keys::TEAL),
    }
}

pub fn pick_list_style(
    _: &iced::Theme,
    status: iced::widget::pick_list::Status,
) -> iced::widget::pick_list::Style {
    let is_open = matches!(status, iced::widget::pick_list::Status::Opened);
    iced::widget::pick_list::Style {
        text_color: theme::color(theme_keys::TEXT_PRIMARY),
        placeholder_color: theme::color(theme_keys::TEXT_MUTED),
        handle_color: if is_open {
            theme::color(theme_keys::TEAL)
        } else {
            theme::color(theme_keys::TEXT_MUTED)
        },
        background: iced::Background::Color(if is_open {
            theme::color(theme_keys::BG_HOVER)
        } else {
            theme::color(theme_keys::BG_SURFACE)
        }),
        border: Border {
            color: if is_open {
                theme::color(theme_keys::TEAL_BORDER)
            } else {
                theme::color(theme_keys::BORDER_SUBTLE)
            },
            width: 1.0,
            radius: 8.0.into(),
        },
    }
}
