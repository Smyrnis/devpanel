use super::{Provider, ProviderFilter, RemoteRepo, ReposTab, SshStatus};
use crate::core::theme::*;
use crate::messages::{Message, ReposMessage};
use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};

fn provider_color(p: &Provider) -> Color {
    match p {
        Provider::GitHub => TEAL,
        Provider::Bitbucket => BLUE,
    }
}
fn provider_bg(p: &Provider) -> Color {
    match p {
        Provider::GitHub => TEAL_BG,
        Provider::Bitbucket => BLUE_BG,
    }
}
fn provider_border(p: &Provider) -> Color {
    match p {
        Provider::GitHub => TEAL_BORDER,
        Provider::Bitbucket => BLUE_BORDER,
    }
}

pub fn render(tab: &ReposTab) -> Element<'_, Message> {
    let header = column![
        text("Repositories").size(22).color(TEXT_PRIMARY),
        Space::with_height(4),
        text("Browse and clone repos from GitHub and Bitbucket connected via SSH")
            .size(13)
            .color(TEXT_MUTED),
    ]
    .spacing(0);

    let status_bar = container(
        row![
            ssh_pill("GitHub", &tab.github_status),
            Space::with_width(10),
            ssh_pill("Bitbucket", &tab.bitbucket_status),
            Space::with_width(Length::Fill),
            icon_btn(
                if tab.fetching {
                    "Fetching…"
                } else {
                    "Fetch Repos"
                },
                TEAL,
                TEAL_BG,
                TEAL_HOVER,
                TEAL_BORDER,
                if tab.fetching {
                    None
                } else {
                    Some(Message::Repos(ReposMessage::Fetch))
                }
            ),
            Space::with_width(8),
            icon_btn(
                "Check SSH",
                TEXT_SECONDARY,
                BG_HOVER,
                BG_ELEVATED,
                BORDER_SUBTLE,
                Some(Message::Repos(ReposMessage::CheckSsh))
            ),
            Space::with_width(8),
            icon_btn(
                "Open Projects",
                TEXT_SECONDARY,
                BG_HOVER,
                BG_ELEVATED,
                BORDER_SUBTLE,
                Some(Message::Repos(ReposMessage::OpenRoot))
            ),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([12, 16]))
    .width(Length::Fill)
    .style(surface_style());

    let filter_bar: Element<Message> = if !tab.remote_repos.is_empty() {
        container(
            row![
                filter_tab("All", ProviderFilter::All, &tab.active_filter),
                Space::with_width(6),
                filter_tab("GitHub", ProviderFilter::GitHub, &tab.active_filter),
                Space::with_width(6),
                filter_tab("Bitbucket", ProviderFilter::Bitbucket, &tab.active_filter),
                Space::with_width(16),
                text_input("Search repos…", &tab.search_query)
                    .on_input(|v| Message::Repos(ReposMessage::SearchChanged(v)))
                    .size(12)
                    .padding(Padding::from([6, 10]))
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([10, 16]))
        .width(Length::Fill)
        .style(surface_style())
        .into()
    } else {
        Space::with_height(0).into()
    };

    let toast: Element<Message> = match &tab.status_msg {
        Some((ok, msg)) => {
            let (color, bg) = if *ok {
                (GREEN, GREEN_BG)
            } else {
                (RED, RED_BG)
            };
            container(
                row![
                    dot(color),
                    Space::with_width(8),
                    text(msg.as_str()).size(12).color(TEXT_SECONDARY)
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([10, 14]))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(bg.into()),
                border: Border {
                    color: BORDER_SUBTLE,
                    width: 1.0,
                    radius: 8.0.into(),
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
                text("Fetching repositories via SSH…")
                    .size(14)
                    .color(TEXT_MUTED),
                Space::with_height(8),
                text("This may take a moment").size(12).color(TEXT_MUTED),
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
                text("No repos match your filter")
                    .size(14)
                    .color(TEXT_MUTED),
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
            .color(TEXT_MUTED);
            let pager = row![
                small_nav(
                    "Prev",
                    if page > 0 {
                        Some(Message::Repos(ReposMessage::PrevPage))
                    } else {
                        None
                    }
                ),
                Space::with_width(6),
                small_nav(
                    "Next",
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

fn ssh_pill<'a>(label: &'a str, status: &'a SshStatus) -> Element<'a, Message> {
    let (dot_color, status_text, bg) = match status {
        SshStatus::Unknown => (TEXT_MUTED, "not checked", BG_SURFACE),
        SshStatus::Connected => (GREEN, "connected", GREEN_BG),
        SshStatus::Failed(_) => (RED, "no SSH key", RED_BG),
    };
    container(
        row![
            dot(dot_color),
            Space::with_width(6),
            text(label).size(11).color(TEXT_SECONDARY),
            Space::with_width(4),
            text(status_text).size(10).color(dot_color),
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([6, 12]))
    .style(move |_: &iced::Theme| container::Style {
        background: Some(bg.into()),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 20.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn empty_state<'a>() -> Element<'a, Message> {
    let github_hint = container(
        column![
            row![
                text("GitHub").size(12).color(TEAL),
                Space::with_width(8),
                text("ssh -T git@github.com").size(11).color(TEXT_MUTED)
            ]
            .align_y(Alignment::Center),
            Space::with_height(4),
            text("Ensure your SSH key is added at github.com/settings/keys")
                .size(11)
                .color(TEXT_MUTED),
        ]
        .spacing(0),
    )
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(TEAL_BG.into()),
        border: Border {
            color: TEAL_BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    let bitbucket_hint = container(
        column![
            row![
                text("Bitbucket").size(12).color(BLUE),
                Space::with_width(8),
                text("ssh -T git@bitbucket.org").size(11).color(TEXT_MUTED)
            ]
            .align_y(Alignment::Center),
            Space::with_height(4),
            text("Ensure your SSH key is added at bitbucket.org/account/settings/ssh-keys/")
                .size(11)
                .color(TEXT_MUTED),
        ]
        .spacing(0),
    )
    .padding(Padding::from([12, 14]))
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(BLUE_BG.into()),
        border: Border {
            color: BLUE_BORDER,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    let note = container(row![
        text("i").size(10).color(YELLOW), Space::with_width(8),
        text("DevPanel uses the GitHub CLI (gh) if installed, otherwise falls back to git ls-remote via SSH. Run \"Check SSH\" first, then \"Fetch Repos\".").size(11).color(TEXT_MUTED),
    ].align_y(Alignment::Center)).padding(Padding::from([10, 14])).width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
        background: Some(YELLOW_BG.into()),
        border: Border { color: YELLOW_BORDER, width: 1.0, radius: 8.0.into() },
        ..Default::default()
    });

    container(column![
        text("No repos fetched yet").size(15).color(TEXT_SECONDARY),
        Space::with_height(4),
        text("Click \"Check SSH\" to verify your keys, then \"Fetch Repos\" to list your remote repositories.").size(13).color(TEXT_MUTED),
        Space::with_height(20),
        github_hint, Space::with_height(8), bitbucket_hint, Space::with_height(16), note,
    ].spacing(0)).width(Length::Fill).padding(Padding::from([8, 0])).into()
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
        container(text("cloned").size(10).color(GREEN))
            .padding(Padding::from([3, 8]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(GREEN_BG.into()),
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
        container(text("M").size(10).color(YELLOW))
            .padding(Padding::from([3, 8]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(YELLOW_BG.into()),
                border: Border {
                    color: YELLOW_BORDER,
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
        text(repo.name.as_str()).size(14).color(TEXT_PRIMARY),
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
        text("SSH").size(10).color(TEXT_MUTED),
        Space::with_width(6),
        text(repo.ssh_url.as_str()).size(11).color(BORDER_MED),
    ]
    .align_y(Alignment::Center);

    let action_btn: Element<Message> = if repo.is_cloning {
        button(text("Cloning…").size(12).color(TEXT_MUTED))
            .padding(Padding::from([7, 16]))
            .style(|_, _| iced::widget::button::Style {
                background: Some(BG_SURFACE.into()),
                border: Border {
                    color: BORDER_SUBTLE,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            .into()
    } else if repo.is_cloned {
        row![
            button(text("Terminal").size(12).color(TEAL))
                .on_press(Message::Repos(ReposMessage::OpenCloned(repo.name.clone())))
                .padding(Padding::from([7, 16]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(TEAL_HOVER.into()),
                        text_color: TEAL,
                        border: Border {
                            color: TEAL_BORDER,
                            width: 1.0,
                            radius: 8.0.into()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: Some(TEAL_BG.into()),
                        text_color: TEAL,
                        border: Border {
                            color: TEAL_BORDER,
                            width: 1.0,
                            radius: 8.0.into()
                        },
                        ..Default::default()
                    },
                }),
            Space::with_width(6),
            button(text("Editor").size(12).color(BLUE))
                .on_press(Message::Repos(ReposMessage::OpenEditor(repo.name.clone())))
                .padding(Padding::from([7, 16]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(BLUE_BG.into()),
                        text_color: BLUE,
                        border: Border {
                            color: BLUE_BORDER,
                            width: 1.0,
                            radius: 8.0.into()
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: Some(BG_SURFACE.into()),
                        text_color: BLUE,
                        border: Border {
                            color: BLUE_BORDER,
                            width: 1.0,
                            radius: 8.0.into()
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
        button(text("Clone").size(12).color(GREEN))
            .on_press(Message::Repos(ReposMessage::Clone { ssh_url: url, name }))
            .padding(Padding::from([7, 16]))
            .style(|_, status| match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                    iced::widget::button::Style {
                        background: Some(GREEN_HOVER.into()),
                        text_color: GREEN,
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }
                _ => iced::widget::button::Style {
                    background: Some(GREEN_BG.into()),
                    text_color: GREEN,
                    border: Border {
                        radius: 8.0.into(),
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
            text(repo.full_name.as_str()).size(11).color(TEXT_MUTED),
            Space::with_height(6),
            ssh_row,
            Space::with_height(14),
            thin_line(),
            Space::with_height(12),
            row![Space::with_width(Length::Fill), action_btn].align_y(Alignment::Center),
        ]
        .spacing(0),
    )
    .padding(Padding::from([16, 18]))
    .width(Length::Fill)
    .style(card_style())
    .into()
}

fn small_nav<'a>(label: &'a str, on_press: Option<Message>) -> Element<'a, Message> {
    icon_btn(
        label,
        TEXT_SECONDARY,
        BG_HOVER,
        BG_ELEVATED,
        BORDER_SUBTLE,
        on_press,
    )
}

fn card_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(BG_CARD.into()),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}
fn surface_style() -> impl Fn(&iced::Theme) -> container::Style {
    |_| container::Style {
        background: Some(BG_SURFACE.into()),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}
fn thin_line<'a>() -> iced::widget::Container<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_: &iced::Theme| container::Style {
            background: Some(BORDER_SUBTLE.into()),
            ..Default::default()
        })
}
fn dot(color: Color) -> iced::widget::Container<'static, Message> {
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
}
fn icon_btn<'a>(
    label: &'a str,
    color: Color,
    bg: Color,
    bg_hover: Color,
    border: Color,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let b = button(text(label).size(12).color(color))
        .padding(Padding::from([7, 14]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(bg_hover.into()),
                    text_color: color,
                    border: Border {
                        color: border,
                        width: 1.0,
                        radius: 8.0.into(),
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
                    radius: 8.0.into(),
                },
                ..Default::default()
            },
        });
    if let Some(msg) = on_press {
        b.on_press(msg).into()
    } else {
        b.into()
    }
}
fn filter_tab<'a>(
    label: &'a str,
    filter: ProviderFilter,
    active: &ProviderFilter,
) -> Element<'a, Message> {
    let is_active = &filter == active;
    let (color, bg, border) = if is_active {
        (TEAL, TEAL_BG, TEAL_BORDER)
    } else {
        (TEXT_MUTED, BG_SURFACE, BORDER_SUBTLE)
    };
    button(text(label).size(11).color(color))
        .on_press(Message::Repos(ReposMessage::SetFilter(filter)))
        .padding(Padding::from([5, 12]))
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                iced::widget::button::Style {
                    background: Some(TEAL_BG.into()),
                    text_color: TEAL,
                    border: Border {
                        color: TEAL_BORDER,
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
