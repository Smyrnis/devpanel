// src/tabs/apache_touch/view.rs — all UI rendering for the ApacheTouch tab
#![allow(dead_code, unused)]

use super::{ApacheTouchTab, LogKind};
use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

const GREEN_BDR: Color = Color { r: 0.070, g: 0.210, b: 0.110, a: 1.0 };
const RED_BDR:   Color = Color { r: 0.260, g: 0.080, b: 0.070, a: 1.0 };
const TEAL_BG:   Color = Color { r: 0.040, g: 0.160, b: 0.150, a: 1.0 };

pub fn render(tab: &ApacheTouchTab) -> Element<'_, Message> {
    let form   = form_card(tab);
    let banner = status_banner(tab);
    let log    = log_panel(tab);

    scrollable(column![
        column![
            text("ApacheTouch").size(22).color(TEXT_PRIMARY),
            Space::with_height(4),
            text("Create and register Apache VirtualHosts on Debian/Ubuntu").size(13).color(TEXT_MUTED),
        ].spacing(0),
        Space::with_height(22),
        form,
        Space::with_height(14),
        banner,
        Space::with_height(12),
        text("Setup Log").size(11).color(TEXT_MUTED),
        Space::with_height(6),
        log,
        Space::with_height(22),
        help_card(),
        Space::with_height(22),
    ].spacing(0).padding(Padding::from([22, 24])))
    .into()
}

fn form_card(tab: &ApacheTouchTab) -> Element<'_, Message> {
    let divider = || container(Space::with_height(1)).width(Length::Fill).height(1)
        .style(|_: &iced::Theme| container::Style { background: Some(BORDER_SUBTLE.into()), ..Default::default() });

    container(column![
        text("New VirtualHost").size(14).color(TEXT_SECONDARY),
        Space::with_height(20),
        // Row 1
        row![
            column![
                input_label("Project Name *"), Space::with_height(6),
                text_input("my-laravel-app", &tab.project_name)
                    .on_input(Message::AT_ProjectNameChanged).padding(10).size(13),
            ].spacing(0).width(Length::FillPortion(2)),
            Space::with_width(14),
            column![
                input_label("Base Directory"), Space::with_height(6),
                text_input("/var/www", &tab.base_dir)
                    .on_input(Message::AT_BaseDirChanged).padding(10).size(13),
            ].spacing(0).width(Length::FillPortion(2)),
        ].align_y(Alignment::Start),
        Space::with_height(16),
        // Row 2
        row![
            column![
                input_label("Apache Config File"), Space::with_height(6),
                text_input("/etc/apache2/sites-available/projects.conf", &tab.apache_conf)
                    .on_input(Message::AT_ApacheConfChanged).padding(10).size(13),
            ].spacing(0).width(Length::FillPortion(3)),
            Space::with_width(14),
            column![
                input_label("auth.json (optional)"), Space::with_height(6),
                row![
                    text_input("Path to auth.json", &tab.auth_json_path)
                        .on_input(Message::AT_AuthJsonChanged).padding(10).size(13).width(Length::Fill),
                    button(text("Browse").size(12))
                        .on_press(Message::AT_BrowseAuthJson)
                        .padding(Padding::from([10, 14]))
                        .style(|_, status| match status {
                            iced::widget::button::Status::Hovered =>
                                iced::widget::button::Style { background: Some(BG_HOVER.into()), text_color: TEXT_PRIMARY,
                                    border: Border { color: BORDER_MED, width: 1.0, radius: 8.0.into() }, ..Default::default() },
                            _ => iced::widget::button::Style { background: Some(BG_SURFACE.into()), text_color: TEXT_SECONDARY,
                                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default() },
                        }),
                ].spacing(8),
            ].spacing(0).width(Length::FillPortion(3)),
        ].align_y(Alignment::Start),
        Space::with_height(22),
        divider(),
        Space::with_height(18),
        row![
            button(text(if tab.running { "Running..." } else { "Run Setup" }).size(13))
                .on_press_maybe(if tab.running { None } else { Some(Message::AT_RunSetup) })
                .padding(Padding::from([10, 22]))
                .style(btn_style(if tab.running { ACCENT_DIM } else { ACCENT })),
            button(text("Clear Log").size(12))
                .on_press(Message::AT_ClearLog)
                .padding(Padding::from([10, 18]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered =>
                        iced::widget::button::Style { background: Some(BG_HOVER.into()), text_color: TEXT_PRIMARY,
                            border: Border { color: BORDER_MED, width: 1.0, radius: 8.0.into() }, ..Default::default() },
                    _ => iced::widget::button::Style { background: Some(BG_SURFACE.into()), text_color: TEXT_SECONDARY,
                        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() }, ..Default::default() },
                }),
        ].spacing(10),
    ].spacing(0).padding(Padding::from([22, 22])))
    .width(Length::Fill).style(card_style())
    .into()
}

