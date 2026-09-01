#include "service/torrent/metainfo.h"
#include <fstream>
#include <sstream>
#include <algorithm>
#include <openssl/sha.h>
#include <iomanip>

namespace idr {
namespace torrent {

// Helper: Hex string encoder
static std::string ToHex(const unsigned char* data, size_t len) {
    std::stringstream ss;
    for (size_t i = 0; i < len; ++i) {
        ss << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(data[i]);
    }
    return ss.str();
}

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
            // URL decode display name
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

    // Simple robust bencode extractor for main fields
    size_t namePos = content.find("4:name");
    if (namePos != std::string::npos) {
        size_t start = namePos + 6;
        size_t colon = content.find(':', start);
        if (colon != std::string::npos) {
            int len = std::stoi(content.substr(start, colon - start));
            info.name = content.substr(colon + 1, len);
        }
    }

    size_t pieceLenPos = content.find("12:piece length");
    if (pieceLenPos != std::string::npos) {
        size_t start = pieceLenPos + 15;
        if (content[start] == 'i') {
            size_t end = content.find('e', start);
            info.pieceLength = std::stoull(content.substr(start + 1, end - start - 1));
        }
    }

    size_t lengthPos = content.find("6:length");
    if (lengthPos != std::string::npos) {
        size_t start = lengthPos + 8;
        if (content[start] == 'i') {
            size_t end = content.find('e', start);
            info.totalLength = std::stoull(content.substr(start + 1, end - start - 1));
        }
    }

    size_t annPos = content.find("8:announce");
    if (annPos != std::string::npos) {
        size_t start = annPos + 10;
        size_t colon = content.find(':', start);
        if (colon != std::string::npos) {
            int len = std::stoi(content.substr(start, colon - start));
            info.announce = content.substr(colon + 1, len);
            info.trackers.push_back(info.announce);
        }
    }

    // Calculate SHA-1 of info dictionary if found
    size_t infoDictPos = content.find("4:info");
    if (infoDictPos != std::string::npos) {
        size_t infoStart = infoDictPos + 6;
        size_t infoEnd = content.rfind('e');
        if (infoEnd > infoStart) {
            unsigned char hash[SHA_DIGEST_LENGTH];
            SHA1(reinterpret_cast<const unsigned char*>(&content[infoStart]), infoEnd - infoStart, hash);
            info.infoHashHex = ToHex(hash, SHA_DIGEST_LENGTH);
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
