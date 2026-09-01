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
        struct FileManagerModel
        {
            int file_managers_id;
            string directory_path;
            int64_t total_files;
            int64_t created_at;
            int64_t updated_at;
        };

        // -----------------------------------------------------------------------------
        // BOTTOM: Define the ORM Storage
        // -----------------------------------------------------------------------------
        inline auto InitFileManagerStorage(const string &dbPath)
        {
            using namespace sqlite_orm;
            return make_storage(dbPath,
                                make_table("file_managers",
                                           make_column("file_managers_id", &FileManagerModel::file_managers_id, primary_key().autoincrement()),
                                           make_column("directory_path", &FileManagerModel::directory_path),
                                           make_column("total_files", &FileManagerModel::total_files),
                                           make_column("created_at", &FileManagerModel::created_at),
                                           make_column("updated_at", &FileManagerModel::updated_at)));
        }
        
        class FileManagerDB {
            decltype(InitFileManagerStorage("")) storage;
        public:
            FileManagerDB(const string& dbPath) : storage(InitFileManagerStorage(dbPath)) {
                storage.sync_schema();
            }
        
            int Add(FileManagerModel& item) {
                return storage.insert(item);
            }
        
            void Update(FileManagerModel& item) {
                storage.update(item);
            }
        
            auto Get(int id) {
                return storage.get_pointer<FileManagerModel>(id);
            }
        
            void Delete(int id) {
                storage.remove<FileManagerModel>(id);
            }
        
            auto GetAll() {
                return storage.get_all<FileManagerModel>();
            }
        };

    } // namespace model
} // namespace idr
