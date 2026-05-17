use crate::core::error::result_status;
use crate::core::system::run_service_cmd;
use crate::messages::{DashboardMessage, FirstRunMessage, Message, ToolsMessage, VHostsMessage};
use std::future::Future;
use std::pin::Pin;

pub trait SudoCommand: Send + 'static {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>>;

    fn is_first_run_install(&self) -> bool {
        false
    }
}

pub type BoxedSudoCommand = Box<dyn SudoCommand>;

pub fn boxed(command: impl SudoCommand) -> BoxedSudoCommand {
    Box::new(command)
}

pub struct ServiceControlCommand {
    pub service: String,
    pub action: String,
}

impl SudoCommand for ServiceControlCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move { run_service_cmd(self.service, self.action, password).await })
    }
}

pub struct RestartAllCommand;

impl SudoCommand for RestartAllCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let apache =
                run_service_cmd("apache2".into(), "restart".into(), password.clone()).await;
            let mysql = run_service_cmd("mysql".into(), "restart".into(), password).await;
            match (apache, mysql) {
                (
                    Message::Dashboard(DashboardMessage::ServiceResult {
                        success: apache_ok,
                        output: apache_output,
                        ..
                    }),
                    Message::Dashboard(DashboardMessage::ServiceResult {
                        success: mysql_ok,
                        output: mysql_output,
                        ..
                    }),
                ) => Message::Dashboard(DashboardMessage::ServiceResult {
                    service: "apache2 and mysql".into(),
                    action: "restart".into(),
                    success: apache_ok && mysql_ok,
                    output: [apache_output, mysql_output]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join("\n"),
                }),
                _ => Message::Dashboard(DashboardMessage::ServiceResult {
                    service: "apache2 and mysql".into(),
                    action: "restart".into(),
                    success: false,
                    output: "Unexpected restart result".into(),
                }),
            }
        })
    }
}

pub struct PhpSwitchCommand {
    pub version: String,
}

impl SudoCommand for PhpSwitchCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) =
                result_status(crate::tabs::tools::switch_php(self.version, password).await);
            Message::Dashboard(DashboardMessage::PhpSwitchResult(ok, msg))
        })
    }
}

pub struct PhpInstallCommand {
    pub version: String,
    pub install: bool,
}

impl SudoCommand for PhpInstallCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::tabs::tools::apt_php_op(self.version, self.install, password).await,
            );
            Message::Tools(ToolsMessage::PhpOpDone(ok, msg))
        })
    }
}

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
                crate::tabs::vhosts::add_vhost(
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
                crate::tabs::vhosts::edit_vhost(
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
                crate::tabs::vhosts::delete_vhost(self.devpanel_conf, self.index, password).await,
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
                crate::tabs::vhosts::bulk_delete_vhosts(self.devpanel_conf, self.indexes, password)
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
                crate::tabs::vhosts::toggle_https(self.devpanel_conf, self.index, password).await,
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
                crate::tabs::vhosts::save_config_file(self.path, self.content, password).await,
            );
            Message::VHosts(VHostsMessage::SaveConfigDone(ok, msg))
        })
    }
}

pub struct ApacheModToggleCommand {
    pub name: String,
    pub enable: bool,
}

impl SudoCommand for ApacheModToggleCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let name = self.name;
            let enable = self.enable;
            let result_name = name.clone();
            let (ok, msg) = result_status(
                crate::tabs::tools::toggle_apache_module(name, enable, password).await,
            );
            Message::Tools(ToolsMessage::ApacheModDone(ok, msg, result_name, enable))
        })
    }
}

pub struct AptPackageCommand {
    pub package: String,
    pub install: bool,
}

impl SudoCommand for AptPackageCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::tabs::tools::apt_package_op(self.package, self.install, password).await,
            );
            Message::Tools(ToolsMessage::PhpExtDone(ok, msg))
        })
    }
}

pub struct ComposerCommand {
    pub update: bool,
}

impl SudoCommand for ComposerCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) =
                result_status(crate::tabs::tools::composer_op(self.update, password).await);
            Message::Tools(ToolsMessage::ComposerDone(ok, msg))
        })
    }
}

pub struct RedisServiceCommand {
    pub action: String,
}

impl SudoCommand for RedisServiceCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) =
                result_status(crate::tabs::tools::redis_service_op(self.action, password).await);
            Message::Tools(ToolsMessage::RedisDone(ok, msg))
        })
    }
}

pub struct FirstRunInstallCommand {
    pub options: crate::core::first_run_install::FirstRunInstallOptions,
}

impl SudoCommand for FirstRunInstallCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::core::first_run_install::run_first_run_install(password, self.options).await,
            );
            Message::FirstRun(FirstRunMessage::InstallDone(ok, msg))
        })
    }

    fn is_first_run_install(&self) -> bool {
        true
    }
}
