#pragma once

#include "repository/download_repository.h"
#include <sqlite3.h>
#include <memory>

namespace idr {
namespace repository {

/**
 * SqliteDownloadRepository - SQLite-backed implementation of IDownloadRepository.
 * Depends on: model::DownloadItem (for data), utils::database (for raw DB access).
 */
class SqliteDownloadRepository : public IDownloadRepository {
public:
    explicit SqliteDownloadRepository(const std::string& dbPath);
    ~SqliteDownloadRepository() override;

    int64_t Add(const model::DownloadItem& item) override;
    bool    Update(const model::DownloadItem& item) override;
    bool    Remove(int64_t id) override;
    std::optional<model::DownloadItem> FindById(int64_t id) override;
    std::vector<model::DownloadItem>   FindAll() override;
    std::vector<model::DownloadItem>   FindByStatus(model::DownloadStatus status) override;

private:
    sqlite3* m_db = nullptr;
    void InitSchema();
    model::DownloadItem RowToItem(sqlite3_stmt* stmt);
};

} // namespace repository
} // namespace idr
