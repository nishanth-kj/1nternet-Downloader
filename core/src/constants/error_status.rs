use crate::define_code_value_enum;

define_code_value_enum! {
    /// Application Error Status code and string value pairs
    pub enum ErrorStatus(i32, &'static str) {
        Success(0, "SUCCESS"),
        NetworkError(1001, "NETWORK_ERROR"),
        Timeout(1002, "TIMEOUT"),
        RangeNotSupported(1003, "RANGE_NOT_SUPPORTED"),
        InvalidContentRange(1004, "INVALID_CONTENT_RANGE"),
        TlsError(1005, "TLS_ERROR"),
        RedirectError(1006, "REDIRECT_ERROR"),
        TransferInterrupted(1007, "TRANSFER_INTERRUPTED"),
        HttpStatus(1008, "HTTP_STATUS"),
        DiskFull(2001, "DISK_FULL"),
        IoError(2002, "IO_ERROR"),
        PreallocationFailed(2003, "PREALLOCATION_FAILED"),
        InvalidPath(2004, "INVALID_PATH"),
        DatabaseError(3001, "DATABASE_ERROR"),
        RecordNotFound(3002, "RECORD_NOT_FOUND"),
        DatabaseMigrationFailed(3003, "DATABASE_MIGRATION_FAILED"),
        ConstraintViolation(3004, "CONSTRAINT_VIOLATION"),
        DatabaseLockError(3005, "DATABASE_LOCK_ERROR"),
        ProtocolError(4001, "PROTOCOL_ERROR"),
        InvalidUrl(4002, "INVALID_URL"),
        InvalidScheme(4003, "INVALID_SCHEME"),
        IpcError(4004, "IPC_ERROR"),
        InstanceError(4005, "INSTANCE_ERROR"),
        TorrentError(4006, "TORRENT_ERROR"),
        VerificationFailed(5001, "VERIFICATION_FAILED"),
        Cancelled(6001, "CANCELLED"),
        Paused(6002, "PAUSED"),
        SegmentFailed(6003, "SEGMENT_FAILED"),
        SplitFailed(6004, "SPLIT_FAILED"),
        UnknownFileSize(6005, "UNKNOWN_FILE_SIZE"),
        WorkerStalled(6006, "WORKER_STALLED"),
        MaxRetriesExceeded(6007, "MAX_RETRIES_EXCEEDED"),
        ConfigError(7001, "CONFIG_ERROR"),
        ValidationError(7002, "VALIDATION_ERROR"),
        Unknown(9999, "UNKNOWN"),
    }
}

pub type ErrorStatusCode = ErrorStatus;
pub type ErrorCode = ErrorStatus;
