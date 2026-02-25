// src/tabs/tools.rs -- PHP Version Manager + Database CLI

use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

// ── Tinted solid colors (pre-computed, no alpha struct tricks) ────────────
// These are used for borders and backgrounds to avoid rendering glitches.

// BLUE tinted backgrounds / borders
const BLUE_BG: Color = Color {
    r: 0.050,
    g: 0.090,
    b: 0.180,
    a: 1.0,
};
const BLUE_BORDER: Color = Color {
    r: 0.080,
    g: 0.140,
    b: 0.260,
    a: 1.0,
};
const BLUE_HOVER: Color = Color {
    r: 0.070,
    g: 0.120,
    b: 0.230,
    a: 1.0,
};

// GREEN tinted
const GREEN_BG: Color = Color {
    r: 0.050,
    g: 0.160,
    b: 0.090,
    a: 1.0,
};
//const GREEN_BORDER: Color = Color { r: 0.070, g: 0.210, b: 0.110, a: 1.0 };
const GREEN_HOVER: Color = Color {
    r: 0.060,
    g: 0.185,
    b: 0.100,
    a: 1.0,
};

// RED tinted
const RED_BG: Color = Color {
    r: 0.200,
    g: 0.060,
    b: 0.055,
    a: 1.0,
};
//const RED_BORDER: Color = Color { r: 0.260, g: 0.080, b: 0.070, a: 1.0 };
const RED_HOVER: Color = Color {
    r: 0.230,
    g: 0.070,
    b: 0.063,
    a: 1.0,
};

// YELLOW tinted
const YELLOW_BG: Color = Color {
    r: 0.190,
    g: 0.160,
    b: 0.040,
    a: 1.0,
};
const YELLOW_BORDER: Color = Color {
    r: 0.240,
    g: 0.200,
    b: 0.050,
    a: 1.0,
};

// TEAL tinted
const TEAL_BG: Color = Color {
    r: 0.040,
    g: 0.160,
    b: 0.150,
    a: 1.0,
};
const TEAL_BORDER: Color = Color {
    r: 0.060,
    g: 0.210,
    b: 0.200,
    a: 1.0,
};
const TEAL_HOVER: Color = Color {
    r: 0.050,
    g: 0.185,
    b: 0.175,
    a: 1.0,
};

// PURPLE tinted
const PURPLE_BG: Color = Color {
    r: 0.140,
    g: 0.060,
    b: 0.180,
    a: 1.0,
};
const PURPLE_BORDER: Color = Color {
    r: 0.180,
    g: 0.080,
    b: 0.230,
    a: 1.0,
};
const PURPLE_HOVER: Color = Color {
    r: 0.160,
    g: 0.070,
    b: 0.205,
    a: 1.0,
};

// ── PHP version descriptor ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum PhpStatus {
    Installed,
    Available,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PhpRelease {
    pub version: String,
    pub status: PhpStatus,
    pub is_active: bool,
}

// ── State ──────────────────────────────────────────────────────────────────

pub struct ToolsTab {
    pub php_releases: Vec<PhpRelease>,
    pub scanning: bool,
    pub install_log: Vec<(bool, String)>,
    pub db_status: String,
    pub last_php_error: Option<String>,
}

impl ToolsTab {
    pub fn new() -> Self {
        Self {
            php_releases: vec![
                PhpRelease {
                    version: "7.4".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                },
                PhpRelease {
                    version: "8.0".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                },
                PhpRelease {
                    version: "8.1".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                },
                PhpRelease {
                    version: "8.2".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                },
                PhpRelease {
                    version: "8.3".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                },
                PhpRelease {
                    version: "8.4".into(),
                    status: PhpStatus::Unknown,
                    is_active: false,
                },
            ],
            scanning: false,
            install_log: Vec::new(),
            db_status: String::new(),
            last_php_error: None,
        }
    }

    pub fn apply_scan(&mut self, results: Vec<(String, PhpStatus, bool)>) {
        self.scanning = false;
        for release in &mut self.php_releases {
            if let Some((_, status, active)) =
                results.iter().find(|(v, _, _)| v == &release.version)
            {
                release.status = status.clone();
                release.is_active = *active;
            }
        }
    }

