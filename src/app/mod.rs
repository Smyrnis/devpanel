mod handlers;
pub mod update;

use crate::core::{
    config::DevPanelConfig,
    db::{DevPanelDb, UserSettings},
    first_run::FirstRunState,
    sudo_prompt::SudoModal,
    theme::{self, theme_map as theme_keys},
};
use crate::lang::{lang_map::app as keys, text as tr};
use crate::messages::{Message, SudoMessage, Tab};
use crate::ui::tabs::{
    config::ConfigTab, dashboard::DashboardTab, repos::ReposTab, ssh_keys::SshKeysTab,
    tools::ToolsTab, vhosts::VHostsTab,
};
use crate::ui::templates::prelude as ui;

use iced::widget::{Space, button, column, container, row, stack, text};
use iced::{Alignment, Border, Color, Element, Length, Padding, Task};

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub ok: bool,
    pub remaining_ms: u32,
}

pub struct App {
    pub active_tab: Tab,
    pub config: DevPanelConfig,
    pub db: Option<DevPanelDb>,
    pub dashboard: DashboardTab,
    pub ssh_keys: SshKeysTab,
    pub tools: ToolsTab,
    pub repos: ReposTab,
    pub vhosts: VHostsTab,
    pub config_tab: ConfigTab,
    pub notifications: Vec<Toast>,
    pub sudo: SudoModal,
    pub first_run_state: FirstRunState,
    pub first_run_options: crate::core::first_run_install::FirstRunInstallOptions,
    pub first_run_installing: bool,
    pub first_run_log_lines: Vec<String>,
    pub setup_issues_checked: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let config = DevPanelConfig::load();
        let db = DevPanelDb::open().ok();
        let user_settings = match &db {
            Some(d) => UserSettings::load(d),
            None => UserSettings::default(),
        };

