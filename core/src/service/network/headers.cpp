#include "service/network/headers.h"

#include <curl/curl.h>
#include <algorithm>
#include <cctype>

namespace idr {
namespace network {

namespace {

bool IEquals(const std::string& a, const std::string& b) {
    if (a.size() != b.size()) return false;
    return std::equal(a.begin(), a.end(), b.begin(), [](unsigned char x, unsigned char y) {
        return std::tolower(x) == std::tolower(y);
    });
}

std::string Trim(const std::string& s) {
    size_t start = s.find_first_not_of(" \t\r\n");
    if (start == std::string::npos) return "";
    size_t end = s.find_last_not_of(" \t\r\n");
    return s.substr(start, end - start + 1);
}

} // namespace

void HttpHeaders::Set(const std::string& name, const std::string& value) {
    for (auto& entry : m_entries) {
        if (IEquals(entry.first, name)) {
            entry.second = value;
            return;
        }
    }
    m_entries.emplace_back(name, value);
}

void HttpHeaders::AddFromRawLine(const std::string& line) {
    size_t colon = line.find(':');
    if (colon == std::string::npos) return;

    std::string name = Trim(line.substr(0, colon));
    std::string value = Trim(line.substr(colon + 1));
    if (name.empty()) return;

    Set(name, value);
}

std::string HttpHeaders::Get(const std::string& name) const {
    for (const auto& entry : m_entries) {
        if (IEquals(entry.first, name)) return entry.second;
    }
    return "";
}

bool HttpHeaders::Has(const std::string& name) const {
    for (const auto& entry : m_entries) {
        if (IEquals(entry.first, name)) return true;
    }
    return false;
}

curl_slist* HttpHeaders::ToCurlSlist() const {
    curl_slist* list = nullptr;
    for (const auto& entry : m_entries) {
        std::string line = entry.first + ": " + entry.second;
        list = curl_slist_append(list, line.c_str());
    }
    return list;
}

} // namespace network
} // namespace idr
