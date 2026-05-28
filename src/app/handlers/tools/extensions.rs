use iced::Task;

use crate::app::App;
use crate::infra::sudo_prompt::{AptPackageCommand, boxed};
use crate::messages::Message;

impl App {
    pub(super) fn handle_tools_scan_php_exts(&mut self) -> Task<Message> {
        let active = self.active_php_for_extensions();
        Self::scan_php_extensions_task(active)
    }

    pub(super) fn handle_tools_scan_php_exts_done(
        &mut self,
        results: Vec<(String, bool)>,
    ) -> Task<Message> {
        self.tools.apply_ext_scan(results);
        Task::none()
    }

    pub(super) fn handle_tools_toggle_php_ext(
        &mut self,
        package: String,
        install: bool,
    ) -> Task<Message> {
        let action = if install { "Installing" } else { "Removing" };
        self.tools
            .install_log
            .push((true, format!("{} {}...", action, package)));
        self.trigger_sudo(boxed(AptPackageCommand { package, install }))
    }

    pub(super) fn handle_tools_php_ext_done(&mut self, ok: bool, msg: String) -> Task<Message> {
        self.tools.push_log(ok, msg.clone());
        let active = self
            .tools
            .php_releases
            .iter()
            .find(|r| r.is_active)
            .map(|r| r.version.clone());
        Task::batch([
            self.show_toast(msg, ok),
            Self::scan_php_extensions_task(active),
        ])
    }
}
