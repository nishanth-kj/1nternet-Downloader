# Internet Downloader

A desktop-first download manager with a native C++/wxWidgets application, an Android client, a Chrome-compatible browser extension, and a Next.js project website.

> **Project status:** active development. Builds and feature coverage vary by platform. Review the platform limitations and release notes before relying on this application for important downloads.

## Current capabilities

- HTTP/HTTPS downloads through libcurl.
- Pause, resume, cancel, and download queue management in the desktop application.
- Parallel byte-range downloads when the server advertises range support.
- SQLite-backed desktop download metadata.
- BitTorrent integration in the desktop build through libtorrent.
- Android native download engine using JNI and libcurl.
- Browser-to-desktop handoff through the registered `idr://` protocol.
- Static project website deployed with GitHub Pages.

## Platform support and limitations

| Component | Current scope | Important limitation |
| --- | --- | --- |
| Desktop | C++17, wxWidgets, libcurl, SQLite, libtorrent | Validate behavior on the target Windows version before production use. |
| Android | Java/Kotlin application layer with C++/JNI native downloader | Current native build targets `arm64-v8a`; torrent/magnet downloads are not supported in the Android native engine. |
| Browser extension | Manifest V3 extension for Chrome-compatible browsers | Desktop handoff requires the desktop app and the `idr://` protocol registration. Automatic interception is off by default. |
| Website | Next.js static export | The website's Releases link points to GitHub Releases; installable artifacts must actually be published there. |

This repository uses a custom **non-commercial license**. It is not licensed under an OSI-approved open-source license. Read [LICENSE](LICENSE) before redistributing or using the project commercially.

## Repository layout

- `core/` — desktop application, download engine, network layer, persistence, torrent integration, and build scripts.
- `android/` — Android application and native JNI/libcurl downloader.
- `browser/chrome/` — browser extension and desktop protocol handoff.
- `site/` — Next.js website configured for static GitHub Pages export.
- `.github/workflows/` — platform build and website CI workflows.
- `docs/DOWNLOAD_CORRECTNESS_CHECKLIST.md` — manual and release acceptance checklist.
- `SECURITY.md` — vulnerability reporting guidance.

## Desktop build (Windows)

### Prerequisites

- Windows with Visual Studio C++ build tools and Windows SDK.
- CMake 3.20 or newer.
- Git.
- vcpkg with `VCPKG_ROOT` set to the vcpkg checkout path.

### Build

From PowerShell:

```powershell
cd core
./scripts/build.ps1
```

Or configure and build with CMake:

```powershell
cmake -S core -B core/build -DCMAKE_TOOLCHAIN_FILE="$env:VCPKG_ROOT/scripts/buildsystems/vcpkg.cmake"
cmake --build core/build --config Release --parallel
```

The CMake project uses vcpkg packages for libcurl, zlib, SQLite, nlohmann-json, fmt, spdlog, wxWidgets, SqliteOrm, and libtorrent. See `core/vcpkg.json` (if present) and the workflow configuration for the pinned dependency setup.

## Android build

The Android workflow uses JDK 17, Android SDK/NDK, CMake 3.22.1, and vcpkg to build libcurl for the `arm64-v8a` ABI.

```bash
cd android
./gradlew assembleRelease
```

Set `VCPKG_ROOT` and `ANDROID_NDK_HOME` to match the versions configured in `android/app/build.gradle` before building locally. A successful APK build is not a substitute for testing downloads on a real device.

## Browser extension setup

1. Build and install the desktop application.
2. Register the `idr://` protocol using the script under `core/scripts/` where supported.
3. Open the browser's extension management page and enable Developer mode.
4. Load the unpacked extension from `browser/chrome/`.
5. Test a manual handoff before enabling automatic capture.

Automatic interception is disabled by default because the extension cannot reliably confirm that the desktop application accepted a custom-protocol request. Enabling interception may cancel a browser download before handoff succeeds; keep it disabled unless you have verified your local setup.

## Website

The site is in `site/` and uses Next.js static export with the GitHub Pages base path `/1nternet-Downloader`.

```bash
cd site
npm ci
npm run lint
npm run build
```

The website CI workflow runs lint and build checks for pull requests that change site files. Metadata includes canonical URL configuration and generated `robots.txt` and `sitemap.xml` routes.

## Download correctness and release validation

Before distributing a build, use [the download correctness checklist](docs/DOWNLOAD_CORRECTNESS_CHECKLIST.md). At minimum, validate:

- Correct bytes and SHA-256 checksum for a known test file.
- Correct handling of servers that reject or ignore byte ranges.
- Interrupted transfers, resume offsets, redirects, and HTTP error statuses.
- Cancel/remove/shutdown while transfers are active.
- Disk-full and destination-permission failures.
- Browser handoff failure and recovery.
- Android pause/resume and segmented downloads on a physical device.
- Versioned release artifacts, checksums, and release notes.

CI confirms only the checks that it actually runs. Do not infer end-to-end download correctness from a successful compile.

## Security

Please report vulnerabilities privately where possible. See [SECURITY.md](SECURITY.md). Avoid publishing private URLs, tokens, or personal data in issues.

## License

See [LICENSE](LICENSE) for the complete non-commercial terms.
