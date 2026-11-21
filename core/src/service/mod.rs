pub mod database;
pub mod download;
pub mod fast_c;
pub mod r#impl;
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

// Traits
pub use database::DatabaseService;
pub use download::DownloadService;
pub use network::NetworkService;
pub use queue::QueueService;
pub use torrent::TorrentService;

// Concrete implementations from service/impl
pub use r#impl::database::SqliteDatabaseService;
pub use r#impl::download::DownloadTask;
pub use r#impl::network::NetworkClient;
pub use r#impl::queue::QueueManager;
pub use r#impl::torrent::{TorrentManager, TorrentTask};

// Backwards-compatible alias for primary database implementation
pub type Database = SqliteDatabaseService;

// Algorithms and structures
pub use download::{DynamicScheduler, SpeedMeter, WorkerAssignment};
pub use fast_c::{fast_crc32, fast_is_all_zero};
pub use ipc::{decode_idr_url, windows};
pub use network::RemoteCapabilities;
pub use queue::QueueCommand;
pub use torrent::{TorrentSnapshot, TorrentStatus};
