// src/core/db.rs
//
// SQLite-backed persistent settings store.
//
// Schema (v1):
//   settings   — key/value table for all user preferences
//   vhost_tags — optional user-defined labels on virtual hosts
//
// Usage:
//   let db = DevPanelDb::open()?;
//   db.set("apache.log_level", "warn")?;
//   let level = db.get("apache.log_level")?;  // Some("warn")
//
// The database lives at ~/.config/devpanel/devpanel.db.
// All operations are synchronous and fast (local SQLite); call them on a
// Tokio blocking thread if needed:
//   tokio::task::spawn_blocking(|| db.set(...))

use rusqlite::{Connection, Result as SqlResult, params};
use std::path::PathBuf;

// ── Schema ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS vhost_tags (
    server_name TEXT PRIMARY KEY NOT NULL,
    tag         TEXT NOT NULL DEFAULT '',
    notes       TEXT NOT NULL DEFAULT ''
);
"#;

// ── Setting keys (centralised constants) ──────────────────────────────────

/// All recognised setting keys. Use these constants everywhere instead of
/// raw strings so a typo becomes a compile error.
#[allow(dead_code)]
pub mod keys {
    // Apache
    pub const APACHE_LOG_LEVEL: &str = "apache.log_level";
    pub const APACHE_AUTO_RELOAD: &str = "apache.auto_reload_on_save";

    // PHP
    pub const PHP_DEFAULT_VERSION: &str = "php.default_version";
    pub const PHP_DISPLAY_ERRORS: &str = "php.display_errors_dev";

    // Projects
    pub const PROJECTS_ROOT: &str = "projects.root";
    pub const PROJECTS_OPEN_COMMAND: &str = "projects.open_command";

    // UI / behaviour
    pub const UI_CONFIRM_DELETES: &str = "ui.confirm_deletes";
    pub const UI_TOAST_DURATION_MS: &str = "ui.toast_duration_ms";
    pub const UI_SHOW_SETUP_LOG: &str = "ui.show_setup_log_on_warn";

    // SSH
    pub const SSH_DEFAULT_KEY_TYPE: &str = "ssh.default_key_type";

    // Editor
    pub const EDITOR_COMMAND: &str = "editor.command";
}

/// Default values used when a key is absent from the database.
#[allow(dead_code)]
pub mod defaults {
    pub const APACHE_LOG_LEVEL: &str = "warn";
    pub const APACHE_AUTO_RELOAD: &str = "true";
    pub const PHP_DEFAULT_VERSION: &str = "";
    pub const PHP_DISPLAY_ERRORS: &str = "true";
    pub const PROJECTS_OPEN_COMMAND: &str = "xdg-open";
    pub const UI_CONFIRM_DELETES: &str = "true";
    pub const UI_TOAST_DURATION_MS: &str = "4000";
    pub const UI_SHOW_SETUP_LOG: &str = "true";
    pub const SSH_DEFAULT_KEY_TYPE: &str = "Ed25519";
    pub const EDITOR_COMMAND: &str = "xdg-open";
}

// ── DevPanelDb ────────────────────────────────────────────────────────────

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
        conn.execute_batch(SCHEMA_V1)?;
        Ok(DevPanelDb { conn })
    }

    /// Open an in-memory database — used by tests so they never touch disk.
    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA_V1)?;
        Ok(DevPanelDb { conn })
    }

    // ── Settings ──────────────────────────────────────────────────────────

    /// Return the stored value for `key`, or `None` if the key is absent.
    pub fn get(&self, key: &str) -> SqlResult<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Return the stored value for `key`, or `default` if the key is absent.
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key)
            .ok()
            .flatten()
            .unwrap_or_else(|| default.to_string())
    }

    /// Store or update `key` with `value`.
    pub fn set(&self, key: &str, value: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Delete a key from settings (it will revert to its default on next read).
    pub fn delete(&self, key: &str) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Return all key/value pairs in settings — used by the Config tab to
    /// populate the UI on load.
    pub fn all_settings(&self) -> SqlResult<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    // ── Boolean / numeric convenience wrappers ────────────────────────────

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        let default_str = if default { "true" } else { "false" };
        self.get_or(key, default_str) == "true"
    }

    pub fn set_bool(&self, key: &str, value: bool) -> SqlResult<()> {
        self.set(key, if value { "true" } else { "false" })
    }

    pub fn get_u32(&self, key: &str, default: u32) -> u32 {
        self.get_or(key, &default.to_string())
            .parse()
            .unwrap_or(default)
    }

    pub fn set_u32(&self, key: &str, value: u32) -> SqlResult<()> {
        self.set(key, &value.to_string())
    }

    // ── VHost tags ────────────────────────────────────────────────────────

    /// Get the tag and notes for a virtual host by server_name.
    pub fn get_vhost_meta(&self, server_name: &str) -> SqlResult<Option<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag, notes FROM vhost_tags WHERE server_name = ?1")?;
        let mut rows = stmt.query(params![server_name])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?)))
        } else {
            Ok(None)
        }
    }

    /// Store or update the tag/notes for a virtual host.
    pub fn set_vhost_meta(&self, server_name: &str, tag: &str, notes: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO vhost_tags (server_name, tag, notes) VALUES (?1, ?2, ?3)
             ON CONFLICT(server_name) DO UPDATE SET tag = excluded.tag, notes = excluded.notes",
            params![server_name, tag, notes],
        )?;
        Ok(())
    }

    /// All vhost metadata rows — used to populate tags in the VHosts tab.
    pub fn all_vhost_meta(&self) -> SqlResult<Vec<(String, String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT server_name, tag, notes FROM vhost_tags ORDER BY server_name")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        rows.collect()
    }
}

