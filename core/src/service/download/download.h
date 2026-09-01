#pragma once

#include <string>
#include <vector>
#include <atomic>
#include <chrono>
#include <functional>
#include <mutex>
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

struct DownloadSegment {
    int id{0};
    uint64_t startOffset{0};
    uint64_t endOffset{0};
    uint64_t downloadedBytes{0};
    double speed{0.0};
    bool isCompleted{false};
};

class Download {
public:
    Download(const std::string& url, const std::string& destination, int dbId = 0);
    ~Download();

    // Core lifecycle
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
    int GetSegmentCount() const { return m_segmentCount.load(); }
    bool IsTorrent() const { return m_isTorrent; }
    int GetPeers() const { return m_peers.load(); }
    int GetSeeds() const { return m_seeds.load(); }

    std::vector<DownloadSegment> GetSegments();

    void SetState(uint64_t totalBytes, uint64_t downloadedBytes, DownloadStatus status) {
        m_totalBytes = totalBytes;
        m_downloadedBytes = downloadedBytes;
        m_status = status;
    }

    void SetOnStatusChanged(std::function<void(Download*)> callback) {
        m_statusCallback = callback;
    }

private:
    int m_id{0};
    std::string m_url;
    std::string m_destination;
    std::string m_filename;
    bool m_isTorrent{false};

    std::atomic<uint64_t> m_totalBytes{0};
    std::atomic<uint64_t> m_downloadedBytes{0};
    std::atomic<double> m_speed{0.0};
    std::atomic<DownloadStatus> m_status{DownloadStatus::Queued};
    std::atomic<int> m_segmentCount{1};
    std::atomic<int> m_peers{0};
    std::atomic<int> m_seeds{0};

    std::atomic<bool> m_stopRequested{false};
    std::function<void(Download*)> m_statusCallback;

    std::vector<DownloadSegment> m_segments;
    std::mutex m_segmentMutex;

    void NotifyStatusChanged();
    void StartMultiSegmentDownload(uint64_t totalSize, int numSegments);
    void StartSingleDownload();
    void StartTorrentDownload();
};

} // namespace download
} // namespace idr
