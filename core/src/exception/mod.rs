pub mod database;
pub mod download;
pub mod network;
pub mod protocol;
pub mod storage;

pub use database::DatabaseException;
pub use download::DownloadException;
pub use network::NetworkException;
pub use protocol::ProtocolException;
pub use storage::StorageException;

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

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("General exception: {0}")]
    Other(String),
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
