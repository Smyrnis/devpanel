use super::{Provider, ProviderFilter, RemoteRepo, ReposTab, SshStatus};
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::repos as keys, text as tr};
use crate::messages::{Message, ReposMessage};
use crate::ui::templates::view as ui;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

fn provider_color(p: &Provider) -> Color {
    match p {
        Provider::GitHub => theme::color(theme_keys::TEAL),
        Provider::Bitbucket => theme::color(theme_keys::BLUE),
    }
}
fn provider_bg(p: &Provider) -> Color {
    match p {
        Provider::GitHub => theme::color(theme_keys::TEAL_BG),
        Provider::Bitbucket => theme::color(theme_keys::BLUE_BG),
    }
}
fn provider_border(p: &Provider) -> Color {
    match p {
        Provider::GitHub => theme::color(theme_keys::TEAL_BORDER),
        Provider::Bitbucket => theme::color(theme_keys::BLUE_BORDER),
    }
}

pub fn render(tab: &ReposTab) -> Element<'_, Message> {
    let header = ui::page_header(
        tr(keys::TITLE),
        tr(keys::SUBTITLE),
        vec![
            ui::secondary_button(
                tr(keys::OPEN_PROJECTS),
                Message::Repos(ReposMessage::OpenRoot),
            ),
            ui::secondary_button(tr(keys::CHECK_SSH), Message::Repos(ReposMessage::CheckSsh)),
            ui::primary_button(tr(keys::FETCH_REPOS), Message::Repos(ReposMessage::Fetch)),
        ],
    );

    let status_bar = connection_status_panel(tab);

    let filter_bar: Element<Message> = if !tab.remote_repos.is_empty() {
        container(
            row![
                filter_tab(
                    tr(keys::FILTER_ALL),
                    ProviderFilter::All,
                    &tab.active_filter
                ),
                Space::with_width(6),
                filter_tab(tr(keys::GITHUB), ProviderFilter::GitHub, &tab.active_filter),
                Space::with_width(6),
                filter_tab(
                    tr(keys::BITBUCKET),
                    ProviderFilter::Bitbucket,
                    &tab.active_filter
                ),
                Space::with_width(16),
                text_input(tr(keys::SEARCH_PLACEHOLDER), &tab.search_query)
                    .on_input(|v| Message::Repos(ReposMessage::SearchChanged(v)))
                    .size(12)
                    .padding(Padding::from([6, 10]))
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 16]))
        .width(Length::Fill)
        .style(ui::surface_style())
        .into()
    } else {
        Space::with_height(0).into()
    };

    let toast: Element<Message> = match &tab.status_msg {
        Some((ok, msg)) => {
            let (color, bg) = if *ok {
                (
                    theme::color(theme_keys::GREEN),
                    theme::color(theme_keys::GREEN_BG),
                )
            } else {
                (
                    theme::color(theme_keys::RED),
                    theme::color(theme_keys::RED_BG),
                )
            };
            container(
                row![
                    ui::status_dot(color),
                    Space::with_width(8),
                    text(msg.as_str())
                        .size(12)
                        .color(theme::color(theme_keys::TEXT_SECONDARY))
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([10, 14]))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(bg.into()),
                border: Border {
                    color: theme::color(theme_keys::BORDER_SUBTLE),
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
        }
        None => Space::with_height(0).into(),
    };

    let body: Element<Message> = if tab.remote_repos.is_empty() && !tab.fetching {
        empty_state()
    } else if tab.fetching {
        container(
            column![
                text(tr(keys::FETCHING_REPOS))
                    .size(14)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
                Space::with_height(8),
                text(tr(keys::FETCHING_NOTE))
                    .size(12)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(Padding::from([48, 0]))
        .center_x(Length::Fill)
        .into()
    } else {
        let q = tab.search_query.to_lowercase();
        let filtered: Vec<&RemoteRepo> = tab
            .remote_repos
            .iter()
            .filter(|r| {
                let provider_ok = match &tab.active_filter {
                    ProviderFilter::All => true,
                    ProviderFilter::GitHub => r.provider == Provider::GitHub,
                    ProviderFilter::Bitbucket => r.provider == Provider::Bitbucket,
                };
                let search_ok = q.is_empty()
                    || r.name.to_lowercase().contains(&q)
                    || r.full_name.to_lowercase().contains(&q);
                provider_ok && search_ok
            })
            .collect();

        if filtered.is_empty() {
            container(
                text(tr(keys::NO_FILTER_MATCH))
                    .size(14)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            )
            .width(Length::Fill)
            .padding(Padding::from([40, 0]))
            .center_x(Length::Fill)
            .into()
        } else {
            let total_pages = filtered.len().saturating_sub(1) / tab.page_size + 1;
            let page = tab.page.min(total_pages.saturating_sub(1));
            let start = page * tab.page_size;
            let end = (start + tab.page_size).min(filtered.len());
            let page_slice = &filtered[start..end];
            let count = text(format!(
                "{} repo{} - page {}/{}",
                filtered.len(),
                if filtered.len() == 1 { "" } else { "s" },
                page + 1,
                total_pages,
            ))
            .size(11)
            .color(theme::color(theme_keys::TEXT_MUTED));
            let pager = row![
                small_nav(
                    tr(keys::PREV),
                    if page > 0 {
                        Some(Message::Repos(ReposMessage::PrevPage))
                    } else {
                        None
                    }
                ),
                Space::with_width(6),
                small_nav(
                    tr(keys::NEXT),
                    if page + 1 < total_pages {
                        Some(Message::Repos(ReposMessage::NextPage))
                    } else {
                        None
                    }
                ),
            ]
            .align_y(Alignment::Center);
            let cards: Vec<Element<Message>> = page_slice.iter().map(|r| repo_card(r)).collect();
            column![
                row![count, Space::with_width(Length::Fill), pager].align_y(Alignment::Center),
                Space::with_height(10),
                column(cards).spacing(8)
            ]
            .spacing(0)
            .into()
        }
    };

    scrollable(
        column![
            header,
            Space::with_height(18),
            status_bar,
            Space::with_height(10),
            filter_bar,
            Space::with_height(if !tab.remote_repos.is_empty() { 10 } else { 0 }),
            toast,
            Space::with_height(if tab.status_msg.is_some() { 12 } else { 0 }),
            body,
            Space::with_height(24),
        ]
        .spacing(0)
        .padding(Padding::from([22, 24])),
    )
    .into()
}

fn connection_status_panel(tab: &ReposTab) -> Element<'_, Message> {
    container(
        column![
            row![
                text(tr(keys::PROVIDER_ACCESS))
                    .size(13)
                    .color(theme::color(theme_keys::TEXT_SECONDARY)),
                Space::with_width(Length::Fill),
                text(if tab.fetching { tr(keys::FETCHING) } else { "" })
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED)),
            ]
            .align_y(Alignment::Center),
            Space::with_height(12),
            row![
                provider_access_card(
                    tr(keys::GITHUB),
                    &tab.github_status,
                    tr(keys::GITHUB_SSH_COMMAND),
                    theme::color(theme_keys::TEAL),
                    theme::color(theme_keys::TEAL_BG),
                    theme::color(theme_keys::TEAL_BORDER),
                ),
                provider_access_card(
                    tr(keys::BITBUCKET),
                    &tab.bitbucket_status,
                    tr(keys::BITBUCKET_SSH_COMMAND),
                    theme::color(theme_keys::BLUE),
                    theme::color(theme_keys::BLUE_BG),
                    theme::color(theme_keys::BLUE_BORDER),
                ),
                repo_count_card(tab.remote_repos.len()),
            ]
            .spacing(12),
        ]
        .spacing(0),
    )
    .padding(Padding::from([14, 16]))
    .width(Length::Fill)
    .style(ui::surface_style())
    .into()
}

