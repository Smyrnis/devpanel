use super::VHostEntry;
use crate::core::error::{DevPanelError, DevPanelResult};
use crate::core::paths;
use crate::operations::vhost;

pub async fn scan_vhosts(devpanel_conf: String) -> Vec<VHostEntry> {
    let content = tokio::fs::read_to_string(&devpanel_conf)
        .await
        .unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&content);
    if let Ok(db) = crate::core::db::DevPanelDb::open() {
        for entry in &mut entries {
            if let Ok(Some((tag, _))) = db.get_vhost_meta(&entry.server_name) {
                entry.tag = tag;
            }
        }
    }
    entries
}

pub async fn load_config_file(path: String) -> String {
    tokio::fs::read_to_string(&path).await.unwrap_or_default()
}

pub async fn save_config_file(path: String, content: String, password: String) -> DevPanelResult {
    vhost::write_config(&password, &path, &content)
        .await
        .map_err(|e| DevPanelError::Sudo(format!("Save failed: {}", e)))?;
    vhost::reload_apache(&password).await;
    Ok("Config saved and Apache reloaded".into())
}

pub async fn add_vhost(
    devpanel_conf: String,
    server_name: String,
    document_root: String,
    php_version: Option<String>,
    https_enabled: bool,
    password: String,
) -> DevPanelResult {
    let existing = tokio::fs::read_to_string(&devpanel_conf)
        .await
        .unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    let sn = server_name.trim().to_string();
    let dr = document_root.trim().to_string();

    if entries.iter().any(|e| e.server_name == sn) {
        return Err(DevPanelError::Validation(format!(
            "VirtualHost '{}' already exists",
            sn
        )));
    }
    entries.push(VHostEntry {
        server_name: sn.clone(),
        document_root: dr,
        php_version: php_version.clone(),
        https_enabled,
        tag: String::new(),
        index: entries.len(),
    });

    if https_enabled {
        ensure_mkcert_cert(&sn).await?;
    }

    vhost::write_config(&password, &devpanel_conf, &build_conf_content(&entries))
        .await
        .map_err(|e| DevPanelError::Sudo(format!("Write failed: {}", e)))?;

    let hosts = tokio::fs::read_to_string(paths::HOSTS_FILE)
        .await
        .unwrap_or_default();
    if !hosts.contains(&sn) {
        let _ = vhost::append_host(&password, &sn).await;
    }
    vhost::reload_apache(&password).await;

    let php_note = php_version
        .map(|v| format!(" (PHP {})", v))
        .unwrap_or_default();
    Ok(format!(
        "VirtualHost '{}'{} created and Apache reloaded",
        sn, php_note
    ))
}

pub async fn edit_vhost(
    devpanel_conf: String,
    index: usize,
    server_name: String,
    document_root: String,
    php_version: Option<String>,
    https_enabled: bool,
    password: String,
) -> DevPanelResult {
    let existing = tokio::fs::read_to_string(&devpanel_conf)
        .await
        .unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    if index >= entries.len() {
        return Err(DevPanelError::Validation("Index out of range".into()));
    }

    let old_sn = entries[index].server_name.clone();
    let new_sn = server_name.trim().to_string();
    entries[index].server_name = new_sn.clone();
    entries[index].document_root = document_root.trim().to_string();
    entries[index].php_version = php_version.clone();
    entries[index].https_enabled = https_enabled;

    if https_enabled {
        ensure_mkcert_cert(&new_sn).await?;
    }

    vhost::write_config(&password, &devpanel_conf, &build_conf_content(&entries))
        .await
        .map_err(|e| DevPanelError::Sudo(format!("Write failed: {}", e)))?;

    if old_sn != new_sn {
        let hosts = tokio::fs::read_to_string(paths::HOSTS_FILE)
            .await
            .unwrap_or_default();
        if !hosts.contains(&new_sn) {
            let _ = vhost::append_host(&password, &new_sn).await;
        }
    }
    vhost::reload_apache(&password).await;

    let php_note = php_version
        .map(|v| format!(" (PHP {})", v))
        .unwrap_or_default();
    Ok(format!("VirtualHost '{}'{} updated", new_sn, php_note))
}

