use crate::core::app_config;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::ui::icons::{self, Icon};
use iced::widget::{Space, button, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding, Shadow, Vector};

pub fn primary_icon_button<'a, Message>(
    icon: Icon,
    label: &'a str,
    msg: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    primary_button_content(
        icon_label(icon, label, theme::color(theme_keys::TEXT_ON_ACCENT)),
        Some(msg),
        Padding::from([8, 15]),
        Length::Fixed(app_config::control_metrics().button_height),
    )
}

pub fn primary_text_button<'a, Message>(label: &'a str, msg: Message) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    primary_text_button_maybe(label, Some(msg))
}

pub fn primary_text_button_maybe<'a, Message>(
    label: &'a str,
    msg: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    primary_button_content(
        text(label)
            .size(app_config::text_metrics().body)
            .color(theme::color(theme_keys::TEXT_ON_ACCENT))
            .into(),
        msg,
        Padding::from([10, 24]),
        Length::Fixed(app_config::control_metrics().button_height),
    )
}

fn primary_button_content<'a, Message>(
    content: Element<'a, Message>,
    msg: Option<Message>,
    padding: Padding,
    height: Length,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let btn = button(content)
        .padding(padding)
        .height(height)
        .style(|_, status| match status {
            iced::widget::button::Status::Disabled => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::BG_HOVER).into()),
                text_color: theme::color(theme_keys::TEXT_MUTED),
                border: Border {
                    color: theme::color(theme_keys::BORDER_SUBTLE),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            },
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                    text_color: theme::color(theme_keys::TEXT_ON_ACCENT),
                    border: Border {
                        color: theme::color(theme_keys::TEAL_BORDER),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: soft_shadow(0.18, 8.0),
                }
            }
            _ => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL).into()),
                text_color: theme::color(theme_keys::TEXT_ON_ACCENT),
                border: Border {
                    color: theme::color(theme_keys::TEAL_BORDER),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: soft_shadow(0.12, 6.0),
            },
        });

    if let Some(msg) = msg {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

pub fn secondary_icon_button<'a, Message>(
    icon: Icon,
    label: &'a str,
    msg: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    secondary_button_content(
        icon_label(icon, label, theme::color(theme_keys::TEXT_SECONDARY)),
        msg,
        Padding::from([8, 15]),
        Length::Fixed(app_config::control_metrics().button_height),
    )
}

pub fn ghost_text_button<'a, Message>(label: &'a str, msg: Message) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    ghost_text_button_maybe(label, Some(msg))
}

pub fn ghost_text_button_maybe<'a, Message>(
    label: &'a str,
    msg: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let btn = button(
        text(label)
            .size(app_config::text_metrics().body)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
    )
    .padding(Padding::from([10, 18]))
    .height(Length::Fixed(app_config::control_metrics().button_height))
    .style(ghost_button_style());

    if let Some(msg) = msg {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

fn secondary_button_content<'a, Message>(
    content: Element<'a, Message>,
    msg: Message,
    padding: Padding,
    height: Length,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    button(content)
        .on_press(msg)
        .padding(padding)
        .height(height)
        .style(ghost_button_style())
        .into()
}

fn icon_label<'a, Message: 'a>(icon: Icon, label: &'a str, color: Color) -> Element<'a, Message> {
    row![
        icon_box(icon, 13.0, color, 15.0),
        Space::with_width(8),
        text(label)
            .size(app_config::text_metrics().body)
            .color(color),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn icon_box<'a, Message: 'a>(
    icon: Icon,
    size: f32,
    color: Color,
    box_size: f32,
) -> Element<'a, Message> {
    icons::solid_box(icon, size, color, box_size)
}

pub fn action_button<'a, Message>(
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    border: Color,
    on_press: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    action_button_with_style(
        label,
        on_press,
        ActionButtonStyle {
            color,
            bg,
            bg_hover,
            border,
            padding: [7, 14],
            radius: 8.0,
            height: 36.0,
            width: 0.0,
        },
    )
}

pub fn action_icon_button<'a, Message>(
    icon: Icon,
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    border: Color,
    on_press: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    action_button_with_content(
        row![
            icon_box(icon, 12.0, color, 14.0),
            Space::with_width(7),
            text(label)
                .size(app_config::text_metrics().caption)
                .color(color),
        ]
        .align_y(Alignment::Center)
        .into(),
        on_press,
        ActionButtonStyle {
            color,
            bg,
            bg_hover,
            border,
            padding: [7, 13],
            radius: 7.0,
            height: 36.0,
            width: 90.0,
        },
    )
}

pub fn compact_action_button<'a, Message>(
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    border: Color,
    on_press: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    action_button_with_style(
        label,
        on_press,
        ActionButtonStyle {
            color,
            bg,
            bg_hover,
            border,
            padding: [6, 12],
            radius: 8.0,
            height: 34.0,
            width: 0.0,
        },
    )
}

struct ActionButtonStyle {
    color: Color,
    bg: Color,
    bg_hover: Color,
    border: Color,
    padding: [u16; 2],
    radius: f32,
    height: f32,
    width: f32,
}

fn action_button_with_style<'a, Message>(
    label: &'a str,
    on_press: Option<Message>,
    style: ActionButtonStyle,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    action_button_with_content(
        text(label)
            .size(app_config::text_metrics().caption)
            .color(style.color)
            .into(),
        on_press,
        style,
    )
}

fn action_button_with_content<'a, Message>(
    content: Element<'a, Message>,
    on_press: Option<Message>,
    style: ActionButtonStyle,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let btn = button(content)
        .padding(Padding::from(style.padding))
        .height(Length::Fixed(style.height))
        .width(if style.width > 0.0 {
            Length::Fixed(style.width)
        } else {
            Length::Shrink
        })
        .style(move |_, status| match status {
            iced::widget::button::Status::Disabled => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::BG_SURFACE).into()),
                text_color: theme::color(theme_keys::TEXT_MUTED),
                border: Border {
                    color: theme::color(theme_keys::BORDER_SUBTLE),
                    width: 1.0,
                    radius: style.radius.into(),
                },
                ..Default::default()
            },
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(style.bg_hover.into()),
                    text_color: style.color,
                    border: Border {
                        color: style.border,
                        width: 1.0,
                        radius: style.radius.into(),
                    },
                    shadow: soft_shadow(0.1, 5.0),
                }
            }
            _ => iced::widget::button::Style {
                background: Some(style.bg.into()),
                text_color: style.color,
                border: Border {
                    color: style.border,
                    width: 1.0,
                    radius: style.radius.into(),
                },
                shadow: soft_shadow(0.06, 3.0),
            },
        });

    if let Some(msg) = on_press {
        btn.on_press(msg).into()
    } else {
        btn.into()
    }
}

pub fn ghost_button_style()
-> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Disabled => iced::widget::button::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            text_color: theme::color(theme_keys::TEXT_MUTED),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(theme::color(theme_keys::BG_HOVER).into()),
                text_color: theme::color(theme_keys::TEXT_PRIMARY),
                border: Border {
                    color: theme::color(theme_keys::TEAL_BORDER),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: soft_shadow(0.08, 4.0),
            }
        }
        _ => iced::widget::button::Style {
            background: Some(theme::color(theme_keys::BG_CARD).into()),
            text_color: theme::color(theme_keys::TEXT_SECONDARY),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
    }
}

fn soft_shadow(alpha: f32, blur_radius: f32) -> Shadow {
    Shadow {
        color: Color {
            a: alpha,
            ..theme::color(theme_keys::SHADOW_HEAVY)
        },
        offset: Vector::new(0.0, 1.0),
        blur_radius,
    }
}
