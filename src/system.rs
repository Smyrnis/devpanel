// src/system.rs — OS-level helpers: process launch, terminal detection, clipboard

use std::path::PathBuf;

// ── Path helpers ──────────────────────────────────────────────────────────

pub fn get_home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
}

// ── URL / file openers ────────────────────────────────────────────────────

pub fn xdg_open(path: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}

pub fn open_url(url: &str) -> std::io::Result<()> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

pub fn open_php_ini(active_php: &Option<String>) -> std::io::Result<()> {
    if let Some(version) = active_php {
        let short = version.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
        let cli_ini    = format!("/etc/php/{}/cli/php.ini", short);
        let apache_ini = format!("/etc/php/{}/apache2/php.ini", short);
        if std::path::Path::new(&cli_ini).exists() {
            return xdg_open(&cli_ini);
        }
        if std::path::Path::new(&apache_ini).exists() {
            return xdg_open(&apache_ini);
        }
    }
    xdg_open("/etc/php")
}

// ── Terminal helpers ──────────────────────────────────────────────────────

pub fn open_terminal_at(path: &str) {
    let Some(term) = find_terminal() else { return };
    let cd_cmd = format!("cd {} && exec bash", shell_quote(path));
    let result = match term.as_str() {
        "gnome-terminal" => std::process::Command::new("gnome-terminal")
            .arg("--working-directory").arg(path).spawn(),
        "xfce4-terminal" => std::process::Command::new("xfce4-terminal")
            .arg("--working-directory").arg(path).spawn(),
        "konsole" => std::process::Command::new("konsole")
            .arg("--workdir").arg(path).spawn(),
        "mate-terminal" => std::process::Command::new("mate-terminal")
            .arg("--working-directory").arg(path).spawn(),
        "xterm" => std::process::Command::new("xterm")
            .args(["-e", "bash", "-c", &cd_cmd]).spawn(),
        "lxterminal" | "tilix" => std::process::Command::new(&term)
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&cd_cmd)))
            .spawn(),
        _ => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &cd_cmd]).spawn(),
    };
    let _ = result;
}

pub fn open_db_terminal(binary: &str, socket_auth: bool) -> Result<String, String> {
    let mysql_cmd = if socket_auth {
        format!("sudo {} -u root", binary)
    } else {
        format!("sudo {} -u root -p", binary)
    };
    let inner = format!(
        "{cmd}; printf '\\n\\033[0;33m--- session ended, press Enter to close ---\\033[0m\\n'; read _",
        cmd = mysql_cmd,
    );
    let term = find_terminal().ok_or_else(|| {
        "No terminal emulator found. Install gnome-terminal, xterm, konsole, or xfce4-terminal."
            .to_string()
    })?;
    let result = match term.as_str() {
        "gnome-terminal" | "mate-terminal" => std::process::Command::new(&term)
            .args(["--", "bash", "-c", &inner]).spawn(),
        "xterm" | "konsole" => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &inner]).spawn(),
        "xfce4-terminal" => std::process::Command::new("xfce4-terminal")
            .arg("--command")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        "tilix" => std::process::Command::new("tilix")
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        "lxterminal" => std::process::Command::new("lxterminal")
            .arg("-e")
            .arg(format!("bash -c {}", shell_quote(&inner)))
            .spawn(),
        "x-terminal-emulator" => std::process::Command::new("x-terminal-emulator")
            .args(["-e", "bash", "-c", &inner]).spawn(),
        _ => std::process::Command::new(&term)
            .args(["-e", "bash", "-c", &inner]).spawn(),
    };
    match result {
        Ok(_)  => Ok(format!("Launched '{}' in {}", mysql_cmd, term)),
        Err(e) => Err(format!("Failed to open {}: {}", term, e)),
    }
}

/// Finds an available terminal emulator by walking $PATH directly,
/// avoiding the overhead of spawning a `which` subprocess per candidate.
fn find_terminal() -> Option<String> {
    const CANDIDATES: &[&str] = &[
        "gnome-terminal",
        "xfce4-terminal",
        "konsole",
        "tilix",
        "mate-terminal",
        "lxterminal",
        "xterm",
        "x-terminal-emulator",
    ];
    for &t in CANDIDATES {
        if binary_exists(t) {
            return Some(t.to_string());
        }
    }
    // Last resort: absolute path check
    if std::path::Path::new("/usr/bin/xterm").exists() {
        return Some("xterm".to_string());
    }
    None
}

/// Checks whether a binary exists on PATH without spawning a subprocess.
fn binary_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

/// Wraps a string in single quotes for shell, escaping any embedded single quotes.
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

// ── Async service command ─────────────────────────────────────────────────

/// Runs `systemctl <action> <service>` via sudo and returns a ServiceResult message.
/// Takes owned Strings — no Box::leak required.
pub async fn run_service_cmd(
    service:  String,
    action:   String,
    password: String,
) -> crate::app::Message {
    let result = crate::sudo_prompt::sudo_cmd_with_password(
        &password,
        &["systemctl", &action, &service],
    )
    .await;
    crate::app::Message::ServiceResult {
        service,
        action,
        success: result.is_ok(),
        output:  result.err().unwrap_or_default(),
    }
}

pub async fn ssh_add(path: String) -> (bool, String) {
    match tokio::process::Command::new("ssh-add").arg(&path).output().await {
        Ok(o) if o.status.success() => (true, format!("Key added: {}", path)),
        Ok(o)  => (false, String::from_utf8_lossy(&o.stderr).to_string()),
        Err(e) => (false, e.to_string()),
    }
}

// ── Clipboard helpers ─────────────────────────────────────────────────────

pub async fn copy_to_clipboard(text: String) {
    if try_xclip(&text).await   { return; }
    if try_wl_copy(&text).await { return; }
    if try_xsel(&text).await    { return; }
    fallback_script_file(&text).await;
}

async fn try_xclip(text: &str) -> bool {
    let mut cmd = tokio::process::Command::new("xclip");
    cmd.args(["-selection", "clipboard"]);
    pipe_to_cmd(cmd, text).await
}

async fn try_wl_copy(text: &str) -> bool {
    pipe_to_cmd(tokio::process::Command::new("wl-copy"), text).await
}

async fn try_xsel(text: &str) -> bool {
    let mut cmd = tokio::process::Command::new("xsel");
    cmd.args(["-b", "-i"]);
    pipe_to_cmd(cmd, text).await
}

/// Shared stdin-pipe logic for all clipboard tools.
async fn pipe_to_cmd(mut cmd: tokio::process::Command, text: &str) -> bool {
    use tokio::io::AsyncWriteExt;
    let Ok(mut child) = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        return false;
    };
    if let Some(mut stdin) = child.stdin.take() {
        let ok = stdin.write_all(text.as_bytes()).await.is_ok()
            && stdin.flush().await.is_ok();
        drop(stdin);
        let _ = child.wait().await;
        ok
    } else {
        false
    }
}

async fn fallback_script_file(commands: &str) {
    let path = get_home().join(".devpanel_php_install.sh");
    if tokio::fs::write(&path, format!("#!/bin/bash\n{}\n", commands))
        .await
        .is_ok()
    {
        let _ = std::process::Command::new("chmod")
            .args(["+x", path.to_string_lossy().as_ref()])
            .output();
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    }
}