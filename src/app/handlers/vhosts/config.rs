use iced::Task;

use crate::app::App;
use crate::infra::sudo_prompt::{SaveConfigCommand, boxed};
use crate::messages::{Message, VHostsMessage};

impl App {
    pub(super) fn handle_vhosts_open_config_editor(&mut self) -> Task<Message> {
        self.vhosts.view_mode = crate::ui::tabs::vhosts::VHostView::ConfigEditor;
        self.vhosts.config_loading = true;
        let conf = self.vhosts.devpanel_conf.clone();
        Task::perform(crate::ui::tabs::vhosts::load_config_file(conf), |text| {
            Message::VHosts(VHostsMessage::ConfigLoaded(text))
        })
    }

    pub(super) fn handle_vhosts_close_config_editor(&mut self) -> Task<Message> {
        self.vhosts.view_mode = crate::ui::tabs::vhosts::VHostView::List;
        Task::none()
    }

    pub(super) fn handle_vhosts_config_loaded(&mut self, text: String) -> Task<Message> {
        self.vhosts.load_config_text(text);
        Task::none()
    }

    pub(super) fn handle_vhosts_config_editor_action(
        &mut self,
        action: iced::widget::text_editor::Action,
    ) -> Task<Message> {
        let is_edit = action.is_edit();
        self.vhosts.config_content.perform(action);
        if is_edit {
            self.vhosts.config_dirty = true;
        }
        Task::none()
    }

    pub(super) fn handle_vhosts_save_config_file(&mut self) -> Task<Message> {
        self.vhosts.config_loading = true;
        let content = self.vhosts.config_content.text();
        let conf = self.vhosts.devpanel_conf.clone();
        self.trigger_sudo(boxed(SaveConfigCommand {
            content,
            path: conf,
        }))
    }

    pub(super) fn handle_vhosts_save_config_done(
        &mut self,
        ok: bool,
        msg: String,
    ) -> Task<Message> {
        self.vhosts.config_loading = false;
        if ok {
            self.vhosts.config_dirty = false;
        }
        self.vhosts.status_msg = Some((ok, msg.clone()));

        if ok {
            Task::batch([
                self.show_toast(msg, ok),
                Self::scan_vhosts_task(self.vhosts.devpanel_conf.clone()),
            ])
        } else {
            self.show_toast(msg, ok)
        }
    }
}
