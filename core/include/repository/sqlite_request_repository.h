#pragma once

#include "repository/request_repository.h"
#include <sqlite3.h>
#include <memory>
#include <string>

namespace idr {
namespace repository {

class SqliteRequestRepository : public IRequestRepository {
public:
    explicit SqliteRequestRepository(const std::string& dbPath);
    ~SqliteRequestRepository() override;

    int64_t Add(const model::RequestModel& item) override;
    bool    Update(const model::RequestModel& item) override;
    bool    Remove(int64_t id) override;
    std::optional<model::RequestModel> FindById(int64_t id) override;
    std::vector<model::RequestModel>   FindAll() override;

private:
    sqlite3* m_db = nullptr;
    void InitSchema();
    model::RequestModel RowToItem(sqlite3_stmt* stmt);
};

} // namespace repository
} // namespace idr
