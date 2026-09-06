#include <jni.h>
#include <string>
#include <vector>
#include <memory>
#include <mutex>
#include <atomic>
#include <thread>
#include <fstream>
#include <sstream>
#include <chrono>
#include <filesystem>
#include <algorithm>
#include <curl/curl.h>
#include <android/log.h>

#define LOG_TAG "IDR_Native"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

namespace idr_android {

namespace net {

// Minimal real libcurl helpers used by NativeDownloadTask. This mirrors the
// desktop engine's approach (HEAD probe -> parallel byte-range GETs -> reassemble)
// but stays in one file since Android doesn't share the desktop's network/ layer.

constexpr const char* kUserAgent = "Mozilla/5.0 (Linux; Android) InternetDownloader/1.0";

struct ProbeResult {
    bool ok{false};
    int64_t contentLength{-1};
    bool acceptRanges{false};
};

size_t DiscardWriteCallback(char*, size_t size, size_t nmemb, void*) {
    return size * nmemb; // HEAD probe: ignore body
}

size_t HeaderCallback(char* buffer, size_t size, size_t nitems, void* userdata) {
    auto* out = static_cast<ProbeResult*>(userdata);
    size_t total = size * nitems;
    std::string line(buffer, total);
    if (line.find("Accept-Ranges") != std::string::npos && line.find("bytes") != std::string::npos) {
        out->acceptRanges = true;
    }
    return total;
}

ProbeResult Probe(const std::string& url) {
    ProbeResult result;
    CURL* curl = curl_easy_init();
    if (!curl) return result;

    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_NOBODY, 1L);
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
    curl_easy_setopt(curl, CURLOPT_USERAGENT, kUserAgent);
    curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION, HeaderCallback);
    curl_easy_setopt(curl, CURLOPT_HEADERDATA, &result);

    CURLcode res = curl_easy_perform(curl);
    if (res == CURLE_OK) {
        curl_off_t cl = -1;
        curl_easy_getinfo(curl, CURLINFO_CONTENT_LENGTH_DOWNLOAD_T, &cl);
        result.contentLength = static_cast<int64_t>(cl);
        result.ok = true;
    }

    curl_easy_cleanup(curl);
    return result;
}

struct RangeWriteContext {
    std::ofstream* out{nullptr};
    std::atomic<uint64_t>* downloadedCounter{nullptr};
    std::atomic<bool>* stopRequested{nullptr};
};

size_t RangeWriteCallback(char* contents, size_t size, size_t nmemb, void* userp) {
    auto* ctx = static_cast<RangeWriteContext*>(userp);
    if (!ctx || !ctx->out) return 0;
    if (ctx->stopRequested && ctx->stopRequested->load()) return 0; // aborts the transfer

    size_t total = size * nmemb;
    if (!ctx->out->write(contents, static_cast<std::streamsize>(total))) return 0;
    if (ctx->downloadedCounter) *ctx->downloadedCounter += total;
    return total;
}

// Downloads [start, end] (inclusive; end < 0 means "to end of file") into partPath.
// Returns true on success.
bool DownloadRange(const std::string& url, const std::string& partPath, int64_t start, int64_t end,
                    std::atomic<uint64_t>& downloadedCounter, std::atomic<bool>& stopRequested) {
    std::ofstream out(partPath, std::ios::binary | std::ios::app);
    if (!out.is_open()) return false;

    CURL* curl = curl_easy_init();
    if (!curl) return false;

    std::string range = end >= 0 ? (std::to_string(start) + "-" + std::to_string(end))
                                  : (std::to_string(start) + "-");

    RangeWriteContext ctx{&out, &downloadedCounter, &stopRequested};

    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_RANGE, range.c_str());
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
    curl_easy_setopt(curl, CURLOPT_USERAGENT, kUserAgent);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, RangeWriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &ctx);

    CURLcode res = curl_easy_perform(curl);
    curl_easy_cleanup(curl);

    return res == CURLE_OK;
}

} // namespace net

