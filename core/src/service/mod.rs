// -----------------------------------------------------------------------------
// Global Tokio Runtime & Async Task Dispatching
// -----------------------------------------------------------------------------

use std::future::Future;
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("idm-worker")
            .build()
            .expect("Failed to initialize multi-threaded Tokio runtime")
    })
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    get_runtime().spawn(future)
}

// -----------------------------------------------------------------------------
// Service Traits (Contracts) defined in service/
// -----------------------------------------------------------------------------
pub mod download;
pub mod fast_c;
pub mod ipc;
pub mod network;
pub mod queue;
pub mod torrent;

pub use download::{DownloadService, SchedulerService, SpeedMeterService};
pub use fast_c::FastOpsService;
pub use ipc::IpcService;
pub use network::NetworkService;
pub use queue::QueueService;
pub use torrent::TorrentService;

// -----------------------------------------------------------------------------
// Concrete Implementations & Logic from service/impl/
// -----------------------------------------------------------------------------
pub mod r#impl;

pub use r#impl::download::{DownloadTask, DynamicScheduler, SpeedMeter, WorkerAssignment};
pub use r#impl::fast_c::{fast_crc32, fast_is_all_zero, FastCOps};
pub use r#impl::ipc::{decode_idr_url, windows, WindowsIpcService};
pub use r#impl::network::NetworkClient;
pub use r#impl::queue::QueueManager;
pub use r#impl::torrent::{TorrentManager, TorrentTask};

// Models and definitions
pub use network::RemoteCapabilities;
pub use queue::QueueCommand;
pub use torrent::{TorrentSnapshot, TorrentStatus};
