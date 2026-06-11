use iced::Task;

use crate::app::App;

use crate::core::first_run;

use crate::infra::sudo_prompt::{FirstRunInstallCommand, boxed};

use crate::messages::{FirstRunMessage, Message, ToolsMessage};

async fn load_setup_log_lines() -> Vec<String> {
    crate::core::setup_log::read_setup_log_async()
        .await
        .into_iter()
        .map(|entry| format!("[{:?}] {}", entry.level, entry.message))
        .collect()
}

impl App {
    pub(crate) fn handle_first_run(&mut self, msg: FirstRunMessage) -> Task<Message> {
        match msg {
            FirstRunMessage::Continue => {
                self.first_run_installing = true;
                self.trigger_sudo(boxed(FirstRunInstallCommand {
                    options: self.first_run_options,
                }))
            }
            FirstRunMessage::Exit => {
                std::process::exit(0);
            }
            FirstRunMessage::TogglePackage(package) => {
                self.first_run_expanded = if self.first_run_expanded == Some(package) {
                    None
                } else {
                    Some(package)
                };
                Task::none()
            }
            FirstRunMessage::ToggleApache(v) => {
                self.first_run_options.install_apache = v;
                Task::none()
            }
            FirstRunMessage::TogglePhp(v) => {
                self.first_run_options.install_php = v;
                Task::none()
            }
            FirstRunMessage::ToggleMysql(v) => {
                self.first_run_options.install_mysql = v;
                Task::none()
            }
            FirstRunMessage::TogglePhpExtras(v) => {
                self.first_run_options.install_php_extras = v;
                Task::none()
            }
            FirstRunMessage::ToggleComposer(v) => {
                self.first_run_options.install_composer = v;
                Task::none()
            }
            FirstRunMessage::ToggleNodeNvm(v) => {
                self.first_run_options.install_node_nvm = v;
                Task::none()
            }
            FirstRunMessage::ScanStatus => Task::perform(
                crate::installer::service::scan_first_run_status(),
                |status| Message::FirstRun(FirstRunMessage::StatusScanned(status)),
            ),
            FirstRunMessage::StatusScanned(status) => {
                self.first_run_status = status;
                Task::none()
            }
            FirstRunMessage::ProgressTick => {
                if self.first_run_installing {
                    Task::perform(load_setup_log_lines(), |lines| {
                        Message::FirstRun(FirstRunMessage::LogLoaded(lines))
                    })
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
                    self.tools.scanning = true;
                    self.dashboard.runtimes_scanning = true;
                    return Task::batch([
                        self.show_toast(msg, ok),
                        Task::perform(
                            crate::installer::service::scan_first_run_status(),
                            |status| Message::FirstRun(FirstRunMessage::StatusScanned(status)),
                        ),
                        Task::perform(
                            crate::domain::dashboard::service::probe_services(),
                            |snapshot| {
                                Message::Dashboard(
                                    crate::messages::DashboardMessage::StatusRefreshed(snapshot),
                                )
                            },
                        ),
                        Task::perform(
                            crate::ui::tabs::tools::scan_php_versions(
                                self.dashboard.active_php_version.clone(),
                            ),
                            |r| Message::Tools(ToolsMessage::ScanDone(r)),
                        ),
                        Task::perform(crate::domain::tools::service::scan_installed_tools(), |r| {
                            Message::Dashboard(
                                crate::messages::DashboardMessage::RuntimesRefreshed(r),
                            )
                        }),
                    ]);
                }
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(
                        crate::installer::service::scan_first_run_status(),
                        |status| Message::FirstRun(FirstRunMessage::StatusScanned(status)),
                    ),
                    Task::perform(
                        crate::domain::dashboard::service::probe_services(),
                        |snapshot| {
                            Message::Dashboard(crate::messages::DashboardMessage::StatusRefreshed(
                                snapshot,
                            ))
                        },
                    ),
                    Task::perform(load_setup_log_lines(), |lines| {
                        Message::FirstRun(FirstRunMessage::LogLoaded(lines))
                    }),
                ])
            }
        }
    }
}
