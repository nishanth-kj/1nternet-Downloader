#include "service/network/cookies.h"

#include <curl/curl.h>

namespace idr {
namespace network {

void CookieJar::Apply(CURL* curl) const {
    if (m_path.empty()) return;
    curl_easy_setopt(curl, CURLOPT_COOKIEFILE, m_path.c_str());
    curl_easy_setopt(curl, CURLOPT_COOKIEJAR, m_path.c_str());
}

} // namespace network
} // namespace idr
