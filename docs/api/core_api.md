# Core API (desktop engine)

These are the public C++ entry points in `core/src`. Headers are the source of truth.

## `idr::download::DownloadManager` (`service/download/manager.h`)

Process-wide singleton: `DownloadManager::GetInstance()`.

| Method | Behaviour |
| --- | --- |
| `AddDownload(url, destination, autoStart = true)` | Persists the download. If `autoStart`, it is queued and started when a slot is free; otherwise it is added as Paused |
| `PauseDownload(id)` / `ResumeDownload(id)` / `RestartDownload(id)` | Pause; re-queue; delete partial data and re-queue |
| `RemoveDownload(id, deleteFile)` | Stops and joins the worker, releases torrent resources, deletes temp files, and deletes the destination too if `deleteFile` |
| `PauseAll()` / `ResumeAll()` / `ClearCompleted()` | Bulk operations |
| `ProcessQueue()` | Starts Queued downloads up to `maxConcurrentDownloads` (the GUI calls it every 500 ms) |
| `GetDownloads()` / `FindDownload(id)` | Snapshot of shared handles |

## `idr::download::Download` (`service/download/download.h`)

The GUI reads these thread-safe getters: `GetStatus()`, `GetTotalBytes()`,
`GetDownloadedBytes()`, `GetSpeedBytesPerSec()`, `GetSegmentCount()`,
`GetSegments()`, `GetErrorMessage()`, `GetPeers()`/`GetSeeds()` (torrents) and
`IsTorrent()`. Lifecycle calls should normally go through the manager, so that
the queue limit applies.

## `idr::network::HttpClient` (`service/network/http.h`)

- `Head(req)`: status, content length, `acceptRangesBytes`, final URL, and headers
  (final response only).
- `Get(req, onData, validator = {})` streams the body to `onData`. `validator(status,
  headers)` runs once before the first body byte, and returning `false` aborts
  without writing anything.
- `HttpRequest` fields: URL, range, `maxRecvBytesPerSec`, `connectTimeoutSeconds`,
  `stallTimeoutSeconds`, headers, proxy, TLS, cookie jar.

## `idr::system` (`utils/system/`)

- `Config::Get().GetSettings()` / `SetSettings(s)` read and write `settings.json`
  in `AppDataDir()`.
- `Log()` returns the spdlog logger for `%APPDATA%\InternetDownloader\logs\idr.log`.
- `SingleInstanceGuard` (`ipc.h`) provides the single-instance mutex and named-pipe
  URL forwarding.

## `idr::utils`

- `Sha256File(path, &error, cancelFlag)` returns 64 lowercase hex characters, or
  an empty string on failure.
- `UrlDecode(s)` decodes percent-encoding.

## `idr::torrent::TorrentManager` (`service/torrent/manager.h`)

`AddTorrent(magnetOrPath, saveDirectory, autoStart)` returns a `TorrentTask`.
`TorrentTask::GetSnapshot()` returns status, bytes, speed, peers and seeds from a
single `torrent_handle::status()` call.
