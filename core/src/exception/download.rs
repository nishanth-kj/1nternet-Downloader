use crate::constants::ErrorCode;
use crate::exception::base::{AnyExceptionObject, BaseException};
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

    #[error("Download exception [{code} - {value}]: {message}")]
    Custom {
        code: i32,
        value: &'static str,
        message: String,
        object: Option<AnyExceptionObject>,
    },
}

impl DownloadException {
    pub fn custom<T: 'static + Send + Sync>(
        code: i32,
        value: &'static str,
        message: impl Into<String>,
        object: Option<T>,
    ) -> Self {
        Self::Custom {
            code,
            value,
            message: message.into(),
            object: object.map(AnyExceptionObject::new),
        }
    }
}

impl BaseException for DownloadException {
    fn code(&self) -> i32 {
        match self {
            Self::Cancelled => ErrorCode::Cancelled.code(),
            Self::Paused => ErrorCode::Paused.code(),
            Self::SegmentFailed { .. } => ErrorCode::SegmentFailed.code(),
            Self::SplitFailed(_) => ErrorCode::SplitFailed.code(),
            Self::UnknownFileSize => ErrorCode::UnknownFileSize.code(),
            Self::ChecksumMismatch { .. } => ErrorCode::VerificationFailed.code(),
            Self::WorkerStalled(_) => ErrorCode::WorkerStalled.code(),
            Self::MaxRetriesExceeded { .. } => ErrorCode::MaxRetriesExceeded.code(),
            Self::Custom { code, .. } => *code,
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::Cancelled => ErrorCode::Cancelled.value(),
            Self::Paused => ErrorCode::Paused.value(),
            Self::SegmentFailed { .. } => ErrorCode::SegmentFailed.value(),
            Self::SplitFailed(_) => ErrorCode::SplitFailed.value(),
            Self::UnknownFileSize => ErrorCode::UnknownFileSize.value(),
            Self::ChecksumMismatch { .. } => ErrorCode::VerificationFailed.value(),
            Self::WorkerStalled(_) => ErrorCode::WorkerStalled.value(),
            Self::MaxRetriesExceeded { .. } => ErrorCode::MaxRetriesExceeded.value(),
            Self::Custom { value, .. } => value,
        }
    }

    fn object(&self) -> Option<&(dyn std::any::Any + Send + Sync)> {
        match self {
            Self::Cancelled | Self::Paused | Self::UnknownFileSize => None,
            Self::SegmentFailed { segment_id, .. } => Some(segment_id),
            Self::SplitFailed(msg) => Some(msg),
            Self::ChecksumMismatch { expected, .. } => Some(expected),
            Self::WorkerStalled(msg) => Some(msg),
            Self::MaxRetriesExceeded { retries, .. } => Some(retries),
            Self::Custom { object, .. } => object.as_ref().map(|o| &*o.0 as &(dyn std::any::Any + Send + Sync)),
        }
    }
}
