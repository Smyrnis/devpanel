use iced::Task;

use crate::app::App;
use crate::core::{
    db::{DevPanelDb, UserSettings},
    first_run, setup_log,
    sudo_prompt::{
        ModalState, PendingAction, clear_saved_password, save_password, validate_sudo_password,
    },
    system::{
        copy_to_clipboard, get_home, open_db_terminal, open_php_ini, open_terminal_at, open_url,
        run_service_cmd, ssh_add, xdg_open,
    },
};
use crate::messages::{
    ConfigMessage, DashboardMessage, FirstRunMessage, Message, ReposMessage, SshKeysMessage,
    SudoMessage, Tab, ToolsMessage, VHostsMessage,
};
use crate::tabs::repos::SshStatus;

impl App {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectTab(tab) => self.handle_select_tab(tab),
            Message::NotificationTick => {
                for notification in &mut self.notifications {
                    notification.remaining_ms = notification.remaining_ms.saturating_sub(1000);
                }
                self.notifications.retain(|n| n.remaining_ms > 0);
                Task::none()
            }
            Message::DismissAllNotifications => {
                self.notifications.clear();
                Task::none()
            }
            Message::Dashboard(m) => self.handle_dashboard(m),
            Message::SshKeys(m) => self.handle_ssh_keys(m),
            Message::Tools(m) => self.handle_tools(m),
            Message::Repos(m) => self.handle_repos(m),
            Message::VHosts(m) => self.handle_vhosts(m),
            Message::Sudo(m) => self.handle_sudo(m),
            Message::FirstRun(m) => self.handle_first_run(m),
            Message::Config(m) => self.handle_config(m),
        }
    }
}

impl App {
    fn handle_select_tab(&mut self, tab: Tab) -> Task<Message> {
        self.active_tab = tab.clone();
        match tab {
            Tab::Dashboard => Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
            Tab::SshKeys => Task::perform(crate::tabs::ssh_keys::list_keys(), |keys| {
                Message::SshKeys(SshKeysMessage::KeysListed(keys))
            }),
            Tab::Tools => {
                self.tools.scanning = true;
                self.tools.tools_scanning = true;
                Task::batch([
                    Task::perform(
                        crate::tabs::tools::scan_php_versions(
                            self.dashboard.active_php_version.clone(),
                        ),
                        |r| Message::Tools(ToolsMessage::ScanDone(r)),
                    ),
                    Task::perform(crate::tabs::tools::scan_installed_tools(), |r| {
                        Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                    }),
                ])
            }
            Tab::Repos => Task::none(),
            Tab::VHosts => {
                self.vhosts.scanning = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                    Message::VHosts(VHostsMessage::ScanDone(v))
                })
            }
            Tab::Config => Task::none(),
        }
    }
}

impl App {
    fn handle_sudo(&mut self, msg: SudoMessage) -> Task<Message> {
        match msg {
            SudoMessage::PasswordChanged(v) => {
                self.sudo.password_input = v;
                Task::none()
            }
            SudoMessage::ToggleShow(v) => {
                self.sudo.show_password = v;
                Task::none()
            }
            SudoMessage::ToggleSave(v) => {
                self.sudo.save_password = v;
                if !v {
                    clear_saved_password();
                }
                Task::none()
            }
            SudoMessage::Cancel => {
                if matches!(
                    self.sudo_pending_action,
                    Some(PendingAction::FirstRunInstall { .. })
                ) {
                    self.first_run_installing = false;
                }
                self.sudo.state = ModalState::Hidden;
                self.sudo.password_input.clear();
                Task::none()
            }
            SudoMessage::Submit => {
                let pass = self.sudo.password_input.clone();
                if pass.is_empty() {
                    return Task::none();
                }
                self.sudo.state = ModalState::Validating;
                Task::perform(validate_sudo_password(pass), |ok| {
                    Message::Sudo(SudoMessage::ValidationResult(ok))
                })
            }
            SudoMessage::ValidationResult(valid) => {
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
            SudoMessage::ClearSaved => {
                clear_saved_password();
                self.sudo.cached_password = None;
                self.sudo.save_password = false;
                self.show_toast("Saved password cleared.".into(), true)
            }
        }
    }

    pub fn trigger_sudo(&mut self, action: PendingAction) -> Task<Message> {
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

    pub fn dispatch_sudo_action(
        &mut self,
        action: PendingAction,
        password: String,
    ) -> Task<Message> {
        match action {
            PendingAction::ServiceControl {
                service,
                action: svc_action,
            } => Task::perform(run_service_cmd(service, svc_action, password), |r| r),

            PendingAction::PhpSwitch(version) => Task::perform(
                crate::tabs::tools::switch_php(version, password),
                |(ok, msg)| Message::Dashboard(DashboardMessage::PhpSwitchResult(ok, msg)),
            ),

            PendingAction::RestartAll => Task::batch([
                Task::perform(
                    run_service_cmd("apache2".into(), "restart".into(), password.clone()),
                    |r| r,
                ),
                Task::perform(
                    run_service_cmd("mysql".into(), "restart".into(), password),
                    |r| r,
                ),
            ]),

            PendingAction::PhpInstall(version) => Task::perform(
                crate::tabs::tools::apt_php_op(version, true, password),
                |(ok, msg)| Message::Tools(ToolsMessage::PhpOpDone(ok, msg)),
            ),

            PendingAction::PhpRemove(version) => Task::perform(
                crate::tabs::tools::apt_php_op(version, false, password),
                |(ok, msg)| Message::Tools(ToolsMessage::PhpOpDone(ok, msg)),
            ),

            PendingAction::VHostAdd {
                server_name,
                document_root,
                php_version,
                https_enabled,
            } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    crate::tabs::vhosts::add_vhost(
                        conf,
                        server_name,
                        document_root,
                        php_version,
                        https_enabled,
                        password,
                    ),
                    |(ok, msg)| Message::VHosts(VHostsMessage::CreateDone(ok, msg)),
                )
            }

            PendingAction::VHostEdit {
                index,
                server_name,
                document_root,
                php_version,
                https_enabled,
            } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    crate::tabs::vhosts::edit_vhost(
                        conf,
                        index,
                        server_name,
                        document_root,
                        php_version,
                        https_enabled,
                        password,
                    ),
                    |(ok, msg)| Message::VHosts(VHostsMessage::SaveEditDone(ok, msg)),
                )
            }

