#[derive(Debug, thiserror::Error)]
pub enum DevPanelError {
    #[error("sudo command failed: {0}")]
    Sudo(String),
    #[error("file I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("apache configuration error: {0}")]
    Apache(String),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("command failed: {0}")]
    Command(String),
    #[error("validation error: {0}")]
    Validation(String),
}

pub type DevPanelResult<T = String> = Result<T, DevPanelError>;

pub fn result_status(result: DevPanelResult) -> (bool, String) {
    match result {
        Ok(message) => (true, message),
        Err(error) => (false, error.to_string()),
    }
}