    pub fn push_log(&mut self, ok: bool, msg: String) {
        if !ok && msg.contains("failed") {
            self.last_php_error = Some(msg.clone());
        }
        self.install_log.push((ok, msg));
    }

    pub fn view(&self) -> Element<'_, Message> {
        scrollable(
            column![
                column![
                    text("Tools").size(22).color(TEXT_PRIMARY),
                    Space::with_height(4),
                    text("Install PHP versions and open database shells")
                        .size(13)
                        .color(TEXT_MUTED),
                ]
                .spacing(0),
                Space::with_height(22),
                row![self.php_panel(), Space::with_width(14), self.db_panel(),]
                    .align_y(Alignment::Start),
                Space::with_height(16),
                self.log_panel(),
                if self.last_php_error.is_some() {
                    Space::with_height(16)
                } else {
                    Space::with_height(0)
                },
                if self.last_php_error.is_some() {
                    self.error_suggestion_panel()
                } else {
                    Space::with_height(0).into()
                },
                Space::with_height(22),
            ]
            .spacing(0)
            .padding(Padding::from([22, 24])),
        )
        .into()
    }

    // ── PHP panel ──────────────────────────────────────────────────────────

    fn php_panel(&self) -> Element<'_, Message> {
        let divider = thin_line();

        let scan_label = if self.scanning { "Scanning..." } else { "Scan" };

        let header = row![
            column![
                text("PHP Versions").size(14).color(TEXT_SECONDARY),
                Space::with_height(3),
                text("Install or remove PHP via apt")
                    .size(11)
                    .color(TEXT_MUTED),
            ]
            .spacing(0)
            .width(Length::Fill),
            button(text(scan_label).size(12).color(TEAL))
                .on_press_maybe(if self.scanning {
                    None
                } else {
                    Some(Message::TOOLS_ScanPhp)
                })
                .padding(Padding::from([7, 14]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(TEAL_HOVER.into()),
                        text_color: TEAL,
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: Some(TEAL_BG.into()),
                        text_color: TEAL,
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                }),
        ]
        .align_y(Alignment::Center);

        let rows: Vec<Element<Message>> =
            self.php_releases.iter().map(|r| self.php_row(r)).collect();

        let notice = container(
            row![
                text("i").size(10).color(BLUE),
                Space::with_width(8),
                text("Requires ondrej/php PPA or equivalent. Install extensions separately.")
                    .size(11)
                    .color(TEXT_MUTED),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BLUE_BG.into()),
            border: Border {
                color: BLUE_BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });

        container(
            column![
                header,
                Space::with_height(18),
                divider,
                Space::with_height(14),
                column(rows).spacing(8),
                Space::with_height(16),
                notice,
            ]
            .spacing(0)
            .padding(Padding::from([22, 22])),
        )
        .width(Length::FillPortion(3))
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_CARD.into()),
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn php_row<'a>(&self, r: &'a PhpRelease) -> Element<'a, Message> {
        let (status_color, status_label) = match r.status {
            PhpStatus::Installed => (GREEN, "Installed"),
            PhpStatus::Available => (TEXT_MUTED, "Available"),
            PhpStatus::Unknown => (TEXT_MUTED, "Unknown"),
        };

        let active_badge: Element<Message> = if r.is_active {
            container(text("Active").size(10).color(TEAL))
                .padding(Padding::from([3, 8]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(TEAL_BG.into()),
                    border: Border {
                        radius: 20.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        } else {
            Space::with_width(0).into()
        };

        let dot =
            container(Space::with_width(7))
                .width(7)
                .height(7)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(status_color.into()),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

        let action_btn: Element<Message> = match r.status {
            PhpStatus::Installed => {
                let ver = r.version.clone();
                button(text("Remove").size(12).color(RED))
                    .on_press(Message::TOOLS_RemovePhp(ver))
                    .padding(Padding::from([6, 14]))
                    .style(|_, status| match status {
                        iced::widget::button::Status::Hovered
                        | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                            background: Some(RED_HOVER.into()),
                            text_color: RED,
                            border: Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        _ => iced::widget::button::Style {
                            background: Some(RED_BG.into()),
                            text_color: RED,
                            border: Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    })
                    .into()
            }
            PhpStatus::Available | PhpStatus::Unknown => {
                let ver = r.version.clone();
                button(text("Install").size(12).color(GREEN))
                    .on_press(Message::TOOLS_InstallPhp(ver))
                    .padding(Padding::from([6, 14]))
                    .style(|_, status| match status {
                        iced::widget::button::Status::Hovered
                        | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                            background: Some(GREEN_HOVER.into()),
                            text_color: GREEN,
                            border: Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        _ => iced::widget::button::Style {
                            background: Some(GREEN_BG.into()),
                            text_color: GREEN,
                            border: Border {
                                radius: 8.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    })
                    .into()
            }
        };

        container(
            row![
                dot,
                Space::with_width(12),
                column![
                    row![
                        text(format!("PHP {}", r.version))
                            .size(14)
                            .color(TEXT_PRIMARY),
                        Space::with_width(8),
                        active_badge,
                    ]
                    .align_y(Alignment::Center),
                    Space::with_height(2),
                    text(status_label).size(11).color(status_color),
                ]
                .spacing(0)
                .width(Length::Fill),
                action_btn,
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([12, 14]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_SURFACE.into()),
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    // ── Database CLI panel ─────────────────────────────────────────────────

    fn db_panel(&self) -> Element<'_, Message> {
        let divider = thin_line();

        let open_mysql_btn = db_btn(
            "MySQL / MariaDB",
            "Open root shell in terminal",
            BLUE,
            BLUE_BG,
            BLUE_HOVER,
            BLUE_BORDER,
            Message::TOOLS_OpenMysqlCli,
        );
        let open_mariadb_btn = db_btn(
            "MariaDB (explicit)",
            "Forces mariadb binary if installed",
            PURPLE,
            PURPLE_BG,
            PURPLE_HOVER,
            PURPLE_BORDER,
            Message::TOOLS_OpenMariadbCli,
        );
        let mysql_socket_btn = db_btn(
            "MySQL (socket auth)",
            "Connect via unix socket, no password",
            TEAL,
            TEAL_BG,
            TEAL_HOVER,
            TEAL_BORDER,
            Message::TOOLS_OpenMysqlSocket,
        );

        let status_row: Element<Message> = if !self.db_status.is_empty() {
            container(text(&self.db_status).size(12).color(TEXT_SECONDARY))
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
                })
                .into()
        } else {
            Space::with_height(0).into()
        };

        let note = container(
            row![
                text("!").size(10).color(YELLOW),
                Space::with_width(8),
                text("Opens your system terminal emulator as root.")
                    .size(11)
                    .color(TEXT_MUTED),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(YELLOW_BG.into()),
            border: Border {
                color: YELLOW_BORDER,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        });

        container(
            column![
                text("Database CLI").size(14).color(TEXT_SECONDARY),
                Space::with_height(3),
                text("Launch a root MySQL/MariaDB shell")
                    .size(11)
                    .color(TEXT_MUTED),
                Space::with_height(18),
                divider,
                Space::with_height(14),
                open_mysql_btn,
                Space::with_height(8),
                open_mariadb_btn,
                Space::with_height(8),
                mysql_socket_btn,
                Space::with_height(16),
                status_row,
                Space::with_height(if self.db_status.is_empty() { 0 } else { 12 }),
                note,
            ]
            .spacing(0)
            .padding(Padding::from([22, 22])),
        )
        .width(Length::FillPortion(2))
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_CARD.into()),
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    // ── Activity log ───────────────────────────────────────────────────────

    fn error_suggestion_panel(&self) -> Element<'_, Message> {
        let php_version = self
            .install_log
            .iter()
            .rev()
            .find_map(|(ok, msg)| {
                if !*ok && msg.contains("PHP") {
                    msg.split("PHP ")
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "8.2".to_string());

        let fix_commands = format!(
            "# Add the packages.sury.org/php repository.\nsudo apt-get update\nsudo apt-get install -y lsb-release ca-certificates apt-transport-https curl\nsudo curl -sSLo /tmp/debsuryorg-archive-keyring.deb https://packages.sury.org/debsuryorg-archive-keyring.deb\nsudo dpkg -i /tmp/debsuryorg-archive-keyring.deb\nsudo sh -c 'echo \"deb [signed-by=/usr/share/keyrings/debsuryorg-archive-keyring.gpg] https://packages.sury.org/php/ $(lsb_release -sc) main\" > /etc/apt/sources.list.d/php.list'\nsudo apt-get update\n\n# Install PHP.\n# To install manualy\nsudo apt-get install -y php{}"
        , php_version);

        let copy_btn = button(text("Get Text File").size(11).color(TEXT_PRIMARY))
            .on_press(Message::TOOLS_CopyFixCommands(fix_commands.clone()))
            .padding(Padding::from([6, 12]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(BLUE_HOVER.into()),
                    text_color: Color::WHITE,
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                _ => iced::widget::button::Style {
                    background: Some(BLUE_BG.into()),
                    text_color: TEXT_PRIMARY,
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            });

        container(
            column![
                row![
                    text("⚠ PHP Not Found").size(13).color(Color {
                        r: 1.0,
                        g: 0.650,
                        b: 0.0,
                        a: 1.0
                    }),
                    Space::with_width(Length::Fill),
                ]
                .align_y(Alignment::Center),
                Space::with_height(10),
                text("The ondrej/php PPA is not installed. Run these commands to add it:")
                    .size(12)
                    .color(TEXT_SECONDARY),
                Space::with_height(12),
                container(scrollable(text(fix_commands).size(10).color(BORDER_MED)).height(180),)
                    .padding(Padding::from([12, 14]))
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(
                            Color {
                                r: 0.08,
                                g: 0.08,
                                b: 0.08,
                                a: 1.0,
                            }
                            .into(),
                        ),
                        border: Border {
                            color: BORDER_SUBTLE,
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    }),
                Space::with_height(10),
                copy_btn,
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .padding(Padding::from([16, 18]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(
                Color {
                    r: 0.200,
                    g: 0.120,
                    b: 0.080,
                    a: 1.0,
                }
                .into(),
            ),
            border: Border {
                color: Color {
                    r: 1.0,
                    g: 0.650,
                    b: 0.0,
                    a: 1.0,
                },
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn log_panel(&self) -> Element<'_, Message> {
        if self.install_log.is_empty() {
            return Space::with_height(0).into();
        }

        let rows: Vec<Element<Message>> = self
            .install_log
            .iter()
            .map(|(ok, msg)| {
                let (prefix, color) = if *ok { ("OK  ", GREEN) } else { ("ERR ", RED) };
                row![
                    text(prefix).size(11).color(color),
                    text(msg.as_str()).size(12).color(TEXT_SECONDARY),
                ]
                .into()
            })
            .collect();

        container(
            column![
                row![
                    text("Activity Log")
                        .size(12)
                        .color(TEXT_MUTED)
                        .width(Length::Fill),
                    button(text("Clear").size(11).color(TEXT_MUTED))
                        .on_press(Message::TOOLS_ClearLog)
                        .padding(Padding::from([4, 10]))
                        .style(|_, status| match status {
                            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                                background: Some(BG_HOVER.into()),
                                text_color: TEXT_PRIMARY,
                                border: Border {
                                    radius: 6.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            _ => iced::widget::button::Style {
                                background: None,
                                text_color: TEXT_MUTED,
                                ..Default::default()
                            },
                        }),
                ]
                .align_y(Alignment::Center),
                Space::with_height(10),
                scrollable(column(rows).spacing(5).padding(Padding::from([4, 0]))).height(150),
            ]
            .spacing(0)
            .padding(Padding::from([16, 18])),
        )
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_SURFACE.into()),
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn thin_line<'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_SUBTLE.into()),
            ..Default::default()
        })
}

fn db_btn<'a>(
    title: &'a str,
    subtitle: &'a str,
    accent: Color,
    bg: Color,
    bg_hover: Color,
    _border: Color,
    msg: Message,
) -> Element<'a, Message> {
    button(
        row![
            container(Space::with_width(4))
                .width(4)
                .height(28)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(accent.into()),
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(12),
            column![
                text(title).size(13).color(TEXT_PRIMARY),
                Space::with_height(2),
                text(subtitle).size(11).color(TEXT_MUTED),
            ]
            .spacing(0)
            .width(Length::Fill),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(msg)
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(bg_hover.into()),
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
            background: Some(bg.into()),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
    })
    .into()
}
