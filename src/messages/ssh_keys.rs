#[derive(Debug, Clone)]
pub enum SshKeysMessage {
    EmailChanged(String),
    KeyNameChanged(String),
    KeyTypeChanged(crate::domain::ssh_keys::KeyType),
    PassphraseChanged(String),
    TogglePassphrase(bool),
    GenerateKey,
    GenerateDone(bool, String),
    AddExisting,
    AddExistingDone(bool, String),
    OpenDir,
    ListKeys,
    KeysListed(Vec<crate::domain::ssh_keys::KeyEntry>),
    CopyPublicKey(String),
    CopyPublicKeyDone(bool, String),
}
