pub mod backend;
pub mod view;

pub use backend::{generate_key, list_keys};

use crate::messages::Message;
use iced::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Ed25519,
    Rsa4096,
    Ecdsa,
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Ed25519 => write!(f, "Ed25519"),
            KeyType::Rsa4096 => write!(f, "RSA 4096"),
            KeyType::Ecdsa => write!(f, "ECDSA 521"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusKind {
    None,
    Success,
    Error,
    #[allow(dead_code)]
    Info,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub name: String,
    pub path: String,
    pub has_pub: bool,
}

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

    pub fn view(&self) -> Element<'_, Message> {
        view::render(self)
    }
}
