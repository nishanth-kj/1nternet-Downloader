#pragma once

#include "model/request_model.h"
#include <vector>
#include <optional>
#include <cstdint>

namespace idr {
namespace repository {

class IRequestRepository {
public:
    virtual ~IRequestRepository() = default;

    virtual int64_t Add(const model::RequestModel& item) = 0;
    virtual bool Update(const model::RequestModel& item) = 0;
    virtual bool Remove(int64_t id) = 0;
    virtual std::optional<model::RequestModel> FindById(int64_t id) = 0;
    virtual std::vector<model::RequestModel> FindAll() = 0;
};

} // namespace repository
} // namespace idr
