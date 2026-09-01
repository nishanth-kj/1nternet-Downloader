#pragma once

#include "model/download_item.h"
#include <vector>
#include <optional>
#include <cstdint>

namespace idr {
namespace repository {

/**
 * IDownloadRepository - interface for download persistence.
 * The repository owns lifetime of DownloadItem records in storage.
 * The service layer talks ONLY through this interface.
 */
class IDownloadRepository {
public:
    virtual ~IDownloadRepository() = default;

    virtual int64_t Add(const model::DownloadItem& item) = 0;
    virtual bool Update(const model::DownloadItem& item) = 0;
    virtual bool Remove(int64_t id) = 0;
    virtual std::optional<model::DownloadItem> FindById(int64_t id) = 0;
    virtual std::vector<model::DownloadItem> FindAll() = 0;
    virtual std::vector<model::DownloadItem> FindByStatus(model::DownloadStatus status) = 0;
};

} // namespace repository
} // namespace idr
