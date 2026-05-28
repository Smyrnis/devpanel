use super::{KeyEntry, KeyType};
use crate::core::error::{DevPanelError, DevPanelResult};
use crate::helpers::time::format_unix_day;
use crate::infra::system::ssh_dir;
use tokio::process::Command;

pub async fn generate_key(
    email: String,
    key_name: String,
    key_type: KeyType,
    passphrase: String,
) -> DevPanelResult {
    let ssh_dir = ssh_dir();
    let key_name = if key_name.is_empty() {
        default_key_name(key_type)
    } else {
        key_name
    };
    let key_path = ssh_dir.join(&key_name);

    if crate::core::dry_run::active() {
        crate::core::dry_run::log_user_action(&format!(
            "ssh-keygen preview for {}",
            key_path.display()
        ));
        return Ok(format!(
            "[dry-run] Would generate key: {}",
            key_path.display()
        ));
    }

    if !ssh_dir.exists() {
        tokio::fs::create_dir_all(&ssh_dir).await?;
        let _ = Command::new("chmod")
            .args(["700", &ssh_dir.to_string_lossy()])
            .status()
            .await;
    }
    if key_path.exists() {
        return Err(DevPanelError::Validation(format!(
            "{} already exists - choose a different name",
            key_path.display()
        )));
    }
    let mut cmd = Command::new("ssh-keygen");
    match key_type {
        KeyType::Ed25519 => {
            cmd.arg("-t").arg("ed25519");
        }
        KeyType::Rsa4096 => {
            cmd.arg("-t").arg("rsa").arg("-b").arg("4096");
        }
        KeyType::Ecdsa => {
            cmd.arg("-t").arg("ecdsa").arg("-b").arg("521");
        }
    }
    let comment = if email.is_empty() {
        key_name.clone()
    } else {
        email
    };
    cmd.arg("-f")
        .arg(&key_path)
        .arg("-C")
        .arg(&comment)
        .arg("-N")
        .arg(&passphrase);
    match cmd.output().await {
        Ok(o) if o.status.success() => {
            let _ = Command::new("chmod")
                .args(["600", &key_path.to_string_lossy()])
                .status()
                .await;
            let _ = Command::new("ssh-add").arg(&key_path).output().await;
            Ok(format!("Key generated: {}", key_path.display()))
        }
        Ok(o) => Err(DevPanelError::Command(format!(
            "ssh-keygen failed: {}",
            String::from_utf8_lossy(&o.stderr)
        ))),
        Err(e) => Err(DevPanelError::Io(e)),
    }
}

pub async fn list_keys() -> Vec<KeyEntry> {
    let dir = ssh_dir();
    let mut entries = Vec::new();
    let Ok(mut dir_reader) = tokio::fs::read_dir(&dir).await else {
        return entries;
    };
    let mut files = Vec::new();
    while let Ok(Some(entry)) = dir_reader.next_entry().await {
        files.push(entry.file_name().to_string_lossy().to_string());
    }
    let loaded_fingerprints = loaded_agent_fingerprints().await;
    for fname in &files {
        if fname.ends_with(".pub")
            || matches!(
                fname.as_str(),
                "config" | "known_hosts" | "known_hosts.old" | "authorized_keys"
            )
            || fname.starts_with('.')
        {
            continue;
        }
        let path = dir.join(fname);
        let has_pub = files.contains(&format!("{}.pub", fname));
        let fingerprint = key_fingerprint(&path).await;
        let loaded_in_agent = fingerprint
            .as_deref()
            .map(|fp| loaded_fingerprints.iter().any(|loaded| loaded == fp))
            .unwrap_or(false);
        entries.push(KeyEntry {
            name: fname.clone(),
            path: path.to_string_lossy().to_string(),
            has_pub,
            fingerprint,
            created: key_created_label(&path).await,
            loaded_in_agent,
        });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

pub async fn read_public_key(path: String) -> DevPanelResult {
    let pub_path = format!("{}.pub", path);
    tokio::fs::read_to_string(&pub_path)
        .await
        .map(|s| s.trim().to_string())
        .map_err(|e| {
            DevPanelError::Io(std::io::Error::new(
                e.kind(),
                format!("Could not read {}: {}", pub_path, e),
            ))
        })
}

fn default_key_name(key_type: KeyType) -> String {
    match key_type {
        KeyType::Ed25519 => "id_ed25519".to_string(),
        KeyType::Rsa4096 => "id_rsa".to_string(),
        KeyType::Ecdsa => "id_ecdsa".to_string(),
    }
}

async fn key_fingerprint(path: &std::path::Path) -> Option<String> {
    if crate::core::dry_run::active() {
        return None;
    }

    let out = Command::new("ssh-keygen")
        .args(["-l", "-f", path.to_string_lossy().as_ref()])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout.split_whitespace().nth(1).map(|s| s.to_string())
}

async fn loaded_agent_fingerprints() -> Vec<String> {
    if crate::core::dry_run::active() {
        return Vec::new();
    }

    let out = Command::new("ssh-add").arg("-l").output().await;
    let Ok(out) = out else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1).map(|s| s.to_string()))
        .collect()
}

async fn key_created_label(path: &std::path::Path) -> Option<String> {
    let metadata = tokio::fs::metadata(path).await.ok()?;
    let time = metadata.created().or_else(|_| metadata.modified()).ok()?;
    let secs = time.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    Some(format_unix_day(secs))
}
