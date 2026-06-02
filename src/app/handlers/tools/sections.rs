use iced::Task;

use crate::app::App;
use crate::domain::tools::ToolSection;
use crate::messages::{Message, ToolsMessage};

impl App {
    pub(super) fn scan_installed_tools_task() -> Task<Message> {
        Task::perform(crate::ui::tabs::tools::scan_installed_tools(), |r| {
            Message::Tools(ToolsMessage::InstalledToolsScanned(r))
        })
    }

    pub(super) fn scan_apache_modules_task() -> Task<Message> {
        Task::perform(crate::ui::tabs::tools::scan_apache_modules(), |r| {
            Message::Tools(ToolsMessage::ScanApacheModsDone(r))
        })
    }

    pub(super) fn scan_php_extensions_task(active: Option<String>) -> Task<Message> {
        Task::perform(crate::ui::tabs::tools::scan_php_extensions(active), |r| {
            Message::Tools(ToolsMessage::ScanPhpExtsDone(r))
        })
    }

    pub(super) fn handle_tools_toggle_section(&mut self, section: ToolSection) -> Task<Message> {
        if self.tools.active_section.as_ref() == Some(&section) {
            self.tools.active_section = None;
            return Task::none();
        }

        self.tools.active_section = Some(section.clone());
        match section {
            ToolSection::Php => self.handle_tools_scan_php(),
            ToolSection::ApacheMods => {
                self.tools.mods_scanning = true;
                Self::scan_apache_modules_task()
            }
            ToolSection::PhpExts => {
                let active = self.active_php_for_extensions();
                Self::scan_php_extensions_task(active)
            }
            ToolSection::Runtimes | ToolSection::Database => {
                self.tools.tools_scanning = true;
                Self::scan_installed_tools_task()
            }
        }
    }

    pub(super) fn handle_tools_scan_installed_tools(&mut self) -> Task<Message> {
        self.tools.scanning = true;
        self.tools.mods_scanning = true;
        self.tools.tools_scanning = true;
        let active = self.active_php_for_extensions();
        Task::batch([
            Self::scan_installed_tools_task(),
            Self::scan_php_versions_task(self.dashboard.active_php_version.clone()),
            Self::scan_apache_modules_task(),
            Self::scan_php_extensions_task(active),
        ])
    }
}
