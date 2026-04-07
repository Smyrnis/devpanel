use super::{ApacheModule, PhpStatus};
use crate::core::sudo_prompt::sudo_cmd_with_password;
use tokio::process::Command;

struct PhpVersionMeta {
    /// The version string shown in the UI, e.g. "5.6" or "8.2"
    version: &'static str,
    /// Binary name(s) to probe under /usr/bin/, in order of preference
    binaries: &'static [&'static str],
    /// Name passed to a2enmod / a2dismod
    mod_name: &'static str,
    /// apt package name used for install / remove
    apt_pkg: &'static str,
}

const PHP_VERSIONS: &[PhpVersionMeta] = &[
    PhpVersionMeta {
        version: "5.6",
        binaries: &["php5.6", "php5"],
        mod_name: "php5.6",
        apt_pkg: "php5.6",
    },
    PhpVersionMeta {
        version: "7.4",
        binaries: &["php7.4"],
        mod_name: "php7.4",
        apt_pkg: "php7.4",
    },
    PhpVersionMeta {
        version: "8.0",
        binaries: &["php8.0"],
        mod_name: "php8.0",
        apt_pkg: "php8.0",
    },
    PhpVersionMeta {
        version: "8.1",
        binaries: &["php8.1"],
        mod_name: "php8.1",
        apt_pkg: "php8.1",
    },
    PhpVersionMeta {
        version: "8.2",
        binaries: &["php8.2"],
        mod_name: "php8.2",
        apt_pkg: "php8.2",
    },
    PhpVersionMeta {
        version: "8.3",
        binaries: &["php8.3"],
        mod_name: "php8.3",
        apt_pkg: "php8.3",
    },
    PhpVersionMeta {
        version: "8.4",
        binaries: &["php8.4"],
        mod_name: "php8.4",
        apt_pkg: "php8.4",
    },
];

pub async fn scan_php_versions(
    active_php: Option<String>,
) -> Vec<(String, PhpStatus, bool, bool, bool)> {
    let active_short = active_php
        .as_deref()
        .map(|v| v.splitn(3, '.').take(2).collect::<Vec<_>>().join("."));

    let mut results = Vec::new();

    for meta in PHP_VERSIONS {
        let installed = {
            let mut found = false;
            for bin in meta.binaries {
                if tokio::fs::metadata(format!("/usr/bin/{}", bin))
                    .await
                    .is_ok()
                {
                    found = true;
                    break;
                }
            }
            found
        };

        let status = if installed {
            PhpStatus::Installed
        } else {
            let avail = Command::new("apt-cache")
                .args(["show", meta.apt_pkg])
                .output()
                .await
                .map(|o| o.status.success())
                .unwrap_or(false);
            if avail {
                PhpStatus::Available
            } else {
                PhpStatus::Unknown
            }
        };

        let is_active = active_short.as_deref() == Some(meta.version);

        let (mod_available, mod_enabled) = {
            let primary_load = format!("/etc/apache2/mods-available/{}.load", meta.mod_name);
            let primary_enable = format!("/etc/apache2/mods-enabled/{}.load", meta.mod_name);

            if tokio::fs::metadata(&primary_load).await.is_ok() {
                let enabled = tokio::fs::metadata(&primary_enable).await.is_ok();
                (true, enabled)
            } else if meta.version == "5.6" {
                let legacy_load = "/etc/apache2/mods-available/php5.load";
                let legacy_enable = "/etc/apache2/mods-enabled/php5.load";
                let avail = tokio::fs::metadata(legacy_load).await.is_ok();
                let enabled = tokio::fs::metadata(legacy_enable).await.is_ok();
                (avail, enabled)
            } else {
                (false, false)
            }
        };

        results.push((
            meta.version.to_string(),
            status,
            is_active,
            mod_available,
            mod_enabled,
        ));
    }

    results
}

