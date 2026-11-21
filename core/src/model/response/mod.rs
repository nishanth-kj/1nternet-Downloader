use crate::model::database::{DownloadId, DownloadStatus};
use serde::{Deserialize, Serialize};

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
        if self.status == DownloadStatus::Completed {
            return Some(0);
        }
        if self.speed_bytes_per_sec <= 0.0 || self.total_bytes == 0 {
            return None;
        }
        let remaining = self.total_bytes.saturating_sub(self.downloaded_bytes);
        Some((remaining as f64 / self.speed_bytes_per_sec) as u64)
    }
}

/// Segment status response for real-time visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSegmentResponse {
    pub segment_id: u32,
    pub start_offset: u64,
    pub end_offset: u64,
    pub downloaded_bytes: u64,
    pub progress_ratio: f32,
    pub is_active: bool,
    pub is_completed: bool,
}

impl From<&DownloadSegment> for DownloadSegmentResponse {
    fn from(s: &DownloadSegment) -> Self {
        Self {
            segment_id: s.segment_id,
            start_offset: s.start_offset,
            end_offset: s.end_offset,
            downloaded_bytes: s.downloaded_bytes,
            progress_ratio: s.progress_ratio(),
            is_active: s.is_active,
            is_completed: s.is_completed,
        }
    }
}

/// Server capability response from initial HTTP HEAD probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilitiesResponse {
    pub total_size: Option<u64>,
    pub supports_ranges: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
    pub suggested_filename: Option<String>,
}

/// IPC message response delivered over named pipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub accepted: bool,
    pub download_id: Option<DownloadId>,
    pub message: String,
}

/// Queue summary metrics response
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueSummaryResponse {
    pub total_downloads: usize,
    pub active_downloads: usize,
    pub queued_downloads: usize,
    pub completed_downloads: usize,
    pub paused_downloads: usize,
    pub failed_downloads: usize,
    pub total_speed_bps: f64,
}
