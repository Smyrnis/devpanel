use crate::core::dry_run;
use std::path::PathBuf;

const OBFUSCATION_KEY: &[u8] = b"devpanel_xor_v1_";

fn obfuscate(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ OBFUSCATION_KEY[i % OBFUSCATION_KEY.len()])
        .collect()
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".config")
        .join("devpanel")
        .join("sudo.dat")
}

pub fn load_saved_password() -> Option<String> {
    let path = config_path();
    let raw = std::fs::read(&path).ok()?;
    let decoded = obfuscate(&raw);
    String::from_utf8(decoded).ok()
}

pub fn save_password(password: &str) {
    if dry_run::active() {
        dry_run::log("save_password - skipped in dry-run mode");
        return;
    }
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let encoded = obfuscate(password.as_bytes());
    let _ = std::fs::write(&path, encoded);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
}

pub fn clear_saved_password() {
    if dry_run::active() {
        dry_run::log("clear_saved_password - skipped in dry-run mode");
        return;
    }
    let _ = std::fs::remove_file(config_path());
}

pub fn has_saved_password() -> bool {
    config_path().exists()
}
