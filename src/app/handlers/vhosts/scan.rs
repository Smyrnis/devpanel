use iced::Task;

use crate::app::App;
use crate::domain::vhosts::VHostEntry;
use crate::messages::{Message, VHostsMessage};

impl App {
    pub(super) fn scan_vhosts_task(conf: String) -> Task<Message> {
        Task::perform(crate::ui::tabs::vhosts::scan_vhosts(conf), |v| {
            Message::VHosts(VHostsMessage::ScanDone(v))
        })
    }

    pub(super) fn handle_vhosts_scan(&mut self) -> Task<Message> {
        self.vhosts.scanning = true;
        Self::scan_vhosts_task(self.vhosts.devpanel_conf.clone())
    }

    pub(super) fn handle_vhosts_scan_done(&mut self, vhosts: Vec<VHostEntry>) -> Task<Message> {
        self.vhosts.set_vhosts(vhosts);
        Task::none()
    }

    pub(super) fn reload_vhosts_with_toast(&mut self, ok: bool, msg: String) -> Task<Message> {
        self.vhosts.status_msg = Some((ok, msg.clone()));
        Task::batch([
            self.show_toast(msg, ok),
            Self::scan_vhosts_task(self.vhosts.devpanel_conf.clone()),
        ])
    }

    pub(super) fn handle_vhosts_config_file_changed(&mut self) -> Task<Message> {
        if self.vhosts.config_dirty {
            return Task::none();
        }

        self.vhosts.scanning = true;
        let conf = self.vhosts.devpanel_conf.clone();
        let reload_editor = matches!(
            self.vhosts.view_mode,
            crate::ui::tabs::vhosts::VHostView::ConfigEditor
        );
        let scan = Self::scan_vhosts_task(conf.clone());

        if reload_editor {
            Task::batch([
                scan,
                Task::perform(crate::ui::tabs::vhosts::load_config_file(conf), |text| {
                    Message::VHosts(VHostsMessage::ConfigLoaded(text))
                }),
            ])
        } else {
            scan
        }
    }
}
