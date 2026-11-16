<p align="center">
  <img src="site/public/logo.svg" alt="Internet Downloader logo" width="120" height="120">
</p>

# Internet Downloader

A desktop-first download manager with companion Android and browser integrations.

> **Project status:** active development. Feature coverage varies by platform; see
> [Platform support](docs/platforms.md) before relying on it for important downloads.

## Features

- HTTP and HTTPS downloads through libcurl
- Parallel byte-range downloads when the server supports them, with live per-connection progress and speed
- Download queue with a configurable number of simultaneous downloads
- Pause, resume, restart and cancel
- Automatic retries with backoff that resume from the bytes already on disk
- Strict integrity checks: ranged responses are validated before any byte is written, final sizes are verified, and SHA-256 verification is built in
- Settings for the download folder, connections per download, retries and a speed limit
- SQLite download history, JSON settings and a rotating log in `%APPDATA%\InternetDownloader`
- BitTorrent downloads on desktop through libtorrent
- Android downloader powered by a native C++ engine
- Browser extension handoff to the desktop application

## Components

| Directory | Description | Status |
| --- | --- | --- |
| `core/` | Native Windows desktop application (C++17, wxWidgets) | Primary application |
| `android/` | Android application (Java + JNI/C++) | arm64-v8a; no BitTorrent |
| `browser/` | Browser extensions for Chrome, Edge, Brave, Opera, Firefox and Safari | Uses the desktop `idr://` protocol |
| `site/` | Project website (Next.js static export) | GitHub Pages |

See [Platform support](docs/platforms.md) for exact limitations.

## Quick start

### Windows desktop

**Requirements:** Windows 10/11, Visual Studio C++ build tools, CMake 3.21+, Git, and [vcpkg](https://github.com/microsoft/vcpkg).

```powershell
cd core
./scripts/build.ps1                # Debug; add -Preset release for Release
./scripts/register_protocol.ps1    # registers idr:// for the browser extensions
./build/Debug/InternetDownloader.exe
```

`build.ps1` finds vcpkg through `VCPKG_ROOT`, then `C:\vcpkg`, then the copy bundled with
Visual Studio, and loads the MSVC environment for you. See
[Development setup](docs/development.md) for the plain CMake commands and troubleshooting.

### Android

**Requirements:** JDK 17 or 21, Android SDK, Android NDK 27.0.12077973, CMake 3.22.1, and vcpkg.

Set `VCPKG_ROOT` and `ANDROID_NDK_HOME`, build the Android curl dependency for the project's overlay triplet, then run:

```bash
cd android
./gradlew assembleDebug
```

The exact setup is documented in [Development setup](docs/development.md).

### Browser extension

Install the desktop application, register the `idr://` protocol, then load the appropriate unpacked extension from `browser/<browser>/`. Automatic interception is disabled by default.

## Documentation

Project documentation lives in [docs/](docs/README.md):

- [Architecture](docs/architecture.md)
- [Development setup](docs/development.md)
- [Platform support and limitations](docs/platforms.md)
- [Release and distribution](docs/releases.md)
- [Download correctness checklist](docs/DOWNLOAD_CORRECTNESS_CHECKLIST.md)

Contributors and AI coding agents should start with [AGENTS.md](AGENTS.md). The detailed
specification, design, conventions and roadmap are in [agents/](agents/README.md).

## Project structure

```text
.
├── agents/               Specification, design, conventions, workflows, roadmap
├── android/              Android application and JNI downloader
├── browser/              Browser extensions
├── core/                 C++ desktop application
├── docs/                 Project and developer documentation
├── site/                 Static project website
└── .github/workflows/    Build and deployment workflows
```

## Security

Do not include credentials, private download URLs, or personal data in bug reports. See [SECURITY.md](SECURITY.md) for vulnerability reporting.

## License

This project uses a custom non-commercial license. It is not an OSI-approved open-source license. Read [LICENSE](LICENSE) before using, modifying, or redistributing the software.
