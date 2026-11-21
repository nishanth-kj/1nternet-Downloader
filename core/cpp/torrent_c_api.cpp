#include "torrent_c_api.h"
#include "manager.h"
#include <cstring>

extern "C" {

int32_t idr_torrent_add(const char* source, const char* destination_dir, int32_t auto_start) {
    if (!source || !destination_dir) return -1;
    auto task = idr::torrent::TorrentManager::GetInstance().AddTorrent(source, destination_dir, auto_start != 0);
    return task ? task->GetId() : -1;
}

int32_t idr_torrent_get_snapshot(int32_t id, struct CTorrentSnapshot* out_snapshot) {
    if (!out_snapshot) return 0;
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
}

void idr_torrent_pause(int32_t id) {
    auto task = idr::torrent::TorrentManager::GetInstance().FindTorrent(id);
    if (task) task->Pause();
}

void idr_torrent_resume(int32_t id) {
    auto task = idr::torrent::TorrentManager::GetInstance().FindTorrent(id);
    if (task) task->Resume();
}

void idr_torrent_remove(int32_t id) {
    idr::torrent::TorrentManager::GetInstance().RemoveTorrent(id);
}

}
