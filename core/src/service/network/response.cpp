#include "service/network/response.h"

namespace idr {
namespace network {

std::string HttpResponse::Summary() const {
    if (!ok) return "error: " + errorMessage;
    return "HTTP " + std::to_string(statusCode) +
           " (content-length=" + std::to_string(contentLength) +
           ", accept-ranges=" + (acceptRangesBytes ? "yes" : "no") + ")";
}

} // namespace network
} // namespace idr
