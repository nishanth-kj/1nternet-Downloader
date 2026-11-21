#pragma once

#include <string>
#include <vector>
#include <cstdint>
#include <map>

namespace idr {
namespace torrent {

struct TorrentFileEntry {
    std::string path;
    uint64_t length{0};
};

struct TorrentMetaInfo {
    std::string name;
    std::string infoHashHex;
    std::string announce;
    std::vector<std::string> trackers;
    uint64_t pieceLength{0};
    uint64_t totalLength{0};
    std::vector<std::string> pieceHashes;
    std::vector<TorrentFileEntry> files;
    bool isMultiFile{false};
    bool isValid{false};

    static TorrentMetaInfo ParseTorrentFile(const std::string& filePath);
    static TorrentMetaInfo ParseMagnetUri(const std::string& magnetUri);
};

} // namespace torrent
} // namespace idr
