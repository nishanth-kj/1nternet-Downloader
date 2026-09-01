#pragma once

#include "repository/custom_repository.h"
#include <sqlite3.h>
#include <memory>
#include <string>

namespace idr {
namespace repository {

class SqliteCustomRepository : public ICustomRepository {
public:
    explicit SqliteCustomRepository(const std::string& dbPath);
    ~SqliteCustomRepository() override;

    int64_t Add(const model::CustomModel& item) override;
    bool    Update(const model::CustomModel& item) override;
    bool    Remove(int64_t id) override;
    std::optional<model::CustomModel> FindById(int64_t id) override;
    std::optional<model::CustomModel> FindByKey(const std::string& key) override;
    std::vector<model::CustomModel> FindByType(const std::string& type) override;
    std::vector<model::CustomModel> FindAll() override;

private:
    sqlite3* m_db = nullptr;
    void InitSchema();
    model::CustomModel RowToItem(sqlite3_stmt* stmt);
};

} // namespace repository
} // namespace idr
