# AGENTS.md

Instructions for AI coding agents working in this repository. This is the canonical
cross-tool file, and the only one: there is no `CLAUDE.md`. Detailed docs live in [`agents/`](agents/README.md).

## What this is

Internet Downloader ("IDR") is a download manager. The monorepo contains four
independently built products:

| Folder | Product | Stack |
| --- | --- | --- |
| `core/` | **Flagship** Windows desktop app: segmented HTTP(S) and BitTorrent | C++17, wxWidgets, libcurl, libtorrent-rasterbar 2.x, sqlite_orm, spdlog, nlohmann_json, vcpkg, CMake |
| `android/` | Android app with its own native engine | Java UI + C++ JNI, static libcurl, Gradle 8.9, NDK 27 |
| `browser/` | MV3 extensions (Chrome, Edge, Brave, Opera, Firefox, Safari) handing links to the app via `idr://` | Vanilla JS, no build step |
| `site/` | Marketing/docs site on GitHub Pages | Next.js 16, React 19, Tailwind 4 |

Each component folder has its own `AGENTS.md` with component rules.

## Read before changing code

- [agents/PROJECT_SPEC.md](agents/PROJECT_SPEC.md): requirements and scope.
- [agents/DESIGN.md](agents/DESIGN.md): architecture, threading, state machine, persistence.
- [agents/CONVENTIONS.md](agents/CONVENTIONS.md): **hard invariants** and style.
- [agents/WORKFLOWS.md](agents/WORKFLOWS.md): build, verify and release commands.
- [agents/ROADMAP.md](agents/ROADMAP.md): known gaps.

## Commands

```powershell
cd core; ./scripts/build.ps1          # Windows desktop (finds vcpkg via VCPKG_ROOT, C:\vcpkg, or VS-bundled)
```

```bash
cd android && ./gradlew assembleDebug  # needs NDK 27.0.12077973, JDK <= 21, local.properties
cd site && npm ci && npm run lint && npm run build
node --check browser/<browser>/background.js   # extensions have no build step
```

## Verification is mandatory

There is no automated test suite, and a successful compile proves nothing about
downloads. After touching `core/src/service/`, build and run the end-to-end check
in [`core/tests/e2e/README.md`](core/tests/e2e/README.md), then compare SHA-256s. In
your summary, say what you verified and what you didn't.

## Top invariants (full list in agents/CONVENTIONS.md)

1. libtorrent headers only in `core/src/service/torrent/torrent_{manager,metainfo}.cpp`
   (the pimpl in `manager.h`). No hand-rolled BitTorrent or bencode code.
2. All HTTP goes through `idr::network::HttpClient`. Never write a response byte
   before it is validated (206 + exact `Content-Range` for ranges).
3. `MyApp::OnInitCmdLine` in `core/main.cpp` must keep the optional `url` param, or
   the `idr://` handoff silently breaks.
4. Engine threads are owned and joined, never detached. Status callbacks must not
   lock `DownloadManager::m_mutex`.
5. Runtime data lives in `%APPDATA%\InternetDownloader\`, never in the CWD.
6. The six `browser/*` copies must stay in sync (CI enforces this). Edit `chrome/` first.
7. Android's vcpkg triplet stays at API 24. Android has no torrent support by design.

## Don't touch without asking

`core/tests/*` and `plugins/` (reserved),
`core/src/utils/database/database.h` (intentional placeholder), and `LICENSE` (non-commercial).

## Local run notes

- Use PowerShell for `core/scripts/*.ps1`. In Git Bash, use `cygpath -w` when you hand
  paths to Windows programs.
- Before running the desktop app for verification, point `downloadDir` in
  `%APPDATA%\InternetDownloader\settings.json` at a scratch folder. Write that JSON
  with Node rather than hand-escaping backslashes. Afterwards kill the app
  (`taskkill //IM InternetDownloader.exe //F`) and delete the test data.
- The repository root must stay free of runtime data such as `downloads.db`.
