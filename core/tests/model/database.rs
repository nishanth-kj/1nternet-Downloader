use idm_lib::model::database::downloads::{
    delete_download_by_id, insert_download, select_all_downloads, select_download_by_id,
    update_download_progress, update_download_status,
};
use idm_lib::model::database::segments::{
    delete_segments_by_download_id, save_segments, select_segments_by_download_id,
};
use idm_lib::model::database::torrents::{
    delete_torrent_by_id, insert_torrent, select_all_torrents, select_torrent_by_id,
};
use idm_lib::model::database::{init_schema, SegmentItem, TorrentItem};
use idm_lib::model::{DownloadItem, DownloadStatus};
use idm_lib::utils::connect_sqlite;

#[test]
fn test_model_database_crud() {
    let temp_dir = std::env::temp_dir().join(format!(
        "idm_model_db_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("model_test.db");

    let mut conn = connect_sqlite(&db_path).expect("Database connection must open");
    init_schema(&conn).expect("Schema must initialize cleanly");

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

    // 1. Insert download
    let id = insert_download(&conn, &item).expect("Insert download must succeed");
    assert!(id > 0);

    // 2. Query download by ID
    let queried = select_download_by_id(&conn, id)
        .expect("Query must succeed")
        .expect("Item must exist");
    assert_eq!(queried.filename, "item.bin");
    assert_eq!(queried.status, DownloadStatus::Queued);

    // 3. Update progress
    update_download_progress(&conn, id, 2500, 5000, DownloadStatus::Downloading)
        .expect("Progress update must succeed");
    let updated = select_download_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(updated.downloaded_bytes, 2500);
    assert_eq!(updated.status, DownloadStatus::Downloading);

    // 4. Update status
    update_download_status(&conn, id, DownloadStatus::Paused).expect("Status update must succeed");
    let paused = select_download_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(paused.status, DownloadStatus::Paused);

    // 5. Query all downloads
    let all = select_all_downloads(&conn).expect("Select all must succeed");
    assert_eq!(all.len(), 1);

    // 6. Test segments persistence
    let segments = vec![
        SegmentItem {
            download_id: id,
            segment_id: 0,
            start_offset: 0,
            end_offset: 2499,
            downloaded_bytes: 2500,
            is_completed: true,
        },
        SegmentItem {
            download_id: id,
            segment_id: 1,
            start_offset: 2500,
            end_offset: 4999,
            downloaded_bytes: 0,
            is_completed: false,
        },
    ];
    save_segments(&mut conn, id, &segments).expect("Save segments must succeed");
    let loaded_segs = select_segments_by_download_id(&conn, id).expect("Get segments must succeed");
    assert_eq!(loaded_segs.len(), 2);
    assert_eq!(loaded_segs[0].downloaded_bytes, 2500);
    assert!(loaded_segs[0].is_completed);
    assert!(!loaded_segs[1].is_completed);

    delete_segments_by_download_id(&conn, id).expect("Delete segments must succeed");
    let empty_segs = select_segments_by_download_id(&conn, id).unwrap();
    assert!(empty_segs.is_empty());

    // 7. Test torrent model queries
    let torrent = TorrentItem::new(
        "abc123hash".to_string(),
        "test_torrent".to_string(),
        "C:\\Downloads\\Torrent".to_string(),
    );
    let t_id = insert_torrent(&conn, &torrent).expect("Insert torrent must succeed");
    assert!(t_id > 0);
    let queried_torrent = select_torrent_by_id(&conn, t_id).unwrap().unwrap();
    assert_eq!(queried_torrent.info_hash, "abc123hash");
    let all_torrents = select_all_torrents(&conn).unwrap();
    assert_eq!(all_torrents.len(), 1);
    delete_torrent_by_id(&conn, t_id).expect("Delete torrent must succeed");
    assert!(select_torrent_by_id(&conn, t_id).unwrap().is_none());

    // 8. Delete download
    delete_download_by_id(&conn, id).expect("Delete download must succeed");
    assert!(select_download_by_id(&conn, id).unwrap().is_none());

    let _ = std::fs::remove_dir_all(&temp_dir);
}
