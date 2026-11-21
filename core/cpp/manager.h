#pragma once

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

// This header intentionally does not include any libtorrent headers: the real
// lt::session/lt::torrent_handle live behind a pimpl in torrent_manager.cpp, so
// consumers (the download engine, the GUI) never pay libtorrent's compile-time cost
// and never need to link against it directly.

namespace idr {
namespace torrent {

enum class TorrentStatus {
    Queued = 0,
    Downloading = 1,
    Paused = 2,
    Completed = 3,
    Seeding = 4,
    Error = 5
};

struct TorrentSnapshot {
    std::string name;
    uint64_t totalBytes{0};
    uint64_t downloadedBytes{0};
    uint64_t uploadedBytes{0};
    double downloadSpeed{0.0};
    double uploadSpeed{0.0};
    int peers{0};
    int seeds{0};
    TorrentStatus status{TorrentStatus::Queued};
};

class TorrentManager;

// A single torrent/magnet download backed by a real libtorrent session. Instances
// are only ever created by TorrentManager.
class TorrentTask {
public:
    ~TorrentTask();

    int GetId() const { return m_id; }
    std::string GetSource() const { return m_source; } // magnet URI or .torrent file path
    std::string GetDestinationDir() const { return m_destinationDir; }

    // Queries live state from libtorrent. Safe to call frequently (e.g. from a UI
    // poll timer); each call performs one round trip into the libtorrent session.
    TorrentSnapshot GetSnapshot() const;

    void Pause();
    void Resume();
    void Stop();

private:
    friend class TorrentManager;
    TorrentTask(int id, std::string source, std::string destinationDir);
    TorrentTask(const TorrentTask&) = delete;
    TorrentTask& operator=(const TorrentTask&) = delete;

    struct Impl;
    std::unique_ptr<Impl> m_impl;

    int m_id;
    std::string m_source;
    std::string m_destinationDir;
};

// Process-wide owner of the single libtorrent session. Mirrors DownloadManager's
// singleton shape so the GUI can treat torrents and HTTP downloads symmetrically.
class TorrentManager {
public:
    static TorrentManager& GetInstance();

    // `destinationDir` is a directory: libtorrent writes files using the names/layout
    // stored in the torrent metadata, so (unlike HTTP downloads) callers pick a folder,
    // not an exact file name. Returns nullptr if the magnet/.torrent file is invalid.
    std::shared_ptr<TorrentTask> AddTorrent(const std::string& source, const std::string& destinationDir, bool autoStart = true);

    std::vector<std::shared_ptr<TorrentTask>> GetTorrents();
    std::shared_ptr<TorrentTask> FindTorrent(int id);

    // Detaches the torrent from the session, freeing its libtorrent resources.
    void RemoveTorrent(int id);

    void PauseAll();
    void ResumeAll();

private:
    TorrentManager();
    ~TorrentManager();
    TorrentManager(const TorrentManager&) = delete;
    TorrentManager& operator=(const TorrentManager&) = delete;

    struct Impl;
    std::unique_ptr<Impl> m_impl;
};

} // namespace torrent
} // namespace idr
