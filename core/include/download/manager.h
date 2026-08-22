#pragma once

#include "download.h"
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

    // Add a new download to the manager
    std::shared_ptr<Download> AddDownload(const std::string& url, const std::string& destination);

    // Get all active/completed downloads
    std::vector<std::shared_ptr<Download>> GetDownloads();

    // Control all
    void PauseAll();
    void ResumeAll();

private:
    DownloadManager();
    ~DownloadManager();

    std::vector<std::shared_ptr<Download>> m_downloads;
    std::mutex m_mutex;

    // We disable copying for Singleton
    DownloadManager(const DownloadManager&) = delete;
    DownloadManager& operator=(const DownloadManager&) = delete;
};

} // namespace download
} // namespace idr
