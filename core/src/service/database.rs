use crate::exception::Result;
use crate::model::{DownloadId, DownloadItem, DownloadStatus};
use std::path::Path;

/// Service trait defining persistence contract for download history and state
pub trait DatabaseService: Send + Sync {
    fn insert_download(&self, item: &DownloadItem) -> Result<DownloadId>;
    fn update_progress(
        &self,
        id: DownloadId,
        downloaded_bytes: u64,
        total_bytes: u64,
        status: DownloadStatus,
    ) -> Result<()>;
    fn update_status(&self, id: DownloadId, status: DownloadStatus) -> Result<()>;
    fn get_download(&self, id: DownloadId) -> Result<Option<DownloadItem>>;
    fn get_all_downloads(&self) -> Result<Vec<DownloadItem>>;
    fn delete_download(&self, id: DownloadId) -> Result<()>;
    fn path(&self) -> &Path;
}
