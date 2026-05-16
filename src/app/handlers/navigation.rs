use iced::Task;

use crate::app::App;
use crate::messages::{Message, SshKeysMessage, Tab, ToolsMessage, VHostsMessage};

impl App {
    pub(crate) fn handle_select_tab(&mut self, tab: Tab) -> Task<Message> {
        self.active_tab = tab.clone();
        match tab {
            Tab::Dashboard => Task::perform(crate::tabs::dashboard::probe_services(), |r| r),
            Tab::SshKeys => Task::perform(crate::tabs::ssh_keys::list_keys(), |keys| {
                Message::SshKeys(SshKeysMessage::KeysListed(keys))
            }),
            Tab::Tools => {
                self.tools.scanning = true;
                self.tools.tools_scanning = true;
                Task::batch([
                    Task::perform(
                        crate::tabs::tools::scan_php_versions(
                            self.dashboard.active_php_version.clone(),
                        ),
                        |r| Message::Tools(ToolsMessage::ScanDone(r)),
                    ),
                    Task::perform(crate::tabs::tools::scan_installed_tools(), |r| {
                        Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                    }),
                ])
            }
            Tab::Repos => Task::none(),
            Tab::VHosts => {
                self.vhosts.scanning = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::tabs::vhosts::scan_vhosts(conf), |v| {
                    Message::VHosts(VHostsMessage::ScanDone(v))
                })
            }
            Tab::Config => Task::none(),
        }
    }
}
