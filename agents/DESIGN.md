# Design — desktop app (`core/`) and how the products connect

## 1. System context

```text
 Browser extension ──idr://<url>──▶ Windows protocol handler (HKCU, register_protocol.ps1)
                                         │
                                         ▼
                        InternetDownloader.exe  (2nd launch forwards URL over
                        named pipe to the running instance, then exits)
                                         │
             ┌───────────────────────────┼───────────────────────────┐
             ▼                           ▼                           ▼
        GUI (wxWidgets)         DownloadManager (singleton)   TorrentManager (singleton)
        polls every 500 ms ───▶ list + queue + SQLite sync ──▶ one lt::session (pimpl)
                                         │
                                         ▼
                                Download (one per item)
                                HEAD probe → segmented or single stream
                                         │
                                         ▼
                                network::HttpClient (libcurl)
```

The Android app is a separate codebase with its own engine. It mirrors the
desktop's range-validation rules but shares no code.

## 2. Source layout (`core/src`)

| Path | Responsibility |
| --- | --- |
| `gui/main_window.*` | Frame, menus, list, sidebar, action panel, link grabber, timer |
| `gui/settings_dialog.*` | Options dialog for `system::Settings` |
| `gui/tray_icon.*` | System tray icon and menu |
| `service/download/download.*` | One transfer: probe, segment workers, retries, assembly, torrent polling |
| `service/download/manager.*` | Owns all `Download`s, the queue, and SQLite sync |
| `service/download/progress.*` | `SpeedMeter`: smoothed bytes/s |
| `service/network/*` | `HttpClient::Head/Get`, `CurlHandle` RAII, headers, proxy, TLS, cookies |
| `service/torrent/manager.h` + `torrent_manager.cpp` | `TorrentManager`/`TorrentTask` around libtorrent (pimpl) |
| `service/torrent/metainfo.h` + `torrent_metainfo.cpp` | Magnet/.torrent parsing via libtorrent |
| `model/sqlite_download_model.h` | `DownloadModel` and `DownloadDB` (sqlite_orm) — the real DB layer |
| `utils/system/config.*` | `Settings` + `Config` singleton (JSON in app data) |
| `utils/system/platform.*` | `AppDataDir()`, `DefaultDownloadsDir()` (Known Folders) |
| `utils/system/logger.*` | `Log()`: rotating spdlog file logger |
| `utils/system/ipc.*` | Single-instance mutex + named-pipe URL forwarding |
| `utils/checksum.*` | `Sha256File()` via Windows CNG (bcrypt) |
| `utils/string_utils.*` | `UrlDecode()` |
| `utils/database/database.h` | Placeholder, intentionally unused |

## 3. Download state machine

```text
            add (autoStart)            ProcessQueue (slot free)
   (new) ──────────────────▶ Queued ──────────────────────────▶ Downloading
                               ▲  ▲                             │  │  │
                     Resume /  │  │ Restart                     │  │  └─ all bytes verified ─▶ Completed
                     ResumeAll │  └─────────────────────────────┘  │
                               │                                   ├─ Pause / shutdown ─▶ Paused
                         Paused, Error ◀── permanent failure or ───┘
                                           retries exhausted
```

- Only `DownloadManager::ProcessQueueLocked()` moves **Queued → Downloading**. The
  GUI timer calls it every 500 ms, and it is also called after add, resume,
  remove and Options changes. A finished download frees its slot without a
  callback into the manager, which avoids lock-order problems.
- The status enum values (0–4) are persisted in SQLite. Never renumber them.

## 4. Threading model

| Thread | Owns | Touches |
| --- | --- | --- |
| GUI (wx main) | widgets, `wxTimer` | `DownloadManager` API, `Download` getters (atomics) |
| Coordinator (1 per active download, `Download::m_workerThread`) | HEAD probe, segment threads, assembly | file system, `m_segmentMutex`, atomics |
| Segment workers (N per segmented download) | one `.partN` file each | libcurl, `m_segmentMutex` |
| libtorrent session threads | everything torrent | — (polled via `GetSnapshot()`) |
| IPC pipe server | named pipe | posts to GUI via `CallAfter` |
| Hash/link-grabber threads | detached, short-lived | post to GUI via `CallAfter`, guarded by `m_alive` |

Rules:
- The coordinator joins every segment worker before it returns.
  `Download::~Download` calls `Stop()` and then `WaitForWorker()`. No engine thread
  is detached.
- `m_stopRequested` is checked in every write callback (returning 0 aborts curl) and
  inside backoff sleeps (100 ms slices). Pause therefore takes effect within about
  one chunk or 100 ms.
- Status callbacks (`NotifyStatusChanged` → `SyncDownloadState`) run on any thread
  and take **only** `m_dbMutex`, never `m_mutex`. Manager methods hold `m_mutex`
  while they call `Download` methods. Lock order: `m_mutex` → `Download` internals → `m_dbMutex`.

## 5. HTTP engine details

1. `HttpClient::Head` → total size and `Accept-Ranges`. Headers are reset at every
   `HTTP/` status line, so redirect hops don't leak headers into the final response.
2. Choose the segment count: if `.part0..N` files already exist, use **N**, so the
   layout matches the earlier run even if the Options changed. Otherwise use
   `ChooseSegmentCount(size, maxSegments)`.
3. Each segment worker loops: measure the `.partN` size → request the remaining range
   with a `ResponseValidator` (requires 206 + exact `Content-Range`) → append →
   on failure classify as permanent or transient → back off.
4. The coordinator samples `m_downloadedBytes` every 250 ms into `SpeedMeter`. It
   joins the workers, then calls `AssembleParts()`, which checks the size, swaps
   in the file, and deletes the parts.
5. Single stream: the output file is opened **lazily on the first validated byte**,
   so a 404 or a rejected resume never truncates existing data. A resume is
   appended only after a `206 bytes <existing>-` response. A `200` sets
   `allowResume=false` and the next attempt restarts.

libcurl options per connection: 30 s connect timeout, stall abort when < 1 B/s for
60 s, `CURLOPT_MAX_RECV_SPEED_LARGE` when limited, and `NOSIGNAL`.

## 6. Persistence

- `downloads` table (sqlite_orm, additive `sync_schema()`): id, url, destination,
  filename, total/downloaded bytes, status, mime_type, checksum, created/updated_at.
- `SyncDownloadState` reads the existing row first, so it keeps `created_at`,
  `mime_type` and `checksum`, which aren't tracked in memory.
- Paths: the DB path is passed to SQLite as UTF-8. Other paths are native narrow
  strings, consistent with the rest of the codebase.

## 7. IPC / protocol handoff

`main.cpp` declares an optional positional `url` param in `OnInitCmdLine` (required —
see CONVENTIONS.md). It decodes `idr://…`, then `SingleInstanceGuard` either becomes
the server (first instance) or sends the URL over `\\.\pipe\…` and returns `false`
from `OnInit`. Forwarded URLs reach `MainWindow::AddDownloadFromExternal` via `CallAfter`.

## 8. Error reporting

`Download::SetError` keeps the **first** (root-cause) message and logs it.
`Fail()` sets the status to Error and notifies listeners. The GUI shows
`Error: <message>` in the Status column. Failed retries are logged as warnings.
