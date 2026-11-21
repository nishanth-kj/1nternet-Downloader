pub mod downloads;
pub mod segments;
pub mod torrents;

pub use downloads::{
    COL_CHECKSUM, COL_CREATED_AT, COL_DESTINATION, COL_DOWNLOADED_BYTES, COL_FILENAME, COL_ID,
    COL_MIME_TYPE, COL_STATUS, COL_TOTAL_BYTES, COL_UPDATED_AT, COL_URL,
    CREATE_TABLE_SQL as CREATE_DOWNLOADS_TABLE_SQL, DownloadId, DownloadItem, DownloadStatus,
    TABLE_NAME as TABLE_DOWNLOADS,
};
pub use segments::SegmentItem;
pub use torrents::{
    CREATE_TABLE_SQL as CREATE_TORRENTS_TABLE_SQL, TABLE_NAME as TABLE_TORRENTS, TorrentItem,
};
