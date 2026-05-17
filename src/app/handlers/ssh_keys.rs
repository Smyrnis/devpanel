use iced::Task;

use crate::app::App;

use crate::core::error::result_status;

use crate::core::system::{get_home, ssh_add, xdg_open};

use crate::messages::{Message, SshKeysMessage};

impl App {
    pub(crate) fn handle_ssh_keys(&mut self, msg: SshKeysMessage) -> Task<Message> {
        match msg {
            SshKeysMessage::EmailChanged(v) => {
                self.ssh_keys.email = v;
                Task::none()
            }
            SshKeysMessage::KeyNameChanged(v) => {
                self.ssh_keys.key_name = v;
                Task::none()
            }
            SshKeysMessage::KeyTypeChanged(t) => {
                self.ssh_keys.key_type = t;
                Task::none()
            }
            SshKeysMessage::PassphraseChanged(v) => {
                self.ssh_keys.passphrase = v;
                Task::none()
            }
            SshKeysMessage::TogglePassphrase(v) => {
                self.ssh_keys.show_passphrase = v;
                Task::none()
            }

            SshKeysMessage::GenerateKey => {
                let (email, name, ktype, pass) = (
                    self.ssh_keys.email.clone(),
                    self.ssh_keys.key_name.clone(),
                    self.ssh_keys.key_type,
                    self.ssh_keys.passphrase.clone(),
                );
                Task::perform(
                    crate::tabs::ssh_keys::generate_key(email, name, ktype, pass),
                    |result| {
                        let (ok, msg) = result_status(result);
                        Message::SshKeys(SshKeysMessage::GenerateDone(ok, msg))
                    },
                )
            }

            SshKeysMessage::GenerateDone(ok, msg) => {
                use crate::tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                self.ssh_keys.status_message = msg.clone();
                if ok {
                    Task::batch([
                        self.show_toast(msg, ok),
                        Task::perform(crate::tabs::ssh_keys::list_keys(), |keys| {
                            Message::SshKeys(SshKeysMessage::KeysListed(keys))
                        }),
                    ])
                } else {
                    self.show_toast(msg, ok)
                }
            }

            SshKeysMessage::AddExisting => {
                let path = format!("{}/.ssh/{}", get_home().display(), self.ssh_keys.key_name);
                Task::perform(ssh_add(path), |result| {
                    let (ok, msg) = result_status(result);
                    Message::SshKeys(SshKeysMessage::AddExistingDone(ok, msg))
                })
            }

            SshKeysMessage::AddExistingDone(ok, msg) => {
                use crate::tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                self.ssh_keys.status_message = msg.clone();
                self.show_toast(msg, ok)
            }

            SshKeysMessage::OpenDir => {
                let _ = xdg_open(&format!("{}/.ssh", get_home().display()));
                Task::none()
            }

            SshKeysMessage::ListKeys => Task::perform(crate::tabs::ssh_keys::list_keys(), |keys| {
                Message::SshKeys(SshKeysMessage::KeysListed(keys))
            }),

            SshKeysMessage::KeysListed(keys) => {
                self.ssh_keys.keys_list = keys;
                Task::none()
            }
            SshKeysMessage::CopyPublicKey(path) => Task::perform(
                async move {
                    match crate::tabs::ssh_keys::read_public_key(path).await {
                        Ok(text) => {
                            crate::core::system::copy_to_clipboard(text).await;
                            (true, "Public key copied".to_string())
                        }
                        Err(e) => (false, e.to_string()),
                    }
                },
                |(ok, msg)| Message::SshKeys(SshKeysMessage::CopyPublicKeyDone(ok, msg)),
            ),
            SshKeysMessage::CopyPublicKeyDone(ok, msg) => {
                use crate::tabs::ssh_keys::StatusKind;
                self.ssh_keys.status_kind = if ok {
                    StatusKind::Success
                } else {
                    StatusKind::Error
                };
                self.ssh_keys.status_message = msg.clone();
                self.show_toast(msg, ok)
            }
        }
    }
}
