# 1IDM Rust + Iced Target Architecture & Migration Specification

**Document:** `docs/RUST_TARGET_ARCHITECTURE.md`  
**Date:** 2026-11-21  
**Status:** Architecture Design Record  

---

## 1. System Architecture Overview

The target 1IDM desktop application is a high-performance, native Windows download manager written in **Rust**, with a reactive desktop user interface built with **Iced**.

```
┌────────────────────────────────────────────────────────┐
│                      Iced UI                           │
│  (Sidebar, Downloads Table, Detail Pane, Settings)     │
└───────────────────────────┬────────────────────────────┘
                            │ Messages / Subscriptions
                            ▼
┌────────────────────────────────────────────────────────┐
│                 Rust Application Core                  │
│       (Application Lifecycle, IPC, Coordination)       │
└───────────────────────────┬────────────────────────────┘
                            │
       ┌────────────────────┼────────────────────┐
       ▼                    ▼                    ▼
┌──────────────┐    ┌──────────────┐    ┌─────────────────┐
│ Dynamic HTTP │    │    SQLite    │    │   libtorrent    │
│  Engine      │    │ Persistence  │    │   C++ Bridge    │
│(Tokio/Reqwest│    │  (rusqlite)  │    │     (cxx)       │
└──────────────┘    └──────────────┘    └─────────────────┘
```

---

## 2. Cargo Workspace & Crate Boundaries

Inside `core/`, a modular Cargo workspace is established:

```
core/
├── Cargo.toml
├── crates/
│   ├── idm-common/     # Domain models, DownloadStatus, Segment, Settings, errors
│   ├── idm-storage/    # SQLite persistence compatible with existing downloads.db
│   ├── idm-network/    # RFC 9110 HTTP/HTTPS client, HEAD probe, connection reuse
│   ├── idm-download/   # Dynamic work-stealing download engine, sparse file I/O
│   ├── idm-queue/      # Concurrency-limited FIFO queue manager
│   ├── idm-ipc/        # Win32 Named Mutex and Named Pipe (idr:// handoff)
│   ├── idm-torrent/    # Thin Rust/C++ bridge to libtorrent-rasterbar 2.x
│   ├── idm-ui/         # Native Iced GUI (state, messages, theme, widgets)
│   └── idm-app/        # Main executable binary and lifecycle bootstrap
```

### Dependency Invariants
1. `idm-common` depends on no other internal crates.
2. `idm-download` depends on `idm-common`, `idm-network`, and `idm-storage`.
3. `idm-ui` depends on `idm-common` and communicates with the engine via async message channels; it never touches raw network or file sockets directly.
4. `idm-app` coordinates all crates, launches the Tokio runtime, starts the IPC server, and runs the Iced application loop.

---

## 3. Data Flow & Concurrency Model

### 3.1 Non-Blocking Reactive UI
The Iced UI operates strictly as an event-driven viewer:
- **UI → Engine**: User actions (Add Download, Pause, Resume, Delete, Set Settings) emit Iced `Message`s converted to async commands sent across a bounded Tokio `mpsc::Sender<EngineCommand>`.
- **Engine → UI**: An Iced `Subscription` listens on a broadcast or watch channel (`watch::Receiver<DownloadSnapshot>`) emitting updates at throttled intervals (e.g. 100ms – 250ms), ensuring the UI renders at a silky 60 FPS without ever stalling on disk or network I/O.

### 3.2 Dynamic Download Engine
- **Probing**: Probes URL with HTTP HEAD/GET (`Range: bytes=0-0`). Detects `Content-Length`, `Accept-Ranges`, `ETag`, and `Content-Disposition`.
- **Sparse File Pre-allocation**: Directly creates destination `.tmp` file and sets file length, eliminating temporary `.partN` concatenation passes.
- **Dynamic Work Stealing**: Active workers write directly to file offsets using random-access seek/write. When an active worker finishes, it dynamically steals the upper half of the largest uncompleted range from the slowest worker, provided remaining bytes $> 2 \times \text{MIN\_SPLIT\_SIZE}$.
