use iced::Task;

use crate::app::App;

use crate::core::setup_log;

use crate::core::paths;

use crate::core::sudo_prompt::{PhpSwitchCommand, RestartAllCommand, ServiceControlCommand, boxed};

use crate::core::system::{open_php_ini, open_url, xdg_open};

use crate::messages::{DashboardMessage, Message, Tab};

impl App {
    pub(crate) fn handle_dashboard(&mut self, msg: DashboardMessage) -> Task<Message> {
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
            DashboardMessage::RestartAll => self.trigger_sudo(boxed(RestartAllCommand)),

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

            DashboardMessage::SwitchPhpVersion(v) => {
                self.trigger_sudo(boxed(PhpSwitchCommand { version: v }))
            }

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
