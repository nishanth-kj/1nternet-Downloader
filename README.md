<div align="center">

<img src="site/public/logo.png" alt="1IDM" width="96" height="96">

# 1nternet Downloader

A fast download manager for Windows and Android, with a browser extension.

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white)
![Android](https://img.shields.io/badge/Android-3DDC84?logo=android&logoColor=white)
![License](https://img.shields.io/badge/License-Non--commercial-orange)

</div>

## Features

- **Windows app:**
  - Segmented downloads with live speed, time left and a per-connection map.
  - Torrents: magnet, `.torrent` links and files.
  - Classic list with categories, sortable columns, download and completion windows,
    batch add, clipboard monitoring, light/dark themes, and start with Windows.
- **Browser extension:** sends links to the app, and finds videos and audio on pages
  with a download button over the player.
- **Android app:** its own download and torrent engines.

## Quick start (Windows)

```powershell
cd core
cargo run
./scripts/register_protocol.ps1 -ExePath (Resolve-Path target\debug\1idm.exe)   # once, for the extension
```

Then load `browser/chrome/` (or your browser's folder) as an unpacked extension.

Torrent support needs libtorrent from vcpkg. See
[docs/desktop/building.md](docs/desktop/building.md).

## Repository

| Folder | What it is |
| --- | --- |
| `core/` | Windows app (Rust) |
| `android/` | Android app |
| `browser/` | Browser extension for Chrome, Edge, Brave, Opera, Firefox, Safari |
| `site/` | Website (Next.js) |
| `docs/` | [Documentation](docs/README.md) |

## License

[Non-commercial](LICENSE): personal, educational and research use. Commercial use
needs written permission from Nishanth K J.

Security reports: [SECURITY.md](SECURITY.md)
