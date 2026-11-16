#include "model/sqlite_download_model.h"

#include <sqlite_orm/sqlite_orm.h>

namespace idr {
namespace model {

namespace {

auto InitDownloadStorage(const std::string& dbPath) {
    using namespace sqlite_orm;
    return make_storage(dbPath,
                        make_table("downloads",
                                   make_column("downloads_id", &DownloadModel::downloads_id, primary_key().autoincrement()),
                                   make_column("url", &DownloadModel::url),
                                   make_column("destination", &DownloadModel::destination),
                                   make_column("filename", &DownloadModel::filename),
                                   make_column("total_bytes", &DownloadModel::total_bytes),
                                   make_column("downloaded_bytes", &DownloadModel::downloaded_bytes),
                                   make_column("status", &DownloadModel::status),
                                   make_column("mime_type", &DownloadModel::mime_type),
                                   make_column("checksum", &DownloadModel::checksum),
                                   make_column("created_at", &DownloadModel::created_at),
                                   make_column("updated_at", &DownloadModel::updated_at)));
}

} // namespace

struct DownloadDB::Impl {
    using StorageType = decltype(InitDownloadStorage(""));
    StorageType storage;

    explicit Impl(const std::string& dbPath)
        : storage(InitDownloadStorage(dbPath))
    {
        storage.sync_schema();
    }
};

DownloadDB::DownloadDB(const std::string& dbPath)
    : m_impl(std::make_unique<Impl>(dbPath))
{
}

DownloadDB::~DownloadDB() = default;

int DownloadDB::Add(DownloadModel& item) {
    return m_impl->storage.insert(item);
}

void DownloadDB::Update(const DownloadModel& item) {
    m_impl->storage.update(item);
}

std::unique_ptr<DownloadModel> DownloadDB::Get(int id) {
    return m_impl->storage.get_pointer<DownloadModel>(id);
}

void DownloadDB::Delete(int id) {
    m_impl->storage.remove<DownloadModel>(id);
}

std::vector<DownloadModel> DownloadDB::GetAll() {
    return m_impl->storage.get_all<DownloadModel>();
}

} // namespace model
} // namespace idr
