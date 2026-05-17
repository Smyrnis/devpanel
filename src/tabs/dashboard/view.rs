use super::DashboardTab;
use crate::core::paths;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::dashboard as keys, text as tr};
use crate::messages::DashboardMessage;
use crate::messages::Message;
use iced::widget::{Space, button, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub fn render(tab: &DashboardTab) -> Element<'_, Message> {
    let info_bar = container(
        row![
            status_dot(theme::color(theme_keys::TEAL)),
            Space::with_width(8),
            text(&tab.distro)
                .size(12)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_width(16),
            sep_vertical(),
            Space::with_width(16),
            text(tr(keys::WEB_ROOT))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_width(6),
            text(&tab.web_root)
                .size(12)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_width(16),
            sep_vertical(),
            Space::with_width(16),
            text(tr(keys::APACHE))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_width(6),
            text(&tab.apache_conf_dir)
                .size(12)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([11, 18]))
    .width(Length::Fill)
    .style(surface_style());

    let services = row![
        service_card(
            tr(keys::APACHE),
            tr(keys::APACHE_SUBTITLE),
            tab.apache_uptime.as_deref(),
            tab.apache_running,
            theme::color(theme_keys::GREEN),
            ServiceActions {
                start: Message::Dashboard(DashboardMessage::StartApache),
                stop: Message::Dashboard(DashboardMessage::StopApache),
                restart: Message::Dashboard(DashboardMessage::RestartApache),
            },
        ),
        service_card(
            tr(keys::MYSQL),
            tr(keys::MYSQL_SUBTITLE),
            tab.mysql_uptime.as_deref(),
            tab.mysql_running,
            theme::color(theme_keys::BLUE),
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
            (
                tr(keys::LOCALHOST),
                Message::Dashboard(DashboardMessage::OpenLocalhost)
            ),
            (
                tr(keys::PHPMYADMIN),
                Message::Dashboard(DashboardMessage::OpenPhpMyAdmin)
            ),
            (
                tr(keys::PROJECTS),
                Message::Dashboard(DashboardMessage::OpenProjectsFolder)
            ),
        ]),
        quick_row(&[
            (
                tr(keys::APACHE_CONFIG),
                Message::Dashboard(DashboardMessage::NavigateApache2Conf)
            ),
            (
                tr(keys::SITES_AVAILABLE),
                Message::Dashboard(DashboardMessage::NavigateApache2Sites)
            ),
            (
                tr(keys::DEVPANEL_CONFIG),
                Message::VHosts(crate::messages::VHostsMessage::OpenDevpanelConf)
            ),
        ]),
        quick_row(&[
            (
                paths::PHP_ETC_DIR,
                Message::Dashboard(DashboardMessage::NavigatePhpDir)
            ),
            (
                paths::MYSQL_ETC_DIR,
                Message::Dashboard(DashboardMessage::NavigateMysqlDir)
            ),
            (
                paths::HOSTS_FILE,
                Message::Dashboard(DashboardMessage::NavigateHostsFile)
            ),
        ]),
        quick_row(&[
            (
                tr(keys::WEB_ROOT),
                Message::Dashboard(DashboardMessage::OpenWebRoot)
            ),
            (
                tr(keys::PHP_INI),
                Message::Dashboard(DashboardMessage::OpenPhpIni)
            ),
            (
                tr(keys::RESTART_ALL),
                Message::Dashboard(DashboardMessage::RestartAll)
            ),
        ]),
    ]
    .spacing(8);

    let failures: Element<Message> = if tab.recent_failures.is_empty() {
        Space::with_height(0).into()
    } else {
        let rows: Vec<Element<Message>> = tab
            .recent_failures
            .iter()
            .map(|line| {
                text(line.as_str())
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED))
                    .into()
            })
            .collect();
        container(
            column![
                text(tr(keys::RECENT_FAILURES))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_SECONDARY)),
                Space::with_height(8),
                column(rows).spacing(4),
            ]
            .spacing(0),
        )
        .padding(Padding::from([14, 16]))
        .width(Length::Fill)
        .style(card_style(theme::color(theme_keys::BTN_DANGER)))
        .into()
    };

    let content = scrollable(
        column![
            info_bar,
            Space::with_height(20),
            services,
            Space::with_height(if tab.recent_failures.is_empty() {
                0
            } else {
                16
            }),
            failures,
            Space::with_height(28),
            text(tr(keys::QUICK_ACTIONS))
                .size(13)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(12),
            quick_grid,
            Space::with_height(24),
        ]
        .spacing(0)
        .padding(Padding::from([20, 22])),
    );

    if tab.php_info_loading || tab.php_info.is_some() {
        let body = tab
            .php_info
            .as_deref()
            .unwrap_or(tr(keys::LOADING_PHP_INFO));
        column![
            content,
            container(
                column![
                    row![
                        text(tr(keys::PHP_INFO))
                            .size(18)
                            .color(theme::color(theme_keys::TEXT_PRIMARY)),
                        Space::with_width(Length::Fill),
                        button(
                            text(tr(keys::CLOSE))
                                .size(12)
                                .color(theme::color(theme_keys::TEXT_MUTED))
                        )
                        .on_press(Message::Dashboard(DashboardMessage::ClosePhpInfo))
                        .padding(Padding::from([6, 12]))
                        .style(ghost_btn_style()),
                    ]
                    .align_y(Alignment::Center),
                    Space::with_height(10),
                    scrollable(
                        text(body)
                            .size(12)
                            .color(theme::color(theme_keys::TEXT_SECONDARY))
                    )
                    .height(Length::Fixed(240.0)),
                ]
                .spacing(0)
            )
            .padding(Padding::from([16, 18]))
            .width(Length::Fill)
            .style(card_style(theme::color(theme_keys::PURPLE_BORDER))),
        ]
        .spacing(0)
        .into()
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
    let status_color = if running {
        theme::color(theme_keys::GREEN)
    } else {
        theme::color(theme_keys::TEXT_MUTED)
    };
    let status_label = if running {
        tr(keys::RUNNING)
    } else {
        tr(keys::STOPPED)
    };
    let status_bg = if running {
        theme::color(theme_keys::GREEN_BG)
    } else {
        theme::color(theme_keys::STOPPED_BG)
    };
    let accent_pill_bg = if accent == theme::color(theme_keys::GREEN) {
        theme::color(theme_keys::GREEN_BG)
    } else {
        theme::color(theme_keys::BLUE_BG)
    };

    let top = row![
        container(
            text(if running { tr(keys::ON) } else { tr(keys::OFF) })
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

    let btn_row = row![
        button(text(tr(keys::START)).size(13).width(Length::Fill).center())
            .on_press(actions.start)
            .padding(Padding::from([7, 0]))
            .width(Length::FillPortion(1))
            .style(btn_style(theme::color(theme_keys::BTN_SUCCESS))),
        button(text(tr(keys::STOP)).size(13).width(Length::Fill).center())
            .on_press(actions.stop)
            .padding(Padding::from([7, 0]))
            .width(Length::FillPortion(1))
            .style(btn_style(theme::color(theme_keys::BTN_DANGER))),
        button(
            text(tr(keys::RESTART))
                .size(13)
                .width(Length::Fill)
                .center()
        )
        .on_press(actions.restart)
        .padding(Padding::from([7, 0]))
        .width(Length::FillPortion(1))
        .style(btn_style(theme::color(theme_keys::BTN_WARN))),
    ]
    .spacing(7);

    let card_border = if running {
        theme::color(theme_keys::GREEN_BG)
    } else {
        theme::color(theme_keys::BORDER_SUBTLE)
    };

    container(
        column![
            top,
            Space::with_height(14),
            text(name)
                .size(19)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(3),
            text(subtitle)
                .size(12)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(6),
            text(
                uptime
                    .map(|u| format!("{} {}", tr(keys::UPTIME_PREFIX), u))
                    .unwrap_or_else(|| tr(keys::UPTIME_UNKNOWN).into())
            )
            .size(11)
            .color(theme::color(theme_keys::TEXT_MUTED)),
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
    let version_text = tab
        .active_php_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_AVAILABLE_SHORT));

    let running_dot = container(
        row![
            status_dot(theme::color(theme_keys::PURPLE)),
            Space::with_width(5),
            text(version_text)
                .size(11)
                .color(theme::color(theme_keys::PURPLE)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([4, 9]))
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::PURPLE_BG).into()),
        border: Border {
            radius: 20.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let top = row![
        container(
            text(tr(keys::PHP))
                .size(9)
                .color(theme::color(theme_keys::PURPLE))
        )
        .padding(Padding::from([4, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::PURPLE_BG).into()),
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
            let border_color = if is_open {
                theme::color(theme_keys::PURPLE)
            } else {
                theme::color(theme_keys::PURPLE_BORDER)
            };
            pick_list::Style {
                text_color: theme::color(theme_keys::PURPLE),
                placeholder_color: theme::color(theme_keys::TEXT_MUTED),
                handle_color: theme::color(theme_keys::PURPLE),
                background: iced::Background::Color(if is_open {
                    theme::color(theme_keys::PURPLE_HOVER)
                } else {
                    theme::color(theme_keys::PURPLE_BG)
                }),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 12.0.into(),
                },
            }
        })
        .into()
    } else {
        container(
            text(tr(keys::PHP_NOT_DETECTED))
                .size(13)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        )
        .padding(Padding::from([9, 12]))
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
    };

    let php_info_btn = button(text(tr(keys::PHP_INFO)).size(13))
        .on_press(Message::Dashboard(DashboardMessage::ShowPhpInfo))
        .padding(Padding::from([7, 14]))
        .style(|_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::PURPLE_HOVER).into()),
                    text_color: theme::color(theme_keys::PURPLE),
                    border: Border {
                        color: theme::color(theme_keys::PURPLE_BORDER),
                        width: 1.0,
                        radius: 7.0.into(),
                    },
                    ..Default::default()
                }
            }
            _ => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::PURPLE_BG).into()),
                text_color: theme::color(theme_keys::PURPLE),
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
            text(tr(keys::PHP_ENGINE))
                .size(19)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(3),
            text(tr(keys::VERSION_SWITCHER))
                .size(12)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(16),
            thin_line(),
            Space::with_height(14),
            text(tr(keys::ACTIVE_VERSION))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(6),
            picker,
            Space::with_height(8),
            php_info_btn,
        ]
        .spacing(0),
    )
    .padding(Padding::from([18, 18]))
    .width(Length::FillPortion(1))
    .style(card_style(theme::color(theme_keys::BORDER_SUBTLE)))
    .into()
}

fn quick_row<'a>(items: &[(&'a str, Message)]) -> Element<'a, Message> {
    let btns: Vec<Element<Message>> = items
        .iter()
        .map(|(label, msg)| {
            button(
                text(*label)
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_PRIMARY)),
            )
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
        background: Some(theme::color(theme_keys::BG_CARD).into()),
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
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: theme::color(theme_keys::BORDER_SUBTLE),
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
                text_color: theme::color(theme_keys::WHITE),
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
            text_color: theme::color(theme_keys::WHITE),
            border: Border {
                color: Color::BLACK,
                width: 1.5,
                radius: 7.0.into(),
            },
            ..Default::default()
        },
    }
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

fn thin_line<'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BORDER_SUBTLE).into()),
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
            background: Some(theme::color(theme_keys::BORDER_MED).into()),
            ..Default::default()
        })
        .into()
}