enum class Status {
    Queued = 0,
    Downloading = 1,
    Paused = 2,
    Completed = 3,
    Error = 4
};

struct NativeDownloadTask {
    int id{0};
    std::string url;
    std::string destination;
    std::string filename;
    bool isTorrent{false};

    std::atomic<uint64_t> totalBytes{0};
    std::atomic<uint64_t> downloadedBytes{0};
    std::atomic<double> speed{0.0};
    std::atomic<Status> status{Status::Queued};
    std::atomic<int> numThreads{4};
    std::atomic<bool> stopRequested{false};

    std::thread workerThread;

    NativeDownloadTask(int taskId, const std::string& u, const std::string& dest, int threads)
        : id(taskId), url(u), destination(dest), numThreads(threads) {
        if (url.find("magnet:?") == 0 || url.rfind(".torrent") != std::string::npos) {
            isTorrent = true;
            filename = "torrent_download_" + std::to_string(id);
        } else {
            size_t pos = url.find_last_of('/');
            if (pos != std::string::npos && pos + 1 < url.length()) {
                std::string sub = url.substr(pos + 1);
                size_t q = sub.find('?');
                filename = (q != std::string::npos) ? sub.substr(0, q) : sub;
            } else {
                filename = "download_" + std::to_string(id) + ".bin";
            }
        }
    }

    ~NativeDownloadTask() {
        Stop();
        if (workerThread.joinable()) workerThread.join();
    }

    void Start() {
        if (status == Status::Downloading) return;
        status = Status::Downloading;
        stopRequested = false;

        if (workerThread.joinable()) {
            workerThread.join();
        }

        workerThread = std::thread(&NativeDownloadTask::ExecuteDownload, this);
    }

    void Pause() {
        stopRequested = true;
        status = Status::Paused;
        speed = 0.0;
    }

    void Resume() {
        if (status == Status::Paused || status == Status::Queued || status == Status::Error) {
            Start();
        }
    }

    void Stop() {
        stopRequested = true;
        if (status == Status::Downloading) {
            status = Status::Paused;
        }
        speed = 0.0;
    }

    void ExecuteDownload() {
        LOGI("Starting native download task #%d: %s (Threads: %d)", id, url.c_str(), numThreads.load());

        if (isTorrent) {
            // BitTorrent is not yet implemented in the Android build (it would require
            // cross-compiling libtorrent + boost for Android, which the desktop build
            // uses but this native module does not yet link). Report an error instead
            // of faking progress.
            LOGE("Torrent/magnet downloads are not yet supported on Android: %s", url.c_str());
            status = Status::Error;
            return;
        }

        auto probe = net::Probe(url);
        if (probe.ok && probe.contentLength > 0) {
            totalBytes = static_cast<uint64_t>(probe.contentLength);
        }

        int threads = std::max(1, numThreads.load());
        bool canSegment = probe.ok && probe.acceptRanges && probe.contentLength > 2 * 1024 * 1024 && threads > 1;

        bool success = false;

        if (canSegment) {
            uint64_t total = static_cast<uint64_t>(probe.contentLength);
            uint64_t partSize = total / static_cast<uint64_t>(threads);

            std::vector<std::thread> workers;
            std::vector<bool> partOk(threads, false);

            for (int i = 0; i < threads; ++i) {
                int64_t start = static_cast<int64_t>(i * partSize);
                int64_t end = (i == threads - 1) ? static_cast<int64_t>(total - 1)
                                                  : static_cast<int64_t>((i + 1) * partSize - 1);
                std::string partPath = destination + ".part" + std::to_string(i);

                workers.emplace_back([this, i, start, end, partPath, &partOk]() {
                    partOk[i] = net::DownloadRange(url, partPath, start, end, downloadedBytes, stopRequested);
                });
            }

            for (auto& w : workers) {
                if (w.joinable()) w.join();
            }

            bool allOk = std::all_of(partOk.begin(), partOk.end(), [](bool ok) { return ok; });

            if (allOk && !stopRequested) {
                std::ofstream finalFile(destination, std::ios::binary | std::ios::trunc);
                bool assembled = finalFile.is_open();
                for (int i = 0; assembled && i < threads; ++i) {
                    std::string partPath = destination + ".part" + std::to_string(i);
                    std::ifstream partFile(partPath, std::ios::binary);
                    if (!partFile.is_open()) {
                        assembled = false;
                        break;
                    }
                    finalFile << partFile.rdbuf();
                }
                finalFile.close();

                for (int i = 0; i < threads; ++i) {
                    std::error_code ec;
                    std::filesystem::remove(destination + ".part" + std::to_string(i), ec);
                }

                success = assembled;
            }
        } else {
            success = net::DownloadRange(url, destination, 0, -1, downloadedBytes, stopRequested);
        }

        if (stopRequested) {
            return; // paused/cancelled mid-flight; state already reflects that
        }

        speed = 0.0;
        if (success) {
            std::error_code ec;
            if (totalBytes == 0 && std::filesystem::exists(destination, ec)) {
                totalBytes = static_cast<uint64_t>(std::filesystem::file_size(destination, ec));
            }
            status = Status::Completed;
            LOGI("Native download task #%d completed successfully", id);
        } else {
            status = Status::Error;
            LOGE("Native download task #%d failed", id);
        }
    }
};

