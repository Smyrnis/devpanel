use super::SudoCommand;
use crate::core::error::result_status;
use crate::messages::{Message, ToolsMessage};
use std::future::Future;
use std::pin::Pin;

pub struct PhpInstallCommand {
    pub version: String,
    pub install: bool,
}

impl SudoCommand for PhpInstallCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::ui::tabs::tools::apt_php_op(self.version, self.install, password).await,
            );
            Message::Tools(ToolsMessage::PhpOpDone(ok, msg))
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
                crate::ui::tabs::tools::toggle_apache_module(name, enable, password).await,
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
                crate::ui::tabs::tools::apt_package_op(self.package, self.install, password).await,
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
                result_status(crate::ui::tabs::tools::composer_op(self.update, password).await);
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
            let (ok, msg) = result_status(
                crate::ui::tabs::tools::redis_service_op(self.action, password).await,
            );
            Message::Tools(ToolsMessage::RedisDone(ok, msg))
        })
    }
}
