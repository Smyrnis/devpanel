use iced::Task;

use crate::app::App;

use crate::core::sudo_prompt::{
    ApacheModToggleCommand, AptPackageCommand, ComposerCommand, PhpInstallCommand,
    RedisServiceCommand, boxed,
};

use crate::core::system::{copy_to_clipboard, open_db_terminal};

use crate::messages::{Message, ToolsMessage};

impl App {
    pub(crate) fn handle_tools(&mut self, msg: ToolsMessage) -> Task<Message> {
        match msg {
            ToolsMessage::ScanPhp => {
                self.tools.scanning = true;
                Task::perform(
                    crate::ui::tabs::tools::scan_php_versions(
                        self.dashboard.active_php_version.clone(),
                    ),
                    |r| Message::Tools(ToolsMessage::ScanDone(r)),
                )
            }

            ToolsMessage::ScanDone(results) => {
                self.tools.apply_scan(results);
                self.sync_php_versions_to_vhosts();
                Task::none()
            }

            ToolsMessage::InstallPhp(ver) => {
                self.tools
                    .push_log(true, format!("Queued install: PHP {}", ver));
                self.trigger_sudo(boxed(PhpInstallCommand {
                    version: ver,
                    install: true,
                }))
            }
            ToolsMessage::RemovePhp(ver) => {
                self.tools
                    .push_log(true, format!("Queued removal: PHP {}", ver));
                self.trigger_sudo(boxed(PhpInstallCommand {
                    version: ver,
                    install: false,
                }))
            }

            ToolsMessage::PhpOpDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.tools.scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::ui::tabs::dashboard::probe_services(), |r| r),
                    Task::perform(
                        crate::ui::tabs::tools::scan_php_versions(
                            self.dashboard.active_php_version.clone(),
                        ),
                        |r| Message::Tools(ToolsMessage::ScanDone(r)),
                    ),
                ])
            }

            ToolsMessage::OpenMysqlCli => {
                self.tools.db_status = match open_db_terminal("mysql", false) {
                    Ok(s) => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            ToolsMessage::OpenMariadbCli => {
                self.tools.db_status = match open_db_terminal("mariadb", false) {
                    Ok(s) => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }
            ToolsMessage::OpenMysqlSocket => {
                self.tools.db_status = match open_db_terminal("mysql", true) {
                    Ok(s) => format!("Launched: {}", s),
                    Err(e) => format!("Error: {}", e),
                };
                Task::none()
            }

            ToolsMessage::ClearLog => {
                self.tools.install_log.clear();
                self.tools.db_status.clear();
                Task::none()
            }
            ToolsMessage::CopyFixCommands(commands) => {
                Task::perform(copy_to_clipboard(commands), |_| {
                    Message::Tools(ToolsMessage::CopyDone)
                })
            }

            ToolsMessage::CopyDone => self.show_toast("Commands copied to clipboard!".into(), true),

            ToolsMessage::SetSection(s) => {
                self.tools.active_section = s;
                Task::none()
            }

            ToolsMessage::ToolSearchChanged(v) => {
                self.tools.tool_search = v;
                Task::none()
            }

            ToolsMessage::ScanInstalledTools => {
                self.tools.tools_scanning = true;
                Task::perform(crate::ui::tabs::tools::scan_installed_tools(), |r| {
                    Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                })
            }

            ToolsMessage::InstalledToolsScanned(tools) => {
                self.tools.apply_tools_scan(tools);
                Task::none()
            }

            ToolsMessage::InstallComposer => {
                self.tools.push_log(true, "Queued Composer install".into());
                self.trigger_sudo(boxed(ComposerCommand { update: false }))
            }

            ToolsMessage::UpdateComposer => {
                self.tools.push_log(true, "Queued Composer update".into());
                self.trigger_sudo(boxed(ComposerCommand { update: true }))
            }

            ToolsMessage::ComposerDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.tools.tools_scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::ui::tabs::tools::scan_installed_tools(), |r| {
                        Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                    }),
                ])
            }

            ToolsMessage::CopyNvmInstallCommand => {
                let command = "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash".to_string();
                Task::perform(copy_to_clipboard(command), |_| {
                    Message::Tools(ToolsMessage::CopyDone)
                })
            }

            ToolsMessage::RedisStart => {
                self.tools.push_log(true, "Starting Redis".into());
                self.trigger_sudo(boxed(RedisServiceCommand {
                    action: "start".into(),
                }))
            }

            ToolsMessage::RedisStop => {
                self.tools.push_log(true, "Stopping Redis".into());
                self.trigger_sudo(boxed(RedisServiceCommand {
                    action: "stop".into(),
                }))
            }

            ToolsMessage::RedisDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                self.tools.tools_scanning = true;
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::ui::tabs::tools::scan_installed_tools(), |r| {
                        Message::Tools(ToolsMessage::InstalledToolsScanned(r))
                    }),
                ])
            }

            ToolsMessage::ScanApacheMods => {
                Task::perform(crate::ui::tabs::tools::scan_apache_modules(), |r| {
                    Message::Tools(ToolsMessage::ScanApacheModsDone(r))
                })
            }

            ToolsMessage::ScanApacheModsDone(results) => {
                self.tools.apply_mod_scan(results);
                self.sync_php_versions_to_vhosts();
                Task::none()
            }

            ToolsMessage::ModFilterChanged(v) => {
                self.tools.mod_filter = v;
                Task::none()
            }

            ToolsMessage::EnableApacheMod(name) => {
                self.tools
                    .install_log
                    .push((true, format!("Enabling mod_{}…", name)));
                self.trigger_sudo(boxed(ApacheModToggleCommand { name, enable: true }))
            }
            ToolsMessage::DisableApacheMod(name) => {
                self.tools
                    .install_log
                    .push((true, format!("Disabling mod_{}…", name)));
                self.trigger_sudo(boxed(ApacheModToggleCommand {
                    name,
                    enable: false,
                }))
            }

            ToolsMessage::ApacheModDone(ok, msg, name, enabled) => {
                self.tools.push_log(ok, msg.clone());
                if ok {
                    self.tools.set_mod_enabled(&name, enabled);
                }
                self.sync_php_versions_to_vhosts();
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::ui::tabs::tools::scan_apache_modules(), |r| {
                        Message::Tools(ToolsMessage::ScanApacheModsDone(r))
                    }),
                ])
            }

            ToolsMessage::ScanPhpExts => {
                let active = self
                    .tools
                    .php_releases
                    .iter()
                    .find(|r| r.is_active)
                    .map(|r| r.version.clone());
                Task::perform(crate::ui::tabs::tools::scan_php_extensions(active), |r| {
                    Message::Tools(ToolsMessage::ScanPhpExtsDone(r))
                })
            }

            ToolsMessage::ScanPhpExtsDone(results) => {
                self.tools.apply_ext_scan(results);
                Task::none()
            }

            ToolsMessage::InstallPhpExt(pkg) => {
                self.tools
                    .install_log
                    .push((true, format!("Installing {}…", pkg)));
                self.trigger_sudo(boxed(AptPackageCommand {
                    package: pkg,
                    install: true,
                }))
            }
            ToolsMessage::RemovePhpExt(pkg) => {
                self.tools
                    .install_log
                    .push((true, format!("Removing {}…", pkg)));
                self.trigger_sudo(boxed(AptPackageCommand {
                    package: pkg,
                    install: false,
                }))
            }

            ToolsMessage::PhpExtDone(ok, msg) => {
                self.tools.push_log(ok, msg.clone());
                let active = self
                    .tools
                    .php_releases
                    .iter()
                    .find(|r| r.is_active)
                    .map(|r| r.version.clone());
                Task::batch([
                    self.show_toast(msg, ok),
                    Task::perform(crate::ui::tabs::tools::scan_php_extensions(active), |r| {
                        Message::Tools(ToolsMessage::ScanPhpExtsDone(r))
                    }),
                ])
            }
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
}