class NativeDownloadEngine {
public:
    static NativeDownloadEngine& GetInstance() {
        static NativeDownloadEngine instance;
        return instance;
    }

    int AddDownload(const std::string& url, const std::string& dest, int threads = 4) {
        std::lock_guard<std::mutex> lock(m_mutex);
        int newId = ++m_nextId;
        auto task = std::make_shared<NativeDownloadTask>(newId, url, dest, threads);
        m_tasks.push_back(task);
        task->Start();
        return newId;
    }

    void PauseDownload(int id) {
        std::lock_guard<std::mutex> lock(m_mutex);
        for (auto& t : m_tasks) {
            if (t->id == id) {
                t->Pause();
                break;
            }
        }
    }

    void ResumeDownload(int id) {
        std::lock_guard<std::mutex> lock(m_mutex);
        for (auto& t : m_tasks) {
            if (t->id == id) {
                t->Resume();
                break;
            }
        }
    }

    void CancelDownload(int id) {
        std::lock_guard<std::mutex> lock(m_mutex);
        auto it = std::find_if(m_tasks.begin(), m_tasks.end(), [id](const std::shared_ptr<NativeDownloadTask>& t) {
            return t->id == id;
        });
        if (it != m_tasks.end()) {
            (*it)->Stop();
            m_tasks.erase(it);
        }
    }

    std::string GetDownloadJson(int id) {
        std::lock_guard<std::mutex> lock(m_mutex);
        for (auto& t : m_tasks) {
            if (t->id == id) {
                std::stringstream ss;
                ss << "{"
                   << "\"id\":" << t->id << ","
                   << "\"filename\":\"" << t->filename << "\","
                   << "\"downloaded\":" << t->downloadedBytes.load() << ","
                   << "\"total\":" << t->totalBytes.load() << ","
                   << "\"speed\":" << t->speed.load() << ","
                   << "\"status\":" << static_cast<int>(t->status.load()) << ","
                   << "\"threads\":" << t->numThreads.load() << ","
                   << "\"isTorrent\":" << (t->isTorrent ? "true" : "false")
                   << "}";
                return ss.str();
            }
        }
        return "{}";
    }