fn provider_access_card<'a>(
    label: &'a str,
    status: &'a SshStatus,
    command: &'a str,
    accent: Color,
    bg: Color,
    border: Color,
) -> Element<'a, Message> {
    let status_text = ssh_status_text(status);
    container(
        column![
            row![
                ui::status_dot(ssh_status_color(status)),
                Space::with_width(7),
                text(label).size(12).color(accent),
                Space::with_width(Length::Fill),
                text(status_text).size(10).color(ssh_status_color(status)),
            ]
            .align_y(Alignment::Center),
            Space::with_height(8),
            text(tr(keys::PROVIDER_COMMAND))
                .size(10)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(3),
            text(command)
                .size(11)
                .color(theme::color(theme_keys::TEXT_SECONDARY)),
        ]
        .spacing(0),
    )
    .padding(Padding::from([12, 14]))
    .width(Length::FillPortion(1))
    .style(move |_: &iced::Theme| container::Style {
        background: Some(bg.into()),
        border: Border {
            color: border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn repo_count_card(count: usize) -> Element<'static, Message> {
    container(
        column![
            text(count.to_string())
                .size(20)
                .color(theme::color(theme_keys::TEXT_PRIMARY)),
            Space::with_height(3),
            text(tr(keys::REPOS_LOADED))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0),
    )
    .padding(Padding::from([12, 14]))
    .width(Length::FillPortion(1))
    .style(ui::card_style())
    .into()
}

fn ssh_status_text(status: &SshStatus) -> String {
    match status {
        SshStatus::Unknown => tr(keys::SSH_UNKNOWN).to_string(),
        SshStatus::Connected => tr(keys::SSH_CONNECTED).to_string(),
        SshStatus::Failed(reason) if !reason.trim().is_empty() => reason.clone(),
        SshStatus::Failed(_) => tr(keys::SSH_FAILED).to_string(),
    }
}

fn ssh_status_color(status: &SshStatus) -> Color {
    match status {
        SshStatus::Unknown => theme::color(theme_keys::TEXT_MUTED),
        SshStatus::Connected => theme::color(theme_keys::GREEN),
        SshStatus::Failed(_) => theme::color(theme_keys::RED),
    }
}

fn empty_state<'a>() -> Element<'a, Message> {
    let github_hint = container(
        column![
            row![
                text(tr(keys::GITHUB))
                    .size(12)
                    .color(theme::color(theme_keys::TEAL)),
                Space::with_width(8),
                text(tr(keys::GITHUB_SSH_COMMAND))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED))
            ]
            .align_y(Alignment::Center),
            Space::with_height(4),
            text(tr(keys::GITHUB_HINT))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0),
    )
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::TEAL_BG).into()),
        border: Border {
            color: theme::color(theme_keys::TEAL_BORDER),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    let bitbucket_hint = container(
        column![
            row![
                text(tr(keys::BITBUCKET))
                    .size(12)
                    .color(theme::color(theme_keys::BLUE)),
                Space::with_width(8),
                text(tr(keys::BITBUCKET_SSH_COMMAND))
                    .size(11)
                    .color(theme::color(theme_keys::TEXT_MUTED))
            ]
            .align_y(Alignment::Center),
            Space::with_height(4),
            text(tr(keys::BITBUCKET_HINT))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .spacing(0),
    )
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::BLUE_BG).into()),
        border: Border {
            color: theme::color(theme_keys::BLUE_BORDER),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    let note = container(
        row![
            text("i").size(10).color(theme::color(theme_keys::YELLOW)),
            Space::with_width(8),
            text(tr(keys::CLI_NOTE))
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([10, 14]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(theme::color(theme_keys::YELLOW_BG).into()),
        border: Border {
            color: theme::color(theme_keys::YELLOW_BORDER),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    });

    column![
        ui::empty_state(
            tr(keys::EMPTY_TITLE),
            tr(keys::EMPTY_BODY),
            vec![
                ui::secondary_button(tr(keys::CHECK_SSH), Message::Repos(ReposMessage::CheckSsh),),
                ui::primary_button(tr(keys::FETCH_REPOS), Message::Repos(ReposMessage::Fetch),),
            ],
        ),
        Space::with_height(20),
        container(
            column![
                github_hint,
                Space::with_height(8),
                bitbucket_hint,
                Space::with_height(16),
                note,
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .padding(Padding::from([8, 0]))
    ]
    .spacing(0)
    .into()
}

fn repo_card<'a>(repo: &'a RemoteRepo) -> Element<'a, Message> {
    let p_color = provider_color(&repo.provider);
    let p_bg = provider_bg(&repo.provider);
    let p_border = provider_border(&repo.provider);

    let provider_badge = container(text(repo.provider.label()).size(10).color(p_color))
        .padding(Padding::from([3, 8]))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(p_bg.into()),
            border: Border {
                color: p_border,
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        });

    let cloned_badge: Element<Message> = if repo.is_cloned {
        container(
            text(tr(keys::CLONED))
                .size(10)
                .color(theme::color(theme_keys::GREEN)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::GREEN_BG).into()),
            border: Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_width(0).into()
    };
    let dirty_badge: Element<Message> = if repo.is_dirty {
        container(
            text(tr(keys::DIRTY_BADGE))
                .size(10)
                .color(theme::color(theme_keys::YELLOW)),
        )
        .padding(Padding::from([3, 8]))
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme::color(theme_keys::YELLOW_BG).into()),
            border: Border {
                color: theme::color(theme_keys::YELLOW_BORDER),
                width: 1.0,
                radius: 20.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_width(0).into()
    };

    let name_row = row![
        text(repo.name.as_str())
            .size(14)
            .color(theme::color(theme_keys::TEXT_PRIMARY)),
        Space::with_width(8),
        provider_badge,
        Space::with_width(6),
        cloned_badge,
        Space::with_width(6),
        dirty_badge,
        Space::with_width(Length::Fill),
    ]
    .align_y(Alignment::Center);

    let ssh_row = row![
        text(tr(keys::SSH_LABEL))
            .size(10)
            .color(theme::color(theme_keys::TEXT_MUTED)),
        Space::with_width(6),
        text(repo.ssh_url.as_str())
            .size(11)
            .color(theme::color(theme_keys::BORDER_MED)),
    ]
    .align_y(Alignment::Center);

    let action_btn: Element<Message> = if repo.is_cloning {
        button(
            text(tr(keys::CLONING))
                .size(12)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        )
        .padding(Padding::from([7, 16]))
        .style(|_, _| iced::widget::button::Style {
            background: Some(theme::color(theme_keys::BG_SURFACE).into()),
            border: Border {
                color: theme::color(theme_keys::BORDER_SUBTLE),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
    } else if repo.is_cloned {
        row![
            button(
                text(tr(keys::TERMINAL))
                    .size(12)
                    .color(theme::color(theme_keys::TEAL))
            )
            .on_press(Message::Repos(ReposMessage::OpenCloned(repo.name.clone())))
            .padding(Padding::from([7, 16]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                    iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::TEAL_HOVER).into()),
                        text_color: theme::color(theme_keys::TEAL),
                        border: Border {
                            color: theme::color(theme_keys::TEAL_BORDER),
                            width: 1.0,
                            radius: 6.0.into()
                        },
                        ..Default::default()
                    },
                _ => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::TEAL_BG).into()),
                    text_color: theme::color(theme_keys::TEAL),
                    border: Border {
                        color: theme::color(theme_keys::TEAL_BORDER),
                        width: 1.0,
                        radius: 6.0.into()
                    },
                    ..Default::default()
                },
            }),
            Space::with_width(6),
            button(
                text(tr(keys::EDITOR))
                    .size(12)
                    .color(theme::color(theme_keys::BLUE))
            )
            .on_press(Message::Repos(ReposMessage::OpenEditor(repo.name.clone())))
            .padding(Padding::from([7, 16]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed =>
                    iced::widget::button::Style {
                        background: Some(theme::color(theme_keys::BLUE_BG).into()),
                        text_color: theme::color(theme_keys::BLUE),
                        border: Border {
                            color: theme::color(theme_keys::BLUE_BORDER),
                            width: 1.0,
                            radius: 6.0.into()
                        },
                        ..Default::default()
                    },
                _ => iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::BG_SURFACE).into()),
                    text_color: theme::color(theme_keys::BLUE),
                    border: Border {
                        color: theme::color(theme_keys::BLUE_BORDER),
                        width: 1.0,
                        radius: 6.0.into()
                    },
                    ..Default::default()
                },
            }),
        ]
        .align_y(Alignment::Center)
        .into()
    } else {
        let url = repo.ssh_url.clone();
        let name = repo.name.clone();
        button(
            text(tr(keys::CLONE))
                .size(12)
                .color(theme::color(theme_keys::GREEN)),
        )
        .on_press(Message::Repos(ReposMessage::Clone { ssh_url: url, name }))
        .padding(Padding::from([7, 16]))
        .style(|_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::GREEN_HOVER).into()),
                    text_color: theme::color(theme_keys::GREEN),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
            _ => iced::widget::button::Style {
                background: Some(theme::color(theme_keys::GREEN_BG).into()),
                text_color: theme::color(theme_keys::GREEN),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        })
        .into()
    };

    container(
        column![
            name_row,
            Space::with_height(3),
            text(repo.full_name.as_str())
                .size(11)
                .color(theme::color(theme_keys::TEXT_MUTED)),
            Space::with_height(6),
            ssh_row,
            Space::with_height(14),
            ui::thin_line(),
            Space::with_height(12),
            row![Space::with_width(Length::Fill), action_btn].align_y(Alignment::Center),
        ]
        .spacing(0),
    )
    .padding(Padding::from([16, 18]))
    .width(Length::Fill)
    .style(ui::card_style())
    .into()
}

