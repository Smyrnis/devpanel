use crate::core::dry_run;

pub async fn validate_sudo_password(password: String) -> bool {
    if dry_run::active() {
        dry_run::log("validate_sudo_password - auto-passing in dry-run mode");
        return !password.is_empty();
    }
    use tokio::io::AsyncWriteExt;
    let result = tokio::process::Command::new("sudo")
        .args(["-S", "-k", "true"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    match result {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(format!("{}\n", password).as_bytes()).await;
            }
            child.wait().await.map(|s| s.success()).unwrap_or(false)
        }
        Err(_) => false,
    }
}

pub async fn sudo_cmd_with_password(password: &str, args: &[&str]) -> Result<String, String> {
    if dry_run::active() {
        let cmd_str = args.join(" ");
        dry_run::log(&format!("sudo_cmd_with_password: sudo {}", cmd_str));
        return Ok(format!("[dry-run] would run: sudo {}", cmd_str));
    }

    use tokio::io::AsyncWriteExt;
    let mut child = tokio::process::Command::new("sudo")
        .arg("-S")
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .await
            .map_err(|e| e.to_string())?;
    }

    let output = child.wait_with_output().await.map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(clean_sudo_stderr(&output.stderr, &output.status))
    }
}

pub async fn sudo_tee_append_with_password(
    password: &str,
    path: &str,
    content: &str,
) -> Result<(), String> {
    if dry_run::active() {
        let preview: String = content.chars().take(120).collect();
        dry_run::log(&format!(
            "sudo_tee_append_with_password: would append {} bytes to {}\n  preview: {}{}",
            content.len(),
            path,
            preview,
            if content.len() > 120 { "..." } else { "" }
        ));
        return Ok(());
    }

    use tokio::io::AsyncWriteExt;
    let script = format!("tee -a {}", shell_escape(path));
    let mut child = tokio::process::Command::new("sudo")
        .args(["-S", "sh", "-c", &script])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        stdin
            .write_all(content.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
    }

    let output = child.wait_with_output().await.map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "sudo tee failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn clean_sudo_stderr(stderr: &[u8], status: &std::process::ExitStatus) -> String {
    let stderr = String::from_utf8_lossy(stderr).to_string();
    let clean = stderr
        .lines()
        .filter(|l| !l.trim_start().starts_with("[sudo]") && !l.contains("password for"))
        .collect::<Vec<_>>()
        .join("\n");
    if clean.trim().is_empty() {
        format!("sudo exited with status {}", status)
    } else {
        clean
    }
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
