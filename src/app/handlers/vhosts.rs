use iced::Task;

use crate::app::App;

use crate::core::sudo_prompt::{
    SaveConfigCommand, VHostAddCommand, VHostBulkDeleteCommand, VHostDeleteCommand,
    VHostEditCommand, VHostToggleHttpsCommand, boxed,
};

use crate::core::system::{open_url, xdg_open};

use crate::messages::{Message, VHostsMessage};

impl App {
    pub(crate) fn handle_vhosts(&mut self, msg: VHostsMessage) -> Task<Message> {
        match msg {
            VHostsMessage::Scan => {
                self.vhosts.scanning = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                    Message::VHosts(VHostsMessage::ScanDone(v))
                })
            }

            VHostsMessage::ScanDone(vhosts) => {
                self.vhosts.set_vhosts(vhosts);
                Task::none()
            }
            VHostsMessage::ShowAddForm => {
                self.vhosts.form.open_add();
                Task::none()
            }
            VHostsMessage::HideForm => {
                self.vhosts.form.hide();
                Task::none()
            }

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

            VHostsMessage::Create => {
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

            VHostsMessage::CreateDone(ok, msg) => {
                self.vhosts.form.hide();
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                        Message::VHosts(VHostsMessage::ScanDone(v))
                    }),
                ])
            }

            VHostsMessage::EditRequest(idx) => {
                if let Some(entry) = self.vhosts.vhosts.iter().find(|e| e.index == idx).cloned() {
                    self.vhosts.form.open_edit(&entry);
                }
                Task::none()
            }

            VHostsMessage::SaveEdit => {
                let sn = self.vhosts.form.server_name.trim().to_string();
                let dr = self.vhosts.form.document_root.trim().to_string();
                let php = self.vhosts.form.php_version.clone();
                let https_enabled = self.vhosts.form.https_enabled;
                let idx = match self.vhosts.form.mode {
                    crate::tabs::vhosts::FormMode::Edit(i) => i,
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

            VHostsMessage::SaveEditDone(ok, msg) => {
                self.vhosts.form.hide();
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                        Message::VHosts(VHostsMessage::ScanDone(v))
                    }),
                ])
            }

            VHostsMessage::OpenBrowser(sn) => {
                let _ = open_url(&format!("http://{}", sn));
                Task::none()
            }
            VHostsMessage::OpenDevpanelConf => {
                let _ = xdg_open(&self.vhosts.devpanel_conf);
                Task::none()
            }

            VHostsMessage::DeleteRequest(idx) => {
                self.vhosts.confirm_delete = Some(idx);
                Task::none()
            }
            VHostsMessage::DeleteCancel => {
                self.vhosts.confirm_delete = None;
                Task::none()
            }

            VHostsMessage::DeleteConfirm(idx) => {
                self.vhosts.confirm_delete = None;
                self.trigger_sudo(boxed(VHostDeleteCommand {
                    devpanel_conf: self.vhosts.devpanel_conf.clone(),
                    index: idx,
                }))
            }

            VHostsMessage::BulkDeleteConfirm => {
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

            VHostsMessage::DeleteDone(ok, msg) => {
                if ok {
                    self.vhosts.selected.clear();
                }
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                        Message::VHosts(VHostsMessage::ScanDone(v))
                    }),
                ])
            }

            VHostsMessage::ToggleSelected(idx) => {
                if self.vhosts.selected.contains(&idx) {
                    self.vhosts.selected.retain(|i| *i != idx);
                } else {
                    self.vhosts.selected.push(idx);
                }
                Task::none()
            }
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
            VHostsMessage::ApplyBulkTag => {
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
            VHostsMessage::ToggleHttps(idx) => self.trigger_sudo(boxed(VHostToggleHttpsCommand {
                devpanel_conf: self.vhosts.devpanel_conf.clone(),
                index: idx,
            })),
            VHostsMessage::DuplicateRequest(idx) => {
                if let Some(entry) = self.vhosts.vhosts.iter().find(|e| e.index == idx).cloned() {
                    self.vhosts.form.open_add();
                    self.vhosts.form.server_name = format!("copy-of-{}", entry.server_name);
                    self.vhosts.form.document_root = entry.document_root;
                    self.vhosts.form.php_version = entry.php_version;
                    self.vhosts.form.https_enabled = entry.https_enabled;
                }
                Task::none()
            }

            VHostsMessage::OpenConfigEditor => {
                self.vhosts.view_mode = crate::tabs::vhosts::VHostView::ConfigEditor;
                self.vhosts.config_loading = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::tabs::vhosts::load_config_file(conf), |text| {
                    Message::VHosts(VHostsMessage::ConfigLoaded(text))
                })
            }

            VHostsMessage::CloseConfigEditor => {
                self.vhosts.view_mode = crate::tabs::vhosts::VHostView::List;
                Task::none()
            }

            VHostsMessage::ConfigLoaded(text) => {
                self.vhosts.load_config_text(text);
                Task::none()
            }

            VHostsMessage::ConfigFileChanged => {
                if self.vhosts.config_dirty {
                    return Task::none();
                }
                self.vhosts.scanning = true;
                let conf = self.vhosts.devpanel_conf.clone();
                let reload_editor = matches!(
                    self.vhosts.view_mode,
                    crate::tabs::vhosts::VHostView::ConfigEditor
                );
                let scan = Task::perform(crate::tabs::vhosts::scan_vhosts(conf.clone()), |v| {
                    Message::VHosts(VHostsMessage::ScanDone(v))
                });
                if reload_editor {
                    Task::batch([
                        scan,
                        Task::perform(crate::tabs::vhosts::load_config_file(conf), |text| {
                            Message::VHosts(VHostsMessage::ConfigLoaded(text))
                        }),
                    ])
                } else {
                    scan
                }
            }

            VHostsMessage::ConfigEditorAction(action) => {
                let is_edit = action.is_edit();
                self.vhosts.config_content.perform(action);
                if is_edit {
                    self.vhosts.config_dirty = true;
                }
                Task::none()
            }

            VHostsMessage::SaveConfigFile => {
                self.vhosts.config_loading = true;
                let content = self.vhosts.config_content.text();
                let conf = self.vhosts.devpanel_conf.clone();
                self.trigger_sudo(boxed(SaveConfigCommand {
                    content,
                    path: conf,
                }))
            }

            VHostsMessage::SaveConfigDone(ok, msg) => {
                self.vhosts.config_loading = false;
                if ok {
                    self.vhosts.config_dirty = false;
                }
                self.vhosts.status_msg = Some((ok, msg.clone()));
                let conf = self.vhosts.devpanel_conf.clone();
                if ok {
                    Task::batch([
                        self.show_toast(msg, ok),
                        Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                            Message::VHosts(VHostsMessage::ScanDone(v))
                        }),
                    ])
                } else {
                    self.show_toast(msg, ok)
                }
            }
        }
    }
}
