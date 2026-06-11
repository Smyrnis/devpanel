mod apache;
mod database;
mod extensions;
mod log;
mod php;
mod runtimes;
mod sections;

use iced::Task;

use crate::app::App;
use crate::messages::{Message, ToolsMessage};

impl App {
    pub(crate) fn handle_tools(&mut self, msg: ToolsMessage) -> Task<Message> {
        match msg {
            ToolsMessage::ScanPhp => self.handle_tools_scan_php(),
            ToolsMessage::ScanDone(results) => self.handle_tools_scan_done(results),
            ToolsMessage::InstallPhp(ver) => self.handle_tools_install_php(ver, true),
            ToolsMessage::RemovePhp(ver) => self.handle_tools_install_php(ver, false),
            ToolsMessage::PhpOpDone(ok, msg) => self.handle_tools_php_op_done(ok, msg),

            ToolsMessage::OpenMysqlCli => self.handle_tools_open_db_terminal("mysql", false),
            ToolsMessage::OpenMariadbCli => self.handle_tools_open_db_terminal("mariadb", false),
            ToolsMessage::OpenMysqlSocket => self.handle_tools_open_db_terminal("mysql", true),

            ToolsMessage::ClearLog => self.handle_tools_clear_log(),
            ToolsMessage::ToggleLog => self.handle_tools_toggle_log(),
            ToolsMessage::CopyFixCommands(commands) => {
                self.handle_tools_copy_fix_commands(commands)
            }
            ToolsMessage::CopyDone => self.handle_tools_copy_done(),

            ToolsMessage::ToggleSection(section) => self.handle_tools_toggle_section(section),
            ToolsMessage::ToolSearchChanged(value) => {
                self.tools.tool_search = value;
                Task::none()
            }
            ToolsMessage::ScanInstalledTools => self.handle_tools_scan_installed_tools(),
            ToolsMessage::InstalledToolsScanned(tools) => {
                self.handle_tools_installed_tools_scanned(tools)
            }

            ToolsMessage::RedisStart => self.handle_tools_redis("start"),
            ToolsMessage::RedisStop => self.handle_tools_redis("stop"),
            ToolsMessage::RedisDone(ok, msg) => self.handle_tools_redis_done(ok, msg),

            ToolsMessage::ScanApacheMods => self.handle_tools_scan_apache_mods(),
            ToolsMessage::ScanApacheModsDone(results) => {
                self.handle_tools_scan_apache_mods_done(results)
            }
            ToolsMessage::ModFilterChanged(value) => {
                self.tools.mod_filter = value;
                Task::none()
            }
            ToolsMessage::EnableApacheMod(name) => self.handle_tools_toggle_apache_mod(name, true),
            ToolsMessage::DisableApacheMod(name) => {
                self.handle_tools_toggle_apache_mod(name, false)
            }
            ToolsMessage::ApacheModDone(ok, msg, name, enabled) => {
                self.handle_tools_apache_mod_done(ok, msg, name, enabled)
            }

            ToolsMessage::ScanPhpExts => self.handle_tools_scan_php_exts(),
            ToolsMessage::ScanPhpExtsDone(results) => self.handle_tools_scan_php_exts_done(results),
            ToolsMessage::InstallPhpExt(pkg) => self.handle_tools_toggle_php_ext(pkg, true),
            ToolsMessage::RemovePhpExt(pkg) => self.handle_tools_toggle_php_ext(pkg, false),
            ToolsMessage::PhpExtDone(ok, msg) => self.handle_tools_php_ext_done(ok, msg),
        }
    }

    pub(crate) fn sync_php_versions_to_vhosts(&mut self) {
        let enabled: Vec<String> = self
            .tools
            .php_releases
            .iter()
            .filter(|r| r.apache_mod_enabled)
            .map(|r| r.version.clone())
            .collect();
        self.vhosts.update_php_versions(enabled);
    }

    fn active_php_for_extensions(&self) -> Option<String> {
        self.tools
            .php_releases
            .iter()
            .find(|r| r.is_active)
            .map(|r| r.version.clone())
            .or_else(|| self.dashboard.active_php_version.clone())
    }
}
