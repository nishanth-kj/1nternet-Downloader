#pragma once

#include <functional>

#include "service/network/request.h"
#include "service/network/response.h"

namespace idr {
namespace network {

// Called for each chunk of body data as it arrives. Return the number of bytes
// consumed; returning anything less than `len` aborts the transfer (matches curl's
// write-callback contract).
using DataCallback = std::function<size_t(const char* data, size_t len)>;

// Thin, real libcurl-backed HTTP/HTTPS client used by the download engine.
class HttpClient {
public:
    // Issues a HEAD request to discover content length and range support without
    // downloading the body.
    static HttpResponse Head(const HttpRequest& req);

    // Issues a GET request (optionally with a byte range), streaming the body to onData.
    static HttpResponse Get(const HttpRequest& req, const DataCallback& onData);
};

} // namespace network
} // namespace idr
