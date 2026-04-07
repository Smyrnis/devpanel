use crate::messages::{DashboardMessage, Message};
use tokio::process::Command;

pub async fn probe_services() -> Message {
    let apache = service_active("apache2").await;
    let mysql = service_active("mysql").await || service_active("mariadb").await;
    let (php, php_versions) = detect_php().await;
    Message::Dashboard(DashboardMessage::StatusRefreshed {
        apache,
        mysql,
        php,
        php_versions,
    })
}

pub async fn service_active(name: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", name])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

pub async fn detect_php() -> (Option<String>, Vec<String>) {
    let mut versions = Vec::new();
    if let Ok(mut dir) = tokio::fs::read_dir("/usr/bin").await {
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

pub fn detect_distro() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }
    "Linux".to_string()
}
