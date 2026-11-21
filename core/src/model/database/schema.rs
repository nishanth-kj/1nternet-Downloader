/// SQLite Table schema constants and DDL statements
pub const TABLE_DOWNLOADS: &str = "downloads";
pub const TABLE_TORRENTS: &str = "torrents";

pub const COL_ID: &str = "downloads_id";
pub const COL_URL: &str = "url";
pub const COL_DESTINATION: &str = "destination";
pub const COL_FILENAME: &str = "filename";
pub const COL_TOTAL_BYTES: &str = "total_bytes";
pub const COL_DOWNLOADED_BYTES: &str = "downloaded_bytes";
pub const COL_STATUS: &str = "status";
pub const COL_MIME_TYPE: &str = "mime_type";
pub const COL_CHECKSUM: &str = "checksum";
pub const COL_CREATED_AT: &str = "created_at";
pub const COL_UPDATED_AT: &str = "updated_at";

pub const CREATE_DOWNLOADS_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS downloads (
        downloads_id INTEGER PRIMARY KEY AUTOINCREMENT,
        url TEXT NOT NULL,
        destination TEXT NOT NULL,
        filename TEXT NOT NULL,
        total_bytes INTEGER DEFAULT 0,
        downloaded_bytes INTEGER DEFAULT 0,
        status INTEGER DEFAULT 0,
        mime_type TEXT,
        checksum TEXT,
        created_at INTEGER,
        updated_at INTEGER
    );
";

pub const CREATE_TORRENTS_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS torrents (
        torrent_id INTEGER PRIMARY KEY AUTOINCREMENT,
        info_hash TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        save_path TEXT NOT NULL,
        total_bytes INTEGER DEFAULT 0,
        downloaded_bytes INTEGER DEFAULT 0,
        status INTEGER DEFAULT 0,
        seed_ratio REAL DEFAULT 0.0,
        created_at INTEGER
    );
";