pub async fn scan_apache_modules() -> Vec<ApacheModule> {
    let avail_dir = "/etc/apache2/mods-available";
    let enabled_dir = "/etc/apache2/mods-enabled";
    let mut names: Vec<String> = Vec::new();
    if let Ok(mut dir) = tokio::fs::read_dir(avail_dir).await {
        while let Ok(Some(entry)) = dir.next_entry().await {
            let fname = entry.file_name().to_string_lossy().to_string();
            if let Some(name) = fname.strip_suffix(".load") {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    let mut results = Vec::new();
    for name in names {
        let enabled = tokio::fs::metadata(format!("{}/{}.load", enabled_dir, name))
            .await
            .is_ok();
        results.push(ApacheModule { name, enabled });
    }
    results
}

pub async fn scan_php_extensions(active_ver: Option<String>) -> Vec<(String, bool)> {
    let ext_names = [
        "curl", "gd", "mbstring", "xml", "zip", "mysql", "pgsql", "redis", "intl", "bcmath",
        "soap", "imagick", "xdebug", "sqlite3",
    ];
    let mut results = Vec::new();
    for name in &ext_names {
        let pkg = match &active_ver {
            Some(ver) => format!("php{}-{}", ver, name),
            None => format!("php-{}", name),
        };
        let output = tokio::process::Command::new("dpkg")
            .args(["-l", &pkg])
            .output()
            .await;
        let installed = match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|l| l.starts_with("ii")),
            Err(_) => false,
        };
        results.push((name.to_string(), installed));
    }
    results
}

pub async fn toggle_apache_module(
    name: String,
    enable: bool,
    password: String,
) -> (bool, String, String, bool) {
    let cmd = if enable { "a2enmod" } else { "a2dismod" };
    match sudo_cmd_with_password(&password, &[cmd, &name]).await {
        Ok(_) => {
            let _ = sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"]).await;
            (
                true,
                format!(
                    "mod_{} {} — Apache reloaded",
                    name,
                    if enable { "enabled" } else { "disabled" }
                ),
                name,
                enable,
            )
        }
        Err(e) => (false, format!("Failed: {}", e), name, enable),
    }
}

pub async fn apt_php_op(version: String, install: bool, password: String) -> (bool, String) {
    let op = if install { "install" } else { "remove" };

    if let Err(e) = sudo_cmd_with_password(&password, &["apt-get", "update"]).await {
        return (false, format!("Failed to update package lists: {}", e));
    }

    let pkg = if version == "5.6" {
        if install {
            format!(
                "php5.6 php5.6-cli php5.6-common php5.6-mysql \
                 php5.6-xml php5.6-mbstring php5.6-curl"
            )
        } else {
            format!("php5.6 php5.6-*")
        }
    } else if install {
        format!(
            "php{0} php{0}-cli php{0}-common php{0}-mysql php{0}-xml php{0}-mbstring",
            version
        )
    } else {
        format!("php{0} php{0}-*", version)
    };

    let full_cmd = format!("DEBIAN_FRONTEND=noninteractive apt-get -y {} {}", op, pkg);

    match sudo_cmd_with_password(&password, &["sh", "-c", &full_cmd]).await {
        Ok(output) => {
            if output.contains("not found") || output.contains("Unable to locate") {
                if version == "5.6" {
                    (
                        false,
                        format!(
                            "PHP 5.6 not found in repositories.\n\
                         Add the ondrej/php PPA first:\n\
                         sudo add-apt-repository ppa:ondrej/php\n\
                         sudo apt-get update"
                        ),
                    )
                } else {
                    (false, format!("PHP {} not found in repositories.", version))
                }
            } else {
                (true, format!("PHP {} {}ed successfully", version, op))
            }
        }
        Err(e) => (false, format!("PHP {} {} failed: {}", version, op, e)),
    }
}

pub async fn apt_package_op(package: String, install: bool, password: String) -> (bool, String) {
    let args = if install {
        vec!["apt-get", "install", "-y", &package]
    } else {
        vec!["apt-get", "remove", "-y", &package]
    };
    match sudo_cmd_with_password(&password, &args).await {
        Ok(_) => (
            true,
            format!(
                "{} {}d successfully",
                package,
                if install { "installe" } else { "remove" }
            ),
        ),
        Err(e) => (false, format!("Failed: {}", e)),
    }
}

pub async fn switch_php(version: String, password: String) -> (bool, String) {
    let bin = if version == "5.6" {
        if std::path::Path::new("/usr/bin/php5.6").exists() {
            "/usr/bin/php5.6".to_string()
        } else {
            "/usr/bin/php5".to_string()
        }
    } else {
        format!("/usr/bin/php{}", version.trim_start_matches("php"))
    };

    match sudo_cmd_with_password(&password, &["update-alternatives", "--set", "php", &bin]).await {
        Ok(_) => (true, format!("Switched to PHP {}", version)),
        Err(e) => (false, e),
    }
}
