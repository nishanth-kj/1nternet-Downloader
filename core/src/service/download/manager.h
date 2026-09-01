#pragma once

#include "service/download/download.h"
#include "model/sqlite_download_model.h"
#include <vector>
#include <memory>
#include <mutex>

namespace idr {
namespace download {

class DownloadManager {
public:
    static DownloadManager& GetInstance() {
        static DownloadManager instance;
        return instance;
    }

    // Add a new download to the manager and persist to database
    std::shared_ptr<Download> AddDownload(const std::string& url, const std::string& destination, bool autoStart = true);

    // Remove a download by database ID
    bool RemoveDownload(int id, bool deleteFile = false);

    // Control single download
    void PauseDownload(int id);
    void ResumeDownload(int id);
    void RestartDownload(int id);

    // Get all downloads
    std::vector<std::shared_ptr<Download>> GetDownloads();

    // Control all
    void PauseAll();
    void ResumeAll();
    void ClearCompleted();

    // Database sync
    void SyncDownloadState(Download* dl);

private:
    DownloadManager();
    ~DownloadManager();

    void LoadFromDatabase();

    std::vector<std::shared_ptr<Download>> m_downloads;
    std::mutex m_mutex;
    std::unique_ptr<idr::model::DownloadDB> m_db;

    // Singleton semantics
    DownloadManager(const DownloadManager&) = delete;
    DownloadManager& operator=(const DownloadManager&) = delete;
};

} // namespace download
} // namespace idr
