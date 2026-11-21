use idm_lib::constants::ErrorStatus;
use idm_lib::exception::{
    BaseError, BaseException, DatabaseException, DownloadException, IdmException,
    NetworkException, ProtocolException, StorageException,
};

#[derive(Debug, PartialEq, Eq)]
struct SituationalContext {
    user_id: u64,
    retry_attempt: u32,
    endpoint: String,
}

#[test]
fn test_base_error_with_custom_object() {
    let ctx = SituationalContext {
        user_id: 999,
        retry_attempt: 2,
        endpoint: "https://example.com/cdn".into(),
    };

    let base_err = BaseError::new(
        ErrorStatus::NetworkError.code(),
        ErrorStatus::NetworkError.value(),
        "Custom network connection error occurred",
    )
    .with_object(ctx);

    // Verify code and value
    assert_eq!(base_err.code(), 1001);
    assert_eq!(base_err.value(), "NETWORK_ERROR");

    // Verify downcast_object returns exact type
    let downcasted = base_err.downcast_object::<SituationalContext>();
    assert!(downcasted.is_some());
    assert_eq!(downcasted.unwrap().user_id, 999);
    assert_eq!(downcasted.unwrap().retry_attempt, 2);
    assert_eq!(downcasted.unwrap().endpoint, "https://example.com/cdn");

    // Verify mismatched type returns None safely
    assert!(base_err.downcast_object::<String>().is_none());
    assert!(base_err.downcast_object::<u32>().is_none());
}

#[test]
fn test_database_exception_code_value_object() {
    let not_found = DatabaseException::NotFound(42);
    assert_eq!(not_found.code(), ErrorStatus::RecordNotFound.code());
    assert_eq!(not_found.value(), ErrorStatus::RecordNotFound.value());
    assert_eq!(not_found.downcast_object::<i64>(), Some(&42));

    let custom_db = DatabaseException::custom(
        3999,
        "CUSTOM_DB_ERR",
        "Corrupted transaction block",
        Some("trx_001".to_string()),
    );
    assert_eq!(custom_db.code(), 3999);
    assert_eq!(custom_db.value(), "CUSTOM_DB_ERR");
    assert_eq!(
        custom_db.downcast_object::<String>(),
        Some(&"trx_001".to_string())
    );
}

#[test]
fn test_download_exception_code_value_object() {
    let seg_err = DownloadException::SegmentFailed {
        segment_id: 7,
        reason: "Connection reset".into(),
    };
    assert_eq!(seg_err.code(), ErrorStatus::SegmentFailed.code());
    assert_eq!(seg_err.value(), ErrorStatus::SegmentFailed.value());
    assert_eq!(seg_err.downcast_object::<u32>(), Some(&7));

    let checksum_err = DownloadException::ChecksumMismatch {
        expected: "aabbcc".into(),
        actual: "ddeeff".into(),
    };
    assert_eq!(checksum_err.code(), ErrorStatus::VerificationFailed.code());
    assert_eq!(checksum_err.value(), ErrorStatus::VerificationFailed.value());
    assert_eq!(
        checksum_err.downcast_object::<String>(),
        Some(&"aabbcc".to_string())
    );
}

#[test]
fn test_network_exception_code_value_object() {
    let timeout = NetworkException::Timeout(5000);
    assert_eq!(timeout.code(), ErrorStatus::Timeout.code());
    assert_eq!(timeout.value(), ErrorStatus::Timeout.value());
    assert_eq!(timeout.downcast_object::<u64>(), Some(&5000));

    let http_err = NetworkException::HttpStatus {
        code: 404,
        message: "Not Found".into(),
    };
    assert_eq!(http_err.code(), 404);
    assert_eq!(http_err.value(), ErrorStatus::HttpStatus.value());
    assert_eq!(http_err.downcast_object::<u16>(), Some(&404));
}

#[test]
fn test_storage_and_protocol_exceptions() {
    let storage_space = StorageException::InsufficientSpace {
        path: "D:\\Downloads".into(),
        required_bytes: 10_000,
        available_bytes: 2_000,
    };
    assert_eq!(storage_space.code(), ErrorStatus::DiskFull.code());
    assert_eq!(storage_space.value(), ErrorStatus::DiskFull.value());
    assert_eq!(
        storage_space.downcast_object::<String>(),
        Some(&"D:\\Downloads".to_string())
    );

    let proto_url = ProtocolException::InvalidUrl("ftp://wrong-scheme".into());
    assert_eq!(proto_url.code(), ErrorStatus::InvalidUrl.code());
    assert_eq!(proto_url.value(), ErrorStatus::InvalidUrl.value());
    assert_eq!(
        proto_url.downcast_object::<String>(),
        Some(&"ftp://wrong-scheme".to_string())
    );
}

#[test]
fn test_idm_exception_hierarchy_polymorphism() {
    let net_err = NetworkException::ConnectionFailed("Reset by peer".into());
    let idm_err: IdmException = net_err.into();

    assert_eq!(idm_err.code(), ErrorStatus::NetworkError.code());
    assert_eq!(idm_err.value(), ErrorStatus::NetworkError.value());
    assert_eq!(idm_err.to_string(), "Connection failed: Reset by peer");
    assert_eq!(
        idm_err.downcast_object::<String>(),
        Some(&"Reset by peer".to_string())
    );
}
