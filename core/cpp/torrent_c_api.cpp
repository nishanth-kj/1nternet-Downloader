#include "torrent_c_api.h"
#include "manager.h"
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

// Every entry point catches everything: unwinding a C++ exception into the Rust caller
// is undefined behaviour.

extern "C" {

int32_t idr_torrent_add(const char* source, const char* destination_dir, int32_t auto_start) {
    if (!source || !destination_dir) return -1;
    try {
        auto task = idr::torrent::TorrentManager::GetInstance().AddTorrent(source, destination_dir, auto_start != 0);
        return task ? task->GetId() : -1;
    } catch (...) {
        return -1;
    }
}

int32_t idr_torrent_get_snapshot(int32_t id, struct CTorrentSnapshot* out_snapshot) {
    if (!out_snapshot) return 0;
    try {
        auto task = idr::torrent::TorrentManager::GetInstance().FindTorrent(id);
        if (!task) return 0;
        auto snap = task->GetSnapshot();
        std::strncpy(out_snapshot->name, snap.name.c_str(), sizeof(out_snapshot->name) - 1);
        out_snapshot->name[sizeof(out_snapshot->name) - 1] = '\0';
        out_snapshot->total_bytes = snap.totalBytes;
        out_snapshot->downloaded_bytes = snap.downloadedBytes;
        out_snapshot->uploaded_bytes = snap.uploadedBytes;
        out_snapshot->download_speed = snap.downloadSpeed;
        out_snapshot->upload_speed = snap.uploadSpeed;
        out_snapshot->peers = static_cast<uint32_t>(snap.peers);
        out_snapshot->seeds = static_cast<uint32_t>(snap.seeds);
        out_snapshot->status = static_cast<int32_t>(snap.status);
        return 1;
    } catch (...) {
        return 0;
    }
}

void idr_torrent_pause(int32_t id) {
    try {
        auto task = idr::torrent::TorrentManager::GetInstance().FindTorrent(id);
        if (task) task->Pause();
    } catch (...) {
    }
}

void idr_torrent_resume(int32_t id) {
    try {
        auto task = idr::torrent::TorrentManager::GetInstance().FindTorrent(id);
        if (task) task->Resume();
    } catch (...) {
    }
}

void idr_torrent_remove(int32_t id) {
    idr_torrent_remove_files(id, 0);
}

void idr_torrent_remove_files(int32_t id, int32_t delete_files) {
    try {
        idr::torrent::TorrentManager::GetInstance().RemoveTorrent(id, delete_files != 0);
    } catch (...) {
    }
}

int32_t idr_torrent_error(int32_t id, char* buf, int32_t buf_len) {
    if (!buf || buf_len <= 0) return 0;
    buf[0] = '\0';
    try {
        auto task = idr::torrent::TorrentManager::GetInstance().FindTorrent(id);
        if (!task) return 0;
        std::string err = task->GetError();
        if (err.empty()) return 0;
        std::strncpy(buf, err.c_str(), static_cast<size_t>(buf_len) - 1);
        buf[buf_len - 1] = '\0';
        return 1;
    } catch (...) {
        return 0;
    }
}


namespace {

void CopyStr(char* dst, size_t cap, const std::string& src) {
    std::strncpy(dst, src.c_str(), cap - 1);
    dst[cap - 1] = '\0';
}

std::shared_ptr<idr::torrent::TorrentTask> Find(int32_t id) {
    return idr::torrent::TorrentManager::GetInstance().FindTorrent(id);
}

} // namespace

int32_t idr_torrent_files(int32_t id, struct CTorrentFile* out, int32_t max) {
    try {
        auto t = Find(id);
        if (!t) return -1;
        auto files = t->GetFiles();
        for (int32_t i = 0; out && i < max && i < static_cast<int32_t>(files.size()); ++i) {
            const auto& f = files[static_cast<size_t>(i)];
            out[i].index = f.index;
            out[i].priority = f.priority;
            out[i].size = f.size;
            out[i].done = f.done;
            CopyStr(out[i].path, sizeof(out[i].path), f.path);
        }
        return static_cast<int32_t>(files.size());
    } catch (...) {
        return -1;
    }
}

int32_t idr_torrent_trackers(int32_t id, struct CTorrentTracker* out, int32_t max) {
    try {
        auto t = Find(id);
        if (!t) return -1;
        auto trackers = t->GetTrackers();
        for (int32_t i = 0; out && i < max && i < static_cast<int32_t>(trackers.size()); ++i) {
            const auto& x = trackers[static_cast<size_t>(i)];
            out[i].tier = x.tier;
            out[i].status = x.status;
            out[i].seeds = x.seeds;
            out[i].peers = x.peers;
            out[i].downloaded = x.downloaded;
            CopyStr(out[i].url, sizeof(out[i].url), x.url);
            CopyStr(out[i].message, sizeof(out[i].message), x.message);
        }
        return static_cast<int32_t>(trackers.size());
    } catch (...) {
        return -1;
    }
}

int32_t idr_torrent_peers(int32_t id, struct CTorrentPeer* out, int32_t max) {
    try {
        auto t = Find(id);
        if (!t) return -1;
        auto peers = t->GetPeers();
        for (int32_t i = 0; out && i < max && i < static_cast<int32_t>(peers.size()); ++i) {
            const auto& p = peers[static_cast<size_t>(i)];
            out[i].down_speed = p.downSpeed;
            out[i].up_speed = p.upSpeed;
            out[i].progress = p.progress;
            out[i].is_seed = p.seed ? 1 : 0;
            out[i].downloaded = p.downloaded;
            out[i].uploaded = p.uploaded;
            CopyStr(out[i].address, sizeof(out[i].address), p.address);
            CopyStr(out[i].client, sizeof(out[i].client), p.client);
        }
        return static_cast<int32_t>(peers.size());
    } catch (...) {
        return -1;
    }
}

int32_t idr_torrent_pieces(int32_t id, uint8_t* out, int32_t max) {
    try {
        auto t = Find(id);
        if (!t) return -1;
        auto pieces = t->GetPieces();
        if (out && max > 0) {
            const size_t n = (std::min)(pieces.size(), static_cast<size_t>(max));
            if (n) std::memcpy(out, pieces.data(), n);
        }
        return static_cast<int32_t>(pieces.size());
    } catch (...) {
        return -1;
    }
}

int32_t idr_torrent_extra(int32_t id, struct CTorrentExtra* out) {
    if (!out) return 0;
    try {
        auto t = Find(id);
        if (!t) return 0;
        auto x = t->GetExtra();
        out->all_time_upload = x.allTimeUpload;
        out->all_time_download = x.allTimeDownload;
        out->seeding_seconds = x.seedingSeconds;
        out->active_seconds = x.activeSeconds;
        out->num_pieces = x.numPieces;
        out->pieces_have = x.piecesHave;
        out->piece_length = x.pieceLength;
        CopyStr(out->info_hash, sizeof(out->info_hash), x.infoHash);
        CopyStr(out->save_path, sizeof(out->save_path), x.savePath);
        CopyStr(out->comment, sizeof(out->comment), x.comment);
        CopyStr(out->creator, sizeof(out->creator), x.creator);
        return 1;
    } catch (...) {
        return 0;
    }
}

void idr_torrent_set_file_priority(int32_t id, int32_t index, int32_t priority) {
    try {
        if (auto t = Find(id)) t->SetFilePriority(index, priority);
    } catch (...) {
    }
}

void idr_torrent_set_resume_dir(const char* dir) {
    if (!dir) return;
    try {
        idr::torrent::TorrentManager::GetInstance().SetResumeDir(dir);
    } catch (...) {
    }
}

int32_t idr_torrent_save_resume(int32_t timeout_ms) {
    try {
        return idr::torrent::TorrentManager::GetInstance().SaveResumeData(timeout_ms);
    } catch (...) {
        return 0;
    }
}


namespace {

std::vector<std::string> SplitLines(const char* text) {
    std::vector<std::string> out;
    if (!text) return out;
    std::istringstream in(text);
    std::string line;
    while (std::getline(in, line)) {
        while (!line.empty() && (line.back() == '\r' || line.back() == ' ')) line.pop_back();
        size_t start = line.find_first_not_of(' ');
        if (start != std::string::npos) out.push_back(line.substr(start));
    }
    return out;
}

} // namespace

void idr_torrent_apply_settings(const struct CTorrentSessionSettings* s) {
    if (!s) return;
    try {
        idr::torrent::TorrentSessionSettings x;
        x.listenPort = s->listen_port;
        x.randomPort = s->random_port != 0;
        x.dht = s->dht != 0;
        x.lsd = s->lsd != 0;
        x.pex = s->pex != 0;
        x.utp = s->utp != 0;
        x.upnp = s->upnp != 0;
        x.natpmp = s->natpmp != 0;
        x.encryption = s->encryption;
        x.maxConnections = s->max_connections;
        x.maxPeersPerTorrent = s->max_peers_per_torrent;
        x.downloadLimitKBps = s->download_limit_kbps;
        x.uploadLimitKBps = s->upload_limit_kbps;
        x.announceToAllTrackers = s->announce_all_trackers != 0;
        x.validateHttpsTrackers = s->validate_https_trackers != 0;
        x.userAgent = std::string(s->user_agent, strnlen(s->user_agent, sizeof(s->user_agent)));
        idr::torrent::TorrentManager::ApplySettings(x);
    } catch (...) {
    }
}

void idr_torrent_set_sequential(int32_t id, int32_t on) {
    try {
        if (auto t = Find(id)) t->SetSequential(on != 0);
    } catch (...) {
    }
}

int32_t idr_torrent_is_sequential(int32_t id) {
    try {
        auto t = Find(id);
        return t && t->IsSequential() ? 1 : 0;
    } catch (...) {
        return 0;
    }
}

void idr_torrent_add_tracker(int32_t id, const char* url) {
    if (!url) return;
    try {
        if (auto t = Find(id)) t->AddTracker(url);
    } catch (...) {
    }
}

void idr_torrent_force_recheck(int32_t id) {
    try {
        if (auto t = Find(id)) t->ForceRecheck();
    } catch (...) {
    }
}

void idr_torrent_force_reannounce(int32_t id) {
    try {
        if (auto t = Find(id)) t->ForceReannounce();
    } catch (...) {
    }
}

int32_t idr_torrent_create(const char* source_path, const char* output_file, const char* trackers,
                           const char* web_seeds, const char* comment, int32_t piece_size_kib,
                           int32_t is_private, char* err, int32_t err_len) {
    std::string message;
    try {
        if (!source_path || !output_file) {
            message = "Missing source or output path.";
        } else {
            idr::torrent::TorrentCreateOptions o;
            o.sourcePath = source_path;
            o.outputFile = output_file;
            o.trackers = SplitLines(trackers);
            o.webSeeds = SplitLines(web_seeds);
            o.comment = comment ? comment : "";
            o.pieceSizeKiB = piece_size_kib;
            o.isPrivate = is_private != 0;
            message = idr::torrent::CreateTorrentFile(o);
        }
    } catch (...) {
        message = "Creating the torrent failed.";
    }
    if (err && err_len > 0) CopyStr(err, static_cast<size_t>(err_len), message);
    return message.empty() ? 1 : 0;
}

}
