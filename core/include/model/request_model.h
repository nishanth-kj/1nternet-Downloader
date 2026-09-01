#pragma once

#include <string>
#include <map>
#include <vector>
#include <cstdint>
#include <chrono>

namespace idr {
namespace model {

enum class HttpMethod {
    GET,
    POST,
    HEAD,
    PUT,
    DELETE_
};

inline std::string MethodToString(HttpMethod m) {
    switch (m) {
        case HttpMethod::GET:     return "GET";
        case HttpMethod::POST:    return "POST";
        case HttpMethod::HEAD:    return "HEAD";
        case HttpMethod::PUT:     return "PUT";
        case HttpMethod::DELETE_: return "DELETE";
        default:                  return "GET";
    }
}

inline HttpMethod MethodFromString(const std::string& s) {
    if (s == "POST")   return HttpMethod::POST;
    if (s == "HEAD")   return HttpMethod::HEAD;
    if (s == "PUT")    return HttpMethod::PUT;
    if (s == "DELETE") return HttpMethod::DELETE_;
    return HttpMethod::GET;
}

/**
 * RequestModel - represents an outgoing HTTP download request.
 * Shared between network service (sends it) and repository (persists it).
 */
struct RequestModel {
    int64_t     id          = 0;
    std::string url;
    HttpMethod  method      = HttpMethod::GET;
    std::map<std::string, std::string> headers;
    std::string body;                        // for POST/PUT
    std::string userAgent;
    std::string proxyUrl;
    int         timeoutSeconds  = 30;
    int         maxRetries      = 3;
    bool        followRedirects = true;
    std::chrono::system_clock::time_point createdAt;
};

} // namespace model
} // namespace idr
