use tokio::io::AsyncWriteExt;

pub async fn run(password: &str, args: &[&str]) -> Result<String, String> {
    crate::core::sudo_prompt::sudo_cmd_with_password(password, args).await
}

pub async fn systemctl(password: &str, action: &str, service: &str) -> Result<String, String> {
    run(password, &["systemctl", action, service]).await
}

pub async fn append_file(password: &str, path: &str, content: &str) -> Result<(), String> {
    crate::core::sudo_prompt::sudo_tee_append_with_password(password, path, content).await
}

pub async fn write_file(password: &str, path: &str, content: &str) -> Result<(), String> {
    if crate::core::dry_run::active() {
        crate::core::dry_run::log(&format!(
            "common_sudo::write_file: would write {} bytes to {}",
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
