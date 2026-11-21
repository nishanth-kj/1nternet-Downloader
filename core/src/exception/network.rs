use crate::constants::ErrorCode;
use crate::exception::base::{AnyExceptionObject, BaseException};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NetworkException {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request timed out after {0} ms")]
    Timeout(u64),

    #[error("HTTP error status {code}: {message}")]
    HttpStatus { code: u16, message: String },

    #[error("Range requests not supported by server: {0}")]
    RangeNotSupported(String),

    #[error("Content-Range header mismatch: expected {expected}, received {received}")]
    InvalidContentRange { expected: String, received: String },

    #[error("SSL/TLS handshake failed: {0}")]
    TlsError(String),

    #[error("Redirect loop or invalid redirect: {0}")]
    RedirectError(String),

    #[error("Network transfer interrupted: {0}")]
    TransferInterrupted(String),

    #[error("Network exception [{code} - {value}]: {message}")]
    Custom {
        code: i32,
        value: &'static str,
        message: String,
        object: Option<AnyExceptionObject>,
    },
}

impl NetworkException {
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

impl BaseException for NetworkException {
    fn code(&self) -> i32 {
        match self {
            Self::ConnectionFailed(_) => ErrorCode::NetworkError.code(),
            Self::Timeout(_) => ErrorCode::Timeout.code(),
            Self::HttpStatus { code, .. } => *code as i32,
            Self::RangeNotSupported(_) => ErrorCode::RangeNotSupported.code(),
            Self::InvalidContentRange { .. } => ErrorCode::InvalidContentRange.code(),
            Self::TlsError(_) => ErrorCode::TlsError.code(),
            Self::RedirectError(_) => ErrorCode::RedirectError.code(),
            Self::TransferInterrupted(_) => ErrorCode::TransferInterrupted.code(),
            Self::Custom { code, .. } => *code,
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::ConnectionFailed(_) => ErrorCode::NetworkError.value(),
            Self::Timeout(_) => ErrorCode::Timeout.value(),
            Self::HttpStatus { .. } => ErrorCode::HttpStatus.value(),
            Self::RangeNotSupported(_) => ErrorCode::RangeNotSupported.value(),
            Self::InvalidContentRange { .. } => ErrorCode::InvalidContentRange.value(),
            Self::TlsError(_) => ErrorCode::TlsError.value(),
            Self::RedirectError(_) => ErrorCode::RedirectError.value(),
            Self::TransferInterrupted(_) => ErrorCode::TransferInterrupted.value(),
            Self::Custom { value, .. } => value,
        }
    }

    fn object(&self) -> Option<&(dyn std::any::Any + Send + Sync)> {
        match self {
            Self::ConnectionFailed(msg) => Some(msg),
            Self::Timeout(ms) => Some(ms),
            Self::HttpStatus { code, .. } => Some(code),
            Self::RangeNotSupported(msg) => Some(msg),
            Self::InvalidContentRange { expected, .. } => Some(expected),
            Self::TlsError(msg) => Some(msg),
            Self::RedirectError(msg) => Some(msg),
            Self::TransferInterrupted(msg) => Some(msg),
            Self::Custom { object, .. } => object.as_ref().map(|o| &*o.0 as &(dyn std::any::Any + Send + Sync)),
        }
    }
}
