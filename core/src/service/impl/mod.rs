pub mod download;
pub mod fast_c;
pub mod ipc;
pub mod network;
pub mod queue;
pub mod torrent;

pub use download::{DownloadTask, DynamicScheduler, SpeedMeter, WorkerAssignment};
pub use fast_c::{fast_crc32, fast_is_all_zero, FastCOps};
pub use ipc::{decode_idr_url, windows, WindowsIpcService};
pub use network::NetworkClient;
pub use queue::QueueManager;
pub use torrent::{TorrentManager, TorrentTask};
