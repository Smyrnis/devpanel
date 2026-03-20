// src/tabs/vhosts/backend.rs — file I/O, Apache conf parsing/generation, sudo writes

use super::VHostEntry;
use tokio::io::AsyncWriteExt;

// ── Public async tasks ────────────────────────────────────────────────────

pub async fn scan_vhosts(devpanel_conf: String) -> Vec<VHostEntry> {
    let content = tokio::fs::read_to_string(&devpanel_conf).await.unwrap_or_default();
    parse_vhosts_from_content(&content)
}

pub async fn load_config_file(path: String) -> String {
    tokio::fs::read_to_string(&path).await.unwrap_or_default()
}

pub async fn save_config_file(path: String, content: String, password: String) -> (bool, String) {
    match write_conf(&path, &content, &password).await {
        Ok(_) => {
            let _ = crate::sudo_prompt::sudo_cmd_with_password(
                &password, &["systemctl", "reload", "apache2"],
            ).await;
            (true, "Config saved and Apache reloaded".into())
        }
        Err(e) => (false, format!("Save failed: {}", e)),
    }
}

pub async fn add_vhost(
    devpanel_conf: String,
    server_name:   String,
    document_root: String,
    php_version:   Option<String>,
    password:      String,
) -> (bool, String) {
    let existing = tokio::fs::read_to_string(&devpanel_conf).await.unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    let sn = server_name.trim().to_string();
    let dr = document_root.trim().to_string();

    if entries.iter().any(|e| e.server_name == sn) {
        return (false, format!("VirtualHost '{}' already exists", sn));
    }
    entries.push(VHostEntry {
        server_name:   sn.clone(),
        document_root: dr,
        php_version:   php_version.clone(),
        index:         entries.len(),
    });

    if let Err(e) = write_conf(&devpanel_conf, &build_conf_content(&entries), &password).await {
        return (false, format!("Write failed: {}", e));
    }

    let hosts = tokio::fs::read_to_string("/etc/hosts").await.unwrap_or_default();
    if !hosts.contains(&sn) {
        let _ = crate::sudo_prompt::sudo_tee_append_with_password(
            &password, "/etc/hosts", &format!("127.0.0.1    {}\n", sn),
        ).await;
    }
    let _ = crate::sudo_prompt::sudo_cmd_with_password(
        &password, &["systemctl", "reload", "apache2"],
    ).await;

    let php_note = php_version
        .map(|v| format!(" (PHP {})", v))
        .unwrap_or_default();
    (true, format!("VirtualHost '{}'{} created and Apache reloaded", sn, php_note))
}

pub async fn edit_vhost(
    devpanel_conf: String,
    index:         usize,
    server_name:   String,
    document_root: String,
    php_version:   Option<String>,
    password:      String,
) -> (bool, String) {
    let existing = tokio::fs::read_to_string(&devpanel_conf).await.unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    if index >= entries.len() { return (false, "Index out of range".into()); }

    let old_sn = entries[index].server_name.clone();
    let new_sn = server_name.trim().to_string();
    entries[index].server_name   = new_sn.clone();
    entries[index].document_root = document_root.trim().to_string();
    entries[index].php_version   = php_version.clone();

    if let Err(e) = write_conf(&devpanel_conf, &build_conf_content(&entries), &password).await {
        return (false, format!("Write failed: {}", e));
    }

    if old_sn != new_sn {
        let hosts = tokio::fs::read_to_string("/etc/hosts").await.unwrap_or_default();
        if !hosts.contains(&new_sn) {
            let _ = crate::sudo_prompt::sudo_tee_append_with_password(
                &password, "/etc/hosts", &format!("127.0.0.1    {}\n", new_sn),
            ).await;
        }
    }
    let _ = crate::sudo_prompt::sudo_cmd_with_password(
        &password, &["systemctl", "reload", "apache2"],
    ).await;

    let php_note = php_version
        .map(|v| format!(" (PHP {})", v))
        .unwrap_or_default();
    (true, format!("VirtualHost '{}'{} updated", new_sn, php_note))
}

pub async fn delete_vhost(
    devpanel_conf: String,
    index:         usize,
    password:      String,
) -> (bool, String) {
    let existing = tokio::fs::read_to_string(&devpanel_conf).await.unwrap_or_default();
    let mut entries = parse_vhosts_from_content(&existing);
    if index >= entries.len() { return (false, "Index out of range".into()); }

    let removed = entries[index].server_name.clone();
    entries.remove(index);
    for (i, e) in entries.iter_mut().enumerate() { e.index = i; }

    if let Err(e) = write_conf(&devpanel_conf, &build_conf_content(&entries), &password).await {
        return (false, format!("Write failed: {}", e));
    }
    let _ = crate::sudo_prompt::sudo_cmd_with_password(
        &password, &["systemctl", "reload", "apache2"],
    ).await;
    (true, format!("VirtualHost '{}' removed", removed))
}

// ── Parse / Build helpers ─────────────────────────────────────────────────

pub fn parse_vhosts_from_content(content: &str) -> Vec<VHostEntry> {
    let mut entries  = Vec::new();
    let mut idx      = 0usize;
    let mut in_block = false;
    let mut sn       = String::new();
    let mut dr       = String::new();
    let mut php_ver: Option<String> = None;

    for line in content.lines() {
        let t = line.trim().to_lowercase();
        if t.starts_with("<virtualhost") {
            in_block = true;
            sn.clear();
            dr.clear();
            php_ver = None;
        } else if t.starts_with("</virtualhost>") && in_block {
            if !sn.is_empty() {
                entries.push(VHostEntry {
                    server_name:   sn.clone(),
                    document_root: dr.clone(),
                    php_version:   php_ver.clone(),
                    index:         idx,
                });
                idx += 1;
            }
            in_block = false;
        } else if in_block {
            let orig = line.trim();
            let lower = orig.to_lowercase();
            if lower.starts_with("servername")   { sn = parse_directive(orig, "ServerName"); }
            if lower.starts_with("documentroot") { dr = parse_directive(orig, "DocumentRoot"); }
            // Parse:  SetHandler application/x-httpd-php8.2
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
        let sn   = e.server_name.trim_end_matches('/');
        let slug = sn.replace('.', "_");

        // Build the Directory block — inject SetHandler when a PHP version is pinned
        let set_handler = match &e.php_version {
            Some(ver) => format!(
                "\n        SetHandler application/x-httpd-php{}",
                ver
            ),
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
            sn          = sn,
            dr          = e.document_root,
            slug        = slug,
            set_handler = set_handler,
        ));
    }
    out
}

// ── Internal helpers ──────────────────────────────────────────────────────

fn parse_directive(content: &str, directive: &str) -> String {
    for line in content.lines() {
        let t = line.trim();
        if t.to_lowercase().starts_with(&directive.to_lowercase()) {
            let rest = &t[directive.len()..];
            return rest.trim().split_whitespace().next().unwrap_or("").to_string();
        }
    }
    String::new()
}

/// Extract "8.2" from "SetHandler application/x-httpd-php8.2"
fn extract_php_version_from_sethandler(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let prefix = "x-httpd-php";
    let pos = lower.find(prefix)?;
    let ver = line[pos + prefix.len()..].trim().to_string();
    if ver.is_empty() { None } else { Some(ver) }
}

async fn write_conf(path: &str, content: &str, password: &str) -> Result<(), String> {
    let mut child = tokio::process::Command::new("sudo")
        .args(["-S", "tee", path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn().map_err(|e| e.to_string())?;

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