# Conventions

## Rules that must not be broken

Each one comes from a real bug or a deliberate decision.

### Downloads

1. **Never write bytes that weren't validated.** A ranged request must get `206` with
   the exact `Content-Range` asked for, before anything is written.
2. **Never mark a download Completed without checking the size on disk** against the
   expected total.
3. **All HTTP goes through `NetworkClient`** (`core/src/service/impl/network.rs`).
4. **`DownloadStatus` values are stored as integers.** Add new values at the end; never
   renumber them.

### Torrents

5. **libtorrent headers only in `core/cpp/torrent_manager.cpp` and
   `torrent_metainfo.cpp`.** Rust reaches libtorrent only through
   `cpp/torrent_c_api.h`, from `service/impl/torrent.rs`.
6. **No hand-written BitTorrent or bencode code.**
7. **The C API catches every C++ exception.** None may cross into Rust.

### App

8. **Runtime data lives in `%APPDATA%\InternetDownloader\`** (`Config::app_data_dir()`),
   never in the working directory. `idr://` launches inherit the browser's directory.
9. **Keep the `idr://` path working:** `main.rs` reads `args[1]` through
   `decode_idr_url`, and a second instance forwards it over the named pipe.
10. **The UI thread never blocks** on network or disk. Use `service::spawn`.

### Other products

11. **The six browser copies stay in sync.** Edit `browser/chrome/` first; CI checks
    the rest.
12. **Android's vcpkg triplet stays at API 24.**

## Ask the owner before changing

- `core/tests/` and `plugins/` (reserved). The suite in `core/tests/` builds
  `DownloadSnapshot` directly, so changing that struct's fields needs the owner's OK.
- `LICENSE` (non-commercial).
- `site/AGENTS.md`: `next dev` regenerates its marker block.

## Rust (`core/`)

- Edition 2021. Keep the build warning-free (`cargo build` shows warnings).
- Errors are `IdmException` (`core/src/exception/`); return `Result`, don't panic in
  engine code.
- Traits live in `service/*.rs`, implementations in `service/impl/`.
- SQL lives next to its table struct in `model/database/`.
- `unsafe` is allowed only for FFI, with a `// SAFETY:` comment.
- Log through `tracing`: `info` for lifecycle, `warn` for retries, `error` for failures.

## C and C++ (`core/c`, `core/cpp`, Android native)

- C++17. Bridge code exposes plain C functions and `#[repr(C)]`-compatible structs only.
- In files that may include `<windows.h>`, write `(std::max)(a, b)` and `(std::min)`.

## JavaScript (`browser/`)

- Plain JS, MV3, no build step. Use the `chrome.*` callback APIs and check
  `chrome.runtime.lastError`.

## Documentation

- Write for someone new to the code: short sentences, plain words, no marketing.
- Update the matching document in the same change as the code.

## Commits

Conventional prefixes (`feat:`, `fix:`, `docs:`, `refactor:`, `ci:`). Branch rules are
in [workflow.md](workflow.md).
