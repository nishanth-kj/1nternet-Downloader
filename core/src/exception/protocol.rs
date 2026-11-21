use crate::constants::ErrorCode;
use crate::exception::base::{AnyExceptionObject, BaseException};
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

    #[error("Protocol exception [{code} - {value}]: {message}")]
    Custom {
        code: i32,
        value: &'static str,
        message: String,
        object: Option<AnyExceptionObject>,
    },
}

impl ProtocolException {
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

impl BaseException for ProtocolException {
    fn code(&self) -> i32 {
        match self {
            Self::InvalidUrl(_) => ErrorCode::InvalidUrl.code(),
            Self::InvalidScheme(_) => ErrorCode::InvalidScheme.code(),
            Self::IpcError(_) => ErrorCode::IpcError.code(),
            Self::InstanceError(_) => ErrorCode::InstanceError.code(),
            Self::TorrentError(_) => ErrorCode::TorrentError.code(),
            Self::Custom { code, .. } => *code,
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::InvalidUrl(_) => ErrorCode::InvalidUrl.value(),
            Self::InvalidScheme(_) => ErrorCode::InvalidScheme.value(),
            Self::IpcError(_) => ErrorCode::IpcError.value(),
            Self::InstanceError(_) => ErrorCode::InstanceError.value(),
            Self::TorrentError(_) => ErrorCode::TorrentError.value(),
            Self::Custom { value, .. } => value,
        }
    }

    fn object(&self) -> Option<&(dyn std::any::Any + Send + Sync)> {
        match self {
            Self::InvalidUrl(url) => Some(url),
            Self::InvalidScheme(scheme) => Some(scheme),
            Self::IpcError(msg) => Some(msg),
            Self::InstanceError(msg) => Some(msg),
            Self::TorrentError(msg) => Some(msg),
            Self::Custom { object, .. } => object.as_ref().map(|o| &*o.0 as &(dyn std::any::Any + Send + Sync)),
        }
    }
}