            PendingAction::VHostDelete { index } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    crate::tabs::vhosts::delete_vhost(conf, index, password),
                    |(ok, msg)| Message::VHosts(VHostsMessage::DeleteDone(ok, msg)),
                )
            }

            PendingAction::VHostBulkDelete { indexes } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    crate::tabs::vhosts::bulk_delete_vhosts(conf, indexes, password),
                    |(ok, msg)| Message::VHosts(VHostsMessage::DeleteDone(ok, msg)),
                )
            }

            PendingAction::VHostToggleHttps { index } => {
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(
                    crate::tabs::vhosts::toggle_https(conf, index, password),
                    |(ok, msg)| Message::VHosts(VHostsMessage::SaveEditDone(ok, msg)),
                )
            }

            PendingAction::ApacheModToggle { name, enable } => Task::perform(
                crate::tabs::tools::toggle_apache_module(name, enable, password),
                |(ok, msg, n, en)| Message::Tools(ToolsMessage::ApacheModDone(ok, msg, n, en)),
            ),

            PendingAction::AptInstall { package } => Task::perform(
                crate::tabs::tools::apt_package_op(package, true, password),
                |(ok, msg)| Message::Tools(ToolsMessage::PhpExtDone(ok, msg)),
            ),

            PendingAction::AptRemove { package } => Task::perform(
                crate::tabs::tools::apt_package_op(package, false, password),
                |(ok, msg)| Message::Tools(ToolsMessage::PhpExtDone(ok, msg)),
            ),

            PendingAction::ComposerInstall { update } => Task::perform(
                crate::tabs::tools::composer_op(update, password),
                |(ok, msg)| Message::Tools(ToolsMessage::ComposerDone(ok, msg)),
            ),

            PendingAction::RedisService { action } => Task::perform(
                crate::tabs::tools::redis_service_op(action, password),
                |(ok, msg)| Message::Tools(ToolsMessage::RedisDone(ok, msg)),
            ),

            PendingAction::SaveConfig { path, content } => Task::perform(
                crate::tabs::vhosts::save_config_file(path, content, password),
                |(ok, msg)| Message::VHosts(VHostsMessage::SaveConfigDone(ok, msg)),
            ),

            PendingAction::FirstRunInstall { options } => Task::perform(
                crate::core::first_run_install::run_first_run_install(password, options),
                |(ok, msg)| Message::FirstRun(FirstRunMessage::InstallDone(ok, msg)),
            ),
        }
    }
}

