// src/main.rs -- DevPanel: Apache + SSH + VirtualHost manager for Debian/Ubuntu

mod sudo_prompt;
mod tabs;
mod theme;

use sudo_prompt::{
    clear_saved_password, save_password, sudo_cmd_with_password, sudo_tee_append_with_password,
    validate_sudo_password, ModalState, PendingAction, SudoModal,
};
use tabs::apache_touch::{ApacheTouchTab, LogEntry};
use tabs::dashboard::DashboardTab;
use tabs::ssh_keys::{KeyEntry, KeyType, SshKeysTab, StatusKind};
use tabs::tools::ToolsTab;
use theme::*;

use iced::widget::{button, column, container, row, stack, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding, Task, Theme};
use std::path::PathBuf;
use tokio::process::Command;

// ── Active tab ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    ApacheTouch,
    SshKeys,
    Tools,
}

// ── Top-level messages ────────────────────────────────────────────────────

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    SelectTab(Tab),

    // Dashboard -- service control
    StartApache,
    StopApache,
    RestartApache,
    StartMySQL,
    StopMySQL,
    RestartMySQL,
    ServiceResult {
        service: String,
        action: String,
        success: bool,
        output: String,
    },

    // Dashboard -- PHP
    SwitchPHPVersion(String),
    PhpSwitchResult(bool, String),
    ShowPHPInfo,

    // Dashboard -- quick actions
    OpenLocalhost,
    OpenPhpMyAdmin,
    OpenWebRoot,
    OpenApacheConfig,
    OpenMySQLConfig,
    OpenPHPConfig,
    OpenPhpIni,
    EditHosts,
    RestartAll,
    ClearCache,

    // Dashboard -- status refresh
    RefreshStatus,
    StatusRefreshed {
        apache: bool,
        mysql: bool,
        php: Option<String>,
        php_versions: Vec<String>,
    },

    // ApacheTouch
    AT_ProjectNameChanged(String),
    AT_BaseDirChanged(String),
    AT_ApacheConfChanged(String),
    AT_AuthJsonChanged(String),
    AT_BrowseAuthJson,
    AT_RunSetup,
    AT_SetupDone(Vec<LogEntry>),
    AT_ClearLog,

    // SSH Keys
    SSH_EmailChanged(String),
    SSH_KeyNameChanged(String),
    SSH_KeyTypeChanged(KeyType),
    SSH_PassphraseChanged(String),
    SSH_TogglePassphrase(bool),
    SSH_GenerateKey,
    SSH_GenerateDone(bool, String),
    SSH_AddExisting,
    SSH_AddExistingDone(bool, String),
    SSH_OpenDir,
    SSH_ListKeys,
    SSH_KeysListed(Vec<KeyEntry>),

    // Tools
    TOOLS_ScanPhp,
    TOOLS_ScanDone(Vec<(String, tabs::tools::PhpStatus, bool)>),
    TOOLS_InstallPhp(String),
    TOOLS_RemovePhp(String),
    TOOLS_PhpOpDone(bool, String),
    TOOLS_OpenMysqlCli,
    TOOLS_OpenMariadbCli,
    TOOLS_OpenMysqlSocket,
    TOOLS_ClearLog,
    TOOLS_ClearToast,
    TOOLS_CopyFixCommands(String),
    TOOLS_CopyDone,

    // Sudo modal
    Sudo_PasswordChanged(String),
    Sudo_ToggleShow(bool),
    Sudo_ToggleSave(bool),
    Sudo_Submit,
    Sudo_ValidationResult(bool),
    Sudo_Cancel,
    Sudo_ClearSaved,
}

// ── App state ─────────────────────────────────────────────────────────────

struct App {
    active_tab: Tab,
    dashboard: DashboardTab,
    apache_touch: ApacheTouchTab,
    ssh_keys: SshKeysTab,
    tools: ToolsTab,
    toast: Option<Toast>,
    sudo: SudoModal,
    /// Stash the pending action before transitioning to Validating state
    sudo_pending_action: Option<PendingAction>,
}

