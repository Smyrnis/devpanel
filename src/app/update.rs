use iced::Task;

use crate::app::App;
use crate::messages::Message;

impl App {
    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SelectTab(tab) => self.handle_select_tab(tab),
            Message::WindowResized(size) => {
                self.window_size = size;
                Task::none()
            }
            Message::NotificationTick => self.handle_notification_tick(),
            Message::DismissAllNotifications => self.handle_dismiss_all_notifications(),
            Message::Dashboard(m) => self.handle_dashboard(m),
            Message::SshKeys(m) => self.handle_ssh_keys(m),
            Message::Tools(m) => self.handle_tools(m),
            Message::VHosts(m) => self.handle_vhosts(m),
            Message::Sudo(m) => self.handle_sudo(m),
            Message::FirstRun(m) => self.handle_first_run(m),
            Message::Config(m) => self.handle_config(m),
        }
    }
}
