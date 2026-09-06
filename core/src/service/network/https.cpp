#include "service/network/https.h"

#include <curl/curl.h>

namespace idr {
namespace network {

void ApplyTls(CURL* curl, const TlsConfig& cfg) {
    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, cfg.verifyPeer ? 1L : 0L);
    curl_easy_setopt(curl, CURLOPT_SSL_VERIFYHOST, cfg.verifyHost ? 2L : 0L);

    if (!cfg.caBundlePath.empty()) {
        curl_easy_setopt(curl, CURLOPT_CAINFO, cfg.caBundlePath.c_str());
    }
}

} // namespace network
} // namespace idr
