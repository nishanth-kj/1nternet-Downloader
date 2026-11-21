pub use crate::exception::{
    DatabaseException, DownloadException, IdmException, NetworkException, ProtocolException,
    Result, StorageException,
};

/// Backward compatible alias for IdmException
pub type IdmError = IdmException;
