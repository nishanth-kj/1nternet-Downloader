# Project specification — 1nternet Downloader ("1IDM")

## 1. Purpose

1IDM is a download manager. It speeds up HTTP(S) downloads by splitting a file
into byte ranges and downloading them in parallel. It survives interruptions,
reports failures honestly, and supports BitTorrent. The browser extensions
hand downloads off to the desktop app.

The governing principle is **correctness over speed**: a download is never marked
complete unless the bytes on disk are exactly what the server sent.

## 2. Products

| Product | Folder | Platform | Status |
| --- | --- | --- | --- |
| Desktop app (flagship) | `core/` | Windows 10/11 x64, C++17 + wxWidgets | Working; HTTP and torrent |
| Android app | `android/` | Android 7.0+ (API 24), arm64-v8a only | Working for HTTP; no torrent |
| Browser extensions | `browser/` | Chrome, Edge, Brave, Opera, Firefox, Safari (MV3) | Working; hands off via `idr://` |
| Website | `site/` | Next.js static export on GitHub Pages | Working |

## 3. Desktop functional requirements

### 3.1 Adding downloads
- **R-ADD-1**: Add an HTTP/HTTPS/FTP URL or a magnet link from a dialog. The
  dialog is pre-filled from the clipboard when the clipboard holds a supported link.
- **R-ADD-2**: Add a local `.torrent` file. Torrent payloads are saved into a
  chosen *folder* (libtorrent lays out files from the metadata).
- **R-ADD-3**: Accept `idr://<percent-encoded-url>` on the command line. If an
  instance is already running, forward the URL to it over a named pipe and exit.
- **R-ADD-4**: External adds (browser/`idr://`) never prompt. They save into the
  configured download folder with a unique filename (`name_1.ext`, …). A name is
  never reused while it exists on disk or is the destination of another listed download.
- **R-ADD-5**: Unsupported schemes are rejected with a message.

### 3.2 Transfer engine
- **R-ENG-1**: HEAD-probe first. If the server advertises `Accept-Ranges: bytes`
  and a length ≥ 2 MiB, download in up to 4 (< 50 MiB) or `maxSegments` parallel
  ranged connections. Otherwise use one resumable stream.
- **R-ENG-2**: Write each ranged response only if it is `206` with exactly the
  requested `Content-Range`. Check this before writing a single byte.
- **R-ENG-3**: If a server advertises ranges but answers ranged GETs with `200`,
  fall back to a single stream automatically.
- **R-ENG-4**: Retry transient failures (network errors, stalls > 60 s, 408/429/5xx)
  up to `maxRetries` times per segment or stream, with exponential backoff (1, 2,
  4, 8, 16 s). Each retry resumes from the bytes already on disk. 4xx errors are permanent.
- **R-ENG-5**: Assemble segments into `<dest>.assembling`. Verify the total size,
  then atomically replace the destination, keeping a backup until the swap succeeds.
- **R-ENG-6**: A single stream resumes only when the server returns `206` at
  exactly the existing offset. Otherwise it restarts from zero rather than appending wrong bytes.
- **R-ENG-7**: Report a disk-full or permission failure as an error with a message,
  never as a completed download.
- **R-ENG-8**: An optional per-download speed limit (KiB/s) is split evenly across
  the download's connections.

### 3.3 Queue and lifecycle
- **R-Q-1**: At most `maxConcurrentDownloads` downloads are active at once. New and
  resumed downloads wait as **Queued** and start in list order when a slot frees.
- **R-Q-2**: Pause, Resume, Restart, Remove, Delete file, Pause All, Resume All and
  Clear Completed are all available. Restart deletes partial data. Remove and Clear
  release libtorrent resources and delete temporary files.
- **R-Q-3**: Downloads that were active at shutdown come back as **Paused**.
- **R-Q-4**: Removing a download, restarting it, or shutting down the app never
  leaves a worker thread writing into a destroyed object or a deleted file.

### 3.4 Persistence and settings
- **R-P-1**: The download list lives in SQLite at `%APPDATA%\InternetDownloader\downloads.db`.
  It is synced on every status transition.
- **R-P-2**: Settings live in `%APPDATA%\InternetDownloader\settings.json`, written
  atomically: download folder, simultaneous downloads (1–16), connections per
  download (1–32), retries (0–20), and speed limit.
- **R-P-3**: Logs go to `%APPDATA%\InternetDownloader\logs\idr.log` (rotating, 5 MiB × 3).

### 3.5 GUI
- **R-UI-1**: The list shows ID, name, size, progress %, live speed, threads or peers,
  status (with the error reason), and URL. It refreshes every 500 ms.
- **R-UI-2**: The sidebar filters by state and by file type.
- **R-UI-3**: A segment view shows each connection's byte range and state.
- **R-UI-4**: **Verify SHA-256** hashes a completed file in the background, copies
  the digest to the clipboard, and compares it with a pasted expected value.
- **R-UI-5**: A built-in link grabber fetches a page and lists downloadable links.
- **R-UI-6**: Closing the window hides it to the system tray. Exiting from the tray
  or menu quits the app.

### 3.6 BitTorrent (desktop only)
- **R-BT-1**: Magnets and `.torrent` files go through libtorrent-rasterbar 2.x via
  `TorrentManager`, which owns one `lt::session`. No hand-rolled protocol code.
- **R-BT-2**: Progress, speed, peers and seeds are mirrored from libtorrent snapshots
  every 500 ms. Once complete, the torrent is shown as **Seeding** until it is removed.

## 4. Browser extension requirements
- **R-EXT-1**: The context menu and the popup send links to the app as
  `idr://<encodeURIComponent(url)>` through a short-lived background tab.
- **R-EXT-2**: Automatic download capture is **off by default**. A protocol
  handoff can't confirm that the app received the URL. When capture is on, the
  browser download is cancelled only after the handoff tab was created.
- **R-EXT-3**: Only `http:`, `https:` and `magnet:` URLs are forwarded.
- **R-EXT-4**: All six browser copies carry the same logic. CI fails if they drift.

## 5. Android requirements
- **R-AND-1**: A native C++ engine (JNI + static libcurl) performs a HEAD probe,
  ranged GET workers and reassembly, with the same range validation rules as desktop.
- **R-AND-2**: The app handles `magnet:` and `idr:` intents. Torrent and magnet
  downloads report **Error** on purpose (not implemented).

## 6. Non-functional requirements
- **Integrity**: Every completion path checks the final file size. The SHA-256
  check is available to users.
- **Thread safety**: The GUI reads atomics only. Every database access is serialized.
  Workers are owned and joined, never detached.
- **Privacy**: No telemetry. Logs contain URLs and paths and stay local.
- **Portability of builds**: `core/scripts/build.ps1` finds vcpkg via `VCPKG_ROOT`,
  then `C:\vcpkg`, then the copy bundled with Visual Studio.

## 7. Out of scope (for now)
- Hand-written BitTorrent, DHT, tracker or bencode code (see CONVENTIONS.md).
- Desktop builds for macOS or Linux. Android ABIs other than arm64-v8a. Torrent on Android.
- Native messaging hosts. The extensions use only the `idr://` protocol.
- Video-site stream extraction. Scheduled (time-of-day) downloads.
