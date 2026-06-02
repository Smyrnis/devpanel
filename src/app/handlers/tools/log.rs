use iced::Task;

use crate::app::App;
use crate::infra::system::copy_to_clipboard;
use crate::messages::{Message, ToolsMessage};

impl App {
    pub(super) fn handle_tools_clear_log(&mut self) -> Task<Message> {
        self.tools.install_log.clear();
        self.tools.db_status.clear();
        self.tools.last_php_error = None;
        self.tools.log_expanded = false;
        Task::none()
    }

    pub(super) fn handle_tools_toggle_log(&mut self) -> Task<Message> {
        self.tools.log_expanded = !self.tools.log_expanded;
        Task::none()
    }

    pub(super) fn handle_tools_copy_fix_commands(&mut self, commands: String) -> Task<Message> {
        Task::perform(copy_to_clipboard(commands), |_| {
            Message::Tools(ToolsMessage::CopyDone)
        })
    }

    pub(super) fn handle_tools_copy_done(&mut self) -> Task<Message> {
        self.show_toast("Commands copied to clipboard!".into(), true)
    }
}
