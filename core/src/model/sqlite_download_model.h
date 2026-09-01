#pragma once

#include <string>
#include <vector>
#include <memory>
#include <sqlite_orm/sqlite_orm.h>

namespace idr
{
    namespace model
    {

        using namespace std;

        // -----------------------------------------------------------------------------
        // Model Definition
        // -----------------------------------------------------------------------------
        struct DownloadModel
        {
            int downloads_id = 0;
            string url;
            string destination;
            string filename;
            int64_t total_bytes = 0;
            int64_t downloaded_bytes = 0;
            int status = 0;
            string mime_type;
            string checksum;
            int64_t created_at = 0;
            int64_t updated_at = 0;
        };

        // -----------------------------------------------------------------------------
        // ORM Storage Initializer
        // -----------------------------------------------------------------------------
        inline auto InitDownloadStorage(const string &dbPath)
        {
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

        // -----------------------------------------------------------------------------
        // Download CRUD Database Handler
        // -----------------------------------------------------------------------------
        class DownloadDB
        {
        public:
            using StorageType = decltype(InitDownloadStorage(""));

            explicit DownloadDB(const string &dbPath = "downloads.db")
                : m_storage(InitDownloadStorage(dbPath))
            {
                m_storage.sync_schema();
            }

            int Add(DownloadModel &item)
            {
                return m_storage.insert(item);
            }

            void Update(const DownloadModel &item)
            {
                m_storage.update(item);
            }

            std::unique_ptr<DownloadModel> Get(int id)
            {
                return m_storage.get_pointer<DownloadModel>(id);
            }

            void Delete(int id)
            {
                m_storage.remove<DownloadModel>(id);
            }

            std::vector<DownloadModel> GetAll()
            {
                return m_storage.get_all<DownloadModel>();
            }

        private:
            StorageType m_storage;
        };

    } // namespace model
} // namespace idr
