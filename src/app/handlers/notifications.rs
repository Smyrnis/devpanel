use iced::Task;

use crate::app::App;
use crate::messages::Message;

impl App {
    pub(crate) fn handle_notification_tick(&mut self) -> Task<Message> {
        for notification in &mut self.notifications {
            notification.remaining_ms = notification.remaining_ms.saturating_sub(1000);
        }
        self.notifications.retain(|n| n.remaining_ms > 0);
        Task::none()
    }

    pub(crate) fn handle_dismiss_all_notifications(&mut self) -> Task<Message> {
        self.notifications.clear();
        Task::none()
    }
}
