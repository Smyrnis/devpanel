fn sentinel_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".config")
        .join("devpanel")
        .join("first_run_done")
}

pub fn is_first_run() -> bool {
    !sentinel_path().exists()
}

pub fn mark_done() {
    let path = sentinel_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, "1");
}

#[derive(Debug, Clone, PartialEq)]
pub enum FirstRunState {
    Visible,
    Hidden,
}

impl Default for FirstRunState {
    fn default() -> Self {
        if is_first_run() {
            FirstRunState::Visible
        } else {
            FirstRunState::Hidden
        }
    }
}
