#pragma once

#include <string>
#include <atomic>
#include <chrono>
#include <functional>
#include <curl/curl.h>

namespace idr {
namespace download {

enum class DownloadStatus {
    Queued = 0,
    Downloading = 1,
    Paused = 2,
    Completed = 3,
    Error = 4
};

class Download {
public:
    Download(const std::string& url, const std::string& destination, int dbId = 0);
    ~Download();

    // Core functionality
    void Start();
    void Pause();
    void Resume();
    void Stop();

    // Getters & Setters
    int GetId() const { return m_id; }
    void SetId(int id) { m_id = id; }
    std::string GetUrl() const { return m_url; }
    std::string GetDestination() const { return m_destination; }
    std::string GetFilename() const { return m_filename; }
    uint64_t GetTotalBytes() const { return m_totalBytes.load(); }
    uint64_t GetDownloadedBytes() const { return m_downloadedBytes.load(); }
    double GetSpeedBytesPerSec() const { return m_speed.load(); }
    DownloadStatus GetStatus() const { return m_status.load(); }

    void SetState(uint64_t totalBytes, uint64_t downloadedBytes, DownloadStatus status) {
        m_totalBytes = totalBytes;
        m_downloadedBytes = downloadedBytes;
        m_status = status;
    }

    void SetOnStatusChanged(std::function<void(Download*)> callback) {
        m_statusCallback = callback;
    }

    // Internal libcurl callbacks
    size_t WriteCallback(void* contents, size_t size, size_t nmemb);
    int ProgressCallback(curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow);

private:
    int m_id{0};
    std::string m_url;
    std::string m_destination;
    std::string m_filename;

    std::atomic<uint64_t> m_totalBytes{0};
    std::atomic<uint64_t> m_downloadedBytes{0};
    std::atomic<uint64_t> m_startOffset{0};
    std::atomic<double> m_speed{0.0};
    std::atomic<DownloadStatus> m_status{DownloadStatus::Queued};

    std::atomic<bool> m_stopRequested{false};

    // Time tracking for speed calculations
    std::chrono::steady_clock::time_point m_lastSpeedUpdate;
    uint64_t m_bytesAtLastUpdate{0};

    std::function<void(Download*)> m_statusCallback;

    void NotifyStatusChanged();
};

} // namespace download
} // namespace idr
