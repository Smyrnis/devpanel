use crate::core::theme::{self, theme_map as theme_keys};
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
    packages: &'static [&'static str],
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
                packages: &[],
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
                packages: &["apache2"],
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
                    .size(16)
                    .style(styles::checkbox_style)
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::Php,
                icon: Icon::Php,
                title: "PHP 8.5",
                requirement: "optional",
                packages: &[
                    "php8.5",
                    "php8.5-cli",
                    "php8.5-common",
                    "libapache2-mod-php8.5",
                ],
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
                    .size(16)
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
                packages: &["mysql-server"],
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
                    .size(16)
                    .style(styles::checkbox_style)
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::PhpExtras,
                icon: Icon::Code,
                title: "PHP 8.5 Extras",
                requirement: "optional",
                packages: &["php8.5-mysql", "php8.5-xml", "php8.5-mbstring"],
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
                    .size(16)
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
            group.packages,
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
    packages: &[&'static str],
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
                    .size(12)
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
                    .size(12)
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
        "PHP 8.5" => "Optional latest PHP runtime and Apache module.",
        "MySQL" => "Optional database server for local LAMP projects.",
        "PHP 8.5 Extras" => "Optional latest PHP extensions commonly needed by web apps.",
        _ => "Package group used by the local development stack.",
    }
}
