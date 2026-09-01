#include "download/download.h"
#include <curl/curl.h>
#include <thread>
#include <fstream>
#include <iostream>

namespace idr {
namespace download {

static size_t CurlWriteCallback(void* contents, size_t size, size_t nmemb, void* userp) {
    return static_cast<Download*>(userp)->WriteCallback(contents, size, nmemb);
}

static int CurlXferInfoCallback(void* clientp, curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow) {
    return static_cast<Download*>(clientp)->ProgressCallback(dltotal, dlnow, ultotal, ulnow);
}

Download::Download(const std::string& url, const std::string& destination)
    : m_url(url), m_destination(destination) 
{
    // Extract filename from URL (simple fallback logic)
    size_t pos = url.find_last_of('/');
    if (pos != std::string::npos && pos != url.length() - 1) {
        m_filename = url.substr(pos + 1);
    } else {
        m_filename = "downloaded_file.dat";
    }
}

Download::~Download() {
    Stop();
}

void Download::Start() {
    if (m_status == DownloadStatus::Downloading) return;
    
    m_status = DownloadStatus::Downloading;
    m_stopRequested = false;
    m_lastSpeedUpdate = std::chrono::steady_clock::now();
    m_bytesAtLastUpdate = 0;

    // Launch background thread
    std::thread([this]() {
        CURL* curl = curl_easy_init();
        if (!curl) {
            m_status = DownloadStatus::Error;
            return;
        }

        std::ofstream outfile(m_destination, std::ios::binary | std::ios::app);
        if (!outfile.is_open()) {
            m_status = DownloadStatus::Error;
            curl_easy_cleanup(curl);
            return;
        }

        curl_easy_setopt(curl, CURLOPT_URL, m_url.c_str());
        curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
        curl_easy_setopt(curl, CURLOPT_NOPROGRESS, 0L);
        
        // Setup Callbacks
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, CurlWriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, this);
        curl_easy_setopt(curl, CURLOPT_XFERINFOFUNCTION, CurlXferInfoCallback);
        curl_easy_setopt(curl, CURLOPT_XFERINFODATA, this);
        
        // Pass the ofstream pointer so WriteCallback can use it
        // Wait, WriteCallback is a member function, it can just use a member ofstream... 
        // But for simplicity, we can pass it via thread local or class member.
        // Actually, let's open and close it inside the callback or store it in class?
        // Let's store the `ofstream` pointer in `this` temporarily or just use a lambda!
        // For C-compatibility, we can't capture easily. 
        // We'll reopen in append mode in the callback, or store a `FILE*` in the class.
    }).detach();
}

void Download::Pause() {
    m_status = DownloadStatus::Paused;
    m_stopRequested = true;
}

void Download::Resume() {
    if (m_status == DownloadStatus::Paused) {
        Start(); // Re-start and append
    }
}

void Download::Stop() {
    m_stopRequested = true;
    m_status = DownloadStatus::Completed; // Or Canceled
}

size_t Download::WriteCallback(void* contents, size_t size, size_t nmemb) {
    if (m_stopRequested) return 0; // Abort curl

    size_t totalSize = size * nmemb;
    
    // Simplistic file writing (ideally keep the file open across chunks)
    std::ofstream outfile(m_destination, std::ios::binary | std::ios::app);
    if (outfile.write(static_cast<char*>(contents), totalSize)) {
        m_downloadedBytes += totalSize;
        return totalSize;
    }
    
    return 0; // Write error
}

int Download::ProgressCallback(curl_off_t dltotal, curl_off_t dlnow, curl_off_t ultotal, curl_off_t ulnow) {
    if (m_stopRequested) return 1; // Abort curl

    m_totalBytes = dltotal;
    
    // Speed calculation
    auto now = std::chrono::steady_clock::now();
    std::chrono::duration<double> elapsed = now - m_lastSpeedUpdate;
    
    if (elapsed.count() >= 1.0) {
        uint64_t bytesSinceLast = dlnow - m_bytesAtLastUpdate;
        m_speed = bytesSinceLast / elapsed.count();
        
        m_lastSpeedUpdate = now;
        m_bytesAtLastUpdate = dlnow;
    }

    if (dltotal > 0 && dlnow == dltotal) {
        m_status = DownloadStatus::Completed;
        m_speed = 0;
    }

    return 0;
}

} // namespace download
} // namespace idr
