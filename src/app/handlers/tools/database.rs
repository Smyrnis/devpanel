use iced::Task;

use crate::app::App;
use crate::infra::system::open_db_terminal;
use crate::messages::Message;

impl App {
    pub(super) fn handle_tools_open_db_terminal(
        &mut self,
        command: &'static str,
        socket_auth: bool,
    ) -> Task<Message> {
        self.tools.db_status = match open_db_terminal(command, socket_auth) {
            Ok(s) => format!("Launched: {}", s),
            Err(e) => format!("Error: {}", e),
        };
        Task::none()
    }
}
