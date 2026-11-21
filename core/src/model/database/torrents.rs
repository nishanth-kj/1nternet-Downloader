use crate::constants::DownloadStatus;
use crate::exception::{DatabaseException, Result};
use crate::model::database::downloads::DownloadId;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// Diesel ORM Table Schema & Table Struct Definition
// -----------------------------------------------------------------------------

diesel::table! {
    torrents (torrent_id) {
        torrent_id -> BigInt,
        info_hash -> Text,
        name -> Text,
        save_path -> Text,
        total_bytes -> BigInt,
        downloaded_bytes -> BigInt,
        status -> Integer,
        seed_ratio -> Float,
        created_at -> BigInt,
    }
}

/// Torrents table ORM struct representing a persistent BitTorrent task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentItem {
    pub id: DownloadId,
    pub info_hash: String,
    pub name: String,
    pub save_path: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub seed_ratio: f32,
    pub created_at: i64,
}

impl TorrentItem {
    pub fn new(info_hash: String, name: String, save_path: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: 0,
            info_hash,
            name,
            save_path,
            total_bytes: 0,
            downloaded_bytes: 0,
            status: DownloadStatus::Queued,
            seed_ratio: 0.0,
            created_at: now,
        }
    }
}

// -----------------------------------------------------------------------------
// Native SQL DDL & Query Functions
// -----------------------------------------------------------------------------

pub const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS torrents (
        torrent_id INTEGER PRIMARY KEY AUTOINCREMENT,
        info_hash TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        save_path TEXT NOT NULL,
        total_bytes INTEGER DEFAULT 0,
        downloaded_bytes INTEGER DEFAULT 0,
        status INTEGER DEFAULT 0,
        seed_ratio REAL DEFAULT 0.0,
        created_at INTEGER
    );
";

/// Native query function to insert a torrent item
pub fn insert_torrent(conn: &Connection, item: &TorrentItem) -> Result<DownloadId> {
    conn.execute(
        "INSERT INTO torrents (
            info_hash, name, save_path, total_bytes, downloaded_bytes,
            status, seed_ratio, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
        params![
            item.info_hash,
            item.name,
            item.save_path,
            item.total_bytes as i64,
            item.downloaded_bytes as i64,
            item.status.to_i32(),
            item.seed_ratio,
            item.created_at,
        ],
    )
    .map_err(|e| DatabaseException::Sqlite(format!("Failed to insert torrent: {}", e)))?;

    Ok(conn.last_insert_rowid())
}

/// Native query function to select a torrent by id
pub fn select_torrent_by_id(conn: &Connection, id: DownloadId) -> Result<Option<TorrentItem>> {
    let mut stmt = conn
        .prepare(
            "SELECT torrent_id, info_hash, name, save_path, total_bytes, downloaded_bytes,
                   status, seed_ratio, created_at
            FROM torrents
            WHERE torrent_id = ?1;",
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
        let item = TorrentItem {
            id: row.get(0).unwrap_or(0),
            info_hash: row.get(1).unwrap_or_default(),
            name: row.get(2).unwrap_or_default(),
            save_path: row.get(3).unwrap_or_default(),
            total_bytes: row.get::<_, i64>(4).unwrap_or(0).max(0) as u64,
            downloaded_bytes: row.get::<_, i64>(5).unwrap_or(0).max(0) as u64,
            status: DownloadStatus::from_i32(status_int),
            seed_ratio: row.get(7).unwrap_or(0.0),
            created_at: row.get(8).unwrap_or(0),
        };
        Ok(Some(item))
    } else {
        Ok(None)
    }
}

/// Native query function to select all torrents
pub fn select_all_torrents(conn: &Connection) -> Result<Vec<TorrentItem>> {
    let mut stmt = conn
        .prepare(
            "SELECT torrent_id, info_hash, name, save_path, total_bytes, downloaded_bytes,
                   status, seed_ratio, created_at
            FROM torrents
            ORDER BY created_at DESC;",
        )
        .map_err(|e| DatabaseException::Sqlite(format!("Prepare error: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            let status_int: i32 = row.get(6)?;
            Ok(TorrentItem {
                id: row.get(0)?,
                info_hash: row.get(1)?,
                name: row.get(2)?,
                save_path: row.get(3)?,
                total_bytes: row.get::<_, i64>(4)?.max(0) as u64,
                downloaded_bytes: row.get::<_, i64>(5)?.max(0) as u64,
                status: DownloadStatus::from_i32(status_int),
                seed_ratio: row.get(7)?,
                created_at: row.get(8)?,
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

/// Native query function to delete a torrent by id
pub fn delete_torrent_by_id(conn: &Connection, id: DownloadId) -> Result<()> {
    conn.execute(
        "DELETE FROM torrents WHERE torrent_id = ?1;",
        params![id],
    )
    .map_err(|e| DatabaseException::Sqlite(format!("Failed to delete torrent: {}", e)))?;
    Ok(())
}

/// Native query function to update progress and status of a torrent
pub fn update_torrent_progress(
    conn: &Connection,
    id: DownloadId,
    downloaded_bytes: u64,
    status: DownloadStatus,
    seed_ratio: f32,
) -> Result<()> {
    conn.execute(
        "UPDATE torrents
        SET downloaded_bytes = ?1, status = ?2, seed_ratio = ?3
        WHERE torrent_id = ?4;",
        params![downloaded_bytes as i64, status.to_i32(), seed_ratio, id],
    )
    .map_err(|e| DatabaseException::Sqlite(format!("Failed to update torrent progress: {}", e)))?;
    Ok(())
}
