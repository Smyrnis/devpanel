// src/tabs/ssh_keys/backend.rs — SSH key generation and ~/.ssh directory listing

use super::{KeyEntry, KeyType};

// ── Public async tasks ────────────────────────────────────────────────────

/// Generate a new SSH key pair in ~/.ssh using ssh-keygen.
pub async fn generate_key(
    email:      String,
    key_name:   String,
    key_type:   KeyType,
    passphrase: String,
) -> (bool, String) {
    let ssh_dir = ssh_dir();
    if let Err(e) = tokio::fs::create_dir_all(&ssh_dir).await {
        return (false, format!("Cannot create ~/.ssh: {}", e));
    }

    let key_path = ssh_dir.join(if key_name.is_empty() {
        default_key_name(key_type)
    } else {
        key_name.clone()
    });

    if key_path.exists() {
        return (false, format!("{} already exists — choose a different name", key_path.display()));
    }

    let (type_flag, bits_flags): (&str, Vec<&str>) = match key_type {
        KeyType::Ed25519 => ("ed25519", vec![]),
        KeyType::Rsa4096 => ("rsa",     vec!["-b", "4096"]),
        KeyType::Ecdsa   => ("ecdsa",   vec!["-b", "521"]),
    };

    let comment = if email.is_empty() {
        key_path.file_name().unwrap_or_default().to_string_lossy().to_string()
    } else {
        email.clone()
    };

    let mut args = vec![
        "-t", type_flag,
        "-C", &comment,
        "-f", key_path.to_str().unwrap_or(""),
        "-N", &passphrase,
    ];
    args.extend_from_slice(&bits_flags);

    let out = tokio::process::Command::new("ssh-keygen")
        .args(&args)
        .output().await;

    match out {
        Ok(o) if o.status.success() =>
            (true, format!("Key generated: {}", key_path.display())),
        Ok(o) =>
            (false, String::from_utf8_lossy(&o.stderr).trim().to_string()),
        Err(e) =>
            (false, format!("ssh-keygen not found: {}", e)),
    }
}

/// List all key files in ~/.ssh (private keys only — excludes .pub, known_hosts, config).
pub async fn list_keys() -> Vec<KeyEntry> {
    let dir = ssh_dir();
    let mut entries = Vec::new();

    let mut dir_reader = match tokio::fs::read_dir(&dir).await {
        Ok(d)  => d,
        Err(_) => return entries,
    };

    while let Ok(Some(entry)) = dir_reader.next_entry().await {
        let fname = entry.file_name().to_string_lossy().to_string();
        // Skip public keys, known_hosts, config, and dotfiles
        if fname.ends_with(".pub")
            || fname == "known_hosts"
            || fname == "known_hosts.old"
            || fname == "config"
            || fname.starts_with('.')
        {
            continue;
        }
        // Only include files (not directories)
        if let Ok(meta) = entry.metadata().await {
            if !meta.is_file() { continue; }
        }
        let path    = dir.join(&fname);
        let has_pub = dir.join(format!("{}.pub", fname)).exists();
        entries.push(KeyEntry {
            name:    fname,
            path:    path.to_string_lossy().to_string(),
            has_pub,
        });
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
