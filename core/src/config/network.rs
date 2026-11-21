use std::time::Duration;

pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
pub const CONNECT_TIMEOUT_SECS: u64 = 10;
pub const POOL_IDLE_TIMEOUT_SECS: u64 = 90;
pub const POOL_MAX_IDLE_PER_HOST: usize = 16;
pub const MIN_SEGMENT_DOWNLOAD_SIZE: u64 = 1024 * 512; // 512 KB to enable dynamic multi-part

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(DEFAULT_TIMEOUT_SECS);
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(CONNECT_TIMEOUT_SECS);
pub const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(POOL_IDLE_TIMEOUT_SECS);

pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 1IDM/1.0";
