#pragma once

// Not used. The application's SQLite persistence lives entirely in
// idr::model::DownloadDB (see src/model/sqlite_download_model.h), which wraps
// sqlite_orm directly. This header is kept as a placeholder in case a
// lower-level, non-ORM database utility is ever needed; nothing includes it today.
