use iced::Task;

use crate::app::App;

use crate::core::first_run;

use crate::core::sudo_prompt::{FirstRunInstallCommand, boxed};

use crate::messages::{FirstRunMessage, Message, ToolsMessage};

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
