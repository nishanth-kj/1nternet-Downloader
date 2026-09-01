#include <string>
#include <sqlite_orm/sqlite_orm.h>

namespace idr {
namespace model {

using namespace std;

// -----------------------------------------------------------------------------
// TOP: Define the Model
// -----------------------------------------------------------------------------
struct MetadataModel {
    int metadata_id;
    string key;
    string value;
    int64_t created_at;
    int64_t updated_at;
};

// -----------------------------------------------------------------------------
// BOTTOM: Define the ORM Storage
// -----------------------------------------------------------------------------
inline auto InitMetadataStorage(const string& dbPath) {
    using namespace sqlite_orm;
    return make_storage(dbPath,
        make_table("metadata",
            make_column("metadata_id", &MetadataModel::metadata_id, primary_key().autoincrement()),
            make_column("key", &MetadataModel::key, unique()),
            make_column("value", &MetadataModel::value),
            make_column("created_at", &MetadataModel::created_at),
            make_column("updated_at", &MetadataModel::updated_at)
        )
    );
}

} // namespace model
} // namespace idr
