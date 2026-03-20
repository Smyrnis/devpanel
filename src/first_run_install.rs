// src/first_run_install.rs
// Async task that runs when the user clicks "Continue" in the welcome modal.
// In dry-run (dev) mode the whole thing is skipped — nothing is installed.

use crate::dry_run;
use crate::sudo_prompt::sudo_cmd_with_password;

const BASE_PACKAGES: &[&str] = &[
    "apache2",
    "libapache2-mod-php",
    "php8.2",
    "php8.2-cli",
    "php8.2-common",
    "php8.2-mysql",
    "php8.2-xml",
    "php8.2-mbstring",
    "mysql-server",
];

const PHP_VERSIONS: &[&str] = &["7.4", "8.0", "8.1", "8.2", "8.3", "8.4"];

pub async fn run_first_run_install(password: String) -> (bool, String) {
    if dry_run::active() {
        dry_run::log("run_first_run_install — skipped entirely in dry-run mode");
        let pkg_list = BASE_PACKAGES.join(", ");
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

    let pkg_list    = BASE_PACKAGES.join(" ");
    let install_cmd = format!("DEBIAN_FRONTEND=noninteractive apt-get install -y {}", pkg_list);
    if let Err(e) = sudo_cmd_with_password(&password, &["sh", "-c", &install_cmd]).await {
        return (false, format!("Package install failed: {}", e));
    }

    for ver in PHP_VERSIONS {
        let mod_name = format!("php{}", ver);
        let mod_load = format!("/etc/apache2/mods-available/{}.load", mod_name);
        if std::path::Path::new(&mod_load).exists() {
            let _ = sudo_cmd_with_password(&password, &["a2enmod", &mod_name]).await;
        }
    }

    let _ = sudo_cmd_with_password(&password, &["a2enmod", "rewrite"]).await;

    if let Err(_) = sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"]).await {
        let _ = sudo_cmd_with_password(&password, &["systemctl", "start", "apache2"]).await;
    }

    (true, "Setup complete — Apache and PHP are ready".into())
}