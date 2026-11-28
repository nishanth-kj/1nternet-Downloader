# Torrents (desktop)

Torrents use **libtorrent-rasterbar 2.x** through a small C++ bridge. Rust never sees
libtorrent types; it calls a plain C API.

```text
src/service/impl/queue.rs   ── QueueManager: torrents in the normal list
        │
src/service/impl/torrent.rs ── TorrentManager: safe Rust wrapper, FFI declarations
        │  extern "C"
cpp/torrent_c_api.{h,cpp}   ── C API; catches every C++ exception
        │
cpp/manager.h               ── TorrentManager / TorrentTask (pimpl, no libtorrent headers)
cpp/torrent_manager.cpp     ── the one lt::session and the torrent handles
```

## Build

`build.rs` compiles `cpp/torrent_manager.cpp` and `cpp/torrent_c_api.cpp` (C++17) with
the same defines as vcpkg's `LibtorrentRasterbar` target, links `torrent-rasterbar.lib`,
and sets `cfg(idr_libtorrent)`. If libtorrent isn't found, that cfg isn't set and
every `TorrentManager` call reports torrent support as unavailable, instead of the
build failing.

Keep the defines in `build.rs` in step with
`vcpkg_installed/x64-windows/share/LibtorrentRasterbar/LibtorrentRasterbarTargets.cmake`
whenever libtorrent is upgraded. A mismatch breaks the ABI.

## Behaviour

- **Sources:** magnet links, http(s) links to `.torrent` files (fetched into
  `%APPDATA%\InternetDownloader\torrents\` first), and local `.torrent` paths.
- **Session:** created on first use, so no BitTorrent ports open until a torrent is
  added. It listens on 6881 with DHT, LSD, UPnP and NAT-PMP enabled.
- **Start and pause:** torrents are added *without* `auto_managed`, so libtorrent's
  queue never overrides a user's pause. Because libtorrent adds torrents paused by
  default, the bridge clears `paused` explicitly when the download should start.
- **In the list:** `QueueManager::torrent_snapshots` maps each torrent to a
  `DownloadSnapshot` (`is_torrent`, seeds and peers, speed, size). Seeding counts as
  Completed. Status changes are written to the database.
- **Restart:** saved torrents are re-added from their source; libtorrent re-checks
  existing data.
- **Remove:** can delete the downloaded data (`lt::session::delete_files`).

## Rules

- libtorrent headers appear only in `cpp/torrent_manager.cpp` (and
  `cpp/torrent_metainfo.cpp`). `manager.h` stays free of `lt::` types.
- No hand-written BitTorrent or bencode code. Use libtorrent's parsers.
- Every function in `torrent_c_api.cpp` catches all exceptions. An exception crossing
  into Rust is undefined behaviour.
- The struct layout in `torrent_c_api.h` and `CTorrentSnapshot` in `torrent.rs` must
  match exactly.

## Not done yet

Upload rate and ratio in the UI, fast-resume data, per-file selection, trackers and
peers views, and speed limits. See the stages in the [roadmap](../roadmap.md).
