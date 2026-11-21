pub mod base;
pub mod database;
pub mod download;
pub mod network;
pub mod protocol;
pub mod storage;

pub use base::{AnyExceptionObject, BaseError, BaseException};
pub use database::DatabaseException;
pub use download::DownloadException;
pub use network::NetworkException;
pub use protocol::ProtocolException;
pub use storage::StorageException;

use crate::constants::ErrorCode;
use thiserror::Error;

/// Central application exception hierarchy for 1nternet Downloader
#[derive(Error, Debug, Clone)]
pub enum IdmException {
    #[error(transparent)]
    Network(#[from] NetworkException),

    #[error(transparent)]
    Download(#[from] DownloadException),

    #[error(transparent)]
    Database(#[from] DatabaseException),

    #[error(transparent)]
    Storage(#[from] StorageException),

    #[error(transparent)]
    Protocol(#[from] ProtocolException),

    #[error(transparent)]
    Base(#[from] BaseError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("General exception: {0}")]
    Other(String),
}

impl BaseException for IdmException {
    fn code(&self) -> i32 {
        match self {
            Self::Network(e) => e.code(),
            Self::Download(e) => e.code(),
            Self::Database(e) => e.code(),
            Self::Storage(e) => e.code(),
            Self::Protocol(e) => e.code(),
            Self::Base(e) => e.code(),
            Self::Config(_) => ErrorCode::ConfigError.code(),
            Self::Validation(_) => ErrorCode::ValidationError.code(),
            Self::Other(_) => ErrorCode::Unknown.code(),
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::Network(e) => e.value(),
            Self::Download(e) => e.value(),
            Self::Database(e) => e.value(),
            Self::Storage(e) => e.value(),
            Self::Protocol(e) => e.value(),
            Self::Base(e) => e.value(),
            Self::Config(_) => ErrorCode::ConfigError.value(),
            Self::Validation(_) => ErrorCode::ValidationError.value(),
            Self::Other(_) => ErrorCode::Unknown.value(),
        }
    }

    fn object(&self) -> Option<&(dyn std::any::Any + Send + Sync)> {
        match self {
            Self::Network(e) => e.object(),
            Self::Download(e) => e.object(),
            Self::Database(e) => e.object(),
            Self::Storage(e) => e.object(),
            Self::Protocol(e) => e.object(),
            Self::Base(e) => e.object(),
            Self::Config(msg) => Some(msg),
            Self::Validation(msg) => Some(msg),
            Self::Other(msg) => Some(msg),
        }
    }
}

impl From<std::io::Error> for IdmException {
    fn from(err: std::io::Error) -> Self {
        IdmException::Storage(StorageException::Io(err))
    }
}

impl From<reqwest::Error> for IdmException {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            IdmException::Network(NetworkException::Timeout(0))
        } else if err.is_connect() {
            IdmException::Network(NetworkException::ConnectionFailed(err.to_string()))
        } else {
            IdmException::Network(NetworkException::ConnectionFailed(err.to_string()))
        }
    }
}

impl From<rusqlite::Error> for IdmException {
    fn from(err: rusqlite::Error) -> Self {
        IdmException::Database(DatabaseException::Sqlite(err.to_string()))
    }
}

pub type Result<T> = std::result::Result<T, IdmException>;