pub async fn delete_vhost(devpanel_conf: String, index: usize, password: String) -> DevPanelResult {
    let existing = tokio::fs::read_to_string(&devpanel_conf)
        .await
        .unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    if index >= entries.len() {
        return Err(DevPanelError::Validation("Index out of range".into()));
    }

    let removed = entries[index].server_name.clone();
    entries.remove(index);
    for (i, e) in entries.iter_mut().enumerate() {
        e.index = i;
    }

    vhost::write_config(&password, &devpanel_conf, &build_conf_content(&entries))
        .await
        .map_err(|e| DevPanelError::Sudo(format!("Write failed: {}", e)))?;
    vhost::reload_apache(&password).await;
    Ok(format!("VirtualHost '{}' removed", removed))
}

pub async fn bulk_delete_vhosts(
    devpanel_conf: String,
    indexes: Vec<usize>,
    password: String,
) -> DevPanelResult {
    let existing = tokio::fs::read_to_string(&devpanel_conf)
        .await
        .unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    let mut sorted = indexes;
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.iter().any(|idx| *idx >= entries.len()) {
        return Err(DevPanelError::Validation(
            "One or more selected VirtualHosts no longer exist".into(),
        ));
    }
    for idx in sorted.iter().rev() {
        entries.remove(*idx);
    }
    for (i, e) in entries.iter_mut().enumerate() {
        e.index = i;
    }
    vhost::write_config(&password, &devpanel_conf, &build_conf_content(&entries))
        .await
        .map_err(|e| DevPanelError::Sudo(format!("Write failed: {}", e)))?;
    vhost::reload_apache(&password).await;
    Ok(format!("Removed {} VirtualHost(s)", sorted.len()))
}

pub async fn toggle_https(devpanel_conf: String, index: usize, password: String) -> DevPanelResult {
    let existing = tokio::fs::read_to_string(&devpanel_conf)
        .await
        .unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    if index >= entries.len() {
        return Err(DevPanelError::Validation("Index out of range".into()));
    }
    entries[index].https_enabled = !entries[index].https_enabled;
    let server_name = entries[index].server_name.clone();
    if entries[index].https_enabled {
        ensure_mkcert_cert(&server_name).await?;
    }
    vhost::write_config(&password, &devpanel_conf, &build_conf_content(&entries))
        .await
        .map_err(|e| DevPanelError::Sudo(format!("Write failed: {}", e)))?;
    vhost::reload_apache(&password).await;
    let state = if entries[index].https_enabled {
        "enabled"
    } else {
        "disabled"
    };
    Ok(format!("HTTPS {} for {}", state, server_name))
}

pub fn parse_vhosts_from_content(content: &str) -> Vec<VHostEntry> {
    let mut entries = Vec::new();
    let mut idx = 0usize;
    let mut in_block = false;
    let mut block_is_https = false;
    let mut sn = String::new();
    let mut dr = String::new();
    let mut php_ver: Option<String> = None;

    for line in content.lines() {
        let t = line.trim().to_lowercase();
        if t.starts_with("<virtualhost") {
            in_block = true;
            block_is_https = t.contains(":443");
            sn.clear();
            dr.clear();
            php_ver = None;
        } else if t.starts_with("</virtualhost>") && in_block {
            if !sn.is_empty() && !block_is_https {
                let https_enabled = content
                    .to_lowercase()
                    .contains(&format!("servername {}", sn.to_lowercase()))
                    && content.to_lowercase().contains("<virtualhost *:443>");
                entries.push(VHostEntry {
                    server_name: sn.clone(),
                    document_root: dr.clone(),
                    php_version: php_ver.clone(),
                    https_enabled,
                    tag: String::new(),
                    index: idx,
                });
                idx += 1;
            }
            in_block = false;
        } else if in_block {
            let orig = line.trim();
            let lower = orig.to_lowercase();
            if lower.starts_with("servername") {
                sn = extract_directive_value(orig);
            }
            if lower.starts_with("documentroot") {
                dr = extract_directive_value(orig);
            }
            if lower.contains("sethandler") && lower.contains("x-httpd-php") {
                php_ver = extract_php_version_from_sethandler(orig);
            }
        }
    }
    entries
}

