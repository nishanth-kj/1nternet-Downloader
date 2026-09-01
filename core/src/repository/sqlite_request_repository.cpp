#include "repository/sqlite_request_repository.h"
#include <stdexcept>

namespace idr {
namespace repository {

SqliteRequestRepository::SqliteRequestRepository(const std::string& dbPath) {
    if (sqlite3_open(dbPath.c_str(), &m_db) != SQLITE_OK) {
        throw std::runtime_error("Failed to open database");
    }
    InitSchema();
}

SqliteRequestRepository::~SqliteRequestRepository() {
    if (m_db) sqlite3_close(m_db);
}

void SqliteRequestRepository::InitSchema() {
    const char* sql = R"(
        CREATE TABLE IF NOT EXISTS requests (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            url             TEXT    NOT NULL,
            method          TEXT    NOT NULL,
            user_agent      TEXT,
            proxy_url       TEXT,
            timeout_seconds INTEGER,
            max_retries     INTEGER,
            follow_redirects INTEGER,
            created_at      INTEGER NOT NULL
        );
    )";
    sqlite3_exec(m_db, sql, nullptr, nullptr, nullptr);
}

model::RequestModel SqliteRequestRepository::RowToItem(sqlite3_stmt* stmt) {
    return model::RequestModel(); // TODO: implement
}

int64_t SqliteRequestRepository::Add(const model::RequestModel& item) { return 0; /* TODO */ }
bool SqliteRequestRepository::Update(const model::RequestModel& item) { return false; /* TODO */ }
bool SqliteRequestRepository::Remove(int64_t id) { return false; /* TODO */ }
std::optional<model::RequestModel> SqliteRequestRepository::FindById(int64_t id) { return std::nullopt; /* TODO */ }
std::vector<model::RequestModel> SqliteRequestRepository::FindAll() { return {}; /* TODO */ }

} // namespace repository
} // namespace idr