    std::string GetAllDownloadsJson() {
        std::lock_guard<std::mutex> lock(m_mutex);
        std::stringstream ss;
        ss << "[";
        for (size_t i = 0; i < m_tasks.size(); ++i) {
            auto& t = m_tasks[i];
            if (i > 0) ss << ",";
            ss << "{"
               << "\"id\":" << t->id << ","
               << "\"filename\":\"" << t->filename << "\","
               << "\"url\":\"" << t->url << "\","
               << "\"destination\":\"" << t->destination << "\","
               << "\"downloaded\":" << t->downloadedBytes.load() << ","
               << "\"total\":" << t->totalBytes.load() << ","
               << "\"speed\":" << t->speed.load() << ","
               << "\"status\":" << static_cast<int>(t->status.load()) << ","
               << "\"threads\":" << t->numThreads.load() << ","
               << "\"isTorrent\":" << (t->isTorrent ? "true" : "false")
               << "}";
        }
        ss << "]";
        return ss.str();
    }

    void PauseAll() {
        std::lock_guard<std::mutex> lock(m_mutex);
        for (auto& t : m_tasks) t->Pause();
    }

    void ResumeAll() {
        std::lock_guard<std::mutex> lock(m_mutex);
        for (auto& t : m_tasks) t->Resume();
    }

private:
    NativeDownloadEngine() { curl_global_init(CURL_GLOBAL_ALL); }
    int m_nextId{0};
    std::vector<std::shared_ptr<NativeDownloadTask>> m_tasks;
    std::mutex m_mutex;
};

} // namespace idr_android

// ============================================================================
// JNI Export Functions
// ============================================================================
extern "C" {

JNIEXPORT jstring JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_stringFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    std::string info = "Internet Downloader Native C++ Core v1.0.0 (Multi-threaded, libcurl-backed)";
    return env->NewStringUTF(info.c_str());
}

JNIEXPORT jint JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_nativeStartDownload(
        JNIEnv* env,
        jobject /* this */,
        jstring urlStr,
        jstring destStr,
        jint threads) {
    const char* url = env->GetStringUTFChars(urlStr, nullptr);
    const char* dest = env->GetStringUTFChars(destStr, nullptr);

    int id = idr_android::NativeDownloadEngine::GetInstance().AddDownload(url, dest, threads);

    env->ReleaseStringUTFChars(urlStr, url);
    env->ReleaseStringUTFChars(destStr, dest);
    return id;
}

JNIEXPORT void JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_nativePauseDownload(
        JNIEnv* /* env */,
        jobject /* this */,
        jint id) {
    idr_android::NativeDownloadEngine::GetInstance().PauseDownload(id);
}

JNIEXPORT void JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_nativeResumeDownload(
        JNIEnv* /* env */,
        jobject /* this */,
        jint id) {
    idr_android::NativeDownloadEngine::GetInstance().ResumeDownload(id);
}

JNIEXPORT void JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_nativeCancelDownload(
        JNIEnv* /* env */,
        jobject /* this */,
        jint id) {
    idr_android::NativeDownloadEngine::GetInstance().CancelDownload(id);
}

JNIEXPORT void JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_nativePauseAll(
        JNIEnv* /* env */,
        jobject /* this */) {
    idr_android::NativeDownloadEngine::GetInstance().PauseAll();
}

JNIEXPORT void JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_nativeResumeAll(
        JNIEnv* /* env */,
        jobject /* this */) {
    idr_android::NativeDownloadEngine::GetInstance().ResumeAll();
}

JNIEXPORT jstring JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_nativeGetDownloadProgress(
        JNIEnv* env,
        jobject /* this */,
        jint id) {
    std::string json = idr_android::NativeDownloadEngine::GetInstance().GetDownloadJson(id);
    return env->NewStringUTF(json.c_str());
}

JNIEXPORT jstring JNICALL
Java_com_nishanth_1kj_internetdownloader_MainActivity_nativeGetAllDownloads(
        JNIEnv* env,
        jobject /* this */) {
    std::string json = idr_android::NativeDownloadEngine::GetInstance().GetAllDownloadsJson();
    return env->NewStringUTF(json.c_str());
}

} // extern "C"
