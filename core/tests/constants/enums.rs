use idm_lib::constants::{
    CategoryCode, DownloadStatusCode, ErrorStatus, HttpStatusCode, Status, TorrentStatusCode,
};

#[test]
fn test_generic_status_code_value() {
    let idle = Status::Idle;
    assert_eq!(idle.code(), 0);
    assert_eq!(idle.value(), "Idle");

    let running = Status::Running;
    assert_eq!(running.code(), 2);
    assert_eq!(running.value(), "Running");

    let from_c = Status::from_code(4);
    assert_eq!(from_c, Some(Status::Completed));

    let from_v = Status::from_value("failed");
    assert_eq!(from_v, Some(Status::Failed));
}

#[test]
fn test_error_status_code_value() {
    let net = ErrorStatus::NetworkError;
    assert_eq!(net.code(), 1001);
    assert_eq!(net.value(), "NETWORK_ERROR");

    let from_code = ErrorStatus::from_code(3001);
    assert_eq!(from_code, Some(ErrorStatus::DatabaseError));

    let from_val = ErrorStatus::from_value("DISK_FULL");
    assert_eq!(from_val, Some(ErrorStatus::DiskFull));
}

#[test]
fn test_category_code_value() {
    let cat = CategoryCode::Downloading;
    assert_eq!(cat.code(), 1);
    assert_eq!(cat.value(), "Downloading");

    let from_c = CategoryCode::from_code(1);
    assert_eq!(from_c, Some(CategoryCode::Downloading));

    let from_v = CategoryCode::from_value("downloading");
    assert_eq!(from_v, Some(CategoryCode::Downloading));
}

#[test]
fn test_download_and_torrent_status_codes() {
    let dl_status = DownloadStatusCode::Downloading;
    assert_eq!(dl_status.code(), 1);
    assert_eq!(dl_status.value(), "Downloading");

    let torrent_status = TorrentStatusCode::Seeding;
    assert_eq!(torrent_status.code(), 4);
    assert_eq!(torrent_status.value(), "Seeding");

    let http_ok = HttpStatusCode::Ok;
    assert_eq!(http_ok.code(), 200);
    assert_eq!(http_ok.value(), "OK");
}