impl App {
    fn handle_dashboard(&mut self, msg: DashboardMessage) -> Task<Message> {
        match msg {
            DashboardMessage::RefreshStatus => {
                Task::perform(crate::tabs::dashboard::probe_services(), |r| r)
            }

            DashboardMessage::StatusRefreshed {
                apache,
                mysql,
                php,
                php_versions,
                apache_uptime,
                mysql_uptime,
                recent_failures,
            } => {
                self.dashboard.update_status(
                    apache,
                    mysql,
                    php,
                    apache_uptime,
                    mysql_uptime,
                    recent_failures,
                );
                self.dashboard.set_php_versions(php_versions);
                if !self.setup_issues_checked {
                    self.setup_issues_checked = true;
                    let show = self.config_tab.settings.ui_show_setup_log;
                    if show {
                        let issues = setup_log::read_setup_issues();
                        if !issues.is_empty() {
                            let summary = format!(
                                "{} post-install issue(s) — check /var/log/devpanel/setup.log",
                                issues.len()
                            );
                            return self.show_toast(summary, false);
                        }
                    }
                }
                Task::none()
            }

            DashboardMessage::ResetIssuesCheck => {
                self.setup_issues_checked = false;
                Task::none()
            }

            DashboardMessage::AutoRefreshTick => {
                if self.active_tab == Tab::Dashboard {
                    Task::perform(crate::tabs::dashboard::probe_services(), |r| r)
                } else {
                    Task::none()
                }
            }

            DashboardMessage::StartApache => self.trigger_sudo(PendingAction::ServiceControl {
                service: "apache2".into(),
                action: "start".into(),
            }),
            DashboardMessage::StopApache => self.trigger_sudo(PendingAction::ServiceControl {
                service: "apache2".into(),
                action: "stop".into(),
            }),
            DashboardMessage::RestartApache => self.trigger_sudo(PendingAction::ServiceControl {
                service: "apache2".into(),
                action: "restart".into(),
            }),
            DashboardMessage::StartMySQL => self.trigger_sudo(PendingAction::ServiceControl {
                service: "mysql".into(),
                action: "start".into(),
            }),
            DashboardMessage::StopMySQL => self.trigger_sudo(PendingAction::ServiceControl {
                service: "mysql".into(),
                action: "stop".into(),
            }),
            DashboardMessage::RestartMySQL => self.trigger_sudo(PendingAction::ServiceControl {
                service: "mysql".into(),
                action: "restart".into(),
            }),
            DashboardMessage::RestartAll => self.trigger_sudo(PendingAction::RestartAll),

            DashboardMessage::ServiceResult {
                service,
                action,
                success,
                output,
            } => {
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

            DashboardMessage::SwitchPhpVersion(v) => self.trigger_sudo(PendingAction::PhpSwitch(v)),

            DashboardMessage::PhpSwitchResult(ok, msg) => Task::batch([
                self.show_toast(msg, ok),
                Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
            ]),

            DashboardMessage::ShowPhpInfo => {
                self.dashboard.php_info_loading = true;
                self.dashboard.php_info = None;
                Task::perform(
                    crate::tabs::dashboard::backend::php_info_summary(),
                    |text| Message::Dashboard(DashboardMessage::PhpInfoLoaded(text)),
                )
            }
            DashboardMessage::PhpInfoLoaded(text) => {
                self.dashboard.php_info_loading = false;
                self.dashboard.php_info = Some(text);
                Task::none()
            }
            DashboardMessage::ClosePhpInfo => {
                self.dashboard.php_info_loading = false;
                self.dashboard.php_info = None;
                Task::none()
            }
            DashboardMessage::OpenLocalhost => {
                let _ = open_url("http://localhost");
                Task::none()
            }
            DashboardMessage::OpenPhpMyAdmin => {
                let _ = open_url("http://localhost/phpmyadmin");
                Task::none()
            }
            DashboardMessage::OpenWebRoot => {
                let _ = xdg_open(&self.dashboard.web_root);
                Task::none()
            }
            DashboardMessage::OpenProjectsFolder => {
                let _ = xdg_open(&self.config.repos_root);
                Task::none()
            }
            DashboardMessage::NavigateApache2Conf => {
                let _ = xdg_open("/etc/apache2/apache2.conf");
                Task::none()
            }
            DashboardMessage::NavigateApache2Sites => {
                let _ = xdg_open("/etc/apache2/sites-available");
                Task::none()
            }
            DashboardMessage::NavigatePhpDir => {
                let _ = xdg_open("/etc/php");
                Task::none()
            }
            DashboardMessage::NavigateMysqlDir => {
                let _ = xdg_open("/etc/mysql");
                Task::none()
            }
            DashboardMessage::NavigateHostsFile => {
                let _ = xdg_open("/etc/hosts");
                Task::none()
            }
            DashboardMessage::OpenPhpIni => {
                let _ = open_php_ini(&self.dashboard.active_php_version);
                Task::none()
            }
        }
    }
}

impl App {
    fn handle_ssh_keys(&mut self, msg: SshKeysMessage) -> Task<Message> {
        match msg {
            SshKeysMessage::EmailChanged(v) => {
                self.ssh_keys.email = v;
                Task::none()
            }
            SshKeysMessage::KeyNameChanged(v) => {
                self.ssh_keys.key_name = v;
                Task::none()
            }
            SshKeysMessage::KeyTypeChanged(t) => {
                self.ssh_keys.key_type = t;
                Task::none()
            }
            SshKeysMessage::PassphraseChanged(v) => {
                self.ssh_keys.passphrase = v;
                Task::none()
            }
            SshKeysMessage::TogglePassphrase(v) => {
                self.ssh_keys.show_passphrase = v;
                Task::none()
            }

            SshKeysMessage::GenerateKey => {
                let (email, name, ktype, pass) = (
                    self.ssh_keys.email.clone(),
                    self.ssh_keys.key_name.clone(),
                    self.ssh_keys.key_type,
                    self.ssh_keys.passphrase.clone(),
                );
                Task::perform(
                    crate::tabs::ssh_keys::generate_key(email, name, ktype, pass),
                    |(ok, msg)| Message::SshKeys(SshKeysMessage::GenerateDone(ok, msg)),
                )
            }

            SshKeysMessage::GenerateDone(ok, msg) => {
                use crate::tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                self.ssh_keys.status_message = msg.clone();
                if ok {
                    Task::batch([
                        self.show_toast(msg, ok),
                        Task::perform(crate::tabs::ssh_keys::list_keys(), |keys| {
                            Message::SshKeys(SshKeysMessage::KeysListed(keys))
                        }),
                    ])
                } else {
                    self.show_toast(msg, ok)
                }
            }

            SshKeysMessage::AddExisting => {
                let path = format!("{}/.ssh/{}", get_home().display(), self.ssh_keys.key_name);
                Task::perform(ssh_add(path), |(ok, msg)| {
                    Message::SshKeys(SshKeysMessage::AddExistingDone(ok, msg))
                })
            }

            SshKeysMessage::AddExistingDone(ok, msg) => {
                use crate::tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                self.ssh_keys.status_message = msg.clone();
                self.show_toast(msg, ok)
            }

            SshKeysMessage::OpenDir => {
                let _ = xdg_open(&format!("{}/.ssh", get_home().display()));
                Task::none()
            }

            SshKeysMessage::ListKeys => Task::perform(crate::tabs::ssh_keys::list_keys(), |keys| {
                Message::SshKeys(SshKeysMessage::KeysListed(keys))
            }),

            SshKeysMessage::KeysListed(keys) => {
                self.ssh_keys.keys_list = keys;
                Task::none()
            }
            SshKeysMessage::CopyPublicKey(path) => Task::perform(
                async move {
                    match crate::tabs::ssh_keys::read_public_key(path).await {
                        Ok(text) => {
                            crate::core::system::copy_to_clipboard(text).await;
                            (true, "Public key copied".to_string())
                        }
                        Err(e) => (false, e),
                    }
                },
                |(ok, msg)| Message::SshKeys(SshKeysMessage::CopyPublicKeyDone(ok, msg)),
            ),
            SshKeysMessage::CopyPublicKeyDone(ok, msg) => {
                use crate::tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                self.ssh_keys.status_message = msg.clone();
                self.show_toast(msg, ok)
            }
        }
    }
}

impl App {
    fn handle_tools(&mut self, msg: ToolsMessage) -> Task<Message> {
        match msg {
            ToolsMessage::ScanPhp => {
                self.tools.scanning = true;
                Task::perform(
                    crate::tabs::tools::scan_php_versions(
                        self.dashboard.active_php_version.clone(),
                    ),
                    |r| Message::Tools(ToolsMessage::ScanDone(r)),
                )
            }

            ToolsMessage::ScanDone(results) => {
                self.tools.apply_scan(results);
                self.sync_php_versions_to_vhosts();
                Task::none()
            }

            ToolsMessage::InstallPhp(ver) => {
                self.tools
                    .push_log(true, format!("Queued install: PHP {}", ver));
                self.trigger_sudo(PendingAction::PhpInstall(ver))
            }
            ToolsMessage::RemovePhp(ver) => {
                self.tools
                    .push_log(true, format!("Queued removal: PHP {}", ver));
                self.trigger_sudo(PendingAction::PhpRemove(ver))
            }

            ToolsMessage::PhpOpDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.tools.scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
                    Task::perform(
                        crate::tabs::tools::scan_php_versions(
                            self.dashboard.active_php_version.clone(),
                        ),
                        |r| Message::Tools(ToolsMessage::ScanDone(r)),
                    ),
                ])
            }

