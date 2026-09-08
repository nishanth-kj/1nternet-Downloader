#pragma once

#include <curl/curl.h>

#include "service/network/request.h"

namespace idr {
namespace network {

// RAII wrapper around a curl easy handle that knows how to apply an HttpRequest's
// common options (URL, range, proxy, TLS, cookies, custom headers).
class CurlHandle {
public:
    CurlHandle();
    ~CurlHandle();

    CurlHandle(const CurlHandle&) = delete;
    CurlHandle& operator=(const CurlHandle&) = delete;

    CURL* Get() const { return m_curl; }
    bool IsValid() const { return m_curl != nullptr; }

    // Applies the request's common options. Any custom headers are turned into a
    // curl_slist owned by this CurlHandle and freed on destruction.
    void ApplyRequest(const HttpRequest& req);

private:
    CURL* m_curl{nullptr};
    curl_slist* m_headerList{nullptr};
};

} // namespace network
} // namespace idr
