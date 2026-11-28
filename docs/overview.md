# Overview

1nternet Downloader ("1IDM") is a download manager. The repository holds four products
that are built separately and share no code.

| Folder | Product | Stack |
| --- | --- | --- |
| `core/` | Windows desktop app | Rust, Iced (UI), Tokio, reqwest, SQLite (rusqlite), libtorrent through a C++ bridge |
| `android/` | Android app | Java UI, C++ HTTP engine over JNI (libcurl), libtorrent4j |
| `browser/` | Browser extension (Chrome, Edge, Brave, Opera, Firefox, Safari) | Manifest V3, plain JavaScript, no build step |
| `site/` | Website on GitHub Pages | Next.js, React, Tailwind |

## How they connect

```text
Browser extension ──idr://<url>──► Windows ──► 1idm.exe (new or already running)
                                              │
                    second instance ──named pipe──► running instance adds the download
```

- The extension never downloads anything itself. It opens `idr://<encoded url>`, and
  Windows starts `1idm.exe` with that argument (registered by
  `core/scripts/register_protocol.ps1`).
- Only one desktop instance runs. A second launch forwards its URL to the first over
  a named pipe and exits.
- The Android app is standalone. It doesn't talk to the desktop app or the extension.

## Platform support

| Product | Target | Notes |
| --- | --- | --- |
| Desktop | Windows 10/11 x64 | Torrents need libtorrent from vcpkg at build time |
| Android | arm64-v8a, Android 7.0+ (API 24) | Other ABIs aren't built |
| Extension | Chromium browsers, Firefox 115+, Safari (packaged with Xcode) | Needs the desktop app and the `idr://` handler |
| Website | Static export | No backend; it links to release files |

## Licence

Non-commercial (see `LICENSE`). Other projects, such as qBittorrent, may be read for
reference, but their code isn't copied into this repository.
