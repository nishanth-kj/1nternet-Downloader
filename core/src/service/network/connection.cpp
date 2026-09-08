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
