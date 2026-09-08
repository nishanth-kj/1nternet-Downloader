#pragma once

#include <string>

namespace idr {
namespace utils {

// Decodes a percent-encoded (application/x-www-form-urlencoded style) string, e.g.
// as produced by JavaScript's encodeURIComponent(). '+' is decoded as a space.
std::string UrlDecode(const std::string& input);

} // namespace utils
} // namespace idr
