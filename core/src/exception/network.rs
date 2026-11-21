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
}