fn small_nav<'a>(label: &'a str, on_press: Option<Message>) -> Element<'a, Message> {
    ui::action_button(
        label,
        theme::color(theme_keys::TEXT_SECONDARY),
        theme::color(theme_keys::BG_HOVER),
        theme::color(theme_keys::BG_ELEVATED),
        theme::color(theme_keys::BORDER_SUBTLE),
        on_press,
    )
}
fn filter_tab<'a>(
    label: &'a str,
    filter: ProviderFilter,
    active: &ProviderFilter,
) -> Element<'a, Message> {
    let is_active = &filter == active;
    let (color, bg, border) = if is_active {
        (
            theme::color(theme_keys::TEAL),
            theme::color(theme_keys::TEAL_BG),
            theme::color(theme_keys::TEAL_BORDER),
        )
    } else {
        (
            theme::color(theme_keys::TEXT_MUTED),
            theme::color(theme_keys::BG_SURFACE),
            theme::color(theme_keys::BORDER_SUBTLE),
        )
    };
    button(text(label).size(11).color(color))
        .on_press(Message::Repos(ReposMessage::SetFilter(filter)))
        .padding(Padding::from([5, 12]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(theme::color(theme_keys::TEAL_BG).into()),
                    text_color: theme::color(theme_keys::TEAL),
                    border: Border {
                        color: theme::color(theme_keys::TEAL_BORDER),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                }
            }
            _ => iced::widget::button::Style {
                background: Some(bg.into()),
                text_color: color,
                border: Border {
                    color: border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            },
        })
        .into()
}
