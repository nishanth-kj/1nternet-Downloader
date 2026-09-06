#pragma once

#include <cstdint>
#include <string>

#include "service/network/cookies.h"
#include "service/network/headers.h"
#include "service/network/https.h"
#include "service/network/proxy.h"

namespace idr {
namespace network {

// Describes a single HTTP(S) request. Used for both HEAD probes and byte-range GETs,
// which is all the download engine needs.
struct HttpRequest {
    std::string url;
    std::string userAgent{"Mozilla/5.0 (Windows NT 10.0; Win64; x64) InternetDownloader/1.0"};

    bool headOnly{false};
    bool followRedirects{true};
    long timeoutSeconds{0}; // 0 = curl default (no timeout)

    // Byte range, inclusive. rangeEnd < 0 means "to end of file". Both < 0 means no range.
    int64_t rangeStart{-1};
    int64_t rangeEnd{-1};

    HttpHeaders headers;
    ProxyConfig proxy;
    TlsConfig tls;
    std::string cookieJarPath;

    bool HasRange() const { return rangeStart >= 0; }
    std::string BuildRangeHeader() const;
};

} // namespace network
} // namespace idr
