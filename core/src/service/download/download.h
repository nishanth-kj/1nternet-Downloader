#pragma once

#include <string>
#include <vector>
#include <atomic>
#include <chrono>
#include <functional>
#include <memory>
#include <mutex>
#include <thread>

#include "service/download/progress.h"

namespace idr {
namespace system {
struct Settings;
}
namespace torrent {
class TorrentTask;
}
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
    // Marks a paused/failed download as waiting for a free slot; the
    // DownloadManager's queue starts it when one is available.
    void Enqueue();
    // Wait for the owned coordinator and all of its child workers to finish.
    void WaitForWorker();

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
    std::string GetErrorMessage() const;

    std::vector<DownloadSegment> GetSegments();

    void SetState(uint64_t totalBytes, uint64_t downloadedBytes, DownloadStatus status) {
        m_totalBytes = totalBytes;
        m_downloadedBytes = downloadedBytes;
        m_status = status;
    }

    void SetOnStatusChanged(std::function<void(Download*)> callback) {
        m_statusCallback = callback;
    }

    // Path of the temporary file holding segment `index` of a segmented download.
    std::string PartPath(int index) const;
    // Deletes every temporary artifact (.partN, .assembling, ...) for this download.
    void RemoveTemporaryFiles() const;

    // Releases the underlying libtorrent resources (if any). Called by DownloadManager
    // before deleting the download's files, so libtorrent isn't still writing to them.
    void ReleaseTorrentResources();

private:
    enum class AttemptResult { Completed, Stopped, Retryable, Fatal };

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
    // Set when a segment's ranged GET came back as a full 200 body despite the HEAD
    // advertising Accept-Ranges; the coordinator then falls back to a single stream.
    std::atomic<bool> m_rangeRejected{false};
    std::function<void(Download*)> m_statusCallback;

    std::vector<DownloadSegment> m_segments;
    std::mutex m_segmentMutex;

    mutable std::mutex m_errorMutex;
    std::string m_errorMessage;

    SpeedMeter m_speedMeter;

    // Own the coordinator worker so destruction cannot race detached callbacks.
    std::mutex m_workerMutex;
    std::thread m_workerThread;

    std::mutex m_torrentMutex;
    std::shared_ptr<idr::torrent::TorrentTask> m_torrentTask;

    void NotifyStatusChanged();
    void SetError(const std::string& message);
    void Fail(const std::string& message);
    // Sleeps up to `duration`, returning early (false) if a stop was requested.
    bool InterruptibleSleep(std::chrono::milliseconds duration) const;
    int CountExistingParts() const;

    void Run();
    void StartMultiSegmentDownload(uint64_t totalSize, int numSegments,
                                   const idr::system::Settings& settings);
    void RunSegment(int index, int64_t maxRecvBytesPerSec, int maxRetries);
    bool AssembleParts(int numSegments);

    void StartSingleDownload(const idr::system::Settings& settings, bool serverAcceptsRanges);
    AttemptResult AttemptSingleDownload(const idr::system::Settings& settings, bool& allowResume);

    void StartTorrentDownload();
};

} // namespace download
} // namespace idr
