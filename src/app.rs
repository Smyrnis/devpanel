// src/app.rs — App state, Message enum, update(), view(), sidebar()

use crate::config::DevPanelConfig;
use crate::sudo_prompt::{
    clear_saved_password, save_password, validate_sudo_password, ModalState, PendingAction,
    SudoModal,
};
use crate::system::{
    copy_to_clipboard, get_home, open_db_terminal, open_php_ini, open_terminal_at, open_url,
    run_service_cmd, ssh_add, xdg_open,
};
use crate::tabs::dashboard::DashboardTab;
use crate::tabs::repos::{ReposTab, SshStatus};
use crate::tabs::ssh_keys::{KeyEntry, KeyType, SshKeysTab};
use crate::tabs::tools::{ToolSection, ToolsTab};
use crate::tabs::vhosts::{VHostEntry, VHostsTab};
use crate::theme::*;

use iced::widget::{button, column, container, row, stack, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Padding, Task};

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
    ServiceResult { service: String, action: String, success: bool, output: String },
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
    StatusRefreshed { apache: bool, mysql: bool, php: Option<String>, php_versions: Vec<String> },
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
    TOOLS_ScanDone(Vec<(String, crate::tabs::tools::PhpStatus, bool, bool, bool)>),
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
    TOOLS_ScanApacheModsDone(Vec<crate::tabs::tools::ApacheModule>),
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
    REPOS_FetchDone(Vec<crate::tabs::repos::RemoteRepo>),
    REPOS_Clone { ssh_url: String, name: String },
    REPOS_CloneDone(bool, String, String),
    REPOS_OpenCloned(String),
    REPOS_SearchChanged(String),
    REPOS_SetFilter(crate::tabs::repos::ProviderFilter),
    REPOS_OpenRoot,
    // VHosts
    VH_Scan,
    VH_ScanDone(Vec<VHostEntry>),
    VH_ShowAddForm,
    VH_HideForm,
    VH_FormServerNameChanged(String),
    VH_FormDocRootChanged(String),
    VH_FormPhpVersionChanged(Option<String>),
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
    VH_OpenConfigEditor,
    VH_CloseConfigEditor,
    VH_ConfigLoaded(String),
    VH_ConfigEditorAction(iced::widget::text_editor::Action),
    VH_SaveConfigFile,
    VH_SaveConfigDone(bool, String),
    // Auto-refresh
    AutoRefreshTick,
    // First-run
    FirstRun_Continue,
    FirstRun_Exit,
    FirstRun_InstallDone(bool, String),
    // Sudo
    Sudo_PasswordChanged(String),
    Sudo_ToggleShow(bool),
    Sudo_ToggleSave(bool),
    Sudo_Submit,
    Sudo_ValidationResult(bool),
    Sudo_Cancel,
    Sudo_ClearSaved,
}

// ── Toast ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub ok:      bool,
}

// ── App state ─────────────────────────────────────────────────────────────

