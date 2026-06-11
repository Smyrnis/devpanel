use iced::Task;

use crate::app::App;
use crate::messages::{Message, Tab, ToolsMessage, VHostsMessage};

impl App {
    pub(crate) fn handle_select_tab(&mut self, tab: Tab) -> Task<Message> {
        self.active_tab = tab.clone();
        match tab {
            Tab::Dashboard => {
                self.dashboard.runtimes_scanning = true;
                Task::batch([
                    Task::perform(
                        crate::domain::dashboard::service::probe_services(),
                        |snapshot| {
                            Message::Dashboard(crate::messages::DashboardMessage::StatusRefreshed(
                                snapshot,
                            ))
                        },
                    ),
                    Task::perform(
                        crate::domain::tools::service::scan_installed_tools(),
                        |tools| {
                            Message::Dashboard(
                                crate::messages::DashboardMessage::RuntimesRefreshed(tools),
                            )
                        },
                    ),
                ])
            }
            Tab::Tools => {
                self.tools.scanning = true;
                self.tools.tools_scanning = true;
                Task::batch([
                    Task::perform(
                        crate::ui::tabs::tools::scan_php_versions(
                            self.dashboard.active_php_version.clone(),
                        ),
                        |r| Message::Tools(ToolsMessage::ScanDone(r)),
                    ),
                    Task::perform(crate::ui::tabs::tools::scan_installed_tools(), |r| {
                        Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                    }),
                ])
            }
            Tab::VHosts => {
                self.vhosts.scanning = true;
                let conf = self.vhosts.devpanel_conf.clone();
                Task::perform(crate::ui::tabs::vhosts::scan_vhosts(conf), |v| {
                    Message::VHosts(VHostsMessage::ScanDone(v))
                })
            }
            Tab::Config => Task::none(),
        }
    }
}
