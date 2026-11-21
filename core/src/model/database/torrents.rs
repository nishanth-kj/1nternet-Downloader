use crate::model::database::downloads::{DownloadId, DownloadStatus};
use serde::{Deserialize, Serialize};

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
// Native SQL Queries & Table Constants
// -----------------------------------------------------------------------------

pub const TABLE_NAME: &str = "torrents";

pub const COL_ID: &str = "torrent_id";
pub const COL_INFO_HASH: &str = "info_hash";
pub const COL_NAME: &str = "name";
pub const COL_SAVE_PATH: &str = "save_path";
pub const COL_TOTAL_BYTES: &str = "total_bytes";
pub const COL_DOWNLOADED_BYTES: &str = "downloaded_bytes";
pub const COL_STATUS: &str = "status";
pub const COL_SEED_RATIO: &str = "seed_ratio";
pub const COL_CREATED_AT: &str = "created_at";

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

pub const INSERT_SQL: &str = "
    INSERT INTO torrents (
        info_hash, name, save_path, total_bytes, downloaded_bytes,
        status, seed_ratio, created_at
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);
";

pub const UPDATE_PROGRESS_SQL: &str = "
    UPDATE torrents
    SET downloaded_bytes = ?1, total_bytes = ?2, status = ?3, seed_ratio = ?4
    WHERE torrent_id = ?5;
";

pub const UPDATE_STATUS_SQL: &str = "
    UPDATE torrents
    SET status = ?1
    WHERE torrent_id = ?2;
";

pub const SELECT_BY_ID_SQL: &str = "
    SELECT torrent_id, info_hash, name, save_path, total_bytes, downloaded_bytes,
           status, seed_ratio, created_at
    FROM torrents
    WHERE torrent_id = ?1;
";

pub const SELECT_BY_HASH_SQL: &str = "
    SELECT torrent_id, info_hash, name, save_path, total_bytes, downloaded_bytes,
           status, seed_ratio, created_at
    FROM torrents
    WHERE info_hash = ?1;
";

pub const SELECT_ALL_SQL: &str = "
    SELECT torrent_id, info_hash, name, save_path, total_bytes, downloaded_bytes,
           status, seed_ratio, created_at
    FROM torrents
    ORDER BY created_at DESC;
";

pub const DELETE_BY_ID_SQL: &str = "
    DELETE FROM torrents
    WHERE torrent_id = ?1;
";
