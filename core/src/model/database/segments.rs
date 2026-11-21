use crate::exception::{DatabaseException, Result};
use crate::model::database::downloads::DownloadId;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// Diesel ORM Table Schema & Table Struct Definition
// -----------------------------------------------------------------------------

diesel::table! {
    segments (download_id, segment_id) {
        download_id -> BigInt,
        segment_id -> Integer,
        start_offset -> BigInt,
        end_offset -> BigInt,
        downloaded_bytes -> BigInt,
        is_completed -> Integer,
    }
}

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
// Native SQL DDL & Query Functions
// -----------------------------------------------------------------------------

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

/// Native query function to batch-save segment items in a transaction
pub fn save_segments(
    conn: &mut Connection,
    download_id: DownloadId,
    segments: &[SegmentItem],
) -> Result<()> {
    let tx = conn
        .transaction()
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to begin transaction: {}", e)))?;

    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT OR REPLACE INTO segments (
                    download_id, segment_id, start_offset, end_offset, downloaded_bytes, is_completed
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            )
            .map_err(|e| DatabaseException::Sqlite(format!("Prepare error: {}", e)))?;

        for seg in segments {
            stmt.execute(params![
                download_id,
                seg.segment_id,
                seg.start_offset as i64,
                seg.end_offset as i64,
                seg.downloaded_bytes as i64,
                if seg.is_completed { 1 } else { 0 },
            ])
            .map_err(|e| DatabaseException::Sqlite(format!("Failed to save segment: {}", e)))?;
        }
    }

    tx.commit()
        .map_err(|e| DatabaseException::Sqlite(format!("Failed to commit segments: {}", e)))?;

    Ok(())
}

/// Native query function to query all segments for a given download
pub fn select_segments_by_download_id(
    conn: &Connection,
    download_id: DownloadId,
) -> Result<Vec<SegmentItem>> {
    let mut stmt = conn
        .prepare(
            "SELECT download_id, segment_id, start_offset, end_offset, downloaded_bytes, is_completed
            FROM segments
            WHERE download_id = ?1
            ORDER BY segment_id ASC;",
        )
        .map_err(|e| DatabaseException::Sqlite(format!("Prepare error: {}", e)))?;

    let rows = stmt
        .query_map(params![download_id], |row| {
            Ok(SegmentItem {
                download_id: row.get(0)?,
                segment_id: row.get(1)?,
                start_offset: row.get::<_, i64>(2)?.max(0) as u64,
                end_offset: row.get::<_, i64>(3)?.max(0) as u64,
                downloaded_bytes: row.get::<_, i64>(4)?.max(0) as u64,
                is_completed: row.get::<_, i32>(5)? == 1,
            })
        })
        .map_err(|e| DatabaseException::Sqlite(format!("Query map error: {}", e)))?;

    let mut items = Vec::new();
    for r in rows {
        if let Ok(seg) = r {
            items.push(seg);
        }
    }
    Ok(items)
}

/// Native query function to delete all segments for a download
pub fn delete_segments_by_download_id(
    conn: &Connection,
    download_id: DownloadId,
) -> Result<()> {
    conn.execute(
        "DELETE FROM segments WHERE download_id = ?1;",
        params![download_id],
    )
    .map_err(|e| DatabaseException::Sqlite(format!("Failed to delete segments: {}", e)))?;
    Ok(())
}
