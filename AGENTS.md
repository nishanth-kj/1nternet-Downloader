# AGENTS.md

Instructions for AI coding agents in this repository. This is the only agent file
(there is no `CLAUDE.md`). Full documentation is in [`docs/`](docs/README.md).

## What this is

1nternet Downloader ("1IDM"), a download manager. Four products, built separately:

| Folder | Product | Stack |
| --- | --- | --- |
| `core/` | Windows desktop app | Rust, Iced, Tokio, reqwest, SQLite; libtorrent through a C++ bridge |
| `android/` | Android app | Java UI, C++ engine over JNI (libcurl), libtorrent4j |
| `browser/` | Browser extension (six copies) | Manifest V3, plain JS, no build step |
| `site/` | Website | Next.js, React, Tailwind |

Each folder has its own `AGENTS.md` with its rules.

## Read first

- [docs/overview.md](docs/overview.md): how the products connect
- [docs/contributing/conventions.md](docs/contributing/conventions.md): rules that must not be broken
- [docs/contributing/testing.md](docs/contributing/testing.md): how to verify a change
- [docs/roadmap.md](docs/roadmap.md): known bugs and planned work

## Commands

```powershell
cd core; cargo build; cargo test; cargo run         # desktop
```

```bash
cd android && ./gradlew assembleDebug               # NDK 27.0.12077973, JDK 17-21
cd site && npm ci && npm run lint && npm run build  # website
node --check browser/<name>/background.js           # extension (also popup.js, content.js)
```

## Most important rules

1. Never write download bytes before validating the response (`206` and the exact
   `Content-Range` for ranges). Never mark Completed without checking the size on disk.
2. libtorrent headers only in `core/cpp/torrent_manager.cpp` and `torrent_metainfo.cpp`.
   Rust calls the C API in `cpp/torrent_c_api.h`. No hand-written BitTorrent code.
3. Runtime data lives in `%APPDATA%\InternetDownloader\`, never in the working directory
   or the repository.
4. The UI thread never blocks; engine work runs on the Tokio runtime.
5. Edit `browser/chrome/` first, then sync the other five copies; CI fails on drift.
6. Android's vcpkg triplet stays at API 24.

## Verify, then report

A build that compiles proves nothing about downloads. After changing `core/src/service/`
or `core/cpp/`, run the manual check in [testing.md](docs/contributing/testing.md). Test
with a scratch `APPDATA` so you don't touch the owner's downloads, and close the owner's
running copy first only if they agree (a second launch hands its URL to the first).

In your summary, say what you verified and what you didn't.

## Ask before changing

`core/tests/*`, `plugins/`, `LICENSE` (non-commercial), and the generated block in
`site/AGENTS.md`.
