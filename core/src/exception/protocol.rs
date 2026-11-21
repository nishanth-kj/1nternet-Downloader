use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ProtocolException {
    #[error("Invalid or malformed URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid custom protocol scheme (expected idr://): {0}")]
    InvalidScheme(String),

    #[error("IPC named pipe error: {0}")]
    IpcError(String),

    #[error("Single-instance mutex collision or acquisition error: {0}")]
    InstanceError(String),

    #[error("BitTorrent protocol error: {0}")]
    TorrentError(String),
}
