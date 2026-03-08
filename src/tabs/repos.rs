// src/tabs/repos.rs — Remote repository browser (GitHub + Bitbucket via SSH)

use crate::theme::*;
use crate::Message;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding};

// ── Colors ────────────────────────────────────────────────────────────────
// All tinted backgrounds derived from index.php *-dim variables.

// GREEN  — rgba(48,209,88,.12) on #0a0a0a
const GREEN_BG: Color    = Color { r: 0.071, g: 0.122, b: 0.082, a: 1.0 };
const GREEN_HOVER: Color = Color { r: 0.090, g: 0.148, b: 0.100, a: 1.0 };

// BLUE   — rgba(10,132,255,.10) on #0a0a0a
const BLUE_BG: Color     = Color { r: 0.047, g: 0.090, b: 0.157, a: 1.0 };
const BLUE_BORDER: Color = Color { r: 0.070, g: 0.130, b: 0.220, a: 1.0 };

// TEAL is now GREEN (brand accent = index.php --green #30d158)
const TEAL_BG: Color     = GREEN_BG;
const TEAL_HOVER: Color  = GREEN_HOVER;
const TEAL_BORDER: Color = Color { r: 0.100, g: 0.200, b: 0.118, a: 1.0 }; // rgba(48,209,88,.28)

// RED    — rgba(255,69,58,.10) on #0a0a0a
const RED_BG: Color      = Color { r: 0.137, g: 0.071, b: 0.067, a: 1.0 };

// YELLOW — rgba(255,214,10,.10) on #0a0a0a
const YELLOW_BG: Color     = Color { r: 0.137, g: 0.122, b: 0.043, a: 1.0 };
const YELLOW_BORDER: Color = Color { r: 0.180, g: 0.160, b: 0.055, a: 1.0 };

// ── Data types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    GitHub,
    Bitbucket,
}

