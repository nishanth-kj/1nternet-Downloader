#pragma once

#include <string>
#include <map>
#include <cstdint>
#include <chrono>

namespace idr {
namespace model {

/**
 * ResponseModel - represents the HTTP response received for a download request.
 * Shared between network service (receives it) and repository (persists it).
 */
struct ResponseModel {
    int64_t     id              = 0;
    int64_t     requestId       = 0;       // FK -> RequestModel::id
    int         statusCode      = 0;       // e.g. 200, 206, 404
    std::string statusMessage;             // e.g. "OK", "Partial Content"
    std::map<std::string, std::string> headers;
    std::string contentType;
    int64_t     contentLength   = -1;      // -1 = unknown
    bool        isPartialContent = false;  // true when status 206
    std::string redirectUrl;               // set if server redirected
    std::string errorMessage;             // set on network/TLS failure
    bool        success         = false;
    std::chrono::system_clock::time_point receivedAt;
};

} // namespace model
} // namespace idr
