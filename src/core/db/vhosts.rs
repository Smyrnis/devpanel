use super::DevPanelDb;
use rusqlite::{Result as SqlResult, params};

impl DevPanelDb {
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

    pub fn set_vhost_meta(&self, server_name: &str, tag: &str, notes: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO vhost_tags (server_name, tag, notes) VALUES (?1, ?2, ?3)
             ON CONFLICT(server_name) DO UPDATE SET tag = excluded.tag, notes = excluded.notes",
            params![server_name, tag, notes],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn all_vhost_meta(&self) -> SqlResult<Vec<(String, String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT server_name, tag, notes FROM vhost_tags ORDER BY server_name")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        rows.collect()
    }
}
