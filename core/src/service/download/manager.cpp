#include "service/download/manager.h"
#include "model/sqlite_download_model.h"
#include "utils/system/config.h"
#include "utils/system/logger.h"
#include "utils/system/platform.h"
#include <algorithm>
#include <chrono>
#include <curl/curl.h>
#include <filesystem>

#ifdef _WIN32
#include <windows.h>
#endif

namespace idr {
namespace download {

namespace {
int64_t NowSeconds() {
    return std::chrono::duration_cast<std::chrono::seconds>(
        std::chrono::system_clock::now().time_since_epoch()).count();
}

std::string DefaultDatabasePath() {
    const std::filesystem::path dbName("downloads.db");
#ifdef _WIN32
    wchar_t modulePath[MAX_PATH]{};
    if (GetModuleFileNameW(nullptr, modulePath, MAX_PATH) > 0) {
        std::filesystem::path dir = std::filesystem::path(modulePath).parent_path();
        for (int i = 0; i < 6 && !dir.empty(); ++i) {
            if (std::filesystem::exists(dir / "vcpkg.json") ||
                std::filesystem::exists(dir / "CMakeLists.txt")) {
                return (dir / dbName).string();
            }
            dir = dir.parent_path();
        }
        return (std::filesystem::path(modulePath).parent_path() / dbName).string();
    }
#endif
    return dbName.string();
}
} // namespace

DownloadManager::DownloadManager() {
    curl_global_init(CURL_GLOBAL_ALL);
    try {
        const auto dbPath = idr::system::AppDataDir() / "downloads.db";
        m_db = std::make_unique<idr::model::DownloadDB>(dbPath.u8string());
    } catch (const std::exception& e) {
        idr::system::Log()->error("cannot open app-data database: {}", e.what());
        m_db = std::make_unique<idr::model::DownloadDB>(DefaultDatabasePath());
    }
    LoadFromDatabase();
}

DownloadManager::~DownloadManager() {
    PauseAll();

    auto downloads = GetDownloads();
    for (const auto& download : downloads) {
        download->WaitForWorker();
        // Externally held Download handles must not callback into a manager
        // whose database and mutex are being destroyed.
        download->SetOnStatusChanged({});
    }

    {
        std::lock_guard<std::mutex> lock(m_mutex);
        m_downloads.clear();
    }
    curl_global_cleanup();
}

void DownloadManager::LoadFromDatabase() {
    std::lock_guard<std::mutex> lock(m_mutex);
    if (!m_db) return;

    try {
        std::vector<idr::model::DownloadModel> records;
        {
            std::lock_guard<std::mutex> dbLock(m_dbMutex);
            records = m_db->GetAll();
        }
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
    } catch (const std::exception& e) {
        idr::system::Log()->error("failed to load downloads from database: {}", e.what());
    }
}

std::shared_ptr<Download> DownloadManager::AddDownload(const std::string& url, const std::string& destination, bool autoStart) {
    std::lock_guard<std::mutex> lock(m_mutex);

    int dbId = 0;
    const auto now = NowSeconds();

    if (m_db) {
        idr::model::DownloadModel item;
        item.url = url;
        item.destination = destination;
        std::filesystem::path p(destination);
        item.filename = p.filename().string();
        item.status = static_cast<int>(autoStart ? DownloadStatus::Queued : DownloadStatus::Paused);
        item.created_at = now;
        item.updated_at = now;
        try {
            std::lock_guard<std::mutex> dbLock(m_dbMutex);
            dbId = m_db->Add(item);
        } catch (const std::exception& e) {
            idr::system::Log()->error("failed to persist new download: {}", e.what());
        }
    }
    if (dbId <= 0) {
        // Not persisted: still give it a unique in-memory id.
        int maxId = 0;
        for (const auto& d : m_downloads) maxId = (std::max)(maxId, d->GetId());
        dbId = maxId + 1;
    }

    auto dl = std::make_shared<Download>(url, destination, dbId);
    dl->SetState(0, 0, autoStart ? DownloadStatus::Queued : DownloadStatus::Paused);
    dl->SetOnStatusChanged([this](Download* d) {
        SyncDownloadState(d);
    });

    m_downloads.push_back(dl);
    idr::system::Log()->info("download #{} added: {}", dbId, url);

    if (autoStart) ProcessQueueLocked();
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
        dl->WaitForWorker();
        dl->ReleaseTorrentResources();
        dl->SetOnStatusChanged({});

        // Temporary segment files are never useful once the entry is gone.
        dl->RemoveTemporaryFiles();
        if (deleteFile) {
            std::error_code ec;
            std::filesystem::remove(dl->GetDestination(), ec);
        }

        if (m_db) {
            try {
                std::lock_guard<std::mutex> dbLock(m_dbMutex);
                m_db->Delete(id);
            } catch (...) {}
        }

        m_downloads.erase(it);
        ProcessQueueLocked();
        return true;
    }
    return false;
}

std::shared_ptr<Download> DownloadManager::FindDownload(int id) {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        if (dl->GetId() == id) return dl;
    }
    return nullptr;
}

