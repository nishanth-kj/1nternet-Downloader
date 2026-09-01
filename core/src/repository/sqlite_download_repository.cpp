#include "repository/sqlite_download_repository.h"
#include "model/download_item.h"
#include <stdexcept>
#include <chrono>

namespace idr {
namespace repository {

SqliteDownloadRepository::SqliteDownloadRepository(const std::string& dbPath) {
    if (sqlite3_open(dbPath.c_str(), &m_db) != SQLITE_OK) {
        throw std::runtime_error("Failed to open database: " + std::string(sqlite3_errmsg(m_db)));
    }
    InitSchema();
}

SqliteDownloadRepository::~SqliteDownloadRepository() {
    if (m_db) sqlite3_close(m_db);
}

void SqliteDownloadRepository::InitSchema() {
    const char* sql = R"(
        CREATE TABLE IF NOT EXISTS downloads (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            url             TEXT    NOT NULL,
            destination     TEXT    NOT NULL,
            filename        TEXT    NOT NULL,
            total_bytes     INTEGER DEFAULT 0,
            downloaded_bytes INTEGER DEFAULT 0,
            status          TEXT    NOT NULL DEFAULT 'Queued',
            mime_type       TEXT    DEFAULT '',
            checksum        TEXT    DEFAULT '',
            created_at      INTEGER NOT NULL,
            updated_at      INTEGER NOT NULL
        );
    )";
    char* errMsg = nullptr;
    if (sqlite3_exec(m_db, sql, nullptr, nullptr, &errMsg) != SQLITE_OK) {
        std::string err(errMsg);
        sqlite3_free(errMsg);
        throw std::runtime_error("Schema init failed: " + err);
    }
}

model::DownloadItem SqliteDownloadRepository::RowToItem(sqlite3_stmt* stmt) {
    model::DownloadItem item;
    item.id              = sqlite3_column_int64(stmt, 0);
    item.url             = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 1));
    item.destination     = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 2));
    item.filename        = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 3));
    item.totalBytes      = static_cast<uint64_t>(sqlite3_column_int64(stmt, 4));
    item.downloadedBytes = static_cast<uint64_t>(sqlite3_column_int64(stmt, 5));
    item.status          = model::StatusFromString(
        reinterpret_cast<const char*>(sqlite3_column_text(stmt, 6)));
    item.mimeType        = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 7));
    item.checksum        = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 8));
    auto created = std::chrono::seconds(sqlite3_column_int64(stmt, 9));
    auto updated = std::chrono::seconds(sqlite3_column_int64(stmt, 10));
    item.createdAt = std::chrono::system_clock::time_point(created);
    item.updatedAt = std::chrono::system_clock::time_point(updated);
    return item;
}

int64_t SqliteDownloadRepository::Add(const model::DownloadItem& item) {
    const char* sql = R"(
        INSERT INTO downloads (url, destination, filename, total_bytes, downloaded_bytes,
            status, mime_type, checksum, created_at, updated_at)
        VALUES (?,?,?,?,?,?,?,?,?,?);
    )";
    sqlite3_stmt* stmt = nullptr;
    sqlite3_prepare_v2(m_db, sql, -1, &stmt, nullptr);

    auto now = std::chrono::duration_cast<std::chrono::seconds>(
        std::chrono::system_clock::now().time_since_epoch()).count();

    sqlite3_bind_text(stmt,  1, item.url.c_str(),         -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt,  2, item.destination.c_str(), -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt,  3, item.filename.c_str(),    -1, SQLITE_TRANSIENT);
    sqlite3_bind_int64(stmt, 4, static_cast<int64_t>(item.totalBytes));
    sqlite3_bind_int64(stmt, 5, static_cast<int64_t>(item.downloadedBytes));
    sqlite3_bind_text(stmt,  6, model::StatusToString(item.status).c_str(), -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt,  7, item.mimeType.c_str(),    -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt,  8, item.checksum.c_str(),    -1, SQLITE_TRANSIENT);
    sqlite3_bind_int64(stmt, 9, now);
    sqlite3_bind_int64(stmt, 10, now);

    sqlite3_step(stmt);
    sqlite3_finalize(stmt);
    return sqlite3_last_insert_rowid(m_db);
}

