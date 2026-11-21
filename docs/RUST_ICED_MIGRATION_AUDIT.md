# 1IDM — Complete Repository Audit & Migration Specification

**Document:** `docs/RUST_ICED_MIGRATION_AUDIT.md`  
**Date:** 2026-11-21  
**Author:** 1IDM Systems Engineering & Architecture Team  
**Status:** Approved / Baseline for Rust + Iced Migration  

---

## 1. Executive Summary

1nternet Downloader ("1IDM") is a cross-platform download manager repository currently consisting of:
1. **Windows Desktop App (`core/`)**: C++17, wxWidgets 3.2, libcurl, sqlite_orm, libtorrent-rasterbar 2.x, spdlog, nlohmann_json, CMake + vcpkg.
2. **Android Application (`android/`)**: Java UI + native C++ JNI (`native-lib.cpp`) with statically cross-compiled libcurl (API 24, arm64-v8a).
3. **Browser Extensions (`browser/`)**: Manifest V3 extensions across six browser targets (Chrome, Edge, Brave, Opera, Firefox, Safari) dispatching links via the custom `idr://` protocol.
4. **Marketing Site (`site/`)**: Next.js 16 (App Router), React 19, TypeScript, Tailwind CSS 4, exported statically to GitHub Pages.

This audit evaluates the codebase in depth to govern the migration to a **Rust-first architecture** with an **Iced desktop GUI**, retaining C/C++ strictly where technical necessity demands (such as libtorrent FFI).

---

## 2. Component Inventory

| Component Path | Language / Stack | Current Responsibility | Migration Target | Target Status |
| :--- | :--- | :--- | :--- | :--- |
| `core/main.cpp` | C++17, wxWidgets | Application bootstrap, CLI argument parsing (`idr://`), single-instance check | Rust binary (`crates/idm-app`) | Planned |
| `core/src/gui/main_window.*` | C++17, wxWidgets | Main frame, splitters, sidebar, download list, link grabber, status polling | Rust + Iced (`crates/idm-ui`) | Planned |
| `core/src/gui/settings_dialog.*` | C++17, wxWidgets | Modal configuration dialog for speed limits, concurrency, directories | Rust + Iced (`crates/idm-ui`) | Planned |
| `core/src/gui/tray_icon.*` | C++17, wxWidgets | Taskbar tray notification icon, minimize-to-tray, tray context menu | Rust Windows tray (`tray-icon` / `crates/idm-platform`) | Planned |
| `core/src/service/download/download.*` | C++17, std::thread | Single transfer coordinator, static segment division, file assembly, state machine | Rust async engine (`crates/idm-download`) | Planned |
| `core/src/service/download/manager.*` | C++17 | Process-wide singleton, concurrency-capped queue, SQLite synchronization | Rust actor/service (`crates/idm-core`) | Planned |
| `core/src/service/download/progress.*` | C++17 | EWMA speed meter and progress tracking | Rust smooth speed calculator (`crates/idm-download`) | Planned |
| `core/src/service/network/*` | C++17, libcurl | HTTP/HTTPS client, header parsing, cookie jar, proxy, SSL options | Rust async networking (`reqwest` / `hyper` / `rustls`) | Planned |
| `core/src/service/torrent/*` | C++17, libtorrent 2.x | Pimpl wrapping `lt::session` and `lt::torrent_handle`, magnet URI parsing | Thin C++ bridge (`native/cpp`) + Rust FFI (`crates/idm-torrent`) | Planned |
| `core/src/model/sqlite_download_model.*` | C++17, sqlite_orm | SQLite schema, CRUD operations on `downloads` table | Rust SQLite (`rusqlite` / `sqlx`) (`crates/idm-storage`) | Planned |
| `core/src/utils/system/ipc.*` | C++17, Win32 API | Named Mutex (`InternetDownloader_SingleInstance_Mutex`) + Named Pipe (`\\.\pipe\InternetDownloader_IPC`) | Rust Win32/tokio named pipe (`crates/idm-ipc`) | Planned |
| `core/src/utils/system/config.*` | C++17, nlohmann_json | Configuration persistence (`settings.json` in `%APPDATA%\InternetDownloader`) | Rust `serde_json` + atomic writes (`crates/idm-config`) | Planned |
| `core/src/utils/system/logger.*` | C++17, spdlog | Rotating file logger (5 MiB × 3) in AppData | Rust `tracing` + `tracing-appender` | Planned |
| `core/src/utils/checksum.*` | C++17, Windows CNG | SHA-256 calculation via Win32 `bcrypt` | Rust `sha2` crate (portable SIMD acceleration) | Planned |
| `core/scripts/register_protocol.ps1` | PowerShell | Registers HKCU `idr://` protocol handler in Windows registry | Rust native registration / preserve script | Preserved |
| `browser/*` | Vanilla JavaScript (MV3) | Context menus, optional download interception, popup link handoff | Preserve untouched (strict API parity) | Preserved |
| `android/*` | Java + C++ (JNI) | Mobile HTTP client using standalone native curl | Independent product; untouched | Preserved |

