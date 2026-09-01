#include <string>
#include <sqlite_orm/sqlite_orm.h>

namespace idr {
namespace model {

using namespace std;

// -----------------------------------------------------------------------------
// TOP: Define the Model
// -----------------------------------------------------------------------------
struct FileModel {
    int files_id;
    string path;
    int64_t size;
    int64_t created_at;
    int64_t updated_at;
};

// -----------------------------------------------------------------------------
// BOTTOM: Define the ORM Storage
// -----------------------------------------------------------------------------
inline auto InitFileStorage(const string& dbPath) {
    using namespace sqlite_orm;
    return make_storage(dbPath,
        make_table("files",
            make_column("files_id", &FileModel::files_id, primary_key().autoincrement()),
            make_column("path", &FileModel::path),
            make_column("size", &FileModel::size),
            make_column("created_at", &FileModel::created_at),
            make_column("updated_at", &FileModel::updated_at)
        )
    );
}

} // namespace model
} // namespace idr