bool SqliteDownloadRepository::Update(const model::DownloadItem& item) {
    const char* sql = R"(
        UPDATE downloads SET url=?, destination=?, filename=?, total_bytes=?,
            downloaded_bytes=?, status=?, mime_type=?, checksum=?, updated_at=?
        WHERE id=?;
    )";
    sqlite3_stmt* stmt = nullptr;
    sqlite3_prepare_v2(m_db, sql, -1, &stmt, nullptr);
    auto now = std::chrono::duration_cast<std::chrono::seconds>(
        std::chrono::system_clock::now().time_since_epoch()).count();

    sqlite3_bind_text(stmt,  1, item.url.c_str(),         -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt,  2, item.destination.c_str(), -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt,  3, item.filename.c_str(),    -1, SQLITE_TRANSIENT);
    sqlite3_bind_int64(stmt, 4, static_cast<int64_t>(item.totalBytes));
    sqlite3_bind_int64(stmt, 5, static_cast<int64_t>(item.downloadedBytes));
    sqlite3_bind_text(stmt,  6, model::StatusToString(item.status).c_str(), -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt,  7, item.mimeType.c_str(),    -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt,  8, item.checksum.c_str(),    -1, SQLITE_TRANSIENT);
    sqlite3_bind_int64(stmt, 9, now);
    sqlite3_bind_int64(stmt, 10, item.id);

    int rc = sqlite3_step(stmt);
    sqlite3_finalize(stmt);
    return rc == SQLITE_DONE;
}

bool SqliteDownloadRepository::Remove(int64_t id) {
    const char* sql = "DELETE FROM downloads WHERE id=?;";
    sqlite3_stmt* stmt = nullptr;
    sqlite3_prepare_v2(m_db, sql, -1, &stmt, nullptr);
    sqlite3_bind_int64(stmt, 1, id);
    int rc = sqlite3_step(stmt);
    sqlite3_finalize(stmt);
    return rc == SQLITE_DONE;
}

std::optional<model::DownloadItem> SqliteDownloadRepository::FindById(int64_t id) {
    const char* sql = "SELECT * FROM downloads WHERE id=?;";
    sqlite3_stmt* stmt = nullptr;
    sqlite3_prepare_v2(m_db, sql, -1, &stmt, nullptr);
    sqlite3_bind_int64(stmt, 1, id);
    std::optional<model::DownloadItem> result;
    if (sqlite3_step(stmt) == SQLITE_ROW) result = RowToItem(stmt);
    sqlite3_finalize(stmt);
    return result;
}

std::vector<model::DownloadItem> SqliteDownloadRepository::FindAll() {
    const char* sql = "SELECT * FROM downloads ORDER BY created_at DESC;";
    sqlite3_stmt* stmt = nullptr;
    sqlite3_prepare_v2(m_db, sql, -1, &stmt, nullptr);
    std::vector<model::DownloadItem> results;
    while (sqlite3_step(stmt) == SQLITE_ROW) results.push_back(RowToItem(stmt));
    sqlite3_finalize(stmt);
    return results;
}

std::vector<model::DownloadItem> SqliteDownloadRepository::FindByStatus(model::DownloadStatus status) {
    const char* sql = "SELECT * FROM downloads WHERE status=? ORDER BY created_at DESC;";
    sqlite3_stmt* stmt = nullptr;
    sqlite3_prepare_v2(m_db, sql, -1, &stmt, nullptr);
    auto s = model::StatusToString(status);
    sqlite3_bind_text(stmt, 1, s.c_str(), -1, SQLITE_TRANSIENT);
    std::vector<model::DownloadItem> results;
    while (sqlite3_step(stmt) == SQLITE_ROW) results.push_back(RowToItem(stmt));
    sqlite3_finalize(stmt);
    return results;
}

} // namespace repository
} // namespace idr
