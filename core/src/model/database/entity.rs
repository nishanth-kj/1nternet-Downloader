use crate::model::download::{DownloadId, DownloadStatus};
use serde::{Deserialize, Serialize};

/// Download item entity mapping directly to SQLite `downloads` table
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

/// Torrent metadata record stored in database
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
