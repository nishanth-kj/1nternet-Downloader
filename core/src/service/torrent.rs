use crate::exception::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TorrentStatus {
    Queued = 0,
    Downloading = 1,
    Paused = 2,
    Completed = 3,
    Seeding = 4,
    Error = 5,
}

impl TorrentStatus {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => TorrentStatus::Queued,
            1 => TorrentStatus::Downloading,
            2 => TorrentStatus::Paused,
            3 => TorrentStatus::Completed,
            4 => TorrentStatus::Seeding,
            _ => TorrentStatus::Error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentSnapshot {
    pub name: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub peers: u32,
    pub seeds: u32,
    pub status: TorrentStatus,
}

pub struct TorrentTask {
    pub id: i64,
    pub source: String,
    pub destination_dir: PathBuf,
}

impl TorrentTask {
    pub fn get_snapshot(&self) -> TorrentSnapshot {
        TorrentSnapshot {
            name: self.source.clone(),
            total_bytes: 0,
            downloaded_bytes: 0,
            uploaded_bytes: 0,
            download_speed: 0.0,
            upload_speed: 0.0,
            peers: 0,
            seeds: 0,
            status: TorrentStatus::Queued,
        }
    }

    pub fn pause(&self) {}
    pub fn resume(&self) {}
    pub fn remove(&self) {}
}

pub struct TorrentManager {
    _private: (),
}

impl TorrentManager {
    pub fn new() -> Self {
        info!("Initialized Rust TorrentManager abstraction");
        Self { _private: () }
    }

    pub fn is_torrent_url(url: &str) -> bool {
        url.starts_with("magnet:?") || url.ends_with(".torrent")
    }

    pub fn add_torrent<P: AsRef<Path>>(&self, source: &str, destination_dir: P) -> Result<Arc<TorrentTask>> {
        let dest = destination_dir.as_ref();
        info!("Adding torrent: {} into {:?}", source, dest);

        let task_id = chrono::Utc::now().timestamp_micros();
        Ok(Arc::new(TorrentTask {
            id: task_id,
            source: source.to_string(),
            destination_dir: dest.to_path_buf(),
        }))
    }
}

impl Default for TorrentManager {
    fn default() -> Self {
        Self::new()
    }
}
