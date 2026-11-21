pub mod entity;
pub use entity::{DownloadItem, TorrentItem};

use crate::exception::{DatabaseException, IdmException, Result};
use crate::model::download::{DownloadId, DownloadStatus};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::info;

pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    pub fn open_default() -> Result<Self> {
        let dir = crate::config::Config::app_data_dir();
        std::fs::create_dir_all(&dir).map_err(IdmException::from)?;
        let path = dir.join("downloads.db");
        Self::open(&path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let conn = Connection::open(&path_buf)
            .map_err(|e| DatabaseException::Sqlite(format!("Failed to open SQLite database: {}", e)))?;

        // Initialize schema if not present (identical to C++ sqlite_orm schema)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS downloads (
                downloads_id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                destination TEXT NOT NULL,
                filename TEXT NOT NULL,
                total_bytes INTEGER DEFAULT 0,
                downloaded_bytes INTEGER DEFAULT 0,
                status INTEGER DEFAULT 0,
                mime_type TEXT,
                checksum TEXT,
                created_at INTEGER,
                updated_at INTEGER
            )",
            [],
        )
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to create downloads table: {}", e)))?;

        info!("SQLite database opened at: {:?}", path_buf);

        Ok(Self {
            conn: Mutex::new(conn),
            path: path_buf,
        })
    }

    pub fn insert_download(&self, item: &DownloadItem) -> Result<DownloadId> {
        let conn = self.conn.lock().map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
        conn.execute(
            "INSERT INTO downloads (
                url, destination, filename, total_bytes, downloaded_bytes,
                status, mime_type, checksum, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                item.url,
                item.destination,
                item.filename,
                item.total_bytes as i64,
                item.downloaded_bytes as i64,
                item.status.to_i32(),
                item.mime_type,
                item.checksum,
                item.created_at,
                item.updated_at,
            ],
        )
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to insert download: {}", e)))?;

        let id = conn.last_insert_rowid();
        Ok(id)
    }

    pub fn update_progress(&self, id: DownloadId, downloaded_bytes: u64, total_bytes: u64, status: DownloadStatus) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE downloads SET downloaded_bytes = ?1, total_bytes = ?2, status = ?3, updated_at = ?4 WHERE downloads_id = ?5",
            params![downloaded_bytes as i64, total_bytes as i64, status.to_i32(), now, id],
        )
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to update progress: {}", e)))?;
        Ok(())
    }

    pub fn update_status(&self, id: DownloadId, status: DownloadStatus) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE downloads SET status = ?1, updated_at = ?2 WHERE downloads_id = ?3",
            params![status.to_i32(), now, id],
        )
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to update status: {}", e)))?;
        Ok(())
    }

    pub fn get_download(&self, id: DownloadId) -> Result<Option<DownloadItem>> {
        let conn = self.conn.lock().map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
        let mut stmt = conn
            .prepare(
                "SELECT downloads_id, url, destination, filename, total_bytes, downloaded_bytes,
                        status, mime_type, checksum, created_at, updated_at
                 FROM downloads WHERE downloads_id = ?1",
            )
            .map_err(|e| DatabaseException::Sqlite(format!("Prepare error: {}", e)))?;

        let mut rows = stmt
            .query(params![id])
            .map_err(|e| DatabaseException::Sqlite(format!("Query error: {}", e)))?;

        if let Some(row) = rows.next().map_err(|e| DatabaseException::Sqlite(format!("Row error: {}", e)))? {
            let status_int: i32 = row.get(6).unwrap_or(0);
            let item = DownloadItem {
                id: row.get(0).unwrap_or(0),
                url: row.get(1).unwrap_or_default(),
                destination: row.get(2).unwrap_or_default(),
                filename: row.get(3).unwrap_or_default(),
                total_bytes: row.get::<_, i64>(4).unwrap_or(0).max(0) as u64,
                downloaded_bytes: row.get::<_, i64>(5).unwrap_or(0).max(0) as u64,
                status: DownloadStatus::from_i32(status_int),
                mime_type: row.get(7).ok(),
                checksum: row.get(8).ok(),
                created_at: row.get(9).unwrap_or(0),
                updated_at: row.get(10).unwrap_or(0),
            };
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_downloads(&self) -> Result<Vec<DownloadItem>> {
        let conn = self.conn.lock().map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
        let mut stmt = conn
            .prepare(
                "SELECT downloads_id, url, destination, filename, total_bytes, downloaded_bytes,
                        status, mime_type, checksum, created_at, updated_at
                 FROM downloads ORDER BY created_at DESC",
            )
            .map_err(|e| DatabaseException::Sqlite(format!("Prepare error: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                let status_int: i32 = row.get(6)?;
                Ok(DownloadItem {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    destination: row.get(2)?,
                    filename: row.get(3)?,
                    total_bytes: row.get::<_, i64>(4)?.max(0) as u64,
                    downloaded_bytes: row.get::<_, i64>(5)?.max(0) as u64,
                    status: DownloadStatus::from_i32(status_int),
                    mime_type: row.get(7)?,
                    checksum: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| DatabaseException::Sqlite(format!("Query map error: {}", e)))?;

        let mut items = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                items.push(item);
            }
        }
        Ok(items)
    }

    pub fn delete_download(&self, id: DownloadId) -> Result<()> {
        let conn = self.conn.lock().map_err(|_| DatabaseException::LockError("Poisoned DB mutex".into()))?;
        conn.execute("DELETE FROM downloads WHERE downloads_id = ?1", params![id])
            .map_err(|e| DatabaseException::Sqlite(format!("Failed to delete download: {}", e)))?;
        Ok(())
    }
}
