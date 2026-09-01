#pragma once

#include "model/response_model.h"
#include <vector>
#include <optional>
#include <cstdint>

namespace idr {
namespace repository {

class IResponseRepository {
public:
    virtual ~IResponseRepository() = default;

    virtual int64_t Add(const model::ResponseModel& item) = 0;
    virtual bool Update(const model::ResponseModel& item) = 0;
    virtual bool Remove(int64_t id) = 0;
    virtual std::optional<model::ResponseModel> FindById(int64_t id) = 0;
    virtual std::vector<model::ResponseModel> FindByRequestId(int64_t requestId) = 0;
    virtual std::vector<model::ResponseModel> FindAll() = 0;
};

} // namespace repository
} // namespace idr
