# core/ — Windows desktop app (Rust)

Docs: [building](../docs/desktop/building.md) · [architecture](../docs/desktop/architecture.md) ·
[UI](../docs/desktop/ui.md) · [torrents](../docs/desktop/torrents.md)

## Commands

```powershell
cargo build      # also compiles c/, cpp/ (libtorrent bridge) and app.rc
cargo test
cargo run        # close any running copy first, or URLs are forwarded to it
cargo check      # when the exe is locked by a running copy
```

## Where things live

- `src/main.rs`: startup, `idr://` argument, single instance, logging, launches the UI.
- `src/service/impl/`: `network.rs` (HTTP), `download.rs` (segmented engine),
  `queue.rs` (list, persistence, torrents), `torrent.rs` (libtorrent FFI), `ipc.rs`.
- `src/model/database/`: SQLite tables with their SQL.
- `src/ui/`: `app.rs` (state and messages), `view.rs` (main window), `dialogs.rs`,
  `prefs.rs`, `theme.rs`, `format.rs`.
- `cpp/`: libtorrent bridge. `c/`: small C helpers.

## Rules for this folder

- Keep the build warning-free.
- Torrent code: libtorrent headers only in `cpp/torrent_manager.cpp`; every C API
  function catches all exceptions; keep the `CTorrentSnapshot` layouts identical in C++
  and Rust; keep the libtorrent defines in `build.rs` matching vcpkg's.
- UI: user settings go in Options, not the View menu. The accent colour is only for
  download progress. Reuse the style helpers in `view.rs`.
- Changing `DownloadSnapshot` fields breaks `tests/` (owner approval needed).
- After engine changes, run the manual check in
  [testing.md](../docs/contributing/testing.md) with a scratch `APPDATA`.
- `scripts/build.ps1` wraps cargo (`-Release`, `-Test`, `-InstallDeps` for libtorrent).
  `build/` and `out/` are leftovers from the old C++ build.
