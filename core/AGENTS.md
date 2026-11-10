# core/ — Windows desktop app

Design: [../agents/DESIGN.md](../agents/DESIGN.md) · Invariants: [../agents/CONVENTIONS.md](../agents/CONVENTIONS.md)

## Build and run

```powershell
./scripts/build.ps1                  # Debug → build/Debug/InternetDownloader.exe
./scripts/build.ps1 -Clean           # after a vcpkg or toolchain change
./scripts/register_protocol.ps1      # once per machine: register idr://
```

Sources are globbed (`file(GLOB_RECURSE src/*.cpp)`), so re-run the configure step
after you add or remove files. The project compiles at `/W4`; keep its own code
warning-free.

## Where things live

- `main.cpp`: wx entry point, `idr://` parsing, single-instance handoff.
- `src/gui/`: `MainWindow` (polls the manager every 500 ms and calls
  `ProcessQueue()`), `SettingsDialog`, tray icon.
- `src/service/download/`: `Download` (probe → segments or single stream, retries,
  assembly, torrent polling), `DownloadManager` (list, queue, SQLite sync),
  `SpeedMeter`.
- `src/service/network/`: `HttpClient::Head/Get(req, onData, validator)` over
  `CurlHandle`.
- `src/service/torrent/`: libtorrent wrapper (pimpl). This is the **only** place
  libtorrent headers may appear.
- `src/model/sqlite_download_model.h`: the real DB layer. `src/utils/database/` is
  an unused placeholder.
- `src/utils/system/`: `Config`/`Settings`, `AppDataDir()`, `Log()`, IPC.
  `src/utils/checksum.*` is SHA-256 via bcrypt.

## Rules specific to this folder

- Read the "Hard invariants" in `agents/CONVENTIONS.md` before you edit
  `download.cpp`, `manager.cpp` or `main.cpp`.
- Use `(std::max)`/`(std::min)` wherever `<windows.h>` may be included.
- Never call `curl_easy_*` outside `src/service/network/`.
- After engine changes, run `tests/e2e/README.md` and compare SHA-256s; a compile
  alone doesn't count as verification.
- Runtime files: `%APPDATA%\InternetDownloader\{downloads.db,settings.json,logs\idr.log}`.
