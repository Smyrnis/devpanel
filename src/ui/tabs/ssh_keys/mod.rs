pub mod view;

pub use crate::domain::ssh_keys::{KeyEntry, KeyType, StatusKind};

pub struct SshKeysTab {
    pub email: String,
    pub key_name: String,
    pub key_type: KeyType,
    pub passphrase: String,
    pub show_passphrase: bool,
    pub status_message: String,
    pub status_kind: StatusKind,
    pub keys_list: Vec<KeyEntry>,
}

impl SshKeysTab {
    pub fn new() -> Self {
        Self {
            email: String::new(),
            key_name: String::new(),
            key_type: KeyType::Ed25519,
            passphrase: String::new(),
            show_passphrase: false,
            status_message: String::new(),
            status_kind: StatusKind::None,
            keys_list: Vec::new(),
        }
    }
}

impl Default for SshKeysTab {
    fn default() -> Self {
        Self::new()
    }
}
