#pragma once

#include <string>

typedef void CURL;

namespace idr {
namespace network {

struct ProxyConfig {
    bool enabled{false};
    std::string host;
    int port{0};
    std::string username;
    std::string password;
};

// Applies the proxy configuration to a curl easy handle. No-op when disabled or host is empty.
void ApplyProxy(CURL* curl, const ProxyConfig& cfg);

} // namespace network
} // namespace idr
