use super::DashboardTab;
use crate::core::paths;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::dashboard as keys, text as tr};
use crate::messages::DashboardMessage;
use crate::messages::Message;
use crate::ui::icons::{self, Icon};
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

pub fn render(tab: &DashboardTab, compact: bool) -> Element<'_, Message> {
    let header_fn = if compact {
        ui::page_header_compact
    } else {
        ui::page_header
    };
    let header = header_fn(
        tr(keys::TITLE),
        tr(keys::SUBTITLE),
        vec![
            ui::secondary_icon_button(
                Icon::Globe,
                tr(keys::OPEN_LOCALHOST),
                Message::Dashboard(DashboardMessage::OpenLocalhost),
            ),
            ui::primary_icon_button(
                Icon::Refresh,
                tr(keys::RESTART_ALL),
                Message::Dashboard(DashboardMessage::RestartAll),
            ),
        ],
    );

    let env_metrics: Element<Message> = if compact {
        column![
            meta_item(Icon::Server, "OS", &tab.distro),
            meta_item(Icon::Folder, tr(keys::WEB_ROOT), &tab.web_root),
            meta_item(Icon::Apache, tr(keys::APACHE), &tab.apache_conf_dir),
            meta_item(
                Icon::Php,
                tr(keys::ACTIVE_PHP),
                tab.active_php_version
                    .as_deref()
                    .unwrap_or(tr(keys::NOT_AVAILABLE_SHORT)),
            ),
        ]
        .spacing(10)
        .into()
    } else {
        row![
            meta_item(Icon::Server, "OS", &tab.distro),
            meta_item(Icon::Folder, tr(keys::WEB_ROOT), &tab.web_root),
            meta_item(Icon::Apache, tr(keys::APACHE), &tab.apache_conf_dir),
            meta_item(
                Icon::Php,
                tr(keys::ACTIVE_PHP),
                tab.active_php_version
                    .as_deref()
                    .unwrap_or(tr(keys::NOT_AVAILABLE_SHORT)),
            ),
        ]
        .spacing(16)
        .align_y(Alignment::Start)
        .into()
    };

    let environment = container(
        column![
            text(tr(keys::ENVIRONMENT))
                .size(13)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
            Space::with_height(10),
            env_metrics,
        ]
        .spacing(0),
    )
    .padding(Padding::from([16, 18]))
    .width(Length::Fill)
    .style(ui::card_style());

    let apache_card = service_card(
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
    );
    let mysql_card = service_card(
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
    );
    let php_card = php_card(tab);
    let services: Element<Message> = if compact {
        column![apache_card, mysql_card, php_card]
            .spacing(10)
            .into()
    } else {
        row![apache_card, mysql_card, php_card].spacing(12).into()
    };

    let quick_grid = column![
        quick_group(
            tr(keys::GROUP_OPEN),
            &[
                QuickAction {
                    icon: Icon::Globe,
                    label: tr(keys::LOCALHOST),
                    meta: "http://localhost",
                    msg: Message::Dashboard(DashboardMessage::OpenLocalhost),
                },
                QuickAction {
                    icon: Icon::Database,
                    label: tr(keys::PHPMYADMIN),
                    meta: "http://localhost/phpmyadmin",
                    msg: Message::Dashboard(DashboardMessage::OpenPhpMyAdmin),
                },
                QuickAction {
                    icon: Icon::Folder,
                    label: tr(keys::PROJECTS),
                    meta: tab.web_root.as_str(),
                    msg: Message::Dashboard(DashboardMessage::OpenProjectsFolder),
                },
            ],
        ),
        Space::with_height(14),
        quick_group(
            tr(keys::GROUP_CONFIGURATION),
            &[
                QuickAction {
                    icon: Icon::Apache,
                    label: tr(keys::APACHE_CONFIG),
                    meta: paths::APACHE_CONF_DIR,
                    msg: Message::Dashboard(DashboardMessage::NavigateApache2Conf),
                },
                QuickAction {
                    icon: Icon::Host,
                    label: tr(keys::SITES_AVAILABLE),
                    meta: paths::APACHE_SITES_AVAILABLE,
                    msg: Message::Dashboard(DashboardMessage::NavigateApache2Sites),
                },
                QuickAction {
                    icon: Icon::Config,
                    label: tr(keys::DEVPANEL_CONFIG),
                    meta: paths::DEVPANEL_CONF,
                    msg: Message::VHosts(crate::messages::VHostsMessage::OpenDevpanelConf),
                },
            ],
        ),
        Space::with_height(14),
        quick_group(
            tr(keys::GROUP_SYSTEM),
            &[
                QuickAction {
                    icon: Icon::Php,
                    label: paths::PHP_ETC_DIR,
                    meta: paths::PHP_ETC_DIR,
                    msg: Message::Dashboard(DashboardMessage::NavigatePhpDir),
                },
                QuickAction {
                    icon: Icon::Database,
                    label: paths::MYSQL_ETC_DIR,
                    meta: paths::MYSQL_ETC_DIR,
                    msg: Message::Dashboard(DashboardMessage::NavigateMysqlDir),
                },
                QuickAction {
                    icon: Icon::Code,
                    label: paths::HOSTS_FILE,
                    meta: paths::HOSTS_FILE,
                    msg: Message::Dashboard(DashboardMessage::NavigateHostsFile),
                },
            ],
        ),
        Space::with_height(8),
        quick_row(&[
            QuickAction {
                icon: Icon::Folder,
                label: tr(keys::WEB_ROOT),
                meta: tab.web_root.as_str(),
                msg: Message::Dashboard(DashboardMessage::OpenWebRoot),
            },
            QuickAction {
                icon: Icon::Php,
                label: tr(keys::PHP_INI),
                meta: paths::PHP_ETC_DIR,
                msg: Message::Dashboard(DashboardMessage::OpenPhpIni),
            },
            QuickAction {
                icon: Icon::Refresh,
                label: tr(keys::RESTART_ALL),
                meta: "Apache + MySQL",
                msg: Message::Dashboard(DashboardMessage::RestartAll),
            },
        ]),
    ]
    .spacing(0);

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
        .style(ui::card_style_with_border(theme::color(
            theme_keys::BTN_DANGER,
        )))
        .into()
    };

    let content = scrollable(
        column![
            header,
            Space::with_height(18),
            environment,
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
                        .style(ui::ghost_button_style()),
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
            .style(ui::card_style_with_border(theme::color(
                theme_keys::PURPLE_BORDER
            ))),
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
                ui::status_dot(status_color),
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
        button(
            row![
                icons::solid_box(Icon::Plus, 11.0, theme::color(theme_keys::WHITE), 13.0),
                Space::with_width(6),
                text(tr(keys::START)).size(13)
            ]
            .align_y(Alignment::Center)
        )
        .on_press(actions.start)
        .padding(Padding::from([7, 0]))
        .width(Length::FillPortion(1))
        .style(btn_style(theme::color(theme_keys::BTN_SUCCESS))),
        button(
            row![
                icons::solid_box(Icon::Stop, 10.0, theme::color(theme_keys::WHITE), 13.0),
                Space::with_width(6),
                text(tr(keys::STOP)).size(13)
            ]
            .align_y(Alignment::Center)
        )
        .on_press(actions.stop)
        .padding(Padding::from([7, 0]))
        .width(Length::FillPortion(1))
        .style(btn_style(theme::color(theme_keys::BTN_DANGER))),
        button(
            row![
                icons::solid_box(Icon::Refresh, 11.0, theme::color(theme_keys::WHITE), 13.0),
                Space::with_width(6),
                text(tr(keys::RESTART)).size(13)
            ]
            .align_y(Alignment::Center)
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
            ui::thin_line(),
            Space::with_height(14),
            btn_row,
        ]
        .spacing(0),
    )
    .padding(Padding::from([18, 18]))
    .width(Length::FillPortion(1))
    .style(ui::card_style_with_border(card_border))
    .into()
}

