# Architecture overview

Internet Downloader is a monorepo containing four products that are built and
released independently:

| Product | Folder | Summary |
| --- | --- | --- |
| Desktop app | `core/` | C++17 + wxWidgets Windows app with a segmented HTTP(S) engine (libcurl) and BitTorrent (libtorrent-rasterbar) |
| Android app | `android/` | Java UI over a native C++ engine (JNI + static libcurl); HTTP only |
| Browser extensions | `browser/` | MV3 extensions that hand links to the desktop app via the `idr://` protocol |
| Website | `site/` | Next.js static site on GitHub Pages |

The desktop app has three layers:

1. **GUI** (`core/src/gui/`) polls the engine every 500 ms. It never blocks on the
   network.
2. **Download service** (`core/src/service/download/`): the `DownloadManager`
   owns the list, the concurrency-limited queue, and SQLite persistence. Each
   `Download` probes the server, then runs parallel ranged connections or a single
   resumable stream, with retries and strict response validation.
3. **Transport and torrent** (`core/src/service/network/`, `core/src/service/torrent/`):
   a libcurl wrapper, plus a pimpl'd libtorrent session.

The full design is in [agents/DESIGN.md](../../agents/DESIGN.md): threading model,
state machine, persistence, and IPC.
Requirements are in [agents/PROJECT_SPEC.md](../../agents/PROJECT_SPEC.md).
