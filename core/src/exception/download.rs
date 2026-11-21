use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DownloadException {
    #[error("Download was cancelled by user")]
    Cancelled,

    #[error("Download was paused")]
    Paused,

    #[error("Segment #{segment_id} failed: {reason}")]
    SegmentFailed { segment_id: u32, reason: String },

    #[error("Dynamic split failed: {0}")]
    SplitFailed(String),

    #[error("File size undetermined (missing Content-Length)")]
    UnknownFileSize,

    #[error("Integrity verification failed for file: expected {expected}, actual {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Speed stalled or worker timed out: {0}")]
    WorkerStalled(String),

    #[error("Maximum retries ({retries}) exceeded: {last_error}")]
    MaxRetriesExceeded { retries: u32, last_error: String },
}
