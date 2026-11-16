#pragma once

#include <cstdint>
#include <filesystem>
#include <mutex>
#include <string>

namespace idr {
namespace system {

// User-adjustable settings, persisted as JSON in AppDataDir()/settings.json.
struct Settings {
    std::string downloadDir;          // default folder for new downloads (native narrow path)
    int maxConcurrentDownloads{3};    // downloads allowed in the Downloading state at once
    int maxSegments{8};               // upper bound on parallel ranged connections per download
    int maxRetries{3};                // retries per segment / stream after a transient failure
    int64_t speedLimitKBps{0};        // per-download receive cap in KiB/s, 0 = unlimited

    // Clamps every field into its supported range.
    void Normalize();
};

class Config {
public:
    static Config& Get();

    Settings GetSettings() const;
    // Normalizes, stores and writes the settings to disk. Returns false if the
    // file could not be written (the in-memory settings are still updated).
    bool SetSettings(const Settings& settings);

    std::filesystem::path FilePath() const { return m_path; }

private:
    Config();
    void Load();
    bool SaveLocked() const;

    mutable std::mutex m_mutex;
    Settings m_settings;
    std::filesystem::path m_path;

    Config(const Config&) = delete;
    Config& operator=(const Config&) = delete;
};

} // namespace system
} // namespace idr