---

## 3. Feature Inventory

1. **HTTP/HTTPS Accelerated Download**:
   - HEAD probe capability check (`Accept-Ranges: bytes`, `Content-Length`).
   - Static parallel chunking (1, 4, or up to `maxSegments` connections).
   - Strict byte-range response validation (`HTTP 206` + exact `Content-Range` header).
   - Automatic fallback to single resumable stream if `HTTP 200` is returned to range requests.
   - Individual `.partN` temporary files per connection.
   - Staged assembly into `.assembling` followed by atomic rename and backup rollback (`.previous`).
2. **Resumption & Crash Recovery**:
   - Resumption of individual segments from bytes on disk (`.partN` size).
   - Single-stream resumption via `Range: bytes=N-`.
   - SQLite state synchronization on every state transition.
3. **Transient Error Handling & Retries**:
   - Exponential backoff (1s, 2s, 4s, 8s, 16s) up to `maxRetries`.
   - Differentiation between permanent HTTP errors (4xx except 408/429) and transient errors (network timeouts, 5xx, 429).
4. **Queue Management**:
   - FIFO queue respecting `maxConcurrentDownloads` (default 3, range 1-16).
   - Actions: Pause, Resume, Restart, Remove, Delete File, Pause All, Resume All, Clear Completed.
5. **BitTorrent / Magnet Links**:
   - Protocol detection for `magnet:?` and `.torrent` files.
   - Live peer/seed counts, upload/download speed mirroring from libtorrent session every 500ms.
   - Automatic transition to `Seeding` upon completion.
6. **Browser Integration**:
   - Interception of browser downloads and context menu handoff via `idr://<encoded_url>`.
   - Single-instance named-pipe forwarding.
7. **Built-in Link Grabber**:
   - HTML page fetch and regex parsing for downloadable media and archive extensions.
8. **Integrity Verification**:
   - Background SHA-256 verification of completed downloads with clipboard auto-copy.
9. **Desktop UI Features**:
   - Category filtering (All, Downloading, Completed, Paused, Failed, Torrents).
   - Speed throttling per download (KiB/s).
   - System tray minimization and background operation.

---

## 4. Existing Dependency Inventory

### C++ Desktop (`core/vcpkg.json` + CMake)
- `CURL::libcurl` (HTTP/HTTPS client)
- `ZLIB::ZLIB` (Compression)
- `unofficial::sqlite3::sqlite3` & `sqlite_orm::sqlite_orm` (Database)
- `wxWidgets` (`wx::core`, `wx::base` 3.2+) (Desktop GUI)
- `spdlog::spdlog` (Logging)
- `Boost` (System, Chrono, Asio required by libtorrent)
- `LibtorrentRasterbar::torrent-rasterbar` (BitTorrent 2.x)
- `nlohmann_json::nlohmann_json` (Configuration parsing)
- Win32 libraries: `winhttp`, `ws2_32`, `crypt32`, `bcrypt`, `shell32`, `ole32`

---

## 5. Existing Download Engine Architecture & Flaws

