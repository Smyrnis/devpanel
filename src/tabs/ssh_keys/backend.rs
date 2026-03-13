// src/tabs/ssh_keys/backend.rs — SSH key generation and ~/.ssh directory listing

use super::{KeyEntry, KeyType};
use tokio::process::Command;

pub async fn generate_key(
    email:      String,
    key_name:   String,
    key_type:   KeyType,
    passphrase: String,
) -> (bool, String) {
    let ssh_dir = ssh_dir();
    if !ssh_dir.exists() {
        if let Err(e) = tokio::fs::create_dir_all(&ssh_dir).await {
            return (false, format!("Could not create ~/.ssh: {}", e));
        }
        let _ = Command::new("chmod").args(["700", &ssh_dir.to_string_lossy()]).status().await;
    }
    let key_name = if key_name.is_empty() { default_key_name(key_type) } else { key_name };
    let key_path = ssh_dir.join(&key_name);
    if key_path.exists() {
        return (false, format!("{} already exists — choose a different name", key_path.display()));
    }
    let mut cmd = Command::new("ssh-keygen");
    match key_type {
        KeyType::Ed25519 => { cmd.arg("-t").arg("ed25519"); }
        KeyType::Rsa4096 => { cmd.arg("-t").arg("rsa").arg("-b").arg("4096"); }
        KeyType::Ecdsa   => { cmd.arg("-t").arg("ecdsa").arg("-b").arg("521"); }
    }
    let comment = if email.is_empty() { key_name.clone() } else { email };
    cmd.arg("-f").arg(&key_path)
       .arg("-C").arg(&comment)
       .arg("-N").arg(&passphrase);
    match cmd.output().await {
        Ok(o) if o.status.success() => {
            let _ = Command::new("chmod").args(["600", &key_path.to_string_lossy()]).status().await;
            let _ = Command::new("ssh-add").arg(&key_path).output().await;
            (true, format!("Key generated: {}", key_path.display()))
        }
        Ok(o)  => (false, format!("ssh-keygen failed: {}", String::from_utf8_lossy(&o.stderr))),
        Err(e) => (false, format!("ssh-keygen not found: {}", e)),
    }
}

pub async fn list_keys() -> Vec<KeyEntry> {
    let dir = ssh_dir();
    let mut entries = Vec::new();
    let Ok(mut dir_reader) = tokio::fs::read_dir(&dir).await else { return entries; };
    let mut files = Vec::new();
    while let Ok(Some(entry)) = dir_reader.next_entry().await {
        files.push(entry.file_name().to_string_lossy().to_string());
    }
    for fname in &files {
        if fname.ends_with(".pub")
            || matches!(fname.as_str(), "config" | "known_hosts" | "known_hosts.old" | "authorized_keys")
            || fname.starts_with('.')
        { continue; }
        let path    = dir.join(fname);
        let has_pub = files.contains(&format!("{}.pub", fname));
        entries.push(KeyEntry { name: fname.clone(), path: path.to_string_lossy().to_string(), has_pub });
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn ssh_dir() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/root"))
        .join(".ssh")
}

fn default_key_name(key_type: KeyType) -> String {
    match key_type {
        KeyType::Ed25519 => "id_ed25519".to_string(),
        KeyType::Rsa4096 => "id_rsa".to_string(),
        KeyType::Ecdsa   => "id_ecdsa".to_string(),
    }
}
