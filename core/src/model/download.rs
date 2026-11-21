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

/// Download item representation directly mapping to SQLite `downloads` table
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

/// Real-time segment representation for active dynamic downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSegment {
    pub segment_id: u32,
    pub start_offset: u64,
    pub end_offset: u64,
    pub downloaded_bytes: u64,
    pub is_active: bool,
    pub is_completed: bool,
}

impl DownloadSegment {
    pub fn total_bytes(&self) -> u64 {
        if self.end_offset >= self.start_offset {
            (self.end_offset - self.start_offset) + 1
        } else {
            0
        }
    }

    pub fn remaining_bytes(&self) -> u64 {
        self.total_bytes().saturating_sub(self.downloaded_bytes)
    }

    pub fn progress_ratio(&self) -> f32 {
        let total = self.total_bytes();
        if total == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f32 / total as f32).clamp(0.0, 1.0)
        }
    }
}

/// Complete runtime snapshot emitted to UI and event subscribers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSnapshot {
    pub id: DownloadId,
    pub url: String,
    pub destination: String,
    pub filename: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub status: DownloadStatus,
    pub segment_count: usize,
    pub segments: Vec<DownloadSegment>,
    pub is_torrent: bool,
    pub peers: u32,
    pub seeds: u32,
    pub error_message: Option<String>,
    pub progress_percent: f32,
    pub eta_seconds: Option<u64>,
}

impl DownloadSnapshot {
    pub fn calculate_progress(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            ((self.downloaded_bytes as f64 / self.total_bytes as f64) * 100.0) as f32
        }
    }

    pub fn calculate_eta(&self) -> Option<u64> {
        if self.speed_bytes_per_sec <= 0.0 || self.total_bytes == 0 || self.downloaded_bytes >= self.total_bytes {
            None
        } else {
            let remaining = self.total_bytes.saturating_sub(self.downloaded_bytes);
            Some((remaining as f64 / self.speed_bytes_per_sec) as u64)
        }
    }
}
