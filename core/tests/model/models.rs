use idm_lib::model::{
    AddDownloadRequest, DownloadAction, DownloadActionRequest, DownloadSnapshot, DownloadStatus,
};

#[test]
fn test_download_snapshot_calculations() {
    let snap = DownloadSnapshot {
        id: 1,
        url: "https://example.com/large.iso".to_string(),
        destination: "C:\\Downloads\\large.iso".to_string(),
        filename: "large.iso".to_string(),
        total_bytes: 1000,
        downloaded_bytes: 500,
        speed_bytes_per_sec: 100.0,
        status: DownloadStatus::Downloading,
        segment_count: 2,
        segments: Vec::new(),
        is_torrent: false,
        peers: 0,
        seeds: 0,
        error_message: None,
        progress_percent: 0.0,
        eta_seconds: None,
    };

    assert_eq!(snap.calculate_progress(), 50.0);
    assert_eq!(snap.calculate_eta(), Some(5)); // (1000-500)/100 = 5 seconds
}

#[test]
fn test_request_models() {
    let req = AddDownloadRequest::new("https://example.com/test.iso");
    assert_eq!(req.url, "https://example.com/test.iso");
    assert!(req.auto_start);

    let action = DownloadActionRequest {
        id: 42,
        action: DownloadAction::Pause,
    };
    assert_eq!(action.id, 42);
    assert_eq!(action.action, DownloadAction::Pause);
}
