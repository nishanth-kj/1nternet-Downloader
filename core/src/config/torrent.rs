//! Torrent options (Options › Torrent), saved to `%APPDATA%\InternetDownloader\torrent.json`.

use crate::config::Config;
use serde::{Deserialize, Serialize};

/// Connection encryption for peers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TorrentEncryption {
    /// Only encrypted connections.
    Require,
    /// Encrypted when the peer supports it (default).
    #[default]
    Prefer,
    /// Plain connections only.
    Disable,
}

impl TorrentEncryption {
    pub const ALL: [TorrentEncryption; 3] = [TorrentEncryption::Require, TorrentEncryption::Prefer, TorrentEncryption::Disable];

    pub fn code(self) -> i32 {
        match self {
            TorrentEncryption::Require => 0,
            TorrentEncryption::Prefer => 1,
            TorrentEncryption::Disable => 2,
        }
    }
}

impl std::fmt::Display for TorrentEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TorrentEncryption::Require => "Require encryption",
            TorrentEncryption::Prefer => "Prefer encryption",
            TorrentEncryption::Disable => "No encryption",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TorrentSettings {
    // Connection
    pub listen_port: u16,
    pub random_port: bool,
    pub max_connections: u32,
    pub max_peers_per_torrent: u32,
    pub encryption: TorrentEncryption,
    // Peer discovery
    pub dht: bool,
    pub lsd: bool,
    pub pex: bool,
    pub utp: bool,
    pub upnp: bool,
    pub natpmp: bool,
    // Trackers
    pub announce_to_all_trackers: bool,
    pub validate_https_trackers: bool,
    /// Added to every new torrent, one URL per line.
    pub extra_trackers: String,
    // Speed (KB/s, 0 = unlimited)
    pub download_limit_kbps: u32,
    pub upload_limit_kbps: u32,
    // Downloading
    pub sequential_by_default: bool,
    // Seeding
    pub seed_finished: bool,
    /// Stop seeding at this upload/download ratio (0 = no limit).
    pub seed_ratio_limit: f64,
    /// Stop seeding after this many minutes (0 = no limit).
    pub seed_time_limit_minutes: u32,
    pub user_agent: String,
}

impl Default for TorrentSettings {
    fn default() -> Self {
        Self {
            listen_port: 6881,
            random_port: false,
            max_connections: 200,
            max_peers_per_torrent: 50,
            encryption: TorrentEncryption::Prefer,
            dht: true,
            lsd: true,
            pex: true,
            utp: true,
            upnp: true,
            natpmp: true,
            announce_to_all_trackers: false,
            validate_https_trackers: true,
            extra_trackers: String::new(),
            download_limit_kbps: 0,
            upload_limit_kbps: 0,
            sequential_by_default: false,
            seed_finished: true,
            seed_ratio_limit: 0.0,
            seed_time_limit_minutes: 0,
            user_agent: "1nternet Downloader".into(),
        }
    }
}

impl TorrentSettings {
    fn path() -> std::path::PathBuf {
        Config::app_data_dir().join("torrent.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    pub fn extra_tracker_list(&self) -> Vec<String> {
        self.extra_trackers
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    }
}
