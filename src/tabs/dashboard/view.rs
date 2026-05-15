use super::DashboardTab;
use crate::core::theme::*;
use crate::messages::DashboardMessage;
use crate::messages::Message;
use iced::widget::{Space, button, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

const GREEN_BG: Color = Color { r: 0.071, g: 0.122, b: 0.082, a: 1.0 };
const PURPLE_BG: Color = Color { r: 0.110, g: 0.055, b: 0.165, a: 1.0 };
const PURPLE_BG2: Color = Color { r: 0.140, g: 0.070, b: 0.200, a: 1.0 };
const PURPLE_BDR: Color = Color { r: 0.170, g: 0.085, b: 0.240, a: 1.0 };
const BLUE_BG: Color = Color { r: 0.047, g: 0.090, b: 0.157, a: 1.0 };
const STOPPED_BG: Color = Color { r: 0.110, g: 0.110, b: 0.110, a: 1.0 };
const STATUS_STOP: Color = TEXT_MUTED;

pub fn render(tab: &DashboardTab) -> Element<'_, Message> {
    let info_bar = container(
        row![
            status_dot(TEAL),
            Space::with_width(8),
            text(&tab.distro).size(12).color(TEXT_SECONDARY),
            Space::with_width(16),
            sep_vertical(),
            Space::with_width(16),
            text("Web Root").size(11).color(TEXT_MUTED),
            Space::with_width(6),
            text(&tab.web_root).size(12).color(TEXT_PRIMARY),
            Space::with_width(16),
            sep_vertical(),
            Space::with_width(16),
            text("Apache").size(11).color(TEXT_MUTED),
            Space::with_width(6),
            text(&tab.apache_conf_dir).size(12).color(TEXT_PRIMARY),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([11, 18]))
    .width(Length::Fill)
    .style(surface_style());

    let services = row![
        service_card(
            "Apache", "HTTP Server", tab.apache_uptime.as_deref(), tab.apache_running, GREEN,
            ServiceActions {
                start: Message::Dashboard(DashboardMessage::StartApache),
                stop: Message::Dashboard(DashboardMessage::StopApache),
                restart: Message::Dashboard(DashboardMessage::RestartApache),
            },
        ),
        service_card(
            "MySQL", "Database", tab.mysql_uptime.as_deref(), tab.mysql_running, BLUE,
            ServiceActions {
                start: Message::Dashboard(DashboardMessage::StartMySQL),
                stop: Message::Dashboard(DashboardMessage::StopMySQL),
                restart: Message::Dashboard(DashboardMessage::RestartMySQL),
            },
        ),
        php_card(tab),
    ]
    .spacing(12);

    let quick_grid = column![
        quick_row(&[
            ("Localhost",   Message::Dashboard(DashboardMessage::OpenLocalhost)),
            ("phpMyAdmin",  Message::Dashboard(DashboardMessage::OpenPhpMyAdmin)),
            ("Projects",    Message::Dashboard(DashboardMessage::OpenProjectsFolder)),
        ]),
        quick_row(&[
            ("apache2.conf", Message::Dashboard(DashboardMessage::NavigateApache2Conf)),
            ("sites-avail",  Message::Dashboard(DashboardMessage::NavigateApache2Sites)),
            ("devpanel.conf", Message::VHosts(crate::messages::VHostsMessage::OpenDevpanelConf)),
        ]),
        quick_row(&[
            ("/etc/php",   Message::Dashboard(DashboardMessage::NavigatePhpDir)),
            ("/etc/mysql", Message::Dashboard(DashboardMessage::NavigateMysqlDir)),
            ("/etc/hosts", Message::Dashboard(DashboardMessage::NavigateHostsFile)),
        ]),
        quick_row(&[
            ("Web Root",    Message::Dashboard(DashboardMessage::OpenWebRoot)),
            ("php.ini",     Message::Dashboard(DashboardMessage::OpenPhpIni)),
            ("Restart All", Message::Dashboard(DashboardMessage::RestartAll)),
        ]),
    ]
    .spacing(8);

    let failures: Element<Message> = if tab.recent_failures.is_empty() {
        Space::with_height(0).into()
    } else {
        let rows: Vec<Element<Message>> = tab.recent_failures.iter()
            .map(|line| text(line.as_str()).size(11).color(TEXT_MUTED).into())
            .collect();
        container(column![
            text("Recent Apache Failures").size(13).color(TEXT_SECONDARY),
            Space::with_height(8),
            column(rows).spacing(4),
        ].spacing(0))
        .padding(Padding::from([14, 16]))
        .width(Length::Fill)
        .style(card_style(BTN_DANGER))
        .into()
    };

    let content = scrollable(
        column![
            info_bar,
            Space::with_height(20),
            services,
            Space::with_height(if tab.recent_failures.is_empty() { 0 } else { 16 }),
            failures,
            Space::with_height(28),
            text("Quick Actions").size(13).color(TEXT_SECONDARY),
            Space::with_height(12),
            quick_grid,
            Space::with_height(24),
        ]
        .spacing(0)
        .padding(Padding::from([20, 22])),
    );

    if tab.php_info_loading || tab.php_info.is_some() {
        let body = tab.php_info.as_deref().unwrap_or("Loading PHP info...");
        column![
            content,
            container(column![
                row![
                    text("PHP Info").size(18).color(TEXT_PRIMARY),
                    Space::with_width(Length::Fill),
                    button(text("Close").size(12).color(TEXT_MUTED))
                        .on_press(Message::Dashboard(DashboardMessage::ClosePhpInfo))
                        .padding(Padding::from([6, 12]))
                        .style(ghost_btn_style()),
                ].align_y(Alignment::Center),
                Space::with_height(10),
                scrollable(text(body).size(12).color(TEXT_SECONDARY)).height(Length::Fixed(240.0)),
            ].spacing(0))
            .padding(Padding::from([16, 18]))
            .width(Length::Fill)
            .style(card_style(PURPLE_BDR)),
        ].spacing(0).into()
    } else {
        content.into()
    }
}

struct ServiceActions {
    start: Message,
    stop: Message,
    restart: Message,
}

fn service_card<'a>(
    name: &'a str,
    subtitle: &'a str,
    uptime: Option<&'a str>,
    running: bool,
    accent: Color,
    actions: ServiceActions,
) -> Element<'a, Message> {
    let status_color = if running { GREEN } else { STATUS_STOP };
    let status_label = if running { "Running" } else { "Stopped" };
    let status_bg = if running { GREEN_BG } else { STOPPED_BG };
    let accent_pill_bg = if accent == GREEN { GREEN_BG } else { BLUE_BG };

    let top = row![
        container(text(if running { "ON" } else { "OFF" }).size(9).color(accent))
            .padding(Padding::from([4, 8]))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(accent_pill_bg.into()),
                border: Border { radius: 6.0.into(), ..Default::default() },
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
            border: Border { radius: 20.0.into(), ..Default::default() },
            ..Default::default()
        }),
    ]
    .align_y(Alignment::Center);

    let btn_row = row![
        button(text("Start").size(13).width(Length::Fill).center())
            .on_press(actions.start).padding(Padding::from([7, 0]))
            .width(Length::FillPortion(1)).style(btn_style(BTN_SUCCESS)),
        button(text("Stop").size(13).width(Length::Fill).center())
            .on_press(actions.stop).padding(Padding::from([7, 0]))
            .width(Length::FillPortion(1)).style(btn_style(BTN_DANGER)),
        button(text("Restart").size(13).width(Length::Fill).center())
            .on_press(actions.restart).padding(Padding::from([7, 0]))
            .width(Length::FillPortion(1)).style(btn_style(BTN_WARN)),
    ]
    .spacing(7);

    let card_border = if running { GREEN_BG } else { BORDER_SUBTLE };

    container(
        column![
            top,
            Space::with_height(14),
            text(name).size(19).color(TEXT_PRIMARY),
            Space::with_height(3),
            text(subtitle).size(12).color(TEXT_MUTED),
            Space::with_height(6),
            text(uptime.map(|u| format!("Up {}", u)).unwrap_or_else(|| "Uptime n/a".into()))
                .size(11).color(TEXT_MUTED),
            Space::with_height(16),
            thin_line(),
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

fn php_card(tab: &DashboardTab) -> Element<'_, Message> {
    let version_text = tab.active_php_version.as_deref().unwrap_or("n/a");

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
        border: Border { radius: 20.0.into(), ..Default::default() },
        ..Default::default()
    });

    let top = row![
        container(text("PHP").size(9).color(PURPLE))
            .padding(Padding::from([4, 8]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(PURPLE_BG.into()),
                border: Border { radius: 6.0.into(), ..Default::default() },
                ..Default::default()
            }),
        Space::with_width(Length::Fill),
        running_dot,
    ]
    .align_y(Alignment::Center);

    let picker: Element<Message> = if !tab.php_versions.is_empty() {
        pick_list(
            &tab.php_versions[..],
            tab.active_php_version.as_ref(),
            |v| Message::Dashboard(DashboardMessage::SwitchPhpVersion(v)),
        )
        .padding(Padding::from([10, 14]))
        .width(Length::Fill)
        .style(|_theme, status| {
            use iced::widget::pick_list;
            let is_open = matches!(status, pick_list::Status::Opened);
            let border_color = if is_open { PURPLE } else { PURPLE_BDR };
            pick_list::Style {
                text_color: PURPLE,
                placeholder_color: TEXT_MUTED,
                handle_color: PURPLE,
                background: iced::Background::Color(if is_open { PURPLE_BG2 } else { PURPLE_BG }),
                border: Border { color: border_color, width: 1.0, radius: 12.0.into() },
            }
        })
        .into()
    } else {
        container(text("No PHP detected").size(13).color(TEXT_MUTED))
            .padding(Padding::from([9, 12]))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(BG_SURFACE.into()),
                border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
                ..Default::default()
            })
            .into()
    };

    let php_info_btn = button(text("PHP Info").size(13))
        .on_press(Message::Dashboard(DashboardMessage::ShowPhpInfo))
        .padding(Padding::from([7, 14]))
        .style(|_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                iced::widget::button::Style {
                    background: Some(PURPLE_BG2.into()), text_color: PURPLE,
                    border: Border { color: PURPLE_BDR, width: 1.0, radius: 7.0.into() },
                    ..Default::default()
                },
            _ => iced::widget::button::Style {
                background: Some(PURPLE_BG.into()), text_color: PURPLE,
                border: Border { radius: 7.0.into(), ..Default::default() },
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
            thin_line(),
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

fn quick_row<'a>(items: &[(&'a str, Message)]) -> Element<'a, Message> {
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

