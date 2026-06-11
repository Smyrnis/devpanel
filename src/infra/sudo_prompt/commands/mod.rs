use crate::messages::Message;
use std::future::Future;
use std::pin::Pin;

mod dashboard;
mod first_run;
mod tools;
mod vhosts;

pub use dashboard::{PhpSwitchCommand, ServiceControlCommand};
pub use first_run::FirstRunInstallCommand;
pub use tools::{
    ApacheModToggleCommand, AptPackageCommand, ComposerCommand, ComposerVersionCommand,
    NodeNvmAction, NodeNvmCommand, PhpInstallCommand, RedisServiceCommand,
};
pub use vhosts::{
    SaveConfigCommand, VHostAddCommand, VHostBulkDeleteCommand, VHostDeleteCommand,
    VHostEditCommand, VHostToggleHttpsCommand,
};

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
