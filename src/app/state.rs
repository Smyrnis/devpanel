use crate::core::{
    config::DevPanelConfig,
    db::{DevPanelDb, UserSettings},
    first_run::FirstRunState,
};
use crate::infra::sudo_prompt::SudoModal;
use crate::installer::{FirstRunInstallOptions, FirstRunPackage, FirstRunSetupStatus};
use crate::messages::{Message, Tab};
use crate::ui::tabs::{
    config::ConfigTab, dashboard::DashboardTab, ssh_keys::SshKeysTab, tools::ToolsTab,
    vhosts::VHostsTab,
};
use iced::{Size, Task};

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub ok: bool,
    pub remaining_ms: u32,
}

pub struct App {
    pub active_tab: Tab,
    pub db: Option<DevPanelDb>,
    pub dashboard: DashboardTab,
    pub ssh_keys: SshKeysTab,
    pub tools: ToolsTab,
    pub vhosts: VHostsTab,
    pub config_tab: ConfigTab,
    pub notifications: Vec<Toast>,
    pub sudo: SudoModal,
    pub first_run_state: FirstRunState,
    pub first_run_options: FirstRunInstallOptions,
    pub first_run_status: FirstRunSetupStatus,
    pub first_run_expanded: Option<FirstRunPackage>,
    pub first_run_installing: bool,
    pub first_run_log_lines: Vec<String>,
    pub setup_issues_checked: bool,
    pub window_size: Size,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let config = DevPanelConfig::load();
        let db = DevPanelDb::open().ok();
        let user_settings = match &db {
            Some(d) => UserSettings::load(d),
            None => UserSettings::default(),
        };

        let app = Self {
            vhosts: VHostsTab::new(config.devpanel_conf.clone()),
            config_tab: ConfigTab::new(user_settings),
            active_tab: Tab::Dashboard,
            dashboard: DashboardTab::new(),
            ssh_keys: SshKeysTab::new(),
            tools: ToolsTab::new(),
            db,
            notifications: Vec::new(),
            sudo: SudoModal::new(),
            first_run_state: FirstRunState::default(),
            first_run_options: FirstRunInstallOptions::default(),
            first_run_status: FirstRunSetupStatus::default(),
            first_run_expanded: Some(FirstRunPackage::ProjectsDir),
            first_run_installing: false,
            first_run_log_lines: Vec::new(),
            setup_issues_checked: false,
            window_size: Size::new(1040.0, 660.0),
        };
        (
            app,
            Task::batch([
                Task::perform(
                    crate::domain::dashboard::service::probe_services(),
                    |snapshot| {
                        Message::Dashboard(crate::messages::DashboardMessage::StatusRefreshed(
                            snapshot,
                        ))
                    },
                ),
                Task::done(Message::FirstRun(
                    crate::messages::FirstRunMessage::ScanStatus,
                )),
            ]),
        )
    }
}
