// src/sudo_prompt.rs
// Sudo password modal: in-memory session cache + optional encrypted-at-rest storage.
// Storage is XOR-obfuscated (not true encryption — it just prevents casual plaintext
// reads). Users who need real security should use sudoers NOPASSWD instead.
//
// In dry-run (dev) mode every sudo command is logged and skipped — nothing
// on the host system is touched.

use crate::core::dry_run;
use crate::core::theme::*;
use crate::messages::Message;
use iced::widget::{Space, button, checkbox, column, container, row, text, text_input};
use iced::{Alignment, Border, Color, Element, Length, Padding};
use std::path::PathBuf;

const OBFUSCATION_KEY: &[u8] = b"devpanel_xor_v1_";

fn obfuscate(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ OBFUSCATION_KEY[i % OBFUSCATION_KEY.len()])
        .collect()
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".config")
        .join("devpanel")
        .join("sudo.dat")
}

pub fn load_saved_password() -> Option<String> {
    let path = config_path();
    let raw = std::fs::read(&path).ok()?;
    let decoded = obfuscate(&raw);
    String::from_utf8(decoded).ok()
}

pub fn save_password(password: &str) {
    if dry_run::active() {
        dry_run::log("save_password — skipped in dry-run mode");
        return;
    }
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let encoded = obfuscate(password.as_bytes());
    let _ = std::fs::write(&path, encoded);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
}

pub fn clear_saved_password() {
    if dry_run::active() {
        dry_run::log("clear_saved_password — skipped in dry-run mode");
        return;
    }
    let _ = std::fs::remove_file(config_path());
}

pub fn has_saved_password() -> bool {
    config_path().exists()
}

pub async fn validate_sudo_password(password: String) -> bool {
    if dry_run::active() {
        dry_run::log("validate_sudo_password — auto-passing in dry-run mode");
        return !password.is_empty();
    }
    use tokio::io::AsyncWriteExt;
    let result = tokio::process::Command::new("sudo")
        .args(["-S", "-k", "true"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    match result {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(format!("{}\n", password).as_bytes()).await;
            }
            child.wait().await.map(|s| s.success()).unwrap_or(false)
        }
        Err(_) => false,
    }
}

pub async fn sudo_cmd_with_password(password: &str, args: &[&str]) -> Result<String, String> {
    if dry_run::active() {
        let cmd_str = args.join(" ");
        dry_run::log(&format!("sudo_cmd_with_password: sudo {}", cmd_str));
        return dry_run::ok(&format!("sudo {}", cmd_str));
    }

    use tokio::io::AsyncWriteExt;
    let mut child = tokio::process::Command::new("sudo")
        .arg("-S")
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .await
            .map_err(|e| e.to_string())?;
    }

    let output = child.wait_with_output().await.map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let clean = stderr
            .lines()
            .filter(|l| !l.trim_start().starts_with("[sudo]") && !l.contains("password for"))
            .collect::<Vec<_>>()
            .join("\n");
        Err(if clean.trim().is_empty() {
            format!("sudo exited with status {}", output.status)
        } else {
            clean
        })
    }
}

pub async fn sudo_tee_append_with_password(
    password: &str,
    path: &str,
    content: &str,
) -> Result<(), String> {
    if dry_run::active() {
        let preview: String = content.chars().take(120).collect();
        dry_run::log(&format!(
            "sudo_tee_append_with_password: would append {} bytes to {}\n  preview: {}{}",
            content.len(),
            path,
            preview,
            if content.len() > 120 { "…" } else { "" }
        ));
        return Ok(());
    }

    use tokio::io::AsyncWriteExt;
    let script = format!("tee -a {}", shell_escape(path));
    let mut child = tokio::process::Command::new("sudo")
        .args(["-S", "sh", "-c", &script])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        stdin
            .write_all(content.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
    }

    let output = child.wait_with_output().await.map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "sudo tee failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalState {
    Hidden,
    Asking { pending_action: PendingAction },
    Validating,
    Failed,
}

