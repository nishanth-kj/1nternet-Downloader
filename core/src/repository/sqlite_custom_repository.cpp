#include "repository/sqlite_custom_repository.h"
#include <stdexcept>

namespace idr {
namespace repository {

SqliteCustomRepository::SqliteCustomRepository(const std::string& dbPath) {
    if (sqlite3_open(dbPath.c_str(), &m_db) != SQLITE_OK) {
        throw std::runtime_error("Failed to open database");
    }
    InitSchema();
}

SqliteCustomRepository::~SqliteCustomRepository() {
    if (m_db) sqlite3_close(m_db);
}

void SqliteCustomRepository::InitSchema() {
    const char* sql = R"(
        CREATE TABLE IF NOT EXISTS custom_models (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            key         TEXT    NOT NULL UNIQUE,
            value       TEXT    NOT NULL,
            type        TEXT    NOT NULL,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL
        );
    )";
    sqlite3_exec(m_db, sql, nullptr, nullptr, nullptr);
}

model::CustomModel SqliteCustomRepository::RowToItem(sqlite3_stmt* stmt) {
    return model::CustomModel(); // TODO: implement
}

int64_t SqliteCustomRepository::Add(const model::CustomModel& item) { return 0; /* TODO */ }
bool SqliteCustomRepository::Update(const model::CustomModel& item) { return false; /* TODO */ }
bool SqliteCustomRepository::Remove(int64_t id) { return false; /* TODO */ }
std::optional<model::CustomModel> SqliteCustomRepository::FindById(int64_t id) { return std::nullopt; /* TODO */ }
std::optional<model::CustomModel> SqliteCustomRepository::FindByKey(const std::string& key) { return std::nullopt; /* TODO */ }
std::vector<model::CustomModel> SqliteCustomRepository::FindByType(const std::string& type) { return {}; /* TODO */ }
std::vector<model::CustomModel> SqliteCustomRepository::FindAll() { return {}; /* TODO */ }

} // namespace repository
} // namespace idr
