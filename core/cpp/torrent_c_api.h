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
struct CTorrentFile {
    int32_t index;
    int32_t priority;
    uint64_t size;
    uint64_t done;
    char path[520];
};

struct CTorrentTracker {
    int32_t tier;
    int32_t status;
    int32_t seeds;
    int32_t peers;
    int32_t downloaded;
    char url[520];
    char message[256];
};

struct CTorrentPeer {
    double down_speed;
    double up_speed;
    float progress;
    int32_t is_seed;
    uint64_t downloaded;
    uint64_t uploaded;
    char address[64];
    char client[96];
};

struct CTorrentExtra {
    uint64_t all_time_upload;
    uint64_t all_time_download;
    int64_t seeding_seconds;
    int64_t active_seconds;
    int32_t num_pieces;
    int32_t pieces_have;
    int32_t piece_length;
    char info_hash[72];
    char save_path[520];
    char comment[256];
    char creator[128];
};
struct CTorrentSessionSettings {
    int32_t listen_port;
    int32_t random_port;
    int32_t dht;
    int32_t lsd;
    int32_t pex;
    int32_t utp;
    int32_t upnp;
    int32_t natpmp;
    int32_t encryption;
    int32_t max_connections;
    int32_t max_peers_per_torrent;
    int32_t download_limit_kbps;
    int32_t upload_limit_kbps;
    int32_t announce_all_trackers;
    int32_t validate_https_trackers;
    char user_agent[128];
};
#pragma pack(pop)

int32_t idr_torrent_add(const char* source, const char* destination_dir, int32_t auto_start);
int32_t idr_torrent_get_snapshot(int32_t id, struct CTorrentSnapshot* out_snapshot);
void idr_torrent_pause(int32_t id);
void idr_torrent_resume(int32_t id);
void idr_torrent_remove(int32_t id);
// Removes the torrent; when delete_files is non-zero the downloaded data is deleted too.
void idr_torrent_remove_files(int32_t id, int32_t delete_files);
// Copies the torrent's error text into buf (NUL-terminated). Returns 1 if there is an error.
int32_t idr_torrent_error(int32_t id, char* buf, int32_t buf_len);

// Details queries. Each fills up to max entries and returns the total count
// (which may be larger than max), or -1 if the torrent doesn't exist.
int32_t idr_torrent_files(int32_t id, struct CTorrentFile* out, int32_t max);
int32_t idr_torrent_trackers(int32_t id, struct CTorrentTracker* out, int32_t max);
int32_t idr_torrent_peers(int32_t id, struct CTorrentPeer* out, int32_t max);
int32_t idr_torrent_pieces(int32_t id, uint8_t* out, int32_t max);
int32_t idr_torrent_extra(int32_t id, struct CTorrentExtra* out);
// priority: 0 = skip, 1..7 (4 = normal, 7 = top).
void idr_torrent_set_file_priority(int32_t id, int32_t index, int32_t priority);

// Fast-resume. Set the folder once at startup; save periodically and before exit.
void idr_torrent_set_resume_dir(const char* dir);
int32_t idr_torrent_save_resume(int32_t timeout_ms);

void idr_torrent_apply_settings(const struct CTorrentSessionSettings* settings);
void idr_torrent_set_sequential(int32_t id, int32_t on);
int32_t idr_torrent_is_sequential(int32_t id);
void idr_torrent_add_tracker(int32_t id, const char* url);
void idr_torrent_force_recheck(int32_t id);
void idr_torrent_force_reannounce(int32_t id);

// trackers / web_seeds: newline-separated lists (may be empty). Returns 1 on success;
// otherwise 0 and an error message in err.
int32_t idr_torrent_create(const char* source_path, const char* output_file, const char* trackers,
                           const char* web_seeds, const char* comment, int32_t piece_size_kib,
                           int32_t is_private, char* err, int32_t err_len);

#ifdef __cplusplus
}
#endif
