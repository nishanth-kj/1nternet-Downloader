use crate::model::database::DownloadId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request payload to queue or start a new download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDownloadRequest {
    pub url: String,
    pub destination: Option<PathBuf>,
    pub auto_start: bool,
}

impl AddDownloadRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            destination: None,
            auto_start: true,
        }
    }
}

/// Action to perform on a download item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadAction {
    Pause,
    Resume,
    Cancel,
    Delete { delete_file: bool },
    Retry,
}

/// Request payload to perform an action on a specific download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadActionRequest {
    pub id: DownloadId,
    pub action: DownloadAction,
}

/// Request payload to update application settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSettingsRequest {
    pub download_dir: Option<String>,
    pub max_concurrent_downloads: Option<u32>,
    pub max_segments: Option<u32>,
    pub max_retries: Option<u32>,
    pub speed_limit_k_bps: Option<i64>,
}

/// Request payload to queue a torrent / magnet download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTorrentRequest {
    pub source: String,
    pub destination_dir: Option<PathBuf>,
}