// ── Path helper ───────────────────────────────────────────────────────────

#[allow(dead_code)]
fn db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join(".config")
        .join("devpanel")
        .join("devpanel.db")
}

// ── Snapshot of all user-facing settings ─────────────────────────────────
//
// UserSettings is a plain struct loaded once on startup and refreshed
// whenever the Config tab saves changes.  The Config tab binds directly
// to this struct so it never has to call db.get() per-field.

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserSettings {
    pub apache_log_level: String,
    pub apache_auto_reload: bool,
    pub php_default_version: String,
    pub php_display_errors: bool,
    pub projects_open_command: String,
    pub ui_confirm_deletes: bool,
    pub ui_toast_duration_ms: u32,
    pub ui_show_setup_log: bool,
    pub ssh_default_key_type: String,
    pub editor_command: String,
}

#[allow(dead_code)]
impl UserSettings {
    pub fn load(db: &DevPanelDb) -> Self {
        UserSettings {
            apache_log_level: db.get_or(keys::APACHE_LOG_LEVEL, defaults::APACHE_LOG_LEVEL),
            apache_auto_reload: db.get_bool(keys::APACHE_AUTO_RELOAD, true),
            php_default_version: db
                .get_or(keys::PHP_DEFAULT_VERSION, defaults::PHP_DEFAULT_VERSION),
            php_display_errors: db.get_bool(keys::PHP_DISPLAY_ERRORS, true),
            projects_open_command: db
                .get_or(keys::PROJECTS_OPEN_COMMAND, defaults::PROJECTS_OPEN_COMMAND),
            ui_confirm_deletes: db.get_bool(keys::UI_CONFIRM_DELETES, true),
            ui_toast_duration_ms: db.get_u32(keys::UI_TOAST_DURATION_MS, 4000),
            ui_show_setup_log: db.get_bool(keys::UI_SHOW_SETUP_LOG, true),
            ssh_default_key_type: db
                .get_or(keys::SSH_DEFAULT_KEY_TYPE, defaults::SSH_DEFAULT_KEY_TYPE),
            editor_command: db.get_or(keys::EDITOR_COMMAND, defaults::EDITOR_COMMAND),
        }
    }

    /// Persist the entire struct back to the database in one transaction.
    pub fn save(&self, db: &DevPanelDb) -> SqlResult<()> {
        db.set(keys::APACHE_LOG_LEVEL, &self.apache_log_level)?;
        db.set_bool(keys::APACHE_AUTO_RELOAD, self.apache_auto_reload)?;
        db.set(keys::PHP_DEFAULT_VERSION, &self.php_default_version)?;
        db.set_bool(keys::PHP_DISPLAY_ERRORS, self.php_display_errors)?;
        db.set(keys::PROJECTS_OPEN_COMMAND, &self.projects_open_command)?;
        db.set_bool(keys::UI_CONFIRM_DELETES, self.ui_confirm_deletes)?;
        db.set_u32(keys::UI_TOAST_DURATION_MS, self.ui_toast_duration_ms)?;
        db.set_bool(keys::UI_SHOW_SETUP_LOG, self.ui_show_setup_log)?;
        db.set(keys::SSH_DEFAULT_KEY_TYPE, &self.ssh_default_key_type)?;
        db.set(keys::EDITOR_COMMAND, &self.editor_command)?;
        Ok(())
    }
}
