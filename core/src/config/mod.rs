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
    3
}
fn default_max_segments() -> u32 {
    8
}
fn default_max_retries() -> u32 {
    3
}

impl Default for Settings {
    fn default() -> Self {
        let download_dir = directories::UserDirs::new()
            .and_then(|u| u.download_dir().map(|p| p.to_string_lossy().to_string()))
            .unwrap_or_else(|| "C:\\Downloads".to_string());

        Self {
            download_dir,
            max_concurrent_downloads: 3,
            max_segments: 8,
            max_retries: 3,
            speed_limit_k_bps: 0,
        }
    }
}

impl Settings {
    pub fn normalize(&mut self) {
        self.max_concurrent_downloads = self.max_concurrent_downloads.clamp(1, 16);
        self.max_segments = self.max_segments.clamp(1, 32);
        self.max_retries = self.max_retries.clamp(0, 20);
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

    pub fn get(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    pub fn update<F>(&self, f: F) -> crate::exception::Result<Settings>
    where
        F: FnOnce(&mut Settings),
    {
        let mut settings = self.settings.write().unwrap();
        f(&mut settings);
        settings.normalize();

        let json = serde_json::to_string_pretty(&*settings)
            .map_err(|e| crate::exception::IdmException::Validation(e.to_string()))?;
        fs::write(&self.path, json).map_err(crate::exception::IdmException::from)?;

        Ok(settings.clone())
    }
}
