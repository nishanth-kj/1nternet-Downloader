#include "repository/sqlite_response_repository.h"
#include <stdexcept>

namespace idr {
namespace repository {

SqliteResponseRepository::SqliteResponseRepository(const std::string& dbPath) {
    if (sqlite3_open(dbPath.c_str(), &m_db) != SQLITE_OK) {
        throw std::runtime_error("Failed to open database");
    }
    InitSchema();
}

SqliteResponseRepository::~SqliteResponseRepository() {
    if (m_db) sqlite3_close(m_db);
}

void SqliteResponseRepository::InitSchema() {
    const char* sql = R"(
        CREATE TABLE IF NOT EXISTS responses (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            request_id      INTEGER NOT NULL,
            status_code     INTEGER,
            status_message  TEXT,
            content_type    TEXT,
            content_length  INTEGER,
            is_partial      INTEGER,
            redirect_url    TEXT,
            error_message   TEXT,
            success         INTEGER,
            received_at     INTEGER NOT NULL
        );
    )";
    sqlite3_exec(m_db, sql, nullptr, nullptr, nullptr);
}

model::ResponseModel SqliteResponseRepository::RowToItem(sqlite3_stmt* stmt) {
    return model::ResponseModel(); // TODO: implement
}

int64_t SqliteResponseRepository::Add(const model::ResponseModel& item) { return 0; /* TODO */ }
bool SqliteResponseRepository::Update(const model::ResponseModel& item) { return false; /* TODO */ }
bool SqliteResponseRepository::Remove(int64_t id) { return false; /* TODO */ }
std::optional<model::ResponseModel> SqliteResponseRepository::FindById(int64_t id) { return std::nullopt; /* TODO */ }
std::vector<model::ResponseModel> SqliteResponseRepository::FindByRequestId(int64_t requestId) { return {}; /* TODO */ }
std::vector<model::ResponseModel> SqliteResponseRepository::FindAll() { return {}; /* TODO */ }

} // namespace repository
} // namespace idr
