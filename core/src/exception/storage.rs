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
}

impl Clone for StorageException {
    fn clone(&self) -> Self {
        match self {
            Self::Io(e) => Self::Io(std::io::Error::new(e.kind(), e.to_string())),
            Self::InsufficientSpace { path, required_bytes, available_bytes } => Self::InsufficientSpace {
                path: path.clone(),
                required_bytes: *required_bytes,
                available_bytes: *available_bytes,
            },
            Self::PreallocationFailed { bytes, reason } => Self::PreallocationFailed {
                bytes: *bytes,
                reason: reason.clone(),
            },
            Self::InvalidPath(s) => Self::InvalidPath(s.clone()),
        }
    }
}
