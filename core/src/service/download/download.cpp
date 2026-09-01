#include "service/download/download.h"
#include <curl/curl.h>
#include <thread>
#include <fstream>
#include <iostream>
#include <filesystem>

namespace idr {
namespace download {

static size_t CurlWriteCallback(void* contents, size_t size, size_t nmemb, void* userp) {
    return static_cast<Download*>(userp)->WriteCallback(contents, size, nmemb);
}

static int CurlXferInfoCallback(void* clientp, curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow) {
    return static_cast<Download*>(clientp)->ProgressCallback(dltotal, dlnow, ultotal, ulnow);
}

Download::Download(const std::string& url, const std::string& destination, int dbId)
    : m_id(dbId), m_url(url), m_destination(destination)
{
    // Extract filename from destination or URL
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

Download::~Download() {
    Stop();
}

void Download::NotifyStatusChanged() {
    if (m_statusCallback) {
        m_statusCallback(this);
    }
}

void Download::Start() {
    if (m_status == DownloadStatus::Downloading) return;

    m_status = DownloadStatus::Downloading;
    m_stopRequested = false;
    m_lastSpeedUpdate = std::chrono::steady_clock::now();
    m_bytesAtLastUpdate = 0;
    NotifyStatusChanged();

    std::thread([this]() {
        CURL* curl = curl_easy_init();
        if (!curl) {
            m_status = DownloadStatus::Error;
            NotifyStatusChanged();
            return;
        }

        // Determine existing file size for resume support
        uint64_t existingSize = 0;
        if (std::filesystem::exists(m_destination)) {
            existingSize = std::filesystem::file_size(m_destination);
        }
        m_startOffset = existingSize;
        m_downloadedBytes = existingSize;

        curl_easy_setopt(curl, CURLOPT_URL, m_url.c_str());
        curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
        curl_easy_setopt(curl, CURLOPT_NOPROGRESS, 0L);
        curl_easy_setopt(curl, CURLOPT_FAILONERROR, 1L);
        curl_easy_setopt(curl, CURLOPT_USERAGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) InternetDownloader/1.0");

        if (existingSize > 0) {
            curl_easy_setopt(curl, CURLOPT_RESUME_FROM_LARGE, static_cast<curl_off_t>(existingSize));
        }

        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, CurlWriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, this);
        curl_easy_setopt(curl, CURLOPT_XFERINFOFUNCTION, CurlXferInfoCallback);
        curl_easy_setopt(curl, CURLOPT_XFERINFODATA, this);

        CURLcode res = curl_easy_perform(curl);

        if (res == CURLE_OK) {
            if (!m_stopRequested) {
                m_status = DownloadStatus::Completed;
                m_speed = 0.0;
            }
        } else if (res == CURLE_ABORTED_BY_CALLBACK) {
            // User requested pause or stop
            if (m_status != DownloadStatus::Paused) {
                m_status = DownloadStatus::Paused;
            }
            m_speed = 0.0;
        } else {
            if (!m_stopRequested) {
                m_status = DownloadStatus::Error;
                m_speed = 0.0;
            }
        }

        curl_easy_cleanup(curl);
        NotifyStatusChanged();
    }).detach();
}

void Download::Pause() {
    m_stopRequested = true;
    m_status = DownloadStatus::Paused;
    m_speed = 0.0;
    NotifyStatusChanged();
}

void Download::Resume() {
    if (m_status == DownloadStatus::Paused || m_status == DownloadStatus::Error || m_status == DownloadStatus::Queued) {
        Start();
    }
}

void Download::Stop() {
    m_stopRequested = true;
    if (m_status == DownloadStatus::Downloading) {
        m_status = DownloadStatus::Paused;
    }
    m_speed = 0.0;
    NotifyStatusChanged();
}

size_t Download::WriteCallback(void* contents, size_t size, size_t nmemb) {
    if (m_stopRequested) return 0; // Abort curl

    size_t totalSize = size * nmemb;

    std::ofstream outfile(m_destination, std::ios::binary | std::ios::app);
    if (outfile.is_open() && outfile.write(static_cast<char*>(contents), totalSize)) {
        m_downloadedBytes += totalSize;
        return totalSize;
    }

    return 0; // Error writing
}

int Download::ProgressCallback(curl_off_t dltotal, curl_off_t dlnow, curl_off_t /*ultotal*/, curl_off_t /*ulnow*/) {
    if (m_stopRequested) return 1; // Abort curl

    if (dltotal > 0) {
        m_totalBytes = m_startOffset + dltotal;
    }

    auto now = std::chrono::steady_clock::now();
    std::chrono::duration<double> elapsed = now - m_lastSpeedUpdate;

    if (elapsed.count() >= 0.5) {
        uint64_t currentDl = m_downloadedBytes.load();
        if (currentDl >= m_bytesAtLastUpdate) {
            uint64_t delta = currentDl - m_bytesAtLastUpdate;
            m_speed = delta / elapsed.count();
        }
        m_lastSpeedUpdate = now;
        m_bytesAtLastUpdate = currentDl;
    }

    return 0;
}

} // namespace download
} // namespace idr
