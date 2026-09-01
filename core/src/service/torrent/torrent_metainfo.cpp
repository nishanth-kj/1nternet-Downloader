#include "service/torrent/metainfo.h"
#include <fstream>
#include <sstream>
#include <algorithm>
#include <iomanip>
#include <cstring>

namespace idr {
namespace torrent {

// Self-contained, robust SHA-1 implementation (no external OpenSSL DLL dependency)
namespace sha1_internal {

inline uint32_t LeftRotate(uint32_t value, size_t count) {
    return (value << count) ^ (value >> (32 - count));
}

static std::string ComputeSHA1(const std::string& input) {
    uint32_t h0 = 0x67452301;
    uint32_t h1 = 0xEFCDAB89;
    uint32_t h2 = 0x98BADCFE;
    uint32_t h3 = 0x10325476;
    uint32_t h4 = 0xC3D2E1F0;

    std::vector<uint8_t> msg(input.begin(), input.end());
    uint64_t origBits = msg.size() * 8;

    msg.push_back(0x80);
    while ((msg.size() + 8) % 64 != 0) {
        msg.push_back(0x00);
    }

    for (int i = 7; i >= 0; --i) {
        msg.push_back(static_cast<uint8_t>((origBits >> (i * 8)) & 0xFF));
    }

    for (size_t chunk = 0; chunk < msg.size(); chunk += 64) {
        uint32_t w[80];
        for (size_t i = 0; i < 16; ++i) {
            w[i] = (static_cast<uint32_t>(msg[chunk + i * 4]) << 24) |
                   (static_cast<uint32_t>(msg[chunk + i * 4 + 1]) << 16) |
                   (static_cast<uint32_t>(msg[chunk + i * 4 + 2]) << 8) |
                   (static_cast<uint32_t>(msg[chunk + i * 4 + 3]));
        }
        for (size_t i = 16; i < 80; ++i) {
            w[i] = LeftRotate(w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16], 1);
        }

        uint32_t a = h0;
        uint32_t b = h1;
        uint32_t c = h2;
        uint32_t d = h3;
        uint32_t e = h4;

        for (size_t i = 0; i < 80; ++i) {
            uint32_t f, k;
            if (i < 20) {
                f = (b & c) | ((~b) & d);
                k = 0x5A827999;
            } else if (i < 40) {
                f = b ^ c ^ d;
                k = 0x6ED9EBA1;
            } else if (i < 60) {
                f = (b & c) | (b & d) | (c & d);
                k = 0x8F1BBCDC;
            } else {
                f = b ^ c ^ d;
                k = 0xCA62C1D6;
            }

            uint32_t temp = LeftRotate(a, 5) + f + e + k + w[i];
            e = d;
            d = c;
            c = LeftRotate(b, 30);
            b = a;
            a = temp;
        }

        h0 += a;
        h1 += b;
        h2 += c;
        h3 += d;
        h4 += e;
    }

    std::stringstream ss;
    ss << std::hex << std::setfill('0')
       << std::setw(8) << h0
       << std::setw(8) << h1
       << std::setw(8) << h2
       << std::setw(8) << h3
       << std::setw(8) << h4;
    return ss.str();
}

} // namespace sha1_internal

TorrentMetaInfo TorrentMetaInfo::ParseMagnetUri(const std::string& magnetUri) {
    TorrentMetaInfo info;
    if (magnetUri.find("magnet:?") != 0) return info;

    std::string query = magnetUri.substr(8);
    std::stringstream ss(query);
    std::string item;

    while (std::getline(ss, item, '&')) {
        size_t eq = item.find('=');
        if (eq == std::string::npos) continue;

        std::string key = item.substr(0, eq);
        std::string val = item.substr(eq + 1);

        if (key == "xt" && val.find("urn:btih:") == 0) {
            info.infoHashHex = val.substr(9);
            std::transform(info.infoHashHex.begin(), info.infoHashHex.end(), info.infoHashHex.begin(), ::tolower);
            info.isValid = true;
        } else if (key == "dn") {
            std::string decoded;
            for (size_t i = 0; i < val.length(); ++i) {
                if (val[i] == '+') decoded += ' ';
                else if (val[i] == '%' && i + 2 < val.length()) {
                    int ch;
                    std::stringstream hss(val.substr(i + 1, 2));
                    hss >> std::hex >> ch;
                    decoded += static_cast<char>(ch);
                    i += 2;
                } else decoded += val[i];
            }
            info.name = decoded;
        } else if (key == "tr") {
            info.trackers.push_back(val);
            if (info.announce.empty()) info.announce = val;
        }
    }

    if (info.name.empty() && info.isValid) {
        info.name = "magnet_" + info.infoHashHex.substr(0, 8);
    }

    return info;
}

TorrentMetaInfo TorrentMetaInfo::ParseTorrentFile(const std::string& filePath) {
    TorrentMetaInfo info;
    std::ifstream file(filePath, std::ios::binary);
    if (!file.is_open()) return info;

    std::string content((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
    if (content.empty() || content[0] != 'd') return info;

    size_t namePos = content.find("4:name");
    if (namePos != std::string::npos) {
        size_t start = namePos + 6;
        size_t colon = content.find(':', start);
        if (colon != std::string::npos) {
            try {
                int len = std::stoi(content.substr(start, colon - start));
                info.name = content.substr(colon + 1, len);
            } catch (...) {}
        }
    }

    size_t pieceLenPos = content.find("12:piece length");
    if (pieceLenPos != std::string::npos) {
        size_t start = pieceLenPos + 15;
        if (content[start] == 'i') {
            size_t end = content.find('e', start);
            try {
                info.pieceLength = std::stoull(content.substr(start + 1, end - start - 1));
            } catch (...) {}
        }
    }

    size_t lengthPos = content.find("6:length");
    if (lengthPos != std::string::npos) {
        size_t start = lengthPos + 8;
        if (content[start] == 'i') {
            size_t end = content.find('e', start);
            try {
                info.totalLength = std::stoull(content.substr(start + 1, end - start - 1));
            } catch (...) {}
        }
    }

    size_t annPos = content.find("8:announce");
    if (annPos != std::string::npos) {
        size_t start = annPos + 10;
        size_t colon = content.find(':', start);
        if (colon != std::string::npos) {
            try {
                int len = std::stoi(content.substr(start, colon - start));
                info.announce = content.substr(colon + 1, len);
                info.trackers.push_back(info.announce);
            } catch (...) {}
        }
    }

    // Compute SHA-1 of info dictionary
    size_t infoDictPos = content.find("4:info");
    if (infoDictPos != std::string::npos) {
        size_t infoStart = infoDictPos + 6;
        size_t infoEnd = content.rfind('e');
        if (infoEnd > infoStart) {
            std::string infoDict = content.substr(infoStart, infoEnd - infoStart);
            info.infoHashHex = sha1_internal::ComputeSHA1(infoDict);
            info.isValid = true;
        }
    }

    if (info.name.empty()) {
        info.name = "torrent_download";
    }

    return info;
}

} // namespace torrent
} // namespace idr

