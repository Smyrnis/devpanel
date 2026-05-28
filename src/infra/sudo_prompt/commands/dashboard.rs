use super::SudoCommand;
use crate::core::error::result_status;
use crate::infra::system::run_service_cmd;
use crate::messages::{DashboardMessage, Message};
use std::future::Future;
use std::pin::Pin;

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

pub struct PhpSwitchCommand {
    pub version: String,
}

impl SudoCommand for PhpSwitchCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) =
                result_status(crate::ui::tabs::tools::switch_php(self.version, password).await);
            Message::Dashboard(DashboardMessage::PhpSwitchResult(ok, msg))
        })
    }
}
