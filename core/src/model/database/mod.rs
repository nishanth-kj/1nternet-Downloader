pub mod downloads;
pub mod segments;
pub mod torrents;

use crate::exception::{DatabaseException, Result};
use rusqlite::Connection;

/// Initialize database schema across all model domains
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute(downloads::CREATE_TABLE_SQL, [])
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to create downloads table: {}", e)))?;

    conn.execute(torrents::CREATE_TABLE_SQL, [])
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to create torrents table: {}", e)))?;

    conn.execute(segments::CREATE_TABLE_SQL, [])
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to create segments table: {}", e)))?;

    Ok(())
}

pub use downloads::{
    delete_download_by_id, insert_download, select_all_downloads, select_download_by_id,
    update_download_progress, update_download_status,
    CREATE_TABLE_SQL as CREATE_DOWNLOADS_TABLE_SQL, DownloadId, DownloadItem, DownloadStatus,
};
pub use segments::{
    delete_segments_by_download_id, save_segments, select_segments_by_download_id,
    CREATE_TABLE_SQL as CREATE_SEGMENTS_TABLE_SQL, SegmentItem,
};
pub use torrents::{
    delete_torrent_by_id, insert_torrent, select_all_torrents, select_torrent_by_id,
    update_torrent_progress, CREATE_TABLE_SQL as CREATE_TORRENTS_TABLE_SQL, TorrentItem,
};
