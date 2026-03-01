// src/tabs/dashboard.rs

use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, pick_list, row, scrollable, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

// ── Pre-computed tinted solids (no alpha struct tricks) ───────────────────
const GREEN_BG: Color = Color {
    r: 0.050,
    g: 0.160,
    b: 0.090,
    a: 1.0,
};
const PURPLE_BG: Color = Color {
    r: 0.140,
    g: 0.060,
    b: 0.180,
    a: 1.0,
};
const PURPLE_BG2: Color = Color {
    r: 0.180,
    g: 0.080,
    b: 0.230,
    a: 1.0,
}; // slightly lighter
const PURPLE_BDR: Color = Color {
    r: 0.200,
    g: 0.090,
    b: 0.260,
    a: 1.0,
};
const BLUE_BG: Color = Color {
    r: 0.050,
    g: 0.090,
    b: 0.180,
    a: 1.0,
};
const STOPPED_BG: Color = Color {
    r: 0.150,
    g: 0.150,
    b: 0.160,
    a: 1.0,
};
const STATUS_STOP: Color = Color {
    r: 0.500,
    g: 0.500,
    b: 0.520,
    a: 1.0,
};

// ── Card / surface styles ─────────────────────────────────────────────────

fn card_style(border_color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

fn surface_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(BG_SURFACE.into()),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn btn_style(
    bg: Color,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(
                    Color::from_rgba(bg.r * 0.82, bg.g * 0.82, bg.b * 0.82, 1.0).into(),
                ),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::BLACK,
                    width: 1.5,
                    radius: 7.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: Color::WHITE,
            border: Border {
                color: Color::BLACK,
                width: 1.5,
                radius: 7.0.into(),
            },
            ..Default::default()
        },
    }
}

fn ghost_btn_style(
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
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
    }
}

// ── State ─────────────────────────────────────────────────────────────────

pub struct DashboardTab {
    pub apache_running: bool,
    pub mysql_running: bool,
    pub php_versions: Vec<String>,
    pub active_php_version: Option<String>,
    pub distro: String,
    pub web_root: String,
    pub apache_conf_dir: String,
}

impl DashboardTab {
    pub fn new() -> Self {
        Self {
            apache_running: false,
            mysql_running: false,
            php_versions: Vec::new(),
            active_php_version: None,
            distro: detect_distro(),
            web_root: "/var/www/html".into(),
            apache_conf_dir: "/etc/apache2".into(),
        }
    }

    pub fn update_status(&mut self, apache: bool, mysql: bool, php: Option<String>) {
        self.apache_running = apache;
        self.mysql_running = mysql;
        self.active_php_version = php;
    }

    pub fn set_php_versions(&mut self, versions: Vec<String>) {
        self.php_versions = versions;
    }

    pub fn view(&self) -> Element<'_, Message> {
        let info_bar = container(
            row![
                status_dot(TEAL),
                Space::with_width(8),
                text(&self.distro).size(12).color(TEXT_SECONDARY),
                Space::with_width(16),
                sep_vertical(),
                Space::with_width(16),
                text("Web Root").size(11).color(TEXT_MUTED),
                Space::with_width(6),
                text(&self.web_root).size(12).color(TEXT_PRIMARY),
                Space::with_width(16),
                sep_vertical(),
                Space::with_width(16),
                text("Apache").size(11).color(TEXT_MUTED),
                Space::with_width(6),
                text(&self.apache_conf_dir).size(12).color(TEXT_PRIMARY),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([11, 18]))
        .width(Length::Fill)
        .style(surface_style());

        let services = row![
            self.service_card(
                "Apache",
                "HTTP Server",
                self.apache_running,
                GREEN,
                Message::StartApache,
                Message::StopApache,
                Message::RestartApache
            ),
            self.service_card(
                "MySQL",
                "Database",
                self.mysql_running,
                BLUE,
                Message::StartMySQL,
                Message::StopMySQL,
                Message::RestartMySQL
            ),
            self.php_card(),
        ]
        .spacing(12);

        let qa_title = text("Quick Actions").size(13).color(TEXT_SECONDARY);

        let quick_grid = column![
            self.quick_row(&[
                ("Localhost",    Message::OpenLocalhost),
                ("phpMyAdmin",   Message::OpenPhpMyAdmin),
                ("Projects",     Message::OpenProjectsFolder),
            ]),
            self.quick_row(&[
                ("apache2.conf", Message::NavigateApache2Conf),
                ("sites-avail",  Message::NavigateApache2Sites),
                ("devpanel.conf",Message::VH_OpenDevpanelConf),
            ]),
            self.quick_row(&[
                ("/etc/php",     Message::NavigatePhpDir),
                ("/etc/mysql",   Message::NavigateMysqlDir),
                ("/etc/hosts",   Message::NavigateHostsFile),
            ]),
            self.quick_row(&[
                ("Web Root",     Message::OpenWebRoot),
                ("php.ini",      Message::OpenPhpIni),
                ("Restart All",  Message::RestartAll),
            ]),
        ]
        .spacing(8);

        scrollable(
            column![
                info_bar,
                Space::with_height(20),
                services,
                Space::with_height(28),
                qa_title,
                Space::with_height(12),
                quick_grid,
                Space::with_height(24),
            ]
            .spacing(0)
            .padding(Padding::from([20, 22])),
        )
        .into()
    }

    // ── Service card ──────────────────────────────────────────────────────

