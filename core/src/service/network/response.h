#pragma once

#include <cstdint>
#include <string>

#include "service/network/headers.h"

namespace idr {
namespace network {

struct HttpResponse {
    bool ok{false};
    long statusCode{0};
    int64_t contentLength{-1};
    bool acceptRangesBytes{false};
    bool rejectedByValidator{false}; // a ResponseValidator aborted the body
    std::string finalUrl;
    std::string errorMessage;
    HttpHeaders headers;

    std::string Summary() const;
};

} // namespace network
} // namespace idr
