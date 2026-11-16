#include "utils/system/config.h"

#include "utils/system/logger.h"
#include "utils/system/platform.h"

#include <algorithm>
#include <fstream>

#include <nlohmann/json.hpp>

namespace idr {
namespace system {

void Settings::Normalize() {
    maxConcurrentDownloads = std::clamp(maxConcurrentDownloads, 1, 16);
    maxSegments = std::clamp(maxSegments, 1, 32);
    maxRetries = std::clamp(maxRetries, 0, 20);
    speedLimitKBps = std::max<int64_t>(0, speedLimitKBps);
    if (downloadDir.empty()) downloadDir = DefaultDownloadsDir().string();
}

Config& Config::Get() {
    static Config instance;
    return instance;
}

Config::Config() : m_path(AppDataDir() / "settings.json") {
    Load();
}

void Config::Load() {
    std::lock_guard<std::mutex> lock(m_mutex);
    Settings s;
    std::ifstream in(m_path);
    if (in.is_open()) {
        try {
            const auto j = nlohmann::json::parse(in);
            s.downloadDir = j.value("downloadDir", s.downloadDir);
            s.maxConcurrentDownloads = j.value("maxConcurrentDownloads", s.maxConcurrentDownloads);
            s.maxSegments = j.value("maxSegments", s.maxSegments);
            s.maxRetries = j.value("maxRetries", s.maxRetries);
            s.speedLimitKBps = j.value("speedLimitKBps", s.speedLimitKBps);
        } catch (const std::exception& e) {
            Log()->warn("settings.json is invalid, using defaults: {}", e.what());
            s = Settings{};
        }
    }
    s.Normalize();
    m_settings = s;
}

Settings Config::GetSettings() const {
    std::lock_guard<std::mutex> lock(m_mutex);
    return m_settings;
}

bool Config::SetSettings(const Settings& settings) {
    std::lock_guard<std::mutex> lock(m_mutex);
    m_settings = settings;
    m_settings.Normalize();
    return SaveLocked();
}

bool Config::SaveLocked() const {
    nlohmann::json j{
        {"downloadDir", m_settings.downloadDir},
        {"maxConcurrentDownloads", m_settings.maxConcurrentDownloads},
        {"maxSegments", m_settings.maxSegments},
        {"maxRetries", m_settings.maxRetries},
        {"speedLimitKBps", m_settings.speedLimitKBps},
    };

    // Write-then-rename so a crash mid-write can't leave a truncated settings file.
    const auto tmp = m_path.string() + ".tmp";
    {
        std::ofstream out(tmp, std::ios::trunc);
        if (!out.is_open()) return false;
        out << j.dump(2);
        if (!out.good()) return false;
    }
    std::error_code ec;
    std::filesystem::rename(tmp, m_path, ec);
    if (ec) {
        Log()->error("failed to save settings to {}: {}", m_path.string(), ec.message());
        return false;
    }
    return true;
}

} // namespace system
} // namespace idr