impl Default for ModalState {
    fn default() -> Self {
        Self::Hidden
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    ServiceControl {
        service: String,
        action: String,
    },
    PhpSwitch(String),
    RestartAll,
    PhpInstall(String),
    PhpRemove(String),
    VHostAdd {
        server_name: String,
        document_root: String,
        php_version: Option<String>,
    },
    VHostEdit {
        index: usize,
        server_name: String,
        document_root: String,
        php_version: Option<String>,
    },
    VHostDelete {
        index: usize,
    },
    SaveConfig {
        path: String,
        content: String,
    },
    ApacheModToggle {
        name: String,
        enable: bool,
    },
    AptInstall {
        package: String,
    },
    AptRemove {
        package: String,
    },
    FirstRunInstall,
}

#[derive(Debug, Clone, Default)]
pub struct SudoModal {
    pub state: ModalState,
    pub password_input: String,
    pub save_password: bool,
    pub show_password: bool,
    pub cached_password: Option<String>,
}

impl SudoModal {
    pub fn new() -> Self {
        let cached = load_saved_password();
        Self {
            state: ModalState::Hidden,
            password_input: String::new(),
            save_password: has_saved_password(),
            show_password: false,
            cached_password: cached,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.state != ModalState::Hidden
    }

    pub fn get_password(&self) -> Option<String> {
        self.cached_password.clone()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let overlay_bg = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.72,
        };

        let dev_banner: Element<Message> = if dry_run::active() {
            container(
                row![
                    text("DEV").size(9).color(YELLOW),
                    Space::with_width(6),
                    text("Dry-run mode — any password is accepted, no real commands run.")
                        .size(11)
                        .color(TEXT_MUTED),
                ]
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([7, 12]))
            .width(Length::Fill)
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
                    color: Color {
                        r: 0.240,
                        g: 0.200,
                        b: 0.050,
                        a: 1.0,
                    },
                    width: 1.0,
                    radius: 7.0.into(),
                },
                ..Default::default()
            })
            .into()
        } else {
            Space::with_height(0).into()
        };

        let modal_content = match &self.state {
            ModalState::Hidden => return Space::with_width(0).into(),

            ModalState::Asking { .. } | ModalState::Failed => {
                let error_msg: Element<Message> = if self.state == ModalState::Failed {
                    container(
                        row![
                            container(text("x").size(10).color(Color::WHITE))
                                .padding(Padding::from([3, 6]))
                                .style(|_: &iced::Theme| container::Style {
                                    background: Some(RED.into()),
                                    border: Border {
                                        radius: 20.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            Space::with_width(10),
                            text("Incorrect password. Please try again.")
                                .size(13)
                                .color(TEXT_PRIMARY),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .padding(Padding::from([10, 12]))
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(
                            Color {
                                r: 0.200,
                                g: 0.060,
                                b: 0.055,
                                a: 1.0,
                            }
                            .into(),
                        ),
                        border: Border {
                            color: Color {
                                r: 0.300,
                                g: 0.090,
                                b: 0.080,
                                a: 1.0,
                            },
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    })
                    .into()
                } else {
                    Space::with_height(0).into()
                };

                let pass_input = text_input(
                    if self.show_password {
                        "sudo password"
                    } else {
                        "(hidden)"
                    },
                    &self.password_input,
                )
                .on_input(Message::SudoPasswordChanged)
                .on_submit(Message::SudoSubmit)
                .secure(!self.show_password)
                .padding(11)
                .size(13);

                let show_btn =
                    button(text(if self.show_password { "Hide" } else { "Show" }).size(12))
                        .on_press(Message::SudoToggleShow(!self.show_password))
                        .padding(Padding::from([11, 14]))
                        .style(ghost_btn_style());

                let input_row = row![pass_input.width(Length::Fill), show_btn]
                    .spacing(8)
                    .align_y(Alignment::Center);

                let save_check = checkbox("Save password for this session", self.save_password)
                    .on_toggle(Message::SudoToggleSave)
                    .size(14)
                    .text_size(13);

                let submit_btn = button(text("Unlock").size(13))
                    .on_press(Message::SudoSubmit)
                    .padding(Padding::from([10, 28]))
                    .style(teal_btn_style());

                let cancel_btn = button(text("Cancel").size(13))
                    .on_press(Message::SudoCancel)
                    .padding(Padding::from([10, 18]))
                    .style(ghost_btn_style());

                let divider = container(Space::with_height(1))
                    .width(Length::Fill)
                    .height(1)
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(BORDER_SUBTLE.into()),
                        ..Default::default()
                    });

                column![
                    container(
                        row![
                            container(text("sudo").size(10).color(TEAL))
                                .padding(Padding::from([3, 8]))
                                .style(|_: &iced::Theme| container::Style {
                                    background: Some(
                                        Color {
                                            r: 0.040,
                                            g: 0.160,
                                            b: 0.150,
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
                            Space::with_width(10),
                            text("Authentication Required").size(16).color(TEXT_PRIMARY),
                        ]
                        .align_y(Alignment::Center),
                    ),
                    Space::with_height(6),
                    text("Enter your sudo password to continue.")
                        .size(13)
                        .color(TEXT_SECONDARY),
                    Space::with_height(12),
                    dev_banner,
                    Space::with_height(20),
                    divider,
                    Space::with_height(16),
                    error_msg,
                    Space::with_height(if self.state == ModalState::Failed {
                        12
                    } else {
                        0
                    }),
                    text("Password").size(11).color(TEXT_MUTED),
                    Space::with_height(6),
                    input_row,
                    Space::with_height(14),
                    save_check,
                    Space::with_height(24),
                    row![submit_btn, Space::with_width(10), cancel_btn].align_y(Alignment::Center),
                ]
                .spacing(0)
            }

            ModalState::Validating => column![
                text("Authentication Required").size(16).color(TEXT_PRIMARY),
                Space::with_height(16),
                text("Verifying...").size(13).color(TEXT_SECONDARY),
            ]
            .spacing(0),
        };

        let card = container(modal_content.padding(30))
            .width(420)
            .style(|_: &iced::Theme| container::Style {
                background: Some(BG_ELEVATED.into()),
                border: Border {
                    color: BORDER_MED,
                    width: 1.0,
                    radius: 16.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color {
                        a: 0.7,
                        ..Color::BLACK
                    },
                    offset: iced::Vector::new(0.0, 12.0),
                    blur_radius: 48.0,
                },
                ..Default::default()
            });

        container(
            container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(overlay_bg.into()),
            ..Default::default()
        })
        .into()
    }
}

fn teal_btn_style()
-> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    |_, status| match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            iced::widget::button::Style {
                background: Some(TEAL_DIM.into()),
                text_color: Color::WHITE,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(TEAL.into()),
            text_color: Color {
                r: 0.05,
                g: 0.05,
                b: 0.06,
                a: 1.0,
            },
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
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
                background: Some(BG_HOVER.into()),
                text_color: TEXT_PRIMARY,
                border: Border {
                    color: BORDER_MED,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        }
        _ => iced::widget::button::Style {
            background: Some(BG_CARD.into()),
            text_color: TEXT_SECONDARY,
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        },
    }
}