fn php_card(tab: &DashboardTab) -> Element<'_, Message> {
    let version_text = tab
        .active_php_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_AVAILABLE_SHORT));

    let running_dot = container(
        row![
            ui::status_dot(theme::color(theme_keys::PURPLE)),
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
        ui::dropdown(
            &tab.php_versions[..],
            tab.active_php_version.as_ref(),
            |v| Message::Dashboard(DashboardMessage::SwitchPhpVersion(v)),
        )
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
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
    };

    let php_info_btn = ui::secondary_icon_button(
        Icon::Info,
        tr(keys::PHP_INFO),
        Message::Dashboard(DashboardMessage::ShowPhpInfo),
    );
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
            ui::thin_line(),
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
    .style(ui::card_style_with_border(theme::color(
        theme_keys::BORDER_SUBTLE,
    )))
    .into()
}

struct QuickAction<'a> {
    icon: Icon,
    label: &'a str,
    meta: &'a str,
    msg: Message,
}

fn quick_row<'a>(items: &[QuickAction<'a>]) -> Element<'a, Message> {
    let btns: Vec<Element<Message>> =
        items
            .iter()
            .map(|item| {
                button(
                    row![
                        container(Space::with_width(3)).width(3).height(34).style(
                            |_: &iced::Theme| container::Style {
                                background: Some(theme::color(theme_keys::TEAL).into()),
                                border: Border {
                                    radius: 2.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        ),
                        Space::with_width(12),
                        icons::solid(item.icon, 14.0, theme::color(theme_keys::TEXT_SECONDARY)),
                        Space::with_width(10),
                        column![
                            text(item.label)
                                .size(13)
                                .color(theme::color(theme_keys::TEXT_PRIMARY)),
                            Space::with_height(3),
                            text(item.meta)
                                .size(10)
                                .color(theme::color(theme_keys::TEXT_MUTED)),
                        ]
                        .spacing(0)
                        .width(Length::Fill),
                    ]
                    .align_y(Alignment::Center),
                )
                .on_press(item.msg.clone())
                .padding(Padding::from([10, 12]))
                .width(Length::FillPortion(1))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::BG_HOVER).into()),
                        text_color: theme::color(theme_keys::TEXT_PRIMARY),
                        border: Border {
                            color: theme::color(theme_keys::BORDER_MED),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::BG_CARD).into()),
                        text_color: theme::color(theme_keys::TEXT_SECONDARY),
                        border: Border {
                            color: theme::color(theme_keys::BORDER_SUBTLE),
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    },
                })
                .into()
            })
            .collect();
    row(btns).spacing(8).into()
}

fn quick_group<'a>(title: &'a str, items: &[QuickAction<'a>]) -> Element<'a, Message> {
    column![
        text(title)
            .size(11)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_height(8),
        quick_row(items),
    ]
    .spacing(0)
    .into()
}

fn meta_item<'a>(icon: Icon, label: &'a str, value: &'a str) -> Element<'a, Message> {
    row![
        icons::solid(icon, 15.0, theme::color(theme_keys::TEXT_MUTED)),
        Space::with_width(9),
        column![
            text(label)
                .size(10)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(3),
            text(value)
                .size(12)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
        ]
        .spacing(0),
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .width(Length::FillPortion(1))
    .into()
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
                    radius: 6.0.into(),
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
                radius: 6.0.into(),
            },
            ..Default::default()
        },
    }
}
