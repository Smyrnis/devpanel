use iced::Task;

use crate::app::App;
use crate::infra::sudo_prompt::{ComposerCommand, RedisServiceCommand, boxed};
use crate::infra::system::copy_to_clipboard;
use crate::messages::{Message, ToolsMessage};

impl App {
    pub(super) fn handle_tools_installed_tools_scanned(
        &mut self,
        tools: crate::domain::tools::InstalledTools,
    ) -> Task<Message> {
        self.tools.apply_tools_scan(tools);
        Task::none()
    }

    pub(super) fn handle_tools_composer(&mut self, update: bool) -> Task<Message> {
        let action = if update { "update" } else { "install" };
        self.tools
            .push_log(true, format!("Queued Composer {}", action));
        self.trigger_sudo(boxed(ComposerCommand { update }))
    }

    pub(super) fn handle_tools_composer_done(&mut self, ok: bool, msg: String) -> Task<Message> {
        self.tools.push_log(ok, msg.clone());
        self.tools.tools_scanning = true;
        Task::batch([self.show_toast(msg, ok), Self::scan_installed_tools_task()])
    }

    pub(super) fn handle_tools_copy_nvm_install_command(&mut self) -> Task<Message> {
        let command =
            "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash"
                .to_string();
        Task::perform(copy_to_clipboard(command), |_| {
            Message::Tools(ToolsMessage::CopyDone)
        })
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
