// src/tabs/tools/backend.rs — PHP/Apache/package operations

use super::{ApacheModule, PhpStatus};
use crate::sudo_prompt::sudo_cmd_with_password;
use tokio::process::Command;

pub async fn scan_php_versions(
    active_php: Option<String>,
) -> Vec<(String, PhpStatus, bool, bool, bool)> {
    let active_short = active_php.as_deref()
        .map(|v| v.splitn(3, '.').take(2).collect::<Vec<_>>().join("."));
    let mut results = Vec::new();
    for ver in &["7.4", "8.0", "8.1", "8.2", "8.3", "8.4"] {
        let installed = tokio::fs::metadata(format!("/usr/bin/php{}", ver)).await.is_ok();
        let status = if installed {
            PhpStatus::Installed
        } else {
            let avail = Command::new("apt-cache")
                .args(["show", &format!("php{}", ver)])
                .output().await
                .map(|o| o.status.success())
                .unwrap_or(false);
            if avail { PhpStatus::Available } else { PhpStatus::Unknown }
        };
        let is_active   = active_short.as_deref() == Some(ver);
        let mod_name    = format!("php{}", ver);
        let mod_available = tokio::fs::metadata(
            format!("/etc/apache2/mods-available/{}.load", mod_name)).await.is_ok();
        let mod_enabled = tokio::fs::metadata(
            format!("/etc/apache2/mods-enabled/{}.load",   mod_name)).await.is_ok();
        results.push((ver.to_string(), status, is_active, mod_available, mod_enabled));
    }
    results
}

pub async fn scan_apache_modules() -> Vec<ApacheModule> {
    let avail_dir   = "/etc/apache2/mods-available";
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
        let enabled = tokio::fs::metadata(format!("{}/{}.load", enabled_dir, name)).await.is_ok();
        results.push(ApacheModule { name, enabled });
    }
    results
}

pub async fn scan_php_extensions(active_ver: Option<String>) -> Vec<(String, bool)> {
    let ext_names = [
        "curl", "gd", "mbstring", "xml", "zip", "mysql", "pgsql",
        "redis", "intl", "bcmath", "soap", "imagick", "xdebug", "sqlite3",
    ];
    let mut results = Vec::new();
    for name in &ext_names {
        let pkg = match &active_ver {
            Some(ver) => format!("php{}-{}", ver, name),
            None      => format!("php-{}", name),
        };
        let output = tokio::process::Command::new("dpkg").args(["-l", &pkg]).output().await;
        let installed = match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout).lines().any(|l| l.starts_with("ii")),
            Err(_)  => false,
        };
        results.push((name.to_string(), installed));
    }
    results
}

pub async fn toggle_apache_module(
    name:     String,
    enable:   bool,
    password: String,
) -> (bool, String, String, bool) {
    let cmd = if enable { "a2enmod" } else { "a2dismod" };
    match sudo_cmd_with_password(&password, &[cmd, &name]).await {
        Ok(_) => {
            let _ = sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"]).await;
            (true, format!("mod_{} {} — Apache reloaded", name,
                if enable { "enabled" } else { "disabled" }), name, enable)
        }
        Err(e) => (false, format!("Failed: {}", e), name, enable),
    }
}

pub async fn apt_php_op(version: String, install: bool, password: String) -> (bool, String) {
    let op = if install { "install" } else { "remove" };
    if let Err(e) = sudo_cmd_with_password(&password, &["apt-get", "update"]).await {
        return (false, format!("Failed to update package lists: {}", e));
    }
    let pkg = if install {
        format!("php{0} php{0}-cli php{0}-common php{0}-mysql php{0}-xml php{0}-mbstring", version)
    } else {
        format!("php{0} php{0}-*", version)
    };
    let full_cmd = format!("DEBIAN_FRONTEND=noninteractive apt-get -y {} {}", op, pkg);
    match sudo_cmd_with_password(&password, &["sh", "-c", &full_cmd]).await {
        Ok(output) => {
            if output.contains("not found") || output.contains("Unable to locate") {
                (false, format!("PHP {} not found in repositories.", version))
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
        Ok(_)  => (true,  format!("{} {}d successfully", package, if install { "installe" } else { "remove" })),
        Err(e) => (false, format!("Failed: {}", e)),
    }
}

pub async fn switch_php(version: String, password: String) -> (bool, String) {
    let ver = version.trim_start_matches("php").to_string();
    let bin = format!("/usr/bin/php{}", ver);
    match sudo_cmd_with_password(&password, &["update-alternatives", "--set", "php", &bin]).await {
        Ok(_)  => (true,  format!("Switched to PHP {}", ver)),
        Err(e) => (false, e),
    }
}
