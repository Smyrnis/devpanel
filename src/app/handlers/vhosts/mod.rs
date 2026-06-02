mod config;
mod form;
mod scan;
mod selection;

use iced::Task;

use crate::app::App;
use crate::infra::system::{open_url, xdg_open};
use crate::messages::{Message, VHostsMessage};

impl App {
    pub(crate) fn handle_vhosts(&mut self, msg: VHostsMessage) -> Task<Message> {
        match msg {
            VHostsMessage::Scan => self.handle_vhosts_scan(),
            VHostsMessage::ScanDone(vhosts) => self.handle_vhosts_scan_done(vhosts),

            VHostsMessage::ShowAddForm => self.handle_vhosts_show_add_form(),
            VHostsMessage::HideForm => self.handle_vhosts_hide_form(),
            VHostsMessage::FormServerNameChanged(v) => {
                self.vhosts.form.server_name = v;
                Task::none()
            }
            VHostsMessage::FormDocRootChanged(v) => {
                self.vhosts.form.document_root = v;
                Task::none()
            }
            VHostsMessage::FormPhpVersionChanged(v) => {
                self.vhosts.form.php_version = v;
                Task::none()
            }
            VHostsMessage::FormHttpsChanged(v) => {
                self.vhosts.form.https_enabled = v;
                Task::none()
            }
            VHostsMessage::Create => self.handle_vhosts_create(),
            VHostsMessage::CreateDone(ok, msg) => {
                self.vhosts.form.hide();
                self.reload_vhosts_with_toast(ok, msg)
            }
            VHostsMessage::EditRequest(idx) => self.handle_vhosts_edit_request(idx),
            VHostsMessage::SaveEdit => self.handle_vhosts_save_edit(),
            VHostsMessage::SaveEditDone(ok, msg) => {
                self.vhosts.form.hide();
                self.reload_vhosts_with_toast(ok, msg)
            }
            VHostsMessage::DuplicateRequest(idx) => self.handle_vhosts_duplicate_request(idx),

            VHostsMessage::OpenBrowser(sn) => {
                let _ = open_url(&format!("http://{}", sn));
                Task::none()
            }
            VHostsMessage::OpenDocumentRoot(path) => {
                let _ = xdg_open(&path);
                Task::none()
            }
            VHostsMessage::OpenDevpanelConf => {
                let _ = xdg_open(&self.vhosts.devpanel_conf);
                Task::none()
            }
            VHostsMessage::ToggleExpanded(idx) => {
                self.vhosts.expanded_vhost = if self.vhosts.expanded_vhost == Some(idx) {
                    None
                } else {
                    Some(idx)
                };
                Task::none()
            }

            VHostsMessage::DeleteRequest(idx) => self.handle_vhosts_delete_request(idx),
            VHostsMessage::DeleteCancel => {
                self.vhosts.confirm_delete = None;
                Task::none()
            }
            VHostsMessage::DeleteConfirm(idx) => self.handle_vhosts_delete_confirm(idx),
            VHostsMessage::BulkDeleteConfirm => self.handle_vhosts_bulk_delete_confirm(),
            VHostsMessage::DeleteDone(ok, msg) => self.handle_vhosts_delete_done(ok, msg),
            VHostsMessage::SearchChanged(v) => {
                self.vhosts.search_query = v;
                Task::none()
            }
            VHostsMessage::ToggleSelected(idx) => self.handle_vhosts_toggle_selected(idx),
            VHostsMessage::SelectAll => {
                self.vhosts.selected = self.vhosts.vhosts.iter().map(|v| v.index).collect();
                Task::none()
            }
            VHostsMessage::ClearSelection => {
                self.vhosts.selected.clear();
                Task::none()
            }
            VHostsMessage::BulkTagChanged(v) => {
                self.vhosts.bulk_tag = v;
                Task::none()
            }
            VHostsMessage::ApplyBulkTag => self.handle_vhosts_apply_bulk_tag(),
            VHostsMessage::ToggleHttps(idx) => self.handle_vhosts_toggle_https(idx),

            VHostsMessage::OpenConfigEditor => self.handle_vhosts_open_config_editor(),
            VHostsMessage::CloseConfigEditor => self.handle_vhosts_close_config_editor(),
            VHostsMessage::ConfigLoaded(text) => self.handle_vhosts_config_loaded(text),
            VHostsMessage::ConfigFileChanged => self.handle_vhosts_config_file_changed(),
            VHostsMessage::ConfigEditorAction(action) => {
                self.handle_vhosts_config_editor_action(action)
            }
            VHostsMessage::SaveConfigFile => self.handle_vhosts_save_config_file(),
            VHostsMessage::SaveConfigDone(ok, msg) => self.handle_vhosts_save_config_done(ok, msg),
        }
    }
}
