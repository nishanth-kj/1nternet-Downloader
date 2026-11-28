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

// Plain-data views for the details window (no libtorrent types leak out of the .cpp).
struct TorrentFileInfo {
    int index{0};
    std::string path;
    uint64_t size{0};
    uint64_t done{0};
    int priority{4}; // 0 = skip, 1..7 (4 = normal, 7 = top)
};

struct TorrentTrackerInfo {
    std::string url;
    std::string message;
    int tier{0};
    int status{0}; // 0 = not contacted, 1 = working, 2 = updating, 3 = error
    int seeds{-1};
    int peers{-1};
    int downloaded{-1};
};

struct TorrentPeerInfo {
    std::string address;
    std::string client;
    double downSpeed{0.0};
    double upSpeed{0.0};
    float progress{0.0f};
    uint64_t downloaded{0};
    uint64_t uploaded{0};
    bool seed{false};
};

struct TorrentExtraInfo {
    uint64_t allTimeUpload{0};
    uint64_t allTimeDownload{0};
    int64_t seedingSeconds{0};
    int64_t activeSeconds{0};
    int numPieces{0};
    int piecesHave{0};
    int pieceLength{0};
    std::string infoHash;
    std::string savePath;
    std::string comment;
    std::string creator;
};

// Session-wide options (Options › Torrent). Applied now if the session exists,
// otherwise when it is first created.
struct TorrentSessionSettings {
    int listenPort{6881};
    bool randomPort{false};
    bool dht{true};
    bool lsd{true};
    bool pex{true};          // applies to torrents added afterwards
    bool utp{true};
    bool upnp{true};
    bool natpmp{true};
    int encryption{1};       // 0 = require, 1 = prefer (allow both), 2 = disable
    int maxConnections{200};
    int maxPeersPerTorrent{50};
    int downloadLimitKBps{0}; // 0 = unlimited
    int uploadLimitKBps{0};
    bool announceToAllTrackers{false};
    bool validateHttpsTrackers{true};
    std::string userAgent{"1nternet Downloader"};
};

// Builds a .torrent file from a file or folder. Returns an empty string on success,
// otherwise an error message.
struct TorrentCreateOptions {
    std::string sourcePath;
    std::string outputFile;
    std::vector<std::string> trackers;  // one tier each, in order
    std::vector<std::string> webSeeds;
    std::string comment;
    int pieceSizeKiB{0};                // 0 = automatic
    bool isPrivate{false};
};
std::string CreateTorrentFile(const TorrentCreateOptions& options);

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

    // Human-readable error from libtorrent, or an empty string when there is none.
    std::string GetError() const;

    // Details-window queries. Empty results before a magnet link has its metadata.
    std::vector<TorrentFileInfo> GetFiles() const;
    void SetFilePriority(int index, int priority);
    std::vector<TorrentTrackerInfo> GetTrackers() const;
    std::vector<TorrentPeerInfo> GetPeers() const;
    TorrentExtraInfo GetExtra() const;
    std::vector<uint8_t> GetPieces() const; // one byte per piece: 1 = have

    void SetSequential(bool on);
    bool IsSequential() const;
    void AddTracker(const std::string& url);
    void ForceRecheck();
    void ForceReannounce();

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
    // With deleteFiles, libtorrent also deletes the downloaded data.
    void RemoveTorrent(int id, bool deleteFiles = false);

    void PauseAll();
    void ResumeAll();

    // Folder for fast-resume files (<info-hash>.fastresume). Set before adding torrents;
    // AddTorrent then resumes from a matching file instead of re-checking all data.
    void SetResumeDir(const std::string& dir);

    // Stores the options and applies them to the live session (if there is one).
    static void ApplySettings(const TorrentSessionSettings& settings);

    // Asks every torrent with unsaved changes for resume data and writes the files.
    // Blocks up to timeoutMs. Returns the number of files written.
    int SaveResumeData(int timeoutMs);

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
