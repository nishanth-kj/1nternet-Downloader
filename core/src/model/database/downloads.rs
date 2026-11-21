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
// Native SQL Queries & Table Constants
// -----------------------------------------------------------------------------

pub const TABLE_NAME: &str = "downloads";

pub const COL_ID: &str = "downloads_id";
pub const COL_URL: &str = "url";
pub const COL_DESTINATION: &str = "destination";
pub const COL_FILENAME: &str = "filename";
pub const COL_TOTAL_BYTES: &str = "total_bytes";
pub const COL_DOWNLOADED_BYTES: &str = "downloaded_bytes";
pub const COL_STATUS: &str = "status";
pub const COL_MIME_TYPE: &str = "mime_type";
pub const COL_CHECKSUM: &str = "checksum";
pub const COL_CREATED_AT: &str = "created_at";
pub const COL_UPDATED_AT: &str = "updated_at";

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

pub const INSERT_SQL: &str = "
    INSERT INTO downloads (
        url, destination, filename, total_bytes, downloaded_bytes,
        status, mime_type, checksum, created_at, updated_at
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);
";

pub const UPDATE_PROGRESS_SQL: &str = "
    UPDATE downloads
    SET downloaded_bytes = ?1, total_bytes = ?2, status = ?3, updated_at = ?4
    WHERE downloads_id = ?5;
";

pub const UPDATE_STATUS_SQL: &str = "
    UPDATE downloads
    SET status = ?1, updated_at = ?2
    WHERE downloads_id = ?3;
";

pub const SELECT_BY_ID_SQL: &str = "
    SELECT downloads_id, url, destination, filename, total_bytes, downloaded_bytes,
           status, mime_type, checksum, created_at, updated_at
    FROM downloads
    WHERE downloads_id = ?1;
";

pub const SELECT_ALL_SQL: &str = "
    SELECT downloads_id, url, destination, filename, total_bytes, downloaded_bytes,
           status, mime_type, checksum, created_at, updated_at
    FROM downloads
    ORDER BY created_at DESC;
";

pub const DELETE_BY_ID_SQL: &str = "
    DELETE FROM downloads
    WHERE downloads_id = ?1;
";
