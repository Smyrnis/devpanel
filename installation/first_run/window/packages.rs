use crate::core::theme::{self, theme_map as theme_keys};
use crate::installer::service;
use crate::installer::{
    FirstRunInstallOptions, FirstRunPackage, FirstRunPackageStatus, FirstRunSetupStatus,
};
use crate::lang::{lang_map::install as keys, text as tr};
use crate::messages::{FirstRunMessage, Message};
use crate::ui::icons::Icon;
use crate::ui::templates::prelude as ui;
use crate::ui::utils::styles;
use iced::widget::{checkbox, column, text};
use iced::{Element, Font};

struct InstallGroup {
    package_group: FirstRunPackage,
    icon: Icon,
    title: &'static str,
    requirement: &'static str,
    packages: Vec<String>,
}

pub(super) fn install_rows<'a>(
    options: FirstRunInstallOptions,
    status: FirstRunSetupStatus,
    expanded: Option<FirstRunPackage>,
    installing: bool,
) -> Element<'a, Message> {
    ui::row_group(vec![
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::ProjectsDir,
                icon: Icon::Folder,
                title: "Projects Directory",
                requirement: "required",
                packages: Vec::new(),
            },
            status.status_for(FirstRunPackage::ProjectsDir),
            true,
            expanded == Some(FirstRunPackage::ProjectsDir),
            vec![],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::Apache,
                icon: Icon::Apache,
                title: "Apache",
                requirement: "optional",
                packages: vec!["apache2".to_string()],
            },
            status.status_for(FirstRunPackage::Apache),
            options.install_apache,
            expanded == Some(FirstRunPackage::Apache),
            vec![
                checkbox("", options.install_apache)
                    .on_toggle_maybe(if installing {
                        None
                    } else {
                        Some(|v| Message::FirstRun(FirstRunMessage::ToggleApache(v)))
                    })
                    .size(crate::core::app_config::control_metrics().checkbox_size)
                    .style(styles::checkbox_style)
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::Php,
                icon: Icon::Php,
                title: "Latest PHP",
                requirement: "optional",
                packages: service::latest_php_packages(),
            },
            status.status_for(FirstRunPackage::Php),
            options.install_php,
            expanded == Some(FirstRunPackage::Php),
            vec![
                checkbox("", options.install_php)
                    .on_toggle_maybe(if installing {
                        None
                    } else {
                        Some(|v| Message::FirstRun(FirstRunMessage::TogglePhp(v)))
                    })
                    .size(crate::core::app_config::control_metrics().checkbox_size)
                    .style(styles::checkbox_style)
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::Mysql,
                icon: Icon::Database,
                title: "MySQL",
                requirement: "optional",
                packages: vec!["mysql-server".to_string()],
            },
            status.status_for(FirstRunPackage::Mysql),
            options.install_mysql,
            expanded == Some(FirstRunPackage::Mysql),
            vec![
                checkbox("", options.install_mysql)
                    .on_toggle_maybe(if installing {
                        None
                    } else {
                        Some(|v| Message::FirstRun(FirstRunMessage::ToggleMysql(v)))
                    })
                    .size(crate::core::app_config::control_metrics().checkbox_size)
                    .style(styles::checkbox_style)
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::PhpExtras,
                icon: Icon::Code,
                title: "Latest PHP Extras",
                requirement: "optional",
                packages: service::latest_php_extra_packages(),
            },
            status.status_for(FirstRunPackage::PhpExtras),
            options.install_php_extras,
            expanded == Some(FirstRunPackage::PhpExtras),
            vec![
                checkbox("", options.install_php_extras)
                    .on_toggle_maybe(if installing {
                        None
                    } else {
                        Some(|v| Message::FirstRun(FirstRunMessage::TogglePhpExtras(v)))
                    })
                    .size(crate::core::app_config::control_metrics().checkbox_size)
                    .style(styles::checkbox_style)
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::Composer,
                icon: Icon::Tools,
                title: "Composer",
                requirement: "optional",
                packages: vec![
                    "curl".to_string(),
                    "ca-certificates".to_string(),
                    "composer installer".to_string(),
                ],
            },
            status.status_for(FirstRunPackage::Composer),
            options.install_composer,
            expanded == Some(FirstRunPackage::Composer),
            vec![
                checkbox("", options.install_composer)
                    .on_toggle_maybe(if installing {
                        None
                    } else {
                        Some(|v| Message::FirstRun(FirstRunMessage::ToggleComposer(v)))
                    })
                    .size(crate::core::app_config::control_metrics().checkbox_size)
                    .style(styles::checkbox_style)
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::NodeNvm,
                icon: Icon::Code,
                title: "Node via NVM",
                requirement: "optional",
                packages: vec![
                    "curl".to_string(),
                    "ca-certificates".to_string(),
                    "nvm".to_string(),
                    format!("node {}", crate::core::app_config::default_node_version()),
                ],
            },
            status.status_for(FirstRunPackage::NodeNvm),
            options.install_node_nvm,
            expanded == Some(FirstRunPackage::NodeNvm),
            vec![
                checkbox("", options.install_node_nvm)
                    .on_toggle_maybe(if installing {
                        None
                    } else {
                        Some(|v| Message::FirstRun(FirstRunMessage::ToggleNodeNvm(v)))
                    })
                    .size(crate::core::app_config::control_metrics().checkbox_size)
                    .style(styles::checkbox_style)
                    .into(),
            ],
        ),
    ])
}