### 5.1 The Current Threading & Worker Model
```
[GUI Thread (wxWidgets)]
       │ (Calls Start(), Pause(), polls atomics every 500ms)
       ▼
[Download Coordinator Thread (1 per download)]
       │
       ├─ Probes HEAD -> decides segment count N
       ├─ Spawns N std::thread workers
       │     ├─ Worker 0: curl_easy_perform() -> file.part0
       │     ├─ Worker 1: curl_easy_perform() -> file.part1
       │     └─ Worker N: curl_easy_perform() -> file.partN
       ├─ Coordinator polls/sleeps until all workers join
       └─ Assembles parts: reads part0..partN into file.assembling -> rename
```

### 5.2 Critical Limitations of the Existing Engine
1. **Static Pre-Segmentation (No Dynamic Allocation)**:
   - The file is divided strictly into `numSegments` fixed slices upfront: `partSize = totalBytes / numSegments`.
   - If Worker 0 is fast (e.g. 10 MB/s) and finishes its 250 MB chunk in 25s, while Worker 1 is throttled by a congested route (e.g. 500 KB/s), Worker 0 **exits and idles**. Total download time is bottlenecked by the slowest segment.
   - **No dynamic work-stealing or chunk-splitting exists**.
2. **Part File Proliferation & Assembly I/O Bottleneck**:
   - Writing separate `.part0`, `.part1` files requires a complete sequential read-write pass of the entire file upon completion (`AssembleParts`). For a 20 GB file, this writes 20 GB of parts, then reads 20 GB and writes 20 GB of final file, thrashing disk I/O and doubling disk space requirements.
3. **Heavy OS Thread Overhead**:
   - Each connection is an OS thread blocked inside `curl_easy_perform`. With 16 downloads of 8 segments each, that amounts to 128 threads plus coordinators.
4. **Coarse Cancellation & Sleep Slices**:
   - Backoff uses 100ms slicing loops checking an atomic boolean `m_stopRequested`.

---

## 6. Existing Networking Behavior

- **Inspection**: Sends HTTP HEAD via `HttpClient::Head`.
- **Validation**: Headers checked for `Accept-Ranges: bytes` and `Content-Length > 0`.
- **Header Parsing**: Re-initialized on every HTTP status line to avoid leaking redirect headers.
- **Range Header**: Formatted as `Range: bytes=start-end`.
- **Response Validation**: Enforced via `ResponseValidator` callback requiring `status == 206` and header prefix matching `"bytes start-end/"`.
- **Single-Stream Resume**: Verified against `Content-Range: bytes existingSize-`.

---

## 7. Existing Database Design

- **Technology**: SQLite 3 with `sqlite_orm` C++ template library.
- **Location**: `%APPDATA%\InternetDownloader\downloads.db`.
- **Schema**:
```sql
CREATE TABLE downloads (
    downloads_id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    destination TEXT NOT NULL,
    filename TEXT NOT NULL,
    total_bytes INTEGER NOT NULL,
    downloaded_bytes INTEGER NOT NULL,
    status INTEGER NOT NULL,  -- 0=Queued, 1=Downloading, 2=Paused, 3=Completed, 4=Error
    mime_type TEXT,
    checksum TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```
- **Concurrency**: Guarded in C++ by `DownloadManager::m_dbMutex`.

---

## 8. Existing IPC & Protocol Integration

- **Protocol**: `idr://<percent-encoded-url>`.
- **Single Instance**: Windows Named Mutex `InternetDownloader_SingleInstance_Mutex`.
- **Named Pipe**: `\\.\pipe\InternetDownloader_IPC`.
- **Message Flow**:
  1. Secondary instance launches with command line `InternetDownloader.exe idr://...`.
  2. Detects existing mutex via `GetLastError() == ERROR_ALREADY_EXISTS`.
  3. Writes decoded URL to `\\.\pipe\InternetDownloader_IPC` and immediately terminates.
  4. Primary instance background thread reads pipe and posts `AddDownloadFromExternal` to main UI.

---

## 9. Existing Torrent Implementation

- **Technology**: `libtorrent-rasterbar 2.x`.
- **Isolation**: Encapsulated in `torrent_manager.cpp` / `torrent_metainfo.cpp` behind a pimpl header (`manager.h`). No libtorrent types leak into GUI or download engine.
- **Lifecycle**: Process-wide `lt::session`. Polled at 500ms intervals for `TorrentSnapshot` (peers, seeds, speeds, progress).

