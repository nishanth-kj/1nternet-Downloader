use crate::exception::{DatabaseException, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// SQLite database connection helper utility for managing connections and schema pragmas
pub struct DbConnection;

impl DbConnection {
    /// Opens an optimized SQLite connection with WAL mode and foreign key constraints enabled
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Connection> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(path_ref).map_err(|e| {
            DatabaseException::Sqlite(format!("Failed to open SQLite database at {:?}: {}", path_ref, e))
        })?;

        // Enable Write-Ahead Logging (WAL) for high concurrency
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;
             PRAGMA busy_timeout = 5000;"
        )
        .map_err(|e| {
            DatabaseException::Sqlite(format!("Failed to configure SQLite connection pragmas: {}", e))
        })?;

        Ok(conn)
    }

    /// Resolves the canonical database path in application data directory
    pub fn default_path() -> PathBuf {
        crate::config::Config::database_path()
    }

    /// Opens the default persistent SQLite connection
    pub fn open_default() -> Result<Connection> {
        Self::open(Self::default_path())
    }
}

/// Convenience standalone function to connect to SQLite
pub fn connect_sqlite<P: AsRef<Path>>(path: P) -> Result<Connection> {
    DbConnection::open(path)
}

/// Convenience standalone function to connect to default database
pub fn connect_default_sqlite() -> Result<Connection> {
    DbConnection::open_default()
}
