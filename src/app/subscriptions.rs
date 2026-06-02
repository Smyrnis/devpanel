use crate::app::App;
use crate::messages::Message;

impl App {
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            iced::time::every(std::time::Duration::from_secs(5))
                .map(|_| Message::Dashboard(crate::messages::DashboardMessage::AutoRefreshTick)),
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::NotificationTick),
            iced::time::every(std::time::Duration::from_secs(1))
                .map(|_| Message::FirstRun(crate::messages::FirstRunMessage::ProgressTick)),
            crate::infra::file_watcher::vhost_config(self.vhosts.devpanel_conf.clone()),
            iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
        ])
    }
}
