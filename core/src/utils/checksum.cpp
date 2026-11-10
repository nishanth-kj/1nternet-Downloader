#include "utils/checksum.h"

#include <cctype>
#include <fstream>
#include <vector>

#ifdef _WIN32
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <bcrypt.h>
#endif

namespace idr {
namespace utils {

std::string NormalizeHexDigest(const std::string& digest) {
    std::string out;
    out.reserve(digest.size());
    for (char c : digest) {
        if (std::isspace(static_cast<unsigned char>(c))) continue;
        out.push_back(static_cast<char>(std::tolower(static_cast<unsigned char>(c))));
    }
    return out;
}

#ifdef _WIN32
std::string Sha256File(const std::string& path, std::string* error,
                       const std::atomic<bool>* cancel) {
    auto fail = [error](const std::string& msg) {
        if (error) *error = msg;
        return std::string();
    };

    std::ifstream in(path, std::ios::binary);
    if (!in.is_open()) return fail("cannot open file");

    BCRYPT_ALG_HANDLE alg = nullptr;
    if (!BCRYPT_SUCCESS(BCryptOpenAlgorithmProvider(&alg, BCRYPT_SHA256_ALGORITHM, nullptr, 0))) {
        return fail("SHA-256 provider unavailable");
    }

    BCRYPT_HASH_HANDLE hash = nullptr;
    if (!BCRYPT_SUCCESS(BCryptCreateHash(alg, &hash, nullptr, 0, nullptr, 0, 0))) {
        BCryptCloseAlgorithmProvider(alg, 0);
        return fail("cannot create hash object");
    }

    std::vector<char> buffer(1 << 20);
    bool ok = true;
    std::string reason;
    while (in) {
        if (cancel && cancel->load()) { ok = false; reason = "cancelled"; break; }
        in.read(buffer.data(), static_cast<std::streamsize>(buffer.size()));
        const auto got = in.gcount();
        if (got <= 0) break;
        if (!BCRYPT_SUCCESS(BCryptHashData(hash, reinterpret_cast<PUCHAR>(buffer.data()),
                                           static_cast<ULONG>(got), 0))) {
            ok = false; reason = "hashing failed"; break;
        }
    }
    if (ok && in.bad()) { ok = false; reason = "read error"; }

    unsigned char digest[32] = {};
    if (ok && !BCRYPT_SUCCESS(BCryptFinishHash(hash, digest, sizeof(digest), 0))) {
        ok = false; reason = "hashing failed";
    }
    BCryptDestroyHash(hash);
    BCryptCloseAlgorithmProvider(alg, 0);
    if (!ok) return fail(reason);

    static const char* hex = "0123456789abcdef";
    std::string out;
    out.reserve(64);
    for (unsigned char b : digest) {
        out.push_back(hex[b >> 4]);
        out.push_back(hex[b & 0x0F]);
    }
    return out;
}
#else
std::string Sha256File(const std::string&, std::string* error, const std::atomic<bool>*) {
    if (error) *error = "SHA-256 is only implemented on Windows";
    return {};
}
#endif

} // namespace utils
} // namespace idr
