use super::packages::{INSTALL_ITEMS, package_installed};
use crate::core::first_run_install::FirstRunInstallOptions;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::install as keys, text as tr};
use crate::messages::Message;
use iced::widget::{Space, button, checkbox, column, container, row, scrollable, text};
use iced::{Alignment, Border, Element, Length, Padding};

pub fn view<'a>(
    options: FirstRunInstallOptions,
    installing: bool,
    log_lines: &'a [String],
) -> Element<'a, Message> {
    let header = column![
        row![
            container(
                text(tr(keys::BADGE_NEW))
                    .size(9)
                    .color(theme::color(theme_keys::TEAL))
            )
            .padding(Padding::from([3, 8]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::TEAL_BG).into()),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(10),
            text(tr(keys::WELCOME_TITLE))
                .size(20)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
        ]
        .align_y(Alignment::Center),
        Space::with_height(8),
        text(tr(keys::WELCOME_BODY))
            .size(13)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
        Space::with_height(4),
        text(tr(keys::SUDO_NOTE))
            .size(12)
            .color(theme::color(theme_keys::TEXT_MUTED)),
    ]
    .spacing(0);

    let pkg_rows: Vec<Element<Message>> = INSTALL_ITEMS
        .iter()
        .map(|item| {
            let installed = package_installed(item.package);
            let skipped = (!options.install_mysql && item.package == "mysql-server")
                || (!options.install_php_extras
                    && item.package.starts_with("php8.2-")
                    && !item.core);
            let (dot_color, status) = if installed {
                (theme::color(theme_keys::GREEN), tr(keys::STATUS_INSTALLED))
            } else if skipped {
                (
                    theme::color(theme_keys::TEXT_MUTED),
                    tr(keys::STATUS_SKIPPED),
                )
            } else {
                (
                    theme::color(theme_keys::YELLOW),
                    tr(keys::STATUS_WILL_INSTALL),
                )
            };
            package_row(item.package, tr(item.purpose_key), status, dot_color)
        })
        .collect();

    let log_panel: Element<Message> = if installing || !log_lines.is_empty() {
        setup_log_panel(log_lines)
    } else {
        Space::with_height(0).into()
    };

    let card = container(
        column![
            header,
            Space::with_height(20),
            divider(),
            Space::with_height(16),
            text(tr(keys::PACKAGES_TO_INSTALL))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(8),
            column(pkg_rows).spacing(5),
            Space::with_height(14),
            skip_options(options),
            Space::with_height(14),
            php_note(),
            Space::with_height(if installing || !log_lines.is_empty() {
                14
            } else {
                0
            }),
            log_panel,
            Space::with_height(24),
            row![
                continue_button(installing),
                Space::with_width(10),
                exit_button(installing)
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(0)
        .padding(Padding::from([32, 32])),
    )
    .width(560)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_ELEVATED).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_MED),
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: theme::color(theme_keys::SHADOW_HEAVY),
            offset: iced::Vector::new(0.0, 16.0),
            blur_radius: 56.0,
        },
        ..Default::default()
    });

    container(
        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::OVERLAY_STRONG).into()),
        ..Default::default()
    })
    .into()
}

fn package_row<'a>(
    package: &'static str,
    purpose: &'static str,
    status: &'static str,
    dot_color: iced::Color,
) -> Element<'a, Message> {
    container(
        row![
            container(Space::with_width(6))
                .width(6)
                .height(6)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(dot_color.into()),
                    border: Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(10),
            text(package)
                .size(12)
                .color(theme::color(theme_keys::TEXT_PRIMARY))
                .width(220),
            text(purpose)
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED))
                .width(Length::Fill),
            text(status).size(10).color(dot_color),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([7, 10]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 7.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn php_note<'a>() -> Element<'a, Message> {
    container(
        row![
            text("i").size(10).color(theme::color(theme_keys::BLUE)),
            Space::with_width(8),
            column![
                text(tr(keys::PHP_APACHE_NOTE))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(2),
                text(tr(keys::PHP_VHOST_NOTE))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0),
        ]
        .align_y(Alignment::Start),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BLUE_DIM).into()),
        border: Border {
            color: theme::color(theme_keys::BLUE_BORDER),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn skip_options<'a>(options: FirstRunInstallOptions) -> Element<'a, Message> {
    use crate::messages::FirstRunMessage;
    container(
        column![
            checkbox(tr(keys::INSTALL_MYSQL), options.install_mysql)
                .on_toggle(|v| Message::FirstRun(FirstRunMessage::ToggleMysql(v)))
                .size(12),
            Space::with_height(6),
            checkbox(tr(keys::INSTALL_PHP_EXTRAS), options.install_php_extras)
                .on_toggle(|v| Message::FirstRun(FirstRunMessage::TogglePhpExtras(v)))
                .size(12),
        ]
        .spacing(0),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn setup_log_panel<'a>(log_lines: &'a [String]) -> Element<'a, Message> {
    let rows: Vec<Element<Message>> = log_lines
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|line| {
            text(line.as_str())
                .size(10)
                .color(theme::color(theme_keys::TEXT_MUTED))
                .into()
        })
        .collect();
    container(
        column![
            text(tr(keys::SETUP_LOG))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(6),
            scrollable(column(rows).spacing(3)).height(110),
        ]
        .spacing(0),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_BASE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn continue_button<'a>(installing: bool) -> Element<'a, Message> {
    use crate::messages::FirstRunMessage;
    button(
        text(if installing {
            tr(keys::INSTALLING)
        } else {
            tr(keys::CONTINUE_INSTALL)
        })
        .size(13),
    )
    .on_press_maybe(if installing {
        None
    } else {
        Some(Message::FirstRun(FirstRunMessage::Continue))
    })
    .padding(Padding::from([11, 28]))
    .style(|_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                text_color: theme::color(theme_keys::TEAL),
                border: Border {
                    color: theme::color(theme_keys::TEAL),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(theme::color(theme_keys::TEAL).into()),
            text_color: theme::color(theme_keys::TEXT_ON_ACCENT),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    })
    .into()
}

fn exit_button<'a>(installing: bool) -> Element<'a, Message> {
    use crate::messages::FirstRunMessage;
    button(text(tr(keys::EXIT)).size(13))
        .on_press_maybe(if installing {
            None
        } else {
            Some(Message::FirstRun(FirstRunMessage::Exit))
        })
        .padding(Padding::from([11, 20]))
        .style(ghost_btn_style())
        .into()
}

fn divider<'a>() -> Element<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BORDER_SUBTLE).into()),
            ..Default::default()
        })
        .into()
}

fn ghost_btn_style()
-> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(theme::color(theme_keys::BG_HOVER).into()),
                text_color: theme::color(theme_keys::TEXT_PRIMARY),
                border: Border {
                    color: theme::color(theme_keys::BORDER_MED),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
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
