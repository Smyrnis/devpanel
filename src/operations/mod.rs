//! Privileged and system command operations.
//!
//! UI and app handlers should call these wrappers instead of constructing
//! `sudo -S`, `systemctl`, `apt`, or shell command strings directly.

use tokio::io::AsyncWriteExt;

pub mod apache;
pub mod mysql;
pub mod php;
pub mod tools;
pub mod vhost;

pub async fn run(password: &str, args: &[&str]) -> Result<String, String> {
    crate::infra::sudo_prompt::sudo_cmd_with_password(password, args).await
}

pub async fn systemctl(password: &str, action: &str, service: &str) -> Result<String, String> {
    run(password, &["systemctl", action, service]).await
}

pub async fn append_file(password: &str, path: &str, content: &str) -> Result<(), String> {
    crate::infra::sudo_prompt::sudo_tee_append_with_password(password, path, content).await
}

pub async fn rewrite_file_lines<F>(password: &str, path: &str, keep_line: F) -> Result<(), String>
where
    F: Fn(&str) -> bool,
{
    if crate::core::dry_run::active() {
        crate::core::dry_run::log(&format!(
            "operations::rewrite_file_lines: would rewrite filtered lines in {}",
            path
        ));
        return Ok(());
    }

    let existing = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| e.to_string())?;
    let mut rewritten = String::new();
    for line in existing.lines() {
        if keep_line(line) {
            rewritten.push_str(line);
            rewritten.push('\n');
        }
    }
    write_file(password, path, &rewritten).await
}

pub async fn write_file(password: &str, path: &str, content: &str) -> Result<(), String> {
    if crate::core::dry_run::active() {
        crate::core::dry_run::log(&format!(
            "operations::write_file: would write {} bytes to {}",
            content.len(),
            path
        ));
        return Ok(());
    }

    let mut child = tokio::process::Command::new("sudo")
        .args(["-S", "tee", path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(format!("{}\n", password).as_bytes()).await;
        let _ = stdin.write_all(content.as_bytes()).await;
    }
    let out = child.wait_with_output().await.map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}
