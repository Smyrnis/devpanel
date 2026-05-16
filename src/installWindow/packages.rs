pub(super) struct InstallItem {
    pub package: &'static str,
    pub purpose: &'static str,
    pub core: bool,
}

pub(super) const INSTALL_ITEMS: &[InstallItem] = &[
    InstallItem {
        package: "apache2",
        purpose: "HTTP server",
        core: true,
    },
    InstallItem {
        package: "libapache2-mod-php",
        purpose: "PHP module for Apache",
        core: true,
    },
    InstallItem {
        package: "php8.2",
        purpose: "PHP 8.2 CLI + common extensions",
        core: true,
    },
    InstallItem {
        package: "php8.2-cli",
        purpose: "PHP command-line interface",
        core: true,
    },
    InstallItem {
        package: "php8.2-common",
        purpose: "Shared PHP extensions",
        core: true,
    },
    InstallItem {
        package: "php8.2-mysql",
        purpose: "MySQL / MariaDB driver",
        core: false,
    },
    InstallItem {
        package: "php8.2-xml",
        purpose: "XML / DOM / SimpleXML support",
        core: false,
    },
    InstallItem {
        package: "php8.2-mbstring",
        purpose: "Multibyte string functions",
        core: false,
    },
    InstallItem {
        package: "mysql-server",
        purpose: "MySQL / MariaDB database server",
        core: false,
    },
];

pub(super) fn package_installed(package: &str) -> bool {
    std::process::Command::new("dpkg")
        .args(["-s", package])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
