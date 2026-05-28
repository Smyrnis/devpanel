use iced::Task;

use crate::app::App;
use crate::infra::sudo_prompt::{VHostAddCommand, VHostEditCommand, boxed};
use crate::messages::Message;

impl App {
    pub(super) fn handle_vhosts_show_add_form(&mut self) -> Task<Message> {
        self.vhosts.form.open_add();
        Task::none()
    }

    pub(super) fn handle_vhosts_hide_form(&mut self) -> Task<Message> {
        self.vhosts.form.hide();
        Task::none()
    }

    pub(super) fn handle_vhosts_create(&mut self) -> Task<Message> {
        let sn = self.vhosts.form.server_name.trim().to_string();
        let dr = self.vhosts.form.document_root.trim().to_string();
        let php = self.vhosts.form.php_version.clone();
        let https_enabled = self.vhosts.form.https_enabled;

        self.trigger_sudo(boxed(VHostAddCommand {
            devpanel_conf: self.vhosts.devpanel_conf.clone(),
            server_name: sn,
            document_root: dr,
            php_version: php,
            https_enabled,
        }))
    }

    pub(super) fn handle_vhosts_edit_request(&mut self, idx: usize) -> Task<Message> {
        if let Some(entry) = self.vhosts.vhosts.iter().find(|e| e.index == idx).cloned() {
            self.vhosts.form.open_edit(&entry);
        }
        Task::none()
    }

    pub(super) fn handle_vhosts_save_edit(&mut self) -> Task<Message> {
        let sn = self.vhosts.form.server_name.trim().to_string();
        let dr = self.vhosts.form.document_root.trim().to_string();
        let php = self.vhosts.form.php_version.clone();
        let https_enabled = self.vhosts.form.https_enabled;
        let idx = match self.vhosts.form.mode {
            crate::domain::vhosts::FormMode::Edit(i) => i,
            _ => return Task::none(),
        };

        self.trigger_sudo(boxed(VHostEditCommand {
            devpanel_conf: self.vhosts.devpanel_conf.clone(),
            index: idx,
            server_name: sn,
            document_root: dr,
            php_version: php,
            https_enabled,
        }))
    }

    pub(super) fn handle_vhosts_duplicate_request(&mut self, idx: usize) -> Task<Message> {
        if let Some(entry) = self.vhosts.vhosts.iter().find(|e| e.index == idx).cloned() {
            self.vhosts.form.open_add();
            self.vhosts.form.server_name = format!("copy-of-{}", entry.server_name);
            self.vhosts.form.document_root = entry.document_root;
            self.vhosts.form.php_version = entry.php_version;
            self.vhosts.form.https_enabled = entry.https_enabled;
        }
        Task::none()
    }
}
