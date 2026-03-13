// src/main.rs -- DevPanel: Apache + SSH + VirtualHost + Repos manager

mod sudo_prompt;
mod tabs;
mod theme;

use sudo_prompt::{
    clear_saved_password, save_password, sudo_cmd_with_password, validate_sudo_password,
    ModalState, PendingAction, SudoModal,
};
use tabs::dashboard::DashboardTab;
use tabs::repos::{ReposTab, SshStatus};
use tabs::ssh_keys::{KeyEntry, KeyType, SshKeysTab};
use tabs::tools::ToolSection;
use tabs::tools::ToolsTab;
use tabs::vhosts::{VHostEntry, VHostsTab};
use theme::*;

use iced::widget::{button, column, container, row, stack, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding, Task, Theme};
use std::path::PathBuf;
use tokio::process::Command;

// ── Config ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DevPanelConfig {
    pub repos_root: String,
    pub devpanel_conf: String,
    pub hosts_file: String,
}

impl DevPanelConfig {
    pub fn load() -> Self {
        let config_path = get_home().join(".config/devpanel/config.toml");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            DevPanelConfig {
                repos_root: parse_toml_str(&content, "repos_root")
                    .unwrap_or_else(default_repos_root),
                devpanel_conf: parse_toml_str(&content, "devpanel_conf")
                    .unwrap_or_else(default_devpanel_conf),
                hosts_file: parse_toml_str(&content, "hosts_file")
                    .unwrap_or_else(|| "/etc/hosts".to_string()),
            }
        } else {
            DevPanelConfig {
                repos_root: default_repos_root(),
                devpanel_conf: default_devpanel_conf(),
                hosts_file: "/etc/hosts".to_string(),
            }
        }
    }

    pub fn save(&self) {
        let dir = get_home().join(".config/devpanel");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(
            dir.join("config.toml"),
            format!(
                "repos_root    = \"{}\"
devpanel_conf = \"{}\"
hosts_file    = \"{}\"
",
                self.repos_root, self.devpanel_conf, self.hosts_file
            ),
        );
    }
}

fn parse_toml_str(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with(&format!("{} =", key)) || t.starts_with(&format!("{}=", key)) {
            if let Some(eq) = t.find('=') {
                let val = t[eq + 1..].trim();
                if val.starts_with('"') && val.ends_with('"') {
                    return Some(val[1..val.len() - 1].to_string());
                }
            }
        }
    }
    None
}

fn default_repos_root() -> String {
    let c = get_home().join("projects");
    if c.exists() {
        c.to_string_lossy().to_string()
    } else {
        "/var/www/html".to_string()
    }
}
fn default_devpanel_conf() -> String {
    "/etc/apache2/sites-available/devpanel.conf".to_string()
}

// ── Active tab ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Repos,
    VHosts,
    SshKeys,
    Tools,
}