            ToolsMessage::OpenMysqlCli => {
                self.tools.db_status = match open_db_terminal("mysql", false) {
                    Ok(s) => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            ToolsMessage::OpenMariadbCli => {
                self.tools.db_status = match open_db_terminal("mariadb", false) {
                    Ok(s) => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            ToolsMessage::OpenMysqlSocket => {
                self.tools.db_status = match open_db_terminal("mysql", true) {
                    Ok(s) => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }

            ToolsMessage::ClearLog => {
                self.tools.install_log.clear();
                self.tools.db_status.clear();
                Task::none()
            }
            ToolsMessage::CopyFixCommands(commands) => {
                Task::perform(copy_to_clipboard(commands), |_| {
                    Message::Tools(ToolsMessage::CopyDone)
                })
            }

            ToolsMessage::CopyDone => self.show_toast("Commands copied to clipboard!".into(), true),

            ToolsMessage::SetSection(s) => {
                self.tools.active_section = s;
                Task::none()
            }

            ToolsMessage::ToolSearchChanged(v) => {
                self.tools.tool_search = v;
                Task::none()
            }

            ToolsMessage::ScanInstalledTools => {
                self.tools.tools_scanning = true;
                Task::perform(crate::tabs::tools::scan_installed_tools(), |r| {
                    Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                })
            }

            ToolsMessage::InstalledToolsScanned(tools) => {
                self.tools.apply_tools_scan(tools);
                Task::none()
            }

            ToolsMessage::InstallComposer => {
                self.tools.push_log(true, "Queued Composer install".into());
                self.trigger_sudo(PendingAction::ComposerInstall { update: false })
            }

            ToolsMessage::UpdateComposer => {
                self.tools.push_log(true, "Queued Composer update".into());
                self.trigger_sudo(PendingAction::ComposerInstall { update: true })
            }

            ToolsMessage::ComposerDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.tools.tools_scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::tools::scan_installed_tools(), |r| {
                        Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                    }),
                ])
            }

            ToolsMessage::CopyNvmInstallCommand => {
                let command = "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash".to_string();
                Task::perform(copy_to_clipboard(command), |_| {
                    Message::Tools(ToolsMessage::CopyDone)
                })
            }

            ToolsMessage::RedisStart => {
                self.tools.push_log(true, "Starting Redis".into());
                self.trigger_sudo(PendingAction::RedisService {
                    action: "start".into(),
                })
            }

            ToolsMessage::RedisStop => {
                self.tools.push_log(true, "Stopping Redis".into());
                self.trigger_sudo(PendingAction::RedisService {
                    action: "stop".into(),
                })
            }

            ToolsMessage::RedisDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.tools.tools_scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::tools::scan_installed_tools(), |r| {
                        Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                    }),
                ])
            }

            ToolsMessage::ScanApacheMods => {
                Task::perform(crate::tabs::tools::scan_apache_modules(), |r| {
                    Message::Tools(ToolsMessage::ScanApacheModsDone(r))
                })
            }

            ToolsMessage::ScanApacheModsDone(results) => {
                self.tools.apply_mod_scan(results);
                self.sync_php_versions_to_vhosts();
                Task::none()
            }

            ToolsMessage::ModFilterChanged(v) => {
                self.tools.mod_filter = v;
                Task::none()
            }

            ToolsMessage::EnableApacheMod(name) => {
                self.tools
                    .install_log
                    .push((true, format!("Enabling mod_{}…", name)));
                self.trigger_sudo(PendingAction::ApacheModToggle { name, enable: true })
            }
            ToolsMessage::DisableApacheMod(name) => {
                self.tools
                    .install_log
                    .push((true, format!("Disabling mod_{}…", name)));
                self.trigger_sudo(PendingAction::ApacheModToggle {
                    name,
                    enable: false,
                })
            }

            ToolsMessage::ApacheModDone(ok, msg, name, enabled) => {
                self.tools.push_log(ok, msg.clone());
                if ok {
                    self.tools.set_mod_enabled(&name, enabled);
                }
                self.sync_php_versions_to_vhosts();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::tools::scan_apache_modules(), |r| {
                        Message::Tools(ToolsMessage::ScanApacheModsDone(r))
                    }),
                ])
            }

            ToolsMessage::ScanPhpExts => {
                let active = self
                    .tools
                    .php_releases
                    .iter()
                    .find(|r| r.is_active)
                    .map(|r| r.version.clone());
                Task::perform(crate::tabs::tools::scan_php_extensions(active), |r| {
                    Message::Tools(ToolsMessage::ScanPhpExtsDone(r))
                })
            }

            ToolsMessage::ScanPhpExtsDone(results) => {
                self.tools.apply_ext_scan(results);
                Task::none()
            }

            ToolsMessage::InstallPhpExt(pkg) => {
                self.tools
                    .install_log
                    .push((true, format!("Installing {}…", pkg)));
                self.trigger_sudo(PendingAction::AptInstall { package: pkg })
            }
            ToolsMessage::RemovePhpExt(pkg) => {
                self.tools
                    .install_log
                    .push((true, format!("Removing {}…", pkg)));
                self.trigger_sudo(PendingAction::AptRemove { package: pkg })
            }

            ToolsMessage::PhpExtDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                let active = self
                    .tools
                    .php_releases
                    .iter()
                    .find(|r| r.is_active)
                    .map(|r| r.version.clone());
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::tools::scan_php_extensions(active), |r| {
                        Message::Tools(ToolsMessage::ScanPhpExtsDone(r))
                    }),
                ])
            }
        }
    }

    fn sync_php_versions_to_vhosts(&mut self) {
        let enabled: Vec<String> = self
            .tools
            .php_releases
            .iter()
            .filter(|r| r.apache_mod_enabled)
            .map(|r| r.version.clone())
            .collect();
        self.vhosts.update_php_versions(enabled);
    }
}

