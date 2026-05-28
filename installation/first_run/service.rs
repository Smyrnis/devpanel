use crate::core::dry_run;
use crate::core::error::{DevPanelError, DevPanelResult};
use crate::core::paths;
use crate::core::setup_log::{self, LogLevel};
use crate::installer::{FirstRunInstallOptions, FirstRunPackageStatus, FirstRunSetupStatus};
use crate::operations::{self, apache, tools};

const APACHE_PACKAGES: &[&str] = &["apache2", "libapache2-mod-php"];

const PHP_PACKAGES: &[&str] = &["php8.2", "php8.2-cli", "php8.2-common"];

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

pub fn selected_packages(options: FirstRunInstallOptions) -> Vec<&'static str> {
    let mut packages: Vec<&str> = Vec::new();
    if options.install_apache {
        packages.extend(APACHE_PACKAGES);
    }
    if options.install_php {
        packages.extend(PHP_PACKAGES);
    }
    if options.install_php_extras {
        packages.extend(PHP_EXTRA_PACKAGES);
    }
    if options.install_mysql {
        packages.extend(MYSQL_PACKAGES);
    }
    packages
}

pub async fn scan_first_run_status() -> FirstRunSetupStatus {
    if dry_run::active() {
        return FirstRunSetupStatus::default();
    }

    FirstRunSetupStatus {
        projects_dir: projects_dir_status(),
        apache: package_group_status(APACHE_PACKAGES),
        php: package_group_status(PHP_PACKAGES),
        mysql: package_group_status(MYSQL_PACKAGES),
        php_extras: package_group_status(PHP_EXTRA_PACKAGES),
    }
}

fn projects_dir_status() -> FirstRunPackageStatus {
    if target_user()
        .and_then(|user| std::env::var("HOME").ok().map(|home| (user, home)))
        .map(|(_, home)| std::path::Path::new(&home).join("projects").exists())
        .unwrap_or(false)
    {
        FirstRunPackageStatus::Installed
    } else {
        FirstRunPackageStatus::NotInstalled
    }
}

fn package_group_status(packages: &[&str]) -> FirstRunPackageStatus {
    if packages.iter().all(|package| package_installed(package)) {
        FirstRunPackageStatus::Installed
    } else {
        FirstRunPackageStatus::NotInstalled
    }
}

