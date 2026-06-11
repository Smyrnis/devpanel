use super::controls::{action_row, running_text};
use crate::core::paths;
use crate::core::theme::{self, theme_map as theme_keys};
use crate::lang::{lang_map::dashboard as keys, text as tr};
use crate::messages::{DashboardMessage, Message, VHostsMessage};
use crate::ui::icons::Icon;
use crate::ui::tabs::dashboard::DashboardTab;
use crate::ui::templates::prelude as ui;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length, Padding};

pub(super) fn apache_panel(tab: &DashboardTab) -> Element<'_, Message> {
    ui::expanded_panel(vec![
        ui::panel_section(
            tr(keys::QUICK_ACTIONS),
            vec![action_row(vec![
                ui::secondary_icon_button(
                    Icon::Globe,
                    tr(keys::LOCALHOST),
                    Message::Dashboard(DashboardMessage::OpenLocalhost),
                ),
                ui::secondary_icon_button(
                    Icon::Config,
                    tr(keys::DEVPANEL_CONFIG),
                    Message::VHosts(VHostsMessage::OpenDevpanelConf),
                ),
                ui::secondary_icon_button(
                    Icon::Host,
                    tr(keys::SITES_AVAILABLE),
                    Message::Dashboard(DashboardMessage::NavigateApache2Sites),
                ),
            ])],
        ),
        ui::panel_section(
            tr(keys::STATUS),
            vec![
                ui::detail_row(
                    tr(keys::SERVICE),
                    ui::detail_text(running_text(tab.apache_running)),
                ),
                ui::detail_row(tr(keys::CONFIG), ui::path_chip(paths::APACHE_CONF_FILE)),
                ui::detail_row(
                    tr(keys::SITES),
                    ui::path_chip(paths::APACHE_SITES_AVAILABLE),
                ),
            ],
        ),
        apache_problems(tab),
        ui::panel_section(
            tr(keys::ADVANCED),
            vec![action_row(vec![
                ui::secondary_icon_button(
                    Icon::Apache,
                    tr(keys::APACHE_CONFIG),
                    Message::Dashboard(DashboardMessage::NavigateApache2Conf),
                ),
                ui::secondary_icon_button(
                    Icon::Folder,
                    tr(keys::WEB_ROOT),
                    Message::Dashboard(DashboardMessage::OpenWebRoot),
                ),
                ui::secondary_icon_button(
                    Icon::Code,
                    paths::HOSTS_FILE,
                    Message::Dashboard(DashboardMessage::NavigateHostsFile),
                ),
            ])],
        ),
    ])
}

pub(super) fn mysql_panel(tab: &DashboardTab) -> Element<'_, Message> {
    ui::expanded_panel(vec![
        ui::panel_section(
            tr(keys::QUICK_ACTIONS),
            vec![action_row(vec![
                ui::secondary_icon_button(
                    Icon::Database,
                    tr(keys::PHPMYADMIN),
                    Message::Dashboard(DashboardMessage::OpenPhpMyAdmin),
                ),
                ui::secondary_icon_button(
                    Icon::Terminal,
                    tr(keys::MYSQL_TERMINAL),
                    Message::Dashboard(DashboardMessage::OpenMysqlTerminal),
                ),
                ui::secondary_icon_button(
                    Icon::Config,
                    tr(keys::MYSQL_CONFIG),
                    Message::Dashboard(DashboardMessage::NavigateMysqlDir),
                ),
            ])],
        ),
        ui::panel_section(
            tr(keys::STATUS),
            vec![
                ui::detail_row(
                    tr(keys::SERVICE),
                    ui::detail_text(running_text(tab.mysql_running)),
                ),
                ui::detail_row(tr(keys::CONFIG), ui::path_chip(paths::MYSQL_ETC_DIR)),
            ],
        ),
    ])
}

pub(super) fn php_panel(tab: &DashboardTab) -> Element<'_, Message> {
    let picker: Element<Message> = if tab.php_versions.is_empty() {
        container(
            text(tr(keys::PHP_NOT_DETECTED))
                .size(crate::core::app_config::text_metrics().caption)
                .color(theme::color(theme_keys::TEXT_MUTED)),
        )
        .padding(Padding::from([9, 12]))
        .width(Length::Fill)
        .style(ui::surface_style())
        .into()
    } else {
        ui::dropdown(
            &tab.php_versions[..],
            tab.active_php_version.as_ref(),
            |v| Message::Dashboard(DashboardMessage::SwitchPhpVersion(v)),
        )
    };

    let version_rows: Vec<Element<Message>> = tab
        .php_versions
        .iter()
        .map(|version| {
            let active = tab.active_php_version.as_ref() == Some(version);
            row![
                text(version.as_str())
                    .size(crate::core::app_config::text_metrics().body)
                    .color(theme::color(theme_keys::TEXT_PRIMARY))
                    .width(Length::Fill),
                ui::small_badge(
                    if active {
                        tr(keys::ACTIVE)
                    } else {
                        tr(keys::INSTALLED)
                    },
                    if active {
                        ui::BadgeTone::Success
                    } else {
                        ui::BadgeTone::Neutral
                    },
                ),
            ]
            .align_y(Alignment::Center)
            .into()
        })
        .collect();

    ui::expanded_panel(vec![
        ui::panel_section(
            tr(keys::QUICK_ACTIONS),
            vec![action_row(vec![
                ui::secondary_icon_button(
                    Icon::Info,
                    tr(keys::PHP_INFO),
                    Message::Dashboard(DashboardMessage::ShowPhpInfo),
                ),
                ui::secondary_icon_button(
                    Icon::Php,
                    tr(keys::PHP_INI),
                    Message::Dashboard(DashboardMessage::OpenPhpIni),
                ),
                ui::secondary_icon_button(
                    Icon::Tools,
                    tr(keys::MANAGE_EXTENSIONS),
                    Message::Dashboard(DashboardMessage::ManagePhpExtensions),
                ),
                ui::secondary_icon_button(
                    Icon::Folder,
                    tr(keys::PHP_FOLDER),
                    Message::Dashboard(DashboardMessage::NavigatePhpDir),
                ),
            ])],
        ),
        ui::panel_section(tr(keys::CHANGE_VERSION), vec![picker]),
        ui::panel_section(
            tr(keys::INSTALLED_VERSIONS),
            if version_rows.is_empty() {
                vec![ui::detail_text(tr(keys::PHP_NOT_DETECTED))]
            } else {
                version_rows
            },
        ),
    ])
}