impl App {
    fn handle_repos(&mut self, msg: ReposMessage) -> Task<Message> {
        match msg {
            ReposMessage::CheckSsh => Task::perform(crate::tabs::repos::check_ssh(), |r| {
                Message::Repos(ReposMessage::SshChecked(
                    r.github_ok,
                    r.github_msg,
                    r.bb_ok,
                    r.bb_msg,
                ))
            }),

            ReposMessage::SshChecked(gok, gmsg, bok, bmsg) => {
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

            ReposMessage::Fetch => {
                self.repos.fetching = true;
                let root = self.repos.repos_root.clone();
                Task::perform(crate::tabs::repos::fetch_remote_repos(root), |repos| {
                    Message::Repos(ReposMessage::FetchDone(repos))
                })
            }

            ReposMessage::FetchDone(repos) => {
                self.repos.set_repos(repos);
                Task::none()
            }
            ReposMessage::NextPage => {
                let pages = self.repos.remote_repos.len().saturating_sub(1) / self.repos.page_size;
                self.repos.page = (self.repos.page + 1).min(pages);
                Task::none()
            }
            ReposMessage::PrevPage => {
                self.repos.page = self.repos.page.saturating_sub(1);
                Task::none()
            }

            ReposMessage::Clone { ssh_url, name } => {
                self.repos.mark_cloning(&ssh_url, true);
                let root = self.repos.repos_root.clone();
                Task::perform(
                    crate::tabs::repos::clone_repo(ssh_url, name, root),
                    |(ok, msg, url)| Message::Repos(ReposMessage::CloneDone(ok, msg, url)),
                )
            }

            ReposMessage::CloneDone(ok, msg, ssh_url) => {
                if ok {
                    self.repos.mark_cloned(&ssh_url);
                } else {
                    self.repos.mark_cloning(&ssh_url, false);
                }
                self.repos.status_msg = Some((ok, msg.clone()));
                self.show_toast(msg, ok)
            }

            ReposMessage::OpenCloned(name) => {
                open_terminal_at(&format!("{}/{}", self.repos.repos_root, name));
                Task::none()
            }
            ReposMessage::OpenEditor(name) => {
                let path = format!("{}/{}", self.repos.repos_root, name);
                let cmd = self.config_tab.settings.editor_command.clone();
                match crate::core::system::open_in_editor(&cmd, &path) {
                    Ok(_) => Task::none(),
                    Err(e) => self.show_toast(e, false),
                }
            }

            ReposMessage::SearchChanged(v) => {
                self.repos.search_query = v;
                Task::none()
            }
            ReposMessage::SetFilter(f) => {
                self.repos.active_filter = f;
                Task::none()
            }
            ReposMessage::OpenRoot => {
                let _ = xdg_open(&self.repos.repos_root);
                Task::none()
            }
        }
    }
}

impl App {
    fn handle_vhosts(&mut self, msg: VHostsMessage) -> Task<Message> {
        match msg {
            VHostsMessage::Scan => {
                self.vhosts.scanning = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                    Message::VHosts(VHostsMessage::ScanDone(v))
                })
            }

