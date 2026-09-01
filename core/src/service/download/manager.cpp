#include "download/manager.h"

namespace idr {
namespace download {

DownloadManager::DownloadManager() {
    // Curl global init should theoretically be called once per app
    curl_global_init(CURL_GLOBAL_ALL);
}

DownloadManager::~DownloadManager() {
    curl_global_cleanup();
}

std::shared_ptr<Download> DownloadManager::AddDownload(const std::string& url, const std::string& destination) {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    auto dl = std::make_shared<Download>(url, destination);
    m_downloads.push_back(dl);
    
    // Automatically start it
    dl->Start();
    
    return dl;
}

std::vector<std::shared_ptr<Download>> DownloadManager::GetDownloads() {
    std::lock_guard<std::mutex> lock(m_mutex);
    return m_downloads; // Returns a copy of the shared_ptr vector
}

void DownloadManager::PauseAll() {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        dl->Pause();
    }
}

void DownloadManager::ResumeAll() {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        dl->Resume();
    }
}

} // namespace download
} // namespace idr
