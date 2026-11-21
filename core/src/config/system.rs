use std::time::Duration;

pub const DEFAULT_MAX_CONCURRENT_DOWNLOADS: u32 = 3;
pub const DEFAULT_MAX_SEGMENTS: u32 = 8;
pub const DEFAULT_MAX_RETRIES: u32 = 3;

pub const MIN_CONCURRENT_DOWNLOADS: u32 = 1;
pub const MAX_CONCURRENT_DOWNLOADS: u32 = 16;

pub const MIN_SEGMENTS: u32 = 1;
pub const MAX_SEGMENTS: u32 = 32;

pub const MIN_RETRIES: u32 = 0;
pub const MAX_RETRIES_LIMIT: u32 = 20;

pub const QUEUE_EVENT_CHANNEL_CAPACITY: usize = 64;
pub const SNAPSHOT_CHANNEL_CAPACITY: usize = 32;

pub const IPC_RETRY_COUNT: usize = 10;
pub const IPC_RETRY_DELAY_MS: u64 = 50;
pub const IPC_RETRY_DELAY: Duration = Duration::from_millis(IPC_RETRY_DELAY_MS);

pub const DEFAULT_WINDOW_WIDTH: f32 = 1100.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 720.0;
pub const UI_TICK_INTERVAL_MS: u64 = 250;
pub const UI_TICK_INTERVAL: Duration = Duration::from_millis(UI_TICK_INTERVAL_MS);