impl Provider {
    pub fn label(&self) -> &'static str {
        match self {
            Provider::GitHub => "GitHub",
            Provider::Bitbucket => "Bitbucket",
        }
    }
    pub fn color(&self) -> Color {
        match self {
            Provider::GitHub => TEAL,
            Provider::Bitbucket => BLUE,
        }
    }
    pub fn bg(&self) -> Color {
        match self {
            Provider::GitHub => TEAL_BG,
            Provider::Bitbucket => BLUE_BG,
        }
    }
    pub fn border(&self) -> Color {
        match self {
            Provider::GitHub => TEAL_BORDER,
            Provider::Bitbucket => BLUE_BORDER,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteRepo {
    pub name: String,      // e.g. "my-project"
    pub full_name: String, // e.g. "username/my-project"
    pub ssh_url: String,   // e.g. "git@github.com:username/my-project.git"
    pub provider: Provider,
    pub is_cloned: bool,  // true if ~/projects/<name> already exists
    pub is_cloning: bool, // clone in progress
}

#[derive(Debug, Clone, PartialEq)]
pub enum SshStatus {
    Unknown,
    Connected,
    Failed(String),
}

// ── Tab state ─────────────────────────────────────────────────────────────

pub struct ReposTab {
    pub repos_root: String,
    pub remote_repos: Vec<RemoteRepo>,
    pub fetching: bool,
    pub github_status: SshStatus,
    pub bitbucket_status: SshStatus,
    pub search_query: String,
    pub status_msg: Option<(bool, String)>,
    pub active_filter: ProviderFilter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderFilter {
    All,
    GitHub,
    Bitbucket,
}

impl ReposTab {
    pub fn new(repos_root: String, _unused: String) -> Self {
        Self {
            repos_root,
            remote_repos: Vec::new(),
            fetching: false,
            github_status: SshStatus::Unknown,
            bitbucket_status: SshStatus::Unknown,
            search_query: String::new(),
            status_msg: None,
            active_filter: ProviderFilter::All,
        }
    }

    pub fn set_repos(&mut self, repos: Vec<RemoteRepo>) {
        self.fetching = false;
        self.remote_repos = repos;
    }

    pub fn mark_cloning(&mut self, ssh_url: &str, cloning: bool) {
        if let Some(r) = self.remote_repos.iter_mut().find(|r| r.ssh_url == ssh_url) {
            r.is_cloning = cloning;
        }
    }

    pub fn mark_cloned(&mut self, ssh_url: &str) {
        if let Some(r) = self.remote_repos.iter_mut().find(|r| r.ssh_url == ssh_url) {
            r.is_cloning = false;
            r.is_cloned = true;
        }
    }

    // ── View ──────────────────────────────────────────────────────────────

    pub fn view(&self) -> Element<'_, Message> {
        let header = column![
            text("Repositories").size(22).color(TEXT_PRIMARY),
            Space::with_height(4),
            text("Browse and clone repos from GitHub and Bitbucket connected via SSH")
                .size(13)
                .color(TEXT_MUTED),
        ]
        .spacing(0);

        // SSH status bar
        let status_bar = container(
            row![
                self.ssh_pill("GitHub", &self.github_status),
                Space::with_width(10),
                self.ssh_pill("Bitbucket", &self.bitbucket_status),
                Space::with_width(Length::Fill),
                icon_btn(
                    if self.fetching {
                        "Fetching…"
                    } else {
                        "Fetch Repos"
                    },
                    TEAL,
                    TEAL_BG,
                    TEAL_HOVER,
                    TEAL_BORDER,
                    if self.fetching {
                        None
                    } else {
                        Some(Message::REPOS_Fetch)
                    },
                ),
                Space::with_width(8),
                icon_btn(
                    "Check SSH",
                    TEXT_SECONDARY,
                    BG_HOVER,
                    BG_ELEVATED,
                    BORDER_SUBTLE,
                    Some(Message::REPOS_CheckSsh)
                ),
                Space::with_width(8),
                icon_btn(
                    "Open Projects",
                    TEXT_SECONDARY,
                    BG_HOVER,
                    BG_ELEVATED,
                    BORDER_SUBTLE,
                    Some(Message::REPOS_OpenRoot)
                ),
            ]
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([12, 16]))
        .width(Length::Fill)
        .style(surface_style());

        // Filter + search bar
        let filter_bar: Element<Message> = if !self.remote_repos.is_empty() {
            container(
                row![
                    filter_tab("All", ProviderFilter::All, &self.active_filter),
                    Space::with_width(6),
                    filter_tab("GitHub", ProviderFilter::GitHub, &self.active_filter),
                    Space::with_width(6),
                    filter_tab("Bitbucket", ProviderFilter::Bitbucket, &self.active_filter),
                    Space::with_width(16),
                    text_input("Search repos…", &self.search_query)
                        .on_input(Message::REPOS_SearchChanged)
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

        // Status message
        let toast: Element<Message> = if let Some((ok, msg)) = &self.status_msg {
            let (color, bg) = if *ok {
                (GREEN, GREEN_BG)
            } else {
                (RED, RED_BG)
            };
            container(
                row![
                    container(Space::with_width(6)).width(6).height(6).style(
                        move |_: &iced::Theme| container::Style {
                            background: Some(color.into()),
                            border: Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    ),
                    Space::with_width(8),
                    text(msg.as_str()).size(12).color(TEXT_SECONDARY),
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
        } else {
            Space::with_height(0).into()
        };

        // Main body
        let body: Element<Message> = if self.remote_repos.is_empty() && !self.fetching {
            self.empty_state()
        } else if self.fetching {
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
            let filtered: Vec<&RemoteRepo> = self
                .remote_repos
                .iter()
                .filter(|r| {
                    let provider_match = match &self.active_filter {
                        ProviderFilter::All => true,
                        ProviderFilter::GitHub => r.provider == Provider::GitHub,
                        ProviderFilter::Bitbucket => r.provider == Provider::Bitbucket,
                    };
                    let search_match = self.search_query.is_empty()
                        || r.name
                            .to_lowercase()
                            .contains(&self.search_query.to_lowercase())
                        || r.full_name
                            .to_lowercase()
                            .contains(&self.search_query.to_lowercase());
                    provider_match && search_match
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
                let count_label = text(format!(
                    "{} repo{}",
                    filtered.len(),
                    if filtered.len() == 1 { "" } else { "s" }
                ))
                .size(11)
                .color(TEXT_MUTED);

                let cards: Vec<Element<Message>> =
                    filtered.iter().map(|r| self.repo_card(r)).collect();

                column![
                    count_label,
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
                Space::with_height(if !self.remote_repos.is_empty() { 10 } else { 0 }),
                toast,
                Space::with_height(if self.status_msg.is_some() { 12 } else { 0 }),
                body,
                Space::with_height(24),
            ]
            .spacing(0)
            .padding(Padding::from([22, 24])),
        )
        .into()
    }

    // ── SSH pill ──────────────────────────────────────────────────────────

    fn ssh_pill<'a>(&self, label: &'a str, status: &'a SshStatus) -> Element<'a, Message> {
        let (dot_color, status_text, bg) = match status {
            SshStatus::Unknown => (TEXT_MUTED, "not checked", BG_SURFACE),
            SshStatus::Connected => (GREEN, "connected", GREEN_BG),
            SshStatus::Failed(_) => (RED, "no SSH key", RED_BG),
        };
        container(
            row![
                container(Space::with_width(6))
                    .width(6)
                    .height(6)
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(dot_color.into()),
                        border: Border {
                            radius: 3.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
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

    // ── Empty state ───────────────────────────────────────────────────────

    fn empty_state(&self) -> Element<'_, Message> {
        let github_hint = container(
            column![
                row![
                    text("GitHub").size(12).color(TEAL),
                    Space::with_width(8),
                    text("ssh -T git@github.com").size(11).color(TEXT_MUTED),
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
                    text("ssh -T git@bitbucket.org").size(11).color(TEXT_MUTED),
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
            text("i").size(10).color(YELLOW),
            Space::with_width(8),
            text("DevPanel uses the GitHub CLI (gh) if installed, otherwise falls back to git ls-remote via SSH. Run \"Check SSH\" first, then \"Fetch Repos\".").size(11).color(TEXT_MUTED),
        ].align_y(Alignment::Center))
        .padding(Padding::from([10, 14])).width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(YELLOW_BG.into()), border: Border { color: YELLOW_BORDER, width: 1.0, radius: 8.0.into() }, ..Default::default()
        });

        container(column![
            text("No repos fetched yet").size(15).color(TEXT_SECONDARY),
            Space::with_height(4),
            text("Click \"Check SSH\" to verify your keys, then \"Fetch Repos\" to list your remote repositories.")
                .size(13).color(TEXT_MUTED),
            Space::with_height(20),
            github_hint,
            Space::with_height(8),
            bitbucket_hint,
            Space::with_height(16),
            note,
        ].spacing(0))
        .width(Length::Fill)
        .padding(Padding::from([8, 0]))
        .into()
    }

    // ── Single repo card ──────────────────────────────────────────────────

    fn repo_card<'a>(&self, repo: &'a RemoteRepo) -> Element<'a, Message> {
        let provider_badge = container(
            text(repo.provider.label())
                .size(10)
                .color(repo.provider.color()),
        )
        .padding(Padding::from([3, 8]))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(repo.provider.bg().into()),
            border: Border {
                color: repo.provider.border(),
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

        let name_row = row![
            text(repo.name.as_str()).size(14).color(TEXT_PRIMARY),
            Space::with_width(8),
            provider_badge,
            Space::with_width(6),
            cloned_badge,
            Space::with_width(Length::Fill),
        ]
        .align_y(Alignment::Center);

        let full_name_row = row![text(repo.full_name.as_str()).size(11).color(TEXT_MUTED),]
            .align_y(Alignment::Center);

        let ssh_row = row![
            text("SSH").size(10).color(TEXT_MUTED),
            Space::with_width(6),
            text(repo.ssh_url.as_str()).size(11).color(BORDER_MED),
        ]
        .align_y(Alignment::Center);

        let clone_btn: Element<Message> = if repo.is_cloning {
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
            button(text("Open").size(12).color(TEAL))
                .on_press(Message::REPOS_OpenCloned(repo.name.clone()))
                .padding(Padding::from([7, 16]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(TEAL_HOVER.into()),
                        text_color: TEAL,
                        border: Border {
                            color: TEAL_BORDER,
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: Some(TEAL_BG.into()),
                        text_color: TEAL,
                        border: Border {
                            color: TEAL_BORDER,
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    },
                })
                .into()
        } else {
            let url = repo.ssh_url.clone();
            let name = repo.name.clone();
            button(text("Clone").size(12).color(GREEN))
                .on_press(Message::REPOS_Clone { ssh_url: url, name })
                .padding(Padding::from([7, 16]))
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => iced::widget::button::Style {
                        background: Some(GREEN_HOVER.into()),
                        text_color: GREEN,
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
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
                full_name_row,
                Space::with_height(6),
                ssh_row,
                Space::with_height(14),
                thin_line(),
                Space::with_height(12),
                row![Space::with_width(Length::Fill), clone_btn].align_y(Alignment::Center),
            ]
            .spacing(0),
        )
        .padding(Padding::from([16, 18]))
        .width(Length::Fill)
        .style(card_style())
        .into()
    }
}

// ── Style helpers ─────────────────────────────────────────────────────────

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
    let msg = Message::REPOS_SetFilter(filter);
    button(text(label).size(11).color(color))
        .on_press(msg)
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

// ── Async tasks ───────────────────────────────────────────────────────────

pub struct SshCheckResult {
    pub github_ok: bool,
    pub github_msg: String,
    pub bb_ok: bool,
    pub bb_msg: String,
}

pub async fn check_ssh() -> SshCheckResult {
    let (gok, gmsg) = check_ssh_host("git@github.com").await;
    let (bok, bmsg) = check_ssh_host("git@bitbucket.org").await;
    SshCheckResult {
        github_ok: gok,
        github_msg: gmsg,
        bb_ok: bok,
        bb_msg: bmsg,
    }
}

async fn check_ssh_host(host_str: &str) -> (bool, String) {
    // ssh -o BatchMode=yes -o ConnectTimeout=8 -T git@github.com
    // GitHub returns exit code 1 but stderr "Hi username!" — that's success
    // Bitbucket returns exit code 1 but stderr "logged in as username" — success
    let out = tokio::process::Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            "-o",
            "StrictHostKeyChecking=no",
            "-T",
            host_str,
        ])
        .output()
        .await;

    match out {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_lowercase();
            let stdout = String::from_utf8_lossy(&o.stdout).to_lowercase();
            let combined = format!("{}{}", stderr, stdout);
            // GitHub says "hi <user>!", Bitbucket says "logged in as"
            let connected = combined.contains("hi ")
                || combined.contains("logged in as")
                || combined.contains("successfully authenticated")
                || o.status.success();
            if connected {
                // Try to extract username
                let username = extract_ssh_username(&combined);
                (true, username)
            } else {
                (
                    false,
                    combined
                        .trim()
                        .lines()
                        .next()
                        .unwrap_or("no key")
                        .to_string(),
                )
            }
        }
        Err(e) => (false, e.to_string()),
    }
}

fn extract_ssh_username(msg: &str) -> String {
    // "hi username!" or "logged in as username"
    if let Some(after_hi) = msg.split("hi ").nth(1) {
        let name = after_hi.split(['!', ' ', '\n']).next().unwrap_or("").trim();
        if !name.is_empty() {
            return format!("@{}", name);
        }
    }
    if let Some(after_as) = msg.split("logged in as ").nth(1) {
        let name = after_as.split(['.', ' ', '\n']).next().unwrap_or("").trim();
        if !name.is_empty() {
            return format!("@{}", name);
        }
    }
    "connected".to_string()
}

/// Fetch remote repositories from GitHub and Bitbucket.
/// Tries `gh` CLI first (most reliable), falls back to `git ls-remote --symrefs`.
pub async fn fetch_remote_repos(repos_root: String) -> Vec<RemoteRepo> {
    let mut repos: Vec<RemoteRepo> = Vec::new();

    // ── GitHub ────────────────────────────────────────────────────────────
    let gh_repos = fetch_github_repos().await;
    repos.extend(gh_repos);

    // ── Bitbucket ─────────────────────────────────────────────────────────
    let bb_repos = fetch_bitbucket_repos().await;
    repos.extend(bb_repos);

    // Mark already-cloned repos
    for repo in &mut repos {
        let local_path = std::path::PathBuf::from(&repos_root).join(&repo.name);
        repo.is_cloned = local_path.exists();
    }

    repos.sort_by(|a, b| {
        // Uncloned first, then alphabetical
        match (a.is_cloned, b.is_cloned) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.name.cmp(&b.name),
        }
    });

    repos
}

async fn fetch_github_repos() -> Vec<RemoteRepo> {
    // Try gh CLI first
    if let Some(repos) = try_gh_cli().await {
        return repos;
    }
    // Fallback: SSH key check passed so try git ls-remote with known org/user
    // We can get the username from ssh -T
    if let Some(username) = get_github_username_via_ssh().await {
        return fetch_github_via_ls_remote(&username).await;
    }
    Vec::new()
}

async fn try_gh_cli() -> Option<Vec<RemoteRepo>> {
    // gh repo list --json name,sshUrl,fullName --limit 200
    let out = tokio::process::Command::new("gh")
        .args([
            "repo",
            "list",
            "--json",
            "name,sshUrl,nameWithOwner",
            "--limit",
            "200",
        ])
        .output()
        .await
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let json = String::from_utf8_lossy(&out.stdout);
    if json.trim().is_empty() || json.trim() == "[]" {
        return None;
    }

    let repos = parse_gh_json(&json);
    if repos.is_empty() {
        None
    } else {
        Some(repos)
    }
}

fn parse_gh_json(json: &str) -> Vec<RemoteRepo> {
    // Manual JSON parsing — no serde dependency needed for this flat structure
    // Expected: [{"name":"foo","nameWithOwner":"user/foo","sshUrl":"git@github.com:user/foo.git"}, ...]
    let mut repos = Vec::new();
    let trimmed = json.trim().trim_start_matches('[').trim_end_matches(']');

    for obj in split_json_objects(trimmed) {
        let name = extract_json_str(&obj, "name").unwrap_or_default();
        let full_name = extract_json_str(&obj, "nameWithOwner").unwrap_or_default();
        let ssh_url = extract_json_str(&obj, "sshUrl").unwrap_or_default();

        if name.is_empty() || ssh_url.is_empty() {
            continue;
        }

        repos.push(RemoteRepo {
            name,
            full_name: if full_name.is_empty() {
                ssh_url.clone()
            } else {
                full_name
            },
            ssh_url,
            provider: Provider::GitHub,
            is_cloned: false,
            is_cloning: false,
        });
    }
    repos
}

async fn get_github_username_via_ssh() -> Option<String> {
    let out = tokio::process::Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            "-T",
            "git@github.com",
        ])
        .output()
        .await
        .ok()?;
    let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
    // "Hi username! You have successfully authenticated"
    if let Some(after) = stderr.split("hi ").nth(1) {
        let username = after.split(['!', ' ']).next()?.trim().to_string();
        if !username.is_empty() {
            return Some(username);
        }
    }
    None
}

async fn fetch_github_via_ls_remote(_username: &str) -> Vec<RemoteRepo> {
    // Without the gh CLI, we cannot list all repos via SSH alone.
    // The user should install the GitHub CLI: https://cli.github.com/
    Vec::new()
}

async fn fetch_bitbucket_repos() -> Vec<RemoteRepo> {
    // Bitbucket doesn't have a CLI equivalent to gh, so we use the REST API v2
    // authenticated via SSH agent — but REST needs HTTP auth, not SSH.
    // Best approach: parse ~/.ssh/config or known local git repos for bitbucket remotes,
    // and also try the bitbucket API with app passwords if available.
    // For now: scan ~/.gitconfig and local project dirs for bitbucket remotes,
    // plus try `git ls-remote` on configured remotes.
    let repos = scan_local_gitconfigs_for_bitbucket().await;
    repos
}

async fn scan_local_gitconfigs_for_bitbucket() -> Vec<RemoteRepo> {
    // Scan ~/projects/ for any existing repos that have bitbucket remotes
    // and report them as "known" bitbucket repos (already cloned)
    // Also check ~/.gitconfig for any global remote entries
    let repos = Vec::new();
    let home = get_home_dir();

    // Check ~/.ssh/config for bitbucket hosts
    let ssh_config_path = home.join(".ssh").join("config");
    if let Ok(content) = tokio::fs::read_to_string(&ssh_config_path).await {
        let bb_user = extract_bitbucket_user_from_ssh_config(&content);
        if let Some(username) = bb_user {
            // We have a configured bitbucket user — try to list via API
            if let Some(bb_repos) = try_bitbucket_api(&username).await {
                return bb_repos;
            }
        }
    }

    repos
}

async fn try_bitbucket_api(username: &str) -> Option<Vec<RemoteRepo>> {
    // Bitbucket Cloud REST API v2 — uses HTTP Basic with app password
    // We can't do this without credentials, but we can check if bb CLI or env var is set
    // Check for BITBUCKET_TOKEN env var
    let token = std::env::var("BITBUCKET_TOKEN").ok()?;

    let url = format!(
        "https://api.bitbucket.org/2.0/repositories/{}?pagelen=100&role=member",
        username
    );

    let out = tokio::process::Command::new("curl")
        .args(["-s", "-u", &format!("{}:{}", username, token), &url])
        .output()
        .await
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let json = String::from_utf8_lossy(&out.stdout);
    let repos = parse_bitbucket_api_json(&json, username);
    if repos.is_empty() {
        None
    } else {
        Some(repos)
    }
}

fn parse_bitbucket_api_json(json: &str, username: &str) -> Vec<RemoteRepo> {
    let mut repos = Vec::new();
    // Extract "slug" fields from Bitbucket API response
    // {"values": [{"slug": "my-repo", "full_name": "user/my-repo", ...}]}
    if let Some(values_start) = json.find("\"values\"") {
        let after = &json[values_start..];
        for obj in split_json_objects(after) {
            let slug = extract_json_str(&obj, "slug").unwrap_or_default();
            let full_name = extract_json_str(&obj, "full_name").unwrap_or_default();
            if slug.is_empty() {
                continue;
            }
            let ssh_url = format!("git@bitbucket.org:{}/{}.git", username, slug);
            repos.push(RemoteRepo {
                name: slug.clone(),
                full_name: if full_name.is_empty() {
                    format!("{}/{}", username, slug)
                } else {
                    full_name
                },
                ssh_url,
                provider: Provider::Bitbucket,
                is_cloned: false,
                is_cloning: false,
            });
        }
    }
    repos
}

fn extract_bitbucket_user_from_ssh_config(content: &str) -> Option<String> {
    let mut in_bitbucket = false;
    for line in content.lines() {
        let t = line.trim().to_lowercase();
        if t.starts_with("host") && t.contains("bitbucket") {
            in_bitbucket = true;
        }
        if in_bitbucket && t.starts_with("user ") {
            return Some(t["user ".len()..].trim().to_string());
        }
        if in_bitbucket && t.starts_with("host ") && !t.contains("bitbucket") {
            in_bitbucket = false;
        }
    }
    None
}

/// Clone a repository into repos_root/<name>.
pub async fn clone_repo(
    ssh_url: String,
    name: String,
    repos_root: String,
) -> (bool, String, String) {
    let dest = std::path::PathBuf::from(&repos_root).join(&name);

    if dest.exists() {
        return (
            false,
            format!("{} already exists in projects", name),
            ssh_url,
        );
    }

    let out = tokio::process::Command::new("git")
        .args(["clone", &ssh_url, dest.to_string_lossy().as_ref()])
        .output()
        .await;

    match out {
        Ok(o) if o.status.success() => (
            true,
            format!("Cloned {} into ~/projects/{}", name, name),
            ssh_url,
        ),
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            let msg = stderr.lines().last().unwrap_or("clone failed").to_string();
            (false, format!("Clone failed: {}", msg), ssh_url)
        }
        Err(e) => (false, format!("git not found: {}", e), ssh_url),
    }
}

// ── JSON helpers (no external crate needed) ───────────────────────────────

fn split_json_objects(input: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut depth = 0i32;
    let mut start = None;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        objects.push(input[s..=i].to_string());
                        start = None;
                    }
                }
            }
            '"' => {
                // Skip string contents
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' {
                        i += 1;
                    } else if chars[i] == '"' {
                        break;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    objects
}

fn extract_json_str(obj: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = obj.find(&pattern)?;
    let after_key = &obj[idx + pattern.len()..];
    let colon = after_key.find(':')?;
    let after_colon = after_key[colon + 1..].trim();
    if !after_colon.starts_with('"') {
        return None;
    }
    let inner = &after_colon[1..];
    let mut result = String::new();
    let mut chars = inner.chars();
    loop {
        match chars.next()? {
            '\\' => match chars.next()? {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                'n' => result.push('\n'),
                c => {
                    result.push('\\');
                    result.push(c);
                }
            },
            '"' => break,
            c => result.push(c),
        }
    }
    Some(result)
}

fn get_home_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/root"))
}
