use crate::core::first_run_install::FirstRunInstallOptions;
use crate::core::theme::*;
use crate::messages::Message;
use iced::widget::{Space, button, checkbox, column, container, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

fn sentinel_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("devpanel")
        .join("first_run_done")
}

pub fn is_first_run() -> bool {
    !sentinel_path().exists()
}

pub fn mark_done() {
    let path = sentinel_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, "1");
}

#[derive(Debug, Clone, PartialEq)]
pub enum FirstRunState {
    Visible,
    Hidden,
}

impl Default for FirstRunState {
    fn default() -> Self {
        if is_first_run() {
            FirstRunState::Visible
        } else {
            FirstRunState::Hidden
        }
    }
}

struct Item {
    package: &'static str,
    purpose: &'static str,
    core: bool,
}

const INSTALL_ITEMS: &[Item] = &[
    Item {
        package: "apache2",
        purpose: "HTTP server",
        core: true,
    },
    Item {
        package: "libapache2-mod-php",
        purpose: "PHP module for Apache",
        core: true,
    },
    Item {
        package: "php8.2",
        purpose: "PHP 8.2 CLI + common extensions",
        core: true,
    },
    Item {
        package: "php8.2-cli",
        purpose: "PHP command-line interface",
        core: true,
    },
    Item {
        package: "php8.2-common",
        purpose: "Shared PHP extensions",
        core: true,
    },
    Item {
        package: "php8.2-mysql",
        purpose: "MySQL / MariaDB driver",
        core: false,
    },
    Item {
        package: "php8.2-xml",
        purpose: "XML / DOM / SimpleXML support",
        core: false,
    },
    Item {
        package: "php8.2-mbstring",
        purpose: "Multibyte string functions",
        core: false,
    },
    Item {
        package: "mysql-server",
        purpose: "MySQL / MariaDB database server",
        core: false,
    },
];