// ── Messages ──────────────────────────────────────────────────────────────

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(Tab),
    // Dashboard
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
    SwitchPHPVersion(String),
    PhpSwitchResult(bool, String),
    ShowPHPInfo,
    OpenLocalhost,
    OpenPhpMyAdmin,
    OpenWebRoot,
    OpenPhpIni,
    RestartAll,
    OpenProjectsFolder,
    NavigateApache2Conf,
    NavigateApache2Sites,
    NavigatePhpDir,
    NavigateMysqlDir,
    NavigateHostsFile,
    RefreshStatus,
    StatusRefreshed {
        apache: bool,
        mysql: bool,
        php: Option<String>,
        php_versions: Vec<String>,
    },
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
    TOOLS_ScanDone(Vec<(String, tabs::tools::PhpStatus, bool, bool, bool)>),
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
    TOOLS_SetSection(ToolSection),
    TOOLS_ScanApacheMods,
    TOOLS_ScanApacheModsDone(Vec<tabs::tools::ApacheModule>),
    TOOLS_EnableApacheMod(String),
    TOOLS_DisableApacheMod(String),
    TOOLS_ApacheModDone(bool, String, String, bool),
    TOOLS_ModFilterChanged(String),
    TOOLS_ScanPhpExts,
    TOOLS_ScanPhpExtsDone(Vec<(String, bool)>),
    TOOLS_InstallPhpExt(String),
    TOOLS_RemovePhpExt(String),
    TOOLS_PhpExtDone(bool, String),
    // Repos
    REPOS_CheckSsh,
    REPOS_SshChecked(bool, String, bool, String),
    REPOS_Fetch,
    REPOS_FetchDone(Vec<tabs::repos::RemoteRepo>),
    REPOS_Clone {
        ssh_url: String,
        name: String,
    },
    REPOS_CloneDone(bool, String, String),
    REPOS_OpenCloned(String),
    REPOS_SearchChanged(String),
    REPOS_SetFilter(tabs::repos::ProviderFilter),
    REPOS_OpenRoot,
    // VHosts
    VH_Scan,
    VH_ScanDone(Vec<VHostEntry>),
    VH_ShowAddForm,
    VH_HideForm,
    VH_FormServerNameChanged(String),
    VH_FormDocRootChanged(String),
    VH_Create,
    VH_CreateDone(bool, String),
    VH_EditRequest(usize),
    VH_SaveEdit,
    VH_SaveEditDone(bool, String),
    VH_OpenBrowser(String),
    VH_OpenDevpanelConf,
    VH_DeleteRequest(usize),
    VH_DeleteConfirm(usize),
    VH_DeleteCancel,
    VH_DeleteDone(bool, String),
    // Config Editor
    VH_OpenConfigEditor,
    VH_CloseConfigEditor,
    VH_ConfigLoaded(String),
    VH_ConfigEditorAction(iced::widget::text_editor::Action),
    VH_SaveConfigFile,
    VH_SaveConfigDone(bool, String),
    // Auto-refresh
    AutoRefreshTick,
    // Sudo
    Sudo_PasswordChanged(String),
    Sudo_ToggleShow(bool),
    Sudo_ToggleSave(bool),
    Sudo_Submit,
    Sudo_ValidationResult(bool),
    Sudo_Cancel,
    Sudo_ClearSaved,
    // ApacheTouch
    AT_ProjectNameChanged(String),
    AT_BaseDirChanged(String),
    AT_ApacheConfChanged(String),
    AT_AuthJsonChanged(String),
    AT_BrowseAuthJson,
    AT_RunSetup,
    AT_ClearLog,
    AT_SetupDone(Vec<tabs::apache_touch::LogEntry>, bool),
}

// ── App state ─────────────────────────────────────────────────────────────

struct App {
    active_tab: Tab,
    config: DevPanelConfig,
    dashboard: DashboardTab,
    ssh_keys: SshKeysTab,
    tools: ToolsTab,
    repos: ReposTab,
    vhosts: VHostsTab,
    toast: Option<Toast>,
    sudo: SudoModal,
    sudo_pending_action: Option<PendingAction>,
}

#[derive(Clone)]
struct Toast {
    message: String,
    ok: bool,
}