#[derive(Clone)]
struct Toast {
    message: String,
    ok: bool,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            active_tab: Tab::Dashboard,
            dashboard: DashboardTab::new(),
            apache_touch: ApacheTouchTab::new(),
            ssh_keys: SshKeysTab::new(),
            tools: ToolsTab::new(),
            toast: None,
            sudo: SudoModal::new(),
            sudo_pending_action: None,
        };
        (app, Task::perform(probe_services(), |r| r))
    }

    // ── Trigger a sudo action -- shows modal if no cached password ─────────

    fn trigger_sudo(&mut self, action: PendingAction) -> Task<Message> {
        if let Some(password) = self.sudo.get_password() {
            // Already have password -- execute immediately
            self.dispatch_sudo_action(action, password)
        } else {
            // Stash the action and show the modal
            self.sudo_pending_action = Some(action.clone());
            self.sudo.state = ModalState::Asking {
                pending_action: action,
            };
            self.sudo.password_input.clear();
            Task::none()
        }
    }

    // ── Execute the pending action with a verified password ────────────────

    fn dispatch_sudo_action(&mut self, action: PendingAction, password: String) -> Task<Message> {
        match action {
            PendingAction::ServiceControl {
                service,
                action: svc_action,
            } => {
                let svc = Box::leak(service.into_boxed_str()) as &'static str;
                let act = Box::leak(svc_action.into_boxed_str()) as &'static str;
                Task::perform(run_service_cmd_with_pass(svc, act, password), |r| r)
            }
            PendingAction::PhpSwitch(version) => {
                Task::perform(switch_php_with_pass(version, password), |(ok, msg)| {
                    Message::PhpSwitchResult(ok, msg)
                })
            }
            PendingAction::ApacheTouchSetup => {
                let pn = self.apache_touch.project_name.trim().to_string();
                let aj = self.apache_touch.auth_json_path.trim().to_string();
                let bd = self.apache_touch.base_dir.trim().to_string();
                let conf = self.apache_touch.apache_conf.trim().to_string();
                self.apache_touch.running = true;
                self.apache_touch.log.clear();
                Task::perform(
                    run_apache_touch(pn, aj, bd, conf, password),
                    Message::AT_SetupDone,
                )
            }
            PendingAction::RestartAll => Task::batch([
                Task::perform(
                    run_service_cmd_with_pass("apache2", "restart", password.clone()),
                    |r| r,
                ),
                Task::perform(
                    run_service_cmd_with_pass("mysql", "restart", password),
                    |r| r,
                ),
            ]),
            PendingAction::PhpInstall(version) => {
                Task::perform(apt_php_op(version, true, password), |(ok, msg)| {
                    Message::TOOLS_PhpOpDone(ok, msg)
                })
            }
            PendingAction::PhpRemove(version) => {
                Task::perform(apt_php_op(version, false, password), |(ok, msg)| {
                    Message::TOOLS_PhpOpDone(ok, msg)
                })
            }
        }
    }

    // ── Update ────────────────────────────────────────────────────────────

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            // ── Navigation ────────────────────────────────────────────────
            Message::SelectTab(tab) => {
                self.active_tab = tab.clone();
                if tab == Tab::Dashboard {
                    return Task::perform(probe_services(), |r| r);
                }
                if tab == Tab::SshKeys {
                    return Task::perform(list_ssh_keys(), Message::SSH_KeysListed);
                }
                if tab == Tab::Tools {
                    self.tools.scanning = true;
                    return Task::perform(
                        scan_php_versions(self.dashboard.active_php_version.clone()),
                        Message::TOOLS_ScanDone,
                    );
                }
                Task::none()
            }

            // ── Sudo modal ────────────────────────────────────────────────
            Message::Sudo_PasswordChanged(v) => {
                self.sudo.password_input = v;
                Task::none()
            }
            Message::Sudo_ToggleShow(v) => {
                self.sudo.show_password = v;
                Task::none()
            }
            Message::Sudo_ToggleSave(v) => {
                self.sudo.save_password = v;
                if !v {
                    clear_saved_password();
                }
                Task::none()
            }
            Message::Sudo_Cancel => {
                self.sudo.state = ModalState::Hidden;
                self.sudo.password_input.clear();
                // If setup was pending, mark it as not running
                self.apache_touch.running = false;
                Task::none()
            }
            Message::Sudo_Submit => {
                let pass = self.sudo.password_input.clone();
                if pass.is_empty() {
                    return Task::none();
                }
                // Transition to Validating (keep pending_action_stash intact)
                self.sudo.state = ModalState::Validating;
                Task::perform(validate_sudo_password(pass), Message::Sudo_ValidationResult)
            }
            Message::Sudo_ValidationResult(valid) => {
                if !valid {
                    // Restore Asking state with the stashed action
                    if let Some(action) = self.sudo_pending_action.clone() {
                        self.sudo.state = ModalState::Failed;
                        // Put the action back in the Asking variant for display
                        self.sudo.state = ModalState::Failed;
                        let _ = action; // keep stash intact for retry
                    } else {
                        self.sudo.state = ModalState::Failed;
                    }
                    self.sudo.password_input.clear();
                    return Task::none();
                }

                // Password correct -- cache it
                let password = self.sudo.password_input.clone();
                self.sudo.cached_password = Some(password.clone());
                self.sudo.password_input.clear();
                self.sudo.state = ModalState::Hidden;

                if self.sudo.save_password {
                    save_password(&password);
                }

                // Dispatch the stashed action
                if let Some(action) = self.sudo_pending_action.take() {
                    self.dispatch_sudo_action(action, password)
                } else {
                    Task::none()
                }
            }
            Message::Sudo_ClearSaved => {
                clear_saved_password();
                self.sudo.cached_password = None;
                self.sudo.save_password = false;
                self.toast = Some(Toast {
                    message: "Saved password cleared.".into(),
                    ok: true,
                });
                Task::none()
            }

            // ── Dashboard: status refresh ──────────────────────────────────
            Message::RefreshStatus => Task::perform(probe_services(), |r| r),
            Message::StatusRefreshed {
                apache,
                mysql,
                php,
                php_versions,
            } => {
                self.dashboard.update_status(apache, mysql, php);
                self.dashboard.set_php_versions(php_versions);
                Task::none()
            }

            // ── Dashboard: service control ────────────────────────────────
            Message::StartApache => self.trigger_sudo(PendingAction::ServiceControl {
                service: "apache2".into(),
                action: "start".into(),
            }),
            Message::StopApache => self.trigger_sudo(PendingAction::ServiceControl {
                service: "apache2".into(),
                action: "stop".into(),
            }),
            Message::RestartApache => self.trigger_sudo(PendingAction::ServiceControl {
                service: "apache2".into(),
                action: "restart".into(),
            }),
            Message::StartMySQL => self.trigger_sudo(PendingAction::ServiceControl {
                service: "mysql".into(),
                action: "start".into(),
            }),
            Message::StopMySQL => self.trigger_sudo(PendingAction::ServiceControl {
                service: "mysql".into(),
                action: "stop".into(),
            }),
            Message::RestartMySQL => self.trigger_sudo(PendingAction::ServiceControl {
                service: "mysql".into(),
                action: "restart".into(),
            }),

            Message::ServiceResult {
                service,
                action,
                success,
                output,
            } => {
                self.toast = Some(Toast {
                    message: if success {
                        format!("{} {}ed", service, action)
                    } else {
                        format!("Failed to {} {}: {}", action, service, output)
                    },
                    ok: success,
                });
                Task::perform(probe_services(), |r| r)
            }

            // ── Dashboard: PHP ────────────────────────────────────────────
            Message::SwitchPHPVersion(v) => self.trigger_sudo(PendingAction::PhpSwitch(v)),
            Message::PhpSwitchResult(ok, msg) => {
                self.toast = Some(Toast { message: msg, ok });
                Task::perform(probe_services(), |r| r)
            }
            Message::ShowPHPInfo => {
                let _ = open_url("http://localhost/phpinfo.php");
                Task::none()
            }

            // ── Dashboard: quick actions ──────────────────────────────────
            Message::OpenLocalhost => {
                let _ = open_url("http://localhost");
                Task::none()
            }
            Message::OpenPhpMyAdmin => {
                let _ = open_url("http://localhost/phpmyadmin");
                Task::none()
            }
            Message::OpenWebRoot => {
                let _ = xdg_open(&self.dashboard.web_root);
                Task::none()
            }
            Message::OpenApacheConfig => {
                let _ = xdg_open(&self.dashboard.apache_conf_dir);
                Task::none()
            }
            Message::OpenMySQLConfig => {
                let _ = xdg_open("/etc/mysql");
                Task::none()
            }
            Message::OpenPHPConfig => {
                let _ = xdg_open("/etc/php");
                Task::none()
            }
            Message::OpenPhpIni => {
                let _ = open_php_ini(&self.dashboard.active_php_version);
                Task::none()
            }
            Message::EditHosts => {
                let _ = xdg_open("/etc/hosts");
                Task::none()
            }
            Message::ClearCache => Task::none(),
            Message::RestartAll => self.trigger_sudo(PendingAction::RestartAll),

            // ── ApacheTouch ───────────────────────────────────────────────
            Message::AT_ProjectNameChanged(v) => {
                self.apache_touch.project_name = v;
                Task::none()
            }
            Message::AT_BaseDirChanged(v) => {
                self.apache_touch.base_dir = v;
                Task::none()
            }
            Message::AT_ApacheConfChanged(v) => {
                self.apache_touch.apache_conf = v;
                Task::none()
            }
            Message::AT_AuthJsonChanged(v) => {
                self.apache_touch.auth_json_path = v;
                Task::none()
            }
            Message::AT_BrowseAuthJson => {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.apache_touch.auth_json_path = path.to_string_lossy().to_string();
                }
                Task::none()
            }
            Message::AT_ClearLog => {
                self.apache_touch.log.clear();
                self.apache_touch.finished_ok = None;
                Task::none()
            }
            Message::AT_RunSetup => {
                if self.apache_touch.project_name.trim().is_empty() {
                    self.apache_touch.log = vec![LogEntry::err("Project name cannot be empty.")];
                    self.apache_touch.finished_ok = Some(false);
                    return Task::none();
                }
                self.trigger_sudo(PendingAction::ApacheTouchSetup)
            }
            Message::AT_SetupDone(entries) => {
                let ok = !entries
                    .iter()
                    .any(|e| matches!(e.kind, tabs::apache_touch::LogKind::Error));
                self.apache_touch.finished_ok = Some(ok);
                self.apache_touch.log = entries;
                self.apache_touch.running = false;
                Task::none()
            }

            // ── SSH Keys ──────────────────────────────────────────────────
            Message::SSH_EmailChanged(v) => {
                self.ssh_keys.email = v;
                Task::none()
            }
            Message::SSH_KeyNameChanged(v) => {
                self.ssh_keys.key_name = v;
                Task::none()
            }
            Message::SSH_KeyTypeChanged(t) => {
                self.ssh_keys.key_type = t;
                Task::none()
            }
            Message::SSH_PassphraseChanged(v) => {
                self.ssh_keys.passphrase = v;
                Task::none()
            }
            Message::SSH_TogglePassphrase(b) => {
                self.ssh_keys.show_passphrase = b;
                Task::none()
            }
            Message::SSH_GenerateKey => {
                let email = self.ssh_keys.email.clone();
                let name = self.ssh_keys.key_name.clone();
                let ktype = self.ssh_keys.key_type;
                let pass = self.ssh_keys.passphrase.clone();
                if email.trim().is_empty() || name.trim().is_empty() {
                    self.ssh_keys.status_message = "Email and key name are required.".into();
                    self.ssh_keys.status_kind = StatusKind::Error;
                    return Task::none();
                }
                Task::perform(generate_ssh_key(email, name, ktype, pass), |(ok, msg)| {
                    Message::SSH_GenerateDone(ok, msg)
                })
            }
            Message::SSH_GenerateDone(ok, msg) => {
                self.ssh_keys.status_message = msg;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                Task::perform(list_ssh_keys(), Message::SSH_KeysListed)
            }
            Message::SSH_AddExisting => {
                let ssh_dir = get_home().join(".ssh");
                if let Some(path) = rfd::FileDialog::new().set_directory(&ssh_dir).pick_file() {
                    return Task::perform(
                        ssh_add(path.to_string_lossy().to_string()),
                        |(ok, msg)| Message::SSH_AddExistingDone(ok, msg),
                    );
                }
                Task::none()
            }
            Message::SSH_AddExistingDone(ok, msg) => {
                self.ssh_keys.status_message = msg;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                Task::none()
            }
            Message::SSH_OpenDir => {
                let p = get_home().join(".ssh");
                let _ = xdg_open(&p.to_string_lossy());
                Task::none()
            }
            Message::SSH_ListKeys => Task::perform(list_ssh_keys(), Message::SSH_KeysListed),
            Message::SSH_KeysListed(keys) => {
                self.ssh_keys.keys_list = keys;
                Task::none()
            }

            // ── Tools ─────────────────────────────────────────────────────
            Message::TOOLS_ScanPhp => {
                self.tools.scanning = true;
                Task::perform(
                    scan_php_versions(self.dashboard.active_php_version.clone()),
                    Message::TOOLS_ScanDone,
                )
            }
            Message::TOOLS_ScanDone(results) => {
                self.tools.apply_scan(results);
                Task::none()
            }
            Message::TOOLS_InstallPhp(ver) => {
                self.tools
                    .push_log(true, format!("Queued install: PHP {}", ver));
                self.trigger_sudo(PendingAction::PhpInstall(ver))
            }
            Message::TOOLS_RemovePhp(ver) => {
                self.tools
                    .push_log(true, format!("Queued removal: PHP {}", ver));
                self.trigger_sudo(PendingAction::PhpRemove(ver))
            }
            Message::TOOLS_PhpOpDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.toast = Some(Toast { message: msg, ok });
                // Re-scan after install/remove
                self.tools.scanning = true;
                Task::batch([
                    Task::perform(probe_services(), |r| r),
                    Task::perform(
                        scan_php_versions(self.dashboard.active_php_version.clone()),
                        Message::TOOLS_ScanDone,
                    ),
                    Task::perform(
                        async {
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        },
                        |_| Message::TOOLS_ClearToast,
                    ),
                ])
            }
            Message::TOOLS_OpenMysqlCli => {
                let result = open_db_terminal("mysql", false);
                self.tools.db_status = match result {
                    Ok(cmd) => format!("Launched: {}", cmd),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            Message::TOOLS_OpenMariadbCli => {
                let result = open_db_terminal("mariadb", false);
                self.tools.db_status = match result {
                    Ok(cmd) => format!("Launched: {}", cmd),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            Message::TOOLS_OpenMysqlSocket => {
                let result = open_db_terminal("mysql", true);
                self.tools.db_status = match result {
                    Ok(cmd) => format!("Launched: {}", cmd),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            Message::TOOLS_ClearLog => {
                self.tools.install_log.clear();
                self.tools.db_status.clear();
                Task::none()
            }
            Message::TOOLS_ClearToast => {
                self.toast = None;
                Task::none()
            }
            Message::TOOLS_CopyFixCommands(commands) => {
                Task::perform(copy_to_clipboard_async(commands), |_| {
                    Message::TOOLS_CopyDone
                })
            }
            Message::TOOLS_CopyDone => {
                self.toast = Some(Toast {
                    message: "Commands copied to clipboard!".to_string(),
                    ok: true,
                });
                Task::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::TOOLS_ClearToast,
                )
            }
        }
    }

    // ── View ──────────────────────────────────────────────────────────────

    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.sidebar();

        let tab_content: Element<Message> = match &self.active_tab {
            Tab::Dashboard => self.dashboard.view(),
            Tab::ApacheTouch => self.apache_touch.view(),
            Tab::SshKeys => self.ssh_keys.view(),
            Tab::Tools => self.tools.view(),
        };

        // Toast notification — Apple-style pill
        let main_body: Element<Message> = if let Some(toast) = &self.toast {
            let (color, border_color) = if toast.ok {
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
            let icon = if toast.ok { "+" } else { "x" };
            let banner = container(
                row![
                    container(text(icon).size(11).color(Color::WHITE))
                        .padding(Padding::from([3, 7]))
                        .style(move |_: &iced::Theme| container::Style {
                            background: Some(color.into()),
                            border: Border {
                                radius: 20.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    Space::with_width(10),
                    text(format!("{}", toast.message))
                        .size(13)
                        .color(TEXT_PRIMARY),
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
            sidebar,
            container(main_body)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(BG_BASE.into()),
                    ..Default::default()
                }),
        ];

        // Overlay the sudo modal on top if visible
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

    // ── Sidebar ───────────────────────────────────────────────────────────

    fn sidebar(&self) -> Element<'_, Message> {
        // Logo area
        let logo = container(
            column![row![
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
            .align_y(Alignment::Center),]
            .spacing(0),
        )
        .padding(Padding::from([22, 16]));

        let nav = column![
            self.nav_item("Dashboard", Tab::Dashboard),
            self.nav_item("VirtualHost", Tab::ApacheTouch),
            self.nav_item("SSH Keys", Tab::SshKeys),
            self.nav_item("Tools", Tab::Tools),
        ]
        .spacing(2)
        .padding(Padding::from([0, 8]));

        // Sudo status indicator
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
                    .align_y(Alignment::Center),
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
                    .on_press(Message::Sudo_ClearSaved)
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

        let refresh_btn = button(
            row![
                text("R").size(11).color(TEXT_MUTED),
                Space::with_width(6),
                text("Refresh").size(12).color(TEXT_MUTED),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(Message::RefreshStatus)
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
        });

        let bottom = container(
            column![
                sudo_indicator,
                Space::with_height(10),
                refresh_btn,
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
        // Solid pre-computed teal tint — no alpha struct
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

        let (icon_char, icon_color): (&str, Color) = match &tab {
            Tab::Dashboard => ("*", if active { TEAL } else { TEXT_MUTED }),
            Tab::ApacheTouch => ("+", if active { TEAL } else { TEXT_MUTED }),
            Tab::SshKeys => ("#", if active { TEAL } else { TEXT_MUTED }),
            Tab::Tools => ("~", if active { TEAL } else { TEXT_MUTED }),
        };

        button(
            row![
                text(icon_char).size(12).color(icon_color),
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

// ── Divider helper ────────────────────────────────────────────────────────

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

// ── Entry point ───────────────────────────────────────────────────────────

static ICON_BYTES: &[u8] = include_bytes!("../icon.png");

fn load_window_icon() -> Option<iced::window::Icon> {
    use image::GenericImageView;
    let img = image::load_from_memory(ICON_BYTES).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = img.dimensions();
    iced::window::icon::from_rgba(rgba.into_raw(), w, h).ok()
}

fn make_fallback_icon() -> Option<iced::window::Icon> {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let cx = 15.5f32;
    let cy = 15.5f32;
    let r = 13.0f32;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let idx = ((y * size + x) * 4) as usize;
            rgba[idx + 3] = 0xFF;
            if dx * dx + dy * dy <= r * r {
                rgba[idx] = 0x33;
                rgba[idx + 1] = 0xBC;
                rgba[idx + 2] = 0xAC;
            }
        }
    }
    iced::window::icon::from_rgba(rgba, size, size).ok()
}

fn main() -> iced::Result {
    let icon = load_window_icon().or_else(make_fallback_icon);
    iced::application("DevPanel", App::update, App::view)
        .theme(|_| Theme::Dark)
        .window(iced::window::Settings {
            size: iced::Size::new(1040.0, 600.0),
            min_size: Some(iced::Size::new(820.0, 560.0)),
            icon,
            ..Default::default()
        })
        .run_with(App::new)
}

// =============================================================================
// Async tasks
// =============================================================================

async fn probe_services() -> Message {
    let apache = service_active("apache2").await;
    let mysql = service_active("mysql").await || service_active("mariadb").await;
    let (php, php_versions) = detect_php().await;
    Message::StatusRefreshed {
        apache,
        mysql,
        php,
        php_versions,
    }
}

async fn service_active(name: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", name])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn detect_php() -> (Option<String>, Vec<String>) {
    let mut versions = Vec::new();
    if let Ok(mut dir) = tokio::fs::read_dir("/usr/bin").await {
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(rest) = name.strip_prefix("php") {
                let t = rest.trim_start_matches('-');
                if t.contains('.') && t.len() <= 4 {
                    versions.push(name);
                }
            }
        }
    }
    versions.sort();
    let active = Command::new("php")
        .arg("--version")
        .output()
        .await
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            s.lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1))
                .map(|v| v.to_string())
        });
    (active, versions)
}

async fn run_service_cmd_with_pass(service: &str, action: &str, password: String) -> Message {
    let result = sudo_cmd_with_password(&password, &["systemctl", action, service]).await;
    Message::ServiceResult {
        service: service.to_string(),
        action: action.to_string(),
        success: result.is_ok(),
        output: result.err().unwrap_or_default(),
    }
}

async fn switch_php_with_pass(version: String, password: String) -> (bool, String) {
    let ver = version.trim_start_matches("php").to_string();
    let bin = format!("/usr/bin/php{}", ver);
    match sudo_cmd_with_password(&password, &["update-alternatives", "--set", "php", &bin]).await {
        Ok(_) => (true, format!("Switched to PHP {}", ver)),
        Err(e) => (false, e),
    }
}

async fn run_apache_touch(
    project_name: String,
    auth_json: String,
    base_dir: String,
    apache_conf: String,
    password: String,
) -> Vec<LogEntry> {
    let mut log = Vec::new();
    let project_path = PathBuf::from(&base_dir).join(&project_name);
    let domain = format!("{}.local", project_name);

    log.push(LogEntry::info(format!(
        "Setting up project: {}",
        project_name
    )));

    if !project_path.exists() {
        log.push(LogEntry::err(format!(
            "Directory {} does not exist.",
            project_path.display()
        )));
        return log;
    }
    log.push(LogEntry::ok(format!(
        "Directory found: {}",
        project_path.display()
    )));

    // .env
    let env_ex = project_path.join(".env.example");
    let env_dst = project_path.join(".env");
    if env_ex.exists() {
        match tokio::fs::copy(&env_ex, &env_dst).await {
            Ok(_) => log.push(LogEntry::ok(".env created from .env.example")),
            Err(e) => log.push(LogEntry::err(format!("Copy .env failed: {}", e))),
        }
    } else {
        log.push(LogEntry::warn(".env.example not found - skipping"));
    }

    // auth.json
    if !auth_json.is_empty() {
        let src = PathBuf::from(&auth_json);
        if src.exists() {
            let dst = project_path.join("auth.json");
            match tokio::fs::copy(&src, &dst).await {
                Ok(_) => log.push(LogEntry::ok("auth.json copied")),
                Err(e) => log.push(LogEntry::err(format!("Copy auth.json failed: {}", e))),
            }
        } else {
            log.push(LogEntry::warn(format!(
                "auth.json not found at {}",
                auth_json
            )));
        }
    }

    // /etc/hosts
    log.push(LogEntry::cmd(format!("Checking /etc/hosts for {}", domain)));
    let hosts = tokio::fs::read_to_string("/etc/hosts")
        .await
        .unwrap_or_default();
    if hosts.contains(&domain) {
        log.push(LogEntry::info(format!("{} already in /etc/hosts", domain)));
    } else {
        let line = format!("127.0.0.1    {}\n", domain);
        match sudo_tee_append_with_password(&password, "/etc/hosts", &line).await {
            Ok(_) => log.push(LogEntry::ok(format!("Added {} to /etc/hosts", domain))),
            Err(e) => log.push(LogEntry::err(format!("hosts write failed: {}", e))),
        }
    }

    // VirtualHost block
    log.push(LogEntry::cmd(format!(
        "Checking Apache config: {}",
        apache_conf
    )));
    let existing = tokio::fs::read_to_string(&apache_conf)
        .await
        .unwrap_or_default();
    if existing.contains(&format!("ServerName {}", domain)) {
        log.push(LogEntry::info("VirtualHost already present in config"));
    } else {
        let block = format!(
            "\n<VirtualHost *:80>\n\
             \x20   ServerName {domain}\n\
             \x20   ServerAlias www.{domain}\n\
             \x20   DocumentRoot {proj}/public\n\
             \n\
             \x20   <Directory {proj}/public>\n\
             \x20       AllowOverride All\n\
             \x20       Require all granted\n\
             \x20   </Directory>\n\
             \n\
             \x20   ErrorLog ${{APACHE_LOG_DIR}}/{name}_error.log\n\
             \x20   CustomLog ${{APACHE_LOG_DIR}}/{name}_access.log combined\n\
             </VirtualHost>\n\n",
            domain = domain,
            proj = project_path.display(),
            name = project_name,
        );
        match sudo_tee_append_with_password(&password, &apache_conf, &block).await {
            Ok(_) => log.push(LogEntry::ok(format!(
                "VirtualHost block added to {}",
                apache_conf
            ))),
            Err(e) => log.push(LogEntry::err(format!("Apache config write failed: {}", e))),
        }
    }

    // a2ensite
    log.push(LogEntry::cmd("Running a2ensite..."));
    let conf_base = PathBuf::from(&apache_conf)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    match sudo_cmd_with_password(&password, &["a2ensite", &conf_base]).await {
        Ok(_) => log.push(LogEntry::ok(format!("a2ensite {} - OK", conf_base))),
        Err(_) => log.push(LogEntry::warn(
            "a2ensite returned non-zero (may already be enabled)",
        )),
    }

    // Reload Apache
    log.push(LogEntry::cmd("Reloading Apache..."));
    match sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"]).await {
        Ok(_) => log.push(LogEntry::ok("Apache reloaded successfully")),
        Err(e) => log.push(LogEntry::err(format!("Apache reload failed: {}", e))),
    }

    log.push(LogEntry::ok(format!("Done! Visit: http://{}", domain)));
    log
}

// SSH key generation (no sudo needed)
async fn generate_ssh_key(
    email: String,
    name: String,
    ktype: KeyType,
    passphrase: String,
) -> (bool, String) {
    let ssh_dir = get_home().join(".ssh");
    if !ssh_dir.exists() {
        if let Err(e) = tokio::fs::create_dir_all(&ssh_dir).await {
            return (false, format!("Could not create ~/.ssh: {}", e));
        }
        let _ = Command::new("chmod")
            .args(["700", &ssh_dir.to_string_lossy()])
            .status()
            .await;
    }
    let key_path = ssh_dir.join(&name);
    if key_path.exists() {
        return (false, format!("Key already exists: {}", key_path.display()));
    }
    let mut cmd = Command::new("ssh-keygen");
    match ktype {
        KeyType::Ed25519 => {
            cmd.arg("-t").arg("ed25519");
        }
        KeyType::Rsa4096 => {
            cmd.arg("-t").arg("rsa").arg("-b").arg("4096");
        }
        KeyType::Ecdsa => {
            cmd.arg("-t").arg("ecdsa").arg("-b").arg("521");
        }
    }
    cmd.arg("-f")
        .arg(&key_path)
        .arg("-C")
        .arg(&email)
        .arg("-N")
        .arg(&passphrase);
    match cmd.output().await {
        Ok(o) if o.status.success() => {
            let _ = Command::new("chmod")
                .args(["600", &key_path.to_string_lossy()])
                .status()
                .await;
            let _ = Command::new("ssh-add").arg(&key_path).output().await;
            (true, format!("Key generated: {}", key_path.display()))
        }
        Ok(o) => (
            false,
            format!("ssh-keygen failed: {}", String::from_utf8_lossy(&o.stderr)),
        ),
        Err(e) => (false, format!("ssh-keygen not found: {}", e)),
    }
}

async fn ssh_add(path: String) -> (bool, String) {
    match Command::new("ssh-add").arg(&path).output().await {
        Ok(o) if o.status.success() => (true, format!("Key added: {}", path)),
        Ok(o) => (false, String::from_utf8_lossy(&o.stderr).to_string()),
        Err(e) => (false, e.to_string()),
    }
}

async fn list_ssh_keys() -> Vec<KeyEntry> {
    let ssh_dir = get_home().join(".ssh");
    let mut keys = Vec::new();
    let Ok(mut dir) = tokio::fs::read_dir(&ssh_dir).await else {
        return keys;
    };
    let mut files = Vec::new();
    while let Ok(Some(entry)) = dir.next_entry().await {
        files.push(entry.file_name().to_string_lossy().to_string());
    }
    for name in &files {
        if name.ends_with(".pub")
            || matches!(name.as_str(), "config" | "known_hosts" | "authorized_keys")
        {
            continue;
        }
        let path_str = ssh_dir.join(name).to_string_lossy().to_string();
        let has_pub = files.contains(&format!("{}.pub", name));
        keys.push(KeyEntry {
            name: name.clone(),
            path: path_str,
            has_pub,
        });
    }
    keys.sort_by(|a, b| a.name.cmp(&b.name));
    keys
}

fn get_home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
}

fn xdg_open(path: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}

fn open_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

fn open_php_ini(active_php: &Option<String>) -> std::io::Result<()> {
    // Try to find the active php.ini via `php --ini`, fallback to common paths
    if let Some(version) = active_php {
        // Strip full version to major.minor (e.g. "8.2.10" -> "8.2")
        let short = version.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
        let candidate = format!("/etc/php/{}/cli/php.ini", short);
        if std::path::Path::new(&candidate).exists() {
            return xdg_open(&candidate);
        }
        // Try apache2 variant too
        let candidate2 = format!("/etc/php/{}/apache2/php.ini", short);
        if std::path::Path::new(&candidate2).exists() {
            return xdg_open(&candidate2);
        }
    }
    // Fallback: open /etc/php directory
    xdg_open("/etc/php")
}

// ── Tools: PHP scanner ────────────────────────────────────────────────────

async fn scan_php_versions(
    active_php: Option<String>,
) -> Vec<(String, tabs::tools::PhpStatus, bool)> {
    use tabs::tools::PhpStatus;

    // Active version short form e.g. "8.2"
    let active_short = active_php
        .as_deref()
        .map(|v| v.splitn(3, '.').take(2).collect::<Vec<_>>().join("."));

    let candidates = ["7.4", "8.0", "8.1", "8.2", "8.3", "8.4"];
    let mut results = Vec::new();

    for ver in &candidates {
        // Check if /usr/bin/php{ver} exists → installed
        let bin = format!("/usr/bin/php{}", ver);
        let installed = tokio::fs::metadata(&bin).await.is_ok();

        let status = if installed {
            PhpStatus::Installed
        } else {
            // Quick apt-cache check (non-blocking, best-effort)
            let avail = Command::new("apt-cache")
                .args(["show", &format!("php{}", ver)])
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false);
            if avail {
                PhpStatus::Available
            } else {
                PhpStatus::Unknown
            }
        };

        let is_active = active_short.as_deref() == Some(ver);
        results.push((ver.to_string(), status, is_active));
    }

    results
}

// ── Tools: apt install/remove PHP ────────────────────────────────────────

async fn apt_php_op(version: String, install: bool, password: String) -> (bool, String) {
    let op = if install { "install" } else { "remove" };

    // First update package lists to ensure we have the latest info
    match sudo_cmd_with_password(&password, &["apt-get", "update"]).await {
        Err(e) => {
            return (false, format!("Failed to update package lists: {}", e));
        }
        _ => {}
    }

    // Install the meta-package + common extensions
    let pkg = if install {
        format!(
            "php{} php{}-cli php{}-common php{}-mysql php{}-xml php{}-mbstring",
            version, version, version, version, version, version
        )
    } else {
        format!("php{} php{}-*", version, version)
    };

    let full_cmd = format!("DEBIAN_FRONTEND=noninteractive apt-get -y {} {}", op, pkg);

    match sudo_cmd_with_password(&password, &["sh", "-c", &full_cmd]).await {
        Ok(output) => {
            // Check if the operation was actually successful by checking output
            if output.contains("not found") || output.contains("Unable to locate") {
                (
                    false,
                    format!(
                        "PHP {} not found in repositories. Make sure the ondrej/php PPA is added.",
                        version
                    ),
                )
            } else {
                (true, format!("PHP {} {}ed successfully", version, op))
            }
        }
        Err(e) => (false, format!("PHP {} {} failed: {}", version, op, e)),
    }
}

// ── Tools: open DB terminal ───────────────────────────────────────────────

/// Opens a terminal emulator with mysql/mariadb as root.
/// Returns Ok(description) or Err(reason).
fn open_db_terminal(binary: &str, socket_auth: bool) -> Result<String, String> {
    // Build the mysql command
    let mysql_cmd = if socket_auth {
        format!("sudo {} -u root", binary)
    } else {
        format!("sudo {} -u root -p", binary)
    };

    // Wrap it so the terminal stays open on exit
    let shell_cmd = format!(
        "{}; echo '--- session ended, press Enter ---'; read _",
        mysql_cmd
    );

    // Try terminal emulators in order of preference
    let terminals: &[(&str, &[&str])] = &[
        ("x-terminal-emulator", &["-e"]),
        ("gnome-terminal", &["--", "bash", "-c"]),
        ("xfce4-terminal", &["-e"]),
        ("konsole", &["-e"]),
        ("xterm", &["-e"]),
        ("lxterminal", &["-e"]),
        ("mate-terminal", &["-e"]),
        ("tilix", &["-e"]),
    ];

    for (term, args) in terminals {
        // Check if the terminal exists
        if std::process::Command::new("which")
            .arg(term)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let mut cmd = std::process::Command::new(term);
            cmd.args(*args);
            if *term == "gnome-terminal" {
                // gnome-terminal passes args differently
                cmd.arg(&shell_cmd);
            } else {
                cmd.arg(format!("bash -c '{}'", shell_cmd.replace('\'', "'\\''")));
            }
            return match cmd.spawn() {
                Ok(_) => Ok(format!("{} ({})", term, mysql_cmd)),
                Err(e) => Err(format!("Failed to spawn {}: {}", term, e)),
            };
        }
    }

    Err("No supported terminal emulator found. Install xterm or gnome-terminal.".into())
}

// ── Tools: copy to clipboard (async) ───────────────────────────────────────

async fn copy_to_clipboard_async(text: String) {
    // Method 1: Try xclip (X11) - most reliable
    if try_copy_with_xclip(&text).await {
        return;
    }

    // Method 2: Try wl-copy (Wayland)
    if try_copy_with_wl_copy(&text).await {
        return;
    }

    // Method 3: Try xsel (alternative X11)
    if try_copy_with_xsel(&text).await {
        return;
    }

    // Fallback: Create a temporary shell script file
    create_temp_script_file(&text).await;
}

async fn create_temp_script_file(commands: &str) {
    let home = get_home();
    let script_path = home.join(".devpanel_php_install.sh");

    // Create script with shebang and the commands
    let script_content = format!("#!/bin/bash\n{}\n", commands);

    if let Ok(_) = tokio::fs::write(&script_path, &script_content).await {
        // Make it executable
        let _ = std::process::Command::new("chmod")
            .args(["+x", &script_path.to_string_lossy().to_string()])
            .output();

        // Open it with default editor/text viewer so user can see and copy
        let _ = std::process::Command::new("xdg-open")
            .arg(&script_path)
            .spawn();
    }
}

async fn try_copy_with_xclip(text: &str) -> bool {
    use tokio::io::AsyncWriteExt;

    match tokio::process::Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let write_result = stdin.write_all(text.as_bytes()).await;
                let flush_result = stdin.flush().await;
                drop(stdin);
                let _ = child.wait().await;
                write_result.is_ok() && flush_result.is_ok()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

async fn try_copy_with_wl_copy(text: &str) -> bool {
    use tokio::io::AsyncWriteExt;

    match tokio::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let write_result = stdin.write_all(text.as_bytes()).await;
                let flush_result = stdin.flush().await;
                drop(stdin);
                let _ = child.wait().await;
                write_result.is_ok() && flush_result.is_ok()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

async fn try_copy_with_xsel(text: &str) -> bool {
    use tokio::io::AsyncWriteExt;

    match tokio::process::Command::new("xsel")
        .arg("-b")
        .arg("-i")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let write_result = stdin.write_all(text.as_bytes()).await;
                let flush_result = stdin.flush().await;
                drop(stdin);
                let _ = child.wait().await;
                write_result.is_ok() && flush_result.is_ok()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
