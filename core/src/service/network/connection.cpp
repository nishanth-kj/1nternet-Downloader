#include "service/network/connection.h"

#include "service/network/cookies.h"
#include "service/network/https.h"
#include "service/network/proxy.h"

namespace idr {
namespace network {

CurlHandle::CurlHandle() : m_curl(curl_easy_init()) {}

CurlHandle::~CurlHandle() {
    if (m_headerList) curl_slist_free_all(m_headerList);
    if (m_curl) curl_easy_cleanup(m_curl);
}

void CurlHandle::ApplyRequest(const HttpRequest& req) {
    if (!m_curl) return;

    curl_easy_setopt(m_curl, CURLOPT_URL, req.url.c_str());
    curl_easy_setopt(m_curl, CURLOPT_USERAGENT, req.userAgent.c_str());
    curl_easy_setopt(m_curl, CURLOPT_FOLLOWLOCATION, req.followRedirects ? 1L : 0L);
    curl_easy_setopt(m_curl, CURLOPT_NOBODY, req.headOnly ? 1L : 0L);

    if (req.timeoutSeconds > 0) {
        curl_easy_setopt(m_curl, CURLOPT_TIMEOUT, req.timeoutSeconds);
    }
    if (req.connectTimeoutSeconds > 0) {
        curl_easy_setopt(m_curl, CURLOPT_CONNECTTIMEOUT, req.connectTimeoutSeconds);
    }
    if (req.stallTimeoutSeconds > 0) {
        curl_easy_setopt(m_curl, CURLOPT_LOW_SPEED_LIMIT, 1L);
        curl_easy_setopt(m_curl, CURLOPT_LOW_SPEED_TIME, req.stallTimeoutSeconds);
    }
    if (req.maxRecvBytesPerSec > 0) {
        curl_easy_setopt(m_curl, CURLOPT_MAX_RECV_SPEED_LARGE,
                         static_cast<curl_off_t>(req.maxRecvBytesPerSec));
    }
    // Worker threads must not receive SIGALRM-based DNS timeouts.
    curl_easy_setopt(m_curl, CURLOPT_NOSIGNAL, 1L);

    if (req.HasRange()) {
        curl_easy_setopt(m_curl, CURLOPT_RANGE, req.BuildRangeHeader().c_str());
    }

    ApplyTls(m_curl, req.tls);
    ApplyProxy(m_curl, req.proxy);

    if (!req.cookieJarPath.empty()) {
        CookieJar jar(req.cookieJarPath);
        jar.Apply(m_curl);
    }

    if (!req.headers.All().empty()) {
        m_headerList = req.headers.ToCurlSlist();
        curl_easy_setopt(m_curl, CURLOPT_HTTPHEADER, m_headerList);
    }
}

} // namespace network
} // namespace idr