    fn service_card<'a>(
        &self,
        name: &'a str,
        subtitle: &'a str,
        running: bool,
        accent: Color,
        start: Message,
        stop: Message,
        restart: Message,
    ) -> Element<'a, Message> {
        let status_color = if running { GREEN } else { STATUS_STOP };
        let status_label = if running { "Running" } else { "Stopped" };
        let status_bg = if running { GREEN_BG } else { STOPPED_BG };

        // Accent pill (top-left) — solid tinted bg
        let accent_pill_bg = if accent == GREEN { GREEN_BG } else { BLUE_BG };

        let top = row![
            container(
                text(if running { "ON" } else { "OFF" })
                    .size(9)
                    .color(accent)
            )
            .padding(Padding::from([4, 8]))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(accent_pill_bg.into()),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(Length::Fill),
            container(
                row![
                    status_dot(status_color),
                    Space::with_width(5),
                    text(status_label).size(11).color(status_color),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([4, 9]))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(status_bg.into()),
                border: Border {
                    radius: 20.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ]
        .align_y(Alignment::Center);

        let divider = thin_line();

        let btn_row = row![
            button(text("Start").size(13).width(Length::Fill).center())
                .on_press(start)
                .padding(Padding::from([7, 0]))
                .width(Length::FillPortion(1))
                .style(btn_style(BTN_SUCCESS)),
            button(text("Stop").size(13).width(Length::Fill).center())
                .on_press(stop)
                .padding(Padding::from([7, 0]))
                .width(Length::FillPortion(1))
                .style(btn_style(BTN_DANGER)),
            button(text("Restart").size(13).width(Length::Fill).center())
                .on_press(restart)
                .padding(Padding::from([7, 0]))
                .width(Length::FillPortion(1))
                .style(btn_style(BTN_WARN)),
        ]
        .spacing(7);

        // Running state gets a tinted border accent
        let card_border = if running { GREEN_BG } else { BORDER_SUBTLE };

        container(
            column![
                top,
                Space::with_height(14),
                text(name).size(19).color(TEXT_PRIMARY),
                Space::with_height(3),
                text(subtitle).size(12).color(TEXT_MUTED),
                Space::with_height(16),
                divider,
                Space::with_height(14),
                btn_row,
            ]
            .spacing(0),
        )
        .padding(Padding::from([18, 18]))
        .width(Length::FillPortion(1))
        .style(card_style(card_border))
        .into()
    }

    // ── PHP card ──────────────────────────────────────────────────────────

    fn php_card(&self) -> Element<'_, Message> {
        let version_text = self.active_php_version.as_deref().unwrap_or("n/a");

        let running_dot = container(
            row![
                status_dot(PURPLE),
                Space::with_width(5),
                text(version_text).size(11).color(PURPLE),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([4, 9]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(PURPLE_BG.into()),
            border: Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let top = row![
            container(text("PHP").size(9).color(PURPLE))
                .padding(Padding::from([4, 8]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(PURPLE_BG.into()),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            Space::with_width(Length::Fill),
            running_dot,
        ]
        .align_y(Alignment::Center);

        let picker: Element<Message> = if !self.php_versions.is_empty() {
            pick_list(
                &self.php_versions[..],
                self.active_php_version.as_ref(),
                Message::SwitchPHPVersion,
            )
            .padding(9)
            .width(Length::Fill)
            .into()
        } else {
            container(text("No PHP detected").size(13).color(TEXT_MUTED))
                .padding(Padding::from([9, 12]))
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
        };

        let divider = thin_line();

        let php_info_btn = button(text("PHP Info").size(13))
            .on_press(Message::ShowPHPInfo)
            .padding(Padding::from([7, 14]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                    iced::widget::button::Style {
                        background: Some(PURPLE_BG2.into()),
                        text_color: PURPLE,
                        border: Border {
                            color: PURPLE_BDR,
                            width: 1.0,
                            radius: 7.0.into(),
                        },
                        ..Default::default()
                    }
                }
                _ => iced::widget::button::Style {
                    background: Some(PURPLE_BG.into()),
                    text_color: PURPLE,
                    border: Border {
                        radius: 7.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            });

        container(
            column![
                top,
                Space::with_height(14),
                text("PHP Engine").size(19).color(TEXT_PRIMARY),
                Space::with_height(3),
                text("Version Switcher").size(12).color(TEXT_MUTED),
                Space::with_height(16),
                divider,
                Space::with_height(14),
                text("Active Version").size(11).color(TEXT_MUTED),
                Space::with_height(6),
                picker,
                Space::with_height(8),
                php_info_btn,
            ]
            .spacing(0),
        )
        .padding(Padding::from([18, 18]))
        .width(Length::FillPortion(1))
        .style(card_style(BORDER_SUBTLE))
        .into()
    }

    // ── Quick action row ──────────────────────────────────────────────────

    fn quick_row<'a>(&self, items: &[(&'a str, Message)]) -> Element<'a, Message> {
        let btns: Vec<Element<Message>> = items
            .iter()
            .map(|(label, msg)| {
                button(text(*label).size(13).color(TEXT_PRIMARY))
                    .on_press(msg.clone())
                    .padding(Padding::from([14, 12]))
                    .width(Length::FillPortion(1))
                    .style(ghost_btn_style())
                    .into()
            })
            .collect();

        row(btns).spacing(8).into()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn thin_line<'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_SUBTLE.into()),
            ..Default::default()
        })
}

fn status_dot(color: Color) -> Element<'static, Message> {
    container(Space::with_width(6))
        .width(6)
        .height(6)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(color.into()),
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn sep_vertical() -> Element<'static, Message> {
    container(Space::with_width(1))
        .width(1)
        .height(12)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_MED.into()),
            ..Default::default()
        })
        .into()
}

fn detect_distro() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }
    "Linux".to_string()
}
