use crate::lang::lang_map::install as keys;

pub(super) struct InstallItem {
    pub package: &'static str,
    pub purpose_key: &'static str,
    pub core: bool,
}

pub(super) const INSTALL_ITEMS: &[InstallItem] = &[
    InstallItem {
        package: "apache2",
        purpose_key: keys::PURPOSE_HTTP_SERVER,
        core: true,
    },
    InstallItem {
        package: "libapache2-mod-php",
        purpose_key: keys::PURPOSE_APACHE_PHP_MODULE,
        core: true,
    },
    InstallItem {
        package: "php8.2",
        purpose_key: keys::PURPOSE_PHP_COMMON,
        core: true,
    },
    InstallItem {
        package: "php8.2-cli",
        purpose_key: keys::PURPOSE_PHP_CLI,
        core: true,
    },
    InstallItem {
        package: "php8.2-common",
        purpose_key: keys::PURPOSE_SHARED_PHP_EXTENSIONS,
        core: true,
    },
    InstallItem {
        package: "php8.2-mysql",
        purpose_key: keys::PURPOSE_MYSQL_DRIVER,
        core: false,
    },
    InstallItem {
        package: "php8.2-xml",
        purpose_key: keys::PURPOSE_XML_SUPPORT,
        core: false,
    },
    InstallItem {
        package: "php8.2-mbstring",
        purpose_key: keys::PURPOSE_MBSTRING,
        core: false,
    },
    InstallItem {
        package: "mysql-server",
        purpose_key: keys::PURPOSE_MYSQL_SERVER,
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
