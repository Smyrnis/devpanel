pub mod update;

use crate::core::{
    config::DevPanelConfig,
    first_run::{self, FirstRunState},
    sudo_prompt::{PendingAction, SudoModal},
    theme::*,
};
use crate::messages::{Message, SudoMessage, Tab};
use crate::tabs::{
    dashboard::DashboardTab, repos::ReposTab, ssh_keys::SshKeysTab, tools::ToolsTab,
    vhosts::VHostsTab,
};

use iced::widget::{Space, button, column, container, row, stack, text};
use iced::{Alignment, Border, Color, Element, Length, Padding, Task};

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub ok: bool,
}

pub struct App {
    pub active_tab: Tab,
    pub config: DevPanelConfig,
    pub dashboard: DashboardTab,
    pub ssh_keys: SshKeysTab,
    pub tools: ToolsTab,
    pub repos: ReposTab,
    pub vhosts: VHostsTab,
    pub toast: Option<Toast>,
    pub sudo: SudoModal,
    pub sudo_pending_action: Option<PendingAction>,
    pub first_run_state: FirstRunState,
    pub setup_issues_checked: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let config = DevPanelConfig::load();
        let app = Self {
            repos: ReposTab::new(config.repos_root.clone(), config.repos_root.clone()),
            vhosts: VHostsTab::new(config.devpanel_conf.clone()),
            active_tab: Tab::Dashboard,
            dashboard: DashboardTab::new(),
            ssh_keys: SshKeysTab::new(),
            tools: ToolsTab::new(),
            config,
            toast: None,
            sudo: SudoModal::new(),
            sudo_pending_action: None,
            first_run_state: FirstRunState::default(),
            setup_issues_checked: false,
        };
        (
            app,
            Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
        )
    }

    pub fn show_toast(&mut self, message: String, ok: bool) -> Task<Message> {
        self.toast = Some(Toast { message, ok });
        Task::perform(
            async { tokio::time::sleep(tokio::time::Duration::from_secs(4)).await },
            |_| Message::Tools(crate::messages::ToolsMessage::ClearToast),
        )
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(5))
            .map(|_| Message::Dashboard(crate::messages::DashboardMessage::AutoRefreshTick))
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.first_run_state == FirstRunState::Visible {
            return first_run::view();
        }

        let tab_content: Element<Message> = match &self.active_tab {
            Tab::Dashboard => self.dashboard.view(),
            Tab::SshKeys => self.ssh_keys.view(),
            Tab::Tools => self.tools.view(),
            Tab::Repos => self.repos.view(),
            Tab::VHosts => self.vhosts.view(),
        };

        let main_body: Element<Message> = if let Some(toast) = &self.toast {
            let (accent, border_color) = if toast.ok {
                (
                    GREEN,
                    Color {
                        r: 0.070,
                        g: 0.210,
                        b: 0.110,
                        a: 1.0,
                    },
                )
            } else {
                (
                    RED,
                    Color {
                        r: 0.300,
                        g: 0.090,
                        b: 0.080,
                        a: 1.0,
                    },
                )
            };
            let banner = container(
                row![
                    container(
                        text(if toast.ok { "+" } else { "x" })
                            .size(11)
                            .color(Color::WHITE)
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
                    text(toast.message.as_str()).size(13).color(TEXT_PRIMARY),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([10, 18]))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(BG_SURFACE.into()),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            });
            column![banner, tab_content].into()
        } else {
            tab_content
        };

        let app_area = row![
            self.sidebar(),
            container(main_body)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(BG_BASE.into()),
                    ..Default::default()
                }),
        ];

        if self.sudo.is_visible() {
            stack![
                container(app_area).width(Length::Fill).height(Length::Fill),
                self.sudo.view(),
            ]
            .into()
        } else {
            container(app_area)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let logo = container(
            column![
                row![
                    container(Space::with_width(3))
                        .width(3)
                        .height(26)
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(TEAL.into()),
                            border: Border {
                                radius: 2.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    Space::with_width(10),
                    column![
                        text("dev").size(19).color(TEAL),
                        text("panel").size(19).color(TEXT_PRIMARY),
                    ]
                    .spacing(0),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(0),
        )
        .padding(Padding::from([22, 16]));

        let nav = column![
            self.nav_item("Dashboard", Tab::Dashboard),
            self.nav_item("Repos", Tab::Repos),
            self.nav_item("VirtualHosts", Tab::VHosts),
            self.nav_item("SSH Keys", Tab::SshKeys),
            self.nav_item("Tools", Tab::Tools),
        ]
        .spacing(2)
        .padding(Padding::from([0, 8]));

        let sudo_indicator: Element<Message> = if self.sudo.cached_password.is_some() {
            column![
                container(
                    row![
                        container(Space::with_width(6)).width(6).height(6).style(
                            |_: &iced::Theme| container::Style {
                                background: Some(GREEN.into()),
                                border: Border {
                                    radius: 3.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        ),
                        Space::with_width(7),
                        text("sudo active").size(11).color(GREEN),
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
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                Space::with_height(5),
                button(text("Clear sudo").size(11).color(TEXT_MUTED))
                    .on_press(Message::Sudo(SudoMessage::ClearSaved))
                    .padding(Padding::from([4, 10]))
                    .style(|_, status| match status {
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(BG_HOVER.into()),
                            text_color: RED,
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
            .spacing(0)
            .into()
        } else {
            container(
                row![
                    container(Space::with_width(6))
                        .width(6)
                        .height(6)
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(YELLOW.into()),
                            border: Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    Space::with_width(7),
                    text("sudo locked").size(11).color(TEXT_MUTED),
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
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
        };

        let bottom = container(
            column![
                sudo_indicator,
                Space::with_height(10),
                button(
                    row![
                        text("R").size(11).color(TEXT_MUTED),
                        Space::with_width(6),
                        text("Refresh").size(12).color(TEXT_MUTED),
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
                        background: Some(BG_HOVER.into()),
                        text_color: TEXT_PRIMARY,
                        border: Border {
                            radius: 8.0.into(),
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
                Space::with_height(8),
                text(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .size(11)
                    .color(TEXT_MUTED),
            ]
            .spacing(0)
            .align_x(Alignment::Start),
        )
        .padding(Padding::from([10, 14]));

        container(
            column![
                logo,
                divider(),
                Space::with_height(10),
                nav,
                Space::with_height(Length::Fill),
                divider(),
                bottom,
            ]
            .height(Length::Fill),
        )
        .width(192)
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BG_SURFACE.into()),
            ..Default::default()
        })
        .into()
    }

    fn nav_item<'a>(&self, label: &'a str, tab: Tab) -> Element<'a, Message> {
        let active = self.active_tab == tab;
        let bg = if active {
            Color {
                r: 0.060,
                g: 0.185,
                b: 0.175,
                a: 1.0,
            }
        } else {
            Color::TRANSPARENT
        };
        let text_color = if active { TEXT_PRIMARY } else { TEXT_SECONDARY };
        let icon_color = if active { TEAL } else { TEXT_MUTED };
        button(
            row![
                text("").size(12).color(icon_color),
                Space::with_width(10),
                text(label).size(13).color(text_color),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::SelectTab(tab))
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: Some(BG_HOVER.into()),
                text_color: TEXT_PRIMARY,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => iced::widget::button::Style {
                background: Some(bg.into()),
                text_color,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .into()
    }
}

fn divider<'a>() -> Element<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_SUBTLE.into()),
            ..Default::default()
        })
        .into()
}
