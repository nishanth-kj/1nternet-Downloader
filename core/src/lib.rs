pub mod config;
pub mod exception;
pub mod model;
pub mod service;
pub mod ui;
pub mod utils;

pub use config::{Config, Settings};
pub use exception::{
    DatabaseException, DownloadException, IdmException, NetworkException, ProtocolException,
    Result, StorageException,
};
pub use model::{
    AddDownloadRequest, AddTorrentRequest, DownloadAction, DownloadActionRequest,
    DownloadId, DownloadItem, DownloadSegment, DownloadSnapshot, DownloadStatus, IdmError,
    TorrentItem, UpdateSettingsRequest,
};
pub use service::{
    decode_idr_url, fast_crc32, fast_is_all_zero, get_runtime, spawn, windows, Database,
    DatabaseService, DownloadService, DownloadTask, DynamicScheduler, NetworkClient,
    NetworkService, QueueCommand, QueueManager, QueueService, RemoteCapabilities, SpeedMeter,
    SqliteDatabaseService, TorrentManager, TorrentService, TorrentSnapshot, TorrentStatus,
    TorrentTask, WorkerAssignment,
};
pub use ui::IcedApp;
pub use utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_fast_c_primitives() {
        let data = b"123456789";
        let crc = fast_crc32(data);
        assert_eq!(crc, 0xCBF43926);

        let zeroes = vec![0u8; 512];
        assert!(fast_is_all_zero(&zeroes));

        let mut non_zeroes = vec![0u8; 512];
        non_zeroes[256] = 0xAA;
        assert!(!fast_is_all_zero(&non_zeroes));
    }

    #[test]
    fn test_scheduler_no_gaps_no_overlaps() {
        let total_bytes: u64 = 100 * 1024 * 1024; // 100 MB
        let initial_workers = 4;
        let sched = DynamicScheduler::new(total_bytes, initial_workers);

        assert_eq!(sched.segments.len(), 4);
        assert_eq!(sched.segments[0].start_offset, 0);

        let mut previous_end = 0;
        for (i, seg) in sched.segments.iter().enumerate() {
            if i > 0 {
                assert_eq!(seg.start_offset, previous_end + 1, "Gap or overlap detected between segments");
            }
            assert!(seg.end_offset >= seg.start_offset, "Segment end must be >= start");
            previous_end = seg.end_offset;
        }

        assert_eq!(previous_end, total_bytes - 1, "Last segment must reach total_bytes - 1");
    }

    #[test]
    fn test_dynamic_work_stealing_split() {
        let total_bytes: u64 = 20 * 1024 * 1024; // 20 MB
        let mut sched = DynamicScheduler::new(total_bytes, 1);

        let work1 = sched.request_work().expect("Worker 0 should get initial segment");
        assert_eq!(work1.segment_id, 0);
        assert_eq!(work1.resume_offset, 0);
        assert_eq!(work1.end_offset, total_bytes - 1);

        let work2 = sched.request_work().expect("Worker 1 should steal second half");
        assert_eq!(work2.segment_id, 1);
        assert!(work2.resume_offset > 0);
        assert_eq!(work2.end_offset, total_bytes - 1);

        assert_eq!(sched.segments[0].end_offset, work2.resume_offset - 1);
        assert_eq!(sched.segments[0].end_offset + 1, sched.segments[1].start_offset);
    }

    #[test]
    fn test_split_threshold_enforcement() {
        let total_bytes: u64 = 1024 * 1024; // 1 MB (< 2MB threshold)
        let mut sched = DynamicScheduler::new(total_bytes, 1);

        let _work1 = sched.request_work().expect("Worker 0 claims work");
        let work2 = sched.request_work();
        assert!(work2.is_none(), "Should not split segment smaller than minimum threshold");
    }

    #[test]
    fn test_speed_meter() {
        let mut meter = SpeedMeter::new(0);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let speed = meter.update(1024 * 512); // 512 KB
        assert!(speed > 0.0, "Speed must be calculated as > 0 bytes/sec");
    }

    #[test]
    fn test_idr_url_decoding() {
        let idr = "idr://https%3A%2F%2Fexample.com%2Finstaller.exe";
        let decoded = decode_idr_url(idr);
        assert_eq!(decoded, Some("https://example.com/installer.exe".to_string()));

        let raw = "https://example.com/file.zip";
        let unchanged = decode_idr_url(raw);
        assert_eq!(unchanged, Some("https://example.com/file.zip".to_string()));
    }

    #[test]
    fn test_models_calculations() {
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
    fn test_database_crud() {
        let temp_dir = std::env::temp_dir().join(format!("idm_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("test.db");

        let db = SqliteDatabaseService::open(&db_path).expect("Database must open");
        let item = DownloadItem {
            id: 0,
            url: "https://example.com/item.bin".to_string(),
            destination: "C:\\Downloads\\item.bin".to_string(),
            filename: "item.bin".to_string(),
            total_bytes: 5000,
            downloaded_bytes: 0,
            status: DownloadStatus::Queued,
            mime_type: Some("application/octet-stream".to_string()),
            checksum: None,
            created_at: 1000,
            updated_at: 1000,
        };

        let id = db.insert_download(&item).expect("Insert must succeed");
        assert!(id > 0);

        let queried = db.get_download(id).expect("Query must succeed").expect("Item must exist");
        assert_eq!(queried.filename, "item.bin");
        assert_eq!(queried.status, DownloadStatus::Queued);

        db.update_progress(id, 2500, 5000, DownloadStatus::Downloading).expect("Progress update must succeed");
        let updated = db.get_download(id).unwrap().unwrap();
        assert_eq!(updated.downloaded_bytes, 2500);
        assert_eq!(updated.status, DownloadStatus::Downloading);

        db.delete_download(id).expect("Delete must succeed");
        assert!(db.get_download(id).unwrap().is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_utils_formatting() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.00 GB");
        assert_eq!(format_speed(1024.0 * 1024.0 * 5.0), "5.00 MB/s");
        assert_eq!(format_eta(65), "01:05");
        assert_eq!(format_eta(3665), "1h 01m");
        assert_eq!(format_percent(50, 100), "50.0%");
    }

    #[test]
    fn test_exception_hierarchy() {
        let net_err = NetworkException::ConnectionFailed("Reset by peer".into());
        let idm_err: IdmException = net_err.into();
        assert!(matches!(idm_err, IdmException::Network(_)));
        assert_eq!(idm_err.to_string(), "Connection failed: Reset by peer");
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

    #[test]
    fn test_traits_polymorphism() {
        let temp_dir = std::env::temp_dir().join(format!("idm_trait_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("test_trait.db");

        let db = Arc::new(SqliteDatabaseService::open(&db_path).expect("Database must open"));
        let db_trait: Arc<dyn DatabaseService> = db;

        let item = DownloadItem::new(
            "https://example.com/poly.bin".to_string(),
            "C:\\Downloads\\poly.bin".to_string(),
            "poly.bin".to_string(),
        );

        let id = db_trait.insert_download(&item).expect("Insert via trait must succeed");
        assert!(id > 0);

        let loaded = db_trait.get_download(id).unwrap().unwrap();
        assert_eq!(loaded.filename, "poly.bin");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
