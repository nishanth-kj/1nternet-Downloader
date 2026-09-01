#include "service/download/manager.h"
#include <algorithm>
#include <filesystem>
#include <chrono>

namespace idr {
namespace download {

DownloadManager::DownloadManager() {
    curl_global_init(CURL_GLOBAL_ALL);
    m_db = std::make_unique<idr::model::DownloadDB>("downloads.db");
    LoadFromDatabase();
}

DownloadManager::~DownloadManager() {
    PauseAll();
    curl_global_cleanup();
}

void DownloadManager::LoadFromDatabase() {
    std::lock_guard<std::mutex> lock(m_mutex);
    if (!m_db) return;

    try {
        auto records = m_db->GetAll();
        for (const auto& rec : records) {
            auto dl = std::make_shared<Download>(rec.url, rec.destination, rec.downloads_id);
            DownloadStatus st = static_cast<DownloadStatus>(rec.status);
            // If it was downloading before closing, restore as paused so user can resume
            if (st == DownloadStatus::Downloading) {
                st = DownloadStatus::Paused;
            }
            dl->SetState(rec.total_bytes, rec.downloaded_bytes, st);
            dl->SetOnStatusChanged([this](Download* d) {
                SyncDownloadState(d);
            });
            m_downloads.push_back(dl);
        }
    } catch (...) {
        // Ignore DB load errors on corrupt or initial state
    }
}

std::shared_ptr<Download> DownloadManager::AddDownload(const std::string& url, const std::string& destination, bool autoStart) {
    std::lock_guard<std::mutex> lock(m_mutex);

    int dbId = 0;
    auto now = std::chrono::duration_cast<std::chrono::seconds>(
        std::chrono::system_clock::now().time_since_epoch()).count();

    if (m_db) {
        idr::model::DownloadModel item;
        item.url = url;
        item.destination = destination;
        std::filesystem::path p(destination);
        item.filename = p.filename().string();
        item.status = autoStart ? static_cast<int>(DownloadStatus::Downloading) : static_cast<int>(DownloadStatus::Queued);
        item.created_at = now;
        item.updated_at = now;
        try {
            dbId = m_db->Add(item);
        } catch (...) {
            dbId = static_cast<int>(m_downloads.size() + 1);
        }
    }

    auto dl = std::make_shared<Download>(url, destination, dbId);
    dl->SetOnStatusChanged([this](Download* d) {
        SyncDownloadState(d);
    });

    m_downloads.push_back(dl);

    if (autoStart) {
        dl->Start();
    }

    return dl;
}

bool DownloadManager::RemoveDownload(int id, bool deleteFile) {
    std::lock_guard<std::mutex> lock(m_mutex);

    auto it = std::find_if(m_downloads.begin(), m_downloads.end(), [id](const std::shared_ptr<Download>& d) {
        return d->GetId() == id;
    });

    if (it != m_downloads.end()) {
        auto dl = *it;
        dl->Stop();

        if (deleteFile) {
            std::error_code ec;
            std::filesystem::remove(dl->GetDestination(), ec);
        }

        if (m_db) {
            try {
                m_db->Delete(id);
            } catch (...) {}
        }

        m_downloads.erase(it);
        return true;
    }
    return false;
}

void DownloadManager::PauseDownload(int id) {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        if (dl->GetId() == id) {
            dl->Pause();
            break;
        }
    }
}

void DownloadManager::ResumeDownload(int id) {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        if (dl->GetId() == id) {
            dl->Resume();
            break;
        }
    }
}

void DownloadManager::RestartDownload(int id) {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        if (dl->GetId() == id) {
            dl->Stop();
            std::error_code ec;
            std::filesystem::remove(dl->GetDestination(), ec);
            dl->SetState(0, 0, DownloadStatus::Queued);
            dl->Start();
            break;
        }
    }
}

std::vector<std::shared_ptr<Download>> DownloadManager::GetDownloads() {
    std::lock_guard<std::mutex> lock(m_mutex);
    return m_downloads;
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

void DownloadManager::ClearCompleted() {
    std::lock_guard<std::mutex> lock(m_mutex);
    auto it = m_downloads.begin();
    while (it != m_downloads.end()) {
        if ((*it)->GetStatus() == DownloadStatus::Completed) {
            if (m_db) {
                try {
                    m_db->Delete((*it)->GetId());
                } catch (...) {}
            }
            it = m_downloads.erase(it);
        } else {
            ++it;
        }
    }
}

void DownloadManager::SyncDownloadState(Download* dl) {
    if (!dl || !m_db) return;

    auto now = std::chrono::duration_cast<std::chrono::seconds>(
        std::chrono::system_clock::now().time_since_epoch()).count();

    try {
        idr::model::DownloadModel item;
        item.downloads_id = dl->GetId();
        item.url = dl->GetUrl();
        item.destination = dl->GetDestination();
        item.filename = dl->GetFilename();
        item.total_bytes = dl->GetTotalBytes();
        item.downloaded_bytes = dl->GetDownloadedBytes();
        item.status = static_cast<int>(dl->GetStatus());
        item.updated_at = now;
        m_db->Update(item);
    } catch (...) {}
}

} // namespace download
} // namespace idr