impl App {
    fn new() -> (Self, Task<Message>) {
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
        };
        (app, Task::perform(tabs::dashboard::probe_services(), |r| r))
    }

    fn trigger_sudo(&mut self, action: PendingAction) -> Task<Message> {
        if let Some(password) = self.sudo.get_password() {
            self.dispatch_sudo_action(action, password)
        } else {
            self.sudo_pending_action = Some(action.clone());
            self.sudo.state = ModalState::Asking {
                pending_action: action,
            };
            self.sudo.password_input.clear();
            Task::none()
        }
    }

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
                Task::perform(tabs::tools::switch_php(version, password), |(ok, msg)| {
                    Message::PhpSwitchResult(ok, msg)
                })
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
                Task::perform(tabs::tools::apt_php_op(version, true, password), |(ok, msg)| {
                    Message::TOOLS_PhpOpDone(ok, msg)
                })
            }
            PendingAction::PhpRemove(version) => {
                Task::perform(tabs::tools::apt_php_op(version, false, password), |(ok, msg)| {
                    Message::TOOLS_PhpOpDone(ok, msg)
                })
            }
            PendingAction::VHostAdd {
                server_name,
                document_root,
            } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    tabs::vhosts::add_vhost(conf, server_name, document_root, password),
                    |(ok, msg)| Message::VH_CreateDone(ok, msg),
                )
            }
            PendingAction::VHostEdit {
                index,
                server_name,
                document_root,
            } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    tabs::vhosts::edit_vhost(conf, index, server_name, document_root, password),
                    |(ok, msg)| Message::VH_SaveEditDone(ok, msg),
                )
            }
            PendingAction::VHostDelete { index } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    tabs::vhosts::delete_vhost(conf, index, password),
                    |(ok, msg)| Message::VH_DeleteDone(ok, msg),
                )
            }
            PendingAction::ApacheModToggle { name, enable } => Task::perform(
                tabs::tools::toggle_apache_module(name, enable, password),
                |(ok, msg, n, en)| Message::TOOLS_ApacheModDone(ok, msg, n, en),
            ),
            PendingAction::AptInstall { package } => {
                Task::perform(tabs::tools::apt_package_op(package, true, password), |(ok, msg)| {
                    Message::TOOLS_PhpExtDone(ok, msg)
                })
            }
            PendingAction::AptRemove { package } => {
                Task::perform(tabs::tools::apt_package_op(package, false, password), |(ok, msg)| {
                    Message::TOOLS_PhpExtDone(ok, msg)
                })
            }
            PendingAction::SaveConfig { path, content } => {
                Task::perform(tabs::vhosts::save_config_file(path, content, password), |(ok, msg)| {
                    Message::VH_SaveConfigDone(ok, msg)
                })
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(5))
            .map(|_| Message::AutoRefreshTick)
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectTab(tab) => {
                self.active_tab = tab.clone();
                match tab {
                    Tab::Dashboard => Task::perform(tabs::dashboard::probe_services(), |r| r),
                    Tab::SshKeys => Task::perform(tabs::ssh_keys::list_keys(), Message::SSH_KeysListed),
                    Tab::Tools => {
                        self.tools.scanning = true;
                        Task::perform(
                            tabs::tools::scan_php_versions(self.dashboard.active_php_version.clone()),
                            Message::TOOLS_ScanDone,
                        )
                    }
                    Tab::Repos => Task::none(),
                    Tab::VHosts => {
                        self.vhosts.scanning = true;
                        let conf = self.vhosts.devpanel_conf.clone();
                        Task::perform(tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone)
                    }
                }
            }
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
                Task::none()
            }
            Message::Sudo_Submit => {
                let pass = self.sudo.password_input.clone();
                if pass.is_empty() {
                    return Task::none();
                }
                self.sudo.state = ModalState::Validating;
                Task::perform(validate_sudo_password(pass), Message::Sudo_ValidationResult)
            }
            Message::Sudo_ValidationResult(valid) => {
                if !valid {
                    self.sudo.state = ModalState::Failed;
                    self.sudo.password_input.clear();
                    return Task::none();
                }
                let password = self.sudo.password_input.clone();
                self.sudo.cached_password = Some(password.clone());
                self.sudo.password_input.clear();
                self.sudo.state = ModalState::Hidden;
                if self.sudo.save_password {
                    save_password(&password);
                }
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
            Message::RefreshStatus => Task::perform(tabs::dashboard::probe_services(), |r| r),
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
                Task::perform(tabs::dashboard::probe_services(), |r| r)
            }
            Message::SwitchPHPVersion(v) => self.trigger_sudo(PendingAction::PhpSwitch(v)),
            Message::PhpSwitchResult(ok, msg) => {
                self.toast = Some(Toast { message: msg, ok });
                Task::perform(tabs::dashboard::probe_services(), |r| r)
            }
            Message::ShowPHPInfo => {
                let _ = open_url("http://localhost/phpinfo.php");
                Task::none()
            }
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
            Message::OpenProjectsFolder => {
                let _ = xdg_open(&self.config.repos_root);
                Task::none()
            }
            Message::NavigateApache2Conf => {
                let _ = xdg_open("/etc/apache2/apache2.conf");
                Task::none()
            }
            Message::NavigateApache2Sites => {
                let _ = xdg_open("/etc/apache2/sites-available");
                Task::none()
            }
            Message::NavigatePhpDir => {
                let _ = xdg_open("/etc/php");
                Task::none()
            }
            Message::NavigateMysqlDir => {
                let _ = xdg_open("/etc/mysql");
                Task::none()
            }
            Message::NavigateHostsFile => {
                let _ = xdg_open("/etc/hosts");
                Task::none()
            }
            Message::OpenPhpIni => {
                let _ = open_php_ini(&self.dashboard.active_php_version);
                Task::none()
            }
            Message::RestartAll => self.trigger_sudo(PendingAction::RestartAll),
            Message::TOOLS_ScanPhp => {
                self.tools.scanning = true;
                Task::perform(
                    tabs::tools::scan_php_versions(self.dashboard.active_php_version.clone()),
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
                self.tools.scanning = true;
                Task::batch([
                    Task::perform(tabs::dashboard::probe_services(), |r| r),
                    Task::perform(
                        tabs::tools::scan_php_versions(self.dashboard.active_php_version.clone()),
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
                self.tools.db_status = match open_db_terminal("mysql", false) {
                    Ok(cmd) => format!("Launched: {}", cmd),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            Message::TOOLS_OpenMariadbCli => {
                self.tools.db_status = match open_db_terminal("mariadb", false) {
                    Ok(cmd) => format!("Launched: {}", cmd),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            Message::TOOLS_OpenMysqlSocket => {
                self.tools.db_status = match open_db_terminal("mysql", true) {
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
                    message: "Commands copied to clipboard!".into(),
                    ok: true,
                });
                Task::perform(
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    },
                    |_| Message::TOOLS_ClearToast,
                )
            }
            Message::TOOLS_SetSection(section) => {
                self.tools.active_section = section;
                Task::none()
            }
            Message::TOOLS_ScanApacheMods => {
                Task::perform(tabs::tools::scan_apache_modules(), Message::TOOLS_ScanApacheModsDone)
            }
            Message::TOOLS_ScanApacheModsDone(results) => {
                self.tools.apply_mod_scan(results);
                Task::none()
            }
            Message::TOOLS_ModFilterChanged(v) => {
                self.tools.mod_filter = v;
                Task::none()
            }
            Message::TOOLS_EnableApacheMod(name) => {
                self.tools
                    .install_log
                    .push((true, format!("Enabling mod_{}...", name)));
                self.trigger_sudo(PendingAction::ApacheModToggle { name, enable: true })
            }
            Message::TOOLS_DisableApacheMod(name) => {
                self.tools
                    .install_log
                    .push((true, format!("Disabling mod_{}...", name)));
                self.trigger_sudo(PendingAction::ApacheModToggle {
                    name,
                    enable: false,
                })
            }
            Message::TOOLS_ApacheModDone(ok, msg, name, enabled) => {
                self.tools.push_log(ok, msg.clone());
                self.toast = Some(Toast { message: msg, ok });
                if ok {
                    self.tools.set_mod_enabled(&name, enabled);
                }
                Task::perform(tabs::tools::scan_apache_modules(), Message::TOOLS_ScanApacheModsDone)
            }
            Message::TOOLS_ScanPhpExts => {
                let active = self
                    .tools
                    .php_releases
                    .iter()
                    .find(|r| r.is_active)
                    .map(|r| r.version.clone());
                Task::perform(tabs::tools::scan_php_extensions(active), Message::TOOLS_ScanPhpExtsDone)
            }
            Message::TOOLS_ScanPhpExtsDone(results) => {
                self.tools.apply_ext_scan(results);
                Task::none()
            }
            Message::TOOLS_InstallPhpExt(pkg) => {
                self.tools
                    .install_log
                    .push((true, format!("Installing {}...", pkg)));
                self.trigger_sudo(PendingAction::AptInstall { package: pkg })
            }
            Message::TOOLS_RemovePhpExt(pkg) => {
                self.tools
                    .install_log
                    .push((true, format!("Removing {}...", pkg)));
                self.trigger_sudo(PendingAction::AptRemove { package: pkg })
            }
            Message::TOOLS_PhpExtDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.toast = Some(Toast { message: msg, ok });
                let active = self
                    .tools
                    .php_releases
                    .iter()
                    .find(|r| r.is_active)
                    .map(|r| r.version.clone());
                Task::perform(tabs::tools::scan_php_extensions(active), Message::TOOLS_ScanPhpExtsDone)
            }
            // ── Repos ──────────────────────────────────────────────────────
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
            Message::SSH_TogglePassphrase(v) => {
                self.ssh_keys.show_passphrase = v;
                Task::none()
            }
            Message::SSH_GenerateKey => {
                let email = self.ssh_keys.email.clone();
                let name = self.ssh_keys.key_name.clone();
                let ktype = self.ssh_keys.key_type.clone();
                let passphrase = self.ssh_keys.passphrase.clone();
                Task::perform(
                    tabs::ssh_keys::generate_key(email, name, ktype, passphrase),
                    |(ok, msg)| Message::SSH_GenerateDone(ok, msg),
                )
            }
            Message::SSH_GenerateDone(ok, msg) => {
                use tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                self.ssh_keys.status_message = msg.clone();
                self.toast = Some(Toast { message: msg, ok });
                if ok {
                    Task::perform(tabs::ssh_keys::list_keys(), Message::SSH_KeysListed)
                } else {
                    Task::none()
                }
            }
            Message::SSH_AddExisting => {
                let path = format!("{}/.ssh/{}", get_home().display(), self.ssh_keys.key_name);
                Task::perform(ssh_add(path), |(ok, msg)| {
                    Message::SSH_AddExistingDone(ok, msg)
                })
            }
            Message::SSH_AddExistingDone(ok, msg) => {
                use tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                self.ssh_keys.status_message = msg.clone();
                self.toast = Some(Toast { message: msg, ok });
                Task::none()
            }
            Message::SSH_OpenDir => {
                let _ = xdg_open(&format!("{}/.ssh", get_home().display()));
                Task::none()
            }
            Message::SSH_ListKeys => Task::perform(tabs::ssh_keys::list_keys(), Message::SSH_KeysListed),
            Message::SSH_KeysListed(keys) => {
                self.ssh_keys.keys_list = keys;
                Task::none()
            }
            // ── Repos (remote browser) ────────────────────────────────────
            Message::REPOS_CheckSsh => Task::perform(tabs::repos::check_ssh(), |r| {
                Message::REPOS_SshChecked(r.github_ok, r.github_msg, r.bb_ok, r.bb_msg)
            }),
            Message::REPOS_SshChecked(gok, gmsg, bok, bmsg) => {
                self.repos.github_status = if gok {
                    SshStatus::Connected
                } else {
                    SshStatus::Failed(gmsg)
                };
                self.repos.bitbucket_status = if bok {
                    SshStatus::Connected
                } else {
                    SshStatus::Failed(bmsg)
                };
                Task::none()
            }
            Message::REPOS_Fetch => {
                self.repos.fetching = true;
                let root = self.repos.repos_root.clone();
                Task::perform(
                    tabs::repos::fetch_remote_repos(root),
                    Message::REPOS_FetchDone,
                )
            }
            Message::REPOS_FetchDone(repos) => {
                self.repos.set_repos(repos);
                Task::none()
            }
            Message::REPOS_Clone { ssh_url, name } => {
                self.repos.mark_cloning(&ssh_url, true);
                let root = self.repos.repos_root.clone();
                Task::perform(
                    tabs::repos::clone_repo(ssh_url, name, root),
                    |(ok, msg, url)| Message::REPOS_CloneDone(ok, msg, url),
                )
            }
            Message::REPOS_CloneDone(ok, msg, ssh_url) => {
                if ok {
                    self.repos.mark_cloned(&ssh_url);
                } else {
                    self.repos.mark_cloning(&ssh_url, false);
                }
                self.repos.status_msg = Some((ok, msg.clone()));
                self.toast = Some(Toast { message: msg, ok });
                Task::none()
            }
            Message::REPOS_OpenCloned(name) => {
                let path = format!("{}/{}", self.repos.repos_root, name);
                open_terminal_at(&path);
                Task::none()
            }
            Message::REPOS_SearchChanged(v) => {
                self.repos.search_query = v;
                Task::none()
            }
            Message::REPOS_SetFilter(f) => {
                self.repos.active_filter = f;
                Task::none()
            }
            Message::REPOS_OpenRoot => {
                let _ = xdg_open(&self.repos.repos_root);
                Task::none()
            }
            // ── VHosts ────────────────────────────────────────────────────
            Message::VH_Scan => {
                self.vhosts.scanning = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone)
            }
            Message::VH_ScanDone(vhosts) => {
                self.vhosts.set_vhosts(vhosts);
                Task::none()
            }
            Message::VH_ShowAddForm => {
                self.vhosts.form.open_add();
                Task::none()
            }
            Message::VH_HideForm => {
                self.vhosts.form.hide();
                Task::none()
            }
            Message::VH_FormServerNameChanged(v) => {
                self.vhosts.form.server_name = v;
                Task::none()
            }
            Message::VH_FormDocRootChanged(v) => {
                self.vhosts.form.document_root = v;
                Task::none()
            }
            Message::VH_Create => {
                let sn = self.vhosts.form.server_name.trim().to_string();
                let dr = self.vhosts.form.document_root.trim().to_string();
                self.trigger_sudo(PendingAction::VHostAdd {
                    server_name: sn,
                    document_root: dr,
                })
            }
            Message::VH_CreateDone(ok, msg) => {
                self.vhosts.form.hide();
                self.vhosts.status_msg = Some((ok, msg.clone()));
                self.toast = Some(Toast { message: msg, ok });
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone)
            }
            Message::VH_EditRequest(idx) => {
                if let Some(entry) = self.vhosts.vhosts.iter().find(|e| e.index == idx) {
                    let entry = entry.clone();
                    self.vhosts.form.open_edit(&entry);
                }
                Task::none()
            }
            Message::VH_SaveEdit => {
                let sn = self.vhosts.form.server_name.trim().to_string();
                let dr = self.vhosts.form.document_root.trim().to_string();
                let idx = if let tabs::vhosts::FormMode::Edit(i) = self.vhosts.form.mode {
                    i
                } else {
                    return Task::none();
                };
                self.trigger_sudo(PendingAction::VHostEdit {
                    index: idx,
                    server_name: sn,
                    document_root: dr,
                })
            }
            Message::VH_SaveEditDone(ok, msg) => {
                self.vhosts.form.hide();
                self.vhosts.status_msg = Some((ok, msg.clone()));
                self.toast = Some(Toast { message: msg, ok });
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone)
            }
            Message::VH_OpenBrowser(sn) => {
                let _ = open_url(&format!("http://{}", sn));
                Task::none()
            }
            Message::VH_OpenDevpanelConf => {
                let _ = xdg_open(&self.vhosts.devpanel_conf);
                Task::none()
            }
            Message::VH_DeleteRequest(idx) => {
                self.vhosts.confirm_delete = Some(idx);
                Task::none()
            }
            Message::VH_DeleteConfirm(idx) => {
                self.vhosts.confirm_delete = None;
                self.trigger_sudo(PendingAction::VHostDelete { index: idx })
            }
            Message::VH_DeleteCancel => {
                self.vhosts.confirm_delete = None;
                Task::none()
            }
            Message::VH_DeleteDone(ok, msg) => {
                self.vhosts.status_msg = Some((ok, msg.clone()));
                self.toast = Some(Toast { message: msg, ok });
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone)
            }
            // Config Editor
            Message::VH_OpenConfigEditor => {
                self.vhosts.view_mode = tabs::vhosts::VHostView::ConfigEditor;
                self.vhosts.config_loading = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(tabs::vhosts::load_config_file(conf), Message::VH_ConfigLoaded)
            }
            Message::VH_CloseConfigEditor => {
                self.vhosts.view_mode = tabs::vhosts::VHostView::List;
                Task::none()
            }
            Message::VH_ConfigLoaded(text) => {
                self.vhosts.load_config_text(text);
                Task::none()
            }
            Message::VH_ConfigEditorAction(action) => {
                let is_edit = action.is_edit();
                self.vhosts.config_content.perform(action);
                if is_edit { self.vhosts.config_dirty = true; }
                Task::none()
            }
            Message::VH_SaveConfigFile => {
                self.vhosts.config_loading = true;
                let content = self.vhosts.config_content.text();
                let conf = self.vhosts.devpanel_conf.clone();
                self.trigger_sudo(PendingAction::SaveConfig { content, path: conf })
            }
            Message::VH_SaveConfigDone(ok, msg) => {
                self.vhosts.config_loading = false;
                if ok { self.vhosts.config_dirty = false; }
                self.vhosts.status_msg = Some((ok, msg.clone()));
                self.toast = Some(Toast { message: msg, ok });
                if ok {
                    let conf = self.vhosts.devpanel_conf.clone();
                    Task::perform(tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone)
                } else {
                    Task::none()
                }
            }
            // Auto-refresh
            Message::AutoRefreshTick => {
                if self.active_tab == Tab::Dashboard {
                    Task::perform(tabs::dashboard::probe_services(), |r| r)
                } else {
                    Task::none()
                }
            }
            // ApacheTouch (tab not yet surfaced in nav)
            Message::AT_ProjectNameChanged(_)
            | Message::AT_BaseDirChanged(_)
            | Message::AT_ApacheConfChanged(_)
            | Message::AT_AuthJsonChanged(_)
            | Message::AT_BrowseAuthJson
            | Message::AT_RunSetup
            | Message::AT_ClearLog
            | Message::AT_SetupDone(_, _) => Task::none(),
        } // end match msg
    } // end fn update

    fn view(&self) -> Element<'_, Message> {
        let tab_content: Element<Message> = match &self.active_tab {
            Tab::Dashboard => self.dashboard.view(),
            Tab::SshKeys => self.ssh_keys.view(),
            Tab::Tools => self.tools.view(),
            Tab::Repos => self.repos.view(),
            Tab::VHosts => self.vhosts.view(),
        };

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
            let banner = container(
                row![
                    container(
                        text(if toast.ok { "+" } else { "x" })
                            .size(11)
                            .color(Color::WHITE)
                    )
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
                self.sudo.view()
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
                    text("panel").size(19).color(TEXT_PRIMARY)
                ]
                .spacing(0),
            ]
            .align_y(Alignment::Center)]
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

        let bottom = container(
            column![
                sudo_indicator,
                Space::with_height(10),
                button(
                    row![
                        text("R").size(11).color(TEXT_MUTED),
                        Space::with_width(6),
                        text("Refresh").size(12).color(TEXT_MUTED)
                    ]
                    .align_y(Alignment::Center)
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
                bottom
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
        let (icon, icon_color): (&str, Color) = match &tab {
            Tab::Dashboard => ("", if active { TEAL } else { TEXT_MUTED }),
            Tab::Repos => ("", if active { TEAL } else { TEXT_MUTED }),
            Tab::VHosts => ("", if active { TEAL } else { TEXT_MUTED }),
            Tab::SshKeys => ("", if active { TEAL } else { TEXT_MUTED }),
            Tab::Tools => ("", if active { TEAL } else { TEXT_MUTED }),
        };
        button(
            row![
                text(icon).size(12).color(icon_color),
                Space::with_width(10),
                text(label).size(13).color(text_color)
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
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            rgba[idx + 3] = 0xFF;
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            if dx * dx + dy * dy <= 13.0 * 13.0 {
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
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .window(iced::window::Settings {
            size: iced::Size::new(1040.0, 660.0),
            min_size: Some(iced::Size::new(860.0, 560.0)),
            icon,
            ..Default::default()
        })
        .run_with(App::new)
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



async fn ssh_add(path: String) -> (bool, String) {
    match tokio::process::Command::new("ssh-add").arg(&path).output().await {
        Ok(o) if o.status.success() => (true, format!("Key added: {}", path)),
        Ok(o) => (false, String::from_utf8_lossy(&o.stderr).to_string()),
        Err(e) => (false, e.to_string()),
    }
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
    if let Some(version) = active_php {
        let short = version.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
        let c1 = format!("/etc/php/{}/cli/php.ini", short);
        if std::path::Path::new(&c1).exists() {
            return xdg_open(&c1);
        }
        let c2 = format!("/etc/php/{}/apache2/php.ini", short);
        if std::path::Path::new(&c2).exists() {
            return xdg_open(&c2);
        }
    }
    xdg_open("/etc/php")
}

fn open_terminal_at(path: &str) {
    let Some(term) = find_terminal() else {
        return;
    };
    let cd_cmd = format!("cd {} && exec bash", shell_quote(path));
    let result = match term.as_str() {
        "gnome-terminal" => std::process::Command::new("gnome-terminal")
            .arg("--working-directory")
            .arg(path)
            .spawn(),
        "xfce4-terminal" => std::process::Command::new("xfce4-terminal")
            .arg("--working-directory")
            .arg(path)
            .spawn(),
        "konsole" => std::process::Command::new("konsole")
            .arg("--workdir")
            .arg(path)
            .spawn(),
        "mate-terminal" => std::process::Command::new("mate-terminal")
            .arg("--working-directory")
            .arg(path)
            .spawn(),
        // xterm and friends: use -e bash -c "cd X && exec bash" as split args
        "xterm" => std::process::Command::new("xterm")
            .args(["-e", "bash", "-c", &cd_cmd])
            .spawn(),
        // lxterminal / tilix: -e takes a single shell string
        "lxterminal" | "tilix" => std::process::Command::new(&term)
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&cd_cmd)))
            .spawn(),
        _ => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &cd_cmd])
            .spawn(),
    };
    let _ = result;
}

fn open_db_terminal(binary: &str, socket_auth: bool) -> Result<String, String> {
    // Build mysql command (runs as root via sudo)
    let mysql_cmd = if socket_auth {
        format!("sudo {} -u root", binary)
    } else {
        format!("sudo {} -u root -p", binary)
    };

    // Shell script that runs mysql then waits before closing
    // Use only POSIX-safe escapes — no \033 (not POSIX sh), use printf octal instead
    let inner = format!(
        "{cmd}; printf '\\n\\033[0;33m--- session ended, press Enter to close ---\\033[0m\\n'; read _",
        cmd = mysql_cmd
    );

    let term = match find_terminal() {
        Some(t) => t,
        None => return Err(
            "No terminal emulator found. Install gnome-terminal, xterm, konsole, or xfce4-terminal.".into()
        ),
    };

    // Every terminal gets bash -c "<inner>" with args passed individually
    // (never concatenated into a single shell-escaped string) so the shell
    // does not double-interpret the command.
    let result = match term.as_str() {
        // These support `-- bash -c "cmd"` (recommended, cleanest)
        "gnome-terminal" | "mate-terminal" => std::process::Command::new(&term)
            .args(["--", "bash", "-c", &inner])
            .spawn(),
        // These support `-e bash -c "cmd"` with args split
        "xterm" | "konsole" => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &inner])
            .spawn(),
        // xfce4-terminal: --command takes a single string, but it runs it
        // through the shell, so pass `bash -c 'cmd'` properly quoted
        "xfce4-terminal" => std::process::Command::new("xfce4-terminal")
            .arg("--command")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        // tilix: same as xfce4 — -e takes a shell string
        "tilix" => std::process::Command::new("tilix")
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        // lxterminal: -e takes a single string run via sh -c
        "lxterminal" => std::process::Command::new("lxterminal")
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        // x-terminal-emulator: Debian alternative, usually points to xterm/gnome-terminal
        // Try -- first (works for gnome), fall back to -e
        "x-terminal-emulator" => std::process::Command::new("x-terminal-emulator")
            .args(["-e", "bash", "-c", &inner])
            .spawn(),
        // Unknown terminal: best-effort -e
        _ => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &inner])
            .spawn(),
    };

    match result {
        Ok(_) => Ok(format!("Launched '{}' in {}", mysql_cmd, term)),
        Err(e) => Err(format!("Failed to open {}: {}", term, e)),
    }
}

