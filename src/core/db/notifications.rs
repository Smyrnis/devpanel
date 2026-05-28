use super::DevPanelDb;
use rusqlite::{Result as SqlResult, params};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct NotificationRecord {
    pub id: i64,
    pub created_at: i64,
    pub ok: bool,
    pub message: String,
}

impl DevPanelDb {
    pub fn add_notification(&self, ok: bool, message: &str) -> SqlResult<()> {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or_default();
        self.conn.execute(
            "INSERT INTO notifications (created_at, ok, message) VALUES (?1, ?2, ?3)",
            params![created_at, if ok { 1 } else { 0 }, message],
        )?;
        self.conn.execute(
            "DELETE FROM notifications
             WHERE id NOT IN (SELECT id FROM notifications ORDER BY id DESC LIMIT 100)",
            [],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn recent_notifications(&self, limit: usize) -> SqlResult<Vec<NotificationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, ok, message FROM notifications ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(NotificationRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                ok: row.get::<_, i64>(2)? != 0,
                message: row.get(3)?,
            })
        })?;
        rows.collect()
    }
}
