use crate::core::dry_run;
use crate::core::sudo_prompt::sudo_cmd_with_password;

const BASE_PACKAGES: &[&str] = &[
    "apache2",
    "libapache2-mod-php",
    "php8.2",
    "php8.2-cli",
    "php8.2-common",
];

const PHP_EXTRA_PACKAGES: &[&str] = &["php8.2-mysql", "php8.2-xml", "php8.2-mbstring"];

const MYSQL_PACKAGES: &[&str] = &["mysql-server"];

const PHP_VERSION_MODS: &[(&str, &str)] = &[
    ("5.6", "php5.6"),
    ("5.6", "php5"),
    ("7.4", "php7.4"),
    ("8.0", "php8.0"),
    ("8.1", "php8.1"),
    ("8.2", "php8.2"),
    ("8.3", "php8.3"),
    ("8.4", "php8.4"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstRunInstallOptions {
    pub install_mysql: bool,
    pub install_php_extras: bool,
}

impl Default for FirstRunInstallOptions {
    fn default() -> Self {
        Self {
            install_mysql: true,
            install_php_extras: true,
        }
    }
}

pub async fn run_first_run_install(
    password: String,
    options: FirstRunInstallOptions,
) -> (bool, String) {
    let mut packages: Vec<&str> = BASE_PACKAGES.to_vec();
    if options.install_php_extras {
        packages.extend(PHP_EXTRA_PACKAGES);
    }
    if options.install_mysql {
        packages.extend(MYSQL_PACKAGES);
    }

    if dry_run::active() {
        dry_run::log("run_first_run_install — skipped entirely in dry-run mode");
        let pkg_list = packages.join(", ");
        return (
            true,
            format!(
                "[dry-run] Would have installed: {}\nAnd enabled mod_phpX.Y for all available PHP versions.",
                pkg_list
            ),
        );
    }

    if let Err(e) = sudo_cmd_with_password(&password, &["apt-get", "update"]).await {
        return (false, format!("apt-get update failed: {}", e));
    }

    let pkg_list = packages.join(" ");
    let install_cmd = format!(
        "DEBIAN_FRONTEND=noninteractive apt-get install -y {}",
        pkg_list
    );
    if let Err(e) = sudo_cmd_with_password(&password, &["sh", "-c", &install_cmd]).await {
        return (false, format!("Package install failed: {}", e));
    }

    let mut enabled_mods: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (_, mod_name) in PHP_VERSION_MODS {
        if enabled_mods.contains(*mod_name) {
            continue;
        }
        let mod_load = format!("/etc/apache2/mods-available/{}.load", mod_name);
        if std::path::Path::new(&mod_load).exists() {
            let _ = sudo_cmd_with_password(&password, &["a2enmod", mod_name]).await;
            enabled_mods.insert(mod_name.to_string());
        }
    }

    let _ = sudo_cmd_with_password(&password, &["a2enmod", "rewrite"]).await;

    if sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"])
        .await
        .is_err()
    {
        let _ = sudo_cmd_with_password(&password, &["systemctl", "start", "apache2"]).await;
    }

    (true, "Setup complete — Apache and PHP are ready".into())
}