fn find_terminal() -> Option<String> {
    let candidates = [
        "gnome-terminal",
        "xfce4-terminal",
        "konsole",
        "tilix",
        "mate-terminal",
        "lxterminal",
        "xterm",
        "x-terminal-emulator",
    ];
    for t in &candidates {
        let ok = std::process::Command::new("which")
            .arg(t)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return Some(t.to_string());
        }
    }
    // Last resort: check if /usr/bin/xterm exists without `which`
    if std::path::Path::new("/usr/bin/xterm").exists() {
        return Some("xterm".to_string());
    }
    None
}

/// Wrap a string in single quotes for shell, escaping any embedded single quotes.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

async fn copy_to_clipboard_async(text: String) {
    if try_copy_with_xclip(&text).await {
        return;
    }
    if try_copy_with_wl_copy(&text).await {
        return;
    }
    if try_copy_with_xsel(&text).await {
        return;
    }
    create_temp_script_file(&text).await;
}

async fn create_temp_script_file(commands: &str) {
    let path = get_home().join(".devpanel_php_install.sh");
    if tokio::fs::write(&path, format!("#!/bin/bash\n{}\n", commands))
        .await
        .is_ok()
    {
        let _ = std::process::Command::new("chmod")
            .args(["+x", &path.to_string_lossy().to_string()])
            .output();
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
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
        Ok(mut c) => {
            if let Some(mut s) = c.stdin.take() {
                let ok = s.write_all(text.as_bytes()).await.is_ok() && s.flush().await.is_ok();
                drop(s);
                let _ = c.wait().await;
                ok
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
        Ok(mut c) => {
            if let Some(mut s) = c.stdin.take() {
                let ok = s.write_all(text.as_bytes()).await.is_ok() && s.flush().await.is_ok();
                drop(s);
                let _ = c.wait().await;
                ok
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
        Ok(mut c) => {
            if let Some(mut s) = c.stdin.take() {
                let ok = s.write_all(text.as_bytes()).await.is_ok() && s.flush().await.is_ok();
                drop(s);
                let _ = c.wait().await;
                ok
            } else {
                false
            }
        }
        Err(_) => false,
    }
}


// ── Scan PHP extensions ────────────────────────────────────────────────────

