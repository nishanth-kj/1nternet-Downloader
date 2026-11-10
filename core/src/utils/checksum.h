#pragma once

#include <atomic>
#include <string>

namespace idr {
namespace utils {

// Computes the SHA-256 of a file (Windows CNG / bcrypt) and returns it as 64
// lowercase hex characters. Returns an empty string on failure or cancellation,
// with a reason in *error if provided.
std::string Sha256File(const std::string& path, std::string* error = nullptr,
                       const std::atomic<bool>* cancel = nullptr);

// Lowercases and strips whitespace so pasted hashes compare reliably.
std::string NormalizeHexDigest(const std::string& digest);

} // namespace utils
} // namespace idr
