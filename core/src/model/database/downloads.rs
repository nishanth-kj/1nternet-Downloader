pub use crate::constants::DownloadStatus;
use crate::exception::{DatabaseException, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

pub type DownloadId = i64;

// -----------------------------------------------------------------------------
// Diesel ORM Table Schema & Table Struct Definition
// -----------------------------------------------------------------------------

diesel::table! {
    downloads (downloads_id) {
        downloads_id -> BigInt,
        url -> Text,
        destination -> Text,
        filename -> Text,
        total_bytes -> BigInt,
        downloaded_bytes -> BigInt,
        status -> Integer,
        mime_type -> Nullable<Text>,
        checksum -> Nullable<Text>,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

/// Downloads table ORM struct representing a persistent download record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: DownloadId,
    pub url: String,
    pub destination: String,
    pub filename: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl DownloadItem {
    pub fn new(url: String, destination: String, filename: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: 0,
            url,
            destination,
            filename,
            total_bytes: 0,
            downloaded_bytes: 0,
            status: DownloadStatus::Queued,
            mime_type: None,
            checksum: None,
            created_at: now,
            updated_at: now,
        }
    }
}

// -----------------------------------------------------------------------------
// Native SQL DDL & Query Functions
// -----------------------------------------------------------------------------

pub const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS downloads (
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
    );
";

/// Native query function to insert a download item into the database
pub fn insert_download(conn: &Connection, item: &DownloadItem) -> Result<DownloadId> {
    conn.execute(
        "INSERT INTO downloads (
            url, destination, filename, total_bytes, downloaded_bytes,
            status, mime_type, checksum, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
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

    Ok(conn.last_insert_rowid())
}

/// Native query function to update progress of an existing download
pub fn update_download_progress(
    conn: &Connection,
    id: DownloadId,
    downloaded_bytes: u64,
    total_bytes: u64,
    status: DownloadStatus,
) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE downloads
        SET downloaded_bytes = ?1, total_bytes = ?2, status = ?3, updated_at = ?4
        WHERE downloads_id = ?5;",
        params![
            downloaded_bytes as i64,
            total_bytes as i64,
            status.to_i32(),
            now,
            id
        ],
    )
    .map_err(|e| DatabaseException::Sqlite(format!("Failed to update progress: {}", e)))?;
    Ok(())
}

/// Native query function to update the status of a download
pub fn update_download_status(
    conn: &Connection,
    id: DownloadId,
    status: DownloadStatus,
) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE downloads
        SET status = ?1, updated_at = ?2
        WHERE downloads_id = ?3;",
        params![status.to_i32(), now, id],
    )
    .map_err(|e| DatabaseException::Sqlite(format!("Failed to update status: {}", e)))?;
    Ok(())
}

/// Native query function to query a download item by its primary key id
pub fn select_download_by_id(
    conn: &Connection,
    id: DownloadId,
) -> Result<Option<DownloadItem>> {
    let mut stmt = conn
        .prepare(
            "SELECT downloads_id, url, destination, filename, total_bytes, downloaded_bytes,
                   status, mime_type, checksum, created_at, updated_at
            FROM downloads
            WHERE downloads_id = ?1;",
        )
        .map_err(|e| DatabaseException::Sqlite(format!("Prepare error: {}", e)))?;

    let mut rows = stmt
        .query(params![id])
        .map_err(|e| DatabaseException::Sqlite(format!("Query error: {}", e)))?;

    if let Some(row) = rows
        .next()
        .map_err(|e| DatabaseException::Sqlite(format!("Fetch error: {}", e)))?
    {
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

/// Native query function to query all downloads ordered by creation date descending
pub fn select_all_downloads(conn: &Connection) -> Result<Vec<DownloadItem>> {
    let mut stmt = conn
        .prepare(
            "SELECT downloads_id, url, destination, filename, total_bytes, downloaded_bytes,
                   status, mime_type, checksum, created_at, updated_at
            FROM downloads
            ORDER BY created_at DESC;",
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

/// Native query function to delete a download by its primary key id
pub fn delete_download_by_id(conn: &Connection, id: DownloadId) -> Result<()> {
    conn.execute(
        "DELETE FROM downloads WHERE downloads_id = ?1;",
        params![id],
    )
    .map_err(|e| DatabaseException::Sqlite(format!("Failed to delete download: {}", e)))?;
    Ok(())
}
