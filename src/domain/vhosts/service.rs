use super::VHostEntry;
use super::certs::ensure_mkcert_cert;
use super::config_text::{build_conf_content, parse_vhosts_from_content};
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