pub fn build_conf_content(entries: &[VHostEntry]) -> String {
    let mut out = String::from(
        "# DevPanel managed VirtualHosts\n# Managed by DevPanel — use the UI to add/edit/remove entries.\n\n",
    );
    for e in entries {
        let sn = e.server_name.trim_end_matches('/');
        let slug = sn.replace('.', "_");
        let cert_base = cert_base_dir();
        let cert_file = cert_base.join(format!("{}.pem", slug));
        let key_file = cert_base.join(format!("{}-key.pem", slug));
        let set_handler = match &e.php_version {
            Some(ver) => format!("\n        SetHandler application/x-httpd-php{}", ver),
            None => String::new(),
        };

        out.push_str(&format!(
            "<VirtualHost *:80>\n\
             \tServerName {sn}\n\
             \tServerAlias www.{sn}\n\
             \tDocumentRoot {dr}\n\n\
             \t<Directory {dr}>\n\
             \t\tOptions Indexes FollowSymLinks\n\
             \t\tAllowOverride All\n\
             \t\tRequire all granted{set_handler}\n\
             \t</Directory>\n\n\
             \tErrorLog ${{APACHE_LOG_DIR}}/{slug}_error.log\n\
             \tCustomLog ${{APACHE_LOG_DIR}}/{slug}_access.log combined\n\
             </VirtualHost>\n\n",
            sn = sn,
            dr = e.document_root,
            slug = slug,
            set_handler = set_handler,
        ));
        if e.https_enabled {
            out.push_str(&format!(
                "<VirtualHost *:443>\n\
                 \tServerName {sn}\n\
                 \tServerAlias www.{sn}\n\
                 \tDocumentRoot {dr}\n\n\
                 \tSSLEngine on\n\
                 \tSSLCertificateFile {cert}\n\
                 \tSSLCertificateKeyFile {key}\n\n\
                 \t<Directory {dr}>\n\
                 \t\tOptions Indexes FollowSymLinks\n\
                 \t\tAllowOverride All\n\
                 \t\tRequire all granted{set_handler}\n\
                 \t</Directory>\n\n\
                 \tErrorLog ${{APACHE_LOG_DIR}}/{slug}_ssl_error.log\n\
                 \tCustomLog ${{APACHE_LOG_DIR}}/{slug}_ssl_access.log combined\n\
                 </VirtualHost>\n\n",
                sn = sn,
                dr = e.document_root,
                slug = slug,
                cert = cert_file.display(),
                key = key_file.display(),
                set_handler = set_handler,
            ));
        }
    }
    out
}

fn cert_base_dir() -> std::path::PathBuf {
    crate::core::system::get_home()
        .join(".local")
        .join("share")
        .join("devpanel")
        .join("certs")
}

async fn ensure_mkcert_cert(server_name: &str) -> DevPanelResult<()> {
    let base = cert_base_dir();
    tokio::fs::create_dir_all(&base).await.map_err(|e| {
        DevPanelError::Io(std::io::Error::new(
            e.kind(),
            format!("Could not create certificate directory: {}", e),
        ))
    })?;
    let slug = server_name.trim_end_matches('/').replace('.', "_");
    let cert = base.join(format!("{}.pem", slug));
    let key = base.join(format!("{}-key.pem", slug));
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

fn extract_directive_value(line: &str) -> String {
    line.split_whitespace().nth(1).unwrap_or("").to_string()
}

#[allow(dead_code)]
fn parse_directive(content: &str, directive: &str) -> String {
    for line in content.lines() {
        let t = line.trim();
        if t.to_lowercase().starts_with(&directive.to_lowercase()) {
            return extract_directive_value(t);
        }
    }
    String::new()
}

fn extract_php_version_from_sethandler(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let prefix = "x-httpd-php";
    let pos = lower.find(prefix)?;
    let ver = line[pos + prefix.len()..].trim().to_string();
    if ver.is_empty() { None } else { Some(ver) }
}
