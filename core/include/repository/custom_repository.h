#pragma once

#include "model/custom_model.h"
#include <vector>
#include <optional>
#include <cstdint>

namespace idr {
namespace repository {

class ICustomRepository {
public:
    virtual ~ICustomRepository() = default;

    virtual int64_t Add(const model::CustomModel& item) = 0;
    virtual bool Update(const model::CustomModel& item) = 0;
    virtual bool Remove(int64_t id) = 0;
    virtual std::optional<model::CustomModel> FindById(int64_t id) = 0;
    virtual std::optional<model::CustomModel> FindByKey(const std::string& key) = 0;
    virtual std::vector<model::CustomModel> FindByType(const std::string& type) = 0;
    virtual std::vector<model::CustomModel> FindAll() = 0;
};

} // namespace repository
} // namespace idr
