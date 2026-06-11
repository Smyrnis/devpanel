use iced::Task;

use crate::app::App;
use crate::infra::sudo_prompt::{RedisServiceCommand, boxed};
use crate::messages::Message;

impl App {
    pub(super) fn handle_tools_installed_tools_scanned(
        &mut self,
        tools: crate::domain::tools::InstalledTools,
    ) -> Task<Message> {
        self.tools.apply_tools_scan(tools);
        Task::none()
    }

    pub(super) fn handle_tools_redis(&mut self, action: &'static str) -> Task<Message> {
        let label = match action {
            "start" => "Starting Redis",
            "stop" => "Stopping Redis",
            _ => "Managing Redis",
        };
        self.tools.push_log(true, label.into());
        self.trigger_sudo(boxed(RedisServiceCommand {
            action: action.into(),
        }))
    }

    pub(super) fn handle_tools_redis_done(&mut self, ok: bool, msg: String) -> Task<Message> {
        self.tools.push_log(ok, msg.clone());
        self.tools.tools_scanning = true;
        Task::batch([self.show_toast(msg, ok), Self::scan_installed_tools_task()])
    }
}
