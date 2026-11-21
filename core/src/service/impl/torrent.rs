use crate::exception::Result;
use crate::service::torrent::{TorrentService, TorrentSnapshot, TorrentStatus};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

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

impl TorrentService for TorrentManager {
    fn is_torrent_url(&self, url: &str) -> bool {
        Self::is_torrent_url(url)
    }

    fn add_torrent(&self, source: &str, destination_dir: &Path) -> Result<Arc<TorrentTask>> {
        self.add_torrent(source, destination_dir)
    }
}
