#include <string>
#include <vector>
#include <sqlite_orm/sqlite_orm.h>

namespace idr {
namespace model {

using namespace std;

// -----------------------------------------------------------------------------
// TOP: Define the Model
// -----------------------------------------------------------------------------
struct HistoryModel {
    int history_id;
    string download_url;
    int status;
    int64_t created_at;
    int64_t updated_at;
};

// -----------------------------------------------------------------------------
// BOTTOM: Define the ORM Storage
// -----------------------------------------------------------------------------
inline auto InitHistoryStorage(const string& dbPath) {
    using namespace sqlite_orm;
    return make_storage(dbPath,
        make_table("history",
            make_column("history_id", &HistoryModel::history_id, primary_key().autoincrement()),
            make_column("download_url", &HistoryModel::download_url),
            make_column("status", &HistoryModel::status),
            make_column("created_at", &HistoryModel::created_at),
            make_column("updated_at", &HistoryModel::updated_at)
        )
    );
}

} // namespace model
} // namespace idr
