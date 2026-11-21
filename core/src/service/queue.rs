use crate::model::{DownloadId, DownloadSnapshot};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

pub enum QueueCommand {
    Add {
        url: String,
        destination: Option<PathBuf>,
        auto_start: bool,
    },
    Pause(DownloadId),
    Resume(DownloadId),
    Delete {
        id: DownloadId,
        delete_file: bool,
    },
    PauseAll,
    ResumeAll,
    SyncDatabase,
}

/// Service trait defining queue scheduler contracts
pub trait QueueService: Send + Sync {
    fn subscribe(&self) -> broadcast::Receiver<DownloadSnapshot>;
    fn pause_download(&self, id: DownloadId);
    fn resume_download(self: &Arc<Self>, id: DownloadId);
    fn delete_download(&self, id: DownloadId, delete_file: bool);
    fn pause_all(&self);
    fn resume_all(self: &Arc<Self>);
    fn get_all_snapshots(&self) -> Vec<DownloadSnapshot>;
}

pub use crate::service::r#impl::queue::QueueManager;