fn card_style(border_color: Color) -> impl Fn(&iced::Theme) -> container::Style {
    move |_: &iced::Theme| container::Style {
        background: Some(BG_CARD.into()),
        border: Border { color: border_color, width: 1.0, radius: 10.0.into() },
        ..Default::default()
    }
}

fn surface_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_: &iced::Theme| container::Style {
        background: Some(BG_SURFACE.into()),
        border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
        ..Default::default()
    }
}

fn btn_style(bg: Color) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
            iced::widget::button::Style {
                background: Some(Color::from_rgba(bg.r * 0.82, bg.g * 0.82, bg.b * 0.82, 1.0).into()),
                text_color: Color::WHITE,
                border: Border { color: Color::BLACK, width: 1.5, radius: 7.0.into() },
                ..Default::default()
            },
        _ => iced::widget::button::Style {
            background: Some(bg.into()), text_color: Color::WHITE,
            border: Border { color: Color::BLACK, width: 1.5, radius: 7.0.into() },
            ..Default::default()
        },
    }
}

fn ghost_btn_style() -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
            iced::widget::button::Style {
                background: Some(BG_HOVER.into()), text_color: TEXT_PRIMARY,
                border: Border { color: BORDER_MED, width: 1.0, radius: 8.0.into() },
                ..Default::default()
            },
        _ => iced::widget::button::Style {
            background: Some(BG_CARD.into()), text_color: TEXT_SECONDARY,
            border: Border { color: BORDER_SUBTLE, width: 1.0, radius: 8.0.into() },
            ..Default::default()
        },
    }
}

fn thin_line<'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1)).width(Length::Fill).height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_SUBTLE.into()),
            ..Default::default()
        })
}

fn status_dot(color: Color) -> Element<'static, Message> {
    container(Space::with_width(6)).width(6).height(6)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(color.into()),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn sep_vertical() -> Element<'static, Message> {
    container(Space::with_width(1)).width(1).height(12)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_MED.into()),
            ..Default::default()
        })
        .into()
}