        let app = Self {
            repos: ReposTab::new(config.repos_root.clone(), config.repos_root.clone()),
            vhosts: VHostsTab::new(config.devpanel_conf.clone()),
            config_tab: ConfigTab::new(user_settings),
            active_tab: Tab::Dashboard,
            dashboard: DashboardTab::new(),
            ssh_keys: SshKeysTab::new(),
            tools: ToolsTab::new(),
            config,
            db,
            notifications: Vec::new(),
            sudo: SudoModal::new(),
            first_run_state: FirstRunState::default(),
            first_run_options: crate::core::first_run_install::FirstRunInstallOptions::default(),
            first_run_installing: false,
            first_run_log_lines: Vec::new(),
            setup_issues_checked: false,
        };
        (
            app,
            Task::perform(crate::ui::tabs::dashboard::probe_services(), |r| r),
        )
    }

    pub fn show_toast(&mut self, message: String, ok: bool) -> Task<Message> {
        let remaining_ms = self.config_tab.settings.ui_toast_duration_ms.max(1000);
        if let Some(db) = &self.db {
            let _ = db.add_notification(ok, &message);
        }
        self.notifications.push(Toast {
            message,
            ok,
            remaining_ms,
        });
        Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            iced::time::every(std::time::Duration::from_secs(5))
                .map(|_| Message::Dashboard(crate::messages::DashboardMessage::AutoRefreshTick)),
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::NotificationTick),
            iced::time::every(std::time::Duration::from_secs(1))
                .map(|_| Message::FirstRun(crate::messages::FirstRunMessage::ProgressTick)),
            crate::core::file_watcher::vhost_config(self.vhosts.devpanel_conf.clone()),
        ])
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.first_run_state == FirstRunState::Visible {
            return crate::ui::install_window::view(
                self.first_run_options,
                self.first_run_installing,
                &self.first_run_log_lines,
            );
        }

        let tab_content: Element<Message> = match &self.active_tab {
            Tab::Dashboard => self.dashboard.view(),
            Tab::SshKeys => self.ssh_keys.view(),
            Tab::Tools => self.tools.view(),
            Tab::Repos => self.repos.view(),
            Tab::VHosts => self.vhosts.view(),
            Tab::Config => self.config_tab.view(),
        };

        let content_area = column![
            self.context_bar(),
            container(tab_content).width(Length::Fill)
        ]
        .height(Length::Fill)
        .spacing(0);

        let app_area = row![
            self.sidebar(),
            container(content_area)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(theme::color(theme_keys::BG_BASE).into()),
                    ..Default::default()
                }),
        ];

        let base = stack![
            container(app_area).width(Length::Fill).height(Length::Fill),
            self.notification_overlay(),
        ];

        if self.sudo.is_visible() {
            stack![base, self.sudo.view()].into()
        } else {
            base.into()
        }
    }

    fn notification_overlay(&self) -> Element<'_, Message> {
        if self.notifications.is_empty() {
            return Space::with_height(0).into();
        }

        let mut cards: Vec<Element<Message>> = self
            .notifications
            .iter()
            .rev()
            .take(3)
            .map(notification_card)
            .collect();
        cards.insert(
            0,
            button(
                text(tr(keys::DISMISS_ALL))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            )
            .on_press(Message::DismissAllNotifications)
            .padding(Padding::from([6, 12]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::BG_HOVER).into()),
                    text_color: theme::color(theme_keys::TEXT_PRIMARY),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                _ => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::BG_SURFACE).into()),
                    text_color: theme::color(theme_keys::TEXT_MUTED),
                    border: Border {
                        color: theme::color(theme_keys::BORDER_SUBTLE),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                },
            })
            .into(),
        );

        container(
            column![
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(Length::Fill),
                    column(cards).spacing(8).width(340),
                ],
            ]
            .padding(Padding::from([20, 20])),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn context_bar(&self) -> Element<'_, Message> {
        container(
            row![
                context_item(
                    tr(keys::ENVIRONMENT),
                    self.dashboard.distro.as_str(),
                    theme::color(theme_keys::TEAL),
                ),
                context_separator(),
                context_item(
                    tr(keys::WEB_ROOT),
                    self.dashboard.web_root.as_str(),
                    theme::color(theme_keys::TEXT_SECONDARY),
                ),
                context_separator(),
                context_item(
                    tr(keys::APACHE),
                    self.dashboard.apache_conf_dir.as_str(),
                    theme::color(theme_keys::BLUE),
                ),
                context_separator(),
                context_item(
                    tr(keys::PHP),
                    self.dashboard
                        .active_php_version
                        .as_deref()
                        .unwrap_or(tr(keys::NOT_AVAILABLE_SHORT)),
                    theme::color(theme_keys::PURPLE),
                ),
                Space::with_width(Length::Fill),
                button(
                    text(tr(keys::REFRESH))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                )
                .on_press(Message::Dashboard(
                    crate::messages::DashboardMessage::RefreshStatus
                ))
                .padding(Padding::from([6, 12]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::BG_HOVER).into()),
                        text_color: theme::color(theme_keys::TEXT_PRIMARY),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: None,
                        text_color: theme::color(theme_keys::TEXT_MUTED),
                        ..Default::default()
                    },
                }),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 18]))
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let logo = container(
            column![
                row![
                    container(Space::with_width(3))
                        .width(3)
                        .height(26)
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(theme::color(theme_keys::TEAL).into()),
                            border: Border {
                                radius: 2.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    Space::with_width(10),
                    column![
                        text(tr(keys::LOGO_DEV))
                            .size(19)
                            .color(theme::color(theme_keys::TEAL)),
                        text(tr(keys::LOGO_PANEL))
                            .size(19)
                            .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    ]
                    .spacing(0),
                ]
                .align_y(Alignment::Center),
                Space::with_height(10),
                container(
                    text(tr(keys::LOCAL_ENVIRONMENT))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                )
                .padding(Padding::from([4, 10]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(theme::color(theme_keys::BG_CARD).into()),
                    border: Border {
                        color: theme::color(theme_keys::BORDER_SUBTLE),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }),
            ]
            .spacing(0)
            .align_x(Alignment::Start),
        )
        .padding(Padding::from([22, 16]));

        let nav = column![
            text(tr(keys::NAVIGATION))
                .size(10)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(6),
            self.nav_item(tr(keys::NAV_DASHBOARD), Tab::Dashboard),
            self.nav_item(tr(keys::NAV_REPOS), Tab::Repos),
            self.nav_item(tr(keys::NAV_VHOSTS), Tab::VHosts),
            self.nav_item(tr(keys::NAV_SSH_KEYS), Tab::SshKeys),
            self.nav_item(tr(keys::NAV_TOOLS), Tab::Tools),
            self.nav_item(tr(keys::NAV_CONFIG), Tab::Config),
        ]
        .spacing(3)
        .padding(Padding::from([0, 12]));

        let sudo_indicator: Element<Message> = if self.sudo.cached_password.is_some() {
            column![
                container(
                    row![
                        container(Space::with_width(6)).width(6).height(6).style(
                            |_: &iced::Theme| container::Style {
                                background: Some(theme::color(theme_keys::GREEN).into()),
                                border: Border {
                                    radius: 3.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        ),
                        Space::with_width(7),
                        text(tr(keys::SUDO_ACTIVE))
                            .size(11)
                            .color(theme::color(theme_keys::GREEN)),
                    ]
                    .align_y(Alignment::Center)
                )
                .padding(Padding::from([6, 10]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(
                        Color {
                            r: 0.050,
                            g: 0.160,
                            b: 0.090,
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
                Space::with_height(5),
                button(
                    text(tr(keys::CLEAR_SUDO))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED))
                )
                .on_press(Message::Sudo(SudoMessage::ClearSaved))
                .padding(Padding::from([4, 10]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::BG_HOVER).into()),
                        text_color: theme::color(theme_keys::RED),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: None,
                        text_color: theme::color(theme_keys::TEXT_MUTED),
                        ..Default::default()
                    },
                }),
            ]
            .spacing(0)
            .into()
        } else {
            container(
                row![
                    container(Space::with_width(6))
                        .width(6)
                        .height(6)
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(theme::color(theme_keys::YELLOW).into()),
                            border: Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    Space::with_width(7),
                    text(tr(keys::SUDO_LOCKED))
                        .size(11)
                        .color(theme::color(theme_keys::TEXT_MUTED)),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([6, 10]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(
                    Color {
                        r: 0.190,
                        g: 0.160,
                        b: 0.040,
                        a: 1.0,
                    }
                    .into(),
                ),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
        };

        let bottom = container(
            column![
                text(tr(keys::SYSTEM))
                    .size(10)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(6),
                sidebar_status_row(tr(keys::APACHE), self.dashboard.apache_running),
                Space::with_height(5),
                sidebar_status_row("MySQL", self.dashboard.mysql_running),
                Space::with_height(10),
                sudo_indicator,
                Space::with_height(10),
                button(
                    row![
                        text("R")
                            .size(11)
                            .color(theme::color(theme_keys::TEXT_MUTED)),
                        Space::with_width(6),
                        text(tr(keys::REFRESH))
                            .size(12)
                            .color(theme::color(theme_keys::TEXT_MUTED)),
                    ]
                    .align_y(Alignment::Center)
                )
                .on_press(Message::Dashboard(
                    crate::messages::DashboardMessage::RefreshStatus
                ))
                .padding(Padding::from([8, 12]))
                .width(Length::Fill)
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::BG_HOVER).into()),
                        text_color: theme::color(theme_keys::TEXT_PRIMARY),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: None,
                        text_color: theme::color(theme_keys::TEXT_MUTED),
                        ..Default::default()
                    },
                }),
                Space::with_height(8),
                text(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .align_x(Alignment::Start),
        )
        .padding(Padding::from([10, 14]));

        container(
            column![
                logo,
                ui::divider(),
                Space::with_height(10),
                nav,
                Space::with_height(Length::Fill),
                ui::divider(),
                bottom,
            ]
            .height(Length::Fill),
        )
        .width(192)
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            ..Default::default()
        })
        .into()
    }

    fn nav_item<'a>(&self, label: &'a str, tab: Tab) -> Element<'a, Message> {
        let active = self.active_tab == tab;
        let bg = if active {
            theme::color(theme_keys::TEAL_BG)
        } else {
            Color::TRANSPARENT
        };
        let text_color = if active {
            theme::color(theme_keys::TEXT_PRIMARY)
        } else {
            theme::color(theme_keys::TEXT_SECONDARY)
        };
        let accent =
            container(Space::with_width(3))
                .width(3)
                .height(22)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(
                        if active {
                            theme::color(theme_keys::TEAL)
                        } else {
                            Color::TRANSPARENT
                        }
                        .into(),
                    ),
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

        let content = row![
            accent,
            Space::with_width(12),
            text(label).size(13).color(text_color),
        ]
        .align_y(Alignment::Center);

        button(row![content, Space::with_width(Length::Fill),].align_y(Alignment::Center))
            .on_press(Message::SelectTab(tab))
            .padding(Padding::from([8, 10]))
            .width(Length::Fill)
            .style(move |_, status| match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::BG_HOVER).into()),
                    text_color: theme::color(theme_keys::TEXT_PRIMARY),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                _ => iced::widget::button::Style {
                    background: Some(bg.into()),
                    text_color,
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            })
            .into()
    }
}

