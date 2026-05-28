use super::SudoCommand;
use crate::core::error::result_status;
use crate::messages::{Message, VHostsMessage};
use std::future::Future;
use std::pin::Pin;

pub struct VHostAddCommand {
    pub devpanel_conf: String,
    pub server_name: String,
    pub document_root: String,
    pub php_version: Option<String>,
    pub https_enabled: bool,
}

impl SudoCommand for VHostAddCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::ui::tabs::vhosts::add_vhost(
                    self.devpanel_conf,
                    self.server_name,
                    self.document_root,
                    self.php_version,
                    self.https_enabled,
                    password,
                )
                .await,
            );
            Message::VHosts(VHostsMessage::CreateDone(ok, msg))
        })
    }
}

pub struct VHostEditCommand {
    pub devpanel_conf: String,
    pub index: usize,
    pub server_name: String,
    pub document_root: String,
    pub php_version: Option<String>,
    pub https_enabled: bool,
}

impl SudoCommand for VHostEditCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::ui::tabs::vhosts::edit_vhost(
                    self.devpanel_conf,
                    self.index,
                    self.server_name,
                    self.document_root,
                    self.php_version,
                    self.https_enabled,
                    password,
                )
                .await,
            );
            Message::VHosts(VHostsMessage::SaveEditDone(ok, msg))
        })
    }
}

pub struct VHostDeleteCommand {
    pub devpanel_conf: String,
    pub index: usize,
}

impl SudoCommand for VHostDeleteCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::ui::tabs::vhosts::delete_vhost(self.devpanel_conf, self.index, password)
                    .await,
            );
            Message::VHosts(VHostsMessage::DeleteDone(ok, msg))
        })
    }
}

pub struct VHostBulkDeleteCommand {
    pub devpanel_conf: String,
    pub indexes: Vec<usize>,
}

impl SudoCommand for VHostBulkDeleteCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::ui::tabs::vhosts::bulk_delete_vhosts(
                    self.devpanel_conf,
                    self.indexes,
                    password,
                )
                .await,
            );
            Message::VHosts(VHostsMessage::DeleteDone(ok, msg))
        })
    }
}

pub struct VHostToggleHttpsCommand {
    pub devpanel_conf: String,
    pub index: usize,
}

impl SudoCommand for VHostToggleHttpsCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::ui::tabs::vhosts::toggle_https(self.devpanel_conf, self.index, password)
                    .await,
            );
            Message::VHosts(VHostsMessage::SaveEditDone(ok, msg))
        })
    }
}

pub struct SaveConfigCommand {
    pub path: String,
    pub content: String,
}

impl SudoCommand for SaveConfigCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::ui::tabs::vhosts::save_config_file(self.path, self.content, password).await,
            );
            Message::VHosts(VHostsMessage::SaveConfigDone(ok, msg))
        })
    }
}
