use iced::Task;

use crate::app::App;
use crate::infra::sudo_prompt::{
    VHostBulkDeleteCommand, VHostDeleteCommand, VHostToggleHttpsCommand, boxed,
};
use crate::messages::Message;

impl App {
    pub(super) fn handle_vhosts_delete_request(&mut self, idx: usize) -> Task<Message> {
        self.vhosts.confirm_delete = Some(idx);
        self.vhosts.expanded_vhost = Some(idx);
        Task::none()
    }

    pub(super) fn handle_vhosts_delete_confirm(&mut self, idx: usize) -> Task<Message> {
        self.vhosts.confirm_delete = None;
        self.trigger_sudo(boxed(VHostDeleteCommand {
            devpanel_conf: self.vhosts.devpanel_conf.clone(),
            index: idx,
        }))
    }

    pub(super) fn handle_vhosts_bulk_delete_confirm(&mut self) -> Task<Message> {
        let indexes = self.vhosts.selected.clone();
        if indexes.is_empty() {
            return Task::none();
        }

        self.vhosts.confirm_delete = None;
        self.trigger_sudo(boxed(VHostBulkDeleteCommand {
            devpanel_conf: self.vhosts.devpanel_conf.clone(),
            indexes,
        }))
    }

    pub(super) fn handle_vhosts_delete_done(&mut self, ok: bool, msg: String) -> Task<Message> {
        if ok {
            self.vhosts.selected.clear();
        }
        self.reload_vhosts_with_toast(ok, msg)
    }

    pub(super) fn handle_vhosts_toggle_selected(&mut self, idx: usize) -> Task<Message> {
        if self.vhosts.selected.contains(&idx) {
            self.vhosts.selected.retain(|i| *i != idx);
        } else {
            self.vhosts.selected.push(idx);
        }
        Task::none()
    }

    pub(super) fn handle_vhosts_apply_bulk_tag(&mut self) -> Task<Message> {
        let tag = self.vhosts.bulk_tag.trim().to_string();
        if tag.is_empty() || self.vhosts.selected.is_empty() {
            return Task::none();
        }

        if let Some(db) = &self.db {
            for idx in &self.vhosts.selected {
                if let Some(vh) = self.vhosts.vhosts.iter_mut().find(|v| v.index == *idx) {
                    let _ = db.set_vhost_meta(&vh.server_name, &tag, "");
                    vh.tag = tag.clone();
                }
            }
        }

        self.vhosts.status_msg = Some((
            true,
            format!("Tagged {} VirtualHost(s)", self.vhosts.selected.len()),
        ));
        self.vhosts.bulk_tag.clear();
        Task::none()
    }

    pub(super) fn handle_vhosts_toggle_https(&mut self, idx: usize) -> Task<Message> {
        self.trigger_sudo(boxed(VHostToggleHttpsCommand {
            devpanel_conf: self.vhosts.devpanel_conf.clone(),
            index: idx,
        }))
    }
}