pub struct App {
    pub active_tab:          Tab,
    pub config:              DevPanelConfig,
    pub dashboard:           DashboardTab,
    pub ssh_keys:            SshKeysTab,
    pub tools:               ToolsTab,
    pub repos:               ReposTab,
    pub vhosts:              VHostsTab,
    pub toast:               Option<Toast>,
    pub sudo:                SudoModal,
    pub sudo_pending_action: Option<PendingAction>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let config = DevPanelConfig::load();
        let app = Self {
            repos:               ReposTab::new(config.repos_root.clone(), config.repos_root.clone()),
            vhosts:              VHostsTab::new(config.devpanel_conf.clone()),
            active_tab:          Tab::Dashboard,
            dashboard:           DashboardTab::new(),
            ssh_keys:            SshKeysTab::new(),
            tools:               ToolsTab::new(),
            config,
            toast:               None,
            sudo:                SudoModal::new(),
            sudo_pending_action: None,
        };
        (app, Task::perform(crate::tabs::dashboard::probe_services(), |r| r))
    }

    // ── Toast helper ──────────────────────────────────────────────────────

    fn show_toast(&mut self, message: String, ok: bool) -> Task<Message> {
        self.toast = Some(Toast { message, ok });
        Task::perform(
            async { tokio::time::sleep(tokio::time::Duration::from_secs(4)).await },
            |_| Message::TOOLS_ClearToast,
        )
    }

    // ── Sudo helpers ──────────────────────────────────────────────────────

    pub fn trigger_sudo(&mut self, action: PendingAction) -> Task<Message> {
        if let Some(password) = self.sudo.get_password() {
            self.dispatch_sudo_action(action, password)
        } else {
            self.sudo_pending_action = Some(action.clone());
            self.sudo.state = ModalState::Asking { pending_action: action };
            self.sudo.password_input.clear();
            Task::none()
        }
    }

    pub fn dispatch_sudo_action(&mut self, action: PendingAction, password: String) -> Task<Message> {
        match action {
            // ── No more Box::leak — service and action are owned Strings ──
            PendingAction::ServiceControl { service, action: svc_action } => {
                Task::perform(run_service_cmd(service, svc_action, password), |r| r)
            }
            PendingAction::PhpSwitch(version) => Task::perform(
                crate::tabs::tools::switch_php(version, password),
                |(ok, msg)| Message::PhpSwitchResult(ok, msg),
            ),
            PendingAction::RestartAll => Task::batch([
                Task::perform(
                    run_service_cmd("apache2".to_string(), "restart".to_string(), password.clone()),
                    |r| r,
                ),
                Task::perform(
                    run_service_cmd("mysql".to_string(), "restart".to_string(), password),
                    |r| r,
                ),
            ]),
            PendingAction::PhpInstall(version) => Task::perform(
                crate::tabs::tools::apt_php_op(version, true, password),
                |(ok, msg)| Message::TOOLS_PhpOpDone(ok, msg),
            ),
            PendingAction::PhpRemove(version) => Task::perform(
                crate::tabs::tools::apt_php_op(version, false, password),
                |(ok, msg)| Message::TOOLS_PhpOpDone(ok, msg),
            ),
            PendingAction::VHostAdd { server_name, document_root, php_version } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    crate::tabs::vhosts::add_vhost(conf, server_name, document_root, php_version, password),
                    |(ok, msg)| Message::VH_CreateDone(ok, msg),
                )
            }
            PendingAction::VHostEdit { index, server_name, document_root, php_version } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    crate::tabs::vhosts::edit_vhost(conf, index, server_name, document_root, php_version, password),
                    |(ok, msg)| Message::VH_SaveEditDone(ok, msg),
                )
            }
            PendingAction::VHostDelete { index } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    crate::tabs::vhosts::delete_vhost(conf, index, password),
                    |(ok, msg)| Message::VH_DeleteDone(ok, msg),
                )
            }
            PendingAction::ApacheModToggle { name, enable } => Task::perform(
                crate::tabs::tools::toggle_apache_module(name, enable, password),
                |(ok, msg, n, en)| Message::TOOLS_ApacheModDone(ok, msg, n, en),
            ),
            PendingAction::AptInstall { package } => Task::perform(
                crate::tabs::tools::apt_package_op(package, true, password),
                |(ok, msg)| Message::TOOLS_PhpExtDone(ok, msg),
            ),
            PendingAction::AptRemove { package } => Task::perform(
                crate::tabs::tools::apt_package_op(package, false, password),
                |(ok, msg)| Message::TOOLS_PhpExtDone(ok, msg),
            ),
            PendingAction::SaveConfig { path, content } => Task::perform(
                crate::tabs::vhosts::save_config_file(path, content, password),
                |(ok, msg)| Message::VH_SaveConfigDone(ok, msg),
            ),
            PendingAction::FirstRunInstall => Task::perform(
                crate::first_run_install::run_first_run_install(password),
                |(ok, msg)| Message::FirstRun_InstallDone(ok, msg),
            ),
        }
    }

    // ── Subscription ──────────────────────────────────────────────────────

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(5))
            .map(|_| Message::AutoRefreshTick)
    }

    // ── Update ────────────────────────────────────────────────────────────

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectTab(tab) => {
                self.active_tab = tab.clone();
                match tab {
                    Tab::Dashboard => Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
                    Tab::SshKeys   => Task::perform(crate::tabs::ssh_keys::list_keys(), Message::SSH_KeysListed),
                    Tab::Tools => {
                        self.tools.scanning = true;
                        Task::perform(
                            crate::tabs::tools::scan_php_versions(self.dashboard.active_php_version.clone()),
                            Message::TOOLS_ScanDone,
                        )
                    }
                    Tab::Repos  => Task::none(),
                    Tab::VHosts => {
                        self.vhosts.scanning = true;
                        let conf = self.vhosts.devpanel_conf.clone();
                        Task::perform(crate::tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone)
                    }
                }
            }

            // ── Sudo ──────────────────────────────────────────────────────
            Message::Sudo_PasswordChanged(v) => { self.sudo.password_input = v; Task::none() }
            Message::Sudo_ToggleShow(v)      => { self.sudo.show_password = v;  Task::none() }
            Message::Sudo_ToggleSave(v) => {
                self.sudo.save_password = v;
                if !v { clear_saved_password(); }
                Task::none()
            }
            Message::Sudo_Cancel => {
                self.sudo.state = ModalState::Hidden;
                self.sudo.password_input.clear();
                Task::none()
            }
            Message::Sudo_Submit => {
                let pass = self.sudo.password_input.clone();
                if pass.is_empty() { return Task::none(); }
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
                if self.sudo.save_password { save_password(&password); }
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
                self.show_toast("Saved password cleared.".into(), true)
            }

            // ── Dashboard ─────────────────────────────────────────────────
            Message::RefreshStatus => Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
            Message::StatusRefreshed { apache, mysql, php, php_versions } => {
                self.dashboard.update_status(apache, mysql, php);
                self.dashboard.set_php_versions(php_versions);
                Task::none()
            }
            Message::StartApache   => self.trigger_sudo(PendingAction::ServiceControl { service: "apache2".into(), action: "start".into() }),
            Message::StopApache    => self.trigger_sudo(PendingAction::ServiceControl { service: "apache2".into(), action: "stop".into() }),
            Message::RestartApache => self.trigger_sudo(PendingAction::ServiceControl { service: "apache2".into(), action: "restart".into() }),
            Message::StartMySQL    => self.trigger_sudo(PendingAction::ServiceControl { service: "mysql".into(),   action: "start".into() }),
            Message::StopMySQL     => self.trigger_sudo(PendingAction::ServiceControl { service: "mysql".into(),   action: "stop".into() }),
            Message::RestartMySQL  => self.trigger_sudo(PendingAction::ServiceControl { service: "mysql".into(),   action: "restart".into() }),
            Message::RestartAll    => self.trigger_sudo(PendingAction::RestartAll),
            Message::ServiceResult { service, action, success, output } => {
                let msg = if success {
                    format!("{} {}ed", service, action)
                } else {
                    format!("Failed to {} {}: {}", action, service, output)
                };
                Task::batch([
                    self.show_toast(msg, success),
                    Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
                ])
            }
            Message::SwitchPHPVersion(v) => self.trigger_sudo(PendingAction::PhpSwitch(v)),
            Message::PhpSwitchResult(ok, msg) => Task::batch([
                self.show_toast(msg, ok),
                Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
            ]),
            Message::ShowPHPInfo        => { let _ = open_url("http://localhost/phpinfo.php"); Task::none() }
            Message::OpenLocalhost      => { let _ = open_url("http://localhost");             Task::none() }
            Message::OpenPhpMyAdmin     => { let _ = open_url("http://localhost/phpmyadmin");  Task::none() }
            Message::OpenWebRoot        => { let _ = xdg_open(&self.dashboard.web_root);       Task::none() }
            Message::OpenProjectsFolder => { let _ = xdg_open(&self.config.repos_root);        Task::none() }
            Message::NavigateApache2Conf  => { let _ = xdg_open("/etc/apache2/apache2.conf");      Task::none() }
            Message::NavigateApache2Sites => { let _ = xdg_open("/etc/apache2/sites-available");   Task::none() }
            Message::NavigatePhpDir       => { let _ = xdg_open("/etc/php");                       Task::none() }
            Message::NavigateMysqlDir     => { let _ = xdg_open("/etc/mysql");                     Task::none() }
            Message::NavigateHostsFile    => { let _ = xdg_open("/etc/hosts");                     Task::none() }
            Message::OpenPhpIni           => { let _ = open_php_ini(&self.dashboard.active_php_version); Task::none() }

            // ── Tools ─────────────────────────────────────────────────────
            Message::TOOLS_ScanPhp => {
                self.tools.scanning = true;
                Task::perform(
                    crate::tabs::tools::scan_php_versions(self.dashboard.active_php_version.clone()),
                    Message::TOOLS_ScanDone,
                )
            }
            Message::TOOLS_ScanDone(results) => {
                self.tools.apply_scan(results);
                let enabled: Vec<String> = self.tools.php_releases.iter()
                    .filter(|r| r.apache_mod_enabled)
                    .map(|r| r.version.clone())
                    .collect();
                self.vhosts.update_php_versions(enabled);
                Task::none()
            }
            Message::TOOLS_InstallPhp(ver) => {
                self.tools.push_log(true, format!("Queued install: PHP {}", ver));
                self.trigger_sudo(PendingAction::PhpInstall(ver))
            }
            Message::TOOLS_RemovePhp(ver) => {
                self.tools.push_log(true, format!("Queued removal: PHP {}", ver));
                self.trigger_sudo(PendingAction::PhpRemove(ver))
            }
            Message::TOOLS_PhpOpDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.tools.scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
                    Task::perform(
                        crate::tabs::tools::scan_php_versions(self.dashboard.active_php_version.clone()),
                        Message::TOOLS_ScanDone,
                    ),
                ])
            }
            Message::TOOLS_OpenMysqlCli => {
                self.tools.db_status = match open_db_terminal("mysql", false) {
                    Ok(s)  => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            Message::TOOLS_OpenMariadbCli => {
                self.tools.db_status = match open_db_terminal("mariadb", false) {
                    Ok(s)  => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            Message::TOOLS_OpenMysqlSocket => {
                self.tools.db_status = match open_db_terminal("mysql", true) {
                    Ok(s)  => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            Message::TOOLS_ClearLog   => { self.tools.install_log.clear(); self.tools.db_status.clear(); Task::none() }
            Message::TOOLS_ClearToast => { self.toast = None; Task::none() }
            Message::TOOLS_CopyFixCommands(commands) => {
                Task::perform(copy_to_clipboard(commands), |_| Message::TOOLS_CopyDone)
            }
            Message::TOOLS_CopyDone => self.show_toast("Commands copied to clipboard!".into(), true),
            Message::TOOLS_SetSection(s) => { self.tools.active_section = s; Task::none() }
            Message::TOOLS_ScanApacheMods => Task::perform(
                crate::tabs::tools::scan_apache_modules(),
                Message::TOOLS_ScanApacheModsDone,
            ),
            Message::TOOLS_ScanApacheModsDone(results) => {
                self.tools.apply_mod_scan(results);
                let enabled: Vec<String> = self.tools.php_releases.iter()
                    .filter(|r| r.apache_mod_enabled)
                    .map(|r| r.version.clone())
                    .collect();
                self.vhosts.update_php_versions(enabled);
                Task::none()
            }
            Message::TOOLS_ModFilterChanged(v) => { self.tools.mod_filter = v; Task::none() }
            Message::TOOLS_EnableApacheMod(name) => {
                self.tools.install_log.push((true, format!("Enabling mod_{}...", name)));
                self.trigger_sudo(PendingAction::ApacheModToggle { name, enable: true })
            }
            Message::TOOLS_DisableApacheMod(name) => {
                self.tools.install_log.push((true, format!("Disabling mod_{}...", name)));
                self.trigger_sudo(PendingAction::ApacheModToggle { name, enable: false })
            }
            Message::TOOLS_ApacheModDone(ok, msg, name, enabled) => {
                self.tools.push_log(ok, msg.clone());
                if ok { self.tools.set_mod_enabled(&name, enabled); }
                let enabled_vers: Vec<String> = self.tools.php_releases.iter()
                    .filter(|r| r.apache_mod_enabled)
                    .map(|r| r.version.clone())
                    .collect();
                self.vhosts.update_php_versions(enabled_vers);
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::tools::scan_apache_modules(), Message::TOOLS_ScanApacheModsDone),
                ])
            }
            Message::TOOLS_ScanPhpExts => {
                let active = self.tools.php_releases.iter().find(|r| r.is_active).map(|r| r.version.clone());
                Task::perform(crate::tabs::tools::scan_php_extensions(active), Message::TOOLS_ScanPhpExtsDone)
            }
            Message::TOOLS_ScanPhpExtsDone(results) => { self.tools.apply_ext_scan(results); Task::none() }
            Message::TOOLS_InstallPhpExt(pkg) => {
                self.tools.install_log.push((true, format!("Installing {}...", pkg)));
                self.trigger_sudo(PendingAction::AptInstall { package: pkg })
            }
            Message::TOOLS_RemovePhpExt(pkg) => {
                self.tools.install_log.push((true, format!("Removing {}...", pkg)));
                self.trigger_sudo(PendingAction::AptRemove { package: pkg })
            }
            Message::TOOLS_PhpExtDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                let active = self.tools.php_releases.iter().find(|r| r.is_active).map(|r| r.version.clone());
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::tools::scan_php_extensions(active), Message::TOOLS_ScanPhpExtsDone),
                ])
            }

            // ── SSH Keys ──────────────────────────────────────────────────
            Message::SSH_EmailChanged(v)      => { self.ssh_keys.email = v;            Task::none() }
            Message::SSH_KeyNameChanged(v)    => { self.ssh_keys.key_name = v;         Task::none() }
            Message::SSH_KeyTypeChanged(t)    => { self.ssh_keys.key_type = t;         Task::none() }
            Message::SSH_PassphraseChanged(v) => { self.ssh_keys.passphrase = v;       Task::none() }
            Message::SSH_TogglePassphrase(v)  => { self.ssh_keys.show_passphrase = v;  Task::none() }
            Message::SSH_GenerateKey => {
                let (email, name, ktype, pass) = (
                    self.ssh_keys.email.clone(),
                    self.ssh_keys.key_name.clone(),
                    self.ssh_keys.key_type.clone(),
                    self.ssh_keys.passphrase.clone(),
                );
                Task::perform(
                    crate::tabs::ssh_keys::generate_key(email, name, ktype, pass),
                    |(ok, msg)| Message::SSH_GenerateDone(ok, msg),
                )
            }
            Message::SSH_GenerateDone(ok, msg) => {
                use crate::tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind    = if ok { StatusKind::Success } else { StatusKind::Error };
                self.ssh_keys.status_message = msg.clone();
                if ok {
                    Task::batch([
                        self.show_toast(msg, ok),
                        Task::perform(crate::tabs::ssh_keys::list_keys(), Message::SSH_KeysListed),
                    ])
                } else {
                    self.show_toast(msg, ok)
                }
            }
            Message::SSH_AddExisting => {
                let path = format!("{}/.ssh/{}", get_home().display(), self.ssh_keys.key_name);
                Task::perform(ssh_add(path), |(ok, msg)| Message::SSH_AddExistingDone(ok, msg))
            }
            Message::SSH_AddExistingDone(ok, msg) => {
                use crate::tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind    = if ok { StatusKind::Success } else { StatusKind::Error };
                self.ssh_keys.status_message = msg.clone();
                self.show_toast(msg, ok)
            }
            Message::SSH_OpenDir => {
                let _ = xdg_open(&format!("{}/.ssh", get_home().display()));
                Task::none()
            }
            Message::SSH_ListKeys  => Task::perform(crate::tabs::ssh_keys::list_keys(), Message::SSH_KeysListed),
            Message::SSH_KeysListed(keys) => { self.ssh_keys.keys_list = keys; Task::none() }

            // ── Repos ─────────────────────────────────────────────────────
            Message::REPOS_CheckSsh => Task::perform(crate::tabs::repos::check_ssh(), |r| {
                Message::REPOS_SshChecked(r.github_ok, r.github_msg, r.bb_ok, r.bb_msg)
            }),
            Message::REPOS_SshChecked(gok, gmsg, bok, bmsg) => {
                self.repos.github_status    = if gok { SshStatus::Connected } else { SshStatus::Failed(gmsg) };
                self.repos.bitbucket_status = if bok { SshStatus::Connected } else { SshStatus::Failed(bmsg) };
                Task::none()
            }
            Message::REPOS_Fetch => {
                self.repos.fetching = true;
                let root = self.repos.repos_root.clone();
                Task::perform(crate::tabs::repos::fetch_remote_repos(root), Message::REPOS_FetchDone)
            }
            Message::REPOS_FetchDone(repos) => { self.repos.set_repos(repos); Task::none() }
            Message::REPOS_Clone { ssh_url, name } => {
                self.repos.mark_cloning(&ssh_url, true);
                let root = self.repos.repos_root.clone();
                Task::perform(
                    crate::tabs::repos::clone_repo(ssh_url, name, root),
                    |(ok, msg, url)| Message::REPOS_CloneDone(ok, msg, url),
                )
            }
            Message::REPOS_CloneDone(ok, msg, ssh_url) => {
                if ok { self.repos.mark_cloned(&ssh_url); } else { self.repos.mark_cloning(&ssh_url, false); }
                self.repos.status_msg = Some((ok, msg.clone()));
                self.show_toast(msg, ok)
            }
            Message::REPOS_OpenCloned(name) => {
                open_terminal_at(&format!("{}/{}", self.repos.repos_root, name));
                Task::none()
            }
            Message::REPOS_SearchChanged(v) => { self.repos.search_query = v;   Task::none() }
            Message::REPOS_SetFilter(f)     => { self.repos.active_filter = f;  Task::none() }
            Message::REPOS_OpenRoot         => { let _ = xdg_open(&self.repos.repos_root); Task::none() }

            // ── VHosts ────────────────────────────────────────────────────
            Message::VH_Scan => {
                self.vhosts.scanning = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone)
            }
            Message::VH_ScanDone(vhosts) => { self.vhosts.set_vhosts(vhosts); Task::none() }
            Message::VH_ShowAddForm       => { self.vhosts.form.open_add();    Task::none() }
            Message::VH_HideForm          => { self.vhosts.form.hide();        Task::none() }
            Message::VH_FormServerNameChanged(v)  => { self.vhosts.form.server_name   = v; Task::none() }
            Message::VH_FormDocRootChanged(v)     => { self.vhosts.form.document_root = v; Task::none() }
            Message::VH_FormPhpVersionChanged(v)  => { self.vhosts.form.php_version   = v; Task::none() }
            Message::VH_Create => {
                let sn  = self.vhosts.form.server_name.trim().to_string();
                let dr  = self.vhosts.form.document_root.trim().to_string();
                let php = self.vhosts.form.php_version.clone();
                self.trigger_sudo(PendingAction::VHostAdd { server_name: sn, document_root: dr, php_version: php })
            }
            Message::VH_CreateDone(ok, msg) => {
                self.vhosts.form.hide();
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone),
                ])
            }
            Message::VH_EditRequest(idx) => {
                if let Some(entry) = self.vhosts.vhosts.iter().find(|e| e.index == idx).cloned() {
                    self.vhosts.form.open_edit(&entry);
                }
                Task::none()
            }
            Message::VH_SaveEdit => {
                let sn  = self.vhosts.form.server_name.trim().to_string();
                let dr  = self.vhosts.form.document_root.trim().to_string();
                let php = self.vhosts.form.php_version.clone();
                let idx = match self.vhosts.form.mode {
                    crate::tabs::vhosts::FormMode::Edit(i) => i,
                    _ => return Task::none(),
                };
                self.trigger_sudo(PendingAction::VHostEdit { index: idx, server_name: sn, document_root: dr, php_version: php })
            }
            Message::VH_SaveEditDone(ok, msg) => {
                self.vhosts.form.hide();
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone),
                ])
            }
            Message::VH_OpenBrowser(sn)    => { let _ = open_url(&format!("http://{}", sn)); Task::none() }
            Message::VH_OpenDevpanelConf   => { let _ = xdg_open(&self.vhosts.devpanel_conf); Task::none() }
            Message::VH_DeleteRequest(idx) => { self.vhosts.confirm_delete = Some(idx);  Task::none() }
            Message::VH_DeleteCancel       => { self.vhosts.confirm_delete = None;        Task::none() }
            Message::VH_DeleteConfirm(idx) => {
                self.vhosts.confirm_delete = None;
                self.trigger_sudo(PendingAction::VHostDelete { index: idx })
            }
            Message::VH_DeleteDone(ok, msg) => {
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone),
                ])
            }
            Message::VH_OpenConfigEditor => {
                self.vhosts.view_mode       = crate::tabs::vhosts::VHostView::ConfigEditor;
                self.vhosts.config_loading  = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::tabs::vhosts::load_config_file(conf), Message::VH_ConfigLoaded)
            }
            Message::VH_CloseConfigEditor  => { self.vhosts.view_mode = crate::tabs::vhosts::VHostView::List; Task::none() }
            Message::VH_ConfigLoaded(text) => { self.vhosts.load_config_text(text); Task::none() }
            Message::VH_ConfigEditorAction(action) => {
                let is_edit = action.is_edit();
                self.vhosts.config_content.perform(action);
                if is_edit { self.vhosts.config_dirty = true; }
                Task::none()
            }
            Message::VH_SaveConfigFile => {
                self.vhosts.config_loading = true;
                let content = self.vhosts.config_content.text();
                let conf    = self.vhosts.devpanel_conf.clone();
                self.trigger_sudo(PendingAction::SaveConfig { content, path: conf })
            }
            Message::VH_SaveConfigDone(ok, msg) => {
                self.vhosts.config_loading = false;
                if ok { self.vhosts.config_dirty = false; }
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                if ok {
                    Task::batch([
                        self.show_toast(msg, ok),
                        Task::perform(crate::tabs::vhosts::scan_vhosts(conf), Message::VH_ScanDone),
                    ])
                } else {
                    self.show_toast(msg, ok)
                }
            }

            // ── First run ─────────────────────────────────────────────────
            Message::FirstRun_Continue => {
                crate::first_run::mark_done();
                self.trigger_sudo(PendingAction::FirstRunInstall)
            }
            Message::FirstRun_Exit => {
                std::process::exit(0);
            }
            Message::FirstRun_InstallDone(ok, msg) => {
                self.tools.scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
                    Task::perform(
                        crate::tabs::tools::scan_php_versions(self.dashboard.active_php_version.clone()),
                        Message::TOOLS_ScanDone,
                    ),
                ])
            }

            // ── Auto-refresh ──────────────────────────────────────────────
            Message::AutoRefreshTick => {
                if self.active_tab == Tab::Dashboard {
                    Task::perform(crate::tabs::dashboard::probe_services(), |r| r)
                } else {
                    Task::none()
                }
            }
        }
    }

    // ── View ──────────────────────────────────────────────────────────────

    pub fn view(&self) -> Element<'_, Message> {
        let tab_content: Element<Message> = match &self.active_tab {
            Tab::Dashboard => self.dashboard.view(),
            Tab::SshKeys   => self.ssh_keys.view(),
            Tab::Tools     => self.tools.view(),
            Tab::Repos     => self.repos.view(),
            Tab::VHosts    => self.vhosts.view(),
        };

        let main_body: Element<Message> = if let Some(toast) = &self.toast {
            let (accent, border_color) = if toast.ok {
                (GREEN, Color { r: 0.070, g: 0.210, b: 0.110, a: 1.0 })
            } else {
                (RED,   Color { r: 0.300, g: 0.090, b: 0.080, a: 1.0 })
            };
            let banner = container(
                row![
                    container(text(if toast.ok { "+" } else { "x" }).size(11).color(Color::WHITE))
                        .padding(Padding::from([3, 7]))
                        .style(move |_: &iced::Theme| container::Style {
                            background: Some(accent.into()),
                            border: Border { radius: 20.0.into(), ..Default::default() },
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
                border: Border { color: border_color, width: 1.0, ..Default::default() },
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
            container(app_area).width(Length::Fill).height(Length::Fill).into()
        }
    }

    // ── Sidebar ───────────────────────────────────────────────────────────

    fn sidebar(&self) -> Element<'_, Message> {
        let logo = container(
            column![
                row![
                    container(Space::with_width(3)).width(3).height(26).style(|_: &iced::Theme| container::Style {
                        background: Some(TEAL.into()),
                        border: Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
                    Space::with_width(10),
                    column![
                        text("dev").size(19).color(TEAL),
                        text("panel").size(19).color(TEXT_PRIMARY),
                    ].spacing(0),
                ].align_y(Alignment::Center)
            ].spacing(0),
        )
        .padding(Padding::from([22, 16]));

        let nav = column![
            self.nav_item("Dashboard",    Tab::Dashboard),
            self.nav_item("Repos",        Tab::Repos),
            self.nav_item("VirtualHosts", Tab::VHosts),
            self.nav_item("SSH Keys",     Tab::SshKeys),
            self.nav_item("Tools",        Tab::Tools),
        ]
        .spacing(2)
        .padding(Padding::from([0, 8]));

        let sudo_indicator: Element<Message> = if self.sudo.cached_password.is_some() {
            column![
                container(
                    row![
                        container(Space::with_width(6)).width(6).height(6).style(|_: &iced::Theme| container::Style {
                            background: Some(GREEN.into()),
                            border: Border { radius: 3.0.into(), ..Default::default() },
                            ..Default::default()
                        }),
                        Space::with_width(7),
                        text("sudo active").size(11).color(GREEN),
                    ].align_y(Alignment::Center),
                )
                .padding(Padding::from([6, 10]))
                .style(|_: &iced::Theme| container::Style {
                    background: Some(Color { r: 0.050, g: 0.160, b: 0.090, a: 1.0 }.into()),
                    border: Border { radius: 8.0.into(), ..Default::default() },
                    ..Default::default()
                }),
                Space::with_height(5),
                button(text("Clear sudo").size(11).color(TEXT_MUTED))
                    .on_press(Message::Sudo_ClearSaved)
                    .padding(Padding::from([4, 10]))
                    .style(|_, status| match status {
                        iced::widget::button::Status::Hovered => iced::widget::button::Style {
                            background: Some(BG_HOVER.into()), text_color: RED,
                            border: Border { radius: 6.0.into(), ..Default::default() },
                            ..Default::default()
                        },
                        _ => iced::widget::button::Style {
                            background: None, text_color: TEXT_MUTED, ..Default::default()
                        },
                    }),
            ]
            .spacing(0)
            .into()
        } else {
            container(
                row![
                    container(Space::with_width(6)).width(6).height(6).style(|_: &iced::Theme| container::Style {
                        background: Some(YELLOW.into()),
                        border: Border { radius: 3.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
                    Space::with_width(7),
                    text("sudo locked").size(11).color(TEXT_MUTED),
                ].align_y(Alignment::Center),
            )
            .padding(Padding::from([6, 10]))
            .style(|_: &iced::Theme| container::Style {
                background: Some(Color { r: 0.190, g: 0.160, b: 0.040, a: 1.0 }.into()),
                border: Border { radius: 8.0.into(), ..Default::default() },
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
                    ].align_y(Alignment::Center),
                )
                .on_press(Message::RefreshStatus)
                .padding(Padding::from([8, 12]))
                .width(Length::Fill)
                .style(|_, status| match status {
                    iced::widget::button::Status::Hovered => iced::widget::button::Style {
                        background: Some(BG_HOVER.into()), text_color: TEXT_PRIMARY,
                        border: Border { radius: 8.0.into(), ..Default::default() },
                        ..Default::default()
                    },
                    _ => iced::widget::button::Style {
                        background: None, text_color: TEXT_MUTED, ..Default::default()
                    },
                }),
                Space::with_height(8),
                text(format!("v{}", env!("CARGO_PKG_VERSION"))).size(11).color(TEXT_MUTED),
            ]
            .spacing(0)
            .align_x(Alignment::Start),
        )
        .padding(Padding::from([10, 14]));

        container(
            column![logo, divider(), Space::with_height(10), nav, Space::with_height(Length::Fill), divider(), bottom]
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
        let active     = self.active_tab == tab;
        let bg         = if active { Color { r: 0.060, g: 0.185, b: 0.175, a: 1.0 } } else { Color::TRANSPARENT };
        let text_color = if active { TEXT_PRIMARY }   else { TEXT_SECONDARY };
        let icon_color = if active { TEAL }            else { TEXT_MUTED };
        button(
            row![
                text("").size(12).color(icon_color),
                Space::with_width(10),
                text(label).size(13).color(text_color),
            ].align_y(Alignment::Center),
        )
        .on_press(Message::SelectTab(tab))
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(move |_, status| match status {
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: Some(BG_HOVER.into()), text_color: TEXT_PRIMARY,
                border: Border { radius: 8.0.into(), ..Default::default() },
                ..Default::default()
            },
            _ => iced::widget::button::Style {
                background: Some(bg.into()), text_color,
                border: Border { radius: 8.0.into(), ..Default::default() },
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