fn install_row<'a>(
    group: InstallGroup,
    status: FirstRunPackageStatus,
    selected: bool,
    expanded: bool,
    actions: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let installed = status == FirstRunPackageStatus::Installed;
    let is_projects_dir = group.package_group == FirstRunPackage::ProjectsDir;
    let status = status_label(installed, selected, is_projects_dir);
    let tone = if installed {
        ui::BadgeTone::Success
    } else if selected {
        ui::BadgeTone::Warning
    } else {
        ui::BadgeTone::Neutral
    };

    let row = ui::summary_row(
        group.icon,
        group.title,
        format!("{} - {}", group.requirement, status),
        tone,
        actions,
        expanded,
        Some(Message::FirstRun(FirstRunMessage::TogglePackage(
            group.package_group,
        ))),
    );

    if !expanded {
        return row;
    }

    column![
        row,
        install_details(
            group.package_group,
            group.title,
            &group.packages,
            selected,
            installed,
        )
    ]
    .spacing(6)
    .into()
}

fn install_details<'a>(
    package_group: FirstRunPackage,
    title: &'static str,
    packages: &[String],
    selected: bool,
    installed: bool,
) -> Element<'a, Message> {
    let is_projects_dir = package_group == FirstRunPackage::ProjectsDir;
    let state = status_label(installed, selected, is_projects_dir);
    let setup_rows = if is_projects_dir {
        vec![
            ui::detail_row(
                "Path",
                text(projects_dir_path())
                    .size(crate::core::app_config::text_metrics().caption)
                    .font(Font::MONOSPACE)
                    .color(theme::color(theme_keys::TEXT_SECONDARY))
                    .into(),
            ),
            ui::detail_row("Action", ui::detail_text("Create directory")),
            ui::detail_row("State", ui::detail_text(state)),
        ]
    } else {
        vec![
            ui::detail_row(
                "Names",
                text(packages.join(", "))
                    .size(crate::core::app_config::text_metrics().caption)
                    .font(Font::MONOSPACE)
                    .color(theme::color(theme_keys::TEXT_SECONDARY))
                    .into(),
            ),
            ui::detail_row("State", ui::detail_text(state)),
        ]
    };

    ui::expanded_panel(vec![
        ui::panel_section("Why", vec![ui::detail_text(install_reason(title))]),
        ui::panel_section(
            if is_projects_dir { "Setup" } else { "Packages" },
            setup_rows,
        ),
    ])
}

fn status_label(installed: bool, selected: bool, creates_directory: bool) -> &'static str {
    if installed {
        tr(keys::STATUS_INSTALLED)
    } else if creates_directory {
        tr(keys::STATUS_WILL_CREATE)
    } else if selected {
        tr(keys::STATUS_WILL_INSTALL)
    } else {
        tr(keys::STATUS_SKIPPED)
    }
}

fn projects_dir_path() -> String {
    std::env::var("HOME")
        .map(|home| format!("{home}/projects"))
        .unwrap_or_else(|_| "~/projects".to_string())
}

fn install_reason(title: &str) -> &'static str {
    match title {
        "Projects Directory" => "Required workspace folder for local projects.",
        "Apache" => "Optional web server for local HTTP virtual hosts.",
        "Latest PHP" => "Optional latest PHP runtime and Apache module.",
        "MySQL" => "Optional database server for local LAMP projects.",
        "Latest PHP Extras" => "Optional latest PHP extensions commonly needed by web apps.",
        "Composer" => "Optional PHP dependency manager installed globally.",
        "Node via NVM" => "Optional JavaScript runtime managed per user through NVM.",
        _ => "Package group used by the local development stack.",
    }
}
