pub use crate::exception::{
    DatabaseException, DownloadException, IdmException, NetworkException, ProtocolException,
    Result, StorageException,
};

use serde::{Deserialize, Serialize};

/// Generic operational result wrapper for UI and RPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error_message: Option<String>,
}

impl<T> OperationResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error_message: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error_message: Some(message.into()),
        }
    }
}
