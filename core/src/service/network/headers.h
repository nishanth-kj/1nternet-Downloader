#pragma once

#include <string>
#include <vector>
#include <utility>

struct curl_slist;

namespace idr {
namespace network {

// Ordered, case-insensitive collection of HTTP header name/value pairs.
class HttpHeaders {
public:
    void Set(const std::string& name, const std::string& value);
    void AddFromRawLine(const std::string& line); // parses a single "Name: Value" line from curl's header callback
    std::string Get(const std::string& name) const;
    bool Has(const std::string& name) const;

    const std::vector<std::pair<std::string, std::string>>& All() const { return m_entries; }

    // Builds a curl_slist for CURLOPT_HTTPHEADER. Caller owns the result and must
    // free it with curl_slist_free_all().
    curl_slist* ToCurlSlist() const;

private:
    std::vector<std::pair<std::string, std::string>> m_entries;
};

} // namespace network
} // namespace idr
