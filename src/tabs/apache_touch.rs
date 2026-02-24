// src/tabs/apache_touch.rs

use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

fn card_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
        ..Default::default()
    }
}

fn surface_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(BG_SURFACE.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 6.0.into() },
        ..Default::default()
    }
}

fn input_label<'a>(label: &'a str) -> Element<'a, Message> {
    text(label).size(12).color(TEXT_MUTED).into()
}

fn section_title<'a>(t: &'a str) -> Element<'a, Message> {
    text(t).size(15).color(TEXT_SECONDARY).into()
}

fn btn_style(
    bg: Color,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(
                    Color::from_rgb(bg.r * 0.82, bg.g * 0.82, bg.b * 0.82).into(),
                ),
                text_color: Color::WHITE,
                border: Border { radius: 6.0.into(), ..Default::default() },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: Color::WHITE,
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        },
    }
}

// ── Log entry ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum LogKind {
    Info,
    Success,
    Warning,
    Error,
    Cmd,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub kind: LogKind,
    pub message: String,
}

impl LogEntry {
    pub fn info(msg: impl Into<String>) -> Self {
        Self { kind: LogKind::Info, message: msg.into() }
    }
    pub fn ok(msg: impl Into<String>) -> Self {
        Self { kind: LogKind::Success, message: msg.into() }
    }
    pub fn warn(msg: impl Into<String>) -> Self {
        Self { kind: LogKind::Warning, message: msg.into() }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Self { kind: LogKind::Error, message: msg.into() }
    }
    pub fn cmd(msg: impl Into<String>) -> Self {
        Self { kind: LogKind::Cmd, message: msg.into() }
    }
}

// ── State ─────────────────────────────────────────────────────────────────

pub struct ApacheTouchTab {
    pub project_name: String,
    pub auth_json_path: String,
    pub base_dir: String,
    pub apache_conf: String,
    pub log: Vec<LogEntry>,
    pub running: bool,
    pub finished_ok: Option<bool>, // Some(true)=success, Some(false)=fail, None=idle
}

impl ApacheTouchTab {
    pub fn new() -> Self {
        Self {
            project_name: String::new(),
            auth_json_path: String::new(),
            base_dir: "/var/www".into(),
            apache_conf: "/etc/apache2/sites-available/projects.conf".into(),
            log: Vec::new(),
            running: false,
            finished_ok: None,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        // ── Form ──────────────────────────────────────────────────────────
        let form = container(
            column![
                section_title("New VirtualHost"),
                Space::with_height(12),
                // row 1
                row![
                    column![
                        input_label("Project Name *"),
                        Space::with_height(4),
                        text_input("e.g. my-laravel-app", &self.project_name)
                            .on_input(Message::AT_ProjectNameChanged)
                            .padding(10)
                            .size(14),
                    ]
                    .spacing(0)
                    .width(Length::FillPortion(2)),
                    Space::with_width(12),
                    column![
                        input_label("Base Directory"),
                        Space::with_height(4),
                        text_input("/var/www", &self.base_dir)
                            .on_input(Message::AT_BaseDirChanged)
                            .padding(10)
                            .size(14),
                    ]
                    .spacing(0)
                    .width(Length::FillPortion(2)),
                ]
                .align_y(Alignment::Start),
                Space::with_height(10),
                // row 2
                row![
                    column![
                        input_label("Apache Config File"),
                        Space::with_height(4),
                        text_input(
                            "/etc/apache2/sites-available/projects.conf",
                            &self.apache_conf
                        )
                        .on_input(Message::AT_ApacheConfChanged)
                        .padding(10)
                        .size(14),
                    ]
                    .spacing(0)
                    .width(Length::FillPortion(3)),
                    Space::with_width(12),
                    column![
                        input_label("auth.json (optional)"),
                        Space::with_height(4),
                        row![
                            text_input("Path to auth.json", &self.auth_json_path)
                                .on_input(Message::AT_AuthJsonChanged)
                                .padding(10)
                                .size(14)
                                .width(Length::Fill),
                            button(text("Browse").size(12))
                                .on_press(Message::AT_BrowseAuthJson)
                                .padding(Padding::from([10, 14]))
                                .style(btn_style(ACCENT_DIM)),
                        ]
                        .spacing(6),
                    ]
                    .spacing(0)
                    .width(Length::FillPortion(3)),
                ]
                .align_y(Alignment::Start),
                Space::with_height(16),
                // Action buttons
                row![
                    button(
                        text(if self.running { "  Running..." } else { ">> Run Setup" }).size(14)
                    )
                    .on_press_maybe(if self.running { None } else { Some(Message::AT_RunSetup) })
                    .padding(Padding::from([10, 24]))
                    .style(btn_style(if self.running { ACCENT_DIM } else { ACCENT })),
                    button(text("Clear Log").size(13))
                        .on_press(Message::AT_ClearLog)
                        .padding(Padding::from([10, 18]))
                        .style(|_, status| match status {
                            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                                background: Some(BG_HOVER.into()),
                                text_color: TEXT_PRIMARY,
                                border: Border { color: BORDER_MED, width: 1.0, radius: 6.0.into() },
                                ..Default::default()
                            },
                            _ => iced::widget::button::Style {
                                background: Some(BG_SURFACE.into()),
                                text_color: TEXT_SECONDARY,
                                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 6.0.into() },
                                ..Default::default()
                            },
                        }),
                ]
                .spacing(10),
            ]
            .spacing(0)
            .padding(20),
        )
        .width(Length::Fill)
        .style(card_style());

