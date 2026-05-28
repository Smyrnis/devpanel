use iced::Task;

use crate::app::App;
use crate::infra::sudo_prompt::{ApacheModToggleCommand, boxed};
use crate::messages::Message;

impl App {
    pub(super) fn handle_tools_scan_apache_mods(&mut self) -> Task<Message> {
        self.tools.mods_scanning = true;
        Self::scan_apache_modules_task()
    }

    pub(super) fn handle_tools_scan_apache_mods_done(
        &mut self,
        results: Vec<crate::domain::tools::ApacheModule>,
    ) -> Task<Message> {
        self.tools.apply_mod_scan(results);
        self.sync_php_versions_to_vhosts();
        Task::none()
    }

    pub(super) fn handle_tools_toggle_apache_mod(
        &mut self,
        name: String,
        enable: bool,
    ) -> Task<Message> {
        let action = if enable { "Enabling" } else { "Disabling" };
        self.tools
            .install_log
            .push((true, format!("{} mod_{}...", action, name)));
        self.trigger_sudo(boxed(ApacheModToggleCommand { name, enable }))
    }

    pub(super) fn handle_tools_apache_mod_done(
        &mut self,
        ok: bool,
        msg: String,
        name: String,
        enabled: bool,
    ) -> Task<Message> {
        self.tools.push_log(ok, msg.clone());
        if ok {
            self.tools.set_mod_enabled(&name, enabled);
        }
        self.sync_php_versions_to_vhosts();
        Task::batch([self.show_toast(msg, ok), Self::scan_apache_modules_task()])
    }
}
