use idm_lib::config::Settings;
use idm_lib::model::database::init_schema;
use idm_lib::model::DownloadStatus;
use idm_lib::service::QueueManager;
use idm_lib::utils::connect_sqlite;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test_queue_manager_with_native_database_queries() {
    let temp_dir = std::env::temp_dir().join(format!(
        "idm_queue_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("queue_test.db");

    let conn = connect_sqlite(&db_path).expect("Database must open");
    init_schema(&conn).expect("Schema must initialize cleanly");
    let conn = Arc::new(Mutex::new(conn));

    let mut settings = Settings::default();
    settings.download_dir = temp_dir.to_string_lossy().to_string();

    let qm = QueueManager::new(Arc::clone(&conn), settings.clone());

    // 1. Add download into queue (auto_start = false for predictable state)
    let download_id = qm
        .add_download("https://example.com/test_file.zip", None, false)
        .await
        .expect("Add download must succeed");

    assert!(download_id > 0);

    // 2. Verify download is persisted via model query
    {
        let conn_guard = conn.lock().unwrap();
        let item = idm_lib::model::database::downloads::select_download_by_id(&conn_guard, download_id)
            .expect("Query download must succeed")
            .expect("Item must exist in database");
        assert_eq!(item.id, download_id);
        assert_eq!(item.status, DownloadStatus::Queued);
        assert_eq!(item.filename, "test_file.zip");
    }

    // 3. Test pause download
    qm.pause_download(download_id);
    {
        let conn_guard = conn.lock().unwrap();
        let item = idm_lib::model::database::downloads::select_download_by_id(&conn_guard, download_id)
            .unwrap()
            .unwrap();
        assert_eq!(item.status, DownloadStatus::Paused);
    }

    // 4. Test delete download
    qm.delete_download(download_id, false);
    {
        let conn_guard = conn.lock().unwrap();
        let item = idm_lib::model::database::downloads::select_download_by_id(&conn_guard, download_id).unwrap();
        assert!(item.is_none());
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}
