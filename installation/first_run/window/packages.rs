use crate::core::theme::{self, theme_map as theme_keys};
use crate::installer::{
    FirstRunInstallOptions, FirstRunPackage, FirstRunPackageStatus, FirstRunSetupStatus,
};
use crate::lang::{lang_map::install as keys, text as tr};
use crate::messages::{FirstRunMessage, Message};
use crate::ui::icons::Icon;
use crate::ui::templates::prelude as ui;
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
                packages: &["apache2", "libapache2-mod-php"],
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
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::Php,
                icon: Icon::Php,
                title: "PHP",
                requirement: "optional",
                packages: &["php8.2", "php8.2-cli", "php8.2-common"],
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
                    .into(),
            ],
        ),
        install_row(
            InstallGroup {
                package_group: FirstRunPackage::PhpExtras,
                icon: Icon::Code,
                title: "PHP Extras",
                requirement: "optional",
                packages: &["php8.2-mysql", "php8.2-xml", "php8.2-mbstring"],
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
    let status = if installed {
        tr(keys::STATUS_INSTALLED).to_string()
    } else if selected {
        tr(keys::STATUS_WILL_INSTALL).to_string()
    } else {
        tr(keys::STATUS_SKIPPED).to_string()
    };
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
        install_details(group.title, group.packages, selected, installed)
    ]
    .spacing(6)
    .into()
}

fn install_details<'a>(
    title: &'static str,
    packages: &[&'static str],
    selected: bool,
    installed: bool,
) -> Element<'a, Message> {
    let package_list = if packages.is_empty() {
        "none".to_string()
    } else {
        packages.join(", ")
    };
    let state = if installed {
        tr(keys::STATUS_INSTALLED)
    } else if selected {
        tr(keys::STATUS_WILL_INSTALL)
    } else {
        tr(keys::STATUS_SKIPPED)
    };

    ui::expanded_panel(vec![
        ui::panel_section("Why", vec![ui::detail_text(install_reason(title))]),
        ui::panel_section(
            if packages.is_empty() {
                "Setup"
            } else {
                "Packages"
            },
            vec![
                ui::detail_row(
                    if packages.is_empty() {
                        "Action"
                    } else {
                        "Names"
                    },
                    text(package_list)
                        .size(12)
                        .font(Font::MONOSPACE)
                        .color(theme::color(theme_keys::TEXT_SECONDARY))
                        .into(),
                ),
                ui::detail_row("State", ui::detail_text(state)),
            ],
        ),
    ])
}

fn install_reason(title: &str) -> &'static str {
    match title {
        "Projects Directory" => "Required workspace folder for local projects.",
        "Apache" => "Optional web server for local HTTP virtual hosts.",
        "PHP" => "Optional PHP runtime and command-line tooling.",
        "MySQL" => "Optional database server for local LAMP projects.",
        "PHP Extras" => "Optional extensions commonly needed by PHP web apps.",
        _ => "Package group used by the local development stack.",
    }
}
