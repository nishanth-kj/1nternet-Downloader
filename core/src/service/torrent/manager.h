#pragma once

#include "service/torrent/metainfo.h"
#include <string>
#include <vector>
#include <memory>
#include <mutex>
#include <atomic>
#include <thread>

namespace idr {
namespace torrent {

class TorrentTask {
public:
    TorrentTask(const std::string& source, const std::string& destination, int dbId = 0);
    ~TorrentTask();

    void Start();
    void Pause();
    void Resume();
    void Stop();

    int GetId() const { return m_id; }
    std::string GetName() const { return m_metaInfo.name; }
    std::string GetSource() const { return m_source; }
    std::string GetDestination() const { return m_destination; }
    uint64_t GetTotalBytes() const { return m_metaInfo.totalLength; }
    uint64_t GetDownloadedBytes() const { return m_downloadedBytes.load(); }
    uint64_t GetUploadedBytes() const { return m_uploadedBytes.load(); }
    double GetDownloadSpeed() const { return m_downSpeed.load(); }
    double GetUploadSpeed() const { return m_upSpeed.load(); }
    int GetPeersCount() const { return m_peersCount.load(); }
    int GetSeedsCount() const { return m_seedsCount.load(); }
    int GetStatus() const { return m_status.load(); } // 0=Queued, 1=Downloading, 2=Paused, 3=Seeding/Completed, 4=Error

    TorrentMetaInfo GetMetaInfo() const { return m_metaInfo; }

private:
    int m_id{0};
    std::string m_source; // Magnet URI or .torrent path
    std::string m_destination;
    TorrentMetaInfo m_metaInfo;

    std::atomic<uint64_t> m_downloadedBytes{0};
    std::atomic<uint64_t> m_uploadedBytes{0};
    std::atomic<double> m_downSpeed{0.0};
    std::atomic<double> m_upSpeed{0.0};
    std::atomic<int> m_peersCount{0};
    std::atomic<int> m_seedsCount{0};
    std::atomic<int> m_status{0};
    std::atomic<bool> m_stopRequested{false};

    void WorkerLoop();
};

class TorrentManager {
public:
    static TorrentManager& GetInstance() {
        static TorrentManager instance;
        return instance;
    }

    std::shared_ptr<TorrentTask> AddTorrent(const std::string& source, const std::string& destination, bool autoStart = true);
    std::vector<std::shared_ptr<TorrentTask>> GetTorrents();
    void PauseAll();
    void ResumeAll();

private:
    TorrentManager() = default;
    ~TorrentManager() = default;

    std::vector<std::shared_ptr<TorrentTask>> m_torrents;
    std::mutex m_mutex;

    TorrentManager(const TorrentManager&) = delete;
    TorrentManager& operator=(const TorrentManager&) = delete;
};

} // namespace torrent
} // namespace idr
