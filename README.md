<div align="center">

<img src="site/public/logo.png" alt="1IDM" width="96" height="96">

# 1IDM

**1nternet Downloader**

Two independent apps — a Windows desktop manager and an Android manager — plus a browser extension for the desktop app.

[Windows](#windows) · [Android](#android) · [Extension](#browser-extension) · [License](#license)

<br />

![C++](https://img.shields.io/badge/C++17-00599C?logo=cplusplus&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white)
![Android](https://img.shields.io/badge/Android-3DDC84?logo=android&logoColor=white)
![License](https://img.shields.io/badge/License-Non--commercial-orange)

</div>

---

`core/` and `android/` are **separate projects**. They do not share code, a download engine, or a database.

| Project | What it is |
| --- | --- |
| **`core/`** | Native Windows app (wxWidgets, libcurl, libtorrent, SQLite) |
| **`android/`** | Android app (Java UI, its own JNI/libcurl engine) |
| **`browser/`** | MV3 extension that sends links to the **Windows** app |
| **`site/`** | Next.js site for GitHub Pages |

---

## Windows

Segmented HTTP/HTTPS, pause and resume, BitTorrent (magnet / `.torrent`), queue, tray icon, SQLite history at `core/downloads.db`.

```powershell
cd core
./scripts/build.ps1                 # Debug  → core/out/Debug/InternetDownloader.exe
./scripts/build.ps1 -Preset release # Release → core/out/Release/InternetDownloader.exe
```

Needs Visual Studio C++ tools, CMake 3.20+, and Ninja. `build.ps1` finds Visual Studio's vcpkg. First configure after a new dependency can take several minutes; later compiles are seconds.

Register the protocol handler once (HKCU, no admin):

```powershell
./scripts/register_protocol.ps1
```

## Android

Own engines: JNI/libcurl for HTTP(S), libtorrent4j for magnet/`.torrent`. Pause, resume, queue, browser grabber, SQLite history. `arm64-v8a`. No desktop IPC.

```bash
vcpkg install curl:arm64-android --classic --overlay-triplets=android/vcpkg-triplets
cd android
./gradlew assembleRelease
```

Needs JDK 17–21, NDK `27.0.12077973`, and `VCPKG_ROOT`. Recreate `android/local.properties` (`sdk.dir=...`) on a fresh machine.

## Browser extension

Talks to the **Windows** app over `idr://`. Unused by Android.

1. Build and run the desktop app.
2. Register `idr://` with `core/scripts/register_protocol.ps1`.
3. Load `browser/chrome/` as an unpacked extension.
4. Keep automatic intercept off until a manual handoff works.

## Website

```bash
cd site
npm ci
npm run build
```

## License

[Non-commercial](LICENSE) — personal, educational, and research use. Commercial use needs written permission from Nishanth K J.

Security reports: [SECURITY.md](SECURITY.md)
