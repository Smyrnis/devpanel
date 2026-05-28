#[derive(Debug, Clone)]
pub enum SudoMessage {
    PasswordChanged(String),
    ToggleShow(bool),
    ToggleSave(bool),
    Cancel,
    Submit,
    ValidationResult(bool),
    ClearSaved,
}
