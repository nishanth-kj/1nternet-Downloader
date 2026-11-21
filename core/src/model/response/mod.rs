use crate::model::download::{DownloadId, DownloadSegment};
use serde::{Deserialize, Serialize};

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
