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
    pub fingerprint: Option<String>,
    pub created: Option<String>,
    pub loaded_in_agent: bool,
}
