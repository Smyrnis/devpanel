use super::SudoCommand;
use super::password_store::{has_saved_password, load_saved_password};

#[derive(PartialEq, Eq, Default)]
pub enum ModalState {
    #[default]
    Hidden,
    Asking,
    Validating,
    Failed,
}

#[derive(Default)]
pub struct SudoModal {
    pub state: ModalState,
    pub password_input: String,
    pub save_password: bool,
    pub show_password: bool,
    pub cached_password: Option<String>,
    pub pending_command: Option<Box<dyn SudoCommand>>,
}

impl SudoModal {
    pub fn new() -> Self {
        let cached = load_saved_password();
        Self {
            state: ModalState::Hidden,
            password_input: String::new(),
            save_password: has_saved_password(),
            show_password: false,
            cached_password: cached,
            pending_command: None,
        }
    }

    pub fn is_visible(&self) -> bool {
        !matches!(self.state, ModalState::Hidden)
    }

    pub fn get_password(&self) -> Option<String> {
        self.cached_password.clone()
    }
}
