//! Debian-family path constants.
//!
//! Keep these in sync with `installation/lib/paths.sh`, which is the shell-side
//! source used by installation scripts. Drift between the Rust and shell values
//! can make setup succeed while the app reads from a different location.

pub const HOSTS_FILE: &str = "/etc/hosts";
pub const WEB_ROOT: &str = "/var/www/html";
pub const APACHE_CONF_FILE: &str = "/etc/apache2/apache2.conf";
pub const APACHE_SITES_AVAILABLE: &str = "/etc/apache2/sites-available";
pub const APACHE_MODS_AVAILABLE: &str = "/etc/apache2/mods-available";
pub const APACHE_MODS_ENABLED: &str = "/etc/apache2/mods-enabled";
pub const DEVPANEL_CONF: &str = "/etc/apache2/sites-available/devpanel.conf";
pub const PHP_BIN_DIR: &str = "/usr/bin";
pub const PHP_ETC_DIR: &str = "/etc/php";
pub const MYSQL_ETC_DIR: &str = "/etc/mysql";
pub const SETUP_LOG: &str = "/var/log/devpanel/setup.log";
pub const COMPOSER_INSTALL_DIR: &str = "/usr/local/bin";
pub const XTERM_BIN: &str = "/usr/bin/xterm";

pub fn php_binary(version: &str) -> String {
    format!("{}/php{}", PHP_BIN_DIR, version.trim_start_matches("php"))
}

pub fn php_cli_ini(version: &str) -> String {
    format!("{}/{}/cli/php.ini", PHP_ETC_DIR, version)
}

pub fn php_apache_ini(version: &str) -> String {
    format!("{}/{}/apache2/php.ini", PHP_ETC_DIR, version)
}

pub fn apache_mod_available(name: &str) -> String {
    format!("{}/{}.load", APACHE_MODS_AVAILABLE, name)
}

pub fn apache_mod_enabled(name: &str) -> String {
    format!("{}/{}.load", APACHE_MODS_ENABLED, name)
}
