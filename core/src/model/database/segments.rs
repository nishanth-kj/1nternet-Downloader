use crate::model::database::downloads::DownloadId;
use serde::{Deserialize, Serialize};

/// Segments table ORM struct representing an active or completed dynamic segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentItem {
    pub download_id: DownloadId,
    pub segment_id: u32,
    pub start_offset: u64,
    pub end_offset: u64,
    pub downloaded_bytes: u64,
    pub is_completed: bool,
}

// -----------------------------------------------------------------------------
// Native SQL Queries & Table Constants
// -----------------------------------------------------------------------------

pub const TABLE_NAME: &str = "segments";

pub const COL_DOWNLOAD_ID: &str = "download_id";
pub const COL_SEGMENT_ID: &str = "segment_id";
pub const COL_START_OFFSET: &str = "start_offset";
pub const COL_END_OFFSET: &str = "end_offset";
pub const COL_DOWNLOADED_BYTES: &str = "downloaded_bytes";
pub const COL_IS_COMPLETED: &str = "is_completed";

pub const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS segments (
        download_id INTEGER NOT NULL,
        segment_id INTEGER NOT NULL,
        start_offset INTEGER NOT NULL,
        end_offset INTEGER NOT NULL,
        downloaded_bytes INTEGER DEFAULT 0,
        is_completed INTEGER DEFAULT 0,
        PRIMARY KEY (download_id, segment_id),
        FOREIGN KEY (download_id) REFERENCES downloads(downloads_id) ON DELETE CASCADE
    );
";

pub const INSERT_OR_REPLACE_SQL: &str = "
    INSERT OR REPLACE INTO segments (
        download_id, segment_id, start_offset, end_offset, downloaded_bytes, is_completed
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6);
";

pub const SELECT_BY_DOWNLOAD_ID_SQL: &str = "
    SELECT download_id, segment_id, start_offset, end_offset, downloaded_bytes, is_completed
    FROM segments
    WHERE download_id = ?1
    ORDER BY segment_id ASC;
";

pub const DELETE_BY_DOWNLOAD_ID_SQL: &str = "
    DELETE FROM segments
    WHERE download_id = ?1;
";