pub(super) fn composer_panel(tab: &DashboardTab) -> Element<'_, Message> {
    let installed = tab.installed_tools.composer_version.is_some();
    let status = tab
        .installed_tools
        .composer_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_INSTALLED));

    let controls: Element<'_, Message> = if installed {
        column![
            ui::dropdown(
                crate::core::app_config::composer_versions(),
                Some(&tab.selected_composer_version),
                |version| Message::Dashboard(DashboardMessage::ComposerVersionSelected(version)),
            ),
            action_row(vec![
                ui::secondary_icon_button(
                    Icon::Code,
                    tr(keys::APPLY_VERSION),
                    Message::Dashboard(DashboardMessage::SwitchComposerVersion(
                        tab.selected_composer_version.clone(),
                    )),
                ),
                ui::secondary_icon_button(
                    Icon::Refresh,
                    tr(keys::UPDATE),
                    Message::Dashboard(DashboardMessage::UpdateComposer),
                ),
            ]),
        ]
        .spacing(8)
        .into()
    } else {
        action_row(vec![ui::secondary_icon_button(
            Icon::Code,
            tr(keys::INSTALL),
            Message::Dashboard(DashboardMessage::InstallComposer),
        )])
    };

    ui::expanded_panel(vec![
        ui::panel_section(
            tr(keys::STATUS),
            vec![ui::detail_row(
                tr(keys::COMPOSER_VERSION),
                ui::detail_text(status),
            )],
        ),
        ui::panel_section(tr(keys::VERSION_MANAGEMENT), vec![controls]),
    ])
}

pub(super) fn node_panel(tab: &DashboardTab) -> Element<'_, Message> {
    let tools = &tab.installed_tools;
    let node = tools
        .node_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_INSTALLED));
    let npm = tools
        .npm_version
        .as_deref()
        .unwrap_or(tr(keys::NOT_INSTALLED));
    let nvm = if tools.nvm_available {
        tr(keys::AVAILABLE)
    } else {
        tr(keys::NOT_INSTALLED)
    };

    let controls: Element<'_, Message> = if tools.nvm_available {
        let version = tab.selected_node_version.clone();
        column![
            ui::dropdown(
                crate::core::app_config::node_versions(),
                Some(&tab.selected_node_version),
                |selected| Message::Dashboard(DashboardMessage::NodeVersionSelected(selected)),
            ),
            action_row(vec![
                ui::secondary_icon_button(
                    Icon::Plus,
                    tr(keys::INSTALL_VERSION),
                    Message::Dashboard(DashboardMessage::InstallNodeVersion(version.clone())),
                ),
                ui::secondary_icon_button(
                    Icon::Refresh,
                    tr(keys::USE_VERSION),
                    Message::Dashboard(DashboardMessage::SwitchNodeVersion(version.clone())),
                ),
                ui::secondary_icon_button(
                    Icon::Check,
                    tr(keys::SET_DEFAULT),
                    Message::Dashboard(DashboardMessage::SetDefaultNodeVersion(version)),
                ),
            ]),
        ]
        .spacing(8)
        .into()
    } else {
        action_row(vec![ui::secondary_icon_button(
            Icon::Plus,
            tr(keys::INSTALL_NVM),
            Message::Dashboard(DashboardMessage::InstallNvm),
        )])
    };

    ui::expanded_panel(vec![
        ui::panel_section(
            tr(keys::STATUS),
            vec![
                ui::detail_row(tr(keys::NODE_VERSION), ui::detail_text(node)),
                ui::detail_row(tr(keys::NPM_VERSION), ui::detail_text(npm)),
                ui::detail_row(tr(keys::NVM), ui::detail_text(nvm)),
            ],
        ),
        ui::panel_section(tr(keys::VERSION_MANAGEMENT), vec![controls]),
    ])
}

fn apache_problems(tab: &DashboardTab) -> Element<'_, Message> {
    if tab.recent_failures.is_empty() {
        return ui::panel_section(
            tr(keys::PROBLEMS),
            vec![ui::detail_text(tr(keys::NO_RECENT_APACHE_FAILURES))],
        );
    }

    let failures: Vec<Element<Message>> = tab
        .recent_failures
        .iter()
        .map(|line| {
            text(line.as_str())
                .size(crate::core::app_config::text_metrics().caption)
                .color(theme::color(theme_keys::TEXT_MUTED))
                .into()
        })
        .collect();

    ui::panel_section(tr(keys::RECENT_FAILURES), failures)
}
