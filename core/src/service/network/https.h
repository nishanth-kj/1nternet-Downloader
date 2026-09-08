#pragma once

#include <string>

typedef void CURL;

namespace idr {
namespace network {

// TLS/SSL options for HTTPS requests. Peer/host verification is on by default;
// callers should only disable it for diagnostics, never for production downloads.
struct TlsConfig {
    bool verifyPeer{true};
    bool verifyHost{true};
    std::string caBundlePath; // optional path to a custom CA bundle (PEM)
};

void ApplyTls(CURL* curl, const TlsConfig& cfg);

} // namespace network
} // namespace idr
