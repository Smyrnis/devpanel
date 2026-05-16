use super::{ApacheModule, InstalledTools, PhpStatus};
use crate::core::paths;
use crate::core::sudo_prompt::sudo_cmd_with_password;
use tokio::process::Command;

struct PhpVersionMeta {
    version: &'static str,
    binaries: &'static [&'static str],
    mod_name: &'static str,
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
                if tokio::fs::metadata(format!("{}/{}", paths::PHP_BIN_DIR, bin))
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
            let primary_load = paths::apache_mod_available(meta.mod_name);
            let primary_enable = paths::apache_mod_enabled(meta.mod_name);
            if tokio::fs::metadata(&primary_load).await.is_ok() {
                let enabled = tokio::fs::metadata(&primary_enable).await.is_ok();
                (true, enabled)
            } else if meta.version == "5.6" {
                let legacy_load = paths::apache_mod_available("php5");
                let legacy_enable = paths::apache_mod_enabled("php5");
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
    let avail_dir = paths::APACHE_MODS_AVAILABLE;
    let enabled_dir = paths::APACHE_MODS_ENABLED;
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
            "php5.6 php5.6-cli php5.6-common php5.6-mysql php5.6-xml php5.6-mbstring php5.6-curl"
                .to_string()
        } else {
            "php5.6 php5.6-*".to_string()
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
                    (false, "PHP 5.6 not found in repositories.\nAdd the ondrej/php PPA first:\nsudo add-apt-repository ppa:ondrej/php\nsudo apt-get update".to_string())
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
        if std::path::Path::new(&paths::php_binary("5.6")).exists() {
            paths::php_binary("5.6")
        } else {
            paths::php_binary("5")
        }
    } else {
        paths::php_binary(&version)
    };

    match sudo_cmd_with_password(&password, &["update-alternatives", "--set", "php", &bin]).await {
        Ok(_) => (true, format!("Switched to PHP {}", version)),
        Err(e) => (false, e),
    }
}

pub async fn scan_installed_tools() -> InstalledTools {
    let composer_version = command_first_line("composer", &["--version"]).await;
    let node_version = command_first_line("node", &["--version"]).await;
    let npm_version = command_first_line("npm", &["--version"]).await;
    let nvm_available = std::env::var("HOME")
        .map(|home| std::path::Path::new(&home).join(".nvm/nvm.sh").exists())
        .unwrap_or(false);
    let redis_installed = command_first_line("redis-server", &["--version"])
        .await
        .is_some();
    let redis_running = service_active("redis-server").await || service_active("redis").await;
    let redis_memory = if redis_installed {
        redis_memory_usage().await
    } else {
        None
    };

    InstalledTools {
        composer_version,
        node_version,
        npm_version,
        nvm_available,
        redis_installed,
        redis_running,
        redis_memory,
    }
}

pub async fn composer_op(update: bool, password: String) -> (bool, String) {
    if update {
        return match sudo_cmd_with_password(&password, &["composer", "self-update"]).await {
            Ok(_) => (true, "Composer updated".into()),
            Err(e) => (false, format!("Composer update failed: {}", e)),
        };
    }

    let cmd = format!(
        "php -r \"copy('https://getcomposer.org/installer', '/tmp/composer-setup.php');\" \
         && php /tmp/composer-setup.php --install-dir={} --filename=composer \
         && rm -f /tmp/composer-setup.php",
        paths::COMPOSER_INSTALL_DIR,
    );
    match sudo_cmd_with_password(&password, &["sh", "-c", &cmd]).await {
        Ok(_) => (true, "Composer installed globally".into()),
        Err(e) => (false, format!("Composer install failed: {}", e)),
    }
}

pub async fn redis_service_op(action: String, password: String) -> (bool, String) {
    let service = if service_active("redis-server").await || service_exists("redis-server").await {
        "redis-server"
    } else {
        "redis"
    };
    match sudo_cmd_with_password(&password, &["systemctl", &action, service]).await {
        Ok(_) => (true, format!("Redis {}ed", action)),
        Err(e) => (false, format!("Redis {} failed: {}", action, e)),
    }
}

async fn command_first_line(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().await.ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

async fn service_active(name: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", name])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn service_exists(name: &str) -> bool {
    Command::new("systemctl")
        .args(["status", name, "--no-pager"])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn redis_memory_usage() -> Option<String> {
    let out = Command::new("redis-cli")
        .args(["info", "memory"])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|line| {
            line.strip_prefix("used_memory_human:")
                .map(|s| s.trim().to_string())
        })
}
