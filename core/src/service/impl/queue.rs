use crate::config::Settings;
use crate::exception::Result;
use crate::model::{DownloadId, DownloadItem, DownloadSnapshot, DownloadStatus};
use crate::service::database::DatabaseService;
use crate::service::r#impl::download::DownloadTask;
use crate::service::queue::QueueService;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::info;

struct ManagedTask {
    task: Arc<DownloadTask>,
    #[allow(dead_code)]
    handle: Option<JoinHandle<()>>,
}

pub struct QueueManager {
    database: Arc<dyn DatabaseService>,
    tasks: Arc<RwLock<HashMap<DownloadId, ManagedTask>>>,
    settings: Arc<RwLock<Settings>>,
    event_tx: broadcast::Sender<DownloadSnapshot>,
}

impl QueueManager {
    pub fn new(database: Arc<dyn DatabaseService>, settings: Settings) -> Arc<Self> {
        let (event_tx, _) = broadcast::channel(64);
        let qm = Arc::new(Self {
            database,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(settings)),
            event_tx,
        });

        qm.load_history();
        qm
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DownloadSnapshot> {
        QueueService::subscribe(self)
    }

    fn load_history(&self) {
        if let Ok(items) = self.database.get_all_downloads() {
            info!("Loaded {} items from database history", items.len());
            let mut tasks = self.tasks.write();
            let settings = self.settings.read().clone();

            for item in items {
                let dest = PathBuf::from(&item.destination);
                if let Ok(task) = DownloadTask::new(item.id, item.url, dest, settings.clone()) {
                    tasks.insert(
                        item.id,
                        ManagedTask {
                            task: Arc::new(task),
                            handle: None,
                        },
                    );
                }
            }
        }
    }

    pub async fn add_download(
        self: &Arc<Self>,
        url: &str,
        destination: Option<PathBuf>,
        auto_start: bool,
    ) -> Result<DownloadId> {
        let settings = self.settings.read().clone();
        let filename = crate::service::network::NetworkClient::extract_filename_from_url(url);
        let dest_dir = destination.unwrap_or_else(|| PathBuf::from(&settings.download_dir));
        let full_path = dest_dir.join(&filename);

        let now = chrono::Utc::now().timestamp();
        let item = DownloadItem {
            id: 0,
            url: url.to_string(),
            destination: full_path.to_string_lossy().to_string(),
            filename: filename.clone(),
            total_bytes: 0,
            downloaded_bytes: 0,
            status: if auto_start {
                DownloadStatus::Downloading
            } else {
                DownloadStatus::Queued
            },
            mime_type: None,
            checksum: None,
            created_at: now,
            updated_at: now,
        };

        let id = self.database.insert_download(&item)?;

        let task = Arc::new(DownloadTask::new(
            id,
            url.to_string(),
            full_path,
            settings,
        )?);

        let mut sub = task.subscribe();
        let event_tx_clone = self.event_tx.clone();
        let db_clone = Arc::clone(&self.database);

        crate::service::spawn(async move {
            while let Ok(snap) = sub.recv().await {
                let _ = event_tx_clone.send(snap.clone());
                if snap.status == DownloadStatus::Completed || snap.status == DownloadStatus::Error {
                    let _ = db_clone.update_progress(
                        snap.id,
                        snap.downloaded_bytes,
                        snap.total_bytes,
                        snap.status,
                    );
                }
            }
        });

        let mut handle = None;
        if auto_start {
            let task_clone = Arc::clone(&task);
            handle = Some(crate::service::spawn(async move {
                let _ = task_clone.run().await;
            }));
        }

        self.tasks.write().insert(
            id,
            ManagedTask {
                task,
                handle,
            },
        );

        Ok(id)
    }

    pub fn pause_download(&self, id: DownloadId) {
        QueueService::pause_download(self, id);
    }

    pub fn resume_download(self: &Arc<Self>, id: DownloadId) {
        QueueService::resume_download(self, id);
    }

    pub fn delete_download(&self, id: DownloadId, delete_file: bool) {
        QueueService::delete_download(self, id, delete_file);
    }

    pub fn pause_all(&self) {
        QueueService::pause_all(self);
    }

    pub fn resume_all(self: &Arc<Self>) {
        QueueService::resume_all(self);
    }

    pub fn get_all_snapshots(&self) -> Vec<DownloadSnapshot> {
        QueueService::get_all_snapshots(self)
    }
}

impl QueueService for QueueManager {
    fn subscribe(&self) -> broadcast::Receiver<DownloadSnapshot> {
        self.event_tx.subscribe()
    }

    fn pause_download(&self, id: DownloadId) {
        if let Some(m) = self.tasks.read().get(&id) {
            m.task.pause();
            let _ = self.database.update_status(id, DownloadStatus::Paused);
        }
    }

    fn resume_download(self: &Arc<Self>, id: DownloadId) {
        let task_opt = self.tasks.read().get(&id).map(|m| Arc::clone(&m.task));
        if let Some(task) = task_opt {
            task.resume();
            let _ = self.database.update_status(id, DownloadStatus::Downloading);

            let task_clone = Arc::clone(&task);
            let handle = crate::service::spawn(async move {
                let _ = task_clone.run().await;
            });

            if let Some(m) = self.tasks.write().get_mut(&id) {
                m.handle = Some(handle);
            }
        }
    }

    fn delete_download(&self, id: DownloadId, delete_file: bool) {
        let task_opt = self.tasks.write().remove(&id);
        if let Some(m) = task_opt {
            m.task.pause();
            if delete_file && m.task.destination.exists() {
                let _ = std::fs::remove_file(&m.task.destination);
            }
        }
        let _ = self.database.delete_download(id);
    }

    fn pause_all(&self) {
        let tasks = self.tasks.read();
        for m in tasks.values() {
            m.task.pause();
            let _ = self.database.update_status(m.task.id, DownloadStatus::Paused);
        }
    }

    fn resume_all(self: &Arc<Self>) {
        let ids: Vec<DownloadId> = self.tasks.read().keys().copied().collect();
        for id in ids {
            self.resume_download(id);
        }
    }

    fn get_all_snapshots(&self) -> Vec<DownloadSnapshot> {
        let tasks = self.tasks.read();
        tasks.values().map(|m| m.task.get_snapshot()).collect()
    }
}