pub async fn run_first_run_install(
    password: String,
    options: FirstRunInstallOptions,
) -> DevPanelResult {
    let packages = selected_packages(options);

    if dry_run::active() {
        setup_log::append_setup_log(LogLevel::Info, "First-run dry-run preview requested");
        let pkg_list = if packages.is_empty() {
            "no optional packages selected".to_string()
        } else {
            packages.join(", ")
        };
        return Ok(format!(
            "[dry-run] Would create the projects directory.\n[dry-run] Optional packages: {}",
            pkg_list
        ));
    }

    setup_log::append_setup_log(LogLevel::Step, "Starting in-app first-run setup");
    setup_log::append_setup_log(
        LogLevel::Info,
        &format!(
            "Selected packages: {}",
            if packages.is_empty() {
                "none".to_string()
            } else {
                packages.join(", ")
            }
        ),
    );

    run_projects_dir_setup_script(&password).await?;

    if !packages.is_empty() {
        setup_log::append_setup_log(LogLevel::Cmd, "apt-get update");
        tools::apt_update(&password).await.map_err(|e| {
            setup_log::append_setup_log(LogLevel::Error, &format!("apt-get update failed: {e}"));
            DevPanelError::Sudo(format!("apt-get update failed: {}", e))
        })?;
        setup_log::append_setup_log(LogLevel::Ok, "apt-get update completed");

        setup_log::append_setup_log(
            LogLevel::Cmd,
            &format!("apt-get install -y {}", packages.join(" ")),
        );
        tools::apt_install_packages(&password, &packages)
            .await
            .map_err(|e| {
                setup_log::append_setup_log(
                    LogLevel::Error,
                    &format!("Package install failed: {e}"),
                );
                DevPanelError::Sudo(format!("Package install failed: {}", e))
            })?;
        setup_log::append_setup_log(LogLevel::Ok, "Package install completed");
    } else {
        setup_log::append_setup_log(LogLevel::Info, "No optional packages selected");
    }

    if options.install_apache || options.install_php {
        let mut enabled_mods: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (_, mod_name) in PHP_VERSION_MODS {
            if enabled_mods.contains(*mod_name) {
                continue;
            }
            let mod_load = paths::apache_mod_available(mod_name);
            if std::path::Path::new(&mod_load).exists() {
                setup_log::append_setup_log(LogLevel::Cmd, &format!("a2enmod {mod_name}"));
                match apache::enable_module(&password, mod_name).await {
                    Ok(_) => setup_log::append_setup_log(
                        LogLevel::Ok,
                        &format!("Enabled Apache module {mod_name}"),
                    ),
                    Err(e) => setup_log::append_setup_log(
                        LogLevel::Warn,
                        &format!("Could not enable Apache module {mod_name}: {e}"),
                    ),
                }
                enabled_mods.insert(mod_name.to_string());
            }
        }

        setup_log::append_setup_log(LogLevel::Cmd, "a2enmod rewrite");
        if let Err(e) = apache::enable_module(&password, "rewrite").await {
            setup_log::append_setup_log(
                LogLevel::Warn,
                &format!("Could not enable Apache rewrite module: {e}"),
            );
        }

        setup_log::append_setup_log(LogLevel::Cmd, "systemctl reload apache2");
        if apache::reload(&password).await.is_err() {
            setup_log::append_setup_log(LogLevel::Warn, "Apache reload failed; trying start");
            match apache::start(&password).await {
                Ok(_) => setup_log::append_setup_log(LogLevel::Ok, "Apache started"),
                Err(e) => setup_log::append_setup_log(
                    LogLevel::Warn,
                    &format!("Apache start failed after reload failure: {e}"),
                ),
            }
        } else {
            setup_log::append_setup_log(LogLevel::Ok, "Apache reloaded");
        }
    } else {
        setup_log::append_setup_log(LogLevel::Info, "Apache setup skipped");
    }

    setup_log::append_setup_log(LogLevel::Ok, "In-app first-run setup complete");
    Ok("Setup complete".into())
}

async fn run_projects_dir_setup_script(password: &str) -> DevPanelResult {
    let script = projects_dir_setup_script_path().ok_or_else(|| {
        let message = "Projects directory setup script not found".to_string();
        setup_log::append_setup_log(LogLevel::Error, &message);
        DevPanelError::Command(message)
    })?;
    let target_user = target_user().ok_or_else(|| {
        let message = "Could not determine target user for setup script".to_string();
        setup_log::append_setup_log(LogLevel::Error, &message);
        DevPanelError::Command(message)
    })?;

    let script = script.to_string_lossy().to_string();
    let sudo_user = format!("SUDO_USER={target_user}");
    setup_log::append_setup_log(LogLevel::Cmd, &format!("env {sudo_user} bash {script}"));

    operations::run(
        password,
        &["env", sudo_user.as_str(), "bash", script.as_str()],
    )
    .await
    .map_err(|e| {
        setup_log::append_setup_log(
            LogLevel::Error,
            &format!("Projects directory setup script failed: {e}"),
        );
        DevPanelError::Sudo(format!("Projects directory setup script failed: {e}"))
    })?;

    setup_log::append_setup_log(LogLevel::Ok, "Projects directory setup completed");
    Ok("Projects directory setup completed".to_string())
}

fn projects_dir_setup_script_path() -> Option<std::path::PathBuf> {
    let local = std::env::current_dir().ok().map(|root| {
        root.join("installation")
            .join("devpanel-create-projects-dir.sh")
    });
    if let Some(path) = local
        && path.exists()
    {
        return Some(path);
    }

    let installed = std::path::PathBuf::from(
        "/usr/share/devpanel/installation/devpanel-create-projects-dir.sh",
    );
    installed.exists().then_some(installed)
}

fn target_user() -> Option<String> {
    ["SUDO_USER", "USER", "LOGNAME"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|value| !value.trim().is_empty() && value != "root")
}

fn package_installed(package: &str) -> bool {
    std::process::Command::new("dpkg")
        .args(["-s", package])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
