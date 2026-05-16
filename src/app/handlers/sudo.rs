use iced::Task;

use crate::app::App;
use crate::core::sudo_prompt::{
    BoxedSudoCommand, ModalState, clear_saved_password, save_password, validate_sudo_password,
};
use crate::messages::{Message, SudoMessage};

impl App {
    pub(crate) fn handle_sudo(&mut self, msg: SudoMessage) -> Task<Message> {
        match msg {
            SudoMessage::PasswordChanged(v) => {
                self.sudo.password_input = v;
                Task::none()
            }
            SudoMessage::ToggleShow(v) => {
                self.sudo.show_password = v;
                Task::none()
            }
            SudoMessage::ToggleSave(v) => {
                self.sudo.save_password = v;
                if !v {
                    clear_saved_password();
                }
                Task::none()
            }
            SudoMessage::Cancel => {
                if self
                    .sudo
                    .pending_command
                    .as_ref()
                    .is_some_and(|command| command.is_first_run_install())
                {
                    self.first_run_installing = false;
                }
                self.sudo.pending_command = None;
                self.sudo.state = ModalState::Hidden;
                self.sudo.password_input.clear();
                Task::none()
            }
            SudoMessage::Submit => {
                let pass = self.sudo.password_input.clone();
                if pass.is_empty() {
                    return Task::none();
                }
                self.sudo.state = ModalState::Validating;
                Task::perform(validate_sudo_password(pass), |ok| {
                    Message::Sudo(SudoMessage::ValidationResult(ok))
                })
            }
            SudoMessage::ValidationResult(valid) => {
                if !valid {
                    self.sudo.state = ModalState::Failed;
                    self.sudo.password_input.clear();
                    return Task::none();
                }
                let password = self.sudo.password_input.clone();
                self.sudo.cached_password = Some(password.clone());
                self.sudo.password_input.clear();
                self.sudo.state = ModalState::Hidden;
                if self.sudo.save_password {
                    save_password(&password);
                }
                if let Some(command) = self.sudo.pending_command.take() {
                    self.dispatch_sudo_command(command, password)
                } else {
                    Task::none()
                }
            }
            SudoMessage::ClearSaved => {
                clear_saved_password();
                self.sudo.cached_password = None;
                self.sudo.save_password = false;
                self.show_toast("Saved password cleared.".into(), true)
            }
        }
    }

    pub fn trigger_sudo(&mut self, command: BoxedSudoCommand) -> Task<Message> {
        if let Some(password) = self.sudo.get_password() {
            self.dispatch_sudo_command(command, password)
        } else {
            self.sudo.pending_command = Some(command);
            self.sudo.state = ModalState::Asking;
            self.sudo.password_input.clear();
            Task::none()
        }
    }

    fn dispatch_sudo_command(
        &mut self,
        command: BoxedSudoCommand,
        password: String,
    ) -> Task<Message> {
        Task::perform(command.execute(&password), |message| message)
    }
}
