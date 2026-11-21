#include "torrent_bridge.h"
#include <string.h>

/* Default stub implementation when compiled without C++ libtorrent linkage */
#if defined(_MSC_VER)
#define WEAK_STUB __declspec(selectany)
#else
#define WEAK_STUB __attribute__((weak))
#endif

int32_t idr_torrent_add_fallback(const char* source, const char* destination_dir, int32_t auto_start) {
    (void)source;
    (void)destination_dir;
    (void)auto_start;
    return -1;
}

int32_t idr_torrent_get_snapshot_fallback(int32_t id, CTorrentSnapshot* out_snapshot) {
    (void)id;
    if (out_snapshot) {
        memset(out_snapshot, 0, sizeof(CTorrentSnapshot));
    }
    return 0;
}

void idr_torrent_pause_fallback(int32_t id) {
    (void)id;
}

void idr_torrent_resume_fallback(int32_t id) {
    (void)id;
}

void idr_torrent_remove_fallback(int32_t id) {
    (void)id;
}
