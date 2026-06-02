//! Application message definitions grouped by domain.

mod dashboard;
mod first_run;
mod settings;
mod ssh_keys;
mod sudo;
mod tools;
mod vhosts;

use iced::Size;

pub use dashboard::DashboardMessage;
pub use first_run::FirstRunMessage;
pub use settings::ConfigMessage;
pub use ssh_keys::SshKeysMessage;
pub use sudo::SudoMessage;
pub use tools::ToolsMessage;
pub use vhosts::VHostsMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    VHosts,
    Tools,
    Config,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectTab(Tab),
    WindowResized(Size),
    NotificationTick,
    DismissAllNotifications,
    Dashboard(DashboardMessage),
    SshKeys(SshKeysMessage),
    Tools(ToolsMessage),
    VHosts(VHostsMessage),
    Sudo(SudoMessage),
    FirstRun(FirstRunMessage),
    Config(ConfigMessage),
}
