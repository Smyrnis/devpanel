use tokio::process::Command;

pub async fn command_first_line(cmd: &str, args: &[&str]) -> Option<String> {
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

pub async fn service_active(name: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", name])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

pub async fn service_exists(name: &str) -> bool {
    Command::new("systemctl")
        .args(["status", name, "--no-pager"])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}
