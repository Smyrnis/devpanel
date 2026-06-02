use iced::Task;

use crate::app::App;

use crate::core::setup_log;

use crate::core::paths;

use crate::infra::sudo_prompt::{PhpSwitchCommand, ServiceControlCommand, boxed};

use crate::infra::system::{open_db_terminal, open_php_ini, open_url, xdg_open};

use crate::domain::tools::ToolSection;
use crate::messages::{DashboardMessage, Message, Tab, ToolsMessage};

fn probe_dashboard_task() -> Task<Message> {
    Task::perform(
        crate::domain::dashboard::service::probe_services(),
        |snapshot| Message::Dashboard(DashboardMessage::StatusRefreshed(snapshot)),
    )
}

impl App {
    pub(crate) fn handle_dashboard(&mut self, msg: DashboardMessage) -> Task<Message> {
        match msg {
            DashboardMessage::StatusRefreshed(snapshot) => {
                self.dashboard.update_status(
                    snapshot.apache,
                    snapshot.mysql,
                    snapshot.php,
                    snapshot.apache_uptime,
                    snapshot.mysql_uptime,
                    snapshot.recent_failures,
                );
                self.dashboard.set_php_versions(snapshot.php_versions);
                if !self.setup_issues_checked {
                    self.setup_issues_checked = true;
                    let show = self.config_tab.settings.ui_show_setup_log;
                    if show {
                        let issues = setup_log::read_setup_issues();
                        if !issues.is_empty() {
                            let summary = format!(
                                "{} post-install issue(s) - check {}",
                                issues.len(),
                                paths::SETUP_LOG,
                            );
                            return self.show_toast(summary, false);
                        }
                    }
                }
                Task::none()
            }

            DashboardMessage::ToggleService(service) => {
                self.dashboard.expanded_service =
                    if self.dashboard.expanded_service == Some(service) {
                        None
                    } else {
                        Some(service)
                    };
                Task::none()
            }

            DashboardMessage::AutoRefreshTick => {
                if self.active_tab == Tab::Dashboard {
                    probe_dashboard_task()
                } else {
                    Task::none()
                }
            }

            DashboardMessage::StartApache => self.trigger_sudo(boxed(ServiceControlCommand {
                service: "apache2".into(),
                action: "start".into(),
            })),
            DashboardMessage::StopApache => self.trigger_sudo(boxed(ServiceControlCommand {
                service: "apache2".into(),
                action: "stop".into(),
            })),
            DashboardMessage::RestartApache => self.trigger_sudo(boxed(ServiceControlCommand {
                service: "apache2".into(),
                action: "restart".into(),
            })),
            DashboardMessage::StartMySQL => self.trigger_sudo(boxed(ServiceControlCommand {
                service: "mysql".into(),
                action: "start".into(),
            })),
            DashboardMessage::StopMySQL => self.trigger_sudo(boxed(ServiceControlCommand {
                service: "mysql".into(),
                action: "stop".into(),
            })),
            DashboardMessage::RestartMySQL => self.trigger_sudo(boxed(ServiceControlCommand {
                service: "mysql".into(),
                action: "restart".into(),
            })),
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
                Task::batch([self.show_toast(msg, success), probe_dashboard_task()])
            }

            DashboardMessage::SwitchPhpVersion(v) => {
                self.trigger_sudo(boxed(PhpSwitchCommand { version: v }))
            }

            DashboardMessage::PhpSwitchResult(ok, msg) => {
                Task::batch([self.show_toast(msg, ok), probe_dashboard_task()])
            }

            DashboardMessage::ShowPhpInfo => {
                self.dashboard.php_info_loading = true;
                self.dashboard.php_info = None;
                Task::perform(
                    crate::domain::dashboard::service::php_info_summary(),
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
            DashboardMessage::OpenMysqlTerminal => {
                let result = open_db_terminal("mysql", false);
                let (ok, msg) = match result {
                    Ok(s) => (true, format!("Launched MySQL terminal: {}", s)),
                    Err(e) => (false, e),
                };
                self.show_toast(msg, ok)
            }
            DashboardMessage::ManagePhpExtensions => {
                self.active_tab = Tab::Tools;
                self.tools.active_section = Some(ToolSection::PhpExts);
                Task::perform(
                    crate::ui::tabs::tools::scan_php_extensions(
                        self.dashboard.active_php_version.clone(),
                    ),
                    |r| Message::Tools(ToolsMessage::ScanPhpExtsDone(r)),
                )
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
            DashboardMessage::NavigateApache2Conf => {
                let _ = xdg_open(paths::APACHE_CONF_FILE);
                Task::none()
            }
            DashboardMessage::NavigateApache2Sites => {
                let _ = xdg_open(paths::APACHE_SITES_AVAILABLE);
                Task::none()
            }
            DashboardMessage::NavigatePhpDir => {
                let _ = xdg_open(paths::PHP_ETC_DIR);
                Task::none()
            }
            DashboardMessage::NavigateMysqlDir => {
                let _ = xdg_open(paths::MYSQL_ETC_DIR);
                Task::none()
            }
            DashboardMessage::NavigateHostsFile => {
                let _ = xdg_open(paths::HOSTS_FILE);
                Task::none()
            }
            DashboardMessage::OpenPhpIni => {
                let _ = open_php_ini(&self.dashboard.active_php_version);
                Task::none()
            }
        }
    }
}
