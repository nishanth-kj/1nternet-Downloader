#include <string>
#include <sqlite_orm/sqlite_orm.h>

namespace idr
{
    namespace model
    {

        using namespace std;

        // -----------------------------------------------------------------------------
        // TOP: Define the Model
        // -----------------------------------------------------------------------------
        struct DownloadModel
        {
            int downloads_id;
            string url;
            string destination;
            string filename;
            int64_t total_bytes;
            int64_t downloaded_bytes;
            int status;
            string mime_type;
            string checksum;
            int64_t created_at;
            int64_t updated_at;
        };

        // -----------------------------------------------------------------------------
        // BOTTOM: Define the ORM Storage
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
        
        class DownloadDB {
            decltype(InitDownloadStorage("")) storage;
        public:
            DownloadDB(const string& dbPath) : storage(InitDownloadStorage(dbPath)) {
                storage.sync_schema();
            }
        
            int Add(DownloadModel& item) {
                return storage.insert(item);
            }
        
            void Update(DownloadModel& item) {
                storage.update(item);
            }
        
            auto Get(int id) {
                return storage.get_pointer<DownloadModel>(id);
            }
        
            void Delete(int id) {
                storage.remove<DownloadModel>(id);
            }
        
            auto GetAll() {
                return storage.get_all<DownloadModel>();
            }
        };

    } // namespace model
} // namespace idr