void DownloadManager::PauseDownload(int id) {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        if (dl->GetId() == id) {
            dl->Pause();
            break;
        }
    }
    ProcessQueueLocked();
}

void DownloadManager::EnqueueLocked(const std::shared_ptr<Download>& dl) {
    const auto st = dl->GetStatus();
    if (st == DownloadStatus::Paused || st == DownloadStatus::Error) {
        // Join a finished/aborted worker before it is restarted by the queue.
        dl->WaitForWorker();
        dl->Enqueue();
    }
}

void DownloadManager::ResumeDownload(int id) {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        if (dl->GetId() == id) {
            EnqueueLocked(dl);
            break;
        }
    }
    ProcessQueueLocked();
}

void DownloadManager::RestartDownload(int id) {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        if (dl->GetId() == id) {
            dl->Stop();
            // The worker must be gone before its files are deleted underneath it.
            dl->WaitForWorker();
            if (!dl->IsTorrent()) {
                dl->RemoveTemporaryFiles();
                std::error_code ec;
                std::filesystem::remove(dl->GetDestination(), ec);
            }
            dl->SetState(0, 0, DownloadStatus::Queued);
            SyncDownloadState(dl.get());
            break;
        }
    }
    ProcessQueueLocked();
}

std::vector<std::shared_ptr<Download>> DownloadManager::GetDownloads() {
    std::lock_guard<std::mutex> lock(m_mutex);
    return m_downloads;
}

void DownloadManager::PauseAll() {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        const auto st = dl->GetStatus();
        if (st == DownloadStatus::Downloading || st == DownloadStatus::Queued) {
            dl->Pause();
        }
    }
}

void DownloadManager::ResumeAll() {
    std::lock_guard<std::mutex> lock(m_mutex);
    for (auto& dl : m_downloads) {
        EnqueueLocked(dl);
    }
    ProcessQueueLocked();
}

void DownloadManager::ClearCompleted() {
    std::lock_guard<std::mutex> lock(m_mutex);
    auto it = m_downloads.begin();
    while (it != m_downloads.end()) {
        if ((*it)->GetStatus() == DownloadStatus::Completed) {
            auto dl = *it;
            // Completed torrents keep seeding inside libtorrent until released.
            dl->Stop();
            dl->WaitForWorker();
            dl->ReleaseTorrentResources();
            dl->SetOnStatusChanged({});
            if (m_db) {
                try {
                    std::lock_guard<std::mutex> dbLock(m_dbMutex);
                    m_db->Delete(dl->GetId());
                } catch (...) {}
            }
            it = m_downloads.erase(it);
        } else {
            ++it;
        }
    }
}

void DownloadManager::ProcessQueue() {
    std::lock_guard<std::mutex> lock(m_mutex);
    ProcessQueueLocked();
}

void DownloadManager::ProcessQueueLocked() {
    const int limit = idr::system::Config::Get().GetSettings().maxConcurrentDownloads;
    int active = 0;
    for (const auto& dl : m_downloads) {
        if (dl->GetStatus() == DownloadStatus::Downloading) ++active;
    }
    for (auto& dl : m_downloads) {
        if (active >= limit) break;
        if (dl->GetStatus() == DownloadStatus::Queued) {
            dl->Start();
            ++active;
        }
    }
}

void DownloadManager::SyncDownloadState(Download* dl) {
    if (!dl || !m_db) return;

    try {
        idr::model::DownloadModel item;
        item.downloads_id = dl->GetId();
        item.url = dl->GetUrl();
        item.destination = dl->GetDestination();
        item.filename = dl->GetFilename();
        item.total_bytes = static_cast<int64_t>(dl->GetTotalBytes());
        item.downloaded_bytes = static_cast<int64_t>(dl->GetDownloadedBytes());
        item.status = static_cast<int>(dl->GetStatus());

        std::lock_guard<std::mutex> dbLock(m_dbMutex);
        // Preserve created_at (and any other columns we don't track in memory).
        if (auto existing = m_db->Get(item.downloads_id)) {
            item.created_at = existing->created_at;
            item.mime_type = existing->mime_type;
            item.checksum = existing->checksum;
        } else {
            return; // not persisted (e.g. DB insert failed)
        }
        item.updated_at = NowSeconds();
        m_db->Update(item);
    } catch (const std::exception& e) {
        idr::system::Log()->warn("failed to sync download #{}: {}", dl->GetId(), e.what());
    }
}

} // namespace download
} // namespace idr