fn status_banner(tab: &ApacheTouchTab) -> Element<'_, Message> {
    match tab.finished_ok {
        Some(true) => container(row![
            icon_badge("+", GREEN),
            Space::with_width(10),
            text("VirtualHost created successfully — Apache reloaded").size(13).color(TEXT_PRIMARY),
        ].align_y(Alignment::Center))
        .padding(Padding::from([12, 16])).width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_CARD.into()),
            border: Border { color: GREEN_BDR, width: 1.0, radius: 10.0.into() },
            ..Default::default()
        }).into(),

        Some(false) => container(row![
            icon_badge("x", RED),
            Space::with_width(10),
            text("Setup finished with errors — check the log below").size(13).color(TEXT_PRIMARY),
        ].align_y(Alignment::Center))
        .padding(Padding::from([12, 16])).width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_CARD.into()),
            border: Border { color: RED_BDR, width: 1.0, radius: 10.0.into() },
            ..Default::default()
        }).into(),

        _ => Space::with_height(0).into(),
    }
}

fn log_panel(tab: &ApacheTouchTab) -> Element<'_, Message> {
    let content: Element<Message> = if tab.log.is_empty() {
        container(text("Log output will appear here after running setup").size(13).color(TEXT_MUTED))
            .padding(Padding::from([16, 16])).into()
    } else {
        let rows: Vec<Element<Message>> = tab.log.iter().map(|e| {
            let (prefix, color) = match e.kind {
                LogKind::Info    => ("  ", TEXT_SECONDARY),
                LogKind::Success => ("+ ", GREEN),
                LogKind::Warning => ("! ", YELLOW),
                LogKind::Error   => ("x ", RED),
                LogKind::Cmd     => ("$ ", TEAL),
            };
            row![
                text(prefix).size(12).color(color),
                text(e.message.as_str()).size(12).color(color),
            ].into()
        }).collect();
        scrollable(column(rows).spacing(6).padding(Padding::from([12, 14]))).height(220).into()
    };

    container(content).width(Length::Fill).style(surface_style()).into()
}

fn help_card<'a>() -> Element<'a, Message> {
    container(column![
        text("What this does").size(12).color(TEXT_SECONDARY),
        Space::with_height(12),
        help_row("1", "Checks the project directory exists under Base Directory"),
        help_row("2", "Copies .env.example → .env if found"),
        help_row("3", "Copies auth.json to the project root (optional)"),
        help_row("4", "Adds 127.0.0.1 <project>.local to /etc/hosts"),
        help_row("5", "Appends a <VirtualHost *:80> block to the Apache config"),
        help_row("6", "Runs sudo a2ensite and reloads Apache"),
    ].spacing(6).padding(Padding::from([18, 18])))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(BG_SURFACE.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 10.0.into() },
        ..Default::default()
    }).into()
}

fn help_row<'a>(num: &'a str, desc: &'a str) -> Element<'a, Message> {
    row![
        container(text(num).size(10).color(TEAL))
            .padding(Padding::from([3, 7]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(TEAL_BG.into()),
                border: Border { radius: 20.0.into(), ..Default::default() },
                ..Default::default()
            }),
        Space::with_width(10),
        text(desc).size(12).color(TEXT_SECONDARY),
    ].align_y(Alignment::Center).into()
}

fn card_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 10.0.into() },
        ..Default::default()
    }
}
fn surface_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(BG_SURFACE.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 10.0.into() },
        ..Default::default()
    }
}
fn btn_style(bg: Color) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
            iced::widget::button::Style {
                background: Some(Color::from_rgba(bg.r, bg.g, bg.b, 0.82).into()),
                text_color: Color::WHITE,
                border: Border { radius: 8.0.into(), ..Default::default() },
                shadow: iced::Shadow { color: Color { a: 0.3, ..bg }, offset: iced::Vector::new(0.0, 2.0), blur_radius: 8.0 },
                ..Default::default()
            },
        _ => iced::widget::button::Style {
            background: Some(bg.into()),
            text_color: Color::WHITE,
            border: Border { radius: 8.0.into(), ..Default::default() },
            ..Default::default()
        },
    }
}
fn input_label<'a>(label: &'a str) -> Element<'a, Message> {
    text(label).size(11).color(TEXT_MUTED).into()
}
fn icon_badge<'a>(icon: &'a str, color: Color) -> iced::widget::Container<'a, Message> {
    container(text(icon).size(10).color(Color::WHITE))
        .padding(Padding::from([3, 7]))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(color.into()),
            border: Border { radius: 20.0.into(), ..Default::default() },
            ..Default::default()
        })
}
