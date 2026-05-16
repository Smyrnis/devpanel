use super::get_home;

pub async fn copy_to_clipboard(text: String) {
    if try_xclip(&text).await {
        return;
    }
    if try_wl_copy(&text).await {
        return;
    }
    if try_xsel(&text).await {
        return;
    }
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
        let ok = stdin.write_all(text.as_bytes()).await.is_ok() && stdin.flush().await.is_ok();
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
