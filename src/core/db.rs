mod notifications;
mod schema;
mod settings;
mod user_settings;
mod vhosts;

use rusqlite::{Connection, Result as SqlResult};
use std::path::PathBuf;

#[allow(unused_imports)]
pub use notifications::NotificationRecord;
#[allow(unused_imports)]
pub use settings::{defaults, keys};
pub use user_settings::UserSettings;

#[allow(dead_code)]
pub struct DevPanelDb {
    conn: Connection,
}

#[allow(dead_code)]
impl DevPanelDb {
    /// Open (or create) the database at `~/.config/devpanel/devpanel.db`.
    pub fn open() -> SqlResult<Self> {
        let path = db_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path)?;
        schema::migrate(&conn)?;
        Ok(DevPanelDb { conn })
    }

    /// Open an in-memory database, used by tests so they never touch disk.
    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        schema::migrate(&conn)?;
        Ok(DevPanelDb { conn })
    }
}

#[allow(dead_code)]
fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".config")
        .join("devpanel")
        .join("devpanel.db")
}