---

## 10. Existing Browser Extension Integration

- Six identical directories: `browser/{chrome,edge,brave,opera,firefox,safari}/`.
- Manifest V3 with `contextMenus` and `downloads` interception.
- Hand-off: Creates a short-lived background tab with `idr://<encoded_url>`. The OS invokes `register_protocol.ps1` handler.

---

## 11. Target Architecture & Migration Matrix

```
┌────────────────────────────────────────────────────────┐
│                   Iced Native GUI                      │
│   (Sidebar, Downloads Table, Detail Pane, Settings)    │
└──────────────────────────┬─────────────────────────────┘
                           │ (Iced Messages / Subscriptions)
                           ▼
┌────────────────────────────────────────────────────────┐
│               Rust Core Orchestrator                   │
│   (State Machine, Queue Manager, IPC, Configuration)   │
└──────────────────────────┬─────────────────────────────┘
                           │
       ┌───────────────────┼────────────────────┐
       ▼                   ▼                    ▼
┌──────────────┐   ┌──────────────┐   ┌───────────────────┐
│ Dynamic HTTP │   │   SQLite     │   │   libtorrent      │
│  Downloader  │   │ Persistence  │   │   C++ Bridge      │
│ (Tokio/Async)│   │  (rusqlite)  │   │  (cxx / C-ABI)    │
└──────────────┘   └──────────────┘   └───────────────────┘
```

### Migration Matrix

| Existing Component | Current Implementation | Target Rust Implementation | Migration Strategy | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Desktop UI** | wxWidgets 3.2 (C++) | Iced (0.13+) | Full rewrite in Rust | Pending |
| **Download Engine** | C++ threads + libcurl | Tokio async + Reqwest/Hyper | Full rewrite in Rust with dynamic work-stealing | Pending |
| **Storage / DB** | sqlite_orm (C++) | `rusqlite` with schema compatibility | Direct compatibility with existing SQLite schema | Pending |
| **Queue Manager** | C++ `DownloadManager` | Rust Tokio actor/channel queue | Full rewrite in Rust | Pending |
| **IPC / Handoff** | Win32 Named Pipe (C++) | Rust `tokio::net::windows::named_pipe` | Drop-in protocol and pipe name compatibility | Planned |
| **Configuration** | `settings.json` (C++) | `serde_json` + atomic file write | 100% schema and path compatibility | Planned |
| **BitTorrent** | libtorrent 2.x (C++) | Thin C++ bridge (`native/cpp`) + Rust FFI | Retain libtorrent via isolated C++/Rust bridge | Planned |
| **Checksum** | Win32 CNG BCrypt (C++) | Pure Rust `sha2` crate (SIMD) | Drop-in replacement | Planned |
| **Browser Extensions**| MV3 JavaScript | Preserved as-is | Untouched | Retained |

---

## 12. Migration Risks & Mitigations

1. **Risk: Database Schema Incompatibility**:
   - *Mitigation*: The Rust database layer will target the exact SQLite table structure (`downloads`), preserving existing `downloads.db` files without migration failure.
2. **Risk: Loss of BitTorrent Functionality**:
   - *Mitigation*: libtorrent will be retained behind an FFI boundary using a thin C/C++ bridge, preserving full compatibility with magnets and `.torrent` files.
3. **Risk: Dynamic Segmentation Data Corruption**:
   - *Mitigation*: Range boundaries will be mathematically proven, validated against exact byte intervals, with pre-allocated sparse files instead of multiple temporary `.partN` files.
4. **Risk: GUI Latency under Heavy I/O**:
   - *Mitigation*: Tokio actor-based async architecture keeps all network and disk I/O completely off the Iced UI thread, communicating strictly via async channels and throttled state snapshots.

---

## 13. Verification & Rollback Strategy

- The existing C++ code in `core/` will remain intact and buildable side-by-side until the Rust + Iced implementation achieves complete parity.
- The Cargo workspace will live inside `core/` alongside CMake, allowing dual verification.
- Release candidates will verify the `e2e` suite, SHA-256 integrity checks, and browser handoff before the old C++ engine is retired.
