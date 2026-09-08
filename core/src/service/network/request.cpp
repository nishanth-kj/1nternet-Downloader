#include "service/network/request.h"

namespace idr {
namespace network {

std::string HttpRequest::BuildRangeHeader() const {
    if (!HasRange()) return "";
    if (rangeEnd < 0) return std::to_string(rangeStart) + "-";
    return std::to_string(rangeStart) + "-" + std::to_string(rangeEnd);
}

} // namespace network
} // namespace idr
