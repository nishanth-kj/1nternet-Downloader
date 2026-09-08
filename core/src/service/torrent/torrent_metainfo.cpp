#include "service/torrent/metainfo.h"

#include <libtorrent/add_torrent_params.hpp>
#include <libtorrent/error_code.hpp>
#include <libtorrent/file_storage.hpp>
#include <libtorrent/info_hash.hpp>
#include <libtorrent/load_torrent.hpp>
#include <libtorrent/magnet_uri.hpp>
#include <libtorrent/sha1_hash.hpp>
#include <libtorrent/torrent_info.hpp>

namespace lt = libtorrent;

namespace idr {
namespace torrent {

namespace {

std::string ToHex(const lt::sha1_hash& hash) {
    static const char* digits = "0123456789abcdef";
    std::string out;
    out.reserve(40);
    for (int i = 0; i < 20; ++i) {
        unsigned char b = static_cast<unsigned char>(hash[i]);
        out.push_back(digits[(b >> 4) & 0xF]);
        out.push_back(digits[b & 0xF]);
    }
    return out;
}

} // namespace

TorrentMetaInfo TorrentMetaInfo::ParseMagnetUri(const std::string& magnetUri) {
    TorrentMetaInfo info;

    lt::error_code ec;
    lt::add_torrent_params atp = lt::parse_magnet_uri(magnetUri, ec);
    if (ec) return info;

    info.name = atp.name;
    for (const auto& tracker : atp.trackers) {
        info.trackers.push_back(tracker);
    }
    if (!info.trackers.empty()) info.announce = info.trackers.front();

    if (atp.info_hashes.has_v1()) {
        info.infoHashHex = ToHex(atp.info_hashes.v1);
    }

    info.isValid = true;
    if (info.name.empty()) {
        info.name = info.infoHashHex.empty() ? "magnet_download" : ("magnet_" + info.infoHashHex.substr(0, 8));
    }

    return info;
}

TorrentMetaInfo TorrentMetaInfo::ParseTorrentFile(const std::string& filePath) {
    TorrentMetaInfo info;

    lt::add_torrent_params atp;
    try {
        atp = lt::load_torrent_file(filePath);
    } catch (const std::exception&) {
        return info;
    }

    if (!atp.ti || !atp.ti->is_valid()) return info;

    info.name = atp.ti->name();
    info.pieceLength = static_cast<uint64_t>(atp.ti->piece_length());
    info.totalLength = static_cast<uint64_t>(atp.ti->total_size());

    if (atp.ti->info_hashes().has_v1()) {
        info.infoHashHex = ToHex(atp.ti->info_hashes().v1);
    }

    for (const auto& tracker : atp.trackers) {
        info.trackers.push_back(tracker);
    }
    if (!info.trackers.empty()) info.announce = info.trackers.front();

    const lt::file_storage& files = atp.ti->layout();
    info.isMultiFile = files.num_files() > 1;
    for (auto idx : files.file_range()) {
        TorrentFileEntry entry;
        entry.path = files.file_path(idx);
        entry.length = static_cast<uint64_t>(files.file_size(idx));
        info.files.push_back(entry);
    }

    info.isValid = true;
    if (info.name.empty()) info.name = "torrent_download";

    return info;
}

} // namespace torrent
} // namespace idr
