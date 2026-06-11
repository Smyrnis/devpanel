use super::controls::{restart_button, service_power_button, service_status, service_tone};
use super::panels;
use crate::domain::dashboard::DashboardService;
use crate::lang::{lang_map::dashboard as keys, text as tr};
use crate::messages::{DashboardMessage, Message};
use crate::ui::icons::Icon;
use crate::ui::tabs::dashboard::DashboardTab;
use crate::ui::templates::prelude as ui;
use iced::Element;
use iced::widget::column;

pub(super) fn service_block(tab: &DashboardTab, service: DashboardService) -> Element<'_, Message> {
    let expanded = tab.expanded_service == Some(service);
    let row = match service {
        DashboardService::Apache => apache_row(tab, expanded),
        DashboardService::MySql => mysql_row(tab, expanded),
        DashboardService::Php => php_row(tab, expanded),
        DashboardService::Composer => composer_row(tab, expanded),
        DashboardService::Node => node_row(tab, expanded),
    };

    if !expanded {
        return row;
    }

    let panel = match service {
        DashboardService::Apache => panels::apache_panel(tab),
        DashboardService::MySql => panels::mysql_panel(tab),
        DashboardService::Php => panels::php_panel(tab),
        DashboardService::Composer => panels::composer_panel(tab),
        DashboardService::Node => panels::node_panel(tab),
    };

    column![row, panel].spacing(6).into()
}

fn composer_row(tab: &DashboardTab, expanded: bool) -> Element<'_, Message> {
    let installed = tab.installed_tools.composer_version.is_some();
    let version = tab
        .installed_tools
        .composer_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_INSTALLED));
    ui::summary_row(
        Icon::Code,
        tr(keys::COMPOSER),
        version,
        if installed {
            ui::BadgeTone::Success
        } else {
            ui::BadgeTone::Neutral
        },
        vec![],
        expanded,
        Some(Message::Dashboard(DashboardMessage::ToggleService(
            DashboardService::Composer,
        ))),
    )
}

fn node_row(tab: &DashboardTab, expanded: bool) -> Element<'_, Message> {
    let installed = tab.installed_tools.node_version.is_some();
    let version = tab
        .installed_tools
        .node_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_INSTALLED));
    ui::summary_row(
        Icon::Tools,
        tr(keys::NODE_JS),
        version,
        if installed {
            ui::BadgeTone::Success
        } else {
            ui::BadgeTone::Neutral
        },
        vec![],
        expanded,
        Some(Message::Dashboard(DashboardMessage::ToggleService(
            DashboardService::Node,
        ))),
    )
}

fn apache_row(tab: &DashboardTab, expanded: bool) -> Element<'_, Message> {
    ui::summary_row(
        Icon::Apache,
        tr(keys::APACHE),
        service_status(tab.apache_running, tab.apache_uptime.as_deref()),
        service_tone(tab.apache_running),
        vec![
            service_power_button(
                tab.apache_running,
                Message::Dashboard(DashboardMessage::StartApache),
                Message::Dashboard(DashboardMessage::StopApache),
            ),
            restart_button(Message::Dashboard(DashboardMessage::RestartApache)),
        ],
        expanded,
        Some(Message::Dashboard(DashboardMessage::ToggleService(
            DashboardService::Apache,
        ))),
    )
}

fn mysql_row(tab: &DashboardTab, expanded: bool) -> Element<'_, Message> {
    ui::summary_row(
        Icon::Database,
        tr(keys::MYSQL),
        service_status(tab.mysql_running, tab.mysql_uptime.as_deref()),
        service_tone(tab.mysql_running),
        vec![
            service_power_button(
                tab.mysql_running,
                Message::Dashboard(DashboardMessage::StartMySQL),
                Message::Dashboard(DashboardMessage::StopMySQL),
            ),
            restart_button(Message::Dashboard(DashboardMessage::RestartMySQL)),
        ],
        expanded,
        Some(Message::Dashboard(DashboardMessage::ToggleService(
            DashboardService::MySql,
        ))),
    )
}

fn php_row(tab: &DashboardTab, expanded: bool) -> Element<'_, Message> {
    let version = tab
        .active_php_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_AVAILABLE_SHORT));
    ui::summary_row(
        Icon::Php,
        tr(keys::PHP),
        format!("{} {}", version, tr(keys::ACTIVE_VERSION)),
        ui::BadgeTone::Info,
        vec![if expanded {
            ui::secondary_icon_button(
                Icon::Info,
                tr(keys::PHP_INFO),
                Message::Dashboard(DashboardMessage::ShowPhpInfo),
            )
        } else {
            ui::secondary_icon_button(
                Icon::Php,
                tr(keys::CHANGE_VERSION),
                Message::Dashboard(DashboardMessage::ToggleService(DashboardService::Php)),
            )
        }],
        expanded,
        Some(Message::Dashboard(DashboardMessage::ToggleService(
            DashboardService::Php,
        ))),
    )
}
