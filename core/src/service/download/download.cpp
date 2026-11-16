#include "service/download/download.h"
#include "service/network/http.h"
#include "service/network/request.h"
#include "service/network/response.h"
#include "service/torrent/manager.h"
#include "service/torrent/metainfo.h"
#include "utils/system/config.h"
#include "utils/system/logger.h"

#include <algorithm>
#include <chrono>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <thread>

namespace idr {
namespace download {

namespace {

constexpr int kMaxSegmentFiles = 32;

bool HasExpectedContentRange(const idr::network::HttpHeaders& headers,
                             uint64_t start, uint64_t end) {
    const std::string header = headers.Get("Content-Range");
    const std::string expectedPrefix =
        "bytes " + std::to_string(start) + "-" + std::to_string(end) + "/";
    return header.rfind(expectedPrefix, 0) == 0;
}

// Status codes that won't change by retrying the same request.
bool IsPermanentHttpFailure(long statusCode) {
    return statusCode >= 400 && statusCode < 500 && statusCode != 408 && statusCode != 429;
}

int ChooseSegmentCount(uint64_t contentLength, int maxSegments) {
    constexpr uint64_t MiB = 1024 * 1024;
    if (maxSegments <= 1 || contentLength < 2 * MiB) return 1;
    if (contentLength < 50 * MiB) return std::min(4, maxSegments);
    return maxSegments;
}

uint64_t FileSizeOrZero(const std::string& path) {
    std::error_code ec;
    const uint64_t size = std::filesystem::file_size(path, ec);
    return ec ? 0 : size;
}

std::chrono::milliseconds Backoff(int attempt) {
    const int seconds = std::min(1 << std::min(attempt, 4), 16);
    return std::chrono::milliseconds(seconds * 1000);
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
    WaitForWorker();
}

void Download::WaitForWorker() {
    std::lock_guard<std::mutex> lock(m_workerMutex);
    if (m_workerThread.joinable() &&
        m_workerThread.get_id() != std::this_thread::get_id()) {
        m_workerThread.join();
    }
}

void Download::NotifyStatusChanged() {
    if (m_statusCallback) {
        m_statusCallback(this);
    }
}

std::string Download::GetErrorMessage() const {
    std::lock_guard<std::mutex> lock(m_errorMutex);
    return m_errorMessage;
}

void Download::SetError(const std::string& message) {
    {
        std::lock_guard<std::mutex> lock(m_errorMutex);
        if (!m_errorMessage.empty()) return; // keep the first (root-cause) error
        m_errorMessage = message;
    }
    idr::system::Log()->error("download #{} ({}): {}", m_id, m_url, message);
}

void Download::Fail(const std::string& message) {
    SetError(message);
    m_status = DownloadStatus::Error;
    m_speed = 0.0;
    NotifyStatusChanged();
}

bool Download::InterruptibleSleep(std::chrono::milliseconds duration) const {
    const auto deadline = std::chrono::steady_clock::now() + duration;
    while (std::chrono::steady_clock::now() < deadline) {
        if (m_stopRequested) return false;
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
    return !m_stopRequested;
}

std::vector<DownloadSegment> Download::GetSegments() {
    std::lock_guard<std::mutex> lock(m_segmentMutex);
    return m_segments;
}

std::string Download::PartPath(int index) const {
    return m_destination + ".part" + std::to_string(index);
}

int Download::CountExistingParts() const {
    // Workers create every .partN file up front, so a contiguous run of part
    // files records the segment layout a previous session used.
    int count = 0;
    std::error_code ec;
    while (count < kMaxSegmentFiles && std::filesystem::exists(PartPath(count), ec)) ++count;
    return count;
}

void Download::RemoveTemporaryFiles() const {
    std::error_code ec;
    for (int i = 0; i < kMaxSegmentFiles; ++i) std::filesystem::remove(PartPath(i), ec);
    std::filesystem::remove(m_destination + ".assembling", ec);
    std::filesystem::remove(m_destination + ".resume", ec); // legacy single-stream temp file
}

void Download::Start() {
    std::lock_guard<std::mutex> workerLock(m_workerMutex);
    if (m_status == DownloadStatus::Downloading) return;

    // A previous worker may have finished but not yet been joined.
    if (m_workerThread.joinable()) {
        if (m_workerThread.get_id() == std::this_thread::get_id()) return;
        m_workerThread.join();
    }

    {
        std::lock_guard<std::mutex> lock(m_errorMutex);
        m_errorMessage.clear();
    }
    m_status = DownloadStatus::Downloading;
    m_stopRequested = false;
    m_rangeRejected = false;
    NotifyStatusChanged();

    if (m_isTorrent) {
        m_workerThread = std::thread([this]() { StartTorrentDownload(); });
        return;
    }

    // The owned coordinator probes the server and joins all segment workers before
    // returning. No detached thread may outlive this Download object.
    m_workerThread = std::thread([this]() { Run(); });
}

void Download::Run() {
    try {
        const auto settings = idr::system::Config::Get().GetSettings();
        idr::system::Log()->info("download #{} starting: {} -> {}", m_id, m_url, m_destination);

        std::error_code dirEc;
        const auto parent = std::filesystem::path(m_destination).parent_path();
        if (!parent.empty()) std::filesystem::create_directories(parent, dirEc);

        idr::network::HttpRequest headReq;
        headReq.url = m_url;
        const auto headResp = idr::network::HttpClient::Head(headReq);

        if (m_stopRequested) return;
        if (headResp.ok && headResp.contentLength > 0) {
            m_totalBytes = static_cast<uint64_t>(headResp.contentLength);
        }

        const bool rangeCapable = headResp.ok && headResp.acceptRangesBytes &&
                                  headResp.contentLength > 0;
        int numSegments = 1;
        if (rangeCapable) {
            const int existingParts = CountExistingParts();
            numSegments = existingParts > 1
                ? existingParts
                : ChooseSegmentCount(static_cast<uint64_t>(headResp.contentLength),
                                     settings.maxSegments);
        }

        if (numSegments > 1) {
            StartMultiSegmentDownload(static_cast<uint64_t>(headResp.contentLength),
                                      numSegments, settings);
        } else {
            StartSingleDownload(settings, rangeCapable);
        }
    } catch (const std::exception& e) {
        if (!m_stopRequested) Fail(std::string("internal error: ") + e.what());
    } catch (...) {
        if (!m_stopRequested) Fail("internal error");
    }
}

void Download::StartTorrentDownload() {
    std::shared_ptr<idr::torrent::TorrentTask> torrentTask;
    {
        std::lock_guard<std::mutex> lock(m_torrentMutex);
        if (!m_torrentTask) {
            // The GUI's save dialog hands us a file path, but libtorrent needs a
            // directory because it lays files out according to torrent metadata.
            std::filesystem::path destPath(m_destination);
            std::string saveDir = destPath.has_parent_path() ? destPath.parent_path().string() : ".";
            if (saveDir.empty()) saveDir = ".";

            m_torrentTask = idr::torrent::TorrentManager::GetInstance().AddTorrent(m_url, saveDir, true);
            if (!m_torrentTask) {
                Fail("could not add torrent");
                return;
            }
        } else {
            m_torrentTask->Resume();
        }
        torrentTask = m_torrentTask;
    }

    while (!m_stopRequested) {
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
        if (m_stopRequested || !torrentTask) break;

        auto snap = torrentTask->GetSnapshot();

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
            Fail("torrent error");
            break;
        }

        NotifyStatusChanged();
    }
}

// ── Segmented (multi-connection) download ──────────────────────────────────

void Download::StartMultiSegmentDownload(uint64_t totalSize, int numSegments,
                                         const idr::system::Settings& settings) {
    m_totalBytes = totalSize;
    m_segmentCount = numSegments;

    uint64_t alreadyOnDisk = 0;
    {
        std::lock_guard<std::mutex> lock(m_segmentMutex);
        m_segments.clear();

        const uint64_t partSize = totalSize / numSegments;
        for (int i = 0; i < numSegments; ++i) {
            DownloadSegment seg;
            seg.id = i;
            seg.startOffset = i * partSize;
            seg.endOffset = (i == numSegments - 1) ? (totalSize - 1) : ((i + 1) * partSize - 1);

            const uint64_t expected = seg.endOffset - seg.startOffset + 1;
            uint64_t existing = FileSizeOrZero(PartPath(i));
            if (existing > expected) {
                // Layout mismatch or corruption: the part can't be trusted.
                std::error_code ec;
                std::filesystem::remove(PartPath(i), ec);
                existing = 0;
            }
            seg.downloadedBytes = existing;
            seg.isCompleted = existing == expected;
            alreadyOnDisk += existing;
            m_segments.push_back(seg);
        }
    }
    m_downloadedBytes = alreadyOnDisk;
    m_speedMeter.Reset(alreadyOnDisk);

    // The speed limit applies per download; split it evenly across connections.
    int64_t perConnectionLimit = 0;
    if (settings.speedLimitKBps > 0) {
        perConnectionLimit = std::max<int64_t>(1024, settings.speedLimitKBps * 1024 / numSegments);
    }

    std::atomic<int> remaining{numSegments};
    std::vector<std::thread> workers;
    workers.reserve(numSegments);
    for (int i = 0; i < numSegments; ++i) {
        workers.emplace_back([this, i, perConnectionLimit, &settings, &remaining]() {
            RunSegment(i, perConnectionLimit, settings.maxRetries);
            --remaining;
        });
    }

    // Sample aggregate progress while the workers run so the GUI sees live speed.
    while (remaining > 0) {
        std::this_thread::sleep_for(std::chrono::milliseconds(250));
        m_speed = m_speedMeter.Update(m_downloadedBytes);
    }

    // Join every segment before examining or assembling its output.
    for (auto& worker : workers) {
        if (worker.joinable()) worker.join();
    }
    m_speed = 0.0;

    if (m_stopRequested) return;

    bool allDone = true;
    {
        std::lock_guard<std::mutex> lock(m_segmentMutex);
        for (const auto& seg : m_segments) allDone = allDone && seg.isCompleted;
    }
    if (!allDone && m_rangeRejected) {
        idr::system::Log()->warn("download #{}: server ignored byte ranges, falling back to a single stream", m_id);
        {
            std::lock_guard<std::mutex> lock(m_errorMutex);
            m_errorMessage.clear();
        }
        StartSingleDownload(settings, false); // also removes the unusable .partN files
        return;
    }
    if (!allDone) {
        Fail(GetErrorMessage().empty() ? "one or more segments failed" : GetErrorMessage());
        return;
    }

    if (AssembleParts(numSegments)) {
        m_downloadedBytes = m_totalBytes.load();
        m_status = DownloadStatus::Completed;
        idr::system::Log()->info("download #{} completed ({} bytes, {} segments)",
                                 m_id, m_totalBytes.load(), numSegments);
        NotifyStatusChanged();
    } else if (!m_stopRequested) {
        Fail(GetErrorMessage().empty() ? "failed to assemble downloaded parts" : GetErrorMessage());
    }
}

void Download::RunSegment(int index, int64_t maxRecvBytesPerSec, int maxRetries) {
    uint64_t segStart = 0, segEnd = 0;
    {
        std::lock_guard<std::mutex> lock(m_segmentMutex);
        if (m_segments[index].isCompleted) return;
        segStart = m_segments[index].startOffset;
        segEnd = m_segments[index].endOffset;
    }
    const uint64_t expectedSize = segEnd - segStart + 1;
    const std::string partPath = PartPath(index);

    // Keeps the per-segment and aggregate counters consistent with what's on disk.
    auto setSegmentBytes = [this, index](uint64_t value) {
        std::lock_guard<std::mutex> lock(m_segmentMutex);
        const uint64_t previous = m_segments[index].downloadedBytes;
        m_segments[index].downloadedBytes = value;
        if (value >= previous) m_downloadedBytes += value - previous;
        else m_downloadedBytes -= previous - value;
    };

    for (int attempt = 0;; ++attempt) {
        if (m_stopRequested) return;

        uint64_t existing = FileSizeOrZero(partPath);
        if (existing > expectedSize) {
            std::error_code ec;
            std::filesystem::remove(partPath, ec);
            existing = 0;
        }
        setSegmentBytes(existing);
        if (existing == expectedSize) {
            std::lock_guard<std::mutex> lock(m_segmentMutex);
            m_segments[index].isCompleted = true;
            return;
        }

        std::ofstream outFile(partPath, std::ios::binary | std::ios::app);
        if (!outFile.is_open()) {
            SetError("cannot write " + partPath);
            return;
        }

        const uint64_t requestedStart = segStart + existing;
        idr::network::HttpRequest req;
        req.url = m_url;
        req.rangeStart = static_cast<int64_t>(requestedStart);
        req.rangeEnd = static_cast<int64_t>(segEnd);
        req.maxRecvBytesPerSec = maxRecvBytesPerSec;

        // Refuse anything but the exact range before a single byte hits the part file.
        auto validator = [requestedStart, segEnd](long code, const idr::network::HttpHeaders& headers) {
            return code == 206 && HasExpectedContentRange(headers, requestedStart, segEnd);
        };

        bool writeFailed = false;
        auto writer = [&](const char* data, size_t len) -> size_t {
            if (m_stopRequested) return 0;
            if (!outFile.write(data, static_cast<std::streamsize>(len))) {
                writeFailed = true;
                return 0;
            }
            {
                std::lock_guard<std::mutex> lock(m_segmentMutex);
                m_segments[index].downloadedBytes += len;
            }
            m_downloadedBytes += len;
            return len;
        };

        const auto resp = idr::network::HttpClient::Get(req, writer, validator);
        outFile.close();

        const uint64_t actualSize = FileSizeOrZero(partPath);
        setSegmentBytes(std::min(actualSize, expectedSize));
        if (resp.ok && actualSize == expectedSize) {
            std::lock_guard<std::mutex> lock(m_segmentMutex);
            m_segments[index].isCompleted = true;
            return;
        }
        if (m_stopRequested) return;

        if (writeFailed) {
            SetError("disk write failed for " + partPath + " (disk full or no permission?)");
            return;
        }
        if (resp.rejectedByValidator && resp.statusCode == 200) {
            m_rangeRejected = true;
            return; // coordinator falls back to a single stream
        }
        if (resp.rejectedByValidator || IsPermanentHttpFailure(resp.statusCode)) {
            SetError("segment " + std::to_string(index) + ": " +
                     (resp.rejectedByValidator ? "server did not honor the byte range (HTTP " +
                                                     std::to_string(resp.statusCode) + ")"
                                               : resp.errorMessage));
            return;
        }
        if (attempt >= maxRetries) {
            SetError("segment " + std::to_string(index) + " failed after " +
                     std::to_string(attempt + 1) + " attempts: " +
                     (resp.errorMessage.empty() ? "incomplete transfer" : resp.errorMessage));
            return;
        }

        idr::system::Log()->warn("download #{} segment {} attempt {} failed ({}), retrying",
                                 m_id, index, attempt + 1, resp.errorMessage);
        if (!InterruptibleSleep(Backoff(attempt))) return;
    }
}

bool Download::AssembleParts(int numSegments) {
    const std::string assembledPath = m_destination + ".assembling";
    std::ofstream finalFile(assembledPath, std::ios::binary | std::ios::trunc);
    bool assembled = finalFile.is_open();
    for (int i = 0; assembled && i < numSegments; ++i) {
        std::ifstream partFile(PartPath(i), std::ios::binary);
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
    if (!assembled || std::filesystem::file_size(assembledPath, sizeEc) != m_totalBytes || sizeEc) {
        SetError("failed to assemble parts into " + m_destination + " (disk full?)");
        std::error_code cleanupEc;
        std::filesystem::remove(assembledPath, cleanupEc);
        return false;
    }

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
        SetError("cannot replace existing file " + m_destination);
        return false;
    }

    std::error_code renameEc;
    std::filesystem::rename(assembledPath, m_destination, renameEc);
    if (renameEc) {
        if (hadDestination) {
            std::error_code restoreEc;
            std::filesystem::rename(backupPath, m_destination, restoreEc);
        }
        SetError("cannot move assembled file into place: " + renameEc.message());
        return false;
    }
    if (hadDestination) {
        std::error_code cleanupEc;
        std::filesystem::remove(backupPath, cleanupEc);
    }

    for (int i = 0; i < numSegments; ++i) {
        std::error_code removeEc;
        std::filesystem::remove(PartPath(i), removeEc);
    }
    return true;
}

// ── Single-stream download ────────────────────────────────────────────────

void Download::StartSingleDownload(const idr::system::Settings& settings, bool serverAcceptsRanges) {
    m_segmentCount = 1;
    {
        std::lock_guard<std::mutex> lock(m_segmentMutex);
        m_segments.clear();
    }
    // Leftover .partN files from a segmented attempt can't be reused by a single stream.
    if (CountExistingParts() > 0) {
        std::error_code ec;
        for (int i = 0; i < kMaxSegmentFiles; ++i) std::filesystem::remove(PartPath(i), ec);
    }

    bool allowResume = serverAcceptsRanges;
    for (int attempt = 0;; ++attempt) {
        const AttemptResult result = AttemptSingleDownload(settings, allowResume);
        m_speed = 0.0;
        switch (result) {
        case AttemptResult::Completed:
            m_status = DownloadStatus::Completed;
            idr::system::Log()->info("download #{} completed ({} bytes)", m_id,
                                     m_downloadedBytes.load());
            NotifyStatusChanged();
            return;
        case AttemptResult::Stopped:
            return; // Pause()/Stop() already set the status
        case AttemptResult::Fatal:
            Fail(GetErrorMessage().empty() ? "download failed" : GetErrorMessage());
            return;
        case AttemptResult::Retryable:
            break;
        }

        if (attempt >= settings.maxRetries) {
            Fail("failed after " + std::to_string(attempt + 1) + " attempts: " +
                 (GetErrorMessage().empty() ? "incomplete transfer" : GetErrorMessage()));
            return;
        }
        {
            // A retryable failure isn't the final error; let the next attempt report its own.
            std::lock_guard<std::mutex> lock(m_errorMutex);
            idr::system::Log()->warn("download #{} attempt {} failed ({}), retrying",
                                     m_id, attempt + 1, m_errorMessage);
            m_errorMessage.clear();
        }
        if (!InterruptibleSleep(Backoff(attempt))) return;
    }
}

Download::AttemptResult Download::AttemptSingleDownload(const idr::system::Settings& settings,
                                                         bool& allowResume) {
    uint64_t existingSize = allowResume ? FileSizeOrZero(m_destination) : 0;
    const uint64_t knownTotal = m_totalBytes;
    if (knownTotal > 0 && existingSize == knownTotal) {
        m_downloadedBytes = existingSize;
        return AttemptResult::Completed;
    }
    if (knownTotal > 0 && existingSize > knownTotal) existingSize = 0;
    const bool resuming = existingSize > 0;

    m_downloadedBytes = existingSize;
    m_speedMeter.Reset(existingSize);

    idr::network::HttpRequest req;
    req.url = m_url;
    if (resuming) req.rangeStart = static_cast<int64_t>(existingSize);
    if (settings.speedLimitKBps > 0) req.maxRecvBytesPerSec = settings.speedLimitKBps * 1024;

    // Only a 206 continuing at exactly our offset may be appended; a fresh download
    // accepts any 2xx. Nothing is written (and the file isn't truncated) otherwise.
    bool rangeIgnored = false;
    auto validator = [&](long code, const idr::network::HttpHeaders& headers) {
        if (resuming) {
            if (code == 206 &&
                headers.Get("Content-Range").rfind("bytes " + std::to_string(existingSize) + "-", 0) == 0) {
                return true;
            }
            if (code == 200) rangeIgnored = true;
            return false;
        }
        if (code < 200 || code >= 300) return false;
        if (m_totalBytes == 0) {
            const std::string len = headers.Get("Content-Length");
            if (!len.empty()) m_totalBytes = std::strtoull(len.c_str(), nullptr, 10);
        }
        return true;
    };

    std::ofstream outFile;
    bool writeFailed = false;
    uint64_t streamedBytes = 0;
    auto writer = [&](const char* data, size_t len) -> size_t {
        if (m_stopRequested) return 0;
        if (!outFile.is_open()) {
            outFile.open(m_destination, std::ios::binary | (resuming ? std::ios::app : std::ios::trunc));
            if (!outFile.is_open()) {
                writeFailed = true;
                return 0;
            }
        }
        if (!outFile.write(data, static_cast<std::streamsize>(len))) {
            writeFailed = true;
            return 0;
        }
        streamedBytes += len;
        m_downloadedBytes = existingSize + streamedBytes;
        m_speed = m_speedMeter.Update(m_downloadedBytes);
        return len;
    };

    const auto resp = idr::network::HttpClient::Get(req, writer, validator);
    if (outFile.is_open()) outFile.close();
    if (outFile.fail() && streamedBytes > 0) writeFailed = true;

    if (m_stopRequested) return AttemptResult::Stopped;

    if (writeFailed) {
        SetError("disk write failed for " + m_destination + " (disk full or no permission?)");
        return AttemptResult::Fatal;
    }
    if (rangeIgnored) {
        // The server restarted from byte 0 instead of resuming: start over cleanly.
        allowResume = false;
        SetError("server ignored the resume request; restarting from the beginning");
        return AttemptResult::Retryable;
    }
    if (!resp.ok) {
        SetError(resp.errorMessage.empty() ? "transfer failed" : resp.errorMessage);
        return IsPermanentHttpFailure(resp.statusCode) ? AttemptResult::Fatal
                                                        : AttemptResult::Retryable;
    }

    const uint64_t finalSize = FileSizeOrZero(m_destination);
    if (resp.ok && streamedBytes == 0 && !resuming) {
        // Zero-byte body: make sure an empty destination file exists.
        std::ofstream touch(m_destination, std::ios::binary | std::ios::trunc);
    }
    if (m_totalBytes > 0 && finalSize != m_totalBytes) {
        SetError("size mismatch: expected " + std::to_string(m_totalBytes.load()) +
                 " bytes, got " + std::to_string(finalSize));
        // Keep what we have; a range-capable server lets the retry continue from here.
        return AttemptResult::Retryable;
    }
    m_downloadedBytes = finalSize;
    if (m_totalBytes == 0) m_totalBytes = finalSize;
    return AttemptResult::Completed;
}

// ── Lifecycle controls ────────────────────────────────────────────────────

void Download::Pause() {
    if (m_status == DownloadStatus::Completed) return;
    m_stopRequested = true;
    m_status = DownloadStatus::Paused;
    m_speed = 0.0;
    std::shared_ptr<idr::torrent::TorrentTask> torrentTask;
    {
        std::lock_guard<std::mutex> lock(m_torrentMutex);
        torrentTask = m_torrentTask;
    }
    if (torrentTask) torrentTask->Pause();
    NotifyStatusChanged();
}

void Download::Resume() {
    if (m_status == DownloadStatus::Paused || m_status == DownloadStatus::Error || m_status == DownloadStatus::Queued) {
        Start();
    }
}

void Download::Enqueue() {
    const auto st = m_status.load();
    if (st == DownloadStatus::Paused || st == DownloadStatus::Error) {
        m_status = DownloadStatus::Queued;
        NotifyStatusChanged();
    }
}

void Download::Stop() {
    m_stopRequested = true;
    std::shared_ptr<idr::torrent::TorrentTask> torrentTask;
    {
        std::lock_guard<std::mutex> lock(m_torrentMutex);
        torrentTask = m_torrentTask;
    }
    if (torrentTask) torrentTask->Stop();
    if (m_status == DownloadStatus::Downloading) {
        m_status = DownloadStatus::Paused;
    }
    m_speed = 0.0;
    NotifyStatusChanged();
}

void Download::ReleaseTorrentResources() {
    std::shared_ptr<idr::torrent::TorrentTask> torrentTask;
    {
        std::lock_guard<std::mutex> lock(m_torrentMutex);
        torrentTask = std::move(m_torrentTask);
    }
    if (torrentTask) {
        idr::torrent::TorrentManager::GetInstance().RemoveTorrent(torrentTask->GetId());
    }
}

} // namespace download
} // namespace idr