            VHostsMessage::ScanDone(vhosts) => {
                self.vhosts.set_vhosts(vhosts);
                Task::none()
            }
            VHostsMessage::ShowAddForm => {
                self.vhosts.form.open_add();
                Task::none()
            }
            VHostsMessage::HideForm => {
                self.vhosts.form.hide();
                Task::none()
            }

            VHostsMessage::FormServerNameChanged(v) => {
                self.vhosts.form.server_name = v;
                Task::none()
            }
            VHostsMessage::FormDocRootChanged(v) => {
                self.vhosts.form.document_root = v;
                Task::none()
            }
            VHostsMessage::FormPhpVersionChanged(v) => {
                self.vhosts.form.php_version = v;
                Task::none()
            }
            VHostsMessage::FormHttpsChanged(v) => {
                self.vhosts.form.https_enabled = v;
                Task::none()
            }

            VHostsMessage::Create => {
                let sn = self.vhosts.form.server_name.trim().to_string();
                let dr = self.vhosts.form.document_root.trim().to_string();
                let php = self.vhosts.form.php_version.clone();
                let https_enabled = self.vhosts.form.https_enabled;
                self.trigger_sudo(PendingAction::VHostAdd {
                    server_name: sn,
                    document_root: dr,
                    php_version: php,
                    https_enabled,
                })
            }

