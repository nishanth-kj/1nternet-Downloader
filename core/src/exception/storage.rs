use crate::constants::ErrorCode;
use crate::exception::base::{AnyExceptionObject, BaseException};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageException {
    #[error("File I/O error: {0}")]
    Io(std::io::Error),

    #[error("Insufficient disk space at {path}: required {required_bytes} B, available {available_bytes} B")]
    InsufficientSpace {
        path: String,
        required_bytes: u64,
        available_bytes: u64,
    },

    #[error("Failed to pre-allocate sparse file of size {bytes} B: {reason}")]
    PreallocationFailed { bytes: u64, reason: String },

    #[error("Invalid destination path: {0}")]
    InvalidPath(String),

    #[error("Storage exception [{code} - {value}]: {message}")]
    Custom {
        code: i32,
        value: &'static str,
        message: String,
        object: Option<AnyExceptionObject>,
    },
}

impl Clone for StorageException {
    fn clone(&self) -> Self {
        match self {
            Self::Io(e) => Self::Io(std::io::Error::new(e.kind(), e.to_string())),
            Self::InsufficientSpace {
                path,
                required_bytes,
                available_bytes,
            } => Self::InsufficientSpace {
                path: path.clone(),
                required_bytes: *required_bytes,
                available_bytes: *available_bytes,
            },
            Self::PreallocationFailed { bytes, reason } => Self::PreallocationFailed {
                bytes: *bytes,
                reason: reason.clone(),
            },
            Self::InvalidPath(s) => Self::InvalidPath(s.clone()),
            Self::Custom {
                code,
                value,
                message,
                object,
            } => Self::Custom {
                code: *code,
                value,
                message: message.clone(),
                object: object.clone(),
            },
        }
    }
}

impl StorageException {
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

impl BaseException for StorageException {
    fn code(&self) -> i32 {
        match self {
            Self::Io(_) => ErrorCode::IoError.code(),
            Self::InsufficientSpace { .. } => ErrorCode::DiskFull.code(),
            Self::PreallocationFailed { .. } => ErrorCode::PreallocationFailed.code(),
            Self::InvalidPath(_) => ErrorCode::InvalidPath.code(),
            Self::Custom { code, .. } => *code,
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::Io(_) => ErrorCode::IoError.value(),
            Self::InsufficientSpace { .. } => ErrorCode::DiskFull.value(),
            Self::PreallocationFailed { .. } => ErrorCode::PreallocationFailed.value(),
            Self::InvalidPath(_) => ErrorCode::InvalidPath.value(),
            Self::Custom { value, .. } => value,
        }
    }

    fn object(&self) -> Option<&(dyn std::any::Any + Send + Sync)> {
        match self {
            Self::Io(e) => Some(e),
            Self::InsufficientSpace { path, .. } => Some(path),
            Self::PreallocationFailed { bytes, .. } => Some(bytes),
            Self::InvalidPath(path) => Some(path),
            Self::Custom { object, .. } => object.as_ref().map(|o| &*o.0 as &(dyn std::any::Any + Send + Sync)),
        }
    }
}
