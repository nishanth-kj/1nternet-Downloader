#pragma once

#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

#pragma pack(push, 8)
struct CTorrentSnapshot {
    char name[256];
    uint64_t total_bytes;
    uint64_t downloaded_bytes;
    uint64_t uploaded_bytes;
    double download_speed;
    double upload_speed;
    uint32_t peers;
    uint32_t seeds;
    int32_t status;
};
#pragma pack(pop)

int32_t idr_torrent_add(const char* source, const char* destination_dir, int32_t auto_start);
int32_t idr_torrent_get_snapshot(int32_t id, struct CTorrentSnapshot* out_snapshot);
void idr_torrent_pause(int32_t id);
void idr_torrent_resume(int32_t id);
void idr_torrent_remove(int32_t id);

#ifdef __cplusplus
}
#endif