            VHostsMessage::CreateDone(ok, msg) => {
                self.vhosts.form.hide();
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                        Message::VHosts(VHostsMessage::ScanDone(v))
                    }),
                ])
            }

            VHostsMessage::EditRequest(idx) => {
                if let Some(entry) = self.vhosts.vhosts.iter().find(|e| e.index == idx).cloned() {
                    self.vhosts.form.open_edit(&entry);
                }
                Task::none()
            }

            VHostsMessage::SaveEdit => {
                let sn = self.vhosts.form.server_name.trim().to_string();
                let dr = self.vhosts.form.document_root.trim().to_string();
                let php = self.vhosts.form.php_version.clone();
                let https_enabled = self.vhosts.form.https_enabled;
                let idx = match self.vhosts.form.mode {
                    crate::tabs::vhosts::FormMode::Edit(i) => i,
                    _ => return Task::none(),
                };
                self.trigger_sudo(PendingAction::VHostEdit {
                    index: idx,
                    server_name: sn,
                    document_root: dr,
                    php_version: php,
                    https_enabled,
                })
            }

            VHostsMessage::SaveEditDone(ok, msg) => {
                self.vhosts.form.hide();
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                        Message::VHosts(VHostsMessage::ScanDone(v))
                    }),
                ])
            }

            VHostsMessage::OpenBrowser(sn) => {
                let _ = open_url(&format!("http://{}", sn));
                Task::none()
            }
            VHostsMessage::OpenDevpanelConf => {
                let _ = xdg_open(&self.vhosts.devpanel_conf);
                Task::none()
            }

            VHostsMessage::DeleteRequest(idx) => {
                self.vhosts.confirm_delete = Some(idx);
                Task::none()
            }
            VHostsMessage::DeleteCancel => {
                self.vhosts.confirm_delete = None;
                Task::none()
            }

            VHostsMessage::DeleteConfirm(idx) => {
                self.vhosts.confirm_delete = None;
                self.trigger_sudo(PendingAction::VHostDelete { index: idx })
            }

            VHostsMessage::BulkDeleteConfirm => {
                let indexes = self.vhosts.selected.clone();
                if indexes.is_empty() {
                    return Task::none();
                }
                self.vhosts.confirm_delete = None;
                self.trigger_sudo(PendingAction::VHostBulkDelete { indexes })
            }

            VHostsMessage::DeleteDone(ok, msg) => {
                if ok {
                    self.vhosts.selected.clear();
                }
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                        Message::VHosts(VHostsMessage::ScanDone(v))
                    }),
                ])
            }

            VHostsMessage::ToggleSelected(idx) => {
                if self.vhosts.selected.contains(&idx) {
                    self.vhosts.selected.retain(|i| *i != idx);
                } else {
                    self.vhosts.selected.push(idx);
                }
                Task::none()
            }
            VHostsMessage::SelectAll => {
                self.vhosts.selected = self.vhosts.vhosts.iter().map(|v| v.index).collect();
                Task::none()
            }
            VHostsMessage::ClearSelection => {
                self.vhosts.selected.clear();
                Task::none()
            }
            VHostsMessage::BulkTagChanged(v) => {
                self.vhosts.bulk_tag = v;
                Task::none()
            }
            VHostsMessage::ApplyBulkTag => {
                let tag = self.vhosts.bulk_tag.trim().to_string();
                if tag.is_empty() || self.vhosts.selected.is_empty() {
                    return Task::none();
                }
                if let Some(db) = &self.db {
                    for idx in &self.vhosts.selected {
                        if let Some(vh) = self.vhosts.vhosts.iter_mut().find(|v| v.index == *idx) {
                            let _ = db.set_vhost_meta(&vh.server_name, &tag, "");
                            vh.tag = tag.clone();
                        }
                    }
                }
                self.vhosts.status_msg = Some((
                    true,
                    format!("Tagged {} VirtualHost(s)", self.vhosts.selected.len()),
                ));
                self.vhosts.bulk_tag.clear();
                Task::none()
            }
            VHostsMessage::ToggleHttps(idx) => {
                self.trigger_sudo(PendingAction::VHostToggleHttps { index: idx })
            }
            VHostsMessage::DuplicateRequest(idx) => {
                if let Some(entry) = self.vhosts.vhosts.iter().find(|e| e.index == idx).cloned() {
                    self.vhosts.form.open_add();
                    self.vhosts.form.server_name = format!("copy-of-{}", entry.server_name);
                    self.vhosts.form.document_root = entry.document_root;
                    self.vhosts.form.php_version = entry.php_version;
                    self.vhosts.form.https_enabled = entry.https_enabled;
                }
                Task::none()
            }

            VHostsMessage::OpenConfigEditor => {
                self.vhosts.view_mode = crate::tabs::vhosts::VHostView::ConfigEditor;
                self.vhosts.config_loading = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::tabs::vhosts::load_config_file(conf), |text| {
                    Message::VHosts(VHostsMessage::ConfigLoaded(text))
                })
            }

            VHostsMessage::CloseConfigEditor => {
                self.vhosts.view_mode = crate::tabs::vhosts::VHostView::List;
                Task::none()
            }

            VHostsMessage::ConfigLoaded(text) => {
                self.vhosts.load_config_text(text);
                Task::none()
            }

            VHostsMessage::ConfigEditorAction(action) => {
                let is_edit = action.is_edit();
                self.vhosts.config_content.perform(action);
                if is_edit {
                    self.vhosts.config_dirty = true;
                }
                Task::none()
            }

            VHostsMessage::SaveConfigFile => {
                self.vhosts.config_loading = true;
                let content = self.vhosts.config_content.text();
                let conf = self.vhosts.devpanel_conf.clone();
                self.trigger_sudo(PendingAction::SaveConfig {
                    content,
                    path: conf,
                })
            }

            VHostsMessage::SaveConfigDone(ok, msg) => {
                self.vhosts.config_loading = false;
                if ok {
                    self.vhosts.config_dirty = false;
                }
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                if ok {
                    Task::batch([
                        self.show_toast(msg, ok),
                        Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                            Message::VHosts(VHostsMessage::ScanDone(v))
                        }),
                    ])
                } else {
                    self.show_toast(msg, ok)
                }
            }
        }
    }
}

