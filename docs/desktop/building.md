# Building and running the desktop app

## Requirements

- Rust (stable, MSVC toolchain) and Visual Studio with the C++ workload. The C++
  compiler builds the libtorrent bridge and `app.rc` (the exe icon).
- For torrent support: libtorrent from vcpkg in `core/vcpkg_installed/x64-windows`,
  or another vcpkg install tree named by `IDR_VCPKG_INSTALLED`. Without it the app
  still builds and torrent adds report "torrent support isn't included in this build".
  `cargo build` prints a warning when this happens.

To create the vcpkg tree once, run `./scripts/build.ps1 -InstallDeps` in `core/` (it runs
`vcpkg install` with `core/vcpkg.json`, which lists only libtorrent).

## Commands

Run everything from `core/`.

```powershell
cargo run                  # debug build, then start the app
cargo build --release      # target\release\1idm.exe
cargo test                 # unit tests (src/) and the suite in tests/
cargo check                # type-check without linking (works while the app is running)
```

`build.rs` does three things besides compiling Rust:

1. Compiles `c/fast_ops.c`.
2. Compiles the C++ libtorrent bridge in `cpp/` and copies `torrent-rasterbar.dll`,
   `libssl-3-x64.dll` and `libcrypto-3-x64.dll` next to the exe.
3. Embeds `assets/icons/app.ico` through `app.rc`.

The version comes from the root `.version` file.

## Registering `idr://` (once per machine)

```powershell
./scripts/register_protocol.ps1                                   # release build
./scripts/register_protocol.ps1 -ExePath target\debug\1idm.exe   # debug build
```

## Where the app keeps its data

`%APPDATA%\InternetDownloader\`:

| File | Contents |
| --- | --- |
| `settings.json` | Engine settings (download folder, connections, retries, speed limit) |
| `ui.json` | Window preferences (columns, sort, theme, colour, panes, dialogs) |
| `downloads.db` | Download list (SQLite) |
| `torrents\` | `.torrent` files fetched from http(s) links |
| `logs\idr.log` | Log |

Delete the folder to reset the app. Never keep runtime data in the repository.

## Trying changes without touching your own downloads

The app reads `APPDATA`, so a scratch folder keeps tests separate:

```powershell
$env:APPDATA = "$env:TEMP\idm-test"
mkdir "$env:APPDATA\InternetDownloader" -Force
# optional: write settings.json with downloadDir pointing at a scratch folder
cargo run
```

Close any other running copy first. If one is already running, the new launch hands
its URL to that copy, using your real data, and exits.
