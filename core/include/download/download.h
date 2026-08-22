#pragma once

#include <string>
#include <atomic>
#include <chrono>
#include <curl/curl.h>

namespace idr {
namespace download {

enum class DownloadStatus {
    Queued,
    Downloading,
    Paused,
    Completed,
    Error
};

class Download {
public:
    Download(const std::string& url, const std::string& destination);
    ~Download();

    // Core functionality
    void Start();
    void Pause();
    void Resume();
    void Stop();

    // Getters
    std::string GetUrl() const { return m_url; }
    std::string GetFilename() const { return m_filename; }
    uint64_t GetTotalBytes() const { return m_totalBytes; }
    uint64_t GetDownloadedBytes() const { return m_downloadedBytes; }
    double GetSpeedBytesPerSec() const { return m_speed; }
    DownloadStatus GetStatus() const { return m_status; }

    // Internal libcurl callbacks
    size_t WriteCallback(void* contents, size_t size, size_t nmemb);
    int ProgressCallback(curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow);

private:
    std::string m_url;
    std::string m_destination;
    std::string m_filename;
    
    std::atomic<uint64_t> m_totalBytes{0};
    std::atomic<uint64_t> m_downloadedBytes{0};
    std::atomic<double> m_speed{0.0};
    std::atomic<DownloadStatus> m_status{DownloadStatus::Queued};

    std::atomic<bool> m_stopRequested{false};
    
    // Time tracking for speed calculations
    std::chrono::steady_clock::time_point m_lastSpeedUpdate;
    uint64_t m_bytesAtLastUpdate{0};
};

} // namespace download
} // namespace idr
