#pragma once

#include "repository/response_repository.h"
#include <sqlite3.h>
#include <memory>
#include <string>

namespace idr {
namespace repository {

class SqliteResponseRepository : public IResponseRepository {
public:
    explicit SqliteResponseRepository(const std::string& dbPath);
    ~SqliteResponseRepository() override;

    int64_t Add(const model::ResponseModel& item) override;
    bool    Update(const model::ResponseModel& item) override;
    bool    Remove(int64_t id) override;
    std::optional<model::ResponseModel> FindById(int64_t id) override;
    std::vector<model::ResponseModel> FindByRequestId(int64_t requestId) override;
    std::vector<model::ResponseModel> FindAll() override;

private:
    sqlite3* m_db = nullptr;
    void InitSchema();
    model::ResponseModel RowToItem(sqlite3_stmt* stmt);
};

} // namespace repository
} // namespace idr
