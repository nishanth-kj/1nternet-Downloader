#include <string>
#include <vector>
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
    int status;
    int64_t created_at;
    int64_t updated_at;
};

// -----------------------------------------------------------------------------
// BOTTOM: Define the ORM Storage
// -----------------------------------------------------------------------------
inline auto InitMetadataStorage(const string &dbPath) {
    using namespace sqlite_orm;
    return make_storage(dbPath,
        make_table("metadata",
            make_column("metadata_id", &MetadataModel::metadata_id, primary_key().autoincrement()),
            make_column("key", &MetadataModel::key, unique()),
            make_column("value", &MetadataModel::value),
            make_column("status", &MetadataModel::status),
            make_column("created_at", &MetadataModel::created_at),
            make_column("updated_at", &MetadataModel::updated_at)
        )
    );
}

class MetadataDB {
    decltype(InitMetadataStorage("")) storage;
public:
    MetadataDB(const string &dbPath) : storage(InitMetadataStorage(dbPath)) {
        storage.sync_schema();
    }

    int Add(MetadataModel &item) {
        return storage.insert(item);
    }

    void Update(MetadataModel &item) {
        storage.update(item);
    }

    auto Get(int id) {
        return storage.get_pointer<MetadataModel>(id);
    }

    void Delete(int id) {
        storage.remove<MetadataModel>(id);
    }

    auto GetAll() {
        return storage.get_all<MetadataModel>();
    }
};

} // namespace model
} // namespace idr
