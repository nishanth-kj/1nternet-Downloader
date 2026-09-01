#include <string>
#include <vector>
#include <sqlite_orm/sqlite_orm.h>

namespace idr
{
    namespace model
    {

        using namespace std;

        // -----------------------------------------------------------------------------
        // TOP: Define the Model
        // -----------------------------------------------------------------------------
        struct HistoryModel
        {
            int history_id;
            string download_url;
            int status;
            int64_t created_at;
            int64_t updated_at;
        };

        // -----------------------------------------------------------------------------
        // BOTTOM: Define the ORM Storage
        // -----------------------------------------------------------------------------
        inline auto InitHistoryStorage(const string &dbPath)
        {
            using namespace sqlite_orm;
            return make_storage(dbPath,
                                make_table("history",
                                           make_column("history_id", &HistoryModel::history_id, primary_key().autoincrement()),
                                           make_column("download_url", &HistoryModel::download_url),
                                           make_column("status", &HistoryModel::status),
                                           make_column("created_at", &HistoryModel::created_at),
                                           make_column("updated_at", &HistoryModel::updated_at)));
        }

    }

class HistoryDB {
    decltype(InitHistoryStorage("")) storage;
public:
    HistoryDB(const string& dbPath) : storage(InitHistoryStorage(dbPath)) {
        storage.sync_schema();
    }

    int Add(HistoryModel& item) {
        return storage.insert(item);
    }

    void Update(HistoryModel& item) {
        storage.update(item);
    }

    auto Get(int id) {
        return storage.get_pointer<HistoryModel>(id);
    }

    void Delete(int id) {
        storage.remove<HistoryModel>(id);
    }

    auto GetAll() {
        return storage.get_all<HistoryModel>();
    }
};

} // namespace model
} // namespace idr
