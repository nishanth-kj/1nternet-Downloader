use crate::exception::Result;
use crate::model::DownloadSnapshot;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::broadcast;

/// Service trait defining active download lifecycle contract
pub trait DownloadService: Send + Sync {
    fn subscribe(&self) -> broadcast::Receiver<DownloadSnapshot>;
    fn get_snapshot(&self) -> DownloadSnapshot;
    fn pause(&self);
    fn resume(&self);
    fn run(&self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

/// Service trait defining dynamic work-stealing scheduler contract
pub trait SchedulerService: Send + Sync {
    fn request_work(&mut self) -> Option<crate::service::r#impl::download::WorkerAssignment>;
    fn record_progress(&mut self, segment_id: u32, bytes_written: u64);
    fn is_all_completed(&self) -> bool;
    fn get_segments(&self) -> Vec<crate::model::DownloadSegment>;
}

/// Service trait defining speed calculation contract
pub trait SpeedMeterService: Send + Sync {
    fn update(&mut self, current_bytes: u64) -> f64;
}
