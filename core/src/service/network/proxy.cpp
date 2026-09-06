#include "service/network/proxy.h"

#include <curl/curl.h>

namespace idr {
namespace network {

void ApplyProxy(CURL* curl, const ProxyConfig& cfg) {
    if (!cfg.enabled || cfg.host.empty()) return;

    std::string proxyUrl = cfg.host + ":" + std::to_string(cfg.port);
    curl_easy_setopt(curl, CURLOPT_PROXY, proxyUrl.c_str());

    if (!cfg.username.empty()) {
        std::string userpwd = cfg.username + ":" + cfg.password;
        curl_easy_setopt(curl, CURLOPT_PROXYUSERPWD, userpwd.c_str());
    }
}

} // namespace network
} // namespace idr
