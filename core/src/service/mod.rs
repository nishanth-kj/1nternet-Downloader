pub mod download;
pub mod fast_c;
pub mod ipc;
pub mod network;
pub mod queue;
pub mod torrent;

use std::sync::OnceLock;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Returns the shared high-performance multi-threaded Tokio runtime for the 1IDM engine
pub fn get_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .expect("Failed to initialize multi-threaded Tokio runtime for 1IDM engine")
    })
}

/// Spawns an async task onto the background 1IDM engine pool from any thread (UI, IPC, Worker)
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    get_runtime().spawn(future)
}

pub use download::{DownloadTask, DynamicScheduler, SpeedMeter, WorkerAssignment};
pub use fast_c::{fast_crc32, fast_is_all_zero};
pub use ipc::{decode_idr_url, windows};
pub use network::{NetworkClient, RemoteCapabilities};
pub use queue::{QueueCommand, QueueManager};
pub use torrent::{TorrentManager, TorrentSnapshot, TorrentStatus, TorrentTask};
