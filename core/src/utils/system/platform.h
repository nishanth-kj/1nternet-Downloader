#pragma once

#include <filesystem>

namespace idr {
namespace system {

// Per-user application data directory (%APPDATA%\InternetDownloader on Windows).
// Created on first call. Holds downloads.db, settings.json and logs/ so they don't
// depend on the process's working directory (an idr:// launch from a browser
// inherits the browser's CWD, which may not even be writable).
std::filesystem::path AppDataDir();

// The user's "Downloads" known folder, falling back to the home directory.
std::filesystem::path DefaultDownloadsDir();

} // namespace system
} // namespace idr
