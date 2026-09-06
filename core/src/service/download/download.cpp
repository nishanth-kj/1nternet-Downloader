#include "service/download/download.h"
#include "service/network/http.h"
#include "service/network/request.h"
#include "service/network/response.h"
#include "service/torrent/manager.h"
#include "service/torrent/metainfo.h"

#include <algorithm>
#include <chrono>
#include <filesystem>
#include <fstream>
#include <thread>

namespace idr {
namespace download {

namespace {

// Streams a data callback's chunks into an already-open ofstream, matching the
// idr::network::DataCallback contract.
idr::network::DataCallback MakeFileWriter(std::ofstream& out) {
    return [&out](const char* data, size_t len) -> size_t {
        if (!out.write(data, static_cast<std::streamsize>(len))) return 0;
        return len;
    };
}

} // namespace

Download::Download(const std::string& url, const std::string& destination, int dbId)
    : m_id(dbId), m_url(url), m_destination(destination)
{
    // Check if it's a torrent or magnet link
    if (url.find("magnet:?") == 0 || url.rfind(".torrent") != std::string::npos) {
        m_isTorrent = true;
        if (url.find("magnet:?") == 0) {
            auto meta = idr::torrent::TorrentMetaInfo::ParseMagnetUri(url);
            m_filename = meta.name.empty() ? "magnet_download" : meta.name;
        } else {
            std::filesystem::path p(url);
            m_filename = p.stem().string();
        }
    } else {
        if (!destination.empty()) {
            std::filesystem::path p(destination);
            m_filename = p.filename().string();
        }
        if (m_filename.empty()) {
            size_t pos = url.find_last_of('/');
            if (pos != std::string::npos && pos != url.length() - 1) {
                std::string sub = url.substr(pos + 1);
                size_t qPos = sub.find('?');
                m_filename = (qPos != std::string::npos) ? sub.substr(0, qPos) : sub;
            } else {
                m_filename = "downloaded_file.dat";
            }
        }
    }
}

Download::~Download() {
    Stop();
}

void Download::NotifyStatusChanged() {
    if (m_statusCallback) {
        m_statusCallback(this);
    }
}

std::vector<DownloadSegment> Download::GetSegments() {
    std::lock_guard<std::mutex> lock(m_segmentMutex);
    return m_segments;
}

void Download::Start() {
    if (m_status == DownloadStatus::Downloading) return;

    m_status = DownloadStatus::Downloading;
    m_stopRequested = false;
    NotifyStatusChanged();

    if (m_isTorrent) {
        StartTorrentDownload();
        return;
    }

    // Probe server headers (Accept-Ranges and Content-Length) to decide whether we
    // can accelerate this download with parallel byte-range connections.
    std::thread([this]() {
        idr::network::HttpRequest headReq;
        headReq.url = m_url;

        auto headResp = idr::network::HttpClient::Head(headReq);

        if (headResp.ok && headResp.contentLength > 0) {
            m_totalBytes = static_cast<uint64_t>(headResp.contentLength);
        }

        if (headResp.ok && headResp.acceptRangesBytes && headResp.contentLength > 2 * 1024 * 1024) {
            // Multi-segment acceleration via parallel byte-range connections.
            int numSegments = (headResp.contentLength > 50 * 1024 * 1024) ? 8 : 4;
            StartMultiSegmentDownload(static_cast<uint64_t>(headResp.contentLength), numSegments);
        } else {
            StartSingleDownload();
        }
    }).detach();
}

void Download::StartTorrentDownload() {
    if (!m_torrentTask) {
        // The GUI's save dialog hands us a *file* path (matching the HTTP download
        // convention), but libtorrent needs a *directory* to write into — it lays
        // files out according to the torrent's own metadata.
        std::filesystem::path destPath(m_destination);
        std::string saveDir = destPath.has_parent_path() ? destPath.parent_path().string() : ".";
        if (saveDir.empty()) saveDir = ".";

        m_torrentTask = idr::torrent::TorrentManager::GetInstance().AddTorrent(m_url, saveDir, true);
        if (!m_torrentTask) {
            m_status = DownloadStatus::Error;
            NotifyStatusChanged();
            return;
        }
    } else {
        m_torrentTask->Resume();
    }

    std::thread([this]() {
        while (!m_stopRequested) {
            std::this_thread::sleep_for(std::chrono::milliseconds(500));
            if (m_stopRequested || !m_torrentTask) break;

            auto snap = m_torrentTask->GetSnapshot();

            if (!snap.name.empty()) m_filename = snap.name;
            m_totalBytes = snap.totalBytes;
            m_downloadedBytes = snap.downloadedBytes;
            m_speed = snap.downloadSpeed;
            m_peers = snap.peers;
            m_seeds = snap.seeds;

            using TS = idr::torrent::TorrentStatus;
            if (snap.status == TS::Completed || snap.status == TS::Seeding) {
                m_status = DownloadStatus::Completed;
                m_speed = 0.0;
                NotifyStatusChanged();
                break;
            }
            if (snap.status == TS::Error) {
                m_status = DownloadStatus::Error;
                NotifyStatusChanged();
                break;
            }

            NotifyStatusChanged();
        }
    }).detach();
}

void Download::StartMultiSegmentDownload(uint64_t totalSize, int numSegments) {
    m_totalBytes = totalSize;
    m_segmentCount = numSegments;

    {
        std::lock_guard<std::mutex> lock(m_segmentMutex);
        m_segments.clear();

        uint64_t partSize = totalSize / numSegments;
        for (int i = 0; i < numSegments; ++i) {
            DownloadSegment seg;
            seg.id = i;
            seg.startOffset = i * partSize;
            seg.endOffset = (i == numSegments - 1) ? (totalSize - 1) : ((i + 1) * partSize - 1);
            seg.downloadedBytes = 0;
            seg.speed = 0.0;
            seg.isCompleted = false;
            m_segments.push_back(seg);
        }
    }

    std::vector<std::thread> workers;
    for (int i = 0; i < numSegments; ++i) {
        workers.emplace_back([this, i]() {
            uint64_t segStart, segEnd;
            {
                std::lock_guard<std::mutex> lock(m_segmentMutex);
                segStart = m_segments[i].startOffset;
                segEnd = m_segments[i].endOffset;
            }

            std::string partPath = m_destination + ".part" + std::to_string(i);
            uint64_t existing = 0;
            if (std::filesystem::exists(partPath)) {
                existing = std::filesystem::file_size(partPath);
            }

            if (segStart + existing > segEnd) {
                std::lock_guard<std::mutex> lock(m_segmentMutex);
                m_segments[i].isCompleted = true;
                return;
            }

            std::ofstream outFile(partPath, std::ios::binary | std::ios::app);
            if (!outFile.is_open()) return;

            idr::network::HttpRequest req;
            req.url = m_url;
            req.rangeStart = static_cast<int64_t>(segStart + existing);
            req.rangeEnd = static_cast<int64_t>(segEnd);

            auto writer = MakeFileWriter(outFile);
            auto resp = idr::network::HttpClient::Get(req, writer);
            outFile.close();

            if (resp.ok && !m_stopRequested) {
                std::lock_guard<std::mutex> lock(m_segmentMutex);
                m_segments[i].isCompleted = true;
            }
        });
    }

    // Monitor thread to calculate aggregate progress and stitch parts upon completion
    std::thread([this, workers = std::move(workers), numSegments]() mutable {
        for (auto& w : workers) {
            if (w.joinable()) w.join();
        }

        if (m_stopRequested) return;

        // Check if all parts completed
        bool allDone = true;
        uint64_t totalDl = 0;
        for (int i = 0; i < numSegments; ++i) {
            std::string partPath = m_destination + ".part" + std::to_string(i);
            if (std::filesystem::exists(partPath)) {
                totalDl += std::filesystem::file_size(partPath);
            } else {
                allDone = false;
            }
        }
        m_downloadedBytes = totalDl;

        if (allDone && totalDl >= m_totalBytes) {
            // Assemble .part files into final destination
            std::ofstream finalFile(m_destination, std::ios::binary);
            for (int i = 0; i < numSegments; ++i) {
                std::string partPath = m_destination + ".part" + std::to_string(i);
                std::ifstream partFile(partPath, std::ios::binary);
                finalFile << partFile.rdbuf();
                partFile.close();
                std::filesystem::remove(partPath);
            }
            finalFile.close();

            m_status = DownloadStatus::Completed;
            m_speed = 0.0;
        } else if (!m_stopRequested) {
            m_status = DownloadStatus::Paused;
        }
        NotifyStatusChanged();
    }).detach();
}

void Download::StartSingleDownload() {
    m_segmentCount = 1;

    std::thread([this]() {
        uint64_t existingSize = 0;
        if (std::filesystem::exists(m_destination)) {
            existingSize = std::filesystem::file_size(m_destination);
        }
        m_downloadedBytes = existingSize;

        std::ofstream outFile(m_destination, std::ios::binary | (existingSize > 0 ? std::ios::app : std::ios::trunc));
        if (!outFile.is_open()) {
            m_status = DownloadStatus::Error;
            NotifyStatusChanged();
            return;
        }

        idr::network::HttpRequest req;
        req.url = m_url;
        if (existingSize > 0) {
            req.rangeStart = static_cast<int64_t>(existingSize);
        }

        auto writer = MakeFileWriter(outFile);
        auto resp = idr::network::HttpClient::Get(req, writer);
        outFile.close();

        if (resp.ok && !m_stopRequested) {
            m_status = DownloadStatus::Completed;
            m_speed = 0.0;
        } else if (!m_stopRequested) {
            m_status = DownloadStatus::Error;
        }
        NotifyStatusChanged();
    }).detach();
}

void Download::Pause() {
    m_stopRequested = true;
    m_status = DownloadStatus::Paused;
    m_speed = 0.0;
    if (m_torrentTask) m_torrentTask->Pause();
    NotifyStatusChanged();
}

void Download::Resume() {
    if (m_status == DownloadStatus::Paused || m_status == DownloadStatus::Error || m_status == DownloadStatus::Queued) {
        Start();
    }
}

void Download::Stop() {
    m_stopRequested = true;
    if (m_torrentTask) m_torrentTask->Stop();
    if (m_status == DownloadStatus::Downloading) {
        m_status = DownloadStatus::Paused;
    }
    m_speed = 0.0;
    NotifyStatusChanged();
}

void Download::ReleaseTorrentResources() {
    if (m_torrentTask) {
        idr::torrent::TorrentManager::GetInstance().RemoveTorrent(m_torrentTask->GetId());
        m_torrentTask.reset();
    }
}

} // namespace download
} // namespace idr
