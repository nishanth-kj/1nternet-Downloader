use crate::exception::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

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

/// Service trait defining BitTorrent operation contract
pub trait TorrentService: Send + Sync {
    fn is_torrent_url(&self, url: &str) -> bool;
    fn add_torrent(&self, source: &str, destination_dir: &Path) -> Result<Arc<crate::service::r#impl::torrent::TorrentTask>>;
}

pub use crate::service::r#impl::torrent::{TorrentManager, TorrentTask};
