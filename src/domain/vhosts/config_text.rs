use super::VHostEntry;
use super::certs::{cert_paths_for_server, server_slug};

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
        "# DevPanel managed VirtualHosts\n# Managed by DevPanel - use the UI to add/edit/remove entries.\n\n",
    );
    for e in entries {
        let sn = e.server_name.trim_end_matches('/');
        let slug = server_slug(sn);
        let (cert_file, key_file) = cert_paths_for_server(sn);
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

fn extract_directive_value(line: &str) -> String {
    line.split_whitespace().nth(1).unwrap_or("").to_string()
}

fn extract_php_version_from_sethandler(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let prefix = "x-httpd-php";
    let pos = lower.find(prefix)?;
    let ver = line[pos + prefix.len()..].trim().to_string();
    if ver.is_empty() { None } else { Some(ver) }
}
