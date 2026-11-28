# Desktop architecture

The desktop app is one Rust crate in `core/` (`idm_lib` library plus the `1idm`
binary), with small C and C++ parts compiled by `build.rs`.

## Source layout

```text
core/
├─ src/main.rs           startup: idr:// argument, single instance, logging, DB, queue, UI
├─ src/lib.rs            re-exports for the binary and tests
├─ src/config/           Settings (settings.json) and constants: limits, timeouts, paths
├─ src/constants/        Persisted enums (DownloadStatus, error codes, categories)
├─ src/exception/        IdmException and per-area error types
├─ src/model/
│  ├─ database/          SQLite tables: downloads, segments, torrents (SQL lives here)
│  ├─ request/           Request types
│  └─ response/          DownloadSnapshot, DownloadSegment (what the UI renders)
├─ src/service/          Traits (download, network, queue, torrent, ipc) plus runtime helpers
│  └─ impl/              Implementations:
│     ├─ network.rs      NetworkClient (reqwest): probe, ranged and full streams
│     ├─ download.rs     DownloadTask, DynamicScheduler (work stealing), SpeedMeter
│     ├─ queue.rs        QueueManager: list, add, pause/resume/remove, persistence, torrents
│     ├─ torrent.rs      TorrentManager: Rust side of the libtorrent bridge
│     ├─ ipc.rs          Single-instance mutex, named pipe, idr:// decoding
│     └─ fast_c.rs       FFI to c/fast_ops.c
├─ src/ui/               Iced UI (see ui.md)
├─ src/utils/            DB connection, hashing, filesystem and format helpers
├─ c/                    fast_ops.c (CRC32, zero checks)
├─ cpp/                  libtorrent bridge (see torrents.md)
└─ tests/                Integration suite (suite.rs) and the e2e test server
```

## Startup (`main.rs`)

1. Read `args[1]`. An `idr://…` argument is decoded into a URL.
2. If another instance holds the single-instance mutex, send the URL over the named pipe
   `\\.\pipe\InternetDownloader_IPC` and exit.
3. Start logging (stdout and `logs\idr.log`), open `downloads.db`, create the schema.
4. Create `QueueManager`. It loads the saved list and re-adds saved torrents.
5. Start the pipe server; URLs received there are added to the queue.
6. Run the Iced window.

## Threading

- One global multi-threaded Tokio runtime (`service::get_runtime()`); start work with
  `service::spawn`.
- Each HTTP download is a `DownloadTask` running on that runtime. Segmented downloads
  run one worker task per connection.
- The UI thread never waits on network or disk. Engine calls from the UI either return
  immediately or `spawn` their work.
- Progress reaches the UI in two ways: tasks publish `DownloadSnapshot`s on a broadcast
  channel, and the UI also polls `get_all_snapshots()` every 250 ms. Both return the
  live speed and segment map.

## HTTP download flow

1. **Probe** (`NetworkClient::probe`): `HEAD`, falling back to `GET Range: bytes=0-0`,
   to learn the size and whether byte ranges work.
2. **Ranges supported:** `DynamicScheduler` splits the file into segments, one worker
   per segment. A worker that finishes takes half of the largest remaining segment
   (work stealing). All workers write into one pre-allocated file.
3. **No ranges:** a single stream (`stream_full`).
4. Status, sizes and progress go to SQLite every 500 ms and on every status change.

Known engine gaps (ranged replies not validated, stolen segments written past their
new end, 404 shown as Complete, segmented resume not saved) are listed in the
[roadmap](../roadmap.md).

## Storage

SQLite at `%APPDATA%\InternetDownloader\downloads.db`. The SQL for each table sits
next to its struct in `src/model/database/`.

- `downloads`: one row per download. Torrents use the same table, marked with
  `mime_type = application/x-bittorrent`. Their `url` is the magnet link or local
  `.torrent` path, and `destination` is the folder.
- `segments`: helpers exist (`QueueManager::save_segments`), but the engine doesn't
  call them yet.
- `DownloadStatus` values are stored as integers. Add new values at the end; never
  renumber them.

## Settings

`Settings` (`settings.json`) holds the download folder, simultaneous downloads,
connections per download, retries and the speed limit. Options changes reach the
running queue through `QueueManager::update_settings` and apply to downloads added
afterwards.

Simultaneous downloads and the speed limit are saved but the engine doesn't enforce
them yet.