        // ── Status banner ─────────────────────────────────────────────────
        let banner: Element<Message> = match self.finished_ok {
            Some(true) => container(
                row![
                    text("[OK]").size(14).color(GREEN),
                    text("  VirtualHost created successfully! Apache reloaded.")
                        .size(13)
                        .color(GREEN),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([10, 16]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(Color { r: GREEN.r, g: GREEN.g, b: GREEN.b, a: 0.08 }.into()),
                border: Border { color: Color { a: 0.4, ..GREEN }, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            })
            .into(),
            Some(false) => container(
                row![
                    text("[ERR]").size(14).color(RED),
                    text("  Setup finished with errors. Check the log below.")
                        .size(13)
                        .color(RED),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([10, 16]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(Color { r: RED.r, g: RED.g, b: RED.b, a: 0.08 }.into()),
                border: Border { color: Color { a: 0.4, ..RED }, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            })
            .into(),
            _ => Space::with_height(0).into(),
        };

        // ── Log panel ─────────────────────────────────────────────────────
        let log_content: Element<Message> = if self.log.is_empty() {
            text("Log output will appear here after running setup...")
                .size(13)
                .color(TEXT_MUTED)
                .into()
        } else {
            let rows: Vec<Element<Message>> = self
                .log
                .iter()
                .map(|e| {
                    let (icon, color) = match e.kind {
                        LogKind::Info    => ("[i]", TEXT_SECONDARY),
                        LogKind::Success => ("[+]", GREEN),
                        LogKind::Warning => ("[!]", YELLOW),
                        LogKind::Error   => ("[-]", RED),
                        LogKind::Cmd     => ("[$]", PURPLE),
                    };
                    row![
                        text(icon).size(13).color(color),
                        text(format!("  {}", e.message)).size(13).color(color),
                    ]
                    .into()
                })
                .collect();

            scrollable(
                column(rows)
                    .spacing(5)
                    .padding(Padding::from([8, 4])),
            )
            .height(220)
            .into()
        };

        let log_panel = container(log_content)
            .width(Length::Fill)
            .padding(14)
            .style(surface_style());

        scrollable(
            column![
                // Header
                column![
                    text("ApacheTouch").size(26).color(TEXT_PRIMARY),
                    text("Create and register Apache VirtualHosts on Debian/Ubuntu")
                        .size(13)
                        .color(TEXT_MUTED),
                ]
                .spacing(3),
                Space::with_height(16),
                form,
                Space::with_height(12),
                banner,
                Space::with_height(8),
                text("Setup Log").size(12).color(TEXT_MUTED),
                Space::with_height(4),
                log_panel,
                Space::with_height(20),
                self.help_card(),
            ]
            .spacing(0)
            .padding(Padding::from([20, 24])),
        )
        .into()
    }

    fn help_card(&self) -> Element<'_, Message> {
        container(
            column![
                text("What this does").size(13).color(TEXT_SECONDARY),
                Space::with_height(8),
                help_row("1.", "Checks the project directory exists under Base Directory"),
                help_row("2.", "Copies .env.example -> .env if found"),
                help_row("3.", "Copies auth.json to the project root (optional)"),
                help_row("4.", "Adds 127.0.0.1 <project>.local to /etc/hosts"),
                help_row("5.", "Appends a <VirtualHost *:80> block to the Apache config"),
                help_row("6.", "Runs sudo a2ensite and reloads Apache"),
            ]
            .spacing(4)
            .padding(16),
        )
        .width(Length::Fill)
        .style(surface_style())
        .into()
    }
}

fn help_row<'a>(num: &'a str, desc: &'a str) -> Element<'a, Message> {
    row![
        text(num).size(12).color(ACCENT),
        text(format!("  {}", desc)).size(12).color(TEXT_MUTED),
    ]
    .into()
}
