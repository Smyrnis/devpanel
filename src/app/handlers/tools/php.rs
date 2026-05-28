use iced::Task;

use crate::app::App;
use crate::infra::sudo_prompt::{PhpInstallCommand, boxed};
use crate::messages::{DashboardMessage, Message, ToolsMessage};

impl App {
    pub(super) fn scan_php_versions_task(active_php_version: Option<String>) -> Task<Message> {
        Task::perform(
            crate::ui::tabs::tools::scan_php_versions(active_php_version),
            |r| Message::Tools(ToolsMessage::ScanDone(r)),
        )
    }

    pub(super) fn handle_tools_scan_php(&mut self) -> Task<Message> {
        self.tools.scanning = true;
        Self::scan_php_versions_task(self.dashboard.active_php_version.clone())
    }

    pub(super) fn handle_tools_scan_done(
        &mut self,
        results: Vec<(String, crate::domain::tools::PhpStatus, bool, bool, bool)>,
    ) -> Task<Message> {
        self.tools.apply_scan(results);
        self.sync_php_versions_to_vhosts();
        Task::none()
    }

    pub(super) fn handle_tools_install_php(
        &mut self,
        version: String,
        install: bool,
    ) -> Task<Message> {
        let label = if install { "install" } else { "removal" };
        self.tools
            .push_log(true, format!("Queued {}: PHP {}", label, version));
        self.trigger_sudo(boxed(PhpInstallCommand { version, install }))
    }

    pub(super) fn handle_tools_php_op_done(&mut self, ok: bool, msg: String) -> Task<Message> {
        self.tools.push_log(ok, msg.clone());
        self.tools.scanning = true;
        Task::batch([
            self.show_toast(msg, ok),
            Task::perform(
                crate::domain::dashboard::service::probe_services(),
                |snapshot| Message::Dashboard(DashboardMessage::StatusRefreshed(snapshot)),
            ),
            Self::scan_php_versions_task(self.dashboard.active_php_version.clone()),
        ])
    }
}
