#pragma once

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

namespace idr {
namespace model {

struct DownloadModel {
    int downloads_id = 0;
    std::string url;
    std::string destination;
    std::string filename;
    int64_t total_bytes = 0;
    int64_t downloaded_bytes = 0;
    int status = 0;
    std::string mime_type;
    std::string checksum;
    int64_t created_at = 0;
    int64_t updated_at = 0;
};

// sqlite_orm stays in the .cpp (pimpl). Including this header must stay cheap
// so GUI translation units never parse the ORM.
class DownloadDB {
public:
    explicit DownloadDB(const std::string& dbPath);
    ~DownloadDB();

    DownloadDB(const DownloadDB&) = delete;
    DownloadDB& operator=(const DownloadDB&) = delete;

    int Add(DownloadModel& item);
    void Update(const DownloadModel& item);
    std::unique_ptr<DownloadModel> Get(int id);
    void Delete(int id);
    std::vector<DownloadModel> GetAll();

private:
    struct Impl;
    std::unique_ptr<Impl> m_impl;
};

} // namespace model
} // namespace idr
