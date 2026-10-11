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
idr::network::DataCallback MakeFileWriter(
    std::ofstream& out, const std::function<void(size_t)>& onChunk = {}) {
    return [&out, onChunk](const char* data, size_t len) -> size_t {
        if (!out.write(data, static_cast<std::streamsize>(len))) return 0;
        if (onChunk) onChunk(len);
        return len;
    };
}

bool HasExpectedContentRange(const idr::network::HttpResponse& response,
                             uint64_t start, uint64_t end) {
    const std::string header = response.headers.Get("Content-Range");
    const std::string expectedPrefix =
        "bytes " + std::to_string(start) + "-" + std::to_string(end) + "/";
    return header.rfind(expectedPrefix, 0) == 0;
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

            // Avoid mutating m_filename here: it's read from the UI thread and is not synchronized.
            // (If live renaming is needed, guard m_filename with a mutex shared by GetFilename().)
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

            const uint64_t expectedSize = segEnd - segStart + 1;
            if (existing > expectedSize) {
                std::error_code ec;
                std::filesystem::remove(partPath, ec);
                existing = 0;
            }
            if (existing == expectedSize) {
                std::lock_guard<std::mutex> lock(m_segmentMutex);
                m_segments[i].isCompleted = true;
                return;
            }

            std::ofstream outFile(partPath, std::ios::binary | std::ios::app);
            if (!outFile.is_open()) return;

            idr::network::HttpRequest req;
            req.url = m_url;
            const uint64_t requestedStart = segStart + existing;
            req.rangeStart = static_cast<int64_t>(requestedStart);
            req.rangeEnd = static_cast<int64_t>(segEnd);

            auto writer = MakeFileWriter(outFile);
            auto resp = idr::network::HttpClient::Get(req, writer);
            outFile.close();

            std::error_code sizeEc;
            const uint64_t actualSize = std::filesystem::file_size(partPath, sizeEc);
            const bool validRange = resp.statusCode == 206 &&
                HasExpectedContentRange(resp, requestedStart, segEnd);
            const bool validSize = !sizeEc && actualSize == expectedSize;

            std::lock_guard<std::mutex> lock(m_segmentMutex);
            m_segments[i].isCompleted = resp.ok && validRange && validSize && !m_stopRequested;
        });
    }

    // Monitor thread to calculate aggregate progress and stitch parts upon completion
    std::thread([this, workers = std::move(workers), numSegments]() mutable {
        for (auto& w : workers) {
            if (w.joinable()) w.join();
        }

        if (m_stopRequested) return;

        // A part file is not proof of success: validate each exact range.
        bool allDone = true;
        uint64_t totalDl = 0;
        for (int i = 0; i < numSegments; ++i) {
            const std::string partPath = m_destination + ".part" + std::to_string(i);
            uint64_t expectedSize = 0;
            {
                std::lock_guard<std::mutex> lock(m_segmentMutex);
                expectedSize = m_segments[i].endOffset - m_segments[i].startOffset + 1;
                allDone = allDone && m_segments[i].isCompleted;
            }
            std::error_code sizeEc;
            const uint64_t actualSize = std::filesystem::file_size(partPath, sizeEc);
            if (sizeEc || actualSize != expectedSize) {
                allDone = false;
            } else {
                totalDl += actualSize;
            }
        }
        m_downloadedBytes = totalDl;

        if (allDone && totalDl == m_totalBytes) {
            const std::string assembledPath = m_destination + ".assembling";
            std::ofstream finalFile(assembledPath, std::ios::binary | std::ios::trunc);
            bool assembled = finalFile.is_open();
            for (int i = 0; assembled && i < numSegments; ++i) {
                const std::string partPath = m_destination + ".part" + std::to_string(i);
                std::ifstream partFile(partPath, std::ios::binary);
                if (!partFile.is_open()) {
                    assembled = false;
                    break;
                }
                finalFile << partFile.rdbuf();
                assembled = finalFile.good();
            }
            finalFile.close();
            assembled = assembled && finalFile.good();

            std::error_code sizeEc;
            if (assembled && std::filesystem::file_size(assembledPath, sizeEc) == m_totalBytes && !sizeEc) {
                // Windows does not replace an existing destination with rename().
                // Keep a backup until the verified temporary file is in place.
                const std::string backupPath = m_destination + ".previous";
                std::error_code existsEc;
                const bool hadDestination = std::filesystem::exists(m_destination, existsEc);
                std::error_code backupEc;
                if (hadDestination && !existsEc) {
                    std::filesystem::remove(backupPath, backupEc);
                    backupEc.clear();
                    std::filesystem::rename(m_destination, backupPath, backupEc);
                }

                if (existsEc || backupEc) {
                    assembled = false;
                } else {
                    std::error_code renameEc;
                    std::filesystem::rename(assembledPath, m_destination, renameEc);
                    assembled = !renameEc;
                    if (!assembled && hadDestination) {
                        std::error_code restoreEc;
                        std::filesystem::rename(backupPath, m_destination, restoreEc);
                    } else if (assembled && hadDestination) {
                        std::error_code cleanupEc;
                        std::filesystem::remove(backupPath, cleanupEc);
                    }
                }
            } else {
                assembled = false;
            }

            if (assembled) {
                for (int i = 0; i < numSegments; ++i) {
                    std::error_code removeEc;
                    std::filesystem::remove(m_destination + ".part" + std::to_string(i), removeEc);
                }
                m_status = DownloadStatus::Completed;
                m_speed = 0.0;
            } else if (!m_stopRequested) {
                m_status = DownloadStatus::Error;
                std::error_code cleanupEc;
                std::filesystem::remove(assembledPath, cleanupEc);
            }
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

        bool canResume = false;
        if (existingSize > 0) {
            idr::network::HttpRequest headReq;
            headReq.url = m_url;
            auto headResp = idr::network::HttpClient::Head(headReq);
            canResume = headResp.ok && headResp.acceptRangesBytes;
            if (!canResume) existingSize = 0;
        }

        m_downloadedBytes = existingSize;
        const std::string streamPath = canResume ? m_destination + ".resume" : m_destination;
        std::ofstream outFile(streamPath, std::ios::binary | std::ios::trunc);
        if (!outFile.is_open()) {
            m_status = DownloadStatus::Error;
            NotifyStatusChanged();
            return;
        }

        idr::network::HttpRequest req;
        req.url = m_url;
        if (canResume) {
            req.rangeStart = static_cast<int64_t>(existingSize);
        }

        const auto startedAt = std::chrono::steady_clock::now();
        uint64_t streamedBytes = 0;
        auto writer = MakeFileWriter(outFile, [this, &streamedBytes, existingSize, startedAt](size_t len) {
            streamedBytes += len;
            m_downloadedBytes = existingSize + streamedBytes;
            const auto elapsed = std::chrono::duration<double>(
                std::chrono::steady_clock::now() - startedAt).count();
            if (elapsed > 0.0) m_speed = static_cast<double>(streamedBytes) / elapsed;
        });
        auto resp = idr::network::HttpClient::Get(req, writer);
        outFile.close();

        bool completed = resp.ok;
        if (canResume && resp.ok) {
            if (resp.statusCode == 206 &&
                resp.headers.Get("Content-Range").rfind(
                    "bytes " + std::to_string(existingSize) + "-", 0) == 0) {
                std::ofstream destination(m_destination, std::ios::binary | std::ios::app);
                std::ifstream resume(streamPath, std::ios::binary);
                destination << resume.rdbuf();
                completed = destination.good();
            } else if (resp.statusCode == 200) {
                std::error_code ec;
                std::filesystem::remove(m_destination, ec);
                std::filesystem::rename(streamPath, m_destination, ec);
                completed = !ec;
                if (completed) m_downloadedBytes = streamedBytes;
            } else {
                completed = false;
            }
        }
        if (canResume) std::filesystem::remove(streamPath);

        if (completed && m_totalBytes > 0) {
            std::error_code sizeEc;
            const uint64_t finalSize = std::filesystem::file_size(m_destination, sizeEc);
            if (sizeEc || finalSize != m_totalBytes) completed = false;
        }

        if (completed && !m_stopRequested) {
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