fn sidebar_status_row<'a>(label: &'a str, running: bool) -> Element<'a, Message> {
    let color = if running {
        theme::color(theme_keys::GREEN)
    } else {
        theme::color(theme_keys::TEXT_MUTED)
    };
    row![
        ui::status_dot(color),
        Space::with_width(7),
        text(label)
            .size(11)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_width(Length::Fill),
        text(if running { tr(keys::ON) } else { tr(keys::OFF) })
            .size(10)
            .color(color),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn context_item<'a>(label: &'a str, value: &'a str, accent: Color) -> Element<'a, Message> {
    row![
        ui::status_dot(accent),
        Space::with_width(7),
        text(label)
            .size(10)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_width(5),
        text(value)
            .size(11)
            .color(theme::color(theme_keys::TEXT_SECONDARY)),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn context_separator<'a>() -> Element<'a, Message> {
    container(Space::with_width(1))
        .width(1)
        .height(18)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BORDER_SUBTLE).into()),
            ..Default::default()
        })
        .into()
}

fn notification_card(toast: &Toast) -> Element<'_, Message> {
    let (accent, border_color) = if toast.ok {
        (
            theme::color(theme_keys::GREEN),
            Color {
                r: 0.070,
                g: 0.210,
                b: 0.110,
                a: 1.0,
            },
        )
    } else {
        (
            theme::color(theme_keys::RED),
            Color {
                r: 0.300,
                g: 0.090,
                b: 0.080,
                a: 1.0,
            },
        )
    };
    let seconds = (toast.remaining_ms / 1000).max(1);
    container(
        row![
            container(
                text(if toast.ok { "+" } else { "x" })
                    .size(11)
                    .color(theme::color(theme_keys::WHITE))
            )
            .padding(Padding::from([3, 7]))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(accent.into()),
                border: Border {
                    radius: 20.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(10),
            text(toast.message.as_str())
                .size(13)
                .color(theme::color(theme_keys::TEXT_PRIMARY))
                .width(Length::Fill),
            text(format!("{}s", seconds))
                .size(10)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 14]))
    .width(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BG_SURFACE).into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow {
            color: Color {
                a: 0.4,
                ..Color::BLACK
            },
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    })
    .into()
}
