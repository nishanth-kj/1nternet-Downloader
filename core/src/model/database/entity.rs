use serde::{Deserialize, Serialize};

pub type DownloadId = i64;

/// Download status matching exact database integer values:
/// 0=Queued, 1=Downloading, 2=Paused, 3=Completed, 4=Error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum DownloadStatus {
    Queued = 0,
    Downloading = 1,
    Paused = 2,
    Completed = 3,
    Error = 4,
}

impl DownloadStatus {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => DownloadStatus::Queued,
            1 => DownloadStatus::Downloading,
            2 => DownloadStatus::Paused,
            3 => DownloadStatus::Completed,
            _ => DownloadStatus::Error,
        }
    }

    pub fn to_i32(self) -> i32 {
        self as i32
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadStatus::Queued => "Queued",
            DownloadStatus::Downloading => "Downloading",
            DownloadStatus::Paused => "Paused",
            DownloadStatus::Completed => "Completed",
            DownloadStatus::Error => "Error",
        }
    }
}

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

/// Torrent metadata record stored in SQLite `torrents` table
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
