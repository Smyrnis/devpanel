use super::{ApacheModule, InstalledTools, PhpStatus};
use crate::core::error::{DevPanelError, DevPanelResult};
use crate::core::paths;
use crate::helpers::process::{command_first_line, service_active, service_exists};
use crate::infra::system::get_home;
use crate::operations::{apache, php, tools};
use tokio::process::Command;

pub async fn scan_php_versions(
    active_php: Option<String>,
) -> Vec<(String, PhpStatus, bool, bool, bool)> {
    let active_short = active_php
        .as_deref()
        .map(|v| v.splitn(3, '.').take(2).collect::<Vec<_>>().join("."));

    let mut results = Vec::new();

    for meta in crate::core::app_config::php_versions() {
        let installed = {
            let mut found = false;
            for bin in &meta.binaries {
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
        } else if crate::core::dry_run::active() {
            PhpStatus::Unknown
        } else {
            let avail = Command::new("apt-cache")
                .args(["show", &meta.apt_package])
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

        let is_active = active_short.as_deref() == Some(meta.version.as_str());

        let (mod_available, mod_enabled) = {
            let primary_load = paths::apache_mod_available(&meta.apache_module);
            let primary_enable = paths::apache_mod_enabled(&meta.apache_module);
            if tokio::fs::metadata(&primary_load).await.is_ok() {
                let enabled = tokio::fs::metadata(&primary_enable).await.is_ok();
                (true, enabled)
            } else if let Some(legacy) = &meta.legacy_apache_module {
                let legacy_load = paths::apache_mod_available(legacy);
                let legacy_enable = paths::apache_mod_enabled(legacy);
                let available = tokio::fs::metadata(legacy_load).await.is_ok();
                let enabled = tokio::fs::metadata(legacy_enable).await.is_ok();
                (available, enabled)
            } else {
                (false, false)
            }
        };

        results.push((meta.version, status, is_active, mod_available, mod_enabled));
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
        if crate::core::dry_run::active() {
            results.push((name.to_string(), false));
            continue;
        }

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

pub async fn toggle_apache_module(name: String, enable: bool, password: String) -> DevPanelResult {
    apache::set_module_and_reload(&password, &name, enable)
        .await
        .map_err(DevPanelError::Sudo)?;
    Ok(format!(
        "mod_{} {} - Apache reloaded",
        name,
        if enable { "enabled" } else { "disabled" }
    ))
}

pub async fn apt_php_op(version: String, install: bool, password: String) -> DevPanelResult {
    let op = if install { "install" } else { "remove" };

    let output = php::apt_php_op(&password, &version, install)
        .await
        .map_err(|e| DevPanelError::Sudo(format!("PHP {} {} failed: {}", version, op, e)))?;

    if output.contains("not found") || output.contains("Unable to locate") {
        if version == "5.6" {
            Err(DevPanelError::Command("PHP 5.6 not found in repositories.\nAdd the ondrej/php PPA first:\nsudo add-apt-repository ppa:ondrej/php\nsudo apt-get update".to_string()))
        } else {
            Err(DevPanelError::Command(format!(
                "PHP {} not found in repositories.",
                version
            )))
        }
    } else {
        Ok(format!("PHP {} {}ed successfully", version, op))
    }
}

pub async fn apt_package_op(package: String, install: bool, password: String) -> DevPanelResult {
    tools::apt_package_op(&password, &package, install)
        .await
        .map_err(DevPanelError::Sudo)?;
    Ok(format!(
        "{} {}d successfully",
        package,
        if install { "installe" } else { "remove" }
    ))
}

pub async fn switch_php(version: String, password: String) -> DevPanelResult {
    php::switch_php(&password, &version)
        .await
        .map_err(DevPanelError::Sudo)?;
    Ok(format!("Switched to PHP {}", version))
}

pub async fn scan_installed_tools() -> InstalledTools {
    let composer_version = command_first_line("composer", &["--version"]).await;
    let node_version = command_first_line("node", &["--version"]).await;
    let npm_version = command_first_line("npm", &["--version"]).await;
    let nvm_available = get_home().join(".nvm/nvm.sh").exists();
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

pub async fn composer_op(update: bool, password: String) -> DevPanelResult {
    tools::composer_op(&password, update).await.map_err(|e| {
        if update {
            DevPanelError::Sudo(format!("Composer update failed: {}", e))
        } else {
            DevPanelError::Sudo(format!("Composer install failed: {}", e))
        }
    })?;
    Ok(if update {
        "Composer updated".into()
    } else {
        "Composer installed globally".into()
    })
}

pub async fn redis_service_op(action: String, password: String) -> DevPanelResult {
    let service = if service_active("redis-server").await || service_exists("redis-server").await {
        "redis-server"
    } else {
        "redis"
    };
    tools::redis_service_op(&password, &action, service)
        .await
        .map_err(|e| DevPanelError::Sudo(format!("Redis {} failed: {}", action, e)))?;
    Ok(format!("Redis {}ed", action))
}

async fn redis_memory_usage() -> Option<String> {
    if crate::core::dry_run::active() {
        return None;
    }

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
