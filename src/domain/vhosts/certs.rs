use crate::core::error::{DevPanelError, DevPanelResult};
use std::path::PathBuf;

pub(super) fn server_slug(server_name: &str) -> String {
    server_name.trim_end_matches('/').replace('.', "_")
}

pub(super) fn cert_paths_for_server(server_name: &str) -> (PathBuf, PathBuf) {
    let base = cert_base_dir();
    let slug = server_slug(server_name);
    (
        base.join(format!("{}.pem", slug)),
        base.join(format!("{}-key.pem", slug)),
    )
}

fn cert_base_dir() -> PathBuf {
    crate::infra::system::get_home()
        .join(".local")
        .join("share")
        .join("devpanel")
        .join("certs")
}

pub(super) async fn ensure_mkcert_cert(server_name: &str) -> DevPanelResult<()> {
    let base = cert_base_dir();
    let (cert, key) = cert_paths_for_server(server_name);

    if crate::core::dry_run::active() {
        crate::core::dry_run::log(&format!(
            "ensure_mkcert_cert: would create cert for {} at {} and {}",
            server_name,
            cert.display(),
            key.display()
        ));
        return Ok(());
    }

    tokio::fs::create_dir_all(&base).await.map_err(|e| {
        DevPanelError::Io(std::io::Error::new(
            e.kind(),
            format!("Could not create certificate directory: {}", e),
        ))
    })?;

    if cert.exists() && key.exists() {
        return Ok(());
    }

    let out = tokio::process::Command::new("mkcert")
        .args([
            "-cert-file",
            cert.to_string_lossy().as_ref(),
            "-key-file",
            key.to_string_lossy().as_ref(),
            server_name,
        ])
        .output()
        .await
        .map_err(|e| {
            DevPanelError::Io(std::io::Error::new(
                e.kind(),
                format!("mkcert is required for HTTPS: {}", e),
            ))
        })?;

    if out.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(DevPanelError::Apache(format!(
            "mkcert failed: {}",
            if stderr.is_empty() {
                "unknown error"
            } else {
                &stderr
            }
        )))
    }
}
