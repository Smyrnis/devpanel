use crate::app::App;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::app as keys, text as tr};
use crate::messages::{Message, SudoMessage, Tab};
use crate::ui::icons::{self, Icon};
use crate::ui::templates::prelude as ui;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Border, Color, Element, Length, Padding};

impl App {
    pub(super) fn sidebar(&self) -> Element<'_, Message> {
        let compact = self.is_compact();
        let logo = logo(compact);
        let nav = nav_items(self, compact);
        let bottom = sidebar_bottom(self, compact);

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
        .width(if compact { 64 } else { 192 })
        .height(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn nav_item<'a>(&self, icon: Icon, label: &'a str, tab: Tab) -> Element<'a, Message> {
        let compact = self.is_compact();
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

        let content: Element<Message> = if compact {
            row![
                accent,
                Space::with_width(Length::Fill),
                icons::solid_box(icon, 15.0, text_color, 18.0),
                Space::with_width(Length::Fill),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                accent,
                Space::with_width(10),
                icons::solid_box(icon, 14.0, text_color, 17.0),
                Space::with_width(10),
                text(label)
                    .size(crate::core::app_config::text_metrics().body)
                    .color(text_color),
            ]
            .align_y(Alignment::Center)
            .into()
        };

        button(row![content, Space::with_width(Length::Fill)].align_y(Alignment::Center))
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

fn logo<'a>(compact: bool) -> Element<'a, Message> {
    if compact {
        container(
            column![
                accent_mark(),
                Space::with_height(14),
                icons::solid(Icon::Dashboard, 18.0, theme::color(theme_keys::TEAL)),
            ]
            .align_x(Alignment::Center),
        )
        .padding(Padding::from([18, 0]))
        .into()
    } else {
        container(
            column![
                row![
                    accent_mark(),
                    Space::with_width(10),
                    column![
                        text(tr(keys::LOGO_DEV))
                            .size(crate::core::app_config::icon_metrics().sidebar_logo)
                            .color(theme::color(theme_keys::TEAL)),
                        text(tr(keys::LOGO_PANEL))
                            .size(crate::core::app_config::icon_metrics().sidebar_logo)
                            .color(theme::color(theme_keys::TEXT_PRIMARY)),
                    ]
                    .spacing(0),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(0)
            .align_x(Alignment::Start),
        )
        .padding(Padding::from([22, 16]))
        .into()
    }
}

fn nav_items(app: &App, compact: bool) -> iced::widget::Column<'_, Message> {
    if compact {
        column![
            app.nav_item(Icon::Dashboard, tr(keys::NAV_DASHBOARD), Tab::Dashboard),
            app.nav_item(Icon::Globe, tr(keys::NAV_VHOSTS), Tab::VHosts),
            app.nav_item(Icon::Tools, tr(keys::NAV_TOOLS), Tab::Tools),
            app.nav_item(Icon::Config, tr(keys::NAV_CONFIG), Tab::Config),
        ]
        .spacing(5)
        .padding(Padding::from([0, 8]))
    } else {
        column![
            text(tr(keys::NAVIGATION))
                .size(crate::core::app_config::text_metrics().tiny)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(6),
            app.nav_item(Icon::Dashboard, tr(keys::NAV_DASHBOARD), Tab::Dashboard),
            app.nav_item(Icon::Globe, tr(keys::NAV_VHOSTS), Tab::VHosts),
            app.nav_item(Icon::Tools, tr(keys::NAV_TOOLS), Tab::Tools),
            app.nav_item(Icon::Config, tr(keys::NAV_CONFIG), Tab::Config),
        ]
        .spacing(3)
        .padding(Padding::from([0, 12]))
    }
}

fn sidebar_bottom(app: &App, compact: bool) -> Element<'_, Message> {
    if compact {
        container(
            column![
                ui::status_dot(if app.dashboard.apache_running {
                    theme::color(theme_keys::GREEN)
                } else {
                    theme::color(theme_keys::TEXT_MUTED)
                }),
                Space::with_height(10),
                ui::status_dot(if app.dashboard.mysql_running {
                    theme::color(theme_keys::GREEN)
                } else {
                    theme::color(theme_keys::TEXT_MUTED)
                }),
            ]
            .align_x(Alignment::Center),
        )
        .padding(Padding::from([10, 0]))
        .into()
    } else {
        container(
            column![
                text(tr(keys::SYSTEM))
                    .size(crate::core::app_config::text_metrics().tiny)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(6),
                sidebar_status_row(tr(keys::APACHE), app.dashboard.apache_running),
                Space::with_height(5),
                sidebar_status_row("MySQL", app.dashboard.mysql_running),
                Space::with_height(10),
                sudo_indicator(app),
                Space::with_height(8),
                text(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .size(crate::core::app_config::text_metrics().caption)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .spacing(0)
            .align_x(Alignment::Start),
        )
        .padding(Padding::from([10, 14]))
        .into()
    }
}

fn accent_mark<'a>() -> Element<'a, Message> {
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
        })
        .into()
}

fn sudo_indicator(app: &App) -> Element<'_, Message> {
    if app.sudo.cached_password.is_some() {
        column![
            container(
                row![
                    dot(theme::color(theme_keys::GREEN)),
                    Space::with_width(7),
                    text(tr(keys::SUDO_ACTIVE))
                        .size(crate::core::app_config::text_metrics().caption)
                        .color(theme::color(theme_keys::GREEN)),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([6, 10]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(theme::color(theme_keys::GREEN_BG).into()),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_height(5),
            button(
                text(tr(keys::CLEAR_SUDO))
                    .size(crate::core::app_config::text_metrics().caption)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            )
            .on_press(Message::Sudo(SudoMessage::ClearSaved))
            .padding(Padding::from([4, 10]))
            .style(ghost_danger_style),
        ]
        .spacing(0)
        .into()
    } else {
        container(
            row![
                dot(theme::color(theme_keys::YELLOW)),
                Space::with_width(7),
                text(tr(keys::SUDO_LOCKED))
                    .size(crate::core::app_config::text_metrics().caption)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([6, 10]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::YELLOW_BG).into()),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
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
            .size(crate::core::app_config::text_metrics().caption)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_width(Length::Fill),
        text(if running { tr(keys::ON) } else { tr(keys::OFF) })
            .size(crate::core::app_config::text_metrics().tiny)
            .color(color),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn dot<'a>(color: Color) -> Element<'a, Message> {
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

fn ghost_danger_style(
    _: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    match status {
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
    }
}
