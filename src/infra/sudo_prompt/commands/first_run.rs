use super::SudoCommand;
use crate::core::error::result_status;
use crate::messages::{FirstRunMessage, Message};
use std::future::Future;
use std::pin::Pin;

pub struct FirstRunInstallCommand {
    pub options: crate::installer::FirstRunInstallOptions,
}

impl SudoCommand for FirstRunInstallCommand {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>> {
        let password = password.to_string();
        Box::pin(async move {
            let (ok, msg) = result_status(
                crate::installer::service::run_first_run_install(password, self.options).await,
            );
            Message::FirstRun(FirstRunMessage::InstallDone(ok, msg))
        })
    }

    fn is_first_run_install(&self) -> bool {
        true
    }
}
