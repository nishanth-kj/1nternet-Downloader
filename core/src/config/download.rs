use std::time::Duration;

pub const MIN_SEGMENT_SIZE_BYTES: u64 = 1024 * 1024; // 1 MB minimum segment
pub const MIN_SPLIT_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024; // 2 MB minimum to allow work-stealing split

pub const MIN_CONNECTIONS: usize = 1;
pub const DEFAULT_CONNECTIONS: usize = 8;
pub const MAX_CONNECTIONS: usize = 16;
pub const MAX_SEGMENTS_LIMIT: usize = 32;

pub const SLIDING_WINDOW_SECS: u64 = 2;
pub const SLIDING_WINDOW_DURATION: Duration = Duration::from_secs(SLIDING_WINDOW_SECS);
pub const MIN_SPEED_WINDOW_SAMPLES: usize = 2;
pub const SPEED_SAMPLE_INTERVAL_SECS: f64 = 0.05;

pub const PROGRESS_POLL_INTERVAL_MS: u64 = 250;
pub const PROGRESS_POLL_INTERVAL: Duration = Duration::from_millis(PROGRESS_POLL_INTERVAL_MS);

pub const WORKER_RETRY_DELAY_MS: u64 = 500;
pub const WORKER_RETRY_DELAY: Duration = Duration::from_millis(WORKER_RETRY_DELAY_MS);