pub fn view<'a>(
    options: FirstRunInstallOptions,
    installing: bool,
    log_lines: &'a [String],
) -> Element<'a, Message> {
    use crate::messages::{FirstRunMessage, Message};

    let overlay_bg = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.78,
    };

    let header = column![
        row![
            container(text("NEW").size(9).color(TEAL))
                .padding(Padding::from([3, 8]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(
                        Color {
                            r: 0.040,
                            g: 0.160,
                            b: 0.150,
                            a: 1.0
                        }
                        .into()
                    ),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(10),
            text("Welcome to DevPanel").size(20).color(TEXT_PRIMARY),
        ]
        .align_y(Alignment::Center),
        Space::with_height(8),
        text("To get started, the following packages will be installed on your system.")
            .size(13)
            .color(TEXT_SECONDARY),
        Space::with_height(4),
        text(
            "This requires your sudo password. You can exit now and install manually if you prefer."
        )
        .size(12)
        .color(TEXT_MUTED),
    ]
    .spacing(0);

    let divider = container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_SUBTLE.into()),
            ..Default::default()
        });

    let pkg_rows: Vec<Element<Message>> = INSTALL_ITEMS
        .iter()
        .map(|item| {
            let installed = package_installed(item.package);
            let skipped = (!options.install_mysql && item.package == "mysql-server")
                || (!options.install_php_extras
                    && item.package.starts_with("php8.2-")
                    && !item.core);
            let (dot_color, status) = if installed {
                (GREEN, "installed")
            } else if skipped {
                (TEXT_MUTED, "skipped")
            } else {
                (YELLOW, "will install")
            };
            container(
                row![
                    container(Space::with_width(6)).width(6).height(6).style(
                        move |_: &iced::Theme| container::Style {
                            background: Some(dot_color.into()),
                            border: Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    ),
                    Space::with_width(10),
                    text(item.package).size(12).color(TEXT_PRIMARY).width(220),
                    text(item.purpose)
                        .size(11)
                        .color(TEXT_MUTED)
                        .width(Length::Fill),
                    text(status).size(10).color(dot_color),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([7, 10]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(BG_SURFACE.into()),
                border: Border {
                    color: BORDER_SUBTLE,
                    width: 1.0,
                    radius: 7.0.into(),
                },
                ..Default::default()
            })
            .into()
        })
        .collect();

    let php_note = container(
        row![
            text("i").size(10).color(BLUE),
            Space::with_width(8),
            column![
                text("PHP mod_phpX.Y modules will also be enabled for Apache.")
                    .size(11)
                    .color(TEXT_MUTED),
                Space::with_height(2),
                text("This allows you to pin individual VirtualHosts to a specific PHP version.")
                    .size(11)
                    .color(TEXT_MUTED),
            ]
            .spacing(0),
        ]
        .align_y(Alignment::Start),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(
            Color {
                r: 0.047,
                g: 0.090,
                b: 0.157,
                a: 1.0,
            }
            .into(),
        ),
        border: Border {
            color: Color {
                r: 0.080,
                g: 0.140,
                b: 0.260,
                a: 1.0,
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    let continue_btn = button(
        text(if installing {
            "Installing..."
        } else {
            "Continue — Install & Setup"
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
                background: Some(
                    Color {
                        r: 0.060,
                        g: 0.185,
                        b: 0.175,
                        a: 1.0,
                    }
                    .into(),
                ),
                text_color: TEAL,
                border: Border {
                    color: TEAL,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(TEAL.into()),
            text_color: Color {
                r: 0.05,
                g: 0.05,
                b: 0.06,
                a: 1.0,
            },
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    });

    let exit_btn = button(text("Exit").size(13))
        .on_press_maybe(if installing {
            None
        } else {
            Some(Message::FirstRun(FirstRunMessage::Exit))
        })
        .padding(Padding::from([11, 20]))
        .style(|_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(BG_HOVER.into()),
                    text_color: TEXT_PRIMARY,
                    border: Border {
                        color: BORDER_MED,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            }
            _ => iced::widget::button::Style {
                background: Some(BG_CARD.into()),
                text_color: TEXT_SECONDARY,
                border: Border {
                    color: BORDER_SUBTLE,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            },
        });

    let skips = container(
        column![
            checkbox("Install MySQL server", options.install_mysql)
                .on_toggle(|v| Message::FirstRun(FirstRunMessage::ToggleMysql(v)))
                .size(12),
            Space::with_height(6),
            checkbox(
                "Install optional PHP extension packages",
                options.install_php_extras
            )
            .on_toggle(|v| Message::FirstRun(FirstRunMessage::TogglePhpExtras(v)))
            .size(12),
        ]
        .spacing(0),
    )
    .padding(Padding::from([10, 12]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(BG_SURFACE.into()),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    let log_panel: Element<Message> = if installing || !log_lines.is_empty() {
        let rows: Vec<Element<Message>> = log_lines
            .iter()
            .rev()
            .take(10)
            .rev()
            .map(|line| text(line.as_str()).size(10).color(TEXT_MUTED).into())
            .collect();
        container(
            column![
                text("Setup log").size(11).color(TEXT_MUTED),
                Space::with_height(6),
                scrollable(column(rows).spacing(3)).height(110),
            ]
            .spacing(0),
        )
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_BASE.into()),
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_height(0).into()
    };

    let card = container(
        column![
            header,
            Space::with_height(20),
            divider,
            Space::with_height(16),
            text("Packages to install").size(11).color(TEXT_MUTED),
            Space::with_height(8),
            column(pkg_rows).spacing(5),
            Space::with_height(14),
            skips,
            Space::with_height(14),
            php_note,
            Space::with_height(if installing || !log_lines.is_empty() {
                14
            } else {
                0
            }),
            log_panel,
            Space::with_height(24),
            row![continue_btn, Space::with_width(10), exit_btn].align_y(Alignment::Center),
        ]
        .spacing(0)
        .padding(Padding::from([32, 32])),
    )
    .width(560)
    .style(|_: &iced::Theme| container::Style {
        background: Some(BG_ELEVATED.into()),
        border: Border {
            color: BORDER_MED,
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: Color {
                a: 0.7,
                ..Color::BLACK
            },
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
    .style(move |_: &iced::Theme| container::Style {
        background: Some(overlay_bg.into()),
        ..Default::default()
    })
    .into()
}

fn package_installed(package: &str) -> bool {
    std::process::Command::new("dpkg")
        .args(["-s", package])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
