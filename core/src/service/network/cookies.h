#pragma once

#include <string>

typedef void CURL;

namespace idr {
namespace network {

// A cookie jar backed by a Netscape-format cookie file on disk. The same file is used
// as both the read source and the write destination, so cookies set by the server
// (e.g. session tokens) persist across requests and across app restarts.
class CookieJar {
public:
    explicit CookieJar(std::string filePath) : m_path(std::move(filePath)) {}

    void Apply(CURL* curl) const;
    const std::string& Path() const { return m_path; }
    bool IsSet() const { return !m_path.empty(); }

private:
    std::string m_path;
};

} // namespace network
} // namespace idr
