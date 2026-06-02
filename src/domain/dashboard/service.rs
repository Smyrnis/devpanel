use crate::core::paths;
use crate::domain::dashboard::DashboardSnapshot;
use crate::helpers::process::service_active;
use crate::helpers::time::format_duration;
use tokio::process::Command;

pub async fn probe_services() -> DashboardSnapshot {
    let apache = service_active("apache2").await;
    let mysql = service_active("mysql").await || service_active("mariadb").await;
    let (php, php_versions) = detect_php().await;
    let apache_uptime = service_uptime("apache2").await;
    let mysql_uptime = service_uptime("mysql")
        .await
        .or(service_uptime("mariadb").await);
    let recent_failures = recent_service_failures("apache2").await;
    DashboardSnapshot {
        apache,
        mysql,
        php,
        php_versions,
        apache_uptime,
        mysql_uptime,
        recent_failures,
    }
}

pub async fn detect_php() -> (Option<String>, Vec<String>) {
    let mut versions = Vec::new();
    if let Ok(mut dir) = tokio::fs::read_dir(paths::PHP_BIN_DIR).await {
        while let Ok(Some(entry)) = dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(rest) = name.strip_prefix("php") {
                let t = rest.trim_start_matches('-');
                if t.contains('.') && t.len() <= 4 {
                    versions.push(name);
                }
            }
        }
    }
    versions.sort();
    if crate::core::dry_run::active() {
        return (None, versions);
    }

    let active = Command::new("php")
        .arg("--version")
        .output()
        .await
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).to_string();
            s.lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1))
                .map(|v| v.to_string())
        });
    (active, versions)
}

pub async fn php_info_summary() -> String {
    if crate::core::dry_run::active() {
        return "[dry-run] php -i not executed".into();
    }

    let out = Command::new("php").arg("-i").output().await;
    let Ok(out) = out else {
        return "php -i could not be executed".into();
    };
    if !out.status.success() {
        return String::from_utf8_lossy(&out.stderr).trim().to_string();
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    let keys = [
        "PHP Version",
        "System",
        "Build Date",
        "Server API",
        "Loaded Configuration File",
        "Scan this dir for additional .ini files",
        "additional .ini files parsed",
        "memory_limit",
        "post_max_size",
        "upload_max_filesize",
        "max_execution_time",
        "display_errors",
        "error_reporting",
    ];
    let mut lines = Vec::new();
    for line in raw.lines() {
        if keys.iter().any(|key| line.starts_with(key)) {
            lines.push(line.replace(" => ", ": "));
        }
    }
    if lines.is_empty() {
        "No PHP info fields found".into()
    } else {
        lines.join("\n")
    }
}

async fn service_uptime(service: &str) -> Option<String> {
    if crate::core::dry_run::active() {
        return None;
    }

    let uptime = tokio::fs::read_to_string("/proc/uptime").await.ok()?;
    let system_uptime_secs: f64 = uptime.split_whitespace().next()?.parse().ok()?;
    let out = Command::new("systemctl")
        .args([
            "show",
            service,
            "-p",
            "ActiveEnterTimestampMonotonic",
            "--value",
        ])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let active_us: f64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    if active_us <= 0.0 {
        return None;
    }
    let active_secs = active_us / 1_000_000.0;
    let elapsed = (system_uptime_secs - active_secs).max(0.0) as u64;
    Some(format_duration(elapsed))
}

async fn recent_service_failures(service: &str) -> Vec<String> {
    if crate::core::dry_run::active() {
        return Vec::new();
    }

    let out = Command::new("journalctl")
        .args(["-u", service, "-n", "20", "--no-pager"])
        .output()
        .await;
    let Ok(out) = out else { return Vec::new() };
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    combined
        .lines()
        .filter(|line| {
            let l = line.to_lowercase();
            l.contains("fail") || l.contains("error") || l.contains("denied")
        })
        .rev()
        .take(3)
        .map(|line| line.trim().to_string())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
