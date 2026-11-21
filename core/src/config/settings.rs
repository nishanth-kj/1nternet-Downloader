use crate::config::system::{
    DEFAULT_MAX_CONCURRENT_DOWNLOADS, DEFAULT_MAX_RETRIES, DEFAULT_MAX_SEGMENTS,
    MAX_CONCURRENT_DOWNLOADS, MAX_RETRIES_LIMIT, MAX_SEGMENTS, MIN_CONCURRENT_DOWNLOADS,
    MIN_RETRIES, MIN_SEGMENTS,
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub download_dir: String,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_downloads: u32,
    #[serde(default = "default_max_segments")]
    pub max_segments: u32,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default)]
    pub speed_limit_k_bps: i64,
}

fn default_max_concurrent() -> u32 {
    DEFAULT_MAX_CONCURRENT_DOWNLOADS
}
fn default_max_segments() -> u32 {
    DEFAULT_MAX_SEGMENTS
}
fn default_max_retries() -> u32 {
    DEFAULT_MAX_RETRIES
}

impl Default for Settings {
    fn default() -> Self {
        let download_dir = directories::UserDirs::new()
            .and_then(|u| u.download_dir().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| "C:\\Downloads".to_string());

        Self {
            download_dir,
            max_concurrent_downloads: DEFAULT_MAX_CONCURRENT_DOWNLOADS,
            max_segments: DEFAULT_MAX_SEGMENTS,
            max_retries: DEFAULT_MAX_RETRIES,
            speed_limit_k_bps: 0,
        }
    }
}

impl Settings {
    pub fn normalize(&mut self) {
        self.max_concurrent_downloads = self
            .max_concurrent_downloads
            .clamp(MIN_CONCURRENT_DOWNLOADS, MAX_CONCURRENT_DOWNLOADS);
        self.max_segments = self.max_segments.clamp(MIN_SEGMENTS, MAX_SEGMENTS);
        self.max_retries = self.max_retries.clamp(MIN_RETRIES, MAX_RETRIES_LIMIT);
        if self.speed_limit_k_bps < 0 {
            self.speed_limit_k_bps = 0;
        }
    }
}

#[derive(Clone)]
pub struct Config {
    settings: Arc<RwLock<Settings>>,
    path: PathBuf,
}

impl Config {
    pub fn app_data_dir() -> PathBuf {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata).join("InternetDownloader")
        } else if let Some(proj_dirs) = ProjectDirs::from("com", "InternetDownloader", "InternetDownloader") {
            proj_dirs.data_dir().to_path_buf()
        } else {
            PathBuf::from("C:\\ProgramData\\InternetDownloader")
        }
    }

    pub fn database_path() -> PathBuf {
        Self::app_data_dir().join("downloads.db")
    }

    pub fn logs_dir() -> PathBuf {
        Self::app_data_dir().join("logs")
    }

    pub fn load() -> Self {
        let dir = Self::app_data_dir();
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("settings.json");

        let mut settings = Settings::default();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(loaded) = serde_json::from_str::<Settings>(&content) {
                    settings = loaded;
                }
            }
        }
        settings.normalize();

        Self {
            settings: Arc::new(RwLock::new(settings)),
            path,
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    pub fn get(&self) -> Settings {
        self.settings()
    }

    pub fn update<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut Settings),
    {
        let mut settings = self.settings.write().unwrap();
        f(&mut settings);
        settings.normalize();

        let json = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&self.path, json).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }
}
