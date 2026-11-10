#include "utils/system/platform.h"

#include <cstdlib>
#include <system_error>

#ifdef _WIN32
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <knownfolders.h>
#include <shlobj.h>
#endif

namespace idr {
namespace system {

namespace {

#ifdef _WIN32
std::filesystem::path KnownFolder(REFKNOWNFOLDERID id) {
    PWSTR raw = nullptr;
    std::filesystem::path result;
    if (SUCCEEDED(SHGetKnownFolderPath(id, KF_FLAG_CREATE, nullptr, &raw)) && raw) {
        result = std::filesystem::path(raw);
    }
    if (raw) CoTaskMemFree(raw);
    return result;
}
#endif

std::filesystem::path HomeDir() {
#ifdef _WIN32
    if (const wchar_t* profile = _wgetenv(L"USERPROFILE")) return std::filesystem::path(profile);
#else
    if (const char* home = std::getenv("HOME")) return std::filesystem::path(home);
#endif
    return std::filesystem::current_path();
}

} // namespace

std::filesystem::path AppDataDir() {
    std::filesystem::path base;
#ifdef _WIN32
    base = KnownFolder(FOLDERID_RoamingAppData);
#endif
    if (base.empty()) base = HomeDir();

    std::filesystem::path dir = base / "InternetDownloader";
    std::error_code ec;
    std::filesystem::create_directories(dir, ec);
    return dir;
}

std::filesystem::path DefaultDownloadsDir() {
    std::filesystem::path dir;
#ifdef _WIN32
    dir = KnownFolder(FOLDERID_Downloads);
#endif
    if (dir.empty()) dir = HomeDir() / "Downloads";
    return dir;
}

} // namespace system
} // namespace idr
