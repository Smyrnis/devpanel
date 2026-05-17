use super::SudoCommand;
use crate::core::error::result_status;
use crate::core::system::run_service_cmd;
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
                result_status(crate::ui::tabs::tools::switch_php(self.version, password).await);
            Message::Dashboard(DashboardMessage::PhpSwitchResult(ok, msg))
        })
    }
}