impl App {
    fn handle_config(&mut self, msg: ConfigMessage) -> Task<Message> {
        match msg {
            ConfigMessage::SetSection(s) => {
                self.config_tab.active_section = s;
                Task::none()
            }

            ConfigMessage::Save => {
                self.config_tab.saving = true;
                let settings = self.config_tab.settings.clone();
                Task::perform(
                    async move {
                        match DevPanelDb::open() {
                            Ok(db) => match settings.save(&db) {
                                Ok(_) => (true, "Settings saved".to_string()),
                                Err(e) => (false, format!("Save failed: {}", e)),
                            },
                            Err(e) => (false, format!("DB error: {}", e)),
                        }
                    },
                    |(ok, msg)| Message::Config(ConfigMessage::SaveDone(ok, msg)),
                )
            }

            ConfigMessage::SaveDone(ok, msg) => {
                self.config_tab.apply_save_result(ok, msg.clone());
                if ok && let Ok(db) = DevPanelDb::open() {
                    let loaded = UserSettings::load(&db);
                    self.config_tab.settings = loaded;
                    self.db = Some(db);
                }
                self.show_toast(msg, ok)
            }

            ConfigMessage::ApacheLogLevelChanged(v) => {
                self.config_tab.settings.apache_log_level = v;
                Task::none()
            }
            ConfigMessage::ApacheAutoReloadChanged(v) => {
                self.config_tab.settings.apache_auto_reload = v;
                Task::none()
            }
            ConfigMessage::PhpDefaultVersionChanged(v) => {
                self.config_tab.settings.php_default_version = v;
                Task::none()
            }
            ConfigMessage::PhpDisplayErrorsChanged(v) => {
                self.config_tab.settings.php_display_errors = v;
                Task::none()
            }
            ConfigMessage::ProjectsOpenCommandChanged(v) => {
                self.config_tab.settings.projects_open_command = v;
                Task::none()
            }
            ConfigMessage::UiConfirmDeletesChanged(v) => {
                self.config_tab.settings.ui_confirm_deletes = v;
                Task::none()
            }
            ConfigMessage::UiToastDurationChanged(v) => {
                self.config_tab.settings.ui_toast_duration_ms = v;
                Task::none()
            }
            ConfigMessage::UiShowSetupLogChanged(v) => {
                self.config_tab.settings.ui_show_setup_log = v;
                Task::none()
            }
            ConfigMessage::SshDefaultKeyTypeChanged(v) => {
                self.config_tab.settings.ssh_default_key_type = v;
                Task::none()
            }
            ConfigMessage::EditorCommandChanged(v) => {
                self.config_tab.settings.editor_command = v;
                Task::none()
            }
        }
    }
}

impl App {
    fn handle_first_run(&mut self, msg: FirstRunMessage) -> Task<Message> {
        match msg {
            FirstRunMessage::Continue => {
                self.first_run_installing = true;
                self.trigger_sudo(PendingAction::FirstRunInstall {
                    options: self.first_run_options,
                })
            }
            FirstRunMessage::Exit => {
                std::process::exit(0);
            }
            FirstRunMessage::ToggleMysql(v) => {
                self.first_run_options.install_mysql = v;
                Task::none()
            }
            FirstRunMessage::TogglePhpExtras(v) => {
                self.first_run_options.install_php_extras = v;
                Task::none()
            }
            FirstRunMessage::ProgressTick => {
                if self.first_run_installing {
                    Task::perform(
                        async {
                            crate::core::setup_log::read_setup_log_async()
                                .await
                                .into_iter()
                                .map(|entry| format!("[{:?}] {}", entry.level, entry.message))
                                .collect()
                        },
                        |lines| Message::FirstRun(FirstRunMessage::LogLoaded(lines)),
                    )
                } else {
                    Task::none()
                }
            }
            FirstRunMessage::LogLoaded(lines) => {
                self.first_run_log_lines = lines;
                Task::none()
            }
            FirstRunMessage::InstallDone(ok, msg) => {
                self.first_run_installing = false;
                if ok {
                    first_run::mark_done();
                    self.first_run_state = first_run::FirstRunState::Hidden;
                }
                self.tools.scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
                    Task::perform(
                        crate::tabs::tools::scan_php_versions(
                            self.dashboard.active_php_version.clone(),
                        ),
                        |r| Message::Tools(ToolsMessage::ScanDone(r)),
                    ),
                ])
            }
        }
    }
}
