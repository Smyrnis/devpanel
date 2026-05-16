use rusqlite::{Connection, Result as SqlResult};

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

CREATE TABLE IF NOT EXISTS notifications (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at INTEGER NOT NULL,
    ok         INTEGER NOT NULL,
    message    TEXT NOT NULL
);
"#;

pub(super) fn migrate(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(SCHEMA_V1)
}
