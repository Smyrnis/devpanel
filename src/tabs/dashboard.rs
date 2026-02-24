// src/tabs/dashboard.rs

use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, pick_list, row, scrollable, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

// ── Styling helpers ───────────────────────────────────────────────────────

fn card_style(border_color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
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
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

fn colored_btn(bg: Color) -> iced::widget::button::Style {
    iced::widget::button::Style {
        background: Some(bg.into()),
        text_color: Color::WHITE,
        border: Border {
            radius: 5.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn btn_style(bg: Color) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(Color::from_rgb(bg.r * 0.82, bg.g * 0.82, bg.b * 0.82).into()),
                text_color: Color::WHITE,
                border: Border { radius: 5.0.into(), ..Default::default() },
                ..Default::default()
            }
        }
        _ => colored_btn(bg),
    }
}

fn ghost_btn_style() -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(BG_HOVER.into()),
                text_color: TEXT_PRIMARY,
                border: Border { color: ACCENT, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(BG_SURFACE.into()),
            text_color: TEXT_SECONDARY,
            border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 6.0.into() },
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
        // ── Top info bar ──────────────────────────────────────────────────
        let info_bar = container(
            row![
                dot(ACCENT),
                text(format!("  {}  ", self.distro.to_uppercase()))
                    .size(12)
                    .color(TEXT_SECONDARY),
                sep(),
                text(format!("  Web Root: {}  ", self.web_root))
                    .size(12)
                    .color(TEXT_SECONDARY),
                sep(),
                text(format!("  Apache Conf: {}  ", self.apache_conf_dir))
                    .size(12)
                    .color(TEXT_SECONDARY),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([8, 14]))
        .width(Length::Fill)
        .style(surface_style());

        // ── Services row ──────────────────────────────────────────────────
        let services = row![
            self.service_card(
                "Apache",
                "HTTP",
                self.apache_running,
                GREEN,
                Message::StartApache,
                Message::StopApache,
                Message::RestartApache,
            ),
            self.service_card(
                "MySQL",
                "DB",
                self.mysql_running,
                BLUE,
                Message::StartMySQL,
                Message::StopMySQL,
                Message::RestartMySQL,
            ),
            self.php_card(),
        ]
        .spacing(12);

        // ── Quick actions ──────────────────────────────────────────────────
        let quick_title = text("Quick Actions").size(15).color(TEXT_SECONDARY);

        let quick_grid = column![
            self.quick_row(&[
                ("www", "localhost",    Message::OpenLocalhost),
                ("db",  "phpMyAdmin",  Message::OpenPhpMyAdmin),
                ("dir", "Web Root",    Message::OpenWebRoot),
            ]),
            self.quick_row(&[
                ("cfg", "Apache Conf", Message::OpenApacheConfig),
                ("sql", "MySQL Conf",  Message::OpenMySQLConfig),
                ("php", "PHP Config",  Message::OpenPHPConfig),
            ]),
            self.quick_row(&[
                ("hst", "Hosts File",  Message::EditHosts),
                ("rst", "Restart All", Message::RestartAll),
                ("clr", "Clear Cache", Message::ClearCache),
            ]),
        ]
        .spacing(8);

        scrollable(
            column![
                info_bar,
                Space::with_height(12),
                services,
                Space::with_height(20),
                quick_title,
                Space::with_height(8),
                quick_grid,
            ]
            .spacing(0)
            .padding(Padding::from([16, 20])),
        )
        .into()
    }

    // ── Service card ──────────────────────────────────────────────────────

    fn service_card<'a>(
        &self,
        name: &'a str,
        badge: &'a str,
        running: bool,
        accent: Color,
        start: Message,
        stop: Message,
        restart: Message,
    ) -> Element<'a, Message> {
        let status_text = if running { "[on]  Running" } else { "[off] Stopped" };
        let status_color = if running { GREEN } else { RED };

        let header = row![
            container(text(badge).size(11).color(accent))
                .padding(Padding::from([3, 8]))
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(Color { a: 0.15, ..accent }.into()),
                    border: Border { color: Color { a: 0.4, ..accent }, width: 1.0, radius: 4.0.into() },
                    ..Default::default()
                }),
            Space::with_width(Length::Fill),
            text(status_text).size(12).color(status_color),
        ]
        .align_y(Alignment::Center);

        let svc_name = text(name).size(20).color(TEXT_PRIMARY);

        let btn_row = row![
            button(text("Start").size(12))
                .on_press(start)
                .padding(Padding::from([6, 14]))
                .style(btn_style(BTN_SUCCESS)),
            button(text("Stop").size(12))
                .on_press(stop)
                .padding(Padding::from([6, 14]))
                .style(btn_style(BTN_DANGER)),
            button(text("Restart").size(12))
                .on_press(restart)
                .padding(Padding::from([6, 14]))
                .style(btn_style(BTN_WARN)),
        ]
        .spacing(6);

        container(
            column![header, Space::with_height(10), svc_name, Space::with_height(14), btn_row]
                .spacing(0),
        )
        .padding(18)
        .width(Length::FillPortion(1))
        .style(card_style(if running { Color { a: 0.35, ..accent } } else { BORDER_SUBTLE }))
        .into()
    }

    // ── PHP card ──────────────────────────────────────────────────────────

    fn php_card(&self) -> Element<'_, Message> {
        let header = row![
            container(text("PHP").size(11).color(PURPLE))
                .padding(Padding::from([3, 8]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(Color { a: 0.15, ..PURPLE }.into()),
                    border: Border { color: Color { a: 0.4, ..PURPLE }, width: 1.0, radius: 4.0.into() },
                    ..Default::default()
                }),
            Space::with_width(Length::Fill),
            text(self.active_php_version.as_deref().unwrap_or("n/a"))
                .size(12)
                .color(TEXT_SECONDARY),
        ]
        .align_y(Alignment::Center);

        let picker: Element<Message> = if !self.php_versions.is_empty() {
            pick_list(
                &self.php_versions[..],
                self.active_php_version.as_ref(),
                Message::SwitchPHPVersion,
            )
            .padding(8)
            .width(Length::Fill)
            .into()
        } else {
            text("No PHP detected").size(13).color(TEXT_MUTED).into()
        };

        let php_info_btn = button(text("PHP Info").size(12))
            .on_press(Message::ShowPHPInfo)
            .padding(Padding::from([6, 14]))
            .style(btn_style(ACCENT_DIM));

        container(
            column![
                header,
                Space::with_height(10),
                text("PHP Engine").size(20).color(TEXT_PRIMARY),
                Space::with_height(14),
                text("Active Version").size(11).color(TEXT_MUTED),
                Space::with_height(4),
                picker,
                Space::with_height(8),
                php_info_btn,
            ]
            .spacing(0),
        )
        .padding(18)
        .width(Length::FillPortion(1))
        .style(card_style(BORDER_SUBTLE))
        .into()
    }

    // ── Quick action helpers ──────────────────────────────────────────────

    fn quick_row<'a>(&self, items: &[(&'a str, &'a str, Message)]) -> Element<'a, Message> {
        let btns: Vec<Element<Message>> = items
            .iter()
            .map(|(icon, label, msg)| {
                button(
                    column![
                        text(*icon).size(13).color(ACCENT),
                        text(*label).size(11).color(TEXT_SECONDARY),
                    ]
                    .spacing(4)
                    .align_x(Alignment::Center),
                )
                .on_press(msg.clone())
                .padding(Padding::from([12, 8]))
                .width(Length::FillPortion(1))
                .style(ghost_btn_style())
                .into()
            })
            .collect();

        row(btns).spacing(8).into()
    }
}

// ── Small helpers ─────────────────────────────────────────────────────────

fn dot(color: Color) -> Element<'static, Message> {
    container(Space::with_width(8))
        .width(8)
        .height(8)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(color.into()),
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn sep() -> Element<'static, Message> {
    container(Space::with_width(1))
        .width(1)
        .height(14)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_SUBTLE.into()),
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
