use crate::core::error::{DevPanelError, DevPanelResult};

pub async fn ssh_add(path: String) -> DevPanelResult {
    match tokio::process::Command::new("ssh-add")
        .arg(&path)
        .output()
        .await
    {
        Ok(o) if o.status.success() => Ok(format!("Key added: {}", path)),
        Ok(o) => Err(DevPanelError::Command(
            String::from_utf8_lossy(&o.stderr).to_string(),
        )),
        Err(e) => Err(DevPanelError::Io(e)),
    }
}
